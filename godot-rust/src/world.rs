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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionRule {
    pub id: String,
    pub producer_archetype_id: String,
    pub actor_definition_id: String,
    pub actor_kind: String,
    pub costs: BTreeMap<String, u32>,
    pub production_ticks: u32,
    pub population_use: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionOrder {
    pub id: String,
    pub rule: ProductionRule,
    pub remaining_ticks: u32,
    pub reserved_costs: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionState {
    pub id: String,
    pub resources: BTreeMap<String, u32>,
    pub population_used: u32,
    pub population_capacity: u32,
    /// Maximum absolute fixed-point wobble accepted in one dispatch score.
    pub wobble_limit: i32,
    pub buildings: BTreeMap<String, FactionBuilding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorProductionProvenance {
    pub faction_id: String,
    pub producer_building_id: String,
    pub production_rule_id: String,
    pub reserved_costs: BTreeMap<String, u32>,
    pub completed_tick: u64,
    pub rally_point_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducedActor {
    pub instance_id: String,
    pub definition_id: String,
    pub actor_kind: String,
    pub faction_id: String,
    pub node_id: String,
    pub current_assignment_id: Option<String>,
    pub provenance: ActorProductionProvenance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactionWorld {
    pub tick: u64,
    pub paused: bool,
    pub factions: BTreeMap<String, FactionState>,
    pub actors: BTreeMap<String, ProducedActor>,
    pub navigation: IslandNavigation,
    pub positions: BTreeMap<String, IslandPoint>,
    pub travel_orders: BTreeMap<String, String>,
    next_actor_serial: u64,
    next_order_serial: u64,
}

/// Physical navigation cells, not rooms or strategic graph nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IslandPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IslandNavigation {
    pub walkable: BTreeSet<IslandPoint>,
    pub destinations: BTreeMap<String, IslandPoint>,
}

impl IslandNavigation {
    /// Deterministic shortest path over equal-cost land cells. Bounded by the
    /// finite authored walkable set; never traverses ocean or blocked cells.
    pub fn path(&self, start: IslandPoint, goal: IslandPoint) -> Option<Vec<IslandPoint>> {
        if !self.walkable.contains(&start) || !self.walkable.contains(&goal) {
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
                if self.walkable.contains(&next) && !parents.contains_key(&next) {
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
    pub fn enqueue_production(
        &mut self,
        faction_id: &str,
        building_id: &str,
        rule: ProductionRule,
    ) -> Result<FactionWorldEvent, FactionWorldError> {
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
        let mut events = self.advance_production_tick();
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
