//! Headless driver for the island simulation, so something other than a human
//! in Godot can play the game.
//!
//! One JSON command per line on stdin, one JSON reply per line on stdout. Every
//! command maps to a verb `FactionWorld` already has and the Godot bridge
//! already exposes: this adds no game rules of its own, so a bot playing
//! through it is playing the same game a player does.
//!
//! Land comes from `scenario_fixture`, the same deterministic island the Rust
//! proofs use, because the shipped rasteriser lives in GDScript.

use project42_sim::scenario_fixture::main_scenario;
use project42_sim::world::{FactionWorld, IslandPoint};
use serde_json::{Value, json};
use std::io::{BufRead, Write};

fn point(value: &Value, key: &str) -> Option<IslandPoint> {
    let cell = value.get(key)?;
    Some(IslandPoint {
        x: cell.get("x")?.as_i64()? as i32,
        y: cell.get("y")?.as_i64()? as i32,
    })
}

/// Strict readers. Coercing a bad value into a default makes the driver lie:
/// a fuzzer found that `pause` with a non-boolean paused the game the caller
/// asked to resume, `assign` with a nonsense slot silently wrote slot 0 and
/// evicted whoever stood there, and `tick` with a bad count advanced one tick
/// and called it success. A harness that guesses produces playtest reports
/// about a game nobody is running, so these refuse instead.
fn return_refusal(reason: String) -> Result<Value, String> {
    Ok(json!({"ok": false, "refused": reason}))
}

fn need_bool(value: &Value, key: &str) -> Result<bool, String> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{key} must be true or false"))
}

fn need_u64(value: &Value, key: &str) -> Result<u64, String> {
    match value.get(key) {
        Some(found) => found
            .as_u64()
            .ok_or_else(|| format!("{key} must be a non-negative whole number")),
        None => Err(format!("{key} is required")),
    }
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// What the bot can see. Deliberately the same shape of information the Godot
/// snapshot carries: the board, the people on it, the party, the campaign, the
/// quests and any lead waiting on a decision. Nothing hidden is leaked here --
/// no faction stockpiles, no undiscovered plans -- because a bot that can see
/// what a player cannot would report on a game nobody is playing.
fn observe(world: &FactionWorld) -> Value {
    let people: Vec<Value> = world
        .actors
        .values()
        .filter_map(|actor| {
            let person = actor.person.as_ref()?;
            let position = world.positions.get(&actor.instance_id)?;
            Some(json!({
                "id": actor.instance_id,
                "name": person.display_name,
                "faction": actor.faction_id,
                "sex": match person.sex { project42_sim::world::PersonSex::Female => "female", _ => "male" },
                "age": person.age,
                "loyal": person.loyal_to_michael,
                "discussed": person.discussed,
                "alive": person.alive_today,
                "undead": actor.undead,
                "x": position.x,
                "y": position.y,
                "backstory": person.backstory,
                "madness": world.island_madness_stage(&actor.instance_id),
            }))
        })
        .collect();

    // The dead Michael still has a claim on: a companion killed today is a
    // casualty, not gone, and her party slot is deliberately kept until he
    // gives it up or the water gives her back. A harness that only listed the
    // living reported those slots as pointing at nobody.
    let casualties: Vec<Value> = world
        .casualties
        .iter()
        .filter_map(|(id, casualty)| {
            let person = casualty.actor.person.as_ref()?;
            Some(json!({
                "id": id,
                "name": person.display_name,
                "faction": casualty.actor.faction_id,
                "loyal": person.loyal_to_michael,
                "died_tick": casualty.death_tick,
                "x": casualty.position.x,
                "y": casualty.position.y,
            }))
        })
        .collect();

    let buildings: Vec<Value> = world
        .factions
        .values()
        .flat_map(|faction| {
            faction.buildings.values().map(move |building| {
                json!({
                    "id": building.id,
                    "faction": faction.id,
                    "archetype": building.archetype_id,
                    "level": building.level,
                    "health": building.health,
                    "operational": building.operational,
                })
            })
        })
        .collect();

    let leads: Vec<Value> = world
        .open_leads()
        .iter()
        .map(|lead| {
            json!({
                "id": lead.id,
                "companion": lead.companion_id,
                "observation": lead.observation,
                "request": lead.request,
                "interpretations": lead.interpretations.iter().map(|i| json!({
                    "id": i.id, "claim": i.claim
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let quests: Vec<Value> = world
        .quest_stages
        .iter()
        .filter_map(|(quest_id, stage_id)| {
            let quest = world.rules.quests.get(quest_id)?;
            let stage = quest.stages.get(stage_id)?;
            Some(json!({
                "id": quest_id,
                "name": quest.display_name,
                "stage": stage_id,
                "objective": stage.objective,
                "done": stage.terminal.is_some(),
            }))
        })
        .collect();

    json!({
        "tick": world.tick,
        "day": world.day(),
        "paused": world.paused,
        "captain": world.positions.get("character.protagonist.captain")
            .map(|p| json!({"x": p.x, "y": p.y})),
        "captain_alive": world.actors.get("character.protagonist.captain")
            .and_then(|a| a.person.as_ref()).is_some_and(|p| p.alive_today),
        "salvage": world.stored_resource("faction.michael", "resource.salvage"),
        // The Godot snapshot shows these, so the bot gets them too. A bot with
        // less information than a player would report on a harder game than
        // anyone is actually playing.
        "salvage_caches": world.salvage_caches.iter().map(|(id, cache)| json!({
            "id": id,
            "x": cache.position.x,
            "y": cache.position.y,
            "remaining": cache.remaining,
            "label": cache.label,
        })).collect::<Vec<_>>(),
        "provisions": world.stored_resource("faction.michael", "resource.provisions"),
        "party": world.party,
        "dogs": world.mechanical_followers(),
        "nearby_salvage": world.nearby_salvage_id(),
        "repair_cost": world.foothold_repair_cost(),
        "develop_cost": world.foothold_development_cost(),
        "machine_costs": world.machine_foothold_costs(),
        "aimed_at": world.player_attack_target,
        "party_size": world.party_size(),
        "loyal_companions": world.loyal_companion_count(),
        "people": people,
        "casualties": casualties,
        "buildings": buildings,
        "leads": leads,
        "quests": quests,
        "flags": world.flags,
        "campaign": {
            "deadline_day": world.rules.campaign_clock.world_deadline_day,
            "confrontation": world.confrontation_cause().map(|c| format!("{c:?}")),
            "heat_signals": world.heat_channels(),
        },
        "eliminated_factions": world.eliminated_factions,
    })
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut world: Option<FactionWorld> = None;

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                let _ = writeln!(
                    stdout,
                    "{}",
                    json!({"ok": false, "error": error.to_string()})
                );
                let _ = stdout.flush();
                continue;
            }
        };
        let command = text(&request, "cmd");

        // `new` is the only command that works without a world.
        if command == "new" {
            match FactionWorld::from_scenario(&main_scenario()) {
                Ok(built) => {
                    let reply = json!({"ok": true, "state": observe(&built)});
                    world = Some(built);
                    let _ = writeln!(stdout, "{reply}");
                }
                Err(error) => {
                    let _ = writeln!(stdout, "{}", json!({"ok": false, "error": error}));
                }
            }
            let _ = stdout.flush();
            continue;
        }
        if command == "quit" {
            break;
        }

        let Some(w) = world.as_mut() else {
            let _ = writeln!(
                stdout,
                "{}",
                json!({"ok": false, "error": "no world; send {\"cmd\":\"new\"} first"})
            );
            let _ = stdout.flush();
            continue;
        };

        let result: Result<Value, String> = match command.as_str() {
            "observe" => Ok(json!({"ok": true})),
            "tick" => {
                let count = match request.get("count") {
                    None => Ok(1),
                    Some(_) => need_u64(&request, "count"),
                };
                match count {
                    Err(reason) => Ok(json!({"ok": false, "refused": reason})),
                    Ok(count) => {
                        let mut events = 0usize;
                        for _ in 0..count.min(100_000) {
                            events += w.advance_island_tick().len();
                        }
                        Ok(json!({"ok": true, "events": events}))
                    }
                }
            }
            "pause" => match need_bool(&request, "paused") {
                Err(reason) => return_refusal(reason),
                Ok(paused) => {
                    w.paused = paused;
                    Ok(json!({"ok": true}))
                }
            },
            // The UI moves the party, not the man. Ordering the captain alone
            // left his companions and his dogs standing where he found them,
            // which is not a move any player can make and made every playtest
            // report describe a game nobody was playing.
            "move_captain" => match point(&request, "to") {
                Some(target) => Ok(json!({"ok": w.move_island_party(target)})),
                None => Err("move_captain needs to:{x,y}".into()),
            },
            "approach" => Ok(json!({"ok": w.approach_island_person(&text(&request, "id"))})),
            "can_talk" => Ok(json!({"ok": w.can_talk_island_person(&text(&request, "id"))})),
            "talk" => {
                let said = w.talk_island_person(&text(&request, "id"));
                Ok(json!({"ok": !said.is_empty(), "said": said}))
            }
            "recruit" => Ok(json!({"ok": w.recruit_island_person(&text(&request, "id"))})),
            "assign" => {
                match need_u64(&request, "slot").and_then(|slot| {
                    usize::try_from(slot).map_err(|_| "slot is out of range".to_string())
                }) {
                    Err(reason) => return_refusal(reason),
                    Ok(slot) => Ok(json!({
                        "ok": w.assign_island_companion(&text(&request, "id"), slot)
                    })),
                }
            }
            "salvage" => match w.salvage_foothold() {
                Ok(()) => Ok(json!({"ok": true})),
                Err(error) => Ok(json!({"ok": false, "refused": error})),
            },
            "build_foothold" => match point(&request, "at") {
                Some(target) => match w.build_foothold(target) {
                    Ok(()) => Ok(json!({"ok": true})),
                    Err(error) => Ok(json!({"ok": false, "refused": error})),
                },
                None => Err("build_foothold needs at:{x,y}".into()),
            },
            "dismiss" => match need_u64(&request, "slot").and_then(|slot| {
                usize::try_from(slot).map_err(|_| "slot is out of range".to_string())
            }) {
                Err(reason) => return_refusal(reason),
                Ok(slot) => Ok(json!({"ok": w.dismiss_island_companion(slot)})),
            },
            // The workshop verbs. A bot that cannot repair, cannot build a dog
            // and cannot bring a companion back from the water is playing a
            // strictly smaller game than the one in front of a player.
            "repair" => match w.repair_foothold() {
                Ok(()) => Ok(json!({"ok": true})),
                Err(error) => Ok(json!({"ok": false, "refused": error})),
            },
            "develop" => match w.develop_foothold() {
                Ok(()) => Ok(json!({"ok": true})),
                Err(error) => Ok(json!({"ok": false, "refused": error})),
            },
            "queue_machine" => match w.queue_foothold_machine() {
                Ok(()) => Ok(json!({"ok": true})),
                Err(error) => Ok(json!({"ok": false, "refused": error})),
            },
            "restore" => match w.restore_foothold_person(&text(&request, "id")) {
                Ok(()) => Ok(json!({"ok": true})),
                Err(error) => Ok(json!({"ok": false, "refused": error})),
            },
            // Michael's carbine is the only way the player starts a fight, and
            // the only way his companions ever fire a shot.
            "aim" => Ok(json!({"ok": w.aim_carbine(&text(&request, "id"))})),
            "news" => Ok(json!({"ok": true, "news": w.island_person_news(&text(&request, "id"))})),
            "resolve_lead" => Ok(json!({
                "ok": w.resolve_lead(&text(&request, "lead"), &text(&request, "interpretation"))
            })),
            // Reloading is how the bot checks the simulation is really
            // deterministic: save, reload, run the same ticks, compare. A
            // deterministic sim that diverges after a round trip is a serious
            // bug and nothing else in the harness would catch it.
            "load" => {
                let payload = text(&request, "save");
                match FactionWorld::load_json(&payload) {
                    Ok(restored) => {
                        *w = restored;
                        Ok(json!({"ok": true}))
                    }
                    Err(error) => Ok(json!({"ok": false, "refused": error})),
                }
            }
            "save" => match w.save_json() {
                Ok(payload) => Ok(json!({"ok": true, "save": payload})),
                Err(error) => Err(error),
            },
            other => Err(format!("unknown command {other}")),
        };

        let reply = match result {
            Ok(mut value) => {
                value["state"] = observe(w);
                value
            }
            Err(error) => json!({"ok": false, "error": error}),
        };
        let _ = writeln!(stdout, "{reply}");
        let _ = stdout.flush();
    }
}
