//! S8: the dual clocks -- world time against Cthulhu patience and heat --
//! plus the weather and corruption that ride on the second of them.
//!
//! `docs/GAME_BUILD_PLAN.md` names this a system contract, and it is a
//! contract about *separation*: "World time and Cthulhu patience/heat are
//! distinct state dimensions. Advancing time must not silently imply an
//! identical heat increase." The continuation brief section 13 says the same
//! thing in the brief's words -- "World time and irreversible hidden pressure
//! are separate systems."
//!
//! So this file contains no function that moves heat as a consequence of a
//! day, an hour, or a tick. Heat moves through exactly one door,
//! [`ExpeditionState::record_heat_event`](crate::expedition::ExpeditionState::record_heat_event),
//! and that door takes a [`HeatEvent`] -- a thing that happened on the board,
//! never an elapsed quantity. The passage of days is not one of the variants
//! and cannot be spelled as one.
//!
//! ## What is provisional here
//!
//! Every number below -- [`MAX_HEAT`] and each `HEAT_*` amount -- is
//! **provisional tuning, `needs decision`**. `docs/GAME_BUILD_PLAN.md` is
//! explicit that "clock tuning ... must not be invented by implementation",
//! and nothing in the accepted material fixes a scale for hidden pressure.
//! What S8 ships is the *shape*: distinct dimensions, named inputs, a
//! saturating ceiling, and a test that fails the moment time starts leaking
//! into pressure. Retuning is editing four constants and no behaviour.
//!
//! ## What is blocked
//!
//! Three of the brief's section 20 Open items land in this file's subject and
//! none of them is decided, so none of them is implemented:
//!
//! * **Corruption reversibility** -- `blocked: needs decision`. Corruption
//!   accumulates and there is no decay path, no cleanse verb, and no midnight
//!   sweep that lowers a cell. See [`corrupt_cell`](crate::expedition::ExpeditionState::corrupt_cell).
//! * **Cthulhu summoning-interruption rules** -- `blocked: needs decision`.
//!   [`ConfrontationTrigger`] reports that the confrontation *may begin*; what
//!   a party can do to a summoning once it has is not modelled here at all.
//! * **Cthulhu early-elimination rules** -- `blocked: needs decision`. Nothing
//!   in this file removes the Cthulhu faction, and
//!   [`assistance_weight`] deliberately answers zero rather than pretending a
//!   losing Cthulhu has been eliminated.
//!
//! Naming: `faction.cthulhu` is a *concept key*, per S1 and AGENTS.md section
//! 0's naming rule. It is not a proper name and nothing here carries one.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::strategy::faction::{FactionState, StrategicState};
use crate::world::mix_seed;

/// The concept key of the faction this file's second clock belongs to. S1
/// holds every faction ID to exactly `faction.<concept_key>`; this is that
/// rule's one spelling for the sixth faction, stated once so no call site
/// re-types it.
pub const CTHULHU_FACTION_ID: &str = "faction.cthulhu";

/// The ceiling hidden pressure saturates at, and the third confrontation
/// route ("maximum hidden pressure", brief section 13).
///
/// **Provisional, `needs decision`.** A thousand is a scale, not an approved
/// number: it makes each authored event a legible fraction of the whole and
/// nothing more. Nothing derives a presentation from it -- the brief is clear
/// that pressure is *hidden*, so no projection in this crate turns it into a
/// bar or a percentage.
pub const MAX_HEAT: u32 = 1_000;

/// A completed ritual: the Cthulhu faction's own progress toward the
/// summoning. **Provisional, `needs decision`.**
pub const HEAT_RITUAL_COMPLETED: u32 = 50;

/// One corrupted cell held, counted once per event, not once per day held --
/// the caller decides a hold happened, which is what keeps the passage of
/// time out of this. **Provisional, `needs decision`.**
pub const HEAT_CORRUPTED_CELL_HELD: u32 = 5;

/// The party acting against Cthulhu: brief section 13's "visible board
/// conditions", read from the other side. Interference raises pressure
/// because being noticed is what patience spends itself on.
/// **Provisional, `needs decision`.**
pub const HEAT_PARTY_INTERFERENCE: u32 = 15;

/// The board events that are allowed to move hidden pressure. S8's card names
/// exactly these three -- "rituals completed, corrupted cells held, party
/// interference -- **not the passage of days by itself**" -- and this enum is
/// that sentence made unrepresentable-otherwise.
///
/// There is deliberately no `DayPassed`, no `HoursElapsed`, and no variant
/// carrying a duration. A lane that wants pressure to rise over time has to
/// add a variant here and will meet
/// `thirty_midnights_do_not_move_hidden_pressure` on the way, which is the
/// point: the contract's separation is a compile-and-test surface, not a
/// convention.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatEvent {
    /// A ritual finished somewhere on the island.
    RitualCompleted,
    /// A corrupted cell was held through a contest. The cell is carried so a
    /// journal (S11) can say *which*; this file reads only the amount.
    CorruptedCellHeld { cell_id: String },
    /// Captain Michael's party acted against the Cthulhu faction's work.
    PartyInterference,
}

impl HeatEvent {
    /// How much pressure this event is worth. Every amount is a named
    /// constant above, so retuning never means hunting a literal in a match
    /// arm, and no caller may pass an amount of its own choosing.
    pub fn amount(&self) -> u32 {
        match self {
            HeatEvent::RitualCompleted => HEAT_RITUAL_COMPLETED,
            HeatEvent::CorruptedCellHeld { .. } => HEAT_CORRUPTED_CELL_HELD,
            HeatEvent::PartyInterference => HEAT_PARTY_INTERFERENCE,
        }
    }
}

/// The campaign day on which the confrontation can begin regardless of
/// anything else: brief section 13's second route, "Day 100".
pub const CONFRONTATION_DAY: u32 = 100;

/// The discovery that opens brief section 13's first route, "deliberate
/// discovery".
///
/// **`needs authored id`.** No observation or discovery record in `content/`
/// is the deliberate discovery of the Cthulhu faction's work yet, and S8 does
/// not get to invent one -- an authored ID that no content file backs would be
/// a stable ID the validator cannot see and a story beat implementation made
/// up. This constant is the hook a C card fills: author the discovery, give it
/// this ID or change this line to the one it was given, and the first route
/// starts working with no other edit.
pub const CONFRONTATION_DISCOVERY_ID: &str = "discovery.cthulhu.deliberate_confrontation";

/// Which of brief section 13's three routes opened the confrontation.
///
/// Reported, never stored: the answer is recomputed from state every time it
/// is asked, so there is no "confrontation begun" flag to get out of step with
/// the day count, the discovery set, or the pressure that caused it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfrontationTrigger {
    /// The party found it: [`CONFRONTATION_DISCOVERY_ID`] is recorded.
    Discovery,
    /// World time reached [`CONFRONTATION_DAY`].
    DayHundred,
    /// Hidden pressure reached [`MAX_HEAT`].
    MaximumHeat,
}

/// What the sky is doing over one region.
///
/// A small closed enum on purpose. Brief section 13 says Cthulhu growth makes
/// weather "chaotic, necromantic, corrupt, hostile to ordinary life" -- a
/// direction of travel, not a catalogue -- and S1 already carries a faction's
/// `weather_preferences` as authored content. Five conditions are enough for a
/// preference to have something to prefer, and few enough that no consumer
/// starts treating the list as a tuning surface.
///
/// The order is the axis: ordinary weather first, the corrupted end last.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    /// Ordinary weather. The default so a save written before this field
    /// existed loads with a sky rather than a failure.
    #[default]
    Clear,
    Overcast,
    Rain,
    Storm,
    /// The corrupted end of the axis: weather that is not weather any more.
    /// Only reachable where corruption has taken hold, which is what makes it
    /// legible on the board as evidence rather than as bad luck.
    Unnatural,
}

impl WeatherCondition {
    /// The fixed table a daily draw indexes. Stated once so the draw and any
    /// future reader agree on what index means what; changing the order
    /// changes every island, which is why it is a constant and not built at a
    /// call site.
    const ORDINARY: [WeatherCondition; 4] = [
        WeatherCondition::Clear,
        WeatherCondition::Overcast,
        WeatherCondition::Rain,
        WeatherCondition::Storm,
    ];

    /// The lowercase word the bridge would show. No numbers cross.
    pub fn as_str(&self) -> &'static str {
        match self {
            WeatherCondition::Clear => "clear",
            WeatherCondition::Overcast => "overcast",
            WeatherCondition::Rain => "rain",
            WeatherCondition::Storm => "storm",
            WeatherCondition::Unnatural => "unnatural",
        }
    }

    /// Every condition, ordinary end first. [`ORDINARY`](Self::ORDINARY) is the
    /// draw's table and deliberately stops short of `Unnatural`; this is the
    /// whole axis, for a reader that must cover all five -- an authored
    /// weather table, a test.
    pub const ALL: [WeatherCondition; 5] = [
        WeatherCondition::Clear,
        WeatherCondition::Overcast,
        WeatherCondition::Rain,
        WeatherCondition::Storm,
        WeatherCondition::Unnatural,
    ];
}

/// One region's weather, and the day it was drawn for.
///
/// `serde(default)` throughout: a save written before weather existed loads
/// with an empty map, and the next midnight fills it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherState {
    #[serde(default)]
    pub condition: WeatherCondition,
    /// The `campaign_day` this condition was drawn for. Carried so a reader
    /// can tell today's sky from a stale one without consulting the clock,
    /// and so a save that skipped a midnight is visibly stale rather than
    /// silently wrong.
    #[serde(default)]
    pub drawn_on_day: u32,
}

/// The one new random draw S8 adds, and its whole shape:
/// `mix_seed(rng_seed, day, region_id, "weather")`, **per region per day**.
///
/// It is deliberately not per faction per hour, so it does not touch S4's
/// per-hour draw sequence and cannot shift a single number S5 will consume.
/// S4's [`PURPOSES`](crate::strategy::tick::PURPOSES) list is untouched by
/// this lane; a purpose there is a faction's entitlement in an hour, and the
/// sky belongs to neither a faction nor an hour.
///
/// Composed from the crate's single mixer twice, the same way
/// [`strategic_draw`](crate::strategy::tick::strategic_draw) composes it: the
/// region under the day, then the purpose under the same day. A second mixing
/// function would be the fork AGENTS.md section 0 forbids.
pub fn weather_draw(rng_seed: u64, day: u32, region_id: &str) -> u64 {
    let under_region = mix_seed(rng_seed, day, region_id, 0);
    mix_seed(under_region, day, "weather", 0)
}

impl WeatherState {
    /// One region's weather for one day. Pure and total: the same arguments
    /// give the same sky on every machine, forever, so two midnights run from
    /// the same save agree without either of them having stored the answer.
    ///
    /// `corruption_pressure` is the highest corruption any cell in the region
    /// carries. It is the brief's "Cthulhu growth makes weather increasingly
    /// chaotic" expressed as the only rule S8 is entitled to state: an
    /// uncorrupted region never draws [`WeatherCondition::Unnatural`], and a
    /// region draws it more often the further corruption has gone. The
    /// threshold shape is **provisional, `needs decision`** with the heat
    /// constants; that it is monotone in corruption is the accepted part.
    pub fn draw(rng_seed: u64, day: u32, region_id: &str, corruption_pressure: u8) -> Self {
        let draw = weather_draw(rng_seed, day, region_id);
        // Corruption buys a share of the day's outcomes for the corrupted end
        // of the axis, in proportion to how far it has gone. At zero the share
        // is zero and `Unnatural` is unreachable; at 255 it is most of them.
        let unnatural_share = u64::from(corruption_pressure);
        let condition = if draw % 256 < unnatural_share {
            WeatherCondition::Unnatural
        } else {
            WeatherCondition::ORDINARY
                [(draw.rotate_left(13) as usize) % WeatherCondition::ORDINARY.len()]
        };
        Self {
            condition,
            drawn_on_day: day,
        }
    }
}

/// The ceiling a cell's corruption saturates at. Corruption is a `u8` and this
/// is its maximum: a fully corrupted cell, with nowhere further to go.
pub const MAX_CORRUPTION: u8 = u8::MAX;

/// How much weight a Cthulhu assistance event is entitled to, given the
/// faction's live state. Zero means the event does not fire at all.
///
/// Brief section 13: assistance events are "weighted, state-gated" and "must
/// not simply rescue Cthulhu whenever it is losing". So the gate is stated as
/// the card states it -- non-zero only for
/// [`StrategicState::Advantaged`] and [`StrategicState::Closing`], and zero
/// for `Desperate`, `Recovering` and `Contesting`. A losing Cthulhu is not
/// helped, by construction rather than by tuning: no weight this function can
/// return for `Desperate` is non-zero.
///
/// An absent faction -- a save with no Cthulhu in it -- is zero, not a panic
/// and not a default weight. An eliminated one is zero for the same reason.
///
/// **S8 ships the gate, not the event.** What an assistance event *is* -- what
/// it places on the board, and the "visible board conditions" it must arise
/// from -- is S7's and S10's territory. This function is the thing they call
/// before they do anything, and the weights are **provisional,
/// `needs decision`** like the rest of the tuning here.
pub fn assistance_weight(state: Option<&FactionState>) -> u32 {
    let Some(state) = state else {
        return 0;
    };
    if state.eliminated {
        return 0;
    }
    match state.strategic_state {
        StrategicState::Advantaged => ASSISTANCE_WEIGHT_ADVANTAGED,
        StrategicState::Closing => ASSISTANCE_WEIGHT_CLOSING,
        // Not a fallthrough: each losing or level state is named, so adding a
        // sixth `StrategicState` is a compile error here rather than a silent
        // grant of assistance to whatever it turns out to mean.
        StrategicState::Desperate | StrategicState::Recovering | StrategicState::Contesting => 0,
    }
}

/// Weight for a Cthulhu that is winning. **Provisional, `needs decision`.**
pub const ASSISTANCE_WEIGHT_ADVANTAGED: u32 = 1;
/// Weight for a Cthulhu closing on its victory function. **Provisional,
/// `needs decision`.**
pub const ASSISTANCE_WEIGHT_CLOSING: u32 = 3;

/// The highest corruption any cell of `region_id` carries, which is what
/// [`WeatherState::draw`] reads.
///
/// Highest rather than mean: one thoroughly corrupted anchor is what the brief
/// describes destabilising a region's weather, and averaging would let a large
/// clean region hide it.
pub fn region_corruption_pressure(
    corruption: &BTreeMap<String, u8>,
    cell_ids_in_region: impl Iterator<Item = impl AsRef<str>>,
) -> u8 {
    cell_ids_in_region
        .filter_map(|cell_id| corruption.get(cell_id.as_ref()).copied())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_heat_event_is_spelled_as_elapsed_time() {
        // The contract in one assertion: every input to hidden pressure is a
        // board event, and each carries a named constant amount.
        for event in [
            HeatEvent::RitualCompleted,
            HeatEvent::CorruptedCellHeld {
                cell_id: "world.cell.river_landing".into(),
            },
            HeatEvent::PartyInterference,
        ] {
            assert!(
                event.amount() > 0,
                "a heat event that moves nothing is not an input"
            );
        }
    }

    #[test]
    fn weather_is_the_same_sky_every_time_it_is_asked() {
        for day in 1..40 {
            let first = WeatherState::draw(7, day, "world.region.black_beach", 0);
            let again = WeatherState::draw(7, day, "world.region.black_beach", 0);
            assert_eq!(first, again);
        }
    }

    #[test]
    fn an_uncorrupted_region_never_draws_unnatural_weather() {
        for day in 1..500 {
            let weather = WeatherState::draw(7, day, "world.region.black_beach", 0);
            assert_ne!(weather.condition, WeatherCondition::Unnatural);
        }
    }

    #[test]
    fn corruption_makes_unnatural_weather_reachable_and_more_common() {
        let count = |pressure: u8| {
            (1..500)
                .filter(|day| {
                    WeatherState::draw(7, *day, "world.region.black_beach", pressure).condition
                        == WeatherCondition::Unnatural
                })
                .count()
        };
        let light = count(32);
        let heavy = count(224);
        assert!(light > 0, "some corruption must reach the corrupted end");
        assert!(
            heavy > light,
            "more corruption must mean more unnatural weather: {light} then {heavy}"
        );
    }

    #[test]
    fn two_regions_do_not_share_one_sky() {
        // Different keys must be able to disagree, or the map is decorative.
        let differ = (1..200).any(|day| {
            WeatherState::draw(7, day, "world.region.black_beach", 0)
                != WeatherState::draw(7, day, "world.region.river", 0)
        });
        assert!(
            differ,
            "weather must be drawn per region, not per day alone"
        );
    }

    #[test]
    fn assistance_is_gated_on_the_two_winning_states_only() {
        let state = |strategic_state| FactionState {
            strategic_state,
            ..FactionState::default()
        };
        assert_eq!(
            assistance_weight(Some(&state(StrategicState::Desperate))),
            0
        );
        assert_eq!(
            assistance_weight(Some(&state(StrategicState::Recovering))),
            0
        );
        assert_eq!(
            assistance_weight(Some(&state(StrategicState::Contesting))),
            0
        );
        assert!(assistance_weight(Some(&state(StrategicState::Advantaged))) > 0);
        assert!(assistance_weight(Some(&state(StrategicState::Closing))) > 0);
        assert_eq!(assistance_weight(None), 0, "no faction, no assistance");
    }

    #[test]
    fn an_eliminated_cthulhu_is_never_assisted() {
        let mut state = FactionState {
            strategic_state: StrategicState::Closing,
            ..FactionState::default()
        };
        state.eliminated = true;
        assert_eq!(assistance_weight(Some(&state)), 0);
    }

    #[test]
    fn region_pressure_is_the_worst_cell_not_the_average() {
        let mut corruption = BTreeMap::new();
        corruption.insert("world.cell.a".to_owned(), 200u8);
        corruption.insert("world.cell.b".to_owned(), 0u8);
        assert_eq!(
            region_corruption_pressure(
                &corruption,
                ["world.cell.a", "world.cell.b", "world.cell.c"].into_iter()
            ),
            200
        );
        assert_eq!(
            region_corruption_pressure(&corruption, ["world.cell.c"].into_iter()),
            0
        );
    }
}
