use std::collections::BTreeMap;

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
use crate::strategy::directive::{DirectiveStatus, Explanation, StrategicDirective};
use crate::strategy::dungeon::{CorruptionBand, HeatBand};
use crate::strategy::faction::{
    FactionDefinition, FactionDefinitions, FactionError, StrategicState,
};
use crate::strategy::force::ForceError;
use crate::strategy::journal::JournalEntry;
use crate::strategy::production::{MachineDefinition, MachineDefinitions, ProductionError};
use crate::strategy::site_rule::{AuthoredSiteRule, AuthoredSiteRuleError, SiteRules};
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
    /// B19: C14's `content/machines/*.json`, as Godot forwards them. Loaded by
    /// `configure` beside the building registry and handed to every midnight,
    /// so S16's production hour can look up the record a machine rule names
    /// instead of journaling a skip against a registry the bridge built empty.
    /// Empty until a configuration supplies records -- a payload authored
    /// before B19 still loads, and the hour then skips exactly as it did
    /// before.
    #[init(val = MachineDefinitions::new())]
    machines: MachineDefinitions,
    /// A7: `content/site_rules/*.json`, as Godot forwards them. Loaded by
    /// `configure` beside the faction and building registries, for the same
    /// reason both of those are: `Battle` stands under the rules a cell
    /// declares, and a registry that never reached Godot would leave the
    /// engine fighting under no rules while the harness fought under the
    /// tomb's. Empty until a configuration supplies records -- a payload
    /// authored before this card still loads, and every battle then stands
    /// under nothing, exactly as it did before.
    #[init(val = SiteRules::new())]
    site_rules: SiteRules,
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
    /// The authored machine records, in the field names [`MachineDefinition`]
    /// already reads, forwarded verbatim by `native_expedition_port.gd`.
    /// `serde(default)` for the same reason `factions` and `buildings` carry
    /// it: a payload that names no machines must still configure.
    #[serde(default)]
    machines: Vec<MachineDefinition>,
    /// The authored site-rule records, in the field names
    /// [`AuthoredSiteRule`] already reads. `serde(default)` for the same
    /// reason `factions` and `buildings` carry it.
    #[serde(default)]
    site_rules: Vec<AuthoredSiteRule>,
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
        // B19: and the machine registry, on the same terms.
        // `MachineDefinitions::insert` runs `MachineDefinition::validate` -- a
        // stable ID under the `machine.` prefix -- and refuses a duplicate
        // record, so a record that breaks either refuses the whole
        // configuration here at the boundary rather than reaching a production
        // hour and being read as a machine nobody authored.
        let mut machines = MachineDefinitions::new();
        for definition in configuration.machines {
            if let Err(error) = machines.insert(definition) {
                return expedition_error_dictionary(&format!(
                    "expedition_configuration_invalid:{}",
                    machine_error_code(&error)
                ));
            }
        }
        self.machines = machines;
        // A7: and the site-rule registry, on the same terms.
        // `SiteRules::insert` runs `AuthoredSiteRule::validate` -- a stable ID
        // under the `site_rule.` prefix, a display name, and an `effect` that
        // is one of the two shapes the closed vocabulary admits -- and refuses
        // a duplicate, so a record with an effect nobody decided refuses the
        // whole configuration here rather than becoming a silent no-op in a
        // fight.
        let mut site_rules = SiteRules::new();
        for definition in configuration.site_rules {
            if let Err(error) = site_rules.insert(definition) {
                return expedition_error_dictionary(&format!(
                    "expedition_configuration_invalid:{}",
                    site_rule_error_code(&error)
                ));
            }
        }
        self.site_rules = site_rules;
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
            &self.machines,
        )
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.state
            .as_ref()
            .map(|state| {
                expedition_state_dictionary(
                    state,
                    &self.geography,
                    &self.factions,
                    &self.buildings,
                    &self.machines,
                )
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
        expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        )
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
        let mut result = expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        );
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
        expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        )
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
            // B19: the tick runs production timers, and a machine rule names a
            // `machine.<...>` record. C14 authored `content/machines/`, the
            // port forwards those records, and `configure` validated them into
            // the registry handed in here -- so a rule that comes due makes the
            // machine the record describes rather than journaling a skip
            // against a lookup that could never succeed.
            &self.machines,
        ) {
            Ok(events) => events,
            Err(error) => return expedition_error_dictionary(expedition_error_code(&error)),
        };
        let mut projected = Array::<VarDictionary>::new();
        for event in &events {
            projected.push(&world_event_dictionary(event));
        }
        let mut result = expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        );
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
        let mut result = expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        );
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
        //
        // A7: and the campaign is what says which site rules hold here -- the
        // encounter's own location's `site_rule_ids`, minus the ones this
        // expedition has already overridden -- and whether Ayla's two
        // site-scoped commands still have their charge. `ExpeditionState`
        // decides all of it (`battle_setup`); this only carries the answer.
        let battle = Battle::prototype_vertical_slice_from_campaign(
            &state.battle_setup(&self.geography, &self.site_rules),
        );
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
        // A7: an overridden site rule and a spent Deny Activation outlive the
        // fight, so they are written back into the campaign before the
        // encounter's own outcome is resolved.
        if let Some(state) = self.state.as_mut() {
            state.record_battle_site_events(&self.geography, &events);
        }
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
            &self.machines,
        )
    }

    /// P5: what the calm information surface reads, and nothing else.
    ///
    /// Two things the expedition snapshot does not carry and a screen cannot
    /// compute for itself:
    ///
    /// * **the directives standing right now**, each with the plain-language
    ///   explanation brief section 5.9 requires the player to be shown *before*
    ///   they confirm. The stored explanation is preferred, because it is the
    ///   one the player actually agreed to; a directive that predates the
    ///   stored field is explained afresh through
    ///   [`ExpeditionState::explain_directive`], which is pure and mutates
    ///   nothing.
    /// * **the tail of S11's journal**, in prose, so the journal a paused
    ///   player reads says what happened rather than showing a variant name.
    ///
    /// Read-only, like `snapshot` and `save_json`: this method takes `&self`,
    /// so there is no way for a screen to move the island by looking at it.
    /// Nothing numeric that came out of the utility model crosses here -- brief
    /// section 9 -- and the words are written beside the facts they describe,
    /// so the engine never rewords what the simulation said.
    ///
    /// `journal_limit` is clamped into `0..=window`, so a screen may ask for
    /// more lines than exist without special-casing an empty campaign.
    #[func]
    fn strategic_surface(&self, journal_limit: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let mut directives = Array::<VarDictionary>::new();
        for directive in state.directives.values() {
            if directive.state != DirectiveStatus::Active {
                continue;
            }
            directives.push(&directive_dictionary(state, directive, &self.geography));
        }
        let entries = state.strategic_journal.recent();
        let limit = journal_limit.clamp(0, entries.len() as i64) as usize;
        let mut journal = Array::<VarDictionary>::new();
        for entry in &entries[entries.len() - limit..] {
            journal.push(&journal_entry_dictionary(entry));
        }
        let mut result = vdict! {
            "configured" => true,
            // A u64 count as decimal text, exactly as `loot_seed` is: Godot
            // integers are signed and a campaign's total is not this screen's
            // arithmetic anyway.
            "journal_total_recorded" => state.strategic_journal.total_recorded().to_string(),
        };
        result.set("directives", &directives);
        result.set("journal", &journal);
        result
    }

    /// **S18: the player raises a body.** Brief section 5.9 gives Captain
    /// Michael's strategic intent to the player, and until this verb existed
    /// nothing outside the crate could act on that intent at all: `raise_force`
    /// and `dispatch_force` were Rust methods with no `#[func]`, so a live
    /// campaign's `forces` array was always empty and card P4 shipped
    /// `blocked: needs a bridge verb`.
    ///
    /// **Not restricted to Michael's faction**, and deliberately so: `faction_id`
    /// is the caller's, exactly as it is on `set_control`, because the engine
    /// side already arranges the board for the fiction (the authored holders of
    /// the tomb and the landing) and a second, narrower rule here would be a
    /// second answer to who may be named. What the *simulation* will not do is
    /// act for Michael on its own -- `act_on_goals` refuses -- and that refusal
    /// is where brief section 5.9 lives.
    ///
    /// `composition` is `{actor id: count}`; Godot writes every number with a
    /// decimal point, so each value is read as a float and floored, the same
    /// coercion the port makes on the way in. Refusals come back as the error
    /// dictionary every other verb uses, with the force module's own error name
    /// -- an ID that is not a `force.` ID, one this campaign already carries, a
    /// cell the graph does not know, a body with nobody in it.
    ///
    /// **Where "refuse while paused" lives, and why it is not here.** Brief
    /// section 17 is kept by pause not being a state: there is no `paused`
    /// field in `ExpeditionState` and no host clock, so a paused game is
    /// exactly a game whose bridge is not called, and the one guard is
    /// `CampaignSession.advance_refused_while_paused` -- the same guard
    /// `resolve_midnight` goes through. Nor is this verb refused while an
    /// encounter is pending, and that is deliberate rather than an omission:
    /// `travel` and `resolve_midnight` are refused there because they *advance*
    /// the world, and raising a body advances nothing. It stands where it was
    /// raised until an hour runs, and a pending encounter is already what stops
    /// an hour from running.
    #[func]
    fn raise_force(
        &mut self,
        force_id: GString,
        faction_id: GString,
        cell_id: GString,
        composition: VarDictionary,
        assignment: GString,
    ) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let mut heads: BTreeMap<String, u32> = BTreeMap::new();
        for (key, value) in composition.iter_shared() {
            let Ok(actor_id) = key.try_to::<GString>() else {
                return expedition_error_dictionary("force_composition_malformed");
            };
            // Godot writes an integer with a decimal point as often as not,
            // so both readings are accepted and floored -- the same coercion
            // the port makes on every other number crossing here. Neither is
            // preferred: whichever the dictionary actually carries is read.
            let count = match value.try_to::<i64>() {
                Ok(count) => count as f64,
                Err(_) => match value.try_to::<f64>() {
                    Ok(count) => count,
                    Err(_) => {
                        return expedition_error_dictionary("force_composition_malformed");
                    }
                },
            };
            if !count.is_finite() || count < 0.0 {
                return expedition_error_dictionary("force_composition_malformed");
            }
            heads.insert(actor_id.to_string(), count.floor() as u32);
        }
        let force_id = force_id.to_string();
        let faction_id = faction_id.to_string();
        let cell_id = cell_id.to_string();
        // Roles are content's open vocabulary and nothing in the crate reads
        // one yet, so none is invented at this boundary.
        let raised = match state.raise_force(
            &force_id,
            &faction_id,
            &cell_id,
            Default::default(),
            heads,
            &assignment.to_string(),
            &self.geography,
        ) {
            Ok(id) => id,
            Err(error) => return expedition_error_dictionary(force_error_code(&error)),
        };
        let strength = state.forces[raised.as_str()].strength;
        // An order given from outside the hour is journalled by whoever gave
        // it, exactly as S7's departure and S13's machine are.
        let event = crate::strategy::tick::StrategicEvent::ForceRaised {
            force_id: raised,
            faction_id,
            cell_id,
            strength,
            day: state.campaign_day,
        };
        let day = state.campaign_day;
        let hour = state.strategic_clock.hour_of_day;
        state
            .strategic_journal
            .push(JournalEntry::new(day, hour, event.clone()));
        self.expedition_snapshot_with_strategic_events(&[event])
    }

    /// **S18: the player sends it somewhere.** The whole road is planned now,
    /// through `ExpeditionState::dispatch_force` -- so an order that cannot
    /// arrive is refused here rather than accepted and quietly never completed,
    /// and the refusal carries the force module's own error name
    /// (`force_unreachable`, `force_unknown`, `force_unknown_cell`).
    ///
    /// The `ForceDeparted` it produces rides back on the snapshot's `events`
    /// array and is written to S11's journal, so a player's order reads in the
    /// journal like any faction's act. The force then marches on the hours the
    /// campaign already runs; nothing here advances a clock.
    #[func]
    fn dispatch_force(&mut self, force_id: GString, destination_cell_id: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return expedition_error_dictionary("expedition_not_configured");
        };
        let day = state.campaign_day;
        let hour = state.strategic_clock.hour_of_day;
        let departed = match state.dispatch_force(
            &force_id.to_string(),
            &destination_cell_id.to_string(),
            &self.geography,
        ) {
            Ok(event) => event,
            Err(error) => return expedition_error_dictionary(force_error_code(&error)),
        };
        state
            .strategic_journal
            .push(JournalEntry::new(day, hour, departed.clone()));
        self.expedition_snapshot_with_strategic_events(&[departed])
    }

    /// The campaign as `snapshot` gives it, plus the strategic events a verb
    /// just produced, projected through the same `journal_entry_dictionary`
    /// the strategic surface reads -- so an order's report and the journal's
    /// line for it are one projection rather than two.
    fn expedition_snapshot_with_strategic_events(
        &self,
        events: &[crate::strategy::tick::StrategicEvent],
    ) -> VarDictionary {
        let state = self.state.as_ref().expect("the caller just held the state");
        let day = state.campaign_day;
        let hour = state.strategic_clock.hour_of_day;
        let mut projected = Array::<VarDictionary>::new();
        for event in events {
            projected.push(&journal_entry_dictionary(&JournalEntry::new(
                day,
                hour,
                event.clone(),
            )));
        }
        let mut result = expedition_state_dictionary(
            state,
            &self.geography,
            &self.factions,
            &self.buildings,
            &self.machines,
        );
        result.set("events", &projected);
        result
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
    machines: &MachineDefinitions,
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
    // B19: and which authored machine records it is holding, on exactly the
    // same terms. Ids only: a crew requirement or a fuel count crossing here
    // would be the record being read through the wrong door, and what is
    // *standing* on the island is `ExpeditionState::machines` and is not
    // projected here -- a `machines` array that mixed records with instances
    // would be two answers to one key.
    let mut machine_ids = Array::<GString>::new();
    for machine_id in machines.ids() {
        machine_ids.push(&GString::from(machine_id));
    }
    // P4/B12: where S7's offscreen forces are, so the board's route distance can
    // draw them moving between cells instead of inventing motion. Five fields,
    // all of them places and names: the force's own ID, whose it is, the cell it
    // stands in, the cell its next hop reaches (empty while it is not marching)
    // and how far into that hop it has marched, as a fraction the screen can
    // interpolate along a tether.
    //
    // What is deliberately NOT here: strength, readiness, supply, composition,
    // roles, assignment and evidence. Brief section 10 makes a force an
    // aggregate the party learns about through evidence, and a screen that could
    // read an army's strength off the snapshot would start drawing a bar over
    // one nobody has seen. `progress` is a ratio rather than the accrued
    // minutes for the same reason: a miniature needs where along the road it
    // is, not the marching arithmetic.
    let mut forces_projected = Array::<VarDictionary>::new();
    for force in state.forces.values() {
        let next_route = force.route.first().and_then(|id| geography.route(id));
        let progress = match next_route {
            Some(route) if route.time_cost_minutes > 0 => {
                f64::from(force.progress_minutes) / f64::from(route.time_cost_minutes)
            }
            _ => 0.0,
        };
        forces_projected.push(&vdict! {
            "id" => force.id.as_str(),
            "faction_id" => force.faction_id.as_str(),
            "position_cell_id" => force.position_cell_id.as_str(),
            "next_cell_id" => next_route.map(|route| route.to_location_id.as_str()).unwrap_or(""),
            "progress" => progress.clamp(0.0, 1.0),
        });
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
    result.set("forces", &forces_projected);
    result.set("buildings", &building_ids);
    // P3: what the sky is allowed to know. Six read-only keys, every one of
    // them a *name* or a position on a clock the simulation already owns, so
    // the atmosphere is driven by the island and never by the host's clock.
    //
    // `weather` is S8's draw, one word per region -- `WeatherCondition::as_str`
    // and no number, because a screen that could read the draw would start
    // drawing the roll. `active_region_id` rides beside it so Godot never has
    // to guess which region the party is standing in; guessing is how two
    // answers to one question start.
    //
    // `corruption` is S9's band per cell, not the `u8` it was banded from, for
    // the reason `strategy/dungeon.rs` gives: a raw value would regenerate the
    // world's look on every point of accumulation, and a screen that could see
    // the number would be reading a hidden quantity through the wrong door. A
    // cell with no corruption at all is simply absent, exactly as it is absent
    // from `ExpeditionState::corruption`.
    //
    // `heat_band` is the one thing brief section 13 permits about hidden
    // pressure: which of four rooms the world is in, named, and never the
    // number. `cthulhu_heat` itself does not cross this boundary and no key
    // here can be arithmetic'd back into it.
    //
    // `hour_of_day` is S4's strategic clock, and `is_night` is
    // `habitat::is_night` -- the same function the Midnight Return already
    // reads, asked here rather than re-decided in GDScript.
    let mut weather = VarDictionary::new();
    for (region_id, state) in &state.weather {
        weather.set(region_id.as_str(), state.condition.as_str());
    }
    let mut corruption = VarDictionary::new();
    for (cell_id, value) in &state.corruption {
        corruption.set(cell_id.as_str(), CorruptionBand::of(*value).as_str());
    }
    result.set("weather", &weather);
    result.set("corruption", &corruption);
    result.set("heat_band", HeatBand::of(state.cthulhu_heat).as_str());
    result.set("hour_of_day", i64::from(state.strategic_clock.hour_of_day));
    result.set(
        "active_region_id",
        geography
            .locations
            .get(&state.active_location_id)
            .map(|location| location.region_id.as_str())
            .unwrap_or(""),
    );
    result.set("is_night", crate::habitat::is_night(&state.time_segment));
    result.set("machines", &machine_ids);
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

/// What went wrong raising or dispatching a force, named. Exhaustive on
/// purpose, like every other projection in this file: a new `ForceError` cannot
/// be added without this boundary being told what to call it. A malformed ID is
/// deferred to the crate's own `ExpeditionError` name rather than given a
/// second one, because ID shape has one owner.
fn force_error_code(value: &ForceError) -> &'static str {
    match value {
        ForceError::MalformedId(error) => expedition_error_code(error),
        ForceError::IdIsNotAForceId { .. } => "force_id_is_not_a_force_id",
        ForceError::DuplicateForce { .. } => "force_already_exists",
        ForceError::UnknownForce { .. } => "force_unknown",
        ForceError::UnknownCell { .. } => "force_unknown_cell",
        ForceError::Unreachable { .. } => "force_unreachable",
        ForceError::EmptyComposition { .. } => "force_composition_empty",
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
/// A7: an authored site rule the bridge refuses, as one wire code.
fn site_rule_error_code(value: &AuthoredSiteRuleError) -> &'static str {
    match value {
        AuthoredSiteRuleError::MalformedId { .. } => "malformed_site_rule_id",
        AuthoredSiteRuleError::WrongIdPrefix { .. } => "wrong_id_prefix",
        AuthoredSiteRuleError::NoDisplayName { .. } => "site_rule_without_display_name",
        AuthoredSiteRuleError::GuardRegenNotPositive { .. } => "site_rule_guard_regen_not_positive",
        AuthoredSiteRuleError::NeedsDecisionMustBeTrue { .. } => {
            "site_rule_needs_decision_must_be_true"
        }
        AuthoredSiteRuleError::DuplicateSiteRule { .. } => "duplicate_site_rule",
    }
}

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

/// What went wrong loading an authored machine record, named. Exhaustive for
/// the same reason `faction_error_code` and `building_error_code` are: a new
/// `ProductionError` cannot be added without this boundary being told what to
/// call it. Godot reads the name after the `expedition_configuration_invalid:`
/// prefix. `MachineDefinitions::insert` can only raise the first three of
/// these, but the enum is one vocabulary for the whole production lane, so the
/// rest are named here rather than collapsed into a catch-all that would go
/// stale the day `insert` learns another rule.
fn machine_error_code(value: &ProductionError) -> &'static str {
    match value {
        ProductionError::MalformedId(error) => expedition_error_code(error),
        ProductionError::WrongIdPrefix { .. } => "wrong_id_prefix",
        ProductionError::DuplicateDefinition { .. } => "duplicate_machine_definition",
        ProductionError::Duplicate { .. } => "duplicate_machine",
        ProductionError::UnknownDefinition { .. } => "unknown_machine_definition",
        ProductionError::MichaelActorKitNamesANonMachine { .. } => {
            "michael_actor_kit_names_a_non_machine"
        }
        ProductionError::UnknownBuilding { .. } => "unknown_building",
        ProductionError::NoSuchRule { .. } => "no_such_production_rule",
        ProductionError::NotOperational { .. } => "building_not_operational",
        ProductionError::BelowMinimumTier { .. } => "below_minimum_tier",
        ProductionError::NotAMachineRule { .. } => "not_a_machine_rule",
        ProductionError::FamilyMismatch { .. } => "machine_family_mismatch",
        ProductionError::UnknownFaction { .. } => "unknown_faction",
        ProductionError::InsufficientResource { .. } => "insufficient_resource",
        ProductionError::Building(error) => building_error_code(error),
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
        // A7: Ayla's seven, on the wire. Every kind here is bound by at least
        // one authored `animation.eventBindings` entry in
        // `content/skills/ayla.*.json`, and `tools/src/validate.mjs` refuses a
        // binding to any kind this match does not name.
        BattleEvent::TargetInspected {
            command_id: id,
            actor_id,
            target_id,
            guard_revealed,
            counter_tag,
        } => {
            kind = "target_inspected";
            command!(id);
            subject!(actor_id);
            subject!(target_id);
            payload.set("guard_revealed", i64::from(guard_revealed));
            payload.set("counter_tag", counter_tag);
        }
        BattleEvent::StatusApplied {
            command_id: id,
            actor_id,
            status_id,
            status_kind,
            source_id,
        } => {
            kind = "status_applied";
            command!(id);
            subject!(actor_id);
            subject!(source_id);
            payload.set("status_id", status_id);
            payload.set("status_kind", status_name(&status_kind));
        }
        BattleEvent::WardLinePlaced {
            command_id: id,
            actor_id,
            band,
        } => {
            kind = "ward_line_placed";
            command!(id);
            subject!(actor_id);
            payload.set("band", i64::from(band));
        }
        BattleEvent::WardLineTriggered {
            command_id: id,
            attacker_id,
            protected_id,
        } => {
            kind = "ward_line_triggered";
            command!(id);
            subject!(attacker_id);
            subject!(protected_id);
        }
        BattleEvent::ActivationDenied {
            command_id: id,
            actor_id,
            target_id,
        } => {
            kind = "activation_denied";
            command!(id);
            subject!(actor_id);
            subject!(target_id);
        }
        BattleEvent::SiteRuleOverridden {
            command_id: id,
            actor_id,
            rule_id,
        } => {
            kind = "site_rule_overridden";
            command!(id);
            subject!(actor_id);
            payload.set("rule_id", rule_id);
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
        StatusKind::Staggered => "staggered",
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

/// One standing directive, as the confirmation screen and the directive list
/// read it. Names, words and the player's own stated limits -- never a score.
fn directive_dictionary(
    state: &ExpeditionState,
    directive: &StrategicDirective,
    geography: &Geography,
) -> VarDictionary {
    let explanation = directive
        .explanation
        .clone()
        .unwrap_or_else(|| state.explain_directive(directive, geography));
    let mut result = vdict! {
        "id" => directive.id.as_str(),
        "faction_id" => directive.faction_id.as_str(),
        "issuing_character_id" => directive.issuing_character_id.as_str(),
        "intent" => serde_name(&directive.intent),
        "priority" => serde_name(&directive.priority),
        "state" => serde_name(&directive.state),
        "reserve_policy" => serde_name(&directive.reserve_policy),
        "target_node_id" => directive.target_node_id.as_deref().unwrap_or(""),
        "target_region_id" => directive.target_region_id.as_deref().unwrap_or(""),
        "target_faction_id" => directive.target_faction_id.as_deref().unwrap_or(""),
        "acceptable_risk" => i64::from(directive.acceptable_risk),
        "completion_condition" => directive.completion_condition.as_str(),
        "withdrawal_condition" => directive.withdrawal_condition.as_str(),
    };
    result.set("explanation", &explanation_dictionary(&explanation));
    result
}

/// S6's explanation, field for field. The screen labels these; it does not
/// write them.
fn explanation_dictionary(explanation: &Explanation) -> VarDictionary {
    let mut blockers = Array::<GString>::new();
    for blocker in &explanation.blockers {
        blockers.push(&GString::from(blocker.as_str()));
    }
    let mut result = vdict! {
        "goal" => explanation.goal.as_str(),
        "why_target" => explanation.why_target.as_str(),
        "resources" => explanation.resources.as_str(),
        "withdrawal_conditions" => explanation.withdrawal_conditions.as_str(),
        "party_could_help" => explanation.party_could_help,
    };
    result.set("blockers", &blockers);
    result
}

/// One journal line: when it happened, what kind of thing it was, the ids it is
/// about, and one sentence of prose.
///
/// `kind` comes from serde rather than from a second list of names written
/// here, so the word the journal shows is the word the save carries. `cell_id`
/// is normalised to *the cell the event is about* -- where a force ended up,
/// not where it started -- because that is the one a map or an alert would
/// point at. `character_ids` is always empty today: no `StrategicEvent` names a
/// person yet, and the key is present so the surface's shape does not change on
/// the day one does.
fn journal_entry_dictionary(entry: &JournalEntry) -> VarDictionary {
    use crate::strategy::tick::StrategicEvent::*;
    let (faction_id, cell_id, route_id, force_id): (String, String, String, String) =
        match &entry.event {
            HourPassed { .. } => (String::new(), String::new(), String::new(), String::new()),
            ForceDeparted {
                force_id,
                faction_id,
                destination_cell_id,
                ..
            } => (
                faction_id.clone(),
                destination_cell_id.clone(),
                String::new(),
                force_id.as_str().to_owned(),
            ),
            ForceMoved {
                force_id,
                faction_id,
                route_id,
                to_cell_id,
                ..
            } => (
                faction_id.clone(),
                to_cell_id.clone(),
                route_id.clone(),
                force_id.as_str().to_owned(),
            ),
            ForceArrived {
                force_id,
                faction_id,
                cell_id,
                ..
            } => (
                faction_id.clone(),
                cell_id.clone(),
                String::new(),
                force_id.as_str().to_owned(),
            ),
            ForceHalted {
                force_id,
                faction_id,
                cell_id,
                ..
            } => (
                faction_id.clone(),
                cell_id.clone(),
                String::new(),
                force_id.as_str().to_owned(),
            ),
            RecoveryLinkLost { faction_id, .. } => (
                faction_id.clone(),
                String::new(),
                String::new(),
                String::new(),
            ),
            FactionEliminated { faction_id, .. } => (
                faction_id.clone(),
                String::new(),
                String::new(),
                String::new(),
            ),
            MachineProduced { faction_id, .. }
            | ProductionYielded { faction_id, .. }
            | ProductionSkipped { faction_id, .. }
            | Gathered { faction_id, .. }
            | ActionSkipped { faction_id, .. } => (
                faction_id.clone(),
                String::new(),
                String::new(),
                String::new(),
            ),
            BuildingStarted {
                faction_id,
                cell_id,
                ..
            } => (
                faction_id.clone(),
                cell_id.clone(),
                String::new(),
                String::new(),
            ),
            ControlTaken {
                force_id,
                faction_id,
                cell_id,
                ..
            }
            | ArrivalContested {
                force_id,
                faction_id,
                cell_id,
                ..
            }
            | ForceRaised {
                force_id,
                faction_id,
                cell_id,
                ..
            } => (
                faction_id.clone(),
                cell_id.clone(),
                String::new(),
                force_id.as_str().to_owned(),
            ),
        };
    let character_ids = Array::<GString>::new();
    let mut result = vdict! {
        "day" => i64::from(entry.day),
        "hour" => i64::from(entry.hour),
        "kind" => serde_variant_name(&entry.event),
        "faction_id" => faction_id.as_str(),
        "cell_id" => cell_id.as_str(),
        "route_id" => route_id.as_str(),
        "force_id" => force_id.as_str(),
        "prose" => journal_prose(entry).as_str(),
    };
    result.set("character_ids", &character_ids);
    result
}

/// One sentence for one thing the island did.
///
/// The match is exhaustive on purpose, exactly as `world_event_dictionary`'s
/// is: a new `StrategicEvent` cannot be added without this boundary being told
/// how to say it, because the crate will not compile until it is. Ids are
/// quoted rather than prettified -- they are concept keys and stable IDs, and
/// the surface draws them in the mono face for that reason.
fn journal_prose(entry: &JournalEntry) -> String {
    use crate::strategy::tick::StrategicEvent::*;
    match &entry.event {
        HourPassed { hour, .. } => format!("Hour {hour} passed on the island."),
        ForceDeparted {
            faction_id,
            from_cell_id,
            destination_cell_id,
            steps,
            ..
        } => format!(
            "{faction_id} sent a force out of {from_cell_id} for {destination_cell_id}, {steps} roads away."
        ),
        ForceMoved {
            faction_id,
            route_id,
            to_cell_id,
            ..
        } => format!("A force of {faction_id} took {route_id} into {to_cell_id}."),
        ForceArrived {
            faction_id,
            cell_id,
            ..
        } => format!("A force of {faction_id} reached {cell_id}."),
        ForceHalted {
            faction_id,
            cell_id,
            reason,
            ..
        } => format!(
            "A force of {faction_id} stopped at {cell_id}: {}.",
            serde_name(reason)
        ),
        RecoveryLinkLost { faction_id, link } => {
            format!("{faction_id} lost a way back: {}.", serde_name(link))
        }
        FactionEliminated { faction_id, day } => {
            format!("{faction_id} is off the board as of day {day}.")
        }
        MachineProduced {
            faction_id,
            family,
            building_instance_id,
            ..
        } => format!(
            "{building_instance_id} turned out a {} for {faction_id}.",
            serde_name(family)
        ),
        ProductionYielded {
            faction_id,
            building_instance_id,
            output_key,
            amount,
            ..
        } => format!("{building_instance_id} yielded {amount} {output_key} for {faction_id}."),
        ProductionSkipped {
            faction_id,
            building_instance_id,
            reason,
            ..
        } => {
            use crate::strategy::production::ProductionSkipReason::*;
            let why = match reason {
                InsufficientResource { key, held, needed } => {
                    format!("it holds {held} {key} and needs {needed}")
                }
                UnknownMachineRecord { def_id } => format!("no machine record {def_id} exists"),
                Refused { detail } => format!("the rule refused: {detail}"),
            };
            format!("{building_instance_id} of {faction_id} skipped its run: {why}.")
        }
        BuildingStarted {
            faction_id,
            building_instance_id,
            def_id,
            cell_id,
            ..
        } => format!("{faction_id} began raising {def_id} at {cell_id} as {building_instance_id}."),
        Gathered {
            faction_id,
            resource_key,
            amount,
            ..
        } => format!("{faction_id} gathered {amount} {resource_key} from the ground it holds."),
        ActionSkipped {
            faction_id,
            goal,
            reason,
            ..
        } => format!(
            "{faction_id} meant to {} and could not: {}.",
            serde_name(goal),
            serde_variant_name(reason)
        ),
        ControlTaken {
            faction_id,
            cell_id,
            from,
            building_instance_ids,
            ..
        } => {
            let standing = match building_instance_ids.len() {
                0 => String::new(),
                1 => " One building there stands as it was.".to_owned(),
                count => format!(" {count} buildings there stand as they were."),
            };
            match from {
                Some(from) => {
                    format!("A force of {faction_id} took {cell_id} from {from}.{standing}")
                }
                None => {
                    format!("A force of {faction_id} took {cell_id}, which nobody held.{standing}")
                }
            }
        }
        ArrivalContested {
            faction_id,
            cell_id,
            held_by,
            ..
        } => format!(
            "A force of {faction_id} reached {cell_id} and found {held_by} standing there; nothing changed hands."
        ),
        ForceRaised {
            faction_id,
            cell_id,
            strength,
            ..
        } => format!("{faction_id} raised a body of {strength} at {cell_id}."),
    }
}

/// The snake_case name serde already gives a unit-like enum value, so the
/// engine sees the word the save carries instead of a second list of names
/// maintained beside it.
fn serde_name<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        _ => String::new(),
    }
}

/// The same idea for an externally tagged enum with struct variants: serde
/// writes `{"force_arrived": {..}}`, and the single key is the variant name.
fn serde_variant_name<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::Object(map)) => map.keys().next().cloned().unwrap_or_default(),
        Ok(serde_json::Value::String(name)) => name,
        _ => String::new(),
    }
}

struct Project42Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Project42Extension {}
