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

use std::collections::BTreeMap;

use project42_sim::habitat::Habitats;
use project42_sim::strategy::building::{
    BuildingDefinition, BuildingDefinitions, CELL_CAPACITY_CELLS, ProductionOutput,
};
use project42_sim::strategy::elimination::RecoveryLink;
use project42_sim::strategy::faction::{
    ConceptKey, FactionDefinition, FactionDefinitions, FactionState, Relationship,
};
use project42_sim::strategy::production::{
    MachineDefinition, MachineDefinitions, MachineFamily, ProductionSkipReason,
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
        &MachineDefinitions::new(),
    );
}

/// The same, with the registries named explicitly. Every hash claim in this
/// file goes through `run_hours` and therefore through the authored buildings
/// and an empty machine registry; this exists so the probes below can say which
/// registries the hour was handed.
///
/// S16: the machine registry is empty in every hash claim because this file's
/// campaign raises no buildings, so no production timer exists to name a
/// machine record. `content/machines/` is C14's card; the probe below supplies
/// the record its own building rule names, exactly as
/// `the_authored_registry_with_one_weighted_record` supplies the faction weight
/// C9's neutral records do not carry yet.
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

/// The probe machine record this file's production probe builds, and the open
/// resource key its rule costs.
///
/// Both are the *test* namespace's. `content/machines/` is C14's card and did
/// not exist when this was written, and brief section 20 leaves the resource
/// list Open -- so the fuel key is a string a fixture's rule names, exactly as
/// S13's unit tests name one, and no resource category is invented here.
const PROBE_DOG: &str = "machine.s16.probe_dog";
const PROBE_FUEL: &str = "resource.open.fuel";
/// One run of the probe's rule. Two runs' worth is never stocked, which is what
/// makes the second interval a skip.
const ONE_RUN_OF_FUEL: u32 = 3;

/// The machine registry the probe's building rule names.
///
/// Supplied by the harness for the same reason
/// `the_authored_registry_with_one_weighted_record` supplies a weighted faction
/// record: the authored thing is deliberately neutral and cannot move the
/// simulation yet. C10's `building.machine_shop` authors a machine rule at
/// `interval_hours: 24`, `minimum_tier: 2` -- the interval and the tier this
/// probe runs on, read off the record rather than restated -- but its
/// `output_key` is the Open placeholder `resource.open.needs_decision`, because
/// no `machine.<...>` record existed to name. C14 authors those records; this
/// is what one will look like when it does.
fn a_probe_machine_registry() -> MachineDefinitions {
    let mut machines = MachineDefinitions::new();
    machines
        .insert(MachineDefinition {
            id: PROBE_DOG.into(),
            family: MachineFamily::MechanicalDog,
            fuel_requirement: 4,
            water_requirement: 2,
            ..MachineDefinition::default()
        })
        .expect("a `machine.` record with a stable ID loads");
    machines
}

/// C10's buildings, with the machine shop's machine rule pointed at the probe
/// record above and given a cost.
///
/// Everything else about the record is the authored one: the same interval, the
/// same minimum tier, the same construction hours, the same envelope. Only the
/// two things content cannot state yet are filled in -- which machine the rule
/// makes, and what one run costs.
fn the_authored_buildings_with_a_machine_rule_that_names_a_record() -> BuildingDefinitions {
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
                "the probe rewrites C10's machine rule, not one of its standing capabilities"
            );
            rule.output_key = PROBE_DOG.into();
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
    let buildings = the_authored_buildings_with_a_machine_rule_that_names_a_record();
    let machines = a_probe_machine_registry();
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
    assert_eq!(standing[0].def_id, PROBE_DOG);
    assert_eq!(standing[0].built_by_building_instance_id, SHOP);
    assert_eq!(state.buildings[SHOP].machines_produced, 1);
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
