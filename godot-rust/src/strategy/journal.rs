//! S11: the strategic event journal -- what the island did, kept in the save.
//! `docs/SHIP_PLAN.md` section 7, card S11.
//!
//! Brief section 17 lists "strategic event history" among the things a save
//! must preserve. Preserving it literally would mean an append-only `Vec` that
//! grows for as long as a campaign runs: twenty-four entries a day from the
//! midnight hours alone, before any faction acts at all. A hundred-day
//! campaign is already 2,400 entries; a long one is unbounded, and an
//! unbounded save is a save that eventually fails to write.
//!
//! So the journal is **bounded but lossless**, in the sense that matters:
//!
//! * [`JOURNAL_WINDOW`] entries are kept whole and readable, the recent past
//!   anything on screen or in a test wants to quote;
//! * everything older is folded, in the order it happened, into a single
//!   `u64` [`StrategicJournal::history_digest`];
//! * [`StrategicJournal::total_recorded`] counts every entry ever pushed.
//!
//! Window plus digest plus count *is* the complete record. Nothing that
//! happened can vanish from the save without changing one of the three, which
//! is the only property a determinism harness can actually check: two
//! campaigns whose histories diverged before the window still have different
//! saves. It is the same trick and the same shape as
//! [`StrategicClock::draw_digest`](crate::strategy::tick::StrategicClock),
//! which S4 uses to keep the evidence of every draw ever made in eight bytes;
//! this file applies it to the events rather than to the draws.
//!
//! ## What an entry may carry
//!
//! A [`JournalEntry`] is a [`StrategicEvent`] plus the day and hour it
//! happened on, and nothing else. Not a utility score (S5 keeps those
//! private), not a disposition or a recruitment stage (S12 keeps those
//! private): a journal that quietly carried them would be a side channel
//! around the projections those lanes exist to enforce, and the fact that it
//! is written to a save the player can open makes it a worse one than most.
//! If an event should be visible, it belongs in `StrategicEvent`, where the
//! lane that owns the fact decides how it reads.
//!
//! ## The window size is provisional
//!
//! [`JOURNAL_WINDOW`] is 256 -- roughly ten in-world days of hour events,
//! enough that the recent past covers more than the day on screen, small
//! enough that the entries add kilobytes rather than megabytes to a save.
//! **The number is provisional.** Nothing derives it and no test asserts a
//! particular value; changing it changes how much detail a save keeps and
//! nothing else about what the island does, because the digest absorbs
//! whatever the window drops.

use serde::{Deserialize, Serialize};

use crate::strategy::tick::StrategicEvent;

/// How many entries the journal keeps whole. **Provisional**: see the module
/// docs. Roughly ten in-world days at twenty-four hour events a day.
pub const JOURNAL_WINDOW: usize = 256;

/// The FNV-1a 64-bit prime, as [`StrategicClock`](crate::strategy::tick::StrategicClock)
/// uses it. Named once here rather than typed twice.
const FNV_PRIME: u64 = 0x100_0000_01B3;

/// One thing the island did, and when.
///
/// The event is the fact; `day` and `hour` are the stamp, stored beside it
/// rather than dug out of the variant so that every entry is comparable
/// whatever S5 and S6 add to the enum.
///
/// `day` and `hour` are `serde(default)`. `event` deliberately is not: an
/// entry with no event is not an event that defaults, it is a lie about the
/// history, and a save that has lost an entry's event should fail to load
/// rather than load an invented one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// The campaign day the event happened on.
    #[serde(default)]
    pub day: u32,
    /// The hour of that day, `0..HOURS_PER_DAY`.
    #[serde(default)]
    pub hour: u8,
    /// What happened.
    pub event: StrategicEvent,
}

impl JournalEntry {
    /// An entry for `event`, stamped with the day and hour it happened on.
    pub fn new(day: u32, hour: u8, event: StrategicEvent) -> Self {
        Self { day, hour, event }
    }
}

/// The saved strategic event history: a bounded window, a digest of everything
/// older, and a count of everything ever recorded.
///
/// The three fields are private and there is exactly one mutator,
/// [`StrategicJournal::push`]. That is not tidiness: the invariant "the digest
/// covers precisely the entries the window no longer holds" is only true if
/// nothing else can drop an entry, add one, or touch the digest. A second way
/// in would be a way to lose history silently.
///
/// Every field is `serde(default)`, so a save written before the journal
/// existed loads with an empty one -- no entries, a zero digest, nothing
/// recorded -- which is the honest reading of a save that never kept a
/// journal.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategicJournal {
    /// The most recent [`JOURNAL_WINDOW`] entries, oldest first.
    #[serde(default)]
    entries: Vec<JournalEntry>,
    /// An order-sensitive fold of every entry the window has dropped.
    #[serde(default)]
    history_digest: u64,
    /// How many entries have been folded into `history_digest`.
    #[serde(default)]
    evicted_count: u64,
}

impl StrategicJournal {
    /// An empty journal: nothing recorded, nothing evicted, digest zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one entry.
    ///
    /// The only mutator. When the window overflows, the oldest entry is folded
    /// into the digest and the evicted count rises, so the entry leaves the
    /// window without leaving the record.
    ///
    /// The loop rather than a single eviction is deliberate: a save written
    /// when [`JOURNAL_WINDOW`] was larger loads with a longer window, and the
    /// first pushes afterwards fold the excess in one entry at a time, in
    /// order, rather than leaving a journal permanently over its bound.
    pub fn push(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
        while self.entries.len() > JOURNAL_WINDOW {
            let evicted = self.entries.remove(0);
            self.absorb(&evicted);
            self.evicted_count = self.evicted_count.saturating_add(1);
        }
    }

    /// The window: the entries still kept whole, oldest first.
    pub fn recent(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// The fold of every entry the window has dropped, in the order it dropped
    /// them. Zero exactly while nothing has been evicted.
    pub fn history_digest(&self) -> u64 {
        self.history_digest
    }

    /// Every entry ever pushed: the window plus everything folded away.
    pub fn total_recorded(&self) -> u64 {
        self.evicted_count.saturating_add(self.entries.len() as u64)
    }

    /// How many entries have left the window.
    pub fn evicted_count(&self) -> u64 {
        self.evicted_count
    }

    /// Fold one evicted entry into the digest: FNV-1a over its serialized
    /// bytes, then a rotate to close the entry.
    ///
    /// Byte-wise FNV-1a makes the fold sensitive to *what* was evicted; the
    /// closing rotate makes it sensitive to *where one entry ends and the next
    /// begins*, so two different sequences of entries cannot fold to the same
    /// number merely by sharing a concatenation. Order-sensitive throughout:
    /// the same events in a different order are a different history, exactly
    /// as the same draws in a different order are a different island.
    ///
    /// Serialization is the crate's own `serde_json`, so the fold sees the
    /// same bytes the save would have carried had the entry stayed. No hashing
    /// crate: `std`'s `DefaultHasher` is explicitly not stable across
    /// releases, and a determinism witness that changes with the toolchain is
    /// not a witness.
    fn absorb(&mut self, entry: &JournalEntry) {
        let bytes = serde_json::to_vec(entry).expect("a JournalEntry always serializes");
        let mut digest = self.history_digest;
        for byte in bytes {
            digest = (digest ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
        }
        self.history_digest = digest.rotate_left(7);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expedition::ExpeditionState;

    /// The nth hour event of a campaign, stamped consistently, so a test can
    /// say "feed it `n` entries" and mean a history that is distinguishable
    /// entry by entry.
    fn hour(n: u64) -> JournalEntry {
        let day = (n / 24) as u32 + 1;
        let hour = (n % 24) as u8;
        JournalEntry::new(day, hour, StrategicEvent::HourPassed { day, hour })
    }

    fn journal_of(count: u64) -> StrategicJournal {
        let mut journal = StrategicJournal::new();
        for n in 0..count {
            journal.push(hour(n));
        }
        journal
    }

    #[test]
    fn a_new_journal_records_nothing() {
        let journal = StrategicJournal::new();
        assert!(journal.recent().is_empty());
        assert_eq!(journal.history_digest(), 0);
        assert_eq!(journal.total_recorded(), 0);
    }

    #[test]
    fn under_the_window_every_entry_is_kept_whole_and_nothing_is_folded() {
        let journal = journal_of(JOURNAL_WINDOW as u64);
        assert_eq!(journal.recent().len(), JOURNAL_WINDOW);
        assert_eq!(journal.recent()[0], hour(0));
        assert_eq!(journal.history_digest(), 0, "nothing has been evicted yet");
        assert_eq!(journal.total_recorded(), JOURNAL_WINDOW as u64);
        assert_eq!(journal.evicted_count(), 0);
    }

    /// The bound is the point: a campaign that runs forever does not write a
    /// save that grows forever.
    #[test]
    fn the_window_never_grows_past_its_bound_however_long_the_campaign_runs() {
        let journal = journal_of(2_400);
        assert_eq!(journal.recent().len(), JOURNAL_WINDOW);
        assert_eq!(journal.total_recorded(), 2_400);
        assert_eq!(journal.evicted_count(), 2_400 - JOURNAL_WINDOW as u64);
        assert_eq!(
            journal.recent()[0],
            hour(2_400 - JOURNAL_WINDOW as u64),
            "the window holds the newest entries, oldest first"
        );
        assert_eq!(journal.recent()[JOURNAL_WINDOW - 1], hour(2_399));
    }

    /// Nothing is lost: the digest is exactly the fold of the entries the
    /// window dropped, recomputed here from the prefix rather than trusted.
    #[test]
    fn the_digest_is_exactly_the_evicted_prefix() {
        let overflow = 37;
        let journal = journal_of(JOURNAL_WINDOW as u64 + overflow);

        let mut recomputed = StrategicJournal::new();
        for n in 0..overflow {
            recomputed.absorb(&hour(n));
        }

        assert_eq!(journal.history_digest(), recomputed.history_digest);
        assert_ne!(journal.history_digest(), 0);
        assert_eq!(journal.evicted_count(), overflow);
    }

    /// **The test the "drop it without folding it" sabotage fails.** Two
    /// campaigns whose windows are identical but whose older history differs
    /// must not have the same save. Without the fold both digests are zero and
    /// the two histories are indistinguishable -- which is precisely the loss
    /// the digest exists to prevent.
    #[test]
    fn two_histories_that_differ_only_before_the_window_are_different_saves() {
        let mut kept = StrategicJournal::new();
        let mut lost = StrategicJournal::new();
        kept.push(hour(0));
        lost.push(JournalEntry::new(
            9,
            9,
            StrategicEvent::HourPassed { day: 9, hour: 9 },
        ));
        for n in 1..=JOURNAL_WINDOW as u64 {
            kept.push(hour(n));
            lost.push(hour(n));
        }

        assert_eq!(
            kept.recent(),
            lost.recent(),
            "the windows are identical: only the evicted entry differed"
        );
        assert_eq!(kept.evicted_count(), 1);
        assert_eq!(lost.evicted_count(), 1);
        assert_ne!(
            kept.history_digest(),
            lost.history_digest(),
            "an evicted entry must still change the save"
        );
    }

    /// Order-sensitive, like every other digest in the strategic layer: the
    /// same events in a different order are a different history.
    #[test]
    fn reordering_the_evicted_prefix_changes_the_digest() {
        let mut forwards = StrategicJournal::new();
        let mut backwards = StrategicJournal::new();
        for n in 0..3 {
            forwards.absorb(&hour(n));
        }
        for n in (0..3).rev() {
            backwards.absorb(&hour(n));
        }
        assert_ne!(forwards.history_digest, backwards.history_digest);
    }

    /// Eviction is deterministic: the same entries in the same order produce
    /// the same journal, byte for byte, which is what lets the journal sit
    /// under the 2,400-hour save hash.
    #[test]
    fn the_same_history_twice_produces_the_same_journal() {
        let once = journal_of(1_000);
        let twice = journal_of(1_000);
        assert_eq!(once, twice);
        assert_eq!(
            serde_json::to_string(&once).expect("serializes"),
            serde_json::to_string(&twice).expect("serializes")
        );
    }

    #[test]
    fn a_full_window_and_a_non_zero_digest_round_trip_byte_identically() {
        let journal = journal_of(1_000);
        let json = serde_json::to_string(&journal).expect("serializes");
        let reloaded: StrategicJournal = serde_json::from_str(&json).expect("loads");
        assert_eq!(reloaded, journal);
        assert_eq!(
            serde_json::to_string(&reloaded).expect("serializes"),
            json,
            "a reloaded journal writes the same bytes"
        );
        assert_ne!(reloaded.history_digest(), 0);
        assert_eq!(reloaded.recent().len(), JOURNAL_WINDOW);
        assert_eq!(reloaded.total_recorded(), 1_000);
    }

    /// The card's Done-when, at the save boundary rather than in isolation: a
    /// strategic save carrying a full window and a non-zero digest reloads
    /// into the same bytes.
    #[test]
    fn a_strategic_save_with_a_full_journal_round_trips_byte_identically() {
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("fresh campaign constructs");
        for n in 0..1_000 {
            state.strategic_journal.push(hour(n));
        }
        assert_ne!(state.strategic_journal.history_digest(), 0);
        assert_eq!(state.strategic_journal.recent().len(), JOURNAL_WINDOW);

        let json = state.to_json();
        let reloaded = ExpeditionState::from_json(&json).expect("a strategic save reloads");
        assert_eq!(reloaded.strategic_journal, state.strategic_journal);
        assert_eq!(reloaded.to_json(), json);
    }

    /// The card's other Done-when: E6's generated v1 fixture predates this
    /// field entirely, and must still load -- with an empty journal, not a
    /// failure and not an invented history.
    #[test]
    fn the_v1_fixture_that_predates_the_journal_still_loads_with_an_empty_one() {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/saves/v1_current.json"
        ));
        assert!(
            !json.contains("strategic_journal"),
            "the fixture must predate the field for this test to prove anything"
        );
        let state = ExpeditionState::from_json(json).expect("a save without a journal still loads");
        assert_eq!(state.strategic_journal, StrategicJournal::new());
        assert!(state.strategic_journal.recent().is_empty());
        assert_eq!(state.strategic_journal.history_digest(), 0);
        assert_eq!(state.strategic_journal.total_recorded(), 0);
    }
}
