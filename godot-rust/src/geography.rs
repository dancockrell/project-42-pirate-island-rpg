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

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, require_stable_id};

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

impl RouteKind {
    /// The authored `travelMode` vocabulary in `content/world/*.world_cell.json`:
    /// `on_foot`, `safe_road`, `jungle_edge`. Anything else is a plain crossing.
    pub fn from_travel_mode(travel_mode: &str) -> Self {
        match travel_mode {
            "safe_road" => RouteKind::SafeRoad,
            "jungle_edge" => RouteKind::JungleEdge,
            _ => RouteKind::Direct,
        }
    }
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
    /// D3's truth-space gate: this route is illegal until this observation
    /// (an `ExpeditionState::discoveries` entry) has been made. `None` means
    /// the route carries no such gate.
    pub required_discovery_id: Option<String>,
}

impl LocationRecord {
    /// The minimal record for a place a portal reaches but no cell declares.
    /// It exists so the graph is closed; a declared `CellDefinition` for the
    /// same ID replaces every field of it.
    pub fn authored(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            region_id: "world.region.unassigned".into(),
            display_name: id.to_owned(),
            exits: Vec::new(),
            observation_ids: Vec::new(),
            interaction_anchor_ids: Vec::new(),
            return_policy: ReturnPolicy::CanRetreatToPrevious,
            persistence_policy: PersistencePolicy::PersistsAcrossVisits,
            encounter_eligible: false,
        }
    }
}

/// The authored wire format for one cell -- the subset of a
/// `content/world/*.world_cell.json` record the simulation needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellDefinition {
    pub id: String,
    pub region_id: String,
    pub display_name: String,
    #[serde(default)]
    pub observation_ids: Vec<String>,
    #[serde(default)]
    pub interaction_anchor_ids: Vec<String>,
    #[serde(default)]
    pub encounter_eligible: bool,
}

/// The authored wire format for one route, as Godot supplies it through the
/// bridge. Distinct from `RouteOption` on purpose: this is what content
/// declares, `RouteOption` is what the simulation runs on. Costs default to
/// zero so authored data that only names a connection still loads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalDefinition {
    pub id: String,
    pub from_location_id: String,
    pub target_location_id: String,
    pub travel_mode: String,
    #[serde(default)]
    pub time_cost_minutes: u32,
    #[serde(default)]
    pub supply_cost: u32,
    #[serde(default)]
    pub risk_level: u8,
    #[serde(default)]
    pub required_discovery_id: Option<String>,
}

/// One authored location-to-battle boundary. Content decides which entry is
/// live; the expedition state decides when it becomes pending.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterTriggerDefinition {
    pub location_id: String,
    pub encounter_id: String,
    pub battle_id: String,
    #[serde(default)]
    pub estate_upgrade_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geography {
    pub locations: BTreeMap<String, LocationRecord>,
    pub routes: BTreeMap<String, RouteOption>,
    /// Authored one-time encounters, keyed by the location that presents them.
    /// `begin_encounter` consults these before any habitat holder.
    pub encounter_triggers: BTreeMap<String, EncounterTriggerDefinition>,
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
                exits: vec![
                    "route.processional_ramp.to_reception_terrace".into(),
                    "route.processional_ramp.to_tomb_threshold".into(),
                ],
                observation_ids: vec!["observation.processional_ramp.collapsed_arch".into()],
                interaction_anchor_ids: vec![],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: false,
            },
        );
        // D3: the Tomb of Returning Names, past the ceremonial approach the
        // Processional Ramp already establishes. Elven public-purpose plan:
        // threshold -> reception/truth space -> (gated) archive core, with an
        // ungated service passage as the recoverable failure branch.
        locations.insert(
            "location.tomb.returning_names.threshold".into(),
            LocationRecord {
                id: "location.tomb.returning_names.threshold".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Threshold of Returning Names".into(),
                exits: vec![
                    "route.tomb_threshold.to_processional_ramp".into(),
                    "route.tomb_threshold.to_reception".into(),
                ],
                observation_ids: vec!["observation.tomb_threshold.sealed_names".into()],
                interaction_anchor_ids: vec![],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.tomb.returning_names.reception".into(),
            LocationRecord {
                id: "location.tomb.returning_names.reception".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Reception of Returning Names".into(),
                exits: vec![
                    "route.tomb_reception.to_threshold".into(),
                    "route.tomb_reception.to_archive_core".into(),
                    "route.tomb_reception.to_service_passage".into(),
                ],
                observation_ids: vec!["observation.tomb_reception.true_name".into()],
                interaction_anchor_ids: vec![],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.tomb.returning_names.archive_core".into(),
            LocationRecord {
                id: "location.tomb.returning_names.archive_core".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Archive Core of Returning Names".into(),
                exits: vec!["route.tomb_archive_core.to_reception".into()],
                observation_ids: vec!["observation.tomb_archive_core.the_returning_names".into()],
                interaction_anchor_ids: vec!["anchor.tomb_archive_core.name_ledger".into()],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: false,
            },
        );
        locations.insert(
            "location.tomb.returning_names.service_passage".into(),
            LocationRecord {
                id: "location.tomb.returning_names.service_passage".into(),
                region_id: "world.region.black_beach".into(),
                display_name: "Service Passage of Returning Names".into(),
                exits: vec![
                    "route.tomb_service_passage.to_reception".into(),
                    "route.tomb_service_passage.to_threshold".into(),
                ],
                observation_ids: vec![
                    "observation.tomb_service_passage.disturbed_grave_goods".into(),
                ],
                interaction_anchor_ids: vec![],
                return_policy: ReturnPolicy::CanRetreatToPrevious,
                persistence_policy: PersistencePolicy::ResetsOnMidnight,
                encounter_eligible: true,
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
                        required_discovery_id: None,
                    },
                )
            };
        let gated_route = |id: &str,
                           from: &str,
                           to: &str,
                           kind: RouteKind,
                           time: u32,
                           supply: u32,
                           risk: u8,
                           required_discovery_id: &str| {
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
                    required_discovery_id: Some(required_discovery_id.into()),
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
            route(
                "route.processional_ramp.to_tomb_threshold",
                "location.black_beach.processional_ramp",
                "location.tomb.returning_names.threshold",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            route(
                "route.tomb_threshold.to_processional_ramp",
                "location.tomb.returning_names.threshold",
                "location.black_beach.processional_ramp",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            route(
                "route.tomb_threshold.to_reception",
                "location.tomb.returning_names.threshold",
                "location.tomb.returning_names.reception",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            route(
                "route.tomb_reception.to_threshold",
                "location.tomb.returning_names.reception",
                "location.tomb.returning_names.threshold",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            gated_route(
                "route.tomb_reception.to_archive_core",
                "location.tomb.returning_names.reception",
                "location.tomb.returning_names.archive_core",
                RouteKind::Direct,
                10,
                0,
                2,
                "observation.tomb_reception.true_name",
            ),
            route(
                "route.tomb_archive_core.to_reception",
                "location.tomb.returning_names.archive_core",
                "location.tomb.returning_names.reception",
                RouteKind::Direct,
                10,
                0,
                2,
            ),
            route(
                "route.tomb_reception.to_service_passage",
                "location.tomb.returning_names.reception",
                "location.tomb.returning_names.service_passage",
                RouteKind::Direct,
                10,
                0,
                3,
            ),
            route(
                "route.tomb_service_passage.to_reception",
                "location.tomb.returning_names.service_passage",
                "location.tomb.returning_names.reception",
                RouteKind::Direct,
                10,
                0,
                3,
            ),
            route(
                "route.tomb_service_passage.to_threshold",
                "location.tomb.returning_names.service_passage",
                "location.tomb.returning_names.threshold",
                RouteKind::Direct,
                15,
                0,
                2,
            ),
        ] {
            routes.insert(id, option);
        }

        Self {
            locations,
            routes,
            encounter_triggers: BTreeMap::new(),
        }
    }

    /// Builds the graph from authored data instead of the Rust fixture -- the
    /// path Godot uses through the bridge. Declared cells carry their region,
    /// observations and anchors; an endpoint a portal touches but no cell
    /// declares is implied, so the graph is always closed.
    pub fn from_authored(
        cells: Vec<CellDefinition>,
        portals: Vec<PortalDefinition>,
        encounter_triggers: Vec<EncounterTriggerDefinition>,
    ) -> Result<Self, ExpeditionError> {
        let mut routes: BTreeMap<String, RouteOption> = BTreeMap::new();
        let mut locations: BTreeMap<String, LocationRecord> = BTreeMap::new();
        for cell in cells {
            require_stable_id("cell.id", &cell.id)?;
            require_stable_id("cell.region_id", &cell.region_id)?;
            locations.insert(
                cell.id.clone(),
                LocationRecord {
                    id: cell.id,
                    region_id: cell.region_id,
                    display_name: cell.display_name,
                    exits: Vec::new(),
                    observation_ids: cell.observation_ids,
                    interaction_anchor_ids: cell.interaction_anchor_ids,
                    return_policy: ReturnPolicy::CanRetreatToPrevious,
                    persistence_policy: PersistencePolicy::PersistsAcrossVisits,
                    encounter_eligible: cell.encounter_eligible,
                },
            );
        }
        for portal in portals {
            require_stable_id("portal.id", &portal.id)?;
            require_stable_id("portal.from_location_id", &portal.from_location_id)?;
            require_stable_id("portal.target_location_id", &portal.target_location_id)?;
            if routes.contains_key(&portal.id) {
                return Err(ExpeditionError::DuplicatePortal { id: portal.id });
            }
            for location_id in [&portal.from_location_id, &portal.target_location_id] {
                locations
                    .entry(location_id.clone())
                    .or_insert_with(|| LocationRecord::authored(location_id));
            }
            locations
                .get_mut(&portal.from_location_id)
                .expect("just inserted")
                .exits
                .push(portal.id.clone());
            routes.insert(
                portal.id.clone(),
                RouteOption {
                    id: portal.id,
                    from_location_id: portal.from_location_id,
                    to_location_id: portal.target_location_id,
                    kind: RouteKind::from_travel_mode(&portal.travel_mode),
                    time_cost_minutes: portal.time_cost_minutes,
                    supply_cost: portal.supply_cost,
                    risk_level: portal.risk_level,
                    required_discovery_id: portal.required_discovery_id,
                },
            );
        }

        let mut triggers = BTreeMap::new();
        for trigger in encounter_triggers {
            require_stable_id("encounter_trigger.location_id", &trigger.location_id)?;
            require_stable_id("encounter_trigger.encounter_id", &trigger.encounter_id)?;
            require_stable_id("encounter_trigger.battle_id", &trigger.battle_id)?;
            if let Some(estate_upgrade_id) = &trigger.estate_upgrade_id {
                require_stable_id("encounter_trigger.estate_upgrade_id", estate_upgrade_id)?;
            }
            // An authored trigger makes its location present an encounter, so
            // the eligibility flag the habitat path already reads must be true.
            locations
                .entry(trigger.location_id.clone())
                .or_insert_with(|| LocationRecord::authored(&trigger.location_id))
                .encounter_eligible = true;
            if triggers
                .insert(trigger.location_id.clone(), trigger)
                .is_some()
            {
                return Err(ExpeditionError::DuplicateEncounterTrigger);
            }
        }

        Ok(Self {
            locations,
            routes,
            encounter_triggers: triggers,
        })
    }

    pub fn location(&self, id: &str) -> Option<&LocationRecord> {
        self.locations.get(id)
    }

    /// The authored one-time encounter this location presents, if any.
    pub fn encounter_trigger_for(&self, location_id: &str) -> Option<&EncounterTriggerDefinition> {
        self.encounter_triggers.get(location_id)
    }

    /// Every location's stable ID, in stable order.
    pub fn all_location_ids(&self) -> impl Iterator<Item = &str> {
        self.locations.keys().map(String::as_str)
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

    /// The first route of a shortest path from `from` to `to`, or `None` when the
    /// two are the same place or nothing connects them.
    ///
    /// Breadth-first over `routes_from`, counting steps rather than minutes: a
    /// pursuer closes ground in moves, and the shortest-in-time road is not always
    /// the shortest-in-moves one. Both the frontier and the tie-break run in stable
    /// route-ID order, so a chase is reproducible from the same save.
    pub fn next_step_toward(&self, from: &str, to: &str) -> Option<&RouteOption> {
        if from == to {
            return None;
        }
        // Each frontier entry remembers the first route taken to reach it, so the
        // answer falls out of the search rather than needing a second walk back.
        let mut visited: BTreeMap<&str, ()> = BTreeMap::new();
        let mut frontier: Vec<(&str, &RouteOption)> = Vec::new();
        visited.insert(from, ());
        for route in self.routes_from(from) {
            if route.to_location_id == to {
                return Some(route);
            }
            if visited.insert(route.to_location_id.as_str(), ()).is_none() {
                frontier.push((route.to_location_id.as_str(), route));
            }
        }
        while !frontier.is_empty() {
            let mut next_frontier: Vec<(&str, &RouteOption)> = Vec::new();
            for (location, first_step) in &frontier {
                for route in self.routes_from(location) {
                    if route.to_location_id == to {
                        return Some(first_step);
                    }
                    if visited.insert(route.to_location_id.as_str(), ()).is_none() {
                        next_frontier.push((route.to_location_id.as_str(), first_step));
                    }
                }
            }
            frontier = next_frontier;
        }
        None
    }

    /// Steps from `from` to `to` along a shortest path, `None` when unreachable.
    /// Used to place a hunter as far from the party as the graph allows.
    pub fn step_distance(&self, from: &str, to: &str) -> Option<usize> {
        if from == to {
            return Some(0);
        }
        let mut visited: BTreeMap<&str, ()> = BTreeMap::new();
        let mut frontier: Vec<&str> = vec![from];
        visited.insert(from, ());
        let mut steps = 0usize;
        while !frontier.is_empty() {
            steps += 1;
            let mut next_frontier: Vec<&str> = Vec::new();
            for location in &frontier {
                for route in self.routes_from(location) {
                    if route.to_location_id == to {
                        return Some(steps);
                    }
                    if visited.insert(route.to_location_id.as_str(), ()).is_none() {
                        next_frontier.push(route.to_location_id.as_str());
                    }
                }
            }
            frontier = next_frontier;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_step_toward_a_neighbor_is_the_direct_route() {
        let geography = Geography::black_beach_vertical_slice();
        let step = geography
            .next_step_toward("location.black_beach", "location.black_beach.estate")
            .expect("adjacent locations connect");
        assert_eq!(step.id, "route.black_beach.to_estate");
    }

    #[test]
    fn next_step_toward_a_distant_location_is_the_first_hop_of_a_shortest_path() {
        let geography = Geography::black_beach_vertical_slice();
        let step = geography
            .next_step_toward(
                "location.black_beach",
                "location.black_beach.reception_terrace",
            )
            .expect("the terrace is reachable");
        // The first hop out of Black Beach toward the terrace is the river landing
        // leg; either river route from there is a legal second hop, but this call
        // only answers for the first one.
        assert_eq!(step.to_location_id, "location.black_beach.river_landing");
    }

    #[test]
    fn next_step_toward_the_same_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward("location.black_beach", "location.black_beach")
                .is_none()
        );
    }

    #[test]
    fn next_step_toward_an_unreachable_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward("location.black_beach", "location.nowhere")
                .is_none()
        );
    }

    #[test]
    fn step_distance_matches_the_actual_shortest_path_length() {
        let geography = Geography::black_beach_vertical_slice();
        assert_eq!(
            geography.step_distance("location.black_beach", "location.black_beach"),
            Some(0)
        );
        assert_eq!(
            geography.step_distance("location.black_beach", "location.black_beach.estate"),
            Some(1)
        );
        assert_eq!(
            geography.step_distance(
                "location.black_beach",
                "location.black_beach.reception_terrace"
            ),
            Some(2)
        );
        assert_eq!(
            geography.step_distance(
                "location.black_beach",
                "location.black_beach.processional_ramp"
            ),
            Some(3)
        );
        assert_eq!(
            geography.step_distance("location.black_beach", "location.nowhere"),
            None
        );
    }

    #[test]
    fn following_next_step_toward_repeatedly_actually_arrives() {
        let geography = Geography::black_beach_vertical_slice();
        let destination = "location.black_beach.processional_ramp";
        let mut current = "location.black_beach".to_owned();
        let mut hops = 0;
        while current != destination {
            let step = geography
                .next_step_toward(&current, destination)
                .expect("still reachable each hop");
            current = step.to_location_id.clone();
            hops += 1;
            assert!(hops <= 10, "pursuit should not loop forever on this graph");
        }
        assert_eq!(
            hops,
            geography
                .step_distance("location.black_beach", destination)
                .unwrap()
        );
    }

    #[test]
    fn the_tomb_archive_core_route_is_gated_on_the_true_name_discovery() {
        let geography = Geography::black_beach_vertical_slice();
        let gated = geography
            .route("route.tomb_reception.to_archive_core")
            .expect("the gated route exists");
        assert_eq!(
            gated.required_discovery_id.as_deref(),
            Some("observation.tomb_reception.true_name")
        );
    }

    #[test]
    fn the_service_passage_offers_a_clear_return_path_without_the_discovery() {
        let geography = Geography::black_beach_vertical_slice();
        let ungated = geography
            .route("route.tomb_reception.to_service_passage")
            .expect("the service passage route exists");
        assert!(ungated.required_discovery_id.is_none());
        let return_route = geography
            .route("route.tomb_service_passage.to_threshold")
            .expect("the service passage returns toward the threshold");
        assert!(return_route.required_discovery_id.is_none());
    }

    #[test]
    fn the_tomb_interior_is_reachable_past_the_processional_ramp() {
        let geography = Geography::black_beach_vertical_slice();
        assert_eq!(
            geography.step_distance(
                "location.black_beach",
                "location.tomb.returning_names.archive_core"
            ),
            Some(6)
        );
    }
}
