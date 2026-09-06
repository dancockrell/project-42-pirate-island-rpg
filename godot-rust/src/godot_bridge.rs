use godot::prelude::*;

use crate::battle::{
    Actor, Battle, BattleEvent, BattlePhase, BattleSnapshot, BattlefieldEffect, Faction,
    RecoveryOpening, StatusInstance, StatusKind,
};
use crate::expedition::{EncounterOutcome, ExpeditionError, ExpeditionState, TimeSegment};
use crate::geography::{
    AuthoredCell, CellDefinition, EncounterTriggerDefinition, Geography, PortalDefinition,
    RouteKind,
};
use crate::habitat::Habitats;
use crate::protocol::{CommandEnvelope, CommandKind, PROTOCOL_VERSION};
use crate::strategy::building::{BuildingDefinition, BuildingDefinitions, BuildingError};
use crate::strategy::faction::{
    FactionDefinition, FactionDefinitions, FactionError, StrategicState,
};
use crate::strategy::utility::Goal;
use crate::world::WorldEvent;
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
    /// B15: C9's `content/factions/*.json`, as Godot forwards them. Loaded by
    /// `configure` and handed to every midnight, so the strategic hours the
    /// engine runs score the same authored records the Rust harness scores.
    /// Empty until a configuration supplies records -- a payload authored
    /// before factions reached the bundle still loads.
    #[init(val = FactionDefinitions::new())]
    factions: FactionDefinitions,
    /// B16: C10's `content/buildings/*.json`, as Godot forwards them. Loaded by
    /// `configure` beside the faction registry and handed to every midnight, so
    /// S10's hourly elimination sweep reads what a building actually makes and
    /// how much of a cell it takes instead of reading every building as
    /// unlookupable. Empty until a configuration supplies records -- a payload
    /// authored before B16 still loads, and the sweep then reads conservatively
    /// exactly as it did before.
    #[init(val = BuildingDefinitions::new())]
    buildings: BuildingDefinitions,
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
    cells: Vec<AuthoredCell>,
    portals: Vec<PortalDefinition>,
    encounter_triggers: Vec<EncounterTriggerDefinition>,
    /// The authored faction records, in the field names
    /// [`FactionDefinition`] already reads, forwarded verbatim by
    /// `native_expedition_port.gd`. `serde(default)` because a payload from
    /// before B15 -- or a test that only cares about roads -- names no
    /// factions and must still configure; the island then runs on neutral
    /// weights exactly as it did before.
    #[serde(default)]
    factions: Vec<FactionDefinition>,
    /// The authored building records, in the field names
    /// [`BuildingDefinition`] already reads, forwarded verbatim by
    /// `native_expedition_port.gd`. `serde(default)` for the same reason
    /// `factions` carries it: a payload that names no buildings must still
    /// configure.
    #[serde(default)]
    buildings: Vec<BuildingDefinition>,
}

#[godot_api]
impl Project42SimulationBridge {
    /// B17: `Project42SimulationBridge` holds no expedition state -- it has no
    /// `configure`, no save and no campaign to ask -- so there are no bond
    /// ranks to read here and this keeps the review fixture's letters, Betty at
    /// SSS included. The campaign's answer is
    /// `Project42ExpeditionBridge::begin_pending_battle`, which is the path
    /// `CampaignEncounterSimulationPort` takes once a session is running.
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
            // The parser's own words after the code. A refused payload used to
            // say only that it was refused, which left the caller comparing a
            // thousand-line JSON string against a struct by eye; serde already
            // names the field and the type it wanted.
            Err(error) => {
                return expedition_error_dictionary(&format!(
                    "expedition_configuration_invalid:{error}"
                ));
            }
        };
        // Cells arrive in their authored shape; the one translation into the
        // simulation's cell lives in geography.rs, so a kind content misspells
        // is refused here rather than becoming a room that does nothing.
        let cells = match configuration
            .cells
            .into_iter()
            .map(CellDefinition::try_from)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(cells) => cells,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        self.geography = match Geography::from_authored(
            cells,
            configuration.portals,
            configuration.encounter_triggers,
        ) {
            Ok(geography) => geography,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        // B15: the authored faction registry, built before the state exists so
        // a bad record refuses the whole configuration rather than leaving the
        // bridge configured with half an island. `FactionDefinitions::insert`
        // validates each record and refuses a second one for a concept, so the
        // ID-drift and duplicate-concept traps S1 named are caught here, at the
        // boundary, and never reach the tick.
        let mut factions = FactionDefinitions::new();
        for definition in configuration.factions {
            if let Err(error) = factions.insert(definition) {
                return expedition_error_dictionary(&format!(
                    "expedition_configuration_invalid:{}",
                    faction_error_code(&error)
                ));
            }
        }
        self.factions = factions;
        // B16: and the building registry, on the same terms. `BuildingDefinitions::insert`
        // runs `BuildingDefinition::validate` -- brief section 5.6's "Michael's
        // buildings never make people", section 8's envelope rule, section 19's
        // socket fields -- and refuses a duplicate ID, so a record that breaks
        // one of those refuses the whole configuration here at the boundary
        // rather than reaching a tick.
        let mut buildings = BuildingDefinitions::new();
        for definition in configuration.buildings {
            if let Err(error) = buildings.insert(definition) {
                return expedition_error_dictionary(&format!(
                    "expedition_configuration_invalid:{}",
                    building_error_code(&error)
                ));
            }
        }
        self.buildings = buildings;
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
            &self.factions,
            &self.buildings,
        )
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.state
            .as_ref()
            .map(|state| {
                expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings)
            })
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
        expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings)
    }

    /// The one "do something here" verb, projected. Salvage the wreck, open a
    /// cache, use a room at the estate: all of it is `use_anchor`, and the
    /// snapshot comes back with an `anchor_outcome` block saying what changed.
    /// Refusals (not here, spent today, missing discovery, encounter pending)
    /// come back as the same error dictionaries travel uses.
    #[func]
    fn use_anchor(&mut self, anchor_id: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let outcome = match state.use_anchor(&anchor_id.to_string(), &self.geography) {
            Ok(outcome) => outcome,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        let mut healed = Array::<GString>::new();
        for id in &outcome.healed_character_ids {
            healed.push(&GString::from(id.as_str()));
        }
        let mut discoveries = Array::<GString>::new();
        for id in &outcome.discoveries_recorded {
            discoveries.push(&GString::from(id.as_str()));
        }
        let mut upgrades = Array::<GString>::new();
        for id in &outcome.upgrades_recorded {
            upgrades.push(&GString::from(id.as_str()));
        }
        let anchor_outcome = vdict! {
            "anchor_id" => outcome.anchor_id.as_str(),
            "rations_gained" => i64::from(outcome.rations_gained),
            "medicine_gained" => i64::from(outcome.medicine_gained),
            "coin_gained" => i64::from(outcome.coin_gained),
            "medicine_spent" => i64::from(outcome.medicine_spent),
            "healed_character_ids" => &healed,
            "discoveries_recorded" => &discoveries,
            "upgrades_recorded" => &upgrades,
        };
        let mut result =
            expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings);
        result.set("anchor_outcome", &anchor_outcome);
        result
    }

    /// Records the observation the player looked at. The named observation must
    /// be a legal command here -- Godot can only ask for what the projected
    /// list already offered -- and the recording itself is
    /// `ExpeditionState::inspect`, which owns what reading a place discovers.
    #[func]
    fn inspect(&mut self, observation_id: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let command = format!("inspect:{}", observation_id);
        if !state
            .legal_next_commands_with_geography(&self.geography)
            .iter()
            .any(|legal| legal == &command)
        {
            return expedition_error_dictionary("observation_not_here");
        }
        if let Err(error) = state.inspect_observation(&observation_id.to_string(), &self.geography)
        {
            return expedition_error_dictionary(expedition_error_code(&error));
        }
        expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings)
    }

    /// Midnight, as the one atomic transaction `ExpeditionState` already owns:
    /// the day turns, the island repopulates, the hunters take their step and
    /// every anchor spent today becomes usable again. Refused, without
    /// mutation, while an encounter is pending. The events midnight produced
    /// ride back on the snapshot so the screen can say what happened.
    #[func]
    fn resolve_midnight(&mut self) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let events = match state.resolve_midnight_in(
            &self.geography,
            &self.habitats,
            &self.factions,
            &self.buildings,
        ) {
            Ok(events) => events,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        let mut projected = Array::<VarDictionary>::new();
        for event in &events {
            projected.push(&world_event_dictionary(event));
        }
        let mut result =
            expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings);
        result.set("events", &projected);
        result
    }

    /// S2's control model, projected. Hands a cell to a faction, or releases
    /// it when `faction_id` is empty -- releasing drops the override rather
    /// than forcing the cell unheld, so a cell the graph gives an owner goes
    /// back to that owner. The snapshot comes back with an `events` array of
    /// the `ControlChanged` events the change produced; setting the control a
    /// cell already has is a legal no-op with an empty array. Refusals
    /// (`unknown_cell`, `invalid_stable_id`) come back as the same error
    /// dictionaries travel uses.
    ///
    /// Nothing about risk is written by this call. Every road touching the
    /// cell simply answers `effective_risk` differently from here on, which is
    /// why the route projection recomputes it every snapshot.
    #[func]
    fn set_control(&mut self, cell_id: GString, faction_id: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let faction_id = faction_id.to_string();
        let faction_id = if faction_id.is_empty() {
            None
        } else {
            Some(faction_id)
        };
        let events = match state.set_control(&cell_id.to_string(), faction_id, &self.geography) {
            Ok(events) => events,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        let mut projected = Array::<VarDictionary>::new();
        for event in &events {
            projected.push(&world_event_dictionary(event));
        }
        let mut result =
            expedition_state_dictionary(state, &self.geography, &self.factions, &self.buildings);
        result.set("events", &projected);
        result
    }

    /// Who effectively holds a cell: the campaign's `ownership` override where
    /// it names one, the graph's authored owner otherwise, and `""` for
    /// unheld. Godot never reads `ownership` raw -- this is the only answer,
    /// so the screen and the simulation cannot disagree about who holds a road.
    #[func]
    fn controller_of(&self, cell_id: GString) -> GString {
        let Some(state) = self.state.as_ref() else {
            return GString::new();
        };
        GString::from(effective_controller(state, &self.geography, &cell_id.to_string()).as_str())
    }

    /// One road's live danger, from `Geography::effective_risk` and nowhere
    /// else: the authored base plus `CONTESTED_RISK_MODIFIER` while the two
    /// endpoints are held by different parties. Returns the integer, or the
    /// `illegal_route` error dictionary when no portal carries this ID.
    #[func]
    fn effective_risk(&self, portal_id: GString) -> Variant {
        let Some(state) = self.state.as_ref() else {
            return expedition_error_dictionary("expedition_not_configured").to_variant();
        };
        let portal_id = portal_id.to_string();
        let Some(route) = self.geography.route(&portal_id) else {
            return expedition_error_dictionary("illegal_route").to_variant();
        };
        i64::from(self.geography.effective_risk(route, &state.ownership)).to_variant()
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
        // B17: the campaign is what says where each woman's bond stands, so the
        // battle it arms carries `ExpeditionState::bond_ranks` and not the
        // review fixture's letters. A fresh campaign's Betty fights with her
        // rank D Guarded Strike alone until an authored scene raises her.
        let battle = Battle::prototype_vertical_slice_from_bond_ranks(&state.bond_ranks);
        self.battle_sequence = 0;
        self.battle = Some(battle);
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

    /// B8: this campaign as the canonical save JSON `ExpeditionState::to_json`
    /// already writes -- byte-identical for equal states, because every map
    /// behind it is a `BTreeMap`. This is a projection of the save format, not
    /// a second one: Godot writes exactly these bytes to a slot and hands
    /// exactly them back to `load_json`, so nothing on the engine side ever
    /// needs to understand the shape it is carrying.
    ///
    /// Empty while nothing is configured. There is no half-save and no
    /// placeholder document: an empty string handed back to `load_json` is
    /// refused as malformed, which is the truthful answer for a slot written
    /// from a session that had no campaign in it.
    #[func]
    fn save_json(&self) -> GString {
        self.state
            .as_ref()
            .map(|state| GString::from(state.to_json().as_str()))
            .unwrap_or_default()
    }

    /// B8: replaces this campaign with the one the JSON carries, through
    /// [`ExpeditionState::from_json`] and nothing else -- the same parse, the
    /// same `validate`, and above all the same `save_version` gate that
    /// `godot-rust/tests/save_migration.rs` holds. A slot from a future build
    /// is refused here for exactly the reason it is refused there, so the game
    /// and the migration harness cannot disagree about what loads.
    ///
    /// A refused save leaves the session precisely as it stood: the field is
    /// assigned only after the parse and the validation have both succeeded,
    /// so a corrupted slot can never become half a campaign and is never
    /// silently replaced by a new game. Refusals come back as the same error
    /// dictionary `configure`, `travel` and `use_anchor` answer with --
    /// `configured: false` plus the `expedition_error_code` name -- because a
    /// second rejection shape on this boundary would be a second thing for
    /// Godot to get right.
    ///
    /// The geography, habitats and faction registry are content and are not in
    /// the save: they stay whatever `configure` last loaded, which is why a
    /// session configures before it continues.
    #[func]
    fn load_json(&mut self, json: GString) -> VarDictionary {
        let state = match ExpeditionState::from_json(&json.to_string()) {
            Ok(state) => state,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        self.state = Some(state);
        expedition_state_dictionary(
            self.state.as_ref().expect("state assigned"),
            &self.geography,
            &self.factions,
            &self.buildings,
        )
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

fn expedition_state_dictionary(
    state: &ExpeditionState,
    geography: &Geography,
    factions: &FactionDefinitions,
    buildings: &BuildingDefinitions,
) -> VarDictionary {
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
    let mut legal_route_commands = Array::<GString>::new();
    let error = state.legal_route_commands(geography).err();
    if error.is_none() {
        for command in state
            .legal_route_commands(geography)
            .expect("already checked")
        {
            let value = GString::from(command.as_str());
            legal_route_commands.push(&value);
        }
    }
    // B3: every legal verb here, not travel alone -- `travel:<portal>`,
    // `inspect:<observation>`, `anchor_action:<anchor>`, or the pending
    // encounter when one is open. The screen draws its controls from this and
    // nothing else, so an anchor already spent today or a gated door is never
    // drawn. `legal_route_commands` stays beside it: it is what the route list
    // and two suites already read, and it carries the refusal reason.
    let mut legal_commands = Array::<GString>::new();
    for command in state.legal_next_commands_with_geography(geography) {
        let value = GString::from(command.as_str());
        legal_commands.push(&value);
    }
    // B14: the routes the screen actually draws, each carrying the risk the
    // party would run *right now*. `risk_level` is `Geography::effective_risk`
    // -- the authored base plus S2's contested modifier while the endpoints are
    // held by different parties -- and the authored base is deliberately not
    // projected beside it: two numbers for one road's danger is the fork, and
    // GDScript that stored either would be a third. `contested` is the same
    // comparison Rust already made, said in a word the screen can mark.
    // Empty while an encounter is pending, for the same reason
    // `legal_route_commands` is: no road is legal until the fight resolves,
    // and a road drawn with a risk on it is a road the screen offers.
    let mut route_options = Array::<VarDictionary>::new();
    for route in if error.is_none() {
        state.legal_routes(geography)
    } else {
        Vec::new()
    } {
        let risk_level = geography.effective_risk(route, &state.ownership);
        route_options.push(&vdict! {
            "portal_id" => route.id.as_str(),
            "to_location_id" => route.to_location_id.as_str(),
            "travel_mode" => route_kind_name(&route.kind),
            "time_cost_minutes" => i64::from(route.time_cost_minutes),
            "supply_cost" => i64::from(route.supply_cost),
            "risk_level" => i64::from(risk_level),
            "contested" => risk_level != route.risk_level,
        });
    }
    let mut discoveries = Array::<GString>::new();
    for discovery_id in &state.discoveries {
        discoveries.push(&GString::from(discovery_id.as_str()));
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
    // B15: what the loaded registry's factions are doing, and nothing else.
    // The list is the *registry's* -- one entry per authored record, whether or
    // not the save has met that faction yet -- because the registry is what the
    // engine and the harness now share, and a faction the campaign has no state
    // for is simply at its defaults.
    //
    // Three fields, all verdicts. Brief section 9 forbids exposing raw utility
    // arithmetic, so no score, no weight and no number from `strategy/utility.rs`
    // crosses this boundary: a screen that could read a score would start
    // drawing one, and the verdict would stop being the interface.
    let mut factions_projected = Array::<VarDictionary>::new();
    for faction_id in factions.ids() {
        let held = state.factions.get(faction_id);
        let strategic_state = held.map(|f| f.strategic_state).unwrap_or_default();
        let mut current_goals = Array::<GString>::new();
        for goal in held.map(|f| f.current_goals.as_slice()).unwrap_or_default() {
            current_goals.push(&GString::from(goal_name(goal)));
        }
        factions_projected.push(&vdict! {
            "id" => faction_id,
            "strategic_state" => strategic_state_name(&strategic_state),
            "current_goals" => &current_goals,
        });
    }
    // B16: which authored building records the bridge is holding, and nothing
    // else. Ids only -- the registry is content, and a screen that could read a
    // tier's hit points or a production interval off the save would be reading
    // the record through the wrong door. What is *standing* on the island is
    // `ExpeditionState::buildings` and is not projected here: no lane places a
    // building through the tick yet, so a `buildings` array that mixed records
    // with instances would be two answers to one key on the day one does.
    let mut building_ids = Array::<GString>::new();
    for building_id in buildings.ids() {
        building_ids.push(&GString::from(building_id));
    }
    let mut result = vdict! {
        "configured" => true,
        "save_version" => i64::from(state.save_version),
        "campaign_day" => i64::from(state.campaign_day),
        "time_segment" => time_segment_name(&state.time_segment),
        "active_location_id" => state.active_location_id.as_str(),
        "party_ids" => &party_ids,
        "route_history" => &route_history,
        "legal_route_commands" => &legal_route_commands,
        "legal_commands" => &legal_commands,
        "route_options" => &route_options,
        "discoveries" => &discoveries,
        "travel_blocked_reason" => error.map(|value| expedition_error_code(&value)).unwrap_or(""),
        "pending_encounter" => &pending_encounter,
        "resolved_encounter_ids" => &resolved_encounter_ids,
        "estate_upgrades" => &estate_upgrades,
    };
    result.set("factions", &factions_projected);
    result.set("buildings", &building_ids);
    result.set("metadata", &metadata);
    result
}

/// Every `WorldEvent` midnight can produce, projected by name. The match is
/// exhaustive on purpose: a new world event cannot be added without this
/// boundary being told what to call it, because the crate will not compile
/// until it is.
fn world_event_dictionary(event: &WorldEvent) -> VarDictionary {
    match event {
        WorldEvent::MidnightFlashStarted { day } => {
            vdict! { "kind" => "midnight_flash_started", "day" => i64::from(*day) }
        }
        WorldEvent::NamedPersonReturned {
            person_id,
            killed_by_player_count,
        } => vdict! {
            "kind" => "named_person_returned",
            "person_id" => person_id.as_str(),
            "killed_by_player_count" => i64::from(*killed_by_player_count),
        },
        WorldEvent::MonsterMaterialized { region_id, monster } => {
            let mut behavior_tags = Array::<GString>::new();
            for tag in &monster.behavior_tags {
                behavior_tags.push(&GString::from(tag.as_str()));
            }
            // `loot_seed` is a full u64 hash. Godot integers are signed, so it
            // is projected as its decimal text rather than silently wrapping
            // negative past i64::MAX.
            let materialized = vdict! {
                "instance_id" => monster.instance_id.as_str(),
                "definition_id" => monster.definition_id.as_str(),
                "level" => i64::from(monster.level),
                "behavior_tags" => &behavior_tags,
                "physical_variant" => monster.physical_variant.as_str(),
                "condition" => monster.condition.as_str(),
                "patrol_purpose" => monster.patrol_purpose.as_str(),
                "loot_seed" => monster.loot_seed.to_string(),
            };
            let mut result = vdict! {
                "kind" => "monster_materialized",
                "region_id" => region_id.as_str(),
            };
            result.set("monster", &materialized);
            result
        }
        // S2 landed this variant while B3 was in flight; the match is
        // exhaustive, so the merge could not compile without it. Shape as S2
        // specified: the effective controller either side, "" for unheld.
        WorldEvent::ControlChanged {
            cell_id,
            from,
            to,
            day,
        } => vdict! {
            "kind" => "control_changed",
            "cell_id" => cell_id.as_str(),
            "from" => from.as_deref().unwrap_or(""),
            "to" => to.as_deref().unwrap_or(""),
            "day" => i64::from(*day),
        },
        WorldEvent::MidnightFlashEnded { day, monster_count } => vdict! {
            "kind" => "midnight_flash_ended",
            "day" => i64::from(*day),
            "monster_count" => *monster_count as i64,
        },
    }
}

fn expedition_error_dictionary(reason: &str) -> VarDictionary {
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    let mut result = vdict! { "configured" => false, "error" => reason };
    result.set("metadata", &metadata);
    result
}

/// What went wrong loading an authored faction record, named. Exhaustive on
/// purpose, like every other projection in this file: a new `FactionError`
/// cannot be added without this boundary being told what to call it. Godot
/// reads the name after the `expedition_configuration_invalid:` prefix, so a
/// content author sees which of S1's two traps their record fell into.
fn faction_error_code(value: &FactionError) -> &'static str {
    match value {
        FactionError::MalformedId(error) => expedition_error_code(error),
        FactionError::IdDoesNotMatchConceptKey { .. } => "id_does_not_match_concept_key",
        FactionError::DuplicateFaction { .. } => "duplicate_faction",
        FactionError::UnknownFaction { .. } => "unknown_faction",
    }
}

/// What went wrong loading an authored building record, named. Exhaustive for
/// the same reason `faction_error_code` is: a new `BuildingError` cannot be
/// added without this boundary being told what to call it. Godot reads the name
/// after the `expedition_configuration_invalid:` prefix, so a content author
/// sees which of `BuildingDefinition::validate`'s rules their record broke.
fn building_error_code(value: &BuildingError) -> &'static str {
    match value {
        BuildingError::MalformedId(error) => expedition_error_code(error),
        BuildingError::WrongIdPrefix { .. } => "wrong_id_prefix",
        BuildingError::Duplicate { .. } => "duplicate_building",
        BuildingError::UnknownDefinition { .. } => "unknown_building_definition",
        BuildingError::UnknownBuilding { .. } => "unknown_building",
        BuildingError::UnknownCell { .. } => "unknown_cell",
        BuildingError::CellNotHeld { .. } => "cell_not_held",
        BuildingError::IncompatibleFaction { .. } => "incompatible_faction",
        BuildingError::EnvelopeOverlap { .. } => "envelope_overlap",
        BuildingError::SocketOutsideEnvelope { .. } => "socket_outside_envelope",
        BuildingError::SocketInWrongField { .. } => "socket_in_wrong_field",
        BuildingError::MichaelBuildingProducesPeople { .. } => "michael_building_produces_people",
        BuildingError::TimerDrivenHumanRole { .. } => "timer_driven_human_role",
        BuildingError::HumanRoleWithoutRecruitmentSupport { .. } => {
            "human_role_without_recruitment_support"
        }
        BuildingError::NeedsDecision { .. } => "building_needs_decision",
        BuildingError::MalformedTiers { .. } => "malformed_tiers",
        BuildingError::ProductionAboveTopTier { .. } => "production_above_top_tier",
        BuildingError::AtTopTier { .. } => "at_top_tier",
        BuildingError::RequirementsNotMet { .. } => "requirements_not_met",
        BuildingError::WrongState { .. } => "wrong_building_state",
        BuildingError::AlreadyHeld { .. } => "already_held",
    }
}

/// Where a faction stands on the board, in the same words `strategy/faction.rs`
/// serializes. Exhaustive, so a sixth board position cannot appear without this
/// boundary naming it.
fn strategic_state_name(value: &StrategicState) -> &'static str {
    match value {
        StrategicState::Desperate => "desperate",
        StrategicState::Recovering => "recovering",
        StrategicState::Contesting => "contesting",
        StrategicState::Advantaged => "advantaged",
        StrategicState::Closing => "closing",
    }
}

/// One goal, in the same words `strategy/utility.rs` serializes. Exhaustive for
/// the same reason.
fn goal_name(value: &Goal) -> &'static str {
    match value {
        Goal::Recover => "recover",
        Goal::Consolidate => "consolidate",
        Goal::Develop => "develop",
        Goal::Expand => "expand",
        Goal::Pressure => "pressure",
        Goal::Withdraw => "withdraw",
    }
}

/// The authored `travelMode` vocabulary, back the way content wrote it, so a
/// projected route names its mode in the same words
/// `RouteKind::from_travel_mode` read. Exhaustive on purpose: a new kind
/// cannot be added without this boundary being told what to call it.
fn route_kind_name(value: &RouteKind) -> &'static str {
    match value {
        RouteKind::Direct => "on_foot",
        RouteKind::SafeRoad => "safe_road",
        RouteKind::JungleEdge => "jungle_edge",
    }
}

/// Who effectively holds a cell: the campaign's `ownership` override where it
/// names one, the graph's authored owner otherwise. The rule is
/// `Geography::effective_risk`'s, and that method stays its owner for risk;
/// this composes the same two public queries so `controller_of` can answer the
/// question on its own, and Godot never reads `ownership` raw.
fn effective_controller(state: &ExpeditionState, geography: &Geography, cell_id: &str) -> String {
    // One owner: Geography::held_by is the composition; this only stringifies.
    geography
        .held_by(cell_id, &state.ownership)
        .unwrap_or_default()
        .to_owned()
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
        ExpeditionError::InsufficientSupplies { .. } => "insufficient_supplies",
        ExpeditionError::DuplicatePortal { .. } => "duplicate_portal",
        ExpeditionError::DuplicateAnchor { .. } => "duplicate_anchor",
        ExpeditionError::UnknownAnchorKind { .. } => "unknown_anchor_kind",
        ExpeditionError::DuplicateEncounterTrigger => "duplicate_encounter_trigger",
        // A3 added these variants and this match is exhaustive, so their
        // codes belong here now. Projecting the anchor commands themselves to
        // Godot is B3's card, not this edit.
        ExpeditionError::AnchorNotHere { .. } => "anchor_not_here",
        ExpeditionError::AnchorSpentToday { .. } => "anchor_spent_today",
        // A4 replaced the estate-only rest command with estate anchors, so
        // `not_at_estate` is gone and these two gates take its place. Same
        // reason as above: the match is exhaustive.
        ExpeditionError::AnchorRequiresDiscovery { .. } => "anchor_requires_discovery",
        ExpeditionError::AnchorAlreadyResolved { .. } => "anchor_already_resolved",
        ExpeditionError::TravelBlockedByEncounter { .. } => "travel_blocked_by_encounter",
        // S12 added these two and this match is exhaustive, so their codes
        // belong here now. Nothing else of S12 reaches the bridge from this
        // edit: projecting a recruitment arc is B3's card, and when it does,
        // `RecruitmentState::projection()` is the only thing it may pass -- a
        // stage word and the current authored beat ID, never a disposition.
        ExpeditionError::UnknownRecruit { .. } => "unknown_recruit",
        ExpeditionError::MilestoneAlreadyRecorded { .. } => "milestone_already_recorded",
        // S2 added `UnknownCell` and this match is exhaustive, so its code
        // belongs here now -- the same one-line obligation A3 and A4 recorded
        // above. Projecting `set_control` and effective route risk to Godot is
        // B3's card, not this edit; this arm adds no bridge surface.
        ExpeditionError::UnknownCell { .. } => "unknown_cell",
        // The code the bridge's inspect already answered with before the rule
        // moved into ExpeditionState, so GDScript sees the same string.
        ExpeditionError::ObservationNotHere { .. } => "observation_not_here",
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
        "guard" => i64::from(actor.guard), "band" => i64::from(actor.band),
        "band_name" => actor.band_kind().map(|band| band.name()).unwrap_or("unknown"),
        "composure" => i64::from(actor.composure),
        // A10: a letter, never a number. The card rail draws the rank as it is
        // authored; nothing on the Godot side may turn it into a meter, and
        // there is no index beside it to make that easy.
        "bond_rank" => actor.bond_rank.as_str(),
        "statuses" => &statuses,
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
        StatusKind::Shaken => "shaken",
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
        RepositionNotLegal { .. } => "reposition_not_legal",
        ShakenCannotUse { .. } => "shaken_cannot_use",
        UnsupportedSkill(_) => "unsupported_skill",
        RetreatNotAllowed => "retreat_not_allowed",
        IllegalRetreatActor(_) => "illegal_retreat_actor",
        BondRankTooLow { .. } => "bond_rank_too_low",
    }
}

struct Project42Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Project42Extension {}
