use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq)]
pub struct DeathMemory {
    pub killed_by_player_count: u32,
    pub last_death_day: Option<u32>,
    pub last_death_context_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedPerson {
    pub id: String,
    pub display_name: String,
    pub alive_today: bool,
    pub death_memory: DeathMemory,
}

impl NamedPerson {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: String,
    pub rule: ProductionRule,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionBuilding {
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
    UnitStruck {
        attacker_id: String,
        target_id: String,
        damage: u32,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionWorld {
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
            && !self.building_obstacles.values().any(|cells| cells.contains(&point))
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
    fn resolve_island_skirmish(&mut self) -> Vec<FactionWorldEvent> {
        let mut strikes = Vec::new();
        for (id, actor) in &self.actors {
            let (Some(state), Some(profile), Some(origin)) = (
                self.unit_combat.get(id),
                self.combat_profiles.get(&actor.definition_id),
                self.positions.get(id),
            ) else {
                continue;
            };
            if state.health == 0 || self.tick < state.next_attack_tick {
                continue;
            }
            let target = self
                .actors
                .iter()
                .filter(|(other_id, other)| {
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
                    if distance > profile.range || !self.navigation.clear_line(*origin, *point) {
                        return None;
                    }
                    Some((distance, other_id))
                })
                .min();
            if let Some((_, target_id)) = target {
                strikes.push((
                    id.clone(),
                    target_id.clone(),
                    profile.damage,
                    profile.cooldown_ticks,
                ));
            }
        }
        let mut damage = BTreeMap::<String, u32>::new();
        let mut events = Vec::new();
        // Resolve simultaneously: ID ordering must not grant first-kill immunity.
        for (attacker_id, target_id, amount, cooldown) in strikes {
            self.unit_combat
                .get_mut(&attacker_id)
                .unwrap()
                .next_attack_tick = self.tick.saturating_add(u64::from(cooldown.max(1)));
            let total = damage.entry(target_id.clone()).or_default();
            *total = total.saturating_add(amount);
            events.push(FactionWorldEvent::UnitStruck {
                attacker_id,
                target_id,
                damage: amount,
            });
        }
        for (id, amount) in damage {
            let state = self.unit_combat.get_mut(&id).unwrap();
            state.health = state.health.saturating_sub(amount);
            if state.health != 0 {
                continue;
            }
            let state = self.unit_combat.remove(&id).unwrap();
            let actor = self.actors.remove(&id).unwrap();
            let position = self.positions.remove(&id).unwrap();
            if let Some(faction) = self.factions.get_mut(&actor.faction_id) {
                faction.population_used =
                    faction.population_used.saturating_sub(state.population_use);
            }
            self.travel_orders.remove(&id);
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
            events.push(FactionWorldEvent::UnitFallen { actor_id: id });
        }
        events
    }

    /// Three-faction integration scenario using the existing authored unit rules.
    /// Economy sizes are preview budgets, not final faction balancing.
    pub fn install_preview_factions(&mut self) -> Result<(), String> {
        if self.tick != 0 || self.factions.len() != 1 || !self.travel_orders.is_empty() {
            return Err("scenario_already_started".into());
        }
        let mut staged = self.clone();
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
        ];
        for (id, preferred, source) in entries {
            let rule: ProductionRule =
                serde_json::from_str(source).map_err(|_| "invalid_authored_production")?;
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
            let building_id = format!("preview.{id}.producer");
            let spawn_id = format!("preview.{id}.holding");
            staged
                .navigation
                .destinations
                .insert(spawn_id.clone(), spawn);
            let building = FactionBuilding {
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
                    population_capacity: 6,
                    wobble_limit: 0,
                    buildings: [(building_id.clone(), building)].into_iter().collect(),
                },
            );
            let policy = FactionPolicy {
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
        for (definition, health, damage, range, cooldown_ticks) in [
            ("actor_def.colonial.line_marine", 12, 3, 4, 6),
            ("actor_def.pirates.deckhand", 10, 2, 1, 2),
            ("actor_def.cthulhu.drowned_cultist", 9, 2, 3, 4),
        ] {
            staged.combat_profiles.insert(
                definition.into(),
                IslandCombatProfile {
                    health,
                    damage,
                    range,
                    cooldown_ticks,
                },
            );
        }
        for first in [
            "faction.colonial_powers.prototype",
            "faction.pirates.prototype",
            "faction.cthulhu.prototype",
        ] {
            for second in [
                "faction.colonial_powers.prototype",
                "faction.pirates.prototype",
                "faction.cthulhu.prototype",
            ] {
                if first != second {
                    staged.hostilities.insert((first.into(), second.into()));
                }
            }
        }
        // The same checked-in contract controls sprite scale/pivot and physical cells.
        #[derive(Deserialize)]
        struct Footprint { blocked_offsets: Vec<[i32; 2]> }
        let footprints: BTreeMap<String, Footprint> = serde_json::from_str(include_str!(
            "../../game/assets/island/buildings.json"
        )).map_err(|_| "invalid_building_contract")?;
        for faction in staged.factions.values() {
            for building in faction.buildings.values() {
                if let Some(footprint) = footprints.get(&building.archetype_id) {
                    let entrance = staged.navigation.destinations[&building.node_id];
                    let cells: BTreeSet<_> = footprint.blocked_offsets.iter().map(|[dx,dy]|
                        IslandPoint { x: entrance.x + dx, y: entrance.y + dy }
                    ).collect();
                    if cells.contains(&entrance) || staged.positions.values().any(|p| cells.contains(p)) {
                        return Err("occupied_building_footprint".into());
                    }
                    staged.navigation.building_obstacles.insert(building.id.clone(), cells);
                }
            }
        }
        // No holding may be sealed off by the installed footprints.
        if staged.factions.values().flat_map(|f| f.buildings.values()).any(|b|
            staged.navigation.path(staged.navigation.destinations[&b.node_id], objective).is_none()
        ) {
            return Err("building_blocks_holding_access".into());
        }
        *self = staged;
        Ok(())
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
            || policy.objectives.len() > 256
        {
            return Err(FactionWorldError::InvalidPolicy(faction_id.into()));
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
            self.actors.remove(&id);
            self.unit_combat.remove(&id);
            self.positions.remove(&id);
            self.travel_orders.remove(&id);
            self.navigation.destinations.remove(&format!("move.{id}"));
        }
        self.eliminated_factions.insert(faction_id.into());
        Ok(FactionWorldEvent::FactionEliminated {
            faction_id: faction_id.into(),
        })
    }

    fn advance_faction_decisions(&mut self) -> Vec<FactionWorldEvent> {
        let mut events = Vec::new();
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
                        building.operational && building.production_queue.is_empty()
                    });
                if idle {
                    if let Ok(event) = self.enqueue_production(&id, building_id, rule.clone()) {
                        events.push(event);
                    }
                }
            }
            let idle_actors: Vec<String> = self
                .actors
                .values()
                .filter(|actor| {
                    actor.faction_id == id && !self.travel_orders.contains_key(&actor.instance_id)
                })
                .map(|actor| actor.instance_id.clone())
                .collect();
            for actor_id in idle_actors {
                let Some(start) = self.positions.get(&actor_id).copied() else {
                    continue;
                };
                let reachable: Vec<DispatchCandidate> = policy
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
                if let Some(best) = reachable.iter().max_by(|a, b| {
                    a.total()
                        .cmp(&b.total())
                        .then_with(|| b.action_id.cmp(&a.action_id))
                }) {
                    if self.navigation.destinations.get(&best.target_node_id) == Some(&start) {
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
        let save: Save = serde_json::from_str(text).map_err(|_| "invalid_save_json")?;
        if save.version != 1 {
            return Err("unsupported_save_version".into());
        }
        let world = save.world;
        if world.navigation.walkable.len() > 16384
            || world.actors.len() > 4096
            || world.factions.len() > 64
            || world.tick == u64::MAX
            || world.next_actor_serial == u64::MAX
            || world.next_order_serial == u64::MAX
        {
            return Err("save_limits_exceeded".into());
        }
        for (id, faction) in &world.factions {
            if id != &faction.id
                || faction.population_used > faction.population_capacity
                || faction.wobble_limit < 0
                || faction.buildings.len() > 4096
            {
                return Err("invalid_saved_faction".into());
            }
            for (building_id, building) in &faction.buildings {
                if building_id != &building.id
                    || &building.faction_id != id
                    || building.production_queue.len() > building.queue_capacity
                    || building.queue_capacity > 4096
                {
                    return Err("invalid_saved_building".into());
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
        let building_ids: BTreeSet<_> = world.factions.values()
            .flat_map(|f| f.buildings.keys()).collect();
        if world.navigation.building_obstacles.len() > 4096
            || world.navigation.building_obstacles.iter().any(|(id, cells)|
                !building_ids.contains(id) || cells.len() > 256)
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
            if id != &casualty.actor.instance_id
                || world.actors.contains_key(id)
                || casualty.death_tick > world.tick
            {
                return Err("invalid_saved_casualty".into());
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
        if building.production_queue.len() >= building.queue_capacity {
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
        let mut completed = Vec::new();
        for faction in self.factions.values_mut() {
            for building in faction.buildings.values_mut() {
                if !building.operational {
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

    /// Single simulation step used by the island runtime. Production and travel
    /// share the same pause boundary and clock. Rendering never advances these.
    pub fn advance_island_tick(&mut self) -> Vec<FactionWorldEvent> {
        if self.paused {
            return Vec::new();
        }
        let mut events = self.advance_faction_decisions();
        events.extend(self.advance_production_tick());
        for (actor_id, destination_id) in self.travel_orders.clone() {
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

    fn production_world() -> FactionWorld {
        let building = FactionBuilding {
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
        let entrance = world.navigation.destinations[
            &world.factions["faction.colonial_powers.prototype"].buildings[&id].node_id];
        assert!(world.navigation.traversable(entrance));
        for cell in &cells {
            assert!(!world.navigation.traversable(*cell));
            assert!(world.navigation.path(entrance, *cell).is_none());
            assert!(!world.navigation.clear_line(entrance, *cell));
        }
        let from = IslandPoint { x: entrance.x - 3, y: entrance.y };
        let route = world.navigation.path(from, entrance).unwrap();
        assert!(route.len() > 4);
        assert!(route.iter().all(|cell| !cells.contains(cell)));
        let loaded = FactionWorld::load_json(&world.save_json().unwrap()).unwrap();
        assert_eq!(loaded.navigation, world.navigation);
        world.eliminate_faction("faction.colonial_powers.prototype").unwrap();
        assert!(!world.navigation.building_obstacles.contains_key(&id));
        assert!(cells.iter().all(|cell| world.navigation.traversable(*cell)));
        let mut invalid = loaded;
        invalid.navigation.building_obstacles.insert("missing".into(), cells);
        assert!(FactionWorld::load_json(&invalid.save_json().unwrap()).is_err());
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
