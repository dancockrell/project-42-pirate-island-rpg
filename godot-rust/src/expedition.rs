//! `ExpeditionState`: the single versioned campaign object, per
//! `docs/GAME_BUILD_PLAN.md` section 4.1 ("Single source of truth").
//!
//! Scope note: this module proves the save/load/query round trip that B0 of
//! `docs/CLAUDE_BACKEND_HANDOFF.md` requires -- deterministic serialization, version
//! rejection, and a stable `legal_next_commands` query across a reload. It does not
//! attempt full cross-reference validation of stable IDs against `content/` records;
//! that is the TypeScript validator's job (`docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md`),
//! and this module has no dependency on the content bundle to check it honestly. It only
//! checks structural shape (non-empty, lowercase-dotted).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::geography::{Geography, RouteOption};
use crate::world::DeathMemory;

pub const CURRENT_SAVE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSegment {
    Dawn,
    Day,
    Dusk,
    Midnight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteStep {
    pub location_id: String,
    pub arrived_on_day: u32,
    pub arrived_segment: TimeSegment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupplyState {
    pub rations: u32,
    pub medicine: u32,
    pub coin: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    pub vitality: i32,
    pub max_vitality: i32,
    pub statuses: Vec<String>,
    pub injured: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HabitatState {
    pub last_spawn_day: Option<u32>,
    pub cleared_today: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HouseholdProgress {
    pub estate_upgrades: BTreeSet<String>,
    pub bond_ranks: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounter_id: String,
    pub battle_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpeditionState {
    pub save_version: u32,
    pub campaign_day: u32,
    pub time_segment: TimeSegment,
    pub party_ids: Vec<String>,
    pub active_location_id: String,
    pub route_history: Vec<RouteStep>,
    pub supplies: SupplyState,
    pub character_states: BTreeMap<String, CharacterState>,
    pub named_person_memory: BTreeMap<String, DeathMemory>,
    pub habitat_states: BTreeMap<String, HabitatState>,
    pub discoveries: BTreeSet<String>,
    pub household_progress: HouseholdProgress,
    pub pending_encounter: Option<EncounterState>,
    pub rng_seed: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpeditionError {
    MalformedJson(String),
    FutureSaveVersion { found: u32, current: u32 },
    InvalidStableId { field: &'static str, value: String },
    InvalidPartySize { found: usize },
    IllegalRoute { route_id: String },
    NoPendingEncounter,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TravelOutcome {
    pub arrived_at: String,
    pub time_cost_minutes: u32,
    pub supply_cost: u32,
}

/// B2: how a battle ended, as reported back through `Battle`'s own `BattlePhase`
/// (`Victory` / `Defeat` / `Retreated`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterOutcome {
    Victory,
    Defeat,
    Retreat,
}

impl ExpeditionState {
    /// Deterministic constructor for a fresh campaign (Phase A1: "new game creates
    /// Michael, Betty, a damaged estate, Black Beach as the current location").
    pub fn new(
        seed: u64,
        party_ids: Vec<String>,
        active_location_id: impl Into<String>,
    ) -> Result<Self, ExpeditionError> {
        let active_location_id = active_location_id.into();
        let state = Self {
            save_version: CURRENT_SAVE_VERSION,
            campaign_day: 1,
            time_segment: TimeSegment::Dawn,
            party_ids,
            active_location_id,
            route_history: Vec::new(),
            supplies: SupplyState {
                rations: 0,
                medicine: 0,
                coin: 0,
            },
            character_states: BTreeMap::new(),
            named_person_memory: BTreeMap::new(),
            habitat_states: BTreeMap::new(),
            discoveries: BTreeSet::new(),
            household_progress: HouseholdProgress {
                estate_upgrades: BTreeSet::new(),
                bond_ranks: BTreeMap::new(),
            },
            pending_encounter: None,
            rng_seed: seed,
        };
        state.validate()?;
        Ok(state)
    }

    /// Serializes with `BTreeMap`/`BTreeSet`-backed fields, so equal states always
    /// produce byte-identical JSON regardless of construction order.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ExpeditionState always serializes")
    }

    pub fn from_json(json: &str) -> Result<Self, ExpeditionError> {
        let state: ExpeditionState = serde_json::from_str(json)
            .map_err(|error| ExpeditionError::MalformedJson(error.to_string()))?;
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), ExpeditionError> {
        if self.save_version > CURRENT_SAVE_VERSION {
            return Err(ExpeditionError::FutureSaveVersion {
                found: self.save_version,
                current: CURRENT_SAVE_VERSION,
            });
        }
        if self.party_ids.is_empty() || self.party_ids.len() > 4 {
            return Err(ExpeditionError::InvalidPartySize {
                found: self.party_ids.len(),
            });
        }
        for id in &self.party_ids {
            require_stable_id("party_ids", id)?;
        }
        require_stable_id("active_location_id", &self.active_location_id)?;
        Ok(())
    }

    /// The narrow query B0's proof requires: legal next commands derived purely from
    /// this state, so a save/reload round trip must yield the same set. Deliberately
    /// minimal -- B1 (location graph) and B2 (encounter integration) extend this with
    /// real travel/target commands once those subsystems exist; this proves the query
    /// is stable across persistence, not that it is complete.
    pub fn legal_next_commands(&self) -> Vec<String> {
        match &self.pending_encounter {
            Some(encounter) => vec![format!("resolve_encounter:{}", encounter.encounter_id)],
            None => vec![format!("depart_location:{}", self.active_location_id)],
        }
    }

    /// B1: the routes departing the current location, per the supplied `Geography`.
    pub fn legal_routes<'a>(&self, geography: &'a Geography) -> Vec<&'a RouteOption> {
        geography.routes_from(&self.active_location_id)
    }

    /// B1's real travel command. Rejects a route that doesn't depart the current
    /// location without mutating state -- the same "reject before mutation"
    /// discipline `battle.rs` uses for illegal commands. On success, applies the
    /// route's time/supply cost, records a `RouteStep`, and updates the active
    /// location.
    pub fn travel(
        &mut self,
        route_id: &str,
        geography: &Geography,
    ) -> Result<TravelOutcome, ExpeditionError> {
        let route = geography
            .route(route_id)
            .filter(|route| route.from_location_id == self.active_location_id)
            .ok_or_else(|| ExpeditionError::IllegalRoute {
                route_id: route_id.to_owned(),
            })?
            .clone();

        self.advance_time(route.time_cost_minutes);
        self.supplies.rations = self.supplies.rations.saturating_sub(route.supply_cost);
        self.route_history.push(RouteStep {
            location_id: route.to_location_id.clone(),
            arrived_on_day: self.campaign_day,
            arrived_segment: self.time_segment.clone(),
        });
        self.active_location_id = route.to_location_id.clone();

        Ok(TravelOutcome {
            arrived_at: self.active_location_id.clone(),
            time_cost_minutes: route.time_cost_minutes,
            supply_cost: route.supply_cost,
        })
    }

    /// B1's observation query. Records every observation at the current location
    /// as a discovery (idempotent -- `discoveries` is a set) and returns their IDs.
    pub fn inspect(&mut self, geography: &Geography) -> Vec<String> {
        let Some(location) = geography.location(&self.active_location_id) else {
            return Vec::new();
        };
        for observation_id in &location.observation_ids {
            self.discoveries.insert(observation_id.clone());
        }
        location.observation_ids.clone()
    }

    /// B2: resolves the current `pending_encounter` against a `Battle` outcome
    /// (Victory / Defeat / Retreat). Errors, without mutating state, if there is no
    /// pending encounter to resolve. A retreat additionally "updates ExpeditionState,
    /// consumes declared time/cost, and retains any declared injuries or discoveries"
    /// per B2's proof: it looks up the route that led into the current location (the
    /// entry immediately before it in `route_history`) and, if the graph still has a
    /// route back along that same path, applies its time/supply cost and returns the
    /// party there. Victory and Defeat only clear the pending encounter -- loot,
    /// injuries and deeper consequences are content the encounter itself declares,
    /// not something this method invents.
    pub fn resolve_encounter(
        &mut self,
        outcome: EncounterOutcome,
        geography: &Geography,
    ) -> Result<(), ExpeditionError> {
        if self.pending_encounter.is_none() {
            return Err(ExpeditionError::NoPendingEncounter);
        }
        if outcome == EncounterOutcome::Retreat {
            let previous_location_id = self
                .route_history
                .iter()
                .rev()
                .nth(1)
                .map(|step| step.location_id.clone());
            if let Some(previous_location_id) = previous_location_id {
                let return_route = self
                    .legal_routes(geography)
                    .into_iter()
                    .find(|route| route.to_location_id == previous_location_id)
                    .cloned();
                if let Some(return_route) = return_route {
                    self.advance_time(return_route.time_cost_minutes);
                    self.supplies.rations = self
                        .supplies
                        .rations
                        .saturating_sub(return_route.supply_cost);
                    self.route_history.push(RouteStep {
                        location_id: return_route.to_location_id.clone(),
                        arrived_on_day: self.campaign_day,
                        arrived_segment: self.time_segment.clone(),
                    });
                    self.active_location_id = return_route.to_location_id;
                }
            }
        }
        self.pending_encounter = None;
        Ok(())
    }

    /// A coarse, deterministic clock: every 360 minutes rolls the campaign one time
    /// segment forward, and a roll past Midnight advances `campaign_day`. Minimal by
    /// design -- B3 (Midnight Return) owns the actual midnight transaction; this only
    /// keeps `campaign_day`/`time_segment` honest for route time costs.
    fn advance_time(&mut self, minutes: u32) {
        const MINUTES_PER_SEGMENT: u32 = 360;
        let mut remaining = minutes;
        while remaining >= MINUTES_PER_SEGMENT {
            remaining -= MINUTES_PER_SEGMENT;
            self.time_segment = match self.time_segment {
                TimeSegment::Dawn => TimeSegment::Day,
                TimeSegment::Day => TimeSegment::Dusk,
                TimeSegment::Dusk => TimeSegment::Midnight,
                TimeSegment::Midnight => {
                    self.campaign_day += 1;
                    TimeSegment::Dawn
                }
            };
        }
    }

    /// The real, geography-aware version of `legal_next_commands`: departing routes
    /// plus observations at the current location, or the pending encounter alone if
    /// one is active. `legal_next_commands` (no geography) stays as the minimal,
    /// geography-independent stub B0 already proved round-trips a save/reload.
    pub fn legal_next_commands_with_geography(&self, geography: &Geography) -> Vec<String> {
        if let Some(encounter) = &self.pending_encounter {
            return vec![format!("resolve_encounter:{}", encounter.encounter_id)];
        }
        let mut commands: Vec<String> = self
            .legal_routes(geography)
            .into_iter()
            .map(|route| format!("travel:{}", route.id))
            .collect();
        if let Some(location) = geography.location(&self.active_location_id) {
            commands.extend(
                location
                    .observation_ids
                    .iter()
                    .map(|id| format!("inspect:{id}")),
            );
        }
        commands
    }
}

fn require_stable_id(field: &'static str, value: &str) -> Result<(), ExpeditionError> {
    let shaped = !value.is_empty()
        && value.contains('.')
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_');
    if shaped {
        Ok(())
    } else {
        Err(ExpeditionError::InvalidStableId {
            field,
            value: value.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ExpeditionState {
        let mut state = ExpeditionState::new(
            42,
            vec![
                "character.protagonist.captain".into(),
                "character.heroine.betty".into(),
            ],
            "location.black_beach",
        )
        .expect("fixture constructs");
        state.named_person_memory.insert(
            "person.villager.tomas".into(),
            DeathMemory {
                killed_by_player_count: 1,
                last_death_day: Some(3),
                last_death_context_id: Some("encounter.prototype.returning_names".into()),
            },
        );
        state
            .discoveries
            .insert("discovery.reception_terrace.elven_marker".into());
        state
    }

    #[test]
    fn round_trip_preserves_every_field() {
        let original = fixture();
        let restored = ExpeditionState::from_json(&original.to_json()).expect("round trip parses");
        assert_eq!(original, restored);
    }

    #[test]
    fn equal_states_serialize_to_identical_bytes() {
        let first = fixture();
        let second = fixture();
        assert_eq!(first.to_json(), second.to_json());
    }

    #[test]
    fn boundary_save_reload_preserves_legal_next_commands() {
        // Boundary: beginning an encounter.
        let mut at_encounter = fixture();
        at_encounter.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        let before = at_encounter.legal_next_commands();
        let restored =
            ExpeditionState::from_json(&at_encounter.to_json()).expect("round trip parses");
        assert_eq!(before, restored.legal_next_commands());

        // Boundary: entering a location (no pending encounter).
        let mut at_location = fixture();
        at_location.active_location_id = "world.cell.reception_terrace".into();
        let before = at_location.legal_next_commands();
        let restored =
            ExpeditionState::from_json(&at_location.to_json()).expect("round trip parses");
        assert_eq!(before, restored.legal_next_commands());
    }

    #[test]
    fn rejects_a_save_version_from_the_future() {
        let mut state = fixture();
        state.save_version = CURRENT_SAVE_VERSION + 1;
        let error = ExpeditionState::from_json(&state.to_json()).unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::FutureSaveVersion {
                found: CURRENT_SAVE_VERSION + 1,
                current: CURRENT_SAVE_VERSION
            }
        );
    }

    #[test]
    fn rejects_malformed_json() {
        let error = ExpeditionState::from_json("{ not json").unwrap_err();
        assert!(matches!(error, ExpeditionError::MalformedJson(_)));
    }

    #[test]
    fn rejects_a_structurally_invalid_stable_id() {
        let mut state = fixture();
        state.active_location_id = "Black Beach".into();
        let error = ExpeditionState::from_json(&state.to_json()).unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::InvalidStableId {
                field: "active_location_id",
                value: "Black Beach".into()
            }
        );
    }

    #[test]
    fn rejects_an_empty_or_oversized_party() {
        assert_eq!(
            ExpeditionState::new(1, vec![], "world.cell.black_beach").unwrap_err(),
            ExpeditionError::InvalidPartySize { found: 0 }
        );
        let five = vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
            "character.heroine.ayla".into(),
            "character.heroine.vix".into(),
            "character.heroine.grisha".into(),
        ];
        assert_eq!(
            ExpeditionState::new(1, five, "world.cell.black_beach").unwrap_err(),
            ExpeditionError::InvalidPartySize { found: 5 }
        );
    }

    #[test]
    fn inspect_records_the_current_locations_observations_as_discoveries() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let observed = state.inspect(&geography);
        assert_eq!(
            observed,
            vec!["observation.black_beach.wreck_of_handsome_jack".to_owned()]
        );
        assert!(
            state
                .discoveries
                .contains("observation.black_beach.wreck_of_handsome_jack")
        );
    }

    #[test]
    fn safe_road_and_jungle_edge_apply_distinct_time_and_supply_consequences() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();

        let mut via_safe_road = fixture();
        via_safe_road.supplies.rations = 10;
        via_safe_road
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        let safe_outcome = via_safe_road
            .travel("route.river_landing.safe_road", &geography)
            .expect("legal route");

        let mut via_jungle_edge = fixture();
        via_jungle_edge.supplies.rations = 10;
        via_jungle_edge
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        let jungle_outcome = via_jungle_edge
            .travel("route.river_landing.jungle_edge", &geography)
            .expect("legal route");

        assert_eq!(
            via_safe_road.active_location_id,
            "location.black_beach.reception_terrace"
        );
        assert_eq!(
            via_jungle_edge.active_location_id,
            "location.black_beach.reception_terrace"
        );
        assert_ne!(
            safe_outcome.time_cost_minutes,
            jungle_outcome.time_cost_minutes
        );
        assert_ne!(safe_outcome.supply_cost, jungle_outcome.supply_cost);
        assert_ne!(
            via_safe_road.supplies.rations,
            via_jungle_edge.supplies.rations
        );
        assert_eq!(via_safe_road.route_history.len(), 2);
        assert_eq!(via_jungle_edge.route_history.len(), 2);
    }

    #[test]
    fn reception_terrace_is_encounter_eligible_and_black_beach_is_not() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        assert!(
            !geography
                .location("location.black_beach")
                .unwrap()
                .encounter_eligible
        );
        assert!(
            geography
                .location("location.black_beach.reception_terrace")
                .unwrap()
                .encounter_eligible
        );
    }

    #[test]
    fn travel_rejects_a_route_not_departing_the_current_location_without_mutation() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let before = state.clone();
        let error = state
            .travel("route.river_landing.safe_road", &geography)
            .expect_err("Black Beach cannot use a river-landing route directly");
        assert_eq!(
            error,
            ExpeditionError::IllegalRoute {
                route_id: "route.river_landing.safe_road".into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn travel_rejects_an_unknown_route_id_without_mutation() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let before = state.clone();
        let error = state
            .travel("route.does_not_exist", &geography)
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::IllegalRoute {
                route_id: "route.does_not_exist".into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn boundary_save_reload_preserves_legal_next_commands_after_a_route_choice() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        state
            .travel("route.river_landing.jungle_edge", &geography)
            .expect("legal route");

        let before = state.legal_next_commands_with_geography(&geography);
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(
            before,
            restored.legal_next_commands_with_geography(&geography)
        );
        assert!(before.iter().any(|command| command.starts_with("travel:")));
        assert!(before.iter().any(|command| command.starts_with("inspect:")));
    }

    #[test]
    fn resolve_encounter_rejects_when_nothing_is_pending() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let error = state
            .resolve_encounter(EncounterOutcome::Victory, &geography)
            .unwrap_err();
        assert_eq!(error, ExpeditionError::NoPendingEncounter);
    }

    #[test]
    fn victory_and_defeat_only_clear_the_pending_encounter() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        for outcome in [EncounterOutcome::Victory, EncounterOutcome::Defeat] {
            let mut state = fixture();
            state.pending_encounter = Some(EncounterState {
                encounter_id: "encounter.prototype.returning_names".into(),
                battle_id: "battle.prototype.returning_names".into(),
            });
            let location_before = state.active_location_id.clone();
            state
                .resolve_encounter(outcome, &geography)
                .expect("resolves");
            assert!(state.pending_encounter.is_none());
            assert_eq!(state.active_location_id, location_before);
        }
    }

    #[test]
    fn retreat_returns_to_the_previous_location_and_consumes_its_route_cost() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.supplies.rations = 10;
        state
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        state
            .travel("route.river_landing.safe_road", &geography)
            .expect("legal route");
        assert_eq!(
            state.active_location_id,
            "location.black_beach.reception_terrace"
        );

        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        let rations_before_retreat = state.supplies.rations;
        state
            .resolve_encounter(EncounterOutcome::Retreat, &geography)
            .expect("resolves");

        assert!(state.pending_encounter.is_none());
        assert_eq!(
            state.active_location_id,
            "location.black_beach.river_landing"
        );
        assert!(state.supplies.rations < rations_before_retreat);
        assert_eq!(
            state.route_history.last().unwrap().location_id,
            "location.black_beach.river_landing"
        );
    }
}
