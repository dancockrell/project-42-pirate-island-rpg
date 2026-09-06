//! S6: `StrategicDirective` -- what the player asks a faction to do, and the
//! plain-language explanation the faction gives back *before* the player
//! confirms it. `docs/SHIP_PLAN.md` section 7, card S6; brief sections 5.9 and
//! 19.
//!
//! **Four rules the file is built around.**
//!
//! *A directive is a request, not an order.* Brief section 5.9 puts the
//! faction on automatic and makes it "easy to direct"; card S6 says outright
//! that directives are "high-weight inputs to S5, not commands". So nothing
//! here writes a [`Goal`](crate::strategy::utility::Goal), sets a
//! [`StrategicState`](crate::strategy::faction::StrategicState) or moves a
//! faction. A live directive is one more term in
//! [`score`](crate::strategy::utility) -- a loud one, loud enough to move the
//! leading goal of a faction that has a choice, and deliberately silent for one
//! that does not. See [`crate::strategy::utility::DIRECTIVE_WEIGHT`] and the
//! survival rule beside it.
//!
//! *The explanation comes first.* Brief section 5.9: "Before confirmation, the
//! faction should explain how it understands a major directive in ordinary
//! language, including obvious risks and competing commitments."
//! [`ExpeditionState::explain_directive`] takes a directive that has not been
//! issued, mutates nothing, and returns words. It is a pure function of the
//! save and the graph, so the screen the player confirms from and the record
//! stored afterwards cannot disagree.
//!
//! *No number that came out of the utility model is in the explanation.* Brief
//! section 9's "do not expose raw utility arithmetic" is why
//! [`Explanation`] is six plain fields of `String`, `Vec<String>` and `bool`.
//! The only numbers that reach it are things the player already stated (a
//! resource limit, an acceptable risk) or facts about the map (how many steps
//! away a place is), never a score. `an_explanation_carries_no_score` holds
//! that true.
//!
//! *A directive persists.* Brief section 5.9: "Directions should persist until
//! completed, cancelled, superseded, made impossible, or returned for
//! reconsideration." Those five words are [`DirectiveStatus`]'s five terminal
//! variants, [`ExpeditionState::mark_directive`] is the only thing that can
//! reach them, and no clock, tick or midnight touches a directive. A directive
//! stays in the save after it ends, because "what was asked for and how it
//! finished" is exactly the history a player would ask about; only an
//! [`DirectiveStatus::Active`] one is scored.
//!
//! ## Where the methods live
//!
//! Every method this lane adds to [`ExpeditionState`] is in the `impl` block at
//! the bottom of *this* file rather than in `expedition.rs`. Three lanes are
//! appending to `expedition.rs` this round; S6 makes exactly two edits there
//! (the `directives` field at the end of the field list, and its initialiser at
//! the end of `new()`), which merge with any other lane's tail append. The
//! behaviour then lives next to the types it is about, which is where it reads
//! best anyway.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, ExpeditionState, require_stable_id};
use crate::geography::{Geography, RouteOption};
use crate::strategy::utility::{BoardView, Goal};

/// The prefix every directive ID carries, as `strategy/mod.rs`'s ID rules
/// require a stable ID to be namespaced. The caller supplies the whole ID;
/// this is what it must start with.
pub const DIRECTIVE_ID_PREFIX: &str = "directive.";

/// Faction IDs are concept keys, never proper names (`strategy/mod.rs`).
const FACTION_ID_PREFIX: &str = "faction.";

// ---------------------------------------------------------------------------
// The vocabulary
// ---------------------------------------------------------------------------

/// Brief section 5.9's strategic directive vocabulary, in the brief's own
/// order, and nothing else.
///
/// Ten intention words, because the brief asks for "a small intention-based
/// vocabulary ... over unit micromanagement". This is not
/// [`Goal`](crate::strategy::utility::Goal): a goal is the shape of a
/// faction's hour and has no target, and an intent is a request about a named
/// place. [`INTENT_GOAL_TABLE`](crate::strategy::utility::INTENT_GOAL_TABLE) is
/// the single place the two vocabularies meet.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// Keep this place. The default because it is the least destructive thing
    /// a misconfigured directive could ask for.
    #[default]
    Protect,
    /// Keep this place stocked.
    Supply,
    /// Build here.
    Develop,
    /// Take ground here.
    Expand,
    /// Lean on someone here, short of a committed attack.
    Pressure,
    /// Commit to taking this place from whoever holds it.
    Attack,
    /// Help whoever is holding here.
    Support,
    /// Find out what is here.
    Investigate,
    /// Stay away from here.
    Avoid,
    /// Give this place up.
    Withdraw,
}

impl Intent {
    /// Every intent, in the brief's order. The one place the list exists.
    pub const ALL: [Intent; 10] = [
        Intent::Protect,
        Intent::Supply,
        Intent::Develop,
        Intent::Expand,
        Intent::Pressure,
        Intent::Attack,
        Intent::Support,
        Intent::Investigate,
        Intent::Avoid,
        Intent::Withdraw,
    ];

    /// The word a screen shows. Lowercase and fixed, so the presentation layer
    /// never invents its own spelling of an intent.
    pub fn as_str(self) -> &'static str {
        match self {
            Intent::Protect => "protect",
            Intent::Supply => "supply",
            Intent::Develop => "develop",
            Intent::Expand => "expand",
            Intent::Pressure => "pressure",
            Intent::Attack => "attack",
            Intent::Support => "support",
            Intent::Investigate => "investigate",
            Intent::Avoid => "avoid",
            Intent::Withdraw => "withdraw",
        }
    }
}

/// Where a directive stands. `Active` plus brief section 5.9's five endings,
/// and no sixth: "Directions should persist until completed, cancelled,
/// superseded, made impossible, or returned for reconsideration."
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DirectiveStatus {
    /// Standing. The only status [`BoardView`] reads.
    #[default]
    Active,
    /// The faction did what was asked.
    Completed,
    /// The player took it back.
    Cancelled,
    /// A later directive replaced it -- see
    /// [`ExpeditionState::supersede_directive`].
    Superseded,
    /// The board made it undoable.
    Impossible,
    /// Handed back for reconsideration: the faction will not do this as asked
    /// and is saying so rather than failing quietly.
    Returned,
}

impl DirectiveStatus {
    /// The five endings. A directive in one of these is finished and cannot be
    /// moved again.
    pub const TERMINAL: [DirectiveStatus; 5] = [
        DirectiveStatus::Completed,
        DirectiveStatus::Cancelled,
        DirectiveStatus::Superseded,
        DirectiveStatus::Impossible,
        DirectiveStatus::Returned,
    ];

    pub fn is_terminal(self) -> bool {
        DirectiveStatus::TERMINAL.contains(&self)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DirectiveStatus::Active => "active",
            DirectiveStatus::Completed => "completed",
            DirectiveStatus::Cancelled => "cancelled",
            DirectiveStatus::Superseded => "superseded",
            DirectiveStatus::Impossible => "impossible",
            DirectiveStatus::Returned => "returned",
        }
    }
}

/// How hard the player is leaning on this request.
///
/// Brief section 19 gives `StrategicDirective` a `priority` field and says
/// nothing about its shape, so it is three words rather than a number: a number
/// invites the player to tune a weight, and the whole point of the vocabulary
/// is that it is intentions rather than dials.
///
/// It scales the directive's signal in the scoring
/// ([`Priority::signal_percent`]). `Urgent` is worth two `Standing`s and
/// `Routine` half of one, so an urgent request outweighs a standing request
/// pulling the other way and two equal requests in opposite directions cancel.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// When there is time for it.
    Routine,
    /// The ordinary case.
    #[default]
    Standing,
    /// Ahead of the standing business.
    Urgent,
}

impl Priority {
    /// This priority as a percentage of a standing request. Provisional.
    pub fn signal_percent(self) -> i32 {
        match self {
            Priority::Routine => 50,
            Priority::Standing => 100,
            Priority::Urgent => 200,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Routine => "routine",
            Priority::Standing => "standing",
            Priority::Urgent => "urgent",
        }
    }
}

/// What the faction may do with the reserve. Brief section 5.9 lists "Reserve
/// commitment" among the things the player controls, and section 19 gives the
/// directive a `reserve_policy` field for it.
///
/// Recorded and explained, not yet spent: there is no reserve to commit until
/// S7's `ForceRecord` exists, so this reaches [`Explanation::resources`] and
/// nothing else. **Acceptance criterion for S7:** the lane that lands forces
/// reads this field where it decides what may be committed, and deletes this
/// paragraph.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReservePolicy {
    /// The reserve stays home. The default, because committing a reserve is
    /// the kind of thing a player should have to ask for.
    #[default]
    Hold,
    /// The reserve may go in if the faction judges it needed.
    Commit,
}

impl ReservePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            ReservePolicy::Hold => "hold",
            ReservePolicy::Commit => "commit",
        }
    }
}

// ---------------------------------------------------------------------------
// The directive
// ---------------------------------------------------------------------------

/// Brief section 19's `StrategicDirective`, field for field.
///
/// The brief's list is the specification and every name on it appears below
/// unrenamed, in the brief's order, so the two can be diffed by eye -- the same
/// discipline `FactionDefinition` follows for the same section.
///
/// **One field is not on the brief's list: `faction_id`.** Section 19 names the
/// directive's `target_faction_id` -- who it is *about* -- and no field for who
/// it is *addressed to*, which a flat map of directives keyed by directive ID
/// cannot do without. Adding it was the honest fix; the alternatives were to
/// overload `target_faction_id` with two meanings (it means "whose ground or
/// whose forces this is about" for `Attack`, `Pressure` and `Support`) or to
/// hard-code Captain Michael's faction as the only recipient, and the brief's
/// island has six factions the player may one day speak to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategicDirective {
    /// `directive.<...>`, supplied by the caller and validated by
    /// [`require_stable_id`]. The key in
    /// [`ExpeditionState::directives`] is this ID.
    #[serde(default)]
    pub id: String,
    /// Who this is asked of: `faction.<concept_key>`. Not on brief section
    /// 19's list -- see the note on this struct.
    #[serde(default)]
    pub faction_id: String,
    /// Who asked. A character ID; Captain Michael in every case the vertical
    /// slice can produce, but a companion managing a delegated portfolio
    /// (brief section 5.9) is the reason this is not assumed.
    #[serde(default)]
    pub issuing_character_id: String,
    /// The request itself.
    #[serde(default)]
    pub intent: Intent,
    /// The cell this is about, `world.cell.*`. Validated against the graph at
    /// issue: a directive cannot invent a place by naming one.
    #[serde(default)]
    pub target_node_id: Option<String>,
    /// The region this is about -- the `region_id` on a `LocationRecord`.
    /// Recorded and explained; nothing here resolves a region to a set of
    /// cells, because no lane has a region index yet.
    #[serde(default)]
    pub target_region_id: Option<String>,
    /// The faction this is about: whose ground to take, whom to lean on, whom
    /// to help. Not the recipient -- that is `faction_id`.
    #[serde(default)]
    pub target_faction_id: Option<String>,
    /// How hard the player is leaning.
    #[serde(default)]
    pub priority: Priority,
    /// The most dangerous road the player is content to see used, on
    /// `RouteOption::risk_level`'s scale. Read by
    /// [`ExpeditionState::explain_directive`], which names a road that exceeds
    /// it as a blocker.
    #[serde(default)]
    pub acceptable_risk: u8,
    /// What may be spent, by open resource key (brief section 20 leaves the
    /// category list Open, so this names no categories of its own). Empty
    /// means "no stated limit", which is not the same as "nothing".
    #[serde(default)]
    pub resource_limit: BTreeMap<String, u32>,
    /// What may be done with the reserve.
    #[serde(default)]
    pub reserve_policy: ReservePolicy,
    /// In the player's words, what finishing this looks like. Prose: nothing
    /// evaluates it, and [`ExpeditionState::mark_directive`] is what records
    /// that it happened.
    #[serde(default)]
    pub completion_condition: String,
    /// In the player's words, when to break off. Prose, and the first thing
    /// [`Explanation::withdrawal_conditions`] shows.
    #[serde(default)]
    pub withdrawal_condition: String,
    /// Where this directive stands. Brief section 19 calls the field `state`;
    /// it keeps that name.
    #[serde(default)]
    pub state: DirectiveStatus,
    /// The explanation the player was shown before confirming, kept with the
    /// directive so "what was I told this would mean" survives a save. `None`
    /// on a directive nobody has explained.
    #[serde(default)]
    pub explanation: Option<Explanation>,
    /// Why this cannot proceed, in the same words
    /// [`Explanation::blockers`] uses. Filled from the explanation at issue and
    /// left alone afterwards -- it is what was true when the player agreed, not
    /// a live feed.
    #[serde(default)]
    pub blocking_reasons: Vec<String>,
}

impl StrategicDirective {
    /// A directive with the two IDs and the intent that make it meaningful, and
    /// defaults everywhere else. The shortest honest constructor: a caller that
    /// wants a target, a limit or a condition sets the field.
    pub fn new(id: impl Into<String>, faction_id: impl Into<String>, intent: Intent) -> Self {
        Self {
            id: id.into(),
            faction_id: faction_id.into(),
            intent,
            ..Self::default()
        }
    }

    /// Structural validation, before anything mutates. IDs are checked for
    /// shape only -- whether the cell exists is
    /// [`ExpeditionState::issue_directive`]'s question, because it needs the
    /// graph and this does not.
    pub fn validate(&self) -> Result<(), DirectiveError> {
        require_stable_id("directive.id", &self.id).map_err(DirectiveError::Id)?;
        if !self.id.starts_with(DIRECTIVE_ID_PREFIX) {
            return Err(DirectiveError::Id(ExpeditionError::InvalidStableId {
                field: "directive.id",
                value: self.id.clone(),
            }));
        }
        require_stable_id("directive.faction_id", &self.faction_id).map_err(DirectiveError::Id)?;
        if !self.faction_id.starts_with(FACTION_ID_PREFIX) {
            return Err(DirectiveError::Id(ExpeditionError::InvalidStableId {
                field: "directive.faction_id",
                value: self.faction_id.clone(),
            }));
        }
        if !self.issuing_character_id.is_empty() {
            require_stable_id("directive.issuing_character_id", &self.issuing_character_id)
                .map_err(DirectiveError::Id)?;
        }
        for (field, id) in [
            ("directive.target_node_id", &self.target_node_id),
            ("directive.target_region_id", &self.target_region_id),
            ("directive.target_faction_id", &self.target_faction_id),
        ] {
            if let Some(id) = id {
                require_stable_id(field, id).map_err(DirectiveError::Id)?;
            }
        }
        for key in self.resource_limit.keys() {
            require_stable_id("directive.resource_limit", key).map_err(DirectiveError::Id)?;
        }
        Ok(())
    }

    /// Everything this directive permits to be spent, summed over whatever
    /// keys it carries.
    ///
    /// Summing across categories rather than weighting them is the only honest
    /// thing available: resource categories are Open (brief section 20), so
    /// there is no authored list of kinds to weight and naming any here would
    /// mint one. The same reading `BoardView::stock_percent` takes of a
    /// stockpile, so the two sides of the supply blocker are measured on one
    /// scale.
    pub fn resource_limit_total(&self) -> u64 {
        self.resource_limit
            .values()
            .map(|amount| u64::from(*amount))
            .sum()
    }
}

/// What went wrong with a directive.
///
/// A type of its own rather than variants on
/// [`ExpeditionError`]: three lanes are editing `expedition.rs` this round and
/// its error enum is a merge point, and every one of these failures is about a
/// directive rather than about a campaign. The one shape that is genuinely
/// shared -- "that is not a stable ID" -- is not restated here; it is carried
/// through [`DirectiveError::Id`], so there is exactly one ID rule in the
/// crate. **Acceptance criterion for folding this in:** when `expedition.rs` is
/// quiet, these become variants of `ExpeditionError` and this enum becomes a
/// deprecated alias for one release, or is deleted outright with its callers
/// moved -- not copied.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectiveError {
    /// A malformed stable ID, as the crate's one ID rule sees it.
    Id(ExpeditionError),
    /// The directive is addressed to a faction this campaign does not carry.
    /// Refused rather than created: `factions` is a map, and a faction is
    /// authored content -- a stray directive does not get to found one.
    UnknownFaction { faction_id: String },
    /// The directive names a cell the graph has never heard of. The same rule
    /// and the same reason as `ExpeditionState::set_control`'s.
    UnknownCell { cell_id: String },
    /// This faction already has a standing directive with the same intent and
    /// the same target. Superseding is deliberate: see
    /// [`ExpeditionState::supersede_directive`].
    AlreadyStanding {
        directive_id: String,
        existing_id: String,
    },
    /// Two live directives cannot share an ID.
    DuplicateDirective { directive_id: String },
    /// No directive by that ID.
    UnknownDirective { directive_id: String },
    /// That directive has already ended. The five endings are endings.
    AlreadyEnded {
        directive_id: String,
        state: DirectiveStatus,
    },
    /// [`ExpeditionState::mark_directive`] moves a directive to one of the five
    /// endings. It is not a way to reopen one.
    NotAnEnding { state: DirectiveStatus },
}

// ---------------------------------------------------------------------------
// The explanation
// ---------------------------------------------------------------------------

/// How the faction says it understands a directive, in ordinary language,
/// before the player confirms it.
///
/// Card S6's six fields exactly. Every one is words: brief section 9 forbids
/// exposing raw utility arithmetic as the normal interface, and this *is* the
/// normal interface -- it is the screen a player reads before agreeing to
/// something. The only numbers that appear inside these strings are ones the
/// player themselves stated (an acceptable risk, a resource limit) or plain
/// facts about the map (a step count, a road's authored danger). No score, no
/// weight, no affinity and no goal ordering crosses.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    /// What this turns into, in the faction's own terms: the abstract goal the
    /// intent argues for, said as a sentence rather than as a `Goal`.
    #[serde(default)]
    pub goal: String,
    /// Why this place -- what the faction can see about the target.
    #[serde(default)]
    pub why_target: String,
    /// What it would spend, and out of what.
    #[serde(default)]
    pub resources: String,
    /// What stands in the way. Empty when nothing does. Brief section 5.9's
    /// "obvious risks and competing commitments".
    #[serde(default)]
    pub blockers: Vec<String>,
    /// When the faction would break this off.
    #[serde(default)]
    pub withdrawal_conditions: String,
    /// Whether Captain Michael's party could reach the target itself. The one
    /// place the character scale and the strategic scale meet on this screen.
    #[serde(default)]
    pub party_could_help: bool,
}

/// How far from where the party stands a target still counts as somewhere the
/// party could go and matter. Three steps: far enough to include the whole of
/// the vertical slice's above-ground map from any of its cells, near enough
/// that "the party could help with this" is not a claim about the far side of
/// an island. Provisional.
pub const PARTY_COULD_HELP_STEPS: usize = 3;

/// The stockpile, summed over whatever open keys exist, that a committed attack
/// wants behind it when the directive states no limit of its own.
///
/// Provisional and on the same scale as `BoardView`'s `WELL_STOCKED`: half of a
/// full stockpile, because an attack that would leave a faction with nothing is
/// the case card S6's Done-when is about. When the directive *does* state a
/// `resource_limit`, that total is what the attack is measured against instead
/// -- the player has said what this is worth, and the faction checks it can
/// afford what was authorised rather than an invented figure.
pub const ATTACK_SUPPLY_FLOOR: u64 = 50;

// ---------------------------------------------------------------------------
// What a directive does to a faction's hour
// ---------------------------------------------------------------------------

impl ExpeditionState {
    /// Explain a directive in ordinary language, **before** it is issued.
    ///
    /// Pure: it reads the save and the graph, mutates nothing, and returns the
    /// same words for the same board on every machine. That is what makes it
    /// usable as a confirmation screen -- the player is shown the explanation,
    /// agrees to it, and [`ExpeditionState::issue_directive`] stores the very
    /// same one alongside the directive.
    ///
    /// It explains a directive that is structurally nonsense too, because a
    /// player who typed a place that does not exist deserves to be told so on
    /// the screen where they can fix it rather than by a refusal afterwards.
    /// The unknown place becomes a blocker; `issue_directive` still refuses.
    pub fn explain_directive(
        &self,
        directive: &StrategicDirective,
        geography: &Geography,
    ) -> Explanation {
        let view = BoardView::of(&directive.faction_id, self, geography);
        let mut blockers: Vec<String> = Vec::new();

        // Where the target is, as the graph knows it.
        let target = directive
            .target_node_id
            .as_deref()
            .filter(|cell_id| geography.location(cell_id).is_some());
        if let Some(named) = &directive.target_node_id
            && target.is_none()
        {
            blockers.push(format!(
                "the faction knows no place called {named}, so it cannot start"
            ));
        }

        let why_target = self.describe_target(directive, target, &view, geography);

        // The road. A contested road between the faction's nearest holding and
        // the target is the risk the player most needs to see before agreeing,
        // and `Geography::effective_risk` is the crate's one answer to how
        // dangerous a road is right now.
        for route in self.route_to_target(&view, target, geography) {
            let effective = geography.effective_risk(route, &self.ownership);
            if effective > route.risk_level {
                blockers.push(format!(
                    "the road {} is contested, which makes it more dangerous than it was built to be",
                    route.id
                ));
            } else if effective > directive.acceptable_risk {
                blockers.push(format!(
                    "the road {} runs at danger {} and this directive accepts {}",
                    route.id, effective, directive.acceptable_risk
                ));
            }
        }

        // Supply. Card S6's Done-when: an attack whose stockpile does not cover
        // what the directive authorises is blocked, and says so.
        let stock: u64 = view
            .resources
            .values()
            .map(|amount| u64::from(*amount))
            .sum();
        let wanted = if directive.resource_limit.is_empty() {
            ATTACK_SUPPLY_FLOOR
        } else {
            directive.resource_limit_total()
        };
        if directive.intent == Intent::Attack && stock < wanted {
            blockers.push(format!(
                "supply is short: an attack of this size wants {wanted} in stores over the \
                 categories this directive names and the faction has {stock}"
            ));
        }

        // Competing commitments -- brief section 5.9 asks for these by name.
        for standing in view.directives.values() {
            if standing.intent.opposes(directive.intent) {
                blockers.push(format!(
                    "it already has a standing directive to {} the same ground ({})",
                    standing.intent.as_str(),
                    standing.id
                ));
            }
        }

        Explanation {
            goal: directive.intent.explained_goal().to_owned(),
            why_target,
            resources: describe_resources(directive, stock),
            blockers,
            withdrawal_conditions: describe_withdrawal(directive),
            party_could_help: self.party_could_help(target, geography),
        }
    }

    /// Issue a directive. Refuses before anything mutates.
    ///
    /// Four refusals, in the order a player would meet them: a malformed ID, a
    /// faction this campaign does not carry, a place the graph has never heard
    /// of, and a standing directive that already says this. The last is why
    /// there is no "replace" flag here -- see
    /// [`ExpeditionState::supersede_directive`].
    ///
    /// The stored directive carries the explanation this call computes, so what
    /// the player was shown and what the save holds are the same words by
    /// construction rather than by discipline.
    pub fn issue_directive(
        &mut self,
        directive: StrategicDirective,
        geography: &Geography,
    ) -> Result<Explanation, DirectiveError> {
        let mut directive = directive;
        directive.validate()?;
        if !self.factions.contains_key(&directive.faction_id) {
            return Err(DirectiveError::UnknownFaction {
                faction_id: directive.faction_id.clone(),
            });
        }
        if let Some(cell_id) = &directive.target_node_id
            && geography.location(cell_id).is_none()
        {
            return Err(DirectiveError::UnknownCell {
                cell_id: cell_id.clone(),
            });
        }
        if self.directives.contains_key(&directive.id) {
            return Err(DirectiveError::DuplicateDirective {
                directive_id: directive.id.clone(),
            });
        }
        if let Some(existing) = self.standing_directive_matching(&directive) {
            return Err(DirectiveError::AlreadyStanding {
                directive_id: directive.id.clone(),
                existing_id: existing,
            });
        }

        let explanation = self.explain_directive(&directive, geography);
        directive.state = DirectiveStatus::Active;
        directive.blocking_reasons = explanation.blockers.clone();
        directive.explanation = Some(explanation.clone());
        self.directives.insert(directive.id.clone(), directive);
        Ok(explanation)
    }

    /// Replace a standing directive with a new one, in one call.
    ///
    /// Superseding is explicit rather than a flag on
    /// [`ExpeditionState::issue_directive`] because it is a different act: it
    /// names the directive being withdrawn, so the player cannot silently
    /// overwrite a request they had forgotten was standing, and the save keeps
    /// both -- the old one as [`DirectiveStatus::Superseded`], which is one of
    /// brief section 5.9's five endings and the only one nothing else can
    /// produce.
    ///
    /// All or nothing: if the replacement is refused, the existing directive is
    /// still standing and untouched.
    pub fn supersede_directive(
        &mut self,
        existing_id: &str,
        replacement: StrategicDirective,
        geography: &Geography,
    ) -> Result<Explanation, DirectiveError> {
        match self.directives.get(existing_id) {
            None => {
                return Err(DirectiveError::UnknownDirective {
                    directive_id: existing_id.to_owned(),
                });
            }
            Some(existing) if existing.state.is_terminal() => {
                return Err(DirectiveError::AlreadyEnded {
                    directive_id: existing_id.to_owned(),
                    state: existing.state,
                });
            }
            Some(_) => {}
        }
        // Take the old one out of the way first so the replacement is not
        // refused for colliding with the thing it replaces, and put it back
        // exactly as it was if the replacement turns out to be illegal.
        let previous = self
            .directives
            .remove(existing_id)
            .expect("checked present immediately above");
        match self.issue_directive(replacement, geography) {
            Ok(explanation) => {
                let mut previous = previous;
                previous.state = DirectiveStatus::Superseded;
                self.directives.insert(previous.id.clone(), previous);
                Ok(explanation)
            }
            Err(error) => {
                self.directives.insert(previous.id.clone(), previous);
                Err(error)
            }
        }
    }

    /// Move a directive to one of brief section 5.9's five endings. The one
    /// mutator for a directive's status.
    ///
    /// There is no counterpart that reopens one and no other writer of
    /// [`StrategicDirective::state`] anywhere in the crate: an ending is an
    /// ending, and a player who changes their mind issues a new directive
    /// rather than resurrecting an old one. A directive already ended is
    /// refused rather than re-ended, so "when did this finish, and how" has one
    /// answer.
    pub fn mark_directive(
        &mut self,
        directive_id: &str,
        status: DirectiveStatus,
    ) -> Result<(), DirectiveError> {
        if !status.is_terminal() {
            return Err(DirectiveError::NotAnEnding { state: status });
        }
        let directive = self.directives.get_mut(directive_id).ok_or_else(|| {
            DirectiveError::UnknownDirective {
                directive_id: directive_id.to_owned(),
            }
        })?;
        if directive.state.is_terminal() {
            return Err(DirectiveError::AlreadyEnded {
                directive_id: directive_id.to_owned(),
                state: directive.state,
            });
        }
        directive.state = status;
        Ok(())
    }

    /// The player takes a directive back. Card S6 names cancellation, so it has
    /// a name; it is [`ExpeditionState::mark_directive`] and nothing else, so
    /// there is still exactly one writer.
    pub fn cancel_directive(&mut self, directive_id: &str) -> Result<(), DirectiveError> {
        self.mark_directive(directive_id, DirectiveStatus::Cancelled)
    }

    /// The standing directives a faction is carrying, oldest ID first.
    /// Terminal ones are kept in the save as history and are not returned here.
    pub fn active_directives(&self, faction_id: &str) -> Vec<&StrategicDirective> {
        self.directives
            .values()
            .filter(|directive| {
                directive.faction_id == faction_id && directive.state == DirectiveStatus::Active
            })
            .collect()
    }

    /// The ID of a standing directive that already says what this one says --
    /// same faction, same intent, same target.
    fn standing_directive_matching(&self, directive: &StrategicDirective) -> Option<String> {
        self.directives
            .values()
            .find(|standing| {
                standing.state == DirectiveStatus::Active
                    && standing.faction_id == directive.faction_id
                    && standing.intent == directive.intent
                    && standing.target_node_id == directive.target_node_id
                    && standing.target_region_id == directive.target_region_id
                    && standing.target_faction_id == directive.target_faction_id
            })
            .map(|standing| standing.id.clone())
    }

    /// Why this place, in the faction's own terms: whether it already holds it,
    /// whether it borders it, and who else is there.
    fn describe_target(
        &self,
        directive: &StrategicDirective,
        target: Option<&str>,
        view: &BoardView,
        geography: &Geography,
    ) -> String {
        let Some(cell_id) = target else {
            return match (&directive.target_region_id, &directive.target_faction_id) {
                (Some(region_id), _) => {
                    format!("no single place is named; this is about everywhere in {region_id}")
                }
                (None, Some(_)) => {
                    "no place is named; this is about another faction wherever it is found"
                        .to_owned()
                }
                (None, None) => {
                    "no place is named, so the faction will read this as standing policy".to_owned()
                }
            };
        };
        let name = geography
            .location(cell_id)
            .map(|record| record.display_name.clone())
            .unwrap_or_else(|| cell_id.to_owned());
        if view.held.contains(cell_id) {
            format!("{name} is ground the faction already holds")
        } else if let Some(_holder) = view.contested_frontier.get(cell_id) {
            format!("{name} is on the border, and another faction holds it")
        } else if view.open_frontier.contains(cell_id) {
            format!("{name} borders the faction's ground and nobody holds it")
        } else if self.ownership.contains_key(cell_id) || geography.controller(cell_id).is_some() {
            format!("{name} is held by another faction and does not touch the faction's ground")
        } else {
            format!("{name} is unheld and does not touch the faction's ground")
        }
    }

    /// The roads between the faction's nearest holding and the target, in the
    /// order they would be walked.
    ///
    /// Walked with [`Geography::next_step_toward`], which is the crate's one
    /// pathfinder, rather than a second search of its own. The party's
    /// `discoveries` are what open the gates: a faction's border does not
    /// depend on what Captain Michael has found (`BoardView::of` says so), but
    /// an explanation shown to the player should not name a road the player has
    /// never heard of.
    ///
    /// Empty when the faction holds the target, holds nothing, or cannot reach
    /// it at all -- unreachability is reported by
    /// [`Self::describe_target`] rather than as a road.
    fn route_to_target<'g>(
        &self,
        view: &BoardView,
        target: Option<&str>,
        geography: &'g Geography,
    ) -> Vec<&'g RouteOption> {
        let Some(target) = target else {
            return Vec::new();
        };
        if view.held.contains(target) {
            return Vec::new();
        }
        // The nearest holding, with the ID as the tie-break so the answer does
        // not depend on iteration luck. `held` is a BTreeSet, so it already
        // iterates in ID order.
        let Some(from) = view
            .held
            .iter()
            .filter_map(|cell| {
                geography
                    .step_distance(cell, target, &self.discoveries)
                    .map(|steps| (steps, cell))
            })
            .min()
            .map(|(_, cell)| cell.clone())
        else {
            return Vec::new();
        };

        let mut walked: Vec<&RouteOption> = Vec::new();
        let mut here = from;
        while here != target {
            let Some(step) = geography.next_step_toward(&here, target, &self.discoveries) else {
                break;
            };
            walked.push(step);
            here = step.to_location_id.clone();
            // The graph is finite and `next_step_toward` takes a shortest path,
            // so this terminates; the guard is against a future graph that can
            // return a step to a place it came from.
            if walked.len() > geography.locations.len() {
                break;
            }
        }
        walked
    }

    /// Could the party get there and matter? Reachable from where it stands,
    /// through the gates it has opened, within [`PARTY_COULD_HELP_STEPS`].
    fn party_could_help(&self, target: Option<&str>, geography: &Geography) -> bool {
        let Some(target) = target else {
            return false;
        };
        geography
            .step_distance(&self.active_location_id, target, &self.discoveries)
            .is_some_and(|steps| steps <= PARTY_COULD_HELP_STEPS)
    }
}

impl Intent {
    /// What this intent argues for, said as a sentence a screen can show.
    ///
    /// The words match [`INTENT_GOAL_TABLE`](crate::strategy::utility::INTENT_GOAL_TABLE)'s
    /// row for this intent, which is the table the scoring actually reads -- so
    /// the explanation describes what will happen rather than what was
    /// intended to happen. `an_explanation_describes_the_table_that_scores_it`
    /// pins the two together.
    pub fn explained_goal(self) -> &'static str {
        match self {
            Intent::Protect => "make what it already holds defensible",
            Intent::Supply => "build and produce rather than reach for more ground",
            Intent::Develop => "build and produce rather than reach for more ground",
            Intent::Expand => "take unheld ground",
            Intent::Pressure => "push on a rival's ground, short of a committed attack",
            Intent::Attack => "push on a rival's ground and take it",
            Intent::Support => "hold the line and produce for whoever is holding it",
            Intent::Investigate => {
                "nothing yet: the faction has no way to spend an hour on finding out, so it \
                 will keep this in mind and carry on as it was"
            }
            Intent::Avoid => "leave that ground alone -- neither take it nor push on it",
            Intent::Withdraw => "give that ground up rather than spend anything holding it",
        }
    }

    /// Whether a standing directive with this intent argues against a new one
    /// with `other`'s intent over the same ground. Used only to name a
    /// competing commitment in an explanation; it refuses nothing.
    fn opposes(self, other: Intent) -> bool {
        use crate::strategy::utility::intent_goal_row;
        let mine = intent_goal_row(self);
        let theirs = intent_goal_row(other);
        Goal::ALL
            .iter()
            .enumerate()
            .any(|(index, _)| mine[index].signum() * theirs[index].signum() < 0)
    }
}

/// What the faction says it would spend. The player's own numbers, read back.
fn describe_resources(directive: &StrategicDirective, stock: u64) -> String {
    let reserve = match directive.reserve_policy {
        ReservePolicy::Hold => "the reserve stays where it is",
        ReservePolicy::Commit => "the reserve may go in if the faction judges it needed",
    };
    if directive.resource_limit.is_empty() {
        return format!(
            "no limit was set, so the faction will spend out of its stores ({stock} in all) as \
             it sees fit; {reserve}"
        );
    }
    let named: Vec<String> = directive
        .resource_limit
        .iter()
        .map(|(key, amount)| format!("{amount} of {key}"))
        .collect();
    format!(
        "up to {} out of stores of {stock} in all; {reserve}",
        named.join(", ")
    )
}

/// When the faction would break this off.
fn describe_withdrawal(directive: &StrategicDirective) -> String {
    let stated = directive.withdrawal_condition.trim();
    let risk = format!(
        "it breaks off if the road it must use runs more dangerous than {}",
        directive.acceptable_risk
    );
    if stated.is_empty() {
        format!("{risk}; nothing else was stated, so the faction will use its own judgement")
    } else {
        format!("{stated}; and {risk}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::faction::{ConceptKey, FactionState};
    use crate::strategy::utility::intent_goal_row;

    const BEACH: &str = "world.cell.black_beach";
    const ESTATE: &str = "world.cell.damaged_estate";
    const LANDING: &str = "world.cell.river_landing";

    /// A campaign with two factions in it and nothing claimed.
    fn a_campaign() -> (ExpeditionState, Geography) {
        let geography = Geography::black_beach_vertical_slice();
        let mut state =
            ExpeditionState::new(42, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        state
            .factions
            .insert(ConceptKey::Michael.faction_id(), FactionState::new());
        state
            .factions
            .insert(ConceptKey::Pirates.faction_id(), FactionState::new());
        (state, geography)
    }

    fn an_attack_on(cell_id: &str) -> StrategicDirective {
        StrategicDirective {
            issuing_character_id: "character.protagonist.captain".into(),
            target_node_id: Some(cell_id.to_owned()),
            acceptable_risk: 9,
            ..StrategicDirective::new(
                "directive.take_the_estate",
                ConceptKey::Michael.faction_id(),
                Intent::Attack,
            )
        }
    }

    // -----------------------------------------------------------------------
    // The explanation, before confirmation
    // -----------------------------------------------------------------------

    /// **Card S6's Done-when.** An attack the faction cannot supply is told so
    /// in words, on the screen the player confirms from, before anything is
    /// issued.
    #[test]
    fn an_attack_that_supply_will_not_cover_names_a_blocker_before_confirmation() {
        let (mut state, geography) = a_campaign();
        state
            .set_control(BEACH, Some(ConceptKey::Michael.faction_id()), &geography)
            .expect("the beach exists");
        state
            .set_control(ESTATE, Some(ConceptKey::Pirates.faction_id()), &geography)
            .expect("the estate exists");

        // Empty stores, and an attack that authorises a spend they cannot cover.
        let mut attack = an_attack_on(ESTATE);
        attack.resource_limit.insert("resource.powder".into(), 7);

        let before = state.clone();
        let explanation = state.explain_directive(&attack, &geography);
        assert_eq!(state, before, "explaining is not a mutation");

        assert!(
            explanation
                .blockers
                .iter()
                .any(|blocker| blocker.contains("supply is short")),
            "an attack with nothing behind it must say so: {:?}",
            explanation.blockers
        );

        // Stock it, and the blocker goes away -- so the blocker is about supply
        // rather than about attacks.
        state
            .factions
            .get_mut(&ConceptKey::Michael.faction_id())
            .expect("the faction is in the campaign")
            .resources
            .insert("resource.powder".into(), 40);
        let stocked = state.explain_directive(&attack, &geography);
        assert!(
            !stocked
                .blockers
                .iter()
                .any(|blocker| blocker.contains("supply is short")),
            "supply covers it now: {:?}",
            stocked.blockers
        );
    }

    /// The other risk the player is entitled to see: the road there is
    /// contested, on `Geography::effective_risk`'s reading and no other.
    #[test]
    fn a_contested_road_to_the_target_is_named() {
        let (mut state, geography) = a_campaign();
        state
            .set_control(BEACH, Some(ConceptKey::Michael.faction_id()), &geography)
            .expect("the beach exists");
        let directive = StrategicDirective {
            target_node_id: Some(ESTATE.to_owned()),
            acceptable_risk: 9,
            ..StrategicDirective::new(
                "directive.hold_the_estate",
                ConceptKey::Michael.faction_id(),
                Intent::Protect,
            )
        };

        // Both ends held by the same faction: the road is not contested.
        state
            .set_control(ESTATE, Some(ConceptKey::Michael.faction_id()), &geography)
            .expect("the estate exists");
        assert!(
            state
                .explain_directive(&directive, &geography)
                .blockers
                .is_empty(),
            "the faction holds both ends of that road"
        );

        // A rival takes the far end and the same road becomes a blocker.
        state
            .set_control(ESTATE, Some(ConceptKey::Pirates.faction_id()), &geography)
            .expect("the estate exists");
        let blockers = state.explain_directive(&directive, &geography).blockers;
        assert!(
            blockers.iter().any(|blocker| blocker.contains("contested")
                && blocker.contains("world.portal.black_beach_to_damaged_estate")),
            "the contested road must be named: {blockers:?}"
        );
    }

    /// Brief section 5.9's "competing commitments": a standing directive that
    /// argues the other way is named before the player confirms a new one.
    #[test]
    fn a_competing_standing_directive_is_named() {
        let (mut state, geography) = a_campaign();
        state
            .issue_directive(
                StrategicDirective {
                    target_node_id: Some(LANDING.to_owned()),
                    ..StrategicDirective::new(
                        "directive.stay_off_the_landing",
                        ConceptKey::Michael.faction_id(),
                        Intent::Avoid,
                    )
                },
                &geography,
            )
            .expect("a first directive is legal");

        let expand = StrategicDirective {
            target_node_id: Some(LANDING.to_owned()),
            ..StrategicDirective::new(
                "directive.take_the_landing",
                ConceptKey::Michael.faction_id(),
                Intent::Expand,
            )
        };
        let blockers = state.explain_directive(&expand, &geography).blockers;
        assert!(
            blockers
                .iter()
                .any(|blocker| blocker.contains("directive.stay_off_the_landing")),
            "the standing directive that argues the other way must be named: {blockers:?}"
        );
    }

    /// Brief section 9: raw utility arithmetic is not the interface. Every
    /// number that reaches the player's screen is one the player stated or one
    /// the map states -- never a score, a weight or an affinity.
    #[test]
    fn an_explanation_carries_no_score() {
        let (mut state, geography) = a_campaign();
        state
            .set_control(BEACH, Some(ConceptKey::Michael.faction_id()), &geography)
            .expect("the beach exists");
        state
            .factions
            .get_mut(&ConceptKey::Michael.faction_id())
            .expect("the faction is in the campaign")
            .resources
            .insert("resource.powder".into(), 3);

        let mut attack = an_attack_on(ESTATE);
        attack.resource_limit.clear();
        attack.resource_limit.insert("resource.powder".into(), 7);
        let explanation = state.explain_directive(&attack, &geography);

        // 9 is the acceptable risk the player set, 7 the limit they authorised,
        // 3 the stores the faction is standing on. Nothing else may appear.
        let allowed = ["9", "7", "3"];
        let mut fields = vec![
            explanation.goal.clone(),
            explanation.why_target.clone(),
            explanation.resources.clone(),
            explanation.withdrawal_conditions.clone(),
        ];
        fields.extend(explanation.blockers.iter().cloned());
        for field in fields {
            for run in field
                .split(|c: char| !c.is_ascii_digit())
                .filter(|run| !run.is_empty())
            {
                assert!(
                    allowed.contains(&run),
                    "{run:?} is a number the player never stated, in {field:?}"
                );
            }
        }
    }

    /// The explanation describes the table that actually scores the directive,
    /// so the player is told what will happen rather than what was meant.
    #[test]
    fn an_explanation_describes_the_table_that_scores_it() {
        for intent in Intent::ALL {
            let words = intent.explained_goal();
            assert!(!words.is_empty(), "{intent:?} explains itself as nothing");
            let moves_nothing = intent_goal_row(intent).iter().all(|pull| *pull == 0);
            assert_eq!(
                moves_nothing,
                words.starts_with("nothing yet"),
                "{intent:?}: the table and the explanation disagree about whether it does anything"
            );
        }
    }

    /// The party's half of the screen: can Captain Michael get there himself?
    #[test]
    fn the_explanation_says_whether_the_party_could_help() {
        let (state, geography) = a_campaign();
        let near = StrategicDirective {
            target_node_id: Some(ESTATE.to_owned()),
            ..StrategicDirective::new(
                "directive.near",
                ConceptKey::Michael.faction_id(),
                Intent::Protect,
            )
        };
        assert!(state.explain_directive(&near, &geography).party_could_help);

        let unreachable = StrategicDirective {
            target_node_id: Some("world.cell.tomb_archive_core".to_owned()),
            ..StrategicDirective::new(
                "directive.far",
                ConceptKey::Michael.faction_id(),
                Intent::Investigate,
            )
        };
        assert!(
            !state
                .explain_directive(&unreachable, &geography)
                .party_could_help,
            "the tomb is behind gates the party has not opened"
        );
    }

    // -----------------------------------------------------------------------
    // Issuing, and the five endings
    // -----------------------------------------------------------------------

    /// Every refusal happens before anything is written.
    #[test]
    fn issuing_refuses_before_it_mutates() {
        let (mut state, geography) = a_campaign();
        let before = state.clone();

        let malformed = StrategicDirective::new(
            "Directive.Shouting",
            ConceptKey::Michael.faction_id(),
            Intent::Protect,
        );
        assert!(matches!(
            state.issue_directive(malformed, &geography),
            Err(DirectiveError::Id(_))
        ));

        let wrong_prefix = StrategicDirective::new(
            "order.take_it",
            ConceptKey::Michael.faction_id(),
            Intent::Protect,
        );
        assert!(matches!(
            state.issue_directive(wrong_prefix, &geography),
            Err(DirectiveError::Id(_))
        ));

        let stranger = StrategicDirective::new(
            "directive.to_nobody",
            "faction.not_in_this_campaign",
            Intent::Protect,
        );
        assert_eq!(
            state.issue_directive(stranger, &geography),
            Err(DirectiveError::UnknownFaction {
                faction_id: "faction.not_in_this_campaign".into()
            })
        );

        let nowhere = StrategicDirective {
            target_node_id: Some("world.cell.invented_by_a_typo".into()),
            ..StrategicDirective::new(
                "directive.to_nowhere",
                ConceptKey::Michael.faction_id(),
                Intent::Protect,
            )
        };
        assert_eq!(
            state.issue_directive(nowhere, &geography),
            Err(DirectiveError::UnknownCell {
                cell_id: "world.cell.invented_by_a_typo".into()
            })
        );

        assert_eq!(state, before, "a refusal writes nothing");
    }

    /// Saying the same thing twice is refused; superseding is the deliberate
    /// way to change your mind.
    #[test]
    fn a_second_directive_saying_the_same_thing_is_refused() {
        let (mut state, geography) = a_campaign();
        let first = StrategicDirective {
            target_node_id: Some(ESTATE.to_owned()),
            ..StrategicDirective::new(
                "directive.hold_it",
                ConceptKey::Michael.faction_id(),
                Intent::Protect,
            )
        };
        state
            .issue_directive(first.clone(), &geography)
            .expect("the first is legal");

        let again = StrategicDirective {
            id: "directive.hold_it_again".into(),
            ..first.clone()
        };
        assert_eq!(
            state.issue_directive(again, &geography),
            Err(DirectiveError::AlreadyStanding {
                directive_id: "directive.hold_it_again".into(),
                existing_id: "directive.hold_it".into(),
            })
        );
        assert_eq!(
            state.issue_directive(first, &geography),
            Err(DirectiveError::DuplicateDirective {
                directive_id: "directive.hold_it".into()
            }),
            "an ID is a key, and two live directives cannot share one"
        );

        // The same intent against a different place is a different request.
        let elsewhere = StrategicDirective {
            id: "directive.hold_the_landing".into(),
            target_node_id: Some(LANDING.to_owned()),
            ..StrategicDirective::new("unused", ConceptKey::Michael.faction_id(), Intent::Protect)
        };
        assert!(state.issue_directive(elsewhere, &geography).is_ok());
    }

    /// Brief section 5.9's persistence rule: a directive stands until one of
    /// exactly five endings, and an ending is an ending.
    #[test]
    fn a_directive_persists_until_one_of_five_endings() {
        assert_eq!(DirectiveStatus::TERMINAL.len(), 5);
        for ending in DirectiveStatus::TERMINAL {
            let (mut state, geography) = a_campaign();
            state
                .issue_directive(
                    StrategicDirective::new(
                        "directive.standing",
                        ConceptKey::Michael.faction_id(),
                        Intent::Develop,
                    ),
                    &geography,
                )
                .expect("legal");
            assert_eq!(
                state
                    .active_directives(&ConceptKey::Michael.faction_id())
                    .len(),
                1
            );

            state
                .mark_directive("directive.standing", ending)
                .expect("an ending is reachable");
            assert_eq!(
                state.directives["directive.standing"].state, ending,
                "the ending is recorded"
            );
            assert!(
                state
                    .active_directives(&ConceptKey::Michael.faction_id())
                    .is_empty(),
                "an ended directive asks for nothing"
            );
            assert!(
                state.directives.contains_key("directive.standing"),
                "it is kept as the record of what was asked and how it finished"
            );
            assert_eq!(
                state.mark_directive("directive.standing", DirectiveStatus::Completed),
                Err(DirectiveError::AlreadyEnded {
                    directive_id: "directive.standing".into(),
                    state: ending,
                }),
                "an ending cannot be re-ended"
            );
        }
    }

    /// `mark_directive` is not a way to reopen a directive, and `cancel` is
    /// `mark` rather than a second writer.
    #[test]
    fn there_is_one_writer_of_a_directives_state() {
        let (mut state, geography) = a_campaign();
        state
            .issue_directive(
                StrategicDirective::new(
                    "directive.standing",
                    ConceptKey::Michael.faction_id(),
                    Intent::Develop,
                ),
                &geography,
            )
            .expect("legal");
        assert_eq!(
            state.mark_directive("directive.standing", DirectiveStatus::Active),
            Err(DirectiveError::NotAnEnding {
                state: DirectiveStatus::Active
            })
        );
        assert_eq!(
            state.mark_directive("directive.no_such_thing", DirectiveStatus::Completed),
            Err(DirectiveError::UnknownDirective {
                directive_id: "directive.no_such_thing".into()
            })
        );
        state.cancel_directive("directive.standing").expect("legal");
        assert_eq!(
            state.directives["directive.standing"].state,
            DirectiveStatus::Cancelled
        );
    }

    /// Superseding names what it replaces, keeps it, and is all-or-nothing.
    #[test]
    fn superseding_is_explicit_and_all_or_nothing() {
        let (mut state, geography) = a_campaign();
        let standing = StrategicDirective {
            target_node_id: Some(ESTATE.to_owned()),
            ..StrategicDirective::new(
                "directive.hold_it",
                ConceptKey::Michael.faction_id(),
                Intent::Protect,
            )
        };
        state
            .issue_directive(standing.clone(), &geography)
            .expect("legal");

        // A refused replacement leaves the standing directive exactly as it was.
        let before = state.clone();
        let illegal = StrategicDirective {
            id: "directive.nowhere".into(),
            target_node_id: Some("world.cell.invented_by_a_typo".into()),
            ..standing.clone()
        };
        assert_eq!(
            state.supersede_directive("directive.hold_it", illegal, &geography),
            Err(DirectiveError::UnknownCell {
                cell_id: "world.cell.invented_by_a_typo".into()
            })
        );
        assert_eq!(state, before, "a refused replacement changes nothing");

        // The legal replacement -- the same intent and target, which
        // `issue_directive` alone would refuse.
        let replacement = StrategicDirective {
            id: "directive.hold_it_harder".into(),
            priority: Priority::Urgent,
            ..standing
        };
        state
            .supersede_directive("directive.hold_it", replacement, &geography)
            .expect("superseding is how you say it differently");
        assert_eq!(
            state.directives["directive.hold_it"].state,
            DirectiveStatus::Superseded
        );
        assert_eq!(
            state.directives["directive.hold_it_harder"].state,
            DirectiveStatus::Active
        );
        assert_eq!(
            state.supersede_directive(
                "directive.hold_it",
                StrategicDirective::new(
                    "directive.third",
                    ConceptKey::Michael.faction_id(),
                    Intent::Protect,
                ),
                &geography,
            ),
            Err(DirectiveError::AlreadyEnded {
                directive_id: "directive.hold_it".into(),
                state: DirectiveStatus::Superseded,
            }),
            "an ended directive cannot be superseded twice"
        );
    }

    /// The explanation the player agreed to is the one the save keeps.
    #[test]
    fn the_stored_directive_carries_the_explanation_the_player_saw() {
        let (mut state, geography) = a_campaign();
        let attack = an_attack_on(ESTATE);
        let shown = state.explain_directive(&attack, &geography);
        let stored = state.issue_directive(attack, &geography).expect("legal");
        assert_eq!(shown, stored);
        let kept = &state.directives["directive.take_the_estate"];
        assert_eq!(kept.explanation.as_ref(), Some(&shown));
        assert_eq!(kept.blocking_reasons, shown.blockers);
    }

    /// Directives are saved and reloaded like every other part of the state,
    /// and a save written before they existed loads with none.
    #[test]
    fn directives_survive_a_save_round_trip() {
        let (mut state, geography) = a_campaign();
        let mut directive = an_attack_on(ESTATE);
        directive.resource_limit.insert("resource.powder".into(), 7);
        directive.priority = Priority::Urgent;
        directive.reserve_policy = ReservePolicy::Commit;
        directive.withdrawal_condition = "break off if the tide turns".into();
        state.issue_directive(directive, &geography).expect("legal");

        let json = state.to_json();
        let reloaded = ExpeditionState::from_json(&json).expect("it round trips");
        assert_eq!(reloaded, state);
        assert_eq!(reloaded.to_json(), json, "byte-identical on the way back");
    }
}
