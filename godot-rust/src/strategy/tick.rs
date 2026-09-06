//! S4: the strategic clock. One in-world hour, the sequence of random draws
//! that hour is entitled to make, and the determinism that makes a save
//! reload into exactly the future it was saved out of.
//! `docs/SHIP_PLAN.md` section 7, card S4.
//!
//! **There is no faction behaviour in this file, and that is deliberate.**
//! S5 owns the utility AI, S6 the directives, S7 the offscreen forces, S8 the
//! weather and corruption clocks, S10 the recovery chain; brief sections
//! 6.1-6.4 are Provisional and may not act at all until S15. So an hour here
//! does the three mechanical things and stops:
//!
//! 1. it advances [`StrategicClock`];
//! 2. for every faction the save carries, it makes [`PURPOSES`]' draws, in
//!    that fixed order, through [`strategic_draw`];
//! 3. it emits [`StrategicEvent::HourPassed`] and nothing invented.
//!
//! An hour with nothing to decide is still an hour. Fixing the draw *sequence*
//! now is the point of doing this before S5 rather than after: once the number
//! of draws per faction per hour is settled, S5 can decide what each drawn
//! number means without moving any other faction's numbers, and every save
//! written between now and then keeps reproducing the same island.
//!
//! ## Pause is not a state
//!
//! Brief section 14 and section 17: the game pauses normally, and "normal
//! pausing must not change simulation outcomes". The way to guarantee that is
//! to give pause nothing to change. There is no `paused` field in
//! `ExpeditionState`, no paused branch in this file, and no clock that reads
//! the host machine. A paused game is a game whose bridge is not calling
//! [`ExpeditionState::strategic_tick`](crate::expedition::ExpeditionState::strategic_tick),
//! and a game that was saved, closed, reopened and resumed is the same thing
//! for longer. `godot-rust/tests/strategic_determinism.rs` proves it by
//! round-tripping the save between arbitrary ticks and comparing bytes.
//!
//! ## One clock, not two
//!
//! `campaign_day` is the character scale; [`StrategicClock`] is the strategic
//! scale; they are the same clock read at two resolutions.
//! `ExpeditionState::resolve_midnight_in` drives twenty-four hours before the
//! character-scale Midnight Return, so after any midnight
//! `total_hours == (campaign_day - 1) * 24` and `hour_of_day == 0`. Nothing
//! else in the crate may move either half on its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::expedition::ExpeditionState;
use crate::geography::Geography;
use crate::strategy::elimination::RecoveryLink;
use crate::strategy::faction::FactionDefinitions;
use crate::strategy::force::{ForceId, HaltReason};
use crate::strategy::production::MachineFamily;
use crate::strategy::utility::{BoardView, GoalWeights, choose_goals, recompute_strategic_state};
use crate::world::mix_seed;

/// Hours in an in-world day. The one place the number lives; `resolve_midnight_in`
/// ticks exactly this many times per midnight.
pub const HOURS_PER_DAY: u8 = 24;

/// Where the strategic simulation stands in the day, and how far it has run.
///
/// Two numbers rather than one because they answer different questions and
/// neither can be derived from the other alone: `hour_of_day` is *when* the
/// island is, and `total_hours` is *how much* island has happened -- which
/// stays meaningful across a `campaign_day` that a story beat could one day
/// move. `draw_digest` is neither; see [`StrategicClock::draw_digest`].
///
/// `serde(default)` throughout, so a save written before the strategic clock
/// existed loads at hour zero of its own day rather than failing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategicClock {
    /// `0..HOURS_PER_DAY`. Zero is the hour that begins a campaign day, so a
    /// save taken between midnights resumes inside the same day it left.
    #[serde(default)]
    pub hour_of_day: u8,
    /// Every strategic hour this campaign has ever run. Monotonic: nothing
    /// rewinds it, and pausing does not advance it.
    #[serde(default)]
    pub total_hours: u64,
    /// A running fold of every draw every hour has made, in the order made.
    ///
    /// It is the determinism witness brief section 17 asks for. While no
    /// faction acts, the drawn numbers decide nothing -- so without this the
    /// save would carry no evidence that the draws happened at all, and the
    /// harness could not tell a correct island from one whose draw order had
    /// silently changed. Folding them into one `u64` keeps that evidence
    /// bounded (S11 owns the unbounded record) and makes any change to the
    /// draw sequence, the purpose list, or the set of acting factions show up
    /// immediately as a different save.
    ///
    /// It is a witness, not an input: nothing draws from it, so S5 may start
    /// consuming the draws without disturbing it.
    #[serde(default)]
    pub draw_digest: u64,
}

impl StrategicClock {
    /// The clock a campaign starts on: the first hour of its first day.
    pub fn new() -> Self {
        Self::default()
    }

    /// The strategic clock's own opinion of which campaign day it is in,
    /// counting from day one. Equal to `ExpeditionState::campaign_day` exactly
    /// when the two clocks have been driven together -- which is what
    /// `resolve_midnight_in` guarantees and what the determinism suite asserts.
    pub fn day_from_hours(&self) -> u32 {
        (self.total_hours / u64::from(HOURS_PER_DAY)) as u32 + 1
    }

    /// Move one hour on, wrapping the day. Private to the tick: an hour passes
    /// because the simulation ran it, never because something wanted the
    /// number to be different.
    fn advance_one_hour(&mut self) {
        self.total_hours = self.total_hours.saturating_add(1);
        self.hour_of_day = (self.hour_of_day + 1) % HOURS_PER_DAY;
    }

    /// Fold one drawn number into the witness. Order-sensitive on purpose: the
    /// same draws made in a different order are a different island.
    fn absorb(&mut self, draw: u64) {
        self.draw_digest = (self.draw_digest ^ draw)
            .wrapping_mul(0x100_0000_01B3)
            .rotate_left(7);
    }
}

/// What a strategic hour did, as reported to whoever ran it.
///
/// One variant today, because one thing happens today. S11 extends this enum
/// with the journal's vocabulary and owns the stored, capped history; S4
/// stores nothing and returns everything, so no two owners keep a record of
/// the same event.
///
/// The card's rule for this enum, and the reason it stays this small: emit
/// what happened and nothing invented. A tick that decided nothing reports
/// that it decided nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategicEvent {
    /// One in-world hour of `day` elapsed. `hour` is the hour that just ran
    /// (`0..HOURS_PER_DAY`), not the one about to.
    HourPassed { day: u32, hour: u8 },
    /// S7: a force was given a destination and set off. `steps` is how many
    /// roads its planned route holds, so a reader knows how far it has to go
    /// without re-walking the graph. Emitted by
    /// [`ExpeditionState::dispatch_force`](crate::expedition::ExpeditionState::dispatch_force)
    /// and handed to its caller to journal: an order given from outside the
    /// hour is not the hour's event.
    ForceDeparted {
        force_id: ForceId,
        faction_id: String,
        from_cell_id: String,
        destination_cell_id: String,
        steps: u32,
    },
    /// S7: a force crossed one real road. Every position change a force ever
    /// makes emits one of these, which is what makes "forces never teleport"
    /// checkable from the journal alone rather than only from the code.
    ForceMoved {
        force_id: ForceId,
        faction_id: String,
        route_id: String,
        from_cell_id: String,
        to_cell_id: String,
    },
    /// S7: a force reached the end of its route, **carrying its full
    /// composition**. Brief section 10 requires composition, damage and supply
    /// to survive the aggregate-to-local transition, so the arrival hands the
    /// materialiser (B11's spawn sockets) what to build rather than a strength
    /// number to guess from.
    ForceArrived {
        force_id: ForceId,
        faction_id: String,
        cell_id: String,
        composition: BTreeMap<String, u32>,
        strength: u32,
        readiness: u8,
        supply: u32,
    },
    /// S7: a force stopped short of where it was sent, and why. It is still on
    /// the board, still whole, still holding its orders -- a force that cannot
    /// march does not vanish.
    ForceHalted {
        force_id: ForceId,
        faction_id: String,
        cell_id: String,
        reason: HaltReason,
    },
    /// S10: a faction lost one of brief section 16's ways back. Emitted the
    /// hour the link stops being there, so the journal carries the *order* a
    /// faction was taken apart in rather than only the fact that it was.
    RecoveryLinkLost {
        faction_id: String,
        link: RecoveryLink,
    },
    /// S10: a faction's recovery chain ran out and it is off the board, on
    /// `day`. There is no matching "returned" variant, and that absence is the
    /// design: brief section 16's "the eliminated faction does not
    /// automatically respawn".
    FactionEliminated { faction_id: String, day: u32 },
    /// S13: one of Captain Michael's buildings turned out a machine. Carries
    /// the family rather than the machine's instance ID because the journal is
    /// a record of *what the island did*, and "a mechanical dog came out of
    /// that yard" survives the machine itself being wrecked, salvaged or
    /// renamed.
    ///
    /// Emitted by
    /// [`ExpeditionState::produce_machine`](crate::expedition::ExpeditionState::produce_machine)
    /// and handed to its caller to journal: production ordered from outside the
    /// hour is not the hour's event, exactly as a force's departure is not.
    MachineProduced {
        faction_id: String,
        family: MachineFamily,
        building_instance_id: String,
        day: u32,
    },
}

/// The draws one faction is entitled to make in one hour, in the order it
/// makes them. **This list and its order are the contract S5 builds on.**
///
/// Each entry names the lane that will consume it, so a purpose cannot be
/// quietly repurposed:
///
/// | purpose | consumer |
/// |---|---|
/// | `strategic.board_position` | S5 recomputes `StrategicState` from the board |
/// | `strategic.goal` | S5's utility scoring, including its bounded personality variation |
/// | `strategic.directive` | S6 chooses and explains a `StrategicDirective` |
/// | `strategic.economy` | S3/S13 construction and production |
/// | `strategic.force` | S7 moves and materialises offscreen forces |
/// | `strategic.relationship` | S6 moves brief section 7's pairwise pressures |
/// | `strategic.recovery` | S10's recovery chain and elimination check |
///
/// Appending a purpose changes every future island (the digest sees it), which
/// is why the list is stated once, here, rather than assembled at a call site.
/// These are *purposes*, not resource categories: brief section 20 leaves the
/// resource list Open and nothing here names one.
pub const PURPOSES: [&str; 7] = [
    "strategic.board_position",
    "strategic.goal",
    "strategic.directive",
    "strategic.economy",
    "strategic.force",
    "strategic.relationship",
    "strategic.recovery",
];

/// The one owner of a strategic random draw: `mix_seed(rng_seed, day, hour,
/// faction_id, purpose)` as the card names it, expressed through the crate's
/// single mixing function rather than a second one written here.
///
/// [`crate::world::mix_seed`] takes `(seed, day, key, slot)`, so a strategic
/// draw is two applications of it: the faction under the hour, then the
/// purpose under the same hour. Composing is what keeps one mixer in the
/// crate; a five-argument copy of it would be the fork AGENTS.md section 0
/// forbids, and changing the existing signature would move every monster
/// `world.rs` has ever spawned.
///
/// No `rand`, no thread state, no host clock: the same five arguments give the
/// same number on every machine, forever.
pub fn strategic_draw(rng_seed: u64, day: u32, hour: u8, faction_id: &str, purpose: &str) -> u64 {
    let under_faction = mix_seed(rng_seed, day, faction_id, hour);
    mix_seed(under_faction, day, purpose, hour)
}

/// Every draw one faction makes in one hour, in [`PURPOSES`] order.
///
/// Pure: it reads nothing and writes nothing, so S5 can call it to ask what
/// this hour offered a faction without running an hour.
pub fn hour_draws(
    rng_seed: u64,
    day: u32,
    hour: u8,
    faction_id: &str,
) -> BTreeMap<&'static str, u64> {
    PURPOSES
        .iter()
        .map(|purpose| {
            (
                *purpose,
                strategic_draw(rng_seed, day, hour, faction_id, purpose),
            )
        })
        .collect()
}

/// One in-world hour. The whole of S4's simulation, and
/// [`ExpeditionState::strategic_tick`](crate::expedition::ExpeditionState::strategic_tick)
/// is a thin call into it.
///
/// The factions that act are the ones the save carries -- `state.factions`,
/// the mutable half `strategy/faction.rs` names as the only faction data a
/// tick may write. A faction that exists only as an authored record has
/// nothing to advance yet, and one whose record is not loaded still lives on
/// the island and still gets its hour. `BTreeMap` iteration, so the order is
/// the same on every machine.
///
/// Eliminated factions draw too. Brief section 16 keeps an eliminated faction
/// in the record because S10's recovery chain and the island's reaction still
/// read it; skipping its draws would make elimination shift every *other*
/// faction's numbers, which is exactly the kind of hidden coupling this fixed
/// sequence exists to prevent.
///
/// `geography` is the board S5 scores against. `factions` is the authored
/// registry, and B15 made it real: the bridge loads `content/factions/*.json`
/// and `resolve_midnight_in` passes it down, so `GoalWeights::from_definition`
/// scores the record where one is loaded and `GoalWeights::default()` -- read
/// the board and nothing else -- where none is. That is the same registry the
/// harness loads from the same files, which is what makes a game launched from
/// Godot and a game run in the test harness one island.
/// `strategic_determinism.rs::the_authored_registry_changes_the_tick` is the
/// guard, and the `the_registry_cannot_change_a_tick_yet` it replaced was
/// written to be deleted by this lane.
///
/// ## What S5 does with the hour's draws
///
/// The seven draws are made and folded into the witness exactly as before: the
/// loop below is untouched, so the digest of a given hour is the number it
/// always was and no save written since S4 has changed meaning.
///
/// Of the two draws [`PURPOSES`] reserves for S5, one is consumed and one is
/// deliberately not:
///
/// * `strategic.goal` is [`choose_goals`]' bounded personality variation, and
///   the only source of variation in the whole of S5.
/// * `strategic.board_position` is **read by nothing**, and that is the
///   decision rather than an omission. Card S5 requires `StrategicState` to be
///   "recomputed each tick from position, never set by hand", and a random
///   nudge -- even one bounded to a threshold's width -- would make the board
///   position partly a matter of chance instead of a fact about the board.
///   [`recompute_strategic_state`] is therefore a pure function of
///   [`BoardView`]. The purpose keeps its slot in the sequence because
///   removing it would change every island ever saved for no gain, and because
///   the position is the natural place for a future *perception* model -- a
///   faction misreading its own position -- to attach without moving anyone
///   else's numbers.
pub(crate) fn run_hour(
    state: &mut ExpeditionState,
    geography: &Geography,
    factions: &FactionDefinitions,
) -> Vec<StrategicEvent> {
    let day = state.campaign_day;
    let hour = state.strategic_clock.hour_of_day;
    let rng_seed = state.rng_seed;

    let faction_ids: Vec<String> = state.factions.keys().cloned().collect();
    for faction_id in &faction_ids {
        let draws = hour_draws(rng_seed, day, hour, faction_id);
        for draw in draws.values() {
            state.strategic_clock.absorb(*draw);
        }

        // Every faction reads the board as it stands at the top of the hour,
        // before any of them has decided anything, so the order the factions
        // are visited in cannot change what any of them sees. Nothing in S5
        // writes ownership, so the view is the same for all of them; when a
        // later lane does move the board mid-hour, this is the line that has
        // to decide whether it wants that, rather than discovering it.
        let view = BoardView::of(faction_id, state, geography);
        let strategic_state = recompute_strategic_state(&view);
        let goal_draw = draws
            .get("strategic.goal")
            .copied()
            .expect("PURPOSES reserves strategic.goal and hour_draws makes every purpose");
        // B15: the authored record decides how loudly this faction hears each
        // consideration; a faction with no loaded record reads the board and
        // nothing else. One registry reaches here now -- the bridge's, loaded
        // from `content/factions/`, and the harness's, loaded from the same
        // files -- so the engine's island and the harness's island are one.
        let weights = match factions.get(faction_id) {
            Some(record) => GoalWeights::from_definition(record),
            None => GoalWeights::default(),
        };
        let goals = choose_goals(&view, strategic_state, goal_draw, &weights);

        let faction = state
            .factions
            .get_mut(faction_id)
            .expect("faction_ids was taken from this map and nothing removes from it");
        faction.strategic_state = strategic_state;
        faction.current_goals = goals;
    }

    state.strategic_clock.advance_one_hour();
    vec![StrategicEvent::HourPassed { day, hour }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::building::BuildingDefinitions;
    use crate::strategy::faction::{ConceptKey, FactionState};

    fn a_campaign_with_two_factions() -> ExpeditionState {
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        state
            .factions
            .insert(ConceptKey::Pirates.faction_id(), FactionState::new());
        state
            .factions
            .insert(ConceptKey::Elves.faction_id(), FactionState::new());
        state
    }

    #[test]
    fn a_fresh_campaign_starts_at_the_first_hour_of_its_first_day() {
        let state = a_campaign_with_two_factions();
        assert_eq!(state.strategic_clock, StrategicClock::new());
        assert_eq!(state.strategic_clock.hour_of_day, 0);
        assert_eq!(state.strategic_clock.total_hours, 0);
        assert_eq!(state.strategic_clock.day_from_hours(), state.campaign_day);
    }

    #[test]
    fn an_hour_reports_the_hour_that_ran_and_then_the_clock_has_moved() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = FactionDefinitions::new();
        let mut state = a_campaign_with_two_factions();

        let events = state.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        assert_eq!(events, vec![StrategicEvent::HourPassed { day: 1, hour: 0 }]);
        assert_eq!(state.strategic_clock.hour_of_day, 1);
        assert_eq!(state.strategic_clock.total_hours, 1);

        let events = state.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        assert_eq!(events, vec![StrategicEvent::HourPassed { day: 1, hour: 1 }]);
        assert_eq!(state.strategic_clock.hour_of_day, 2);
    }

    /// The day wraps at twenty-four and nothing else moves: a strategic hour
    /// never advances the character-scale day by itself. Only
    /// `resolve_midnight_in` turns the day, and it does it once.
    #[test]
    fn twenty_four_hours_wrap_the_day_without_touching_campaign_day() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = FactionDefinitions::new();
        let mut state = a_campaign_with_two_factions();

        for _ in 0..HOURS_PER_DAY {
            state.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        }
        assert_eq!(state.strategic_clock.hour_of_day, 0);
        assert_eq!(state.strategic_clock.total_hours, 24);
        assert_eq!(state.campaign_day, 1, "an hour is not a midnight");
    }

    /// An hour with no factions in it is still an hour: the clock moves, the
    /// event is emitted, and the digest -- which nothing drew into -- does not.
    #[test]
    fn a_tick_with_nothing_to_do_is_still_a_tick() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = FactionDefinitions::new();
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        assert!(state.factions.is_empty());

        let events = state.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        assert_eq!(events, vec![StrategicEvent::HourPassed { day: 1, hour: 0 }]);
        assert_eq!(state.strategic_clock.total_hours, 1);
        assert_eq!(state.strategic_clock.draw_digest, 0);
    }

    /// The draw sequence S5 depends on: seven purposes, named and ordered, one
    /// distinct number each, and the same numbers on the next machine.
    #[test]
    fn the_seven_purposes_are_drawn_in_a_fixed_order_and_do_not_collide() {
        assert_eq!(
            PURPOSES,
            [
                "strategic.board_position",
                "strategic.goal",
                "strategic.directive",
                "strategic.economy",
                "strategic.force",
                "strategic.relationship",
                "strategic.recovery",
            ]
        );

        let draws = hour_draws(7, 3, 11, "faction.pirates");
        assert_eq!(draws.len(), PURPOSES.len());
        for purpose in PURPOSES {
            assert!(draws.contains_key(purpose), "{purpose} was not drawn");
        }
        let mut values: Vec<u64> = draws.values().copied().collect();
        values.sort_unstable();
        values.dedup();
        assert_eq!(
            values.len(),
            PURPOSES.len(),
            "two purposes drew the same number in the same hour"
        );

        assert_eq!(draws, hour_draws(7, 3, 11, "faction.pirates"));
    }

    /// Every argument of a draw actually separates it. If any of these
    /// collapsed, two factions -- or two hours, or two days -- would silently
    /// share a decision.
    #[test]
    fn every_argument_of_a_draw_separates_it() {
        let base = strategic_draw(7, 3, 11, "faction.pirates", "strategic.goal");
        assert_ne!(
            base,
            strategic_draw(8, 3, 11, "faction.pirates", "strategic.goal")
        );
        assert_ne!(
            base,
            strategic_draw(7, 4, 11, "faction.pirates", "strategic.goal")
        );
        assert_ne!(
            base,
            strategic_draw(7, 3, 12, "faction.pirates", "strategic.goal")
        );
        assert_ne!(
            base,
            strategic_draw(7, 3, 11, "faction.elves", "strategic.goal")
        );
        assert_ne!(
            base,
            strategic_draw(7, 3, 11, "faction.pirates", "strategic.force")
        );
    }

    /// The witness is order-sensitive: the same hour run for the same factions
    /// in a different order is a different island, and the digest says so.
    #[test]
    fn the_digest_records_the_order_the_draws_were_made_in() {
        let mut forwards = StrategicClock::new();
        forwards.absorb(11);
        forwards.absorb(22);
        let mut backwards = StrategicClock::new();
        backwards.absorb(22);
        backwards.absorb(11);
        assert_ne!(forwards.draw_digest, backwards.draw_digest);
        assert_eq!(StrategicClock::new().draw_digest, 0);
    }

    /// An eliminated faction keeps its hour, so removing one from the board
    /// never shifts another faction's numbers.
    #[test]
    fn an_eliminated_faction_still_draws() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = FactionDefinitions::new();

        let mut alive = a_campaign_with_two_factions();
        let mut with_one_eliminated = a_campaign_with_two_factions();
        with_one_eliminated
            .factions
            .get_mut(&ConceptKey::Elves.faction_id())
            .expect("the fixture carries this faction")
            .eliminated = true;

        alive.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        with_one_eliminated.strategic_tick(&geography, &definitions, &BuildingDefinitions::new());
        assert_eq!(
            alive.strategic_clock.draw_digest,
            with_one_eliminated.strategic_clock.draw_digest
        );
    }
}
