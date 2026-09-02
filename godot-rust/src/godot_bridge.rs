use godot::prelude::*;

use crate::battle::{
    Actor, Battle, BattleEvent, BattlePhase, BattleSnapshot, BattlefieldEffect, Faction,
    StatusInstance, StatusKind,
};
use crate::protocol::{CommandEnvelope, CommandKind, PROTOCOL_VERSION};

const BATTLE_ID: &str = "battle.prototype.returning_names";

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Project42SimulationBridge {
    #[init(val = None)]
    battle: Option<Battle>,
    #[init(val = 0)]
    sequence: u64,
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
            "guard_absorbed" => i64::from(decision.guard_absorbed),
            "vitality_damage" => i64::from(decision.vitality_damage), "lethal" => decision.lethal,
            "interception_protector_id" => decision.interception_protector_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
            "fatal_intercept_available" => decision.fatal_intercept_available,
        }
    }
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

fn snapshot_dictionary(snapshot: &BattleSnapshot) -> VarDictionary {
    let mut actors = Array::<VarDictionary>::new();
    for actor in &snapshot.actors {
        actors.push(&actor_dictionary(actor));
    }
    let mut effects = Array::<VarDictionary>::new();
    for effect in &snapshot.effects {
        effects.push(&effect_dictionary(effect));
    }
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    vdict! {
        "protocol_version" => i64::from(PROTOCOL_VERSION), "battle_id" => snapshot.battle_id.as_str(),
        "round" => snapshot.round as i64, "phase" => phase_name(&snapshot.phase),
        "active_actor_id" => snapshot.active_actor_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
        "description" => "The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush.",
        "actors" => &actors, "effects" => &effects, "metadata" => &metadata,
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
        } => {
            kind = "status_removed";
            command!(id);
            subject!(actor_id);
            payload.set("status_id", status_id);
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
        UnsupportedSkill(_) => "unsupported_skill",
    }
}

struct Project42Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Project42Extension {}
