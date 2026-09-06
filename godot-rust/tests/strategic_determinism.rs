//! S4's Done-when, as its own suite: the island is reproducible, and neither
//! saving nor pausing is allowed to change what it does.
//!
//! Three claims, each with a test that fails loudly if it stops being true.
//!
//! 1. **Reproducible.** 2,400 strategic hours from seed 7 hash identically on
//!    two independent runs.
//! 2. **Save-transparent.** A save taken at hour 1,200 and reloaded reaches the
//!    same hash at hour 2,400 as the run that was never interrupted.
//! 3. **Pause-transparent.** N hours with any interleaving of "pause" -- a
//!    serialize/deserialize round trip between arbitrary hours, which is what a
//!    pause plus a quit plus a resume actually is -- produce a *byte-identical*
//!    save to N uninterrupted hours (brief section 17).
//!
//! The hash is FNV-1a over `ExpeditionState::to_json`. No new crate: the point
//! is a stable fingerprint of the save, and the save is already byte-stable by
//! construction (`BTreeMap`/`BTreeSet` everywhere).
//!
//! **Why the harness can bite at all.** No faction acts yet, so the drawn
//! numbers decide nothing -- and a harness that hashed only the clock would
//! pass no matter what happened to the draws. `StrategicClock::draw_digest`
//! folds every draw, in order, into the save, so the sequence itself is under
//! the hash. Verified by temporarily seeding a draw from
//! `SystemTime::now()` (test 1 failed: two runs, two hashes) and by
//! temporarily iterating the purposes through a `HashMap` (test 1 failed
//! again, on iteration order); both were restored.

use std::collections::BTreeMap;

use project42_sim::habitat::Habitats;
use project42_sim::strategy::faction::{
    ConceptKey, FactionDefinition, FactionDefinitions, FactionState, Relationship,
};
use project42_sim::strategy::tick::{HOURS_PER_DAY, PURPOSES, StrategicEvent, hour_draws};
use project42_sim::{ExpeditionState, Geography};

/// The card's seed. Stated once so every test in this file runs the same island.
const SEED: u64 = 7;
/// The card's horizon: one hundred in-world days.
const TOTAL_HOURS: u64 = 2_400;
/// The card's save point, exactly halfway.
const SAVE_AT_HOUR: u64 = 1_200;

/// FNV-1a over the save's bytes. Stable across machines and runs, which is all
/// a determinism fingerprint has to be.
fn hash_of(state: &ExpeditionState) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in state.to_json().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01B3);
    }
    hash
}

/// The six concepts, loaded as authored records. Fixtures name factions by
/// `ConceptKey` and never by a proper name -- brief section 4 refuses to invent
/// one, and `FactionDefinition` has no field a name could live in.
fn all_six_definitions() -> FactionDefinitions {
    let mut definitions = FactionDefinitions::new();
    for concept in ConceptKey::ALL {
        definitions
            .insert(FactionDefinition {
                id: concept.faction_id(),
                concept_key: concept,
                ..FactionDefinition::default()
            })
            .expect("each concept is authored exactly once");
    }
    definitions
}

/// A campaign with an island under it and factions on the board.
///
/// `resource.example` is a deliberate abstract placeholder, following
/// `strategy/faction.rs`: brief section 20 leaves the resource list Open, so a
/// fixture must not pretend to know a category name.
fn a_campaign() -> ExpeditionState {
    let mut state = ExpeditionState::new(
        SEED,
        vec!["character.protagonist.captain".into()],
        "world.cell.black_beach",
    )
    .expect("a fresh campaign constructs");

    for (index, concept) in ConceptKey::ALL.into_iter().enumerate() {
        let mut faction = FactionState::new();
        faction
            .resources
            .insert("resource.example".into(), index as u32);
        faction.relationships.insert(
            ConceptKey::Michael.faction_id(),
            Relationship {
                fear: index as i16,
                ..Relationship::default()
            },
        );
        state.factions.insert(concept.faction_id(), faction);
    }
    state
}

fn island() -> (Geography, Habitats) {
    (
        Geography::black_beach_vertical_slice(),
        Habitats::black_beach_vertical_slice(),
    )
}

/// Runs `hours` strategic hours, turning the character-scale day through
/// `resolve_midnight_in` every twenty-four -- which is the only way the game
/// itself ever advances the strategic clock, so the harness measures the real
/// path rather than a test-only one.
fn run_hours(state: &mut ExpeditionState, hours: u64) {
    let (geography, habitats) = island();
    assert_eq!(
        hours % u64::from(HOURS_PER_DAY),
        0,
        "the harness advances whole days, because midnight is what drives the hours"
    );
    for _ in 0..(hours / u64::from(HOURS_PER_DAY)) {
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("nothing in this harness blocks midnight");
    }
}

/// Done-when, first half: the same seed runs the same island twice.
#[test]
fn two_thousand_four_hundred_hours_from_seed_seven_hash_identically_twice() {
    let mut first = a_campaign();
    run_hours(&mut first, TOTAL_HOURS);
    let mut second = a_campaign();
    run_hours(&mut second, TOTAL_HOURS);

    assert_eq!(
        hash_of(&first),
        hash_of(&second),
        "two runs of the same seed produced different islands"
    );
    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.strategic_clock.total_hours, TOTAL_HOURS);
    assert_ne!(
        first.strategic_clock.draw_digest, 0,
        "2,400 hours of six factions must have drawn something"
    );
}

/// Done-when, second half: a save at hour 1,200 reloads into the same future.
#[test]
fn a_save_at_hour_twelve_hundred_reloads_into_the_same_hour_twenty_four_hundred() {
    let mut uninterrupted = a_campaign();
    run_hours(&mut uninterrupted, TOTAL_HOURS);

    let mut saved = a_campaign();
    run_hours(&mut saved, SAVE_AT_HOUR);
    let midpoint_json = saved.to_json();
    let mut reloaded =
        ExpeditionState::from_json(&midpoint_json).expect("the halfway save reloads");
    assert_eq!(
        reloaded.to_json(),
        midpoint_json,
        "the reload is byte-identical before it is asked to simulate anything"
    );
    run_hours(&mut reloaded, TOTAL_HOURS - SAVE_AT_HOUR);

    assert_eq!(
        hash_of(&reloaded),
        hash_of(&uninterrupted),
        "reloading a halfway save changed the island's future"
    );
}

/// Pause is not a state, and this is the proof.
///
/// The interleaving is deliberately irregular -- prime-ish gaps, so the round
/// trips land inside days, on day boundaries, and on hours nothing else is
/// aligned to -- because a pause that only ever happened at a tidy moment would
/// prove nothing about the untidy ones.
#[test]
fn pausing_at_any_hour_produces_a_byte_identical_save() {
    let (geography, _habitats) = island();
    let definitions = all_six_definitions();
    const HOURS: usize = 200;

    let mut uninterrupted = a_campaign();
    for _ in 0..HOURS {
        uninterrupted.strategic_tick(&geography, &definitions);
    }

    // "Pause" after each of these hour counts: serialize, drop the state, load
    // it again, carry on. That is exactly what quitting and resuming does, and
    // it is the strongest form of the pause the bridge implements by simply not
    // calling the tick.
    let pause_after = [1usize, 2, 5, 11, 23, 24, 25, 47, 97, 151, 199];
    let mut interrupted = a_campaign();
    for hour in 0..HOURS {
        if pause_after.contains(&hour) {
            let json = interrupted.to_json();
            interrupted = ExpeditionState::from_json(&json).expect("a paused save reloads");
            assert_eq!(interrupted.to_json(), json);
        }
        interrupted.strategic_tick(&geography, &definitions);
    }

    assert_eq!(
        interrupted.to_json(),
        uninterrupted.to_json(),
        "pausing changed the simulation"
    );
    assert_eq!(hash_of(&interrupted), hash_of(&uninterrupted));
    assert_eq!(interrupted.strategic_clock.total_hours, HOURS as u64);
}

/// The two clocks are one clock. After any number of midnights the strategic
/// hour and the character-scale day agree, and the hour is back at the top of
/// the day.
#[test]
fn the_strategic_hour_and_the_campaign_day_agree_after_every_midnight() {
    let (geography, habitats) = island();
    let mut state = a_campaign();

    for day in 1..=10u32 {
        assert_eq!(state.campaign_day, day);
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("midnight resolves");
        assert_eq!(state.campaign_day, day + 1);
        assert_eq!(
            state.strategic_clock.hour_of_day, 0,
            "a midnight must land on the top of a day"
        );
        assert_eq!(
            state.strategic_clock.total_hours,
            u64::from(day) * u64::from(HOURS_PER_DAY)
        );
        assert_eq!(
            state.strategic_clock.day_from_hours(),
            state.campaign_day,
            "the strategic clock and the character-scale day are one clock"
        );
    }
}

/// The twenty-four hours belong to the day that is ending, not to the one that
/// is starting. If they ran after the Midnight Return instead, every strategic
/// hour would draw under tomorrow's day number and the island would be a day
/// out of step with the party walking through it.
#[test]
fn midnights_hours_are_drawn_under_the_day_that_is_ending() {
    let (geography, _habitats) = island();
    let definitions = FactionDefinitions::new();
    let mut state = a_campaign();

    let events: Vec<StrategicEvent> = (0..HOURS_PER_DAY)
        .map(|_| {
            state
                .strategic_tick(&geography, &definitions)
                .into_iter()
                .next()
                .expect("an hour always reports itself")
        })
        .collect();

    for (hour, event) in events.iter().enumerate() {
        assert_eq!(
            event,
            &StrategicEvent::HourPassed {
                day: 1,
                hour: hour as u8
            }
        );
    }
}

/// While no faction acts, an hour reads only the faction states the save
/// carries -- so `resolve_midnight_in` passing an empty registry cannot change
/// what midnight does.
///
/// **This test is written to be deleted.** The moment S5 scores an authored
/// record it will fail, and the failure is the instruction: the bridge must
/// start loading `content/factions/*.json` and handing it to
/// `resolve_midnight_in`.
#[test]
fn the_registry_cannot_change_a_tick_yet() {
    let (geography, _habitats) = island();

    let mut with_records = a_campaign();
    let mut without_records = a_campaign();
    for _ in 0..HOURS_PER_DAY {
        with_records.strategic_tick(&geography, &all_six_definitions());
        without_records.strategic_tick(&geography, &FactionDefinitions::new());
    }

    assert_eq!(with_records.to_json(), without_records.to_json());
}

/// The draw sequence S5 inherits, pinned as a fixture: seven purposes per
/// faction per hour, in this order, keyed by faction ID. Changing the list or
/// its order changes every island ever saved, so it changes this test first.
#[test]
fn the_per_faction_per_hour_draw_sequence_is_fixed() {
    let ordered: Vec<&str> = PURPOSES.to_vec();
    assert_eq!(
        ordered,
        vec![
            "strategic.board_position",
            "strategic.goal",
            "strategic.directive",
            "strategic.economy",
            "strategic.force",
            "strategic.relationship",
            "strategic.recovery",
        ]
    );

    // Six factions, seven purposes, one hour: forty-two distinct numbers, and
    // no faction shares one with another.
    let mut seen: BTreeMap<u64, String> = BTreeMap::new();
    for concept in ConceptKey::ALL {
        let id = concept.faction_id();
        let draws = hour_draws(SEED, 1, 0, &id);
        assert_eq!(draws.len(), PURPOSES.len());
        for (purpose, value) in draws {
            let key = format!("{id}/{purpose}");
            if let Some(previous) = seen.insert(value, key.clone()) {
                panic!("{key} collided with {previous}");
            }
        }
    }
    assert_eq!(seen.len(), ConceptKey::ALL.len() * PURPOSES.len());
}
