//! Location graph and route logic, per B1 of `docs/CLAUDE_BACKEND_HANDOFF.md`.
//!
//! Scope note: this module owns the *graph and route rules* (what's legal, what it
//! costs, what it changes in `ExpeditionState`) -- not the 3D visual shell. That's
//! `content/world/*.world_cell.json`, marked `implementationOwner: "godot-gdscript-world"`
//! and owned by the frontend track per `docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md`.
//! `Geography::black_beach_vertical_slice()` is a deterministic Rust fixture for the
//! same reason `Battle::prototype_vertical_slice()` is: the backend proof doesn't need
//! to wait on final content authoring.

use std::collections::{BTreeMap, BTreeSet};

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
    /// S2: which faction the *static graph* says holds this place, as a
    /// `faction.<concept_key>` ID. Content authors nothing about ownership yet,
    /// so every authored cell carries `None` and the mutable truth is
    /// [`crate::expedition::ExpeditionState::ownership`], which overrides this.
    /// The field exists so an authored starting map can seed control later
    /// without a second registry appearing to hold it.
    pub owner_faction_id: Option<String>,
    /// S2: standing, per faction, in this place -- the pressure that has not
    /// yet become control. Keyed by `faction.<concept_key>`; empty by default.
    /// Nothing reads it for risk today; `effective_risk` is deliberately a
    /// function of *control*, so influence cannot quietly become a second
    /// answer to "who holds this road".
    pub influence: BTreeMap<String, u16>,
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

/// S2: what a contested road adds to its authored `risk_level`. Two, because
/// the authored slice makes that number mean something exact: the river
/// landing's `safe_road` to the reception terrace is risk 1 and its
/// `jungle_edge` is risk 3, so a contested safe road costs precisely what the
/// jungle costs. The safe way stops being the safe way, which is the whole
/// point of the brief's "a safe road becomes contested" -- rather than a
/// rounding nudge the player never notices.
pub const CONTESTED_RISK_MODIFIER: u8 = 2;

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
            owner_faction_id: None,
            influence: BTreeMap::new(),
        }
    }
}

/// One anchor exactly as `content/world/*.world_cell.json` authors it: a flat
/// record with `kind` as a string and yields as plain fields, which is what a
/// human writes and what Godot forwards verbatim. Accepts both the content
/// spelling (`oncePerDay`) and the wire spelling (`once_per_day`) so the
/// equality test and the bridge read the same struct. `TryFrom` below is the
/// single translation into [`AnchorDefinition`]; nothing else may interpret
/// `kind`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthoredAnchor {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub rations: u32,
    #[serde(default)]
    pub medicine: u32,
    #[serde(default)]
    pub coin: u32,
    #[serde(default, alias = "oncePerDay")]
    pub once_per_day: bool,
    /// Declared by content and held equal to the Rust rule by
    /// `fixture_matches_the_authored_world_cells`; the rule itself lives in
    /// `expedition.rs`, so this is not read at run time.
    #[serde(default, alias = "requiresDiscoveryId")]
    pub requires_discovery_id: Option<String>,
    /// As above. Its stable ID is what a portal's `requiredDiscoveryId` can
    /// legally reference.
    #[serde(default, alias = "grantsDiscoveryId")]
    pub grants_discovery_id: Option<String>,
}

impl TryFrom<AuthoredAnchor> for AnchorDefinition {
    type Error = ExpeditionError;

    fn try_from(authored: AuthoredAnchor) -> Result<Self, Self::Error> {
        let kind = match authored.kind.as_str() {
            "salvage" => AnchorKind::Salvage {
                rations: authored.rations,
                coin: authored.coin,
            },
            "loot_cache" => AnchorKind::LootCache {
                rations: authored.rations,
                medicine: authored.medicine,
                coin: authored.coin,
            },
            "infirmary" => AnchorKind::Infirmary,
            "workshop" => AnchorKind::Workshop,
            "map_table" => AnchorKind::MapTable,
            "inspect" => AnchorKind::Inspect,
            _ => {
                return Err(ExpeditionError::UnknownAnchorKind {
                    anchor_id: authored.id,
                    kind: authored.kind,
                });
            }
        };
        Ok(AnchorDefinition {
            id: authored.id,
            kind,
            once_per_day: authored.once_per_day,
        })
    }
}

/// One cell as Godot forwards it from the content catalog: the authored
/// fields the simulation needs, with anchors in their authored shape.
/// `TryFrom` turns it into a [`CellDefinition`]; the policies default, since
/// content does not author them yet.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthoredCell {
    pub id: String,
    pub region_id: String,
    pub display_name: String,
    #[serde(default)]
    pub observation_ids: Vec<String>,
    #[serde(default)]
    pub anchors: Vec<AuthoredAnchor>,
    #[serde(default)]
    pub encounter_eligible: bool,
}

impl TryFrom<AuthoredCell> for CellDefinition {
    type Error = ExpeditionError;

    fn try_from(authored: AuthoredCell) -> Result<Self, Self::Error> {
        let anchors = authored
            .anchors
            .into_iter()
            .map(AnchorDefinition::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CellDefinition {
            id: authored.id,
            region_id: authored.region_id,
            display_name: authored.display_name,
            observation_ids: authored.observation_ids,
            interaction_anchor_ids: Vec::new(),
            anchors,
            return_policy: ReturnPolicy::default(),
            persistence_policy: PersistencePolicy::default(),
            encounter_eligible: authored.encounter_eligible,
        })
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
            // A4: the estate's rooms are anchors like every other "do something
            // here" verb, so Michael's first strategic core is built out of the
            // mechanism the beach and the tomb already use rather than a second,
            // estate-only command path. Three of the four grant a fact that is
            // permanent once recorded, and that fact is what spends them; only
            // the household room is an ordinary daily look around.
            CellDefinition {
                anchors: vec![
                    AnchorDefinition {
                        id: "anchor.estate.infirmary".into(),
                        kind: AnchorKind::Infirmary,
                        once_per_day: false,
                    },
                    AnchorDefinition {
                        id: "anchor.estate.workshop".into(),
                        kind: AnchorKind::Workshop,
                        once_per_day: false,
                    },
                    AnchorDefinition {
                        id: "anchor.estate.map_table".into(),
                        kind: AnchorKind::MapTable,
                        once_per_day: false,
                    },
                    AnchorDefinition {
                        id: "anchor.estate.household_room".into(),
                        kind: AnchorKind::Inspect,
                        once_per_day: true,
                    },
                ],
                ..cell(
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
                )
            },
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
            // A4: the shortcut the estate's map table reads out of the elven
            // waymark -- a tidal cut along the shore that reaches the terrace
            // without the river at all. It is gated on the map table's own
            // discovery, so it is only a route for a party that has done the
            // work at the estate, and it is the reason the map table is a
            // strategic action rather than a bit of colour.
            PortalDefinition {
                required_discovery_id: Some("discovery.map_table.tidal_cut".into()),
                ..portal(
                    "world.portal.black_beach_to_reception_terrace_tidal_cut",
                    "world.cell.black_beach",
                    "world.cell.reception_terrace",
                    "on_foot",
                    40,
                    1,
                    2,
                )
            },
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
                    // S2: the graph is static content and content authors no
                    // owners, so a cell arrives unheld. `ExpeditionState`
                    // carries who holds it now.
                    owner_faction_id: None,
                    influence: BTreeMap::new(),
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

    /// S2: which faction the static graph says holds this cell. `None` for an
    /// unheld place and for a cell that does not exist -- "nobody holds it" and
    /// "there is no such place" are the same answer to a question about
    /// control, and callers that need the difference ask [`Self::location`].
    ///
    /// This is the *authored* answer only. The live answer is
    /// [`Self::effective_risk`]'s `ownership` argument, which is
    /// [`crate::expedition::ExpeditionState::ownership`] and wins wherever it
    /// names the cell.
    pub fn controller(&self, cell_id: &str) -> Option<&str> {
        self.locations.get(cell_id)?.owner_faction_id.as_deref()
    }

    /// Who holds a cell right now: the campaign's `ownership` override where
    /// it names the cell, else the graph's authored [`Self::controller`].
    /// The one place that composition lives -- `effective_risk` reads it and
    /// the bridge's `controller_of` reads it, so the two can never disagree
    /// about who is standing on a road's far end. B14 had written the same
    /// composition a second time beside the closure this replaced.
    pub fn held_by<'a>(
        &'a self,
        cell_id: &str,
        ownership: &'a BTreeMap<String, String>,
    ) -> Option<&'a str> {
        ownership
            .get(cell_id)
            .map(String::as_str)
            .or_else(|| self.controller(cell_id))
    }

    /// S2, and the one owner of "how dangerous is this road right now".
    ///
    /// The route's authored `risk_level` (C2 authors it on every portal in
    /// `content/world/*.world_cell.json`, and
    /// `fixture_matches_the_authored_world_cells` holds the Rust fixture equal
    /// to it) is the road's *base*. A road whose two endpoints are held by
    /// different parties is contested, and costs [`CONTESTED_RISK_MODIFIER`]
    /// more. Nothing stores the sum: it is recomputed from control every time
    /// it is asked for, so control can change without a single route record
    /// being edited -- which is brief section 1's "a safe road becomes
    /// contested", made mechanical.
    ///
    /// **Contested means the two endpoints' controllers differ**, with "unheld"
    /// counted as a party of its own: `None` against `Some(faction)` is
    /// contested, exactly as `faction.pirates` against `faction.elves` is. Both
    /// readings of the unheld case were defensible; this one is the simpler,
    /// because it is a single comparison rather than a comparison plus a
    /// special case, and it says the plainer thing about the board -- the
    /// frontier of a holding is where the fighting is, and a road running out
    /// of held ground into nobody's ground is exactly that frontier. It also
    /// leaves a fresh campaign, where nothing is held, entirely uncontested.
    ///
    /// `ownership` is [`crate::expedition::ExpeditionState::ownership`]; it
    /// overrides the graph's authored [`Self::controller`] wherever it names a
    /// cell. Deterministic: no draw, no clock, no hidden state.
    pub fn effective_risk(&self, route: &RouteOption, ownership: &BTreeMap<String, String>) -> u8 {
        if self.held_by(&route.from_location_id, ownership)
            == self.held_by(&route.to_location_id, ownership)
        {
            route.risk_level
        } else {
            route.risk_level.saturating_add(CONTESTED_RISK_MODIFIER)
        }
    }

    pub fn routes_from(&self, location_id: &str) -> Vec<&RouteOption> {
        self.routes
            .values()
            .filter(|route| route.from_location_id == location_id)
            .collect()
    }

    /// The routes out of `location_id` that a pursuer may actually take: every
    /// ungated route, plus a gated one whose discovery is in `open_gates`.
    ///
    /// A gate is closed for everyone until the party opens it, and open for
    /// everyone afterward. The party's discoveries are the only openness the
    /// world records, so they are what pursuit consults. The alternative --
    /// letting a hunter take a shortcut the player cannot -- was the state of
    /// things until A4 hung the tidal cut directly between the beach and the
    /// terrace, at which point every chase from the terrace arrived on the sand
    /// in one move through a passage the party had never found.
    fn open_routes_from(
        &self,
        location_id: &str,
        open_gates: &BTreeSet<String>,
    ) -> Vec<&RouteOption> {
        self.routes_from(location_id)
            .into_iter()
            .filter(|route| match &route.required_discovery_id {
                None => true,
                Some(discovery_id) => open_gates.contains(discovery_id),
            })
            .collect()
    }

    /// The first route of a shortest path from `from` to `to`, or `None` when the
    /// two are the same place or nothing connects them through open gates.
    ///
    /// Breadth-first over `open_routes_from`, counting steps rather than minutes:
    /// a pursuer closes ground in moves, and the shortest-in-time road is not
    /// always the shortest-in-moves one. Both the frontier and the tie-break run
    /// in stable route-ID order, so a chase is reproducible from the same save.
    pub fn next_step_toward(
        &self,
        from: &str,
        to: &str,
        open_gates: &BTreeSet<String>,
    ) -> Option<&RouteOption> {
        if from == to {
            return None;
        }
        // Each frontier entry remembers the first route taken to reach it, so the
        // answer falls out of the search rather than needing a second walk back.
        let mut visited: BTreeMap<&str, ()> = BTreeMap::new();
        let mut frontier: Vec<(&str, &RouteOption)> = Vec::new();
        visited.insert(from, ());
        for route in self.open_routes_from(from, open_gates) {
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
                for route in self.open_routes_from(location, open_gates) {
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

    /// Steps from `from` to `to` along a shortest path through open gates, `None`
    /// when unreachable. Used to place a hunter as far from the party as the
    /// graph allows.
    pub fn step_distance(
        &self,
        from: &str,
        to: &str,
        open_gates: &BTreeSet<String>,
    ) -> Option<usize> {
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
                for route in self.open_routes_from(location, open_gates) {
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

    fn no_gates_open() -> BTreeSet<String> {
        BTreeSet::new()
    }

    const SAFE_ROAD: &str = "world.portal.river_landing_to_reception_terrace_safe_road";
    const JUNGLE_EDGE: &str = "world.portal.river_landing_to_reception_terrace_jungle_edge";

    /// S2's Done-when, at the graph's own level: flipping who holds the river
    /// landing changes what the safe road costs, and edits no route record.
    #[test]
    fn taking_the_river_landing_contests_the_safe_road_without_editing_it() {
        let geography = Geography::black_beach_vertical_slice();
        let safe_road = geography.route(SAFE_ROAD).expect("the fixture's safe road");
        let authored_risk = safe_road.risk_level;

        let mut ownership = BTreeMap::new();
        assert_eq!(
            geography.effective_risk(safe_road, &ownership),
            authored_risk,
            "an island nobody holds contests nothing"
        );

        ownership.insert(
            "world.cell.river_landing".to_owned(),
            "faction.pirates".to_owned(),
        );
        assert_eq!(
            geography.effective_risk(safe_road, &ownership),
            authored_risk + CONTESTED_RISK_MODIFIER,
            "one held endpoint against one unheld endpoint is a contested road"
        );
        assert_eq!(
            geography
                .route(SAFE_ROAD)
                .expect("the safe road is still there")
                .risk_level,
            authored_risk,
            "the authored risk on the route record must not have moved"
        );

        // The point of the modifier, stated as the fixture states it: contested,
        // the safe way is worth exactly what the jungle is worth.
        let jungle_edge = geography.route(JUNGLE_EDGE).expect("the fixture's jungle");
        assert_eq!(
            geography.effective_risk(safe_road, &ownership),
            jungle_edge.risk_level
        );

        // Both ends in the same hand, and it is a safe road again -- with, once
        // more, nothing on the record having changed.
        ownership.insert(
            "world.cell.reception_terrace".to_owned(),
            "faction.pirates".to_owned(),
        );
        assert_eq!(
            geography.effective_risk(safe_road, &ownership),
            authored_risk
        );
    }

    #[test]
    fn two_different_holders_contest_the_road_between_them() {
        let geography = Geography::black_beach_vertical_slice();
        let safe_road = geography.route(SAFE_ROAD).expect("the fixture's safe road");
        let ownership = BTreeMap::from([
            (
                "world.cell.river_landing".to_owned(),
                "faction.pirates".to_owned(),
            ),
            (
                "world.cell.reception_terrace".to_owned(),
                "faction.elves".to_owned(),
            ),
        ]);
        assert_eq!(
            geography.effective_risk(safe_road, &ownership),
            safe_road.risk_level + CONTESTED_RISK_MODIFIER
        );
    }

    #[test]
    fn the_authored_graph_holds_no_owners_and_no_influence() {
        let geography = Geography::black_beach_vertical_slice();
        for location in geography.locations.values() {
            assert_eq!(
                location.owner_faction_id, None,
                "{} arrived owned; content authors nothing about ownership yet",
                location.id
            );
            assert!(
                location.influence.is_empty(),
                "{} arrived with influence; content authors none",
                location.id
            );
            assert_eq!(geography.controller(&location.id), None);
        }
        assert_eq!(geography.controller("world.cell.nowhere"), None);
    }

    #[test]
    fn next_step_toward_a_neighbor_is_the_direct_route() {
        let geography = Geography::black_beach_vertical_slice();
        let step = geography
            .next_step_toward(
                "world.cell.black_beach",
                "world.cell.damaged_estate",
                &no_gates_open(),
            )
            .expect("adjacent locations connect");
        assert_eq!(step.id, "world.portal.black_beach_to_damaged_estate");
    }

    #[test]
    fn next_step_toward_a_distant_location_is_the_first_hop_of_a_shortest_path() {
        let geography = Geography::black_beach_vertical_slice();
        let step = geography
            .next_step_toward(
                "world.cell.black_beach",
                "world.cell.reception_terrace",
                &no_gates_open(),
            )
            .expect("the terrace is reachable");
        // The tidal cut hangs directly between the beach and the terrace, but it
        // is gated on the map table's discovery, and nobody has opened it. So a
        // pursuer takes the long way: the estate first, then the river.
        assert_eq!(step.to_location_id, "world.cell.damaged_estate");
    }

    /// The other half of the rule above: once the party has opened a gate, it
    /// is open for pursuit too. A hunter is not held back by a door the party
    /// has already walked through.
    #[test]
    fn an_opened_gate_is_open_for_pursuit_as_well() {
        let geography = Geography::black_beach_vertical_slice();
        let mut open = no_gates_open();
        open.insert("discovery.map_table.tidal_cut".into());
        let step = geography
            .next_step_toward(
                "world.cell.black_beach",
                "world.cell.reception_terrace",
                &open,
            )
            .expect("the terrace is reachable");
        assert_eq!(
            step.id,
            "world.portal.black_beach_to_reception_terrace_tidal_cut"
        );
        assert_eq!(
            geography.step_distance(
                "world.cell.black_beach",
                "world.cell.reception_terrace",
                &open
            ),
            Some(1)
        );
    }

    #[test]
    fn next_step_toward_the_same_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward(
                    "world.cell.black_beach",
                    "world.cell.black_beach",
                    &no_gates_open()
                )
                .is_none()
        );
    }

    #[test]
    fn next_step_toward_an_unreachable_location_is_none() {
        let geography = Geography::black_beach_vertical_slice();
        assert!(
            geography
                .next_step_toward(
                    "world.cell.black_beach",
                    "world.cell.nowhere",
                    &no_gates_open()
                )
                .is_none()
        );
    }

    #[test]
    fn step_distance_matches_the_actual_shortest_path_length() {
        let geography = Geography::black_beach_vertical_slice();
        let open = no_gates_open();
        let distance = |to: &str| geography.step_distance("world.cell.black_beach", to, &open);
        assert_eq!(distance("world.cell.black_beach"), Some(0));
        assert_eq!(distance("world.cell.damaged_estate"), Some(1));
        assert_eq!(distance("world.cell.river_landing"), Some(2));
        // Three by the river. The tidal cut would make it one, but it is gated
        // and closed; `an_opened_gate_is_open_for_pursuit_as_well` covers that.
        assert_eq!(distance("world.cell.reception_terrace"), Some(3));
        assert_eq!(distance("world.cell.processional_ramp"), Some(4));
        assert_eq!(distance("world.cell.nowhere"), None);
    }

    #[test]
    fn following_next_step_toward_repeatedly_actually_arrives() {
        let geography = Geography::black_beach_vertical_slice();
        let destination = "world.cell.processional_ramp";
        let mut current = "world.cell.black_beach".to_owned();
        let mut hops = 0;
        while current != destination {
            let step = geography
                .next_step_toward(&current, destination, &no_gates_open())
                .expect("still reachable each hop");
            current = step.to_location_id.clone();
            hops += 1;
            assert!(hops <= 10, "pursuit should not loop forever on this graph");
        }
        assert_eq!(
            hops,
            geography
                .step_distance("world.cell.black_beach", destination, &no_gates_open())
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
        // The archive core sits behind the true-name gate, and nothing has
        // opened it: the closest a pursuer can get is unreachable. Open it and
        // the core is seven moves from the sand by the river, as it always was.
        assert_eq!(
            geography.step_distance(
                "world.cell.black_beach",
                "world.cell.tomb_archive_core",
                &no_gates_open()
            ),
            None
        );
        let mut open = no_gates_open();
        open.insert("observation.tomb_reception.true_name".into());
        assert_eq!(
            geography.step_distance(
                "world.cell.black_beach",
                "world.cell.tomb_archive_core",
                &open
            ),
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
        let mut authored_observation_ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
        // C13: anchors are authored on the cell that carries them, in the
        // human shape (`kind` as a string, yields as flat fields) rather than
        // serde's externally-tagged enum, and translated here -- the same way
        // `travelMode` is translated to `RouteKind`.
        let mut authored_anchors_by_cell: BTreeMap<String, Vec<AnchorDefinition>> = BTreeMap::new();
        let mut authored_granted_discoveries: BTreeSet<String> = BTreeSet::new();
        let mut authored_map_table: Option<(String, Option<String>, Option<String>)> = None;
        let mut authored_workshop_requirement: Option<String> = None;
        let mut authored_portal_ids_by_cell: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        #[allow(clippy::type_complexity)]
        let mut authored_portals: Vec<(
            String,
            String,
            String,
            RouteKind,
            u32,
            u32,
            u8,
            Option<String>,
        )> = Vec::new();
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
                let portal_id = portal["id"]
                    .as_str()
                    .expect("an authored portal declares a string id")
                    .to_owned();
                // C2 made the three cost fields optional so a connection can be
                // authored before it is priced; an absent one means free, which
                // is exactly what `PortalDefinition`'s `serde(default)` does.
                let cost = |field: &str| portal[field].as_u64().unwrap_or(0);
                authored_portal_ids_by_cell
                    .entry(cell_id.clone())
                    .or_default()
                    .insert(portal_id.clone());
                authored_portals.push((
                    portal_id,
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
                    cost("timeCostMinutes") as u32,
                    cost("supplyCost") as u32,
                    cost("riskLevel") as u8,
                    portal["requiredDiscoveryId"].as_str().map(str::to_owned),
                ));
            }
            let mut anchors_here = Vec::new();
            for anchor in cell["anchors"].as_array().into_iter().flatten() {
                // The same struct and the same translation the bridge uses
                // at run time, so the test cannot pass on a reading of
                // content the game does not share.
                let authored: AuthoredAnchor = serde_json::from_value(anchor.clone())
                    .expect("an authored anchor has the authored shape");
                if let Some(granted) = &authored.grants_discovery_id {
                    authored_granted_discoveries.insert(granted.clone());
                }
                if authored.kind == "map_table" {
                    authored_map_table = Some((
                        authored.id.clone(),
                        authored.requires_discovery_id.clone(),
                        authored.grants_discovery_id.clone(),
                    ));
                }
                if authored.kind == "workshop" {
                    authored_workshop_requirement = authored.requires_discovery_id.clone();
                }
                anchors_here.push(
                    AnchorDefinition::try_from(authored)
                        .expect("content declares only anchor kinds the simulation knows"),
                );
            }
            authored_anchors_by_cell.insert(cell_id.clone(), anchors_here);
            let observation_ids = cell["readableDescriptions"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|description| {
                    description["id"]
                        .as_str()
                        .expect("an authored description declares a string id")
                        .to_owned()
                })
                .collect();
            authored_observation_ids.insert(cell_id.clone(), observation_ids);
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

        // A2 held the two ID sets equal. A4 holds the numbers equal too: a
        // portal's costs and its gate are authored content, and a fixture that
        // carries its own copy of them is the loot-table and skill-rank drift
        // over again, one field further in.
        for (
            portal_id,
            from_location_id,
            to_location_id,
            kind,
            time_cost_minutes,
            supply_cost,
            risk_level,
            required_discovery_id,
        ) in authored_portals
        {
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
            assert_eq!(
                route.time_cost_minutes, time_cost_minutes,
                "{portal_id} takes a different time in the fixture"
            );
            assert_eq!(
                route.supply_cost, supply_cost,
                "{portal_id} costs different rations in the fixture"
            );
            assert_eq!(
                route.risk_level, risk_level,
                "{portal_id} carries a different risk in the fixture"
            );
            assert_eq!(
                route.required_discovery_id, required_discovery_id,
                "{portal_id} is gated differently in the fixture"
            );
        }

        // C13: the anchors a cell carries are authored content too. Held equal
        // in both directions, and the gate on a portal must be a discovery some
        // anchor actually grants -- a gate nothing can open is a sealed room, and
        // a discovery the Rust grants under a different name than content
        // declares is the loot-table drift with a door on it.
        for (cell_id, authored_anchors) in &authored_anchors_by_cell {
            let mut fixture_anchors: Vec<AnchorDefinition> =
                geography.anchors_at(cell_id).into_iter().cloned().collect();
            fixture_anchors.sort_by(|left, right| left.id.cmp(&right.id));
            let mut authored_sorted = authored_anchors.clone();
            authored_sorted.sort_by(|left, right| left.id.cmp(&right.id));
            assert_eq!(
                fixture_anchors, authored_sorted,
                "{cell_id} carries different anchors in the fixture than content/world/ declares"
            );
        }
        for route in geography.routes.values() {
            if let Some(discovery_id) = &route.required_discovery_id {
                let declared_by_content = authored_granted_discoveries.contains(discovery_id)
                    || authored_observation_ids
                        .values()
                        .any(|ids| ids.iter().any(|id| id == discovery_id));
                let tomb_only = route.from_location_id.starts_with("world.cell.tomb_");
                assert!(
                    declared_by_content || tomb_only,
                    "{} is gated on {discovery_id}, which no authored anchor grants and no authored cell observes",
                    route.id
                );
            }
        }
        let (map_table_id, map_table_requires, map_table_grants) =
            authored_map_table.expect("content declares the estate's map table");
        assert_eq!(map_table_id, "anchor.estate.map_table");
        assert_eq!(
            map_table_requires.as_deref(),
            Some(crate::expedition::MAP_TABLE_REQUIRED_DISCOVERY_ID),
            "the map table's authored requirement and the Rust rule disagree"
        );
        assert_eq!(
            map_table_grants.as_deref(),
            Some(crate::expedition::MAP_TABLE_DISCOVERY_TIDAL_CUT),
            "the map table's authored grant and the Rust rule disagree"
        );
        assert_eq!(
            authored_workshop_requirement.as_deref(),
            Some(crate::expedition::WORKSHOP_REQUIRED_DISCOVERY_ID),
            "the workshop's authored requirement and the Rust rule disagree"
        );

        // And the other direction: a fixture route out of an authored cell that
        // content never declares is the same drift arriving from the other side.
        for (cell_id, authored_portal_ids) in &authored_portal_ids_by_cell {
            let fixture_portal_ids: BTreeSet<String> = geography
                .routes_from(cell_id)
                .into_iter()
                // The same B6 tolerance as the cell filter above, and no wider:
                // the ramp's door into the tomb is fixture-only until the tomb
                // is authored. Every other route out of an authored cell must be
                // in content.
                .filter(|route| !route.to_location_id.starts_with("world.cell.tomb_"))
                .map(|route| route.id.clone())
                .collect();
            assert_eq!(
                &fixture_portal_ids, authored_portal_ids,
                "{cell_id} leaves by a different set of portals in the fixture"
            );
        }

        // Observations are mirrored content too: the fixture's `observation_ids`
        // are the authored cell's `readableDescriptions`, in the same order.
        // A4's estate anchors are gated on two of them, so a rename on one side
        // would silently make a room unusable rather than fail loudly here.
        for (cell_id, observation_ids) in &authored_observation_ids {
            let location = geography
                .location(cell_id)
                .unwrap_or_else(|| panic!("authored cell {cell_id} is missing from the fixture"));
            assert_eq!(
                &location.observation_ids, observation_ids,
                "{cell_id} declares different observations in the fixture"
            );
        }
    }

    /// The bridge receives cells in the shape `native_expedition_port.gd`
    /// builds: snake_case keys, `once_per_day`, yields as flat fields. This is
    /// that exact shape, so a rename on either side fails here rather than as
    /// a room that silently does nothing in Godot.
    #[test]
    fn an_authored_cell_on_the_wire_becomes_a_cell_with_its_anchors() {
        let wire = serde_json::json!({
            "id": "world.cell.black_beach",
            "region_id": "world.region.black_beach",
            "display_name": "Black Beach",
            "observation_ids": ["observation.black_beach.wreck"],
            "anchors": [{
                "id": "anchor.black_beach.salvage_point",
                "kind": "salvage",
                "rations": 4,
                "medicine": 0,
                "coin": 2,
                "once_per_day": true
            }],
            "encounter_eligible": false
        });
        let authored: AuthoredCell = serde_json::from_value(wire).expect("the wire shape parses");
        let cell = CellDefinition::try_from(authored).expect("a known kind converts");
        assert_eq!(
            cell.anchors,
            vec![AnchorDefinition {
                id: "anchor.black_beach.salvage_point".into(),
                kind: AnchorKind::Salvage {
                    rations: 4,
                    coin: 2
                },
                once_per_day: true,
            }]
        );
        assert_eq!(cell.observation_ids, vec!["observation.black_beach.wreck"]);
    }

    /// A kind the simulation has no rule for is refused at configuration,
    /// naming the anchor, not defaulted into something harmless.
    #[test]
    fn an_unknown_anchor_kind_is_refused_by_name() {
        let authored = AuthoredAnchor {
            id: "anchor.estate.observatory".into(),
            kind: "observatory".into(),
            rations: 0,
            medicine: 0,
            coin: 0,
            once_per_day: false,
            requires_discovery_id: None,
            grants_discovery_id: None,
        };
        assert_eq!(
            AnchorDefinition::try_from(authored).unwrap_err(),
            ExpeditionError::UnknownAnchorKind {
                anchor_id: "anchor.estate.observatory".into(),
                kind: "observatory".into(),
            }
        );
    }
}
