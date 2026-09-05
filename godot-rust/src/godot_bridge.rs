use godot::prelude::*;

use crate::battle::{
    Actor, Battle, BattleEvent, BattlePhase, BattleSnapshot, BattlefieldEffect, Faction,
    RecoveryOpening, StatusInstance, StatusKind,
};
use crate::expedition::{EncounterOutcome, ExpeditionError, ExpeditionState, TimeSegment};
use crate::geography::{CellDefinition, EncounterTriggerDefinition, Geography, PortalDefinition};
use crate::habitat::Habitats;
use crate::protocol::{CommandEnvelope, CommandKind, PROTOCOL_VERSION};
use serde::Deserialize;

const BATTLE_ID: &str = "battle.prototype.returning_names";

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Project42SimulationBridge {
    #[init(val = None)]
    battle: Option<Battle>,
    #[init(val = 0)]
    sequence: u64,
}

/// Engine-facing route adapter. Godot supplies the already-validated portal
/// records from its generated content bundle; Rust then owns legality, state
/// mutation and save-compatible arrival history.
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Project42ExpeditionBridge {
    #[init(val = None)]
    state: Option<ExpeditionState>,
    #[init(val = Geography::default())]
    geography: Geography,
    /// Habitat ecology is still a Rust fixture, not authored content -- D1
    /// built it that way on purpose. It rides along until content owns it.
    #[init(val = Habitats::black_beach_vertical_slice())]
    habitats: Habitats,
    #[init(val = None)]
    battle: Option<Battle>,
    #[init(val = 0)]
    battle_sequence: u64,
}

/// A deliberately narrow, string-based GDExtension boundary. Godot prepares
/// this payload from the validated content catalog; Rust parses and validates
/// the values before mutating expedition state. This avoids treating Godot's
/// typed Array variants as a backend protocol.
#[derive(Deserialize)]
struct ExpeditionConfiguration {
    seed: u64,
    party_ids: Vec<String>,
    active_location_id: String,
    /// Authored cells with their region, observations and anchors. Optional so
    /// a payload that only names portals still loads; endpoints a portal
    /// touches are implied.
    #[serde(default)]
    cells: Vec<CellDefinition>,
    portals: Vec<PortalDefinition>,
    encounter_triggers: Vec<EncounterTriggerDefinition>,
}

#[godot_api]
impl Project42SimulationBridge {
    #[func]
    fn create_debug_battle(&mut self) -> VarDictionary {
        self.sequence = 0;
        self.battle = Some(Battle::prototype_vertical_slice());
        snapshot_dictionary(&self.battle.as_ref().expect("battle assigned").snapshot())
    }

    #[func]
    fn start_battle(&mut self) -> Array<VarDictionary> {
        let Some(battle) = self.battle.as_mut() else {
            return self.rejection("", "battle_not_created");
        };
        let events = battle.start();
        self.records(events)
    }

    #[func]
    fn submit_command(&mut self, command: VarDictionary) -> Array<VarDictionary> {
        let command_id = field_string(&command, "command_id").unwrap_or_default();
        let envelope = match command_envelope(&command) {
            Ok(value) => value,
            Err(reason) => return self.rejection(&command_id, reason),
        };
        if envelope.battle_id != BATTLE_ID {
            return self.rejection(&command_id, "battle_id_mismatch");
        }
        let skill_command = match envelope.into_skill_command() {
            Ok(value) => value,
            Err(crate::protocol::ProtocolError::UnsupportedVersion { .. }) => {
                return self.rejection(&command_id, "unsupported_protocol_version");
            }
            Err(_) => return self.rejection(&command_id, "invalid_command_envelope"),
        };
        let Some(battle) = self.battle.as_mut() else {
            return self.rejection(&command_id, "battle_not_created");
        };
        match battle.submit(skill_command) {
            Ok(events) => self.records(events),
            Err(error) => self.rejection(&command_id, battle_error_code(&error)),
        }
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.battle.as_ref().map(|battle| snapshot_dictionary(&battle.snapshot())).unwrap_or_else(|| {
            let actors = VarArray::new();
            let error = vdict! { "kind" => "battle_not_created" };
            vdict! { "battle_id" => "", "phase" => "error", "actors" => &actors, "error" => &error }
        })
    }

    #[func]
    fn recommended_enemy_command(&self, command_id: GString) -> VarDictionary {
        let Some(battle) = self.battle.as_ref() else {
            return vdict! { "available" => false, "reason" => "battle_not_created" };
        };
        let Some(decision) = battle.enemy_decision() else {
            return vdict! { "available" => false, "reason" => "active_actor_is_not_hostile" };
        };
        let target_ids: Array<GString> = array![decision.target_id.0.as_str()];
        vdict! {
            "available" => true, "protocol_version" => i64::from(PROTOCOL_VERSION),
            "command_id" => &command_id, "battle_id" => BATTLE_ID,
            "actor_id" => decision.actor_id.0.as_str(), "kind" => "use_skill",
            "skill_id" => decision.skill_id.as_str(), "target_ids" => &target_ids,
            "rationale" => decision.rationale.as_str(), "raw_damage" => i64::from(decision.raw_damage),
            "guard_break_amount" => i64::from(decision.guard_break_amount),
            "guard_absorbed" => i64::from(decision.guard_absorbed),
            "vitality_damage" => i64::from(decision.vitality_damage), "lethal" => decision.lethal,
            "interception_protector_id" => decision.interception_protector_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
            "fatal_intercept_available" => decision.fatal_intercept_available,
        }
    }
}

#[godot_api]
impl Project42ExpeditionBridge {
    #[func]
    fn configure(&mut self, configuration_json: GString) -> VarDictionary {
        let configuration = match serde_json::from_str::<ExpeditionConfiguration>(
            &configuration_json.to_string(),
        ) {
            Ok(value) => value,
            Err(_) => return expedition_error_dictionary("expedition_configuration_invalid"),
        };
        self.geography = match Geography::from_authored(
            configuration.cells,
            configuration.portals,
            configuration.encounter_triggers,
        ) {
            Ok(geography) => geography,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        self.state = match ExpeditionState::new(
            configuration.seed,
            configuration.party_ids,
            configuration.active_location_id,
        ) {
            Ok(state) => Some(state),
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        expedition_state_dictionary(
            self.state.as_ref().expect("state assigned"),
            &self.geography,
        )
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.state
            .as_ref()
            .map(|state| expedition_state_dictionary(state, &self.geography))
            .unwrap_or_else(|| expedition_error_dictionary("expedition_not_configured"))
    }

    #[func]
    fn travel(&mut self, portal_id: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        if let Err(error) = state.travel(&portal_id.to_string(), &self.geography) {
            return expedition_error_dictionary(expedition_error_code(&error));
        }
        // Arrival presents whatever the world actually holds here: an authored
        // trigger, a hunter that caught up, or today's habitat holder.
        state.begin_encounter(&self.geography, &self.habitats);
        expedition_state_dictionary(state, &self.geography)
    }

    /// Starts only the battle declared by the current pending expedition
    /// encounter. Godot may display the returned snapshot; it cannot name a
    /// different battle or synthesize one when no encounter is pending.
    #[func]
    fn begin_pending_battle(&mut self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return battle_error_snapshot("expedition_not_configured");
        };
        let Some(encounter) = state.pending_encounter.as_ref() else {
            return battle_error_snapshot("no_pending_encounter");
        };
        if encounter.battle_id != BATTLE_ID {
            return battle_error_snapshot("unsupported_pending_battle");
        }
        self.battle_sequence = 0;
        self.battle = Some(Battle::prototype_vertical_slice());
        snapshot_dictionary(&self.battle.as_ref().expect("battle assigned").snapshot())
    }

    #[func]
    fn start_pending_battle(&mut self) -> Array<VarDictionary> {
        let Some(battle) = self.battle.as_mut() else {
            return expedition_battle_rejection(
                &mut self.battle_sequence,
                "",
                "battle_not_created",
            );
        };
        let events = battle.start();
        expedition_battle_records(&mut self.battle_sequence, events, Some(battle.snapshot()))
    }

    #[func]
    fn submit_pending_command(&mut self, command: VarDictionary) -> Array<VarDictionary> {
        let command_id = field_string(&command, "command_id").unwrap_or_default();
        let envelope = match command_envelope(&command) {
            Ok(value) => value,
            Err(reason) => {
                return expedition_battle_rejection(&mut self.battle_sequence, &command_id, reason);
            }
        };
        if envelope.battle_id != BATTLE_ID {
            return expedition_battle_rejection(
                &mut self.battle_sequence,
                &command_id,
                "battle_id_mismatch",
            );
        }
        let skill_command = match envelope.into_skill_command() {
            Ok(value) => value,
            Err(crate::protocol::ProtocolError::UnsupportedVersion { .. }) => {
                return expedition_battle_rejection(
                    &mut self.battle_sequence,
                    &command_id,
                    "unsupported_protocol_version",
                );
            }
            Err(_) => {
                return expedition_battle_rejection(
                    &mut self.battle_sequence,
                    &command_id,
                    "invalid_command_envelope",
                );
            }
        };
        let Some(battle) = self.battle.as_mut() else {
            return expedition_battle_rejection(
                &mut self.battle_sequence,
                &command_id,
                "battle_not_created",
            );
        };
        let events = match battle.submit(skill_command) {
            Ok(events) => events,
            Err(error) => {
                return expedition_battle_rejection(
                    &mut self.battle_sequence,
                    &command_id,
                    battle_error_code(&error),
                );
            }
        };
        let snapshot = battle.snapshot();
        let outcome = match snapshot.phase {
            BattlePhase::Victory => Some(EncounterOutcome::Victory),
            BattlePhase::Defeat => Some(EncounterOutcome::Defeat),
            BattlePhase::Retreated => Some(EncounterOutcome::Retreat),
            _ => None,
        };
        if let Some(outcome) = outcome {
            if let Some(state) = self.state.as_mut() {
                let _ = state.resolve_encounter(outcome, &self.geography, &self.habitats);
            }
        }
        expedition_battle_records(&mut self.battle_sequence, events, Some(snapshot))
    }

    #[func]
    fn pending_battle_snapshot(&self) -> VarDictionary {
        self.battle
            .as_ref()
            .map(|battle| snapshot_dictionary(&battle.snapshot()))
            .unwrap_or_else(|| battle_error_snapshot("battle_not_created"))
    }

    #[func]
    fn recommended_pending_enemy_command(&self, command_id: GString) -> VarDictionary {
        let Some(battle) = self.battle.as_ref() else {
            return vdict! { "available" => false, "reason" => "battle_not_created" };
        };
        let Some(decision) = battle.enemy_decision() else {
            return vdict! { "available" => false, "reason" => "active_actor_is_not_hostile" };
        };
        let target_ids: Array<GString> = array![decision.target_id.0.as_str()];
        vdict! {
            "available" => true, "protocol_version" => i64::from(PROTOCOL_VERSION),
            "command_id" => &command_id, "battle_id" => BATTLE_ID,
            "actor_id" => decision.actor_id.0.as_str(), "kind" => "use_skill",
            "skill_id" => decision.skill_id.as_str(), "target_ids" => &target_ids,
            "rationale" => decision.rationale.as_str(), "raw_damage" => i64::from(decision.raw_damage),
            "guard_break_amount" => i64::from(decision.guard_break_amount),
            "guard_absorbed" => i64::from(decision.guard_absorbed),
            "vitality_damage" => i64::from(decision.vitality_damage), "lethal" => decision.lethal,
            "interception_protector_id" => decision.interception_protector_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
            "fatal_intercept_available" => decision.fatal_intercept_available,
        }
    }
}

fn expedition_battle_records(
    sequence: &mut u64,
    events: Vec<BattleEvent>,
    snapshot: Option<BattleSnapshot>,
) -> Array<VarDictionary> {
    let mut records = Array::new();
    for event in events {
        *sequence += 1;
        records.push(&event_dictionary(event, *sequence, snapshot.as_ref()));
    }
    records
}

fn expedition_battle_rejection(
    sequence: &mut u64,
    command_id: &str,
    reason: &str,
) -> Array<VarDictionary> {
    *sequence += 1;
    let subjects = Array::<GString>::new();
    let payload = vdict! { "reason" => reason };
    let mut records = Array::new();
    records.push(&vdict! {
        "event_id" => format!("event.expedition.{:06}", sequence),
        "command_id" => command_id, "sequence" => *sequence as i64,
        "kind" => "command_rejected", "subjects" => &subjects, "payload" => &payload,
    });
    records
}

fn battle_error_snapshot(reason: &str) -> VarDictionary {
    let actors = VarArray::new();
    let error = vdict! { "kind" => reason };
    vdict! { "battle_id" => "", "phase" => "error", "actors" => &actors, "error" => &error }
}

impl Project42SimulationBridge {
    fn records(&mut self, events: Vec<BattleEvent>) -> Array<VarDictionary> {
        let snapshot = self.battle.as_ref().map(Battle::snapshot);
        let mut records = Array::new();
        for event in events {
            self.sequence += 1;
            records.push(&event_dictionary(event, self.sequence, snapshot.as_ref()));
        }
        records
    }

    fn rejection(&mut self, command_id: &str, reason: &str) -> Array<VarDictionary> {
        self.sequence += 1;
        let subjects = Array::<GString>::new();
        let payload = vdict! { "reason" => reason };
        let record = vdict! {
            "event_id" => format!("event.native.{:06}", self.sequence),
            "command_id" => command_id, "sequence" => self.sequence as i64,
            "kind" => "command_rejected", "subjects" => &subjects, "payload" => &payload,
        };
        let mut records = Array::new();
        records.push(&record);
        records
    }
}

fn command_envelope(value: &VarDictionary) -> Result<CommandEnvelope, &'static str> {
    let version = value
        .get("protocol_version")
        .and_then(|v| v.try_to::<i64>().ok())
        .ok_or("protocol_version_missing")?;
    let kind = field_string(value, "kind").ok_or("kind_missing")?;
    if kind != "use_skill" {
        return Err("unsupported_command_kind");
    }
    let target_array = value
        .get("target_ids")
        .and_then(|v| v.try_to::<Array<GString>>().ok())
        .ok_or("target_ids_invalid")?;
    Ok(CommandEnvelope {
        protocol_version: u16::try_from(version).map_err(|_| "protocol_version_invalid")?,
        command_id: field_string(value, "command_id").unwrap_or_default(),
        battle_id: field_string(value, "battle_id").unwrap_or_default(),
        actor_id: field_string(value, "actor_id").unwrap_or_default(),
        kind: CommandKind::UseSkill,
        skill_id: field_string(value, "skill_id").unwrap_or_default(),
        target_ids: target_array
            .iter_shared()
            .map(|value| value.to_string())
            .collect(),
    })
}

fn field_string(value: &VarDictionary, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.try_to::<GString>().ok())
        .map(|v| v.to_string())
}

fn expedition_state_dictionary(state: &ExpeditionState, geography: &Geography) -> VarDictionary {
    let mut party_ids = Array::<GString>::new();
    for id in &state.party_ids {
        let value = GString::from(id.as_str());
        party_ids.push(&value);
    }
    let mut route_history = Array::<VarDictionary>::new();
    for step in &state.route_history {
        route_history.push(&vdict! {
            "location_id" => step.location_id.as_str(),
            "arrived_on_day" => i64::from(step.arrived_on_day),
            "arrived_segment" => time_segment_name(&step.arrived_segment),
        });
    }
    let mut legal_commands = Array::<GString>::new();
    let error = state.legal_route_commands(geography).err();
    if error.is_none() {
        for command in state
            .legal_route_commands(geography)
            .expect("already checked")
        {
            let value = GString::from(command.as_str());
            legal_commands.push(&value);
        }
    }
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    let pending_encounter = state
        .pending_encounter
        .as_ref()
        .map(|encounter| {
            vdict! {
                "encounter_id" => encounter.encounter_id.as_str(),
                "battle_id" => encounter.battle_id.as_str(),
                "estate_upgrade_id" => encounter.estate_upgrade_id.as_deref().unwrap_or(""),
            }
        })
        .unwrap_or_default();
    let mut resolved_encounter_ids = Array::<GString>::new();
    for encounter_id in &state.resolved_encounter_ids {
        resolved_encounter_ids.push(&GString::from(encounter_id.as_str()));
    }
    let mut estate_upgrades = Array::<GString>::new();
    for estate_upgrade_id in &state.household_progress.estate_upgrades {
        estate_upgrades.push(&GString::from(estate_upgrade_id.as_str()));
    }
    let mut result = vdict! {
        "configured" => true,
        "save_version" => i64::from(state.save_version),
        "campaign_day" => i64::from(state.campaign_day),
        "time_segment" => time_segment_name(&state.time_segment),
        "active_location_id" => state.active_location_id.as_str(),
        "party_ids" => &party_ids,
        "route_history" => &route_history,
        "legal_route_commands" => &legal_commands,
        "travel_blocked_reason" => error.map(|value| expedition_error_code(&value)).unwrap_or(""),
        "pending_encounter" => &pending_encounter,
        "resolved_encounter_ids" => &resolved_encounter_ids,
        "estate_upgrades" => &estate_upgrades,
    };
    result.set("metadata", &metadata);
    result
}

fn expedition_error_dictionary(reason: &str) -> VarDictionary {
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    let mut result = vdict! { "configured" => false, "error" => reason };
    result.set("metadata", &metadata);
    result
}

fn time_segment_name(value: &TimeSegment) -> &'static str {
    match value {
        TimeSegment::Dawn => "dawn",
        TimeSegment::Day => "day",
        TimeSegment::Dusk => "dusk",
        TimeSegment::Midnight => "midnight",
    }
}

fn expedition_error_code(value: &ExpeditionError) -> &'static str {
    match value {
        ExpeditionError::MalformedJson(_) => "malformed_json",
        ExpeditionError::FutureSaveVersion { .. } => "future_save_version",
        ExpeditionError::InvalidStableId { .. } => "invalid_stable_id",
        ExpeditionError::InvalidPartySize { .. } => "invalid_party_size",
        ExpeditionError::IllegalRoute { .. } => "illegal_route",
        ExpeditionError::MissingDiscovery { .. } => "missing_discovery",
        ExpeditionError::NoPendingEncounter => "no_pending_encounter",
        ExpeditionError::MidnightBlockedByPendingEncounter => {
            "midnight_blocked_by_pending_encounter"
        }
        ExpeditionError::NotAtEstate => "not_at_estate",
        ExpeditionError::InsufficientSupplies { .. } => "insufficient_supplies",
        ExpeditionError::DuplicatePortal { .. } => "duplicate_portal",
        ExpeditionError::DuplicateAnchor { .. } => "duplicate_anchor",
        ExpeditionError::DuplicateEncounterTrigger => "duplicate_encounter_trigger",
        // A3 added these variants and this match is exhaustive, so their
        // codes belong here now. Projecting the anchor commands themselves to
        // Godot is B3's card, not this edit.
        ExpeditionError::AnchorNotHere { .. } => "anchor_not_here",
        ExpeditionError::AnchorSpentToday { .. } => "anchor_spent_today",
        ExpeditionError::TravelBlockedByEncounter { .. } => "travel_blocked_by_encounter",
    }
}

fn snapshot_dictionary(snapshot: &BattleSnapshot) -> VarDictionary {
    let mut actors = Array::<VarDictionary>::new();
    for actor in &snapshot.actors {
        actors.push(&actor_dictionary(actor));
    }
    let mut effects = Array::<VarDictionary>::new();
    for effect in &snapshot.effects {
        effects.push(&effect_dictionary(effect));
    }
    let mut recovery_openings = Array::<VarDictionary>::new();
    for opening in &snapshot.recovery_openings {
        recovery_openings.push(&recovery_opening_dictionary(opening));
    }
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    vdict! {
        "protocol_version" => i64::from(PROTOCOL_VERSION), "battle_id" => snapshot.battle_id.as_str(),
        "round" => snapshot.round as i64, "phase" => phase_name(&snapshot.phase),
        "active_actor_id" => snapshot.active_actor_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
        "description" => "The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush.",
        "actors" => &actors, "effects" => &effects, "recovery_openings" => &recovery_openings, "metadata" => &metadata,
    }
}

fn actor_dictionary(actor: &Actor) -> VarDictionary {
    let mut statuses = Array::<VarDictionary>::new();
    for status in &actor.statuses {
        statuses.push(&status_dictionary(status));
    }
    vdict! {
        "id" => actor.id.0.as_str(), "display_name" => actor.display_name.as_str(),
        "faction" => faction_name(&actor.faction), "level" => i64::from(actor.level),
        "vitality" => i64::from(actor.vitality), "max_vitality" => i64::from(actor.max_vitality),
        "guard" => i64::from(actor.guard), "band" => i64::from(actor.band), "statuses" => &statuses,
    }
}

fn status_dictionary(status: &StatusInstance) -> VarDictionary {
    vdict! { "id" => status.id.as_str(), "kind" => status_name(&status.kind), "remaining_rounds" => i64::from(status.remaining_rounds), "source_id" => status.source_id.0.as_str() }
}

fn effect_dictionary(effect: &BattlefieldEffect) -> VarDictionary {
    vdict! { "effect_id" => effect.instance_id.as_str(), "source_actor_id" => effect.source_actor_id.0.as_str(), "source_skill_id" => effect.source_skill_id.as_str(), "remaining_pulses" => i64::from(effect.remaining_pulses) }
}

fn recovery_opening_dictionary(opening: &RecoveryOpening) -> VarDictionary {
    vdict! { "actor_id" => opening.actor_id.0.as_str(), "source_skill_id" => opening.source_skill_id.as_str(), "bonus_raw_damage" => i64::from(opening.bonus_raw_damage) }
}

fn event_dictionary(
    event: BattleEvent,
    sequence: u64,
    snapshot: Option<&BattleSnapshot>,
) -> VarDictionary {
    let kind;
    let mut command_id = String::new();
    let mut subjects = Array::<GString>::new();
    let mut payload = VarDictionary::new();
    macro_rules! subject {
        ($id:expr) => {
            subjects.push(&$id.0);
        };
    }
    macro_rules! command {
        ($id:expr) => {
            command_id = $id;
        };
    }
    match event {
        BattleEvent::BattleStarted { round } => {
            kind = "battle_started";
            payload.set("round", round as i64);
        }
        BattleEvent::TurnStarted { round, actor_id } => {
            kind = "turn_started";
            subject!(actor_id);
            payload.set("round", round as i64);
        }
        BattleEvent::EnemyIntentDeclared {
            actor_id,
            skill_id,
            target_ids,
            rationale,
            guard_break_amount,
            raw_damage,
            guard_absorbed,
            vitality_damage,
            lethal,
            interception_protector_id,
            fatal_intercept_available,
        } => {
            kind = "enemy_intent_declared";
            subject!(actor_id);
            for id in target_ids {
                subject!(id);
            }
            payload.set("skill_id", skill_id);
            payload.set("rationale", rationale);
            payload.set("guard_break_amount", i64::from(guard_break_amount));
            payload.set("raw_damage", i64::from(raw_damage));
            payload.set("guard_absorbed", i64::from(guard_absorbed));
            payload.set("vitality_damage", i64::from(vitality_damage));
            payload.set("lethal", lethal);
            payload.set(
                "interception_protector_id",
                interception_protector_id.map(|id| id.0).unwrap_or_default(),
            );
            payload.set("fatal_intercept_available", fatal_intercept_available);
        }
        BattleEvent::CommandAccepted {
            command_id: id,
            actor_id,
            skill_id,
        } => {
            kind = "command_accepted";
            command!(id);
            subject!(actor_id);
            payload.set("skill_id", skill_id);
        }
        BattleEvent::ActorFocused {
            command_id: id,
            actor_id,
        } => {
            kind = "actor_focused";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::DamageApplied {
            command_id: id,
            source_id,
            target_id,
            amount,
        } => {
            kind = "damage_applied";
            command!(id);
            subject!(source_id);
            subject!(target_id);
            payload.set("amount", i64::from(amount));
            let remaining = snapshot
                .and_then(|state| state.actors.iter().find(|actor| actor.id == target_id))
                .map(|actor| actor.vitality)
                .unwrap_or(0);
            payload.set("remaining_vitality", i64::from(remaining));
        }
        BattleEvent::GuardChanged {
            command_id: id,
            actor_id,
            delta,
            total,
        } => {
            kind = "guard_changed";
            command!(id);
            subject!(actor_id);
            payload.set("delta", i64::from(delta));
            payload.set("total", i64::from(total));
        }
        BattleEvent::VitalityChanged {
            command_id: id,
            actor_id,
            delta,
            total,
        } => {
            kind = "vitality_changed";
            command!(id);
            subject!(actor_id);
            payload.set("delta", i64::from(delta));
            payload.set("total", i64::from(total));
        }
        BattleEvent::StatusRemoved {
            command_id: id,
            actor_id,
            status_id,
            status_kind,
        } => {
            kind = "status_removed";
            command!(id);
            subject!(actor_id);
            payload.set("status_id", status_id);
            payload.set("status_kind", status_name(&status_kind));
        }
        BattleEvent::ActorMoved {
            command_id: id,
            actor_id,
            from_band,
            to_band,
        } => {
            kind = "actor_moved";
            command!(id);
            subject!(actor_id);
            payload.set("from_band", i64::from(from_band));
            payload.set("to_band", i64::from(to_band));
        }
        BattleEvent::InterceptionSet {
            command_id: id,
            protector_id,
            protected_id,
        } => {
            kind = "interception_set";
            command!(id);
            subject!(protector_id);
            subject!(protected_id);
        }
        BattleEvent::InterceptionTriggered {
            command_id: id,
            protector_id,
            protected_id,
            attacker_id,
        } => {
            kind = "interception_triggered";
            command!(id);
            subject!(protector_id);
            subject!(protected_id);
            subject!(attacker_id);
        }
        BattleEvent::ReactionWindowOpened {
            command_id: id,
            trigger,
            threatened_actor_id,
        } => {
            kind = "reaction_window_opened";
            command!(id);
            subject!(threatened_actor_id);
            payload.set("trigger", trigger);
        }
        BattleEvent::ReactionTriggered {
            command_id: id,
            reactor_id,
            skill_id,
            protected_id,
        } => {
            kind = "reaction_triggered";
            command!(id);
            subject!(reactor_id);
            subject!(protected_id);
            payload.set("skill_id", skill_id);
        }
        BattleEvent::DefeatPrevented {
            command_id: id,
            actor_id,
            prevented_by_skill_id,
        } => {
            kind = "defeat_prevented";
            command!(id);
            subject!(actor_id);
            payload.set("prevented_by_skill_id", prevented_by_skill_id);
        }
        BattleEvent::ActorRevived {
            command_id: id,
            actor_id,
            vitality,
        } => {
            kind = "actor_revived";
            command!(id);
            subject!(actor_id);
            payload.set("vitality", i64::from(vitality));
        }
        BattleEvent::BonusTurnGranted {
            command_id: id,
            actor_id,
        } => {
            kind = "bonus_turn_granted";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::BattlefieldEffectCreated {
            command_id: id,
            effect_id,
            source_actor_id,
            source_skill_id,
            total_pulses,
        } => {
            kind = "battlefield_effect_created";
            command!(id);
            subject!(source_actor_id);
            payload.set("effect_id", effect_id);
            payload.set("source_skill_id", source_skill_id);
            payload.set("total_pulses", i64::from(total_pulses));
        }
        BattleEvent::BattlefieldEffectPulse {
            effect_id,
            source_actor_id,
            pulses_remaining_after,
        } => {
            kind = "battlefield_effect_pulse";
            subject!(source_actor_id);
            payload.set("effect_id", effect_id);
            payload.set("pulses_remaining_after", i64::from(pulses_remaining_after));
        }
        BattleEvent::BattlefieldEffectRemoved { effect_id, reason } => {
            kind = "battlefield_effect_removed";
            payload.set("effect_id", effect_id);
            payload.set("reason", reason);
        }
        BattleEvent::RecoveryOpeningCreated {
            command_id: id,
            actor_id,
            source_skill_id,
            bonus_raw_damage,
        } => {
            kind = "recovery_opening_created";
            command!(id);
            subject!(actor_id);
            payload.set("source_skill_id", source_skill_id);
            payload.set("bonus_raw_damage", i64::from(bonus_raw_damage));
        }
        BattleEvent::RecoveryOpeningConsumed {
            command_id: id,
            actor_id,
            attacker_id,
            bonus_raw_damage,
        } => {
            kind = "recovery_opening_consumed";
            command!(id);
            subject!(actor_id);
            subject!(attacker_id);
            payload.set("bonus_raw_damage", i64::from(bonus_raw_damage));
        }
        BattleEvent::RecoveryOpeningExpired { actor_id } => {
            kind = "recovery_opening_expired";
            subject!(actor_id);
        }
        BattleEvent::ActorDefeated {
            command_id: id,
            actor_id,
        } => {
            kind = "actor_defeated";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::TurnEnded { round, actor_id } => {
            kind = "turn_ended";
            subject!(actor_id);
            payload.set("round", round as i64);
        }
        BattleEvent::RoundStarted { round } => {
            kind = "round_started";
            payload.set("round", round as i64);
        }
        BattleEvent::BattleEnded { victory } => {
            kind = "battle_ended";
            payload.set("victory", victory);
        }
        BattleEvent::BattleRetreated {
            command_id: id,
            actor_id,
        } => {
            kind = "battle_retreated";
            command!(id);
            subject!(actor_id);
        }
    }
    vdict! { "event_id" => format!("event.native.{sequence:06}"), "command_id" => command_id, "sequence" => sequence as i64, "kind" => kind, "subjects" => &subjects, "payload" => &payload }
}

fn phase_name(value: &BattlePhase) -> &'static str {
    match value {
        BattlePhase::AwaitingActor => "awaiting_actor",
        BattlePhase::AwaitingCommand => "awaiting_command",
        BattlePhase::Resolving => "resolving",
        BattlePhase::Victory => "victory",
        BattlePhase::Defeat => "defeat",
        BattlePhase::Retreated => "retreated",
    }
}
fn faction_name(value: &Faction) -> &'static str {
    match value {
        Faction::Party => "party",
        Faction::Hostile => "hostile",
    }
}
fn status_name(value: &StatusKind) -> &'static str {
    match value {
        StatusKind::Bleeding => "bleeding",
        StatusKind::Poisoned => "poisoned",
        StatusKind::Burning => "burning",
        StatusKind::Stunned => "stunned",
    }
}

fn battle_error_code(value: &crate::battle::BattleError) -> &'static str {
    use crate::battle::BattleError::*;
    match value {
        BattleAlreadyEnded => "battle_already_ended",
        WrongPhase { .. } => "wrong_phase",
        NotActiveActor { .. } => "not_active_actor",
        UnknownActor(_) => "unknown_actor",
        UnknownTarget(_) => "unknown_target",
        ActorDefeated(_) => "actor_defeated",
        ActorIncapacitated { .. } => "actor_incapacitated",
        SkillUnavailable { .. } => "skill_unavailable",
        TargetMustBeDefeated(_) => "target_must_be_defeated",
        FriendlyFire { .. } => "friendly_fire",
        IllegalSelfTarget { .. } => "illegal_self_target",
        IllegalTargetCount { .. } => "illegal_target_count",
        SkillOwnerMismatch { .. } => "skill_owner_mismatch",
        HostileSkillUsedByNonHostile { .. } => "hostile_skill_used_by_non_hostile",
        UnsupportedSkill(_) => "unsupported_skill",
        RetreatNotAllowed => "retreat_not_allowed",
        IllegalRetreatActor(_) => "illegal_retreat_actor",
    }
}

struct Project42Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Project42Extension {}
