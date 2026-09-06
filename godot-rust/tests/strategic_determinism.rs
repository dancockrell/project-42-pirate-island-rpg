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
//! **Why the harness can bite at all.** `StrategicClock::draw_digest` folds
//! every draw, in order, into the save, so the draw sequence is under the hash
//! even where nothing acts on it. Verified by temporarily seeding a draw from
//! `SystemTime::now()` (test 1 failed: two runs, two hashes) and by temporarily
//! iterating the purposes through a `HashMap` (test 1 failed again, on
//! iteration order); both were restored.
//!
//! **B16: and the building registry beside it.** Every midnight is handed the
//! registry loaded from `content/buildings/` as well, the same three files the
//! bridge loads out of the bundle, so S10's hourly elimination sweep reads
//! C10's authored records here exactly as it reads them in the engine. A fifth
//! claim comes with that -- `the_authored_building_registry_reaches_the_hourly_sweep`
//! -- and an honest negative: this file's campaign raises no buildings, so the
//! four hash claims above are unchanged by the registry arriving.
//!
//! **B15: the harness runs the island Godot runs.** Every midnight here is
//! handed the registry loaded from `content/factions/` -- the same six files the
//! expedition bridge loads out of the Godot content bundle -- rather than the
//! empty one `resolve_midnight_in` used to build for itself. A fourth claim
//! comes with that: the registry reaches the hour and the hour is a function of
//! it (`the_authored_registry_changes_the_tick`, which replaced
//! `the_registry_cannot_change_a_tick_yet`).

use std::collections::{BTreeMap, BTreeSet};

use project42_sim::habitat::Habitats;
use project42_sim::strategy::building::{
    BuildingDefinition, BuildingDefinitions, BuildingState, CELL_CAPACITY_CELLS, ProductionOutput,
};
use project42_sim::strategy::elimination::{RecoveryLink, recovery_chain};
use project42_sim::strategy::faction::{
    ConceptKey, FactionDefinition, FactionDefinitions, FactionState, Relationship,
};
use project42_sim::strategy::production::{
    MachineDefinition, MachineDefinitions, MachineFamily, MachineInstance, ProductionSkipReason,
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

/// C9's six records, read off disk exactly as `authored_world.rs` reads content
/// and exactly as the bridge loads them from the Godot bundle.
///
/// B15: the harness used to build its own six placeholder records in code. That
/// was a second answer to "what factions exist", and while the bridge loaded
/// none it did not matter; now that the bridge loads `content/factions/`, the
/// harness reading the same files is what makes "the harness runs the same
/// island Godot does" a fact rather than a hope. Fixtures still name factions by
/// `ConceptKey` and never by a proper name -- brief section 4 refuses to invent
/// one, and `FactionDefinition` has no field a name could live in.
fn the_authored_registry() -> FactionDefinitions {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/factions/");
    let mut definitions = FactionDefinitions::new();
    for entry in std::fs::read_dir(directory).expect("content/factions/ is readable") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().and_then(|name| name.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("the faction record is readable");
        let record: FactionDefinition = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} is a FactionDefinition: {error}", path.display()));
        definitions
            .insert(record)
            .unwrap_or_else(|error| panic!("{} does not load: {error:?}", path.display()));
    }
    assert_eq!(
        definitions.len(),
        ConceptKey::ALL.len(),
        "content/factions/ must author exactly the brief's six concepts"
    );
    definitions
}

/// C10's building records, read off disk the same way -- and for the same
/// reason. Before B16 `strategic_tick` built an empty `BuildingDefinitions` for
/// itself, so the harness and the engine disagreed about what a building on the
/// island makes and how much of a cell it takes the moment either one had a
/// building standing.
fn the_authored_buildings() -> BuildingDefinitions {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/buildings/");
    let mut definitions = BuildingDefinitions::new();
    for entry in std::fs::read_dir(directory).expect("content/buildings/ is readable") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().and_then(|name| name.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("the building record is readable");
        let record: BuildingDefinition = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} is a BuildingDefinition: {error}", path.display()));
        definitions
            .insert(record)
            .unwrap_or_else(|error| panic!("{} does not load: {error:?}", path.display()));
    }
    assert!(
        !definitions.is_empty(),
        "content/buildings/ must author at least one record; C10 is what fills it"
    );
    definitions
}

/// One authored record with its weights turned up, and the other five as
/// authored.
///
/// Why a probe record exists at all: every record C9 shipped is deliberately
/// *neutral*. `resource_priorities` is the single placeholder key
/// `resource.open.needs_decision` at weight zero (the validator refuses any
/// other namespace, because brief section 20 leaves the resource list Open) and
/// `relationship_tendencies` is empty (brief section 7 is Provisional, and an
/// authored opening position would be a number acting before approval). Every
/// signal `GoalWeights::from_definition` reads is therefore zero, and
/// `nudge_weight(0)` is exactly `NEUTRAL_WEIGHT` -- so today's authored registry
/// scores identically to the empty one *by design*, and a test that only
/// compared those two would pass whether or not `run_hour` ever looked at the
/// registry.
///
/// This is what a record looks like once one of those Open questions is
/// answered. `resource.example` is the same deliberate abstract placeholder
/// `a_campaign` uses: a fixture must not pretend to know a category name.
fn the_authored_registry_with_one_weighted_record() -> FactionDefinitions {
    let mut definitions = FactionDefinitions::new();
    for id in the_authored_registry().ids() {
        let mut record = the_authored_registry()
            .get(id)
            .expect("the ID came from this registry")
            .clone();
        if record.concept_key == ConceptKey::Pirates {
            record
                .resource_priorities
                .insert("resource.example".into(), 40);
            record.relationship_tendencies.insert(
                ConceptKey::Michael.faction_id(),
                Relationship {
                    fear: 30,
                    hatred: 30,
                    perceived_opportunity: 40,
                    ..Relationship::default()
                },
            );
        }
        definitions
            .insert(record)
            .expect("each concept appears exactly once");
    }
    definitions
}

/// A campaign with an island under it and factions on the board.
///
/// `resource.example` is a deliberate abstract placeholder, following
/// `strategy/faction.rs`: brief section 20 leaves the resource list Open, so a
/// fixture must not pretend to know a category name.
///
/// **Every signal S5 reads is nonzero here, and that is B15's doing.** The
/// fixture used to stock a faction with `index` units and give it one
/// relationship carrying `fear: index` -- numbers so small that
/// `stock_percent`, `hostility_percent` and `invitation_percent` all rounded to
/// zero for every faction. While no weight was read that cost nothing. It is not
/// free now: a consideration whose *signal* is zero cannot be moved by any
/// weight, so a harness built on that board could not tell a registry that
/// reaches `run_hour` from one that does not, whatever it asserted. The
/// stockpiles and the pairwise pressures below are spread across the signal
/// range so the six factions sit in genuinely different positions and the
/// weights have something to act on.
fn a_campaign() -> ExpeditionState {
    let mut state = ExpeditionState::new(
        SEED,
        vec!["character.protagonist.captain".into()],
        "world.cell.black_beach",
    )
    .expect("a fresh campaign constructs");

    for (index, concept) in ConceptKey::ALL.into_iter().enumerate() {
        let step = index as i16;
        let mut faction = FactionState::new();
        // 0, 20, 40, 60, 80, 100 percent of `WELL_STOCKED`: one faction with
        // nothing, one fully supplied, and four in between.
        faction
            .resources
            .insert("resource.example".into(), index as u32 * 20);
        // A pairwise position toward every other faction, not toward one of
        // them, because `hostility_percent` and `invitation_percent` are means
        // over the neighbours present and a single entry among five is noise.
        for other in ConceptKey::ALL {
            if other == concept {
                continue;
            }
            faction.relationships.insert(
                other.faction_id(),
                Relationship {
                    fear: 10 * step,
                    hatred: 8 * step,
                    territorial_conflict: 6 * step,
                    recent_aggression: 4 * step,
                    perceived_opportunity: 100 - 15 * step,
                    ..Relationship::default()
                },
            );
        }
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
fn run_hours(state: &mut ExpeditionState, hours: u64, factions: &FactionDefinitions) {
    run_hours_with(
        state,
        hours,
        factions,
        &the_authored_buildings(),
        &the_authored_machines(),
    );
}

/// The same, with the registries named explicitly. Every hash claim in this
/// file goes through `run_hours` and therefore through the authored buildings
/// and the authored machines; this exists so the probes below can say which
/// registries the hour was handed.
///
/// S16: the hash claims carry the real machine registry and produce nothing out
/// of it, because this file's campaign raises no buildings and a production
/// timer belongs to a building. The probe below is where a yard is raised and
/// the same authored records make a machine.
fn run_hours_with(
    state: &mut ExpeditionState,
    hours: u64,
    factions: &FactionDefinitions,
    buildings: &BuildingDefinitions,
    machines: &MachineDefinitions,
) {
    let (geography, habitats) = island();
    assert_eq!(
        hours % u64::from(HOURS_PER_DAY),
        0,
        "the harness advances whole days, because midnight is what drives the hours"
    );
    for _ in 0..(hours / u64::from(HOURS_PER_DAY)) {
        state
            .resolve_midnight_in(&geography, &habitats, factions, buildings, machines)
            .expect("nothing in this harness blocks midnight");
    }
}

/// Done-when, first half: the same seed runs the same island twice.
#[test]
fn two_thousand_four_hundred_hours_from_seed_seven_hash_identically_twice() {
    let factions = the_authored_registry();
    let mut first = a_campaign();
    run_hours(&mut first, TOTAL_HOURS, &factions);
    let mut second = a_campaign();
    run_hours(&mut second, TOTAL_HOURS, &factions);

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
    let factions = the_authored_registry();
    let mut uninterrupted = a_campaign();
    run_hours(&mut uninterrupted, TOTAL_HOURS, &factions);

    let mut saved = a_campaign();
    run_hours(&mut saved, SAVE_AT_HOUR, &factions);
    let midpoint_json = saved.to_json();
    let mut reloaded =
        ExpeditionState::from_json(&midpoint_json).expect("the halfway save reloads");
    assert_eq!(
        reloaded.to_json(),
        midpoint_json,
        "the reload is byte-identical before it is asked to simulate anything"
    );
    run_hours(&mut reloaded, TOTAL_HOURS - SAVE_AT_HOUR, &factions);

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
    let definitions = the_authored_registry();
    let buildings = the_authored_buildings();
    const HOURS: usize = 200;

    let mut uninterrupted = a_campaign();
    for _ in 0..HOURS {
        uninterrupted.strategic_tick(
            &geography,
            &definitions,
            &buildings,
            &MachineDefinitions::new(),
        );
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
        interrupted.strategic_tick(
            &geography,
            &definitions,
            &buildings,
            &MachineDefinitions::new(),
        );
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
    let factions = the_authored_registry();
    let mut state = a_campaign();

    for day in 1..=10u32 {
        assert_eq!(state.campaign_day, day);
        state
            .resolve_midnight_in(
                &geography,
                &habitats,
                &factions,
                &the_authored_buildings(),
                &MachineDefinitions::new(),
            )
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
    let buildings = BuildingDefinitions::new();
    let mut state = a_campaign();

    let events: Vec<StrategicEvent> = (0..HOURS_PER_DAY)
        .map(|_| {
            state
                .strategic_tick(
                    &geography,
                    &definitions,
                    &buildings,
                    &MachineDefinitions::new(),
                )
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

/// The registry reaches the tick, and the tick is a function of it.
///
/// B15 replaces `the_registry_cannot_change_a_tick_yet`, which asserted the
/// opposite and said of itself that it was written to be deleted. The seam it
/// guarded is closed: the bridge loads `content/factions/`, `resolve_midnight_in`
/// passes the registry down, and `run_hour` scores it through
/// `GoalWeights::from_definition`.
///
/// Three claims, in the order they matter.
///
/// 1. **The authored records reach the hour.** Six of them, loaded from the
///    same files the bridge loads, driven through 2,400 hours of real midnights.
/// 2. **A record changes the island.** One record with an answered Open
///    question -- see `the_authored_registry_with_one_weighted_record` -- hashes
///    differently from the empty registry over the same 2,400 hours. This is the
///    bite: revert `run_hour` to `GoalWeights::default()` and this assertion
///    fails, because nothing else in the crate reads the registry.
/// 3. **Both are individually reproducible.** A registry that changed the island
///    non-deterministically would be worse than one that changed nothing.
///
/// And one honest negative, pinned rather than hidden: today's *authored*
/// registry hashes identically to the empty one, because every record C9
/// shipped weights nothing (the resource list and the pairwise pressures are
/// still Open, so the records say so instead of inventing an answer). That is a
/// fact about the content, not about the wiring -- claim 2 is what proves the
/// wiring -- and when content answers either Open question this assertion is the
/// one that will fail and tell whoever answered it that the island just moved.
#[test]
fn the_authored_registry_changes_the_tick() {
    let authored = the_authored_registry();
    assert_eq!(authored.len(), 6, "six authored records reach the harness");

    let empty = FactionDefinitions::new();
    let weighted = the_authored_registry_with_one_weighted_record();

    // The island's whole hundred days, not only its last morning. S5's verdicts
    // are recomputed from the board and the hour's draw rather than accumulated,
    // so the final save carries only the *last* hour's goals -- and comparing two
    // registries on one hour is a coin toss, not a proof. This folds the save
    // after every midnight into one number, so a difference on any of the
    // hundred days is a difference in the digest.
    let hash_with = |factions: &FactionDefinitions| {
        let mut state = a_campaign();
        let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
        for _ in 0..(TOTAL_HOURS / u64::from(HOURS_PER_DAY)) {
            run_hours(&mut state, u64::from(HOURS_PER_DAY), factions);
            digest ^= hash_of(&state);
            digest = digest.wrapping_mul(0x100_0000_01B3);
        }
        assert_eq!(state.strategic_clock.total_hours, TOTAL_HOURS);
        digest
    };

    let empty_hash = hash_with(&empty);
    let authored_hash = hash_with(&authored);
    let weighted_hash = hash_with(&weighted);

    // Claim 2: the registry is live.
    assert_ne!(
        weighted_hash, empty_hash,
        "run_hour is ignoring the registry: a weighted record ran the same island as no records"
    );

    // Claim 3: each is a reproducible island in its own right.
    assert_eq!(empty_hash, hash_with(&empty));
    assert_eq!(authored_hash, hash_with(&authored));
    assert_eq!(weighted_hash, hash_with(&weighted));

    // The honest negative.
    assert_eq!(
        authored_hash, empty_hash,
        "an authored record now weights something -- update this assertion and \
         say which Open question content answered"
    );
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

/// B16's probe: the authored building registry reaches S10's hourly sweep.
///
/// S10 wrote `eliminate_exhausted_factions` against a registry it was handed
/// and said so in its own comment: `strategic_tick` built an empty one, every
/// unlookupable building was read conservatively -- standing, productive, and
/// not to be ruined -- and that could only ever delay an elimination. This test
/// is the closing of that gap, measured through the one entry point the game
/// itself uses: `ExpeditionState::strategic_tick`.
///
/// The board is C10's `building.machine_shop`, twice, on ground Michael holds.
/// Its authored envelope is six cells (footprint four plus clearance two), so
/// two of them are exactly [`CELL_CAPACITY_CELLS`] and the cell has no room for
/// a third; its authored `ruin_state` leaves two cells of rubble each, so
/// wrecking both frees the ground again.
///
/// Three claims, and **which of them bites is worth being exact about**:
///
/// 1. **The registry reaches the sweep.** With the real records the hour reads
///    a full cell and `RecoveryLink::ValidConstructionSite` is not in Michael's
///    chain. This is the assertion that fails the moment `strategic_tick` is
///    handed `BuildingDefinitions::new()` again -- an unlookupable building
///    contributes nothing to `envelope_cells_used`, so the cell reads empty and
///    the link comes back.
/// 2. **The card's assertion.** Ruining the buildings costs Michael
///    `RecoveryLink::OperationalCoreBuilding` and
///    `RecoveryLink::WorkerProductionOrRecruitment`, reported by the sweep in
///    the hour it happened. Honestly: this pair is registry-*independent* --
///    `is_working` reads the instance, and an unlookupable building is read as
///    productive -- so it proves the sweep runs in the tick, not that it runs on
///    the records. Claim 1 is what proves the records.
/// 3. **And the records again, from the other side.** Two wrecks leave four of
///    twelve cells occupied, so `ValidConstructionSite` returns. Under an empty
///    registry it never left, so there is nothing to return.
#[test]
fn the_authored_building_registry_reaches_the_hourly_sweep() {
    const BEACH: &str = "world.cell.black_beach";
    const SHOPS: [&str; 2] = [
        "building_instance.b16.machine_shop_north",
        "building_instance.b16.machine_shop_south",
    ];

    let (geography, _habitats) = island();
    let factions = the_authored_registry();
    let buildings = the_authored_buildings();
    let michael = ConceptKey::Michael.faction_id();

    let shop = buildings
        .get("building.machine_shop")
        .expect("C10 authored building.machine_shop");
    assert_eq!(
        shop.envelope_cells() * 2,
        CELL_CAPACITY_CELLS,
        "the probe's arithmetic is the record's: two machine shops fill a cell"
    );

    let mut state = a_campaign();
    state
        .set_control(BEACH, Some(michael.clone()), &geography)
        .expect("the beach is a real cell");
    for instance_id in SHOPS {
        state
            .place_building(
                instance_id,
                "building.machine_shop",
                BEACH,
                &michael,
                &geography,
                &buildings,
            )
            .expect("Michael may raise a machine shop on ground Michael holds");
    }
    // A shop is raised under construction; the sweep reads what is *working*.
    // Nothing in the tick builds -- S13 owns who spends hours on what -- so the
    // probe puts the authored tier-one hours in deliberately.
    let finished = state.advance_construction(
        shop.tier_states
            .first()
            .expect("the record authors a first tier")
            .construction_hours,
    );
    assert_eq!(finished.len(), SHOPS.len(), "both shops finish");

    // One hour, through the one entry point. The sweep runs inside it.
    state.strategic_tick(
        &geography,
        &factions,
        &buildings,
        &MachineDefinitions::new(),
    );
    let held = &state.factions[&michael].recovery_links_held;
    assert!(
        held.contains(&RecoveryLink::OperationalCoreBuilding)
            && held.contains(&RecoveryLink::WorkerProductionOrRecruitment)
            && held.contains(&RecoveryLink::ControlledSettlement),
        "two working shops on held ground are a core, a producer and a settlement: {held:?}"
    );
    // Claim 1: the registry is live in the tick.
    assert!(
        !held.contains(&RecoveryLink::ValidConstructionSite),
        "the sweep is reading an empty building registry: two authored machine \
         shops fill the cell, so there is nowhere left to build"
    );

    for instance_id in SHOPS {
        state
            .ruin_building(instance_id)
            .expect("a standing building can be brought down");
    }
    let events = state.strategic_tick(
        &geography,
        &factions,
        &buildings,
        &MachineDefinitions::new(),
    );
    let lost: Vec<&RecoveryLink> = events
        .iter()
        .filter_map(|event| match event {
            StrategicEvent::RecoveryLinkLost { faction_id, link } if *faction_id == michael => {
                Some(link)
            }
            _ => None,
        })
        .collect();
    // Claim 2: the card's.
    assert!(
        lost.contains(&&RecoveryLink::OperationalCoreBuilding),
        "ruining every working building must cost the operational core: {lost:?}"
    );
    assert!(
        lost.contains(&&RecoveryLink::WorkerProductionOrRecruitment),
        "ruining every working building must cost the ability to make: {lost:?}"
    );
    assert!(
        !state.factions[&michael].eliminated,
        "the ground is still held, so the chain is not empty"
    );
    // Claim 3: the wrecks are read by their records too.
    assert_eq!(
        state.envelope_cells_used(BEACH, &buildings),
        shop.ruined_envelope_cells() * 2,
        "two wrecks leave exactly their authored rubble"
    );
    assert!(
        state.factions[&michael]
            .recovery_links_held
            .contains(&RecoveryLink::ValidConstructionSite),
        "clearing the shops leaves room to build again"
    );
}

/// The machine C10's machine shop rule names, and the open resource key the
/// probe makes one run of it cost.
///
/// The machine ID is content's: C14 authors `content/machines/mechanical_dog.json`
/// and C10's `production.machine_shop.automaton_frames` names it, so this
/// constant is here to be *asserted against the record*, not to stand in for
/// one. The fuel key is the probe's own: brief section 20 leaves the resource
/// list Open, C10 authors no `cost` on the rule, and a harness that needs an
/// unaffordable run has to name a key to be short of. `resource.open.` is the
/// namespace content already uses for exactly that -- the placeholder the
/// validator admits -- and no resource category is decided here.
const AUTHORED_DOG: &str = "machine.mechanical_dog";
const PROBE_FUEL: &str = "resource.open.fuel";
/// One run of the probe's rule. Two runs' worth is never stocked, which is what
/// makes the second interval a skip.
const ONE_RUN_OF_FUEL: u32 = 3;

/// `content/machines/`, loaded the way this file loads `content/buildings/`:
/// every record deserialized and inserted through the registry the simulation
/// uses, with no fixture standing in for one.
///
/// C14's `every_authored_machine_record_loads` is the owner of *whether these
/// records are well formed*; this is the harness carrying them to the tick, so
/// that the machine the probe watches come out of a yard is the authored dog
/// and not a shape this file invented.
fn the_authored_machines() -> MachineDefinitions {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/machines/");
    let mut definitions = MachineDefinitions::new();
    for entry in std::fs::read_dir(directory).expect("content/machines/ is readable") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().and_then(|name| name.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("the machine record is readable");
        let record: MachineDefinition = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} is a MachineDefinition: {error}", path.display()));
        definitions
            .insert(record)
            .unwrap_or_else(|error| panic!("{} does not load: {error:?}", path.display()));
    }
    assert!(
        definitions.get(AUTHORED_DOG).is_some(),
        "content/machines/ must author {AUTHORED_DOG}; C14 is what fills it"
    );
    definitions
}

/// C10's buildings, with a cost put on the machine shop's machine rule.
///
/// Everything else about the record is the authored one, the machine it names
/// included: the same interval, the same minimum tier, the same construction
/// hours, the same envelope. The one thing content cannot state yet is what a
/// run costs -- brief section 20 leaves the resource list Open and C10 authors
/// an empty `cost` -- and a probe for "an unstocked yard skips" needs a rule
/// that can be short of something.
fn the_authored_buildings_with_a_cost_on_the_machine_rule() -> BuildingDefinitions {
    let authored = the_authored_buildings();
    let mut definitions = BuildingDefinitions::new();
    for id in authored.ids().map(str::to_owned).collect::<Vec<String>>() {
        let mut record = authored
            .get(&id)
            .expect("the ID came from this registry")
            .clone();
        if record.id == "building.machine_shop" {
            let rule = record
                .production
                .get_mut(0)
                .expect("C10 authors the machine rule first");
            assert!(
                matches!(rule.output, ProductionOutput::Machine { .. }),
                "the probe prices C10's machine rule, not one of its standing capabilities"
            );
            assert_eq!(
                rule.output_key, AUTHORED_DOG,
                "the probe watches the machine content names, and content changed it"
            );
            assert!(
                rule.cost.is_empty(),
                "C10 authors no cost; if it starts to, the probe must stock that instead"
            );
            rule.cost = BTreeMap::from([(PROBE_FUEL.to_owned(), ONE_RUN_OF_FUEL)]);
        }
        definitions
            .insert(record)
            .expect("a record that loaded once loads again");
    }
    definitions
}

/// Raise C10's machine shop on ground Michael holds and bring it up to the tier
/// its machine rule needs, through the same methods the game uses.
///
/// Construction is not the hour's work -- S16 gave the hour production, not
/// building -- so the probe spends the authored hours itself, exactly as B16's
/// probe does.
fn a_finished_machine_shop_at_the_rule_s_tier(
    state: &mut ExpeditionState,
    geography: &Geography,
    buildings: &BuildingDefinitions,
    instance_id: &str,
) {
    const BEACH: &str = "world.cell.black_beach";
    let michael = ConceptKey::Michael.faction_id();
    let shop = buildings
        .get("building.machine_shop")
        .expect("C10 authored building.machine_shop");

    state
        .set_control(BEACH, Some(michael.clone()), geography)
        .expect("the beach is a real cell");
    state
        .place_building(
            instance_id,
            "building.machine_shop",
            BEACH,
            &michael,
            geography,
            buildings,
        )
        .expect("Michael may raise a machine shop on ground Michael holds");
    state.advance_construction(
        shop.tier(1)
            .expect("the record authors a first tier")
            .construction_hours,
    );

    let rule = &shop.production[0];
    let wanted = shop
        .tier(rule.minimum_tier)
        .expect("the machine rule's minimum tier is one the record authors");
    state
        .upgrade_building(
            instance_id,
            &wanted.construction_requirements.clone(),
            buildings,
        )
        .expect("the probe supplies exactly the flags the authored tier asks for");
    state.advance_construction(wanted.construction_hours);
    assert_eq!(
        state.buildings[instance_id].tier, rule.minimum_tier,
        "the shop stands at the tier its machine rule needs"
    );
}

/// S16's probe: a stocked yard turns out a machine on the authored interval,
/// and an unstocked one skips and says so.
///
/// The whole card, measured through the one entry point the game uses.
/// `ExpeditionState::strategic_tick` is called one hour at a time, so the hour
/// the machine appears is a fact this test can name rather than infer.
///
/// 1. **The interval is the record's.** C10 authors `interval_hours: 24` on the
///    machine shop's machine rule, and the probe reads it off the record. For
///    twenty-three hours nothing comes out; on the twenty-fourth a machine
///    stands in `ExpeditionState::machines` and the hour reports
///    `MachineProduced`. This is the assertion the countdown owns: with no
///    countdown a rule either fires every hour or never fires, and both fail
///    here.
/// 2. **It was paid for.** The faction was stocked with exactly one run's fuel
///    and holds none afterwards.
/// 3. **An unstocked yard skips, and never goes negative.** The second interval
///    comes due against an empty stockpile: one `ProductionSkipped` carrying
///    `InsufficientResource`, journalled by S11 like every other hour's event,
///    no second machine, and the stockpile still exactly zero -- `u32` cannot go
///    negative, and the check-then-spend means it is not even asked to.
/// 4. **Nothing refills it.** Sixty more hours -- two and a half more intervals
///    -- add no machine and no resource. Until a lane owns income, this is what
///    a working yard with an empty yard does, and the journal says so.
#[test]
fn a_stocked_machine_shop_turns_out_a_machine_on_the_authored_interval() {
    const SHOP: &str = "building_instance.s16.machine_shop";

    let (geography, _habitats) = island();
    let factions = the_authored_registry();
    let buildings = the_authored_buildings_with_a_cost_on_the_machine_rule();
    let machines = the_authored_machines();
    let michael = ConceptKey::Michael.faction_id();
    let interval = buildings
        .get("building.machine_shop")
        .expect("C10 authored building.machine_shop")
        .production[0]
        .interval_hours;

    let mut state = a_campaign();
    a_finished_machine_shop_at_the_rule_s_tier(&mut state, &geography, &buildings, SHOP);
    state
        .factions
        .get_mut(&michael)
        .expect("the campaign carries Michael")
        .resources
        .insert(PROBE_FUEL.into(), ONE_RUN_OF_FUEL);

    // Claim 1: nothing comes out before the interval is up.
    for hour in 1..interval {
        let events = state.strategic_tick(&geography, &factions, &buildings, &machines);
        assert!(
            state.machines_of(&michael).is_empty(),
            "hour {hour} of {interval} produced a machine early: {events:?}"
        );
        assert_eq!(
            state.buildings[SHOP].production_countdown[&0],
            interval - hour,
            "the rule's countdown is the authored interval less the hours run"
        );
    }

    let events = state.strategic_tick(&geography, &factions, &buildings, &machines);
    let produced: Vec<&StrategicEvent> = events
        .iter()
        .filter(|event| matches!(event, StrategicEvent::MachineProduced { .. }))
        .collect();
    assert_eq!(
        produced.len(),
        1,
        "the authored interval came up and the yard made exactly one machine: {events:?}"
    );
    assert!(
        matches!(
            produced[0],
            StrategicEvent::MachineProduced { faction_id, family, building_instance_id, day }
                if *faction_id == michael
                    && *family == MachineFamily::MechanicalDog
                    && building_instance_id == SHOP
                    && *day == state.campaign_day
        ),
        "the hour reports whose yard made what: {:?}",
        produced[0]
    );
    let standing = state.machines_of(&michael);
    assert_eq!(standing.len(), 1, "one machine, once");
    assert_eq!(standing[0].def_id, AUTHORED_DOG);
    assert_eq!(standing[0].built_by_building_instance_id, SHOP);
    assert_eq!(state.buildings[SHOP].machines_produced, 1);
    // The instance ID is derived from the campaign, not from a clock the host
    // owns: the yard, the rule, the day and hour it came due on, the yard's
    // machine count and the hour's economy draw. Nothing here is a wall time,
    // so the same seed replayed names the same machine.
    assert!(
        standing[0].id.contains(".d1h23."),
        "the machine names the day and hour it was made on: {}",
        standing[0].id
    );
    // Claim 2: it was paid for.
    assert_eq!(
        state.factions[&michael].resources[PROBE_FUEL], 0,
        "one run of fuel went into the machine"
    );
    // And the countdown started over at the authored interval.
    assert_eq!(state.buildings[SHOP].production_countdown[&0], interval);

    // Claim 3: the next interval comes due against an empty stockpile.
    let mut skips = Vec::new();
    for _ in 0..interval {
        skips.extend(
            state
                .strategic_tick(&geography, &factions, &buildings, &machines)
                .into_iter()
                .filter(|event| matches!(event, StrategicEvent::ProductionSkipped { .. })),
        );
    }
    assert_eq!(
        skips,
        vec![StrategicEvent::ProductionSkipped {
            faction_id: michael.clone(),
            building_instance_id: SHOP.into(),
            rule_id: "production.machine_shop.automaton_frames".into(),
            reason: ProductionSkipReason::InsufficientResource {
                key: PROBE_FUEL.into(),
                held: 0,
                needed: ONE_RUN_OF_FUEL,
            },
            day: state.campaign_day,
        }],
        "an unstocked yard skips once per interval, naming what it is short of"
    );
    assert!(
        state
            .strategic_journal
            .recent()
            .iter()
            .any(|entry| matches!(entry.event, StrategicEvent::ProductionSkipped { .. })),
        "S11 journals the skip like every other hour's event"
    );
    assert_eq!(state.machines_of(&michael).len(), 1, "and made nothing");
    assert_eq!(
        state.factions[&michael].resources[PROBE_FUEL], 0,
        "a refused run spends nothing, so the stockpile is still exactly zero"
    );

    // Claim 4: nothing refills a stockpile, and the yard goes on skipping.
    for _ in 0..(interval * 2 + interval / 2) {
        state.strategic_tick(&geography, &factions, &buildings, &machines);
    }
    assert_eq!(state.machines_of(&michael).len(), 1);
    assert_eq!(state.factions[&michael].resources[PROBE_FUEL], 0);
}

// ---------------------------------------------------------------------------
// S17: the hour's goals become acts
// ---------------------------------------------------------------------------

/// **A scripted opening, and this comment is the card's "say so".** Nothing in
/// `content/` authors an owner for any cell -- `Geography::from_authored` says
/// as much, and `a_campaign` above therefore starts on a board nobody holds. A
/// faction with no ground cannot build on it, gather from it or march out of
/// it, so an island that is never given one would prove only that every branch
/// of S17 refuses. These three assignments are the board the probes act on and
/// nothing more: no doctrine, no proper name, no preference between concepts,
/// and Captain Michael deliberately absent because brief section 5.9 gives his
/// faction to the player and `act_on_goals` refuses to act for it.
const SCRIPTED_OPENING: [(ConceptKey, &str); 3] = [
    (ConceptKey::Pirates, "world.cell.black_beach"),
    (ConceptKey::ColonialPowers, "world.cell.river_landing"),
    (ConceptKey::Elves, "world.cell.tomb_reception"),
];

/// [`a_campaign`] with [`SCRIPTED_OPENING`] on the board.
fn a_campaign_with_ground(geography: &Geography) -> ExpeditionState {
    let mut state = a_campaign();
    for (concept, cell_id) in SCRIPTED_OPENING {
        state
            .set_control(cell_id, Some(concept.faction_id()), geography)
            .expect("every cell in the scripted opening is on the slice");
    }
    state
}

/// S17's Done-when, first half: on the authored island with every authored
/// registry, the factions **act** -- and the island is still reproducible.
///
/// Four claims:
///
/// 1. **Something was built.** At least one faction acted on `Goal::Develop`,
///    paid a real record's `construction_cost` out of its stockpile and raised
///    a real record on ground it holds, and it is still standing at hour 2,400.
///    The **board** is the evidence rather than the journal: S11's window is
///    bounded, an acting island fills it in about half an in-world day, and the
///    hours the cells were first built on have long since folded into the
///    digest by hour 2,400. The event itself -- its faction, its record, its
///    cell and the stockpile it came out of -- is asserted field for field by
///    `strategy::action`'s own tests.
/// 2. **Something marched.** At least one faction acted on `Goal::Expand` or
///    `Goal::Pressure`, and the body it raised was dispatched along real roads
///    -- `origin_cell_id` is where it was sent from and `position_cell_id` is
///    where it got to, so a force that has moved has moved through S7's hops
///    and cannot have teleported.
/// 3. **Byte-identical from seed 7.** The whole of it, twice, hashing the same:
///    acting is not allowed to bring a host-dependent number into the island.
/// 4. **Save-transparent at hour 1,200.** The half-way save reloads into the
///    same hour 2,400 -- so a building half-built, a force half-way down a road
///    and a stockpile half-gathered all survive the round trip.
///
/// The bite: remove the `Goal::Develop` branch from `strategy/action.rs` and
/// claim 1 fails on the building assertion.
#[test]
fn the_authored_island_acts_and_still_reproduces_byte_for_byte() {
    let (geography, _habitats) = island();
    let factions = the_authored_registry();
    let buildings = the_authored_buildings();

    let mut first = a_campaign_with_ground(&geography);
    run_hours(&mut first, TOTAL_HOURS, &factions);
    assert_eq!(first.strategic_clock.total_hours, TOTAL_HOURS);

    // Claim 1: a faction built something, out of the authored registry, on
    // ground it holds.
    assert!(
        !first.buildings.is_empty(),
        "2,400 hours of factions holding ground raised no building at all"
    );
    let raised = first
        .buildings
        .values()
        .next()
        .expect("just asserted non-empty");
    assert!(
        buildings.get(&raised.def_id).is_some(),
        "a faction raised something the authored registry does not carry: {}",
        raised.def_id
    );
    // **S18 changed what can be asserted here, and the change is the rule.**
    // `place_building` refuses a cell its faction does not hold, so the
    // instance existing is the proof that it was raised on the raiser's own
    // ground. What is *not* true any more is that it still is: an arrival can
    // take the cell out from under a standing building, and the building stands
    // as it was, because capture versus destruction is Open (brief section 20).
    // So the assertion is the one that survives that rule -- the building is on
    // real ground, and somebody holds it.
    assert!(
        geography
            .held_by(&raised.cell_id, &first.ownership)
            .is_some(),
        "a building was raised on a cell nobody holds at all: {}",
        raised.cell_id
    );
    assert_ne!(
        raised.faction_id,
        ConceptKey::Michael.faction_id(),
        "the player directs Michael's faction; the simulation built for him"
    );

    // Claim 2: a faction marched.
    assert!(
        !first.forces.is_empty(),
        "2,400 hours of factions holding ground raised no force at all"
    );
    assert!(
        first
            .forces
            .values()
            .any(|force| force.is_marching() || force.position_cell_id != force.origin_cell_id),
        "a force was raised and never sent anywhere: {:?}",
        first.forces
    );

    // Claim 3: and the whole of it reproduces.
    let mut second = a_campaign_with_ground(&geography);
    run_hours(&mut second, TOTAL_HOURS, &factions);
    assert_eq!(
        first.to_json(),
        second.to_json(),
        "an acting island stopped being reproducible"
    );
    assert_eq!(hash_of(&first), hash_of(&second));

    // Claim 4: and survives the half-way save.
    let mut saved = a_campaign_with_ground(&geography);
    run_hours(&mut saved, SAVE_AT_HOUR, &factions);
    let midpoint_json = saved.to_json();
    let mut reloaded =
        ExpeditionState::from_json(&midpoint_json).expect("the halfway save reloads");
    assert_eq!(reloaded.to_json(), midpoint_json);
    run_hours(&mut reloaded, TOTAL_HOURS - SAVE_AT_HOUR, &factions);
    assert_eq!(
        hash_of(&reloaded),
        hash_of(&first),
        "reloading a halfway save changed an acting island's future"
    );
}

/// **S19's third Done-when: construction runs on the clock, and the harness
/// sees a building finish.**
///
/// S17 shipped `BuildingStarted` and said of it, in the variant's own
/// documentation, that nothing put hours into a building on a clock -- so every
/// building S17's `Goal::Develop` founded stood `UnderConstruction` for ever,
/// and `RecoveryLink::OperationalCoreBuilding` could not be earned by building
/// one. `run_hour` now spends one hour on every site every hour through
/// `ExpeditionState::advance_construction`, the one owner of construction time.
///
/// Three claims, on the scripted opening and the authored registries:
///
/// 1. **Something was started**, which is S17's half and the precondition for
///    this card's.
/// 2. **Something finished**, and the event names its faction, its record and
///    the cell it stands on -- checked against the board rather than trusted.
/// 3. **It is operational**, so what the hour did to the save is what the event
///    says it did.
///
/// Four days rather than a hundred: C10's records author four, eight and twelve
/// hours for a first tier, so a site founded in the first day finishes inside
/// it, and a probe that ran for a hundred days would be proving the same thing
/// slowly.
///
/// The bite: drop the `advance_construction(1)` call from `run_hour` and claim
/// 2 fails on "no building finished".
#[test]
fn a_building_founded_by_a_goal_finishes_on_the_island_s_clock() {
    let (geography, habitats) = island();
    let factions = the_authored_registry();
    let buildings = the_authored_buildings();
    let machines = the_authored_machines();

    let mut state = a_campaign_with_ground(&geography);
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(one_day_of_hours(
            &mut state, &geography, &habitats, &factions, &buildings, &machines,
        ));
    }

    // Claim 1: S17's half.
    assert!(
        events
            .iter()
            .any(|event| matches!(event, StrategicEvent::BuildingStarted { .. })),
        "no faction founded anything, so there is nothing for the clock to finish"
    );

    // Claim 2: this card's.
    let finished: Vec<&StrategicEvent> = events
        .iter()
        .filter(|event| matches!(event, StrategicEvent::BuildingFinished { .. }))
        .collect();
    assert!(
        !finished.is_empty(),
        "four days of building sites and nothing finished: construction is not \
         running on the clock"
    );

    // Claim 3: the event and the board agree.
    let StrategicEvent::BuildingFinished {
        faction_id,
        building_instance_id,
        def_id,
        cell_id,
        ..
    } = finished[0]
    else {
        unreachable!("filtered above")
    };
    let standing = state
        .buildings
        .get(building_instance_id)
        .expect("the finished building is on the board");
    assert_eq!(&standing.faction_id, faction_id);
    assert_eq!(&standing.def_id, def_id);
    assert_eq!(&standing.cell_id, cell_id);
    assert!(
        buildings.get(def_id).is_some(),
        "a building finished that the authored registry does not carry: {def_id}"
    );
    assert_eq!(
        standing.state,
        BuildingState::Operational,
        "a building the hour reported finished is standing finished in the save"
    );
    assert_eq!(
        standing.construction_hours_remaining, 0,
        "and it has no hours left in it"
    );
}

/// The weak opening the M3 done-when describes: one faction holding one cell
/// with nothing stockpiled, against a rival holding three. **Scripted, and this
/// is the card's "say so"** -- content authors no owner for any cell, so an
/// imbalance has to be put on the board for anything to be measured against it.
const M3_WEAK_CELL: &str = "world.cell.damaged_estate";
const M3_RIVAL_CELLS: [&str; 3] = [
    "world.cell.river_landing",
    "world.cell.reception_terrace",
    "world.cell.processional_ramp",
];

/// **The M3 done-when, and it passes now.** Card S18.
///
/// S17 shipped this test as its own honest negative: the M3 opening eliminated
/// nobody, because `RecoveryLink::ControlledSettlement` could never fall --
/// `ExpeditionState::ownership` was written by `set_control` alone and no lane
/// called it from a tick, so a faction that could not be pushed off its ground
/// kept that link, and `ValidConstructionSite` with it, forever. S18 gives an
/// arrival its consequence (`ExpeditionState::resolve_arrivals`), and the board
/// moves. So this is the card's test now, in two claims -- one that passes and
/// one that still says what is missing, because the second gap is real and
/// naming it is worth more than hiding it.
///
/// **S19 flipped claim 2**: the second half of this test was S18's honest
/// negative and is now a second elimination. See it below.
///
/// **Claim 1, the done-when: a faction is eliminated with its recovery chain
/// demonstrably exhausted, well inside a hundred days, byte-identically
/// twice.** The faction is Captain Michael's, and *which* faction it is is the
/// point rather than a convenience: brief section 5.9 gives his strategic
/// intent to the player, `act_on_goals` therefore takes no autonomous act for
/// him, and a harness has no player in it. So his faction is the one on this
/// island that holds ground and does nothing about it -- it never gathers,
/// never builds and never raises a body -- and a rival's column walks onto his
/// undefended cell and takes it. Every link he had goes with it, S10's sweep
/// finds the chain empty in the same hour, and he is off the board. That is
/// brief section 16 working end to end: an arrival, a change of control, a
/// chain read link by link, one elimination and no respawn.
///
/// **Scripted, and this is the card's "say so".** Content authors no owner for
/// any cell, so the imbalance is put on the board by hand: one cell to the
/// faction that will lose it, [`M3_RIVAL_CELLS`] to the faction that will take
/// it, every stockpile and every relationship cleared. Nothing else is
/// arranged and no constant is tuned.
///
/// **Claim 2, and card S19 flipped it: `ResourceReserve` falls now.** S18 left
/// this claim as the honest negative -- an *autonomous* faction in the weak
/// seat lost its cell exactly as Michael's did and was still not eliminated,
/// because the units it had gathered before the column arrived stayed in its
/// stores for ever: nothing in the crate could lower a stockpile, and taking a
/// cell deliberately does not take the stockpile that cell earned (capture
/// versus destruction is Open, brief section 20). One unit is
/// [`RESOURCE_RESERVE_FLOOR`] and a chain with one link is not exhausted, so
/// brief section 16 was right to refuse.
///
/// A machine is what lowers it. The same opening, one machine standing on the
/// weak cell and a day's reserve in the stores: every hour the machine draws
/// its authored upkeep out of that reserve, the rival column takes the ground,
/// the trickle that was topping the reserve up stops with the ground, and
/// within a day of that the reserve is at zero, `RecoveryLinkLost` names it,
/// the chain is empty and the faction is off the board -- byte-identically
/// twice. Nothing is tuned: [`GATHER_PER_HELD_CELL_PER_HOUR`] is not lowered,
/// [`RESOURCE_RESERVE_FLOOR`] is not raised, and the machine's draw is a
/// record's field rather than a constant in this file.
///
/// Both halves of the scripting say so out loud:
/// `the_authored_machines_and_one_that_burns_what_this_island_gathers` (nothing
/// on this island produces `resource.open.fuel`, so a probe record burns the
/// one key S17's trickle fills) and `a_standing_machine` (no faction but
/// Michael's can build a machine, so one is put on the board by hand).
///
/// The bite: make `resolve_arrivals` leave a held cell alone (drop the `from`
/// arm that takes an undefended rival's ground) and claim 1 fails on
/// "nothing took the cell out of the hands of the faction that never defended
/// it"; drop the `consume_machine_upkeep` call from `run_hour` and claim 2
/// fails on "an autonomous faction lost its ground and drank its reserve dry
/// and was still not eliminated".
///
/// [`GATHER_PER_HELD_CELL_PER_HOUR`]: project42_sim::strategy::action::GATHER_PER_HELD_CELL_PER_HOUR
/// [`RESOURCE_RESERVE_FLOOR`]: project42_sim::strategy::elimination::RESOURCE_RESERVE_FLOOR
#[test]
fn the_m3_opening_eliminates_a_faction_with_its_recovery_chain_exhausted() {
    let (geography, habitats) = island();
    let factions = the_authored_registry();
    let buildings = the_authored_buildings();
    let machines = the_authored_machines();
    let rival = ConceptKey::ColonialPowers.faction_id();

    // ---- Claim 1: the done-when.
    let undefended = ConceptKey::Michael.faction_id();
    let mut state = m3_opening(&geography, &undefended, &rival);
    let mut events = Vec::new();
    let mut eliminated_on_day = None;
    for day in 0..100u32 {
        events.extend(one_day_of_hours(
            &mut state, &geography, &habitats, &factions, &buildings, &machines,
        ));
        if eliminated_on_day.is_none() && state.factions[&undefended].eliminated {
            eliminated_on_day = Some(day);
        }
    }

    assert!(
        eliminated_on_day.is_some(),
        "a hundred days of the M3 opening eliminated nobody: {:?}",
        recovery_chain(&state, &undefended, &geography, &buildings)
    );

    // It was an arrival that did it, and the event names the loser.
    let took_the_cell: Vec<&StrategicEvent> = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                StrategicEvent::ControlTaken { cell_id, from, .. }
                    if cell_id == M3_WEAK_CELL && from.as_deref() == Some(undefended.as_str())
            )
        })
        .collect();
    assert_eq!(
        took_the_cell.len(),
        1,
        "nothing took the cell out of the hands of the faction that never \
         defended it, so nothing can have exhausted its chain"
    );
    let StrategicEvent::ControlTaken {
        faction_id: taker, ..
    } = took_the_cell[0]
    else {
        unreachable!("filtered above")
    };
    assert_eq!(
        *taker, rival,
        "the faction that marched is the faction that holds it"
    );

    // The chain is exhausted, link by journalled link, and the elimination is
    // reported exactly once. Brief section 16 has no "returned" event and this
    // asserts there is none.
    let lost: BTreeSet<RecoveryLink> = events
        .iter()
        .filter_map(|event| match event {
            StrategicEvent::RecoveryLinkLost { faction_id, link } if *faction_id == undefended => {
                Some(*link)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        lost,
        BTreeSet::from([
            RecoveryLink::ControlledSettlement,
            RecoveryLink::ValidConstructionSite,
        ]),
        "every way back the faction actually held is reported lost, in brief \
         section 16's own vocabulary"
    );
    let eliminations = events
        .iter()
        .filter(|event| {
            matches!(event, StrategicEvent::FactionEliminated { faction_id, .. } if *faction_id == undefended)
        })
        .count();
    assert_eq!(
        eliminations, 1,
        "an exhausted chain eliminates once and only once"
    );
    assert!(
        recovery_chain(&state, &undefended, &geography, &buildings).is_empty(),
        "and there is demonstrably nothing left"
    );
    assert!(state.factions[&undefended].has_ever_held);

    // Byte-identical twice: an island that can take ground is still an island
    // that reproduces.
    let mut again = m3_opening(&geography, &undefended, &rival);
    for _ in 0..100 {
        again
            .resolve_midnight_in(&geography, &habitats, &factions, &buildings, &machines)
            .expect("nothing in this opening blocks midnight");
    }
    assert_eq!(
        again.to_json(),
        state.to_json(),
        "a hundred days of the M3 opening stopped being reproducible"
    );
    assert_eq!(hash_of(&again), hash_of(&state));

    // ---- Claim 2: the link that used to stand for ever, and now falls.
    let autonomous = ConceptKey::FoxPeople.faction_id();
    let burners = the_authored_machines_and_one_that_burns_what_this_island_gathers();
    let mut state = m3_opening(&geography, &autonomous, &rival);
    a_standing_machine(&mut state, &autonomous, M3_WEAK_CELL, &burners);
    a_scripted_reserve(&mut state, &autonomous);

    let mut events = Vec::new();
    let mut eliminated_on_day = None;
    for day in 0..100u32 {
        events.extend(one_day_of_hours(
            &mut state, &geography, &habitats, &factions, &buildings, &burners,
        ));
        if eliminated_on_day.is_none() && state.factions[&autonomous].eliminated {
            eliminated_on_day = Some(day);
        }
    }

    assert_eq!(
        geography.held_by(M3_WEAK_CELL, &state.ownership),
        Some(rival.as_str()),
        "the weak cell falls for an autonomous faction exactly as it does for \
         the one nobody directs"
    );
    assert!(
        eliminated_on_day.is_some(),
        "an autonomous faction lost its ground and drank its reserve dry and was \
         still not eliminated in a hundred days: {:?}, holding {:?}",
        recovery_chain(&state, &autonomous, &geography, &buildings),
        state.factions[&autonomous].resources
    );
    assert!(
        recovery_chain(&state, &autonomous, &geography, &buildings).is_empty(),
        "and there is demonstrably nothing left"
    );

    // The stockpile is what fell, and the machine is what lowered it. Both
    // halves are asserted, because either alone could be an accident: a stock
    // of zero with no `MachineStarved` would mean something else spent it.
    assert_eq!(
        state.factions[&autonomous]
            .resources
            .values()
            .fold(0u32, |total, held| total + held),
        0,
        "the reserve S18 could not touch is empty"
    );
    let lost: BTreeSet<RecoveryLink> = events
        .iter()
        .filter_map(|event| match event {
            StrategicEvent::RecoveryLinkLost { faction_id, link } if *faction_id == autonomous => {
                Some(*link)
            }
            _ => None,
        })
        .collect();
    assert!(
        lost.contains(&RecoveryLink::ResourceReserve),
        "the reserve is reported lost in brief section 16's own vocabulary: {lost:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            StrategicEvent::MachineStarved { faction_id, key, .. }
                if *faction_id == autonomous && key == GATHERED_RESOURCE_KEY
        )),
        "the machine says what it ran out of, in the key the island actually fills"
    );
    let starved = state
        .machines
        .values()
        .find(|machine| machine.faction_id == autonomous)
        .expect("the scripted machine is still on the board");
    assert!(
        !starved.is_standing(
            burners
                .get(&starved.def_id)
                .expect("the probe registry carries the record it was built from")
        ),
        "a machine nobody can fuel is not standing"
    );

    // Byte-identical twice, with the reserve falling in it.
    let mut again = m3_opening(&geography, &autonomous, &rival);
    a_standing_machine(&mut again, &autonomous, M3_WEAK_CELL, &burners);
    a_scripted_reserve(&mut again, &autonomous);
    for _ in 0..100 {
        again
            .resolve_midnight_in(&geography, &habitats, &factions, &buildings, &burners)
            .expect("nothing in this opening blocks midnight");
    }
    assert_eq!(
        again.to_json(),
        state.to_json(),
        "a hundred days of a faction drinking its reserve dry stopped being \
         reproducible"
    );
}

/// The one key this island actually fills.
///
/// S17's trickle is the only income in the crate and it gathers under
/// `action::GATHERED_RESOURCE_KEY`; C14's two authored machines burn
/// `resource.open.fuel` and drink `resource.open.water`, and **nothing on this
/// island produces either**. So an authored machine on the authored island
/// starves from the hour it is built and the stockpile it cannot touch stays
/// exactly where it was -- which is true, is the honest state of the economy,
/// and measures nothing about S19's wiring.
const GATHERED_RESOURCE_KEY: &str = "resource.open.gathered";

/// The authored machine registry with **one probe record added**, burning the
/// key the island fills.
///
/// The same fixture shape as `the_authored_registry_with_one_weighted_record`
/// above, and for the same reason: content's records answer brief section 20's
/// Open resource list with placeholders, and a probe has to answer one of them
/// to have anything to measure. Three overrides on a clone of the authored dog
/// and nothing else -- its own fixture ID, both resource keys pointed at the
/// key S17's trickle fills, and **one unit an hour**, so the draw can take a
/// stockpile all the way to zero rather than stalling above it (the charge is
/// every key or none, so a machine that costs six cannot spend a reserve of
/// five).
///
/// Content is not edited to make this pass. The day something on the island
/// produces `resource.open.fuel`, this probe stops needing to exist and the
/// authored record does the same job.
fn the_authored_machines_and_one_that_burns_what_this_island_gathers() -> MachineDefinitions {
    const PROBE: &str = "machine.s19.probe.burns_what_is_gathered";
    let authored = the_authored_machines();
    let mut definitions = MachineDefinitions::new();
    for id in authored.ids().map(str::to_owned).collect::<Vec<String>>() {
        definitions
            .insert(
                authored
                    .get(&id)
                    .expect("the ID just came from this registry")
                    .clone(),
            )
            .expect("a record that loaded once loads again");
    }
    let mut probe = authored
        .get(AUTHORED_DOG)
        .expect("C14 authored the dog")
        .clone();
    probe.id = PROBE.to_owned();
    probe.fuel_resource_key = GATHERED_RESOURCE_KEY.to_owned();
    probe.water_resource_key = GATHERED_RESOURCE_KEY.to_owned();
    probe.fuel_requirement = 1;
    probe.water_requirement = 0;
    definitions.insert(probe).expect("the probe record loads");
    definitions
}

/// A day's worth of the one key this island fills, in the weak faction's
/// stores at hour zero.
///
/// `m3_opening` clears every stockpile, which is what makes claim 1's faction a
/// faction with nothing. Claim 2 is about a reserve *falling*, and a link that
/// was never held cannot be reported lost -- so the reserve it is going to lose
/// is put there deliberately, one unit for each hour of one day, and the
/// machine beside it drinks one unit an hour. Nothing is tuned to make the
/// arithmetic work: the trickle is S17's, the draw is the probe record's, and
/// twenty-four is simply a day.
const M3_SCRIPTED_RESERVE: u32 = HOURS_PER_DAY as u32;

fn a_scripted_reserve(state: &mut ExpeditionState, faction_id: &str) {
    state
        .factions
        .get_mut(faction_id)
        .expect("the campaign carries every concept")
        .resources
        .insert(GATHERED_RESOURCE_KEY.to_owned(), M3_SCRIPTED_RESERVE);
}

/// Put one machine of `faction_id` on `cell_id`, standing and full.
///
/// **Scripted, and this is the card's "say so".** A faction that is not Captain
/// Michael's has no way to build a machine -- `produce_machine` runs off a
/// building's production rule and C10 authors machine rules only on his yard --
/// so a machine belonging to anybody else has to be put on the board by hand,
/// exactly as the M3 opening's ownership is. It is built from the probe
/// registry's record, full of what that record asks for, so its first hour is
/// an ordinary hour of upkeep and not a repair.
fn a_standing_machine(
    state: &mut ExpeditionState,
    faction_id: &str,
    cell_id: &str,
    machines: &MachineDefinitions,
) {
    let definition = machines
        .get("machine.s19.probe.burns_what_is_gathered")
        .expect("the probe registry carries the probe record");
    state.machines.insert(
        "machine_instance.s19.probe".to_owned(),
        MachineInstance {
            id: "machine_instance.s19.probe".to_owned(),
            def_id: definition.id.clone(),
            faction_id: faction_id.to_owned(),
            cell_id: cell_id.to_owned(),
            built_by_building_instance_id: String::new(),
            fuel_remaining: definition.fuel_requirement,
            water_remaining: definition.water_requirement,
            damage: 0,
        },
    );
}

/// The scripted M3 opening: one cell to the faction that will lose it, three to
/// the faction that will take it, and every stockpile and relationship cleared
/// -- "no stockpile" and "no history" literally, because `a_campaign` seeds both
/// to spread S5's signals and the M3 opening is about a faction that has
/// nothing.
fn m3_opening(geography: &Geography, weak: &str, rival: &str) -> ExpeditionState {
    let mut state = a_campaign();
    for faction in state.factions.values_mut() {
        faction.resources.clear();
        faction.relationships.clear();
    }
    state
        .set_control(M3_WEAK_CELL, Some(weak.to_owned()), geography)
        .expect("the weak faction's one cell is on the slice");
    for cell_id in M3_RIVAL_CELLS {
        state
            .set_control(cell_id, Some(rival.to_owned()), geography)
            .expect("every rival cell is on the slice");
    }
    state
}

/// One whole day, run the way the game runs one, with the day's strategic
/// events kept.
///
/// `resolve_midnight_in` is the only path the game ever advances the clock by
/// and it returns the midnight's *world* events, not the hours' strategic ones;
/// S11's journal, which does hold them, is deliberately bounded and a single
/// acting day overflows its window. So the day's twenty-four hours are replayed
/// on a clone -- the same `strategic_tick` calls midnight itself makes, from
/// the same state -- and then the real state is advanced through midnight. The
/// two are asserted to agree on the board and on who is off it, so the replay
/// cannot become a second, quieter island.
fn one_day_of_hours(
    state: &mut ExpeditionState,
    geography: &Geography,
    habitats: &Habitats,
    factions: &FactionDefinitions,
    buildings: &BuildingDefinitions,
    machines: &MachineDefinitions,
) -> Vec<StrategicEvent> {
    let mut replay = state.clone();
    let mut events = Vec::new();
    for _ in 0..HOURS_PER_DAY {
        events.extend(replay.strategic_tick(geography, factions, buildings, machines));
    }
    state
        .resolve_midnight_in(geography, habitats, factions, buildings, machines)
        .expect("nothing in this opening blocks midnight");
    assert_eq!(
        replay.ownership, state.ownership,
        "the replayed day and the day the island actually ran disagree about \
         who holds the board"
    );
    let eliminated = |campaign: &ExpeditionState| -> BTreeSet<String> {
        campaign
            .factions
            .iter()
            .filter(|(_, faction)| faction.eliminated)
            .map(|(id, _)| id.clone())
            .collect()
    };
    assert_eq!(
        eliminated(&replay),
        eliminated(state),
        "the replayed day and the day the island actually ran disagree about \
         who is off the board"
    );
    events
}
