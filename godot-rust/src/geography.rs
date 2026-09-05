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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnPolicy {
    #[default]
    CanRetreatToPrevious,
    NoRetreat,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistencePolicy {
    #[default]
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

/// What a "do something here" anchor actually does. One vocabulary for every
/// such verb -- salvage, loot caches, and (A4) the estate's rooms -- so the
/// game never grows a second, parallel "interact with this place" mechanism.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorKind {
    /// Working a wreck or a tide line for what it still holds. The declared
    /// rations are the floor; the actual yield varies deterministically by day.
    Salvage { rations: u32, coin: u32 },
    /// A cache someone else left. Fixed contents: a cache is a known quantity.
    LootCache {
        rations: u32,
        medicine: u32,
        coin: u32,
    },
    /// Treating the party's injuries. A4 moves the estate's infirmary rule here.
    Infirmary,
    /// Fitting recovered parts into equipment. A4 gives it its upgrade.
    Workshop,
    /// Reading the map for a route nobody has walked yet. A4 gives it its
    /// discovery.
    MapTable,
    /// Looking closely at what is already here, recording what the place says.
    Inspect,
}

/// One anchor as content declares it. It lives on the cell that carries it;
/// `Geography.anchors` holds the definition and the cell's
/// `interaction_anchor_ids` names it, so "is this anchor here?" is a question
/// about the graph rather than about a second registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorDefinition {
    pub id: String,
    pub kind: AnchorKind,
    /// Whether using it exhausts it until the next campaign day.
    #[serde(default)]
    pub once_per_day: bool,
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
    /// The actionable anchors this cell declares. `from_authored` registers
    /// each one and appends its ID to `interaction_anchor_ids`, so an anchor
    /// is never listed in one place and defined in another.
    #[serde(default)]
    pub anchors: Vec<AnchorDefinition>,
    /// Whether leaving the way you came is legal here. Defaults to the
    /// permissive answer, which is what an ordinary outdoor cell wants.
    #[serde(default)]
    pub return_policy: ReturnPolicy,
    /// Whether what the party changed here survives the next midnight.
    /// Ceremonial interiors reset; the coast and the estate do not.
    #[serde(default)]
    pub persistence_policy: PersistencePolicy,
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
    /// Every actionable anchor in the world, keyed by its own stable ID. Which
    /// cell an anchor stands in is answered by that cell's
    /// `interaction_anchor_ids`.
    pub anchors: BTreeMap<String, AnchorDefinition>,
}

impl Geography {
    /// Deterministic fixture for the first chapter's exact required graph, on the
    /// same IDs `content/world/*.world_cell.json` authors:
    /// `world.cell.black_beach -> damaged_estate -> river_landing ->
    /// reception_terrace` (by `safe_road` or `jungle_edge`, distinct cost and
    /// risk) `-> processional_ramp`, and past the ramp D3's tomb. The estate is
    /// the hinge inland rather than a side trip off the beach, which is the
    /// authored map and the brief's account of how Michael gets off the sand.
    ///
    /// It is built through `from_authored`, the one constructor, so the fixture
    /// and the bridge cannot disagree about what a graph is;
    /// `fixture_matches_the_authored_world_cells` holds it equal to the content.
    pub fn black_beach_vertical_slice() -> Self {
        // `encounter_eligible` follows each authored cell's `battleEntries[].status`,
        // not merely whether `battleEntries` is non-empty. Four of the five cells
        // declare a `future_spawn_socket` -- a reserved place for an encounter that
        // does not exist yet -- and only `world.cell.reception_terrace` declares a
        // live `vertical_slice_encounter`. Keying on presence would put a fight on
        // the whole coast today. The sockets become eligible when their entries do.
        let cell = |id: &str,
                    display_name: &str,
                    observation_ids: &[&str],
                    interaction_anchor_ids: &[&str],
                    persistence_policy: PersistencePolicy,
                    encounter_eligible: bool| CellDefinition {
            id: id.into(),
            region_id: "world.region.black_beach".into(),
            display_name: display_name.into(),
            observation_ids: observation_ids.iter().map(|id| (*id).to_owned()).collect(),
            interaction_anchor_ids: interaction_anchor_ids
                .iter()
                .map(|id| (*id).to_owned())
                .collect(),
            anchors: Vec::new(),
            return_policy: ReturnPolicy::CanRetreatToPrevious,
            persistence_policy,
            encounter_eligible,
        };
        let cells = vec![
            // The wreck of the Handsome Jack is the party's first and only
            // source of supply: the beach is where the economy starts, so the
            // salvage anchor stands here rather than at the estate. A3 gives it
            // a floor of four rations and two coin; the day seed decides whether
            // a given day's work turns up a fifth ration.
            CellDefinition {
                anchors: vec![AnchorDefinition {
                    id: "anchor.black_beach.salvage_point".into(),
                    kind: AnchorKind::Salvage {
                        rations: 4,
                        coin: 2,
                    },
                    once_per_day: true,
                }],
                ..cell(
                    "world.cell.black_beach",
                    "Black Beach",
                    &[
                        "observation.black_beach.wreck",
                        "observation.black_beach.boiler",
                    ],
                    &[
                        "interact.black_beach.boiler_wreck",
                        "interact.black_beach.estate_climb",
                    ],
                    PersistencePolicy::PersistsAcrossVisits,
                    false,
                )
            },
            cell(
                "world.cell.damaged_estate",
                "Damaged Coastal Estate",
                &[
                    "observation.damaged_estate.veranda",
                    "observation.damaged_estate.river_gate",
                ],
                &[
                    "interact.damaged_estate.infirmary_table",
                    "interact.damaged_estate.river_gate",
                ],
                PersistencePolicy::PersistsAcrossVisits,
                false,
            ),
            cell(
                "world.cell.river_landing",
                "River Landing",
                &[
                    "observation.river_landing.road_marker",
                    "observation.river_landing.waterline",
                ],
                &[
                    "interact.river_landing.road_marker",
                    "interact.river_landing.route_choice",
                ],
                PersistencePolicy::PersistsAcrossVisits,
                false,
            ),
            cell(
                "world.cell.reception_terrace",
                "Reception Terrace",
                &[
                    "observation.reception_terrace.arrival",
                    "observation.reception_terrace.battle_lane",
                ],
                &[
                    "interact.reception_terrace.claw_marks",
                    "interact.reception_terrace.processional_ramp",
                ],
                PersistencePolicy::PersistsAcrossVisits,
                true,
            ),
            cell(
                "world.cell.processional_ramp",
                "Processional Ramp",
                &[
                    "observation.processional_ramp.plaque",
                    "observation.processional_ramp.threshold",
                ],
                &[
                    "interact.processional_ramp.name_plaque",
                    "interact.processional_ramp.tomb_threshold",
                ],
                PersistencePolicy::ResetsOnMidnight,
                false,
            ),
            // D3: the Tomb of Returning Names, past the ceremonial approach the
            // Processional Ramp already establishes. Elven public-purpose plan:
            // threshold -> reception/truth space -> (gated) archive core, with an
            // ungated service passage as the recoverable failure branch. These four
            // cells are fixture-only until B6 authors them under `content/world/`,
            // which is why `fixture_matches_the_authored_world_cells` skips them.
            cell(
                "world.cell.tomb_threshold",
                "Threshold of Returning Names",
                &["observation.tomb_threshold.sealed_names"],
                &[],
                PersistencePolicy::ResetsOnMidnight,
                false,
            ),
            cell(
                "world.cell.tomb_reception",
                "Reception of Returning Names",
                &["observation.tomb_reception.true_name"],
                &[],
                PersistencePolicy::ResetsOnMidnight,
                false,
            ),
            cell(
                "world.cell.tomb_archive_core",
                "Archive Core of Returning Names",
                &["observation.tomb_archive_core.the_returning_names"],
                &["interact.tomb_archive_core.name_ledger"],
                PersistencePolicy::ResetsOnMidnight,
                false,
            ),
            // The wrong turn pays for itself: the disturbed grave goods the
            // passage already describes are a real cache, which is what makes
            // the ungated branch a recoverable mistake rather than a punishment.
            // The passage resets at midnight, and so does the cache.
            CellDefinition {
                anchors: vec![AnchorDefinition {
                    id: "anchor.tomb_service_passage.disturbed_grave_goods".into(),
                    kind: AnchorKind::LootCache {
                        rations: 1,
                        medicine: 1,
                        coin: 6,
                    },
                    once_per_day: true,
                }],
                ..cell(
                    "world.cell.tomb_service_passage",
                    "Service Passage of Returning Names",
                    &["observation.tomb_service_passage.disturbed_grave_goods"],
                    &[],
                    PersistencePolicy::ResetsOnMidnight,
                    true,
                )
            },
        ];

        let portal = |id: &str,
                      from: &str,
                      to: &str,
                      travel_mode: &str,
                      time_cost_minutes: u32,
                      supply_cost: u32,
                      risk_level: u8| PortalDefinition {
            id: id.into(),
            from_location_id: from.into(),
            target_location_id: to.into(),
            travel_mode: travel_mode.into(),
            time_cost_minutes,
            supply_cost,
            risk_level,
            required_discovery_id: None,
        };
        let portals = vec![
            portal(
                "world.portal.black_beach_to_damaged_estate",
                "world.cell.black_beach",
                "world.cell.damaged_estate",
                "on_foot",
                15,
                0,
                0,
            ),
            portal(
                "world.portal.damaged_estate_to_black_beach",
                "world.cell.damaged_estate",
                "world.cell.black_beach",
                "on_foot",
                15,
                0,
                0,
            ),
            portal(
                "world.portal.damaged_estate_to_river_landing",
                "world.cell.damaged_estate",
                "world.cell.river_landing",
                "on_foot",
                30,
                0,
                1,
            ),
            portal(
                "world.portal.river_landing_to_damaged_estate",
                "world.cell.river_landing",
                "world.cell.damaged_estate",
                "on_foot",
                30,
                0,
                1,
            ),
            // The two river options stay genuinely different, per B1: the road is
            // slower and cheap, the jungle edge is quick, hungry and dangerous.
            portal(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                "world.cell.river_landing",
                "world.cell.reception_terrace",
                "safe_road",
                60,
                2,
                1,
            ),
            portal(
                "world.portal.river_landing_to_reception_terrace_jungle_edge",
                "world.cell.river_landing",
                "world.cell.reception_terrace",
                "jungle_edge",
                35,
                4,
                3,
            ),
            portal(
                "world.portal.reception_terrace_to_river_landing",
                "world.cell.reception_terrace",
                "world.cell.river_landing",
                "on_foot",
                45,
                1,
                2,
            ),
            portal(
                "world.portal.reception_terrace_to_processional_ramp",
                "world.cell.reception_terrace",
                "world.cell.processional_ramp",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.processional_ramp_to_reception_terrace",
                "world.cell.processional_ramp",
                "world.cell.reception_terrace",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.processional_ramp_to_tomb_threshold",
                "world.cell.processional_ramp",
                "world.cell.tomb_threshold",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.tomb_threshold_to_processional_ramp",
                "world.cell.tomb_threshold",
                "world.cell.processional_ramp",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.tomb_threshold_to_tomb_reception",
                "world.cell.tomb_threshold",
                "world.cell.tomb_reception",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.tomb_reception_to_tomb_threshold",
                "world.cell.tomb_reception",
                "world.cell.tomb_threshold",
                "on_foot",
                10,
                0,
                2,
            ),
            // D3's gate: the archive core stays shut until the reception's true
            // name has actually been observed. The service passage below is the
            // ungated wrong turn that makes failing here recoverable.
            PortalDefinition {
                required_discovery_id: Some("observation.tomb_reception.true_name".into()),
                ..portal(
                    "world.portal.tomb_reception_to_tomb_archive_core",
                    "world.cell.tomb_reception",
                    "world.cell.tomb_archive_core",
                    "on_foot",
                    10,
                    0,
                    2,
                )
            },
            portal(
                "world.portal.tomb_archive_core_to_tomb_reception",
                "world.cell.tomb_archive_core",
                "world.cell.tomb_reception",
                "on_foot",
                10,
                0,
                2,
            ),
            portal(
                "world.portal.tomb_reception_to_tomb_service_passage",
                "world.cell.tomb_reception",
                "world.cell.tomb_service_passage",
                "on_foot",
                10,
                0,
                3,
            ),
            portal(
                "world.portal.tomb_service_passage_to_tomb_reception",
                "world.cell.tomb_service_passage",
                "world.cell.tomb_reception",
                "on_foot",
                10,
                0,
                3,
            ),
            portal(
                "world.portal.tomb_service_passage_to_tomb_threshold",
                "world.cell.tomb_service_passage",
                "world.cell.tomb_threshold",
                "on_foot",
                15,
                0,
                2,
            ),
        ];

        Self::from_authored(cells, portals, Vec::new())
            .expect("the fixture's own IDs and portals are well formed")
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
        let mut anchors: BTreeMap<String, AnchorDefinition> = BTreeMap::new();
        for cell in cells {
            require_stable_id("cell.id", &cell.id)?;
            require_stable_id("cell.region_id", &cell.region_id)?;
            // An anchor is registered once and listed once. The cell's own
            // `interaction_anchor_ids` stays the single answer to "what can be
            // done here", so a declared anchor cannot go missing from the list
            // and a listed anchor cannot go missing from the registry.
            let mut interaction_anchor_ids = cell.interaction_anchor_ids;
            for anchor in cell.anchors {
                require_stable_id("anchor.id", &anchor.id)?;
                if anchors.contains_key(&anchor.id) {
                    return Err(ExpeditionError::DuplicateAnchor { id: anchor.id });
                }
                interaction_anchor_ids.push(anchor.id.clone());
                anchors.insert(anchor.id.clone(), anchor);
            }
            locations.insert(
                cell.id.clone(),
                LocationRecord {
                    id: cell.id,
                    region_id: cell.region_id,
                    display_name: cell.display_name,
                    exits: Vec::new(),
                    observation_ids: cell.observation_ids,
                    interaction_anchor_ids,
                    return_policy: cell.return_policy,
                    persistence_policy: cell.persistence_policy,
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
            anchors,
        })
    }

    pub fn location(&self, id: &str) -> Option<&LocationRecord> {
        self.locations.get(id)
    }

    /// The anchor with this ID, wherever it stands.
    pub fn anchor(&self, id: &str) -> Option<&AnchorDefinition> {
        self.anchors.get(id)
    }

    /// The actionable anchors standing at a location, in the order the cell
    /// lists them. A `interaction_anchor_ids` entry with no definition is
    /// presentation-only content (an authored `interact.*` hotspot) and is not
    /// an action, so it is skipped rather than faked.
    pub fn anchors_at(&self, location_id: &str) -> Vec<&AnchorDefinition> {
        self.locations
            .get(location_id)
            .into_iter()
            .flat_map(|location| location.interaction_anchor_ids.iter())
            .filter_map(|id| self.anchors.get(id))
            .collect()
    }

    /// Whether this anchor stands at this location -- the check `use_anchor`
    /// makes before it touches anything.
    pub fn anchor_is_at(&self, anchor_id: &str, location_id: &str) -> bool {
        self.locations.get(location_id).is_some_and(|location| {
            location
                .interaction_anchor_ids
                .iter()
                .any(|id| id == anchor_id)
        }) && self.anchors.contains_key(anchor_id)
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
            .next_step_toward("world.cell.black_beach", "world.cell.damaged_estate")
            .expect("adjacent locations connect");
        assert_eq!(step.id, "world.portal.black_beach_to_damaged_estate");
    }

    #[test]
    fn next_step_toward_a_distant_location_is_the_first_hop_of_a_shortest_path() {
        let geography = Geography::black_beach_vertical_slice();
        let step = geography
            .next_step_toward("world.cell.black_beach", "world.cell.reception_terrace")
            .expect("the terrace is reachable");
        // The authored map puts the estate between the beach and everything
        // inland, so the first hop off the sand is always the climb to the
        // estate; the river fork is a later decision, not this one.
        assert_eq!(step.to_location_id, "world.cell.damaged_estate");
    }

    #[test]
    fn next_step_toward_the_same_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward("world.cell.black_beach", "world.cell.black_beach")
                .is_none()
        );
    }

    #[test]
    fn next_step_toward_an_unreachable_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward("world.cell.black_beach", "world.cell.nowhere")
                .is_none()
        );
    }

    #[test]
    fn step_distance_matches_the_actual_shortest_path_length() {
        let geography = Geography::black_beach_vertical_slice();
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.black_beach"),
            Some(0)
        );
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.damaged_estate"),
            Some(1)
        );
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.river_landing"),
            Some(2)
        );
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.reception_terrace"),
            Some(3)
        );
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.processional_ramp"),
            Some(4)
        );
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.nowhere"),
            None
        );
    }

    #[test]
    fn following_next_step_toward_repeatedly_actually_arrives() {
        let geography = Geography::black_beach_vertical_slice();
        let destination = "world.cell.processional_ramp";
        let mut current = "world.cell.black_beach".to_owned();
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
                .step_distance("world.cell.black_beach", destination)
                .unwrap()
        );
    }

    #[test]
    fn the_tomb_archive_core_route_is_gated_on_the_true_name_discovery() {
        let geography = Geography::black_beach_vertical_slice();
        let gated = geography
            .route("world.portal.tomb_reception_to_tomb_archive_core")
            .expect("the gated route exists");
        assert_eq!(
            gated.required_discovery_id.as_deref(),
            Some("observation.tomb_reception.true_name")
        );
        // The gate names an observation the reception actually declares; a gate
        // on a string nothing publishes is a silently unreachable room.
        assert!(
            geography
                .location("world.cell.tomb_reception")
                .expect("the reception exists")
                .observation_ids
                .iter()
                .any(|id| id == "observation.tomb_reception.true_name")
        );
    }

    #[test]
    fn the_service_passage_offers_a_clear_return_path_without_the_discovery() {
        let geography = Geography::black_beach_vertical_slice();
        let ungated = geography
            .route("world.portal.tomb_reception_to_tomb_service_passage")
            .expect("the service passage route exists");
        assert!(ungated.required_discovery_id.is_none());
        let return_route = geography
            .route("world.portal.tomb_service_passage_to_tomb_threshold")
            .expect("the service passage returns toward the threshold");
        assert!(return_route.required_discovery_id.is_none());
    }

    #[test]
    fn the_tomb_interior_is_reachable_past_the_processional_ramp() {
        let geography = Geography::black_beach_vertical_slice();
        assert_eq!(
            geography.step_distance("world.cell.black_beach", "world.cell.tomb_archive_core"),
            Some(7)
        );
    }

    /// A2's anti-drift assertion, and the reason this task existed: the fixture
    /// and `content/world/*.world_cell.json` used to name the same five places
    /// with two different ID sets, so they drifted. `content/world/` is the map
    /// the frontend actually draws, so it is canonical, and this test fails the
    /// moment a cell or a portal exists on one side and not the other.
    #[test]
    fn fixture_matches_the_authored_world_cells() {
        use std::collections::BTreeSet;

        let world_directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/world/");
        let mut authored_cell_ids: BTreeSet<String> = BTreeSet::new();
        let mut authored_portals: Vec<(String, String, String, RouteKind)> = Vec::new();
        for entry in std::fs::read_dir(world_directory).expect("content/world/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".world_cell.json"))
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("the cell file is readable");
            let cell: serde_json::Value =
                serde_json::from_str(&text).expect("the cell file is JSON");
            let cell_id = cell["id"]
                .as_str()
                .expect("an authored cell declares a string id")
                .to_owned();
            for portal in cell["portals"].as_array().into_iter().flatten() {
                authored_portals.push((
                    portal["id"]
                        .as_str()
                        .expect("an authored portal declares a string id")
                        .to_owned(),
                    cell_id.clone(),
                    portal["targetCellId"]
                        .as_str()
                        .expect("an authored portal declares a targetCellId")
                        .to_owned(),
                    RouteKind::from_travel_mode(
                        portal["travelMode"]
                            .as_str()
                            .expect("an authored portal declares a travelMode"),
                    ),
                ));
            }
            authored_cell_ids.insert(cell_id);
        }
        assert!(
            !authored_cell_ids.is_empty(),
            "content/world/ must contain authored world cells for this test to mean anything"
        );

        let geography = Geography::black_beach_vertical_slice();
        // B6 authors the four tomb rooms as world cells. Until it does they exist
        // only in this fixture, and they are the single difference this test
        // tolerates. When B6 lands, delete this filter: the set comparison below
        // then covers the tomb too, with nothing else to change.
        let fixture_cell_ids: BTreeSet<String> = geography
            .all_location_ids()
            .filter(|id| !id.starts_with("world.cell.tomb_"))
            .map(str::to_owned)
            .collect();
        assert_eq!(
            fixture_cell_ids, authored_cell_ids,
            "the fixture's cells and content/world/'s cells must be the same set"
        );

        for (portal_id, from_location_id, to_location_id, kind) in authored_portals {
            let route = geography.route(&portal_id).unwrap_or_else(|| {
                panic!("authored portal {portal_id} is missing from the fixture")
            });
            assert_eq!(
                route.from_location_id, from_location_id,
                "{portal_id} leaves a different cell in the fixture"
            );
            assert_eq!(
                route.to_location_id, to_location_id,
                "{portal_id} arrives somewhere else in the fixture"
            );
            assert_eq!(
                route.kind, kind,
                "{portal_id} has a different travel mode in the fixture"
            );
        }
    }
}
