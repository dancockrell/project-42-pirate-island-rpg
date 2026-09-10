use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Provisional scenario tuning. An explicit organic roster avoids guessing
/// susceptibility from a sprite, actor role, or a future machine's sex field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CthulhuMadness {
    faction_id: String,
    ritual_archetype_id: String,
    radius_cells: u32,
    exposure_percent: u8,
    exposure_gain: u8,
    decay_per_tick: u8,
    warning_threshold: u8,
    conversion_threshold: u8,
    /// How many days the water keeps a body. Every corpse used to be kept for
    /// the whole campaign so the shrine could raise it whenever there was
    /// room, which meant a hundred-day island carried hundreds of the dead in
    /// its save and never released their names -- so the persona pools ran dry
    /// and two living women ended up with the same one. A body older than this
    /// is gone. Michael's own dead are exempt: her party slot is deliberately
    /// kept until he gives it up or the water gives her back.
    #[serde(default = "default_corpse_persist_days")]
    corpse_persist_days: u32,
    susceptible_definitions: BTreeSet<String>,
}

fn default_corpse_persist_days() -> u32 {
    7
}

impl CthulhuMadness {
    fn valid(&self) -> bool {
        self.radius_cells <= 32
            && self.exposure_percent <= 100
            && self.exposure_gain > 0
            && self.decay_per_tick > 0
            && self.warning_threshold > 0
            && self.warning_threshold < self.conversion_threshold
            && (1..=1000).contains(&self.corpse_persist_days)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandBuildingFootprint {
    blocked_offsets: Vec<[i32; 2]>,
    #[serde(default)]
    placement_entrance: Option<[i32; 2]>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldingExpansion {
    faction_id: String,
    building_id: String,
    archetype_id: String,
    entrance: [i32; 2],
    minimum_tick: u64,
    costs: BTreeMap<String, u32>,
    construction_ticks: u32,
    holding_health: u32,
}

impl HoldingExpansion {
    fn valid(&self) -> bool {
        (1..=100000).contains(&self.construction_ticks)
            && (1..=100000).contains(&self.holding_health)
            && !self.costs.is_empty()
            && self.costs.values().all(|cost| (1..=100000).contains(cost))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeathMemory {
    pub killed_by_player_count: u32,
    pub last_death_day: Option<u32>,
    pub last_death_context_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedPerson {
    pub id: String,
    pub display_name: String,
    pub alive_today: bool,
    pub death_memory: DeathMemory,
    #[serde(default)]
    pub sex: PersonSex,
    #[serde(default)]
    pub age: Option<u16>,
    #[serde(default)]
    pub backstory: String,
    /// Hex string survives JSON consumers that represent numbers as doubles.
    #[serde(default)]
    pub generation_seed: Option<String>,
    #[serde(default)]
    pub recruitment_offer: String,
    #[serde(default)]
    pub companion_response: String,
    #[serde(default)]
    pub discussed: bool,
    #[serde(default)]
    pub loyal_to_michael: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonSex {
    #[default]
    Unknown,
    Male,
    Female,
    Other,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonaPool {
    sex: PersonSex,
    age_min: u16,
    age_max: u16,
    given_names: Vec<String>,
    family_names: Vec<String>,
    histories: Vec<String>,
    #[serde(default)]
    recruitment_offer: String,
    #[serde(default)]
    companion_responses: Vec<String>,
}

fn produced_person(
    pools: &BTreeMap<String, Vec<PersonaPool>>,
    definition: &str,
    id: &str,
    serial: u64,
    taken: &BTreeSet<String>,
) -> Option<NamedPerson> {
    let seed = mix_seed(serial, 0, id, 0);
    let variants = pools.get(definition)?;
    let pool = variants.get((seed % variants.len().max(1) as u64) as usize)?;
    let pick = |values: &[String], rotation: u32| {
        values[(seed.rotate_left(rotation) % values.len() as u64) as usize].clone()
    };
    // The pools are small, so the same full name comes up often. Two living
    // women called Alice Bell reads as a bug rather than a coincidence, so walk
    // the family names (then the given names) until the island has a name it is
    // not already using. Still deterministic: same seed, same world, same name.
    let given = pick(&pool.given_names, 0);
    let family = pick(&pool.family_names, 13);
    let mut display_name = format!("{given} {family}");
    if taken.contains(&display_name) {
        let start = (seed.rotate_left(13) % pool.family_names.len() as u64) as usize;
        'search: for given_step in 0..pool.given_names.len() {
            let given_index = ((seed % pool.given_names.len() as u64) as usize + given_step)
                % pool.given_names.len();
            for family_step in 1..=pool.family_names.len() {
                let candidate = format!(
                    "{} {}",
                    pool.given_names[given_index],
                    pool.family_names[(start + family_step) % pool.family_names.len()]
                );
                if !taken.contains(&candidate) {
                    display_name = candidate;
                    break 'search;
                }
            }
        }
    }
    Some(NamedPerson {
        id: id.into(),
        display_name,
        alive_today: true,
        sex: pool.sex,
        age: Some(
            pool.age_min
                + (seed.rotate_left(23) % u64::from(pool.age_max - pool.age_min + 1)) as u16,
        ),
        backstory: pick(&pool.histories, 37),
        generation_seed: Some(format!("{seed:016x}")),
        recruitment_offer: pool.recruitment_offer.clone(),
        companion_response: if pool.companion_responses.is_empty() {
            String::new()
        } else {
            pick(&pool.companion_responses, 37)
        },
        ..Default::default()
    })
}

impl NamedPerson {
    fn valid_for_actor(&self, id: &str, alive: bool) -> bool {
        self.id == id
            && self.alive_today == alive
            && !self.display_name.trim().is_empty()
            && self.display_name.len() <= 120
            && !self.display_name.chars().any(char::is_control)
            && self.backstory.len() <= 4096
            && self.recruitment_offer.len() <= 4096
            && self.companion_response.len() <= 4096
            && (!self.loyal_to_michael
                || (self.sex == PersonSex::Female && self.age.is_some_and(|age| age >= 18)))
            && self.age.is_none_or(|age| age >= 18)
            && self
                .generation_seed
                .as_ref()
                .is_none_or(|seed| seed.len() == 16 && seed.bytes().all(|c| c.is_ascii_hexdigit()))
    }
    pub fn record_player_caused_death(&mut self, day: u32, context_id: &str) {
        self.alive_today = false;
        self.death_memory.killed_by_player_count += 1;
        self.death_memory.last_death_day = Some(day);
        self.death_memory.last_death_context_id = Some(context_id.to_owned());
    }

    pub fn return_at_midnight(&mut self) {
        self.alive_today = true;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnedMonster {
    pub instance_id: String,
    pub definition_id: String,
    pub level: u8,
    pub behavior_tags: [String; 2],
    pub physical_variant: String,
    pub condition: String,
    pub patrol_purpose: String,
    pub loot_seed: u64,
}

#[derive(Clone, Debug)]
pub struct WorldClock {
    pub day: u32,
    pub minute_of_day: u16,
    pub named_people: BTreeMap<String, NamedPerson>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnRule {
    pub region_id: String,
    pub definition_ids: Vec<String>,
    pub region_base_level: u8,
    pub daily_count: u8,
    pub pressure: i8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorldEvent {
    MidnightFlashStarted {
        day: u32,
    },
    NamedPersonReturned {
        person_id: String,
        killed_by_player_count: u32,
    },
    MonsterMaterialized {
        region_id: String,
        monster: SpawnedMonster,
    },
    MidnightFlashEnded {
        day: u32,
        monster_count: usize,
    },
}

impl WorldClock {
    pub fn advance_to_next_midnight(&mut self) {
        self.day += 1;
        self.minute_of_day = 0;
        for person in self.named_people.values_mut() {
            person.return_at_midnight();
        }
    }

    pub fn level_for_daily_spawn(&self, region_base: u8, pressure: i8) -> u8 {
        let daily_growth = ((self.day.saturating_sub(1)) / 3).min(12) as i16;
        (i16::from(region_base) + daily_growth + i16::from(pressure)).clamp(1, 99) as u8
    }

    pub fn resolve_midnight(&mut self, rules: &[SpawnRule], world_seed: u64) -> Vec<WorldEvent> {
        self.day += 1;
        self.minute_of_day = 0;
        let mut events = vec![WorldEvent::MidnightFlashStarted { day: self.day }];
        for person in self.named_people.values_mut() {
            person.return_at_midnight();
            events.push(WorldEvent::NamedPersonReturned {
                person_id: person.id.clone(),
                killed_by_player_count: person.death_memory.killed_by_player_count,
            });
        }
        let mut count = 0usize;
        for rule in rules {
            assert!(
                !rule.definition_ids.is_empty(),
                "spawn rule requires definitions"
            );
            for slot in 0..rule.daily_count {
                let seed = mix_seed(world_seed, self.day, &rule.region_id, slot);
                let definition_id =
                    rule.definition_ids[(seed as usize) % rule.definition_ids.len()].clone();
                let variant = ["lean", "scarred", "bright-crested", "mud-dark"]
                    [(seed.rotate_left(9) as usize) % 4];
                let condition = ["hungry", "territorial", "wounded", "watchful"]
                    [(seed.rotate_left(17) as usize) % 4];
                let purpose = [
                    "holding a crossing",
                    "stalking the road",
                    "feeding near ruins",
                    "guarding a nest",
                ][(seed.rotate_left(27) as usize) % 4];
                let monster = SpawnedMonster {
                    instance_id: format!(
                        "spawn.day{}.{}.{}",
                        self.day,
                        rule.region_id.replace('.', "_"),
                        slot
                    ),
                    definition_id,
                    level: self.level_for_daily_spawn(rule.region_base_level, rule.pressure),
                    behavior_tags: [condition.into(), "individual-threat".into()],
                    physical_variant: variant.into(),
                    condition: condition.into(),
                    patrol_purpose: purpose.into(),
                    loot_seed: seed,
                };
                events.push(WorldEvent::MonsterMaterialized {
                    region_id: rule.region_id.clone(),
                    monster,
                });
                count += 1;
            }
        }
        events.push(WorldEvent::MidnightFlashEnded {
            day: self.day,
            monster_count: count,
        });
        events
    }
}

fn mix_seed(mut seed: u64, day: u32, region_id: &str, slot: u8) -> u64 {
    seed ^= u64::from(day).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    seed ^= u64::from(slot) << 32;
    for byte in region_id.bytes() {
        seed ^= u64::from(byte);
        seed = seed.wrapping_mul(0x100_0000_01B3);
    }
    seed ^ (seed >> 29)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionRule {
    pub id: String,
    #[serde(alias = "producerArchetypeId")]
    pub producer_archetype_id: String,
    #[serde(alias = "outputDefinitionId")]
    pub actor_definition_id: String,
    #[serde(alias = "actorKind")]
    pub actor_kind: String,
    pub costs: BTreeMap<String, u32>,
    #[serde(alias = "productionTicks")]
    pub production_ticks: u32,
    #[serde(alias = "populationUse")]
    pub population_use: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct IslandFactionTuning {
    population_capacity: u32,
    holding_level: u32,
    holding_health: u32,
    combat: IslandCombatProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: String,
    pub rule: ProductionRule,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionBuilding {
    #[serde(default)]
    pub construction: Option<BuildingWork>,
    #[serde(default)]
    pub repair: Option<BuildingWork>,
    #[serde(default = "starting_building_level")]
    pub level: u32,
    #[serde(default = "default_building_health")]
    pub max_health: u32,
    #[serde(default)]
    pub development: Option<BuildingDevelopment>,
    #[serde(default = "default_building_health")]
    pub health: u32,
    pub id: String,
    pub faction_id: String,
    pub archetype_id: String,
    pub node_id: String,
    pub rally_point_id: String,
    pub operational: bool,
    pub queue_capacity: usize,
    pub production_queue: Vec<ProductionOrder>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingWork {
    pub builder_id: String,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldingRepairRule {
    costs: BTreeMap<String, u32>,
    ticks: u32,
    health_gain: u32,
    emergency_health_percent: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichaelMachinery {
    definition_id: String,
    display_name: String,
    capacity: usize,
    combat: IslandCombatProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SalvageCache {
    pub position: IslandPoint,
    pub remaining: u32,
    pub initial_amount: u32,
    pub label: String,
    pub source_building_id: String,
    pub source_faction: String,
    pub level: u32,
    pub created_tick: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldingSalvageRule {
    /// The catalogue resource a wreck, a razed holding and Michael's workshop
    /// are all paid in. The simulation reads the key from the pack instead of
    /// compiling one in, so a scenario may salvage something else entirely.
    #[serde(rename = "resourceId")]
    resource_id: String,
    base_yield: u32,
    per_completed_level: u32,
    workshop_yield: u32,
}

/// One catalogue record. A resource key is declared here or it does not exist:
/// nothing in the simulation may invent one by writing to a stockpile.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioResource {
    #[serde(rename = "displayName")]
    pub display_name: String,
}

/// A faction's authored opening economy. What it holds, what it earns, and how
/// much of each resource it can store; every key must be in the catalogue.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioFactionEconomy {
    #[serde(default)]
    pub stockpile: BTreeMap<String, u32>,
    #[serde(rename = "incomePerTick", default)]
    pub income_per_tick: BTreeMap<String, u32>,
    #[serde(rename = "storageCaps", default)]
    pub storage_caps: BTreeMap<String, u32>,
}

impl ScenarioFactionEconomy {
    /// An authored economy is legal only against a catalogue: every key
    /// declared, every earned resource capped, nothing starting over its cap.
    fn checked(&self, catalogue: &BTreeMap<String, ScenarioResource>) -> Result<(), String> {
        let mut keys = self.stockpile.iter();
        let named = self
            .stockpile
            .keys()
            .chain(self.income_per_tick.keys())
            .chain(self.storage_caps.keys());
        if named.clone().any(|id| !catalogue.contains_key(id))
            || named.count() > MAXIMUM_RESOURCES * 3
            || self
                .stockpile
                .values()
                .chain(self.income_per_tick.values())
                .chain(self.storage_caps.values())
                .any(|amount| *amount > 100000)
            || self
                .income_per_tick
                .keys()
                .any(|id| !self.storage_caps.contains_key(id))
            || keys.any(|(id, stored)| {
                self.storage_caps
                    .get(id)
                    .is_some_and(|capacity| stored > capacity)
            })
        {
            return Err("invalid_scenario_economy".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FootholdConfig {
    cache_position: IslandPoint,
    cache_salvage: u32,
    build_salvage: u32,
    construction_ticks: u32,
    restore_salvage: u32,
    building_health: u32,
}

impl FootholdConfig {
    fn valid(&self) -> bool {
        [
            self.cache_salvage,
            self.build_salvage,
            self.construction_ticks,
            self.restore_salvage,
            self.building_health,
        ]
        .iter()
        .all(|v| (1..=100000).contains(v))
    }
}

/// Version 2 added the world's own `ScenarioRules`; version 1 saves migrate by
/// adopting the rules of the scenario they are resumed into.
const SAVE_VERSION: u32 = 2;
/// Provisional per-actor inventory bound; the world refuses a longer one.
const MAX_ACTOR_INVENTORY: usize = 32;
/// Provisional bound on how many actors may carry anything at once.
const MAX_INVENTORY_ENTRIES: usize = 4096;

/// A pack's trigger list and the record of what has fired are both bounded,
/// for the same reason every other collection is: a pack and a save are both
/// untrusted input.
pub const MAX_TRIGGERS: usize = 512;
pub const MAX_FLAGS: usize = 512;
pub const MAX_TRIGGER_HISTORY: usize = 512;
/// A campaign can only owe the player so many unanswered judgments before the
/// choice stops meaning anything.
pub const MAX_OPEN_LEADS: usize = 32;
pub const MAX_QUESTS: usize = 64;
pub const MAX_QUEST_STAGES: usize = 32;

/// The heat ledger is append-only, so it needs a ceiling for the same reason
/// every other saved collection has one: a save is untrusted input.
pub const HEAT_LEDGER_CAP: usize = 512;

/// A bound on an untrusted pack's or save's catalogue, in the shape of every
/// other collection cap the loader applies.
const MAXIMUM_RESOURCES: usize = 256;

/// `resource.<name>`, the form the catalogue, the validator and the schema all
/// agree on. One owner of the shape.
fn valid_resource_id(id: &str) -> bool {
    id.len() <= 256
        && id.strip_prefix("resource.").is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}

/// `quest.<name>`, the form the catalogue, the validator and the schema all
/// agree on.
fn valid_quest_id(id: &str) -> bool {
    id.len() <= 256
        && id.strip_prefix("quest.").is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}

fn default_building_health() -> u32 {
    80
}

fn starting_building_level() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingDevelopmentRule {
    pub max_level: u32,
    pub costs: BTreeMap<String, u32>,
    pub ticks: u32,
    pub health_gain: u32,
    pub population_gain: u32,
}

impl BuildingDevelopmentRule {
    fn valid(&self) -> bool {
        (2..=5).contains(&self.max_level)
            && (1..=100000).contains(&self.ticks)
            && self.health_gain <= 10000
            && self.population_gain <= 64
            && !self.costs.is_empty()
            && self.costs.len() <= 32
            && self.costs.values().all(|cost| (1..=100000).contains(cost))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingDevelopment {
    pub rule: BuildingDevelopmentRule,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionState {
    pub id: String,
    pub resources: BTreeMap<String, u32>,
    pub population_used: u32,
    pub population_capacity: u32,
    /// Maximum absolute fixed-point wobble accepted in one dispatch score.
    pub wobble_limit: i32,
    pub buildings: BTreeMap<String, FactionBuilding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorProductionProvenance {
    pub faction_id: String,
    pub producer_building_id: String,
    pub production_rule_id: String,
    pub reserved_costs: BTreeMap<String, u32>,
    pub completed_tick: u64,
    pub rally_point_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducedActor {
    /// Exposure persists through saves. A living cult convert retains the
    /// threshold as its binding marker; a native cultist begins at zero.
    #[serde(default)]
    pub madness: u8,
    #[serde(default)]
    pub undead: bool,
    #[serde(default)]
    pub person: Option<NamedPerson>,
    pub instance_id: String,
    pub definition_id: String,
    pub actor_kind: String,
    pub faction_id: String,
    pub node_id: String,
    pub current_assignment_id: Option<String>,
    pub provenance: ActorProductionProvenance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchScore {
    pub goal_progress: i32,
    pub target_threat: i32,
    pub faction_hatred: i32,
    pub expected_loot: i32,
    pub strategic_position: i32,
    pub supply_cost: i32,
    pub travel_risk: i32,
    pub home_defense_deficit: i32,
    pub role_fitness: i32,
}

impl DispatchScore {
    pub fn total_without_wobble(&self) -> i32 {
        self.goal_progress
            + self.target_threat
            + self.faction_hatred
            + self.expected_loot
            + self.strategic_position
            + self.supply_cost
            + self.travel_risk
            + self.home_defense_deficit
            + self.role_fitness
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchCandidate {
    pub action_id: String,
    pub assignment: String,
    pub target_node_id: String,
    pub score: DispatchScore,
    /// Fixed-point variation authored or generated from the recorded world seed.
    pub wobble: i32,
}

impl DispatchCandidate {
    pub fn total(&self) -> i32 {
        self.score.total_without_wobble() + self.wobble
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactionWorldEvent {
    MadnessConverted {
        actor_id: String,
        previous_faction_id: String,
    },
    MidnightReturned {
        actor_id: String,
        previous_faction_id: String,
        position: IslandPoint,
    },
    UnitStruck {
        attacker_id: String,
        target_id: String,
        damage: u32,
        origin: IslandPoint,
        target_position: IslandPoint,
        attacker_definition: String,
    },
    UnitFallen {
        actor_id: String,
    },
    FactionEliminated {
        faction_id: String,
    },
    ProductionQueued {
        faction_id: String,
        building_id: String,
        order_id: String,
        production_rule_id: String,
    },
    ActorProduced {
        actor: ProducedActor,
    },
    ActorAssigned {
        actor_id: String,
        faction_id: String,
        action_id: String,
        assignment: String,
        target_node_id: String,
        total_score: i32,
    },
    ActorMoved {
        actor_id: String,
        position: IslandPoint,
    },
    ActorArrived {
        actor_id: String,
        destination_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactionWorldError {
    FactionEliminated(String),
    InvalidPolicy(String),
    UnknownFaction(String),
    UnknownBuilding(String),
    UnknownActor(String),
    BuildingNotOperational(String),
    ProducerFactionMismatch,
    ProducerArchetypeMismatch,
    QueueFull(String),
    InvalidProductionTicks,
    InsufficientPopulation,
    InsufficientResource {
        resource_id: String,
        required: u32,
        available: u32,
    },
    /// A cost named something the scenario's catalogue does not declare. The
    /// stockpile refuses it rather than quietly inventing the resource.
    UnknownResource(String),
    NoDispatchCandidates,
    MissingIslandPosition(String),
    UnreachableDestination(String),
    WobbleOutOfBounds {
        action_id: String,
        wobble: i32,
        limit: i32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivalTruce {
    pub a: String,
    pub b: String,
    pub threat: String,
    pub expires_tick: u64,
    pub resume_ab: bool,
    pub resume_ba: bool,
}

/// How much one faction can stand another, on the same scale the authored
/// relationship matrix already uses. Until now that matrix's `standing` was
/// read once and thrown away, and the only rule that could start a war was
/// "Cthulhu has grown too strong" -- so once the opening war burned out, the
/// survivors stood in the same clearing for the rest of the campaign with no
/// rule anywhere that could make them fight again. This is that number kept
/// alive: it drifts every decision tick, and crossing a threshold opens or
/// closes a war.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionStanding {
    pub a: String,
    pub b: String,
    /// What the matrix authored. Quiet years pull the living value back to it,
    /// so peace restores the relationship the setting describes rather than
    /// leaving every pair permanently at the worst it ever was.
    pub baseline: i32,
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyNotice {
    pub tick: u64,
    pub factions: Vec<String>,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct InitialDiplomacy {
    #[serde(rename = "factionIds")]
    faction_ids: Vec<String>,
    relationships: Vec<InitialRelationship>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct InitialRelationship {
    a: String,
    b: String,
    #[serde(rename = "atWar")]
    at_war: bool,
    /// The matrix has always carried this. Nothing read it until standings.
    #[serde(default)]
    standing: i32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivalRules {
    decision_ticks: u64,
    truce_ticks: u64,
    dominance_percent: u64,
    strength_horizon_ticks: u64,
    /// Standing at or below this opens a war; at or above `peace_standing` an
    /// open war stops. Defaulted so a version-2 save written before standings
    /// existed still loads and starts drifting from its authored baselines.
    #[serde(default = "default_war_standing")]
    war_standing: i32,
    #[serde(default = "default_peace_standing")]
    peace_standing: i32,
    /// How fast being overshadowed by a neighbour spends the relationship.
    #[serde(default = "default_fear_step")]
    fear_step: i32,
    /// How fast an open war spends itself. Wars end because both sides are
    /// tired, which is the only way a war between two survivors can end
    /// without one of them ceasing to exist.
    #[serde(default = "default_war_weariness_step")]
    war_weariness_step: i32,
    /// How fast quiet time repairs a relationship toward its baseline.
    #[serde(default = "default_recovery_step")]
    recovery_step: i32,
    /// How much slower a shot is when the target is a holding rather than a
    /// person. Muskets and spears are made for people; battering a fort is
    /// slow, deliberate work. Without this a walk-up army levels a faction's
    /// only holding in an afternoon, which is why the opening war used to end
    /// the campaign on day one.
    #[serde(default = "default_siege_cooldown_multiplier")]
    siege_cooldown_multiplier: u32,
}

fn default_siege_cooldown_multiplier() -> u32 {
    4
}

fn default_war_standing() -> i32 {
    -50
}
fn default_peace_standing() -> i32 {
    -20
}
fn default_fear_step() -> i32 {
    2
}
fn default_war_weariness_step() -> i32 {
    1
}
fn default_recovery_step() -> i32 {
    1
}

impl SurvivalRules {
    fn valid(&self) -> bool {
        (1..=1024).contains(&self.decision_ticks)
            && (1..=100000).contains(&self.truce_ticks)
            && (101..=1000).contains(&self.dominance_percent)
            && (1..=1024).contains(&self.strength_horizon_ticks)
            && (-100..=100).contains(&self.war_standing)
            && (-100..=100).contains(&self.peace_standing)
            && self.war_standing < self.peace_standing
            && (1..=50).contains(&self.fear_step)
            && (1..=50).contains(&self.war_weariness_step)
            && (1..=50).contains(&self.recovery_step)
            && (1..=64).contains(&self.siege_cooldown_multiplier)
    }
}

fn faction_label(id: &str) -> &str {
    match id {
        "faction.colonial_powers.prototype" => "the colonial powers",
        "faction.pirates.prototype" => "the pirates",
        "faction.elves.prototype" => "the elves",
        "faction.eastern_fox_people.prototype" => "the fox people",
        "faction.cthulhu.prototype" => "Cthulhu's faction",
        "faction.michael" => "Michael's faction",
        _ => "another faction",
    }
}

/// How many of one item record a single actor may hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioItemStack {
    /// At most one, however many times the item is granted.
    Unique,
    /// Up to this many; the bound is authored and capped at load.
    Stackable(u32),
}

impl ScenarioItemStack {
    fn limit(self) -> u32 {
        match self {
            Self::Unique => 1,
            Self::Stackable(max) => max,
        }
    }
    fn valid(self) -> bool {
        (1..=64).contains(&self.limit())
    }
}

/// The closed set of outcomes an item may name. A pack cannot invent an effect:
/// every variant here is something `FactionWorld` already performs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScenarioItemEffect {
    /// Credited once to the holder's faction when the item is granted. The key
    /// is a plain string until the resource catalogue contract owns it.
    GrantResource { resource: String, amount: u32 },
    /// Raises the holder's own combat numbers for as long as the item is held.
    CombatBonus {
        health: u32,
        damage: u32,
        range: u32,
    },
}

/// One authored item record. Nothing here is executable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioItem {
    pub display_name: String,
    pub stack: ScenarioItemStack,
    pub effect: ScenarioItemEffect,
}

impl ScenarioItem {
    fn valid(&self) -> bool {
        if self.display_name.trim().is_empty() || !self.stack.valid() {
            return false;
        }
        match &self.effect {
            ScenarioItemEffect::GrantResource { resource, amount } => {
                resource.starts_with("resource.") && (1..=100_000).contains(amount)
            }
            // A held bonus must not push the resolved range past the cap that
            // `invalid_saved_combat_profile` enforces on a base profile.
            ScenarioItemEffect::CombatBonus {
                health,
                damage,
                range,
            } => *health <= 10_000 && *damage <= 10_000 && *range <= 16,
        }
    }
}

/// Every rule the island tick reads. The world carries it so a save remembers
/// which scenario's rules it was playing and a pack's rules travel with its save.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioRules {
    /// The pack's resource catalogue. It travels with the world and with its
    /// save, like every other rule, so a mod's resources survive a reload.
    pub resources: BTreeMap<String, ScenarioResource>,
    pub madness: CthulhuMadness,
    pub expansion: HoldingExpansion,
    pub repairs: BTreeMap<String, HoldingRepairRule>,
    pub salvage: HoldingSalvageRule,
    pub foothold: FootholdConfig,
    pub machinery: MichaelMachinery,
    pub machine_production: ProductionRule,
    /// The workshop's own build-out rule, lifted out of the pack's development
    /// catalogue. Every other faction carries its development rules inside its
    /// policy; Michael has no policy, so his lives here beside the machine
    /// rule, which is there for exactly the same reason. Defaulted, so a
    /// version-2 save written before the workshop could be built out loads
    /// with a workshop that simply cannot grow.
    #[serde(default)]
    pub foothold_development: Option<BuildingDevelopmentRule>,
    pub survival: SurvivalRules,
    pub footprints: BTreeMap<String, IslandBuildingFootprint>,
    pub personas: BTreeMap<String, Vec<PersonaPool>>,
    /// Defaulted so a version-2 save written before the campaign clock landed
    /// still parses; the default is deliberately invalid, so such a save is
    /// then refused by name rather than played with no deadline.
    #[serde(default)]
    pub campaign_clock: CampaignClock,
    /// The scenario's item catalogue, keyed by stable ID. A pack may carry
    /// none; a save made before items existed loads with an empty catalogue.
    #[serde(default)]
    pub items: BTreeMap<String, ScenarioItem>,
    /// The authored women this scenario's factions will produce, in authored
    /// order. A notable is never placed on the board: she is the person a
    /// building produces when its turn comes, carrying a written identity
    /// where an ordinary worker carries a rolled one. Kept on the rules so she
    /// still arrives after a save and reload.
    #[serde(default)]
    pub notables: Vec<ScenarioNotable>,
    /// The pack's companion leads, in authored order. A pack may carry none.
    #[serde(default)]
    pub leads: Vec<ScenarioLead>,
    /// The pack's triggers, in authored order, which is also firing order. A
    /// pack may carry none; a save made before triggers existed loads with an
    /// empty list and the tick's trigger pass does nothing.
    #[serde(default)]
    pub triggers: Vec<ScenarioTrigger>,
    /// The pack's quest catalogue, keyed by stable ID. A pack may carry none;
    /// a save made before quests existed loads with an empty catalogue.
    #[serde(default)]
    pub quests: BTreeMap<String, ScenarioQuest>,
}

/// One quest: an authored stage machine. `SetQuestStage`, a trigger effect, is
/// the only thing that moves it -- a quest has no logic of its own, only stages
/// and the text each one shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioQuest {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "initialStage")]
    pub initial_stage: String,
    pub stages: BTreeMap<String, QuestStage>,
}

impl ScenarioQuest {
    fn valid(&self) -> bool {
        !self.display_name.trim().is_empty()
            && self.display_name.len() <= 128
            && !self.stages.is_empty()
            && self.stages.len() <= MAX_QUEST_STAGES
            && self.stages.contains_key(&self.initial_stage)
            && self
                .stages
                .keys()
                .all(|id| !id.is_empty() && id.len() <= 128)
            && self.stages.values().all(QuestStage::valid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestStage {
    pub objective: String,
    /// Absent for an ordinary stage. A terminal stage is where a quest
    /// settles: `SetQuestStage` refuses to move a quest out of one.
    #[serde(default)]
    pub terminal: Option<QuestOutcome>,
}

impl QuestStage {
    fn valid(&self) -> bool {
        self.objective.len() <= 1024
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestOutcome {
    Success,
    Failure,
}

/// One authored rule: when every condition holds, apply every effect. There is
/// no scripting here — both lists are closed sets the simulation implements, so
/// a trigger can neither read nor write anything `FactionWorld` does not own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioTrigger {
    pub id: String,
    /// A trigger fires once unless it says otherwise. A repeating trigger fires
    /// at most once per tick, whenever its conditions hold.
    #[serde(default)]
    pub repeat: bool,
    pub when: Vec<TriggerCondition>,
    pub then: Vec<TriggerEffect>,
}

/// A companion's lead: something she noticed in the simulated world, two
/// readings of it she can defend, and a request for the player's judgment.
///
/// A lead is deliberately built from the two closed sets triggers already use.
/// What opens it is a `TriggerCondition`, so she can only notice things the
/// simulation actually keeps; what an answer does is `TriggerEffect`s, so her
/// conclusion changes the same board everything else changes. She invents no
/// evidence and gets no private mechanism.
///
/// The lead belongs to a companion and cannot open until she is standing with
/// Michael, which is what makes the authority's third contract structural:
/// "Michael cannot receive every objective and perform all intellectual work
/// himself."
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioLead {
    pub id: String,
    /// Whose lead this is. She must be loyal to Michael before she brings it.
    pub companion_id: String,
    pub opens_when: Vec<TriggerCondition>,
    /// What she says she saw, in her own words.
    pub observation: String,
    /// What she is asking him to decide or support.
    pub request: String,
    /// Two readings she can defend. The authority says two, not one and not a
    /// menu: a lead is a judgment call, not a quiz with a right answer.
    pub interpretations: Vec<LeadInterpretation>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeadInterpretation {
    pub id: String,
    /// What she thinks it means if the player backs this reading.
    pub claim: String,
    pub then: Vec<TriggerEffect>,
}

impl ScenarioLead {
    fn valid(&self) -> bool {
        !self.id.is_empty()
            && self.id.len() <= 256
            && !self.observation.trim().is_empty()
            && !self.request.trim().is_empty()
            && !self.opens_when.is_empty()
            && self.interpretations.len() == 2
            && self.interpretations[0].id != self.interpretations[1].id
            && self.interpretations.iter().all(|interpretation| {
                !interpretation.id.is_empty()
                    && !interpretation.claim.trim().is_empty()
                    && !interpretation.then.is_empty()
            })
    }
}

impl ScenarioTrigger {
    fn valid(&self) -> bool {
        !self.id.is_empty()
            && self.id.len() <= 256
            && !self.when.is_empty()
            && self.when.len() <= 16
            && !self.then.is_empty()
            && self.then.len() <= 16
    }
}

/// Every condition reads state the simulation already keeps. Deliberately
/// absent: anything about regions. `IslandNavigation` has `walkable`,
/// `destinations` and `building_obstacles` and no region type, so "entering a
/// region" would mean inventing geography; a trigger names an explicit cell
/// until the island-network contract makes regions real.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerCondition {
    DayAtLeast {
        day: u64,
    },
    FactionEliminated {
        faction_id: String,
    },
    ResourceAtLeast {
        faction_id: String,
        resource_id: String,
        amount: u32,
    },
    /// Any operational holding of that archetype at that level or above.
    HoldingLevelAtLeast {
        faction_id: String,
        archetype_id: String,
        level: u32,
    },
    /// Any living actor of that faction standing on that cell.
    ActorAtCell {
        faction_id: String,
        x: i32,
        y: i32,
    },
    FlagSet {
        flag: String,
    },
    /// The heat ledger carries at least one event on that channel.
    HeatSignalled {
        signal: String,
    },
    ConfrontationBegun,
    /// How many actors have `person.loyal_to_michael` set -- the real state
    /// `recruit_island_person` writes, not a proxy for it.
    LoyalCompanionCount {
        at_least: u32,
    },
    /// How many of the four `party` slots `assign_island_companion` has
    /// actually filled -- the real array, not a proxy for it.
    PartySize {
        at_least: u32,
    },
}

/// Every effect is something the simulation already does, reached through the
/// verb that already guards it. No effect narrates: by the owner's decision of
/// 9 September a trigger changes the world and does not describe the change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerEffect {
    SetFlag {
        flag: String,
    },
    GrantResource {
        faction_id: String,
        resource_id: String,
        amount: u32,
    },
    GrantItem {
        actor_id: String,
        item_id: String,
    },
    RecordHeat {
        id: String,
        signal: String,
        severity: u32,
    },
    /// Hostility both ways, as `advance_diplomacy` sets it. Turning two
    /// factions hostile ends any survival truce between them, so the
    /// load-time invariant that a truce pair is never hostile still holds.
    SetHostility {
        a: String,
        b: String,
        hostile: bool,
    },
    /// A quest's only mover. Refused if the quest or stage is undeclared, if
    /// the quest is already on that stage, or if its current stage is
    /// terminal: once a quest completes or fails it stays there.
    SetQuestStage {
        quest_id: String,
        stage_id: String,
    },
}

impl ScenarioRules {
    fn valid(&self) -> bool {
        self.madness.valid()
            && self.expansion.valid()
            && self.foothold.valid()
            && self.survival.valid()
            && self.machinery.capacity > 0
            && self.machine_production.production_ticks > 0
            && !self.footprints.is_empty()
            && !self.personas.is_empty()
            && !self.repairs.is_empty()
            && self.campaign_clock.valid()
            && self.resources.contains_key(&self.salvage.resource_id)
            && self.items.len() <= 256
            && self.items.values().all(ScenarioItem::valid)
            && self.leads.len() <= MAX_TRIGGERS
            && self.leads.iter().all(ScenarioLead::valid)
            && {
                let ids: BTreeSet<&String> = self.leads.iter().map(|lead| &lead.id).collect();
                ids.len() == self.leads.len()
            }
            && self.triggers.len() <= MAX_TRIGGERS
            && self.triggers.iter().all(ScenarioTrigger::valid)
            && {
                let ids: BTreeSet<&String> = self.triggers.iter().map(|t| &t.id).collect();
                ids.len() == self.triggers.len()
            }
            && self.quests.len() <= MAX_QUESTS
            && self.quests.values().all(ScenarioQuest::valid)
    }

    /// The catalogue's answer, and the only one: an undeclared key is not a
    /// resource, however plausible it looks.
    fn knows(&self, resource_id: &str) -> bool {
        self.resources.contains_key(resource_id)
    }
}

/// A manifest's `items` key: the bundle builder embeds the pack's catalogue
/// document there, or leaves the contract's empty array when it carries none.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ScenarioItems {
    Records(BTreeMap<String, ScenarioItem>),
    /// `[]`: this pack has no item catalogue.
    Absent(Vec<serde_json::Value>),
}

impl Default for ScenarioItems {
    fn default() -> Self {
        Self::Absent(Vec::new())
    }
}

impl ScenarioItems {
    fn valid(&self) -> bool {
        match self {
            Self::Records(records) => {
                !records.is_empty()
                    && records.len() <= 256
                    && records.values().all(ScenarioItem::valid)
                    // The prefix names the domain (docs/ARCHITECTURE.md).
                    && records.keys().all(|id| {
                        id.split('.').count() >= 2
                            && id.split('.').all(|segment| {
                                !segment.is_empty()
                                    && segment
                                        .bytes()
                                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                            })
                    })
            }
            Self::Absent(values) => values.is_empty(),
        }
    }

    fn catalogue(&self) -> BTreeMap<String, ScenarioItem> {
        match self {
            Self::Records(records) => records.clone(),
            Self::Absent(_) => BTreeMap::new(),
        }
    }
}

/// The three ways the campaign can end up at its confrontation. The scenario
/// authors the order they are arbitrated in when more than one lands on the
/// same tick; the simulation never invents a fourth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfrontationCause {
    #[serde(rename = "deliberate_discovery")]
    DeliberateDiscovery,
    #[serde(rename = "terminal_heat")]
    TerminalHeat,
    #[serde(rename = "day_100")]
    Day100,
}

impl ConfrontationCause {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeliberateDiscovery => "deliberate_discovery",
            Self::TerminalHeat => "terminal_heat",
            Self::Day100 => "day_100",
        }
    }
}

/// The simulated occurrences a scenario declares as heat. `on` is closed: the
/// simulation implements exactly these, so a pack cannot name an occurrence
/// nothing produces and quietly get a clock that never advances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatTrigger {
    MadnessConversion,
    MidnightReturn,
    FactionEliminated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeatSource {
    pub id: String,
    pub on: HeatTrigger,
    pub signal: String,
    pub severity: u32,
}

/// The campaign's clock, as the scenario authors it. It is a rule, not play
/// state: it lives on `ScenarioRules` so a mod's deadline travels with its save.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignClock {
    pub world_deadline_day: u64,
    /// The channels the island signals through. Heat is stored as a ledger of
    /// events on these channels and is never exposed as a number.
    pub signal_channels: Vec<String>,
    pub terminal_severity: u32,
    pub sources: Vec<HeatSource>,
    pub priority: Vec<ConfrontationCause>,
    /// Eliminating this faction is the player finding the truth on purpose.
    pub deliberate_discovery_faction: String,
}

impl CampaignClock {
    fn valid(&self) -> bool {
        let channels: BTreeSet<&String> = self.signal_channels.iter().collect();
        let source_ids: BTreeSet<&String> = self.sources.iter().map(|s| &s.id).collect();
        let causes: BTreeSet<ConfrontationCause> = self.priority.iter().copied().collect();
        self.world_deadline_day > 0
            && self.terminal_severity > 0
            && self.signal_channels.len() >= 5
            && self.signal_channels.iter().all(|c| !c.is_empty())
            && channels.len() == self.signal_channels.len()
            && !self.sources.is_empty()
            && self.sources.len() <= 64
            && source_ids.len() == self.sources.len()
            && self.sources.iter().all(|source| {
                !source.id.is_empty()
                    && source.id.len() <= 256
                    && (1..=100).contains(&source.severity)
                    && channels.contains(&source.signal)
            })
            && causes.len() == 3
            && self.priority.len() == 3
            && self.deliberate_discovery_faction.starts_with("faction.")
    }
}

/// One irreversible entry in the heat ledger. There is no verb that removes one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeatEvent {
    pub id: String,
    pub signal: String,
    pub severity: u32,
    pub tick: u64,
}

/// Recorded once, never rewritten: the first cause to fire settles the campaign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confrontation {
    pub cause: ConfrontationCause,
    pub tick: u64,
    pub day: u64,
}

/// The scenario pack's resolved manifest, as the bundle builder writes it.
/// Nothing here is executable: it is the data the one simulation runs on.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioDefinition {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub id: String,
    pub geography: ScenarioGeography,
    pub start: ScenarioStart,
    pub factions: Vec<ScenarioFaction>,
    pub rules: ScenarioRuleSet,
    pub buildings: BTreeMap<String, IslandBuildingFootprint>,
    pub personas: BTreeMap<String, Vec<PersonaPool>>,
    /// The authored records for every character this pack places. A character's
    /// name, sex, age and history come from here and from nowhere else, so the
    /// simulation never carries a second copy of an identity to drift from.
    #[serde(default)]
    pub characters: Vec<ScenarioCharacter>,
    /// Where the pack puts the characters it carries. Identity belongs to the
    /// record; allegiance and standing place belong to the scenario, so the
    /// same woman can be a pirate in one pack and a colonial in another.
    #[serde(default)]
    pub placements: Vec<ScenarioPlacement>,
    /// One or more catalogue documents, merged in order into one catalogue.
    #[serde(default)]
    pub resources: Vec<BTreeMap<String, ScenarioResource>>,
    #[serde(default)]
    pub items: ScenarioItems,
    /// One or more trigger documents, concatenated in order into one list.
    #[serde(default)]
    pub triggers: Vec<Vec<ScenarioTrigger>>,
    /// One or more quest catalogue documents, merged in order into one catalogue.
    #[serde(default)]
    pub quests: Vec<BTreeMap<String, ScenarioQuest>>,
    /// One or more lead documents, concatenated in order into one list.
    #[serde(default)]
    pub leads: Vec<Vec<ScenarioLead>>,
    /// Rasterised land, supplied by the caller. Polygon-to-cell rasterisation
    /// lives in GDScript until a later contract moves it into Rust, so a
    /// definition is not playable until its land is set.
    #[serde(skip)]
    pub land: Option<(BTreeSet<IslandPoint>, IslandPoint)>,
}

/// One authored character record, as much of it as the island reads. The
/// record carries far more (art, skills, provenance); those belong to the
/// presentation and battle owners, so they are deliberately not deserialised
/// here rather than copied into the simulation and left to rot.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioCharacter {
    pub id: String,
    pub island: ScenarioCharacterIdentity,
}

/// Where an authored woman comes from. She is produced into the island's
/// ordinary population, not placed above it: the faction that raises her and
/// the actor definition she is raised as give her the same sprite, combat
/// profile and provenance any other worker of that definition gets, so every
/// verb that works on a generated woman works on her.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioPlacement {
    #[serde(rename = "characterId")]
    pub character_id: String,
    #[serde(rename = "factionId")]
    pub faction_id: String,
    #[serde(rename = "definitionId")]
    pub definition_id: String,
}

/// A notable resolved against the record the pack carries, ready for the
/// faction that raises her. Identity is copied in here so it travels with the
/// save exactly as triggers and quests do.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioNotable {
    pub character_id: String,
    pub faction_id: String,
    pub definition_id: String,
    pub identity: ScenarioCharacterIdentity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioCharacterIdentity {
    #[serde(rename = "boardName")]
    pub board_name: String,
    pub sex: PersonSex,
    /// `NamedPerson` keeps an age as a `u16`; the record is read into the same
    /// width so no conversion can quietly reinterpret an authored age.
    pub age: u16,
    pub backstory: String,
    /// The persona pools fill these for a generated woman, and
    /// `recruit_island_person` reads the offer for everyone. An authored
    /// notable therefore has to author her own, or she could be talked to and
    /// never asked -- the exact trap this field exists to close.
    #[serde(rename = "recruitmentOffer", default)]
    pub recruitment_offer: String,
    #[serde(rename = "companionResponse", default)]
    pub companion_response: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioGeography {
    #[serde(rename = "terrainTexture")]
    pub terrain_texture: String,
    pub navigation: serde_json::Value,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioStart {
    #[serde(rename = "captainId")]
    pub captain_id: String,
    #[serde(rename = "combatProfile")]
    pub combat_profile: IslandCombatProfile,
    /// The captain's own faction opens with this. Optional, and empty in the
    /// main pack, because Michael starts the island with nothing but a wreck.
    #[serde(default)]
    pub economy: ScenarioFactionEconomy,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioFaction {
    pub id: String,
    pub seed: [i32; 2],
    pub tuning: IslandFactionTuning,
    pub production: ProductionRule,
    pub economy: ScenarioFactionEconomy,
}

/// The manifest's `rules` block. Keys are the contract's; the runtime splits
/// them into the saved `ScenarioRules` and the start-only rules beside it.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioRuleSet {
    #[serde(rename = "cthulhuMadness")]
    pub madness: CthulhuMadness,
    #[serde(rename = "colonialExpansion")]
    pub expansion: HoldingExpansion,
    #[serde(rename = "holdingRepairs")]
    pub repairs: BTreeMap<String, HoldingRepairRule>,
    #[serde(rename = "holdingSalvage")]
    pub salvage: HoldingSalvageRule,
    #[serde(rename = "holdingDevelopment")]
    pub development: BTreeMap<String, BuildingDevelopmentRule>,
    #[serde(rename = "michaelFoothold")]
    pub foothold: FootholdConfig,
    #[serde(rename = "michaelMachinery")]
    pub machinery: MichaelMachinery,
    /// Provisional key: Michael's workshop production rule has no slot in the
    /// contract's manifest, and the simulation reads it every tick.
    #[serde(rename = "michaelMachineProduction")]
    pub machine_production: ProductionRule,
    #[serde(rename = "survivalDiplomacy")]
    pub survival: SurvivalRules,
    #[serde(rename = "initialDiplomacy")]
    pub initial_diplomacy: InitialDiplomacy,
    #[serde(rename = "campaignClock")]
    pub campaign_clock: ScenarioCampaignClock,
}

/// The authored campaign-clock record, embedded verbatim by the bundle builder.
/// Its shape is `content/schemas/campaign_clock.schema.json`; the descriptive
/// keys (`id`, `kind`, `irreversible`, `causes`) are the record's own contract
/// and the schema holds them, so the runtime reads only what it executes.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioCampaignClock {
    #[serde(rename = "worldDeadlineDay")]
    pub world_deadline_day: u64,
    pub heat: ScenarioHeatRules,
    pub confrontation: ScenarioConfrontationRules,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioHeatRules {
    #[serde(rename = "signalChannels")]
    pub signal_channels: Vec<String>,
    #[serde(rename = "terminalSeverity")]
    pub terminal_severity: u32,
    pub sources: Vec<HeatSource>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ScenarioConfrontationRules {
    #[serde(rename = "sameTransactionPriority")]
    pub priority: Vec<ConfrontationCause>,
    #[serde(rename = "deliberateDiscoveryFactionId")]
    pub deliberate_discovery_faction: String,
}

impl ScenarioCampaignClock {
    fn resolve(&self) -> CampaignClock {
        CampaignClock {
            world_deadline_day: self.world_deadline_day,
            signal_channels: self.heat.signal_channels.clone(),
            terminal_severity: self.heat.terminal_severity,
            sources: self.heat.sources.clone(),
            priority: self.confrontation.priority.clone(),
            deliberate_discovery_faction: self.confrontation.deliberate_discovery_faction.clone(),
        }
    }
}

impl ScenarioDefinition {
    /// Parse one generated scenario document (`game/generated/scenarios/<id>.json`).
    pub fn from_document(text: &str) -> Result<Self, String> {
        if text.len() > 8 * 1024 * 1024 {
            return Err("scenario_too_large".into());
        }
        #[derive(Deserialize)]
        struct Document {
            format: String,
            version: u32,
            scenario: serde_json::Value,
        }
        let document: Document =
            serde_json::from_str(text).map_err(|_| "invalid_scenario_document".to_string())?;
        if document.format != "project42.scenario" || document.version != 1 {
            return Err("unsupported_scenario_document".into());
        }
        let definition: Self = serde_json::from_value(document.scenario)
            .map_err(|_| "invalid_scenario_manifest".to_string())?;
        if definition.schema_version != 1
            || !definition.id.starts_with("scenario.")
            || !definition.items.valid()
            || !definition
                .triggers
                .iter()
                .flatten()
                .all(ScenarioTrigger::valid)
            || !definition
                .quests
                .iter()
                .flatten()
                .all(|(_, quest)| quest.valid())
        {
            return Err("invalid_scenario_manifest".into());
        }
        definition.resource_catalogue()?;
        definition.quest_catalogue()?;
        Ok(definition)
    }

    /// Supply the rasterised land and the captain's start cell.
    pub fn set_land(&mut self, cells: BTreeSet<IslandPoint>, start: IslandPoint) -> bool {
        if cells.is_empty() || cells.len() > 16384 || !cells.contains(&start) {
            return false;
        }
        self.land = Some((cells, start));
        true
    }

    /// Every catalogue document merged into one catalogue. A key declared twice
    /// is a pack error, not a silent overwrite.
    pub fn resource_catalogue(&self) -> Result<BTreeMap<String, ScenarioResource>, String> {
        let mut catalogue: BTreeMap<String, ScenarioResource> = BTreeMap::new();
        for document in &self.resources {
            for (id, record) in document {
                if !valid_resource_id(id)
                    || record.display_name.trim().is_empty()
                    || record.display_name.len() > 128
                    || catalogue.insert(id.clone(), record.clone()).is_some()
                {
                    return Err("invalid_scenario_resources".into());
                }
            }
        }
        if catalogue.len() > MAXIMUM_RESOURCES {
            return Err("invalid_scenario_resources".into());
        }
        Ok(catalogue)
    }

    /// Every quest document merged into one catalogue. A key declared twice is
    /// a pack error, not a silent overwrite; `ScenarioQuest::valid` catches a
    /// malformed stage machine (an unreachable initial stage, an empty
    /// catalogue) once the merge itself has succeeded.
    pub fn quest_catalogue(&self) -> Result<BTreeMap<String, ScenarioQuest>, String> {
        let mut catalogue: BTreeMap<String, ScenarioQuest> = BTreeMap::new();
        for document in &self.quests {
            for (id, record) in document {
                if !valid_quest_id(id) || catalogue.insert(id.clone(), record.clone()).is_some() {
                    return Err("invalid_scenario_quests".into());
                }
            }
        }
        if catalogue.len() > MAX_QUESTS {
            return Err("invalid_scenario_quests".into());
        }
        Ok(catalogue)
    }

    pub fn scenario_rules(&self) -> ScenarioRules {
        ScenarioRules {
            resources: self.resource_catalogue().unwrap_or_default(),
            madness: self.rules.madness.clone(),
            expansion: self.rules.expansion.clone(),
            repairs: self.rules.repairs.clone(),
            salvage: self.rules.salvage.clone(),
            foothold: self.rules.foothold.clone(),
            machinery: self.rules.machinery.clone(),
            machine_production: self.rules.machine_production.clone(),
            foothold_development: self
                .rules
                .development
                .get("site_archetype.michael.field_workshop")
                .cloned(),
            survival: self.rules.survival.clone(),
            footprints: self.buildings.clone(),
            personas: self.personas.clone(),
            campaign_clock: self.rules.campaign_clock.resolve(),
            items: self.items.catalogue(),
            // Every path that builds rules from a definition carries the
            // roster, including a legacy save migrated into this scenario --
            // otherwise a migrated campaign would quietly never raise anyone.
            // A placement whose record is missing is dropped here and refused
            // by `from_scenario`, which is where saying no belongs.
            notables: self
                .placements
                .iter()
                .filter_map(|placement| {
                    let record = self
                        .characters
                        .iter()
                        .find(|character| character.id == placement.character_id)?;
                    Some(ScenarioNotable {
                        character_id: placement.character_id.clone(),
                        faction_id: placement.faction_id.clone(),
                        definition_id: placement.definition_id.clone(),
                        identity: record.island.clone(),
                    })
                })
                .collect(),
            leads: self.leads.iter().flatten().cloned().collect(),
            triggers: self.triggers.iter().flatten().cloned().collect(),
            quests: self.quest_catalogue().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionWorld {
    /// The scenario's rules travel with the world and with its save.
    pub rules: ScenarioRules,
    #[serde(default)]
    pub survival_truces: Vec<SurvivalTruce>,
    /// One entry per authored pair, seeded from the relationship matrix.
    #[serde(default)]
    pub standings: Vec<FactionStanding>,
    #[serde(default)]
    pub diplomacy_notices: Vec<DiplomacyNotice>,
    #[serde(default)]
    pub salvage_caches: BTreeMap<String, SalvageCache>,
    #[serde(default)]
    pub clock: IslandClock,
    /// Fixed slots preserve deliberate replacement and dead companion identity.
    #[serde(default)]
    pub party: [String; 4],
    #[serde(default)]
    pub approach_target: Option<String>,
    #[serde(default)]
    pub player_attack_target: Option<String>,
    pub tick: u64,
    pub paused: bool,
    pub factions: BTreeMap<String, FactionState>,
    pub actors: BTreeMap<String, ProducedActor>,
    pub navigation: IslandNavigation,
    pub positions: BTreeMap<String, IslandPoint>,
    pub travel_orders: BTreeMap<String, String>,
    #[serde(default)]
    pub policies: BTreeMap<String, FactionPolicy>,
    #[serde(default)]
    pub eliminated_factions: BTreeSet<String>,
    #[serde(default)]
    pub combat_profiles: BTreeMap<String, IslandCombatProfile>,
    #[serde(default)]
    pub unit_combat: BTreeMap<String, IslandCombatState>,
    /// What each actor carries, keyed by actor id like every other side table.
    /// It moves with the person, so recruitment cannot strip her equipment.
    /// An empty inventory writes nothing, so a world with no items saves
    /// exactly as it did before items existed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inventories: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub hostilities: BTreeSet<(String, String)>,
    #[serde(default)]
    pub casualties: BTreeMap<String, IslandCasualty>,
    /// The heat ledger: append-only, in the order it happened. There is no
    /// verb that removes an entry and none that reports its sum to the player.
    #[serde(default)]
    pub heat: Vec<HeatEvent>,
    /// Set once, by the first cause to fire. Never rewritten.
    #[serde(default)]
    pub confrontation: Option<Confrontation>,
    /// World flags a trigger has set. A flag is a name and nothing else: the
    /// simulation never attaches meaning to one, only conditions read it.
    #[serde(default)]
    pub flags: BTreeSet<String>,
    /// Trigger ID to the tick it last fired on. A non-repeating trigger with an
    /// entry here never fires again, across a save and a load.
    #[serde(default)]
    pub fired_triggers: BTreeMap<String, u64>,
    /// Lead ID to the tick its companion brought it to Michael. A lead sits
    /// here until the player answers it; an unanswered lead is a decision the
    /// campaign is still waiting on, which is why it survives a save.
    #[serde(default)]
    pub open_leads: BTreeMap<String, u64>,
    /// Lead ID to the interpretation the player backed. A lead is answered
    /// once: the record is what she believes now because he agreed to it.
    #[serde(default)]
    pub resolved_leads: BTreeMap<String, String>,
    /// Quest ID to its current stage ID. Seeded from each declared quest's
    /// `initialStage` when the world is created; only `SetQuestStage` moves it.
    #[serde(default)]
    pub quest_stages: BTreeMap<String, String>,
    next_actor_serial: u64,
    next_order_serial: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandCombatProfile {
    pub health: u32,
    pub damage: u32,
    pub range: u32,
    pub cooldown_ticks: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandClock {
    pub ticks_per_day: u64,
}

impl Default for IslandClock {
    fn default() -> Self {
        // Provisional pacing, not a wall-clock promise. Saved with each campaign.
        Self {
            ticks_per_day: 1440,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandCombatState {
    pub health: u32,
    pub next_attack_tick: u64,
    pub population_use: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandCasualty {
    pub actor: ProducedActor,
    pub position: IslandPoint,
    pub death_tick: u64,
    pub population_use: u32,
}

/// Scenario-authored economy and priorities; no universal recruitment power.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionPolicy {
    #[serde(default)]
    pub development: BTreeMap<String, BuildingDevelopmentRule>,
    pub income_per_tick: BTreeMap<String, u32>,
    pub storage_caps: BTreeMap<String, u32>,
    pub production: BTreeMap<String, ProductionRule>,
    pub objectives: Vec<DispatchCandidate>,
}

/// Physical navigation cells, not rooms or strategic graph nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IslandPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IslandNavigation {
    pub walkable: BTreeSet<IslandPoint>,
    pub destinations: BTreeMap<String, IslandPoint>,
    #[serde(default)]
    pub building_obstacles: BTreeMap<String, BTreeSet<IslandPoint>>,
}

impl IslandNavigation {
    pub fn traversable(&self, point: IslandPoint) -> bool {
        self.walkable.contains(&point)
            && !self
                .building_obstacles
                .values()
                .any(|cells| cells.contains(&point))
    }
    fn clear_line(&self, start: IslandPoint, goal: IslandPoint) -> bool {
        // Combat profiles cap range at load time; use wide arithmetic at map edges.
        let (mut x, mut y) = (i64::from(start.x), i64::from(start.y));
        let (gx, gy) = (i64::from(goal.x), i64::from(goal.y));
        let dx = (gx - x).abs();
        let dy = -(gy - y).abs();
        let sx = if x < gx { 1 } else { -1 };
        let sy = if y < gy { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            if !self.traversable(IslandPoint {
                x: x as i32,
                y: y as i32,
            }) {
                return false;
            }
            if x == gx && y == gy {
                return true;
            }
            let doubled = error * 2;
            if doubled >= dy {
                error += dy;
                x += sx;
            }
            if doubled <= dx {
                error += dx;
                y += sy;
            }
        }
    }
    /// Deterministic shortest path over equal-cost land cells. Bounded by the
    /// finite authored walkable set; never traverses ocean or blocked cells.
    pub fn path(&self, start: IslandPoint, goal: IslandPoint) -> Option<Vec<IslandPoint>> {
        if !self.traversable(start) || !self.traversable(goal) {
            return None;
        }
        let mut frontier = VecDeque::from([start]);
        let mut parents = BTreeMap::from([(start, start)]);
        while let Some(point) = frontier.pop_front() {
            if point == goal {
                let mut path = vec![goal];
                let mut cursor = goal;
                while cursor != start {
                    cursor = parents[&cursor];
                    path.push(cursor);
                }
                path.reverse();
                return Some(path);
            }
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                let (Some(x), Some(y)) = (point.x.checked_add(dx), point.y.checked_add(dy)) else {
                    continue;
                };
                let next = IslandPoint { x, y };
                if self.traversable(next) && !parents.contains_key(&next) {
                    parents.insert(next, point);
                    frontier.push_back(next);
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridCube {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCube {
    fn translated(self, offset: GridCube) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CubeModule {
    pub origin: GridCube,
    pub size: [u16; 3],
}

impl CubeModule {
    fn cubes(&self) -> impl Iterator<Item = GridCube> + '_ {
        (0..i32::from(self.size[0])).flat_map(move |x| {
            (0..i32::from(self.size[1])).flat_map(move |y| {
                (0..i32::from(self.size[2])).map(move |z| GridCube {
                    x: self.origin.x + x,
                    y: self.origin.y + y,
                    z: self.origin.z + z,
                })
            })
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildingVolume {
    pub grid_standard: String,
    pub modules: Vec<CubeModule>,
}

impl BuildingVolume {
    pub fn occupied_local_cubes(&self) -> Result<BTreeSet<GridCube>, MapPlacementError> {
        if self.grid_standard != "map_cube_v1" {
            return Err(MapPlacementError::UnsupportedGridStandard(
                self.grid_standard.clone(),
            ));
        }
        if self.modules.is_empty() {
            return Err(MapPlacementError::EmptyVolume);
        }
        let mut cubes = BTreeSet::new();
        for module in &self.modules {
            if module.size.contains(&0) {
                return Err(MapPlacementError::ZeroSizedModule);
            }
            for cube in module.cubes() {
                if !cubes.insert(cube) {
                    return Err(MapPlacementError::OverlappingModules(cube));
                }
            }
        }
        Ok(cubes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedBuilding {
    pub id: String,
    pub origin: GridCube,
    pub volume: BuildingVolume,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapPlacementError {
    DuplicateBuilding(String),
    UnsupportedGridStandard(String),
    EmptyVolume,
    ZeroSizedModule,
    OverlappingModules(GridCube),
    CubeOccupied {
        cube: GridCube,
        occupying_building_id: String,
    },
    UnknownPlacedBuilding(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapPlacement {
    pub buildings: BTreeMap<String, PlacedBuilding>,
    pub occupied_cubes: BTreeMap<GridCube, String>,
}

impl MapPlacement {
    pub fn place_building(
        &mut self,
        id: impl Into<String>,
        origin: GridCube,
        volume: BuildingVolume,
    ) -> Result<(), MapPlacementError> {
        let id = id.into();
        if self.buildings.contains_key(&id) {
            return Err(MapPlacementError::DuplicateBuilding(id));
        }
        let local_cubes = volume.occupied_local_cubes()?;
        let world_cubes: Vec<_> = local_cubes
            .into_iter()
            .map(|cube| cube.translated(origin))
            .collect();
        for cube in &world_cubes {
            if let Some(occupying_building_id) = self.occupied_cubes.get(cube) {
                return Err(MapPlacementError::CubeOccupied {
                    cube: *cube,
                    occupying_building_id: occupying_building_id.clone(),
                });
            }
        }
        for cube in world_cubes {
            self.occupied_cubes.insert(cube, id.clone());
        }
        self.buildings
            .insert(id.clone(), PlacedBuilding { id, origin, volume });
        Ok(())
    }

    pub fn remove_building(&mut self, id: &str) -> Result<PlacedBuilding, MapPlacementError> {
        let building = self
            .buildings
            .remove(id)
            .ok_or_else(|| MapPlacementError::UnknownPlacedBuilding(id.to_owned()))?;
        self.occupied_cubes.retain(|_, occupant| occupant != id);
        Ok(building)
    }
}

impl FactionWorld {
    fn diplomacy_viable(&self, id: &str) -> bool {
        !self.eliminated_factions.contains(id)
            && self
                .factions
                .get(id)
                .is_some_and(|f| f.buildings.values().any(|b| b.health > 0))
    }

    fn military_strength(&self, faction: &str) -> u64 {
        self.actors
            .iter()
            .filter(|(id, a)| a.faction_id == faction && self.living_actor(id))
            .filter_map(|(id, _)| {
                self.actor_combat_profile(id)
                    .filter(|p| p.damage > 0)
                    .map(|p| {
                        u64::from(self.unit_combat[id].health) * 100
                            + u64::from(p.damage) * self.rules.survival.strength_horizon_ticks * 100
                                / u64::from(p.cooldown_ticks.max(1))
                    })
            })
            .sum()
    }

    fn common_dominant_threat(&self, a: &str, b: &str, enemy: &str) -> bool {
        a != enemy
            && b != enemy
            && self.diplomacy_viable(enemy)
            && self.hostilities.contains(&(a.into(), enemy.into()))
            && self.hostilities.contains(&(b.into(), enemy.into()))
            && self.military_strength(enemy).saturating_mul(100)
                > self
                    .military_strength(a)
                    .max(self.military_strength(b))
                    .max(1)
                    .saturating_mul(self.rules.survival.dominance_percent)
    }

    fn record_diplomacy(&mut self, factions: Vec<String>, text: String) {
        self.diplomacy_notices.push(DiplomacyNotice {
            tick: self.tick,
            factions,
            text,
        });
        if self.diplomacy_notices.len() > 16 {
            self.diplomacy_notices.remove(0);
        }
    }

    fn advance_diplomacy(&mut self) {
        let rules = self.rules.survival.clone();
        if self.tick == 0 || self.tick % rules.decision_ticks.max(1) != 0 {
            return;
        }
        const CTHULHU: &str = "faction.cthulhu.prototype";
        let ordinary: Vec<String> = [
            "faction.colonial_powers.prototype",
            "faction.eastern_fox_people.prototype",
            "faction.elves.prototype",
            "faction.pirates.prototype",
        ]
        .into_iter()
        .filter(|id| self.diplomacy_viable(id) && self.policies.contains_key(*id))
        .map(String::from)
        .collect();
        if self.diplomacy_viable(CTHULHU) {
            for id in &ordinary {
                if !self.hostilities.contains(&(id.clone(), CTHULHU.into()))
                    && self.military_strength(CTHULHU).saturating_mul(100)
                        > self
                            .military_strength(id)
                            .max(1)
                            .saturating_mul(rules.dominance_percent)
                {
                    self.hostilities.insert((id.clone(), CTHULHU.into()));
                    self.hostilities.insert((CTHULHU.into(), id.clone()));
                    self.record_diplomacy(
                        vec![id.clone(), CTHULHU.into()],
                        format!(
                            "{} have turned against Cthulhu's faction as its army grows.",
                            faction_label(id)
                        ),
                    );
                }
            }
        }
        self.advance_standings(&rules);
        let mut retained = Vec::new();
        for mut truce in std::mem::take(&mut self.survival_truces) {
            if !self.diplomacy_viable(&truce.a) || !self.diplomacy_viable(&truce.b) {
                continue;
            }
            if self.tick < truce.expires_tick {
                retained.push(truce);
                continue;
            }
            if self.common_dominant_threat(&truce.a, &truce.b, &truce.threat) {
                truce.expires_tick = self.tick.saturating_add(rules.truce_ticks);
                retained.push(truce);
            } else {
                if truce.resume_ab {
                    self.hostilities.insert((truce.a.clone(), truce.b.clone()));
                }
                if truce.resume_ba {
                    self.hostilities.insert((truce.b.clone(), truce.a.clone()));
                }
                self.record_diplomacy(
                    vec![truce.a.clone(), truce.b.clone()],
                    format!(
                        "The temporary truce between {} and {} has ended.",
                        faction_label(&truce.a),
                        faction_label(&truce.b)
                    ),
                );
            }
        }
        self.survival_truces = retained;
        let enemies: Vec<String> = ordinary
            .iter()
            .cloned()
            .chain(std::iter::once(CTHULHU.into()))
            .collect();
        for (i, a) in ordinary.iter().enumerate() {
            for b in ordinary.iter().skip(i + 1) {
                if self.survival_truces.iter().any(|t| &t.a == a && &t.b == b) {
                    continue;
                }
                let ab = self.hostilities.contains(&(a.clone(), b.clone()));
                let ba = self.hostilities.contains(&(b.clone(), a.clone()));
                if !ab && !ba {
                    continue;
                }
                let threat = enemies
                    .iter()
                    .filter(|e| self.common_dominant_threat(a, b, e))
                    .max_by_key(|e| (self.military_strength(e), *e))
                    .cloned();
                let Some(threat) = threat else {
                    continue;
                };
                self.hostilities.remove(&(a.clone(), b.clone()));
                self.hostilities.remove(&(b.clone(), a.clone()));
                self.survival_truces.push(SurvivalTruce {
                    a: a.clone(),
                    b: b.clone(),
                    threat: threat.clone(),
                    expires_tick: self.tick.saturating_add(rules.truce_ticks),
                    resume_ab: ab,
                    resume_ba: ba,
                });
                self.record_diplomacy(
                    vec![a.clone(), b.clone()],
                    format!(
                        "{} and {} have agreed to a temporary truce against {}.",
                        faction_label(a),
                        faction_label(b),
                        faction_label(&threat)
                    ),
                );
            }
        }
        let obsolete: Vec<_> = self
            .actors
            .iter()
            .filter_map(|(id, a)| {
                let building = a.current_assignment_id.as_deref()?.strip_prefix("siege.")?;
                let owner = self
                    .factions
                    .values()
                    .find(|f| f.buildings.contains_key(building));
                if owner.is_none_or(|f| {
                    !self
                        .hostilities
                        .contains(&(a.faction_id.clone(), f.id.clone()))
                }) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        for id in obsolete {
            self.cancel_actor_travel(&id);
            self.actors.get_mut(&id).unwrap().current_assignment_id = None;
        }
    }

    /// Fear, weariness and quiet time, applied to every authored pair.
    ///
    /// Three forces, and only three. A faction overshadowed by a neighbour
    /// spends the relationship down; an open war spends itself out, so a war
    /// can end in exhaustion rather than only in somebody's extinction; and a
    /// pair left alone drifts back toward the standing the setting authored.
    /// Crossing `war_standing` opens a war and crossing `peace_standing` ends
    /// one, which is what keeps the island moving after the opening war.
    fn advance_standings(&mut self, rules: &SurvivalRules) {
        let viable: BTreeSet<String> = self
            .factions
            .keys()
            .filter(|id| self.diplomacy_viable(id) && self.policies.contains_key(*id))
            .cloned()
            .collect();
        let strengths: BTreeMap<String, u64> = viable
            .iter()
            .map(|id| (id.clone(), self.military_strength(id)))
            .collect();
        let mut declared = Vec::new();
        let mut settled = Vec::new();
        for standing in &mut self.standings {
            if !viable.contains(&standing.a) || !viable.contains(&standing.b) {
                continue;
            }
            let at_war = self
                .hostilities
                .contains(&(standing.a.clone(), standing.b.clone()))
                || self
                    .hostilities
                    .contains(&(standing.b.clone(), standing.a.clone()));
            let (a_strength, b_strength) = (strengths[&standing.a], strengths[&standing.b]);
            let overshadowed = a_strength.max(b_strength).saturating_mul(100)
                > a_strength
                    .min(b_strength)
                    .max(1)
                    .saturating_mul(rules.dominance_percent);
            if at_war {
                standing.value = standing.value.saturating_add(rules.war_weariness_step);
            } else if overshadowed {
                standing.value = standing.value.saturating_sub(rules.fear_step);
            } else if standing.value < standing.baseline {
                standing.value = standing
                    .value
                    .saturating_add(rules.recovery_step)
                    .min(standing.baseline);
            }
            standing.value = standing.value.clamp(-100, 100);
            if !at_war && standing.value <= rules.war_standing {
                declared.push((standing.a.clone(), standing.b.clone()));
            } else if at_war && standing.value >= rules.peace_standing {
                settled.push((standing.a.clone(), standing.b.clone()));
            }
        }
        for (a, b) in declared {
            // A pair already holding together against a larger threat does not
            // fall out over standing; the truce is the island's one brake here.
            if self
                .survival_truces
                .iter()
                .any(|t| (t.a == a && t.b == b) || (t.a == b && t.b == a))
            {
                continue;
            }
            self.hostilities.insert((a.clone(), b.clone()));
            self.hostilities.insert((b.clone(), a.clone()));
            self.record_diplomacy(
                vec![a.clone(), b.clone()],
                format!(
                    "{} and {} have come to open war.",
                    faction_label(&a),
                    faction_label(&b)
                ),
            );
        }
        for (a, b) in settled {
            self.hostilities.remove(&(a.clone(), b.clone()));
            self.hostilities.remove(&(b.clone(), a.clone()));
            self.record_diplomacy(
                vec![a.clone(), b.clone()],
                format!(
                    "{} and {} have stopped fighting.",
                    faction_label(&a),
                    faction_label(&b)
                ),
            );
        }
    }

    pub fn island_person_news(&self, id: &str) -> String {
        if !self.can_talk_island_person(id) || !self.living_person(id).is_some_and(|p| p.discussed)
        {
            return String::new();
        }
        let faction = &self.actors[id].faction_id;
        // What she has actually heard lately.
        //
        // The diplomacy record has always been written and never read once:
        // wars opened, truces were struck and holdings fell, and every person
        // on the island went on saying the same standing sentence about who
        // they were at war with. A companion repeats anything recent, because
        // she walks with Michael and hears everything; anyone else only
        // repeats what touched their own people. Nothing new is invented here
        // and there is no feed -- it is the same record, finally spoken by
        // somebody.
        let loyal = self
            .living_person(id)
            .is_some_and(|person| person.loyal_to_michael);
        let horizon = self.clock.ticks_per_day.max(1);
        if let Some(notice) = self.diplomacy_notices.iter().rev().find(|notice| {
            self.tick.saturating_sub(notice.tick) <= horizon
                && (loyal || notice.factions.iter().any(|other| other == faction))
        }) {
            return notice.text.clone();
        }
        if let Some(truce) = self
            .survival_truces
            .iter()
            .find(|t| &t.a == faction || &t.b == faction)
        {
            let partner = if &truce.a == faction {
                &truce.b
            } else {
                &truce.a
            };
            let mut news = format!("We have a temporary truce with {}.", faction_label(partner));
            if self.diplomacy_viable(&truce.threat) {
                news.push_str(&format!(
                    " For now, {} are the greater danger.",
                    faction_label(&truce.threat)
                ));
            }
            return news;
        }
        let enemies: Vec<_> = self
            .factions
            .keys()
            .filter(|other| {
                !self.eliminated_factions.contains(*other)
                    && (self.diplomacy_viable(other)
                        || self
                            .actors
                            .iter()
                            .any(|(id, a)| &a.faction_id == *other && self.living_actor(id)))
                    && self
                        .hostilities
                        .contains(&(faction.clone(), (*other).clone()))
            })
            .map(|id| faction_label(id))
            .collect();
        if enemies.is_empty() {
            "We are not fighting another faction at present.".into()
        } else {
            format!("We are at war with {}.", enemies.join(", "))
        }
    }
    pub fn building_construction_ticks(&self, building: &FactionBuilding) -> u32 {
        if building.archetype_id == "site_archetype.michael.field_workshop" {
            self.rules.foothold.construction_ticks
        } else if self.rules.expansion.building_id == building.id {
            self.rules.expansion.construction_ticks
        } else {
            0
        }
    }

    pub fn building_repair_ticks(&self, building: &FactionBuilding) -> u32 {
        self.rules
            .repairs
            .get(&building.archetype_id)
            .map(|r| r.ticks)
            .unwrap_or(0)
    }

    fn building_work_ready(&self, building: &FactionBuilding, job: &BuildingWork) -> bool {
        self.actors
            .get(&job.builder_id)
            .is_some_and(|a| a.faction_id == building.faction_id)
            && self
                .navigation
                .destinations
                .get(&building.node_id)
                .is_some_and(|p| self.within_work_range(&job.builder_id, *p))
    }

    fn assigned_to_other_work(&self, actor_id: &str, action: &str) -> bool {
        self.actors
            .get(actor_id)
            .and_then(|a| a.current_assignment_id.as_deref())
            .is_some_and(|a| {
                a != action && (a.starts_with("repair.") || a.starts_with("construct."))
            })
            || self
                .factions
                .values()
                .flat_map(|f| f.buildings.values())
                .any(|b| {
                    [
                        ("repair", b.repair.as_ref()),
                        ("construct", b.construction.as_ref()),
                    ]
                    .into_iter()
                    .any(|(kind, job)| {
                        job.is_some_and(|j| {
                            j.builder_id == actor_id && format!("{kind}.{}", b.id) != action
                        })
                    })
                })
    }

    fn actively_repairing(&self, id: &str) -> bool {
        self.factions
            .values()
            .flat_map(|f| f.buildings.values())
            .any(|b| {
                b.repair
                    .as_ref()
                    .is_some_and(|j| j.builder_id == id && self.building_work_ready(b, j))
            })
    }

    /// The resource the scenario pays salvage in. One owner of the key, read
    /// from the pack's rules, so no faction stockpile is addressed by literal.
    pub fn salvage_resource(&self) -> &str {
        &self.rules.salvage.resource_id
    }

    /// What a faction holds of one resource. Reading an undeclared key is
    /// simply nothing; only writing one is an error.
    pub fn stored_resource(&self, faction_id: &str, resource_id: &str) -> u32 {
        self.factions
            .get(faction_id)
            .and_then(|faction| faction.resources.get(resource_id))
            .copied()
            .unwrap_or(0)
    }

    /// Whether a faction can pay a cost in full. Every affordability test in
    /// the simulation asks this, so none of them can drift from the spend.
    fn can_afford(&self, faction_id: &str, costs: &BTreeMap<String, u32>) -> bool {
        costs
            .iter()
            .all(|(resource, cost)| self.stored_resource(faction_id, resource) >= *cost)
    }

    /// The one way a stockpile grows. An undeclared key is refused rather than
    /// created, and the gain is clamped to the faction's storage cap wherever
    /// one is declared - income is not the only way resources arrive.
    fn gain_resource(
        &mut self,
        faction_id: &str,
        resource_id: &str,
        amount: u32,
    ) -> Result<(), String> {
        if !self.rules.knows(resource_id) {
            return Err(format!("unknown_resource:{resource_id}"));
        }
        let cap = self
            .policies
            .get(faction_id)
            .and_then(|policy| policy.storage_caps.get(resource_id))
            .copied();
        let faction = self
            .factions
            .get_mut(faction_id)
            .ok_or_else(|| format!("unknown_faction:{faction_id}"))?;
        let stored = faction.resources.entry(resource_id.to_owned()).or_default();
        let gained = stored.saturating_add(amount);
        *stored = cap.map_or(gained, |cap| gained.min(cap));
        Ok(())
    }

    /// The one way a stockpile shrinks. Every key is checked and every cost is
    /// affordable before anything is subtracted, so a spend is all or nothing.
    fn spend_resources(
        &mut self,
        faction_id: &str,
        costs: &BTreeMap<String, u32>,
    ) -> Result<(), String> {
        for resource in costs.keys() {
            if !self.rules.knows(resource) {
                return Err(format!("unknown_resource:{resource}"));
            }
        }
        if !self.can_afford(faction_id, costs) {
            return Err("insufficient_resources".into());
        }
        let faction = self
            .factions
            .get_mut(faction_id)
            .ok_or_else(|| format!("unknown_faction:{faction_id}"))?;
        for (resource, cost) in costs {
            let stored = faction.resources.entry(resource.clone()).or_default();
            *stored = stored.saturating_sub(*cost);
        }
        Ok(())
    }

    fn begin_holding_repair(
        &mut self,
        faction_id: &str,
        building_id: &str,
        builder: &str,
    ) -> Result<(), String> {
        let building = self
            .factions
            .get(faction_id)
            .and_then(|f| f.buildings.get(building_id))
            .ok_or("No holding to repair.")?;
        let rule = self
            .rules
            .repairs
            .get(&building.archetype_id)
            .ok_or("This holding cannot be repaired.")?;
        if building.repair.is_some() {
            return Err("Repairs are already underway.".into());
        }
        if !building.operational
            || building.construction.is_some()
            || building.development.is_some()
        {
            return Err("Finish the building work first.".into());
        }
        if building.health >= building.max_health {
            return Err("The holding is undamaged.".into());
        }
        let action = format!("repair.{building_id}");
        let job = BuildingWork {
            builder_id: builder.into(),
            remaining_ticks: rule.ticks,
            reserved_costs: rule.costs.clone(),
        };
        if !self.building_work_ready(building, &job)
            || self.assigned_to_other_work(builder, &action)
        {
            return Err("A free builder must be beside the holding.".into());
        }
        let costs = rule.costs.clone();
        self.spend_resources(faction_id, &costs)
            .map_err(|_| "Not enough repair materials.".to_string())?;
        let faction = self.factions.get_mut(faction_id).unwrap();
        faction.buildings.get_mut(building_id).unwrap().repair = Some(job);
        self.cancel_actor_travel(builder);
        self.actors.get_mut(builder).unwrap().current_assignment_id = Some(action);
        Ok(())
    }

    pub fn repair_foothold(&mut self) -> Result<(), String> {
        self.begin_holding_repair(
            "faction.michael",
            "site.michael.field_workshop",
            "character.protagonist.captain",
        )
    }

    pub fn foothold_repair_cost(&self) -> u32 {
        self.rules
            .repairs
            .get("site_archetype.michael.field_workshop")
            .and_then(|r| r.costs.get(self.salvage_resource()))
            .copied()
            .unwrap_or(0)
    }

    pub fn machine_foothold_costs(&self) -> (u32, u32, u32) {
        let rule = &self.rules.machine_production;
        (
            rule.costs
                .get(self.salvage_resource())
                .copied()
                .unwrap_or(0),
            rule.production_ticks,
            self.machine_berths() as u32,
        )
    }

    /// How many dogs the workshop can hold. Building the workshop out adds a
    /// berth per level, which is the whole point of building it out: Michael
    /// cannot raise soldiers, so machines are the only force he can grow.
    pub fn machine_berths(&self) -> usize {
        let level = self
            .factions
            .get("faction.michael")
            .and_then(|f| f.buildings.get("site.michael.field_workshop"))
            .map_or(1, |b| b.level) as usize;
        self.rules
            .machinery
            .capacity
            .saturating_add(level.saturating_sub(1))
    }

    /// What the next level of the workshop costs, or nothing when it is
    /// finished. The rule is the same one every faction's holding grows by.
    pub fn foothold_development_cost(&self) -> u32 {
        let Some(building) = self
            .factions
            .get("faction.michael")
            .and_then(|f| f.buildings.get("site.michael.field_workshop"))
        else {
            return 0;
        };
        self.rules
            .foothold_development
            .as_ref()
            .filter(|rule| building.level < rule.max_level)
            .and_then(|rule| rule.costs.get(self.salvage_resource()))
            .map_or(0, |cost| cost.saturating_mul(building.level))
    }

    /// Build the workshop out one level, by Michael's own hands.
    ///
    /// Every faction's holding grows through `policy.development`; Michael has
    /// no policy and never will, so his one building had no way to become
    /// anything, and `build_foothold` answered "Michael already has a workshop
    /// site." for the rest of the campaign. This is the same authored rule,
    /// paid and started by the player instead of by an AI.
    pub fn develop_foothold(&mut self) -> Result<(), String> {
        const FACTION: &str = "faction.michael";
        const SITE: &str = "site.michael.field_workshop";
        let building = self
            .factions
            .get(FACTION)
            .and_then(|f| f.buildings.get(SITE))
            .ok_or("Build the workshop first.")?;
        let rule = self
            .rules
            .foothold_development
            .clone()
            .ok_or("The workshop cannot be built out.")?;
        let entrance = *self
            .navigation
            .destinations
            .get(&building.node_id)
            .ok_or("The workshop is unavailable.")?;
        if !self.within_work_range("character.protagonist.captain", entrance) {
            return Err("Bring Michael beside the workshop.".into());
        }
        if !building.operational
            || building.construction.is_some()
            || building.development.is_some()
            || building.repair.is_some()
            || !building.production_queue.is_empty()
        {
            return Err("Finish the workshop first.".into());
        }
        if building.level >= rule.max_level {
            return Err("The workshop is already built out.".into());
        }
        if building.health != building.max_health {
            return Err("Repair the workshop first.".into());
        }
        let level = building.level;
        let costs: BTreeMap<String, u32> = rule
            .costs
            .iter()
            .map(|(resource, cost)| (resource.clone(), cost.saturating_mul(level)))
            .collect();
        self.spend_resources(FACTION, &costs)
            .map_err(|_| "Not enough salvage to build the workshop out.".to_string())?;
        let remaining_ticks = rule.ticks;
        self.factions
            .get_mut(FACTION)
            .unwrap()
            .buildings
            .get_mut(SITE)
            .unwrap()
            .development = Some(BuildingDevelopment {
            rule,
            remaining_ticks,
            reserved_costs: costs,
        });
        Ok(())
    }

    pub fn machine_display_name(&self) -> &str {
        &self.rules.machinery.display_name
    }

    fn mechanical_dog_count(&self) -> usize {
        let definition = &self.rules.machinery.definition_id;
        self.actors
            .values()
            .filter(|a| a.faction_id == "faction.michael" && &a.definition_id == definition)
            .count()
            + self
                .factions
                .get("faction.michael")
                .into_iter()
                .flat_map(|f| f.buildings.values())
                .flat_map(|b| &b.production_queue)
                .filter(|o| &o.rule.actor_definition_id == definition)
                .count()
    }

    pub fn queue_foothold_machine(&mut self) -> Result<(), String> {
        let faction = "faction.michael";
        let site = "site.michael.field_workshop";
        let building = self
            .factions
            .get(faction)
            .and_then(|f| f.buildings.get(site))
            .ok_or("Build the workshop first.")?;
        let entrance = self
            .navigation
            .destinations
            .get(&building.node_id)
            .ok_or("The workshop is unavailable.")?;
        if !self.within_work_range("character.protagonist.captain", *entrance) {
            return Err("Bring Michael beside the workshop.".into());
        }
        if !building.operational
            || building.construction.is_some()
            || building.development.is_some()
        {
            return Err("Finish the workshop first.".into());
        }
        if !building.production_queue.is_empty() {
            return Err("The workshop is already building a machine.".into());
        }
        if self.mechanical_dog_count() >= self.machine_berths() {
            return Err("All mechanical dog berths are occupied.".into());
        }
        let machine = self.rules.machine_production.clone();
        self.enqueue_production(faction, site, machine)
            .map_err(|_| "Not enough salvage to build a mechanical dog.".to_string())?;
        let machinery = self.rules.machinery.clone();
        self.combat_profiles
            .insert(machinery.definition_id, machinery.combat);
        Ok(())
    }

    fn advance_holding_repairs(&mut self) {
        let sites: Vec<_> = self
            .factions
            .values()
            .filter(|f| self.policies.contains_key(&f.id))
            .flat_map(|f| f.buildings.values())
            .map(|b| (b.faction_id.clone(), b.id.clone()))
            .collect();
        for (faction_id, building_id) in sites {
            let building = &self.factions[&faction_id].buildings[&building_id];
            let action = format!("repair.{building_id}");
            let Some(rule) = self.rules.repairs.get(&building.archetype_id) else {
                continue;
            };
            if building.development.is_some()
                && u64::from(building.health) * 100
                    < u64::from(building.max_health) * u64::from(rule.emergency_health_percent)
            {
                // Abandon the upgrade, not concurrent work: reserved materials
                // are lost and its unfinished level/capacity gain is never paid.
                self.factions
                    .get_mut(&faction_id)
                    .unwrap()
                    .buildings
                    .get_mut(&building_id)
                    .unwrap()
                    .development = None;
            }
            let building = &self.factions[&faction_id].buildings[&building_id];
            if !building.operational
                || building.construction.is_some()
                || building.development.is_some()
                || building.health == building.max_health
                || (building.repair.is_none()
                    && rule.costs.iter().any(|(r, c)| {
                        self.factions[&faction_id]
                            .resources
                            .get(r)
                            .copied()
                            .unwrap_or(0)
                            < *c
                    }))
            {
                self.release_construction_assignment(&faction_id, &action);
                continue;
            }
            let Some(entrance) = self.navigation.destinations.get(&building.node_id).copied()
            else {
                continue;
            };
            let node = building.node_id.clone();
            let incumbent = building.repair.as_ref().map(|j| j.builder_id.as_str());
            let mut candidates: Vec<_> = self
                .actors
                .iter()
                .filter_map(|(id, a)| {
                    if a.faction_id != faction_id
                        || !matches!(a.actor_kind.as_str(), "worker" | "soldier")
                        || !self.living_actor(id)
                        || self.assigned_to_other_work(id, &action)
                    {
                        return None;
                    }
                    let path = self.navigation.path(self.positions[id], entrance)?;
                    Some((
                        !(incumbent == Some(id.as_str())
                            || a.current_assignment_id.as_deref() == Some(&action)),
                        path.len(),
                        id.clone(),
                    ))
                })
                .collect();
            candidates.sort();
            let Some((_, _, builder)) = candidates.into_iter().next() else {
                self.release_construction_assignment(&faction_id, &action);
                continue;
            };
            // Cancel a displaced builder's order before assigning its replacement.
            self.release_construction_assignment(&faction_id, &action);
            if let Some(job) = self
                .factions
                .get_mut(&faction_id)
                .unwrap()
                .buildings
                .get_mut(&building_id)
                .unwrap()
                .repair
                .as_mut()
            {
                job.builder_id = builder.clone();
            }
            if self.within_work_range(&builder, entrance) {
                self.actors.get_mut(&builder).unwrap().current_assignment_id = Some(action.clone());
                if self.factions[&faction_id].buildings[&building_id]
                    .repair
                    .is_none()
                    && self
                        .begin_holding_repair(&faction_id, &building_id, &builder)
                        .is_err()
                {
                    self.release_construction_assignment(&faction_id, &action);
                }
            } else {
                let order = DispatchCandidate {
                    action_id: action,
                    assignment: "repair".into(),
                    target_node_id: node,
                    score: DispatchScore::default(),
                    wobble: 0,
                };
                let _ = self.dispatch_actor(&builder, &[order]);
            }
        }
    }

    fn release_construction_assignment(&mut self, faction: &str, action: &str) {
        for actor in self.actors.values_mut().filter(|a| {
            a.faction_id == faction && a.current_assignment_id.as_deref() == Some(action)
        }) {
            actor.current_assignment_id = None;
            self.travel_orders.remove(&actor.instance_id);
        }
    }

    fn advance_holding_expansion(&mut self) {
        let rule = self.rules.expansion.clone();
        if !self.policies.contains_key(&rule.faction_id) || self.tick < rule.minimum_tick {
            return;
        }
        let Some(faction) = self.factions.get(&rule.faction_id) else {
            return;
        };
        let existing = faction.buildings.get(&rule.building_id);
        let action = format!("construct.{}", rule.building_id);
        if existing.is_some_and(|b| b.operational) {
            for actor in self.actors.values_mut().filter(|a| {
                a.faction_id == rule.faction_id
                    && a.current_assignment_id.as_deref() == Some(&action)
            }) {
                actor.current_assignment_id = None;
                self.travel_orders.remove(&actor.instance_id);
            }
            return;
        }
        // Construction survives the loss of the original base, but a new
        // expedition is funded only from a healthy existing settlement.
        if existing.is_none()
            && (!faction
                .buildings
                .values()
                .any(|b| (b.operational || b.development.is_some()) && b.health == b.max_health)
                || rule
                    .costs
                    .iter()
                    .any(|(r, c)| faction.resources.get(r).copied().unwrap_or(0) < *c))
        {
            self.release_construction_assignment(&rule.faction_id, &action);
            return;
        }
        if existing.is_none()
            && self.tick % 8 != 0
            && !self.actors.values().any(|a| {
                a.faction_id == rule.faction_id
                    && a.current_assignment_id.as_deref() == Some(&action)
            })
        {
            return;
        }
        let entrance = IslandPoint {
            x: rule.entrance[0],
            y: rule.entrance[1],
        };
        let assigned = existing
            .and_then(|b| b.construction.as_ref())
            .map(|j| j.builder_id.as_str());
        let mut candidates: Vec<_> = self
            .actors
            .iter()
            .filter_map(|(id, a)| {
                if a.faction_id != rule.faction_id
                    || a.actor_kind != "soldier"
                    || self.assigned_to_other_work(id, &action)
                    || !self.living_actor(id)
                {
                    return None;
                }
                let distance = self.navigation.path(self.positions[id], entrance)?.len();
                Some((
                    !(assigned == Some(id.as_str())
                        || a.current_assignment_id.as_deref() == Some(&action)),
                    distance,
                    id.clone(),
                ))
            })
            .collect();
        candidates.sort();
        let Some((_, _, builder)) = candidates.into_iter().next() else {
            return;
        };
        let template = self.factions[&rule.faction_id]
            .buildings
            .values()
            .find(|b| b.archetype_id == rule.archetype_id)
            .cloned();
        let Some(template) = template else {
            return;
        };
        self.navigation
            .destinations
            .insert(rule.building_id.clone(), entrance);
        if let Some(job) = self
            .factions
            .get_mut(&rule.faction_id)
            .unwrap()
            .buildings
            .get_mut(&rule.building_id)
            .and_then(|b| b.construction.as_mut())
        {
            job.builder_id = builder.clone();
        }
        // Reach the open entrance before placing walls: a builder two cells
        // away may still occupy the future foundation and invalidate the site.
        if self.positions.get(&builder) != Some(&entrance) {
            if self.travel_orders.get(&builder) != Some(&rule.building_id) {
                let order = DispatchCandidate {
                    action_id: action.clone(),
                    assignment: "construct".into(),
                    target_node_id: rule.building_id.clone(),
                    score: DispatchScore::default(),
                    wobble: 0,
                };
                let _ = self.dispatch_actor(&builder, &[order]);
            }
            return;
        }
        self.actors.get_mut(&builder).unwrap().current_assignment_id = Some(action);
        self.travel_orders.remove(&builder);
        if !self.factions[&rule.faction_id]
            .buildings
            .contains_key(&rule.building_id)
        {
            let building = FactionBuilding {
                id: rule.building_id.clone(),
                faction_id: rule.faction_id.clone(),
                archetype_id: rule.archetype_id.clone(),
                node_id: rule.building_id.clone(),
                rally_point_id: rule.building_id.clone(),
                level: 1,
                health: rule.holding_health,
                max_health: rule.holding_health,
                operational: false,
                queue_capacity: 1,
                production_queue: Vec::new(),
                development: None,
                repair: None,
                construction: Some(BuildingWork {
                    builder_id: builder,
                    remaining_ticks: rule.construction_ticks,
                    reserved_costs: rule.costs.clone(),
                }),
            };
            if let Err(_reason) = self.begin_building_construction(building, entrance) {
                self.release_construction_assignment(
                    &rule.faction_id,
                    &format!("construct.{}", rule.building_id),
                );
                return;
            }
            let policy = self.policies.get_mut(&rule.faction_id).unwrap();
            if let Some(production) = policy.production.get(&template.id).cloned() {
                policy
                    .production
                    .insert(rule.building_id.clone(), production);
            }
            if let Some(development) = policy.development.get(&template.id).cloned() {
                policy
                    .development
                    .insert(rule.building_id.clone(), development);
            }
        }
    }
    pub fn foothold_costs(&self) -> Result<(u32, u32, u32), String> {
        let config = &self.rules.foothold;
        Ok((
            config.build_salvage,
            config.restore_salvage,
            config.construction_ticks,
        ))
    }
    fn within_work_range(&self, actor_id: &str, point: IslandPoint) -> bool {
        self.living_actor(actor_id)
            && self.positions.get(actor_id).is_some_and(|p| {
                p.x.abs_diff(point.x).saturating_add(p.y.abs_diff(point.y)) <= 2
                    && self.navigation.clear_line(*p, point)
            })
    }

    pub fn salvage_foothold(&mut self) -> Result<(), String> {
        let id = self
            .nearby_salvage_id()
            .ok_or("Bring Michael within reach of uncollected salvage.")?
            .to_owned();
        if !self.factions.contains_key("faction.michael") {
            return Err("Michael's faction is unavailable.".into());
        }
        let recovered = self.salvage_caches[&id].remaining;
        let salvage = self.salvage_resource().to_owned();
        // A pickup is a gain like any other: checked against the catalogue and
        // clamped by whatever cap the faction declares.
        self.gain_resource("faction.michael", &salvage, recovered)
            .map_err(|_| "Salvage is not a resource this scenario declares.".to_string())?;
        self.salvage_caches.get_mut(&id).unwrap().remaining = 0;
        Ok(())
    }

    pub fn nearby_salvage_id(&self) -> Option<&str> {
        let origin = self.positions.get("character.protagonist.captain")?;
        self.salvage_caches
            .iter()
            .filter(|(_, cache)| {
                cache.remaining > 0
                    && self.within_work_range("character.protagonist.captain", cache.position)
            })
            .map(|(id, c)| {
                (
                    origin
                        .x
                        .abs_diff(c.position.x)
                        .saturating_add(origin.y.abs_diff(c.position.y)),
                    id.as_str(),
                )
            })
            .min()
            .map(|(_, id)| id)
    }

    fn record_holding_salvage(&mut self, building: &FactionBuilding) {
        // An unfinished foundation has not made recoverable machinery. No
        // production or upgrade reservations are refunded through these ruins.
        if building.construction.is_some() {
            return;
        }
        let Some(position) = self.navigation.destinations.get(&building.node_id).copied() else {
            return;
        };
        let rule = &self.rules.salvage;
        let amount = if building.archetype_id == "site_archetype.michael.field_workshop" {
            rule.workshop_yield
        } else {
            rule.base_yield
                .saturating_add(rule.per_completed_level.saturating_mul(building.level))
        };
        let label = match building.archetype_id.as_str() {
            "site_archetype.colonial.watch_fort" => "Watch fort salvage",
            "site_archetype.pirates.tide_quay" => "Quay salvage",
            "site_archetype.eastern_fox_people.river_market" => "River market salvage",
            "site_archetype.elven.heart_grove" => "Heart grove salvage",
            "site_archetype.cthulhu.drowned_shrine" => "Shrine salvage",
            "site_archetype.michael.field_workshop" => "Workshop salvage",
            _ => "Holding salvage",
        };
        let id = format!("salvage.ruins.{}.{}", building.id, self.tick);
        // One stable record for this destruction; never refill a collected entry.
        self.salvage_caches.entry(id).or_insert(SalvageCache {
            position,
            remaining: amount,
            initial_amount: amount,
            label: label.into(),
            source_building_id: building.id.clone(),
            source_faction: building.faction_id.clone(),
            level: building.level,
            created_tick: self.tick,
        });
    }

    pub fn build_foothold(&mut self, entrance: IslandPoint) -> Result<(), String> {
        const CAPTAIN: &str = "character.protagonist.captain";
        const BUILDING: &str = "site.michael.field_workshop";
        const ARCHETYPE: &str = "site_archetype.michael.field_workshop";
        let config = self.rules.foothold.clone();
        if !self.within_work_range(CAPTAIN, entrance) {
            return Err("Bring Michael within reach of the workshop site.".into());
        }
        let faction = self
            .factions
            .get("faction.michael")
            .ok_or("Michael's faction is unavailable.")?;
        if faction.buildings.contains_key(BUILDING) {
            return Err("Michael already has a workshop site.".into());
        }
        let salvage = self.salvage_resource().to_owned();
        if self.stored_resource("faction.michael", &salvage) < config.build_salvage {
            return Err(format!(
                "Need {} salvage to build the workshop.",
                config.build_salvage
            ));
        }
        let footprint = self
            .rules
            .footprints
            .get(ARCHETYPE)
            .ok_or("The workshop building art is not available yet.")?;
        if footprint.blocked_offsets.is_empty()
            || footprint
                .placement_entrance
                .is_some_and(|[x, y]| entrance != (IslandPoint { x, y }))
        {
            return Err("Choose the reviewed workshop site.".into());
        }
        self.begin_building_construction(
            FactionBuilding {
                repair: None,
                construction: Some(BuildingWork {
                    builder_id: CAPTAIN.into(),
                    remaining_ticks: config.construction_ticks,
                    reserved_costs: [(salvage.clone(), config.build_salvage)]
                        .into_iter()
                        .collect(),
                }),
                level: 1,
                max_health: config.building_health,
                health: config.building_health,
                development: None,
                id: BUILDING.into(),
                faction_id: "faction.michael".into(),
                archetype_id: ARCHETYPE.into(),
                node_id: BUILDING.into(),
                rally_point_id: BUILDING.into(),
                operational: false,
                queue_capacity: 1,
                production_queue: Vec::new(),
            },
            entrance,
        )
    }

    /// Shared paid placement transaction for the captain and autonomous builders.
    fn begin_building_construction(
        &mut self,
        building: FactionBuilding,
        entrance: IslandPoint,
    ) -> Result<(), String> {
        if self.salvage_caches.len()
            + self
                .factions
                .values()
                .map(|f| f.buildings.len())
                .sum::<usize>()
            >= 4096
        {
            return Err("The campaign salvage ledger is full.".into());
        }
        let job = building
            .construction
            .as_ref()
            .ok_or("Missing construction job.")?;
        let builder = job.builder_id.clone();
        let costs = job.reserved_costs.clone();
        let faction_id = building.faction_id.clone();
        let building_id = building.id.clone();
        if !self.within_work_range(&builder, entrance)
            || self
                .actors
                .get(&builder)
                .is_none_or(|a| a.faction_id != faction_id)
            || self.factions[&faction_id]
                .buildings
                .contains_key(&building_id)
            || !self.can_afford(&faction_id, &costs)
        {
            return Err("Builder, site, or resources are unavailable.".into());
        }
        let mut staged = self.clone();
        staged
            .navigation
            .destinations
            .insert(building.node_id.clone(), entrance);
        staged
            .factions
            .get_mut(&faction_id)
            .unwrap()
            .buildings
            .insert(building_id.clone(), building);
        let obstacles = staged.authored_building_obstacles()?;
        let cells = obstacles
            .get(&building_id)
            .ok_or("Building footprint is missing.")?;
        if cells
            .iter()
            .any(|p| !self.navigation.traversable(*p) || self.positions.values().any(|v| v == p))
        {
            return Err(
                "The site needs clear ground, away from people and other buildings.".into(),
            );
        }
        staged.navigation.building_obstacles = obstacles;
        let origin = staged.positions[&builder];
        let reachable = |point| staged.navigation.path(origin, point).is_some();
        if !reachable(entrance)
            || staged.positions.values().any(|p| !reachable(*p))
            || staged.travel_orders.iter().any(|(id, destination)| {
                match (
                    staged.positions.get(id),
                    staged.navigation.destinations.get(destination),
                ) {
                    (Some(start), Some(goal)) => staged.navigation.path(*start, *goal).is_none(),
                    _ => true,
                }
            })
            || staged
                .factions
                .values()
                .flat_map(|f| f.buildings.values())
                .any(|b| !reachable(staged.navigation.destinations[&b.node_id]))
            || staged
                .salvage_caches
                .values()
                .any(|c| c.remaining > 0 && !reachable(c.position))
        {
            return Err("The site would block an island route.".into());
        }
        staged
            .spend_resources(&faction_id, &costs)
            .map_err(|_| "Builder, site, or resources are unavailable.".to_string())?;
        *self = Self::load_json(&staged.save_json()?)?;
        Ok(())
    }

    pub fn restore_foothold_person(&mut self, id: &str) -> Result<(), String> {
        let config = self.rules.foothold.clone();
        let actor = self
            .actors
            .get(id)
            .ok_or("Select a living undead member of Michael's faction.")?;
        if actor.faction_id != "faction.michael"
            || !actor.undead
            || !self.living_actor(id)
            || !actor
                .person
                .as_ref()
                .is_some_and(|p| p.sex == PersonSex::Female && p.age.is_some_and(|age| age >= 18))
        {
            return Err("Restore an owned undead woman; reclaim her first if necessary.".into());
        }
        let workshop = self
            .factions
            .get("faction.michael")
            .and_then(|f| f.buildings.get("site.michael.field_workshop"))
            .filter(|b| b.operational && b.construction.is_none() && b.health > 0)
            .ok_or("Complete the field workshop first.")?;
        let entrance = self.navigation.destinations[&workshop.node_id];
        if !self.within_work_range("character.protagonist.captain", entrance)
            || !self.within_work_range(id, entrance)
        {
            return Err("Bring Michael and her within reach of the workshop.".into());
        }
        let health = self
            .actor_combat_profile(id)
            .ok_or("Her health profile is unavailable.")?
            .health;
        let costs: BTreeMap<String, u32> =
            [(self.salvage_resource().to_owned(), config.restore_salvage)].into();
        self.spend_resources("faction.michael", &costs)
            .map_err(|_| format!("Need {} salvage for restoration.", config.restore_salvage))?;
        self.actors.get_mut(id).unwrap().undead = false;
        self.unit_combat.get_mut(id).unwrap().health = health;
        Ok(())
    }
    pub fn day(&self) -> u64 {
        1 + self.tick / self.clock.ticks_per_day.max(1)
    }

    pub fn minute_of_day(&self) -> u16 {
        ((self.tick % self.clock.ticks_per_day.max(1)) * 1440 / self.clock.ticks_per_day.max(1))
            as u16
    }

    /// Bodies the island has finished with.
    ///
    /// The shrine may raise any corpse whenever it has room, so every death
    /// used to be kept for the rest of the campaign: the save grew without
    /// bound and the dead held their names forever, which drained the persona
    /// pools until two living people shared one. A body older than the
    /// authored span is gone -- except Michael's own dead, whose identity and
    /// party slot the game deliberately keeps.
    fn decay_casualties(&mut self) {
        let span = u64::from(self.rules.madness.corpse_persist_days)
            .saturating_mul(self.clock.ticks_per_day.max(1));
        let now = self.tick;
        self.casualties.retain(|_, casualty| {
            casualty
                .actor
                .person
                .as_ref()
                .is_some_and(|person| person.loyal_to_michael)
                || casualty.actor.faction_id == "faction.michael"
                || now.saturating_sub(casualty.death_tick) <= span
        });
    }

    fn return_midnight_casualties(&mut self) -> Vec<FactionWorldEvent> {
        const CTHULHU: &str = "faction.cthulhu.prototype";
        if self.tick == 0
            || self.tick % self.clock.ticks_per_day.max(1) != 0
            || self.eliminated_factions.contains(CTHULHU)
            || !self.factions.get(CTHULHU).is_some_and(|faction| {
                faction
                    .buildings
                    .values()
                    .any(|b| b.operational && b.health > 0)
            })
        {
            return Vec::new();
        }
        let mut occupied: BTreeSet<_> = self.positions.values().copied().collect();
        let mut events = Vec::new();
        // BTreeMap order provides reproducible allocation when corpses overlap.
        for id in self.casualties.keys().cloned().collect::<Vec<_>>() {
            let casualty = &self.casualties[&id];
            if id == "character.protagonist.captain"
                || casualty.actor.actor_kind == "machine"
                || casualty.death_tick >= self.tick
                || casualty.actor.provenance.producer_building_id.is_empty()
                || self.actors.len() >= 4096
            {
                continue;
            }
            // She keeps what she carried, so her restored health is resolved
            // against her own inventory, not her definition's bare profile.
            let Some(profile) = self.actor_combat_profile(&id) else {
                continue;
            };
            let restored_health = profile.health;
            let Some(population) = self.factions[CTHULHU]
                .population_used
                .checked_add(casualty.population_use)
            else {
                continue;
            };
            // The dead wait their turn like everyone else. Resurrection used to
            // raise the cap to fit whoever returned, which made every death
            // feed Cthulhu permanently: a bigger cap produced more units, which
            // died and returned and raised the cap again. The island reached
            // 309 people in a faction authored to hold 6, every other faction
            // was gone by day two, and the simulation sat frozen from day four
            // to the Day 100 deadline. She is still eligible -- she returns on
            // a night when there is room for her.
            if population > self.factions[CTHULHU].population_capacity {
                continue;
            }
            let corpse = casualty.position;
            let mut candidates = Vec::new();
            for dx in -4_i32..=4 {
                for dy in -4_i32..=4 {
                    let distance = dx.unsigned_abs() + dy.unsigned_abs();
                    if distance > 4 {
                        continue;
                    }
                    let (Some(x), Some(y)) = (corpse.x.checked_add(dx), corpse.y.checked_add(dy))
                    else {
                        continue;
                    };
                    let point = IslandPoint { x, y };
                    if !occupied.contains(&point)
                        && self
                            .navigation
                            .path(corpse, point)
                            .is_some_and(|path| path.len() <= 5)
                    {
                        candidates.push((distance, point));
                    }
                }
            }
            let Some((_, position)) = candidates.into_iter().min() else {
                continue;
            };
            let mut casualty = self.casualties.remove(&id).unwrap();
            let previous_faction_id = casualty.actor.faction_id.clone();
            casualty.actor.faction_id = CTHULHU.into();
            casualty.actor.undead = true;
            casualty.actor.madness = 0;
            casualty.actor.current_assignment_id = None;
            if let Some(person) = casualty.actor.person.as_mut() {
                person.return_at_midnight();
            }
            // This is an explicit supernatural transfer, not another production
            // order: provenance and serials stay unchanged, and the faction's
            // authored capacity is the bound it was always meant to be.
            let faction = self.factions.get_mut(CTHULHU).unwrap();
            faction.population_used = population;
            self.unit_combat.insert(
                id.clone(),
                IslandCombatState {
                    health: restored_health,
                    next_attack_tick: self.tick.saturating_add(1),
                    population_use: casualty.population_use,
                },
            );
            let node = format!("move.{id}");
            self.navigation.destinations.insert(node.clone(), position);
            casualty.actor.node_id = node;
            self.travel_orders.remove(&id);
            self.positions.insert(id.clone(), position);
            self.actors.insert(id.clone(), casualty.actor);
            occupied.insert(position);
            events.push(FactionWorldEvent::MidnightReturned {
                actor_id: id,
                previous_faction_id,
                position,
            });
        }
        events
    }
    pub fn aim_carbine(&mut self, target: &str) -> bool {
        const CAPTAIN: &str = "character.protagonist.captain";
        let (Some(player), Some(other)) = (self.actors.get(CAPTAIN), self.actors.get(target))
        else {
            return false;
        };
        let (Some(from), Some(to), Some(profile)) = (
            self.positions.get(CAPTAIN),
            self.positions.get(target),
            self.actor_combat_profile(CAPTAIN),
        ) else {
            return false;
        };
        if player.faction_id == other.faction_id
            || !self.unit_combat.get(target).is_some_and(|s| s.health > 0)
            || from.x.abs_diff(to.x).saturating_add(from.y.abs_diff(to.y)) > profile.range
            || !self.navigation.clear_line(*from, *to)
        {
            return false;
        }
        self.cancel_approach();
        self.player_attack_target = Some(target.into());
        true
    }

    /// What one actor carries, in the order it was granted.
    pub fn actor_inventory(&self, actor_id: &str) -> Vec<String> {
        self.inventories.get(actor_id).cloned().unwrap_or_default()
    }

    /// Put one catalogue item in one actor's hands. Items reach the world only
    /// through a declared source; this round that source is a direct grant.
    /// Triggers become a source of their own under their own contract.
    pub fn grant_item(&mut self, actor_id: &str, item_id: &str) -> Result<(), String> {
        if !self.actors.contains_key(actor_id) {
            return Err("unknown_item_actor".into());
        }
        let Some(item) = self.rules.items.get(item_id).cloned() else {
            return Err("unknown_item".into());
        };
        let held = self.inventories.entry(actor_id.to_string()).or_default();
        if held.len() >= MAX_ACTOR_INVENTORY {
            return Err("inventory_full".into());
        }
        let count = held.iter().filter(|id| *id == item_id).count() as u32;
        if count >= item.stack.limit() {
            return Err("item_stack_full".into());
        }
        held.push(item_id.to_string());
        if let ScenarioItemEffect::GrantResource { resource, amount } = &item.effect {
            // Credited once, through the same clamp the tick's income uses, so
            // an item cannot put a faction over a storage cap it declares.
            let faction_id = self.actors[actor_id].faction_id.clone();
            let cap = self
                .policies
                .get(&faction_id)
                .and_then(|policy| policy.storage_caps.get(resource))
                .copied();
            if let Some(faction) = self.factions.get_mut(&faction_id) {
                let stored = faction.resources.entry(resource.clone()).or_default();
                *stored = stored.saturating_add(*amount).min(cap.unwrap_or(u32::MAX));
            }
        }
        Ok(())
    }

    /// One actor's combat numbers: the profile its definition declares, raised
    /// by every `CombatBonus` it carries. `IslandCombatProfile` is keyed by
    /// definition, so this resolve-on-read is the per-actor override; it holds
    /// no second copy of the numbers, so nothing can fall out of step with the
    /// inventory. An actor carrying nothing resolves to its base profile.
    pub fn actor_combat_profile(&self, actor_id: &str) -> Option<IslandCombatProfile> {
        let actor = self
            .actors
            .get(actor_id)
            .or_else(|| self.casualties.get(actor_id).map(|c| &c.actor))?;
        let mut profile = self.combat_profiles.get(&actor.definition_id)?.clone();
        for item_id in self.inventories.get(actor_id).into_iter().flatten() {
            if let Some(ScenarioItem {
                effect:
                    ScenarioItemEffect::CombatBonus {
                        health,
                        damage,
                        range,
                    },
                ..
            }) = self.rules.items.get(item_id)
            {
                profile.health = profile.health.saturating_add(*health);
                profile.damage = profile.damage.saturating_add(*damage);
                profile.range = profile.range.saturating_add(*range);
            }
        }
        Some(profile)
    }

    fn island_firing_target(&self, id: &str) -> Option<&String> {
        if self.actively_repairing(id) {
            return None;
        }
        let actor = self.actors.get(id)?;
        let state = self.unit_combat.get(id)?;
        if state.health == 0 {
            return None;
        }
        let profile = self.actor_combat_profile(id)?;
        let origin = self.positions.get(id)?;
        self.actors
            .iter()
            .filter(|(other_id, other)| {
                if actor.faction_id == other.faction_id {
                    return false;
                }
                if id == "character.protagonist.captain" {
                    return self.player_attack_target.as_ref() == Some(*other_id)
                        && self
                            .unit_combat
                            .get(*other_id)
                            .is_some_and(|s| s.health > 0);
                }
                self.hostilities
                    .contains(&(actor.faction_id.clone(), other.faction_id.clone()))
                    && self
                        .unit_combat
                        .get(*other_id)
                        .is_some_and(|v| v.health > 0)
            })
            .filter_map(|(other_id, _)| {
                let point = self.positions.get(other_id)?;
                let distance = origin
                    .x
                    .abs_diff(point.x)
                    .saturating_add(origin.y.abs_diff(point.y));
                (distance <= profile.range && self.navigation.clear_line(*origin, *point))
                    .then_some((distance, other_id))
            })
            .min()
            .map(|(_, id)| id)
    }

    fn island_siege_target(&self, id: &str) -> Option<(String, String, IslandPoint)> {
        if self.actively_repairing(id) {
            return None;
        }
        let actor = self.actors.get(id)?;
        let origin = *self.positions.get(id)?;
        let profile = self.actor_combat_profile(id)?;
        if !self.policies.contains_key(&actor.faction_id)
            || !self.unit_combat.get(id).is_some_and(|s| s.health > 0)
        {
            return None;
        }
        self.factions
            .values()
            .filter(|f| {
                self.hostilities
                    .contains(&(actor.faction_id.clone(), f.id.clone()))
            })
            .flat_map(|f| f.buildings.values())
            .filter_map(|b| {
                let position = *self.navigation.destinations.get(&b.node_id)?;
                let distance = origin
                    .x
                    .abs_diff(position.x)
                    .saturating_add(origin.y.abs_diff(position.y));
                (b.health > 0
                    && distance <= profile.range
                    && self.navigation.clear_line(origin, position))
                .then_some((distance, b.id.clone(), b.faction_id.clone(), position))
            })
            .min()
            .map(|(_, building, faction, position)| (building, faction, position))
    }

    fn resolve_island_skirmish(&mut self) -> Vec<FactionWorldEvent> {
        // Allegiance can change between command submission and resolution.
        // Cancel an obsolete shot, rather than preserving a latent attack that
        // could fire after another later allegiance change.
        if self.player_attack_target.as_ref().is_some_and(|target| {
            match (
                self.actors.get("character.protagonist.captain"),
                self.actors.get(target),
            ) {
                (Some(player), Some(other)) => player.faction_id == other.faction_id,
                _ => true,
            }
        }) {
            self.player_attack_target = None;
        }
        let mut strikes = Vec::new();
        let mut building_strikes = Vec::new();
        for (id, actor) in &self.actors {
            let (Some(state), Some(profile)) =
                (self.unit_combat.get(id), self.actor_combat_profile(id))
            else {
                continue;
            };
            if state.health == 0 || self.tick < state.next_attack_tick {
                continue;
            }
            if let Some(target_id) = self.island_firing_target(id) {
                strikes.push((
                    id.clone(),
                    target_id.clone(),
                    profile.damage,
                    profile.cooldown_ticks,
                ));
            } else if self.policies.contains_key(&actor.faction_id) {
                let Some(origin) = self.positions.get(id).copied() else {
                    continue;
                };
                if let Some((building, faction, destination)) = self.island_siege_target(id) {
                    building_strikes.push((
                        id.clone(),
                        building,
                        faction,
                        origin,
                        destination,
                        actor.definition_id.clone(),
                        profile.damage,
                        profile.cooldown_ticks,
                    ));
                }
            }
        }
        let mut damage = BTreeMap::<String, u32>::new();
        let mut events = Vec::new();
        // Resolve simultaneously: ID ordering must not grant first-kill immunity.
        for (attacker_id, target_id, amount, cooldown) in strikes {
            if attacker_id == "character.protagonist.captain" {
                let first = self.actors[&attacker_id].faction_id.clone();
                let second = self.actors[&target_id].faction_id.clone();
                self.hostilities.insert((first.clone(), second.clone()));
                self.hostilities.insert((second, first));
                self.player_attack_target = None;
            }
            self.unit_combat
                .get_mut(&attacker_id)
                .unwrap()
                .next_attack_tick = self.tick.saturating_add(u64::from(cooldown.max(1)));
            let total = damage.entry(target_id.clone()).or_default();
            *total = total.saturating_add(amount);
            events.push(FactionWorldEvent::UnitStruck {
                origin: self.positions[&attacker_id],
                target_position: self.positions[&target_id],
                attacker_definition: self.actors[&attacker_id].definition_id.clone(),
                attacker_id,
                target_id,
                damage: amount,
            });
        }
        let mut building_damage = BTreeMap::<(String, String), u32>::new();
        for (attacker, building, faction, origin, destination, definition, amount, cooldown) in
            building_strikes
        {
            self.unit_combat
                .get_mut(&attacker)
                .unwrap()
                .next_attack_tick = self.tick.saturating_add(u64::from(
                cooldown
                    .max(1)
                    .saturating_mul(self.rules.survival.siege_cooldown_multiplier.max(1)),
            ));
            let total = building_damage
                .entry((faction, building.clone()))
                .or_default();
            *total = total.saturating_add(amount);
            events.push(FactionWorldEvent::UnitStruck {
                attacker_id: attacker,
                target_id: building,
                damage: amount,
                origin,
                target_position: destination,
                attacker_definition: definition,
            });
        }
        for (id, amount) in damage {
            let state = self.unit_combat.get_mut(&id).unwrap();
            state.health = state.health.saturating_sub(amount);
            if state.health != 0 {
                continue;
            }
            let state = self.unit_combat.remove(&id).unwrap();
            let mut actor = self.actors.remove(&id).unwrap();
            if let Some(person) = actor.person.as_mut() {
                person.alive_today = false;
            }
            let position = self.positions.remove(&id).unwrap();
            if let Some(faction) = self.factions.get_mut(&actor.faction_id) {
                faction.population_used =
                    faction.population_used.saturating_sub(state.population_use);
            }
            self.travel_orders.remove(&id);
            if self.player_attack_target.as_ref() == Some(&id)
                || id == "character.protagonist.captain"
            {
                self.player_attack_target = None;
            }
            self.navigation.destinations.remove(&format!("move.{id}"));
            self.casualties.insert(
                id.clone(),
                IslandCasualty {
                    actor,
                    position,
                    death_tick: self.tick,
                    population_use: state.population_use,
                },
            );
            if self.approach_target.as_ref() == Some(&id) || id == "character.protagonist.captain" {
                self.cancel_approach();
            }
            events.push(FactionWorldEvent::UnitFallen { actor_id: id });
        }
        for ((faction_id, building_id), amount) in building_damage {
            let faction = self.factions.get_mut(&faction_id).unwrap();
            let building = faction.buildings.get_mut(&building_id).unwrap();
            building.health = building.health.saturating_sub(amount);
            if building.health > 0 {
                continue;
            }
            let destroyed = building.clone();
            // Reserved queue costs are lost with the destroyed producer.
            let reserved: u32 = building
                .production_queue
                .iter()
                .map(|o| o.rule.population_use)
                .fold(0u32, u32::saturating_add);
            faction.population_used = faction.population_used.saturating_sub(reserved);
            faction.buildings.remove(&building_id);
            let no_holdings = faction.buildings.is_empty();
            self.release_construction_assignment(&faction_id, &format!("repair.{building_id}"));
            self.release_construction_assignment(&faction_id, &format!("construct.{building_id}"));
            self.navigation.building_obstacles.remove(&building_id);
            self.record_holding_salvage(&destroyed);
            if let Some(policy) = self.policies.get_mut(&faction_id) {
                policy.production.remove(&building_id);
                policy.development.remove(&building_id);
            }
            if no_holdings
                && !(faction_id == "faction.michael"
                    && self.living_actor("character.protagonist.captain"))
            {
                if let Ok(event) = self.eliminate_faction(&faction_id) {
                    events.push(event);
                }
            }
        }
        events
    }

    /// Build the island the scenario describes: its captain, its factions,
    /// their holdings, production, policies and opening diplomacy. One owner
    /// of the rules, from the pack's data instead of from compiled-in JSON.
    pub fn from_scenario(definition: &ScenarioDefinition) -> Result<Self, String> {
        let rules = definition.scenario_rules();
        if !rules.valid() {
            return Err("invalid_scenario_rules".into());
        }
        let Some((land, start)) = definition.land.clone() else {
            return Err("island_land_not_configured".into());
        };
        if !land.contains(&start) {
            return Err("island_land_not_configured".into());
        }
        let captain = definition.start.captain_id.clone();
        if captain.is_empty() || definition.factions.is_empty() {
            return Err("invalid_scenario_start".into());
        }
        // The pack must carry the record for the character it starts. Refusing
        // here rather than substituting a default keeps one owner for a hero's
        // identity: a pack that names a captain it does not carry is wrong, and
        // saying so is better than inventing a name to cover it.
        let Some(captain_record) = definition
            .characters
            .iter()
            .find(|character| character.id == captain)
        else {
            return Err("scenario_captain_record_missing".into());
        };
        let captain_identity = captain_record.island.clone();
        if captain_identity.board_name.is_empty()
            || captain_identity.backstory.is_empty()
            || captain_identity.age == 0
        {
            return Err("invalid_scenario_captain_identity".into());
        }
        let mut staged = Self {
            rules,
            ..Default::default()
        };
        // Every declared quest starts active, on its own authored stage.
        staged.quest_stages = staged
            .rules
            .quests
            .iter()
            .map(|(id, quest)| (id.clone(), quest.initial_stage.clone()))
            .collect();
        staged.navigation.walkable = land;
        staged
            .combat_profiles
            .insert(captain.clone(), definition.start.combat_profile.clone());
        staged.unit_combat.insert(
            captain.clone(),
            IslandCombatState {
                health: definition.start.combat_profile.health,
                next_attack_tick: 0,
                population_use: 1,
            },
        );
        let michael = "faction.michael".to_owned();
        // The captain's faction opens on what `start.economy` authors. The main
        // pack authors nothing, because Michael reaches the island with nothing.
        let start_economy = &definition.start.economy;
        start_economy.checked(&staged.rules.resources)?;
        staged.factions.insert(
            michael.clone(),
            FactionState {
                id: michael.clone(),
                resources: start_economy.stockpile.clone(),
                population_used: 1,
                population_capacity: 1,
                wobble_limit: 0,
                buildings: BTreeMap::new(),
            },
        );
        // A faction with nothing to earn and nothing to store needs no economic
        // policy, and the main pack's captain has neither.
        if !(start_economy.income_per_tick.is_empty() && start_economy.storage_caps.is_empty()) {
            staged
                .set_policy(
                    &michael,
                    FactionPolicy {
                        income_per_tick: start_economy.income_per_tick.clone(),
                        storage_caps: start_economy.storage_caps.clone(),
                        ..Default::default()
                    },
                )
                .map_err(|_| "invalid_scenario_economy")?;
        }
        staged.actors.insert(
            captain.clone(),
            ProducedActor {
                madness: 0,
                undead: false,
                // The hero's identity is the authored record's, read from the
                // document the pack carries. The simulation keeps no second
                // copy of it to drift from.
                person: Some(NamedPerson {
                    id: captain.clone(),
                    display_name: captain_identity.board_name.clone(),
                    alive_today: true,
                    sex: captain_identity.sex,
                    age: Some(captain_identity.age),
                    backstory: captain_identity.backstory.clone(),
                    ..Default::default()
                }),
                instance_id: captain.clone(),
                definition_id: captain.clone(),
                actor_kind: "hero".into(),
                faction_id: michael.clone(),
                node_id: "scenario.shipwreck".into(),
                current_assignment_id: None,
                provenance: ActorProductionProvenance {
                    faction_id: michael,
                    producer_building_id: String::new(),
                    production_rule_id: "scenario_start.shipwreck".into(),
                    reserved_costs: BTreeMap::new(),
                    completed_tick: 0,
                    rally_point_id: "scenario.shipwreck".into(),
                },
            },
        );
        staged.positions.insert(captain, start);
        let objective = start;
        staged
            .navigation
            .destinations
            .insert("island.contested_clearing".into(), objective);
        for entry in &definition.factions {
            let id = entry.id.as_str();
            let tuning = &entry.tuning;
            if staged.factions.contains_key(id) {
                return Err("duplicate_scenario_faction".into());
            }
            if !(1..=4096).contains(&tuning.population_capacity)
                || tuning.combat.health > 100000
                || tuning.combat.damage > 100000
                || tuning.combat.cooldown_ticks > 100000
            {
                return Err("invalid_faction_tuning".into());
            }
            let rule = entry.production.clone();
            let placement = staged
                .rules
                .footprints
                .get(&rule.producer_archetype_id)
                .and_then(|footprint| footprint.placement_entrance)
                .map(|[x, y]| IslandPoint { x, y });
            let preferred = placement.unwrap_or(IslandPoint {
                x: entry.seed[0],
                y: entry.seed[1],
            });
            let spawn = staged
                .navigation
                .walkable
                .iter()
                .filter(|point| staged.navigation.path(**point, objective).is_some())
                .min_by_key(|point| {
                    i64::from(point.x).abs_diff(i64::from(preferred.x))
                        + i64::from(point.y).abs_diff(i64::from(preferred.y))
                })
                .copied()
                .ok_or("no_connected_spawn")?;
            // Reviewed architecture must not slide into trees or inland from
            // a shoreline to satisfy a nearest-cell search.
            if placement.is_some() && spawn != preferred {
                return Err("building_entrance_unreachable".into());
            }
            let building_id = format!("preview.{id}.producer");
            let spawn_id = format!("preview.{id}.holding");
            staged
                .navigation
                .destinations
                .insert(spawn_id.clone(), spawn);
            let building = FactionBuilding {
                construction: None,
                repair: None,
                level: tuning.holding_level,
                max_health: tuning.holding_health,
                development: None,
                health: tuning.holding_health,
                id: building_id.clone(),
                faction_id: id.into(),
                archetype_id: rule.producer_archetype_id.clone(),
                node_id: spawn_id.clone(),
                rally_point_id: spawn_id,
                operational: true,
                queue_capacity: 1,
                production_queue: Vec::new(),
            };
            // The economy is authored, not inferred from the production rule:
            // a second scenario may open its factions on anything it declares.
            entry.economy.checked(&staged.rules.resources)?;
            if rule
                .costs
                .keys()
                .any(|resource| !staged.rules.knows(resource))
            {
                return Err("invalid_scenario_economy".into());
            }
            staged.factions.insert(
                id.into(),
                FactionState {
                    id: id.into(),
                    resources: entry.economy.stockpile.clone(),
                    population_used: 0,
                    population_capacity: tuning.population_capacity,
                    wobble_limit: 0,
                    buildings: [(building_id.clone(), building)].into_iter().collect(),
                },
            );
            staged
                .combat_profiles
                .insert(rule.actor_definition_id.clone(), tuning.combat.clone());
            let policy = FactionPolicy {
                development: definition
                    .rules
                    .development
                    .get(&rule.producer_archetype_id)
                    .map(|development| {
                        [(building_id.clone(), development.clone())]
                            .into_iter()
                            .collect()
                    })
                    .unwrap_or_default(),
                income_per_tick: entry.economy.income_per_tick.clone(),
                storage_caps: entry.economy.storage_caps.clone(),
                production: [(building_id, rule)].into_iter().collect(),
                objectives: vec![DispatchCandidate {
                    action_id: format!("preview.{id}.advance"),
                    assignment: "occupy_clearing".into(),
                    target_node_id: "island.contested_clearing".into(),
                    score: DispatchScore {
                        strategic_position: 10,
                        ..Default::default()
                    },
                    wobble: 0,
                }],
            };
            staged
                .set_policy(id, policy)
                .map_err(|_| "invalid_scenario_policy")?;
        }
        let initial = &definition.rules.initial_diplomacy;
        let mut pairs = BTreeSet::new();
        let count = initial.faction_ids.len();
        if count != definition.factions.len()
            || initial.relationships.len() != count * (count - 1) / 2
        {
            return Err("invalid_initial_diplomacy".into());
        }
        for row in &initial.relationships {
            let mut pair = [row.a.clone(), row.b.clone()];
            pair.sort();
            if row.a == row.b
                || !initial.faction_ids.contains(&row.a)
                || !initial.faction_ids.contains(&row.b)
                || !staged.factions.contains_key(&row.a)
                || !staged.factions.contains_key(&row.b)
                || !pairs.insert(pair)
            {
                return Err("invalid_initial_diplomacy".into());
            }
            if row.at_war {
                staged.hostilities.insert((row.a.clone(), row.b.clone()));
                staged.hostilities.insert((row.b.clone(), row.a.clone()));
            }
            let standing = row.standing.clamp(-100, 100);
            // Sorted, so a pair has exactly one row however the matrix wrote it.
            let (a, b) = if row.a <= row.b {
                (row.a.clone(), row.b.clone())
            } else {
                (row.b.clone(), row.a.clone())
            };
            staged.standings.push(FactionStanding {
                a,
                b,
                baseline: standing,
                value: standing,
            });
        }
        staged.navigation.building_obstacles = staged.authored_building_obstacles()?;
        if staged
            .positions
            .values()
            .any(|point| !staged.navigation.traversable(*point))
        {
            return Err("occupied_building_footprint".into());
        }
        // No holding may be sealed off by the installed footprints.
        if staged
            .factions
            .values()
            .flat_map(|f| f.buildings.values())
            .any(|b| {
                staged
                    .navigation
                    .path(staged.navigation.destinations[&b.node_id], objective)
                    .is_none()
            })
        {
            return Err("building_blocks_holding_access".into());
        }
        // Authored scenarios and resumed campaigns obey the same authoritative
        // profile, building, navigation and population validity boundary.
        let config = staged.rules.foothold.clone();
        if staged
            .navigation
            .path(objective, config.cache_position)
            .is_none()
        {
            return Err("Wreck salvage is unreachable.".into());
        }
        // Authored notables are not placed on the board. The island starts with
        // Michael alone and every other person arrives through production, so a
        // notable only joins the roster her faction will raise. The roster
        // itself is resolved by `scenario_rules`; this refuses the packs that
        // roster is not allowed to have come from.
        let mut rostered: BTreeSet<&str> = BTreeSet::new();
        for placement in &definition.placements {
            if !definition
                .characters
                .iter()
                .any(|character| character.id == placement.character_id)
            {
                return Err("scenario_placement_record_missing".into());
            }
            if !rostered.insert(placement.character_id.as_str()) {
                return Err("duplicate_scenario_placement".into());
            }
            if placement.character_id == definition.start.captain_id {
                return Err("duplicate_scenario_placement".into());
            }
            if !staged.factions.contains_key(&placement.faction_id) {
                return Err("scenario_placement_faction_unknown".into());
            }
            if !staged
                .combat_profiles
                .contains_key(&placement.definition_id)
            {
                return Err("scenario_placement_definition_unknown".into());
            }
        }
        staged.salvage_caches.insert(
            "salvage.wreck".into(),
            SalvageCache {
                position: config.cache_position,
                remaining: config.cache_salvage,
                initial_amount: config.cache_salvage,
                label: "Wreck salvage".into(),
                source_building_id: "scenario.shipwreck".into(),
                source_faction: String::new(),
                level: 0,
                created_tick: 0,
            },
        );
        Self::load_json(&staged.save_json()?)
    }

    fn authored_building_obstacles(
        &self,
    ) -> Result<BTreeMap<String, BTreeSet<IslandPoint>>, String> {
        // New scenarios and old-save migration share the same asset contract.
        let footprints = &self.rules.footprints;
        let mut obstacles = BTreeMap::new();
        for faction in self.factions.values() {
            for building in faction.buildings.values() {
                if let Some(footprint) = footprints.get(&building.archetype_id) {
                    let entrance = self
                        .navigation
                        .destinations
                        .get(&building.node_id)
                        .ok_or("missing_building_entrance")?;
                    // Historical saves keep their real inland holding. Do not
                    // add new artwork's collision volume at that old site.
                    if footprint
                        .placement_entrance
                        .is_some_and(|[x, y]| *entrance != (IslandPoint { x, y }))
                    {
                        continue;
                    }
                    let cells: BTreeSet<_> = footprint
                        .blocked_offsets
                        .iter()
                        .map(|[dx, dy]| {
                            Ok(IslandPoint {
                                x: entrance
                                    .x
                                    .checked_add(*dx)
                                    .ok_or("building_coordinate_overflow")?,
                                y: entrance
                                    .y
                                    .checked_add(*dy)
                                    .ok_or("building_coordinate_overflow")?,
                            })
                        })
                        .collect::<Result<_, String>>()?;
                    if cells.contains(entrance) {
                        return Err("occupied_building_footprint".into());
                    }
                    obstacles.insert(building.id.clone(), cells);
                }
            }
        }
        Ok(obstacles)
    }

    pub fn set_policy(
        &mut self,
        faction_id: &str,
        policy: FactionPolicy,
    ) -> Result<(), FactionWorldError> {
        if self.eliminated_factions.contains(faction_id) {
            return Err(FactionWorldError::FactionEliminated(faction_id.into()));
        }
        let faction = self
            .factions
            .get(faction_id)
            .ok_or_else(|| FactionWorldError::UnknownFaction(faction_id.into()))?;
        if policy
            .income_per_tick
            .keys()
            .any(|resource| !policy.storage_caps.contains_key(resource))
            || policy
                .income_per_tick
                .keys()
                .chain(policy.storage_caps.keys())
                .any(|resource| !self.rules.knows(resource))
            || policy.production.len() > 4096
            || policy.development.len() > 4096
            || policy.objectives.len() > 256
        {
            return Err(FactionWorldError::InvalidPolicy(faction_id.into()));
        }
        for (building_id, rule) in &policy.development {
            if !faction.buildings.contains_key(building_id) || !rule.valid() {
                return Err(FactionWorldError::InvalidPolicy(faction_id.into()));
            }
        }
        for (building_id, rule) in &policy.production {
            let building = faction
                .buildings
                .get(building_id)
                .ok_or_else(|| FactionWorldError::UnknownBuilding(building_id.clone()))?;
            if building.archetype_id != rule.producer_archetype_id
                || rule.production_ticks == 0
                || !self.navigation.destinations.contains_key(&building.node_id)
            {
                return Err(FactionWorldError::InvalidPolicy(faction_id.into()));
            }
        }
        for objective in &policy.objectives {
            if !self
                .navigation
                .destinations
                .contains_key(&objective.target_node_id)
                || i64::from(objective.wobble).abs() > i64::from(faction.wobble_limit)
            {
                return Err(FactionWorldError::InvalidPolicy(faction_id.into()));
            }
        }
        self.policies.insert(faction_id.into(), policy);
        Ok(())
    }

    pub fn eliminate_faction(
        &mut self,
        faction_id: &str,
    ) -> Result<FactionWorldEvent, FactionWorldError> {
        if self.eliminated_factions.contains(faction_id) {
            return Err(FactionWorldError::FactionEliminated(faction_id.into()));
        }
        let faction = self
            .factions
            .get_mut(faction_id)
            .ok_or_else(|| FactionWorldError::UnknownFaction(faction_id.into()))?;
        for id in faction.buildings.keys() {
            self.navigation.building_obstacles.remove(id);
        }
        faction.buildings.clear();
        faction.resources.clear();
        faction.population_used = 0;
        self.policies.remove(faction_id);
        let removed: Vec<String> = self
            .actors
            .values()
            .filter(|actor| actor.faction_id == faction_id)
            .map(|actor| actor.instance_id.clone())
            .collect();
        for id in removed {
            let mut actor = self.actors.remove(&id).unwrap();
            let combat = self.unit_combat.remove(&id);
            let position = self.positions.remove(&id);
            // Elimination must not erase an attached woman's persistent identity
            // or leave her retained party slot dangling after a midnight return.
            if actor.person.as_ref().is_some_and(|p| p.loyal_to_michael) {
                if let (Some(combat), Some(position)) = (combat, position) {
                    actor.person.as_mut().unwrap().alive_today = false;
                    self.casualties.insert(
                        id.clone(),
                        IslandCasualty {
                            actor,
                            position,
                            death_tick: self.tick,
                            population_use: combat.population_use,
                        },
                    );
                }
            }
            self.travel_orders.remove(&id);
            self.navigation.destinations.remove(&format!("move.{id}"));
        }
        self.eliminated_factions.insert(faction_id.into());
        // A faction ending is the loudest thing that happens on this island.
        // It belongs in the record every person reads from.
        self.record_diplomacy(
            vec![faction_id.to_string()],
            format!("{} are finished. Their holdings are gone.", {
                let label = faction_label(faction_id);
                label.to_string()
            }),
        );
        if self
            .approach_target
            .as_ref()
            .is_some_and(|id| !self.can_approach_person(id))
        {
            self.cancel_approach();
        }
        Ok(FactionWorldEvent::FactionEliminated {
            faction_id: faction_id.into(),
        })
    }

    fn advance_faction_decisions(&mut self) -> Vec<FactionWorldEvent> {
        let mut events = Vec::new();
        self.advance_diplomacy();
        self.advance_holding_repairs();
        self.advance_holding_expansion();
        for (id, policy) in self.policies.clone() {
            if self.eliminated_factions.contains(&id) {
                continue;
            }
            let Some(faction) = self.factions.get_mut(&id) else {
                continue;
            };
            // Income requires an actual operational holding, not a surviving ID.
            if !faction
                .buildings
                .values()
                .any(|building| building.operational)
            {
                continue;
            }
            for (resource, income) in &policy.income_per_tick {
                // The helper knows the caps; income is one gain among several.
                let _ = self.gain_resource(&id, resource, *income);
            }
            for (building_id, rule) in &policy.production {
                // One active cycle per producer. Queue capacity is not free parallel throughput.
                let idle = self.factions[&id]
                    .buildings
                    .get(building_id)
                    .is_some_and(|building| {
                        building.operational
                            && building.production_queue.is_empty()
                            && building.development.is_none()
                    });
                if idle {
                    if let Ok(event) = self.enqueue_production(&id, building_id, rule.clone()) {
                        events.push(event);
                    }
                }
            }
            // Build out a staffed, undamaged holding. Production has priority
            // while replacing losses; development occupies the same producer.
            let faction = &self.factions[&id];
            if faction.population_used >= faction.population_capacity {
                for (building_id, rule) in &policy.development {
                    let Some(building) = self.factions[&id].buildings.get(building_id) else {
                        continue;
                    };
                    if !building.operational
                        || building.health != building.max_health
                        || building.level >= rule.max_level
                        || building.repair.is_some()
                        || building.development.is_some()
                        || !building.production_queue.is_empty()
                    {
                        continue;
                    }
                    let costs: BTreeMap<String, u32> = rule
                        .costs
                        .iter()
                        .map(|(resource, cost)| {
                            (resource.clone(), cost.saturating_mul(building.level))
                        })
                        .collect();
                    if self.spend_resources(&id, &costs).is_err() {
                        continue;
                    }
                    self.factions
                        .get_mut(&id)
                        .unwrap()
                        .buildings
                        .get_mut(building_id)
                        .unwrap()
                        .development = Some(BuildingDevelopment {
                        rule: rule.clone(),
                        remaining_ticks: rule.ticks,
                        reserved_costs: costs,
                    });
                }
            }
            // Reserve a small real contingent per threatened holding. Existing
            // defenders retain their jobs; moving attackers can be recalled only
            // every eight ticks. New idle units can answer immediately.
            let mut defense: BTreeMap<String, DispatchCandidate> = BTreeMap::new();
            for building in self.factions[&id]
                .buildings
                .values()
                .filter(|b| b.operational && b.health > 0)
            {
                let Some(home) = self.navigation.destinations.get(&building.node_id).copied()
                else {
                    continue;
                };
                let threats = self
                    .actors
                    .iter()
                    .filter(|(other_id, other)| {
                        self.living_actor(other_id)
                            && self
                                .actor_combat_profile(other_id)
                                .is_some_and(|p| p.damage > 0)
                            && self
                                .hostilities
                                .contains(&(id.clone(), other.faction_id.clone()))
                            && self.positions.get(*other_id).is_some_and(|p| {
                                p.x.abs_diff(home.x).saturating_add(p.y.abs_diff(home.y)) <= 4
                            })
                    })
                    .count();
                if threats == 0 {
                    continue;
                }
                let action = format!("defend.{}", building.id);
                let quota = threats.min(3);
                let mut candidates: Vec<_> = self
                    .actors
                    .iter()
                    .filter_map(|(actor_id, actor)| {
                        if actor.faction_id != id
                            || actor.current_assignment_id.as_deref().is_some_and(|a| {
                                a.starts_with("construct.") || a.starts_with("repair.")
                            })
                            || !self.living_actor(actor_id)
                            || !self
                                .actor_combat_profile(actor_id)
                                .is_some_and(|p| p.damage > 0)
                            || defense.contains_key(actor_id)
                        {
                            return None;
                        }
                        let incumbent =
                            actor.current_assignment_id.as_deref() == Some(action.as_str());
                        if self.travel_orders.contains_key(actor_id)
                            && !incumbent
                            && self.tick % 8 != 0
                        {
                            return None;
                        }
                        let path = self.navigation.path(self.positions[actor_id], home)?;
                        Some((!incumbent, path.len(), actor_id.clone()))
                    })
                    .collect();
                candidates.sort();
                for (_, distance, actor_id) in candidates.into_iter().take(quota) {
                    defense.insert(
                        actor_id,
                        DispatchCandidate {
                            action_id: action.clone(),
                            assignment: "defend".into(),
                            target_node_id: building.node_id.clone(),
                            score: DispatchScore {
                                goal_progress: 200,
                                target_threat: (threats.min(32) as i32) * 20,
                                home_defense_deficit: 20,
                                supply_cost: -(distance.min(100) as i32),
                                ..Default::default()
                            },
                            wobble: 0,
                        },
                    );
                }
            }
            let idle_actors: Vec<String> =
                self.actors
                    .values()
                    .filter(|actor| {
                        actor.faction_id == id
                            && !actor.current_assignment_id.as_deref().is_some_and(|a| {
                                a.starts_with("construct.") || a.starts_with("repair.")
                            })
                            && self
                                .unit_combat
                                .get(&actor.instance_id)
                                .is_none_or(|state| state.health > 0)
                            && (!self.travel_orders.contains_key(&actor.instance_id)
                                || defense.contains_key(&actor.instance_id)
                                || self.tick % 8 == 0
                                    && actor.current_assignment_id.as_deref().is_some_and(
                                        |assignment| assignment.starts_with("defend."),
                                    ))
                    })
                    .map(|actor| actor.instance_id.clone())
                    .collect();
            for actor_id in idle_actors {
                let Some(start) = self.positions.get(&actor_id).copied() else {
                    continue;
                };
                let mut reachable: Vec<DispatchCandidate> = policy
                    .objectives
                    .iter()
                    .filter(|objective| {
                        self.navigation
                            .destinations
                            .get(&objective.target_node_id)
                            .is_some_and(|goal| self.navigation.path(start, *goal).is_some())
                    })
                    .cloned()
                    .collect();
                // Reach the authored rally before advancing on holdings. Once
                // campaigning, retain that intent across destroyed objectives.
                let campaigning = policy.objectives.iter().any(|objective| {
                    self.navigation.destinations.get(&objective.target_node_id) == Some(&start)
                }) || self.actors[&actor_id]
                    .current_assignment_id
                    .as_deref()
                    .is_some_and(|assignment| assignment.starts_with("siege."));
                if campaigning {
                    let mut sieges = Vec::new();
                    for enemy in self
                        .factions
                        .values()
                        .filter(|enemy| self.hostilities.contains(&(id.clone(), enemy.id.clone())))
                    {
                        for building in enemy
                            .buildings
                            .values()
                            .filter(|building| building.health > 0)
                        {
                            let Some(goal) = self.navigation.destinations.get(&building.node_id)
                            else {
                                continue;
                            };
                            let Some(path) = self.navigation.path(start, *goal) else {
                                continue;
                            };
                            let defenders = self
                                .actors
                                .values()
                                .filter(|actor| {
                                    actor.faction_id == enemy.id
                                        && self.positions.get(&actor.instance_id).is_some_and(
                                            |position| {
                                                position.x.abs_diff(goal.x)
                                                    + position.y.abs_diff(goal.y)
                                                    <= 4
                                            },
                                        )
                                })
                                .count();
                            sieges.push(DispatchCandidate {
                                action_id: format!("siege.{}", building.id),
                                assignment: "attack_holding".into(),
                                target_node_id: building.node_id.clone(),
                                score: DispatchScore {
                                    goal_progress: 100,
                                    supply_cost: -(path.len().min(10_000) as i32),
                                    travel_risk: -(defenders.min(1_000) as i32 * 4),
                                    expected_loot: (building
                                        .max_health
                                        .saturating_sub(building.health)
                                        / 4)
                                        as i32,
                                    ..Default::default()
                                },
                                wobble: 0,
                            });
                        }
                    }
                    // A reachable enemy holding replaces a completed rally;
                    // absence of one leaves authored fallback objectives intact.
                    if !sieges.is_empty() {
                        reachable = sieges;
                    }
                }
                if let Some(defending) = defense.get(&actor_id) {
                    reachable.push(defending.clone());
                }
                if let Some(best) = reachable.iter().max_by(|a, b| {
                    a.total()
                        .cmp(&b.total())
                        .then_with(|| b.action_id.cmp(&a.action_id))
                }) {
                    if self.travel_orders.get(&actor_id) == Some(&best.target_node_id)
                        || self.navigation.destinations.get(&best.target_node_id) == Some(&start)
                            && self.actors[&actor_id].current_assignment_id.as_ref()
                                == Some(&best.action_id)
                    {
                        continue;
                    }
                }
                if let Ok(event) = self.dispatch_actor(&actor_id, &reachable) {
                    events.push(event);
                }
            }
        }
        events
    }

    pub fn save_json(&self) -> Result<String, String> {
        #[derive(Serialize)]
        struct Save<'a> {
            version: u32,
            world: &'a FactionWorld,
        }
        serde_json::to_string(&Save {
            version: SAVE_VERSION,
            world: self,
        })
        .map_err(|error| error.to_string())
    }

    /// Deserialize into a new value; callers replace live state only after all
    /// validation succeeds. File-size and collection caps bound untrusted saves.
    /// A version-2 save carries its scenario's rules; only a version-1 save
    /// needs them supplied, and it takes the scenario it is being resumed into.
    pub fn load_json(text: &str) -> Result<Self, String> {
        Self::load_saved(text, None)
    }

    /// Resume a save into a scenario: the migration path for version-1 saves,
    /// which predate the world carrying its own rules.
    pub fn load_json_for_scenario(text: &str, rules: &ScenarioRules) -> Result<Self, String> {
        Self::load_saved(text, Some(rules))
    }

    fn load_saved(text: &str, scenario_rules: Option<&ScenarioRules>) -> Result<Self, String> {
        if text.len() > 8 * 1024 * 1024 {
            return Err("save_too_large".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Save {
            version: u32,
            world: FactionWorld,
        }
        let mut json: serde_json::Value =
            serde_json::from_str(text).map_err(|_| "invalid_save_json")?;
        let version = json.get("version").and_then(serde_json::Value::as_u64);
        if version == Some(1) {
            let Some(rules) = scenario_rules else {
                return Err("save_needs_scenario_rules".into());
            };
            let Some(saved_world) = json.get_mut("world").and_then(|v| v.as_object_mut()) else {
                return Err("invalid_save_json".into());
            };
            if saved_world.contains_key("rules") {
                return Err("invalid_save_json".into());
            }
            saved_world.insert(
                "rules".into(),
                serde_json::to_value(rules).map_err(|_| "invalid_save_json")?,
            );
            // A version-1 save predates quests entirely. Seed every quest the
            // scenario it is resuming into declares onto its own initial
            // stage -- the same invariant `from_scenario` establishes for a
            // freshly created world, restated here for one migrating in.
            if !saved_world.contains_key("quest_stages") {
                let quest_stages: BTreeMap<String, String> = rules
                    .quests
                    .iter()
                    .map(|(id, quest)| (id.clone(), quest.initial_stage.clone()))
                    .collect();
                saved_world.insert(
                    "quest_stages".into(),
                    serde_json::to_value(quest_stages).map_err(|_| "invalid_save_json")?,
                );
            }
            json["version"] = serde_json::json!(SAVE_VERSION);
        }
        let migration_rules = json
            .pointer("/world/rules")
            .and_then(|value| serde_json::from_value::<ScenarioRules>(value.clone()).ok())
            .ok_or("invalid_save_json")?;
        if let Some(saved_world) = json.get_mut("world").and_then(|v| v.as_object_mut()) {
            let legacy = saved_world.remove("foothold_cache");
            if !saved_world.contains_key("salvage_caches") {
                if let Some(old) = legacy.filter(|v| !v.is_null()) {
                    #[derive(Deserialize)]
                    struct LegacyCache {
                        position: IslandPoint,
                        remaining: u32,
                    }
                    let old: LegacyCache =
                        serde_json::from_value(old).map_err(|_| "invalid_saved_foothold_cache")?;
                    let cache = SalvageCache {
                        position: old.position,
                        remaining: old.remaining,
                        initial_amount: old.remaining.max(migration_rules.foothold.cache_salvage),
                        label: "Wreck salvage".into(),
                        source_building_id: "scenario.shipwreck".into(),
                        source_faction: String::new(),
                        level: 0,
                        created_tick: 0,
                    };
                    saved_world.insert(
                        "salvage_caches".into(),
                        serde_json::json!({"salvage.wreck": cache}),
                    );
                }
            }
        }
        let needs_obstacles = json
            .pointer("/world/navigation/building_obstacles")
            .is_none();
        let save: Save = serde_json::from_value(json).map_err(|_| "invalid_save_json")?;
        if save.version != SAVE_VERSION {
            return Err("unsupported_save_version".into());
        }
        let mut world = save.world;
        if !world.rules.valid() {
            return Err("invalid_saved_scenario_rules".into());
        }
        // Every key a save carries must be one the scenario's catalogue
        // declares: a save may not smuggle in a resource the pack never had.
        if world.rules.resources.len() > MAXIMUM_RESOURCES
            || world.rules.resources.iter().any(|(id, resource)| {
                !valid_resource_id(id)
                    || resource.display_name.trim().is_empty()
                    || resource.display_name.len() > 128
            })
            || world.factions.values().any(|faction| {
                faction.resources.keys().any(|id| !world.rules.knows(id))
                    || faction.buildings.values().any(|building| {
                        building
                            .construction
                            .iter()
                            .chain(building.repair.iter())
                            .flat_map(|job| job.reserved_costs.keys())
                            .chain(
                                building
                                    .development
                                    .iter()
                                    .flat_map(|work| work.reserved_costs.keys()),
                            )
                            .chain(
                                building
                                    .production_queue
                                    .iter()
                                    .flat_map(|order| order.reserved_costs.keys()),
                            )
                            .any(|id| !world.rules.knows(id))
                    })
            })
            || world.policies.values().any(|policy| {
                policy
                    .income_per_tick
                    .keys()
                    .chain(policy.storage_caps.keys())
                    .any(|id| !world.rules.knows(id))
            })
        {
            return Err("invalid_saved_resources".into());
        }
        let mut standing_pairs = BTreeSet::new();
        if world.standings.len() > 64
            || world.standings.iter().any(|s| {
                s.a >= s.b
                    || !world.factions.contains_key(&s.a)
                    || !world.factions.contains_key(&s.b)
                    || !(-100..=100).contains(&s.value)
                    || !(-100..=100).contains(&s.baseline)
                    || !standing_pairs.insert((s.a.clone(), s.b.clone()))
            })
        {
            return Err("invalid_saved_standings".into());
        }
        let mut treaty_pairs = BTreeSet::new();
        if world.survival_truces.len() > 6
            || world.diplomacy_notices.len() > 16
            || world.survival_truces.iter().any(|t| {
                t.a >= t.b
                    || t.a == "faction.michael"
                    || t.b == "faction.michael"
                    || t.threat == "faction.michael"
                    || t.threat == t.a
                    || t.threat == t.b
                    || !world.factions.contains_key(&t.a)
                    || !world.factions.contains_key(&t.b)
                    || !world.factions.contains_key(&t.threat)
                    || !treaty_pairs.insert((t.a.clone(), t.b.clone()))
                    || world.hostilities.contains(&(t.a.clone(), t.b.clone()))
                    || world.hostilities.contains(&(t.b.clone(), t.a.clone()))
                    || t.expires_tick > world.tick.saturating_add(world.rules.survival.truce_ticks)
            })
            || world.diplomacy_notices.iter().any(|n| {
                n.tick > world.tick
                    || n.text.len() > 1024
                    || n.factions.len() > 5
                    || n.factions.iter().any(|f| !world.factions.contains_key(f))
            })
        {
            return Err("invalid_saved_diplomacy".into());
        }
        if world.salvage_caches.len()
            + world
                .factions
                .values()
                .map(|f| f.buildings.len())
                .sum::<usize>()
            > 4096
            || world.salvage_caches.iter().any(|(id, c)| {
                id.is_empty()
                    || id.len() > 2048
                    || c.initial_amount > 100000
                    || c.remaining > c.initial_amount
                    || c.label.is_empty()
                    || c.label.len() > 256
                    || c.source_building_id.is_empty()
                    || c.source_building_id.len() > 1024
                    || c.level > 5
                    || c.created_tick > world.tick
                    || (!c.source_faction.is_empty()
                        && !world.factions.contains_key(&c.source_faction))
                    || !world.navigation.walkable.contains(&c.position)
            })
        {
            return Err("invalid_saved_salvage_caches".into());
        }
        if !(1..=1000000).contains(&world.clock.ticks_per_day) {
            return Err("invalid_saved_clock".into());
        }
        let mut heat_ids = BTreeSet::new();
        let mut previous_heat_tick = 0u64;
        if world.heat.len() > HEAT_LEDGER_CAP
            || world.heat.iter().any(|event| {
                let out_of_order = event.tick < previous_heat_tick;
                previous_heat_tick = event.tick;
                out_of_order
                    || event.id.is_empty()
                    || event.id.len() > 256
                    || !(1..=100).contains(&event.severity)
                    || event.tick > world.tick
                    || !heat_ids.insert(event.id.clone())
                    || !world
                        .rules
                        .campaign_clock
                        .signal_channels
                        .contains(&event.signal)
            })
        {
            return Err("invalid_saved_heat_ledger".into());
        }
        if world.flags.len() > MAX_FLAGS
            || world
                .flags
                .iter()
                .any(|flag| flag.is_empty() || flag.len() > 256)
            || world.fired_triggers.len() > MAX_TRIGGER_HISTORY
            || world.fired_triggers.iter().any(|(id, fired_tick)| {
                id.is_empty()
                    || id.len() > 256
                    // The shape casualty.death_tick already uses: recorded play
                    // state may not claim to have happened in the future.
                    || *fired_tick > world.tick
                    || !world.rules.triggers.iter().any(|trigger| &trigger.id == id)
            })
        {
            return Err("invalid_saved_trigger_history".into());
        }
        // A lead the player has not answered yet is a decision the campaign
        // still owes them, so it must name a lead this pack declares and cannot
        // claim to have been raised in the future.
        if world.open_leads.len() > MAX_OPEN_LEADS
            || world.open_leads.iter().any(|(id, opened_tick)| {
                *opened_tick > world.tick
                    || world.resolved_leads.contains_key(id)
                    || !world.rules.leads.iter().any(|lead| &lead.id == id)
            })
        {
            return Err("invalid_saved_open_leads".into());
        }
        // An answered lead records a reading she actually offered.
        if world.resolved_leads.iter().any(|(id, interpretation_id)| {
            !world.rules.leads.iter().any(|lead| {
                &lead.id == id
                    && lead
                        .interpretations
                        .iter()
                        .any(|interpretation| &interpretation.id == interpretation_id)
            })
        }) {
            return Err("invalid_saved_resolved_leads".into());
        }
        // Every declared quest is active on exactly one of its own stages:
        // no quest missing an entry, no entry for an undeclared quest, and no
        // stage that quest does not define.
        if world.quest_stages.len() != world.rules.quests.len()
            || world.quest_stages.iter().any(|(quest_id, stage_id)| {
                world
                    .rules
                    .quests
                    .get(quest_id)
                    .is_none_or(|quest| !quest.stages.contains_key(stage_id))
            })
        {
            return Err("invalid_saved_quest_stages".into());
        }
        if world.confrontation.is_some_and(|record| {
            record.tick > world.tick
                || record.day > world.day()
                || !world.rules.campaign_clock.priority.contains(&record.cause)
        }) {
            return Err("invalid_saved_confrontation".into());
        }
        if world.navigation.walkable.len() > 16384
            || world.actors.len() > 4096
            || world.factions.len() > 64
            || world.tick == u64::MAX
            || world.next_actor_serial == u64::MAX
            || world.next_order_serial == u64::MAX
        {
            return Err("save_limits_exceeded".into());
        }
        if needs_obstacles {
            world.navigation.building_obstacles = world.authored_building_obstacles()?;
        }
        let mut working_builders = BTreeSet::new();
        for (id, faction) in &world.factions {
            if id != &faction.id
                || faction.population_used > faction.population_capacity
                || world
                    .minimum_population(id)
                    .is_none_or(|minimum| faction.population_used < minimum)
                || faction.wobble_limit < 0
                || faction.buildings.len() > 4096
            {
                return Err("invalid_saved_faction".into());
            }
            for (building_id, building) in &faction.buildings {
                for job in building.construction.iter().chain(building.repair.iter()) {
                    if !working_builders.insert(job.builder_id.clone()) {
                        return Err("builder_has_multiple_jobs".into());
                    }
                }
                if let Some(job) = &building.repair {
                    let valid = world
                        .rules
                        .repairs
                        .get(&building.archetype_id)
                        .is_some_and(|r| {
                            (1..=100000).contains(&r.ticks)
                                && (1..=100000).contains(&r.health_gain)
                                && !r.costs.is_empty()
                                && r.costs.values().all(|v| (1..=100000).contains(v))
                                && job.remaining_ticks > 0
                                && job.remaining_ticks <= r.ticks
                                && job.reserved_costs == r.costs
                        });
                    if !valid
                        || !building.operational
                        || building.construction.is_some()
                        || building.development.is_some()
                        || (!world.actors.contains_key(&job.builder_id)
                            && !world.casualties.contains_key(&job.builder_id))
                    {
                        return Err("invalid_saved_repair".into());
                    }
                }
                if let Some(job) = &building.construction {
                    let workshop = faction.id == "faction.michael"
                        && building.archetype_id == "site_archetype.michael.field_workshop"
                        && job.builder_id == "character.protagonist.captain"
                        && {
                            let c = &world.rules.foothold;
                            job.remaining_ticks <= c.construction_ticks
                                && job.reserved_costs
                                    == [(world.salvage_resource().to_owned(), c.build_salvage)]
                                        .into()
                        };
                    let expansion = {
                        let r = &world.rules.expansion;
                        faction.id == r.faction_id
                            && building.id == r.building_id
                            && building.archetype_id == r.archetype_id
                            && world.navigation.destinations.get(&building.node_id)
                                == Some(&IslandPoint {
                                    x: r.entrance[0],
                                    y: r.entrance[1],
                                })
                            && job.remaining_ticks <= r.construction_ticks
                            && job.reserved_costs == r.costs
                    };
                    if building.operational
                        || !(workshop || expansion)
                        || building.development.is_some()
                        || !building.production_queue.is_empty()
                        || !(1..=100000).contains(&job.remaining_ticks)
                        || job.reserved_costs.is_empty()
                        || job
                            .reserved_costs
                            .values()
                            .any(|c| !(1..=100000).contains(c))
                        || !world.actors.contains_key(&job.builder_id)
                            && !world.casualties.contains_key(&job.builder_id)
                    {
                        return Err("invalid_saved_construction".into());
                    }
                }
                if building_id != &building.id
                    || &building.faction_id != id
                    || building.production_queue.len() > building.queue_capacity
                    || building.queue_capacity > 4096
                    || building.health == 0
                    || building.health > building.max_health
                    || building.max_health > 100000
                    || building.level == 0
                    || building.level > 5
                {
                    return Err("invalid_saved_building".into());
                }
                if let Some(order) = &building.development {
                    if !building.production_queue.is_empty()
                        || order.remaining_ticks == 0
                        || order.remaining_ticks > order.rule.ticks
                        || building.level >= order.rule.max_level
                        || !order.rule.valid()
                        || order.reserved_costs
                            != order
                                .rule
                                .costs
                                .iter()
                                .map(|(resource, cost)| {
                                    (resource.clone(), cost.saturating_mul(building.level))
                                })
                                .collect()
                    {
                        return Err("invalid_saved_development".into());
                    }
                }
                for order in &building.production_queue {
                    if order.remaining_ticks == 0
                        || order.remaining_ticks > order.rule.production_ticks
                        || order.rule.producer_archetype_id != building.archetype_id
                    {
                        return Err("invalid_saved_production".into());
                    }
                }
            }
        }
        let building_ids: BTreeSet<_> = world
            .factions
            .values()
            .flat_map(|f| f.buildings.keys())
            .collect();
        if world.navigation.building_obstacles.len() > 4096
            || world
                .navigation
                .building_obstacles
                .iter()
                .any(|(id, cells)| !building_ids.contains(id) || cells.len() > 256)
        {
            return Err("invalid_saved_building_obstacles".into());
        }
        for building in world.factions.values().flat_map(|f| f.buildings.values()) {
            if let Some(entrance) = world.navigation.destinations.get(&building.node_id) {
                if !world.navigation.traversable(*entrance) {
                    return Err("blocked_saved_building_entrance".into());
                }
            }
        }
        // The berths the workshop has, not the base capacity. This was the
        // third place that knew the cap, and the last to be told the workshop
        // can grow: a save with a fourth dog in it was refused by name, so
        // building the workshop out and using the berth it bought corrupted
        // the campaign at the next save.
        if world.mechanical_dog_count() > world.machine_berths() {
            return Err("invalid_saved_machine_capacity".into());
        }
        for (id, actor) in &world.actors {
            if actor.actor_kind == "machine" && (actor.person.is_some() || actor.undead) {
                return Err("invalid_saved_machine_identity".into());
            }
            if actor
                .person
                .as_ref()
                .is_some_and(|person| !person.valid_for_actor(id, true))
            {
                return Err("invalid_saved_person".into());
            }
            if id != &actor.instance_id
                || !world.factions.contains_key(&actor.faction_id)
                || actor.madness > world.rules.madness.conversion_threshold
            {
                return Err("invalid_saved_actor".into());
            }
        }
        for (id, point) in &world.positions {
            if !world.actors.contains_key(id) || !world.navigation.traversable(*point) {
                return Err("invalid_saved_position".into());
            }
        }
        for (id, destination) in &world.travel_orders {
            if !world.positions.contains_key(id)
                || !world.navigation.destinations.contains_key(destination)
            {
                return Err("invalid_saved_travel".into());
            }
        }
        for id in &world.eliminated_factions {
            let Some(faction) = world.factions.get(id) else {
                return Err("invalid_eliminated_faction".into());
            };
            if !faction.buildings.is_empty()
                || world.policies.contains_key(id)
                || world.actors.values().any(|actor| &actor.faction_id == id)
            {
                return Err("eliminated_faction_has_live_state".into());
            }
        }
        for profile in world.combat_profiles.values() {
            if profile.health == 0
                || profile.damage == 0
                || profile.range > 32
                || profile.cooldown_ticks == 0
            {
                return Err("invalid_saved_combat_profile".into());
            }
        }
        for (id, state) in &world.unit_combat {
            if !world.actors.contains_key(id) {
                return Err("dangling_combat_state".into());
            }
            // Health is bounded by the profile the holder's own items resolve to.
            let profile = world
                .actor_combat_profile(id)
                .ok_or("missing_combat_profile")?;
            if state.health == 0 || state.health > profile.health {
                return Err("invalid_saved_health".into());
            }
        }
        // Every carried item resolves against the loaded world: a living or
        // fallen actor of this world holds it, the catalogue defines it, and no
        // stack exceeds what its record allows.
        if world.inventories.len() > MAX_INVENTORY_ENTRIES {
            return Err("invalid_saved_inventory".into());
        }
        for (id, held) in &world.inventories {
            if held.is_empty()
                || held.len() > MAX_ACTOR_INVENTORY
                || !(world.actors.contains_key(id) || world.casualties.contains_key(id))
            {
                return Err("invalid_saved_inventory".into());
            }
            for item_id in held {
                let Some(item) = world.rules.items.get(item_id) else {
                    return Err("invalid_saved_inventory".into());
                };
                if held.iter().filter(|other| *other == item_id).count() as u32 > item.stack.limit()
                {
                    return Err("invalid_saved_inventory".into());
                }
            }
        }
        for (id, casualty) in &world.casualties {
            if casualty
                .actor
                .person
                .as_ref()
                .is_some_and(|person| !person.valid_for_actor(id, false))
            {
                return Err("invalid_saved_casualty_person".into());
            }
            if id != &casualty.actor.instance_id
                || world.actors.contains_key(id)
                || casualty.death_tick > world.tick
                || !world.factions.contains_key(&casualty.actor.faction_id)
                || casualty.population_use > 4096
            {
                return Err("invalid_saved_casualty".into());
            }
        }
        if world
            .approach_target
            .as_ref()
            .is_some_and(|id| !world.can_approach_person(id))
            || (world.approach_target.is_some() && world.player_attack_target.is_some())
        {
            return Err("invalid_saved_approach".into());
        }
        let mut seen = BTreeSet::new();
        for id in world.party.iter().filter(|id| !id.is_empty()) {
            if !seen.insert(id) {
                return Err("duplicate_saved_companion".into());
            }
            let actor = world
                .actors
                .get(id)
                .or_else(|| world.casualties.get(id).map(|c| &c.actor))
                .ok_or("missing_saved_companion")?;
            let person = actor.person.as_ref().ok_or("invalid_saved_companion")?;
            if (actor.faction_id != "faction.michael"
                && !(actor.undead && actor.faction_id == "faction.cthulhu.prototype"))
                || person.sex != PersonSex::Female
                || !person.age.is_some_and(|age| age >= 18)
                || !person.loyal_to_michael
                || (world.actors.contains_key(id) && world.living_person(id).is_none())
            {
                return Err("invalid_saved_companion".into());
            }
        }
        let mut world = world;
        for (id, policy) in world.policies.clone() {
            world
                .set_policy(&id, policy)
                .map_err(|_| "invalid_saved_policy")?;
        }
        Ok(world)
    }

    fn minimum_population(&self, faction_id: &str) -> Option<u32> {
        let faction = self.factions.get(faction_id)?;
        let reservations = faction
            .buildings
            .values()
            .flat_map(|b| &b.production_queue)
            .map(|q| q.rule.population_use);
        let live = self
            .unit_combat
            .iter()
            .filter(|(id, _)| {
                self.actors
                    .get(*id)
                    .is_some_and(|a| a.faction_id == faction_id)
            })
            .map(|(_, state)| state.population_use);
        reservations
            .chain(live)
            .try_fold(0u32, |sum, population| sum.checked_add(population))
    }

    fn living_actor(&self, id: &str) -> bool {
        self.actors.contains_key(id)
            && self.positions.contains_key(id)
            && self
                .unit_combat
                .get(id)
                .is_some_and(|state| state.health > 0)
    }

    fn cancel_actor_travel(&mut self, id: &str) {
        self.travel_orders.remove(id);
        self.navigation.destinations.remove(&format!("move.{id}"));
        if let Some(actor) = self.actors.get_mut(id) {
            actor.current_assignment_id = None;
        }
    }

    fn living_person(&self, id: &str) -> Option<&NamedPerson> {
        self.unit_combat.get(id).filter(|state| state.health > 0)?;
        self.positions.get(id)?;
        self.actors
            .get(id)?
            .person
            .as_ref()
            .filter(|person| person.alive_today)
    }

    fn can_approach_person(&self, id: &str) -> bool {
        let captain = "character.protagonist.captain";
        if !self.living_actor(captain) || id == captain {
            return false;
        }
        let Some(person) = self.living_person(id) else {
            return false;
        };
        if !person.age.is_some_and(|age| age >= 18) || person.display_name.trim().is_empty() {
            return false;
        }
        true
    }

    fn accessible_person(&self, id: &str) -> bool {
        if !self.can_approach_person(id) {
            return false;
        }
        let a = self.positions["character.protagonist.captain"];
        let b = self.positions[id];
        u64::from(a.x.abs_diff(b.x)) + u64::from(a.y.abs_diff(b.y)) <= 2
            && self.navigation.clear_line(a, b)
    }

    pub fn talk_island_person(&mut self, id: &str) -> String {
        if !self.accessible_person(id) {
            return String::new();
        }
        if self.approach_target.as_deref() == Some(id) {
            self.cancel_approach();
        }
        let person = self.actors.get_mut(id).unwrap().person.as_mut().unwrap();
        person.discussed = true;
        self.island_person_dialogue(id)
    }

    /// Authored speech follows current allegiance, never the old recruitment
    /// offer. Older saves without companion prose get a neutral acknowledgement.
    pub fn island_person_dialogue(&self, id: &str) -> String {
        let Some(person) = self.living_person(id) else {
            return String::new();
        };
        if !person.discussed {
            return String::new();
        }
        if self.actors[id].faction_id == "faction.michael" && person.loyal_to_michael {
            if person.companion_response.is_empty() {
                "I'm with you, Michael.".into()
            } else {
                person.companion_response.clone()
            }
        } else {
            if person.recruitment_offer.is_empty() {
                "What news do you want, Captain?".into()
            } else {
                person.recruitment_offer.clone()
            }
        }
    }

    pub fn can_talk_island_person(&self, id: &str) -> bool {
        self.accessible_person(id)
    }

    pub fn recruit_island_person(&mut self, id: &str) -> bool {
        if !self.accessible_person(id) {
            return false;
        }
        let actor = &self.actors[id];
        let person = actor.person.as_ref().unwrap();
        if actor.faction_id == "faction.michael"
            || !person.discussed
            || person.sex != PersonSex::Female
            || !person.age.is_some_and(|age| age >= 18)
            || person.recruitment_offer.trim().is_empty()
        {
            return false;
        }
        if !self.transfer_living_actor(id, "faction.michael") {
            return false;
        }
        let actor = self.actors.get_mut(id).unwrap();
        actor.person.as_mut().unwrap().loyal_to_michael = true;
        actor.madness = 0;
        true
    }

    /// One allegiance transfer for living recruitment, whether voluntary or
    /// supernatural. Reserved production population stays with the old faction.
    /// Midnight resurrection has already debited its dead source and is separate.
    fn transfer_living_actor(&mut self, id: &str, destination_id: &str) -> bool {
        if !self.living_actor(id) || self.eliminated_factions.contains(destination_id) {
            return false;
        }
        let source = self.actors[id].faction_id.clone();
        if source == destination_id {
            return false;
        }
        if self
            .minimum_population(&source)
            .is_none_or(|minimum| self.factions[&source].population_used < minimum)
        {
            return false;
        }
        let population = self.unit_combat[id].population_use;
        let Some(source_used) = self
            .factions
            .get(&source)
            .and_then(|f| f.population_used.checked_sub(population))
        else {
            return false;
        };
        let Some(destination_used) = self
            .factions
            .get(destination_id)
            .and_then(|f| f.population_used.checked_add(population))
        else {
            return false;
        };
        if self.approach_target.as_deref() == Some(id) {
            self.cancel_approach();
        }
        self.factions.get_mut(&source).unwrap().population_used = source_used;
        let destination = self.factions.get_mut(destination_id).unwrap();
        destination.population_used = destination_used;
        // Provisional immigrant accommodation, not additional army production.
        destination.population_capacity = destination.population_capacity.max(destination_used);
        let actor = self.actors.get_mut(id).unwrap();
        actor.faction_id = destination_id.into();
        actor.current_assignment_id = None;
        self.cancel_actor_travel(id);
        if self.player_attack_target.as_deref() == Some(id) {
            self.player_attack_target = None;
        }
        true
    }

    pub fn island_madness_stage(&self, id: &str) -> &'static str {
        let Some(actor) = self.actors.get(id) else {
            return "";
        };
        if actor.undead {
            return "";
        }
        let rule = &self.rules.madness;
        if !actor.undead
            && actor.faction_id == rule.faction_id
            && actor.madness >= rule.conversion_threshold
        {
            "converted"
        } else if actor.madness >= rule.warning_threshold {
            "whisper_haunted"
        } else {
            ""
        }
    }

    fn advance_madness(&mut self) -> Vec<FactionWorldEvent> {
        let rule = self.rules.madness.clone();
        let shrines: Vec<_> = self
            .factions
            .get(&rule.faction_id)
            .filter(|_| !self.eliminated_factions.contains(&rule.faction_id))
            .into_iter()
            .flat_map(|f| f.buildings.values())
            // The ritual continues through upgrades, but an unfinished site
            // cannot whisper. Destroying the last shrine removes all exposure.
            .filter(|b| {
                b.health > 0
                    && b.construction.is_none()
                    && (b.operational || b.development.is_some())
                    && b.archetype_id == rule.ritual_archetype_id
            })
            .filter_map(|b| self.navigation.destinations.get(&b.node_id).copied())
            .collect();
        let mut events = Vec::new();
        for id in self.actors.keys().cloned().collect::<Vec<_>>() {
            let actor = &self.actors[&id];
            if actor.faction_id == rule.faction_id {
                continue;
            }
            let susceptible = id != "character.protagonist.captain"
                && !actor.undead
                && self.living_actor(&id)
                && rule.susceptible_definitions.contains(&actor.definition_id)
                && actor.person.as_ref().is_some_and(|p| {
                    p.alive_today
                        && p.age.is_some_and(|age| age >= 18)
                        && !(p.sex == PersonSex::Female && p.loyal_to_michael)
                });
            if !susceptible {
                self.actors.get_mut(&id).unwrap().madness = 0;
                continue;
            }
            let position = self.positions[&id];
            let exposed = shrines
                .iter()
                .any(|p| p.x.abs_diff(position.x) + p.y.abs_diff(position.y) <= rule.radius_cells);
            // Stable actor/tick sampling means save/reload cannot reroll a
            // whisper. Multiple shrines do not multiply one tick's exposure.
            let roll = mix_seed(self.tick, 0, &id, 71) % 100;
            let actor = self.actors.get_mut(&id).unwrap();
            if !exposed {
                actor.madness = actor.madness.saturating_sub(rule.decay_per_tick);
            } else if roll < u64::from(rule.exposure_percent) {
                actor.madness = actor
                    .madness
                    .saturating_add(rule.exposure_gain)
                    .min(rule.conversion_threshold);
            }
            if actor.madness >= rule.conversion_threshold {
                let previous_faction_id = actor.faction_id.clone();
                if self.transfer_living_actor(&id, &rule.faction_id) {
                    events.push(FactionWorldEvent::MadnessConverted {
                        actor_id: id,
                        previous_faction_id,
                    });
                }
            }
        }
        events
    }

    pub fn assign_island_companion(&mut self, id: &str, slot: usize) -> bool {
        if slot >= 4 || !self.living_actor("character.protagonist.captain") {
            return false;
        }
        let Some(person) = self.living_person(id) else {
            return false;
        };
        if person.sex != PersonSex::Female
            || !person.age.is_some_and(|age| age >= 18)
            || !person.loyal_to_michael
            || self.actors[id].faction_id != "faction.michael"
            || self
                .party
                .iter()
                .enumerate()
                .any(|(i, other)| i != slot && other == id)
        {
            return false;
        }
        self.cancel_approach();
        let previous = std::mem::replace(&mut self.party[slot], id.into());
        if previous != id {
            if self
                .actors
                .get(&previous)
                .is_some_and(|a| a.faction_id == "faction.michael")
            {
                self.cancel_actor_travel(&previous);
            }
        }
        self.cancel_actor_travel(id);
        true
    }

    pub fn dismiss_island_companion(&mut self, slot: usize) -> bool {
        if slot >= 4
            || self.party[slot].is_empty()
            || !self.living_actor("character.protagonist.captain")
        {
            return false;
        }
        self.cancel_approach();
        let id = std::mem::take(&mut self.party[slot]);
        // Remain safely where she is, without an obsolete party travel order.
        if self
            .actors
            .get(&id)
            .is_some_and(|a| a.faction_id == "faction.michael")
        {
            self.cancel_actor_travel(&id);
        }
        true
    }

    pub fn mechanical_followers(&self) -> Vec<String> {
        self.actors
            .iter()
            .filter(|(id, a)| {
                a.faction_id == "faction.michael"
                    && a.definition_id == self.rules.machinery.definition_id
                    && self.living_actor(id)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    fn follower_destination(
        &self,
        id: &str,
        target: IslandPoint,
        occupied: &BTreeSet<IslandPoint>,
    ) -> Option<IslandPoint> {
        let start = *self.positions.get(id)?;
        let mut candidates: Vec<_> = self
            .navigation
            .walkable
            .iter()
            .copied()
            .filter(|point| {
                !occupied.contains(point)
                    && u64::from(point.x.abs_diff(target.x)) + u64::from(point.y.abs_diff(target.y))
                        <= 3
            })
            .collect();
        candidates.sort_by_key(|point| {
            (
                u64::from(point.x.abs_diff(target.x)) + u64::from(point.y.abs_diff(target.y)),
                *point,
            )
        });
        candidates
            .into_iter()
            .find(|point| self.navigation.path(start, *point).is_some())
    }

    fn plan_party_move(&self, target: IslandPoint) -> Result<Vec<(String, IslandPoint)>, String> {
        let captain = "character.protagonist.captain";
        if !self.living_actor(captain)
            || self
                .navigation
                .path(self.positions[captain], target)
                .is_none()
        {
            return Err("Michael cannot reach that destination.".into());
        }
        let mut orders = vec![(captain.to_string(), target)];
        let mut occupied = BTreeSet::from([target]);
        let machines = self.mechanical_followers();
        for id in self.party.iter().chain(&machines) {
            if id.is_empty() || self.casualties.contains_key(id) {
                continue;
            }
            if self
                .actors
                .get(id)
                .is_some_and(|a| a.undead && a.faction_id == "faction.cthulhu.prototype")
            {
                continue;
            }
            let name = if machines.contains(id) {
                self.machine_display_name()
            } else {
                let Some(person) = self.living_person(id) else {
                    return Err("A companion is unavailable.".into());
                };
                if !person.loyal_to_michael || self.actors[id].faction_id != "faction.michael" {
                    return Err(format!(
                        "{} is not an available companion.",
                        person.display_name
                    ));
                }
                &person.display_name
            };
            let Some(destination) = self.follower_destination(id, target, &occupied) else {
                // Equipment never immobilizes the human party. A separated dog
                // keeps its real position and can catch up on a later open route.
                if machines.contains(id) {
                    continue;
                }
                return Err(format!("{} cannot reach the party destination.", name));
            };
            occupied.insert(destination);
            orders.push((id.clone(), destination));
        }
        Ok(orders)
    }

    pub fn party_move_failure(&self, target: IslandPoint) -> String {
        self.plan_party_move(target).err().unwrap_or_default()
    }

    fn cancel_approach(&mut self) {
        if self.approach_target.take().is_none() {
            return;
        }
        self.cancel_actor_travel("character.protagonist.captain");
        for id in self
            .party
            .clone()
            .iter()
            .chain(self.mechanical_followers().iter())
            .filter(|id| !id.is_empty())
        {
            if self
                .actors
                .get(id)
                .is_some_and(|a| a.faction_id == "faction.michael")
            {
                self.cancel_actor_travel(id);
            }
        }
    }

    /// Only choose a destination; all motion remains in the existing party planner.
    fn approach_destination(&self, id: &str) -> Option<IslandPoint> {
        if !self.can_approach_person(id) {
            return None;
        }
        let target = self.positions[id];
        let start = self.positions["character.protagonist.captain"];
        let mut candidates: Vec<_> = self
            .navigation
            .walkable
            .iter()
            .copied()
            .filter_map(|point| {
                let distance =
                    u64::from(point.x.abs_diff(target.x)) + u64::from(point.y.abs_diff(target.y));
                if distance > 2 || !self.navigation.clear_line(point, target) {
                    return None;
                }
                self.navigation
                    .path(start, point)
                    .map(|path| (path.len(), distance, point))
            })
            .collect();
        candidates.sort();
        candidates
            .into_iter()
            .map(|(_, _, point)| point)
            .find(|point| self.plan_party_move(*point).is_ok())
    }

    pub fn approach_island_person(&mut self, id: &str) -> bool {
        let Some(destination) = self.approach_destination(id) else {
            return false;
        };
        if !self.move_island_party(destination) {
            return false;
        }
        // move_island_party cancels any previous explicit control; retain tracking
        // only after that shared order path has completed its atomic preflight.
        self.approach_target = Some(id.into());
        if self.accessible_person(id) {
            self.cancel_approach();
        }
        true
    }

    fn advance_approach(&mut self) {
        let Some(id) = self.approach_target.clone() else {
            return;
        };
        if self.accessible_person(&id) {
            // Hold the party, but do not finalize arrival before the NPC gets
            // her own movement step. She may leave range in this same tick.
            self.cancel_approach();
            self.approach_target = Some(id);
        } else if !self.approach_island_person(&id) {
            self.cancel_approach();
        }
    }

    pub fn move_island_party(&mut self, target: IslandPoint) -> bool {
        let Ok(orders) = self.plan_party_move(target) else {
            return false;
        };
        // Preflight completed for everyone before mutating any current order.
        for (id, destination) in orders {
            self.order_move(&id, destination)
                .expect("preflighted party destination");
        }
        true
    }

    /// Player and AI use the same travel queue. Destination coordinates are
    /// validated before changing the previous order.
    pub fn order_move(
        &mut self,
        actor_id: &str,
        destination: IslandPoint,
    ) -> Result<(), FactionWorldError> {
        if !self.actors.contains_key(actor_id) {
            return Err(FactionWorldError::UnknownActor(actor_id.to_owned()));
        }
        let start = self
            .positions
            .get(actor_id)
            .copied()
            .ok_or_else(|| FactionWorldError::MissingIslandPosition(actor_id.to_owned()))?;
        let destination_id = format!("move.{actor_id}");
        if self.navigation.path(start, destination).is_none() {
            return Err(FactionWorldError::UnreachableDestination(destination_id));
        }
        if actor_id == "character.protagonist.captain" {
            self.cancel_approach();
            self.player_attack_target = None;
        } else if self.actors[actor_id].faction_id == "faction.michael"
            && self.party.iter().any(|id| id == actor_id)
        {
            self.cancel_approach();
        }
        self.navigation
            .destinations
            .insert(destination_id.clone(), destination);
        self.travel_orders
            .insert(actor_id.to_owned(), destination_id);
        Ok(())
    }

    pub fn enqueue_production(
        &mut self,
        faction_id: &str,
        building_id: &str,
        rule: ProductionRule,
    ) -> Result<FactionWorldEvent, FactionWorldError> {
        if self.eliminated_factions.contains(faction_id) {
            return Err(FactionWorldError::FactionEliminated(faction_id.into()));
        }
        if rule.production_ticks == 0 {
            return Err(FactionWorldError::InvalidProductionTicks);
        }
        if rule.actor_definition_id == self.rules.machinery.definition_id
            && (faction_id != "faction.michael"
                || rule != self.rules.machine_production
                // The berths the workshop actually has, which grows with it.
                // Reading the base capacity here made every level past the
                // first advertise a berth the one gated door would refuse --
                // and refuse it as "not enough salvage", which was a lie.
                || self.mechanical_dog_count() >= self.machine_berths())
        {
            return Err(FactionWorldError::QueueFull(building_id.into()));
        }
        let faction = self
            .factions
            .get(faction_id)
            .ok_or_else(|| FactionWorldError::UnknownFaction(faction_id.to_owned()))?;
        let building = faction
            .buildings
            .get(building_id)
            .ok_or_else(|| FactionWorldError::UnknownBuilding(building_id.to_owned()))?;
        if building.faction_id != faction_id {
            return Err(FactionWorldError::ProducerFactionMismatch);
        }
        if !building.operational {
            return Err(FactionWorldError::BuildingNotOperational(
                building_id.to_owned(),
            ));
        }
        if building.archetype_id != rule.producer_archetype_id {
            return Err(FactionWorldError::ProducerArchetypeMismatch);
        }
        if building.development.is_some()
            || building.production_queue.len() >= building.queue_capacity
        {
            return Err(FactionWorldError::QueueFull(building_id.to_owned()));
        }
        if faction.population_used.saturating_add(rule.population_use) > faction.population_capacity
        {
            return Err(FactionWorldError::InsufficientPopulation);
        }
        for (resource_id, required) in &rule.costs {
            if !self.rules.knows(resource_id) {
                return Err(FactionWorldError::UnknownResource(resource_id.clone()));
            }
            let available = faction.resources.get(resource_id).copied().unwrap_or(0);
            if available < *required {
                return Err(FactionWorldError::InsufficientResource {
                    resource_id: resource_id.clone(),
                    required: *required,
                    available,
                });
            }
        }
        self.spend_resources(faction_id, &rule.costs)
            .map_err(|_| FactionWorldError::InvalidPolicy(faction_id.to_owned()))?;
        let faction = self.factions.get_mut(faction_id).unwrap();
        faction.population_used += rule.population_use;
        self.next_order_serial += 1;
        let order_id = format!("production_order.{faction_id}.{}", self.next_order_serial);
        let event = FactionWorldEvent::ProductionQueued {
            faction_id: faction_id.to_owned(),
            building_id: building_id.to_owned(),
            order_id: order_id.clone(),
            production_rule_id: rule.id.clone(),
        };
        faction
            .buildings
            .get_mut(building_id)
            .expect("building was validated above")
            .production_queue
            .push(ProductionOrder {
                id: order_id,
                remaining_ticks: rule.production_ticks,
                reserved_costs: rule.costs.clone(),
                rule,
            });
        Ok(event)
    }

    pub fn advance_production_tick(&mut self) -> Vec<FactionWorldEvent> {
        if self.paused {
            return Vec::new();
        }
        self.tick += 1;
        let construction_ready: BTreeSet<String> = self
            .factions
            .values()
            .flat_map(|f| f.buildings.values())
            .filter(|b| {
                b.construction
                    .as_ref()
                    .is_some_and(|job| self.building_work_ready(b, job))
            })
            .map(|b| b.id.clone())
            .collect();
        let repair_ready: BTreeSet<String> = self
            .factions
            .values()
            .flat_map(|f| f.buildings.values())
            .filter(|b| {
                b.repair
                    .as_ref()
                    .is_some_and(|job| self.building_work_ready(b, job))
            })
            .map(|b| b.id.clone())
            .collect();
        let mut work_finished = Vec::new();
        let mut completed = Vec::new();
        for faction in self.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                if let Some(job) = &mut building.repair {
                    if repair_ready.contains(&building.id) {
                        job.remaining_ticks = job.remaining_ticks.saturating_sub(1);
                        if job.remaining_ticks == 0 {
                            if let Some(rule) = self.rules.repairs.get(&building.archetype_id) {
                                building.health = building
                                    .health
                                    .saturating_add(rule.health_gain)
                                    .min(building.max_health);
                            }
                            building.repair = None;
                            work_finished
                                .push((faction.id.clone(), format!("repair.{}", building.id)));
                        }
                    }
                }
                if let Some(job) = &mut building.construction {
                    if construction_ready.contains(&building.id) {
                        job.remaining_ticks = job.remaining_ticks.saturating_sub(1);
                        if job.remaining_ticks == 0 {
                            building.construction = None;
                            building.operational = true;
                        }
                    }
                    continue;
                }
                if !building.operational {
                    continue;
                }
                if let Some(order) = &mut building.development {
                    order.remaining_ticks = order.remaining_ticks.saturating_sub(1);
                    if order.remaining_ticks == 0 {
                        building.level += 1;
                        building.max_health =
                            building.max_health.saturating_add(order.rule.health_gain);
                        building.health = building.health.saturating_add(order.rule.health_gain);
                        faction.population_capacity = faction
                            .population_capacity
                            .saturating_add(order.rule.population_gain);
                        building.development = None;
                    }
                    continue;
                }
                for order in &mut building.production_queue {
                    order.remaining_ticks = order.remaining_ticks.saturating_sub(1);
                }
                let mut pending = Vec::with_capacity(building.production_queue.len());
                for order in building.production_queue.drain(..) {
                    if order.remaining_ticks == 0 {
                        completed.push((
                            faction.id.clone(),
                            building.id.clone(),
                            building.node_id.clone(),
                            building.rally_point_id.clone(),
                            order,
                        ));
                    } else {
                        pending.push(order);
                    }
                }
                building.production_queue = pending;
            }
        }

        for (faction, action) in work_finished {
            self.release_construction_assignment(&faction, &action);
        }
        completed.sort_by(|left, right| left.4.id.cmp(&right.4.id));
        let mut events = Vec::with_capacity(completed.len());
        // The dead keep their names. Cthulhu returns eligible casualties at
        // midnight as the same people, so a name freed by a death is not free
        // at all -- reusing it produces two Samuel Prices the moment the first
        // one walks back. Count the casualty record as taken.
        let mut names_in_use: BTreeSet<String> = self
            .actors
            .values()
            .filter_map(|actor| actor.person.as_ref())
            .chain(
                self.casualties
                    .values()
                    .filter_map(|casualty| casualty.actor.person.as_ref()),
            )
            .map(|person| person.display_name.clone())
            .collect();
        for (faction_id, building_id, node_id, rally_point_id, order) in completed {
            self.next_actor_serial += 1;
            // The one place a written identity stands in for a rolled one: this
            // faction's next unraised notable for this definition. Nothing else
            // about her production differs -- same rule, same costs, same
            // provenance, same rally point as the worker she arrives instead of.
            let notable = if order.rule.actor_kind == "machine" {
                None
            } else {
                self.rules
                    .notables
                    .iter()
                    .find(|notable| {
                        notable.faction_id == faction_id
                            && notable.definition_id == order.rule.actor_definition_id
                            && !self.actors.contains_key(&notable.character_id)
                            && !self.casualties.contains_key(&notable.character_id)
                    })
                    .cloned()
            };
            let instance_id = match &notable {
                Some(notable) => notable.character_id.clone(),
                None => format!("actor_instance.{faction_id}.{}", self.next_actor_serial),
            };
            let actor = ProducedActor {
                madness: 0,
                undead: false,
                person: if order.rule.actor_kind == "machine" {
                    None
                } else if let Some(notable) = &notable {
                    Some(NamedPerson {
                        id: instance_id.clone(),
                        display_name: notable.identity.board_name.clone(),
                        alive_today: true,
                        sex: notable.identity.sex,
                        age: Some(notable.identity.age),
                        backstory: notable.identity.backstory.clone(),
                        recruitment_offer: notable.identity.recruitment_offer.clone(),
                        companion_response: notable.identity.companion_response.clone(),
                        ..Default::default()
                    })
                } else {
                    produced_person(
                        &self.rules.personas,
                        &order.rule.actor_definition_id,
                        &instance_id,
                        self.next_actor_serial,
                        &names_in_use,
                    )
                },
                instance_id,
                definition_id: order.rule.actor_definition_id,
                actor_kind: order.rule.actor_kind,
                faction_id: faction_id.clone(),
                node_id,
                current_assignment_id: None,
                provenance: ActorProductionProvenance {
                    faction_id,
                    producer_building_id: building_id,
                    production_rule_id: order.rule.id,
                    reserved_costs: order.reserved_costs,
                    completed_tick: self.tick,
                    rally_point_id,
                },
            };
            if let Some(person) = actor.person.as_ref() {
                names_in_use.insert(person.display_name.clone());
            }
            self.actors.insert(actor.instance_id.clone(), actor.clone());
            if let Some(profile) = self.combat_profiles.get(&actor.definition_id) {
                self.unit_combat.insert(
                    actor.instance_id.clone(),
                    IslandCombatState {
                        health: profile.health,
                        next_attack_tick: self.tick,
                        population_use: order.rule.population_use,
                    },
                );
            }
            if let Some(position) = self.navigation.destinations.get(&actor.node_id) {
                self.positions.insert(actor.instance_id.clone(), *position);
            }
            events.push(FactionWorldEvent::ActorProduced { actor });
        }
        events
    }

    fn advance_companion_defense(&mut self) {
        const CAPTAIN: &str = "character.protagonist.captain";
        if !self.living_actor(CAPTAIN) {
            return;
        }
        let machines = self.mechanical_followers();
        let target = self
            .travel_orders
            .get(CAPTAIN)
            .and_then(|node| self.navigation.destinations.get(node))
            .copied()
            .unwrap_or(self.positions[CAPTAIN]);
        let mut occupied: BTreeSet<_> = self.positions.values().copied().collect();
        for id in &machines {
            let point = self.positions[id];
            if self.travel_orders.contains_key(id)
                || point
                    .x
                    .abs_diff(target.x)
                    .saturating_add(point.y.abs_diff(target.y))
                    <= 3
            {
                continue;
            }
            if let Some(destination) = self.follower_destination(id, target, &occupied) {
                if self.order_move(id, destination).is_ok() {
                    occupied.insert(destination);
                }
            }
        }
        if !self.living_actor(CAPTAIN)
            || self.travel_orders.contains_key(CAPTAIN)
            || self.approach_target.is_some()
        {
            return;
        }
        let center = self.positions[CAPTAIN];
        let nearby = |point: IslandPoint| {
            center
                .x
                .abs_diff(point.x)
                .saturating_add(center.y.abs_diff(point.y))
                <= 4
        };
        let mut occupied: BTreeSet<IslandPoint> = self.positions.values().copied().collect();
        for id in self
            .party
            .clone()
            .into_iter()
            .chain(machines)
            .filter(|id| !id.is_empty())
        {
            if !self.living_actor(&id)
                || self.actors[&id].faction_id != "faction.michael"
                || self.travel_orders.contains_key(&id)
                || self.island_firing_target(&id).is_some()
            {
                continue;
            }
            let origin = self.positions[&id];
            if !nearby(origin) {
                continue;
            }
            let actor = &self.actors[&id];
            let next = self
                .actors
                .iter()
                .filter(|(target_id, target)| {
                    self.living_actor(target_id)
                        && self
                            .hostilities
                            .contains(&(actor.faction_id.clone(), target.faction_id.clone()))
                })
                .filter_map(|(target_id, _)| {
                    let target = self.positions[target_id];
                    if !nearby(target)
                        || !(self.navigation.clear_line(origin, target)
                            || self.navigation.clear_line(center, target))
                    {
                        return None;
                    }
                    // Feed occupied cells and the leash into the same pathfinder,
                    // rather than rejecting one blocked shortest path afterward.
                    let mut local_navigation = self.navigation.clone();
                    local_navigation.walkable.retain(|point| {
                        nearby(*point)
                            && (!occupied.contains(point) || *point == origin || *point == target)
                    });
                    let path = local_navigation.path(origin, target)?;
                    let next = *path.get(1)?;
                    if occupied.contains(&next) || !path.iter().all(|point| nearby(*point)) {
                        return None;
                    }
                    Some((path.len(), target_id.clone(), next))
                })
                .min()
                .map(|(_, _, point)| point);
            if let Some(next) = next {
                // One ordinary movement step, consumed below in the existing
                // travel phase. No hidden chase queue survives a new command.
                if self.order_move(&id, next).is_ok() {
                    occupied.insert(next);
                }
            }
        }
    }

    /// Single simulation step used by the island runtime. Production and travel
    /// share the same pause boundary and clock. Rendering never advances these.
    pub fn advance_island_tick(&mut self) -> Vec<FactionWorldEvent> {
        if self.paused {
            return Vec::new();
        }
        let mut events = self.advance_faction_decisions();
        events.extend(self.advance_production_tick());
        self.advance_approach();
        self.advance_companion_defense();
        // Decide from one pre-movement snapshot, not partially moved ID order.
        // Keep the strategic travel order so movement resumes when the shot is lost.
        // Only autonomous factions hold; player-directed actors retain movement.
        let holding: BTreeSet<String> = self
            .actors
            .iter()
            .filter(|(id, actor)| {
                self.policies.contains_key(&actor.faction_id)
                    // A recalled repairer must leave the firing line and reach
                    // its holding. Combat hold must not cancel that work order.
                    && !actor.current_assignment_id.as_deref().is_some_and(|a| a.starts_with("repair."))
                    && (self.island_firing_target(id).is_some()
                        || (!actor.current_assignment_id.as_deref().is_some_and(|a| {
                            a.starts_with("defend.")
                                || a.starts_with("construct.")
                                || a.starts_with("repair.")
                        }) && self.island_siege_target(id).is_some()))
            })
            .map(|(id, _)| id.clone())
            .collect();
        for (actor_id, destination_id) in self.travel_orders.clone() {
            if holding.contains(&actor_id) {
                continue;
            }
            let (Some(start), Some(goal)) = (
                self.positions.get(&actor_id).copied(),
                self.navigation.destinations.get(&destination_id).copied(),
            ) else {
                continue;
            };
            let Some(path) = self.navigation.path(start, goal) else {
                continue;
            };
            if let Some(next) = path.get(1).copied() {
                self.positions.insert(actor_id.clone(), next);
                events.push(FactionWorldEvent::ActorMoved {
                    actor_id: actor_id.clone(),
                    position: next,
                });
            }
            if self.positions.get(&actor_id) == Some(&goal) {
                if let Some(actor) = self.actors.get_mut(&actor_id) {
                    actor.node_id = destination_id.clone();
                }
                self.travel_orders.remove(&actor_id);
                events.push(FactionWorldEvent::ActorArrived {
                    actor_id,
                    destination_id,
                });
            }
        }
        events.extend(self.resolve_island_skirmish());
        events.extend(self.advance_madness());
        events.extend(self.return_midnight_casualties());
        self.decay_casualties();
        if self
            .approach_target
            .as_ref()
            .is_some_and(|id| self.accessible_person(id))
        {
            self.cancel_approach();
        }
        // Triggers run immediately before the campaign clock, at the same
        // settled point, so an effect of theirs -- a recorded heat event, a
        // faction turned hostile -- is visible to the arbitration in the very
        // tick it happens rather than a tick later.
        self.advance_triggers();
        // Leads read the same settled world the triggers just left, so a
        // companion notices what this tick actually did rather than a state
        // half-way through it.
        self.advance_leads();
        // One arbitration point, at the settled end of the tick: movement,
        // combat, elimination, madness and the midnight return have all
        // resolved and `self.tick` has already advanced, so a cause reads a
        // world nothing else will change this tick. A pass before the tick
        // would re-fire on load.
        self.advance_campaign_clock(&events);
        events
    }

    /// Every authored trigger, in authored order. A trigger is a transaction:
    /// either every effect applies or none does and the trigger stays pending,
    /// so a firing can never leave the world half-changed.
    fn advance_triggers(&mut self) {
        let triggers = self.rules.triggers.clone();
        for trigger in &triggers {
            if !trigger.repeat && self.fired_triggers.contains_key(&trigger.id) {
                continue;
            }
            if !trigger
                .when
                .iter()
                .all(|condition| self.trigger_condition_met(condition))
            {
                continue;
            }
            if !self.trigger_effects_applicable(&trigger.then) {
                continue;
            }
            if !self.fired_triggers.contains_key(&trigger.id)
                && self.fired_triggers.len() >= MAX_TRIGGER_HISTORY
            {
                continue;
            }
            for effect in &trigger.then {
                self.apply_trigger_effect(effect);
            }
            self.fired_triggers.insert(trigger.id.clone(), self.tick);
        }
    }

    /// A companion brings a lead when she is standing with Michael and the
    /// thing she noticed is actually true of the world. She cannot raise one
    /// before she is recruited: her judgment is the thing the player recruited
    /// her for, and the authority is explicit that Michael does not do all the
    /// intellectual work himself.
    fn advance_leads(&mut self) {
        let leads = self.rules.leads.clone();
        for lead in &leads {
            if self.open_leads.contains_key(&lead.id) || self.resolved_leads.contains_key(&lead.id)
            {
                continue;
            }
            if !self.companion_is_present(&lead.companion_id) {
                continue;
            }
            if !lead
                .opens_when
                .iter()
                .all(|condition| self.trigger_condition_met(condition))
            {
                continue;
            }
            if self.open_leads.len() >= MAX_OPEN_LEADS {
                continue;
            }
            self.open_leads.insert(lead.id.clone(), self.tick);
        }
    }

    /// She is with Michael, alive, and loyal. A dead or estranged companion
    /// brings nothing, which is the cost of losing her.
    ///
    /// An empty `companion_id` means *any* companion: the lead belongs to
    /// whoever is standing with him rather than to one named woman. Most of the
    /// women a player actually recruits are produced by the factions, not
    /// authored, so a game where only named characters can raise anything is a
    /// game where most players never see a lead at all.
    fn companion_is_present(&self, companion_id: &str) -> bool {
        if companion_id.is_empty() {
            return self.loyal_companion_count() > 0;
        }
        self.actors.get(companion_id).is_some_and(|actor| {
            actor.faction_id == "faction.michael"
                && actor
                    .person
                    .as_ref()
                    .is_some_and(|person| person.loyal_to_michael && person.alive_today)
        })
    }

    /// Who is actually raising a lead: the woman it names, or -- for a lead
    /// any companion can raise -- the first loyal companion by id, so the same
    /// world always attributes it to the same person.
    pub fn lead_speaker(&self, lead: &ScenarioLead) -> Option<&NamedPerson> {
        if !lead.companion_id.is_empty() {
            return self.actors.get(&lead.companion_id)?.person.as_ref();
        }
        self.actors
            .values()
            .filter_map(|actor| actor.person.as_ref())
            .find(|person| person.loyal_to_michael && person.alive_today)
    }

    /// Every lead waiting on the player's judgment, in authored order.
    pub fn open_leads(&self) -> Vec<&ScenarioLead> {
        self.rules
            .leads
            .iter()
            .filter(|lead| self.open_leads.contains_key(&lead.id))
            .collect()
    }

    /// The player backs one of her two readings. Like a trigger, this is a
    /// transaction: either the whole answer lands or the lead stays open and
    /// nothing changed. Answering is the player's verb, not the tick's, and it
    /// is refused unless she is still there to be agreed with.
    pub fn resolve_lead(&mut self, lead_id: &str, interpretation_id: &str) -> bool {
        if !self.open_leads.contains_key(lead_id) {
            return false;
        }
        let Some(lead) = self
            .rules
            .leads
            .iter()
            .find(|lead| lead.id == lead_id)
            .cloned()
        else {
            return false;
        };
        if !self.companion_is_present(&lead.companion_id) {
            return false;
        }
        let Some(interpretation) = lead
            .interpretations
            .iter()
            .find(|interpretation| interpretation.id == interpretation_id)
        else {
            return false;
        };
        if !self.trigger_effects_applicable(&interpretation.then) {
            return false;
        }
        for effect in &interpretation.then {
            self.apply_trigger_effect(effect);
        }
        self.open_leads.remove(lead_id);
        self.resolved_leads
            .insert(lead_id.into(), interpretation_id.into());
        true
    }

    fn trigger_condition_met(&self, condition: &TriggerCondition) -> bool {
        match condition {
            TriggerCondition::DayAtLeast { day } => self.day() >= *day,
            TriggerCondition::FactionEliminated { faction_id } => {
                self.eliminated_factions.contains(faction_id)
            }
            TriggerCondition::ResourceAtLeast {
                faction_id,
                resource_id,
                amount,
            } => self.stored_resource(faction_id, resource_id) >= *amount,
            TriggerCondition::HoldingLevelAtLeast {
                faction_id,
                archetype_id,
                level,
            } => self.factions.get(faction_id).is_some_and(|faction| {
                faction.buildings.values().any(|building| {
                    building.archetype_id == *archetype_id && building.level >= *level
                })
            }),
            TriggerCondition::ActorAtCell { faction_id, x, y } => {
                let cell = IslandPoint { x: *x, y: *y };
                self.positions.iter().any(|(actor_id, position)| {
                    *position == cell
                        && self
                            .actors
                            .get(actor_id)
                            .is_some_and(|actor| actor.faction_id == *faction_id)
                })
            }
            TriggerCondition::FlagSet { flag } => self.flags.contains(flag),
            TriggerCondition::HeatSignalled { signal } => {
                self.heat.iter().any(|event| event.signal == *signal)
            }
            TriggerCondition::ConfrontationBegun => self.confrontation.is_some(),
            TriggerCondition::LoyalCompanionCount { at_least } => {
                self.loyal_companion_count() >= *at_least
            }
            TriggerCondition::PartySize { at_least } => self.party_size() >= *at_least,
        }
    }

    /// Whether every effect would take. Checked before any of them is applied,
    /// which is what makes a firing all-or-nothing.
    fn trigger_effects_applicable(&self, effects: &[TriggerEffect]) -> bool {
        effects.iter().all(|effect| match effect {
            TriggerEffect::SetFlag { flag } => {
                !flag.is_empty()
                    && flag.len() <= 256
                    && (self.flags.contains(flag) || self.flags.len() < MAX_FLAGS)
            }
            TriggerEffect::GrantResource {
                faction_id,
                resource_id,
                amount,
            } => {
                *amount > 0
                    && self.rules.knows(resource_id)
                    && self.factions.contains_key(faction_id)
                    && !self.eliminated_factions.contains(faction_id)
            }
            TriggerEffect::GrantItem { actor_id, item_id } => {
                self.actors.contains_key(actor_id)
                    && self.rules.items.contains_key(item_id)
                    // An item already held is not a failure to grant, but it is
                    // not applicable either: the trigger stays pending rather
                    // than firing on a no-op.
                    && !self
                        .actor_inventory(actor_id)
                        .iter()
                        .any(|held| held == item_id)
            }
            TriggerEffect::RecordHeat {
                id,
                signal,
                severity,
            } => {
                !id.is_empty()
                    && (1..=100).contains(severity)
                    && self.rules.campaign_clock.signal_channels.contains(signal)
                    && self.heat.len() < HEAT_LEDGER_CAP
                    && !self.heat.iter().any(|event| &event.id == id)
            }
            TriggerEffect::SetHostility { a, b, hostile } => {
                a != b
                    && self.factions.contains_key(a)
                    && self.factions.contains_key(b)
                    && (self.hostilities.contains(&(a.clone(), b.clone())) != *hostile
                        || self.hostilities.contains(&(b.clone(), a.clone())) != *hostile)
            }
            TriggerEffect::SetQuestStage { quest_id, stage_id } => {
                self.rules
                    .quests
                    .get(quest_id)
                    .is_some_and(|quest| quest.stages.contains_key(stage_id))
                    && self.quest_stages.get(quest_id).is_some_and(|current| {
                        current != stage_id
                            && self.rules.quests[quest_id].stages[current]
                                .terminal
                                .is_none()
                    })
            }
        })
    }

    fn apply_trigger_effect(&mut self, effect: &TriggerEffect) {
        match effect {
            TriggerEffect::SetFlag { flag } => {
                self.flags.insert(flag.clone());
            }
            TriggerEffect::GrantResource {
                faction_id,
                resource_id,
                amount,
            } => {
                // The same checked door income uses: an undeclared key is
                // refused, and the gain clamps to the faction's storage cap.
                let _ = self.gain_resource(faction_id, resource_id, *amount);
            }
            TriggerEffect::GrantItem { actor_id, item_id } => {
                let _ = self.grant_item(actor_id, item_id);
            }
            TriggerEffect::RecordHeat {
                id,
                signal,
                severity,
            } => {
                self.record_heat_event(id, signal, *severity);
            }
            TriggerEffect::SetHostility { a, b, hostile } => {
                if *hostile {
                    // A truce pair may never be hostile, and the loader refuses
                    // a save where one is. Ending the truce is part of the change.
                    self.survival_truces.retain(|truce| {
                        !(truce.a == *a && truce.b == *b || truce.a == *b && truce.b == *a)
                    });
                    self.hostilities.insert((a.clone(), b.clone()));
                    self.hostilities.insert((b.clone(), a.clone()));
                } else {
                    self.hostilities.remove(&(a.clone(), b.clone()));
                    self.hostilities.remove(&(b.clone(), a.clone()));
                }
            }
            TriggerEffect::SetQuestStage { quest_id, stage_id } => {
                self.quest_stages.insert(quest_id.clone(), stage_id.clone());
            }
        }
    }

    /// How many actors `recruit_island_person` has actually marked loyal.
    /// Reads the same field that verb writes; invents no second count to
    /// drift out of sync with it.
    pub fn loyal_companion_count(&self) -> u32 {
        self.actors
            .values()
            .filter(|actor| {
                actor
                    .person
                    .as_ref()
                    .is_some_and(|person| person.loyal_to_michael)
            })
            .count() as u32
    }

    /// How many of the four `party` slots are actually filled. Reads the
    /// same array `assign_island_companion` writes.
    pub fn party_size(&self) -> u32 {
        self.party.iter().filter(|slot| !slot.is_empty()).count() as u32
    }

    /// Append one entry to the heat ledger. Refuses a duplicate ID, an
    /// undeclared signal channel, an out-of-range severity, and a ledger that
    /// has reached its cap. Returns whether the ledger changed.
    pub fn record_heat_event(&mut self, id: &str, signal: &str, severity: u32) -> bool {
        if id.is_empty()
            || id.len() > 256
            || !(1..=100).contains(&severity)
            || !self
                .rules
                .campaign_clock
                .signal_channels
                .iter()
                .any(|channel| channel == signal)
            || self.heat.len() >= HEAT_LEDGER_CAP
            || self.heat.iter().any(|event| event.id == id)
        {
            return false;
        }
        self.heat.push(HeatEvent {
            id: id.to_string(),
            signal: signal.to_string(),
            severity,
            tick: self.tick,
        });
        true
    }

    /// The ledger's summed severity. Deliberately not exposed through the
    /// bridge: the authored record says heat is never a number the player sees.
    pub fn heat_severity(&self) -> u32 {
        self.heat
            .iter()
            .fold(0u32, |total, event| total.saturating_add(event.severity))
    }

    /// The channels the island has actually signalled on, which is the only
    /// tell the player gets.
    pub fn heat_channels(&self) -> BTreeSet<String> {
        self.heat.iter().map(|event| event.signal.clone()).collect()
    }

    pub fn confrontation_cause(&self) -> Option<ConfrontationCause> {
        self.confrontation.map(|record| record.cause)
    }

    fn advance_campaign_clock(&mut self, events: &[FactionWorldEvent]) {
        let clock = self.rules.campaign_clock.clone();
        for source in &clock.sources {
            for event in events {
                let subject = match (source.on, event) {
                    (
                        HeatTrigger::MadnessConversion,
                        FactionWorldEvent::MadnessConverted { actor_id, .. },
                    ) => actor_id,
                    (
                        HeatTrigger::MidnightReturn,
                        FactionWorldEvent::MidnightReturned { actor_id, .. },
                    ) => actor_id,
                    (
                        HeatTrigger::FactionEliminated,
                        FactionWorldEvent::FactionEliminated { faction_id },
                    ) => faction_id,
                    _ => continue,
                };
                let id = format!("{}:{subject}:{}", source.id, self.tick);
                self.record_heat_event(&id, &source.signal, source.severity);
            }
        }
        // First trigger wins: once a cause is recorded it is the campaign's,
        // whatever happens later.
        if self.confrontation.is_some() {
            return;
        }
        let mut satisfied = BTreeSet::new();
        if self
            .eliminated_factions
            .contains(&clock.deliberate_discovery_faction)
        {
            satisfied.insert(ConfrontationCause::DeliberateDiscovery);
        }
        if self.heat_severity() >= clock.terminal_severity {
            satisfied.insert(ConfrontationCause::TerminalHeat);
        }
        if self.day() >= clock.world_deadline_day {
            satisfied.insert(ConfrontationCause::Day100);
        }
        // Within one tick the authored order decides, so two causes landing
        // together always record the same one.
        if let Some(cause) = clock
            .priority
            .iter()
            .copied()
            .find(|cause| satisfied.contains(cause))
        {
            self.confrontation = Some(Confrontation {
                cause,
                tick: self.tick,
                day: self.day(),
            });
        }
    }

    pub fn dispatch_actor(
        &mut self,
        actor_id: &str,
        candidates: &[DispatchCandidate],
    ) -> Result<FactionWorldEvent, FactionWorldError> {
        if candidates.is_empty() {
            return Err(FactionWorldError::NoDispatchCandidates);
        }
        let actor = self
            .actors
            .get(actor_id)
            .ok_or_else(|| FactionWorldError::UnknownActor(actor_id.to_owned()))?;
        let faction = self
            .factions
            .get(&actor.faction_id)
            .ok_or_else(|| FactionWorldError::UnknownFaction(actor.faction_id.clone()))?;
        for candidate in candidates {
            if i64::from(candidate.wobble).abs() > i64::from(faction.wobble_limit) {
                return Err(FactionWorldError::WobbleOutOfBounds {
                    action_id: candidate.action_id.clone(),
                    wobble: candidate.wobble,
                    limit: faction.wobble_limit,
                });
            }
        }
        let chosen = candidates
            .iter()
            .max_by(|left, right| {
                left.total()
                    .cmp(&right.total())
                    .then_with(|| right.action_id.cmp(&left.action_id))
            })
            .expect("non-empty candidates checked above");
        let start = self
            .positions
            .get(actor_id)
            .copied()
            .ok_or_else(|| FactionWorldError::MissingIslandPosition(actor_id.to_owned()))?;
        let goal = self
            .navigation
            .destinations
            .get(&chosen.target_node_id)
            .copied()
            .ok_or_else(|| {
                FactionWorldError::UnreachableDestination(chosen.target_node_id.clone())
            })?;
        if self.navigation.path(start, goal).is_none() {
            return Err(FactionWorldError::UnreachableDestination(
                chosen.target_node_id.clone(),
            ));
        }
        let actor = self
            .actors
            .get_mut(actor_id)
            .expect("actor was validated above");
        actor.current_assignment_id = Some(chosen.action_id.clone());
        self.travel_orders
            .insert(actor_id.to_owned(), chosen.target_node_id.clone());
        Ok(FactionWorldEvent::ActorAssigned {
            actor_id: actor_id.to_owned(),
            faction_id: actor.faction_id.clone(),
            action_id: chosen.action_id.clone(),
            assignment: chosen.assignment.clone(),
            target_node_id: chosen.target_node_id.clone(),
            total_score: chosen.total(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fox_expansion_produces_real_skirmishers_with_bounded_population_and_saved_identity() {
        let fox = "faction.eastern_fox_people.prototype";
        let definition = "actor_def.eastern_fox_people.spear_skirmisher";
        let mut world = main_scenario_world();
        assert_eq!(world.factions.len(), 6); // Five AI factions plus Michael.
        assert!(world.factions.contains_key("faction.elves.prototype"));
        assert_eq!(world.factions[fox].population_capacity, 8);
        assert_eq!(world.combat_profiles[definition].health, 8);
        assert_eq!(world.combat_profiles[definition].cooldown_ticks, 40);
        let building = world.factions[fox].buildings.values().next().unwrap();
        let holding = world.navigation.destinations[&building.node_id];
        assert_eq!(
            building.archetype_id,
            "site_archetype.eastern_fox_people.river_market"
        );
        // Peaceful observation isolates production and deployment from losses.
        world.hostilities.clear();
        let mut moved = false;
        for _ in 0..20_000 {
            if world
                .actors
                .values()
                .filter(|a| a.faction_id == fox)
                .count()
                == 8
            {
                break;
            }
            world.advance_island_tick();
            moved |= world
                .actors
                .values()
                .filter(|a| a.faction_id == fox)
                .any(|a| world.positions[&a.instance_id] != holding);
        }
        assert!(
            moved,
            "New faction must deploy real actors, not only increment population"
        );
        let actors: Vec<_> = world
            .actors
            .values()
            .filter(|a| a.faction_id == fox)
            .collect();
        assert_eq!(actors.len(), 8);
        assert_eq!(world.factions[fox].population_used, 8);
        for actor in actors {
            assert_eq!(actor.definition_id, definition);
            assert_eq!(actor.actor_kind, "soldier");
            assert_eq!(actor.person.as_ref().unwrap().sex, PersonSex::Female);
            assert!(world.unit_combat.contains_key(&actor.instance_id));
        }
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        for _ in 0..8 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
        }
        world.paused = true;
        let frozen = world.clone();
        assert!(world.advance_island_tick().is_empty());
        assert_eq!(world, frozen);
    }

    /// Every proof reads the scenario document, never a compiled-in rule file.
    fn main_scenario() -> ScenarioDefinition {
        crate::scenario_fixture::main_scenario()
    }

    fn scenario_production(faction_id: &str) -> ProductionRule {
        main_scenario()
            .factions
            .iter()
            .find(|faction| faction.id == faction_id)
            .expect("scenario faction")
            .production
            .clone()
    }

    /// The one main-scenario world every proof runs on.
    fn main_scenario_world() -> FactionWorld {
        FactionWorld::from_scenario(&crate::scenario_fixture::main_scenario())
            .expect("main scenario world")
    }

    /// Run the island until something is true, or give up.
    ///
    /// Fixtures used to count ticks by hand -- "two ticks and a deckhand
    /// exists" -- which baked the authored production timings into every test.
    /// What those tests mean is "once a deckhand exists", so that is what they
    /// now wait for, and retiming the island does not falsify them.
    fn advance_until(
        world: &mut FactionWorld,
        limit: u32,
        ready: impl Fn(&FactionWorld) -> bool,
    ) -> bool {
        for _ in 0..limit {
            if ready(world) {
                return true;
            }
            world.advance_island_tick();
        }
        ready(world)
    }

    /// Wait for the island to actually raise one of each named unit. Tests that
    /// used to tick four times and then pick a marine off the board were
    /// relying on production being all but instantaneous; what they mean is
    /// "once these people exist".
    fn advance_until_raised(world: &mut FactionWorld, definitions: &[&str]) {
        let raised = advance_until(world, 20_000, |w| {
            definitions
                .iter()
                .all(|definition| w.actors.values().any(|a| &a.definition_id == definition))
        });
        assert!(raised, "the island raises {definitions:?}");
    }

    fn production_world() -> FactionWorld {
        let building = FactionBuilding {
            construction: None,
            repair: None,
            level: 1,
            max_health: default_building_health(),
            development: None,
            health: default_building_health(),
            id: "site.colonial.watch_fort.instance_1".into(),
            faction_id: "faction.colonial_powers.prototype".into(),
            archetype_id: "site_archetype.colonial.watch_fort".into(),
            node_id: "network_node.river_fork".into(),
            rally_point_id: "rally.colonial.watch_fort.gate".into(),
            operational: true,
            queue_capacity: 2,
            production_queue: Vec::new(),
        };
        let faction = FactionState {
            id: "faction.colonial_powers.prototype".into(),
            resources: [
                ("resource.provisions".into(), 4),
                ("resource.iron".into(), 2),
            ]
            .into_iter()
            .collect(),
            population_used: 0,
            population_capacity: 4,
            wobble_limit: 5,
            buildings: [(building.id.clone(), building)].into_iter().collect(),
        };
        let rules = main_scenario().scenario_rules();
        // Even a bare production fixture plays a scenario's rules, and a
        // declared quest is never absent from quest_stages -- from_scenario's
        // own seeding invariant, restated here since this fixture builds a
        // FactionWorld by hand rather than through from_scenario.
        let quest_stages = rules
            .quests
            .iter()
            .map(|(id, quest)| (id.clone(), quest.initial_stage.clone()))
            .collect();
        FactionWorld {
            rules,
            quest_stages,
            factions: [(faction.id.clone(), faction)].into_iter().collect(),
            navigation: IslandNavigation {
                building_obstacles: BTreeMap::new(),
                walkable: (0..5)
                    .flat_map(|x| (0..3).map(move |y| IslandPoint { x, y }))
                    .collect(),
                destinations: [
                    ("network_node.river_fork".into(), IslandPoint { x: 0, y: 1 }),
                    (
                        "network_node.smuggler_cove".into(),
                        IslandPoint { x: 4, y: 1 },
                    ),
                    (
                        "network_node.black_beach".into(),
                        IslandPoint { x: 2, y: 2 },
                    ),
                ]
                .into_iter()
                .collect(),
            },
            ..FactionWorld::default()
        }
    }

    fn marine_rule() -> ProductionRule {
        ProductionRule {
            id: "spawn_rule.colonial_watch_fort.soldiers".into(),
            producer_archetype_id: "site_archetype.colonial.watch_fort".into(),
            actor_definition_id: "actor_def.colonial.line_marine".into(),
            actor_kind: "soldier".into(),
            costs: [
                ("resource.provisions".into(), 2),
                ("resource.iron".into(), 1),
            ]
            .into_iter()
            .collect(),
            production_ticks: 2,
            population_use: 1,
        }
    }

    #[test]
    fn named_people_return_but_remember_player_caused_deaths() {
        let mut person = NamedPerson {
            id: "person.port.quartermaster".into(),
            display_name: "Quartermaster".into(),
            alive_today: true,
            death_memory: DeathMemory {
                killed_by_player_count: 0,
                last_death_day: None,
                last_death_context_id: None,
            },
            ..Default::default()
        };
        person.record_player_caused_death(4, "encounter.port.argument");
        let mut world = WorldClock {
            day: 4,
            minute_of_day: 1439,
            named_people: [(person.id.clone(), person)].into_iter().collect(),
        };
        world.advance_to_next_midnight();
        let returned = world.named_people.get("person.port.quartermaster").unwrap();
        assert!(returned.alive_today);
        assert_eq!(returned.death_memory.killed_by_player_count, 1);
        assert_eq!(returned.death_memory.last_death_day, Some(4));
    }

    #[test]
    fn midnight_spawns_repeatable_individual_threats_for_the_new_day() {
        let mut first = WorldClock {
            day: 8,
            minute_of_day: 1439,
            named_people: BTreeMap::new(),
        };
        let mut second = first.clone();
        let rules = [SpawnRule {
            region_id: "region.reception_road".into(),
            definition_ids: vec![
                "enemy.raptor.razorbeak".into(),
                "enemy.boar.thunderback".into(),
            ],
            region_base_level: 5,
            daily_count: 3,
            pressure: 1,
        }];
        let left = first.resolve_midnight(&rules, 42);
        let right = second.resolve_midnight(&rules, 42);
        assert_eq!(left, right);
        assert!(matches!(
            left.first(),
            Some(WorldEvent::MidnightFlashStarted { day: 9 })
        ));
        assert!(matches!(
            left.last(),
            Some(WorldEvent::MidnightFlashEnded {
                day: 9,
                monster_count: 3
            })
        ));
        let monsters: Vec<_> = left
            .iter()
            .filter_map(|event| match event {
                WorldEvent::MonsterMaterialized { monster, .. } => Some(monster),
                _ => None,
            })
            .collect();
        assert_eq!(monsters.len(), 3);
        assert!(
            monsters
                .iter()
                .all(|monster| monster.behavior_tags[1] == "individual-threat")
        );
    }

    #[test]
    fn building_reserves_costs_and_produces_an_actor_with_provenance() {
        let mut world = production_world();
        let queued = world
            .enqueue_production(
                "faction.colonial_powers.prototype",
                "site.colonial.watch_fort.instance_1",
                marine_rule(),
            )
            .unwrap();
        assert!(matches!(queued, FactionWorldEvent::ProductionQueued { .. }));
        let faction = world
            .factions
            .get("faction.colonial_powers.prototype")
            .unwrap();
        assert_eq!(faction.resources["resource.provisions"], 2);
        assert_eq!(faction.resources["resource.iron"], 1);
        assert_eq!(faction.population_used, 1);

        assert!(world.advance_production_tick().is_empty());
        let events = world.advance_production_tick();
        let FactionWorldEvent::ActorProduced { actor } = &events[0] else {
            panic!("second production tick must create the actor")
        };
        assert_eq!(actor.definition_id, "actor_def.colonial.line_marine");
        assert_eq!(actor.node_id, "network_node.river_fork");
        assert_eq!(
            actor.provenance.producer_building_id,
            "site.colonial.watch_fort.instance_1"
        );
        assert_eq!(
            actor.provenance.production_rule_id,
            "spawn_rule.colonial_watch_fort.soldiers"
        );
        assert_eq!(actor.provenance.completed_tick, 2);
        assert_eq!(actor.provenance.reserved_costs["resource.provisions"], 2);
        assert_eq!(actor.current_assignment_id, None);
    }

    #[test]
    fn rejected_order_does_not_partially_spend_resources_or_population() {
        let mut world = production_world();
        let mut expensive = marine_rule();
        expensive.costs.insert("resource.iron".into(), 99);
        let before = world.clone();
        assert_eq!(
            world
                .enqueue_production(
                    "faction.colonial_powers.prototype",
                    "site.colonial.watch_fort.instance_1",
                    expensive,
                )
                .unwrap_err(),
            FactionWorldError::InsufficientResource {
                resource_id: "resource.iron".into(),
                required: 99,
                available: 2,
            }
        );
        assert_eq!(world, before);
    }

    #[test]
    fn production_is_repeatable_for_the_same_initial_state_and_commands() {
        let mut left = production_world();
        let mut right = left.clone();
        for world in [&mut left, &mut right] {
            world
                .enqueue_production(
                    "faction.colonial_powers.prototype",
                    "site.colonial.watch_fort.instance_1",
                    marine_rule(),
                )
                .unwrap();
            world.advance_production_tick();
        }
        assert_eq!(
            left.advance_production_tick(),
            right.advance_production_tick()
        );
        assert_eq!(left, right);
    }

    fn candidate(
        action_id: &str,
        target: &str,
        strategic_position: i32,
        wobble: i32,
    ) -> DispatchCandidate {
        DispatchCandidate {
            action_id: action_id.into(),
            assignment: "reinforce".into(),
            target_node_id: target.into(),
            score: DispatchScore {
                strategic_position,
                role_fitness: 10,
                ..DispatchScore::default()
            },
            wobble,
        }
    }

    fn world_with_produced_actor() -> (FactionWorld, String) {
        let mut world = production_world();
        world
            .enqueue_production(
                "faction.colonial_powers.prototype",
                "site.colonial.watch_fort.instance_1",
                marine_rule(),
            )
            .unwrap();
        world.advance_production_tick();
        world.advance_production_tick();
        let actor_id = world.actors.keys().next().unwrap().clone();
        (world, actor_id)
    }

    #[test]
    fn dispatch_chooses_highest_utility_and_queues_travel() {
        let (mut world, actor_id) = world_with_produced_actor();
        let event = world
            .dispatch_actor(
                &actor_id,
                &[
                    candidate("dispatch.colonial.raid", "network_node.smuggler_cove", 4, 1),
                    candidate(
                        "dispatch.colonial.reinforce",
                        "network_node.river_fork",
                        15,
                        -1,
                    ),
                ],
            )
            .unwrap();
        assert!(matches!(
            event,
            FactionWorldEvent::ActorAssigned { ref action_id, total_score: 24, .. }
                if action_id == "dispatch.colonial.reinforce"
        ));
        let actor = &world.actors[&actor_id];
        assert_eq!(actor.node_id, "network_node.river_fork");
        assert_eq!(
            actor.current_assignment_id.as_deref(),
            Some("dispatch.colonial.reinforce")
        );
    }

    #[test]
    fn island_travel_is_tick_owned_and_pause_freezes_everything() {
        let (mut world, actor_id) = world_with_produced_actor();
        let origin = world.positions[&actor_id];
        world
            .dispatch_actor(
                &actor_id,
                &[candidate("raid", "network_node.smuggler_cove", 10, 0)],
            )
            .unwrap();
        assert_eq!(world.positions[&actor_id], origin);
        assert_eq!(world.actors[&actor_id].node_id, "network_node.river_fork");
        world.paused = true;
        let frozen = world.clone();
        assert!(world.advance_island_tick().is_empty());
        assert!(world.advance_production_tick().is_empty());
        assert_eq!(world, frozen);
        world.paused = false;
        let first = world.advance_island_tick();
        assert!(
            first
                .iter()
                .any(|event| matches!(event, FactionWorldEvent::ActorMoved { .. }))
        );
        assert_eq!(world.positions[&actor_id], IslandPoint { x: 1, y: 1 });
        assert_eq!(world.actors[&actor_id].node_id, "network_node.river_fork");
        for _ in 0..3 {
            world.advance_island_tick();
        }
        assert_eq!(world.positions[&actor_id], IslandPoint { x: 4, y: 1 });
        assert_eq!(
            world.actors[&actor_id].node_id,
            "network_node.smuggler_cove"
        );
        assert!(!world.travel_orders.contains_key(&actor_id));
    }

    #[test]
    fn blocked_island_route_waits_then_resumes_without_teleporting() {
        let (mut world, actor_id) = world_with_produced_actor();
        world
            .dispatch_actor(
                &actor_id,
                &[candidate("raid", "network_node.smuggler_cove", 10, 0)],
            )
            .unwrap();
        for y in 0..3 {
            world.navigation.walkable.remove(&IslandPoint { x: 1, y });
        }
        let origin = world.positions[&actor_id];
        world.advance_island_tick();
        assert_eq!(world.positions[&actor_id], origin);
        assert!(world.travel_orders.contains_key(&actor_id));
        world.navigation.walkable.insert(IslandPoint { x: 1, y: 0 });
        world.advance_island_tick();
        assert_eq!(world.positions[&actor_id], IslandPoint { x: 0, y: 0 });
        for _ in 0..8 {
            world.advance_island_tick();
        }
        assert_eq!(
            world.actors[&actor_id].node_id,
            "network_node.smuggler_cove"
        );
    }

    #[test]
    fn unreachable_dispatch_does_not_replace_existing_order() {
        let (mut world, actor_id) = world_with_produced_actor();
        world
            .dispatch_actor(
                &actor_id,
                &[candidate("raid", "network_node.smuggler_cove", 10, 0)],
            )
            .unwrap();
        let before = world.clone();
        assert!(matches!(
            world.dispatch_actor(&actor_id, &[candidate("bad", "ocean", 20, 0)]),
            Err(FactionWorldError::UnreachableDestination(_))
        ));
        assert_eq!(world, before);
    }

    #[test]
    fn navigation_avoids_water_and_handles_coordinate_limits() {
        let nav = IslandNavigation {
            building_obstacles: BTreeMap::new(),
            walkable: [
                IslandPoint { x: i32::MAX, y: 0 },
                IslandPoint { x: i32::MAX, y: 1 },
            ]
            .into_iter()
            .collect(),
            destinations: BTreeMap::new(),
        };
        assert_eq!(
            nav.path(
                IslandPoint { x: i32::MAX, y: 0 },
                IslandPoint { x: i32::MAX, y: 1 }
            )
            .unwrap()
            .len(),
            2
        );
        assert!(
            nav.path(
                IslandPoint { x: 0, y: 0 },
                IslandPoint { x: i32::MAX, y: 1 }
            )
            .is_none()
        );
    }

    #[test]
    fn saved_world_resumes_pending_production_and_travel_identically() {
        let (mut original, actor_id) = world_with_produced_actor();
        original
            .enqueue_production(
                "faction.colonial_powers.prototype",
                "site.colonial.watch_fort.instance_1",
                marine_rule(),
            )
            .unwrap();
        original
            .order_move(&actor_id, IslandPoint { x: 4, y: 1 })
            .unwrap();
        original.advance_island_tick();
        let payload = original.save_json().unwrap();
        let mut restored = FactionWorld::load_json(&payload).unwrap();
        assert_eq!(original, restored);
        for _ in 0..10 {
            assert_eq!(
                original.advance_island_tick(),
                restored.advance_island_tick()
            );
            assert_eq!(original, restored);
        }
    }

    /// The proof the scenario path replaced the hard-coded island exactly:
    /// the world `prototype_island()` + `install_preview_factions()` built at
    /// the commit before they were deleted, field for field, at tick zero and
    /// after a full day. `godot-rust/tests/fixtures/` holds that world's save.

    /// The hero's identity is the pack's too. Asserting against the record the
    /// definition carries, rather than against the strings the simulation used
    /// to hard-code, is the point: the same literals would pass either way, so
    /// the test also changes the record and proves the board changes with it.
    #[test]
    fn the_captains_identity_is_the_authored_records() {
        let definition = main_scenario();
        let world = main_scenario_world();
        let record = definition
            .characters
            .iter()
            .find(|character| character.id == definition.start.captain_id)
            .expect("the pack carries the record for the captain it starts");
        let person = world.actors[&definition.start.captain_id]
            .person
            .as_ref()
            .expect("the captain is a named person");
        assert_eq!(person.display_name, record.island.board_name);
        assert_eq!(person.sex, record.island.sex);
        assert_eq!(person.age, Some(record.island.age));
        assert_eq!(person.backstory, record.island.backstory);

        // Change the authored record and the island must follow it. Without
        // this the assertions above would still pass against a Rust literal
        // that happened to match the content.
        let mut renamed = main_scenario();
        let captain_id = renamed.start.captain_id.clone();
        for character in &mut renamed.characters {
            if character.id == captain_id {
                character.island.board_name = "Someone Else Entirely".into();
                character.island.age = 41;
            }
        }
        let other = FactionWorld::from_scenario(&renamed).expect("renamed captain world");
        let renamed_person = other.actors[&captain_id].person.as_ref().unwrap();
        assert_eq!(renamed_person.display_name, "Someone Else Entirely");
        assert_eq!(renamed_person.age, Some(41));
    }

    /// A pack that starts a character whose record it does not carry is refused
    /// rather than quietly given a default name.
    #[test]
    fn a_pack_that_does_not_carry_its_captains_record_is_refused() {
        let mut definition = main_scenario();
        definition.characters.clear();
        assert_eq!(
            FactionWorld::from_scenario(&definition).unwrap_err(),
            "scenario_captain_record_missing"
        );
    }

    /// The island still starts with Michael alone. A notable is not standing on
    /// the board at tick 0; she is produced by her faction like anyone else,
    /// and arrives carrying a written identity instead of a rolled one.
    #[test]
    fn an_authored_notable_arrives_through_production_not_placement() {
        let definition = main_scenario();
        let mut world = main_scenario_world();
        let notable = world
            .rules
            .notables
            .first()
            .cloned()
            .expect("the main pack rosters a notable");
        let record = definition
            .characters
            .iter()
            .find(|character| character.id == notable.character_id)
            .expect("the pack carries the record it rosters");
        // The start-alone invariant the rest of the suite depends on.
        assert_eq!(world.actors.len(), 1);
        assert!(!world.actors.contains_key(&notable.character_id));

        let mut arrived = false;
        for _ in 0..4000 {
            world.advance_island_tick();
            if world.actors.contains_key(&notable.character_id) {
                arrived = true;
                break;
            }
        }
        assert!(arrived, "her faction must actually raise her");
        let actor = &world.actors[&notable.character_id];
        assert_eq!(actor.faction_id, notable.faction_id);
        assert_eq!(actor.definition_id, notable.definition_id);
        let person = actor.person.as_ref().expect("a notable is a person");
        assert_eq!(person.display_name, record.island.board_name);
        assert_eq!(person.sex, record.island.sex);
        assert_eq!(person.age, Some(record.island.age));
        assert_eq!(person.recruitment_offer, record.island.recruitment_offer);
        // She came out of a real production rule, with a real producing
        // building behind her -- not a hand-placed provenance.
        assert!(!actor.provenance.producer_building_id.is_empty());
        assert_eq!(actor.provenance.faction_id, notable.faction_id);
        // Her combat state is the definition's, the same one every produced
        // deckhand gets. She is not stronger for being written.
        assert_eq!(
            world.unit_combat[&notable.character_id].health,
            world.combat_profiles[&notable.definition_id].health
        );
        // One identity, one incarnation, however long the island runs. She may
        // well be killed out there -- she is an ordinary deckhand in a real war
        // -- but death moves her to the casualty record rather than erasing
        // her, and a dead notable is never quietly produced a second time.
        for _ in 0..600 {
            world.advance_island_tick();
            let living = world.actors.contains_key(&notable.character_id);
            let dead = world.casualties.contains_key(&notable.character_id);
            assert!(
                living != dead,
                "a notable is either standing or recorded dead, never both and never neither"
            );
        }
    }

    /// The point of the whole contract: a written woman is recruited by exactly
    /// the verbs that recruit a rolled one. If this needed a Betty-shaped
    /// branch anywhere, the authority's "no separate protected heroine caste"
    /// would already be broken.
    #[test]
    fn an_authored_notable_is_recruited_by_the_same_verbs_as_a_generated_woman() {
        let mut world = main_scenario_world();
        let betty = "character.heroine.betty";
        world.hostilities.clear();
        // Wait for the pirates to raise her, exactly as a player would.
        let mut raised = false;
        for _ in 0..4000 {
            world.advance_island_tick();
            if world.actors.contains_key(betty) {
                raised = true;
                break;
            }
        }
        assert!(raised, "the pirates must raise Betty");
        assert_ne!(world.actors[betty].faction_id, "faction.michael");
        assert!(
            !world.actors[betty]
                .person
                .as_ref()
                .unwrap()
                .loyal_to_michael
        );
        // Walk Michael to her rather than teleporting either of them.
        let captain = "character.protagonist.captain";
        let her_ground = world.positions[betty];
        world.order_move(captain, her_ground).unwrap();
        let mut reached = false;
        for _ in 0..4000 {
            world.advance_island_tick();
            if world.approach_island_person(betty) {
                reached = true;
                break;
            }
        }
        assert!(reached, "Michael must be able to reach an authored notable");
        for _ in 0..256 {
            if world.can_talk_island_person(betty) {
                break;
            }
            world.advance_island_tick();
        }
        assert!(world.can_talk_island_person(betty));
        world.talk_island_person(betty);
        assert!(world.recruit_island_person(betty));
        assert!(
            world.actors[betty]
                .person
                .as_ref()
                .unwrap()
                .loyal_to_michael
        );
        assert_eq!(world.actors[betty].faction_id, "faction.michael");
        // Recruitment transfers the same person; it does not clone her, and it
        // does not rewrite where she came from.
        assert_eq!(
            world.actors[betty].provenance.faction_id,
            "faction.pirates.prototype"
        );
        assert_eq!(
            world
                .actors
                .values()
                .filter(|actor| actor.person.as_ref().is_some_and(|p| p.id == betty))
                .count(),
            1
        );
        assert!(world.assign_island_companion(betty, 0));
        assert_eq!(world.party_size(), 1);
    }

    /// The economy is the pack's, not the simulation's: every opening number
    /// comes from `factions[].economy`, and nothing is inferred from costs.
    #[test]
    fn faction_economies_are_authored_by_the_pack() {
        let definition = main_scenario();
        let world = main_scenario_world();
        for entry in &definition.factions {
            let faction = &world.factions[&entry.id];
            let policy = &world.policies[&entry.id];
            assert_eq!(faction.resources, entry.economy.stockpile);
            assert_eq!(policy.income_per_tick, entry.economy.income_per_tick);
            assert_eq!(policy.storage_caps, entry.economy.storage_caps);
            assert!(!entry.economy.stockpile.is_empty());
        }
        // The captain's own faction opens on `start.economy`, which the main
        // pack authors empty because Michael reaches the island with nothing.
        assert!(definition.start.economy.stockpile.is_empty());
        assert!(world.factions["faction.michael"].resources.is_empty());
        assert!(!world.policies.contains_key("faction.michael"));
    }

    /// A captain who starts with supplies is expressible: `start.economy` is
    /// the hero faction's authored opening, in a faction economy's own shape.
    #[test]
    fn the_captain_can_start_with_an_authored_stockpile() {
        let mut definition = main_scenario();
        let salvage = definition.rules.salvage.resource_id.clone();
        definition.start.economy = ScenarioFactionEconomy {
            stockpile: [(salvage.clone(), 6)].into(),
            income_per_tick: [(salvage.clone(), 1)].into(),
            storage_caps: [(salvage.clone(), 8)].into(),
        };
        let world = FactionWorld::from_scenario(&definition).expect("authored captain economy");
        assert_eq!(world.stored_resource("faction.michael", &salvage), 6);
        assert_eq!(world.policies["faction.michael"].storage_caps[&salvage], 8);
        // An opening stockpile above its own cap is a pack error, not a clamp.
        definition.start.economy.stockpile.insert(salvage, 99);
        assert_eq!(
            FactionWorld::from_scenario(&definition).unwrap_err(),
            "invalid_scenario_economy"
        );
    }

    /// An undeclared key is not a resource. A gain refuses it rather than
    /// creating it, which is what `entry(..).or_default()` used to do.
    #[test]
    fn an_uncatalogued_resource_is_refused_rather_than_created() {
        let mut world = main_scenario_world();
        let faction = "faction.pirates.prototype";
        assert!(world.rules.knows("resource.coin"));
        assert!(!world.rules.knows("resource.gunpowder"));
        assert_eq!(
            world.gain_resource(faction, "resource.gunpowder", 5),
            Err("unknown_resource:resource.gunpowder".into())
        );
        assert_eq!(
            world.spend_resources(faction, &[("resource.gunpowder".to_owned(), 1)].into()),
            Err("unknown_resource:resource.gunpowder".into())
        );
        assert!(
            !world.factions[faction]
                .resources
                .contains_key("resource.gunpowder")
        );
        // A production rule naming one is refused by name, not by underflow.
        let mut rule = scenario_production(faction);
        rule.costs = [("resource.gunpowder".to_owned(), 1)].into();
        let building_id = world.factions[faction]
            .buildings
            .keys()
            .next()
            .unwrap()
            .clone();
        assert_eq!(
            world.enqueue_production(faction, &building_id, rule),
            Err(FactionWorldError::UnknownResource(
                "resource.gunpowder".into()
            ))
        );
    }

    /// Storage caps bind every gain, not only income: a salvage pickup used to
    /// walk straight past them.
    #[test]
    fn every_gain_is_clamped_to_the_storage_cap() {
        let mut world = main_scenario_world();
        let salvage = world.salvage_resource().to_owned();
        let michael = "faction.michael";
        world
            .set_policy(
                michael,
                FactionPolicy {
                    storage_caps: [(salvage.clone(), 9)].into(),
                    ..Default::default()
                },
            )
            .expect("a cap on Michael's salvage");
        world.gain_resource(michael, &salvage, 4).unwrap();
        assert_eq!(world.stored_resource(michael, &salvage), 4);
        world.gain_resource(michael, &salvage, 100).unwrap();
        assert_eq!(world.stored_resource(michael, &salvage), 9);
        // A cache pickup is a gain like any other and obeys the same cap.
        let cache = world.salvage_caches.keys().next().unwrap().clone();
        world.salvage_caches.get_mut(&cache).unwrap().remaining = 50;
        let position = world.salvage_caches[&cache].position;
        world
            .positions
            .insert("character.protagonist.captain".into(), position);
        world.salvage_foothold().expect("collect the cache");
        assert_eq!(world.stored_resource(michael, &salvage), 9);
    }

    /// The salvage key is the pack's, read from `rules.holdingSalvage`, so no
    /// stockpile in the simulation is addressed by a compiled-in literal.
    #[test]
    fn the_salvage_resource_is_named_by_the_pack() {
        let world = main_scenario_world();
        assert_eq!(world.salvage_resource(), "resource.salvage");
        assert!(world.rules.resources.contains_key(world.salvage_resource()));
        // A pack whose salvage key is not in its catalogue is refused.
        let mut definition = main_scenario();
        definition.rules.salvage.resource_id = "resource.driftwood".into();
        assert_eq!(
            FactionWorld::from_scenario(&definition).unwrap_err(),
            "invalid_scenario_rules"
        );
    }

    /// The catalogue is data, and untrusted data is bounded and checked like
    /// every other collection the loader admits.
    #[test]
    fn a_save_may_not_smuggle_in_an_uncatalogued_resource() {
        let world = main_scenario_world();
        let save = world.save_json().unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&save).unwrap();
        value["world"]["factions"]["faction.pirates.prototype"]["resources"]["resource.gunpowder"] =
            serde_json::json!(7);
        assert_eq!(
            FactionWorld::load_json(&serde_json::to_string(&value).unwrap()).unwrap_err(),
            "invalid_saved_resources"
        );
        let mut value: serde_json::Value = serde_json::from_str(&save).unwrap();
        value["world"]["rules"]["resources"]["resource.gunpowder"] =
            serde_json::json!({"displayName": ""});
        assert_eq!(
            FactionWorld::load_json(&serde_json::to_string(&value).unwrap()).unwrap_err(),
            "invalid_saved_resources"
        );
        let mut value: serde_json::Value = serde_json::from_str(&save).unwrap();
        let catalogue = value["world"]["rules"]["resources"]
            .as_object_mut()
            .unwrap();
        for index in 0..=MAXIMUM_RESOURCES {
            catalogue.insert(
                format!("resource.filler_{index}"),
                serde_json::json!({"displayName": "Filler"}),
            );
        }
        assert_eq!(
            FactionWorld::load_json(&serde_json::to_string(&value).unwrap()).unwrap_err(),
            "invalid_saved_resources"
        );
    }

    /// The catalogue travels with the world, so a workshop begun in one session
    /// survives the save round trip `begin_building_construction` performs.
    #[test]
    fn the_catalogue_survives_the_construction_save_round_trip() {
        let mut world = main_scenario_world();
        let salvage = world.salvage_resource().to_owned();
        let before = world.rules.resources.clone();
        world
            .gain_resource("faction.michael", &salvage, 40)
            .unwrap();
        // The reviewed workshop cell, as every other foothold proof uses it.
        let entrance = IslandPoint { x: 19, y: 17 };
        world
            .positions
            .insert("character.protagonist.captain".into(), entrance);
        world.build_foothold(entrance).expect("workshop begun");
        assert_eq!(world.rules.resources, before);
        let job = &world.factions["faction.michael"].buildings["site.michael.field_workshop"]
            .construction
            .as_ref()
            .expect("construction job")
            .reserved_costs;
        assert_eq!(job.keys().collect::<Vec<_>>(), vec![&salvage]);
    }

    /// A version-1 save predates the world carrying its rules. It resumes into
    /// the scenario it is loaded for, and then plays the same day identically.

    /// The main scenario's clock is the authored record, not a Rust default.
    #[test]
    fn the_scenario_document_supplies_the_campaign_clock() {
        let clock = main_scenario_world().rules.campaign_clock;
        assert_eq!(clock.world_deadline_day, 100);
        assert_eq!(clock.terminal_severity, 120);
        assert_eq!(
            clock.priority,
            vec![
                ConfrontationCause::DeliberateDiscovery,
                ConfrontationCause::TerminalHeat,
                ConfrontationCause::Day100,
            ]
        );
        assert_eq!(
            clock.deliberate_discovery_faction,
            "faction.cthulhu.prototype"
        );
        assert!(clock.signal_channels.contains(&"dreams".to_string()));
        assert_eq!(clock.sources.len(), 3);
    }

    /// A rule set with no campaign clock is refused rather than played with no
    /// deadline, which is what a version-2 save written before this contract is.
    #[test]
    fn a_world_without_a_campaign_clock_is_refused() {
        let mut world = main_scenario_world();
        world.rules.campaign_clock = CampaignClock::default();
        let payload = world.save_json().unwrap();
        assert_eq!(
            FactionWorld::load_json(&payload).unwrap_err(),
            "invalid_saved_scenario_rules"
        );
    }

    /// Heat is written by the island's own occurrences, not by a verb Godot
    /// calls: a day of play leaves a ledger, on declared channels only, and
    /// the same day played twice leaves the same ledger.
    #[test]
    fn heat_accrues_from_the_islands_own_occurrences() {
        let mut world = main_scenario_world();
        assert!(world.heat.is_empty());
        for _ in 0..1440 {
            world.advance_island_tick();
        }
        assert!(
            !world.heat.is_empty(),
            "a day of the main scenario must signal at least once"
        );
        let channels = &world.rules.campaign_clock.signal_channels;
        let mut seen = BTreeSet::new();
        let mut previous = 0;
        for event in &world.heat {
            assert!(channels.contains(&event.signal), "{}", event.signal);
            assert!(seen.insert(event.id.clone()), "{}", event.id);
            assert!(event.tick >= previous);
            previous = event.tick;
        }
        assert!(world.heat_severity() > 0);
        assert!(
            world
                .heat_channels()
                .is_subset(&channels.iter().cloned().collect())
        );

        let mut twin = main_scenario_world();
        for _ in 0..1440 {
            twin.advance_island_tick();
        }
        assert_eq!(world.heat, twin.heat);
    }

    /// The ledger is append-only and closed: an undeclared channel, a repeated
    /// ID and an out-of-range severity are each refused without writing.
    #[test]
    fn the_heat_ledger_refuses_what_the_clock_does_not_declare() {
        let mut world = main_scenario_world();
        assert!(world.record_heat_event("heat.test:1", "dreams", 3));
        assert_eq!(world.heat.len(), 1);
        assert!(!world.record_heat_event("heat.test:1", "dreams", 3));
        assert!(!world.record_heat_event("heat.test:2", "not_a_channel", 3));
        assert!(!world.record_heat_event("heat.test:3", "dreams", 0));
        assert!(!world.record_heat_event("heat.test:4", "dreams", 101));
        assert!(!world.record_heat_event("", "dreams", 3));
        assert_eq!(world.heat.len(), 1);
        assert_eq!(world.heat_severity(), 3);
    }

    /// Day 100 is a real deadline: it records a confrontation, and once one is
    /// recorded nothing rewrites it, however many later causes come true.
    #[test]
    fn the_deadline_records_a_confrontation_that_is_never_rewritten() {
        let mut world = main_scenario_world();
        world.tick = world.clock.ticks_per_day * 98;
        world.advance_island_tick();
        assert_eq!(world.day(), 99);
        assert!(world.confrontation.is_none());
        world.tick = world.clock.ticks_per_day * 99;
        world.advance_island_tick();
        let recorded = world.confrontation.expect("day 100 must confront");
        assert_eq!(recorded.cause, ConfrontationCause::Day100);
        assert_eq!(recorded.day, 100);
        // A later, higher-priority cause does not displace the recorded one.
        world
            .eliminated_factions
            .insert("faction.cthulhu.prototype".into());
        world.advance_island_tick();
        assert_eq!(world.confrontation, Some(recorded));
    }

    /// When more than one cause is true in the same tick, the authored order
    /// decides, so the recorded cause is never a matter of evaluation order.
    #[test]
    fn causes_landing_together_are_settled_by_the_authored_priority() {
        let mut world = main_scenario_world();
        world.tick = world.clock.ticks_per_day * 99;
        let mut index = 0;
        while world.heat_severity() < world.rules.campaign_clock.terminal_severity {
            assert!(world.record_heat_event(&format!("heat.test:{index}"), "dreams", 3));
            index += 1;
        }
        world
            .eliminated_factions
            .insert("faction.cthulhu.prototype".into());
        world.advance_campaign_clock(&[]);
        assert_eq!(
            world.confrontation.map(|record| record.cause),
            Some(ConfrontationCause::DeliberateDiscovery)
        );

        // With deliberate discovery absent, terminal heat outranks the deadline.
        let mut heated = main_scenario_world();
        heated.tick = heated.clock.ticks_per_day * 99;
        let mut index = 0;
        while heated.heat_severity() < heated.rules.campaign_clock.terminal_severity {
            assert!(heated.record_heat_event(&format!("heat.test:{index}"), "dreams", 3));
            index += 1;
        }
        heated.advance_campaign_clock(&[]);
        assert_eq!(
            heated.confrontation.map(|record| record.cause),
            Some(ConfrontationCause::TerminalHeat)
        );
    }

    /// The ledger and the recorded cause are play state, so they survive a
    /// save and a load exactly.
    #[test]
    fn the_ledger_and_the_confrontation_survive_a_save_round_trip() {
        let mut world = main_scenario_world();
        world.tick = world.clock.ticks_per_day * 99;
        assert!(world.record_heat_event("heat.test:1", "npc_behavior", 5));
        world.advance_campaign_clock(&[]);
        assert!(world.confrontation.is_some());
        let restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(restored.heat, world.heat);
        assert_eq!(restored.confrontation, world.confrontation);
        assert_eq!(restored, world);
    }

    /// A save is untrusted input, so a tampered ledger or a confrontation from
    /// the future is refused by name rather than loaded.
    #[test]
    fn a_tampered_ledger_or_confrontation_is_refused_by_name() {
        fn refusal(mutate: impl FnOnce(&mut FactionWorld)) -> String {
            let mut world = main_scenario_world();
            world.tick = 10;
            mutate(&mut world);
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap_err()
        }
        let entry = |id: &str, signal: &str, tick: u64| HeatEvent {
            id: id.into(),
            signal: signal.into(),
            severity: 3,
            tick,
        };
        assert_eq!(
            refusal(|world| world.heat.push(entry("heat.a", "not_a_channel", 1))),
            "invalid_saved_heat_ledger"
        );
        assert_eq!(
            refusal(|world| world.heat.push(entry("heat.a", "dreams", 99))),
            "invalid_saved_heat_ledger"
        );
        assert_eq!(
            refusal(|world| {
                world.heat.push(entry("heat.a", "dreams", 1));
                world.heat.push(entry("heat.a", "dreams", 2));
            }),
            "invalid_saved_heat_ledger"
        );
        assert_eq!(
            refusal(|world| {
                world.heat.push(entry("heat.a", "dreams", 5));
                world.heat.push(entry("heat.b", "dreams", 1));
            }),
            "invalid_saved_heat_ledger"
        );
        assert_eq!(
            refusal(|world| {
                world.heat = (0..=HEAT_LEDGER_CAP)
                    .map(|index| entry(&format!("heat.{index}"), "dreams", 1))
                    .collect();
            }),
            "invalid_saved_heat_ledger"
        );
        assert_eq!(
            refusal(|world| {
                world.confrontation = Some(Confrontation {
                    cause: ConfrontationCause::Day100,
                    tick: 99,
                    day: 1,
                });
            }),
            "invalid_saved_confrontation"
        );
    }

    /// The main scenario's own three triggers are the authored record, not a
    /// Rust default: they resolve with real conditions and real effects.
    #[test]
    fn the_scenario_document_supplies_its_triggers() {
        let triggers = main_scenario_world().rules.triggers;
        assert_eq!(triggers.len(), 8);
        assert!(
            triggers
                .iter()
                .any(|trigger| trigger.id == "trigger.cthulhu.shrine_grown")
        );
    }

    /// A day condition fires a trigger once the world reaches it, its effects
    /// land in the same tick, and a non-repeating trigger never fires twice.
    #[test]
    fn a_trigger_fires_once_its_condition_holds_and_never_again() {
        let mut world = main_scenario_world();
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.day_five".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 5 }],
            then: vec![TriggerEffect::SetFlag {
                flag: "flag.test.day_five".into(),
            }],
        }];
        world.tick = world.clock.ticks_per_day * 3;
        world.advance_island_tick();
        assert!(!world.flags.contains("flag.test.day_five"));
        assert!(!world.fired_triggers.contains_key("trigger.test.day_five"));
        world.tick = world.clock.ticks_per_day * 4;
        world.advance_island_tick();
        assert!(world.flags.contains("flag.test.day_five"));
        let fired_tick = *world
            .fired_triggers
            .get("trigger.test.day_five")
            .expect("recorded firing");
        assert_eq!(fired_tick, world.tick);
        // The world keeps advancing; a non-repeating trigger does not re-fire,
        // so its recorded tick never moves.
        world.advance_island_tick();
        world.advance_island_tick();
        assert_eq!(
            world.fired_triggers.get("trigger.test.day_five"),
            Some(&fired_tick)
        );
    }

    /// `repeat: true` fires again on every tick the condition holds, and each
    /// firing is still visible in `fired_triggers` at its own most recent tick.
    #[test]
    fn a_repeating_trigger_fires_on_every_qualifying_tick() {
        let mut world = main_scenario_world();
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.repeating".into(),
            repeat: true,
            when: vec![TriggerCondition::FlagSet {
                flag: "flag.test.armed".into(),
            }],
            then: vec![TriggerEffect::RecordHeat {
                id: "heat.test.tick".into(),
                signal: "dreams".into(),
                severity: 1,
            }],
        }];
        world.flags.insert("flag.test.armed".into());
        // The effect is not repeatable (a duplicate heat ID is refused), so the
        // trigger becomes inapplicable and stays pending after its one firing --
        // proving repeat controls re-evaluation, not that an effect must repeat.
        world.advance_island_tick();
        assert_eq!(world.heat.len(), 1);
        let first_tick = world.tick;
        world.advance_island_tick();
        assert_eq!(world.heat.len(), 1);
        assert_eq!(
            world.fired_triggers.get("trigger.test.repeating"),
            Some(&first_tick)
        );
    }

    /// A trigger is a transaction: if any effect could not apply, none of them
    /// do, and the trigger is left pending rather than half-fired.
    #[test]
    fn a_trigger_with_an_inapplicable_effect_changes_nothing() {
        let mut world = main_scenario_world();
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.transaction".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![
                TriggerEffect::SetFlag {
                    flag: "flag.test.would_have_set".into(),
                },
                TriggerEffect::GrantResource {
                    faction_id: "faction.does_not_exist".into(),
                    resource_id: "resource.iron".into(),
                    amount: 1,
                },
            ],
        }];
        world.advance_island_tick();
        assert!(!world.flags.contains("flag.test.would_have_set"));
        assert!(
            !world
                .fired_triggers
                .contains_key("trigger.test.transaction")
        );
    }

    /// `grant_item` reaches the world for the first time through a trigger.
    #[test]
    fn a_trigger_grants_the_captains_signature_weapon() {
        const CAPTAIN: &str = "character.protagonist.captain";
        const WEAPON: &str = "weapon.captain.handsome_jack_steam_carbine";
        let mut world = main_scenario_world();
        assert!(world.actor_inventory(CAPTAIN).is_empty());
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.arm_the_captain".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::GrantItem {
                actor_id: CAPTAIN.into(),
                item_id: WEAPON.into(),
            }],
        }];
        world.advance_island_tick();
        assert_eq!(world.actor_inventory(CAPTAIN), vec![WEAPON.to_string()]);
    }

    /// `set_hostility(true)` ends any survival truce between the two factions,
    /// so the load-time invariant that a truce pair is never hostile still holds.
    #[test]
    fn set_hostility_true_ends_a_survival_truce_between_the_pair() {
        let mut world = main_scenario_world();
        // The main scenario starts colonial and pirates already at war
        // (`content/diplomacy/initial_relationships.prototype.json`). A real
        // survival truce suspends that hostility for as long as it holds, so
        // the fixture models the same suspended state: hostilities removed,
        // a truce recorded to resume them once it ends.
        let (a, b) = (
            "faction.colonial_powers.prototype".to_string(),
            "faction.pirates.prototype".to_string(),
        );
        assert!(world.hostilities.contains(&(a.clone(), b.clone())));
        world.hostilities.remove(&(a.clone(), b.clone()));
        world.hostilities.remove(&(b.clone(), a.clone()));
        world.survival_truces.push(SurvivalTruce {
            a: a.clone(),
            b: b.clone(),
            threat: "faction.cthulhu.prototype".into(),
            expires_tick: world.tick + world.rules.survival.truce_ticks,
            resume_ab: true,
            resume_ba: true,
        });
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.betrayal".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::SetHostility {
                a: a.clone(),
                b: b.clone(),
                hostile: true,
            }],
        }];
        world.advance_island_tick();
        assert!(world.hostilities.contains(&(a.clone(), b.clone())));
        assert!(world.hostilities.contains(&(b.clone(), a.clone())));
        assert!(
            world
                .survival_truces
                .iter()
                .all(|truce| !(truce.a == a && truce.b == b))
        );
        // The save loader's own truce invariant still holds on this world.
        let restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(restored, world);
    }

    /// Flags, the firing record and their effects survive a save round trip.
    #[test]
    fn triggers_and_flags_survive_a_save_round_trip() {
        let mut world = main_scenario_world();
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.persisted".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::SetFlag {
                flag: "flag.test.persisted".into(),
            }],
        }];
        world.advance_island_tick();
        assert!(world.flags.contains("flag.test.persisted"));
        let restored =
            FactionWorld::load_json_for_scenario(&world.save_json().unwrap(), &world.rules.clone())
                .unwrap();
        assert_eq!(restored.flags, world.flags);
        assert_eq!(restored.fired_triggers, world.fired_triggers);
        assert_eq!(restored, world);
    }

    /// A save is untrusted input: a flag or a firing record naming a trigger
    /// this rule set no longer declares is refused by name, not silently kept.
    #[test]
    fn a_tampered_trigger_history_is_refused_by_name() {
        fn refusal(mutate: impl FnOnce(&mut FactionWorld)) -> String {
            let mut world = main_scenario_world();
            mutate(&mut world);
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap_err()
        }
        assert_eq!(
            refusal(|world| {
                world
                    .fired_triggers
                    .insert("trigger.does_not_exist".into(), 0);
            }),
            "invalid_saved_trigger_history"
        );
        assert_eq!(
            refusal(|world| {
                let id = world.rules.triggers[0].id.clone();
                world.fired_triggers.insert(id, world.tick + 1);
            }),
            "invalid_saved_trigger_history"
        );
        assert_eq!(
            refusal(|world| {
                world.flags.insert(String::new());
            }),
            "invalid_saved_trigger_history"
        );
    }

    /// The main scenario's own quest is the authored record, not a Rust
    /// default: it resolves with real stages and a real terminal outcome.
    #[test]
    fn the_scenario_document_supplies_its_quest() {
        let quests = main_scenario_world().rules.quests;
        let quest = quests
            .get("quest.the_cult_beneath_the_water")
            .expect("authored quest");
        assert_eq!(quest.initial_stage, "stage.rumors");
        assert_eq!(quest.stages.len(), 3);
        assert_eq!(
            quest.stages["stage.colonial_response"].terminal,
            Some(QuestOutcome::Success)
        );
    }

    /// Every declared quest starts active on its own authored stage the
    /// moment the world is created from a scenario -- no separate start verb.
    #[test]
    fn a_declared_quest_starts_on_its_initial_stage() {
        let world = main_scenario_world();
        for (id, quest) in &world.rules.quests {
            assert_eq!(world.quest_stages.get(id), Some(&quest.initial_stage));
        }
    }

    /// `LoyalCompanionCount` and the trigger that reads it, exercised through
    /// the real production, approach, talk and recruit path -- not a
    /// synthetic loyal_to_michael flip -- because a mechanism proven only
    /// against a hand-built fixture is exactly what went wrong with the
    /// island network this session: real code serving nothing real.
    #[test]
    fn recruiting_a_real_produced_woman_fires_the_not_alone_trigger() {
        let mut world = main_scenario_world();
        // Peace isolates the authored production and voluntary recruitment
        // loop; no actor, identity, population or position is fabricated.
        world.hostilities.clear();
        assert_eq!(world.loyal_companion_count(), 0);
        let mut recruited_id = None;
        'outer: for _ in 0..2000 {
            world.advance_island_tick();
            let candidates: Vec<String> = world
                .actors
                .iter()
                .filter(|(_, actor)| {
                    actor.faction_id != "faction.michael"
                        && actor.person.as_ref().is_some_and(|person| {
                            person.sex == PersonSex::Female
                                && person.age.is_some_and(|age| age >= 18)
                        })
                })
                .map(|(id, _)| id.clone())
                .collect();
            for id in candidates {
                if !world.approach_island_person(&id) {
                    continue;
                }
                for _ in 0..128 {
                    if world.can_talk_island_person(&id) {
                        break;
                    }
                    world.advance_island_tick();
                }
                if world.can_talk_island_person(&id) {
                    world.talk_island_person(&id);
                    if world.recruit_island_person(&id) {
                        recruited_id = Some(id);
                        break 'outer;
                    }
                }
            }
        }
        let recruited_id =
            recruited_id.expect("a producible woman must be recruitable within the tick bound");
        assert!(
            world.actors[&recruited_id]
                .person
                .as_ref()
                .unwrap()
                .loyal_to_michael
        );
        assert_eq!(world.loyal_companion_count(), 1);
        world.advance_island_tick();
        assert!(
            world
                .fired_triggers
                .contains_key("trigger.michael.first_companion")
        );
        assert_eq!(
            world.quest_stages.get("quest.not_alone_anymore"),
            Some(&"stage.supported".to_string())
        );
        assert_eq!(
            world.stored_resource("faction.michael", "resource.provisions"),
            2
        );
        assert!(world.flags.contains("flag.michael.not_alone"));
    }

    /// `PartySize` and the trigger that reads it, exercised the same way:
    /// through real production and recruitment for four separate women,
    /// each actually assigned to a party slot, not four flags flipped by
    /// hand.
    #[test]
    fn filling_all_four_party_slots_fires_the_household_trigger() {
        let mut world = main_scenario_world();
        world.hostilities.clear();
        assert_eq!(world.party_size(), 0);
        let mut recruited: Vec<String> = Vec::new();
        for _ in 0..8000 {
            if recruited.len() >= 4 {
                break;
            }
            world.advance_island_tick();
            let candidates: Vec<String> = world
                .actors
                .iter()
                .filter(|(id, actor)| {
                    !recruited.contains(id)
                        && actor.faction_id != "faction.michael"
                        && actor.person.as_ref().is_some_and(|person| {
                            person.sex == PersonSex::Female
                                && person.age.is_some_and(|age| age >= 18)
                        })
                })
                .map(|(id, _)| id.clone())
                .collect();
            for id in candidates {
                if recruited.len() >= 4 {
                    break;
                }
                if !world.approach_island_person(&id) {
                    continue;
                }
                for _ in 0..128 {
                    if world.can_talk_island_person(&id) {
                        break;
                    }
                    world.advance_island_tick();
                }
                if world.can_talk_island_person(&id) {
                    world.talk_island_person(&id);
                    if world.recruit_island_person(&id) {
                        assert!(world.assign_island_companion(&id, recruited.len()));
                        recruited.push(id);
                    }
                }
            }
        }
        assert_eq!(
            recruited.len(),
            4,
            "four producible women must be recruitable and assignable within the tick bound"
        );
        assert_eq!(world.party_size(), 4);
        world.advance_island_tick();
        assert!(
            world
                .fired_triggers
                .contains_key("trigger.michael.full_household")
        );
        assert_eq!(
            world.quest_stages.get("quest.the_household_forms"),
            Some(&"stage.complete".to_string())
        );
        assert!(world.flags.contains("flag.michael.household_complete"));
    }

    /// Michael starts at `island.contested_clearing` and the cult's ground is
    /// somewhere he has to actually walk to, so this is proved by ordering the
    /// real move and letting the island's own pathing carry him there, not by
    /// writing a position into the map.
    #[test]
    fn walking_onto_cult_ground_fires_the_shrine_trigger() {
        let mut world = main_scenario_world();
        let captain = "character.protagonist.captain";
        // The walk is the subject here, not the war it crosses.
        world.hostilities.clear();
        let shrine = world.navigation.destinations["preview.faction.cthulhu.prototype.holding"];
        assert_ne!(world.positions[captain], shrine);
        assert!(!world.flags.contains("flag.michael.saw_the_shrine"));
        world.order_move(captain, shrine).unwrap();
        let mut arrived = false;
        for _ in 0..2000 {
            world.advance_island_tick();
            if world.positions[captain] == shrine {
                arrived = true;
                break;
            }
        }
        assert!(arrived, "the captain must be able to walk to the shrine");
        world.advance_island_tick();
        assert!(
            world
                .fired_triggers
                .contains_key("trigger.michael.stands_on_cult_ground")
        );
        assert_eq!(
            world.quest_stages.get("quest.what_he_saw_out_there"),
            Some(&"stage.witnessed".to_string())
        );
        assert!(world.flags.contains("flag.michael.saw_the_shrine"));
    }

    /// Stripping the wreck is Michael's own verb, not something the island does
    /// for him: `salvage_foothold` is the same call the scene's salvage button
    /// makes, and `resource_at_least` reads the stockpile it credits.
    #[test]
    fn stripping_the_wreck_fires_the_wreck_stripped_trigger() {
        let mut world = main_scenario_world();
        let captain = "character.protagonist.captain";
        assert_eq!(
            world.stored_resource("faction.michael", "resource.salvage"),
            0
        );
        world.positions.insert(
            captain.into(),
            world.salvage_caches["salvage.wreck"].position,
        );
        world.salvage_foothold().unwrap();
        assert_eq!(
            world.stored_resource("faction.michael", "resource.salvage"),
            20
        );
        assert!(!world.flags.contains("flag.michael.wreck_stripped"));
        world.advance_island_tick();
        assert!(
            world
                .fired_triggers
                .contains_key("trigger.michael.wreck_stripped")
        );
        assert_eq!(
            world.quest_stages.get("quest.what_the_sea_gave_back"),
            Some(&"stage.stripped".to_string())
        );
        assert!(world.flags.contains("flag.michael.wreck_stripped"));
    }

    /// Eliminating a rival faction is real simulation behavior, driven through
    /// the same siege path `siege_destroys_last_producer_and_elimination_survives_load`
    /// proves, not a synthetic `eliminated_factions.insert()`. The trigger
    /// reads `eliminated_factions`, the same set that path populates.
    #[test]
    fn a_rival_falling_fires_the_a_rival_falls_trigger() {
        let mut world = main_scenario_world();
        advance_until_raised(&mut world, &["actor_def.pirates.deckhand"]);
        let pirate = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        world.actors.retain(|id, _| id == &pirate);
        world.positions.retain(|id, _| id == &pirate);
        world.unit_combat.retain(|id, _| id == &pirate);
        world.travel_orders.clear();
        world.hostilities = [(
            "faction.pirates.prototype".into(),
            "faction.colonial_powers.prototype".into(),
        )]
        .into_iter()
        .collect();
        let fort = world.factions["faction.colonial_powers.prototype"]
            .buildings
            .values()
            .next()
            .unwrap();
        let entrance = world.navigation.destinations[&fort.node_id];
        world.positions.insert(pirate.clone(), entrance);
        let onward = world.navigation.destinations["island.contested_clearing"];
        world.order_move(&pirate, onward).unwrap();
        for faction in world.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                building.operational = false;
            }
        }
        // As above: one raider stands in for the army the trigger is about.
        for building in world
            .factions
            .get_mut("faction.colonial_powers.prototype")
            .unwrap()
            .buildings
            .values_mut()
        {
            building.health = 8;
        }
        let mut saw_elimination = false;
        for _ in 0..40_000 {
            saw_elimination |= world.advance_island_tick().iter().any(|e| {
                matches!(e, FactionWorldEvent::FactionEliminated { faction_id } if faction_id == "faction.colonial_powers.prototype")
            });
            if saw_elimination {
                break;
            }
        }
        assert!(saw_elimination);
        assert!(
            world
                .eliminated_factions
                .contains("faction.colonial_powers.prototype")
        );
        world.advance_island_tick();
        assert!(
            world
                .fired_triggers
                .contains_key("trigger.michael.a_rival_falls")
        );
        assert_eq!(
            world.quest_stages.get("quest.the_island_grows_quiet"),
            Some(&"stage.first_fall".to_string())
        );
        assert!(world.flags.contains("flag.michael.a_rival_falls"));
        assert_eq!(
            world.stored_resource("faction.michael", "resource.provisions"),
            3
        );
    }

    /// The product's core loop, end to end and through real verbs: the island
    /// runs, her faction raises her, Michael recruits her, the cult's shrine
    /// actually grows, she brings him what she noticed, he backs one of her two
    /// readings, and the board changes because he did.
    #[test]
    fn a_companion_raises_a_lead_and_the_players_answer_changes_the_board() {
        let mut world = main_scenario_world();
        let neriah = "character.heroine.neriah";
        let lead = "lead.neriah.the_water_turns";
        // Peace isolates the lead loop from the war. Diplomacy re-arms
        // hostilities on its own, and an elven warden standing in an active
        // front line is quite likely to be killed before Michael ever reaches
        // her -- which is true to the game and useless for testing this.
        macro_rules! peaceful_tick {
            () => {{
                world.hostilities.clear();
                world.advance_island_tick();
            }};
        }
        assert!(world.rules.leads.iter().any(|l| l.id == lead));
        assert!(world.open_leads.is_empty());

        // She has to exist before she can notice anything.
        let mut raised = false;
        for _ in 0..6000 {
            peaceful_tick!();
            if world.actors.contains_key(neriah) {
                raised = true;
                break;
            }
        }
        assert!(raised, "the elves must raise Neriah");

        // Her lead cannot open while she is still an elf: the judgment is the
        // thing Michael recruited her for.
        for _ in 0..200 {
            peaceful_tick!();
            assert!(
                world.actors.contains_key(neriah),
                "peace must keep her alive long enough to be recruited"
            );
            assert!(
                world.open_leads.is_empty(),
                "an unrecruited woman brings Michael nothing"
            );
        }

        let captain = "character.protagonist.captain";
        let her_ground = world.positions[neriah];
        world.order_move(captain, her_ground).unwrap();
        let mut reached = false;
        for _ in 0..6000 {
            peaceful_tick!();
            if world.approach_island_person(neriah) {
                reached = true;
                break;
            }
        }
        assert!(reached, "Michael must be able to reach her");
        for _ in 0..256 {
            if world.can_talk_island_person(neriah) {
                break;
            }
            peaceful_tick!();
        }
        assert!(world.can_talk_island_person(neriah));
        world.talk_island_person(neriah);
        assert!(world.recruit_island_person(neriah));

        // Now she is with him, and the lead waits on the evidence being real.
        let mut opened = false;
        for _ in 0..12000 {
            peaceful_tick!();
            if world.open_leads.contains_key(lead) {
                opened = true;
                break;
            }
        }
        assert!(
            opened,
            "the shrine must actually grow and she must actually notice"
        );
        // She only raises it because the world really is that way.
        assert!(
            world.factions["faction.cthulhu.prototype"]
                .buildings
                .values()
                .any(
                    |building| building.archetype_id == "site_archetype.cthulhu.drowned_shrine"
                        && building.level >= 2
                )
        );
        // Other leads may be open too -- a companion standing with him can raise
        // one of her own -- so assert about hers rather than about the count.
        let hers = world
            .open_leads()
            .into_iter()
            .find(|open| open.id == lead)
            .expect("her lead is open");
        assert_eq!(hers.interpretations.len(), 2);
        assert_eq!(hers.companion_id, neriah);

        // A reading she never offered is refused, and refusing changes nothing.
        assert!(!world.resolve_lead(lead, "interpretation.invented"));
        assert!(world.open_leads.contains_key(lead));
        assert!(!world.flags.contains("flag.neriah.water_is_the_island"));

        let before = world.stored_resource("faction.michael", "resource.provisions");
        assert!(world.resolve_lead(lead, "interpretation.symptom"));
        assert!(world.flags.contains("flag.neriah.water_is_the_island"));
        assert_eq!(
            world.stored_resource("faction.michael", "resource.provisions"),
            before + 2
        );
        // The reading he did not back leaves no trace.
        assert!(!world.flags.contains("flag.neriah.water_is_theirs"));
        // A lead is answered once: hers is gone from the open set, whatever else
        // a companion may still be waiting to ask him.
        assert!(!world.open_leads.contains_key(lead));
        assert_eq!(
            world.resolved_leads.get(lead),
            Some(&"interpretation.symptom".to_string())
        );
        assert!(!world.resolve_lead(lead, "interpretation.deliberate"));

        // His decision survives the campaign being saved and resumed.
        let restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(
            restored.resolved_leads.get(lead),
            Some(&"interpretation.symptom".to_string())
        );
        assert!(!restored.open_leads.contains_key(lead));
    }

    fn test_quest(initial_stage: &str) -> ScenarioQuest {
        ScenarioQuest {
            display_name: "Test quest".into(),
            initial_stage: initial_stage.into(),
            stages: BTreeMap::from([
                (
                    "stage.a".into(),
                    QuestStage {
                        objective: "Do the first thing.".into(),
                        terminal: None,
                    },
                ),
                (
                    "stage.b".into(),
                    QuestStage {
                        objective: "Do the second thing.".into(),
                        terminal: None,
                    },
                ),
                (
                    "stage.done".into(),
                    QuestStage {
                        objective: String::new(),
                        terminal: Some(QuestOutcome::Success),
                    },
                ),
            ]),
        }
    }

    /// `set_quest_stage` moves a quest from its current stage to a declared
    /// one, and once a quest reaches a terminal stage no further effect can
    /// move it again.
    #[test]
    fn set_quest_stage_moves_a_quest_and_then_refuses_to_move_it_again() {
        let mut world = main_scenario_world();
        world.rules.quests = BTreeMap::from([("quest.test".to_string(), test_quest("stage.a"))]);
        world.quest_stages = BTreeMap::from([("quest.test".to_string(), "stage.a".to_string())]);
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.advance".into(),
            repeat: false,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::SetQuestStage {
                quest_id: "quest.test".into(),
                stage_id: "stage.b".into(),
            }],
        }];
        world.advance_island_tick();
        assert_eq!(
            world.quest_stages.get("quest.test"),
            Some(&"stage.b".to_string())
        );
        assert!(world.fired_triggers.contains_key("trigger.test.advance"));
        // Non-repeating and already fired: a second tick does not re-fire it,
        // even though its effect (b -> b would be a no-op, but the trigger
        // itself is settled) stays recorded at the tick it happened.
        let advanced_tick = *world.fired_triggers.get("trigger.test.advance").unwrap();
        world.advance_island_tick();
        assert_eq!(
            world.fired_triggers.get("trigger.test.advance"),
            Some(&advanced_tick)
        );

        // A repeating trigger reaches the terminal stage, then can never move
        // the quest again -- not even to a different stage, which is the
        // terminal-lock rule and not just the same-stage no-op guard.
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.repeat_finish".into(),
            repeat: true,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::SetQuestStage {
                quest_id: "quest.test".into(),
                stage_id: "stage.done".into(),
            }],
        }];
        world.advance_island_tick();
        assert_eq!(
            world.quest_stages.get("quest.test"),
            Some(&"stage.done".to_string())
        );
        let settled_tick = *world
            .fired_triggers
            .get("trigger.test.repeat_finish")
            .unwrap();
        // A different repeating trigger now tries to move the settled quest
        // back to stage.a -- a genuinely different target, not a no-op.
        world.rules.triggers = vec![ScenarioTrigger {
            id: "trigger.test.repeat_revert".into(),
            repeat: true,
            when: vec![TriggerCondition::DayAtLeast { day: 1 }],
            then: vec![TriggerEffect::SetQuestStage {
                quest_id: "quest.test".into(),
                stage_id: "stage.a".into(),
            }],
        }];
        world.advance_island_tick();
        assert_eq!(
            world.quest_stages.get("quest.test"),
            Some(&"stage.done".to_string())
        );
        assert!(
            !world
                .fired_triggers
                .contains_key("trigger.test.repeat_revert")
        );
        assert_eq!(
            world.fired_triggers.get("trigger.test.repeat_finish"),
            Some(&settled_tick)
        );
    }

    /// Quest state survives a save round trip.
    #[test]
    fn quest_stages_survive_a_save_round_trip() {
        let mut world = main_scenario_world();
        world.rules.quests = BTreeMap::from([("quest.test".to_string(), test_quest("stage.a"))]);
        world.quest_stages = BTreeMap::from([("quest.test".to_string(), "stage.b".to_string())]);
        let restored =
            FactionWorld::load_json_for_scenario(&world.save_json().unwrap(), &world.rules.clone())
                .unwrap();
        assert_eq!(restored.quest_stages, world.quest_stages);
        assert_eq!(restored, world);
    }

    /// A save is untrusted input: a quest missing a stage entry, an entry
    /// naming an undeclared quest, or a stage that quest does not define are
    /// each refused by name.
    #[test]
    fn a_tampered_quest_stage_is_refused_by_name() {
        fn refusal(mutate: impl FnOnce(&mut FactionWorld)) -> String {
            let mut world = main_scenario_world();
            world.rules.quests =
                BTreeMap::from([("quest.test".to_string(), test_quest("stage.a"))]);
            world.quest_stages =
                BTreeMap::from([("quest.test".to_string(), "stage.a".to_string())]);
            mutate(&mut world);
            FactionWorld::load_json_for_scenario(&world.save_json().unwrap(), &world.rules.clone())
                .unwrap_err()
        }
        assert_eq!(
            refusal(|world| {
                world.quest_stages.clear();
            }),
            "invalid_saved_quest_stages"
        );
        assert_eq!(
            refusal(|world| {
                world
                    .quest_stages
                    .insert("quest.does_not_exist".into(), "stage.a".into());
            }),
            "invalid_saved_quest_stages"
        );
        assert_eq!(
            refusal(|world| {
                world
                    .quest_stages
                    .insert("quest.test".into(), "stage.does_not_exist".into());
            }),
            "invalid_saved_quest_stages"
        );
    }

    /// Until the pack tooling lands, the fixture stands in for the generated
    /// document. When the real one is present they must be the same document.
    #[test]
    fn saved_world_rejects_version_and_dangling_actor() {
        let world = main_scenario_world();
        let mut data: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        data["version"] = 99.into();
        assert!(FactionWorld::load_json(&data.to_string()).is_err());
        data["version"] = 2.into();
        data["world"]["factions"] = serde_json::json!({});
        assert!(FactionWorld::load_json(&data.to_string()).is_err());
        assert!(FactionWorld::load_json("{broken").is_err());
    }

    fn autonomous_world() -> FactionWorld {
        let mut world = production_world();
        world
            .set_policy(
                "faction.colonial_powers.prototype",
                FactionPolicy {
                    development: BTreeMap::new(),
                    income_per_tick: [
                        ("resource.provisions".into(), 1),
                        ("resource.iron".into(), 1),
                    ]
                    .into_iter()
                    .collect(),
                    storage_caps: [
                        ("resource.provisions".into(), 10),
                        ("resource.iron".into(), 10),
                    ]
                    .into_iter()
                    .collect(),
                    production: [("site.colonial.watch_fort.instance_1".into(), marine_rule())]
                        .into_iter()
                        .collect(),
                    objectives: vec![candidate(
                        "dispatch.raid",
                        "network_node.smuggler_cove",
                        20,
                        0,
                    )],
                },
            )
            .unwrap();
        world
    }

    #[test]
    fn holding_development_reserves_times_pauses_and_persists() {
        let mut world = autonomous_world();
        let faction_id = "faction.colonial_powers.prototype";
        let building_id = "site.colonial.watch_fort.instance_1";
        let rule = BuildingDevelopmentRule {
            max_level: 5,
            costs: [("resource.iron".into(), 4)].into(),
            ticks: 3,
            health_gain: 40,
            population_gain: 2,
        };
        let mut policy = world.policies[faction_id].clone();
        policy.development.insert(building_id.into(), rule);
        policy.income_per_tick.clear();
        world.set_policy(faction_id, policy).unwrap();
        let faction = world.factions.get_mut(faction_id).unwrap();
        faction.population_used = faction.population_capacity;
        faction.resources.insert("resource.iron".into(), 3);
        let old_capacity = faction.population_capacity;
        let before = world.clone();
        world.advance_faction_decisions();
        assert_eq!(
            before, world,
            "unaffordable work must not reserve a partial cost"
        );
        world
            .factions
            .get_mut(faction_id)
            .unwrap()
            .resources
            .insert("resource.iron".into(), 4);
        world.advance_faction_decisions();
        assert_eq!(world.factions[faction_id].resources["resource.iron"], 0);
        assert_eq!(
            world.factions[faction_id].buildings[building_id]
                .development
                .as_ref()
                .unwrap()
                .remaining_ticks,
            3
        );
        assert!(matches!(
            world.enqueue_production(faction_id, building_id, marine_rule()),
            Err(FactionWorldError::QueueFull(_))
        ));
        world.paused = true;
        let frozen = world.clone();
        world.advance_island_tick();
        assert_eq!(world, frozen);
        world.paused = false;
        world.advance_production_tick();
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(restored, world);
        restored
            .factions
            .get_mut(faction_id)
            .unwrap()
            .buildings
            .get_mut(building_id)
            .unwrap()
            .health -= 7;
        restored.advance_production_tick();
        restored.advance_production_tick();
        let building = &restored.factions[faction_id].buildings[building_id];
        assert_eq!(
            (building.level, building.health, building.max_health),
            (2, 113, 120)
        );
        assert!(building.development.is_none());
        assert_eq!(
            restored.factions[faction_id].population_capacity,
            old_capacity + 2
        );
        assert_eq!(
            FactionWorld::load_json(&restored.save_json().unwrap()).unwrap(),
            restored
        );
        // Damage prevents starting the next project; even a prosperous holding
        // cannot develop past the authored level cap.
        let faction = restored.factions.get_mut(faction_id).unwrap();
        faction.population_used = faction.population_capacity;
        faction.resources.insert("resource.iron".into(), 20);
        restored.advance_faction_decisions();
        assert!(
            restored.factions[faction_id].buildings[building_id]
                .development
                .is_none()
        );
        let building = restored
            .factions
            .get_mut(faction_id)
            .unwrap()
            .buildings
            .get_mut(building_id)
            .unwrap();
        building.health = building.max_health;
        building.level = 5;
        restored.advance_faction_decisions();
        assert!(
            restored.factions[faction_id].buildings[building_id]
                .development
                .is_none()
        );
        let mut invalid: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        invalid["world"]["factions"][faction_id]["buildings"][building_id]["development"]["reserved_costs"]
            ["resource.iron"] = 1.into();
        assert!(FactionWorld::load_json(&invalid.to_string()).is_err());
        world.eliminate_faction(faction_id).unwrap();
        for _ in 0..10 {
            world.advance_island_tick();
        }
        assert!(world.factions[faction_id].buildings.is_empty());
    }

    #[test]
    fn authored_development_matches_live_producer_resources() {
        let rules = main_scenario().rules.development.clone();
        for faction_id in [
            "faction.colonial_powers.prototype",
            "faction.pirates.prototype",
            "faction.cthulhu.prototype",
        ] {
            let producer = scenario_production(faction_id);
            let development = &rules[&producer.producer_archetype_id];
            assert_eq!(development.max_level, 5);
            assert!(development
                .costs
                .iter()
                .all(|(resource, cost)| producer.costs.contains_key(resource) && cost * 4 <= 20));
        }
    }

    #[test]
    fn autonomous_faction_builds_and_dispatches_without_player_commands() {
        let mut world = autonomous_world();
        let mut history = Vec::new();
        for _ in 0..40 {
            history.extend(world.advance_island_tick());
        }
        assert!(
            history
                .iter()
                .any(|e| matches!(e, FactionWorldEvent::ProductionQueued { .. }))
        );
        assert!(
            history
                .iter()
                .any(|e| matches!(e, FactionWorldEvent::ActorProduced { .. }))
        );
        assert!(
            history
                .iter()
                .any(|e| matches!(e, FactionWorldEvent::ActorAssigned { .. }))
        );
        assert_eq!(world.actors.len(), 4); // population cap, not unlimited spawning
        assert!(
            world
                .actors
                .values()
                .all(|a| a.node_id == "network_node.smuggler_cove")
        );
        assert!(
            world.factions["faction.colonial_powers.prototype"]
                .resources
                .values()
                .all(|amount| *amount <= 10)
        );
        world.paused = true;
        let frozen = world.clone();
        world.advance_island_tick();
        assert_eq!(world, frozen);
    }

    #[test]
    fn eliminated_faction_stays_gone_after_save_reload_and_many_ticks() {
        let mut world = autonomous_world();
        for _ in 0..6 {
            world.advance_island_tick();
        }
        world
            .eliminate_faction("faction.colonial_powers.prototype")
            .unwrap();
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        for _ in 0..100 {
            restored.advance_island_tick();
        }
        assert!(restored.actors.is_empty());
        assert!(restored.policies.is_empty());
        assert!(restored.travel_orders.is_empty());
        assert!(
            restored.factions["faction.colonial_powers.prototype"]
                .buildings
                .is_empty()
        );
        assert!(matches!(
            restored.enqueue_production(
                "faction.colonial_powers.prototype",
                "site.colonial.watch_fort.instance_1",
                marine_rule()
            ),
            Err(FactionWorldError::FactionEliminated(_))
        ));
    }

    #[test]
    fn autonomous_save_continuation_is_deterministic() {
        let mut original = autonomous_world();
        for _ in 0..5 {
            original.advance_island_tick();
        }
        let mut restored = FactionWorld::load_json(&original.save_json().unwrap()).unwrap();
        for _ in 0..40 {
            assert_eq!(
                original.advance_island_tick(),
                restored.advance_island_tick()
            );
            assert_eq!(original, restored);
        }
    }

    #[test]
    fn autonomous_campaign_reaches_and_destroys_hostile_holdings() {
        let mut world = main_scenario_world();
        let mut siege_assigned = false;
        let mut building_struck = false;
        for step in 0..60_000 {
            // Finite-supply scenario: existing troops and queued cycles stay,
            // but endless replacement income must not mask siege reachability.
            if step == 10_000 {
                for policy in world.policies.values_mut() {
                    policy.income_per_tick.clear();
                }
                for faction in world.factions.values_mut() {
                    faction.resources.clear();
                }
            }
            for event in world.advance_island_tick() {
                match event {
                    FactionWorldEvent::ActorAssigned { assignment, .. }
                        if assignment == "attack_holding" =>
                    {
                        siege_assigned = true
                    }
                    FactionWorldEvent::UnitStruck { target_id, .. }
                        if target_id.ends_with(".producer") =>
                    {
                        building_struck = true
                    }
                    _ => {}
                }
            }
            if !world.eliminated_factions.is_empty() {
                break;
            }
        }
        assert!(
            siege_assigned,
            "rallied troops must choose hostile holdings"
        );
        assert!(
            building_struck,
            "troops must reach siege range without fixture teleportation"
        );
        assert!(
            !world.eliminated_factions.is_empty(),
            "sustained AI war must destroy a producer"
        );
        assert!(world.actors.contains_key("character.protagonist.captain"));
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        for _ in 0..10 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
        }
    }

    #[test]
    fn preview_factions_fight_and_preserve_casualties_without_attacking_michael() {
        let mut world = main_scenario_world();
        let mut history = Vec::new();
        for _ in 0..6000 {
            history.extend(world.advance_island_tick());
        }
        assert!(
            history
                .iter()
                .any(|e| matches!(e, FactionWorldEvent::UnitStruck { .. }))
        );
        assert!(
            history
                .iter()
                .any(|e| matches!(e, FactionWorldEvent::UnitFallen { .. }))
        );
        assert!(!world.casualties.is_empty());
        assert!(world.actors.contains_key("character.protagonist.captain"));
        for id in world.casualties.keys() {
            assert!(!world.actors.contains_key(id));
            assert!(!world.positions.contains_key(id));
            assert!(!world.travel_orders.contains_key(id));
        }
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        for _ in 0..20 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
        }
    }

    #[test]
    fn skirmish_blocks_shots_through_nonwalkable_terrain() {
        let navigation = IslandNavigation {
            building_obstacles: BTreeMap::new(),
            walkable: [IslandPoint { x: 0, y: 0 }, IslandPoint { x: 2, y: 0 }]
                .into_iter()
                .collect(),
            destinations: Default::default(),
        };
        assert!(!navigation.clear_line(IslandPoint { x: 0, y: 0 }, IslandPoint { x: 2, y: 0 }));
    }

    #[test]
    fn fort_footprint_routes_around_walls_and_survives_save() {
        let mut world = main_scenario_world();
        let (id, cells) = world.navigation.building_obstacles.iter().next().unwrap();
        let id = id.clone();
        let cells = cells.clone();
        assert_eq!(cells.len(), 5);
        let entrance = world.navigation.destinations
            [&world.factions["faction.colonial_powers.prototype"].buildings[&id].node_id];
        assert!(world.navigation.traversable(entrance));
        for cell in &cells {
            assert!(!world.navigation.traversable(*cell));
            assert!(world.navigation.path(entrance, *cell).is_none());
            assert!(!world.navigation.clear_line(entrance, *cell));
        }
        let from = IslandPoint {
            x: entrance.x - 3,
            y: entrance.y,
        };
        let route = world.navigation.path(from, entrance).unwrap();
        assert!(route.len() > 4);
        assert!(route.iter().all(|cell| !cells.contains(cell)));
        let loaded = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(loaded.navigation, world.navigation);
        world
            .eliminate_faction("faction.colonial_powers.prototype")
            .unwrap();
        assert!(!world.navigation.building_obstacles.contains_key(&id));
        assert!(cells.iter().all(|cell| world.navigation.traversable(*cell)));
        let mut invalid = loaded;
        invalid
            .navigation
            .building_obstacles
            .insert("missing".into(), cells);
        assert!(FactionWorld::load_json(&invalid.save_json().unwrap()).is_err());
    }

    #[test]
    fn legacy_save_gets_authored_obstacles_without_teleporting_actors() {
        let mut world = main_scenario_world();
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"]["navigation"]
            .as_object_mut()
            .unwrap()
            .remove("building_obstacles");
        let restored = FactionWorld::load_json(&legacy.to_string()).unwrap();
        assert_eq!(restored, world);
        let blocked = *world
            .navigation
            .building_obstacles
            .values()
            .next()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        legacy["world"]["positions"]["character.protagonist.captain"] =
            serde_json::json!({"x": blocked.x, "y": blocked.y});
        assert_eq!(
            FactionWorld::load_json(&legacy.to_string()).unwrap_err(),
            "invalid_saved_position"
        );
        // Overflow is rejected, never a panic from an untrusted old save.
        let building = world.factions["faction.colonial_powers.prototype"]
            .buildings
            .values()
            .next()
            .unwrap();
        legacy["world"]["navigation"]["destinations"][&building.node_id] =
            serde_json::json!({"x": i32::MIN, "y": i32::MIN});
        assert_eq!(
            FactionWorld::load_json(&legacy.to_string()).unwrap_err(),
            "building_coordinate_overflow"
        );
    }

    #[test]
    fn autonomous_ranged_holds_while_melee_closes_and_resumes_when_target_is_lost() {
        let mut world = main_scenario_world();
        advance_until_raised(
            &mut world,
            &[
                "actor_def.colonial.line_marine",
                "actor_def.pirates.deckhand",
            ],
        );
        let marine = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.colonial.line_marine")
            .unwrap()
            .instance_id
            .clone();
        let pirate = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        world.actors.retain(|id, _| id == &marine || id == &pirate);
        world
            .positions
            .retain(|id, _| world.actors.contains_key(id));
        world
            .unit_combat
            .retain(|id, _| world.actors.contains_key(id));
        world.travel_orders.clear();
        for faction in world.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                building.operational = false;
            }
        }
        let origin = IslandPoint { x: 20, y: 16 };
        let enemy = IslandPoint { x: 23, y: 16 };
        world.positions.insert(marine.clone(), origin);
        world.positions.insert(pirate.clone(), enemy);
        world.unit_combat.get_mut(&marine).unwrap().next_attack_tick = 1000;
        world.unit_combat.get_mut(&pirate).unwrap().next_attack_tick = 1000;
        world.order_move(&marine, enemy).unwrap();
        world.order_move(&pirate, origin).unwrap();
        world.advance_island_tick();
        assert_eq!(world.positions[&marine], origin); // holds even while reloading
        assert_eq!(world.positions[&pirate], IslandPoint { x: 22, y: 16 });
        assert!(world.travel_orders.contains_key(&marine));
        world.advance_island_tick();
        assert_eq!(world.positions[&pirate], IslandPoint { x: 21, y: 16 });
        world.advance_island_tick();
        assert_eq!(world.positions[&pirate], IslandPoint { x: 21, y: 16 }); // melee range
        world.unit_combat.get_mut(&pirate).unwrap().health = 0;
        world.advance_island_tick();
        assert_eq!(world.positions[&marine], IslandPoint { x: 21, y: 16 });
    }

    #[test]
    fn player_carbine_is_queued_paused_and_provokes_retaliation_on_hit() {
        let mut world = main_scenario_world();
        advance_until_raised(&mut world, &["actor_def.pirates.deckhand"]);
        let target = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        let captain = "character.protagonist.captain";
        let position = world.positions[&target];
        world.positions.insert(captain.into(), position);
        assert!(!world.aim_carbine(captain));
        assert!(!world.aim_carbine("missing"));
        assert!(world.aim_carbine(&target));
        let hp = world.unit_combat[&target].health;
        assert!(
            world
                .order_move(captain, IslandPoint { x: -100, y: -100 })
                .is_err()
        );
        assert_eq!(world.player_attack_target.as_deref(), Some(target.as_str()));
        world.order_move(captain, position).unwrap();
        assert!(world.player_attack_target.is_none());
        assert!(world.aim_carbine(&target));
        assert!(
            !world
                .hostilities
                .contains(&("faction.pirates.prototype".into(), "faction.michael".into()))
        );
        world.paused = true;
        assert!(world.advance_island_tick().is_empty());
        assert_eq!(world.unit_combat[&target].health, hp);
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
        world.paused = false;
        let events = world.advance_island_tick();
        assert!(events.iter().any(|event| matches!(event, FactionWorldEvent::UnitStruck {attacker_id, target_id, ..} if attacker_id == captain && target_id == &target)));
        assert!(world.player_attack_target.is_none());
        assert!(
            world
                .hostilities
                .contains(&("faction.pirates.prototype".into(), "faction.michael".into()))
        );
        assert!(world.aim_carbine(&target));
        let events = world.advance_island_tick();
        assert!(!events.iter().any(|event| matches!(event, FactionWorldEvent::UnitStruck {attacker_id, ..} if attacker_id == captain)));
    }

    #[test]
    fn persona_catalog_is_safe_and_produced_identity_survives_death_and_reload() {
        let pools = main_scenario().personas.clone();
        assert!(pools.values().all(|variants| !variants.is_empty()));
        for pool in pools.values().flatten() {
            assert!(pool.age_min >= 18 && pool.age_min <= pool.age_max);
            assert!(
                pool.companion_responses.is_empty()
                    || pool.companion_responses.len() == pool.histories.len()
            );
            assert!(
                pool.companion_responses
                    .iter()
                    .all(|line| !line.trim().is_empty() && line.len() <= 4096)
            );
            for values in [&pool.given_names, &pool.family_names, &pool.histories] {
                assert!(!values.is_empty());
                assert!(values.iter().all(|v| !v.trim().is_empty()));
            }
        }
        assert!(
            produced_person(
                &pools,
                "actor_def.unknown_machine",
                "machine.1",
                1,
                &BTreeSet::new()
            )
            .is_none()
        );
        let mut world = main_scenario_world();
        advance_until_raised(
            &mut world,
            &[
                "actor_def.pirates.deckhand",
                "actor_def.colonial.line_marine",
            ],
        );
        let originals: BTreeMap<_, _> = world
            .actors
            .iter()
            .map(|(id, a)| (id.clone(), a.person.clone().unwrap()))
            .collect();
        assert!(originals.len() > 1);
        for (id, person) in &originals {
            assert!(person.valid_for_actor(id, true));
        }
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(world, restored);
        for _ in 0..6000 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
        }
        assert_eq!(world, restored);
        assert!(!world.casualties.is_empty());
        for (id, casualty) in &world.casualties {
            let person = casualty.actor.person.as_ref().unwrap();
            assert!(person.valid_for_actor(id, false));
            if let Some(original) = originals.get(id) {
                let mut expected = original.clone();
                expected.alive_today = false;
                assert_eq!(&expected, person);
            }
        }
        let mut invalid = world.clone();
        invalid
            .actors
            .get_mut("character.protagonist.captain")
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .id = "someone.else".into();
        assert_eq!(
            FactionWorld::load_json(&invalid.save_json().unwrap()),
            Err("invalid_saved_person".into())
        );
        // Older saves have no identity field. Never reroll or guess one at load.
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        for actor in legacy["world"]["actors"]
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            actor.as_object_mut().unwrap().remove("person");
        }
        let loaded = FactionWorld::load_json(&legacy.to_string()).unwrap();
        assert!(loaded.actors.values().all(|a| a.person.is_none()));
    }

    #[test]
    fn queued_carbine_cancels_when_target_changes_to_michaels_faction() {
        let mut world = main_scenario_world();
        advance_until_raised(&mut world, &["actor_def.pirates.deckhand"]);
        let target = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        let captain = "character.protagonist.captain";
        world
            .positions
            .insert(captain.into(), world.positions[&target]);
        assert!(world.aim_carbine(&target));
        let hp = world.unit_combat[&target].health;
        // Exercise the ownership boundary itself; this is not a recruitment API.
        world.actors.get_mut(&target).unwrap().faction_id = "faction.michael".into();
        assert!(world.island_firing_target(captain).is_none());
        let events = world.resolve_island_skirmish();
        assert!(!events.iter().any(|e| matches!(e, FactionWorldEvent::UnitStruck { attacker_id, .. } if attacker_id == captain)));
        assert_eq!(world.unit_combat[&target].health, hp);
        assert!(world.player_attack_target.is_none());
        assert!(
            !world
                .hostilities
                .contains(&("faction.michael".into(), "faction.michael".into()))
        );
        world.actors.get_mut(&target).unwrap().faction_id = "faction.pirates.prototype".into();
        assert!(world.island_firing_target(captain).is_none());
    }

    #[test]
    fn siege_destroys_last_producer_and_elimination_survives_load() {
        let mut world = main_scenario_world();
        advance_until_raised(&mut world, &["actor_def.pirates.deckhand"]);
        let pirate = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        world.actors.retain(|id, _| id == &pirate);
        world.positions.retain(|id, _| id == &pirate);
        world.unit_combat.retain(|id, _| id == &pirate);
        world.travel_orders.clear();
        world.hostilities = [(
            "faction.pirates.prototype".into(),
            "faction.colonial_powers.prototype".into(),
        )]
        .into_iter()
        .collect();
        let fort = world.factions["faction.colonial_powers.prototype"]
            .buildings
            .values()
            .next()
            .unwrap();
        let fort_id = fort.id.clone();
        let entrance = world.navigation.destinations[&fort.node_id];
        world.positions.insert(pirate.clone(), entrance);
        let onward = world.navigation.destinations["island.contested_clearing"];
        world.order_move(&pirate, onward).unwrap();
        assert!(world.island_siege_target(&pirate).is_some());
        for faction in world.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                building.operational = false;
            }
        }
        // A lone raider is not an army. What is under test is that a real
        // strike destroys a real producer and that the elimination survives a
        // load -- not how long six hundred points of masonry takes.
        for building in world
            .factions
            .get_mut("faction.colonial_powers.prototype")
            .unwrap()
            .buildings
            .values_mut()
        {
            building.health = 8;
        }
        let mut saw_elimination = false;
        let before = world.positions[&pirate];
        world.advance_island_tick();
        assert_eq!(world.positions[&pirate], before);
        assert!(world.travel_orders.contains_key(&pirate));
        for _ in 0..40_000 {
            saw_elimination |= world.advance_island_tick().iter().any(|e|
                matches!(e, FactionWorldEvent::FactionEliminated {faction_id} if faction_id == "faction.colonial_powers.prototype"));
        }
        assert!(saw_elimination);
        assert_ne!(world.positions[&pirate], before);
        assert!(world.island_siege_target(&pirate).is_none());
        assert!(
            world.factions["faction.colonial_powers.prototype"]
                .buildings
                .is_empty()
        );
        assert!(!world.navigation.building_obstacles.contains_key(&fort_id));
        let restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert!(
            restored
                .eliminated_factions
                .contains("faction.colonial_powers.prototype")
        );
    }

    #[test]
    fn equal_dispatch_scores_use_stable_action_id_order() {
        let (mut world, actor_id) = world_with_produced_actor();
        let event = world
            .dispatch_actor(
                &actor_id,
                &[
                    candidate("dispatch.zulu", "network_node.black_beach", 10, 0),
                    candidate("dispatch.alpha", "network_node.river_fork", 10, 0),
                ],
            )
            .unwrap();
        assert!(matches!(
            event,
            FactionWorldEvent::ActorAssigned { ref action_id, .. }
                if action_id == "dispatch.alpha"
        ));
    }

    #[test]
    fn out_of_bounds_wobble_is_rejected_before_actor_mutation() {
        let (mut world, actor_id) = world_with_produced_actor();
        let before = world.actors[&actor_id].clone();
        assert_eq!(
            world
                .dispatch_actor(
                    &actor_id,
                    &[candidate(
                        "dispatch.invalid",
                        "network_node.river_fork",
                        10,
                        6
                    )],
                )
                .unwrap_err(),
            FactionWorldError::WobbleOutOfBounds {
                action_id: "dispatch.invalid".into(),
                wobble: 6,
                limit: 5,
            }
        );
        assert_eq!(world.actors[&actor_id], before);
    }

    fn volume(origin: GridCube, size: [u16; 3]) -> BuildingVolume {
        BuildingVolume {
            grid_standard: "map_cube_v1".into(),
            modules: vec![CubeModule { origin, size }],
        }
    }

    #[test]
    fn standard_multi_cube_building_reserves_every_cube() {
        let mut map = MapPlacement::default();
        map.place_building(
            "site.colonial.watch_fort.instance_1",
            GridCube { x: 10, y: 0, z: 20 },
            volume(GridCube { x: 0, y: 0, z: 0 }, [2, 2, 2]),
        )
        .unwrap();
        assert_eq!(map.occupied_cubes.len(), 8);
        assert_eq!(
            map.occupied_cubes[&GridCube { x: 11, y: 1, z: 21 }],
            "site.colonial.watch_fort.instance_1"
        );
    }

    #[test]
    fn colliding_building_is_rejected_without_partial_placement() {
        let mut map = MapPlacement::default();
        map.place_building(
            "site.first",
            GridCube { x: 0, y: 0, z: 0 },
            volume(GridCube { x: 0, y: 0, z: 0 }, [2, 1, 1]),
        )
        .unwrap();
        let before = map.clone();
        assert_eq!(
            map.place_building(
                "site.second",
                GridCube { x: 1, y: 0, z: 0 },
                volume(GridCube { x: 0, y: 0, z: 0 }, [2, 1, 1]),
            )
            .unwrap_err(),
            MapPlacementError::CubeOccupied {
                cube: GridCube { x: 1, y: 0, z: 0 },
                occupying_building_id: "site.first".into(),
            }
        );
        assert_eq!(map, before);
    }

    #[test]
    fn overlapping_modules_are_invalid_even_before_map_placement() {
        let volume = BuildingVolume {
            grid_standard: "map_cube_v1".into(),
            modules: vec![
                CubeModule {
                    origin: GridCube { x: 0, y: 0, z: 0 },
                    size: [2, 1, 1],
                },
                CubeModule {
                    origin: GridCube { x: 1, y: 0, z: 0 },
                    size: [1, 1, 1],
                },
            ],
        };
        assert_eq!(
            volume.occupied_local_cubes().unwrap_err(),
            MapPlacementError::OverlappingModules(GridCube { x: 1, y: 0, z: 0 })
        );
    }

    #[test]
    fn ordinary_production_uses_both_sexes_and_same_unit_can_join_party() {
        for definition in [
            "actor_def.colonial.line_marine",
            "actor_def.cthulhu.drowned_cultist",
        ] {
            let mut world = main_scenario_world();
            // Peace isolates the authored production and voluntary recruitment loop;
            // no actors, identities, population, or positions are fabricated.
            world.hostilities.clear();
            // New factions change shared production serials. Observe actual
            // bounded development/replacement output, not an assumed sex split
            // in the first six seeded births.
            for _ in 0..20_000 {
                world.hostilities.clear();
                world.advance_island_tick();
                // The same living-and-standing filter the assertions below
                // use, so the loop stops on a state those assertions can read
                // rather than on one where the male was killed two ticks ago.
                let produced = || {
                    world.actors.values().filter(|a| {
                        a.definition_id == definition
                            && a.instance_id.starts_with("actor_instance.")
                            && !a.undead
                            && a.person.as_ref().is_some_and(|p| p.alive_today)
                    })
                };
                if produced().any(|a| a.person.as_ref().is_some_and(|p| p.sex == PersonSex::Male))
                    && produced().any(|a| {
                        a.person
                            .as_ref()
                            .is_some_and(|p| p.sex == PersonSex::Female)
                    })
                {
                    break;
                }
            }
            // Ordinary production means the persona pools. An authored notable
            // shares this production stream by design, but she is not a rolled
            // identity and her authored age is not the pool's range, so this
            // test looks only at the rolled ones -- which the instance id
            // already distinguishes.
            // Over a stretch this long some of the rolled units have since
            // been killed and, if they belonged to the shrine, walked back at
            // midnight. That is the midnight rule doing its job, not
            // production emitting corpses, so the roll is read off the ones
            // still standing.
            let produced: Vec<_> = world
                .actors
                .values()
                .filter(|a| {
                    a.definition_id == definition
                        && a.instance_id.starts_with("actor_instance.")
                        && !a.undead
                        && a.person.as_ref().is_some_and(|p| p.alive_today)
                })
                .collect();
            let male = produced
                .iter()
                .find(|a| a.person.as_ref().unwrap().sex == PersonSex::Male)
                .expect("ordinary production includes male units");
            let female = produced
                .iter()
                .find(|a| a.person.as_ref().unwrap().sex == PersonSex::Female)
                .expect("ordinary production includes female units");

            let id = female.instance_id.clone();
            let original = (*female).clone();
            let profile = world.combat_profiles[definition].clone();
            let male_id = male.instance_id.clone();
            assert!((18..=20).contains(&original.person.as_ref().unwrap().age.unwrap()));
            assert!(
                !original
                    .person
                    .as_ref()
                    .unwrap()
                    .recruitment_offer
                    .is_empty()
            );
            assert_eq!(male.definition_id, original.definition_id);
            assert_eq!(
                world.unit_combat[&male_id].health,
                world.unit_combat[&id].health
            );
            assert_eq!(
                world.unit_combat[&male_id].population_use,
                world.unit_combat[&id].population_use
            );
            world.policies.clear();
            for faction in world.factions.values_mut() {
                for building in faction.buildings.values_mut() {
                    building.operational = false;
                }
            }
            assert!(world.approach_island_person(&id));
            for _ in 0..128 {
                if world.can_talk_island_person(&id) {
                    break;
                }
                world.advance_island_tick();
            }
            assert!(
                world.can_talk_island_person(&id),
                "Michael must navigate to the actually produced woman"
            );
            let count = world.actors.len();
            let source_population = world.factions[&original.faction_id].population_used;
            let michael_population = world.factions["faction.michael"].population_used;
            let combat = world.unit_combat[&id].clone();
            assert!(
                !world.recruit_island_person(&id),
                "conversation must be explicit first"
            );
            assert_eq!(
                world.talk_island_person(&id),
                original.person.as_ref().unwrap().recruitment_offer
            );
            assert!(world.recruit_island_person(&id));
            assert!(world.assign_island_companion(&id, 0));
            assert_eq!(world.party[0], id);
            assert_eq!(world.actors.len(), count);
            assert_eq!(world.actors[&id].faction_id, "faction.michael");
            assert_eq!(world.actors[&id].definition_id, original.definition_id);
            assert_eq!(world.actors[&id].provenance, original.provenance);
            assert_eq!(
                world.actors[&id].person.as_ref().unwrap().display_name,
                original.person.as_ref().unwrap().display_name
            );
            assert_eq!(world.unit_combat[&id], combat);
            assert_eq!(
                world.combat_profiles[&world.actors[&id].definition_id],
                profile
            );
            assert!(!world.actors[&id].undead);
            assert_eq!(
                world.factions[&original.faction_id].population_used,
                source_population - combat.population_use
            );
            assert_eq!(
                world.factions["faction.michael"].population_used,
                michael_population + combat.population_use
            );
            assert_eq!(world.actors[&male_id].faction_id, original.faction_id);
            assert!(!world.recruit_island_person(&male_id));
            assert_eq!(
                FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
                world
            );
        }
    }

    #[test]
    fn elven_wardens_replace_slowly_deploy_and_retain_longbow_when_recruited() {
        let mut world = main_scenario_world();
        world.hostilities.clear();
        let elves = "faction.elves.prototype";
        let definition = "actor_def.elven.bow_warden";
        let building = world.factions[elves].buildings.values().next().unwrap();
        let home = world.navigation.destinations[&building.node_id];
        assert_eq!((building.level, building.health), (1, 750));
        assert_eq!(world.factions[elves].population_capacity, 3);
        assert!(!world.actors.values().any(|a| a.faction_id == elves));
        // Peace has to hold for the whole observation, or diplomacy re-arms
        // the war and the wardens are killed as fast as the grove raises them.
        let elves_alive =
            |w: &FactionWorld| w.actors.values().filter(|a| a.faction_id == elves).count();
        let mut raised = false;
        for _ in 0..20_000 {
            if elves_alive(&world) >= 1 {
                raised = true;
                break;
            }
            world.hostilities.clear();
            world.advance_island_tick();
        }
        assert!(raised, "the heart grove raises its first warden");
        assert_eq!(
            world
                .actors
                .values()
                .filter(|a| a.faction_id == elves)
                .count(),
            1
        );
        let mut filled = false;
        for _ in 0..20_000 {
            if elves_alive(&world) == 3 {
                filled = true;
                break;
            }
            world.hostilities.clear();
            world.advance_island_tick();
        }
        assert!(filled, "the grove fills its three slots");
        let wardens: Vec<_> = world
            .actors
            .values()
            .filter(|a| a.faction_id == elves)
            .collect();
        assert_eq!(wardens.len(), 3);
        assert!(
            wardens
                .iter()
                .all(|a| a.definition_id == definition && a.actor_kind == "soldier")
        );
        assert!(
            wardens
                .iter()
                .any(|a| world.positions[&a.instance_id] != home)
        );
        let woman = wardens
            .iter()
            .find(|a| {
                a.person
                    .as_ref()
                    .is_some_and(|p| p.sex == PersonSex::Female)
            })
            .unwrap();
        let id = woman.instance_id.clone();
        let original = (*woman).clone();
        assert!(!original.undead);
        let profile = world.combat_profiles[definition].clone();
        assert_eq!(
            (
                profile.health,
                profile.damage,
                profile.range,
                profile.cooldown_ticks
            ),
            (20, 4, 5, 80)
        );
        world.policies.clear();
        for faction in world.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                building.operational = false;
            }
        }
        assert!(world.approach_island_person(&id));
        for _ in 0..128 {
            if world.can_talk_island_person(&id) {
                break;
            }
            world.advance_island_tick();
        }
        assert!(!world.talk_island_person(&id).is_empty());
        assert!(world.recruit_island_person(&id));
        assert!(world.assign_island_companion(&id, 0));
        assert_eq!(world.factions[elves].population_used, 2);
        assert_eq!(world.actors[&id].provenance, original.provenance);
        assert_eq!(world.actors[&id].definition_id, definition);
        assert_eq!(world.combat_profiles[definition], profile);
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
    }

    #[test]
    fn colonial_expansion_uses_real_travel_and_paid_saved_work_when_settlement_survives() {
        let mut world = main_scenario_world();
        world.hostilities.clear(); // Controlled survival, no gifted resources or teleported builder.
        let rule = world.rules.expansion.clone();
        let mut started = None;
        for _ in 0..40_000 {
            // Peace has to hold for the whole observation. A single clear at
            // the top is not enough now that the settlement is authored to
            // wait days: diplomacy re-arms the war and the colonials are
            // eliminated long before they lay a foundation.
            world.hostilities.clear();
            world.advance_island_tick();
            if let Some(b) = world.factions[&rule.faction_id]
                .buildings
                .get(&rule.building_id)
            {
                started = Some((world.tick, b.construction.clone()));
                break;
            }
        }
        eprintln!(
            "Expansion observation: tick={}, started={started:?}, eliminated={:?}, resources={:?}",
            world.tick, world.eliminated_factions, world.factions[&rule.faction_id].resources
        );
        assert!(
            started.is_some(),
            "a surviving settlement must actually establish the paid site"
        );
        let mut foreign_builder = world.clone();
        let job = foreign_builder.factions[&rule.faction_id].buildings[&rule.building_id]
            .construction
            .clone()
            .unwrap();
        foreign_builder
            .actors
            .get_mut(&job.builder_id)
            .unwrap()
            .faction_id = "faction.michael".into();
        foreign_builder.advance_production_tick();
        assert_eq!(
            foreign_builder.factions[&rule.faction_id].buildings[&rule.building_id]
                .construction
                .as_ref()
                .unwrap()
                .remaining_ticks,
            job.remaining_ticks
        );
        let saved = world.save_json().unwrap();
        world = FactionWorld::load_json(&saved).unwrap();
        world.paused = true;
        let before = world.clone();
        world.advance_island_tick();
        assert_eq!(world, before);
        world.paused = false;
        for _ in 0..20_000 {
            world.hostilities.clear();
            world.advance_island_tick();
            if world.factions[&rule.faction_id]
                .buildings
                .get(&rule.building_id)
                .is_some_and(|b| b.operational)
            {
                break;
            }
        }
        assert!(world.factions[&rule.faction_id].buildings[&rule.building_id].operational);
        let level_gains: u32 = world.factions[&rule.faction_id]
            .buildings
            .values()
            .map(|b| b.level - 1)
            .sum();
        assert_eq!(
            world.factions[&rule.faction_id].population_capacity,
            6 + level_gains * 2,
            "only normal holding upgrades grant capacity"
        );
        for _ in 0..20_000 {
            world.hostilities.clear();
            world.advance_island_tick();
            if world
                .actors
                .values()
                .any(|a| a.provenance.producer_building_id == rule.building_id)
            {
                break;
            }
        }
        assert!(
            world
                .actors
                .values()
                .any(|a| a.provenance.producer_building_id == rule.building_id),
            "finished second holding must produce real units"
        );
    }

    #[test]
    fn authored_wars_survival_truces_and_local_male_news_share_combat_truth() {
        let mut world = main_scenario_world();
        let colonial = "faction.colonial_powers.prototype";
        let pirates = "faction.pirates.prototype";
        let cthulhu = "faction.cthulhu.prototype";
        assert!(
            world
                .hostilities
                .contains(&(colonial.into(), "faction.elves.prototype".into()))
        );
        assert!(
            !world
                .hostilities
                .contains(&(pirates.into(), cthulhu.into()))
        );
        assert!(!world.hostilities.contains(&(
            colonial.into(),
            "faction.eastern_fox_people.prototype".into()
        )));
        // The shrine has to have raised somebody before its army can be the
        // stronger one; eight ticks predates the first cultist by hours.
        advance_until_raised(&mut world, &["actor_def.cthulhu.drowned_cultist"]);
        // Isolate the pressure threshold with a stronger existing cult army,
        // not free actors or a separate war state. Production identity remains.
        world
            .combat_profiles
            .get_mut("actor_def.cthulhu.drowned_cultist")
            .unwrap()
            .damage = 100;
        // Run on to the pack's next diplomacy decision rather than winding the
        // clock back to a tick that predates the world's own history.
        let cadence = world.rules.survival.decision_ticks;
        for _ in 0..=cadence {
            world.advance_island_tick();
            if world.tick % cadence == 0 {
                break;
            }
        }
        // Diplomacy reads the tick it is entering, and `tick` only advances
        // part-way through the step, so the decision is made by the next one.
        world.advance_island_tick();
        assert!(
            world
                .hostilities
                .contains(&(pirates.into(), cthulhu.into()))
        );
        assert!(
            !world
                .hostilities
                .contains(&(colonial.into(), pirates.into()))
        );
        assert!(
            !world
                .hostilities
                .contains(&(pirates.into(), colonial.into()))
        );
        let treaty = world
            .survival_truces
            .iter()
            .find(|t| t.a == colonial && t.b == pirates)
            .unwrap()
            .clone();
        // The truce runs for the span the pack authors, from whenever it was
        // struck -- not from a tick number that was only true of one cadence.
        assert!(treaty.expires_tick > world.tick);
        assert!(treaty.expires_tick <= world.tick + world.rules.survival.truce_ticks);
        assert!(
            world
                .hostilities
                .iter()
                .all(|(a, b)| a != "faction.michael" && b != "faction.michael")
        );
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"]
            .as_object_mut()
            .unwrap()
            .remove("survival_truces");
        legacy["world"]
            .as_object_mut()
            .unwrap()
            .remove("diplomacy_notices");
        assert_eq!(
            FactionWorld::load_json(&legacy.to_string())
                .unwrap()
                .hostilities,
            world.hostilities
        );
        // The truce is renewed for another authored span while the threat
        // that caused it is still the greater one.
        let span = world.rules.survival.truce_ticks;
        let renewed = treaty.expires_tick + span;
        world.tick = treaty.expires_tick;
        world.advance_island_tick();
        assert!(
            world
                .survival_truces
                .iter()
                .any(|t| t.a == colonial && t.b == pirates && t.expires_tick == renewed)
        );
        world
            .combat_profiles
            .get_mut("actor_def.cthulhu.drowned_cultist")
            .unwrap()
            .damage = 0;
        world.tick = renewed;
        world.advance_island_tick();
        assert!(
            world
                .hostilities
                .contains(&(colonial.into(), pirates.into()))
        );
        // Approach an actual generated adult male; talking never makes him a recruit.
        world.hostilities.clear();
        world.policies.clear();
        let male = world
            .actors
            .values()
            .find(|a| {
                a.instance_id != "character.protagonist.captain"
                    && a.person.as_ref().is_some_and(|p| {
                        p.sex == PersonSex::Male && p.age.is_some_and(|age| age >= 18)
                    })
            })
            .unwrap()
            .instance_id
            .clone();
        assert!(world.island_person_news(&male).is_empty());
        assert!(world.approach_island_person(&male));
        for _ in 0..128 {
            if world.can_talk_island_person(&male) {
                break;
            }
            world.advance_island_tick();
        }
        assert!(!world.talk_island_person(&male).is_empty());
        assert!(!world.island_person_news(&male).is_empty());
        assert!(!world.recruit_island_person(&male));
        let captain = world.positions["character.protagonist.captain"];
        let far = world
            .navigation
            .walkable
            .iter()
            .copied()
            .find(|p| {
                p.x.abs_diff(captain.x) + p.y.abs_diff(captain.y) > 6
                    && world.navigation.path(captain, *p).is_some()
            })
            .unwrap();
        assert!(world.move_island_party(far));
        for _ in 0..128 {
            if !world
                .travel_orders
                .contains_key("character.protagonist.captain")
            {
                break;
            }
            world.advance_island_tick();
        }
        assert!(
            world.island_person_news(&male).is_empty(),
            "news is not a remote live dashboard"
        );
    }

    fn recruitment_fixture() -> (FactionWorld, Vec<String>) {
        let mut world = main_scenario_world();
        assert!(
            advance_until(&mut world, 4000, |w| w
                .actors
                .values()
                .any(|a| a.definition_id == "actor_def.pirates.deckhand")),
            "the tide quay raises a deckhand"
        );
        let template_id = world
            .actors
            .values()
            .find(|a| a.definition_id == "actor_def.pirates.deckhand")
            .unwrap()
            .instance_id
            .clone();
        let template = world.actors[&template_id].clone();
        let combat = world.unit_combat[&template_id].clone();
        let mut ids = vec![template_id];
        for i in 1..5 {
            let id = format!("test.recruit.{i}");
            let mut actor = template.clone();
            actor.instance_id = id.clone();
            world.actors.insert(id.clone(), actor);
            world.unit_combat.insert(id.clone(), combat.clone());
            ids.push(id);
        }
        let faction = world.factions.get_mut("faction.pirates.prototype").unwrap();
        faction.population_used += 4;
        faction.population_capacity = 20;
        for (i, id) in ids.iter().enumerate() {
            world.actors.get_mut(id).unwrap().person = Some(NamedPerson {
                id: id.clone(),
                display_name: format!("Test Deckhand {i}"),
                alive_today: true,
                sex: PersonSex::Female,
                age: Some(20),
                recruitment_offer: "A fair share, and I keep my sword.".into(),
                ..Default::default()
            });
            world
                .positions
                .insert(id.clone(), world.positions["character.protagonist.captain"]);
        }
        world.policies.clear();
        world.hostilities.clear();
        world.travel_orders.clear();
        for f in world.factions.values_mut() {
            for b in f.buildings.values_mut() {
                b.operational = false;
            }
        }
        (world, ids)
    }

    #[test]
    fn holding_salvage_actual_siege_creates_collectible_persistent_ruins() {
        let mut world = main_scenario_world();
        let captain = "character.protagonist.captain";
        world.positions.insert(
            captain.into(),
            world.salvage_caches["salvage.wreck"].position,
        );
        world.salvage_foothold().unwrap();
        for _ in 0..60_000 {
            world.advance_island_tick();
            if world.salvage_caches.len() > 1 {
                break;
            }
        }
        let (id, cache) = world
            .salvage_caches
            .iter()
            .find(|(id, _)| id.as_str() != "salvage.wreck")
            .map(|(id, c)| (id.clone(), c.clone()))
            .expect("real siege produced ruined holding salvage");
        assert!(
            !world.factions[&cache.source_faction]
                .buildings
                .contains_key(&cache.source_building_id)
        );
        assert_eq!(cache.initial_amount, 4 + 4 * cache.level);
        let before = world.factions["faction.michael"].resources["resource.salvage"];
        assert_eq!(before, 20); // Destruction never credits Michael remotely.
        let unchanged = world.clone();
        assert!(world.salvage_foothold().is_err());
        assert_eq!(world, unchanged);
        assert!(world.move_island_party(cache.position));
        for _ in 0..100 {
            world.advance_island_tick();
            if world.positions[captain] == cache.position {
                break;
            }
        }
        assert_eq!(world.nearby_salvage_id(), Some(id.as_str()));
        world.salvage_foothold().unwrap();
        assert_eq!(
            world.factions["faction.michael"].resources["resource.salvage"],
            before + cache.remaining
        );
        assert_eq!(world.salvage_caches[&id].remaining, 0);
        let restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(restored, world);
        assert_eq!(
            restored.salvage_caches[&id].initial_amount,
            cache.initial_amount
        );
    }

    #[test]
    fn holding_salvage_legacy_migration_never_refills_collected_materials() {
        let mut world = main_scenario_world();
        let mut saved: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        saved["world"]
            .as_object_mut()
            .unwrap()
            .remove("salvage_caches");
        saved["world"]["foothold_cache"] =
            serde_json::json!({"position":{"x":20,"y":19},"remaining":0});
        let migrated = FactionWorld::load_json(&saved.to_string()).unwrap();
        assert_eq!(migrated.salvage_caches.len(), 1);
        assert_eq!(migrated.salvage_caches["salvage.wreck"].remaining, 0);
        let mut both: serde_json::Value =
            serde_json::from_str(&migrated.save_json().unwrap()).unwrap();
        both["world"]["foothold_cache"] =
            serde_json::json!({"position":{"x":20,"y":19},"remaining":20});
        let restored = FactionWorld::load_json(&both.to_string()).unwrap();
        assert_eq!(restored.salvage_caches, migrated.salvage_caches);
        assert!(!restored.save_json().unwrap().contains("foothold_cache"));
    }

    #[test]
    fn holding_salvage_does_not_refund_construction_or_duplicate_destruction() {
        let mut world = mechanical_workshop_fixture();
        let site = "site.michael.field_workshop";
        let workshop = world.factions["faction.michael"].buildings[site].clone();
        let mut unfinished = workshop.clone();
        unfinished.construction = Some(BuildingWork {
            builder_id: "character.protagonist.captain".into(),
            remaining_ticks: 10,
            reserved_costs: BTreeMap::new(),
        });
        world.record_holding_salvage(&unfinished);
        assert_eq!(world.salvage_caches.len(), 1);
        world.record_holding_salvage(&workshop);
        world.record_holding_salvage(&workshop);
        assert_eq!(world.salvage_caches.len(), 2);
        let id = format!("salvage.ruins.{site}.{}", world.tick);
        assert_eq!(world.salvage_caches[&id].remaining, 4);
        world.salvage_caches.get_mut(&id).unwrap().remaining = 0;
        world.record_holding_salvage(&workshop);
        assert_eq!(world.salvage_caches[&id].remaining, 0);
        world.tick += 1;
        world.record_holding_salvage(&workshop);
        assert_eq!(world.salvage_caches.len(), 3); // A later rebuilt instance has a distinct event key.
    }

    fn mechanical_workshop_fixture() -> FactionWorld {
        let mut world = main_scenario_world();
        world.policies.clear();
        world.hostilities.clear();
        let captain = "character.protagonist.captain";
        world.positions.insert(
            captain.into(),
            world.salvage_caches["salvage.wreck"].position,
        );
        world.salvage_foothold().unwrap();
        let entrance = IslandPoint { x: 19, y: 17 };
        world.positions.insert(captain.into(), entrance);
        world.build_foothold(entrance).unwrap();
        assert!(
            advance_until(&mut world, 8000, |w| w.factions["faction.michael"]
                .buildings
                .get("site.michael.field_workshop")
                .is_some_and(|b| b.operational)),
            "the workshop finishes"
        );
        world
    }

    /// Michael's one building can become something. Every faction grows its
    /// holdings through an AI policy; Michael has none, so before this the
    /// workshop was finished the moment it was finished and `build_foothold`
    /// answered "Michael already has a workshop site." for the rest of the
    /// campaign. Building it out costs salvage, takes real time, and buys a
    /// berth -- the only way the player's own force can grow at all.
    /// The island's own record of its wars, finally spoken by somebody.
    ///
    /// Every war opened, truce struck and faction ended has always been
    /// written to `diplomacy_notices` and read by nothing, so the war moved
    /// constantly and every person on the island went on saying the same
    /// standing sentence. A local repeats what touched their own people; a
    /// companion repeats anything, because she walks with Michael; and once a
    /// notice is a day old, everyone is back to the standing line.
    #[test]
    fn people_repeat_the_war_news_that_touches_them_and_companions_repeat_all_of_it() {
        let (mut world, ids) = recruitment_fixture();
        let local = ids[0].clone();
        world.talk_island_person(&local);
        let local_faction = world.actors[&local].faction_id.clone();
        world.positions.insert(
            "character.protagonist.captain".into(),
            world.positions[&local],
        );
        let standing = world.island_person_news(&local);
        assert!(!standing.is_empty());

        // News about somebody else's war is not this person's to repeat.
        world.record_diplomacy(
            vec!["faction.elves.prototype".into()],
            "The elves have come to open war with the fox people.".into(),
        );
        assert_eq!(world.island_person_news(&local), standing);

        // News about their own people is.
        world.record_diplomacy(
            vec![local_faction.clone()],
            "Our own people have made a truce.".into(),
        );
        assert_eq!(
            world.island_person_news(&local),
            "Our own people have made a truce."
        );

        // A day later it is old news and they are back to the standing line.
        world.tick += world.clock.ticks_per_day + 1;
        assert_eq!(world.island_person_news(&local), standing);

        // A companion hears everything, whoever it was about.
        let companion = ids
            .iter()
            .find(|id| {
                world.living_person(id).is_some_and(|person| {
                    person.sex == PersonSex::Female && !person.recruitment_offer.is_empty()
                })
            })
            .cloned()
            .expect("a recruitable woman");
        world.positions.insert(
            "character.protagonist.captain".into(),
            world.positions[&companion],
        );
        world.talk_island_person(&companion);
        assert!(world.recruit_island_person(&companion));
        world.record_diplomacy(
            vec!["faction.elves.prototype".into()],
            "The elves are finished. Their holdings are gone.".into(),
        );
        assert_eq!(
            world.island_person_news(&companion),
            "The elves are finished. Their holdings are gone."
        );
    }

    /// The berth a level buys is a berth the game will actually fill.
    ///
    /// `queue_foothold_machine` asked `machine_berths`, but the one gated door
    /// underneath it -- `enqueue_production` -- asked the base capacity, so
    /// every level past the first advertised a berth that was then refused,
    /// and refused as "Not enough salvage to build a mechanical dog" while the
    /// player was standing there with the salvage. A playtest bot caught it.
    #[test]
    fn a_level_of_workshop_buys_a_berth_the_gated_door_honours() {
        let mut world = mechanical_workshop_fixture();
        let site = "site.michael.field_workshop";
        world
            .factions
            .get_mut("faction.michael")
            .unwrap()
            .buildings
            .get_mut(site)
            .unwrap()
            .level = 2;
        let berths = world.machine_berths();
        assert_eq!(berths, world.rules.machinery.capacity + 1);
        world
            .factions
            .get_mut("faction.michael")
            .unwrap()
            .resources
            .insert("resource.salvage".into(), 1000);
        for expected in 1..=berths {
            world.queue_foothold_machine().unwrap();
            assert!(
                advance_until(&mut world, 20_000, |w| w.mechanical_followers().len()
                    == expected),
                "the workshop fills berth {expected} of {berths}"
            );
        }
        // And stops at the berths it actually has.
        assert!(world.queue_foothold_machine().is_err());
        // A save carrying the berth the level bought has to load back. This
        // was the third place that knew the cap, and refusing here corrupted
        // the campaign one save after the player used what he had paid for.
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
    }

    /// The island stops carrying every body it has ever made.
    ///
    /// The shrine may raise any corpse whenever it has room, so nothing ever
    /// removed one: a hundred-day campaign carried hundreds of the dead in its
    /// save and they held their names forever, which drained the persona pools
    /// until two living women shared one. Michael's own dead are exempt --
    /// her party slot is deliberately kept.
    #[test]
    fn corpses_decay_after_the_authored_span_and_michaels_dead_do_not() {
        let (mut world, ids) = recruitment_fixture();
        let companion = ids
            .iter()
            .find(|id| {
                world.living_person(id).is_some_and(|person| {
                    person.sex == PersonSex::Female && !person.recruitment_offer.is_empty()
                })
            })
            .cloned()
            .expect("a recruitable woman");
        let stranger = ids
            .iter()
            .find(|id| *id != &companion)
            .cloned()
            .expect("somebody else on the island");
        world.positions.insert(
            "character.protagonist.captain".into(),
            world.positions[&companion],
        );
        world.talk_island_person(&companion);
        assert!(world.recruit_island_person(&companion));

        for id in [&stranger, &companion] {
            let position = world.positions[id];
            let actor = world.actors.remove(id).unwrap();
            let combat = world.unit_combat.remove(id).unwrap();
            world.positions.remove(id);
            world.casualties.insert(
                id.clone(),
                IslandCasualty {
                    actor,
                    position,
                    death_tick: world.tick,
                    population_use: combat.population_use,
                },
            );
        }
        // The shrine cannot raise anyone, so decay is the only thing acting.
        for faction in world.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                building.operational = false;
            }
        }
        let span = u64::from(world.rules.madness.corpse_persist_days) * world.clock.ticks_per_day;
        world.decay_casualties();
        assert!(world.casualties.contains_key(&stranger));

        world.tick += span + 1;
        world.decay_casualties();
        assert!(!world.casualties.contains_key(&stranger));
        assert!(
            world.casualties.contains_key(&companion),
            "his own dead are kept"
        );
    }

    #[test]
    fn the_workshop_can_be_built_out_and_each_level_buys_a_berth() {
        let mut world = mechanical_workshop_fixture();
        let site = "site.michael.field_workshop";
        let berths = world.machine_berths();
        assert_eq!(world.factions["faction.michael"].buildings[site].level, 1);

        // Not for free, and not from across the island.
        let entrance = world.positions["character.protagonist.captain"];
        world.positions.insert(
            "character.protagonist.captain".into(),
            IslandPoint { x: 24, y: 18 },
        );
        assert!(world.develop_foothold().is_err());
        world
            .positions
            .insert("character.protagonist.captain".into(), entrance);

        let cost = world.foothold_development_cost();
        assert!(cost > 0);
        let before = world.stored_resource("faction.michael", "resource.salvage");
        let poor = {
            let mut poor = world.clone();
            poor.factions
                .get_mut("faction.michael")
                .unwrap()
                .resources
                .insert("resource.salvage".into(), cost - 1);
            poor
        };
        let unchanged = poor.clone();
        let mut poor = poor;
        assert!(poor.develop_foothold().is_err());
        assert_eq!(poor, unchanged);

        world.develop_foothold().unwrap();
        assert_eq!(
            world.stored_resource("faction.michael", "resource.salvage"),
            before - cost
        );
        // The work is real work, and it survives the save.
        assert!(
            world.factions["faction.michael"].buildings[site]
                .development
                .is_some()
        );
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
        assert!(world.queue_foothold_machine().is_err());
        assert!(
            advance_until(&mut world, 20_000, |w| w.factions["faction.michael"]
                .buildings[site]
                .level
                == 2),
            "the workshop reaches its second level"
        );
        assert_eq!(world.machine_berths(), berths + 1);
        assert_eq!(world.machine_foothold_costs().2 as usize, berths + 1);

        // And it stops where the pack says it stops.
        let max_level = world.rules.foothold_development.as_ref().unwrap().max_level;
        for _ in 0..max_level {
            world
                .factions
                .get_mut("faction.michael")
                .unwrap()
                .resources
                .insert("resource.salvage".into(), 1000);
            if world.develop_foothold().is_err() {
                break;
            }
            assert!(advance_until(&mut world, 20_000, |w| {
                w.factions["faction.michael"].buildings[site]
                    .development
                    .is_none()
            }));
        }
        assert_eq!(
            world.factions["faction.michael"].buildings[site].level,
            max_level
        );
        assert!(world.develop_foothold().is_err());
    }

    #[test]
    fn mechanical_dog_production_is_paid_persistent_and_equipment_bounded() {
        let mut world = mechanical_workshop_fixture();
        assert_eq!(world.machine_foothold_costs(), (4, 720, 3));
        world.queue_foothold_machine().unwrap();
        let paid = world.clone();
        assert!(world.queue_foothold_machine().is_err());
        assert_eq!(world, paid);
        let mut resumed = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        world.paused = true;
        let paused = world.clone();
        world.advance_island_tick();
        assert_eq!(world, paused);
        world.paused = false;
        for _ in 0..800 {
            assert_eq!(world.advance_island_tick(), resumed.advance_island_tick());
        }
        assert_eq!(world, resumed);
        let dog = world.mechanical_followers().pop().unwrap();
        assert_eq!(world.actors[&dog].actor_kind, "machine");
        assert!(world.actors[&dog].person.is_none());
        assert_eq!(world.unit_combat[&dog].population_use, 0);
        assert_eq!(world.factions["faction.michael"].population_used, 1);
        assert!(!world.assign_island_companion(&dog, 0));
        world.queue_foothold_machine().unwrap();
        assert!(
            advance_until(&mut world, 8000, |w| w.mechanical_followers().len() == 2),
            "the workshop turns out a second dog"
        );
        assert_eq!(
            world.factions["faction.michael"].resources["resource.salvage"],
            0
        );
        let empty = world.clone();
        assert!(world.queue_foothold_machine().is_err());
        assert_eq!(world, empty);
        // Additional hypothetical earned scrap isolates the equipment berth cap.
        world
            .factions
            .get_mut("faction.michael")
            .unwrap()
            .resources
            .insert("resource.salvage".into(), 20);
        world.queue_foothold_machine().unwrap();
        assert!(
            advance_until(&mut world, 8000, |w| w.mechanical_followers().len() == 3),
            "the workshop fills its three slots"
        );
        let full = world.clone();
        assert!(world.queue_foothold_machine().is_err());
        assert_eq!(world, full);
        FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
    }

    #[test]
    fn mechanical_dog_follows_without_a_party_slot_and_catches_up_after_birth() {
        let mut world = mechanical_workshop_fixture();
        world.queue_foothold_machine().unwrap();
        let captain = "character.protagonist.captain";
        let target = IslandPoint { x: 20, y: 20 };
        assert!(world.move_island_party(target));
        assert!(
            advance_until(&mut world, 8000, |w| !w.mechanical_followers().is_empty()),
            "the workshop turns out a dog"
        );
        for _ in 0..45 {
            world.advance_island_tick();
        }
        let dog = world.mechanical_followers().pop().unwrap();
        let point = world.positions[&dog];
        assert!(point.x.abs_diff(target.x) + point.y.abs_diff(target.y) <= 3);
        assert!(world.party.iter().all(String::is_empty));
        assert_eq!(world.positions[captain], target);
        let next = IslandPoint { x: 20, y: 18 };
        assert!(world.move_island_party(next));
        assert!(world.travel_orders.contains_key(&dog));
        for _ in 0..12 {
            world.advance_island_tick();
        }
        assert_eq!(world.positions[captain], next);
        let point = world.positions[&dog];
        assert!(point.x.abs_diff(next.x) + point.y.abs_diff(next.y) <= 3);
    }

    #[test]
    fn mechanical_dog_is_neither_madness_recruit_nor_midnight_corpse() {
        let mut world = mechanical_workshop_fixture();
        world.queue_foothold_machine().unwrap();
        assert!(
            advance_until(&mut world, 8000, |w| !w.mechanical_followers().is_empty()),
            "the workshop turns out a dog"
        );
        let dog = world.mechanical_followers().pop().unwrap();
        let cult = "faction.cthulhu.prototype";
        let shrine = world.factions[cult].buildings.values().next().unwrap();
        world
            .positions
            .insert(dog.clone(), world.navigation.destinations[&shrine.node_id]);
        for _ in 0..2000 {
            world.tick += 1;
            world.advance_madness();
        }
        assert_eq!(world.actors[&dog].madness, 0);
        let pirates = "faction.pirates.prototype";
        let site = world.factions[pirates]
            .buildings
            .keys()
            .next()
            .unwrap()
            .clone();
        let rule = scenario_production(pirates);
        world.enqueue_production(pirates, &site, rule).unwrap();
        for _ in 0..8000 {
            world.advance_production_tick();
            if world.actors.values().any(|a| a.faction_id == pirates) {
                break;
            }
        }
        let enemy = world
            .actors
            .iter()
            .find(|(_, a)| a.faction_id == pirates)
            .unwrap()
            .0
            .clone();
        world.positions.insert(enemy.clone(), world.positions[&dog]);
        world.unit_combat.get_mut(&dog).unwrap().health = 1;
        world
            .hostilities
            .insert((pirates.into(), "faction.michael".into()));
        world
            .hostilities
            .insert(("faction.michael".into(), pirates.into()));
        let enemy_health = world.unit_combat[&enemy].health;
        world.resolve_island_skirmish();
        assert_eq!(world.unit_combat[&enemy].health, enemy_health - 2);
        assert!(world.casualties.contains_key(&dog));
        world.tick =
            world.tick - world.tick % world.clock.ticks_per_day + world.clock.ticks_per_day;
        assert!(!world.return_midnight_casualties().iter().any(
            |e| matches!(e,FactionWorldEvent::MidnightReturned{actor_id,..} if actor_id == &dog)
        ));
        assert!(!world.actors.contains_key(&dog));
        FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
    }

    #[test]
    fn holding_repairs_use_paid_local_units_and_reassign_without_repaying() {
        let (mut world, _) = recruitment_fixture();
        let pirates = "faction.pirates.prototype";
        let faction = world.factions.get_mut(pirates).unwrap();
        faction.resources.insert("resource.provisions".into(), 10);
        faction.resources.insert("resource.coin".into(), 10);
        let building = faction.buildings.values_mut().next().unwrap();
        building.operational = true;
        building.health = 20;
        // These resources are the treasury AFTER the upgrade reservation.
        let development = main_scenario().rules.development.clone();
        let upgrade = development[&building.archetype_id].clone();
        building.development = Some(BuildingDevelopment {
            remaining_ticks: upgrade.ticks,
            reserved_costs: upgrade.costs.clone(),
            rule: upgrade,
        });
        let site = building.id.clone();
        world
            .policies
            .insert(pirates.into(), FactionPolicy::default());
        for _ in 0..100 {
            world.advance_island_tick();
            if world.factions[pirates].buildings[&site].repair.is_some() {
                break;
            }
        }
        let building = &world.factions[pirates].buildings[&site];
        let job = building
            .repair
            .clone()
            .expect("actual unit reached the quay and started work");
        assert!(building.operational);
        assert!(building.development.is_none());
        assert_eq!(building.level, 1);
        assert_eq!(building.max_health, 600);
        assert_eq!(world.factions[pirates].population_capacity, 20);
        assert_eq!(world.factions[pirates].resources["resource.coin"], 8);
        assert!(world.actively_repairing(&job.builder_id));
        assert!(world.island_firing_target(&job.builder_id).is_none());
        let mut resumed = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        resumed.paused = true;
        let paused = resumed.clone();
        resumed.advance_island_tick();
        assert_eq!(resumed, paused);
        // Losing the actual worker stops its paid job; a replacement inherits it.
        assert!(world.transfer_living_actor(&job.builder_id, "faction.michael"));
        world.advance_production_tick();
        assert_eq!(
            world.factions[pirates].buildings[&site]
                .repair
                .as_ref()
                .unwrap()
                .remaining_ticks,
            job.remaining_ticks
        );
        for _ in 0..4000 {
            world.advance_island_tick();
            if world.factions[pirates].buildings[&site].health > 20 {
                break;
            }
        }
        // One completed repair cycle, whatever the pack authors it to be worth.
        let gain = world.rules.repairs["site_archetype.pirates.tide_quay"].health_gain;
        assert_eq!(world.factions[pirates].buildings[&site].health, 20 + gain);
        assert!(world.factions[pirates].buildings[&site].repair.is_none());
        assert_eq!(world.factions[pirates].resources["resource.coin"], 8);
        FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
    }

    #[test]
    fn holding_repairs_michael_workshop_stops_when_away_and_caps_healing() {
        let mut world = main_scenario_world();
        world.policies.clear();
        let captain = "character.protagonist.captain";
        let faction = "faction.michael";
        let site = "site.michael.field_workshop";
        let entrance = IslandPoint { x: 19, y: 17 };
        world.positions.insert(
            captain.into(),
            world.salvage_caches["salvage.wreck"].position,
        );
        world.salvage_foothold().unwrap();
        world.positions.insert(captain.into(), entrance);
        world.build_foothold(entrance).unwrap();
        assert!(
            advance_until(&mut world, 8000, |w| w.factions[faction].buildings[site]
                .operational),
            "the workshop finishes"
        );
        world
            .factions
            .get_mut(faction)
            .unwrap()
            .buildings
            .get_mut(site)
            .unwrap()
            .health = 75;
        let gain = world.rules.repairs["site_archetype.michael.field_workshop"].health_gain;
        world
            .factions
            .get_mut(faction)
            .unwrap()
            .buildings
            .get_mut(site)
            .unwrap()
            // One cycle short of whole, so the cap is what stops the second
            // repair rather than an authored gain that happened to fit.
            .health = world.rules.foothold.building_health - gain;
        world.repair_foothold().unwrap();
        assert_eq!(world.foothold_repair_cost(), 2);
        assert_eq!(world.factions[faction].resources["resource.salvage"], 6);
        assert!(world.repair_foothold().is_err());
        world
            .positions
            .insert(captain.into(), IslandPoint { x: 20, y: 20 });
        for _ in 0..20 {
            world.advance_island_tick();
        }
        // Work does not advance while Michael is away, whatever the pack
        // authors a repair cycle to cost.
        let cycle = world.rules.repairs["site_archetype.michael.field_workshop"].ticks;
        assert_eq!(
            world.factions[faction].buildings[site]
                .repair
                .as_ref()
                .unwrap()
                .remaining_ticks,
            cycle
        );
        world = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        world.positions.insert(captain.into(), entrance);
        for _ in 0..cycle {
            world.advance_island_tick();
        }
        assert_eq!(
            world.factions[faction].buildings[site].health,
            world.rules.foothold.building_health
        );
        assert!(world.factions[faction].buildings[site].repair.is_none());
        assert!(world.repair_foothold().is_err());
        FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
    }

    #[test]
    fn shrine_madness_transfers_the_living_person_and_resumes_identically() {
        let (mut world, ids) = recruitment_fixture();
        let cult = "faction.cthulhu.prototype";
        let victim = &ids[0];
        let shrine = world
            .factions
            .get_mut(cult)
            .unwrap()
            .buildings
            .values_mut()
            .next()
            .unwrap();
        shrine.operational = true;
        let entrance = world.navigation.destinations[&shrine.node_id];
        world.positions.insert(victim.clone(), entrance);
        // Male and female organics are equally susceptible; this is not wooing.
        world
            .actors
            .get_mut(victim)
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .sex = PersonSex::Male;
        let original = world.actors[victim].clone();
        let health = world.unit_combat[victim].clone();
        for _ in 0..8 {
            world.advance_island_tick();
        }
        assert!(world.actors[victim].madness > 0);
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        world.paused = true;
        let paused = world.clone();
        assert!(world.advance_island_tick().is_empty());
        assert_eq!(world, paused);
        world.paused = false;
        let old_population = world.factions[&original.faction_id].population_used;
        let cult_population = world.factions[cult].population_used;
        let mut converted = false;
        for _ in 0..4000 {
            let events = world.advance_island_tick();
            assert_eq!(events, restored.advance_island_tick());
            assert_eq!(world, restored);
            if events.iter().any(|e| matches!(e, FactionWorldEvent::MadnessConverted {actor_id,..} if actor_id == victim)) {
                converted = true;
                break;
            }
        }
        assert!(converted);
        let actor = &world.actors[victim];
        assert_eq!(actor.faction_id, cult);
        assert_eq!(actor.person, original.person);
        assert_eq!(actor.provenance, original.provenance);
        assert!(!actor.undead);
        assert_eq!(world.unit_combat[victim], health);
        assert!(!world.travel_orders.contains_key(victim));
        assert_eq!(
            world.factions[&original.faction_id].population_used,
            old_population - health.population_use
        );
        // Native production may have completed before the comparison window.
        assert_eq!(
            world.factions[cult].population_used,
            cult_population + health.population_use
        );
        assert_eq!(world.island_madness_stage(victim), "converted");
        FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
    }

    #[test]
    fn shrine_madness_respects_loyalty_machines_and_escape() {
        let (mut world, ids) = recruitment_fixture();
        let cult = "faction.cthulhu.prototype";
        let shrine = world
            .factions
            .get_mut(cult)
            .unwrap()
            .buildings
            .values_mut()
            .next()
            .unwrap();
        shrine.operational = true;
        let entrance = world.navigation.destinations[&shrine.node_id];
        for id in &ids {
            world.positions.insert(id.clone(), entrance);
        }
        world
            .actors
            .get_mut(&ids[0])
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .loyal_to_michael = true;
        // Unknown future machine definitions opt out even if given a person record.
        world.actors.get_mut(&ids[1]).unwrap().definition_id =
            "actor_def.michael.clockwork_dog".into();
        world
            .positions
            .insert("character.protagonist.captain".into(), entrance);
        for _ in 0..2000 {
            world.tick += 1;
            world.advance_madness();
        }
        assert_eq!(world.actors[&ids[0]].madness, 0);
        assert_eq!(world.actors[&ids[1]].madness, 0);
        assert_eq!(world.actors["character.protagonist.captain"].madness, 0);
        assert_eq!(world.actors[&ids[2]].faction_id, cult);
        let escaped = &ids[0];
        let person = world
            .actors
            .get_mut(escaped)
            .unwrap()
            .person
            .as_mut()
            .unwrap();
        person.loyal_to_michael = false;
        // Haunted, on the pack's scale rather than a number that was only
        // ever above the old warning threshold.
        let warning = world.rules.madness.warning_threshold;
        world.actors.get_mut(escaped).unwrap().madness = warning;
        assert_eq!(world.island_madness_stage(escaped), "whisper_haunted");
        world
            .positions
            .insert(escaped.clone(), IslandPoint { x: 20, y: 18 });
        for _ in 0..2000 {
            world.tick += 1;
            world.advance_madness();
        }
        assert_eq!(world.actors[escaped].madness, 0);
        // Michael can recover a living convert through the same Talk/Join path.
        world
            .actors
            .get_mut(&ids[2])
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .discussed = true;
        assert!(world.recruit_island_person(&ids[2]));
        assert_eq!(world.actors[&ids[2]].madness, 0);
        for _ in 0..2000 {
            world.tick += 1;
            world.advance_madness();
        }
        assert_eq!(world.actors[&ids[2]].faction_id, "faction.michael");
    }

    #[test]
    fn home_defense_recalls_one_sieger_on_cadence_and_releases_after_threat() {
        let (mut world, ids) = recruitment_fixture();
        let pirates = "faction.pirates.prototype";
        let captain = "character.protagonist.captain";
        let home = IslandPoint { x: 10, y: 16 };
        let siege = IslandPoint { x: 20, y: 16 };
        let building = world
            .factions
            .get_mut(pirates)
            .unwrap()
            .buildings
            .values_mut()
            .next()
            .unwrap();
        building.operational = true;
        let home_node = building.node_id.clone();
        let defense = format!("defend.{}", building.id);
        world
            .navigation
            .destinations
            .insert(home_node.clone(), home);
        let fort_node = world.factions["faction.colonial_powers.prototype"]
            .buildings
            .values()
            .next()
            .unwrap()
            .node_id
            .clone();
        world.navigation.destinations.insert(fort_node, siege);
        world
            .navigation
            .destinations
            .insert("test.outward".into(), IslandPoint { x: 24, y: 16 });
        world.policies.insert(
            pirates.into(),
            FactionPolicy {
                development: BTreeMap::new(),
                income_per_tick: BTreeMap::new(),
                storage_caps: BTreeMap::new(),
                production: BTreeMap::new(),
                objectives: vec![candidate("test.advance", "test.outward", 20, 0)],
            },
        );
        world
            .hostilities
            .insert((pirates.into(), "faction.michael".into()));
        world
            .hostilities
            .insert((pirates.into(), "faction.colonial_powers.prototype".into()));
        world
            .positions
            .insert(captain.into(), IslandPoint { x: 10, y: 19 });
        for id in &ids {
            world.positions.insert(id.clone(), siege);
            world.order_move(id, IslandPoint { x: 24, y: 16 }).unwrap();
        }
        world.tick = 7;
        let mut harmless = world.clone();
        harmless.tick = 8;
        let captain_definition = harmless.actors[captain].definition_id.clone();
        harmless
            .combat_profiles
            .get_mut(&captain_definition)
            .unwrap()
            .damage = 0;
        harmless.advance_island_tick();
        assert!(
            ids.iter()
                .all(|id| harmless.actors[id].current_assignment_id.as_deref() != Some(&defense)),
            "healthy but nonattacking visitors are not home threats"
        );
        world.advance_island_tick();
        assert!(
            ids.iter().all(|id| world.positions[id] == siege),
            "siegers hold until recall cadence"
        );
        assert!(
            ids.iter()
                .all(|id| world.actors[id].current_assignment_id.as_deref() != Some(&defense))
        );
        let events = world.advance_island_tick();
        let defenders: Vec<_> = ids
            .iter()
            .filter(|id| world.actors[*id].current_assignment_id.as_deref() == Some(&defense))
            .cloned()
            .collect();
        assert_eq!(
            defenders.len(),
            1,
            "one hostile reserves only one of five soldiers"
        );
        let defender = &defenders[0];
        assert_ne!(
            world.positions[defender], siege,
            "recall must leave a building siege, not only change assignment"
        );
        assert!(events.iter().any(|event| matches!(event, FactionWorldEvent::ActorMoved {actor_id,..} if actor_id == defender)));
        let before_pause = world.clone();
        world.paused = true;
        assert!(world.advance_island_tick().is_empty());
        world.paused = false;
        assert_eq!(world, before_pause);
        for _ in 0..12 {
            world.advance_island_tick();
        }
        assert_eq!(world.positions[defender], home);
        assert_eq!(
            world.actors[defender].current_assignment_id.as_deref(),
            Some(defense.as_str())
        );
        assert!(!world.travel_orders.contains_key(defender));
        // Diplomacy/removal of pressure releases the same defender through the
        // ordinary authored outward objective, without a separate controller.
        world
            .hostilities
            .remove(&(pirates.into(), "faction.michael".into()));
        world.advance_island_tick();
        assert_ne!(world.positions[defender], home);
        assert_ne!(
            world.actors[defender].current_assignment_id.as_deref(),
            Some(defense.as_str())
        );
        assert!(!world.travel_orders.contains_key(captain));
    }

    fn midnight_fixture() -> (FactionWorld, String) {
        let (mut world, ids) = recruitment_fixture();
        let victim = ids[0].clone();
        world.clock.ticks_per_day = 4;
        // The four-tick day only means anything from a known phase, and
        // recruitment now costs as many ticks as the island's production
        // actually needs rather than a hard-coded two.
        world.tick += (6 - world.tick % 4) % 4;
        world.talk_island_person(&victim);
        assert!(world.recruit_island_person(&victim));
        assert!(world.assign_island_companion(&victim, 0));
        world
            .actors
            .get_mut(&victim)
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .death_memory
            .killed_by_player_count = 7;
        world
            .positions
            .insert(victim.clone(), IslandPoint { x: 12, y: 16 });
        world
            .positions
            .insert(ids[4].clone(), IslandPoint { x: 12, y: 16 });
        world.unit_combat.get_mut(&victim).unwrap().health = 1;
        world.unit_combat.get_mut(&ids[4]).unwrap().next_attack_tick = 0;
        world
            .hostilities
            .insert(("faction.pirates.prototype".into(), "faction.michael".into()));
        world.resolve_island_skirmish();
        assert!(world.casualties.contains_key(&victim));
        world
            .factions
            .get_mut("faction.cthulhu.prototype")
            .unwrap()
            .buildings
            .values_mut()
            .next()
            .unwrap()
            .operational = true;
        (world, victim)
    }

    #[test]
    fn foothold_salvage_construction_and_owned_restoration_are_paid_local_and_persistent() {
        let (mut world, ids) = recruitment_fixture();
        let captain = "character.protagonist.captain";
        let workshop = "site.michael.field_workshop";
        let woman = &ids[0];
        world.talk_island_person(woman);
        assert!(world.recruit_island_person(woman));
        assert!(world.assign_island_companion(woman, 0));
        assert!(world.factions["faction.michael"].resources.is_empty());
        let unchanged = world.clone();
        assert!(world.salvage_foothold().is_err());
        assert_eq!(world, unchanged);
        let cache = world.salvage_caches["salvage.wreck"].position;
        assert!(world.move_island_party(cache));
        for _ in 0..40 {
            world.advance_island_tick();
        }
        assert_eq!(world.positions[captain], cache);
        world.salvage_foothold().unwrap();
        assert_eq!(
            world.factions["faction.michael"].resources["resource.salvage"],
            20
        );
        let unchanged = world.clone();
        assert!(world.salvage_foothold().is_err());
        assert_eq!(world, unchanged);
        let entrance = IslandPoint { x: 19, y: 17 };
        assert!(world.move_island_party(entrance));
        for _ in 0..8 {
            world.advance_island_tick();
        }
        // The party member must not stand in the actual new wall footprint.
        let side = IslandPoint { x: 18, y: 17 };
        world.order_move(woman, side).unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
        world.build_foothold(entrance).unwrap();
        assert!(!world.policies.contains_key("faction.michael"));
        assert_eq!(
            world.factions["faction.michael"].resources["resource.salvage"],
            8
        );
        assert!(!world.factions["faction.michael"].buildings[workshop].operational);
        let unchanged = world.clone();
        assert!(world.build_foothold(entrance).is_err());
        assert_eq!(world, unchanged);
        world.actors.get_mut(woman).unwrap().undead = true;
        assert!(world.restore_foothold_person(woman).is_err());
        world.paused = true;
        let frozen = world.clone();
        world.advance_island_tick();
        assert_eq!(world, frozen);
        world.paused = false;
        // Stop building when Michael leaves; pending work survives save/load.
        assert!(world.move_island_party(IslandPoint { x: 24, y: 18 }));
        for _ in 0..12 {
            world.advance_island_tick();
        }
        let remaining = world.factions["faction.michael"].buildings[workshop]
            .construction
            .as_ref()
            .unwrap()
            .remaining_ticks;
        for _ in 0..4 {
            world.advance_island_tick();
        }
        assert_eq!(
            world.factions["faction.michael"].buildings[workshop]
                .construction
                .as_ref()
                .unwrap()
                .remaining_ticks,
            remaining
        );
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert!(world.move_island_party(entrance));
        assert!(restored.move_island_party(entrance));
        for _ in 0..8000 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
            if world.factions["faction.michael"].buildings[workshop].operational {
                break;
            }
        }
        assert!(world.factions["faction.michael"].buildings[workshop].operational);
        assert!(
            world.factions["faction.michael"].buildings[workshop]
                .construction
                .is_none()
        );
        assert_eq!(
            world.factions["faction.michael"].buildings[workshop].level,
            1
        );
        let identity = world.actors[woman].clone();
        world.unit_combat.get_mut(woman).unwrap().health = 1;
        world.restore_foothold_person(woman).unwrap();
        assert!(!world.actors[woman].undead);
        assert_eq!(world.actors[woman].person, identity.person);
        assert_eq!(world.actors[woman].provenance, identity.provenance);
        assert_eq!(
            world.factions["faction.michael"].resources["resource.salvage"],
            4
        );
        let unchanged = world.clone();
        assert!(world.restore_foothold_person(woman).is_err());
        assert!(world.restore_foothold_person(&ids[1]).is_err());
        assert_eq!(world, unchanged);
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_ok());
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"]
            .as_object_mut()
            .unwrap()
            .remove("salvage_caches");
        assert!(
            FactionWorld::load_json(&legacy.to_string())
                .unwrap()
                .salvage_caches
                .is_empty()
        );
        assert!(world.move_island_party(IslandPoint { x: 24, y: 18 }));
        for _ in 0..12 {
            world.advance_island_tick();
        }
        world.positions.insert(ids[4].clone(), entrance);
        world.unit_combat.get_mut(&ids[4]).unwrap().next_attack_tick = 0;
        world
            .policies
            .insert("faction.pirates.prototype".into(), FactionPolicy::default());
        world
            .hostilities
            .insert(("faction.pirates.prototype".into(), "faction.michael".into()));
        world
            .factions
            .get_mut("faction.michael")
            .unwrap()
            .buildings
            .get_mut(workshop)
            .unwrap()
            .health = 1;
        world.resolve_island_skirmish();
        assert!(
            !world.factions["faction.michael"]
                .buildings
                .contains_key(workshop)
        );
        assert!(world.living_actor(captain) && world.living_actor(woman));
        assert!(!world.eliminated_factions.contains("faction.michael"));
        assert!(!world.navigation.building_obstacles.contains_key(workshop));
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_ok());
    }

    #[test]
    fn midnight_preserves_identity_and_inactive_slot_until_explicit_reacquisition() {
        let (mut world, victim) = midnight_fixture();
        let dead = world.casualties[&victim].clone();
        let previous_population = world.factions["faction.cthulhu.prototype"].population_used;
        let day_before = world.day();
        world.advance_island_tick();
        assert_eq!(world.minute_of_day(), 1080);
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        let events = world.advance_island_tick();
        assert_eq!(events, restored.advance_island_tick());
        assert_eq!(world, restored);
        assert!(events.iter().any(|e| matches!(e, FactionWorldEvent::MidnightReturned {actor_id,..} if actor_id == &victim)));
        assert_eq!(world.day(), day_before + 1);
        assert_eq!(world.minute_of_day(), 0);
        assert!(world.actors[&victim].undead);
        assert_eq!(world.actors[&victim].provenance, dead.actor.provenance);
        let mut expected_person = dead.actor.person.unwrap();
        expected_person.return_at_midnight();
        assert_eq!(
            world.actors[&victim].person.as_ref(),
            Some(&expected_person)
        );
        assert_eq!(world.party[0], victim);
        assert_eq!(
            world.factions["faction.cthulhu.prototype"].population_used,
            previous_population + dead.population_use
        );
        assert!(world.return_midnight_casualties().is_empty());
        let enemy_goal = IslandPoint { x: 15, y: 16 };
        world.order_move(&victim, enemy_goal).unwrap();
        let enemy_order = world.travel_orders[&victim].clone();
        assert!(world.move_island_party(IslandPoint { x: 9, y: 16 }));
        assert_eq!(world.travel_orders[&victim], enemy_order);
        let mut dismissed = world.clone();
        assert!(dismissed.dismiss_island_companion(0));
        assert_eq!(dismissed.travel_orders[&victim], enemy_order);
        let mut eliminated = world.clone();
        eliminated
            .eliminate_faction("faction.cthulhu.prototype")
            .unwrap();
        assert!(eliminated.casualties.contains_key(&victim));
        assert!(FactionWorld::load_json(&eliminated.save_json().unwrap()).is_ok());
        assert!(eliminated.return_midnight_casualties().is_empty());
        // Reacquisition remains a real nearby interaction, even during hostility.
        world.positions.insert(
            "character.protagonist.captain".into(),
            world.positions[&victim],
        );
        world.paused = true;
        let frozen = world.clone();
        world.advance_island_tick();
        assert_eq!(world, frozen);
        assert!(!world.talk_island_person(&victim).is_empty());
        assert!(world.recruit_island_person(&victim));
        assert_eq!(world.actors[&victim].faction_id, "faction.michael");
        assert!(world.actors[&victim].undead); // Allegiance is not biological resurrection.
        assert!(
            world
                .plan_party_move(IslandPoint { x: 9, y: 16 })
                .unwrap()
                .iter()
                .any(|(id, _)| id == &victim)
        );
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_ok());
    }

    #[test]
    fn midnight_defers_blocked_corpses_and_respects_elimination_clock_and_midnight_deaths() {
        let (mut world, victim) = midnight_fixture();
        let corpse = world.casualties[&victim].position;
        let walkable = world.navigation.walkable.clone();
        world.navigation.walkable.remove(&corpse);
        world.advance_island_tick();
        world.advance_island_tick();
        assert!(world.casualties.contains_key(&victim));
        world.navigation.walkable = walkable;
        // A death on the midnight tick belongs to the new day, not this return.
        world.casualties.get_mut(&victim).unwrap().death_tick = world.tick;
        assert!(world.return_midnight_casualties().is_empty());
        for _ in 0..4 {
            world.advance_island_tick();
        }
        assert!(world.actors[&victim].undead);
        let mut invalid = world.clone();
        invalid.clock.ticks_per_day = 0;
        assert_eq!(
            FactionWorld::load_json(&invalid.save_json().unwrap()),
            Err("invalid_saved_clock".into())
        );
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"].as_object_mut().unwrap().remove("clock");
        // A genuine legacy actor lacked this flag and was not a retained foreign slot.
        legacy["world"]["party"] = serde_json::json!(["", "", "", ""]);
        for actor in legacy["world"]["actors"]
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            actor.as_object_mut().unwrap().remove("undead");
        }
        let loaded = FactionWorld::load_json(&legacy.to_string()).unwrap();
        assert_eq!(loaded.clock.ticks_per_day, 1440);
        assert!(loaded.actors.values().all(|a| !a.undead));
        let (mut lost, id) = midnight_fixture();
        lost.eliminate_faction("faction.cthulhu.prototype").unwrap();
        for _ in 0..8 {
            lost.advance_island_tick();
        }
        assert!(lost.casualties.contains_key(&id));
        assert!(!lost.actors.contains_key(&id));
        let (mut defeated, _) = midnight_fixture();
        let captain = "character.protagonist.captain";
        defeated.unit_combat.get_mut(captain).unwrap().health = 1;
        for combat in defeated.unit_combat.values_mut() {
            combat.next_attack_tick = 0;
        }
        defeated.resolve_island_skirmish();
        assert!(defeated.casualties.contains_key(captain));
        for _ in 0..8 {
            defeated.advance_island_tick();
        }
        assert!(defeated.casualties.contains_key(captain));
        assert!(!defeated.actors.contains_key(captain));
    }

    #[test]
    fn active_companion_closes_and_strikes_without_overriding_travel() {
        const CAPTAIN: &str = "character.protagonist.captain";
        let (mut world, ids) = recruitment_fixture();
        let companion = &ids[0];
        let enemy = &ids[1];
        assert!(!world.talk_island_person(companion).is_empty());
        assert!(world.recruit_island_person(companion));
        assert!(world.assign_island_companion(companion, 0));
        for (id, point) in &mut world.positions {
            *point = if id == CAPTAIN {
                IslandPoint { x: 8, y: 16 }
            } else if id == companion {
                IslandPoint { x: 9, y: 16 }
            } else if id == enemy {
                IslandPoint { x: 11, y: 16 }
            } else {
                IslandPoint { x: 30, y: 10 }
            };
        }
        world
            .hostilities
            .insert(("faction.michael".into(), "faction.pirates.prototype".into()));
        let start = world.positions[companion];
        let mut behind = world.clone();
        behind
            .positions
            .insert(companion.clone(), IslandPoint { x: 7, y: 16 });
        behind
            .positions
            .insert(enemy.clone(), IslandPoint { x: 10, y: 16 });
        let mut flanked = false;
        for _ in 0..6 {
            let events = behind.advance_island_tick();
            assert_ne!(behind.positions[companion], behind.positions[CAPTAIN]);
            flanked |= events.iter().any(|event| matches!(event, FactionWorldEvent::UnitStruck {attacker_id,target_id,..} if attacker_id == companion && target_id == enemy));
        }
        assert!(
            flanked,
            "companion must route around Michael instead of remaining stuck behind him"
        );
        let mut neutral = world.clone();
        neutral.hostilities.clear();
        neutral.advance_companion_defense();
        assert!(neutral.travel_orders.is_empty());
        let mut distant = world.clone();
        distant
            .positions
            .insert(enemy.clone(), IslandPoint { x: 14, y: 16 });
        distant.advance_companion_defense();
        assert!(distant.travel_orders.is_empty());
        let mut off_party = world.clone();
        assert!(off_party.dismiss_island_companion(0));
        off_party.advance_companion_defense();
        assert!(off_party.travel_orders.is_empty());
        let mut ordered = world.clone();
        assert!(ordered.move_island_party(IslandPoint { x: 8, y: 20 }));
        let explicit_orders = ordered.clone();
        ordered.advance_companion_defense();
        assert_eq!(ordered, explicit_orders);
        world.paused = true;
        let paused = world.clone();
        world.advance_island_tick();
        assert_eq!(world, paused);
        world.paused = false;
        let events = world.advance_island_tick();
        assert_ne!(world.positions[companion], start);
        assert_eq!(world.positions[companion], IslandPoint { x: 10, y: 16 });
        assert!(events.iter().any(|event| matches!(event, FactionWorldEvent::UnitStruck {attacker_id,target_id,..} if attacker_id == companion && target_id == enemy)));
        assert!(!world.travel_orders.contains_key(companion));
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
    }

    #[test]
    fn approach_follows_moving_person_with_party_and_preserves_save() {
        let (mut world, ids) = recruitment_fixture();
        let captain = "character.protagonist.captain";
        assert!(!world.talk_island_person(&ids[1]).is_empty());
        assert!(world.recruit_island_person(&ids[1]));
        assert!(world.assign_island_companion(&ids[1], 0));
        world
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 20, y: 16 });
        world
            .order_move(&ids[0], IslandPoint { x: 25, y: 16 })
            .unwrap();
        let positions = world.positions.clone();
        assert!(world.approach_island_person(&ids[0]));
        assert_eq!(world.positions, positions); // command never teleports
        assert!(world.travel_orders.contains_key(&ids[1]));
        world.advance_island_tick();
        assert_ne!(world.positions[&ids[0]], positions[&ids[0]]); // NPC was not frozen
        let moved = world.positions[captain];
        assert_eq!(
            moved.x.abs_diff(positions[captain].x) + moved.y.abs_diff(positions[captain].y),
            1
        );
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        for _ in 0..40 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
            if world.approach_target.is_none() {
                break;
            }
        }
        assert!(world.approach_target.is_none());
        assert!(world.can_talk_island_person(&ids[0]));
        assert!(!world.actors[&ids[0]].person.as_ref().unwrap().discussed);
        assert!(!world.travel_orders.contains_key(captain));
        assert!(!world.travel_orders.contains_key(&ids[1]));
    }

    #[test]
    fn approach_arrival_uses_post_movement_target_and_rejects_without_mutation() {
        let (mut world, ids) = recruitment_fixture();
        let captain = "character.protagonist.captain";
        world
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 12, y: 16 });
        assert!(world.approach_island_person(&ids[0]));
        let before = world.clone();
        assert!(!world.approach_island_person("missing"));
        assert!(!world.approach_island_person(captain));
        assert!(!world.move_island_party(IslandPoint { x: -99, y: -99 }));
        assert!(!world.aim_carbine("missing"));
        assert_eq!(world, before);
        // Simulate a previous tick bringing her into range just before she walks
        // away. Tracking must survive until the final position is checked.
        world
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 10, y: 16 });
        world
            .order_move(&ids[0], IslandPoint { x: 15, y: 16 })
            .unwrap();
        world.advance_island_tick();
        assert_eq!(world.positions[captain], IslandPoint { x: 8, y: 16 });
        assert_eq!(world.positions[&ids[0]], IslandPoint { x: 11, y: 16 });
        assert_eq!(world.approach_target.as_deref(), Some(ids[0].as_str()));
        assert!(!world.can_talk_island_person(&ids[0]));
        world
            .order_move(captain, IslandPoint { x: 9, y: 16 })
            .unwrap();
        assert!(world.approach_target.is_none());
        assert!(world.approach_island_person(&ids[0]));
        assert!(world.aim_carbine(&ids[2]));
        assert!(world.approach_target.is_none());
        assert!(!world.travel_orders.contains_key(captain));
        assert!(world.approach_island_person(&ids[0]));
        world
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 10, y: 16 });
        world
            .actors
            .get_mut(&ids[0])
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .discussed = true;
        assert!(world.recruit_island_person(&ids[0]));
        assert!(world.approach_target.is_none());
        assert!(!world.travel_orders.contains_key(captain));
    }

    #[test]
    fn approach_clears_lost_targets_and_load_rejects_dangling_reference() {
        let (mut world, ids) = recruitment_fixture();
        world
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 20, y: 16 });
        assert!(world.approach_island_person(&ids[0]));
        let mut bad = world.clone();
        bad.approach_target = Some("missing".into());
        assert!(FactionWorld::load_json(&bad.save_json().unwrap()).is_err());
        let mut unreachable = world.clone();
        unreachable
            .navigation
            .walkable
            .insert(IslandPoint { x: 1000, y: 1000 });
        unreachable
            .positions
            .insert(ids[0].clone(), IslandPoint { x: 1000, y: 1000 });
        unreachable.advance_island_tick();
        assert!(unreachable.approach_target.is_none());
        assert!(
            !unreachable
                .travel_orders
                .contains_key("character.protagonist.captain")
        );
        world
            .eliminate_faction("faction.pirates.prototype")
            .unwrap();
        assert!(world.approach_target.is_none());
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_ok());
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"]
            .as_object_mut()
            .unwrap()
            .remove("approach_target");
        assert!(
            FactionWorld::load_json(&legacy.to_string())
                .unwrap()
                .approach_target
                .is_none()
        );
    }

    #[test]
    fn approach_target_death_cancels_tracking_before_save() {
        let (mut world, ids) = recruitment_fixture();
        let victim = &ids[0];
        let killer = &ids[2];
        world
            .positions
            .insert(victim.clone(), IslandPoint { x: 12, y: 16 });
        world
            .positions
            .insert(killer.clone(), IslandPoint { x: 12, y: 16 });
        world.actors.get_mut(killer).unwrap().faction_id =
            "faction.colonial_powers.prototype".into();
        world
            .factions
            .get_mut("faction.pirates.prototype")
            .unwrap()
            .population_used -= 1;
        world
            .factions
            .get_mut("faction.colonial_powers.prototype")
            .unwrap()
            .population_used += 1;
        world.unit_combat.get_mut(victim).unwrap().health = 1;
        world.hostilities.insert((
            "faction.colonial_powers.prototype".into(),
            "faction.pirates.prototype".into(),
        ));
        assert!(world.approach_island_person(victim));
        world.resolve_island_skirmish();
        assert!(world.casualties.contains_key(victim));
        assert!(world.approach_target.is_none());
        assert!(
            !world
                .travel_orders
                .contains_key("character.protagonist.captain")
        );
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_ok());
    }

    #[test]
    fn recruitment_transfers_five_people_without_clones_or_queue_loss_and_selects_four() {
        let (mut world, ids) = recruitment_fixture();
        let faction = "faction.pirates.prototype";
        let building = world.factions[faction]
            .buildings
            .keys()
            .next()
            .unwrap()
            .clone();
        world
            .factions
            .get_mut(faction)
            .unwrap()
            .buildings
            .get_mut(&building)
            .unwrap()
            .operational = true;
        let rule = scenario_production("faction.pirates.prototype");
        world.enqueue_production(faction, &building, rule).unwrap();
        world
            .factions
            .get_mut(faction)
            .unwrap()
            .buildings
            .get_mut(&building)
            .unwrap()
            .operational = false;
        let queue = world.factions[faction].buildings[&building]
            .production_queue
            .clone();
        let resources = world.factions[faction].resources.clone();
        let initial_population = world.factions[faction].population_used;
        let count = world.actors.len();
        for id in &ids {
            let actor_before = world.actors[id].clone();
            let state_before = world.unit_combat[id].clone();
            let position = world.positions[id];
            world.order_move(id, position).unwrap();
            world.player_attack_target = Some(id.clone());
            assert!(!world.recruit_island_person(id));
            assert!(!world.talk_island_person(id).is_empty());
            assert!(world.recruit_island_person(id));
            assert!(!world.recruit_island_person(id));
            assert_eq!(world.actors[id].provenance, actor_before.provenance);
            assert_eq!(world.unit_combat[id], state_before);
            assert_eq!(
                world.actors[id].person.as_ref().unwrap().display_name,
                actor_before.person.unwrap().display_name
            );
            assert!(world.actors[id].person.as_ref().unwrap().loyal_to_michael);
            let companion_line = world.talk_island_person(id);
            assert!(!companion_line.is_empty());
            assert_eq!(companion_line, world.island_person_dialogue(id));
            assert_ne!(
                companion_line,
                world.actors[id].person.as_ref().unwrap().recruitment_offer
            );
            assert!(!world.travel_orders.contains_key(id));
            assert!(
                !world
                    .navigation
                    .destinations
                    .contains_key(&format!("move.{id}"))
            );
            assert!(world.player_attack_target.is_none());
        }
        assert_eq!(world.actors.len(), count);
        assert_eq!(
            world.factions[faction].population_used,
            initial_population - 5
        );
        assert_eq!(
            world.factions[faction].buildings[&building].production_queue,
            queue
        );
        assert_eq!(world.factions[faction].resources, resources);
        assert_eq!(world.factions["faction.michael"].population_used, 6);
        assert_eq!(world.factions["faction.michael"].population_capacity, 6);
        assert!(world.party.iter().all(String::is_empty));
        for (slot, id) in ids.iter().take(4).enumerate() {
            assert!(world.assign_island_companion(id, slot));
        }
        assert!(!world.assign_island_companion(&ids[0], 1));
        assert!(!world.assign_island_companion(&ids[4], 4));
        assert!(world.assign_island_companion(&ids[4], 0));
        assert_eq!(world.actors[&ids[0]].faction_id, "faction.michael");
        assert!(world.dismiss_island_companion(1));
        assert_eq!(world.actors[&ids[1]].faction_id, "faction.michael");
        world.eliminate_faction(faction).unwrap();
        assert!(ids.iter().all(|id| world.actors.contains_key(id)));
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
    }

    #[test]
    fn recruitment_revalidates_access_identity_and_population_without_mutation() {
        let (world, ids) = recruitment_fixture();
        let id = &ids[0];
        for mode in 0..8 {
            let mut invalid = world.clone();
            invalid.talk_island_person(id);
            match mode {
                0 => {
                    invalid
                        .actors
                        .get_mut(id)
                        .unwrap()
                        .person
                        .as_mut()
                        .unwrap()
                        .sex = PersonSex::Male
                }
                1 => invalid.actors.get_mut(id).unwrap().person = None,
                2 => {
                    invalid
                        .actors
                        .get_mut(id)
                        .unwrap()
                        .person
                        .as_mut()
                        .unwrap()
                        .age = None
                }
                3 => invalid
                    .actors
                    .get_mut(id)
                    .unwrap()
                    .person
                    .as_mut()
                    .unwrap()
                    .recruitment_offer
                    .clear(),
                4 => {
                    invalid
                        .positions
                        .insert(id.clone(), IslandPoint { x: 30, y: 16 });
                }
                5 => {
                    invalid
                        .unit_combat
                        .get_mut("character.protagonist.captain")
                        .unwrap()
                        .health = 0
                }
                6 => {
                    invalid
                        .factions
                        .get_mut("faction.pirates.prototype")
                        .unwrap()
                        .population_used = 0
                }
                _ => {
                    invalid
                        .factions
                        .get_mut("faction.michael")
                        .unwrap()
                        .population_used = u32::MAX
                }
            }
            let before = invalid.clone();
            assert!(!invalid.recruit_island_person(id));
            assert_eq!(invalid, before);
        }
        let mut blocked = world.clone();
        let a = world.positions[id];
        blocked
            .positions
            .insert(id.clone(), IslandPoint { x: a.x + 2, y: a.y });
        blocked
            .navigation
            .walkable
            .remove(&IslandPoint { x: a.x + 1, y: a.y });
        assert!(blocked.talk_island_person(id).is_empty());
    }

    #[test]
    fn party_travel_is_atomic_persistent_and_retains_dead_identity() {
        let (mut world, ids) = recruitment_fixture();
        for (slot, id) in ids.iter().take(4).enumerate() {
            world.talk_island_person(id);
            assert!(world.recruit_island_person(id));
            assert!(world.assign_island_companion(id, slot));
        }
        let positions = world.positions.clone();
        assert!(world.move_island_party(IslandPoint { x: 12, y: 16 }));
        assert_eq!(world.positions, positions);
        let goals: BTreeSet<_> = world
            .travel_orders
            .values()
            .map(|id| world.navigation.destinations[id])
            .collect();
        assert_eq!(goals.len(), 5);
        let before = world.clone();
        assert!(!world.move_island_party(IslandPoint { x: -100, y: -100 }));
        assert_eq!(world, before);
        let stranded = &ids[0];
        let old = world.positions[stranded];
        world
            .positions
            .insert(stranded.clone(), IslandPoint { x: 1000, y: 1000 });
        world
            .navigation
            .walkable
            .insert(IslandPoint { x: 1000, y: 1000 });
        let before = world.clone();
        assert!(!world.move_island_party(IslandPoint { x: 12, y: 16 }));
        assert!(
            world
                .party_move_failure(IslandPoint { x: 12, y: 16 })
                .contains(&world.actors[stranded].person.as_ref().unwrap().display_name)
        );
        assert_eq!(world, before);
        world.positions.insert(stranded.clone(), old);
        world
            .navigation
            .walkable
            .remove(&IslandPoint { x: 1000, y: 1000 });
        world.paused = true;
        let paused = world.clone();
        world.advance_island_tick();
        assert_eq!(world, paused);
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
        world.paused = false;
        for _ in 0..10 {
            world.advance_island_tick();
        }
        assert_eq!(
            world.positions["character.protagonist.captain"],
            IslandPoint { x: 12, y: 16 }
        );
        assert!(world.travel_orders.is_empty());
        // Use the real combat death path, not a synthetic casualty replacement.
        let victim = &ids[0];
        let enemy = &ids[4];
        let pos = world.positions[victim];
        world.positions.insert(enemy.clone(), pos);
        world.unit_combat.get_mut(enemy).unwrap().next_attack_tick = 0;
        world.unit_combat.get_mut(victim).unwrap().health = 1;
        world
            .hostilities
            .insert(("faction.pirates.prototype".into(), "faction.michael".into()));
        // Keep victim as the unique target in reach.
        for id in ids.iter().skip(1).take(3).chain(std::iter::once(
            &"character.protagonist.captain".to_string(),
        )) {
            world
                .positions
                .insert(id.clone(), IslandPoint { x: 30, y: 16 });
        }
        world.resolve_island_skirmish();
        assert!(world.casualties.contains_key(victim));
        assert_eq!(&world.party[0], victim);
        assert!(
            world.casualties[victim]
                .actor
                .person
                .as_ref()
                .unwrap()
                .loyal_to_michael
        );
        assert_eq!(
            FactionWorld::load_json(&world.save_json().unwrap()).unwrap(),
            world
        );
        assert!(world.move_island_party(IslandPoint { x: 28, y: 16 }));
        assert!(!world.travel_orders.contains_key(victim));
        for mode in 0..4 {
            let mut bad = world.clone();
            match mode {
                0 => bad.party[1] = bad.party[0].clone(),
                1 => bad.party[0] = "missing".into(),
                2 => bad.party[0] = "character.protagonist.captain".into(),
                _ => bad.party[0] = enemy.clone(),
            };
            assert!(FactionWorld::load_json(&bad.save_json().unwrap()).is_err());
        }
        let mut legacy: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        legacy["world"].as_object_mut().unwrap().remove("party");
        assert!(
            FactionWorld::load_json(&legacy.to_string())
                .unwrap()
                .party
                .iter()
                .all(String::is_empty)
        );
    }

    #[test]
    fn legacy_captain_without_person_can_travel_and_recruit_known_people() {
        let (mut world, ids) = recruitment_fixture();
        world
            .actors
            .get_mut("character.protagonist.captain")
            .unwrap()
            .person = None;
        let mut world = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert!(!world.talk_island_person(&ids[0]).is_empty());
        assert!(world.recruit_island_person(&ids[0]));
        assert!(world.assign_island_companion(&ids[0], 0));
        assert!(world.move_island_party(IslandPoint { x: 12, y: 16 }));
        assert!(
            world
                .party_move_failure(IslandPoint { x: 12, y: 16 })
                .is_empty()
        );
        assert!(world.dismiss_island_companion(0));
        assert!(
            !world
                .navigation
                .destinations
                .contains_key(&format!("move.{}", ids[0]))
        );
        assert_eq!(world.actors[&ids[0]].current_assignment_id, None);
    }

    #[test]
    fn undercounted_queue_population_is_rejected_on_load_and_transfer() {
        let (mut world, ids) = recruitment_fixture();
        let faction = "faction.pirates.prototype";
        let building = world.factions[faction]
            .buildings
            .keys()
            .next()
            .unwrap()
            .clone();
        world
            .factions
            .get_mut(faction)
            .unwrap()
            .buildings
            .get_mut(&building)
            .unwrap()
            .operational = true;
        let rule = scenario_production("faction.pirates.prototype");
        world.enqueue_production(faction, &building, rule).unwrap();
        world.talk_island_person(&ids[0]);
        world.factions.get_mut(faction).unwrap().population_used = 5; // Five live people PLUS one reserved: six required.
        assert!(FactionWorld::load_json(&world.save_json().unwrap()).is_err());
        let before = world.clone();
        assert!(!world.recruit_island_person(&ids[0]));
        assert_eq!(world, before);
    }

    #[test]
    fn removal_frees_exactly_the_buildings_reserved_cubes() {
        let mut map = MapPlacement::default();
        map.place_building(
            "site.first",
            GridCube { x: 3, y: 0, z: 4 },
            volume(GridCube { x: 0, y: 0, z: 0 }, [2, 2, 1]),
        )
        .unwrap();
        let removed = map.remove_building("site.first").unwrap();
        assert_eq!(removed.id, "site.first");
        assert!(map.occupied_cubes.is_empty());
        assert!(map.buildings.is_empty());
    }

    /// The one item the main pack authors. Its ID is the character record's
    /// own `signatureWeaponId`, not a second ID minted for the same object.
    const CARBINE: &str = "weapon.captain.handsome_jack_steam_carbine";

    fn carbine_bonus(world: &FactionWorld) -> (u32, u32, u32) {
        match &world.rules.items[CARBINE].effect {
            ScenarioItemEffect::CombatBonus {
                health,
                damage,
                range,
            } => (*health, *damage, *range),
            other => panic!("the authored carbine is a combat modifier, not {other:?}"),
        }
    }

    /// The catalogue travels with the scenario and with its save; a world that
    /// carries no items is unchanged, which the equality proof also asserts.
    #[test]
    fn the_main_scenario_carries_its_item_catalogue_into_the_world_and_its_save() {
        let world = main_scenario_world();
        assert!(world.rules.items.contains_key(CARBINE));
        assert!(world.inventories.is_empty());
        let save = world.save_json().unwrap();
        assert!(
            !save.contains("\"inventories\""),
            "an empty inventory must add nothing to the save"
        );
        let reloaded = FactionWorld::load_json(&save).unwrap();
        assert_eq!(reloaded.rules.items, world.rules.items);
    }

    /// `IslandCombatProfile` is keyed by definition, so an item can only change
    /// one actor's numbers through the per-actor override.
    #[test]
    fn a_granted_item_overrides_only_its_own_holders_combat_numbers() {
        let (mut world, ids) = recruitment_fixture();
        let (holder, bystander) = (ids[0].clone(), ids[1].clone());
        let base = world.actor_combat_profile(&holder).unwrap();
        assert_eq!(
            world.actors[&holder].definition_id,
            world.actors[&bystander].definition_id
        );
        assert!(world.actor_inventory(&holder).is_empty());
        world.grant_item(&holder, CARBINE).unwrap();
        let (health, damage, range) = carbine_bonus(&world);
        let armed = world.actor_combat_profile(&holder).unwrap();
        assert_eq!(armed.health, base.health + health);
        assert_eq!(armed.damage, base.damage + damage);
        assert_eq!(armed.range, base.range + range);
        assert_eq!(armed.cooldown_ticks, base.cooldown_ticks);
        assert_eq!(world.actor_combat_profile(&bystander).unwrap(), base);
        assert_eq!(
            world.combat_profiles[&world.actors[&holder].definition_id],
            base
        );
        // A unique record is carried once however often it is granted.
        assert_eq!(
            world.grant_item(&holder, CARBINE),
            Err("item_stack_full".into())
        );
        assert_eq!(world.actor_inventory(&holder), vec![CARBINE.to_string()]);
        let reloaded = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(reloaded, world);
        assert_eq!(reloaded.actor_combat_profile(&holder).unwrap(), armed);
    }

    /// The other half of the closed effect set. No shipped record uses it yet,
    /// so the catalogue that drives it is the test's own.
    #[test]
    fn a_resource_granting_item_credits_its_holders_faction_once_per_grant() {
        let (mut world, ids) = recruitment_fixture();
        let holder = ids[0].clone();
        let faction = world.actors[&holder].faction_id.clone();
        world.rules.items.insert(
            "item.test.salvage_bundle".into(),
            ScenarioItem {
                display_name: "Test salvage bundle".into(),
                stack: ScenarioItemStack::Stackable(2),
                effect: ScenarioItemEffect::GrantResource {
                    resource: "resource.salvage".into(),
                    amount: 5,
                },
            },
        );
        let before = world.factions[&faction]
            .resources
            .get("resource.salvage")
            .copied()
            .unwrap_or(0);
        world
            .grant_item(&holder, "item.test.salvage_bundle")
            .unwrap();
        world
            .grant_item(&holder, "item.test.salvage_bundle")
            .unwrap();
        assert_eq!(
            world.factions[&faction].resources["resource.salvage"],
            before + 10
        );
        // The stack rule bounds how often the effect can be taken.
        assert_eq!(
            world.grant_item(&holder, "item.test.salvage_bundle"),
            Err("item_stack_full".into())
        );
        assert_eq!(
            world.factions[&faction].resources["resource.salvage"],
            before + 10
        );
        // A faction that declares a cap for the resource keeps it: the grant
        // clamps exactly as the tick's income does.
        let capped = ids[1].clone();
        world.policies.insert(
            faction.clone(),
            FactionPolicy {
                storage_caps: [("resource.salvage".into(), before + 12)]
                    .into_iter()
                    .collect(),
                ..Default::default()
            },
        );
        world
            .grant_item(&capped, "item.test.salvage_bundle")
            .unwrap();
        assert_eq!(
            world.factions[&faction].resources["resource.salvage"],
            before + 12
        );
    }

    /// Recruitment transfers the person, not a copy, and her equipment goes
    /// with her; leaving the active party does not take it back.
    /// Contract: docs/CHARACTER_AND_HAREMLIT_AUTHORING.md.
    #[test]
    fn recruitment_carries_a_womans_equipment_and_leaving_the_party_does_not_remove_it() {
        let (mut world, ids) = recruitment_fixture();
        let woman = ids[0].clone();
        world.grant_item(&woman, CARBINE).unwrap();
        let armed = world.actor_combat_profile(&woman).unwrap();
        let source = world.actors[&woman].faction_id.clone();
        world.talk_island_person(&woman);
        assert!(world.recruit_island_person(&woman));
        assert_eq!(world.actors[&woman].faction_id, "faction.michael");
        assert_ne!(source, "faction.michael");
        assert_eq!(world.actor_inventory(&woman), vec![CARBINE.to_string()]);
        assert_eq!(world.actor_combat_profile(&woman).unwrap(), armed);
        assert_eq!(world.inventories.len(), 1, "no copy stayed behind");
        assert!(world.assign_island_companion(&woman, 0));
        assert!(world.dismiss_island_companion(0));
        assert_eq!(world.actor_inventory(&woman), vec![CARBINE.to_string()]);
        assert_eq!(world.actor_combat_profile(&woman).unwrap(), armed);
        let reloaded = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(reloaded.actor_inventory(&woman), vec![CARBINE.to_string()]);
        assert_eq!(reloaded.actor_combat_profile(&woman).unwrap(), armed);
    }

    /// `begin_building_construction` saves and reloads the world in place, so
    /// anything the world newly carries has to pass its own load validation.
    #[test]
    fn an_inventory_survives_the_building_construction_round_trip() {
        let (mut world, ids) = recruitment_fixture();
        let woman = ids[0].clone();
        world.talk_island_person(&woman);
        assert!(world.recruit_island_person(&woman));
        assert!(world.assign_island_companion(&woman, 0));
        world.grant_item(&woman, CARBINE).unwrap();
        let armed = world.actor_combat_profile(&woman).unwrap();
        let cache = world.salvage_caches["salvage.wreck"].position;
        assert!(world.move_island_party(cache));
        for _ in 0..40 {
            world.advance_island_tick();
        }
        world.salvage_foothold().unwrap();
        // The reviewed workshop site, as the foothold proof uses it.
        let entrance = IslandPoint { x: 19, y: 17 };
        assert!(world.move_island_party(entrance));
        for _ in 0..8 {
            world.advance_island_tick();
        }
        // She must not stand in the new wall footprint.
        world
            .order_move(&woman, IslandPoint { x: 18, y: 17 })
            .unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
        // This saves and reloads the world in place.
        world.build_foothold(entrance).unwrap();
        assert!(
            world.factions["faction.michael"]
                .buildings
                .contains_key("site.michael.field_workshop")
        );
        assert_eq!(world.actor_inventory(&woman), vec![CARBINE.to_string()]);
        assert_eq!(world.actor_combat_profile(&woman).unwrap(), armed);
    }

    /// An effect naming an unknown actor, or an item no catalogue entry
    /// defines, fails by name and changes nothing.
    #[test]
    fn grant_item_refuses_an_unknown_actor_or_an_undefined_item_by_name() {
        let (mut world, ids) = recruitment_fixture();
        let holder = ids[0].clone();
        let before = world.clone();
        assert_eq!(
            world.grant_item("actor.nobody", CARBINE),
            Err("unknown_item_actor".into())
        );
        assert_eq!(
            world.grant_item(&holder, "item.not_in_the_catalogue"),
            Err("unknown_item".into())
        );
        assert_eq!(world, before);
        // The cap is the world's, not the catalogue's.
        world.rules.items.insert(
            "item.test.spare".into(),
            ScenarioItem {
                display_name: "Test spare part".into(),
                stack: ScenarioItemStack::Stackable(64),
                effect: ScenarioItemEffect::CombatBonus {
                    health: 0,
                    damage: 0,
                    range: 0,
                },
            },
        );
        for _ in 0..MAX_ACTOR_INVENTORY {
            world.grant_item(&holder, "item.test.spare").unwrap();
        }
        assert_eq!(
            world.grant_item(&holder, "item.test.spare"),
            Err("inventory_full".into())
        );
    }

    /// A save is untrusted: every carried item must resolve against the world
    /// it is loaded into.
    #[test]
    fn a_saved_inventory_that_does_not_resolve_is_refused() {
        let (mut world, ids) = recruitment_fixture();
        let holder = ids[0].clone();
        world.grant_item(&holder, CARBINE).unwrap();
        let save = world.save_json().unwrap();
        assert!(FactionWorld::load_json(&save).is_ok());
        let edit = |change: &dyn Fn(&mut serde_json::Value)| {
            let mut value: serde_json::Value = serde_json::from_str(&save).unwrap();
            change(&mut value);
            FactionWorld::load_json(&serde_json::to_string(&value).unwrap())
        };
        // An item no catalogue entry defines.
        assert_eq!(
            edit(&|value| {
                value["world"]["inventories"][&holder] =
                    serde_json::json!(["item.not_in_the_catalogue"]);
            })
            .unwrap_err(),
            "invalid_saved_inventory"
        );
        // An actor this world does not have.
        assert_eq!(
            edit(&|value| {
                value["world"]["inventories"]["actor.nobody"] = serde_json::json!([CARBINE]);
            })
            .unwrap_err(),
            "invalid_saved_inventory"
        );
        // More of a unique record than its stack rule allows.
        assert_eq!(
            edit(&|value| {
                value["world"]["inventories"][&holder] = serde_json::json!([CARBINE, CARBINE]);
            })
            .unwrap_err(),
            "invalid_saved_inventory"
        );
        // More than the world's per-actor cap.
        assert_eq!(
            edit(&|value| {
                value["world"]["inventories"][&holder] =
                    serde_json::json!(vec![CARBINE; MAX_ACTOR_INVENTORY + 1]);
            })
            .unwrap_err(),
            "invalid_saved_inventory"
        );
    }
}
