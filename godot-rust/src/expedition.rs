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

/// A route is authored outside of the simulation, then supplied to it as a
/// small deterministic graph. The simulation owns which portal is legal and
/// what a successful trip changes; Godot owns only presentation and input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalDefinition {
    pub id: String,
    pub from_location_id: String,
    pub target_location_id: String,
    pub travel_mode: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteGraph {
    portals_by_id: BTreeMap<String, PortalDefinition>,
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
    DuplicatePortal { id: String },
    UnknownPortal { id: String },
    PortalUnavailable { portal_id: String, active_location_id: String },
    TravelBlockedByEncounter { encounter_id: String },
}

impl RouteGraph {
    pub fn new(portals: Vec<PortalDefinition>) -> Result<Self, ExpeditionError> {
        let mut portals_by_id = BTreeMap::new();
        for portal in portals {
            require_stable_id("portal.id", &portal.id)?;
            require_stable_id("portal.from_location_id", &portal.from_location_id)?;
            require_stable_id("portal.target_location_id", &portal.target_location_id)?;
            if portals_by_id.contains_key(&portal.id) {
                return Err(ExpeditionError::DuplicatePortal {
                    id: portal.id,
                });
            }
            portals_by_id.insert(portal.id.clone(), portal);
        }
        Ok(Self { portals_by_id })
    }

    pub fn portal(&self, portal_id: &str) -> Option<&PortalDefinition> {
        self.portals_by_id.get(portal_id)
    }

    pub fn portals_from(&self, location_id: &str) -> Vec<&PortalDefinition> {
        self.portals_by_id
            .values()
            .filter(|portal| portal.from_location_id == location_id)
            .collect()
    }
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

    /// Returns stable, player-addressable travel commands for the active cell.
    /// Ordering is by portal ID, never authoring-folder order or UI order.
    pub fn legal_route_commands(&self, graph: &RouteGraph) -> Result<Vec<String>, ExpeditionError> {
        if let Some(encounter) = &self.pending_encounter {
            return Err(ExpeditionError::TravelBlockedByEncounter {
                encounter_id: encounter.encounter_id.clone(),
            });
        }
        Ok(graph
            .portals_from(&self.active_location_id)
            .into_iter()
            .map(|portal| format!("travel:{}", portal.id))
            .collect())
    }

    /// Applies one graph-validated trip. It records an arrival boundary that
    /// survives save/reload and never mutates party resources as a hidden UI
    /// side effect. Route hazards and encounter rolls are later transactions.
    pub fn travel(&mut self, graph: &RouteGraph, portal_id: &str) -> Result<(), ExpeditionError> {
        if let Some(encounter) = &self.pending_encounter {
            return Err(ExpeditionError::TravelBlockedByEncounter {
                encounter_id: encounter.encounter_id.clone(),
            });
        }
        let portal = graph.portal(portal_id).ok_or_else(|| ExpeditionError::UnknownPortal {
            id: portal_id.to_owned(),
        })?;
        if portal.from_location_id != self.active_location_id {
            return Err(ExpeditionError::PortalUnavailable {
                portal_id: portal.id.clone(),
                active_location_id: self.active_location_id.clone(),
            });
        }
        self.active_location_id = portal.target_location_id.clone();
        self.route_history.push(RouteStep {
            location_id: self.active_location_id.clone(),
            arrived_on_day: self.campaign_day,
            arrived_segment: self.time_segment.clone(),
        });
        Ok(())
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
            "world.cell.black_beach",
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

    fn route_fixture() -> RouteGraph {
        RouteGraph::new(vec![
            PortalDefinition {
                id: "world.portal.black_beach_to_damaged_estate".into(),
                from_location_id: "world.cell.black_beach".into(),
                target_location_id: "world.cell.damaged_estate".into(),
                travel_mode: "on_foot".into(),
            },
            PortalDefinition {
                id: "world.portal.damaged_estate_to_river_landing".into(),
                from_location_id: "world.cell.damaged_estate".into(),
                target_location_id: "world.cell.river_landing".into(),
                travel_mode: "on_foot".into(),
            },
            PortalDefinition {
                id: "world.portal.river_landing_to_reception_terrace_safe_road".into(),
                from_location_id: "world.cell.river_landing".into(),
                target_location_id: "world.cell.reception_terrace".into(),
                travel_mode: "safe_road".into(),
            },
            PortalDefinition {
                id: "world.portal.river_landing_to_reception_terrace_jungle_edge".into(),
                from_location_id: "world.cell.river_landing".into(),
                target_location_id: "world.cell.reception_terrace".into(),
                travel_mode: "jungle_edge".into(),
            },
        ])
        .expect("route fixture constructs")
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
    fn route_commands_are_location_scoped_and_ordered() {
        let mut state = fixture();
        let graph = route_fixture();
        assert_eq!(
            state.legal_route_commands(&graph).expect("shore has a route"),
            vec!["travel:world.portal.black_beach_to_damaged_estate"]
        );
        state
            .travel(&graph, "world.portal.black_beach_to_damaged_estate")
            .expect("first trip is legal");
        state
            .travel(&graph, "world.portal.damaged_estate_to_river_landing")
            .expect("second trip is legal");
        assert_eq!(
            state.legal_route_commands(&graph).expect("landing has two routes"),
            vec![
                "travel:world.portal.river_landing_to_reception_terrace_jungle_edge",
                "travel:world.portal.river_landing_to_reception_terrace_safe_road"
            ]
        );
    }

    #[test]
    fn travel_persists_arrival_and_rejects_wrong_portals() {
        let mut state = fixture();
        let graph = route_fixture();
        let error = state
            .travel(&graph, "world.portal.damaged_estate_to_river_landing")
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::PortalUnavailable {
                portal_id: "world.portal.damaged_estate_to_river_landing".into(),
                active_location_id: "world.cell.black_beach".into()
            }
        );
        state
            .travel(&graph, "world.portal.black_beach_to_damaged_estate")
            .expect("first trip is legal");
        let restored = ExpeditionState::from_json(&state.to_json()).expect("arrival saves");
        assert_eq!(restored.active_location_id, "world.cell.damaged_estate");
        assert_eq!(restored.route_history.len(), 1);
        assert_eq!(restored.route_history[0].location_id, "world.cell.damaged_estate");
    }

    #[test]
    fn pending_encounter_blocks_route_commands_and_travel() {
        let mut state = fixture();
        let graph = route_fixture();
        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        assert!(matches!(
            state.legal_route_commands(&graph),
            Err(ExpeditionError::TravelBlockedByEncounter { .. })
        ));
        assert!(matches!(
            state.travel(&graph, "world.portal.black_beach_to_damaged_estate"),
            Err(ExpeditionError::TravelBlockedByEncounter { .. })
        ));
    }
}
