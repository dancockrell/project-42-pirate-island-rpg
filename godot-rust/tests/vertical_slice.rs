//! Integration proof for `docs/CLAUDE_BACKEND_HANDOFF.md`'s "Definition of done":
//! the whole first-chapter authoritative chain, demonstrated in one deterministic
//! test run, rather than only in each package's own unit tests. This file proves
//! the modules actually compose -- `Geography`, `ExpeditionState` and `Battle`
//! wired together the way the frontend track will actually drive them -- not new
//! game rules of its own.

use std::collections::BTreeMap;

use project42_sim::battle::Band;
use project42_sim::*;

#[test]
fn the_first_chapter_vertical_slice_runs_start_to_finish() {
    let geography = Geography::black_beach_vertical_slice();
    let habitats = Habitats::black_beach_vertical_slice();

    // 1. Start at Black Beach with the Captain and Betty.
    let mut state = ExpeditionState::new(
        7,
        vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
        ],
        "world.cell.black_beach",
    )
    .expect("fresh campaign constructs");
    // A3: a shipwrecked party owns nothing. Every ration, every dose of
    // medicine and every coin below is produced by something the party
    // actually does -- working the wreck, or beating what holds the terrace.
    state.supplies = SupplyState {
        rations: 0,
        medicine: 0,
        coin: 0,
    };
    // Midnight puts today's individual in every habitat, so the terrace has a
    // real holder to meet rather than a hand-placed encounter.
    state
        .resolve_midnight_in(&geography, &habitats)
        .expect("resolves");
    assert_boundary_round_trips(&state);

    // 2. The road inland costs food the party does not have yet, so the trip
    //    starts at the wreck. This is the economy closing: salvage is the only
    //    reason the safe road is walkable at all.
    state.inspect(&geography);
    assert!(
        state
            .legal_next_commands_with_geography(&geography)
            .contains(&"anchor_action:anchor.black_beach.salvage_point".to_owned())
    );
    state
        .travel("world.portal.black_beach_to_damaged_estate", &geography)
        .expect("the climb off the sand costs no rations");
    state
        .travel("world.portal.damaged_estate_to_river_landing", &geography)
        .expect("the river gate costs no rations");
    // Reading the bronze-green elven marker at the landing is what the estate's
    // map table works from later; the party sees it on the way past.
    let landing_observations = state.inspect(&geography);
    assert!(landing_observations.contains(&"observation.river_landing.road_marker".to_owned()));
    let empty_pack = state
        .travel(
            "world.portal.river_landing_to_reception_terrace_safe_road",
            &geography,
        )
        .unwrap_err();
    assert_eq!(
        empty_pack,
        ExpeditionError::InsufficientSupplies {
            needed: 2,
            available: 0
        }
    );
    assert_eq!(state.active_location_id, "world.cell.river_landing");
    assert_boundary_round_trips(&state);

    state
        .travel("world.portal.river_landing_to_damaged_estate", &geography)
        .expect("legal route");
    state
        .travel("world.portal.damaged_estate_to_black_beach", &geography)
        .expect("legal route");
    let salvaged = state
        .use_anchor("anchor.black_beach.salvage_point", &geography)
        .expect("the Handsome Jack is still on the sand");
    // Seed 7 on campaign day 2 salvages exactly five rations, every run.
    assert_eq!(state.campaign_day, 2);
    assert_eq!(salvaged.rations_gained, 5);
    assert_eq!(salvaged.coin_gained, 2);
    assert_eq!(state.supplies.rations, salvaged.rations_gained);
    assert_eq!(state.supplies.coin, 2);
    // The wreck gives once a day and says so.
    assert_eq!(
        state
            .use_anchor("anchor.black_beach.salvage_point", &geography)
            .unwrap_err(),
        ExpeditionError::AnchorSpentToday {
            anchor_id: "anchor.black_beach.salvage_point".into(),
            used_on_day: state.campaign_day,
        }
    );
    assert_boundary_round_trips(&state);

    // 3. Travel by the safe road and persist its actual consequence.
    state
        .travel("world.portal.black_beach_to_damaged_estate", &geography)
        .expect("legal route");
    state
        .travel("world.portal.damaged_estate_to_river_landing", &geography)
        .expect("legal route");
    let rations_before_river = state.supplies.rations;
    let travel_outcome = state
        .travel(
            "world.portal.river_landing_to_reception_terrace_safe_road",
            &geography,
        )
        .expect("the salvaged rations pay for the road");
    assert_eq!(travel_outcome.arrived_at, "world.cell.reception_terrace");
    assert_eq!(
        state.supplies.rations,
        rations_before_river - travel_outcome.supply_cost
    );
    assert_boundary_round_trips(&state);

    // 4. Enter Reception Terrace and fight the individual holding it through the
    //    command/event boundary, with the party the expedition actually carries:
    //    Captain Michael and Betty. A6 makes the Captain a battle actor, so the
    //    slice now shows him taking orders -- Reposition, then the Weapon Attack
    //    that ends the fight -- rather than standing outside the encounter.
    state.inspect(&geography);
    let battle_id = state
        .begin_encounter(&geography, &habitats)
        .expect("the terrace's individual presents an encounter")
        .battle_id
        .clone();
    assert_boundary_round_trips(&state);

    let mut battle = Battle::new(
        battle_id,
        [
            Actor {
                id: ActorId("character.protagonist.captain".into()),
                display_name: "Michael Corrigan".into(),
                faction: Faction::Party,
                level: 3,
                vitality: 90,
                max_vitality: 90,
                guard: 0,
                band: Band::PartyRear.index(),
                composure: 10,
                initiative: 10,
                statuses: Vec::new(),
                intercepts_for: None,
                skill_uses_remaining: BTreeMap::new(),
            },
            Actor {
                id: ActorId("character.heroine.betty".into()),
                display_name: "Betty".into(),
                faction: Faction::Party,
                level: 3,
                vitality: 100,
                max_vitality: 100,
                guard: 0,
                band: Band::PartyFront.index(),
                composure: 10,
                initiative: 12,
                statuses: Vec::new(),
                intercepts_for: None,
                skill_uses_remaining: BTreeMap::new(),
            },
            Actor {
                id: ActorId("enemy.raptor.razorbeak.prototype".into()),
                display_name: "Razorbeak".into(),
                faction: Faction::Hostile,
                level: 7,
                vitality: 40,
                max_vitality: 70,
                guard: 0,
                band: Band::EnemyFront.index(),
                composure: 10,
                initiative: 11,
                statuses: Vec::new(),
                intercepts_for: None,
                skill_uses_remaining: BTreeMap::new(),
            },
        ],
    );
    battle.start();
    let captain_id = ActorId("character.protagonist.captain".into());
    let razorbeak_id = ActorId("enemy.raptor.razorbeak.prototype".into());

    // Round one: Betty opens, the razorbeak answers, and the Captain steps up
    // from the party's rear band to its front. Reposition takes no target,
    // gains the guard its record authors, and ends his turn.
    battle
        .submit(SkillCommand {
            command_id: "vertical_slice.guarded_strike".into(),
            actor_id: ActorId("character.heroine.betty".into()),
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec![razorbeak_id.clone()],
        })
        .expect("legal command");
    let razorbeak_bite = battle
        .recommended_enemy_command("vertical_slice.razorbeak_round_one")
        .expect("the razorbeak declares its intent");
    battle.submit(razorbeak_bite).expect("legal enemy command");
    assert_eq!(battle.snapshot().active_actor_id, Some(captain_id.clone()));
    let moved = battle
        .submit(SkillCommand {
            command_id: "vertical_slice.captain_reposition".into(),
            actor_id: captain_id.clone(),
            skill_id: "skill.captain.reposition".into(),
            target_ids: Vec::new(),
        })
        .expect("PartyRear steps to PartyFront");
    assert!(moved.iter().any(|event| matches!(
        event,
        BattleEvent::ActorMoved { actor_id, to_band, .. }
            if actor_id == &captain_id && *to_band == Band::PartyFront.index()
    )));
    assert_eq!(
        battle.actor(&captain_id).unwrap().band,
        Band::PartyFront.index()
    );
    assert_eq!(battle.actor(&captain_id).unwrap().guard, 1);

    // Round two: Betty and the razorbeak trade again, and the Captain's Weapon
    // Attack finishes the individual holding the terrace.
    battle
        .submit(SkillCommand {
            command_id: "vertical_slice.guarded_strike_two".into(),
            actor_id: ActorId("character.heroine.betty".into()),
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec![razorbeak_id.clone()],
        })
        .expect("legal command");
    let razorbeak_bite = battle
        .recommended_enemy_command("vertical_slice.razorbeak_round_two")
        .expect("the razorbeak declares its intent");
    battle.submit(razorbeak_bite).expect("legal enemy command");
    let events = battle
        .submit(SkillCommand {
            command_id: "vertical_slice.captain_weapon_attack".into(),
            actor_id: captain_id.clone(),
            skill_id: "skill.captain.weapon_attack".into(),
            target_ids: vec![razorbeak_id.clone()],
        })
        .expect("the Captain's baseline command is legal");
    assert!(events.contains(&BattleEvent::BattleEnded { victory: true }));
    assert_eq!(battle.snapshot().phase, BattlePhase::Victory);
    assert!(battle.actor(&captain_id).unwrap().is_alive());

    // 5. Persist victory; the beaten individual no longer holds the terrace
    //    today, so the party can leave instead of refighting it -- and it pays
    //    the habitat's declared drop table, which is where the medicine the
    //    estate's infirmary spends below actually comes from.
    let supplies_before_victory = state.supplies.clone();
    let resolution = state
        .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
        .expect("resolves");
    assert!(state.pending_encounter.is_none());
    assert!(state.begin_encounter(&geography, &habitats).is_none());
    let loot = resolution
        .loot
        .expect("the terrace holder was carrying its drop");
    assert_eq!(loot.id, "loot.razorbeak.crested.prototype");
    assert_eq!(
        state.supplies.medicine,
        supplies_before_victory.medicine + loot.medicine
    );
    assert!(state.supplies.coin > supplies_before_victory.coin);

    // 5a. D3: cross into the Tomb of Returning Names. The archive core is
    //     gated on the reception space's own truth-telling observation, and
    //     the party returns to the terrace by the same door it entered.
    state
        .travel(
            "world.portal.reception_terrace_to_processional_ramp",
            &geography,
        )
        .expect("legal route");
    state
        .travel(
            "world.portal.processional_ramp_to_tomb_threshold",
            &geography,
        )
        .expect("legal route");
    state
        .travel("world.portal.tomb_threshold_to_tomb_reception", &geography)
        .expect("legal route");
    let archive_core_before_discovery = state
        .travel(
            "world.portal.tomb_reception_to_tomb_archive_core",
            &geography,
        )
        .unwrap_err();
    assert_eq!(
        archive_core_before_discovery,
        ExpeditionError::MissingDiscovery {
            route_id: "world.portal.tomb_reception_to_tomb_archive_core".into(),
            discovery_id: "observation.tomb_reception.true_name".into(),
        }
    );
    let discovered = state.inspect(&geography);
    assert!(discovered.contains(&"observation.tomb_reception.true_name".to_owned()));
    state
        .travel(
            "world.portal.tomb_reception_to_tomb_archive_core",
            &geography,
        )
        .expect("the truth-space discovery unlocks the archive core");
    assert_eq!(state.active_location_id, "world.cell.tomb_archive_core");
    assert_boundary_round_trips(&state);
    state
        .travel(
            "world.portal.tomb_archive_core_to_tomb_reception",
            &geography,
        )
        .expect("legal route");
    state
        .travel("world.portal.tomb_reception_to_tomb_threshold", &geography)
        .expect("legal route");
    state
        .travel(
            "world.portal.tomb_threshold_to_processional_ramp",
            &geography,
        )
        .expect("legal route");
    state
        .travel(
            "world.portal.processional_ramp_to_reception_terrace",
            &geography,
        )
        .expect("legal route");
    assert_eq!(state.active_location_id, "world.cell.reception_terrace");

    state
        .travel(
            "world.portal.reception_terrace_to_river_landing",
            &geography,
        )
        .expect("legal route");
    // The river gate lands the party back at the estate directly: the authored
    // map hangs the estate between the beach and the river, so coming home from
    // the road no longer detours across the sand.
    state
        .travel("world.portal.river_landing_to_damaged_estate", &geography)
        .expect("legal route");
    assert_eq!(state.active_location_id, "world.cell.damaged_estate");
    assert_boundary_round_trips(&state);

    // 6. Take one estate action that changes a durable tactical fact, paid for
    //    with medicine the party won rather than medicine it was handed.
    state.character_states.insert(
        "character.heroine.betty".into(),
        CharacterState {
            vitality: 60,
            max_vitality: 100,
            statuses: Vec::new(),
            injured: true,
        },
    );
    let rest_outcome = state
        .use_anchor("anchor.estate.infirmary", &geography)
        .expect("resolves");
    assert_eq!(rest_outcome.medicine_spent, 1);
    assert!(!state.character_states["character.heroine.betty"].injured);
    assert!(
        state
            .household_progress
            .estate_upgrades
            .contains("estate.upgrade.infirmary_rested")
    );
    assert_boundary_round_trips(&state);

    // 6a. A4: the estate is a strategic core, not a bed. The workshop turns
    //     wreck parts the party looked at on day one into a field rig that makes
    //     every road cheaper, and the map table turns the landing's elven
    //     waymark into a route along the shore that did not exist before.
    let workshop = state
        .use_anchor("anchor.estate.workshop", &geography)
        .expect("the wreck was observed on the first day");
    assert_eq!(
        workshop.upgrades_recorded,
        vec!["estate.upgrade.workshop_field_rig".to_owned()]
    );
    let map_table = state
        .use_anchor("anchor.estate.map_table", &geography)
        .expect("the waymark was read at the landing");
    assert_eq!(
        map_table.discoveries_recorded,
        vec!["discovery.map_table.tidal_cut".to_owned()]
    );
    // Both rooms are one-time gains, so neither is offered again.
    let after_the_estate = state.legal_next_commands_with_geography(&geography);
    assert!(!after_the_estate.contains(&"anchor_action:anchor.estate.infirmary".to_owned()));
    assert!(!after_the_estate.contains(&"anchor_action:anchor.estate.workshop".to_owned()));
    assert!(!after_the_estate.contains(&"anchor_action:anchor.estate.map_table".to_owned()));
    assert_boundary_round_trips(&state);

    // 7. Advance Midnight: restore a killed named person without losing their
    //    death memory, and regenerate deterministic individual habitat
    //    encounters from the day seed.
    state.named_person_memory.insert(
        "person.villager.tomas".into(),
        DeathMemory {
            killed_by_player_count: 1,
            last_death_day: Some(state.campaign_day),
            last_death_context_id: Some("encounter.prototype.returning_names".into()),
        },
    );
    let day_before_midnight = state.campaign_day;
    let yesterdays_holder = state.daily_spawn_records["world.region.black_beach.terrace_precinct"]
        .instance_id
        .clone();
    let midnight_events = state
        .resolve_midnight_in(&geography, &habitats)
        .expect("resolves");
    assert_eq!(state.campaign_day, day_before_midnight + 1);
    assert!(
        midnight_events
            .iter()
            .any(|event| matches!(event, WorldEvent::MonsterMaterialized { .. }))
    );
    let tomas = &state.named_person_memory["person.villager.tomas"];
    assert_eq!(tomas.killed_by_player_count, 1);
    assert_eq!(tomas.last_death_day, Some(day_before_midnight));

    // The terrace the party cleared yesterday is held by a new individual today.
    let todays_holder = &state.daily_spawn_records["world.region.black_beach.terrace_precinct"];
    assert_ne!(todays_holder.instance_id, yesterdays_holder);
    assert!(!state.habitat_states["world.region.black_beach.terrace_precinct"].cleared_today);

    // 8. A4: the estate's work pays off on the road the next day. The tidal cut
    //    is a portal nothing could walk before the map table was read, and it
    //    reaches the terrace from the sand in one leg instead of three. The
    //    field rig pays its ration, so the leg costs nothing at all.
    state
        .travel("world.portal.damaged_estate_to_black_beach", &geography)
        .expect("legal route");
    let rations_before_the_cut = state.supplies.rations;
    let tidal_cut = state
        .travel(
            "world.portal.black_beach_to_reception_terrace_tidal_cut",
            &geography,
        )
        .expect("the map table opened this shore route");
    assert_eq!(tidal_cut.arrived_at, "world.cell.reception_terrace");
    // The road declares one ration; the field rig takes it back off.
    assert_eq!(
        geography
            .route("world.portal.black_beach_to_reception_terrace_tidal_cut")
            .expect("the tidal cut exists")
            .supply_cost,
        1
    );
    assert_eq!(tidal_cut.supply_cost, 0);
    assert_eq!(state.supplies.rations, rations_before_the_cut);

    // 9. Save and reload; every legal-state fact from every prior boundary
    //    survives unchanged.
    assert_boundary_round_trips(&state);
}

fn assert_boundary_round_trips(state: &ExpeditionState) {
    let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
    assert_eq!(*state, restored);
}
