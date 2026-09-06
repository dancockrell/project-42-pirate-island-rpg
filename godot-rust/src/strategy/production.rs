//! S13: what Captain Michael's faction *makes*. `docs/SHIP_PLAN.md` section 7,
//! card S13; brief section 5.4 ("Animal-form machinery"), section 5.6
//! ("Buildings do not manufacture women"), section 5.10 ("Human scarcity and
//! military doctrine") and section 18's per-machine asset field list.
//!
//! The card's sentence is the whole of this file: **machines, capacity and
//! services -- never people.** S3 made that a load-time refusal for a
//! building's `production` list. This file makes it the same refusal one level
//! up, over the *actor kit*: the list of unit records a faction may field. A
//! kit is the other door through which a manufactured person could walk in, so
//! it is shut with the same rule and proved by the same test.
//!
//! Three types, three lifetimes, exactly as S1 split factions and S3 split
//! buildings:
//!
//! * [`MachineDefinition`] is content -- brief section 18's asset fields,
//!   authored once in `content/machines/<id>.json` (a C card, not this one),
//!   loaded into a [`MachineDefinitions`] registry, never mutated.
//! * [`MachineInstance`] is save data. It carries **only what a save must
//!   remember** -- how much fuel and water this particular machine has left and
//!   how badly it is hurt -- and reads every authored field through the
//!   `machine.<...>` ID it names, so content and save cannot drift into two
//!   answers about a footprint.
//! * [`ActorKit`] is neither: it is a *reading* of the `actor_kit` list S1
//!   already put on [`FactionDefinition`], resolved against the machine
//!   registry. It stores nothing. Giving this lane its own second list of unit
//!   IDs would be two owners for one question.
//!
//! ## The eight families
//!
//! [`MachineFamily`] is closed, and it is the brief's list at section 5.4
//! verbatim: mechanical dogs, mechanical cavalry, mechanical bears, mechanical
//! elephants, walkers, steam wagons, rockets, steam airships. Closed because
//! the brief calls these "the faction's principal candidate machine families"
//! -- a named design decision, not an Open category. Contrast the resource keys
//! beside them, which brief section 20 leaves Open and which stay strings here
//! as they do in S1 and S3: **fuel is a key a production rule names**, not a
//! variant of an enum this file refuses to invent.
//!
//! ## What section 5.10 is, and is not, in this lane
//!
//! Section 5.10 is *Derived*: because women cannot be manufactured, the faction
//! should value human life, and its AI should often prefer machines taking the
//! initial exposure. [`ExposurePolicy`] records that as a **fact about the
//! faction record** and nothing more. Nothing in this crate acts on it today:
//! S5's `choose_goals` does not read it, no weight in `strategy/utility.rs`
//! moved, and no force in `strategy/force.rs` is dispatched differently. It is
//! a fact waiting for the lanes entitled to spend it -- S7 when it chooses what
//! marches first, and the B card that closes S5's registry seam so that scoring
//! reads a faction record at all. Writing the preference into the utility
//! weights here would have been this lane deciding S5's shape from outside it.
//!
//! ## Determinism
//!
//! No draw is taken in this file. Every map is a `BTreeMap`, every set a
//! `BTreeSet`, every new field carries `serde(default)`, and
//! [`ExpeditionState::produce_machine`] rejects before it mutates anything.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, ExpeditionState, require_stable_id};
use crate::strategy::building::{
    BUILDING_INSTANCE_ID_PREFIX, BuildingDefinitions, BuildingError, BuildingState,
    ProductionOutput,
};
use crate::strategy::faction::{ConceptKey, FactionDefinition};
use crate::strategy::tick::StrategicEvent;

/// The prefix every authored machine *definition* ID carries, as `building.`
/// is to a building and `faction.` to a faction.
pub const MACHINE_ID_PREFIX: &str = "machine.";

/// The prefix every built machine *instance* ID carries. Content and save data
/// live in separate namespaces for the reason S3 gave: a save that confused the
/// two would be a save that could overwrite content by being loaded.
pub const MACHINE_INSTANCE_ID_PREFIX: &str = "machine_instance.";

/// The machine families brief section 5.4 names, and no others.
///
/// A closed enum rather than an open string, unlike the resource keys this file
/// spends: section 5.4 is an *Accepted direction* listing the faction's
/// principal candidate families, so the set is a decision that has been made,
/// and a typo in it should not compile. Section 20 leaves the resource list
/// Open; it leaves this list closed.
///
/// The variants carry no functions, roles or statistics. Section 5.4's
/// "potential functions" lists are potential, and inventing a role table from
/// them would be this file deciding what the brief left as candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineFamily {
    /// Section 5.4: scouting, alarm, and the small end of the animal forms.
    MechanicalDog,
    /// "A faction-level troop family, not a mandatory personal mount system."
    MechanicalCavalry,
    /// Compact heavy labour, close defence, protecting engineers.
    MechanicalBear,
    /// Heavy transport, crane work, bridge construction, walker recovery.
    MechanicalElephant,
    /// "Each walker needs a functional reason" -- the articulated industrial
    /// machine, used where an animal form or a wagon does not fit.
    Walker,
    /// "More likely than recognizable cars": freight, transport, medical
    /// evacuation, mobile workshops.
    SteamWagon,
    /// Unguided, smoky, weather-sensitive, dangerous to store.
    Rocket,
    /// Lifting gas for lift, steam for everything else. Needs moorings, fuel,
    /// water, lift gas, crews and weather knowledge.
    Airship,
}

impl MachineFamily {
    /// Every family, in declaration order. One list, so a reader, a validator
    /// and a future content loader cannot disagree about how many there are.
    pub const ALL: [MachineFamily; 8] = [
        MachineFamily::MechanicalDog,
        MachineFamily::MechanicalCavalry,
        MachineFamily::MechanicalBear,
        MachineFamily::MechanicalElephant,
        MachineFamily::Walker,
        MachineFamily::SteamWagon,
        MachineFamily::Rocket,
        MachineFamily::Airship,
    ];

    /// The authored key, which is what content writes and what
    /// `serde(rename_all = "snake_case")` produces. The card names these eight
    /// strings; this method is the one place they exist.
    pub fn as_key(self) -> &'static str {
        match self {
            MachineFamily::MechanicalDog => "mechanical_dog",
            MachineFamily::MechanicalCavalry => "mechanical_cavalry",
            MachineFamily::MechanicalBear => "mechanical_bear",
            MachineFamily::MechanicalElephant => "mechanical_elephant",
            MachineFamily::Walker => "walker",
            MachineFamily::SteamWagon => "steam_wagon",
            MachineFamily::Rocket => "rocket",
            MachineFamily::Airship => "airship",
        }
    }

    /// The authored key back to a family, or `None` for a word that is not one.
    pub fn from_key(key: &str) -> Option<Self> {
        MachineFamily::ALL
            .into_iter()
            .find(|family| family.as_key() == key)
    }
}

impl Default for MachineFamily {
    /// Only so [`ProductionOutput`] and [`MachineDefinition`] can be
    /// constructed by `Default` for `#[serde(default)]`, exactly as
    /// [`ConceptKey`]'s `Default` exists in `strategy/faction.rs`. A record
    /// always states its own family, and a production rule whose named machine
    /// record disagrees with the rule's family does not produce -- see
    /// [`ProductionError::FamilyMismatch`]. Nothing chooses this value.
    fn default() -> Self {
        MachineFamily::MechanicalDog
    }
}

/// What went wrong producing a machine, or reading a faction's actor kit.
///
/// A module-local error enum, following `FactionError`, `ForceError` and
/// `BuildingError`: one exhaustive list per subject, so a caller matches on the
/// failures of the thing it asked about rather than on a crate-wide grab bag.
#[derive(Clone, Debug, PartialEq)]
pub enum ProductionError {
    /// The ID is not a stable ID at all.
    MalformedId(ExpeditionError),
    /// A well-shaped ID in the wrong namespace.
    WrongIdPrefix {
        found: String,
        expected: &'static str,
    },
    /// This campaign already carries a machine under this instance ID.
    Duplicate { id: String },
    /// The machine registry does not carry this `machine.<...>` record.
    UnknownDefinition { id: String },
    /// Two records claiming the same `machine.<...>` ID.
    DuplicateDefinition { id: String },
    /// **Brief section 5.6, at the level of the actor kit.** Captain Michael's
    /// faction fields machines. An entry in his `actor_kit` that the machine
    /// registry does not know is not a machine, and this lane will not guess
    /// that it might be one: it is refused, by name, at load.
    MichaelActorKitNamesANonMachine {
        faction_id: String,
        entry_id: String,
    },
    /// No building standing under this instance ID.
    UnknownBuilding { building_instance_id: String },
    /// The building's definition carries no production rule at that index.
    NoSuchRule {
        building_instance_id: String,
        rule_index: usize,
    },
    /// The building is not [`BuildingState::Operational`]. A building still
    /// under construction, damaged or ruined does not make anything -- and
    /// neither, in this lane, does a `Captured` one: whether a taker inherits a
    /// working production line is part of brief section 20's still-Open
    /// "capture versus destruction rules by building type", so this refuses
    /// rather than assumes.
    NotOperational {
        building_instance_id: String,
        state: BuildingState,
    },
    /// The building stands below the tier the rule needs.
    BelowMinimumTier {
        building_instance_id: String,
        tier: u32,
        minimum_tier: u32,
    },
    /// The rule at that index makes capacity, a service, or -- refused long
    /// before here by S3's loader -- a person. It does not make a machine.
    NotAMachineRule {
        building_instance_id: String,
        rule_id: String,
    },
    /// The rule's `output_key` names a machine record whose family is not the
    /// family the rule claims to produce. One of the two is wrong and this lane
    /// cannot tell which, so it produces neither.
    FamilyMismatch {
        rule_id: String,
        def_id: String,
        rule_family: MachineFamily,
        definition_family: MachineFamily,
    },
    /// The building belongs to a faction this save carries no state for. There
    /// is no stockpile to charge, so there is nothing to produce from.
    UnknownFaction { faction_id: String },
    /// The faction does not hold enough of what the rule costs. `key` is an
    /// open resource string from content (brief section 20 leaves the resource
    /// list Open) -- fuel is one such key, not a variant of anything.
    InsufficientResource {
        faction_id: String,
        key: String,
        held: u32,
        needed: u32,
    },
    /// Reading the building went wrong in a way `strategy/building.rs` already
    /// has a word for. Carried rather than restated, so the two files keep one
    /// vocabulary.
    Building(BuildingError),
}

impl From<BuildingError> for ProductionError {
    fn from(error: BuildingError) -> Self {
        ProductionError::Building(error)
    }
}

/// One authored machine record: brief section 18's "Animal automata and
/// vehicles need" list, in the brief's order and with the brief's names.
///
/// The list is copied field for field rather than summarised, for S3's reason:
/// the design document and the code stay one vocabulary, and a reviewer can
/// diff them by eye. Where a field's *unit* is a decision nobody has made yet
/// -- and brief section 20 leaves the island's geometry and the resource list
/// Open -- the field is an abstract count or an open string key, never an
/// invented metre or an enum of goods.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineDefinition {
    /// Section 18, "Stable IDs": `machine.<something>`.
    pub id: String,
    /// Which of section 5.4's eight families this record is one of. Not a
    /// section 18 field: it is the tie between the authored asset and the
    /// closed family list, and [`ExpeditionState::produce_machine`] refuses a
    /// production rule whose family disagrees with it.
    #[serde(default)]
    pub family: MachineFamily,
    /// "Operational footprint" -- what the machine takes up when it is working,
    /// in the same abstract shares of a cell's capacity S3 counts a building's
    /// envelope in. B11 and O3 own real geometry.
    #[serde(default)]
    pub operational_footprint_cells: u32,
    /// "Navigation width" -- how wide a way it needs. Same abstract units.
    #[serde(default)]
    pub navigation_width_cells: u32,
    /// "Turning clearance" -- how much room it needs to come about. An elephant
    /// and a dog differ here more than they differ in footprint.
    #[serde(default)]
    pub turning_clearance_cells: u32,
    /// "Maximum slope", `0..=255` as an authored share of the steepest ground.
    /// The graph carries no slope yet -- S3 found the same and left
    /// `BuildingDefinition::maximum_slope` in exactly this state -- so this is
    /// stored and not yet compared against anything.
    #[serde(default)]
    pub maximum_slope: u8,
    /// "Valid route types": the authored `travelMode` vocabulary from
    /// `content/world/*.world_cell.json` (`on_foot`, `safe_road`,
    /// `jungle_edge`), as open strings. Strings and not
    /// [`crate::geography::RouteKind`] because the vocabulary is content's and
    /// a machine record may name a way this crate has not met; S7 is the lane
    /// that will match these against a road it is about to march down.
    #[serde(default)]
    pub valid_route_types: BTreeSet<String>,
    /// "Bridge requirements": open flags a crossing must satisfy before this
    /// machine may use it. Section 5.11 puts reinforced bridges in the
    /// faction's terrain signature; what reinforces one is not decided here.
    #[serde(default)]
    pub bridge_requirements: BTreeSet<String>,
    /// "Crew or handler requirements": how many people it takes to work this
    /// machine. **This is the number section 5.10 is about.** It is a cost in
    /// the scarcest thing the faction has, which is why a machine that needs
    /// nobody and a machine that needs a crew of six are not interchangeable
    /// however similar their footprints.
    #[serde(default)]
    pub crew_or_handler_requirement: u32,
    /// "Fuel and water requirements", first half: how much fuel a whole machine
    /// of this type carries. The resource *key* it burns is named by the
    /// production rule and by whatever refuels it, not here -- brief section 20
    /// leaves the resource list Open and this file defines no category.
    #[serde(default)]
    pub fuel_requirement: u32,
    /// "Fuel and water requirements", second half. Steam needs water as much as
    /// it needs fuel, and section 5.11 lists fuel *and water* stations in the
    /// faction's terrain signature, so they are two numbers rather than one
    /// "supply".
    #[serde(default)]
    pub water_requirement: u32,
    /// "Repair sockets": the named points a repair attaches to. Section 18's
    /// accepted list requires machines to "retain future-ready pivots, sockets,
    /// rigs, and metadata", so these are authored now and consumed by the art
    /// and repair lanes later.
    #[serde(default)]
    pub repair_sockets: BTreeSet<String>,
    /// "Local and offscreen representations": what this machine looks like
    /// standing in a room, and what it counts as inside one of S7's aggregate
    /// forces. Authored prose until there is a materialiser to consume it,
    /// following S3's `dungeon_relationship`.
    #[serde(default)]
    pub local_and_offscreen_representations: String,
    /// "Wreck footprint": what is left on the ground when it dies. Abstract
    /// cells again, and deliberately separate from
    /// `operational_footprint_cells`, because a dead elephant blocks a road
    /// differently from a live one.
    #[serde(default)]
    pub wreck_footprint_cells: u32,
    /// "Salvage value": what a wreck is worth to whoever works it, in the same
    /// open resource keys everything else in the strategic layer is counted in.
    /// Section 5.11 lists salvage yards in the faction's terrain signature;
    /// this is what feeds one.
    #[serde(default)]
    pub salvage_value: BTreeMap<String, u32>,
}

impl MachineDefinition {
    /// The one gate between an authored machine record and the simulation.
    /// Refuses; never repairs.
    pub fn validate(&self) -> Result<(), ProductionError> {
        require_stable_id("machine.id", &self.id).map_err(ProductionError::MalformedId)?;
        if !self.id.starts_with(MACHINE_ID_PREFIX) {
            return Err(ProductionError::WrongIdPrefix {
                found: self.id.clone(),
                expected: MACHINE_ID_PREFIX,
            });
        }
        Ok(())
    }
}

/// The loaded machine records, keyed by ID. A C card authors the files; this is
/// what the simulation reads. Insertion validates, so an invalid record cannot
/// be in a registry at all -- [`BuildingDefinitions`]'s contract, kept
/// identical on purpose.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineDefinitions {
    by_id: BTreeMap<String, MachineDefinition>,
}

impl MachineDefinitions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates, refuses a duplicate, then stores.
    pub fn insert(&mut self, definition: MachineDefinition) -> Result<(), ProductionError> {
        definition.validate()?;
        if self.by_id.contains_key(&definition.id) {
            return Err(ProductionError::DuplicateDefinition {
                id: definition.id.clone(),
            });
        }
        self.by_id.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&MachineDefinition> {
        self.by_id.get(id)
    }

    pub fn require(&self, id: &str) -> Result<&MachineDefinition, ProductionError> {
        self.by_id
            .get(id)
            .ok_or_else(|| ProductionError::UnknownDefinition { id: id.to_owned() })
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

/// How a faction spends the first exposure of a fight. Brief section 5.10,
/// recorded as a fact and acted on by nothing yet.
///
/// **What this is.** Section 5.10 is *Derived*: because women cannot be
/// manufactured and important recruits carry relationships and history, the
/// faction should value human life, and its AI should often prefer machines
/// taking the initial exposure, automata scouting dangerous routes, and remote
/// fire before committing scarce women. That is a doctrine about *cost*, not a
/// strategy: it does not make the faction passive, it makes machinery a force
/// multiplier for a smaller and unusually valuable human population.
///
/// **What this is not.** It is not a behaviour. No goal is scored differently
/// for carrying it, no force is dispatched differently, and
/// `strategy/utility.rs` gained no term. The lane that spends it is the one
/// that owns what marches first (S7) or the one that closes S5's registry seam
/// so that scoring reads a faction record at all.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExposurePolicy {
    /// Brief section 5.10. Machines take the initial exposure; people are the
    /// scarce thing, spent last.
    MachinesFirst,
    /// Every other faction. Not "people first" -- the brief derives a doctrine
    /// for one faction and does not derive its opposite for the rest, and
    /// naming the absence is more honest than inventing five doctrines to fill
    /// it. Provisional doctrines (brief section 6) land in S15, not here.
    #[default]
    Unstated,
}

impl ExposurePolicy {
    /// The policy a faction record carries, from its concept key alone.
    ///
    /// Derived rather than authored or saved, because it is already implied by
    /// something the record states: section 5.10 derives the doctrine *from*
    /// the fact that this faction cannot manufacture people. Storing it beside
    /// the concept key would be a second copy of one decision, free to disagree
    /// with the first.
    pub fn of(concept_key: ConceptKey) -> Self {
        match concept_key {
            ConceptKey::Michael => ExposurePolicy::MachinesFirst,
            _ => ExposurePolicy::Unstated,
        }
    }
}

/// One entry of a faction's actor kit, once it has been read against the
/// machine registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKitEntry {
    /// A `machine.<...>` record the registry carries, and the family it belongs
    /// to.
    Machine { id: String, family: MachineFamily },
    /// An entry the machine registry does not carry. For five of the six
    /// concepts that is an ordinary actor -- a person, an animal, a thing this
    /// crate has no other loader for yet. For [`ConceptKey::Michael`] it never
    /// loads: see [`ActorKit::of`].
    NonMachine { id: String },
}

impl ActorKitEntry {
    /// The entry's ID, whichever kind it is.
    pub fn id(&self) -> &str {
        match self {
            ActorKitEntry::Machine { id, .. } | ActorKitEntry::NonMachine { id } => id,
        }
    }

    /// The family, for a machine entry.
    pub fn family(&self) -> Option<MachineFamily> {
        match self {
            ActorKitEntry::Machine { family, .. } => Some(*family),
            ActorKitEntry::NonMachine { .. } => None,
        }
    }
}

/// A faction's actor kit as this lane reads it: which unit records it may
/// field, what each of them is, and how it spends exposure.
///
/// **This type stores nothing.** `FactionDefinition::actor_kit` is the list, S1
/// owns it, C9 authors it, and [`ActorKit::of`] is a reading of it against the
/// machine registry. A second stored list of unit IDs would be two owners for
/// one question, and two owners drift.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorKit {
    /// `faction.<concept_key>`, from the record this kit was read from.
    pub faction_id: String,
    /// Every entry of `FactionDefinition::actor_kit`, resolved, in the
    /// `BTreeSet`'s order.
    pub entries: Vec<ActorKitEntry>,
    /// Brief section 5.10, as a fact. Read by nothing that acts.
    pub exposure_policy: ExposurePolicy,
}

impl ActorKit {
    /// Read a faction record's actor kit against the machine registry.
    ///
    /// **The card's first Done-when, at the level of the kit.** For
    /// [`ConceptKey::Michael`] every entry must resolve to a [`MachineFamily`]:
    /// his faction fields machines, and an entry that is not a machine record
    /// is refused by name rather than admitted as "probably a unit". A human
    /// role cannot come in through the kit any more than it can through a
    /// building's `production` list, which S3 already shut.
    ///
    /// For the other five concepts a non-machine entry is ordinary and loads.
    /// The refusal is Michael's because the reason is Michael's: brief section
    /// 5.6 is about what *his* buildings and lists may contain, and the other
    /// five factions have populations that were never manufactured to begin
    /// with.
    pub fn of(
        definition: &FactionDefinition,
        machines: &MachineDefinitions,
    ) -> Result<Self, ProductionError> {
        let michaels = definition.concept_key == ConceptKey::Michael;
        let mut entries = Vec::with_capacity(definition.actor_kit.len());
        for id in &definition.actor_kit {
            match machines.get(id) {
                Some(machine) => entries.push(ActorKitEntry::Machine {
                    id: id.clone(),
                    family: machine.family,
                }),
                None if michaels => {
                    return Err(ProductionError::MichaelActorKitNamesANonMachine {
                        faction_id: definition.id.clone(),
                        entry_id: id.clone(),
                    });
                }
                None => entries.push(ActorKitEntry::NonMachine { id: id.clone() }),
            }
        }
        Ok(Self {
            faction_id: definition.id.clone(),
            entries,
            exposure_policy: ExposurePolicy::of(definition.concept_key),
        })
    }

    /// The families this kit can field, deduplicated and ordered.
    pub fn families(&self) -> BTreeSet<MachineFamily> {
        self.entries
            .iter()
            .filter_map(ActorKitEntry::family)
            .collect()
    }
}

/// One machine that exists on the board: the save-data half.
///
/// **What this carries and why.** Not one of brief section 18's asset fields is
/// copied here. The footprint, navigation width, turning clearance, slope,
/// route types, bridge requirements, crew, fuel and water requirements, repair
/// sockets, wreck footprint and salvage value are read from the
/// [`MachineDefinition`] that `def_id` names -- one owner, referenced by ID --
/// exactly as `BuildingInstance` refuses to copy its envelope. What a save must
/// remember is only what has *changed* about this particular machine since it
/// was built: how much fuel and water are left in it, and how hurt it is. Three
/// numbers, and every one of them is a fact the definition could not supply.
///
/// The family is not stored either: it is the family of `def_id`'s definition,
/// and a second copy could disagree with the first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineInstance {
    /// `machine_instance.<something>`; the key this record sits under in
    /// [`ExpeditionState::machines`].
    pub id: String,
    /// The `machine.<...>` record this was built from.
    pub def_id: String,
    /// `faction.<concept_key>` -- whoever's building made it.
    pub faction_id: String,
    /// Where it stands: the cell its building stands on. Machines do not move
    /// in this lane; S7 owns movement, and a machine that marches does so
    /// inside one of its forces.
    pub cell_id: String,
    /// The `building_instance.<...>` that made it. Kept because brief section
    /// 17 asks a save to preserve what happened, and "which yard built this" is
    /// the kind of provenance a salvage or repair lane will want.
    #[serde(default)]
    pub built_by_building_instance_id: String,
    /// Fuel in it right now. Starts full, at the definition's
    /// `fuel_requirement`.
    #[serde(default)]
    pub fuel_remaining: u32,
    /// Water in it right now. Starts full, at the definition's
    /// `water_requirement`.
    #[serde(default)]
    pub water_remaining: u32,
    /// How hurt it is. Zero is whole; nothing in this lane raises it, because
    /// nothing in this lane shoots at a machine.
    #[serde(default)]
    pub damage: u32,
}

impl ExpeditionState {
    /// Build one machine from one of a building's production rules. The one way
    /// [`ExpeditionState::machines`] gains an entry.
    ///
    /// **Rejects before mutating** on every failure below, in this order, so a
    /// refused production leaves the faction's stockpile, the building and the
    /// machine map exactly as it found them:
    ///
    /// * a malformed or wrongly-prefixed machine instance ID, or one this
    ///   campaign already carries;
    /// * a building instance nobody has placed, or whose definition the
    ///   registry has lost;
    /// * a building that is not [`BuildingState::Operational`], or that stands
    ///   below the rule's `minimum_tier`;
    /// * a rule index the definition does not carry, or a rule whose output is
    ///   not [`ProductionOutput::Machine`];
    /// * a machine record the registry does not carry, or one whose family
    ///   disagrees with the rule's;
    /// * a faction this save carries no state for;
    /// * a stockpile that does not cover **every** key in the rule's `cost`.
    ///
    /// The last of those is the one worth reading twice. `cost` is a map of
    /// *open resource strings* to counts -- brief section 20 leaves the
    /// resource list Open, so fuel is a key a rule names, like
    /// `resource.open.fuel`, and not a variant of an enum this file refuses to
    /// invent. Every key is checked before any key is deducted, so a rule
    /// costing two resources of which the faction holds one takes neither.
    ///
    /// Returns the new machine and the [`StrategicEvent::MachineProduced`] for
    /// the caller to journal, following `dispatch_force`: production ordered
    /// from outside the hour is not the hour's event, so this method does not
    /// journal itself.
    ///
    /// No draw is taken. The same state, the same rule and the same registries
    /// produce the same machine.
    pub fn produce_machine(
        &mut self,
        machine_instance_id: &str,
        building_instance_id: &str,
        rule_index: usize,
        definitions: &BuildingDefinitions,
        machines: &MachineDefinitions,
    ) -> Result<(MachineInstance, StrategicEvent), ProductionError> {
        require_stable_id("machine.instance_id", machine_instance_id)
            .map_err(ProductionError::MalformedId)?;
        if !machine_instance_id.starts_with(MACHINE_INSTANCE_ID_PREFIX) {
            return Err(ProductionError::WrongIdPrefix {
                found: machine_instance_id.to_owned(),
                expected: MACHINE_INSTANCE_ID_PREFIX,
            });
        }
        if self.machines.contains_key(machine_instance_id) {
            return Err(ProductionError::Duplicate {
                id: machine_instance_id.to_owned(),
            });
        }

        let building = self.buildings.get(building_instance_id).ok_or_else(|| {
            ProductionError::UnknownBuilding {
                building_instance_id: building_instance_id.to_owned(),
            }
        })?;
        if building.state != BuildingState::Operational {
            return Err(ProductionError::NotOperational {
                building_instance_id: building_instance_id.to_owned(),
                state: building.state,
            });
        }
        let definition = definitions.require(&building.def_id)?;
        let rule =
            definition
                .production
                .get(rule_index)
                .ok_or_else(|| ProductionError::NoSuchRule {
                    building_instance_id: building_instance_id.to_owned(),
                    rule_index,
                })?;
        if building.tier < rule.minimum_tier {
            return Err(ProductionError::BelowMinimumTier {
                building_instance_id: building_instance_id.to_owned(),
                tier: building.tier,
                minimum_tier: rule.minimum_tier,
            });
        }

        let ProductionOutput::Machine {
            family: rule_family,
        } = rule.output
        else {
            return Err(ProductionError::NotAMachineRule {
                building_instance_id: building_instance_id.to_owned(),
                rule_id: rule.id.clone(),
            });
        };
        let machine = machines.require(&rule.output_key)?;
        if machine.family != rule_family {
            return Err(ProductionError::FamilyMismatch {
                rule_id: rule.id.clone(),
                def_id: machine.id.clone(),
                rule_family,
                definition_family: machine.family,
            });
        }

        let faction_id = building.faction_id.clone();
        let cell_id = building.cell_id.clone();
        let cost = rule.cost.clone();
        // Every key is checked before any key is spent, and nothing below this
        // line can fail. S16 gave the check-then-spend its own function rather
        // than a second copy: the hourly step charges a capacity or service
        // rule the same way this charges a machine rule, and two spellings of
        // "can this faction afford it" would be two answers.
        charge_production_cost(self, &faction_id, &cost)?;

        let instance = MachineInstance {
            id: machine_instance_id.to_owned(),
            def_id: machine.id.clone(),
            faction_id: faction_id.clone(),
            cell_id,
            built_by_building_instance_id: building_instance_id.to_owned(),
            fuel_remaining: machine.fuel_requirement,
            water_remaining: machine.water_requirement,
            damage: 0,
        };
        self.machines.insert(instance.id.clone(), instance.clone());
        let building = self
            .buildings
            .get_mut(building_instance_id)
            .expect("the same key was just read");
        building.machines_produced = building.machines_produced.saturating_add(1);

        Ok((
            instance,
            StrategicEvent::MachineProduced {
                faction_id,
                family: rule_family,
                building_instance_id: building_instance_id.to_owned(),
                day: self.campaign_day,
            },
        ))
    }

    /// Every machine this faction has standing, in instance-ID order.
    pub fn machines_of(&self, faction_id: &str) -> Vec<&MachineInstance> {
        self.machines
            .values()
            .filter(|machine| machine.faction_id == faction_id)
            .collect()
    }
}

/// Why a production rule that came due did not yield.
///
/// A closed enum rather than a formatted sentence, because these end up in the
/// save: [`StrategicEvent::ProductionSkipped`] is journalled by S11, and a
/// reader a year from now should be able to tell "the yard is out of fuel"
/// from "content names a machine record nobody authored" without parsing
/// prose. The last variant carries the [`ProductionError`]'s own words for the
/// failures that are neither, so nothing is silently swallowed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionSkipReason {
    /// The faction does not hold enough of one of the rule's `cost` keys.
    /// **The stockpile was not touched**: every key is checked before any key
    /// is spent, so a rule costing two resources of which the faction holds
    /// one takes neither and no count goes negative.
    ///
    /// `key` is an open resource string from content -- brief section 20
    /// leaves the resource list Open -- and this lane names none of them.
    InsufficientResource { key: String, held: u32, needed: u32 },
    /// The rule's `output_key` names a `machine.<...>` record the registry
    /// handed to the tick does not carry. Expected today and said plainly:
    /// C10's `building.machine_shop` names the Open placeholder resource key
    /// in its machine rule, C14 is the card that authors `content/machines/`,
    /// and the bridge hands the tick an empty machine registry until a later
    /// lane forwards those records through the port.
    UnknownMachineRecord { def_id: String },
    /// Anything else [`ExpeditionState::produce_machine`] refused, in its own
    /// vocabulary. A rule whose family disagrees with the record it names, a
    /// duplicate instance ID, a building the registry lost between the sweep
    /// and the rule: each is a fault worth reading, and none of them is worth
    /// a variant here until something acts on it differently.
    Refused { detail: String },
}

impl ProductionSkipReason {
    /// The reason a refusal gives, from the refusal.
    fn of(error: &ProductionError) -> Self {
        match error {
            ProductionError::InsufficientResource {
                key, held, needed, ..
            } => ProductionSkipReason::InsufficientResource {
                key: key.clone(),
                held: *held,
                needed: *needed,
            },
            ProductionError::UnknownDefinition { id } => {
                ProductionSkipReason::UnknownMachineRecord { def_id: id.clone() }
            }
            other => ProductionSkipReason::Refused {
                detail: format!("{other:?}"),
            },
        }
    }
}

/// Charge one production rule's `cost` to one faction's stockpile: **every key
/// or none**, and never below zero.
///
/// The one owner of "can this faction afford this rule, and if so, spend it".
/// [`ExpeditionState::produce_machine`] charges a machine rule through it and
/// [`advance_production`] charges a capacity or service rule through the same
/// function, because a second spelling of the check would be a second answer.
///
/// `cost` is a map of *open resource strings* to counts. Brief section 20
/// leaves the resource list Open, so fuel is a key a rule names and not a
/// variant of an enum this crate refuses to invent.
fn charge_production_cost(
    state: &mut ExpeditionState,
    faction_id: &str,
    cost: &BTreeMap<String, u32>,
) -> Result<(), ProductionError> {
    let faction =
        state
            .factions
            .get(faction_id)
            .ok_or_else(|| ProductionError::UnknownFaction {
                faction_id: faction_id.to_owned(),
            })?;
    // Every key is checked before any key is spent.
    for (key, needed) in cost {
        let held = faction.resources.get(key).copied().unwrap_or(0);
        if held < *needed {
            return Err(ProductionError::InsufficientResource {
                faction_id: faction_id.to_owned(),
                key: key.clone(),
                held,
                needed: *needed,
            });
        }
    }

    // Nothing below this line can fail.
    let faction = state
        .factions
        .get_mut(faction_id)
        .expect("the same key was just read");
    for (key, needed) in cost {
        let held = faction
            .resources
            .get_mut(key)
            .expect("every key was just found to cover its cost");
        *held -= needed;
    }
    Ok(())
}

/// The instance ID one yield gets, derived and never invented.
///
/// Four parts, and no clock among them: the yard that made it, the rule that
/// made it, how many machines that yard had already made (the counter that
/// makes the ID unique -- `machines_produced` rises with every success and
/// never falls), and the hour's `strategic.economy` draw.
///
/// The draw is in there because it is what this lane was given. [`PURPOSES`]
/// reserves one draw per faction per hour for production, S4 folds it into the
/// determinism witness whether or not anything produces, and folding it into
/// the ID as well means the hour's randomness is *spent by the thing it is
/// reserved for* rather than being a number nobody reads. It also means a
/// change to the draw sequence shows up in the save as a differently named
/// machine, instead of nowhere.
///
/// [`PURPOSES`]: crate::strategy::tick::PURPOSES
fn machine_instance_id(
    building_instance_id: &str,
    rule_index: usize,
    ordinal: u32,
    draw: u64,
) -> String {
    let yard = building_instance_id
        .strip_prefix(BUILDING_INSTANCE_ID_PREFIX)
        .unwrap_or(building_instance_id);
    format!("{MACHINE_INSTANCE_ID_PREFIX}{yard}.{rule_index}.{ordinal}.{draw:016x}")
}

/// **S16: one hour of one faction's production.** Every timer-driven rule of
/// every building that faction has standing counts one hour off, and the ones
/// that reach zero yield.
///
/// Called once per faction per hour from
/// [`run_hour`](crate::strategy::tick::run_hour), inside the loop that makes
/// that faction's draws, with the `strategic.economy` draw that loop just made.
/// **The draw is made every hour whether or not anything produces** -- it is
/// made and folded into `StrategicClock::draw_digest` before this is called, by
/// the same code that makes the other six -- so a faction that builds nothing,
/// a faction that builds something, and a faction that cannot afford what it
/// builds all leave the hash contract exactly where they found it.
///
/// What a rule has to be for its timer to run:
///
/// * its building is [`BuildingState::Operational`]. Not merely
///   [`BuildingInstance::is_working`], which also admits `Captured`:
///   `produce_machine` refuses a captured building because whether a taker
///   inherits a working production line is brief section 20's still-Open
///   "capture versus destruction rules by building type", and a timer that ran
///   on a captured yard would burn its interval down to that refusal over and
///   over. The timer waits with the decision.
/// * `interval_hours > 0`. Zero means "not timer-driven": a standing
///   capability something else draws on, which is what every rule authored
///   before this card means.
/// * the building stands at or above the rule's `minimum_tier`. A tier-one
///   shop does not count down its tier-two line.
/// * **and it does not make a person.** A [`ProductionOutput::HumanRole`] rule
///   never counts down and never yields, whatever its interval says. S3's
///   loader refuses a nonzero interval on one and refuses the output entirely
///   for a building compatible with [`ConceptKey::Michael`]; this is the same
///   refusal at the one place that could have made a timer of it, so that a
///   record which somehow carried one -- an older save's registry, a
///   hand-built definition in a test -- still cannot manufacture a person.
///   Brief section 5.6.
///
/// A rule that reaches zero **resets to its full authored interval whether or
/// not it yielded**. A yard that cannot afford its run does not retry every
/// hour until it can; it waits out another interval, exactly as if it had
/// produced. Anything else would make a poor faction's yard the busiest thing
/// on the island.
///
/// What a yield does:
///
/// * a [`ProductionOutput::Machine`] rule goes through
///   [`ExpeditionState::produce_machine`] -- **the one owner of making a
///   machine**, unchanged by this card, still charging the cost, still
///   refusing before it mutates. There is no second production function here.
/// * a [`ProductionOutput::Capacity`] or [`ProductionOutput::Service`] rule
///   pays its cost and is **journalled**, and that is all it does. S13 records
///   nothing for capacity and services -- there is no capacity field on a
///   faction, no service registry, and no stockpile key this lane is entitled
///   to invent while brief section 20 leaves the resource list Open -- so the
///   honest record of one is [`StrategicEvent::ProductionYielded`] naming the
///   rule and its authored `output_key` and `amount`, for the lane that gives
///   capacity somewhere to go to consume.
///
/// **Nothing refills a stockpile.** A yield adds nothing to
/// `FactionState::resources`; production only ever spends. Until a lane owns
/// income, a campaign's stockpiles fall to zero and its yards skip, and the
/// journal says so in as many words rather than the island quietly stopping.
pub(crate) fn advance_production(
    state: &mut ExpeditionState,
    faction_id: &str,
    buildings: &BuildingDefinitions,
    machines: &MachineDefinitions,
    draw: u64,
) -> Vec<StrategicEvent> {
    let day = state.campaign_day;
    let mut events = Vec::new();
    // The buildings are read once, in `BTreeMap` order, before anything is
    // written: a production must not be able to change which buildings this
    // hour visits.
    let standing: Vec<(String, String, u32)> = state
        .buildings
        .values()
        .filter(|building| {
            building.faction_id == faction_id && building.state == BuildingState::Operational
        })
        .map(|building| (building.id.clone(), building.def_id.clone(), building.tier))
        .collect();

    for (building_instance_id, def_id, tier) in standing {
        // A building whose record the registry does not carry produces
        // nothing, because there are no rules to produce from. S10's sweep
        // reads an unlookupable building conservatively -- standing and
        // productive -- and that stays its reading; this is only the absence
        // of a rule to run.
        let Some(definition) = buildings.get(&def_id) else {
            continue;
        };
        for (rule_index, rule) in definition.production.iter().enumerate() {
            // Brief section 5.6, at the one place a clock could have made a
            // person: before the countdown, not after it.
            if let ProductionOutput::HumanRole { .. } = rule.output {
                continue;
            }
            if rule.interval_hours == 0 || tier < rule.minimum_tier {
                continue;
            }

            let building = state
                .buildings
                .get_mut(&building_instance_id)
                .expect("the instance ID was taken from this map and nothing removes from it");
            let remaining = building
                .production_countdown
                .get(&rule_index)
                .copied()
                .unwrap_or(rule.interval_hours);
            // One hour off the clock. `remaining` counts the hours still to
            // run, so the hour that finds one left is the hour the rule comes
            // due; a rule with no key yet reads as the full interval and
            // spends its first hour here. A rule that comes due resets to its
            // full authored interval below whether or not the yield succeeded,
            // and a `0` that somehow reached a save is read as due rather than
            // subtracted from, so nothing here can underflow.
            let due = remaining <= 1;
            building.production_countdown.insert(
                rule_index,
                if due {
                    rule.interval_hours
                } else {
                    remaining - 1
                },
            );
            if !due {
                continue;
            }

            match rule.output {
                ProductionOutput::Machine { .. } => {
                    let ordinal = state.buildings[&building_instance_id].machines_produced;
                    let instance_id =
                        machine_instance_id(&building_instance_id, rule_index, ordinal, draw);
                    match state.produce_machine(
                        &instance_id,
                        &building_instance_id,
                        rule_index,
                        buildings,
                        machines,
                    ) {
                        Ok((_machine, event)) => events.push(event),
                        Err(error) => events.push(StrategicEvent::ProductionSkipped {
                            faction_id: faction_id.to_owned(),
                            building_instance_id: building_instance_id.clone(),
                            rule_id: rule.id.clone(),
                            reason: ProductionSkipReason::of(&error),
                            day,
                        }),
                    }
                }
                ProductionOutput::Capacity | ProductionOutput::Service => {
                    match charge_production_cost(state, faction_id, &rule.cost) {
                        Ok(()) => events.push(StrategicEvent::ProductionYielded {
                            faction_id: faction_id.to_owned(),
                            building_instance_id: building_instance_id.clone(),
                            rule_id: rule.id.clone(),
                            output_key: rule.output_key.clone(),
                            amount: rule.amount,
                            day,
                        }),
                        Err(error) => events.push(StrategicEvent::ProductionSkipped {
                            faction_id: faction_id.to_owned(),
                            building_instance_id: building_instance_id.clone(),
                            rule_id: rule.id.clone(),
                            reason: ProductionSkipReason::of(&error),
                            day,
                        }),
                    }
                }
                ProductionOutput::HumanRole { .. } => unreachable!(
                    "brief section 5.6: a human-role rule is skipped above and never counts down"
                ),
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::geography::Geography;
    use crate::strategy::building::{
        BuildingDefinition, CaptureRules, FIRST_TIER, ProductionRule, RuinState, TierState,
    };
    use crate::strategy::faction::FactionState;

    /// Content owns machine and building IDs; `content/machines/` is a C card
    /// and does not exist yet. These are the test namespace's, read by nothing
    /// outside this module.
    const DOG: &str = "machine.test.mechanical_dog";
    const YARD: &str = "building.test.machine_yard";
    const YARD_INSTANCE: &str = "building_instance.test.yard";
    const DOG_INSTANCE: &str = "machine_instance.test.first_dog";
    const BEACH: &str = "world.cell.black_beach";

    /// The open resource key this lane spends. Brief section 20 leaves the
    /// resource list Open, so fuel is a *string a rule names*, exactly as S1
    /// and S3 found, and no enum of goods exists to hold it.
    const FUEL: &str = "resource.open.fuel";

    fn michael() -> String {
        ConceptKey::Michael.faction_id()
    }

    fn a_dog_record() -> MachineDefinition {
        MachineDefinition {
            id: DOG.into(),
            family: MachineFamily::MechanicalDog,
            operational_footprint_cells: 1,
            navigation_width_cells: 1,
            turning_clearance_cells: 1,
            maximum_slope: 200,
            valid_route_types: BTreeSet::from(["safe_road".to_owned(), "jungle_edge".to_owned()]),
            bridge_requirements: BTreeSet::new(),
            crew_or_handler_requirement: 1,
            fuel_requirement: 4,
            water_requirement: 2,
            repair_sockets: BTreeSet::from(["socket.test.boiler".to_owned()]),
            local_and_offscreen_representations: "needs decision: the art lane".into(),
            wreck_footprint_cells: 1,
            salvage_value: BTreeMap::from([("resource.open.scrap".to_owned(), 1)]),
        }
    }

    fn a_machine_registry() -> MachineDefinitions {
        let mut machines = MachineDefinitions::new();
        machines.insert(a_dog_record()).expect("the record loads");
        machines
    }

    /// A Michael-compatible yard that makes one mechanical dog for fuel.
    fn a_yard_record() -> BuildingDefinition {
        BuildingDefinition {
            id: YARD.into(),
            faction_compatibility: BTreeSet::from([ConceptKey::Michael]),
            function: "a machine yard".into(),
            footprint_cells: 2,
            clearance_cells: 1,
            tier_states: vec![TierState {
                tier: FIRST_TIER,
                construction_hours: 0,
                construction_requirements: BTreeSet::new(),
                hit_points: 100,
                notes: String::new(),
            }],
            production: vec![ProductionRule {
                id: "production.test.dog".into(),
                output: ProductionOutput::Machine {
                    family: MachineFamily::MechanicalDog,
                },
                output_key: DOG.into(),
                amount: 1,
                interval_hours: 0,
                minimum_tier: FIRST_TIER,
                cost: BTreeMap::from([(FUEL.to_owned(), 3)]),
            }],
            capture_rules: CaptureRules::Capturable,
            ruin_state: RuinState::ClearsCompletely,
            ..BuildingDefinition::default()
        }
    }

    fn registry(definition: BuildingDefinition) -> BuildingDefinitions {
        let mut definitions = BuildingDefinitions::new();
        definitions.insert(definition).expect("the record loads");
        definitions
    }

    /// A campaign with Michael holding the beach, a finished yard on it, and
    /// `fuel` in the stockpile.
    fn a_campaign(fuel: u32, record: BuildingDefinition) -> (ExpeditionState, BuildingDefinitions) {
        let geography = Geography::black_beach_vertical_slice();
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");

        let mut faction = FactionState::new();
        faction.resources.insert(FUEL.to_owned(), fuel);
        state.factions.insert(michael(), faction);
        state
            .set_control(BEACH, Some(michael()), &geography)
            .expect("the beach is a real cell");

        let definitions = registry(record);
        state
            .place_building(
                YARD_INSTANCE,
                YARD,
                BEACH,
                &michael(),
                &geography,
                &definitions,
            )
            .expect("the yard is placed");
        assert_eq!(
            state.buildings[YARD_INSTANCE].state,
            BuildingState::Operational,
            "a zero-hour tier is finished the moment it is placed"
        );
        (state, definitions)
    }

    // ---- S16: the hour's production step ----
    //
    // These sit here rather than at the end of the module so that C14's
    // appended machine-record tests and this lane's timer tests do not have to
    // meet in the same lines. They reuse the fixtures above rather than
    // restating them: the yard, the dog and the fuel key are the same ones.

    /// A `strategic.economy` draw, stated once. Any number does: the draw picks
    /// nothing, it only names what came out.
    const DRAW: u64 = 0x0123_4567_89ab_cdef;

    /// The yard above, with its rule put on a timer and pointed at `output`.
    fn a_yard_record_on_a_timer(
        interval_hours: u32,
        output: ProductionOutput,
    ) -> BuildingDefinition {
        let mut record = a_yard_record();
        let rule = &mut record.production[0];
        rule.interval_hours = interval_hours;
        rule.output = output;
        record
    }

    /// One hour of Michael's production, as `run_hour` runs it.
    fn an_hour(
        state: &mut ExpeditionState,
        definitions: &BuildingDefinitions,
        machines: &MachineDefinitions,
    ) -> Vec<StrategicEvent> {
        advance_production(state, &michael(), definitions, machines, DRAW)
    }

    /// **The card, whole, at the unit scale**: a rule with `interval_hours: 3`
    /// counts down, produces on the third hour through `produce_machine`, pays
    /// for it, starts over, and -- with the stockpile empty -- skips the next
    /// one with a reason instead of going negative.
    ///
    /// The save round trip in the middle is the countdown's `serde(default)`
    /// doing its job: a yard that has waited two of its three hours owes one
    /// hour after a reload, not three.
    #[test]
    fn a_timer_rule_produces_on_its_interval_pays_for_it_and_then_starts_over() {
        let (mut state, definitions) = a_campaign(
            3,
            a_yard_record_on_a_timer(
                3,
                ProductionOutput::Machine {
                    family: MachineFamily::MechanicalDog,
                },
            ),
        );
        let machines = a_machine_registry();

        assert!(
            an_hour(&mut state, &definitions, &machines).is_empty(),
            "the first hour of a three-hour interval makes nothing"
        );
        assert_eq!(state.buildings[YARD_INSTANCE].production_countdown[&0], 2);
        assert!(an_hour(&mut state, &definitions, &machines).is_empty());
        assert_eq!(state.buildings[YARD_INSTANCE].production_countdown[&0], 1);

        state = ExpeditionState::from_json(&state.to_json()).expect("the save reloads");
        assert_eq!(
            state.buildings[YARD_INSTANCE].production_countdown[&0], 1,
            "a reloaded yard owes the hour it owed, not a fresh interval"
        );

        let events = an_hour(&mut state, &definitions, &machines);
        assert_eq!(
            events,
            vec![StrategicEvent::MachineProduced {
                faction_id: michael(),
                family: MachineFamily::MechanicalDog,
                building_instance_id: YARD_INSTANCE.into(),
                day: state.campaign_day,
            }],
            "the third hour is the yield"
        );
        assert_eq!(state.machines_of(&michael()).len(), 1);
        assert_eq!(
            state.factions[&michael()].resources[FUEL],
            0,
            "the rule's cost came out of the stockpile"
        );
        assert_eq!(
            state.buildings[YARD_INSTANCE].production_countdown[&0], 3,
            "and the interval starts over at its authored length"
        );

        // The next interval comes due against an empty stockpile.
        assert!(an_hour(&mut state, &definitions, &machines).is_empty());
        assert!(an_hour(&mut state, &definitions, &machines).is_empty());
        let events = an_hour(&mut state, &definitions, &machines);
        assert_eq!(
            events,
            vec![StrategicEvent::ProductionSkipped {
                faction_id: michael(),
                building_instance_id: YARD_INSTANCE.into(),
                rule_id: "production.test.dog".into(),
                reason: ProductionSkipReason::InsufficientResource {
                    key: FUEL.into(),
                    held: 0,
                    needed: 3,
                },
                day: state.campaign_day,
            }],
            "a yard that cannot pay skips, by name"
        );
        assert_eq!(
            state.machines_of(&michael()).len(),
            1,
            "and makes nothing while it skips"
        );
        assert_eq!(
            state.factions[&michael()].resources[FUEL],
            0,
            "a refused run spends nothing: the stockpile is zero, never below it"
        );
        assert_eq!(
            state.buildings[YARD_INSTANCE].production_countdown[&0], 3,
            "a skipped interval starts over too -- a poor yard does not retry hourly"
        );
    }

    /// A capacity rule is paid for and **journalled**, and that is all it does.
    ///
    /// S13 records nothing for capacity or a service: there is no capacity
    /// field on a faction and no service registry, and a stockpile key to hold
    /// one would be this lane answering brief section 20's Open resource list.
    /// So the record of a yield is the event, and the assertion worth making is
    /// the negative one -- **production never adds to a stockpile**. Nothing in
    /// this round refills one.
    #[test]
    fn a_capacity_rule_is_journalled_and_puts_nothing_into_the_stockpile() {
        let (mut state, definitions) =
            a_campaign(3, a_yard_record_on_a_timer(2, ProductionOutput::Capacity));
        let machines = MachineDefinitions::new();

        assert!(an_hour(&mut state, &definitions, &machines).is_empty());
        let events = an_hour(&mut state, &definitions, &machines);
        assert_eq!(
            events,
            vec![StrategicEvent::ProductionYielded {
                faction_id: michael(),
                building_instance_id: YARD_INSTANCE.into(),
                rule_id: "production.test.dog".into(),
                output_key: DOG.into(),
                amount: 1,
                day: state.campaign_day,
            }],
            "a capacity yield is reported with the rule's authored key and amount"
        );
        assert_eq!(
            state.factions[&michael()].resources[FUEL],
            0,
            "capacity is paid for out of the same stockpile a machine is"
        );
        assert_eq!(
            state.factions[&michael()].resources.len(),
            1,
            "and the yield itself went nowhere: no key was invented to hold it"
        );
        assert!(
            state.machines.is_empty(),
            "a capacity rule does not make a machine"
        );
    }

    /// A timer runs only on a building that is *working for its owner*.
    ///
    /// `Operational` and not [`BuildingInstance::is_working`], which also
    /// admits `Captured`: `produce_machine` refuses a captured building because
    /// brief section 20's "capture versus destruction rules by building type"
    /// is Open, and a countdown that ran on one would spend its interval
    /// reaching that refusal over and over. The timer waits for the decision
    /// instead.
    #[test]
    fn only_an_operational_building_counts_down() {
        for state_after_placing in [
            BuildingState::UnderConstruction,
            BuildingState::Damaged,
            BuildingState::Ruined,
            BuildingState::Captured,
        ] {
            let (mut state, definitions) = a_campaign(
                9,
                a_yard_record_on_a_timer(
                    1,
                    ProductionOutput::Machine {
                        family: MachineFamily::MechanicalDog,
                    },
                ),
            );
            let machines = a_machine_registry();
            state
                .buildings
                .get_mut(YARD_INSTANCE)
                .expect("the yard was placed")
                .state = state_after_placing;

            assert!(
                an_hour(&mut state, &definitions, &machines).is_empty(),
                "a {state_after_placing:?} yard produced something"
            );
            assert!(
                state.buildings[YARD_INSTANCE]
                    .production_countdown
                    .is_empty(),
                "a {state_after_placing:?} yard should not even have started its timer"
            );
            assert_eq!(state.factions[&michael()].resources[FUEL], 9);
        }
    }

    /// **S3's refusal stands, and the hour is not the hole in it.** Brief
    /// section 5.6: buildings do not manufacture people.
    ///
    /// Three locks, and this test turns all three:
    ///
    /// 1. a building compatible with [`ConceptKey::Michael`] carrying *any*
    ///    human-role rule does not load at all;
    /// 2. any other building carrying one with `interval_hours > 0` does not
    ///    load either -- the mechanism is refused, not the faction;
    /// 3. so the only human-role rule that can reach a registry the hour reads
    ///    is one at `interval_hours: 0`, and [`advance_production`] skips it
    ///    *before* the countdown rather than by the interval gate. The witness
    ///    is `production_countdown`: the machine rule beside it gets a key and
    ///    the human-role rule never does.
    #[test]
    fn a_rule_that_makes_a_person_never_runs_on_a_timer() {
        let a_role = || ProductionOutput::HumanRole {
            role: "role.test.mechanic".into(),
        };

        let mut michaels = a_yard_record_on_a_timer(0, a_role());
        michaels.recruitment_support = BTreeSet::from(["recruitment.test.support".to_owned()]);
        assert!(
            matches!(
                michaels.validate(),
                Err(BuildingError::MichaelBuildingProducesPeople { .. })
            ),
            "lock one: Michael's buildings make machines, capacity and services"
        );

        let mut pirates = a_yard_record_on_a_timer(4, a_role());
        pirates.faction_compatibility = BTreeSet::from([ConceptKey::Pirates]);
        pirates.recruitment_support = BTreeSet::from(["recruitment.test.support".to_owned()]);
        assert!(
            matches!(
                pirates.validate(),
                Err(BuildingError::TimerDrivenHumanRole { .. })
            ),
            "lock two: a manufacturing timer is refused whoever owns it"
        );

        // Lock three, on the one shape that does load: a legal human-role rule
        // beside a machine rule on a timer.
        pirates.production[0].interval_hours = 0;
        pirates.production.push(ProductionRule {
            id: "production.test.dog".into(),
            output: ProductionOutput::Machine {
                family: MachineFamily::MechanicalDog,
            },
            output_key: DOG.into(),
            amount: 1,
            interval_hours: 1,
            minimum_tier: FIRST_TIER,
            cost: BTreeMap::new(),
        });
        let definitions = registry(pirates);
        let geography = Geography::black_beach_vertical_slice();
        let pirate_id = ConceptKey::Pirates.faction_id();
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        state
            .factions
            .insert(pirate_id.clone(), FactionState::new());
        state
            .set_control(BEACH, Some(pirate_id.clone()), &geography)
            .expect("the beach is a real cell");
        state
            .place_building(
                YARD_INSTANCE,
                YARD,
                BEACH,
                &pirate_id,
                &geography,
                &definitions,
            )
            .expect("the yard is placed");

        let machines = a_machine_registry();
        for _ in 0..5 {
            advance_production(&mut state, &pirate_id, &definitions, &machines, DRAW);
        }
        let countdown = &state.buildings[YARD_INSTANCE].production_countdown;
        assert!(
            !countdown.contains_key(&0),
            "the human-role rule was given a timer: {countdown:?}"
        );
        assert!(
            countdown.contains_key(&1),
            "the machine rule beside it does count down, so the absence above is the guard \
             and not an idle step: {countdown:?}"
        );
        assert_eq!(
            state.machines_of(&pirate_id).len(),
            5,
            "five hours of a one-hour machine rule are five machines, each with its own ID"
        );
    }

    /// **The card's second Done-when**, whole: a `mechanical_dog` produced from
    /// a tier-1 Michael building, the fuel gone from the stockpile, and a
    /// `MachineProduced` event to show for it.
    #[test]
    fn a_tier_one_michael_building_produces_a_mechanical_dog_and_the_fuel_is_gone() {
        let (mut state, definitions) = a_campaign(10, a_yard_record());
        let machines = a_machine_registry();

        let (dog, event) = state
            .produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines)
            .expect("an operational tier-1 yard with fuel in the bank produces");

        assert_eq!(dog.def_id, DOG, "the instance references the record by ID");
        assert_eq!(dog.faction_id, michael());
        assert_eq!(dog.cell_id, BEACH);
        assert_eq!(dog.built_by_building_instance_id, YARD_INSTANCE);
        assert_eq!(
            (dog.fuel_remaining, dog.water_remaining, dog.damage),
            (4, 2, 0),
            "a new machine is full and whole -- the three things a save must remember"
        );
        assert_eq!(
            state.machines[DOG_INSTANCE], dog,
            "and it is standing on the board"
        );
        assert_eq!(
            state.buildings[YARD_INSTANCE].machines_produced, 1,
            "the yard remembers that it made something"
        );

        // The resource assertion the bite lands on.
        assert_eq!(
            state.factions[&michael()].resources[FUEL],
            7,
            "three of ten fuel is spent; the rule named the key, and no enum did"
        );

        assert_eq!(
            event,
            StrategicEvent::MachineProduced {
                faction_id: michael(),
                family: MachineFamily::MechanicalDog,
                building_instance_id: YARD_INSTANCE.into(),
                day: state.campaign_day,
            }
        );
    }

    /// Every refusal rejects before it mutates: no machine, no spent fuel, no
    /// bumped counter.
    #[test]
    fn a_refused_production_leaves_the_stockpile_and_the_board_alone() {
        let machines = a_machine_registry();

        // Not enough fuel.
        let (mut state, definitions) = a_campaign(2, a_yard_record());
        let before = state.clone();
        assert_eq!(
            state.produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines),
            Err(ProductionError::InsufficientResource {
                faction_id: michael(),
                key: FUEL.into(),
                held: 2,
                needed: 3,
            })
        );
        assert_eq!(state, before, "a refusal changed nothing at all");

        // A building that is not operational.
        let (mut state, definitions) = a_campaign(10, a_yard_record());
        state
            .buildings
            .get_mut(YARD_INSTANCE)
            .expect("the yard stands")
            .state = BuildingState::Damaged;
        let before = state.clone();
        assert_eq!(
            state.produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines),
            Err(ProductionError::NotOperational {
                building_instance_id: YARD_INSTANCE.into(),
                state: BuildingState::Damaged,
            })
        );
        assert_eq!(state, before);

        // A tier below the rule's minimum.
        let mut two_tier = a_yard_record();
        two_tier.tier_states.push(TierState {
            tier: FIRST_TIER + 1,
            construction_hours: 4,
            construction_requirements: BTreeSet::new(),
            hit_points: 200,
            notes: String::new(),
        });
        two_tier.production[0].minimum_tier = FIRST_TIER + 1;
        let (mut state, definitions) = a_campaign(10, two_tier);
        let before = state.clone();
        assert_eq!(
            state.produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines),
            Err(ProductionError::BelowMinimumTier {
                building_instance_id: YARD_INSTANCE.into(),
                tier: FIRST_TIER,
                minimum_tier: FIRST_TIER + 1,
            })
        );
        assert_eq!(state, before);

        // A rule whose output is not a machine at all.
        let mut services = a_yard_record();
        services.production[0].output = ProductionOutput::Service;
        let (mut state, definitions) = a_campaign(10, services);
        let before = state.clone();
        assert_eq!(
            state.produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines),
            Err(ProductionError::NotAMachineRule {
                building_instance_id: YARD_INSTANCE.into(),
                rule_id: "production.test.dog".into(),
            })
        );
        assert_eq!(state, before);

        // A rule whose family disagrees with the record it names.
        let mut mismatched = a_yard_record();
        mismatched.production[0].output = ProductionOutput::Machine {
            family: MachineFamily::Airship,
        };
        let (mut state, definitions) = a_campaign(10, mismatched);
        let before = state.clone();
        assert_eq!(
            state.produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines),
            Err(ProductionError::FamilyMismatch {
                rule_id: "production.test.dog".into(),
                def_id: DOG.into(),
                rule_family: MachineFamily::Airship,
                definition_family: MachineFamily::MechanicalDog,
            })
        );
        assert_eq!(state, before);
    }

    /// Brief section 5.4's eight families, as the card names them, and no
    /// ninth. The keys are the card's strings verbatim.
    #[test]
    fn the_families_are_the_eight_the_brief_names() {
        let keys: Vec<&str> = MachineFamily::ALL
            .into_iter()
            .map(MachineFamily::as_key)
            .collect();
        assert_eq!(
            keys,
            [
                "mechanical_dog",
                "mechanical_cavalry",
                "mechanical_bear",
                "mechanical_elephant",
                "walker",
                "steam_wagon",
                "rocket",
                "airship",
            ]
        );
        for family in MachineFamily::ALL {
            assert_eq!(MachineFamily::from_key(family.as_key()), Some(family));
        }
        assert_eq!(MachineFamily::from_key("mechanical_horse"), None);
        assert_eq!(
            serde_json::to_string(&MachineFamily::SteamWagon).expect("serializes"),
            "\"steam_wagon\"",
            "the authored key and the serialized key are the same word"
        );
    }

    /// Brief section 5.10, as far as this lane takes it: a fact on the record,
    /// and nothing else.
    #[test]
    fn michaels_kit_is_machines_and_records_that_machines_take_the_exposure() {
        let machines = a_machine_registry();

        let mut definition = FactionDefinition {
            id: michael(),
            concept_key: ConceptKey::Michael,
            actor_kit: BTreeSet::from([DOG.to_owned()]),
            ..FactionDefinition::default()
        };
        let kit = ActorKit::of(&definition, &machines).expect("a kit of machines loads");
        assert_eq!(
            kit.entries,
            vec![ActorKitEntry::Machine {
                id: DOG.into(),
                family: MachineFamily::MechanicalDog,
            }]
        );
        assert_eq!(kit.entries[0].id(), DOG);
        assert_eq!(
            kit.families(),
            BTreeSet::from([MachineFamily::MechanicalDog])
        );
        assert_eq!(
            kit.exposure_policy,
            ExposurePolicy::MachinesFirst,
            "brief section 5.10, recorded -- and read by nothing that acts"
        );

        // Brief section 5.6 at the level of the kit: a role is not a machine.
        definition.actor_kit = BTreeSet::from([DOG.to_owned(), "actor.test.engineer".to_owned()]);
        assert_eq!(
            ActorKit::of(&definition, &machines),
            Err(ProductionError::MichaelActorKitNamesANonMachine {
                faction_id: michael(),
                entry_id: "actor.test.engineer".into(),
            }),
            "his faction fields machines; a person in his kit does not load"
        );

        // The other five may field people, and carry no derived doctrine.
        let elves = FactionDefinition {
            id: ConceptKey::Elves.faction_id(),
            concept_key: ConceptKey::Elves,
            actor_kit: BTreeSet::from(["actor.test.engineer".to_owned()]),
            ..FactionDefinition::default()
        };
        let kit = ActorKit::of(&elves, &machines).expect("an ordinary kit loads");
        assert_eq!(
            kit.entries,
            vec![ActorKitEntry::NonMachine {
                id: "actor.test.engineer".into(),
            }]
        );
        assert_eq!(kit.exposure_policy, ExposurePolicy::Unstated);
    }

    /// The registry keeps S3's contract: a malformed or wrongly-prefixed record
    /// does not enter, and a duplicate does not overwrite.
    #[test]
    fn the_machine_registry_refuses_a_record_it_cannot_trust() {
        let mut machines = MachineDefinitions::new();
        let mut wrong = a_dog_record();
        wrong.id = "automaton.test.dog".into();
        assert_eq!(
            machines.insert(wrong),
            Err(ProductionError::WrongIdPrefix {
                found: "automaton.test.dog".into(),
                expected: MACHINE_ID_PREFIX,
            })
        );
        assert!(machines.is_empty());

        machines.insert(a_dog_record()).expect("the record loads");
        assert_eq!(
            machines.insert(a_dog_record()),
            Err(ProductionError::DuplicateDefinition { id: DOG.into() })
        );
        assert_eq!(machines.len(), 1);
        assert_eq!(machines.ids().collect::<Vec<_>>(), vec![DOG]);
    }

    /// A machine round-trips through a save, and carries no copy of the
    /// definition's section 18 fields to drift from.
    #[test]
    fn a_produced_machine_survives_a_save_and_copies_no_authored_field() {
        let (mut state, definitions) = a_campaign(10, a_yard_record());
        let machines = a_machine_registry();
        state
            .produce_machine(DOG_INSTANCE, YARD_INSTANCE, 0, &definitions, &machines)
            .expect("produces");

        let json = state.to_json();
        let loaded = ExpeditionState::from_json(&json).expect("a save with a machine loads");
        assert_eq!(loaded.machines, state.machines);
        assert_eq!(loaded.machines_of(&michael()).len(), 1);

        let written: serde_json::Value = serde_json::from_str(&json).expect("the save is JSON");
        let machine = &written["machines"][DOG_INSTANCE];
        assert!(machine.is_object(), "the machine is in the save");
        for authored in [
            "family",
            "operational_footprint_cells",
            "navigation_width_cells",
            "turning_clearance_cells",
            "maximum_slope",
            "valid_route_types",
            "bridge_requirements",
            "crew_or_handler_requirement",
            "fuel_requirement",
            "water_requirement",
            "repair_sockets",
            "wreck_footprint_cells",
            "salvage_value",
        ] {
            assert!(
                machine.get(authored).is_none(),
                "{authored} belongs to the machine record, not to the save"
            );
        }
    }
}
