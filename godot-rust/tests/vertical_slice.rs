//! Integration proof for `docs/CLAUDE_BACKEND_HANDOFF.md`'s "Definition of done":
//! the whole first-chapter authoritative chain, demonstrated in one deterministic
//! test run, rather than only in each package's own unit tests. This file proves
//! the modules actually compose -- `Geography`, `ExpeditionState` and `Battle`
//! wired together the way the frontend track will actually drive them -- not new
//! game rules of its own.

use std::collections::BTreeMap;

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
        "location.black_beach",
    )
    .expect("fresh campaign constructs");
    state.supplies = SupplyState {
        rations: 10,
        medicine: 2,
        coin: 0,
    };
    // Midnight puts today's individual in every habitat, so the terrace has a
    // real holder to meet rather than a hand-placed encounter.
    state
        .resolve_midnight_in(&geography, &habitats)
        .expect("resolves");
    assert_boundary_round_trips(&state);

    // 2. Travel by the safe road and persist its actual consequence.
    state.inspect(&geography);
    state
        .travel("route.black_beach.to_river_landing", &geography)
        .expect("legal route");
    let rations_before_river = state.supplies.rations;
    let travel_outcome = state
        .travel("route.river_landing.safe_road", &geography)
        .expect("legal route");
    assert_eq!(
        travel_outcome.arrived_at,
        "location.black_beach.reception_terrace"
    );
    assert!(state.supplies.rations < rations_before_river);
    assert_boundary_round_trips(&state);

    // 3. Enter Reception Terrace and resolve Guarded Strike against the
    //    individual holding it, through the command/event boundary.
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
                id: ActorId("character.heroine.betty".into()),
                display_name: "Betty".into(),
                faction: Faction::Party,
                level: 3,
                vitality: 100,
                max_vitality: 100,
                guard: 0,
                band: 0,
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
                vitality: 1,
                max_vitality: 70,
                guard: 0,
                band: 1,
                initiative: 11,
                statuses: Vec::new(),
                intercepts_for: None,
                skill_uses_remaining: BTreeMap::new(),
            },
        ],
    );
    battle.start();
    let events = battle
        .submit(SkillCommand {
            command_id: "vertical_slice.guarded_strike".into(),
            actor_id: ActorId("character.heroine.betty".into()),
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
        })
        .expect("legal command");
    assert!(events.contains(&BattleEvent::BattleEnded { victory: true }));
    assert_eq!(battle.snapshot().phase, BattlePhase::Victory);

    // 4. Persist victory; the beaten individual no longer holds the terrace
    //    today, so the party can leave instead of refighting it.
    state
        .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
        .expect("resolves");
    assert!(state.pending_encounter.is_none());
    assert!(state.begin_encounter(&geography, &habitats).is_none());

    // 4a. D3: cross into the Tomb of Returning Names. The archive core is
    //     gated on the reception space's own truth-telling observation, and
    //     the party returns to the terrace by the same door it entered.
    state
        .travel("route.reception_terrace.to_processional_ramp", &geography)
        .expect("legal route");
    state
        .travel("route.processional_ramp.to_tomb_threshold", &geography)
        .expect("legal route");
    state
        .travel("route.tomb_threshold.to_reception", &geography)
        .expect("legal route");
    let archive_core_before_discovery = state
        .travel("route.tomb_reception.to_archive_core", &geography)
        .unwrap_err();
    assert_eq!(
        archive_core_before_discovery,
        ExpeditionError::MissingDiscovery {
            route_id: "route.tomb_reception.to_archive_core".into(),
            discovery_id: "observation.tomb_reception.true_name".into(),
        }
    );
    let discovered = state.inspect(&geography);
    assert!(discovered.contains(&"observation.tomb_reception.true_name".to_owned()));
    state
        .travel("route.tomb_reception.to_archive_core", &geography)
        .expect("the truth-space discovery unlocks the archive core");
    assert_eq!(
        state.active_location_id,
        "location.tomb.returning_names.archive_core"
    );
    assert_boundary_round_trips(&state);
    state
        .travel("route.tomb_archive_core.to_reception", &geography)
        .expect("legal route");
    state
        .travel("route.tomb_reception.to_threshold", &geography)
        .expect("legal route");
    state
        .travel("route.tomb_threshold.to_processional_ramp", &geography)
        .expect("legal route");
    state
        .travel("route.processional_ramp.to_reception_terrace", &geography)
        .expect("legal route");
    assert_eq!(
        state.active_location_id,
        "location.black_beach.reception_terrace"
    );

    state
        .travel("route.reception_terrace.to_river_landing", &geography)
        .expect("legal route");
    state
        .travel("route.river_landing.to_black_beach", &geography)
        .expect("legal route");
    state
        .travel("route.black_beach.to_estate", &geography)
        .expect("legal route");
    assert_eq!(state.active_location_id, "location.black_beach.estate");
    assert_boundary_round_trips(&state);

    // 5. Take one estate action that changes a durable tactical fact.
    state.character_states.insert(
        "character.heroine.betty".into(),
        CharacterState {
            vitality: 60,
            max_vitality: 100,
            statuses: Vec::new(),
            injured: true,
        },
    );
    let rest_outcome = state.rest_at_estate().expect("resolves");
    assert_eq!(rest_outcome.medicine_spent, 1);
    assert!(!state.character_states["character.heroine.betty"].injured);
    assert!(
        state
            .household_progress
            .estate_upgrades
            .contains("estate.upgrade.infirmary_rested")
    );
    assert_boundary_round_trips(&state);

    // 6. Advance Midnight: restore a killed named person without losing their
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

    // 7. Save and reload; every legal-state fact from every prior boundary
    //    survives unchanged.
    assert_boundary_round_trips(&state);
}

fn assert_boundary_round_trips(state: &ExpeditionState) {
    let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
    assert_eq!(*state, restored);
}
