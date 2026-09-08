use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Deserialize)]
struct IslandBuildingFootprint {
    blocked_offsets: Vec<[i32; 2]>,
    #[serde(default)]
    placement_entrance: Option<[i32; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HoldingExpansion {
    faction_id: String,
    building_id: String,
    archetype_id: String,
    entrance: [i32; 2],
    minimum_tick: u64,
    costs: BTreeMap<String, u32>,
    construction_ticks: u32,
    holding_health: u32,
}

fn holding_expansion() -> Result<&'static HoldingExpansion, String> {
    static RULE: std::sync::OnceLock<Result<HoldingExpansion, String>> = std::sync::OnceLock::new();
    RULE.get_or_init(|| {
        let rule: HoldingExpansion =
            serde_json::from_str(include_str!("../../content/island/colonial_expansion.json"))
                .map_err(|_| "invalid_expansion_rule".to_string())?;
        if !(1..=100000).contains(&rule.construction_ticks)
            || !(1..=100000).contains(&rule.holding_health)
            || rule.costs.is_empty()
            || rule.costs.values().any(|cost| !(1..=100000).contains(cost))
        {
            return Err("invalid_expansion_rule".into());
        }
        Ok(rule)
    })
    .as_ref()
    .map_err(Clone::clone)
}

fn island_building_footprints() -> Result<BTreeMap<String, IslandBuildingFootprint>, String> {
    serde_json::from_str(include_str!("../../game/assets/island/buildings.json"))
        .map_err(|_| "invalid_building_contract".into())
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonaPool {
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

fn produced_person(definition: &str, id: &str, serial: u64) -> Option<NamedPerson> {
    static POOLS: std::sync::OnceLock<BTreeMap<String, Vec<PersonaPool>>> =
        std::sync::OnceLock::new();
    let pools = POOLS.get_or_init(|| {
        serde_json::from_str(include_str!("../../game/assets/island/personas.json"))
            .expect("validated persona catalog")
    });
    let seed = mix_seed(serial, 0, id, 0);
    let variants = pools.get(definition)?;
    let pool = variants.get((seed % variants.len().max(1) as u64) as usize)?;
    let pick = |values: &[String], rotation: u32| {
        values[(seed.rotate_left(rotation) % values.len() as u64) as usize].clone()
    };
    Some(NamedPerson {
        id: id.into(),
        display_name: format!(
            "{} {}",
            pick(&pool.given_names, 0),
            pick(&pool.family_names, 13)
        ),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Deserialize)]
struct IslandFactionTuning {
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
    pub construction: Option<BuildingConstruction>,
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
pub struct BuildingConstruction {
    pub builder_id: String,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FootholdCache {
    pub position: IslandPoint,
    pub remaining: u32,
}

#[derive(Deserialize)]
struct FootholdConfig {
    cache_position: IslandPoint,
    cache_salvage: u32,
    build_salvage: u32,
    construction_ticks: u32,
    restore_salvage: u32,
    building_health: u32,
}

fn foothold_config() -> Result<&'static FootholdConfig, String> {
    static CONFIG: std::sync::OnceLock<Result<FootholdConfig, String>> = std::sync::OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let config: FootholdConfig =
                serde_json::from_str(include_str!("../../content/island/michael_foothold.json"))
                    .map_err(|_| "Invalid foothold configuration.")?;
            if [
                config.cache_salvage,
                config.build_salvage,
                config.construction_ticks,
                config.restore_salvage,
                config.building_health,
            ]
            .iter()
            .any(|v| !(1..=100000).contains(v))
            {
                return Err("Invalid foothold configuration.".into());
            }
            Ok(config)
        })
        .as_ref()
        .map_err(Clone::clone)
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyNotice {
    pub tick: u64,
    pub factions: Vec<String>,
    pub text: String,
}

#[derive(Deserialize)]
struct InitialDiplomacy {
    #[serde(rename = "factionIds")]
    faction_ids: Vec<String>,
    relationships: Vec<InitialRelationship>,
}
#[derive(Deserialize)]
struct InitialRelationship {
    a: String,
    b: String,
    #[serde(rename = "atWar")]
    at_war: bool,
}
#[derive(Deserialize)]
struct SurvivalRules {
    decision_ticks: u64,
    truce_ticks: u64,
    dominance_percent: u64,
    strength_horizon_ticks: u64,
}
fn survival_rules() -> &'static SurvivalRules {
    static RULES: std::sync::OnceLock<SurvivalRules> = std::sync::OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../../content/island/survival_diplomacy.json"))
            .expect("authored survival rules")
    })
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionWorld {
    #[serde(default)]
    pub survival_truces: Vec<SurvivalTruce>,
    #[serde(default)]
    pub diplomacy_notices: Vec<DiplomacyNotice>,
    #[serde(default)]
    pub foothold_cache: Option<FootholdCache>,
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
    #[serde(default)]
    pub hostilities: BTreeSet<(String, String)>,
    #[serde(default)]
    pub casualties: BTreeMap<String, IslandCasualty>,
    next_actor_serial: u64,
    next_order_serial: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
            .filter_map(|(id, a)| {
                self.combat_profiles
                    .get(&a.definition_id)
                    .filter(|p| p.damage > 0)
                    .map(|p| {
                        u64::from(self.unit_combat[id].health) * 100
                            + u64::from(p.damage) * survival_rules().strength_horizon_ticks * 100
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
                    .saturating_mul(survival_rules().dominance_percent)
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
        let rules = survival_rules();
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

    pub fn island_person_news(&self, id: &str) -> String {
        if !self.can_talk_island_person(id) || !self.living_person(id).is_some_and(|p| p.discussed)
        {
            return String::new();
        }
        let faction = &self.actors[id].faction_id;
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
            foothold_config().map(|c| c.construction_ticks).unwrap_or(0)
        } else {
            holding_expansion()
                .ok()
                .filter(|r| r.building_id == building.id)
                .map(|r| r.construction_ticks)
                .unwrap_or(0)
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
        let Ok(rule) = holding_expansion() else {
            return;
        };
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
                construction: Some(BuildingConstruction {
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
        let config = foothold_config()?;
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
        let cache = self
            .foothold_cache
            .as_ref()
            .ok_or("No wreck salvage in this campaign.")?;
        if cache.remaining == 0 {
            return Err("The wreck salvage is exhausted.".into());
        }
        if !self.within_work_range("character.protagonist.captain", cache.position) {
            return Err("Bring Michael within reach of the wreck salvage.".into());
        }
        let faction = self
            .factions
            .get_mut("faction.michael")
            .ok_or("Michael's faction is unavailable.")?;
        let stored = faction
            .resources
            .get("resource.salvage")
            .copied()
            .unwrap_or(0)
            .checked_add(cache.remaining)
            .ok_or("Salvage storage is full.")?;
        faction.resources.insert("resource.salvage".into(), stored);
        self.foothold_cache.as_mut().unwrap().remaining = 0;
        Ok(())
    }

    pub fn build_foothold(&mut self, entrance: IslandPoint) -> Result<(), String> {
        const CAPTAIN: &str = "character.protagonist.captain";
        const BUILDING: &str = "site.michael.field_workshop";
        const ARCHETYPE: &str = "site_archetype.michael.field_workshop";
        let config = foothold_config()?;
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
        let stored = faction
            .resources
            .get("resource.salvage")
            .copied()
            .unwrap_or(0);
        if stored < config.build_salvage {
            return Err(format!(
                "Need {} salvage to build the workshop.",
                config.build_salvage
            ));
        }
        let footprints = island_building_footprints()?;
        let footprint = footprints
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
                construction: Some(BuildingConstruction {
                    builder_id: CAPTAIN.into(),
                    remaining_ticks: config.construction_ticks,
                    reserved_costs: [("resource.salvage".into(), config.build_salvage)]
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
            || costs.iter().any(|(r, c)| {
                self.factions[&faction_id]
                    .resources
                    .get(r)
                    .copied()
                    .unwrap_or(0)
                    < *c
            })
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
                .foothold_cache
                .as_ref()
                .is_some_and(|c| c.remaining > 0 && !reachable(c.position))
        {
            return Err("The site would block an island route.".into());
        }
        for (resource, cost) in costs {
            *staged
                .factions
                .get_mut(&faction_id)
                .unwrap()
                .resources
                .get_mut(&resource)
                .unwrap() -= cost;
        }
        *self = Self::load_json(&staged.save_json()?)?;
        Ok(())
    }

    pub fn restore_foothold_person(&mut self, id: &str) -> Result<(), String> {
        let config = foothold_config()?;
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
            .combat_profiles
            .get(&actor.definition_id)
            .ok_or("Her health profile is unavailable.")?
            .health;
        let faction = self.factions.get_mut("faction.michael").unwrap();
        let stored = faction
            .resources
            .get("resource.salvage")
            .copied()
            .unwrap_or(0);
        if stored < config.restore_salvage {
            return Err(format!(
                "Need {} salvage for restoration.",
                config.restore_salvage
            ));
        }
        faction
            .resources
            .insert("resource.salvage".into(), stored - config.restore_salvage);
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
                || casualty.death_tick >= self.tick
                || casualty.actor.provenance.producer_building_id.is_empty()
                || self.actors.len() >= 4096
            {
                continue;
            }
            let Some(profile) = self.combat_profiles.get(&casualty.actor.definition_id) else {
                continue;
            };
            let restored_health = profile.health;
            let Some(population) = self.factions[CTHULHU]
                .population_used
                .checked_add(casualty.population_use)
            else {
                continue;
            };
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
            casualty.actor.current_assignment_id = None;
            if let Some(person) = casualty.actor.person.as_mut() {
                person.return_at_midnight();
            }
            // This is an explicit supernatural transfer, not another production
            // order: provenance and serials stay unchanged, capacity admits only
            // the population that actually returned.
            let faction = self.factions.get_mut(CTHULHU).unwrap();
            faction.population_used = population;
            faction.population_capacity = faction.population_capacity.max(population);
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
            self.combat_profiles.get(CAPTAIN),
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

    fn island_firing_target(&self, id: &str) -> Option<&String> {
        let actor = self.actors.get(id)?;
        let state = self.unit_combat.get(id)?;
        if state.health == 0 {
            return None;
        }
        let profile = self.combat_profiles.get(&actor.definition_id)?;
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
        let actor = self.actors.get(id)?;
        let origin = *self.positions.get(id)?;
        let profile = self.combat_profiles.get(&actor.definition_id)?;
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
            let (Some(state), Some(profile)) = (
                self.unit_combat.get(id),
                self.combat_profiles.get(&actor.definition_id),
            ) else {
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
                .next_attack_tick = self.tick.saturating_add(u64::from(cooldown.max(1)));
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
            // Reserved queue costs are lost with the destroyed producer.
            let reserved: u32 = building
                .production_queue
                .iter()
                .map(|o| o.rule.population_use)
                .fold(0u32, u32::saturating_add);
            faction.population_used = faction.population_used.saturating_sub(reserved);
            faction.buildings.remove(&building_id);
            self.navigation.building_obstacles.remove(&building_id);
            if let Some(policy) = self.policies.get_mut(&faction_id) {
                policy.production.remove(&building_id);
                policy.development.remove(&building_id);
            }
            if faction.buildings.is_empty()
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

    /// Four autonomous factions plus Michael, using real authored production.
    /// Elven deployment awaits an admitted monster sprite; no invisible army.
    /// Economy sizes are preview budgets, not final faction balancing.
    pub fn install_preview_factions(&mut self) -> Result<(), String> {
        if self.tick != 0 || self.factions.len() != 1 || !self.travel_orders.is_empty() {
            return Err("scenario_already_started".into());
        }
        let mut staged = self.clone();
        let footprints = island_building_footprints()?;
        let roster: BTreeMap<String, IslandFactionTuning> =
            serde_json::from_str(include_str!("../../content/island/faction_roster.json"))
                .map_err(|_| "invalid_faction_roster")?;
        let objective = *staged
            .positions
            .get("character.protagonist.captain")
            .ok_or("missing_michael")?;
        staged
            .navigation
            .destinations
            .insert("island.contested_clearing".into(), objective);
        let entries = [
            (
                "faction.colonial_powers.prototype",
                IslandPoint { x: 12, y: 11 },
                include_str!("../../content/production/colonial_fort_soldiers.json"),
            ),
            (
                "faction.pirates.prototype",
                IslandPoint { x: 35, y: 15 },
                include_str!("../../content/production/tide_quay_deckhands.json"),
            ),
            (
                "faction.cthulhu.prototype",
                IslandPoint { x: 28, y: 23 },
                include_str!("../../content/production/drowned_shrine_cultists.json"),
            ),
            (
                "faction.eastern_fox_people.prototype",
                IslandPoint { x: 33, y: 9 },
                include_str!(
                    "../../content/production/eastern_fox_people_river_market_skirmishers.json"
                ),
            ),
            (
                "faction.elves.prototype",
                IslandPoint { x: 13, y: 17 },
                include_str!("../../content/production/elven_heart_grove_bow_wardens.json"),
            ),
        ];
        if roster.len() != entries.len() {
            return Err("invalid_faction_roster".into());
        }
        for (id, preferred, source) in entries {
            let tuning = roster.get(id).ok_or("missing_faction_tuning")?;
            if !(1..=4096).contains(&tuning.population_capacity)
                || tuning.combat.health > 100000
                || tuning.combat.damage > 100000
                || tuning.combat.cooldown_ticks > 100000
            {
                return Err("invalid_faction_tuning".into());
            }
            let rule: ProductionRule =
                serde_json::from_str(source).map_err(|_| "invalid_authored_production")?;
            let placement = footprints
                .get(&rule.producer_archetype_id)
                .and_then(|footprint| footprint.placement_entrance)
                .map(|[x, y]| IslandPoint { x, y });
            let preferred = placement.unwrap_or(preferred);
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
            staged.factions.insert(
                id.into(),
                FactionState {
                    id: id.into(),
                    resources: rule
                        .costs
                        .iter()
                        .map(|(id, cost)| (id.clone(), cost.saturating_mul(2)))
                        .collect(),
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
                development: {
                    let rules: BTreeMap<String, BuildingDevelopmentRule> = serde_json::from_str(
                        include_str!("../../content/island/holding_development.json"),
                    )
                    .map_err(|_| "invalid_authored_development")?;
                    rules
                        .get(&rule.producer_archetype_id)
                        .map(|development| {
                            [(building_id.clone(), development.clone())]
                                .into_iter()
                                .collect()
                        })
                        .unwrap_or_default()
                },
                income_per_tick: rule.costs.keys().map(|id| (id.clone(), 1)).collect(),
                storage_caps: rule.costs.keys().map(|id| (id.clone(), 20)).collect(),
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
                .map_err(|_| "invalid_preview_policy")?;
        }
        let initial: InitialDiplomacy = serde_json::from_str(include_str!(
            "../../content/diplomacy/initial_relationships.prototype.json"
        ))
        .map_err(|_| "invalid_initial_diplomacy")?;
        let mut pairs = BTreeSet::new();
        let rules = survival_rules();
        if !(1..=1024).contains(&rules.decision_ticks)
            || !(1..=100000).contains(&rules.truce_ticks)
            || !(101..=1000).contains(&rules.dominance_percent)
            || !(1..=1024).contains(&rules.strength_horizon_ticks)
        {
            return Err("invalid_survival_rules".into());
        }
        if initial.faction_ids.len() != 5 || initial.relationships.len() != 10 {
            return Err("invalid_initial_diplomacy".into());
        }
        for row in initial.relationships {
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
                staged.hostilities.insert((row.b, row.a));
            }
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
        let config = foothold_config()?;
        if staged
            .navigation
            .path(objective, config.cache_position)
            .is_none()
        {
            return Err("Wreck salvage is unreachable.".into());
        }
        staged.foothold_cache = Some(FootholdCache {
            position: config.cache_position,
            remaining: config.cache_salvage,
        });
        *self = Self::load_json(&staged.save_json()?)?;
        Ok(())
    }

    fn authored_building_obstacles(
        &self,
    ) -> Result<BTreeMap<String, BTreeSet<IslandPoint>>, String> {
        // New scenarios and old-save migration share the same asset contract.
        let footprints = island_building_footprints()?;
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
                if let Some(cap) = policy.storage_caps.get(resource) {
                    let stored = faction.resources.entry(resource.clone()).or_default();
                    *stored = stored.saturating_add(*income).min(*cap);
                }
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
            let faction = self.factions.get_mut(&id).unwrap();
            if faction.population_used >= faction.population_capacity {
                for (building_id, rule) in &policy.development {
                    let Some(building) = faction.buildings.get_mut(building_id) else {
                        continue;
                    };
                    if !building.operational
                        || building.health != building.max_health
                        || building.level >= rule.max_level
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
                    if costs.iter().any(|(resource, cost)| {
                        faction.resources.get(resource).copied().unwrap_or(0) < *cost
                    }) {
                        continue;
                    }
                    for (resource, cost) in &costs {
                        *faction.resources.get_mut(resource).unwrap() -= cost;
                    }
                    building.development = Some(BuildingDevelopment {
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
                                .combat_profiles
                                .get(&other.definition_id)
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
                            || actor
                                .current_assignment_id
                                .as_deref()
                                .is_some_and(|a| a.starts_with("construct."))
                            || !self.living_actor(actor_id)
                            || !self
                                .combat_profiles
                                .get(&actor.definition_id)
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
                            && !actor
                                .current_assignment_id
                                .as_deref()
                                .is_some_and(|a| a.starts_with("construct."))
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
            version: 1,
            world: self,
        })
        .map_err(|error| error.to_string())
    }

    /// Deserialize into a new value; callers replace live state only after all
    /// validation succeeds. File-size and collection caps bound untrusted saves.
    pub fn load_json(text: &str) -> Result<Self, String> {
        if text.len() > 8 * 1024 * 1024 {
            return Err("save_too_large".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Save {
            version: u32,
            world: FactionWorld,
        }
        let json: serde_json::Value =
            serde_json::from_str(text).map_err(|_| "invalid_save_json")?;
        let needs_obstacles = json
            .pointer("/world/navigation/building_obstacles")
            .is_none();
        let save: Save = serde_json::from_value(json).map_err(|_| "invalid_save_json")?;
        if save.version != 1 {
            return Err("unsupported_save_version".into());
        }
        let mut world = save.world;
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
                    || t.expires_tick > world.tick.saturating_add(survival_rules().truce_ticks)
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
        if world.foothold_cache.as_ref().is_some_and(|c| {
            c.remaining > 100000 || !world.navigation.walkable.contains(&c.position)
        }) {
            return Err("invalid_saved_foothold_cache".into());
        }
        if !(1..=1000000).contains(&world.clock.ticks_per_day) {
            return Err("invalid_saved_clock".into());
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
                if let Some(job) = &building.construction {
                    let workshop = faction.id == "faction.michael"
                        && building.archetype_id == "site_archetype.michael.field_workshop"
                        && job.builder_id == "character.protagonist.captain"
                        && foothold_config().is_ok_and(|c| {
                            job.remaining_ticks <= c.construction_ticks
                                && job.reserved_costs
                                    == [("resource.salvage".into(), c.build_salvage)].into()
                        });
                    let expansion = holding_expansion().is_ok_and(|r| {
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
                    });
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
        for (id, actor) in &world.actors {
            if actor
                .person
                .as_ref()
                .is_some_and(|person| !person.valid_for_actor(id, true))
            {
                return Err("invalid_saved_person".into());
            }
            if id != &actor.instance_id || !world.factions.contains_key(&actor.faction_id) {
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
            let actor = world.actors.get(id).ok_or("dangling_combat_state")?;
            let profile = world
                .combat_profiles
                .get(&actor.definition_id)
                .ok_or("missing_combat_profile")?;
            if state.health == 0 || state.health > profile.health {
                return Err("invalid_saved_health".into());
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

    /// Small physical island fixture for runtime integration, not the final map.
    /// Michael is scenario-seeded alone; no automatic companion recruitment.
    pub fn prototype_island() -> Self {
        let mut world = Self::default();
        // Provisional basic carbine, not Michael's future Echo skill deck.
        world.combat_profiles.insert(
            "character.protagonist.captain".into(),
            IslandCombatProfile {
                health: 30,
                damage: 4,
                range: 4,
                cooldown_ticks: 5,
            },
        );
        world.unit_combat.insert(
            "character.protagonist.captain".into(),
            IslandCombatState {
                health: 30,
                next_attack_tick: 0,
                population_use: 1,
            },
        );
        for x in 0..48 {
            for y in 0..32 {
                if (x - 24_i32).pow(2) * 196 + (y - 16_i32).pow(2) * 484 < 484 * 196 {
                    world.navigation.walkable.insert(IslandPoint { x, y });
                }
            }
        }
        let faction_id = "faction.michael".to_owned();
        world.factions.insert(
            faction_id.clone(),
            FactionState {
                id: faction_id.clone(),
                resources: BTreeMap::new(),
                population_used: 1,
                population_capacity: 1,
                wobble_limit: 0,
                buildings: BTreeMap::new(),
            },
        );
        let actor_id = "character.protagonist.captain".to_owned();
        world.actors.insert(
            actor_id.clone(),
            ProducedActor {
                undead: false,
                person: Some(NamedPerson {
                    id: actor_id.clone(),
                    display_name: "Michael".into(),
                    alive_today: true,
                    sex: PersonSex::Male,
                    age: Some(20),
                    backstory: "Captain of the Handsome Jack. Shipwreck survivor and inventor."
                        .into(),
                    ..Default::default()
                }),
                instance_id: actor_id.clone(),
                definition_id: actor_id.clone(),
                actor_kind: "hero".into(),
                faction_id: faction_id.clone(),
                node_id: "scenario.shipwreck".into(),
                current_assignment_id: None,
                provenance: ActorProductionProvenance {
                    faction_id,
                    producer_building_id: String::new(),
                    production_rule_id: "scenario_start.shipwreck".into(),
                    reserved_costs: BTreeMap::new(),
                    completed_tick: 0,
                    rally_point_id: "scenario.shipwreck".into(),
                },
            },
        );
        world
            .positions
            .insert(actor_id, IslandPoint { x: 8, y: 16 });
        world
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
        let source = actor.faction_id.clone();
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
            .get("faction.michael")
            .and_then(|f| f.population_used.checked_add(population))
        else {
            return false;
        };
        if self.approach_target.as_deref() == Some(id) {
            self.cancel_approach();
        }
        self.factions.get_mut(&source).unwrap().population_used = source_used;
        let destination = self.factions.get_mut("faction.michael").unwrap();
        destination.population_used = destination_used;
        // Provisional immigrant accommodation, not additional army production.
        destination.population_capacity = destination.population_capacity.max(destination_used);
        let actor = self.actors.get_mut(id).unwrap();
        actor.faction_id = "faction.michael".into();
        actor.current_assignment_id = None;
        actor.person.as_mut().unwrap().loyal_to_michael = true;
        self.cancel_actor_travel(id);
        if self.player_attack_target.as_deref() == Some(id) {
            self.player_attack_target = None;
        }
        true
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
        for id in &self.party {
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
            let Some(person) = self.living_person(id) else {
                return Err("A companion is unavailable.".into());
            };
            if !person.loyal_to_michael || self.actors[id].faction_id != "faction.michael" {
                return Err(format!(
                    "{} is not an available companion.",
                    person.display_name
                ));
            }
            let start = self.positions[id];
            let mut candidates: Vec<_> = self
                .navigation
                .walkable
                .iter()
                .copied()
                .filter(|point| {
                    !occupied.contains(point)
                        && u64::from(point.x.abs_diff(target.x))
                            + u64::from(point.y.abs_diff(target.y))
                            <= 3
                })
                .collect();
            candidates.sort_by_key(|point| {
                (
                    u64::from(point.x.abs_diff(target.x)) + u64::from(point.y.abs_diff(target.y)),
                    *point,
                )
            });
            let Some(destination) = candidates
                .into_iter()
                .find(|point| self.navigation.path(start, *point).is_some())
            else {
                return Err(format!(
                    "{} cannot reach the party destination.",
                    person.display_name
                ));
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
        for id in self.party.clone().iter().filter(|id| !id.is_empty()) {
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
        let faction = self
            .factions
            .get_mut(faction_id)
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
            let available = faction.resources.get(resource_id).copied().unwrap_or(0);
            if available < *required {
                return Err(FactionWorldError::InsufficientResource {
                    resource_id: resource_id.clone(),
                    required: *required,
                    available,
                });
            }
        }

        for (resource_id, required) in &rule.costs {
            *faction.resources.entry(resource_id.clone()).or_default() -= *required;
        }
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
                b.construction.as_ref().is_some_and(|job| {
                    self.actors
                        .get(&job.builder_id)
                        .is_some_and(|a| a.faction_id == b.faction_id)
                        && self
                            .navigation
                            .destinations
                            .get(&b.node_id)
                            .is_some_and(|p| self.within_work_range(&job.builder_id, *p))
                })
            })
            .map(|b| b.id.clone())
            .collect();
        let mut completed = Vec::new();
        for faction in self.factions.values_mut() {
            for building in faction.buildings.values_mut() {
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

        completed.sort_by(|left, right| left.4.id.cmp(&right.4.id));
        let mut events = Vec::with_capacity(completed.len());
        for (faction_id, building_id, node_id, rally_point_id, order) in completed {
            self.next_actor_serial += 1;
            let actor = ProducedActor {
                undead: false,
                person: produced_person(
                    &order.rule.actor_definition_id,
                    &format!("actor_instance.{faction_id}.{}", self.next_actor_serial),
                    self.next_actor_serial,
                ),
                instance_id: format!("actor_instance.{faction_id}.{}", self.next_actor_serial),
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
        for id in self.party.clone().into_iter().filter(|id| !id.is_empty()) {
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
                    && (self.island_firing_target(id).is_some()
                        || (!actor.current_assignment_id.as_deref().is_some_and(|a| {
                            a.starts_with("defend.") || a.starts_with("construct.")
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
        events.extend(self.return_midnight_casualties());
        if self
            .approach_target
            .as_ref()
            .is_some_and(|id| self.accessible_person(id))
        {
            self.cancel_approach();
        }
        events
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        assert_eq!(world.factions.len(), 6); // Five AI factions plus Michael.
        assert!(world.factions.contains_key("faction.elves.prototype"));
        assert_eq!(world.factions[fox].population_capacity, 8);
        assert_eq!(world.combat_profiles[definition].health, 8);
        assert_eq!(world.combat_profiles[definition].cooldown_ticks, 2);
        let building = world.factions[fox].buildings.values().next().unwrap();
        let holding = world.navigation.destinations[&building.node_id];
        assert_eq!(
            building.archetype_id,
            "site_archetype.eastern_fox_people.river_market"
        );
        // Peaceful observation isolates production and deployment from losses.
        world.hostilities.clear();
        let mut moved = false;
        for _ in 0..20 {
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

    fn production_world() -> FactionWorld {
        let building = FactionBuilding {
            construction: None,
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
        FactionWorld {
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

    #[test]
    fn saved_world_rejects_version_and_dangling_actor() {
        let world = FactionWorld::prototype_island();
        let mut data: serde_json::Value =
            serde_json::from_str(&world.save_json().unwrap()).unwrap();
        data["version"] = 99.into();
        assert!(FactionWorld::load_json(&data.to_string()).is_err());
        data["version"] = 1.into();
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
        let rules: BTreeMap<String, BuildingDevelopmentRule> = serde_json::from_str(include_str!(
            "../../content/island/holding_development.json"
        ))
        .unwrap();
        for source in [
            include_str!("../../content/production/colonial_fort_soldiers.json"),
            include_str!("../../content/production/tide_quay_deckhands.json"),
            include_str!("../../content/production/drowned_shrine_cultists.json"),
        ] {
            let producer: ProductionRule = serde_json::from_str(source).unwrap();
            let development = &rules[&producer.producer_archetype_id];
            assert_eq!(development.max_level, 5);
            assert!(development.costs.iter().all(|(resource, cost)| producer.costs.contains_key(resource) && cost * 4 <= 20));
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        let mut siege_assigned = false;
        let mut building_struck = false;
        for step in 0..600 {
            // Finite-supply scenario: existing troops and queued cycles stay,
            // but endless replacement income must not mask siege reachability.
            if step == 60 {
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        let mut history = Vec::new();
        for _ in 0..100 {
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
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
        let pools: BTreeMap<String, Vec<PersonaPool>> =
            serde_json::from_str(include_str!("../../game/assets/island/personas.json")).unwrap();
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
        assert!(produced_person("actor_def.unknown_machine", "machine.1", 1).is_none());
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
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
        for _ in 0..100 {
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..4 {
            world.advance_island_tick();
        }
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
        let mut saw_elimination = false;
        let before = world.positions[&pirate];
        world.advance_island_tick();
        assert_eq!(world.positions[&pirate], before);
        assert!(world.travel_orders.contains_key(&pirate));
        for _ in 0..100 {
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
            let mut world = FactionWorld::prototype_island();
            world.install_preview_factions().unwrap();
            // Peace isolates the authored production and voluntary recruitment loop;
            // no actors, identities, population, or positions are fabricated.
            world.hostilities.clear();
            // New factions change shared production serials. Observe actual
            // bounded development/replacement output, not an assumed sex split
            // in the first six seeded births.
            for _ in 0..300 {
                world.advance_island_tick();
                let produced = || {
                    world
                        .actors
                        .values()
                        .filter(|a| a.definition_id == definition)
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
            let produced: Vec<_> = world
                .actors
                .values()
                .filter(|a| a.definition_id == definition)
                .collect();
            let male = produced
                .iter()
                .find(|a| a.person.as_ref().unwrap().sex == PersonSex::Male)
                .expect("ordinary production includes male units");
            let female = produced
                .iter()
                .find(|a| a.person.as_ref().unwrap().sex == PersonSex::Female)
                .expect("ordinary production includes female units");
            assert!(
                produced
                    .iter()
                    .all(|actor| !actor.undead && actor.person.as_ref().unwrap().alive_today)
            );
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        world.hostilities.clear();
        let elves = "faction.elves.prototype";
        let definition = "actor_def.elven.bow_warden";
        let building = world.factions[elves].buildings.values().next().unwrap();
        let home = world.navigation.destinations[&building.node_id];
        assert_eq!((building.level, building.health), (1, 100));
        assert_eq!(world.factions[elves].population_capacity, 3);
        for _ in 0..7 {
            world.advance_island_tick();
        }
        assert!(!world.actors.values().any(|a| a.faction_id == elves));
        world.advance_island_tick();
        assert_eq!(
            world
                .actors
                .values()
                .filter(|a| a.faction_id == elves)
                .count(),
            1
        );
        for _ in 0..24 {
            world.advance_island_tick();
        }
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
            (20, 4, 5, 4)
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        world.hostilities.clear(); // Controlled survival, no gifted resources or teleported builder.
        let rule = holding_expansion().unwrap();
        let mut started = None;
        for _ in 0..500 {
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
        for _ in 0..160 {
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
        for _ in 0..220 {
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
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
        for _ in 0..8 {
            world.advance_island_tick();
        }
        // Isolate the pressure threshold with a stronger existing cult army,
        // not free actors or a separate war state. Production identity remains.
        world
            .combat_profiles
            .get_mut("actor_def.cthulhu.drowned_cultist")
            .unwrap()
            .damage = 100;
        world.tick = 32;
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
        assert_eq!(treaty.expires_tick, 224);
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
        world.tick = 224;
        world.advance_island_tick();
        assert!(
            world
                .survival_truces
                .iter()
                .any(|t| t.a == colonial && t.b == pirates && t.expires_tick == 416)
        );
        world
            .combat_profiles
            .get_mut("actor_def.cthulhu.drowned_cultist")
            .unwrap()
            .damage = 0;
        world.tick = 416;
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
        let mut world = FactionWorld::prototype_island();
        world.install_preview_factions().unwrap();
        for _ in 0..2 {
            world.advance_island_tick();
        }
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
        let cache = world.foothold_cache.as_ref().unwrap().position;
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
        for _ in 0..50 {
            assert_eq!(world.advance_island_tick(), restored.advance_island_tick());
            assert_eq!(world, restored);
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
            .remove("foothold_cache");
        assert!(
            FactionWorld::load_json(&legacy.to_string())
                .unwrap()
                .foothold_cache
                .is_none()
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
        world.advance_island_tick();
        assert_eq!(world.minute_of_day(), 1080);
        let mut restored = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        let events = world.advance_island_tick();
        assert_eq!(events, restored.advance_island_tick());
        assert_eq!(world, restored);
        assert!(events.iter().any(|e| matches!(e, FactionWorldEvent::MidnightReturned {actor_id,..} if actor_id == &victim)));
        assert_eq!(world.day(), 2);
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
        let rule = serde_json::from_str(include_str!(
            "../../content/production/tide_quay_deckhands.json"
        ))
        .unwrap();
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
        let rule = serde_json::from_str(include_str!(
            "../../content/production/tide_quay_deckhands.json"
        ))
        .unwrap();
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
}
