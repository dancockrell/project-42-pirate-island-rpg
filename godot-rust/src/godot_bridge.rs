use godot::prelude::*;

use crate::battle::{
    Actor, Battle, BattleEvent, BattlePhase, BattleSnapshot, BattlefieldEffect, Faction,
    RecoveryOpening, StatusInstance, StatusKind,
};
use crate::protocol::{CommandEnvelope, CommandKind, PROTOCOL_VERSION};
use crate::world::{FactionWorld, FactionWorldEvent, IslandPoint};

const BATTLE_ID: &str = "battle.prototype.returning_names";

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Project42SimulationBridge {
    #[init(val = None)]
    battle: Option<Battle>,
    #[init(val = 0)]
    sequence: u64,
    #[init(val = None)]
    island: Option<FactionWorld>,
}

#[godot_api]
impl Project42SimulationBridge {
    #[func]
    fn aim_island_carbine(&mut self, target: GString) -> bool {
        self.island
            .as_mut()
            .is_some_and(|world| world.aim_carbine(&target.to_string()))
    }
    #[func]
    fn install_preview_factions(&mut self) -> bool {
        let Some(world) = self.island.as_mut() else {
            return false;
        };
        match world.install_preview_factions() {
            Ok(()) => true,
            Err(reason) => {
                godot_error!("Island placement failed: {reason}");
                false
            }
        }
    }
    #[func]
    fn save_island(&self) -> GString {
        self.island
            .as_ref()
            .and_then(|world| world.save_json().ok())
            .unwrap_or_default()
            .as_str()
            .into()
    }

    #[func]
    fn load_island(&mut self, payload: GString) -> bool {
        let Ok(mut world) = FactionWorld::load_json(&payload.to_string()) else {
            return false;
        };
        let Some(current) = self.island.as_ref() else {
            return false;
        };
        if world.navigation.walkable != current.navigation.walkable {
            // One exact additive map correction, not arbitrary save-map coercion.
            // Old island positions and buildings remain where they really were.
            #[derive(serde::Deserialize)]
            struct LandRevision {
                #[serde(rename = "landCorrections")]
                corrections: Vec<[i32; 2]>,
            }
            let Ok(revision) = serde_json::from_str::<LandRevision>(include_str!(
                "../../game/assets/island/navigation.json"
            )) else {
                return false;
            };
            let mut legacy = current.navigation.walkable.clone();
            for [x, y] in revision.corrections {
                if !legacy.remove(&IslandPoint { x, y }) {
                    return false;
                }
            }
            if world.navigation.walkable != legacy {
                return false;
            }
            world.navigation.walkable = current.navigation.walkable.clone();
        }
        if !world
            .positions
            .contains_key("character.protagonist.captain")
            && !world
                .casualties
                .contains_key("character.protagonist.captain")
        {
            return false;
        }
        self.island = Some(world);
        true
    }

    #[func]
    fn valid_island_save(&self, payload: GString) -> bool {
        FactionWorld::load_json(&payload.to_string()).is_ok()
    }

    #[func]
    fn create_island(&mut self) -> VarDictionary {
        self.island = Some(FactionWorld::prototype_island());
        self.island_snapshot()
    }

    #[func]
    fn configure_island_land(&mut self, cells: Array<Vector2i>, start: Vector2i) -> bool {
        if cells.is_empty() || cells.len() > 16384 {
            return false;
        }
        let land: std::collections::BTreeSet<IslandPoint> = cells
            .iter_shared()
            .map(|cell| IslandPoint {
                x: cell.x,
                y: cell.y,
            })
            .collect();
        let start = IslandPoint {
            x: start.x,
            y: start.y,
        };
        if !land.contains(&start) {
            return false;
        }
        let Some(world) = self.island.as_mut() else {
            return false;
        };
        // Initial map setup only. Never teleport a running campaign on reload.
        if world.tick != 0 || !world.travel_orders.is_empty() {
            return false;
        }
        world.navigation.walkable = land;
        world
            .positions
            .insert("character.protagonist.captain".into(), start);
        true
    }

    #[func]
    fn island_snapshot(&self) -> VarDictionary {
        let Some(world) = self.island.as_ref() else {
            return vdict! { "error" => "island_not_created" };
        };
        let mut actors = VarArray::new();
        for (id, position) in &world.positions {
            let person = world.actors[id].person.as_ref();
            let sex = person
                .map(|p| match p.sex {
                    crate::world::PersonSex::Male => "male",
                    crate::world::PersonSex::Female => "female",
                    crate::world::PersonSex::Other => "other",
                    crate::world::PersonSex::Unknown => "unknown",
                })
                .unwrap_or("unknown");
            actors.push(
                &vdict! {
                    "id" => id.as_str(), "x" => position.x, "y" => position.y,
                    "faction" => world.actors[id].faction_id.as_str(),
                    "undead" => world.actors[id].undead,
                    "madness_stage" => world.island_madness_stage(id),
                    "definition" => world.actors[id].definition_id.as_str(),
                    "name" => person.map(|p| p.display_name.as_str()).unwrap_or("Unknown unit"),
                    "biography" => person.map(|p| p.backstory.as_str()).unwrap_or(""),
                    "sex" => sex,
                    "age" => person.and_then(|p| p.age).unwrap_or(0),
                    "discussed" => person.is_some_and(|p| p.discussed),
                    "loyal_to_michael" => person.is_some_and(|p| p.loyal_to_michael),
                    "recruitment_offer" => person.map(|p| p.recruitment_offer.as_str()).unwrap_or(""),
                    "dialogue" => world.island_person_dialogue(id),
                    "news" => world.island_person_news(id),
                    "age" => person.and_then(|p| p.age).map(i64::from).unwrap_or(-1),
                    "health" => world.unit_combat.get(id).map(|v| v.health).unwrap_or(0),
                    "max_health" => world.combat_profiles.get(&world.actors[id].definition_id).map(|v| v.health).unwrap_or(0),
                    "moving" => world.travel_orders.contains_key(id)
                }
                .to_variant(),
            );
        }
        let mut buildings = VarArray::new();
        for faction in world.factions.values() {
            for building in faction.buildings.values() {
                if let Some(position) = world.navigation.destinations.get(&building.node_id) {
                    buildings.push(
                        &vdict! {
                            "id" => building.id.as_str(),
                            "faction" => building.faction_id.as_str(),
                            "archetype" => building.archetype_id.as_str(),
                            "x" => position.x, "y" => position.y,
                        "operational" => building.operational,
                        "health" => building.health,
                        "max_health" => building.max_health,
                        "level" => building.level,
                        "development_remaining" => building.development.as_ref().map_or(0, |order| order.remaining_ticks),
                        "development_ticks" => building.development.as_ref().map_or(0, |order| order.rule.ticks),
                        "construction_remaining" => building.construction.as_ref().map_or(0, |order| order.remaining_ticks),
                        "construction_ticks" => world.building_construction_ticks(building),
                        "repair_remaining" => building.repair.as_ref().map_or(0, |order| order.remaining_ticks),
                        "repair_ticks" => world.building_repair_ticks(building),
                            "queued" => building.production_queue.len() as i64
                        }
                        .to_variant(),
                    );
                }
            }
        }
        let mut land = VarArray::new();
        for point in &world.navigation.walkable {
            land.push(&Vector2i::new(point.x, point.y).to_variant());
        }
        let mut party = VarArray::new();
        let mut party_names = VarArray::new();
        for id in &world.party {
            party.push(&id.as_str().to_variant());
            let name = world
                .actors
                .get(id)
                .or_else(|| world.casualties.get(id).map(|c| &c.actor))
                .and_then(|a| a.person.as_ref())
                .map(|p| p.display_name.as_str())
                .unwrap_or("");
            party_names.push(&name.to_variant());
        }
        let salvage = world.factions.get("faction.michael")
            .and_then(|f| f.resources.get("resource.salvage")).copied().unwrap_or(0);
        let cache = world.foothold_cache.as_ref().map(|c| {
            vdict! { "x" => c.position.x, "y" => c.position.y, "remaining" => c.remaining }
        }).unwrap_or_default();
        let mut snapshot = vdict! { "tick" => world.tick as i64, "day" => world.day() as i64, "minute_of_day" => world.minute_of_day(), "paused" => world.paused, "actors" => &actors, "buildings" => &buildings, "land" => &land, "casualties" => world.casualties.len() as i64, "party" => &party, "party_names" => &party_names, "approach_target" => world.approach_target.as_deref().unwrap_or(""), "salvage" => salvage, "foothold_cache" => &cache };
        snapshot.set("workshop_repair_cost", world.foothold_repair_cost());
        match world.foothold_costs() {
            Ok((build, restore, ticks)) => {
                snapshot.set("workshop_cost", build);
                snapshot.set("restoration_cost", restore);
                snapshot.set("workshop_construction_ticks", ticks);
            }
            Err(error) => { snapshot.set("foothold_error", error.as_str()); }
        }
        snapshot
    }

    #[func]
    fn salvage_island_foothold(&mut self) -> GString {
        self.island.as_mut().ok_or_else(|| "Island is unavailable.".to_owned())
            .and_then(|world| world.salvage_foothold())
            .err().unwrap_or_default().as_str().into()
    }

    #[func]
    fn build_island_foothold(&mut self, target: Vector2i) -> GString {
        self.island.as_mut().ok_or_else(|| "Island is unavailable.".to_owned())
            .and_then(|world| world.build_foothold(IslandPoint { x: target.x, y: target.y }))
            .err().unwrap_or_default().as_str().into()
    }

    #[func]
    fn repair_island_foothold(&mut self) -> GString {
        self.island.as_mut().ok_or_else(|| "Island is unavailable.".to_owned())
            .and_then(|world| world.repair_foothold())
            .err().unwrap_or_default().as_str().into()
    }

    #[func]
    fn restore_island_foothold_person(&mut self, id: GString) -> GString {
        self.island.as_mut().ok_or_else(|| "Island is unavailable.".to_owned())
            .and_then(|world| world.restore_foothold_person(&id.to_string()))
            .err().unwrap_or_default().as_str().into()
    }

    #[func]
    fn approach_island_person(&mut self, id: GString) -> bool {
        self.island
            .as_mut()
            .is_some_and(|w| w.approach_island_person(&id.to_string()))
    }

    #[func]
    fn can_talk_island_person(&self, id: GString) -> bool {
        self.island
            .as_ref()
            .is_some_and(|w| w.can_talk_island_person(&id.to_string()))
    }

    #[func]
    fn talk_island_person(&mut self, id: GString) -> GString {
        self.island
            .as_mut()
            .map(|w| w.talk_island_person(&id.to_string()))
            .unwrap_or_default()
            .as_str()
            .into()
    }

    #[func]
    fn recruit_island_person(&mut self, id: GString) -> bool {
        self.island
            .as_mut()
            .is_some_and(|w| w.recruit_island_person(&id.to_string()))
    }

    #[func]
    fn assign_island_companion(&mut self, id: GString, slot: i64) -> bool {
        usize::try_from(slot).ok().is_some_and(|slot| {
            self.island
                .as_mut()
                .is_some_and(|w| w.assign_island_companion(&id.to_string(), slot))
        })
    }

    #[func]
    fn dismiss_island_companion(&mut self, slot: i64) -> bool {
        usize::try_from(slot).ok().is_some_and(|slot| {
            self.island
                .as_mut()
                .is_some_and(|w| w.dismiss_island_companion(slot))
        })
    }

    #[func]
    fn move_island_party(&mut self, target: Vector2i) -> bool {
        self.island.as_mut().is_some_and(|w| {
            w.move_island_party(IslandPoint {
                x: target.x,
                y: target.y,
            })
        })
    }

    #[func]
    fn party_move_failure(&self, target: Vector2i) -> GString {
        self.island
            .as_ref()
            .map(|w| {
                w.party_move_failure(IslandPoint {
                    x: target.x,
                    y: target.y,
                })
            })
            .unwrap_or_else(|| "Island unavailable.".into())
            .as_str()
            .into()
    }

    #[func]
    fn move_island_actor(&mut self, id: GString, target: Vector2i) -> bool {
        self.island.as_mut().is_some_and(|world| {
            world
                .order_move(
                    &id.to_string(),
                    IslandPoint {
                        x: target.x,
                        y: target.y,
                    },
                )
                .is_ok()
        })
    }

    #[func]
    fn tick_island(&mut self) -> VarDictionary {
        let mut strikes = VarArray::new();
        if let Some(world) = self.island.as_mut() {
            for event in world.advance_island_tick() {
                if let FactionWorldEvent::UnitStruck {
                    attacker_id,
                    target_id,
                    damage,
                    origin,
                    target_position,
                    attacker_definition,
                } = event
                {
                    strikes.push(
                        &vdict! {
                            "attacker" => attacker_id.as_str(), "target" => target_id.as_str(),
                            "damage" => damage, "definition" => attacker_definition.as_str(),
                            "origin" => Vector2i::new(origin.x, origin.y),
                            "destination" => Vector2i::new(target_position.x, target_position.y)
                        }
                        .to_variant(),
                    );
                }
            }
        }
        let mut snapshot = self.island_snapshot();
        snapshot.set("strikes", &strikes);
        snapshot
    }

    #[func]
    fn pause_island(&mut self, paused: bool) {
        if let Some(world) = self.island.as_mut() {
            world.paused = paused;
        }
    }

    #[func]
    fn create_debug_battle(&mut self) -> VarDictionary {
        self.sequence = 0;
        self.battle = Some(Battle::prototype_vertical_slice());
        snapshot_dictionary(&self.battle.as_ref().expect("battle assigned").snapshot())
    }

    #[func]
    fn start_battle(&mut self) -> Array<VarDictionary> {
        let Some(battle) = self.battle.as_mut() else {
            return self.rejection("", "battle_not_created");
        };
        let events = battle.start();
        self.records(events)
    }

    #[func]
    fn submit_command(&mut self, command: VarDictionary) -> Array<VarDictionary> {
        let command_id = field_string(&command, "command_id").unwrap_or_default();
        let envelope = match command_envelope(&command) {
            Ok(value) => value,
            Err(reason) => return self.rejection(&command_id, reason),
        };
        if envelope.battle_id != BATTLE_ID {
            return self.rejection(&command_id, "battle_id_mismatch");
        }
        let skill_command = match envelope.into_skill_command() {
            Ok(value) => value,
            Err(crate::protocol::ProtocolError::UnsupportedVersion { .. }) => {
                return self.rejection(&command_id, "unsupported_protocol_version");
            }
            Err(_) => return self.rejection(&command_id, "invalid_command_envelope"),
        };
        let Some(battle) = self.battle.as_mut() else {
            return self.rejection(&command_id, "battle_not_created");
        };
        match battle.submit(skill_command) {
            Ok(events) => self.records(events),
            Err(error) => self.rejection(&command_id, battle_error_code(&error)),
        }
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        self.battle.as_ref().map(|battle| snapshot_dictionary(&battle.snapshot())).unwrap_or_else(|| {
            let actors = VarArray::new();
            let error = vdict! { "kind" => "battle_not_created" };
            vdict! { "battle_id" => "", "phase" => "error", "actors" => &actors, "error" => &error }
        })
    }

    #[func]
    fn recommended_enemy_command(&self, command_id: GString) -> VarDictionary {
        let Some(battle) = self.battle.as_ref() else {
            return vdict! { "available" => false, "reason" => "battle_not_created" };
        };
        let Some(decision) = battle.enemy_decision() else {
            return vdict! { "available" => false, "reason" => "active_actor_is_not_hostile" };
        };
        let target_ids: Array<GString> = array![decision.target_id.0.as_str()];
        vdict! {
            "available" => true, "protocol_version" => i64::from(PROTOCOL_VERSION),
            "command_id" => &command_id, "battle_id" => BATTLE_ID,
            "actor_id" => decision.actor_id.0.as_str(), "kind" => "use_skill",
            "skill_id" => decision.skill_id.as_str(), "target_ids" => &target_ids,
            "rationale" => decision.rationale.as_str(), "raw_damage" => i64::from(decision.raw_damage),
            "guard_break_amount" => i64::from(decision.guard_break_amount),
            "guard_absorbed" => i64::from(decision.guard_absorbed),
            "vitality_damage" => i64::from(decision.vitality_damage), "lethal" => decision.lethal,
            "interception_protector_id" => decision.interception_protector_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
            "fatal_intercept_available" => decision.fatal_intercept_available,
        }
    }
}

impl Project42SimulationBridge {
    fn records(&mut self, events: Vec<BattleEvent>) -> Array<VarDictionary> {
        let snapshot = self.battle.as_ref().map(Battle::snapshot);
        let mut records = Array::new();
        for event in events {
            self.sequence += 1;
            records.push(&event_dictionary(event, self.sequence, snapshot.as_ref()));
        }
        records
    }

    fn rejection(&mut self, command_id: &str, reason: &str) -> Array<VarDictionary> {
        self.sequence += 1;
        let subjects = Array::<GString>::new();
        let payload = vdict! { "reason" => reason };
        let record = vdict! {
            "event_id" => format!("event.native.{:06}", self.sequence),
            "command_id" => command_id, "sequence" => self.sequence as i64,
            "kind" => "command_rejected", "subjects" => &subjects, "payload" => &payload,
        };
        let mut records = Array::new();
        records.push(&record);
        records
    }
}

fn command_envelope(value: &VarDictionary) -> Result<CommandEnvelope, &'static str> {
    let version = value
        .get("protocol_version")
        .and_then(|v| v.try_to::<i64>().ok())
        .ok_or("protocol_version_missing")?;
    let kind = field_string(value, "kind").ok_or("kind_missing")?;
    if kind != "use_skill" {
        return Err("unsupported_command_kind");
    }
    let target_array = value
        .get("target_ids")
        .and_then(|v| v.try_to::<Array<GString>>().ok())
        .ok_or("target_ids_invalid")?;
    Ok(CommandEnvelope {
        protocol_version: u16::try_from(version).map_err(|_| "protocol_version_invalid")?,
        command_id: field_string(value, "command_id").unwrap_or_default(),
        battle_id: field_string(value, "battle_id").unwrap_or_default(),
        actor_id: field_string(value, "actor_id").unwrap_or_default(),
        kind: CommandKind::UseSkill,
        skill_id: field_string(value, "skill_id").unwrap_or_default(),
        target_ids: target_array
            .iter_shared()
            .map(|value| value.to_string())
            .collect(),
    })
}

fn field_string(value: &VarDictionary, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.try_to::<GString>().ok())
        .map(|v| v.to_string())
}

fn snapshot_dictionary(snapshot: &BattleSnapshot) -> VarDictionary {
    let mut actors = Array::<VarDictionary>::new();
    for actor in &snapshot.actors {
        actors.push(&actor_dictionary(actor));
    }
    let mut effects = Array::<VarDictionary>::new();
    for effect in &snapshot.effects {
        effects.push(&effect_dictionary(effect));
    }
    let mut recovery_openings = Array::<VarDictionary>::new();
    for opening in &snapshot.recovery_openings {
        recovery_openings.push(&recovery_opening_dictionary(opening));
    }
    let metadata = vdict! { "source" => "rust_gdextension", "authoritative" => true };
    vdict! {
        "protocol_version" => i64::from(PROTOCOL_VERSION), "battle_id" => snapshot.battle_id.as_str(),
        "round" => snapshot.round as i64, "phase" => phase_name(&snapshot.phase),
        "active_actor_id" => snapshot.active_actor_id.as_ref().map(|id| id.0.as_str()).unwrap_or(""),
        "description" => "The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush.",
        "actors" => &actors, "effects" => &effects, "recovery_openings" => &recovery_openings, "metadata" => &metadata,
    }
}

fn actor_dictionary(actor: &Actor) -> VarDictionary {
    let mut statuses = Array::<VarDictionary>::new();
    for status in &actor.statuses {
        statuses.push(&status_dictionary(status));
    }
    vdict! {
        "id" => actor.id.0.as_str(), "display_name" => actor.display_name.as_str(),
        "faction" => faction_name(&actor.faction), "level" => i64::from(actor.level),
        "vitality" => i64::from(actor.vitality), "max_vitality" => i64::from(actor.max_vitality),
        "guard" => i64::from(actor.guard), "band" => i64::from(actor.band), "statuses" => &statuses,
    }
}

fn status_dictionary(status: &StatusInstance) -> VarDictionary {
    vdict! { "id" => status.id.as_str(), "kind" => status_name(&status.kind), "remaining_rounds" => i64::from(status.remaining_rounds), "source_id" => status.source_id.0.as_str() }
}

fn effect_dictionary(effect: &BattlefieldEffect) -> VarDictionary {
    vdict! { "effect_id" => effect.instance_id.as_str(), "source_actor_id" => effect.source_actor_id.0.as_str(), "source_skill_id" => effect.source_skill_id.as_str(), "remaining_pulses" => i64::from(effect.remaining_pulses) }
}

fn recovery_opening_dictionary(opening: &RecoveryOpening) -> VarDictionary {
    vdict! { "actor_id" => opening.actor_id.0.as_str(), "source_skill_id" => opening.source_skill_id.as_str(), "bonus_raw_damage" => i64::from(opening.bonus_raw_damage) }
}

fn event_dictionary(
    event: BattleEvent,
    sequence: u64,
    snapshot: Option<&BattleSnapshot>,
) -> VarDictionary {
    let kind;
    let mut command_id = String::new();
    let mut subjects = Array::<GString>::new();
    let mut payload = VarDictionary::new();
    macro_rules! subject {
        ($id:expr) => {
            subjects.push(&$id.0);
        };
    }
    macro_rules! command {
        ($id:expr) => {
            command_id = $id;
        };
    }
    match event {
        BattleEvent::BattleStarted { round } => {
            kind = "battle_started";
            payload.set("round", round as i64);
        }
        BattleEvent::TurnStarted { round, actor_id } => {
            kind = "turn_started";
            subject!(actor_id);
            payload.set("round", round as i64);
        }
        BattleEvent::EnemyIntentDeclared {
            actor_id,
            skill_id,
            target_ids,
            rationale,
            guard_break_amount,
            raw_damage,
            guard_absorbed,
            vitality_damage,
            lethal,
            interception_protector_id,
            fatal_intercept_available,
        } => {
            kind = "enemy_intent_declared";
            subject!(actor_id);
            for id in target_ids {
                subject!(id);
            }
            payload.set("skill_id", skill_id);
            payload.set("rationale", rationale);
            payload.set("guard_break_amount", i64::from(guard_break_amount));
            payload.set("raw_damage", i64::from(raw_damage));
            payload.set("guard_absorbed", i64::from(guard_absorbed));
            payload.set("vitality_damage", i64::from(vitality_damage));
            payload.set("lethal", lethal);
            payload.set(
                "interception_protector_id",
                interception_protector_id.map(|id| id.0).unwrap_or_default(),
            );
            payload.set("fatal_intercept_available", fatal_intercept_available);
        }
        BattleEvent::CommandAccepted {
            command_id: id,
            actor_id,
            skill_id,
        } => {
            kind = "command_accepted";
            command!(id);
            subject!(actor_id);
            payload.set("skill_id", skill_id);
        }
        BattleEvent::ActorFocused {
            command_id: id,
            actor_id,
        } => {
            kind = "actor_focused";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::DamageApplied {
            command_id: id,
            source_id,
            target_id,
            amount,
        } => {
            kind = "damage_applied";
            command!(id);
            subject!(source_id);
            subject!(target_id);
            payload.set("amount", i64::from(amount));
            let remaining = snapshot
                .and_then(|state| state.actors.iter().find(|actor| actor.id == target_id))
                .map(|actor| actor.vitality)
                .unwrap_or(0);
            payload.set("remaining_vitality", i64::from(remaining));
        }
        BattleEvent::GuardChanged {
            command_id: id,
            actor_id,
            delta,
            total,
        } => {
            kind = "guard_changed";
            command!(id);
            subject!(actor_id);
            payload.set("delta", i64::from(delta));
            payload.set("total", i64::from(total));
        }
        BattleEvent::VitalityChanged {
            command_id: id,
            actor_id,
            delta,
            total,
        } => {
            kind = "vitality_changed";
            command!(id);
            subject!(actor_id);
            payload.set("delta", i64::from(delta));
            payload.set("total", i64::from(total));
        }
        BattleEvent::StatusRemoved {
            command_id: id,
            actor_id,
            status_id,
            status_kind,
        } => {
            kind = "status_removed";
            command!(id);
            subject!(actor_id);
            payload.set("status_id", status_id);
            payload.set("status_kind", status_name(&status_kind));
        }
        BattleEvent::ActorMoved {
            command_id: id,
            actor_id,
            from_band,
            to_band,
        } => {
            kind = "actor_moved";
            command!(id);
            subject!(actor_id);
            payload.set("from_band", i64::from(from_band));
            payload.set("to_band", i64::from(to_band));
        }
        BattleEvent::InterceptionSet {
            command_id: id,
            protector_id,
            protected_id,
        } => {
            kind = "interception_set";
            command!(id);
            subject!(protector_id);
            subject!(protected_id);
        }
        BattleEvent::InterceptionTriggered {
            command_id: id,
            protector_id,
            protected_id,
            attacker_id,
        } => {
            kind = "interception_triggered";
            command!(id);
            subject!(protector_id);
            subject!(protected_id);
            subject!(attacker_id);
        }
        BattleEvent::ReactionWindowOpened {
            command_id: id,
            trigger,
            threatened_actor_id,
        } => {
            kind = "reaction_window_opened";
            command!(id);
            subject!(threatened_actor_id);
            payload.set("trigger", trigger);
        }
        BattleEvent::ReactionTriggered {
            command_id: id,
            reactor_id,
            skill_id,
            protected_id,
        } => {
            kind = "reaction_triggered";
            command!(id);
            subject!(reactor_id);
            subject!(protected_id);
            payload.set("skill_id", skill_id);
        }
        BattleEvent::DefeatPrevented {
            command_id: id,
            actor_id,
            prevented_by_skill_id,
        } => {
            kind = "defeat_prevented";
            command!(id);
            subject!(actor_id);
            payload.set("prevented_by_skill_id", prevented_by_skill_id);
        }
        BattleEvent::ActorRevived {
            command_id: id,
            actor_id,
            vitality,
        } => {
            kind = "actor_revived";
            command!(id);
            subject!(actor_id);
            payload.set("vitality", i64::from(vitality));
        }
        BattleEvent::BonusTurnGranted {
            command_id: id,
            actor_id,
        } => {
            kind = "bonus_turn_granted";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::BattlefieldEffectCreated {
            command_id: id,
            effect_id,
            source_actor_id,
            source_skill_id,
            total_pulses,
        } => {
            kind = "battlefield_effect_created";
            command!(id);
            subject!(source_actor_id);
            payload.set("effect_id", effect_id);
            payload.set("source_skill_id", source_skill_id);
            payload.set("total_pulses", i64::from(total_pulses));
        }
        BattleEvent::BattlefieldEffectPulse {
            effect_id,
            source_actor_id,
            pulses_remaining_after,
        } => {
            kind = "battlefield_effect_pulse";
            subject!(source_actor_id);
            payload.set("effect_id", effect_id);
            payload.set("pulses_remaining_after", i64::from(pulses_remaining_after));
        }
        BattleEvent::BattlefieldEffectRemoved { effect_id, reason } => {
            kind = "battlefield_effect_removed";
            payload.set("effect_id", effect_id);
            payload.set("reason", reason);
        }
        BattleEvent::RecoveryOpeningCreated {
            command_id: id,
            actor_id,
            source_skill_id,
            bonus_raw_damage,
        } => {
            kind = "recovery_opening_created";
            command!(id);
            subject!(actor_id);
            payload.set("source_skill_id", source_skill_id);
            payload.set("bonus_raw_damage", i64::from(bonus_raw_damage));
        }
        BattleEvent::RecoveryOpeningConsumed {
            command_id: id,
            actor_id,
            attacker_id,
            bonus_raw_damage,
        } => {
            kind = "recovery_opening_consumed";
            command!(id);
            subject!(actor_id);
            subject!(attacker_id);
            payload.set("bonus_raw_damage", i64::from(bonus_raw_damage));
        }
        BattleEvent::RecoveryOpeningExpired { actor_id } => {
            kind = "recovery_opening_expired";
            subject!(actor_id);
        }
        BattleEvent::ActorDefeated {
            command_id: id,
            actor_id,
        } => {
            kind = "actor_defeated";
            command!(id);
            subject!(actor_id);
        }
        BattleEvent::TurnEnded { round, actor_id } => {
            kind = "turn_ended";
            subject!(actor_id);
            payload.set("round", round as i64);
        }
        BattleEvent::RoundStarted { round } => {
            kind = "round_started";
            payload.set("round", round as i64);
        }
        BattleEvent::BattleEnded { victory } => {
            kind = "battle_ended";
            payload.set("victory", victory);
        }
    }
    vdict! { "event_id" => format!("event.native.{sequence:06}"), "command_id" => command_id, "sequence" => sequence as i64, "kind" => kind, "subjects" => &subjects, "payload" => &payload }
}

fn phase_name(value: &BattlePhase) -> &'static str {
    match value {
        BattlePhase::AwaitingActor => "awaiting_actor",
        BattlePhase::AwaitingCommand => "awaiting_command",
        BattlePhase::Resolving => "resolving",
        BattlePhase::Victory => "victory",
        BattlePhase::Defeat => "defeat",
    }
}
fn faction_name(value: &Faction) -> &'static str {
    match value {
        Faction::Party => "party",
        Faction::Hostile => "hostile",
    }
}
fn status_name(value: &StatusKind) -> &'static str {
    match value {
        StatusKind::Bleeding => "bleeding",
        StatusKind::Poisoned => "poisoned",
        StatusKind::Burning => "burning",
        StatusKind::Stunned => "stunned",
    }
}

fn battle_error_code(value: &crate::battle::BattleError) -> &'static str {
    use crate::battle::BattleError::*;
    match value {
        BattleAlreadyEnded => "battle_already_ended",
        WrongPhase { .. } => "wrong_phase",
        NotActiveActor { .. } => "not_active_actor",
        UnknownActor(_) => "unknown_actor",
        UnknownTarget(_) => "unknown_target",
        ActorDefeated(_) => "actor_defeated",
        ActorIncapacitated { .. } => "actor_incapacitated",
        SkillUnavailable { .. } => "skill_unavailable",
        TargetMustBeDefeated(_) => "target_must_be_defeated",
        FriendlyFire { .. } => "friendly_fire",
        IllegalSelfTarget { .. } => "illegal_self_target",
        IllegalTargetCount { .. } => "illegal_target_count",
        SkillOwnerMismatch { .. } => "skill_owner_mismatch",
        UnsupportedSkill(_) => "unsupported_skill",
    }
}

struct Project42Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Project42Extension {}
