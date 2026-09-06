//! S3: what a building *is* (authored) and what a building *is right now*
//! (saved). `docs/SHIP_PLAN.md` section 7, card S3; brief section 8
//! ("Buildings, spawning, and standardized space") and section 19's
//! "Building definition" block.
//!
//! Two types, two lifetimes, exactly as S1 split factions.
//! [`BuildingDefinition`] is content: authored once in
//! `content/buildings/<id>.json` (C10's card), loaded into a
//! [`BuildingDefinitions`] registry, never mutated by the simulation.
//! [`BuildingInstance`] is save data: it lives in
//! [`ExpeditionState::buildings`] and is the only half a strategic tick may
//! write.
//!
//! `BuildingDefinition`'s field list is brief section 19's block verbatim and
//! in the brief's order, so the design document and the code stay one
//! vocabulary and a reviewer can diff them by eye.
//!
//! **Four constraints this file is built around.**
//!
//! *Nothing essential outside the envelope* (brief section 8). A building
//! declares `footprint_cells` and `clearance_cells`, and everything the brief
//! lists as essential -- collision, walls, roof masses, stairs, open doors,
//! navigation projections, furniture, combat clearance, worker, vendor,
//! defender, spawn and delivery positions -- is inside them. In this crate
//! that is enforced twice: every socket's `offset_cells` must land inside the
//! declared footprint ([`BuildingDefinition::validate`]), and the envelopes of
//! the buildings standing on one cell may not sum past
//! [`CELL_CAPACITY_CELLS`] ([`ExpeditionState::place_building`]).
//!
//! *Buildings do not manufacture women* (brief section 5.6, and section 20
//! rejects "having player-faction buildings generate human soldiers or workers
//! from timers" outright). A definition compatible with
//! [`ConceptKey::Michael`] whose `production` names a
//! [`ProductionOutput::HumanRole`] does not load, full stop. For the other
//! five concepts a human-role rule loads only as *recruitment support*: never
//! on a timer, and only when the record's `recruitment_support` says what the
//! support is. See [`BuildingDefinition::validate`].
//!
//! *The Open numbers stay Open.* Brief section 20 leaves "exact building-tier
//! cap", "exact standard building dimensions" and "capture versus destruction
//! rules by building type" undecided. So the tier cap and the cell capacity
//! are named constants marked `needs decision` rather than magic numbers
//! sprinkled through the logic, dimensions are abstract counts of a cell's
//! capacity rather than metres (B11 and O3 own real geometry), and capture and
//! ruin are enums with a [`CaptureRules::NeedsDecision`] /
//! [`RuinState::NeedsDecision`] variant that **fails to load**, so that when
//! the decision lands it is made once per authored record instead of being
//! guessed here for every building at once.
//!
//! *Determinism.* No draw is taken in this file. Every map is a `BTreeMap`,
//! every set a `BTreeSet`, and every new field carries `serde(default)`.
//!
//! ## Where this lane stops
//!
//! Nothing ticks construction automatically: [`ExpeditionState::advance_construction`]
//! is called with a number of hours by whoever decides that hours passed, and
//! S13 decides which faction builds what. B11 materialises S7's arriving
//! forces through sockets; the seam it attaches to is
//! [`ExpeditionState::actor_sockets_at`], which answers "which actor sockets
//! does this cell offer, and whose are they". `strategy/force.rs` is not
//! edited by this lane.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, ExpeditionState, require_stable_id};
use crate::geography::Geography;
use crate::strategy::faction::{ConceptKey, FACTION_ID_PREFIX};
use crate::strategy::production::MachineFamily;

/// The prefix every authored building *definition* ID carries, as `faction.`
/// is to a faction.
pub const BUILDING_ID_PREFIX: &str = "building.";

/// The prefix every placed building *instance* ID carries. A definition and
/// the buildings raised from it live in different namespaces on purpose: one
/// is content and one is save data, and a save that confused the two would be
/// a save that could overwrite content by being loaded.
pub const BUILDING_INSTANCE_ID_PREFIX: &str = "building_instance.";

/// The first tier every building is placed at. Tiers count from one because
/// the brief speaks of a building's tier as a rank, not an index.
pub const FIRST_TIER: u32 = 1;

/// **needs decision** -- brief section 20, "Exact building-tier cap".
///
/// The highest tier [`ExpeditionState::upgrade_building`] will climb to. Three
/// is a placeholder chosen only so that the upgrade path has more than one
/// step to prove and a ceiling to refuse at; no behaviour anywhere reads the
/// value except the comparison in `upgrade_building`, and a definition may
/// author fewer tiers than the cap allows (it may not author more). When the
/// decision lands, this constant changes and nothing else does.
pub const TIER_CAP: u32 = 3;

/// **needs decision** -- brief section 20, "Exact standard building
/// dimensions".
///
/// How much building one map cell holds, counted in the same abstract
/// envelope cells a definition's `footprint_cells` and `clearance_cells` are
/// counted in. The island graph has no grid yet -- one map node is one
/// playable room, and B11/O3 own the room's real dimensions -- so a building's
/// envelope is a *share of a cell's capacity* rather than a rectangle, and
/// this is the size of the share pool. Twelve is a placeholder that lets a
/// handful of small buildings or one large one share a cell, which is enough
/// to make the overlap rejection a real rule rather than a formality.
pub const CELL_CAPACITY_CELLS: u32 = 12;

/// **needs decision** -- brief section 20, "Capture versus destruction rules
/// by building type".
///
/// The hit points a captured building stands up with. A captured building is
/// a wreck that changed hands, not a fresh one, so this is deliberately the
/// smallest non-zero value rather than a fraction of the tier's hit points:
/// picking a fraction would be inventing the repair economy that the Open item
/// has not yet been given. Nothing branches on the value.
pub const CAPTURE_HP_RESTORED: u32 = 1;

/// What went wrong loading, placing, or acting on a building.
///
/// A module-local error enum, following `FactionError` and `ForceError`: one
/// owner per failure vocabulary. ID *shape* is not re-implemented here --
/// [`BuildingError::MalformedId`] carries `ExpeditionError`'s verdict from the
/// crate's single `require_stable_id` rule.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildingError {
    /// Not a stable ID at all (empty, uppercase, no dot, stray punctuation).
    MalformedId(ExpeditionError),
    /// Well-shaped, but not in the namespace this position requires.
    WrongIdPrefix {
        found: String,
        expected: &'static str,
    },
    /// Two records claim the same definition ID, or two instances the same
    /// instance ID.
    Duplicate { id: String },
    /// A definition ID nothing in the registry defines.
    UnknownDefinition { id: String },
    /// An instance ID this campaign does not carry.
    UnknownBuilding { id: String },
    /// A cell the world graph does not know.
    UnknownCell { cell_id: String },
    /// The faction placing the building does not hold the cell. Brief section
    /// 1: the board is held ground; building on someone else's ground is a
    /// conquest, not a construction order.
    CellNotHeld {
        cell_id: String,
        faction_id: String,
        held_by: Option<String>,
    },
    /// The definition does not list this faction's concept in
    /// `faction_compatibility`.
    IncompatibleFaction { id: String, faction_id: String },
    /// Brief section 8: nothing essential outside the envelope. The sum of
    /// `footprint_cells + clearance_cells` over the buildings already standing
    /// on this cell, plus the one being placed, passes
    /// [`CELL_CAPACITY_CELLS`].
    EnvelopeOverlap {
        cell_id: String,
        occupied: u32,
        requested: u32,
        capacity: u32,
    },
    /// A socket sits outside the footprint it belongs to, which is the
    /// "uncontrolled procedural overhang" brief section 20 rejects.
    SocketOutsideEnvelope {
        socket_id: String,
        offset_cells: u32,
        footprint_cells: u32,
    },
    /// A socket of the wrong kind for the field it was authored in -- a spawn
    /// point in `road_sockets`, say. See [`SocketKind::field`].
    SocketInWrongField {
        socket_id: String,
        kind: SocketKind,
        field: &'static str,
    },
    /// Brief section 5.6 and section 20: Captain Michael's buildings produce
    /// machines, equipment, capacity and services. They never produce people.
    MichaelBuildingProducesPeople { id: String, rule_id: String },
    /// A human-role rule on a timer. Brief section 20 rejects
    /// "having player-faction buildings generate human soldiers or workers
    /// from timers"; this file extends the refusal to every faction, because a
    /// timer that manufactures a person is the same mechanism whoever owns it.
    TimerDrivenHumanRole { id: String, rule_id: String },
    /// A human-role rule on a record that does not say what recruitment it
    /// supports. The only reading under which a human-role rule is legal is
    /// "this building helps recruitment happen", so a record that claims one
    /// without describing the support is refused rather than assumed.
    HumanRoleWithoutRecruitmentSupport { id: String, rule_id: String },
    /// The record left an Open decision to the code. The message names the
    /// brief section that owns the decision.
    NeedsDecision { id: String, field: &'static str },
    /// Tiers must be authored `1, 2, 3, ...` with no gaps, no repeats, and no
    /// more of them than [`TIER_CAP`] allows.
    MalformedTiers { id: String, found: Vec<u32> },
    /// A production rule needs a tier the definition does not author.
    ProductionAboveTopTier {
        id: String,
        rule_id: String,
        minimum_tier: u32,
    },
    /// The building is already at [`TIER_CAP`], or at the top tier it authors.
    AtTopTier { id: String, tier: u32 },
    /// An upgrade whose `construction_requirements` the caller did not meet.
    /// The requirements are open string flags supplied by whoever is building;
    /// this lane does not invent the buildings or technologies that satisfy
    /// them.
    RequirementsNotMet {
        id: String,
        missing: BTreeSet<String>,
    },
    /// The building is in a state this action does not apply to: upgrading a
    /// ruin, capturing a building nobody has knocked down, damaging a wreck.
    WrongState {
        id: String,
        state: BuildingState,
        wanted: &'static str,
    },
    /// A capture attempt by the faction that already holds the building.
    AlreadyHeld { id: String, faction_id: String },
}

/// Where a socket sits and what stands in it.
///
/// Brief section 8 lists worker, vendor, defender, spawn, delivery and storage
/// positions separately, while section 19's definition block collapses them
/// into four fields (`entrance_sockets`, `road_sockets`, `actor_sockets`,
/// `delivery_sockets`). Both are kept and reconciled rather than one being
/// dropped: the four fields are section 19's, verbatim, and this enum is
/// section 8's finer list, with [`SocketKind::field`] saying which field each
/// kind is authored in. [`BuildingDefinition::validate`] holds the two
/// together, so a spawn point cannot hide in `road_sockets`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocketKind {
    /// A door. Where actors enter and leave the building itself.
    #[default]
    Entrance,
    /// Where the building meets a road, so placement can be told from
    /// connection.
    Road,
    /// A working position inside the envelope.
    Worker,
    /// A trading position.
    Vendor,
    /// A fighting position.
    Defender,
    /// Where offscreen forces materialise as actors (brief section 10). B11
    /// fills these; this lane only declares and validates them.
    Spawn,
    /// Where goods arrive.
    Delivery,
    /// Where goods sit once they have arrived.
    Storage,
}

impl SocketKind {
    /// Every kind, in brief section 8's order.
    pub const ALL: [SocketKind; 8] = [
        SocketKind::Entrance,
        SocketKind::Road,
        SocketKind::Worker,
        SocketKind::Vendor,
        SocketKind::Defender,
        SocketKind::Spawn,
        SocketKind::Delivery,
        SocketKind::Storage,
    ];

    /// Which of section 19's four socket fields this kind is authored in. The
    /// one place the mapping exists.
    pub fn field(self) -> &'static str {
        match self {
            SocketKind::Entrance => "entrance_sockets",
            SocketKind::Road => "road_sockets",
            SocketKind::Worker | SocketKind::Vendor | SocketKind::Defender | SocketKind::Spawn => {
                "actor_sockets"
            }
            SocketKind::Delivery | SocketKind::Storage => "delivery_sockets",
        }
    }
}

/// One declared position inside a building's envelope.
///
/// `offset_cells` is an index into the building's own footprint, not a
/// coordinate: the island graph has no grid, so "where in the building" is as
/// fine as this crate can honestly be until B11 and O3 give a room real
/// dimensions. What it *does* buy today is the envelope rule -- an offset at
/// or past `footprint_cells` is a position outside the declared envelope, and
/// that record does not load.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSocket {
    /// Stable within the definition; not a crate-wide ID.
    pub id: String,
    pub kind: SocketKind,
    /// Which envelope cell of this building's footprint the socket sits in,
    /// counted from zero. **needs decision** on real geometry: brief section
    /// 20, "Exact standard building dimensions".
    #[serde(default)]
    pub offset_cells: u32,
}

/// What a production rule makes.
///
/// The distinction brief section 5.6 draws, as a type: the first three are
/// things a building may make, and the fourth is the thing it may not make on
/// a timer. Modelling human roles as a variant rather than leaving them out
/// entirely is deliberate -- a rule that cannot be *expressed* cannot be
/// *refused*, and the refusal is the design decision worth keeping.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionOutput {
    /// Automata, vehicles, weapons, equipment (brief section 5.6), and
    /// **which of brief section 5.4's eight families** this rule turns out.
    ///
    /// S13 gave the variant its family: a rule that says only "a machine" is a
    /// rule nothing can build from, and the family is what
    /// [`ExpeditionState::produce_machine`](crate::expedition::ExpeditionState::produce_machine)
    /// checks the named `machine.<...>` record against before it spends
    /// anything.
    Machine { family: MachineFamily },
    /// Housing, training, medical, command, integration capacity.
    Capacity,
    /// Something the building does for whoever holds it.
    Service,
    /// A person in a role. Legal only as recruitment support, and never for a
    /// building compatible with [`ConceptKey::Michael`]. See
    /// [`BuildingDefinition::validate`].
    HumanRole { role: String },
}

impl Default for ProductionOutput {
    /// Only so [`ProductionRule`] can derive `Default`. `Machine` stopped being
    /// a unit variant when S13 gave it its family, so the `#[default]`
    /// attribute no longer applies and this states the same answer by hand:
    /// the default output is a machine, of the family
    /// [`MachineFamily`]'s own `Default` names for the same reason. No rule is
    /// authored by defaulting -- `output` carries no `serde(default)`, so
    /// content must state it -- and a rule whose family disagrees with the
    /// record it names does not produce.
    fn default() -> Self {
        ProductionOutput::Machine {
            family: MachineFamily::default(),
        }
    }
}

/// One thing a building produces, and how often.
///
/// `output_key` is an open string (brief section 20 leaves the resource list
/// Open, exactly as S1 found), so no enum of goods is invented here.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionRule {
    /// Stable within the definition, so an error message can name the rule.
    pub id: String,
    pub output: ProductionOutput,
    /// The open resource / machine / service key this rule yields.
    #[serde(default)]
    pub output_key: String,
    #[serde(default)]
    pub amount: u32,
    /// Hours between yields. **Zero means "not timer-driven"** -- the rule is
    /// a standing capability something else draws on, not a clock that
    /// produces on its own. A [`ProductionOutput::HumanRole`] rule must be
    /// zero: that is precisely what "recruitment support, not manufacture"
    /// means in code.
    #[serde(default)]
    pub interval_hours: u32,
    /// The tier at which this rule starts applying.
    #[serde(default)]
    pub minimum_tier: u32,
    /// **S13.** What one yield costs the producing faction, as open resource
    /// keys to counts -- the same shape as `construction_cost` beside it, and
    /// for the same reason: brief section 20 leaves the resource list Open, so
    /// fuel is a key a rule names (`resource.open.fuel`, say) rather than a
    /// variant of an enum this crate does not have.
    ///
    /// Charged from `FactionState::resources` by
    /// [`ExpeditionState::produce_machine`](crate::expedition::ExpeditionState::produce_machine),
    /// all keys or none. An empty map is a rule that costs nothing, which is
    /// what every rule authored before S13 landed means.
    #[serde(default)]
    pub cost: BTreeMap<String, u32>,
}

/// One authored tier of a building.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierState {
    /// `1`, `2`, `3`, ... ascending with no gaps. See
    /// [`BuildingError::MalformedTiers`].
    pub tier: u32,
    /// Hours of work to reach this tier from the one below (or, for
    /// [`FIRST_TIER`], from bare ground). Advanced only by
    /// [`ExpeditionState::advance_construction`].
    #[serde(default)]
    pub construction_hours: u32,
    /// Open string flags the *caller* must supply to unlock this tier. This
    /// lane does not invent the buildings, technologies or resources that
    /// satisfy them; S13 and C10 decide what the flags are.
    #[serde(default)]
    pub construction_requirements: BTreeSet<String>,
    /// The hit points a building standing at this tier has when whole.
    #[serde(default)]
    pub hit_points: u32,
    /// Authored note. Provisional detail lives here rather than in an invented
    /// field, following S1's rule for undecided design.
    #[serde(default)]
    pub notes: String,
}

/// Whether this kind of building changes hands or comes down.
///
/// **The decision this enum is waiting on is Open**: brief section 20,
/// "Capture versus destruction rules by building type". So there is a third
/// variant, it is the default, and a record carrying it does **not load** --
/// which puts the decision where it belongs, on the authored record, once per
/// building type, at the moment someone makes it. Defaulting to either real
/// answer would be this repository quietly deciding an Open item for every
/// building at once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureRules {
    /// Knocked to zero hit points, it can be taken by the attacker rather than
    /// destroyed. See [`ExpeditionState::capture_building`].
    Capturable,
    /// Knocked to zero hit points, it is ruined. Nobody takes it.
    DestroyOnly,
    /// Not decided for this building type yet. Refused at load.
    #[default]
    NeedsDecision,
}

/// What is left when a building is ruined.
///
/// The same Open item as [`CaptureRules`] -- brief section 20's "capture
/// versus destruction rules by building type" -- and the same treatment:
/// [`RuinState::NeedsDecision`] is the default and does not load.
/// `LeavesRubble` carries brief section 8's "ruined-state footprint", which is
/// the ground a wreck still occupies and therefore still denies to the next
/// building placed on that cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuinState {
    /// The wreck keeps this much of the cell's capacity.
    LeavesRubble { footprint_cells: u32 },
    /// The wreck occupies nothing; the ground is free again.
    ClearsCompletely,
    /// Not decided for this building type yet. Refused at load.
    #[default]
    NeedsDecision,
}

/// A building's authored design: brief section 19's "Building definition"
/// block, field for field and in order.
///
/// Everything the brief leaves Provisional or Open is stored as authored text
/// or as an open-keyed map, never as an invented enum -- S1's rule, applied
/// again. Each such field says which brief section owns the decision.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingDefinition {
    /// `building.<something>`. See [`BuildingDefinition::validate`].
    pub id: String,
    /// Which faction concepts may raise this building. Concepts, never proper
    /// names (brief section 20 rejects invented faction names), and the field
    /// [`BuildingDefinition::validate`] reads to enforce brief section 5.6.
    #[serde(default)]
    pub faction_compatibility: BTreeSet<ConceptKey>,
    /// What the building is for, as authored prose. The taxonomy of building
    /// functions is not decided; no enum is invented for it.
    #[serde(default)]
    pub function: String,
    /// Envelope, in the abstract cells [`CELL_CAPACITY_CELLS`] counts.
    #[serde(default)]
    pub footprint_cells: u32,
    /// Clearance around the footprint, in the same units. Brief section 8
    /// counts "required combat clearance" as essential and therefore inside
    /// the declared envelope, so this is part of what a placement consumes.
    #[serde(default)]
    pub clearance_cells: u32,
    /// Authored height band. **needs decision** -- brief section 20, "Exact
    /// standard building dimensions" -- so it is an authored string, not an
    /// enum this lane invented.
    #[serde(default)]
    pub height_class: String,
    #[serde(default)]
    pub entrance_sockets: Vec<BuildingSocket>,
    #[serde(default)]
    pub road_sockets: Vec<BuildingSocket>,
    /// Worker, vendor, defender and spawn positions (brief section 8's list,
    /// collapsed into section 19's field). B11 materialises forces through the
    /// spawn ones; see [`ExpeditionState::actor_sockets_at`].
    #[serde(default)]
    pub actor_sockets: Vec<BuildingSocket>,
    /// Delivery and storage positions.
    #[serde(default)]
    pub delivery_sockets: Vec<BuildingSocket>,
    /// Open terrain keys from the world graph, as S1's `movement_preferences`
    /// uses them. Empty means "anywhere the placement rules otherwise allow".
    #[serde(default)]
    pub allowed_terrain: BTreeSet<String>,
    /// **needs decision** on units -- brief section 20, "Exact standard
    /// building dimensions". Stored, validated for shape, read by nothing:
    /// the graph carries no slope yet, and inventing one here would be a
    /// second answer to terrain.
    #[serde(default)]
    pub maximum_slope: u8,
    /// Open resource keys to counts (brief section 20 leaves the resource list
    /// Open, so string keys and no enum). Charged by whoever builds -- S13 --
    /// not by this file, which does not own any faction's stockpile.
    #[serde(default)]
    pub construction_cost: BTreeMap<String, u32>,
    /// Open string flags required to build this at all, over and above the
    /// per-tier requirements in `tier_states`.
    #[serde(default)]
    pub construction_requirements: BTreeSet<String>,
    /// Ascending from [`FIRST_TIER`], at most [`TIER_CAP`] of them.
    #[serde(default)]
    pub tier_states: Vec<TierState>,
    /// What the building makes. Brief section 5.6 is enforced over this list.
    #[serde(default)]
    pub production: Vec<ProductionRule>,
    /// Open service keys this building offers whoever holds it.
    #[serde(default)]
    pub services: BTreeSet<String>,
    /// How this building helps recruitment happen -- reach, stability,
    /// integration capacity (brief section 5.6's permitted list). Open keys.
    /// A [`ProductionOutput::HumanRole`] rule is legal only on a record whose
    /// support is described here.
    #[serde(default)]
    pub recruitment_support: BTreeSet<String>,
    pub capture_rules: CaptureRules,
    pub ruin_state: RuinState,
    /// Brief section 19's `DungeonContext` names a building archetype and tier;
    /// S9 owns the generation signature. Authored prose until then.
    #[serde(default)]
    pub dungeon_relationship: String,
    /// S10 owns loot. Authored prose until then.
    #[serde(default)]
    pub loot_relationship: String,
}

impl BuildingDefinition {
    /// Every rule an authored record must satisfy before it may enter a
    /// registry. Refuses; never repairs.
    ///
    /// The two rules worth reading twice are the last two: an Open decision
    /// left to the code is refused by name, and a building that would
    /// manufacture a person is refused outright.
    pub fn validate(&self) -> Result<(), BuildingError> {
        require_stable_id("building.id", &self.id).map_err(BuildingError::MalformedId)?;
        if !self.id.starts_with(BUILDING_ID_PREFIX) {
            return Err(BuildingError::WrongIdPrefix {
                found: self.id.clone(),
                expected: BUILDING_ID_PREFIX,
            });
        }

        // Brief section 8: nothing essential outside the declared envelope.
        for (field, sockets) in self.socket_fields() {
            for socket in sockets {
                if socket.kind.field() != field {
                    return Err(BuildingError::SocketInWrongField {
                        socket_id: socket.id.clone(),
                        kind: socket.kind,
                        field,
                    });
                }
                if socket.offset_cells >= self.footprint_cells {
                    return Err(BuildingError::SocketOutsideEnvelope {
                        socket_id: socket.id.clone(),
                        offset_cells: socket.offset_cells,
                        footprint_cells: self.footprint_cells,
                    });
                }
            }
        }

        let tiers: Vec<u32> = self.tier_states.iter().map(|state| state.tier).collect();
        let expected: Vec<u32> = (FIRST_TIER..FIRST_TIER + tiers.len() as u32).collect();
        if tiers.is_empty() || tiers != expected || tiers.len() as u32 > TIER_CAP {
            return Err(BuildingError::MalformedTiers {
                id: self.id.clone(),
                found: tiers,
            });
        }

        let michaels = self.faction_compatibility.contains(&ConceptKey::Michael);
        for rule in &self.production {
            if rule.minimum_tier > self.top_tier() {
                return Err(BuildingError::ProductionAboveTopTier {
                    id: self.id.clone(),
                    rule_id: rule.id.clone(),
                    minimum_tier: rule.minimum_tier,
                });
            }
            if !matches!(rule.output, ProductionOutput::HumanRole { .. }) {
                continue;
            }
            // Brief section 5.6, and section 20's rejected list. Michael's
            // faction grows by recruitment, migration, relationships, rescue
            // and factional change -- never because a timer completed.
            if michaels {
                return Err(BuildingError::MichaelBuildingProducesPeople {
                    id: self.id.clone(),
                    rule_id: rule.id.clone(),
                });
            }
            // For the other five: a human role may be *supported*, never
            // *produced*. A non-zero interval is a manufacturing timer whoever
            // owns it, and section 20 rejects the mechanism, not the faction.
            if rule.interval_hours != 0 {
                return Err(BuildingError::TimerDrivenHumanRole {
                    id: self.id.clone(),
                    rule_id: rule.id.clone(),
                });
            }
            if self.recruitment_support.is_empty() {
                return Err(BuildingError::HumanRoleWithoutRecruitmentSupport {
                    id: self.id.clone(),
                    rule_id: rule.id.clone(),
                });
            }
        }

        // Brief section 20: "Capture versus destruction rules by building
        // type" is Open. Content chooses per record; this file will not choose
        // for it.
        if self.capture_rules == CaptureRules::NeedsDecision {
            return Err(BuildingError::NeedsDecision {
                id: self.id.clone(),
                field: "capture_rules (brief section 20: capture versus destruction rules by building type)",
            });
        }
        if self.ruin_state == RuinState::NeedsDecision {
            return Err(BuildingError::NeedsDecision {
                id: self.id.clone(),
                field: "ruin_state (brief section 20: capture versus destruction rules by building type)",
            });
        }
        Ok(())
    }

    /// Section 19's four socket fields with their names, so validation and any
    /// future reader iterate one list rather than four.
    fn socket_fields(&self) -> [(&'static str, &Vec<BuildingSocket>); 4] {
        [
            ("entrance_sockets", &self.entrance_sockets),
            ("road_sockets", &self.road_sockets),
            ("actor_sockets", &self.actor_sockets),
            ("delivery_sockets", &self.delivery_sockets),
        ]
    }

    /// The highest tier this record authors, which may be below [`TIER_CAP`].
    pub fn top_tier(&self) -> u32 {
        self.tier_states
            .last()
            .map(|state| state.tier)
            .unwrap_or(FIRST_TIER)
    }

    /// The authored tier, or `None` for a tier this record does not carry.
    pub fn tier(&self, tier: u32) -> Option<&TierState> {
        self.tier_states.iter().find(|state| state.tier == tier)
    }

    /// What one whole building of this type takes out of a cell's capacity:
    /// footprint plus clearance, because brief section 8 counts required
    /// clearance as essential and therefore inside the declared envelope.
    pub fn envelope_cells(&self) -> u32 {
        self.footprint_cells.saturating_add(self.clearance_cells)
    }

    /// What a *wreck* of this type still takes out of a cell's capacity.
    /// `ClearsCompletely` frees the ground; rubble keeps its authored share.
    pub fn ruined_envelope_cells(&self) -> u32 {
        match self.ruin_state {
            RuinState::LeavesRubble { footprint_cells } => footprint_cells,
            RuinState::ClearsCompletely | RuinState::NeedsDecision => 0,
        }
    }
}

/// The loaded building records, keyed by ID. C10 authors the files; this is
/// what the simulation reads. Insertion validates, so an invalid record cannot
/// be in a registry at all.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildingDefinitions {
    by_id: BTreeMap<String, BuildingDefinition>,
}

impl BuildingDefinitions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates, refuses a duplicate, then stores.
    pub fn insert(&mut self, definition: BuildingDefinition) -> Result<(), BuildingError> {
        definition.validate()?;
        if self.by_id.contains_key(&definition.id) {
            return Err(BuildingError::Duplicate {
                id: definition.id.clone(),
            });
        }
        self.by_id.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&BuildingDefinition> {
        self.by_id.get(id)
    }

    pub fn require(&self, id: &str) -> Result<&BuildingDefinition, BuildingError> {
        self.by_id
            .get(id)
            .ok_or_else(|| BuildingError::UnknownDefinition { id: id.to_owned() })
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// Where a placed building is in its life. The card's five, verbatim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingState {
    /// Placed, not finished. Hours of work remain.
    #[default]
    UnderConstruction,
    /// Finished and working.
    Operational,
    /// Hit, still standing. At zero hit points a *capturable* building waits
    /// here for an attacker; see [`ExpeditionState::damage_building`].
    Damaged,
    /// Down. Terminal: nothing repairs a ruin in this lane.
    Ruined,
    /// Taken by another faction rather than destroyed.
    Captured,
}

/// One building standing on the board: the save-data half.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingInstance {
    /// `building_instance.<something>`; the key this record sits under in
    /// [`ExpeditionState::buildings`].
    pub id: String,
    /// The `building.<...>` record this was raised from. The envelope, sockets
    /// and rules are read from there and are deliberately **not copied here**,
    /// so content and save cannot drift into two answers.
    pub def_id: String,
    pub cell_id: String,
    /// `faction.<concept_key>`. Reassigned by
    /// [`ExpeditionState::capture_building`] and by nothing else.
    pub faction_id: String,
    pub tier: u32,
    pub hp: u32,
    pub state: BuildingState,
    /// Hours of work left on the tier being built. Zero everywhere except
    /// `UnderConstruction`.
    #[serde(default)]
    pub construction_hours_remaining: u32,
    /// S9: the reward budget a dungeon raid draws down. Brief section 12:
    /// "Repeated farming must not generate infinite high-tier loot. Rewards
    /// must correspond to actual stored value" -- so this is that stored
    /// value, a plain count with no resource category attached (the resource
    /// list is Open; brief section 20).
    ///
    /// Owned by `strategy/dungeon.rs`:
    /// [`ExpeditionState::draw_dungeon_reward`](crate::expedition::ExpeditionState::draw_dungeon_reward)
    /// is the only thing that lowers it, and **nothing in this round raises
    /// it** -- refill from production, reinforcement and recovery is S13's and
    /// S3's to decide, and inventing a refill here would be a second answer to
    /// what a building is worth. `serde(default)` so a save written before the
    /// budget existed loads with an empty one.
    #[serde(default)]
    pub stored_value: u32,
    /// **S13.** How many machines this building has turned out, ever. Raised
    /// only by
    /// [`ExpeditionState::produce_machine`](crate::expedition::ExpeditionState::produce_machine)
    /// and by nothing else, and never lowered: it is a record of what this
    /// yard did, not a stock of what stands in it -- the machines themselves
    /// are in `ExpeditionState::machines`, one entry each. `serde(default)` so
    /// a save written before S13 loads with a yard that has made nothing yet.
    #[serde(default)]
    pub machines_produced: u32,
}

impl BuildingInstance {
    /// Whether this building is standing well enough to work -- to offer
    /// sockets, services, or production.
    pub fn is_working(&self) -> bool {
        matches!(
            self.state,
            BuildingState::Operational | BuildingState::Captured
        )
    }
}

impl ExpeditionState {
    /// Raise a building on a cell. The one way [`ExpeditionState::buildings`]
    /// gains an entry.
    ///
    /// Rejects *before mutating* on: a malformed or wrongly-prefixed instance
    /// or faction ID, an instance ID this campaign already carries, a cell the
    /// graph does not know, a definition the registry does not carry, a faction
    /// that is not the cell's effective holder
    /// ([`Geography::held_by`], which is the S2 answer -- the save's override
    /// where there is one, the authored owner otherwise), a definition that
    /// does not list this faction's concept, and -- the rule brief section 8
    /// exists for -- an envelope that does not fit in what is left of the
    /// cell's capacity.
    ///
    /// The new building starts at [`FIRST_TIER`], `UnderConstruction`, with
    /// that tier's hit points and hours. A tier authored with zero hours is
    /// finished the moment it is placed, rather than needing a zero-hour tick.
    pub fn place_building(
        &mut self,
        instance_id: &str,
        def_id: &str,
        cell_id: &str,
        faction_id: &str,
        geography: &Geography,
        definitions: &BuildingDefinitions,
    ) -> Result<(), BuildingError> {
        require_prefixed_id(
            "building.instance_id",
            instance_id,
            BUILDING_INSTANCE_ID_PREFIX,
        )?;
        require_stable_id("faction_id", faction_id).map_err(BuildingError::MalformedId)?;
        if self.buildings.contains_key(instance_id) {
            return Err(BuildingError::Duplicate {
                id: instance_id.to_owned(),
            });
        }
        if geography.location(cell_id).is_none() {
            return Err(BuildingError::UnknownCell {
                cell_id: cell_id.to_owned(),
            });
        }
        let definition = definitions.require(def_id)?;

        let held_by = geography.held_by(cell_id, &self.ownership);
        if held_by != Some(faction_id) {
            return Err(BuildingError::CellNotHeld {
                cell_id: cell_id.to_owned(),
                faction_id: faction_id.to_owned(),
                held_by: held_by.map(str::to_owned),
            });
        }

        // The faction's concept is its ID's suffix -- S1 holds every faction ID
        // to `faction.<concept_key>` -- so compatibility is read off the ID
        // rather than requiring a second registry here.
        let concept = faction_id
            .strip_prefix(FACTION_ID_PREFIX)
            .and_then(ConceptKey::from_key);
        if !concept.is_some_and(|concept| definition.faction_compatibility.contains(&concept)) {
            return Err(BuildingError::IncompatibleFaction {
                id: def_id.to_owned(),
                faction_id: faction_id.to_owned(),
            });
        }

        // Brief section 8: nothing essential outside the envelope, and no two
        // envelopes over the same ground.
        let occupied = self.envelope_cells_used(cell_id, definitions);
        let requested = definition.envelope_cells();
        if occupied.saturating_add(requested) > CELL_CAPACITY_CELLS {
            return Err(BuildingError::EnvelopeOverlap {
                cell_id: cell_id.to_owned(),
                occupied,
                requested,
                capacity: CELL_CAPACITY_CELLS,
            });
        }

        let first = definition
            .tier(FIRST_TIER)
            .ok_or_else(|| BuildingError::MalformedTiers {
                id: def_id.to_owned(),
                found: Vec::new(),
            })?;
        let hours = first.construction_hours;
        self.buildings.insert(
            instance_id.to_owned(),
            BuildingInstance {
                id: instance_id.to_owned(),
                def_id: def_id.to_owned(),
                cell_id: cell_id.to_owned(),
                faction_id: faction_id.to_owned(),
                tier: FIRST_TIER,
                hp: first.hit_points,
                state: if hours == 0 {
                    BuildingState::Operational
                } else {
                    BuildingState::UnderConstruction
                },
                construction_hours_remaining: hours,
                stored_value: 0,
                // S13: a new yard has made nothing.
                machines_produced: 0,
            },
        );
        Ok(())
    }

    /// How much of `cell_id`'s capacity is already spoken for: whole buildings
    /// count their full envelope, wrecks count their authored rubble, and a
    /// building whose definition has gone missing counts nothing (the registry
    /// is the owner of envelopes, and a save cannot conjure one).
    pub fn envelope_cells_used(&self, cell_id: &str, definitions: &BuildingDefinitions) -> u32 {
        self.buildings
            .values()
            .filter(|building| building.cell_id == cell_id)
            .filter_map(|building| {
                definitions
                    .get(&building.def_id)
                    .map(|definition| match building.state {
                        BuildingState::Ruined => definition.ruined_envelope_cells(),
                        _ => definition.envelope_cells(),
                    })
            })
            .fold(0u32, u32::saturating_add)
    }

    /// Put `hours` of work into every building currently under construction,
    /// and return the IDs of the ones that finished, in `BTreeMap` order.
    ///
    /// **Nothing calls this on a clock.** The strategic tick does not build:
    /// S13 decides which faction spends hours on what, and until it does, the
    /// only caller is a test or a deliberate command. That is why this takes
    /// hours rather than reading the clock itself -- a method that read the
    /// clock would be a second answer to "how much time passed", and S4 owns
    /// the first one.
    pub fn advance_construction(&mut self, hours: u32) -> Vec<String> {
        let mut finished = Vec::new();
        for building in self.buildings.values_mut() {
            if building.state != BuildingState::UnderConstruction {
                continue;
            }
            building.construction_hours_remaining =
                building.construction_hours_remaining.saturating_sub(hours);
            if building.construction_hours_remaining == 0 {
                building.state = BuildingState::Operational;
                finished.push(building.id.clone());
            }
        }
        finished
    }

    /// Move one building one tier up, and return the tier it is now building.
    ///
    /// Refuses: an unknown building, one that is not working (a wreck does not
    /// upgrade, and neither does a half-built shed), one already at
    /// [`TIER_CAP`] or at the top tier its record authors, and one whose next
    /// tier's `construction_requirements` are not all in `met_requirements`.
    ///
    /// `met_requirements` is supplied by the caller on purpose: the flags are
    /// open strings, and this lane does not invent the buildings or
    /// technologies that satisfy them. A successful upgrade puts the building
    /// back into `UnderConstruction` for the new tier's hours -- an upgrade is
    /// construction, not a state flag.
    pub fn upgrade_building(
        &mut self,
        instance_id: &str,
        met_requirements: &BTreeSet<String>,
        definitions: &BuildingDefinitions,
    ) -> Result<u32, BuildingError> {
        let building = self.require_building(instance_id)?;
        if !building.is_working() {
            return Err(BuildingError::WrongState {
                id: instance_id.to_owned(),
                state: building.state,
                wanted: "Operational or Captured",
            });
        }
        let definition = definitions.require(&building.def_id)?;
        let next = building.tier.saturating_add(1);
        if building.tier >= TIER_CAP || next > definition.top_tier() {
            return Err(BuildingError::AtTopTier {
                id: instance_id.to_owned(),
                tier: building.tier,
            });
        }
        let tier_state = definition
            .tier(next)
            .ok_or_else(|| BuildingError::AtTopTier {
                id: instance_id.to_owned(),
                tier: building.tier,
            })?;
        let missing: BTreeSet<String> = tier_state
            .construction_requirements
            .difference(met_requirements)
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(BuildingError::RequirementsNotMet {
                id: instance_id.to_owned(),
                missing,
            });
        }

        let hours = tier_state.construction_hours;
        let hit_points = tier_state.hit_points;
        let building = self
            .buildings
            .get_mut(instance_id)
            .expect("the building was found above");
        building.tier = next;
        building.hp = hit_points;
        building.construction_hours_remaining = hours;
        building.state = if hours == 0 {
            BuildingState::Operational
        } else {
            BuildingState::UnderConstruction
        };
        Ok(next)
    }

    /// Take `amount` hit points off a building and return the state it is left
    /// in.
    ///
    /// Above zero, a building that was whole becomes `Damaged`. At zero the
    /// **Open** item decides: a `DestroyOnly` building is `Ruined` here and
    /// now, and a `Capturable` one stays `Damaged` at zero hit points -- a
    /// shell standing empty, waiting for whoever knocked it down to walk in
    /// through [`ExpeditionState::capture_building`]. Ruining it anyway would
    /// be this file answering "capture versus destruction by building type"
    /// for the record, which is exactly what `capture_rules` exists to leave
    /// to content.
    pub fn damage_building(
        &mut self,
        instance_id: &str,
        amount: u32,
        definitions: &BuildingDefinitions,
    ) -> Result<BuildingState, BuildingError> {
        let building = self.require_building(instance_id)?;
        if building.state == BuildingState::Ruined {
            return Err(BuildingError::WrongState {
                id: instance_id.to_owned(),
                state: building.state,
                wanted: "anything but Ruined",
            });
        }
        let capture_rules = definitions.require(&building.def_id)?.capture_rules;
        let building = self
            .buildings
            .get_mut(instance_id)
            .expect("the building was found above");
        building.hp = building.hp.saturating_sub(amount);
        if building.hp > 0 {
            building.state = BuildingState::Damaged;
        } else {
            building.state = match capture_rules {
                CaptureRules::Capturable => BuildingState::Damaged,
                // `NeedsDecision` cannot reach a registry -- `validate`
                // refuses it -- so this arm is the conservative reading and
                // not a decision: a building nobody said was capturable is not
                // captured.
                CaptureRules::DestroyOnly | CaptureRules::NeedsDecision => BuildingState::Ruined,
            };
            if building.state == BuildingState::Ruined {
                building.construction_hours_remaining = 0;
            }
        }
        Ok(building.state)
    }

    /// Hand a knocked-down capturable building to the attacker.
    ///
    /// Refuses: an unknown building, a definition that is not `Capturable`, a
    /// building still standing (hit points above zero -- you take a building by
    /// beating it, not by asking), a ruin, and a "capture" by the faction that
    /// already holds it.
    pub fn capture_building(
        &mut self,
        instance_id: &str,
        attacker_faction_id: &str,
        definitions: &BuildingDefinitions,
    ) -> Result<(), BuildingError> {
        require_stable_id("faction_id", attacker_faction_id).map_err(BuildingError::MalformedId)?;
        let building = self.require_building(instance_id)?;
        if building.faction_id == attacker_faction_id {
            return Err(BuildingError::AlreadyHeld {
                id: instance_id.to_owned(),
                faction_id: attacker_faction_id.to_owned(),
            });
        }
        let definition = definitions.require(&building.def_id)?;
        if definition.capture_rules != CaptureRules::Capturable {
            return Err(BuildingError::WrongState {
                id: instance_id.to_owned(),
                state: building.state,
                wanted: "a definition whose capture_rules are Capturable",
            });
        }
        if building.state == BuildingState::Ruined || building.hp > 0 {
            return Err(BuildingError::WrongState {
                id: instance_id.to_owned(),
                state: building.state,
                wanted: "Damaged at zero hit points",
            });
        }
        let building = self
            .buildings
            .get_mut(instance_id)
            .expect("the building was found above");
        building.faction_id = attacker_faction_id.to_owned();
        building.state = BuildingState::Captured;
        building.hp = CAPTURE_HP_RESTORED;
        building.construction_hours_remaining = 0;
        Ok(())
    }

    /// Bring a building down deliberately, whatever its hit points were.
    ///
    /// The demolition path, distinct from losing a fight: an attacker who does
    /// not want a capturable building standing, or a faction razing its own.
    /// Refuses a building that is already a ruin.
    pub fn ruin_building(&mut self, instance_id: &str) -> Result<(), BuildingError> {
        let building = self.require_building(instance_id)?;
        if building.state == BuildingState::Ruined {
            return Err(BuildingError::WrongState {
                id: instance_id.to_owned(),
                state: building.state,
                wanted: "anything but Ruined",
            });
        }
        let building = self
            .buildings
            .get_mut(instance_id)
            .expect("the building was found above");
        building.state = BuildingState::Ruined;
        building.hp = 0;
        building.construction_hours_remaining = 0;
        Ok(())
    }

    /// Every building standing on one cell, in `BTreeMap` order.
    pub fn buildings_at(&self, cell_id: &str) -> Vec<&BuildingInstance> {
        self.buildings
            .values()
            .filter(|building| building.cell_id == cell_id)
            .collect()
    }

    /// **The seam B11 attaches to.** Every actor socket a cell offers right
    /// now, as `(instance id, socket)` pairs in `BTreeMap` order.
    ///
    /// S7 stops at [`crate::strategy::tick::StrategicEvent::ForceArrived`]
    /// carrying the arriving force's whole composition, and brief section 10
    /// says such a force materialises *through that cell's spawn sockets*.
    /// This is where the sockets come from. It is exposed here rather than by
    /// editing `force.rs`, so that "what a building offers" keeps one owner.
    ///
    /// Only working buildings offer sockets -- a half-built or ruined building
    /// has no working positions -- and the sockets themselves are read from
    /// the definition registry, never from the save, so a save cannot invent a
    /// spawn point.
    pub fn actor_sockets_at<'a>(
        &self,
        cell_id: &str,
        definitions: &'a BuildingDefinitions,
    ) -> Vec<(String, &'a BuildingSocket)> {
        self.buildings
            .values()
            .filter(|building| building.cell_id == cell_id && building.is_working())
            .filter_map(|building| {
                definitions
                    .get(&building.def_id)
                    .map(|definition| (building, definition))
            })
            .flat_map(|(building, definition)| {
                definition
                    .actor_sockets
                    .iter()
                    .map(move |socket| (building.id.clone(), socket))
            })
            .collect()
    }

    fn require_building(&self, instance_id: &str) -> Result<&BuildingInstance, BuildingError> {
        self.buildings
            .get(instance_id)
            .ok_or_else(|| BuildingError::UnknownBuilding {
                id: instance_id.to_owned(),
            })
    }
}

/// A stable ID in a required namespace. The crate's one ID-shape rule plus one
/// prefix check, in one place, so no call site re-implements either.
fn require_prefixed_id(
    field: &'static str,
    value: &str,
    prefix: &'static str,
) -> Result<(), BuildingError> {
    require_stable_id(field, value).map_err(BuildingError::MalformedId)?;
    if value.starts_with(prefix) {
        Ok(())
    } else {
        Err(BuildingError::WrongIdPrefix {
            found: value.to_owned(),
            expected: prefix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::strategy::faction::FactionDefinition;
    use crate::strategy::production::{
        ActorKit, MachineDefinition, MachineDefinitions, ProductionError,
    };

    const BEACH: &str = "world.cell.black_beach";
    const TERRACE: &str = "world.cell.reception_terrace";

    /// Content owns building and actor IDs; `content/buildings/` is C10's card
    /// and does not exist yet. These are the test namespace's, read by nothing
    /// outside this module.
    const WORKSHOP: &str = "building.test.workshop";
    const SHED: &str = "building.test.shed";
    const INSTANCE: &str = "building_instance.test.first";

    fn pirates() -> String {
        ConceptKey::Pirates.faction_id()
    }

    fn elves() -> String {
        ConceptKey::Elves.faction_id()
    }

    fn tiers(count: u32) -> Vec<TierState> {
        (FIRST_TIER..FIRST_TIER + count)
            .map(|tier| TierState {
                tier,
                construction_hours: 4 * tier,
                construction_requirements: if tier == FIRST_TIER {
                    BTreeSet::new()
                } else {
                    BTreeSet::from([format!("requirement.test.tier_{tier}")])
                },
                hit_points: 100 * tier,
                notes: String::new(),
            })
            .collect()
    }

    /// A record that loads: decided capture and ruin, tiers from one, sockets
    /// inside the footprint.
    fn a_definition(id: &str, footprint: u32, clearance: u32) -> BuildingDefinition {
        BuildingDefinition {
            id: id.to_owned(),
            faction_compatibility: BTreeSet::from([ConceptKey::Pirates, ConceptKey::Michael]),
            function: "a test building".into(),
            footprint_cells: footprint,
            clearance_cells: clearance,
            height_class: "needs decision".into(),
            entrance_sockets: vec![BuildingSocket {
                id: "socket.door".into(),
                kind: SocketKind::Entrance,
                offset_cells: 0,
            }],
            actor_sockets: vec![
                BuildingSocket {
                    id: "socket.bench".into(),
                    kind: SocketKind::Worker,
                    offset_cells: 0,
                },
                BuildingSocket {
                    id: "socket.muster".into(),
                    kind: SocketKind::Spawn,
                    offset_cells: 1.min(footprint.saturating_sub(1)),
                },
            ],
            tier_states: tiers(3),
            capture_rules: CaptureRules::Capturable,
            ruin_state: RuinState::LeavesRubble { footprint_cells: 1 },
            ..BuildingDefinition::default()
        }
    }

    fn registry(definitions: Vec<BuildingDefinition>) -> BuildingDefinitions {
        let mut registry = BuildingDefinitions::new();
        for definition in definitions {
            registry
                .insert(definition)
                .expect("the fixture definitions load");
        }
        registry
    }

    fn a_campaign(geography: &Geography, holder: &str) -> ExpeditionState {
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        state
            .set_control(BEACH, Some(holder.to_owned()), geography)
            .expect("the beach is a real cell");
        state
    }

    /// The card's Done-when, and brief section 8's rule: nothing essential
    /// outside the envelope, and two envelopes may not share the same ground.
    ///
    /// Bite proof: deleting the capacity comparison in `place_building` makes
    /// the third placement succeed and this test fail.
    #[test]
    fn a_cell_holds_only_as_much_building_as_its_capacity_allows() {
        let geography = Geography::black_beach_vertical_slice();
        // Envelope 5 each: two fit in CELL_CAPACITY_CELLS = 12, three do not.
        let definitions = registry(vec![a_definition(WORKSHOP, 3, 2)]);
        let mut state = a_campaign(&geography, &pirates());

        for index in 0..2 {
            state
                .place_building(
                    &format!("building_instance.test.yard_{index}"),
                    WORKSHOP,
                    BEACH,
                    &pirates(),
                    &geography,
                    &definitions,
                )
                .expect("the first two envelopes fit on the cell");
        }
        assert_eq!(state.envelope_cells_used(BEACH, &definitions), 10);

        assert_eq!(
            state.place_building(
                "building_instance.test.yard_2",
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            ),
            Err(BuildingError::EnvelopeOverlap {
                cell_id: BEACH.to_owned(),
                occupied: 10,
                requested: 5,
                capacity: CELL_CAPACITY_CELLS,
            }),
            "brief section 8: the third envelope would run outside the cell's capacity"
        );
        assert_eq!(
            state.buildings.len(),
            2,
            "a rejected placement mutates nothing"
        );

        // A wreck keeps only its authored rubble, so ruining one frees ground
        // for a small building without freeing the whole plot.
        state
            .ruin_building("building_instance.test.yard_0")
            .expect("a standing building can be brought down");
        assert_eq!(
            state.envelope_cells_used(BEACH, &definitions),
            6,
            "5 whole + 1 of rubble"
        );
        let small = registry(vec![a_definition(WORKSHOP, 3, 2), {
            let mut small = a_definition(SHED, 2, 0);
            small.ruin_state = RuinState::ClearsCompletely;
            small
        }]);
        state
            .place_building(
                "building_instance.test.shed",
                SHED,
                BEACH,
                &pirates(),
                &geography,
                &small,
            )
            .expect("a small envelope fits beside a wreck");
    }

    /// Placement refuses everything it should, before mutating anything.
    #[test]
    fn placement_refuses_bad_ids_unknown_cells_unheld_ground_and_wrong_factions() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry(vec![a_definition(WORKSHOP, 2, 1)]);
        let mut state = a_campaign(&geography, &pirates());

        assert!(matches!(
            state.place_building(
                "yard",
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions
            ),
            Err(BuildingError::MalformedId(_))
        ));
        assert!(matches!(
            state.place_building(
                "building.test.yard",
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions
            ),
            Err(BuildingError::WrongIdPrefix { .. })
        ));
        assert_eq!(
            state.place_building(
                INSTANCE,
                WORKSHOP,
                "world.cell.nowhere",
                &pirates(),
                &geography,
                &definitions
            ),
            Err(BuildingError::UnknownCell {
                cell_id: "world.cell.nowhere".into()
            })
        );
        assert_eq!(
            state.place_building(
                INSTANCE,
                "building.test.missing",
                BEACH,
                &pirates(),
                &geography,
                &definitions
            ),
            Err(BuildingError::UnknownDefinition {
                id: "building.test.missing".into()
            })
        );
        // The terrace is nobody's; S2's `held_by` is the one answer to who
        // holds a cell, and building on ground you do not hold is a conquest.
        assert_eq!(
            state.place_building(
                INSTANCE,
                WORKSHOP,
                TERRACE,
                &pirates(),
                &geography,
                &definitions
            ),
            Err(BuildingError::CellNotHeld {
                cell_id: TERRACE.into(),
                faction_id: pirates(),
                held_by: None,
            })
        );
        assert_eq!(
            state.place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &elves(),
                &geography,
                &definitions
            ),
            Err(BuildingError::CellNotHeld {
                cell_id: BEACH.into(),
                faction_id: elves(),
                held_by: Some(pirates()),
            }),
            "the elves do not hold the beach, so they cannot build on it"
        );
        assert!(state.buildings.is_empty(), "nothing was mutated");

        state
            .set_control(BEACH, Some(elves()), &geography)
            .expect("control changes hands");
        assert_eq!(
            state.place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &elves(),
                &geography,
                &definitions
            ),
            Err(BuildingError::IncompatibleFaction {
                id: WORKSHOP.into(),
                faction_id: elves(),
            }),
            "the record lists pirates and michael, not elves"
        );
        assert!(state.buildings.is_empty());
    }

    /// The card's Done-when: brief section 5.6 and section 20's rejected list,
    /// as a load-time refusal.
    #[test]
    fn a_michael_building_that_produces_a_person_does_not_load() {
        let human = ProductionRule {
            id: "production.test.people".into(),
            output: ProductionOutput::HumanRole {
                role: "role.test.engineer".into(),
            },
            output_key: "role.test.engineer".into(),
            amount: 1,
            interval_hours: 0,
            minimum_tier: FIRST_TIER,
            cost: BTreeMap::new(),
        };

        let mut michaels = a_definition(WORKSHOP, 2, 1);
        michaels.recruitment_support = BTreeSet::from(["support.test.reach".to_owned()]);
        michaels.production = vec![human.clone()];
        assert_eq!(
            michaels.validate(),
            Err(BuildingError::MichaelBuildingProducesPeople {
                id: WORKSHOP.into(),
                rule_id: "production.test.people".into(),
            }),
            "brief section 5.6: Michael's buildings make machines, capacity and services"
        );
        assert!(
            BuildingDefinitions::new().insert(michaels.clone()).is_err(),
            "and the registry will not carry it"
        );

        // The same building making a machine is fine, however compatible with
        // Michael it is.
        let mut machines = michaels.clone();
        machines.production = vec![ProductionRule {
            id: "production.test.automaton".into(),
            output: ProductionOutput::Machine {
                family: MachineFamily::MechanicalDog,
            },
            output_key: "machine.test.mechanical_dog".into(),
            amount: 1,
            interval_hours: 6,
            minimum_tier: FIRST_TIER,
            cost: BTreeMap::from([("resource.open.fuel".to_owned(), 3)]),
        }];
        assert_eq!(machines.validate(), Ok(()));

        // **S13, the same refusal one level up.** A building's `production`
        // list is not the only door a manufactured person could come through:
        // the faction's `actor_kit` names the unit records it may field. For
        // Michael's concept every one of them must be a machine, and an entry
        // the machine registry does not carry is refused by name rather than
        // admitted as "probably a unit". Same rule, same test, one owner --
        // `strategy/production.rs` holds the reading, this holds the proof.
        let mut registry = MachineDefinitions::new();
        registry
            .insert(MachineDefinition {
                id: "machine.test.mechanical_dog".into(),
                family: MachineFamily::MechanicalDog,
                ..MachineDefinition::default()
            })
            .expect("the machine record loads");
        let mut michaels_faction = FactionDefinition {
            id: ConceptKey::Michael.faction_id(),
            concept_key: ConceptKey::Michael,
            actor_kit: BTreeSet::from(["machine.test.mechanical_dog".to_owned()]),
            ..FactionDefinition::default()
        };
        assert!(
            ActorKit::of(&michaels_faction, &registry).is_ok(),
            "a kit of machines is what his faction fields"
        );
        michaels_faction
            .actor_kit
            .insert("actor.test.engineer".to_owned());
        assert_eq!(
            ActorKit::of(&michaels_faction, &registry),
            Err(ProductionError::MichaelActorKitNamesANonMachine {
                faction_id: ConceptKey::Michael.faction_id(),
                entry_id: "actor.test.engineer".into(),
            }),
            "brief section 5.6 again: his lists carry machines, never people"
        );

        // Another faction may *support* recruitment with a human-role rule,
        // but never run one on a timer, and never without saying what the
        // support is.
        let mut others = a_definition(WORKSHOP, 2, 1);
        others.faction_compatibility = BTreeSet::from([ConceptKey::Elves]);
        others.production = vec![human.clone()];
        assert_eq!(
            others.validate(),
            Err(BuildingError::HumanRoleWithoutRecruitmentSupport {
                id: WORKSHOP.into(),
                rule_id: "production.test.people".into(),
            })
        );

        others.recruitment_support = BTreeSet::from(["support.test.reach".to_owned()]);
        assert_eq!(
            others.validate(),
            Ok(()),
            "recruitment support is the one legal reading of a human-role rule"
        );

        others.production = vec![ProductionRule {
            interval_hours: 12,
            ..human
        }];
        assert_eq!(
            others.validate(),
            Err(BuildingError::TimerDrivenHumanRole {
                id: WORKSHOP.into(),
                rule_id: "production.test.people".into(),
            }),
            "brief section 20 rejects the timer, whoever owns it"
        );
    }

    /// The Open items stay Open: a record that did not choose does not load,
    /// and the message names the section that owes the decision.
    #[test]
    fn a_record_that_leaves_capture_or_ruin_undecided_is_refused_by_name() {
        let mut undecided = a_definition(WORKSHOP, 2, 1);
        undecided.capture_rules = CaptureRules::NeedsDecision;
        match undecided.validate() {
            Err(BuildingError::NeedsDecision { id, field }) => {
                assert_eq!(id, WORKSHOP);
                assert!(field.starts_with("capture_rules"));
                assert!(
                    field.contains("brief section 20"),
                    "the refusal must name the section that owns the decision"
                );
            }
            other => panic!("expected a needs-decision refusal, got {other:?}"),
        }

        let mut undecided = a_definition(WORKSHOP, 2, 1);
        undecided.ruin_state = RuinState::NeedsDecision;
        assert!(matches!(
            undecided.validate(),
            Err(BuildingError::NeedsDecision { .. })
        ));

        // The defaults are the undecided variants on purpose, so a record that
        // simply omits the fields is refused rather than silently assigned an
        // answer this repository was not given.
        assert_eq!(CaptureRules::default(), CaptureRules::NeedsDecision);
        assert_eq!(RuinState::default(), RuinState::NeedsDecision);
    }

    /// Envelopes and socket fields are validated together, so section 8's
    /// finer list and section 19's four fields cannot disagree.
    #[test]
    fn a_socket_outside_the_envelope_or_in_the_wrong_field_does_not_load() {
        let mut overhang = a_definition(WORKSHOP, 2, 1);
        overhang.actor_sockets[1].offset_cells = 2;
        assert_eq!(
            overhang.validate(),
            Err(BuildingError::SocketOutsideEnvelope {
                socket_id: "socket.muster".into(),
                offset_cells: 2,
                footprint_cells: 2,
            }),
            "brief section 20 rejects uncontrolled procedural overhang"
        );

        let mut misfiled = a_definition(WORKSHOP, 2, 1);
        misfiled.road_sockets = vec![BuildingSocket {
            id: "socket.sneaky".into(),
            kind: SocketKind::Spawn,
            offset_cells: 0,
        }];
        assert_eq!(
            misfiled.validate(),
            Err(BuildingError::SocketInWrongField {
                socket_id: "socket.sneaky".into(),
                kind: SocketKind::Spawn,
                field: "road_sockets",
            })
        );

        for kind in SocketKind::ALL {
            assert!(
                matches!(
                    kind.field(),
                    "entrance_sockets" | "road_sockets" | "actor_sockets" | "delivery_sockets"
                ),
                "every section 8 kind lives in one of section 19's four fields"
            );
        }

        let mut ungraded = a_definition(WORKSHOP, 2, 1);
        ungraded.tier_states = vec![
            TierState {
                tier: 1,
                ..TierState::default()
            },
            TierState {
                tier: 3,
                ..TierState::default()
            },
        ];
        assert_eq!(
            ungraded.validate(),
            Err(BuildingError::MalformedTiers {
                id: WORKSHOP.into(),
                found: vec![1, 3],
            })
        );
    }

    /// The card's Done-when: a tier upgrade, its requirements, and the cap.
    #[test]
    fn a_building_climbs_one_tier_at_a_time_up_to_the_cap() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry(vec![a_definition(WORKSHOP, 2, 1)]);
        let mut state = a_campaign(&geography, &pirates());
        state
            .place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the first building goes up");

        let building = &state.buildings[INSTANCE];
        assert_eq!(building.tier, FIRST_TIER);
        assert_eq!(building.state, BuildingState::UnderConstruction);
        assert_eq!(building.construction_hours_remaining, 4);
        assert_eq!(building.hp, 100);

        // An unfinished building does not upgrade.
        assert!(matches!(
            state.upgrade_building(INSTANCE, &BTreeSet::new(), &definitions),
            Err(BuildingError::WrongState { .. })
        ));

        assert!(
            state.advance_construction(3).is_empty(),
            "three of four hours finishes nothing"
        );
        assert_eq!(
            state.advance_construction(1),
            vec![INSTANCE.to_owned()],
            "the fourth hour finishes it"
        );
        assert_eq!(state.buildings[INSTANCE].state, BuildingState::Operational);

        // The tier's requirements are open flags the caller supplies.
        assert_eq!(
            state.upgrade_building(INSTANCE, &BTreeSet::new(), &definitions),
            Err(BuildingError::RequirementsNotMet {
                id: INSTANCE.into(),
                missing: BTreeSet::from(["requirement.test.tier_2".to_owned()]),
            })
        );
        assert_eq!(state.buildings[INSTANCE].tier, FIRST_TIER);

        let mut met = BTreeSet::from(["requirement.test.tier_2".to_owned()]);
        assert_eq!(
            state.upgrade_building(INSTANCE, &met, &definitions),
            Ok(2),
            "requirements met, the building starts building its second tier"
        );
        let building = &state.buildings[INSTANCE];
        assert_eq!(building.state, BuildingState::UnderConstruction);
        assert_eq!(building.construction_hours_remaining, 8);
        assert_eq!(building.hp, 200);

        state.advance_construction(8);
        met.insert("requirement.test.tier_3".to_owned());
        assert_eq!(state.upgrade_building(INSTANCE, &met, &definitions), Ok(3));
        state.advance_construction(12);
        assert_eq!(state.buildings[INSTANCE].tier, TIER_CAP);

        // The cap is the ceiling even though nothing above it was authored.
        assert_eq!(
            state.upgrade_building(INSTANCE, &met, &definitions),
            Err(BuildingError::AtTopTier {
                id: INSTANCE.into(),
                tier: TIER_CAP,
            })
        );

        // A record that authors fewer tiers than the cap stops at its own top.
        let mut short = a_definition(SHED, 2, 0);
        short.tier_states = tiers(1);
        let short_registry = registry(vec![short]);
        state
            .place_building(
                "building_instance.test.shed",
                SHED,
                BEACH,
                &pirates(),
                &geography,
                &short_registry,
            )
            .expect("a one-tier building goes up");
        state.advance_construction(4);
        assert_eq!(
            state.upgrade_building("building_instance.test.shed", &met, &short_registry),
            Err(BuildingError::AtTopTier {
                id: "building_instance.test.shed".into(),
                tier: FIRST_TIER,
            })
        );
    }

    /// Damage, capture and ruin, with the Open item's two answers side by
    /// side: a capturable building knocked to zero waits to be taken, and a
    /// destroy-only one is a ruin the moment it falls.
    #[test]
    fn a_capturable_building_changes_hands_and_a_destroy_only_one_comes_down() {
        let geography = Geography::black_beach_vertical_slice();
        let mut destroy_only = a_definition(SHED, 2, 0);
        destroy_only.capture_rules = CaptureRules::DestroyOnly;
        let definitions = registry(vec![a_definition(WORKSHOP, 2, 1), destroy_only]);
        let mut state = a_campaign(&geography, &pirates());
        state
            .place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the workshop goes up");
        state
            .place_building(
                "building_instance.test.shed",
                SHED,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the shed goes up");
        state.advance_construction(4);

        assert_eq!(
            state.damage_building(INSTANCE, 40, &definitions),
            Ok(BuildingState::Damaged)
        );
        assert_eq!(state.buildings[INSTANCE].hp, 60);
        // Still standing: you take a building by beating it.
        assert!(matches!(
            state.capture_building(INSTANCE, &elves(), &definitions),
            Err(BuildingError::WrongState { .. })
        ));

        assert_eq!(
            state.damage_building(INSTANCE, 999, &definitions),
            Ok(BuildingState::Damaged),
            "a capturable building at zero is a shell, not yet a ruin"
        );
        assert_eq!(state.buildings[INSTANCE].hp, 0);
        assert_eq!(
            state.capture_building(INSTANCE, &pirates(), &definitions),
            Err(BuildingError::AlreadyHeld {
                id: INSTANCE.into(),
                faction_id: pirates(),
            })
        );
        state
            .capture_building(INSTANCE, &elves(), &definitions)
            .expect("the attacker walks in");
        let taken = &state.buildings[INSTANCE];
        assert_eq!(taken.state, BuildingState::Captured);
        assert_eq!(taken.faction_id, elves());
        assert_eq!(taken.hp, CAPTURE_HP_RESTORED);
        assert!(
            taken.is_working(),
            "a captured building works for whoever took it"
        );

        // Destroy-only: the same blow ends it, and nobody may take it.
        assert_eq!(
            state.damage_building("building_instance.test.shed", 999, &definitions),
            Ok(BuildingState::Ruined)
        );
        assert!(matches!(
            state.capture_building("building_instance.test.shed", &elves(), &definitions),
            Err(BuildingError::WrongState { .. })
        ));
        assert!(matches!(
            state.damage_building("building_instance.test.shed", 1, &definitions),
            Err(BuildingError::WrongState { .. })
        ));
        assert!(matches!(
            state.ruin_building("building_instance.test.shed"),
            Err(BuildingError::WrongState { .. })
        ));
        assert_eq!(
            state.damage_building("building_instance.test.missing", 1, &definitions),
            Err(BuildingError::UnknownBuilding {
                id: "building_instance.test.missing".into()
            })
        );
    }

    /// The seam B11 attaches to: only working buildings offer actor sockets,
    /// and the sockets come from the registry rather than the save.
    #[test]
    fn a_cell_offers_the_actor_sockets_of_the_buildings_working_on_it() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry(vec![a_definition(WORKSHOP, 2, 1)]);
        let mut state = a_campaign(&geography, &pirates());
        state
            .place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the workshop goes up");

        assert!(
            state.actor_sockets_at(BEACH, &definitions).is_empty(),
            "a half-built building has no working positions"
        );
        state.advance_construction(4);

        let sockets = state.actor_sockets_at(BEACH, &definitions);
        assert_eq!(sockets.len(), 2);
        assert!(sockets.iter().all(|(id, _)| id == INSTANCE));
        assert!(
            sockets
                .iter()
                .any(|(_, socket)| socket.kind == SocketKind::Spawn),
            "B11 materialises S7's arriving forces through these"
        );
        assert!(state.actor_sockets_at(TERRACE, &definitions).is_empty());
        assert_eq!(state.buildings_at(BEACH).len(), 1);

        state.ruin_building(INSTANCE).expect("it comes down");
        assert!(
            state.actor_sockets_at(BEACH, &definitions).is_empty(),
            "a ruin offers nothing"
        );
    }

    /// The save half: buildings survive serialize -> deserialize -> compare,
    /// and a save written before buildings existed still loads.
    #[test]
    fn buildings_round_trip_and_an_older_save_loads_with_bare_ground() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry(vec![a_definition(WORKSHOP, 2, 1)]);
        let mut state = a_campaign(&geography, &pirates());
        state
            .place_building(
                INSTANCE,
                WORKSHOP,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the workshop goes up");
        state.advance_construction(4);
        state
            .damage_building(INSTANCE, 30, &definitions)
            .expect("it takes a hit");

        let json = state.to_json();
        let reloaded = ExpeditionState::from_json(&json).expect("the save reloads");
        assert_eq!(reloaded, state);
        assert_eq!(reloaded.buildings[INSTANCE].hp, 70);
        assert_eq!(reloaded.to_json(), json, "byte-stable across a reload");

        let mut object: serde_json::Value = serde_json::from_str(&json).expect("a save is JSON");
        object
            .as_object_mut()
            .expect("a save is an object")
            .remove("buildings")
            .expect("today's save carries the field this test then removes");
        let older = ExpeditionState::from_json(&object.to_string())
            .expect("a save predating buildings still loads");
        assert!(older.buildings.is_empty());
    }
    /// C10: the authored records are the contract, and this holds Rust to
    /// them. `content/buildings/` is read through the same `validate` and the
    /// same `BuildingDefinitions::insert` the simulation uses -- the shape
    /// `every_authored_faction_record_loads_and_validates` set in `faction.rs`
    /// and C6 repeated for scenes. Content owns; Rust carries; a test holds
    /// them equal.
    ///
    /// Two failures it exists to catch. A record that stops deserializing --
    /// a socket kind renamed, `ruin_state` written as a bare string when it
    /// carries data -- fails at `serde_json::from_str`, before any rule runs.
    /// A record that deserializes but breaks a rule -- a Michael building that
    /// produces a person, a missing `capture_rules`, a socket outside the
    /// footprint -- fails at `validate`, with the same verdict
    /// `tools/src/validate.mjs` gives it.
    #[test]
    fn every_authored_building_record_loads_and_validates() {
        let building_directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/buildings/");
        let mut definitions = BuildingDefinitions::new();
        let mut from_filenames: BTreeSet<String> = BTreeSet::new();
        for entry in std::fs::read_dir(building_directory).expect("content/buildings/ is readable")
        {
            let path = entry.expect("a readable directory entry").path();
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("the building file is readable");
            let record: BuildingDefinition = serde_json::from_str(&text).unwrap_or_else(|error| {
                panic!("{} is a BuildingDefinition: {error}", path.display())
            });
            record
                .validate()
                .unwrap_or_else(|error| panic!("{} fails validate: {error:?}", path.display()));
            let stem = path
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("a UTF-8 filename");
            assert_eq!(
                record.id,
                format!("{BUILDING_ID_PREFIX}{stem}"),
                "{} must be named after the record it carries",
                path.display()
            );
            assert!(
                from_filenames.insert(record.id.clone()),
                "{} is a second record for {}",
                path.display(),
                record.id
            );
            definitions
                .insert(record)
                .unwrap_or_else(|error| panic!("{} does not load: {error:?}", path.display()));
        }

        assert!(
            !from_filenames.is_empty(),
            "content/buildings/ must author at least one record; C10 is what fills it"
        );
        assert_eq!(
            definitions
                .ids()
                .map(str::to_owned)
                .collect::<BTreeSet<_>>(),
            from_filenames,
            "the registry's IDs must be exactly the directory's"
        );

        // The two halves of brief section 5.6, both authored on purpose so the
        // rule is proven by the content and not only by a fixture: at least one
        // record Michael may raise, and at least one legal human-role rule
        // standing somewhere else.
        let michaels: Vec<&BuildingDefinition> = definitions
            .ids()
            .filter_map(|id| definitions.get(id))
            .filter(|record| record.faction_compatibility.contains(&ConceptKey::Michael))
            .collect();
        assert!(
            !michaels.is_empty(),
            "content/buildings/ must author a building Michael's faction can raise"
        );
        for record in michaels {
            assert!(
                !record
                    .production
                    .iter()
                    .any(|rule| matches!(rule.output, ProductionOutput::HumanRole { .. })),
                "{} produces a person for Michael's faction",
                record.id
            );
        }
        let supported_roles = definitions
            .ids()
            .filter_map(|id| definitions.get(id))
            .filter(|record| {
                record
                    .production
                    .iter()
                    .any(|rule| matches!(rule.output, ProductionOutput::HumanRole { .. }))
            })
            .count();
        assert_eq!(
            supported_roles, 1,
            "exactly one authored record carries the legal human-role case; it is what proves the rule admits recruitment support"
        );
    }
}
