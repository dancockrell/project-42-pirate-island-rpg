//! S5: what a faction can see of the board, where that puts it, and what it
//! therefore wants. `docs/SHIP_PLAN.md` section 7, card S5; brief section 9.
//!
//! This is the first thing a faction *decides*. S4 fixed the draw sequence and
//! decided nothing with it; this file turns the board into a
//! [`StrategicState`] and a short list of [`Goal`]s, once per faction per
//! strategic hour, from `run_hour`.
//!
//! **Three rules the whole file is built around.**
//!
//! *The board position is computed, never set.* [`recompute_strategic_state`]
//! is a pure function of a [`BoardView`], and a `BoardView` is a pure function
//! of `ExpeditionState` plus `Geography`. No draw enters it. Nothing anywhere
//! in the crate assigns `FactionState::strategic_state` by hand; the hour
//! recomputes it. That is what card S5's "recomputed each tick from position,
//! never set by hand" means, and it is why a random nudge -- however small --
//! has no place in it. See `run_hour`'s note on the unread
//! `strategic.board_position` draw.
//!
//! *Raw utility arithmetic does not leave this module.* Brief section 9: "Do
//! not expose raw utility arithmetic as the normal interface." Every score
//! here is a private `i32` inside a private function. No score type is `pub`,
//! no score reaches a serialized struct, and no score reaches a
//! [`StrategicEvent`](crate::strategy::tick::StrategicEvent). What leaves is
//! the *verdict*: a state and an ordered list of abstract goals.
//! `a_faction_state_serializes_no_score` holds that true.
//!
//! *Goals are abstract verbs, not doctrine and not directives.* [`Goal`] is
//! deliberately six plain words. It is **not** S6's `StrategicDirective`
//! vocabulary -- S6 turns a goal into an intent with a target, and a second
//! copy of its ten intents here would be the fork `AGENTS.md` section 0
//! forbids. It is also not doctrine: nothing in this file reads
//! [`ConceptKey`](crate::strategy::faction::ConceptKey) or `doctrine`, so no
//! faction gets a special strategy for being the faction it is. Brief sections
//! 6.1-6.4 stay Provisional until S15.
//!
//! **Every number in this file is provisional.** The brief settles the
//! *considerations* and the *five states*; it settles no thresholds and no
//! weights. The constants below are first numbers chosen to make the bands
//! occupied and the Done-when legible, each with the reasoning that produced
//! it written beside it. They are meant to be tuned against a real island, and
//! they are all named so that tuning is an edit to one constant rather than a
//! hunt through arithmetic.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::ExpeditionState;
use crate::geography::Geography;
use crate::strategy::directive::{DirectiveStatus, Intent, StrategicDirective};
use crate::strategy::faction::{FactionDefinition, Relationship, StrategicState};

// ---------------------------------------------------------------------------
// The board, as one faction can see it
// ---------------------------------------------------------------------------

/// The facts one faction scores from, and nothing else.
///
/// Computed from [`ExpeditionState`] and [`Geography`] by [`BoardView::of`],
/// which reads no draw, no clock and no authored record. That purity is the
/// point: two identical boards produce two identical views on every machine,
/// so everything downstream of a view is reproducible by construction, and the
/// determinism harness can hash the result.
///
/// Control is the *effective* controller throughout -- S2's
/// [`ExpeditionState::ownership`] wherever it names a cell, falling back to
/// [`Geography::controller`]'s authored answer. That is the same precedence
/// `Geography::effective_risk` uses, deliberately: "who holds this cell" has
/// one answer in this crate, and it is not re-derived here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoardView {
    /// `faction.<concept_key>`. Whose view this is.
    pub faction_id: String,
    /// Brief section 16 keeps an eliminated faction in the record. It still
    /// gets a view, and [`recompute_strategic_state`] still has an answer for
    /// it, because S10's recovery chain will read both.
    pub eliminated: bool,
    /// Every cell the world graph has. The denominator for "how much of the
    /// island is mine" -- a share, not a count, so a bigger island does not
    /// silently make every faction poorer.
    pub cells_on_the_board: usize,
    /// The cells this faction effectively controls.
    pub held: BTreeSet<String>,
    /// Cells adjacent to a holding of this faction that *somebody else* holds,
    /// mapped to the faction holding them. This is the frontier that pushes
    /// back: its size is territorial pressure, and its holders are who the
    /// pressure is with.
    pub contested_frontier: BTreeMap<String, String>,
    /// Cells adjacent to a holding that nobody holds. The frontier that does
    /// not push back -- room to grow, which is what makes expansion an
    /// opportunity rather than a fight.
    pub open_frontier: BTreeSet<String>,
    /// How many cells each faction with any control on the board holds,
    /// including this one. Empty means nobody has claimed anything yet.
    pub holdings_by_faction: BTreeMap<String, usize>,
    /// This faction's strategic stockpile, copied as-is. Open-keyed (brief
    /// section 20): the scoring sums over whatever keys exist and names none.
    pub resources: BTreeMap<String, u32>,
    /// What this faction holds about every *other* faction the save carries,
    /// zero-filled where no history has formed. S6 moves these; S5 reads them.
    pub relationships: BTreeMap<String, Relationship>,
    /// S6: the standing directives the player has given *this* faction, keyed
    /// by directive ID. Only [`DirectiveStatus::Active`] ones are here -- a
    /// directive that has been completed, cancelled, superseded, made
    /// impossible or returned stays in the save as history and asks for
    /// nothing.
    ///
    /// This is the one input to scoring that does not come from the board, and
    /// it is still a fact rather than a command: the whole of its effect is the
    /// [`GoalWeights::player_directives`] term in [`score`], which
    /// [`INTENT_GOAL_TABLE`] turns into a signal per goal.
    pub directives: BTreeMap<String, StrategicDirective>,
}

impl BoardView {
    /// Read the board as `faction_id` sees it.
    ///
    /// Adjacency is "a route connects the two cells", taken in both directions.
    /// A one-way portal still makes its far end a neighbour of its near end for
    /// this purpose: the question here is where a holding *touches* other
    /// ground, not where a party may walk, and treating a one-way road as a
    /// non-frontier would hide exactly the border a faction most needs to see.
    /// Route legality (`required_discovery_id`) is a party-scale gate and is
    /// deliberately not consulted; a faction's border does not depend on what
    /// Captain Michael has discovered.
    pub fn of(faction_id: &str, state: &ExpeditionState, geography: &Geography) -> Self {
        let controller = |cell_id: &str| -> Option<&str> {
            state
                .ownership
                .get(cell_id)
                .map(String::as_str)
                .or_else(|| geography.controller(cell_id))
        };

        let mut holdings_by_faction: BTreeMap<String, usize> = BTreeMap::new();
        let mut held: BTreeSet<String> = BTreeSet::new();
        for cell_id in geography.all_location_ids() {
            if let Some(holder) = controller(cell_id) {
                *holdings_by_faction.entry(holder.to_owned()).or_insert(0) += 1;
                if holder == faction_id {
                    held.insert(cell_id.to_owned());
                }
            }
        }

        let mut contested_frontier: BTreeMap<String, String> = BTreeMap::new();
        let mut open_frontier: BTreeSet<String> = BTreeSet::new();
        for route in geography.routes.values() {
            for (near, far) in [
                (&route.from_location_id, &route.to_location_id),
                (&route.to_location_id, &route.from_location_id),
            ] {
                if !held.contains(near) || held.contains(far) {
                    continue;
                }
                match controller(far) {
                    Some(holder) => {
                        contested_frontier.insert(far.clone(), holder.to_owned());
                    }
                    None => {
                        open_frontier.insert(far.clone());
                    }
                }
            }
        }

        let faction = state.factions.get(faction_id);
        let relationships = state
            .factions
            .keys()
            .filter(|other| other.as_str() != faction_id)
            .map(|other| {
                let held_view = faction
                    .map(|f| f.relationship_toward(other))
                    .unwrap_or_default();
                (other.clone(), held_view)
            })
            .collect();

        Self {
            faction_id: faction_id.to_owned(),
            eliminated: faction.is_some_and(|f| f.eliminated),
            cells_on_the_board: geography.locations.len(),
            held,
            contested_frontier,
            open_frontier,
            holdings_by_faction,
            resources: faction.map(|f| f.resources.clone()).unwrap_or_default(),
            relationships,
            directives: state
                .directives
                .iter()
                .filter(|(_, directive)| {
                    directive.state == DirectiveStatus::Active && directive.faction_id == faction_id
                })
                .map(|(id, directive)| (id.clone(), directive.clone()))
                .collect(),
        }
    }

    /// What share of the island this faction holds, in percent. Zero when the
    /// graph is empty, because a faction cannot hold a share of nothing.
    fn share_percent(&self) -> i32 {
        if self.cells_on_the_board == 0 {
            return 0;
        }
        (self.held.len() * 100 / self.cells_on_the_board) as i32
    }

    /// How surrounded the holding is: rival-held neighbours per cell held, in
    /// percent, capped at [`SIGNAL_CEILING`]. Per *cell held* rather than in
    /// absolute cells so that a large empire and a small one are compared on
    /// the same axis -- one contested border cell is an emergency for a
    /// one-cell faction and a rounding error for a twenty-cell one.
    fn pressure_percent(&self) -> i32 {
        per_holding_percent(self.contested_frontier.len(), self.held.len())
    }

    /// Unheld neighbours per cell held, in percent, capped the same way. Room.
    fn room_percent(&self) -> i32 {
        per_holding_percent(self.open_frontier.len(), self.held.len())
    }

    /// Everything stockpiled, over every open resource key, as a percent of
    /// [`WELL_STOCKED`]. Summing across categories rather than weighting them
    /// is the only honest thing S5 can do this round: the weighting lives in
    /// `FactionDefinition::resource_priorities`, and this round does not read
    /// the registry. [`GoalWeights::from_definition`] is where that arrives.
    fn stock_percent(&self) -> i32 {
        let total: u64 = self
            .resources
            .values()
            .map(|amount| u64::from(*amount))
            .sum();
        let scaled = total.saturating_mul(100) / u64::from(WELL_STOCKED);
        scaled.min(SIGNAL_CEILING as u64) as i32
    }

    /// The mean of the four relationship axes that read as "they may come for
    /// me", across every other faction present, clamped to a signal.
    ///
    /// Fear, hatred, territorial conflict and recent aggression, unweighted.
    /// Four axes rather than one because brief section 7 keeps them
    /// independent, and averaging them is the least presumptuous way to reduce
    /// four independent pressures to one number until S6 has moved them enough
    /// for a shape to be visible.
    fn hostility_percent(&self) -> i32 {
        self.mean_over_relationships(|r| {
            (i32::from(r.fear)
                + i32::from(r.hatred)
                + i32::from(r.territorial_conflict)
                + i32::from(r.recent_aggression))
                / 4
        })
    }

    /// The mean of `perceived_opportunity` across every other faction present:
    /// how much of an opening the neighbours look like.
    fn invitation_percent(&self) -> i32 {
        self.mean_over_relationships(|r| i32::from(r.perceived_opportunity))
    }

    fn mean_over_relationships(&self, axis: impl Fn(&Relationship) -> i32) -> i32 {
        if self.relationships.is_empty() {
            return 0;
        }
        let total: i32 = self.relationships.values().map(axis).sum();
        clamp_signal(total / self.relationships.len() as i32)
    }
}

/// A count measured against a holding, in percent, capped. Shared by
/// [`BoardView::pressure_percent`] and [`BoardView::room_percent`] so the two
/// frontiers are read on one scale.
fn per_holding_percent(count: usize, held: usize) -> i32 {
    if held == 0 {
        // Nothing held: every frontier is undefined rather than infinite. The
        // board-position term already knows this faction holds nothing.
        return 0;
    }
    ((count * 100 / held) as i32).min(SIGNAL_CEILING)
}

/// Every signal in this module is a percent in `0..=SIGNAL_CEILING`, so that
/// weights are comparable across considerations and a single runaway input
/// cannot swamp the rest.
const SIGNAL_CEILING: i32 = 100;

/// The stockpile, summed over every open resource key, at which a faction
/// counts as fully supplied. Provisional and frankly arbitrary: no content
/// authors resource amounts yet (brief section 20 leaves the category list
/// Open), so this is a placeholder scale that makes `stock_percent` occupy its
/// range. C9's records and S3's economy are what will set it honestly.
const WELL_STOCKED: u32 = 100;

fn clamp_signal(value: i32) -> i32 {
    value.clamp(-SIGNAL_CEILING, SIGNAL_CEILING)
}

// ---------------------------------------------------------------------------
// The five strategic states
// ---------------------------------------------------------------------------

/// The share of the island, in percent, at or above which a faction has
/// stopped being [`StrategicState::Desperate`].
///
/// Five percent: on the vertical slice's five-cell graph that is one cell, and
/// "holds one cell" is exactly the line between a faction that is still on the
/// board and one that is clinging to it. Provisional.
pub const RECOVERING_FLOOR_PERCENT: i32 = 5;
/// At or above this share a faction is genuinely in the game rather than
/// climbing back into it. Fifteen percent is a sixth of the island: enough to
/// be one of several powers, not enough to be a leading one. Provisional.
pub const CONTESTING_FLOOR_PERCENT: i32 = 15;
/// At or above this share a faction is ahead. Thirty-five percent is more than
/// an even split among three powers, which is the smallest number of powers a
/// six-faction island is likely to be down to when anyone is clearly winning.
/// Provisional.
pub const ADVANTAGED_FLOOR_PERCENT: i32 = 35;
/// At or above this share a faction is closing the game out: an outright
/// majority of the island, sixty percent, so the band cannot be entered by a
/// plurality among near-equals. Provisional.
pub const CLOSING_FLOOR_PERCENT: i32 = 60;

/// Where a faction stands, recomputed from the board and nothing else.
///
/// Pure. Takes no draw and no clock; the same view always gives the same
/// state. Card S5: never set by hand.
///
/// Three rules, in order:
///
/// 1. **An unclaimed board is [`StrategicState::Contesting`] for everyone.**
///    Before anybody holds anything -- which is every fresh campaign, since
///    content authors no ownership -- nothing has been won or lost, and
///    calling six factions Desperate on turn one would say something false
///    about the island and send all six chasing recovery goals. This is also
///    why `StrategicState::default()` is `Contesting`, and the two agree on
///    purpose.
/// 2. **An eliminated faction is [`StrategicState::Desperate`].** Brief
///    section 16 keeps it in the record; it is not "contesting" anything.
///    S10's recovery chain reads this.
/// 3. Otherwise the share of the island gives a band, and being surrounded
///    demotes it by one.
///
/// The demotion is the whole reason board position is not simply a share: a
/// faction holding a third of the island with a rival on every border is not
/// in the same position as one holding a third of it behind a coastline. One
/// band, never two, and never below Desperate -- pressure worsens a position,
/// it does not collapse it, and a second demotion would let one number decide
/// the state on its own.
pub fn recompute_strategic_state(view: &BoardView) -> StrategicState {
    if view.holdings_by_faction.is_empty() {
        return StrategicState::Contesting;
    }
    if view.eliminated {
        return StrategicState::Desperate;
    }

    let band = match view.share_percent() {
        share if share >= CLOSING_FLOOR_PERCENT => StrategicState::Closing,
        share if share >= ADVANTAGED_FLOOR_PERCENT => StrategicState::Advantaged,
        share if share >= CONTESTING_FLOOR_PERCENT => StrategicState::Contesting,
        share if share >= RECOVERING_FLOOR_PERCENT => StrategicState::Recovering,
        _ => StrategicState::Desperate,
    };

    if view.pressure_percent() >= SURROUNDED_PERCENT {
        demote(band)
    } else {
        band
    }
}

/// Rival-held neighbours per cell held, in percent, at which a faction counts
/// as surrounded and loses a band. One hundred: as many contested border cells
/// as cells held. Provisional, and chosen because it is the one value on this
/// axis that means something without a tuning pass -- the border is as large
/// as the country.
pub const SURROUNDED_PERCENT: i32 = 100;

/// One band worse, floored at Desperate.
fn demote(band: StrategicState) -> StrategicState {
    match band {
        StrategicState::Closing => StrategicState::Advantaged,
        StrategicState::Advantaged => StrategicState::Contesting,
        StrategicState::Contesting => StrategicState::Recovering,
        StrategicState::Recovering | StrategicState::Desperate => StrategicState::Desperate,
    }
}

// ---------------------------------------------------------------------------
// Goals
// ---------------------------------------------------------------------------

/// What a faction is trying to do, as an abstract verb.
///
/// Six words, chosen to be the smallest set that covers brief section 9's
/// considerations without deciding anything S6 owns. A `Goal` has no target,
/// no resource commitment and no explanation; it is the *shape* of the hour's
/// intent, and S6's `StrategicDirective` is what turns one into an act against
/// a named place with a stated reason.
///
/// **Not S6's vocabulary and not a doctrine.** S6's intents (Protect, Supply,
/// Develop, Expand, Pressure, Attack, Support, Investigate, Avoid, Withdraw)
/// overlap these words on purpose but are a different, larger list at a
/// different altitude; restating them here would give the island two answers to
/// "what does this faction want". Nothing in this enum is faction-specific.
///
/// `Ord` is derived and load-bearing: it is the tie-break when two goals score
/// equally, so the declaration order below is the order a tie resolves in, and
/// it is the same on every machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
    /// Get back above water: restore what was lost before doing anything else.
    Recover,
    /// Make what is already held defensible.
    Consolidate,
    /// Build and produce on held ground rather than reaching for more.
    Develop,
    /// Take unheld ground.
    Expand,
    /// Push on a rival's ground -- short of the committed attack S6 would have
    /// to explain and could abandon.
    Pressure,
    /// Give ground deliberately to survive.
    Withdraw,
}

impl Goal {
    /// Every goal, in declaration order. The one place the list exists.
    pub const ALL: [Goal; 6] = [
        Goal::Recover,
        Goal::Consolidate,
        Goal::Develop,
        Goal::Expand,
        Goal::Pressure,
        Goal::Withdraw,
    ];
}

/// How many goals an hour commits to. Three: enough for a leading intent with
/// two supports, few enough that "what is this faction doing" still has an
/// answer a person could read out. Provisional.
pub const MAX_GOALS: usize = 3;

/// How much of the score comes from the board position alone, out of
/// [`SIGNAL_CEILING`].
///
/// The affinity table below is the dominant term by construction, and that is
/// deliberate rather than incidental: brief section 9 says outright that
/// "whether a faction merely survives or actively tries to win depends on its
/// board position". Everything else modulates. This is also what makes card
/// S5's Done-when a property of the design rather than of a tuned constant --
/// a Desperate faction leads with `Recover` and an Advantaged one with
/// `Expand` because the board says so, and no combination of the secondary
/// considerations can quietly overturn it.
const BOARD_POSITION_SHARE: i32 = 100;

/// The divisor every secondary consideration is scaled by, so that a
/// full-strength signal at neutral weight contributes at most a quarter of the
/// board-position term. Four secondary considerations pointing the same way can
/// therefore move a goal a band's worth; one cannot. Provisional.
const SECONDARY_SCALE: i32 = 400;

/// How much each goal suits each board position, `0..=SIGNAL_CEILING`.
///
/// This table *is* S5's model of brief section 9, and it is the only place a
/// board position turns into an intention. Read a row as "a faction in this
/// position wants these things, this much":
///
/// * **Desperate** -- survive. Recover leads outright; withdrawing is a real
///   option; expanding and pressuring are not options at all.
/// * **Recovering** -- consolidate what the recovery has restored, and start
///   developing it. Still no appetite for a fight.
/// * **Contesting** -- the middle of the game: develop the base, hold the
///   border, take an opening if one appears. No single answer, which is what
///   contesting means.
/// * **Advantaged** -- take ground. Expansion leads, pressure follows it.
/// * **Closing** -- press the advantage to a conclusion; pressure leads and
///   expansion is nearly its equal, and recovery and withdrawal are gone.
///
/// Every number is provisional. The *shape* -- monotone from survival to
/// aggression as the position improves -- is what the brief asks for; the
/// exact values are a first pass.
const BOARD_POSITION_AFFINITY: [(StrategicState, [i32; 6]); 5] = [
    //                                Rec  Con  Dev  Exp  Pre  Wdr
    (StrategicState::Desperate, [100, 70, 20, 0, 0, 60]),
    (StrategicState::Recovering, [70, 80, 60, 20, 10, 20]),
    (StrategicState::Contesting, [20, 60, 70, 50, 50, 10]),
    (StrategicState::Advantaged, [0, 40, 60, 100, 80, 0]),
    (StrategicState::Closing, [0, 30, 50, 90, 100, 0]),
];

fn affinity(state: StrategicState, goal: Goal) -> i32 {
    let row = BOARD_POSITION_AFFINITY
        .iter()
        .find(|(banded, _)| *banded == state)
        .map(|(_, row)| row)
        .expect("BOARD_POSITION_AFFINITY carries every StrategicState");
    row[Goal::ALL
        .iter()
        .position(|g| *g == goal)
        .expect("Goal::ALL is every goal")]
}

/// The most a faction's personality may move one goal's score, either way.
///
/// Twelve, against a board-position term of up to one hundred: personality
/// colours the order of the supporting goals and can decide a near-tie, and it
/// can never make a Desperate faction lead with expansion. Brief section 9 asks
/// for "bounded personality variation" and this constant is the bound.
/// Provisional.
pub const PERSONALITY_VARIATION_CAP: i32 = 12;

/// The bound is the bound: personality, at full swing both ways, cannot cross
/// the gap between two rows of [`BOARD_POSITION_AFFINITY`]. Checked at compile
/// time, so tuning either constant past the other fails to build rather than
/// quietly letting a Desperate faction decide to expand.
const _: () = assert!(PERSONALITY_VARIATION_CAP * 2 < BOARD_POSITION_SHARE);

/// How much each of brief section 9's considerations counts.
///
/// One field per consideration the brief lists, including the ones that cannot
/// be measured yet -- those are declared at zero with the lane that will supply
/// their input named on the field. Declaring them rather than omitting them is
/// the point: the brief's list is the specification, and a consideration that
/// is missing from this struct would be a consideration nobody is tracking.
///
/// Neutral is one hundred per consideration ([`NEUTRAL_WEIGHT`]), and
/// [`GoalWeights::default`] is neutral throughout. `run_hour` uses the default
/// this round -- see [`GoalWeights::from_definition`] for why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoalWeights {
    /// Brief section 9 "Survival": how badly the position is losing.
    pub survival: i32,
    /// "Threat": the frontier that pushes back, plus what the neighbours are
    /// felt to be.
    pub threat: i32,
    /// "Opportunity": room to grow, plus an opening in a neighbour.
    pub opportunity: i32,
    /// "Territorial pressure": the size of the contested border itself.
    pub territorial_pressure: i32,
    /// "Board position". Scales the affinity table; the dominant term.
    pub board_position: i32,
    /// "Recovery needs": how much ground there is to make back.
    pub recovery_needs: i32,
    /// "Supply", read as the strategic stockpile. Also stands in for "strategic
    /// value" until S3's buildings give a cell a value beyond being a cell.
    pub supply: i32,
    /// "Relationship", and with it "Hatred": scales the relationship-derived
    /// half of threat and opportunity, so turning it down leaves a faction
    /// reading the map and ignoring the neighbours.
    pub relationship: i32,
    /// "Distance". Zero: S7 owns `ForceRecord`, and until a faction has forces
    /// standing somewhere there is no distance to anything. S7 supplies it.
    pub distance: i32,
    /// "Route danger". Zero: `Geography::effective_risk` can already price a
    /// road, but the thing whose route is dangerous is a force in transit,
    /// which is S7's. S7 supplies it.
    pub route_danger: i32,
    /// "Victory progress". Zero: `FactionDefinition::victory_conditions` is
    /// authored prose and brief section 20 leaves ordinary-faction victory
    /// conditions Open, so there is nothing to measure progress against. C9's
    /// records and S15 supply it.
    pub victory_progress: i32,
    /// "Player directives for Captain Michael's faction", and the loudest
    /// consideration here by a factor of nearly five. S6 supplies its signal
    /// from [`BoardView::directives`] through [`INTENT_GOAL_TABLE`]; see
    /// [`DIRECTIVE_WEIGHT`] for why it is as loud as it is and for the one
    /// board position at which it goes quiet.
    pub player_directives: i32,
}

/// A consideration counting exactly as much as every other.
pub const NEUTRAL_WEIGHT: i32 = 100;

/// A consideration whose input no lane supplies yet. Its term is still written
/// into the score, multiplied by a signal that is currently zero, so that the
/// lane which lands the input has one place to change and a weight to raise.
///
/// Three of them now rather than S5's four: S6 has landed the player's
/// directives, and [`directive_signal`] is what that field's term is multiplied
/// by instead. Distance and route danger wait on S7, victory progress on C9 and
/// S15.
const NOT_YET_MEASURABLE: i32 = 0;

// ---------------------------------------------------------------------------
// S6: what the player has asked for
// ---------------------------------------------------------------------------

/// How much the player's standing directives count, against a neutral hundred
/// for every consideration the board supplies.
///
/// Four hundred and eighty, which is [`SECONDARY_SCALE`] times 1.2: a single
/// standing directive at full signal is worth 120 points to the goal it argues
/// for, against a board-position term of at most 100. That is what card S6's
/// "high-weight inputs to S5" and the brief's "easy to direct" ask for -- a
/// directive moves the leading goal of a faction that has a choice, and moves
/// it by more than the whole board position could.
///
/// **It does not silence survival.** Brief section 5.9 keeps the faction on
/// automatic; a faction fighting for its life is exactly where automatic has to
/// mean something. [`directive_signal`] returns zero for a
/// [`StrategicState::Desperate`] faction, so `Recover` leads whatever the
/// player has asked for -- and the directive is not discarded, it is merely
/// unheard: it stays [`DirectiveStatus::Active`] in the save and takes effect
/// the first hour the faction is out of the hole. That is the difference
/// between an input and a command, made mechanical rather than asserted.
///
/// Provisional, like every number in this file. The two facts it has to satisfy
/// are `a_directive_moves_a_contesting_factions_leading_goal` and
/// `survival_outranks_the_player`, and both are tests rather than arithmetic in
/// a comment.
pub const DIRECTIVE_WEIGHT: i32 = 480;

/// How each of brief section 5.9's ten intents reads as a pull on each of
/// [`Goal::ALL`], at a standing priority, in [`SIGNAL_CEILING`] units.
///
/// This is the whole of the mapping between the player's vocabulary and the
/// faction's, and it exists once. Read a row as "asking for this argues for
/// these things and against those":
///
/// * **Protect** -- make the ground defensible. **Supply** and **Develop** --
///   the same abstract verb, because S5's `Develop` is "build and produce on
///   held ground" and both intents are that; they differ in what the player
///   means by it, which is the explanation's business, not the score's.
/// * **Expand** -- take unheld ground. **Pressure** -- lean on a rival.
///   **Attack** -- both, because a committed attack is pressure that intends to
///   end with the ground held, and S5 has no seventh verb for it.
/// * **Support** -- hold and produce for whoever is holding.
/// * **Investigate** -- nothing. There is no abstract verb for *finding out*:
///   S5's six are what a faction does with ground, and knowledge is not ground.
///   Mapping it to `Expand` would turn "go and look" into "go and take", which
///   is the opposite of what the player asked; mapping it to `Develop` would
///   turn it into building. So an Investigate directive is recorded, explained,
///   persists, and moves no goal, and [`Intent::explained_goal`] says exactly
///   that to the player rather than pretending. **Open, and named as Open in
///   the S6 report:** the honest fix is a seventh goal or a knowledge model,
///   and neither is S6's to mint.
/// * **Avoid** -- the only purely negative row. "Stay away from here" is not
///   withdrawal and not defence; in a vocabulary of six verbs it is the absence
///   of expansion and pressure, so it argues against exactly those.
/// * **Withdraw** -- give the ground up, and stop reaching for more.
///
/// Every number is provisional; the *shape* -- one intent, one or two verbs,
/// negatives only where the player said "not" -- is what the brief asks for.
pub const INTENT_GOAL_TABLE: [(Intent, [i32; 6]); 10] = [
    //                          Rec  Con  Dev  Exp  Pre  Wdr
    (Intent::Protect, [0, 100, 0, 0, 0, 0]),
    (Intent::Supply, [0, 0, 100, 0, 0, 0]),
    (Intent::Develop, [0, 0, 100, 0, 0, 0]),
    (Intent::Expand, [0, 0, 0, 100, 0, 0]),
    (Intent::Pressure, [0, 0, 0, 0, 100, 0]),
    (Intent::Attack, [0, 0, 0, 100, 100, 0]),
    (Intent::Support, [0, 100, 100, 0, 0, 0]),
    (Intent::Investigate, [0, 0, 0, 0, 0, 0]),
    (Intent::Avoid, [0, 0, 0, -100, -100, 0]),
    (Intent::Withdraw, [0, 0, 0, -100, -100, 100]),
];

/// One intent's row of [`INTENT_GOAL_TABLE`], in [`Goal::ALL`] order.
pub fn intent_goal_row(intent: Intent) -> [i32; 6] {
    INTENT_GOAL_TABLE
        .iter()
        .find(|(listed, _)| *listed == intent)
        .map(|(_, row)| *row)
        .expect("INTENT_GOAL_TABLE carries every Intent")
}

/// What the player's standing directives say about one goal, as a signal in
/// `-SIGNAL_CEILING..=SIGNAL_CEILING`.
///
/// Directives sum, each scaled by its [`Priority`](crate::strategy::directive::Priority):
/// two standing requests pulling opposite ways cancel, an urgent one outweighs
/// a standing one, and the total is clamped like every other signal here so
/// that stacking directives cannot make one goal unanswerable.
///
/// Zero for a [`StrategicState::Desperate`] faction -- see [`DIRECTIVE_WEIGHT`].
fn directive_signal(view: &BoardView, state: StrategicState, goal: Goal) -> i32 {
    if state == StrategicState::Desperate {
        return 0;
    }
    let index = Goal::ALL
        .iter()
        .position(|g| *g == goal)
        .expect("Goal::ALL is every goal");
    let total: i32 = view
        .directives
        .values()
        .map(|directive| {
            intent_goal_row(directive.intent)[index] * directive.priority.signal_percent() / 100
        })
        .sum();
    clamp_signal(total)
}

impl Default for GoalWeights {
    /// Neutral: every measurable consideration at [`NEUTRAL_WEIGHT`], the
    /// player's directives at [`DIRECTIVE_WEIGHT`], and every consideration
    /// whose input no lane supplies yet at zero. A faction with no authored
    /// record reads the board, hears the player, and knows nothing else.
    ///
    /// The directive weight is *not* neutral and is not meant to be: a
    /// directive that counted the same as the size of a stockpile would not be
    /// direction. It is still a weight in a sum rather than an instruction --
    /// which is what the survival rule in [`directive_signal`] demonstrates.
    fn default() -> Self {
        Self {
            survival: NEUTRAL_WEIGHT,
            threat: NEUTRAL_WEIGHT,
            opportunity: NEUTRAL_WEIGHT,
            territorial_pressure: NEUTRAL_WEIGHT,
            board_position: NEUTRAL_WEIGHT,
            recovery_needs: NEUTRAL_WEIGHT,
            supply: NEUTRAL_WEIGHT,
            relationship: NEUTRAL_WEIGHT,
            distance: 0,
            route_danger: 0,
            victory_progress: 0,
            player_directives: DIRECTIVE_WEIGHT,
        }
    }
}

/// The most an authored record may move one weight from neutral, either way.
/// Fifty against a neutral hundred: a record can halve a consideration or make
/// it half again as loud, and cannot silence one or make it the only one.
/// Provisional.
pub const WEIGHT_VARIATION_CAP: i32 = 50;

impl GoalWeights {
    /// Weights derived from an authored faction record.
    ///
    /// **Not called by `run_hour` this round, on purpose.** The bridge does not
    /// load `content/factions/*.json` yet, so `resolve_midnight_in` hands the
    /// tick an empty `FactionDefinitions`; if the hour scored records, a game
    /// launched from Godot and a game run in the test harness would be two
    /// different islands. `strategic_determinism.rs::the_registry_cannot_change_a_tick_yet`
    /// pins exactly that, and it is still green. This function is the seam, and
    /// it is tested here in isolation.
    ///
    /// Acceptance criteria for closing the seam (`AGENTS.md` section 6): the
    /// bridge loads the faction records and passes them through
    /// `resolve_midnight_in`, `run_hour` swaps `GoalWeights::default()` for
    /// this call, and `the_registry_cannot_change_a_tick_yet` is deleted by the
    /// lane that does it -- that test says of itself that it is written to be
    /// deleted.
    ///
    /// What it reads, and what it deliberately does not: the authored
    /// *priorities* (how much this faction wants resources, and what its
    /// opening posture toward its neighbours is). It does **not** read
    /// `concept_key`, so no faction gets a strategy for being that faction, and
    /// it does not read `doctrine`, `board_position_behavior`, `recovery_rules`
    /// or `elimination_rules`, which are Provisional prose that S15 owns and
    /// that no code may branch on today.
    pub fn from_definition(definition: &FactionDefinition) -> Self {
        let mean = |values: Vec<i32>| -> i32 {
            if values.is_empty() {
                return 0;
            }
            let total: i32 = values.iter().sum();
            total / values.len() as i32
        };

        // How much this faction cares about stockpiles at all, as the mean
        // magnitude of its authored resource priorities. A record that weights
        // nothing wants nothing in particular and stays neutral.
        let supply_appetite = mean(
            definition
                .resource_priorities
                .values()
                .map(|weight| i32::from(*weight).abs())
                .collect(),
        );
        // Its opening posture toward the neighbours, on the same two axes the
        // live scoring reads.
        let opening_hostility = mean(
            definition
                .relationship_tendencies
                .values()
                .map(|r| (i32::from(r.fear) + i32::from(r.hatred)) / 2)
                .collect(),
        );
        let opening_invitation = mean(
            definition
                .relationship_tendencies
                .values()
                .map(|r| i32::from(r.perceived_opportunity))
                .collect(),
        );

        Self {
            supply: nudge_weight(supply_appetite),
            threat: nudge_weight(opening_hostility),
            opportunity: nudge_weight(opening_invitation),
            ..Self::default()
        }
    }
}

/// A weight moved from neutral by an authored signal, bounded both ways.
fn nudge_weight(signal: i32) -> i32 {
    NEUTRAL_WEIGHT + signal.clamp(-WEIGHT_VARIATION_CAP, WEIGHT_VARIATION_CAP)
}

/// What this faction wants this hour, best first, at most [`MAX_GOALS`].
///
/// `goal_draw` is S4's `strategic.goal` draw for this faction this hour, and it
/// is the *only* source of variation here: everything else is a function of the
/// board. Its whole effect is bounded by [`PERSONALITY_VARIATION_CAP`].
///
/// The returned list is the interface. The scores that produced it are local
/// `i32`s and stay local -- brief section 9's "do not expose raw utility
/// arithmetic".
///
/// A goal scoring zero or less is not returned: an hour is allowed to want
/// fewer than three things, and padding the list would put words in a faction's
/// mouth. An eliminated faction is the case where that matters -- it wants to
/// recover and nothing else.
pub fn choose_goals(
    view: &BoardView,
    state: StrategicState,
    goal_draw: u64,
    weights: &GoalWeights,
) -> Vec<Goal> {
    let mut scored: Vec<(i32, Goal)> = Goal::ALL
        .into_iter()
        .map(|goal| (score(view, state, goal, goal_draw, weights), goal))
        .filter(|(score, _)| *score > 0)
        .collect();

    // Descending by score; ties by declaration order, so the ordering is total
    // and identical on every machine.
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    scored
        .into_iter()
        .take(MAX_GOALS)
        .map(|(_, goal)| goal)
        .collect()
}

/// One goal's utility. Private, returns a bare `i32`, and never escapes.
///
/// Every one of brief section 9's considerations appears here exactly once,
/// named, including the ones weighted zero -- so the list of terms in this
/// function can be diffed against the brief by eye, which is the same
/// discipline `FactionDefinition` follows for section 19.
fn score(
    view: &BoardView,
    state: StrategicState,
    goal: Goal,
    goal_draw: u64,
    weights: &GoalWeights,
) -> i32 {
    let share = view.share_percent();
    let pressure = view.pressure_percent();
    let room = view.room_percent();
    let stock = view.stock_percent();
    // The relationship weight scales the relationship-derived signals before
    // they are blended, so "reads the neighbours" and "reads the map" are
    // separable knobs rather than one.
    let hostility = view.hostility_percent() * weights.relationship / NEUTRAL_WEIGHT;
    let invitation = view.invitation_percent() * weights.relationship / NEUTRAL_WEIGHT;

    // Survival: losing the board and being surrounded, equally weighted.
    let survival_urgency = clamp_signal((SIGNAL_CEILING - share + pressure) / 2);
    // Threat: the border that pushes back, and what the neighbours feel like.
    let threat = clamp_signal((pressure + hostility) / 2);
    // Opportunity: somewhere to go, and someone who looks open.
    let opportunity = clamp_signal((room + invitation) / 2);
    // Recovery needs: only a faction that is actually behind has any.
    let recovery_need = match state {
        StrategicState::Desperate => SIGNAL_CEILING,
        StrategicState::Recovering => SIGNAL_CEILING - share,
        _ => 0,
    };

    let board_position = weights.board_position * affinity(state, goal) / BOARD_POSITION_SHARE;

    let secondary = match goal {
        Goal::Recover => {
            weights.survival * survival_urgency + weights.recovery_needs * recovery_need
        }
        Goal::Consolidate => weights.threat * threat + weights.territorial_pressure * pressure,
        // Develop is what a faction does when it is short of things, so an
        // empty stockpile argues for it and a full one does not.
        Goal::Develop => weights.supply * (SIGNAL_CEILING - stock),
        // Expansion needs somewhere to go and something to go with.
        Goal::Expand => weights.opportunity * opportunity + weights.supply * stock / 2,
        Goal::Pressure => {
            weights.opportunity * opportunity + weights.territorial_pressure * pressure
        }
        Goal::Withdraw => weights.survival * survival_urgency + weights.threat * threat / 2,
    };

    // What the player has asked this faction for. The loudest term here, and
    // the only one that is not a fact about the board -- but still a term, and
    // still worth nothing to a faction that is fighting for its life.
    let directives = weights.player_directives * directive_signal(view, state, goal);

    // The considerations no lane supplies yet. Written into every score, at a
    // signal of zero, so the lane that lands the input has one line to change.
    let unsupplied = weights.distance * NOT_YET_MEASURABLE
        + weights.route_danger * NOT_YET_MEASURABLE
        + weights.victory_progress * NOT_YET_MEASURABLE;

    board_position
        + (secondary + directives + unsupplied) / SECONDARY_SCALE
        + personality(goal, goal_draw)
}

/// This faction's bounded personality, for this goal, this hour.
///
/// One byte of the hour's `strategic.goal` draw per goal, folded into
/// `-PERSONALITY_VARIATION_CAP..=PERSONALITY_VARIATION_CAP`. Six goals, eight
/// bytes in a `u64`, so every goal gets its own byte and no two goals move
/// together. No new draw and no second mixer: S4 already drew this number.
fn personality(goal: Goal, goal_draw: u64) -> i32 {
    let index = Goal::ALL
        .iter()
        .position(|g| *g == goal)
        .expect("Goal::ALL is every goal");
    let byte = ((goal_draw >> (index * 8)) & 0xFF) as i32;
    let span = PERSONALITY_VARIATION_CAP * 2 + 1;
    byte % span - PERSONALITY_VARIATION_CAP
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::faction::{ConceptKey, FactionState};

    /// A board the tests state directly, so that a scoring test is about
    /// scoring rather than about the vertical slice's map. `BoardView::of` is
    /// exercised separately against the real graph below.
    ///
    /// Whatever this faction does not hold, the rest of the island does, so
    /// `holdings_by_faction` is empty exactly when the board is unclaimed --
    /// which is the distinction `recompute_strategic_state` turns on.
    fn a_view(held: usize, cells: usize, contested: usize, open: usize) -> BoardView {
        let mine = ConceptKey::Michael.faction_id();
        let theirs = ConceptKey::Pirates.faction_id();
        let mut holdings_by_faction: BTreeMap<String, usize> = BTreeMap::new();
        if held > 0 {
            holdings_by_faction.insert(mine.clone(), held);
        }
        if cells > held {
            holdings_by_faction.insert(theirs.clone(), cells - held);
        }
        BoardView {
            faction_id: mine.clone(),
            eliminated: false,
            cells_on_the_board: cells,
            held: (0..held).map(|n| format!("world.cell.held_{n}")).collect(),
            contested_frontier: (0..contested)
                .map(|n| (format!("world.cell.border_{n}"), theirs.clone()))
                .collect(),
            open_frontier: (0..open).map(|n| format!("world.cell.open_{n}")).collect(),
            holdings_by_faction,
            resources: BTreeMap::new(),
            relationships: BTreeMap::new(),
            directives: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // The board, read from the real graph
    // -----------------------------------------------------------------------

    /// `BoardView::of` reads control the way the rest of the crate does: the
    /// save's `ownership` first, the authored graph second, and a cell nobody
    /// claims belongs to nobody. It splits the frontier into ground that pushes
    /// back and ground that does not.
    #[test]
    fn a_board_view_is_read_from_ownership_and_the_graph_and_nothing_else() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        state
            .factions
            .insert(ConceptKey::Michael.faction_id(), FactionState::new());
        state
            .factions
            .insert(ConceptKey::Pirates.faction_id(), FactionState::new());

        // Nothing is authored as owned, so a fresh board is nobody's.
        let empty = BoardView::of(&ConceptKey::Michael.faction_id(), &state, &geography);
        assert!(empty.held.is_empty());
        assert!(empty.holdings_by_faction.is_empty());
        assert!(empty.contested_frontier.is_empty());
        assert!(empty.open_frontier.is_empty());
        assert_eq!(empty.cells_on_the_board, geography.locations.len());
        assert!(
            empty
                .relationships
                .contains_key(&ConceptKey::Pirates.faction_id()),
            "the view carries every other faction present, zero-filled"
        );
        assert!(
            !empty
                .relationships
                .contains_key(&ConceptKey::Michael.faction_id()),
            "a faction has no relationship with itself"
        );

        state
            .set_control(
                "world.cell.damaged_estate",
                Some(ConceptKey::Michael.faction_id()),
                &geography,
            )
            .expect("a claim on an authored cell is legal");
        state
            .set_control(
                "world.cell.river_landing",
                Some(ConceptKey::Pirates.faction_id()),
                &geography,
            )
            .expect("a claim on an authored cell is legal");

        let view = BoardView::of(&ConceptKey::Michael.faction_id(), &state, &geography);
        assert_eq!(
            view.held,
            BTreeSet::from(["world.cell.damaged_estate".to_owned()])
        );
        assert_eq!(
            view.holdings_by_faction,
            BTreeMap::from([
                (ConceptKey::Michael.faction_id(), 1),
                (ConceptKey::Pirates.faction_id(), 1),
            ])
        );
        assert_eq!(
            view.contested_frontier.get("world.cell.river_landing"),
            Some(&ConceptKey::Pirates.faction_id()),
            "the landing is next to the estate and somebody else holds it"
        );
        assert!(
            view.open_frontier.contains("world.cell.black_beach"),
            "the beach is next to the estate and nobody holds it"
        );
        assert!(
            !view.open_frontier.contains("world.cell.damaged_estate"),
            "a faction is not its own frontier"
        );
        for cell in view.contested_frontier.keys() {
            assert!(
                !view.open_frontier.contains(cell),
                "{cell} is on both frontiers"
            );
        }

        // Pure: reading the same board twice gives the same view, and the view
        // of the other faction is the mirror image rather than a second model.
        assert_eq!(
            view,
            BoardView::of(&ConceptKey::Michael.faction_id(), &state, &geography)
        );
        let theirs = BoardView::of(&ConceptKey::Pirates.faction_id(), &state, &geography);
        assert_eq!(
            theirs.contested_frontier.get("world.cell.damaged_estate"),
            Some(&ConceptKey::Michael.faction_id())
        );
    }

    /// A faction the save does not carry still gets a view: it holds nothing,
    /// wants for nothing, and knows nobody. Nothing panics on the way.
    #[test]
    fn a_faction_not_in_the_save_still_has_a_view() {
        let geography = Geography::black_beach_vertical_slice();
        let state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        let view = BoardView::of(&ConceptKey::Cthulhu.faction_id(), &state, &geography);
        assert!(view.held.is_empty());
        assert!(!view.eliminated);
        assert!(view.resources.is_empty());
        assert!(view.relationships.is_empty());
    }

    // -----------------------------------------------------------------------
    // The five states
    // -----------------------------------------------------------------------

    /// Before anyone claims anything, nobody is winning or losing.
    #[test]
    fn an_unclaimed_board_is_contesting_for_everyone() {
        let mut view = a_view(0, 20, 0, 0);
        view.holdings_by_faction.clear();
        assert_eq!(recompute_strategic_state(&view), StrategicState::Contesting);
        assert_eq!(
            recompute_strategic_state(&view),
            StrategicState::default(),
            "the field's default and the unclaimed board agree on purpose"
        );
    }

    /// The share of the island puts a faction in a band, and all five bands are
    /// reachable. Twenty cells, so one cell is five percent and the thresholds
    /// land on whole cells.
    #[test]
    fn the_share_of_the_island_gives_all_five_bands() {
        let bands: Vec<StrategicState> = [0usize, 1, 3, 7, 12]
            .into_iter()
            .map(|held| recompute_strategic_state(&a_view(held, 20, 0, 0)))
            .collect();
        assert_eq!(
            bands,
            vec![
                StrategicState::Desperate,
                StrategicState::Recovering,
                StrategicState::Contesting,
                StrategicState::Advantaged,
                StrategicState::Closing,
            ]
        );

        // Each floor is the first share that enters its band, and one cell less
        // is the band below -- the thresholds are the constants, not a fudge.
        assert_eq!(RECOVERING_FLOOR_PERCENT, 5);
        assert_eq!(CONTESTING_FLOOR_PERCENT, 15);
        assert_eq!(ADVANTAGED_FLOOR_PERCENT, 35);
        assert_eq!(CLOSING_FLOOR_PERCENT, 60);
        assert_eq!(
            recompute_strategic_state(&a_view(6, 20, 0, 0)),
            StrategicState::Contesting,
            "thirty percent is short of the advantaged floor"
        );
    }

    /// Being surrounded costs exactly one band, however surrounded, and never
    /// takes a faction below the bottom.
    #[test]
    fn being_surrounded_costs_one_band_and_only_one() {
        // Four of twenty cells is Contesting. A border smaller than the
        // holding leaves it there; a border as large as the holding does not.
        assert_eq!(
            recompute_strategic_state(&a_view(4, 20, 3, 0)),
            StrategicState::Contesting
        );
        assert_eq!(
            recompute_strategic_state(&a_view(4, 20, 4, 0)),
            StrategicState::Recovering
        );
        assert_eq!(
            recompute_strategic_state(&a_view(4, 20, 8, 0)),
            StrategicState::Recovering,
            "a bigger border is not a second demotion"
        );
        // The bottom is the bottom: one cell is Recovering, and surrounded it
        // is Desperate and no lower.
        assert_eq!(
            recompute_strategic_state(&a_view(1, 20, 9, 0)),
            StrategicState::Desperate
        );
    }

    /// An eliminated faction is Desperate whatever else the board says --
    /// which is the state S10's recovery chain will read.
    #[test]
    fn an_eliminated_faction_is_desperate() {
        let mut view = a_view(12, 20, 0, 0);
        assert_eq!(recompute_strategic_state(&view), StrategicState::Closing);
        view.eliminated = true;
        assert_eq!(recompute_strategic_state(&view), StrategicState::Desperate);
    }

    /// An empty graph is not a division by zero.
    #[test]
    fn a_board_with_no_cells_does_not_divide_by_zero() {
        let view = a_view(0, 0, 0, 0);
        assert_eq!(view.share_percent(), 0);
        assert_eq!(view.pressure_percent(), 0);
        assert_eq!(view.room_percent(), 0);
        assert_eq!(recompute_strategic_state(&view), StrategicState::Contesting);
    }

    // -----------------------------------------------------------------------
    // Goals
    // -----------------------------------------------------------------------

    /// **Card S5's Done-when.** One function, two boards: the faction clinging
    /// to a single surrounded cell leads with `Recover`, and the one holding
    /// two fifths of the island behind a quiet border leads with `Expand`.
    ///
    /// Same `choose_goals`, same neutral weights, same draw. The only
    /// difference is the board, which is the claim being made: strategy comes
    /// out of position, not out of who the faction is. Neither view names a
    /// concept key, a doctrine or a proper name.
    #[test]
    fn a_desperate_faction_recovers_and_an_advantaged_one_expands() {
        let weights = GoalWeights::default();
        // One draw for both, so the personality term cannot be what separates
        // them. Every draw separates them: see the sweep below.
        let goal_draw = 0x0123_4567_89AB_CDEF;

        // One cell of twenty, with twice its own size in rival border: five
        // percent is the Recovering floor, and being surrounded demotes it.
        let cornered = a_view(1, 20, 2, 0);
        assert_eq!(
            recompute_strategic_state(&cornered),
            StrategicState::Desperate
        );

        // Eight cells of twenty, one contested border cell, four ways to grow,
        // and a full stockpile to grow with.
        let mut ascendant = a_view(8, 20, 1, 4);
        ascendant
            .resources
            .insert("resource.example".into(), WELL_STOCKED);
        assert_eq!(
            recompute_strategic_state(&ascendant),
            StrategicState::Advantaged
        );

        let desperate_goals = choose_goals(
            &cornered,
            recompute_strategic_state(&cornered),
            goal_draw,
            &weights,
        );
        let advantaged_goals = choose_goals(
            &ascendant,
            recompute_strategic_state(&ascendant),
            goal_draw,
            &weights,
        );

        assert_eq!(
            desperate_goals.first(),
            Some(&Goal::Recover),
            "a desperate faction leads with recovery: {desperate_goals:?}"
        );
        assert_eq!(
            advantaged_goals.first(),
            Some(&Goal::Expand),
            "an advantaged faction leads with expansion: {advantaged_goals:?}"
        );
        assert!(!advantaged_goals.contains(&Goal::Recover));
        assert!(!desperate_goals.contains(&Goal::Expand));

        // And the personality draw is a colour, not a decision: no hour of any
        // island flips either verdict.
        for step in 0..512u64 {
            let draw = step.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            assert_eq!(
                choose_goals(&cornered, StrategicState::Desperate, draw, &weights).first(),
                Some(&Goal::Recover)
            );
            assert_eq!(
                choose_goals(&ascendant, StrategicState::Advantaged, draw, &weights).first(),
                Some(&Goal::Expand)
            );
        }
    }

    /// An hour commits to at most three goals, in a stable order, and the same
    /// inputs always give the same list.
    #[test]
    fn goals_are_bounded_ordered_and_reproducible() {
        let view = a_view(6, 20, 3, 3);
        let state = recompute_strategic_state(&view);
        let goals = choose_goals(&view, state, 12_345, &GoalWeights::default());
        assert!(goals.len() <= MAX_GOALS);
        assert_eq!(MAX_GOALS, 3);
        assert_eq!(
            goals,
            choose_goals(&view, state, 12_345, &GoalWeights::default())
        );
        let unique: BTreeSet<Goal> = goals.iter().copied().collect();
        assert_eq!(unique.len(), goals.len(), "a goal is chosen at most once");
    }

    /// Personality is bounded by its constant, every goal gets its own byte of
    /// the draw, and the bound is small against the board-position term.
    #[test]
    fn personality_variation_is_bounded_by_its_constant() {
        let mut seen: BTreeSet<i32> = BTreeSet::new();
        for step in 0..2_048u64 {
            let draw = step.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ step;
            for goal in Goal::ALL {
                let nudge = personality(goal, draw);
                assert!(
                    (-PERSONALITY_VARIATION_CAP..=PERSONALITY_VARIATION_CAP).contains(&nudge),
                    "{goal:?} moved by {nudge}"
                );
                seen.insert(nudge);
            }
        }
        assert!(
            seen.len() > PERSONALITY_VARIATION_CAP as usize,
            "the variation actually varies"
        );
        // Six goals, six distinct bytes: two goals do not move together.
        let draw = 0x0102_0304_0506_0708u64;
        let nudges: Vec<i32> = Goal::ALL
            .into_iter()
            .map(|g| personality(g, draw))
            .collect();
        assert_eq!(nudges.len(), 6);
        assert!(nudges.iter().collect::<BTreeSet<_>>().len() > 1);
    }

    /// The considerations the brief lists but no lane can supply yet are
    /// declared, weighted zero, and contribute nothing -- so raising one of
    /// them alone changes no decision until its input arrives.
    #[test]
    fn the_unsupplied_considerations_are_declared_and_inert() {
        let neutral = GoalWeights::default();
        assert_eq!(neutral.distance, 0);
        assert_eq!(neutral.route_danger, 0);
        assert_eq!(neutral.victory_progress, 0);

        let view = a_view(6, 20, 3, 3);
        let state = recompute_strategic_state(&view);
        let loud = GoalWeights {
            distance: 10_000,
            route_danger: 10_000,
            victory_progress: 10_000,
            // Supplied since S6, but this board carries no directive, so a
            // weight on it is still a weight with nothing behind it.
            player_directives: 10_000,
            ..neutral
        };
        assert_eq!(
            choose_goals(&view, state, 99, &neutral),
            choose_goals(&view, state, 99, &loud),
            "a weight with no signal behind it cannot move a decision"
        );
    }

    // -----------------------------------------------------------------------
    // S6: what the player has asked for
    // -----------------------------------------------------------------------

    fn a_directive(id: &str, intent: Intent) -> StrategicDirective {
        StrategicDirective::new(id, ConceptKey::Michael.faction_id(), intent)
    }

    /// **Card S6's other half.** A directive is a high-weight input: a standing
    /// request to expand moves a Contesting faction's leading goal to `Expand`,
    /// which the board alone would never have chosen -- a Contesting faction
    /// develops first.
    #[test]
    fn a_directive_moves_a_contesting_factions_leading_goal() {
        let view = a_view(4, 20, 2, 2);
        let state = recompute_strategic_state(&view);
        assert_eq!(state, StrategicState::Contesting);
        assert_ne!(
            choose_goals(&view, state, 0, &GoalWeights::default())
                .first()
                .copied(),
            Some(Goal::Expand),
            "the board alone does not lead with expansion here"
        );

        let mut directed = view.clone();
        directed.directives.insert(
            "directive.take_the_north".into(),
            a_directive("directive.take_the_north", Intent::Expand),
        );
        // Every draw, so this is the directive rather than a lucky personality.
        for goal_draw in [0u64, 1, 7, 99, u64::MAX / 3, u64::MAX] {
            assert_eq!(
                choose_goals(&directed, state, goal_draw, &GoalWeights::default())
                    .first()
                    .copied(),
                Some(Goal::Expand),
                "a standing directive to expand must lead, at draw {goal_draw}"
            );
        }
    }

    /// The other side of the same coin: a directive is an input, not a command.
    /// A faction fighting for its life leads with `Recover` whatever it has
    /// been asked for -- and the request is not discarded, merely unheard.
    #[test]
    fn survival_outranks_the_player() {
        let view = a_view(0, 20, 0, 0);
        let state = recompute_strategic_state(&view);
        assert_eq!(state, StrategicState::Desperate);

        for intent in Intent::ALL {
            let mut directed = view.clone();
            directed.directives.insert(
                "directive.whatever".into(),
                StrategicDirective {
                    priority: crate::strategy::directive::Priority::Urgent,
                    ..a_directive("directive.whatever", intent)
                },
            );
            for goal_draw in [0u64, 3, 42, u64::MAX] {
                assert_eq!(
                    choose_goals(&directed, state, goal_draw, &GoalWeights::default())
                        .first()
                        .copied(),
                    Some(Goal::Recover),
                    "a desperate faction asked to {} still recovers first",
                    intent.as_str()
                );
            }
        }
    }

    /// The vocabularies meet in exactly one table, every intent has a row, and
    /// the rows are bounded by the same signal ceiling as everything else.
    #[test]
    fn every_intent_has_exactly_one_row_and_stays_inside_the_ceiling() {
        assert_eq!(INTENT_GOAL_TABLE.len(), Intent::ALL.len());
        for intent in Intent::ALL {
            let matching = INTENT_GOAL_TABLE
                .iter()
                .filter(|(listed, _)| *listed == intent)
                .count();
            assert_eq!(matching, 1, "{intent:?} has {matching} rows");
            for pull in intent_goal_row(intent) {
                assert!(
                    (-SIGNAL_CEILING..=SIGNAL_CEILING).contains(&pull),
                    "{intent:?} pulls {pull}, outside the signal ceiling"
                );
            }
        }
    }

    /// Two equal requests pulling opposite ways cancel on the goal they share
    /// -- `Expand` against `Avoid` leaves the leading goal exactly where the
    /// board put it -- and an urgent one does not. Priority is a word the
    /// player says, and it is the only thing that scales a directive.
    #[test]
    fn opposite_requests_cancel_and_urgency_breaks_the_tie() {
        use crate::strategy::directive::Priority;
        let view = a_view(4, 20, 2, 2);
        let state = recompute_strategic_state(&view);
        let plain = choose_goals(&view, state, 5, &GoalWeights::default())
            .first()
            .copied();

        let mut both = view.clone();
        both.directives.insert(
            "directive.take_it".into(),
            a_directive("directive.take_it", Intent::Expand),
        );
        both.directives.insert(
            "directive.leave_it".into(),
            a_directive("directive.leave_it", Intent::Avoid),
        );
        assert_eq!(
            choose_goals(&both, state, 5, &GoalWeights::default())
                .first()
                .copied(),
            plain,
            "a standing take and a standing leave cancel on the ground they \
             share, and the faction leads with what the board wanted"
        );

        both.directives.insert(
            "directive.take_it".into(),
            StrategicDirective {
                priority: Priority::Urgent,
                ..a_directive("directive.take_it", Intent::Expand)
            },
        );
        assert_eq!(
            choose_goals(&both, state, 5, &GoalWeights::default())
                .first()
                .copied(),
            Some(Goal::Expand),
            "urgency outweighs a standing request pulling the other way"
        );
    }

    /// A directive the player gave someone else is somebody else's business,
    /// and one that has ended asks for nothing.
    #[test]
    fn only_this_factions_standing_directives_reach_its_view() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            11,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        state
            .factions
            .insert(ConceptKey::Michael.faction_id(), FactionState::new());
        state
            .factions
            .insert(ConceptKey::Pirates.faction_id(), FactionState::new());
        state
            .issue_directive(a_directive("directive.mine", Intent::Expand), &geography)
            .expect("legal");
        state
            .issue_directive(
                StrategicDirective::new(
                    "directive.theirs",
                    ConceptKey::Pirates.faction_id(),
                    Intent::Expand,
                ),
                &geography,
            )
            .expect("legal");

        let mine = BoardView::of(&ConceptKey::Michael.faction_id(), &state, &geography);
        assert_eq!(
            mine.directives.keys().collect::<Vec<_>>(),
            vec!["directive.mine"]
        );

        state
            .cancel_directive("directive.mine")
            .expect("cancelling is legal");
        let after = BoardView::of(&ConceptKey::Michael.faction_id(), &state, &geography);
        assert!(
            after.directives.is_empty(),
            "a cancelled directive asks for nothing"
        );
    }

    /// The registry seam: `from_definition` reads the authored priorities,
    /// stays inside its bound, and reads nothing Provisional. Tested in
    /// isolation because `run_hour` deliberately does not call it yet.
    #[test]
    fn weights_from_a_definition_are_bounded_and_ignore_provisional_prose() {
        use crate::strategy::faction::FactionDefinition;

        let neutral = GoalWeights::from_definition(&FactionDefinition::default());
        assert_eq!(
            neutral,
            GoalWeights::default(),
            "a record that authors no priorities is neutral"
        );

        // Two records identical but for their concept key and their Provisional
        // prose. Nothing in this file may tell them apart.
        let mut one = FactionDefinition {
            id: ConceptKey::Elves.faction_id(),
            concept_key: ConceptKey::Elves,
            doctrine: "holds the treeline and never gives it up".into(),
            recovery_rules: "withdraws to the canopy".into(),
            elimination_rules: "the forest goes quiet".into(),
            ..FactionDefinition::default()
        };
        one.resource_priorities
            .insert("resource.example".into(), 20);
        let mut other = FactionDefinition {
            id: ConceptKey::Pirates.faction_id(),
            concept_key: ConceptKey::Pirates,
            doctrine: "takes what floats".into(),
            ..FactionDefinition::default()
        };
        other
            .resource_priorities
            .insert("resource.example".into(), 20);
        assert_eq!(
            GoalWeights::from_definition(&one),
            GoalWeights::from_definition(&other),
            "a faction's strategy may not be keyed on which faction it is"
        );
        assert_eq!(
            GoalWeights::from_definition(&one).supply,
            NEUTRAL_WEIGHT + 20
        );

        // The bound holds against an absurd record either way.
        let mut extreme = FactionDefinition::default();
        extreme
            .resource_priorities
            .insert("resource.example".into(), i16::MAX);
        extreme.relationship_tendencies.insert(
            ConceptKey::Michael.faction_id(),
            Relationship {
                fear: i16::MIN,
                hatred: i16::MIN,
                perceived_opportunity: i16::MAX,
                ..Relationship::default()
            },
        );
        let bounded = GoalWeights::from_definition(&extreme);
        for weight in [bounded.supply, bounded.threat, bounded.opportunity] {
            assert!(
                (NEUTRAL_WEIGHT - WEIGHT_VARIATION_CAP..=NEUTRAL_WEIGHT + WEIGHT_VARIATION_CAP)
                    .contains(&weight),
                "{weight} escaped the bound"
            );
        }
    }

    // -----------------------------------------------------------------------
    // The hour
    // -----------------------------------------------------------------------

    /// The whole of S5 as `run_hour` runs it: one strategic hour recomputes
    /// every faction's board position and goals from the board, and the board
    /// alone decides which faction gets which.
    ///
    /// The registry stays empty here on purpose -- `run_hour` scores with
    /// `GoalWeights::default()` this round, and
    /// `strategic_determinism.rs::the_registry_cannot_change_a_tick_yet` is the
    /// test that holds it to that.
    #[test]
    fn one_strategic_hour_recomputes_every_factions_position_and_goals() {
        use crate::strategy::faction::FactionDefinitions;

        let geography = Geography::black_beach_vertical_slice();
        let definitions = FactionDefinitions::new();
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        for concept in [ConceptKey::Michael, ConceptKey::Pirates] {
            state
                .factions
                .insert(concept.faction_id(), FactionState::new());
        }

        // One faction takes the whole island bar a single cell; the other is
        // left cornered on it.
        let mut cells: Vec<String> = geography.all_location_ids().map(str::to_owned).collect();
        let cornered_cell = cells.pop().expect("the slice has cells");
        for cell in &cells {
            state
                .set_control(cell, Some(ConceptKey::Michael.faction_id()), &geography)
                .expect("a claim on an authored cell is legal");
        }
        state
            .set_control(
                &cornered_cell,
                Some(ConceptKey::Pirates.faction_id()),
                &geography,
            )
            .expect("a claim on an authored cell is legal");

        // Nothing has decided anything yet: both sit on the field's default.
        for faction in state.factions.values() {
            assert_eq!(faction.strategic_state, StrategicState::default());
            assert!(faction.current_goals.is_empty());
        }

        state.strategic_tick(&geography, &definitions);

        let ascendant = state
            .factions
            .get(&ConceptKey::Michael.faction_id())
            .expect("the faction is in the save");
        let cornered = state
            .factions
            .get(&ConceptKey::Pirates.faction_id())
            .expect("the faction is in the save");
        assert_eq!(ascendant.strategic_state, StrategicState::Closing);
        assert_eq!(cornered.strategic_state, StrategicState::Desperate);
        assert_eq!(cornered.current_goals.first(), Some(&Goal::Recover));
        assert!(!ascendant.current_goals.is_empty());
        assert!(!ascendant.current_goals.contains(&Goal::Recover));

        // The hour is reproducible, and running it again on the same board
        // reaches the same verdict rather than drifting.
        let after_one = state.to_json();
        let mut twin = ExpeditionState::from_json(&after_one).expect("the save reloads");
        twin.strategic_tick(&geography, &definitions);
        let mut again = ExpeditionState::from_json(&after_one).expect("the save reloads");
        again.strategic_tick(&geography, &definitions);
        assert_eq!(twin.to_json(), again.to_json());
    }

    // -----------------------------------------------------------------------
    // Brief section 9: raw utility arithmetic does not leave the module
    // -----------------------------------------------------------------------

    /// The saved faction carries the *verdict* and no arithmetic.
    ///
    /// Its serialized keys are exactly the four S1 declared plus S5's
    /// `current_goals`, and every goal is a word rather than a number. A score
    /// leaking into the save -- a cached utility, a "threat level", a tie-break
    /// remainder -- fails here, which is the guard brief section 9 asks for.
    #[test]
    fn a_faction_state_serializes_no_score() {
        let mut faction = FactionState::new();
        faction.strategic_state = StrategicState::Advantaged;
        faction.current_goals = vec![Goal::Expand, Goal::Pressure];
        faction.resources.insert("resource.example".into(), 4);
        faction
            .relationships
            .insert(ConceptKey::Pirates.faction_id(), Relationship::default());

        let json: serde_json::Value =
            serde_json::to_value(&faction).expect("a faction state serializes");
        let object = json.as_object().expect("a faction state is an object");
        let keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            BTreeSet::from([
                "resources",
                "strategic_state",
                "eliminated",
                "relationships",
                "current_goals",
            ]),
            "S5 may add current_goals and nothing else; a score field fails here"
        );

        assert_eq!(
            object["current_goals"],
            serde_json::json!(["expand", "pressure"]),
            "goals serialize as words, not as scores"
        );
        assert!(
            object["strategic_state"].is_string(),
            "a board position is a category, never a number"
        );

        // And the goal vocabulary survives a round trip as itself.
        let reloaded: FactionState = serde_json::from_value(json).expect("a faction state reloads");
        assert_eq!(reloaded, faction);
    }
}
