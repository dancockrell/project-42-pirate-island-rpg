//! Location graph and route logic, per B1 of `docs/CLAUDE_BACKEND_HANDOFF.md`.
//!
//! Scope note: this module owns the *graph and route rules* (what's legal, what it
//! costs, what it changes in `ExpeditionState`) -- not the 3D visual shell. That's
//! `content/world/*.world_cell.json`, marked `implementationOwner: "godot-gdscript-world"`
//! and owned by the frontend track per `docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md`.
//! `Geography::black_beach_vertical_slice()` is a deterministic Rust fixture for the
//! same reason `Battle::prototype_vertical_slice()` is: the backend proof doesn't need
//! to wait on final content authoring.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnPolicy {
    CanRetreatToPrevious,
    NoRetreat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistencePolicy {
    PersistsAcrossVisits,
    ResetsOnMidnight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteKind {
    Direct,
    SafeRoad,
    JungleEdge,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocationRecord {
    pub id: String,
    pub region_id: String,
    pub display_name: String,
    pub exits: Vec<String>,
    pub observation_ids: Vec<String>,
    pub interaction_anchor_ids: Vec<String>,
    pub return_policy: ReturnPolicy,
    pub persistence_policy: PersistencePolicy,
    pub encounter_eligible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteOption {
    pub id: String,
    pub from_location_id: String,
    pub to_location_id: String,
    pub kind: RouteKind,
    pub time_cost_minutes: u32,
    pub supply_cost: u32,
    pub risk_level: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geography {
    pub locations: BTreeMap<String, LocationRecord>,
    pub routes: BTreeMap<String, RouteOption>,
}

impl Geography {
    /// Deterministic fixture for the first chapter's exact required graph:
    /// `location.black_beach -> estate` and `-> river_landing -> reception_terrace`
    /// (via `safe_road` or `jungle_edge`, distinct cost/risk) `-> processional_ramp`.
    pub fn black_beach_vertical_slice() -> Self {
        let mut locations = BTreeMap::new();
        let mut routes = BTreeMap::new();

        locations.insert(
            "location.black_beach".into(),
            LocationRecord {
                id: "location.black_beach".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Black Beach".into(),
                exits: vec![
                    "route.black_beach.to_estate".into(),
                    "route.black_beach.to_river_landing".into(),
                ],
                observation_ids: vec!["observation.black_beach.wreck_of_handsome_jack".into()],
                interaction_anchor_ids: vec!["anchor.black_beach.salvage_point".into()],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::PersistsAcrossVisits,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.black_beach.estate".into(),
            LocationRecord {
                id: "location.black_beach.estate".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Damaged Coastal Estate".into(),
                exits: vec!["route.estate.to_black_beach".into()],
                observation_ids: vec!["observation.estate.storm_damage".into()],
                interaction_anchor_ids: vec![
                    "anchor.estate.workshop".into(),
                    "anchor.estate.infirmary".into(),
                    "anchor.estate.map_table".into(),
                    "anchor.estate.household_room".into(),
                ],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::PersistsAcrossVisits,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.black_beach.river_landing".into(),
            LocationRecord {
                id: "location.black_beach.river_landing".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "River Landing".into(),
                exits: vec![
                    "route.river_landing.safe_road".into(),
                    "route.river_landing.jungle_edge".into(),
                    "route.river_landing.to_black_beach".into(),
                ],
                observation_ids: vec!["observation.river_landing.elven_waymark".into()],
                interaction_anchor_ids: vec!["anchor.river_landing.crossing_point".into()],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::PersistsAcrossVisits,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.black_beach.reception_terrace".into(),
            LocationRecord {
                id: "location.black_beach.reception_terrace".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Reception Terrace".into(),
                exits: vec![
                    "route.reception_terrace.to_river_landing".into(),
                    "route.reception_terrace.to_processional_ramp".into(),
                ],
                observation_ids: vec!["observation.reception_terrace.razorbeak_sign".into()],
                interaction_anchor_ids: vec!["anchor.reception_terrace.loot_point".into()],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::PersistsAcrossVisits,
                encounter_eligible: true,
            },
        );
        locations.insert(
            "location.black_beach.processional_ramp".into(),
            LocationRecord {
                id: "location.black_beach.processional_ramp".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Processional Ramp".into(),
                exits: vec!["route.processional_ramp.to_reception_terrace".into()],
                observation_ids: vec!["observation.processional_ramp.collapsed_arch".into()],
                interaction_anchor_ids: vec![],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: false,
            },
        );

        let route =
            |id: &str, from: &str, to: &str, kind: RouteKind, time: u32, supply: u32, risk: u8| {
                (
                    id.to_owned(),
                    RouteOption {
                        id: id.into(),
                        from_location_id: from.into(),
                        to_location_id: to.into(),
                        kind,
                        time_cost_minutes: time,
                        supply_cost: supply,
                        risk_level: risk,
                    },
                )
            };

        for (id, option) in [
            route(
                "route.black_beach.to_estate",
                "location.black_beach",
                "location.black_beach.estate",
                RouteKind::Direct,
                15,
                0,
                0,
            ),
            route(
                "route.estate.to_black_beach",
                "location.black_beach.estate",
                "location.black_beach",
                RouteKind::Direct,
                15,
                0,
                0,
            ),
            route(
                "route.black_beach.to_river_landing",
                "location.black_beach",
                "location.black_beach.river_landing",
                RouteKind::Direct,
                30,
                0,
                1,
            ),
            route(
                "route.river_landing.to_black_beach",
                "location.black_beach.river_landing",
                "location.black_beach",
                RouteKind::Direct,
                30,
                0,
                1,
            ),
            route(
                "route.river_landing.safe_road",
                "location.black_beach.river_landing",
                "location.black_beach.reception_terrace",
                RouteKind::SafeRoad,
                60,
                2,
                1,
            ),
            route(
                "route.river_landing.jungle_edge",
                "location.black_beach.river_landing",
                "location.black_beach.reception_terrace",
                RouteKind::JungleEdge,
                35,
                4,
                3,
            ),
            route(
                "route.reception_terrace.to_river_landing",
                "location.black_beach.reception_terrace",
                "location.black_beach.river_landing",
                RouteKind::Direct,
                45,
                1,
                2,
            ),
            route(
                "route.reception_terrace.to_processional_ramp",
                "location.black_beach.reception_terrace",
                "location.black_beach.processional_ramp",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            route(
                "route.processional_ramp.to_reception_terrace",
                "location.black_beach.processional_ramp",
                "location.black_beach.reception_terrace",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
        ] {
            routes.insert(id, option);
        }

        Self { locations, routes }
    }

    pub fn location(&self, id: &str) -> Option<&LocationRecord> {
        self.locations.get(id)
    }

    pub fn route(&self, id: &str) -> Option<&RouteOption> {
        self.routes.get(id)
    }

    pub fn routes_from(&self, location_id: &str) -> Vec<&RouteOption> {
        self.routes
            .values()
            .filter(|route| route.from_location_id == location_id)
            .collect()
    }
}
