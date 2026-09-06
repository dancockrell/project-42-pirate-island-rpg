//! S12: how a woman's relationship to Captain Michael and his faction stands.
//!
//! The continuation brief section 5.6 is the whole of the design this file
//! encodes, and it says three things that shape every type below.
//!
//! * **"Attraction creates openings. It does not erase character identity or
//!   automatically resolve allegiance."** A woman may be drawn to Michael and
//!   still stay where she is -- share information, delay an enemy action, warn
//!   him, join later, or never formally join. So attraction is not a track that
//!   fills up into membership. Nothing in this file advances a stage because a
//!   value got large.
//! * **Stages advance only through authored milestones.** There is no timer, no
//!   tick, and no accrual here. [`RecruitmentState::record_milestone`] is the
//!   only mutation that can move [`RecruitmentState::recruitment_stage`], and it
//!   moves it only when an authored [`MilestoneRule`] says this milestone, from
//!   where she stands now, is the beat that moves her. The rule set is data
//!   (a `BTreeMap` of rules), never a chain of `if`s, so C6/C12 can replace it
//!   with authored records without rewriting behaviour.
//! * **"Do not expose this complete structure as a numerical romance
//!   interface"** (brief section 19). The disposition values below cannot be
//!   read as numbers outside this module at all: [`Disposition`] has no public
//!   accessor, only ordering. The single thing the presentation layer is given
//!   is [`RecruitmentProjection`] -- a stage name and the current authored beat
//!   ID, both strings.
//!
//! ## Projection contract for lane B3 (the bridge)
//!
//! The bridge exposes exactly two keys per woman, and both come from
//! [`RecruitmentState::projection`]:
//!
//! * `recruitment_stage`: [`RecruitmentProjection::recruitment_stage`], one of
//!   the fixed lowercase words in [`RecruitmentStage::as_str`].
//! * `current_beat_id`: [`RecruitmentProjection::current_beat_id`], the stable
//!   ID of the most recently recorded authored milestone, or absent when her
//!   arc has not begun.
//!
//! Nothing else crosses. No disposition, no threshold, no count of recorded
//! milestones, no derived percentage, no "progress to next stage". The bridge
//! cannot construct one even if it wanted to: [`Disposition`]'s value is
//! private to this module, so a numeric romance interface is unrepresentable
//! outside `recruitment.rs` rather than merely discouraged.
//!
//! ## What is Open
//!
//! Brief section 20 leaves open the identities of the four women, how each one
//! joins, and the civilian-membership boundary. So this file authors no
//! milestone IDs and no woman beyond the two that already exist
//! (`character.heroine.betty`, `character.heroine.ayla`). Milestone IDs are
//! opaque stable IDs supplied by the caller; the rules that give them meaning
//! are authored content that C6/C12 will write into
//! [`RecruitmentState::milestone_rules`].

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, require_stable_id};

/// Faction IDs are concept keys, never proper names (`strategy/mod.rs`).
const FACTION_ID_PREFIX: &str = "faction.";

/// How far a woman's relationship to Captain Michael's faction has actually
/// come. The ladder is deliberately coarse and named in the brief's own terms:
/// it is a position in an authored arc, not a score.
///
/// The rungs between `Interested` and `Joined` exist because of the brief's
/// "attraction without immediate defection" section: a woman who helps Michael
/// from inside her own faction is a real, stable, possibly permanent state of
/// this simulation, not a way-point on the road to membership.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecruitmentStage {
    /// She does not know who he is.
    #[default]
    Unaware,
    /// She knows of him. Awareness is not interest.
    Aware,
    /// He has her attention -- curiosity, attraction, admiration, a grievance
    /// with her own faction he happens to answer. An opening, nothing more.
    Interested,
    /// She acts for him without leaving where she is: warns him, trades, delays
    /// an enemy, arranges a meeting. The brief's long-term contact.
    Contact,
    /// She has agreed to join and has not yet arrived. Her conditions are met.
    Committed,
    /// She is a member of the faction.
    Joined,
    /// She holds a place in it -- a role, a portfolio, people of her own.
    Integrated,
}

impl RecruitmentStage {
    /// The one place the ladder's order is written down. Advancement compares
    /// ranks; nothing else may.
    const fn rank(self) -> u8 {
        match self {
            RecruitmentStage::Unaware => 0,
            RecruitmentStage::Aware => 1,
            RecruitmentStage::Interested => 2,
            RecruitmentStage::Contact => 3,
            RecruitmentStage::Committed => 4,
            RecruitmentStage::Joined => 5,
            RecruitmentStage::Integrated => 6,
        }
    }

    /// The exact string the bridge shows. Fixed vocabulary: presentation may
    /// map these to prose, never to a number.
    pub const fn as_str(self) -> &'static str {
        match self {
            RecruitmentStage::Unaware => "unaware",
            RecruitmentStage::Aware => "aware",
            RecruitmentStage::Interested => "interested",
            RecruitmentStage::Contact => "contact",
            RecruitmentStage::Committed => "committed",
            RecruitmentStage::Joined => "joined",
            RecruitmentStage::Integrated => "integrated",
        }
    }
}

/// One disposition value, 0..=100.
///
/// It is deliberately write-only from outside this module: it can be
/// constructed and compared, and there is no way to read the number back out.
/// That is the structural half of "do not expose this as a numerical romance
/// interface" -- the bridge cannot leak a value it cannot obtain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Disposition(u8);

impl Disposition {
    pub const NONE: Disposition = Disposition(0);

    /// Clamps rather than refusing, so a save written by a future authoring
    /// tool with a wider range still loads at the same `save_version`.
    pub const fn new(value: u8) -> Self {
        Disposition(if value > 100 { 100 } else { value })
    }
}

/// The named dispositions a milestone rule may test. Naming them as a key type
/// rather than as struct-field accesses is what keeps the rule table data:
/// a rule is a map from one of these to a threshold, so an authored record can
/// carry the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Inclination {
    AwarenessOfMichael,
    AttractionToMichael,
    RomanticInterest,
    TrustInMichael,
    IdeologicalAlignment,
    DissatisfactionWithOrigin,
    Ambition,
    PerceivedSafety,
    PerceivedOpportunity,
    FearOfRetaliation,
}

impl Inclination {
    /// The only reader of a disposition value in the codebase, and it returns
    /// a `Disposition`, not a number.
    fn read(self, state: &RecruitmentState) -> Disposition {
        match self {
            Inclination::AwarenessOfMichael => state.awareness_of_michael,
            Inclination::AttractionToMichael => state.attraction_to_michael,
            Inclination::RomanticInterest => state.romantic_interest,
            Inclination::TrustInMichael => state.trust_in_michael,
            Inclination::IdeologicalAlignment => state.ideological_alignment,
            Inclination::DissatisfactionWithOrigin => state.dissatisfaction_with_origin,
            Inclination::Ambition => state.ambition,
            Inclination::PerceivedSafety => state.perceived_safety,
            Inclination::PerceivedOpportunity => state.perceived_opportunity,
            Inclination::FearOfRetaliation => state.fear_of_retaliation,
        }
    }
}

/// What an authored milestone does to one woman's arc.
///
/// This is the whole of the milestone-to-stage rule. There is no second place
/// where a stage can move, and no `if` chain anywhere that names a milestone
/// ID: a rule is a record, and C6/C12 replaces this table with authored
/// records of the same shape.
///
/// A rule that does not apply is not an error. Recording such a milestone still
/// happens -- the beat played -- it simply leaves the stage where it was, which
/// is exactly the brief's "attraction creates openings, not allegiance".
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneRule {
    /// Where this beat can carry her. It only ever advances: a rule whose
    /// target is at or below her current stage does nothing.
    pub advances_to: RecruitmentStage,
    /// She must stand at least here for the beat to land. An authored beat
    /// written for a long-term contact does not fire on a stranger.
    #[serde(default)]
    pub requires_stage_at_least: RecruitmentStage,
    /// Minimum dispositions. Every entry must be met.
    #[serde(default)]
    pub requires_at_least: BTreeMap<Inclination, Disposition>,
    /// Maximum dispositions -- the shape fear of retaliation needs. Every entry
    /// must be met.
    #[serde(default)]
    pub requires_at_most: BTreeMap<Inclination, Disposition>,
    /// Outstanding conditions of hers that block this beat while they stand.
    #[serde(default)]
    pub blocked_by_conditions: BTreeSet<String>,
    /// Conditions this beat settles. Cleared before the rest of the rule is
    /// judged, so one authored beat may both answer her condition and move her.
    #[serde(default)]
    pub clears_conditions: BTreeSet<String>,
}

impl MilestoneRule {
    /// The single predicate. Called once, from
    /// [`RecruitmentState::record_milestone`], after conditions are cleared.
    fn permits_advance(&self, state: &RecruitmentState) -> bool {
        if self.advances_to.rank() <= state.recruitment_stage.rank() {
            return false;
        }
        if state.recruitment_stage.rank() < self.requires_stage_at_least.rank() {
            return false;
        }
        if self
            .blocked_by_conditions
            .iter()
            .any(|condition| state.conditions.contains(condition))
        {
            return false;
        }
        if self
            .requires_at_least
            .iter()
            .any(|(inclination, floor)| inclination.read(state) < *floor)
        {
            return false;
        }
        if self
            .requires_at_most
            .iter()
            .any(|(inclination, ceiling)| inclination.read(state) > *ceiling)
        {
            return false;
        }
        true
    }
}

/// Everything the presentation layer is ever told about a recruitment arc.
///
/// Constructed only by [`RecruitmentState::projection`]. Both fields are
/// strings; there is no numeric field and no way to add one without editing
/// this type, which is the point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecruitmentProjection {
    /// [`RecruitmentStage::as_str`] of her current stage.
    pub recruitment_stage: &'static str,
    /// The stable ID of the last authored milestone recorded for her, if any.
    pub current_beat_id: Option<String>,
}

/// Brief section 19's `RecruitmentState`, field for field, plus the three
/// fields the milestone rule needs and section 19 does not name
/// (`milestone_rules`, `recorded_milestones`, `current_beat_id` -- each marked
/// below).
///
/// **Never surfaced as numbers.** See this module's projection contract: the
/// bridge receives [`RecruitmentProjection`] and nothing else. The dispositions
/// exist so authored beats can have conditions; they are not a meter.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecruitmentState {
    pub character_id: String,
    /// `faction.<concept_key>`. `None` for a woman with no faction behind her.
    #[serde(default)]
    pub origin_faction_id: Option<String>,
    /// Where she stands today, which is her origin until she leaves it.
    #[serde(default)]
    pub current_faction_id: Option<String>,
    /// Authored role key. A building may support a role; a real woman fills it.
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub awareness_of_michael: Disposition,
    #[serde(default)]
    pub attraction_to_michael: Disposition,
    #[serde(default)]
    pub romantic_interest: Disposition,
    #[serde(default)]
    pub trust_in_michael: Disposition,
    /// Per companion, keyed by character ID -- "a relationship with one of the
    /// companions" is its own reason to join.
    #[serde(default)]
    pub trust_in_companions: BTreeMap<String, Disposition>,
    #[serde(default)]
    pub ideological_alignment: Disposition,
    #[serde(default)]
    pub dissatisfaction_with_origin: Disposition,
    #[serde(default)]
    pub ambition: Disposition,
    /// Authored obligation IDs she is held by -- a household, a debt, a charge.
    #[serde(default)]
    pub personal_obligations: BTreeSet<String>,
    #[serde(default)]
    pub perceived_safety: Disposition,
    #[serde(default)]
    pub perceived_opportunity: Disposition,
    #[serde(default)]
    pub fear_of_retaliation: Disposition,
    /// Moved only by [`RecruitmentState::record_milestone`].
    #[serde(default)]
    pub recruitment_stage: RecruitmentStage,
    /// Authored condition IDs still outstanding: what she requires before she
    /// will go further. A rule may be blocked by one and may clear one.
    #[serde(default)]
    pub conditions: BTreeSet<String>,
    /// Character IDs she would bring with her (brief: group recruitment).
    #[serde(default)]
    pub followers: BTreeSet<String>,
    /// Ties she keeps after joining, character ID to authored relationship key.
    /// Joining Michael does not delete who she was.
    #[serde(default)]
    pub retained_relationships: BTreeMap<String, String>,
    /// Authored key for how far into the faction's life she has been brought.
    /// Left as an opaque key: the civilian-membership boundary is Open (brief
    /// section 20), so this file does not enumerate it.
    #[serde(default)]
    pub integration_state: Option<String>,
    /// Authored key for where her loyalty stands. Opaque for the same reason.
    #[serde(default)]
    pub loyalty_state: Option<String>,
    /// **Not in section 19.** The authored milestone table for her arc, keyed
    /// by milestone ID. Empty until C6/C12 authors it; a milestone with no rule
    /// here is recorded and changes nothing.
    #[serde(default)]
    pub milestone_rules: BTreeMap<String, MilestoneRule>,
    /// **Not in section 19.** Which authored milestones have already played, so
    /// a beat cannot be replayed to walk her up the ladder twice.
    #[serde(default)]
    pub recorded_milestones: BTreeSet<String>,
    /// **Not in section 19.** The last milestone recorded: the beat her arc is
    /// currently sitting on, and the only other thing the bridge is given.
    #[serde(default)]
    pub current_beat_id: Option<String>,
}

impl RecruitmentState {
    /// A woman who has not met him and owes this simulation nothing yet.
    pub fn new(character_id: impl Into<String>) -> Self {
        Self {
            character_id: character_id.into(),
            ..Self::default()
        }
    }

    /// Structural shape only, in the spirit of `expedition.rs`: stable IDs are
    /// lowercase-dotted, and a faction ID is a concept key.
    pub fn validate(&self) -> Result<(), ExpeditionError> {
        require_stable_id("recruitment.character_id", &self.character_id)?;
        for (field, id) in [
            ("recruitment.origin_faction_id", &self.origin_faction_id),
            ("recruitment.current_faction_id", &self.current_faction_id),
        ] {
            if let Some(id) = id {
                require_stable_id(field, id)?;
                if !id.starts_with(FACTION_ID_PREFIX) {
                    return Err(ExpeditionError::InvalidStableId {
                        field,
                        value: id.clone(),
                    });
                }
            }
        }
        if let Some(role) = &self.role {
            require_stable_id("recruitment.role", role)?;
        }
        Ok(())
    }

    /// Plays one authored milestone.
    ///
    /// Returns her stage afterward, or `None` -- with nothing mutated -- if this
    /// milestone has already been recorded. The stage moves only when an
    /// authored [`MilestoneRule`] for this milestone permits it from where she
    /// stands; otherwise the beat is recorded and she does not move. No clock,
    /// no accrual, no threshold crossing on its own.
    pub fn record_milestone(&mut self, milestone_id: &str) -> Option<RecruitmentStage> {
        if self.recorded_milestones.contains(milestone_id) {
            return None;
        }
        let rule = self.milestone_rules.get(milestone_id).cloned();
        self.recorded_milestones.insert(milestone_id.to_owned());
        self.current_beat_id = Some(milestone_id.to_owned());
        if let Some(rule) = rule {
            for condition in &rule.clears_conditions {
                self.conditions.remove(condition);
            }
            if rule.permits_advance(self) {
                self.recruitment_stage = rule.advances_to;
            }
        }
        Some(self.recruitment_stage)
    }

    /// Everything the bridge may see. See this module's projection contract.
    pub fn projection(&self) -> RecruitmentProjection {
        RecruitmentProjection {
            recruitment_stage: self.recruitment_stage.as_str(),
            current_beat_id: self.current_beat_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Betty and Ayla are the women that exist; the other two are Open (brief
    /// section 20), so fixtures never invent a third.
    const BETTY: &str = "character.heroine.betty";

    fn open_invitation() -> MilestoneRule {
        MilestoneRule {
            advances_to: RecruitmentStage::Committed,
            requires_stage_at_least: RecruitmentStage::Interested,
            requires_at_least: BTreeMap::from([(
                Inclination::TrustInMichael,
                Disposition::new(60),
            )]),
            ..MilestoneRule::default()
        }
    }

    fn drawn_to_him() -> RecruitmentState {
        let mut state = RecruitmentState::new(BETTY);
        state.origin_faction_id = Some("faction.free_harbour".into());
        state.recruitment_stage = RecruitmentStage::Interested;
        state.attraction_to_michael = Disposition::new(95);
        state.romantic_interest = Disposition::new(90);
        state.trust_in_michael = Disposition::new(10);
        state
            .milestone_rules
            .insert("milestone.test.open_invitation".into(), open_invitation());
        state
    }

    /// The card's required test. "Attraction creates openings. It does not
    /// erase character identity or automatically resolve allegiance."
    #[test]
    fn high_attraction_with_low_trust_does_not_advance_the_stage() {
        let mut state = drawn_to_him();
        let stage = state
            .record_milestone("milestone.test.open_invitation")
            .expect("first recording is accepted");
        assert_eq!(stage, RecruitmentStage::Interested);
        assert_eq!(state.recruitment_stage, RecruitmentStage::Interested);
        // The beat still played: she was asked, and she stayed where she was.
        assert_eq!(
            state.current_beat_id.as_deref(),
            Some("milestone.test.open_invitation")
        );
    }

    /// The same beat, the same attraction, once trust is actually there.
    #[test]
    fn the_same_milestone_advances_when_trust_is_there() {
        let mut state = drawn_to_him();
        state.trust_in_michael = Disposition::new(75);
        let stage = state
            .record_milestone("milestone.test.open_invitation")
            .expect("first recording is accepted");
        assert_eq!(stage, RecruitmentStage::Committed);
    }

    #[test]
    fn a_beat_written_for_a_contact_does_not_fire_on_a_stranger() {
        let mut state = drawn_to_him();
        state.trust_in_michael = Disposition::new(75);
        state.recruitment_stage = RecruitmentStage::Aware;
        assert_eq!(
            state.record_milestone("milestone.test.open_invitation"),
            Some(RecruitmentStage::Aware)
        );
    }

    #[test]
    fn an_unruled_milestone_is_recorded_and_moves_nothing() {
        let mut state = drawn_to_him();
        assert_eq!(
            state.record_milestone("milestone.test.shared_a_watch"),
            Some(RecruitmentStage::Interested)
        );
        assert!(
            state
                .recorded_milestones
                .contains("milestone.test.shared_a_watch")
        );
    }

    #[test]
    fn a_condition_blocks_until_the_authored_beat_settles_it() {
        let mut state = drawn_to_him();
        state.trust_in_michael = Disposition::new(75);
        state
            .conditions
            .insert("condition.test.sister_is_safe".into());
        let mut rule = open_invitation();
        rule.blocked_by_conditions
            .insert("condition.test.sister_is_safe".into());
        state
            .milestone_rules
            .insert("milestone.test.open_invitation".into(), rule.clone());
        assert_eq!(
            state.record_milestone("milestone.test.open_invitation"),
            Some(RecruitmentStage::Interested)
        );

        rule.clears_conditions
            .insert("condition.test.sister_is_safe".into());
        state
            .milestone_rules
            .insert("milestone.test.the_sister_is_brought_out".into(), rule);
        assert_eq!(
            state.record_milestone("milestone.test.the_sister_is_brought_out"),
            Some(RecruitmentStage::Committed)
        );
        assert!(state.conditions.is_empty());
    }

    #[test]
    fn fear_of_retaliation_holds_her_where_she_is() {
        let mut state = drawn_to_him();
        state.trust_in_michael = Disposition::new(75);
        state.fear_of_retaliation = Disposition::new(90);
        let mut rule = open_invitation();
        rule.requires_at_most
            .insert(Inclination::FearOfRetaliation, Disposition::new(40));
        state
            .milestone_rules
            .insert("milestone.test.open_invitation".into(), rule);
        assert_eq!(
            state.record_milestone("milestone.test.open_invitation"),
            Some(RecruitmentStage::Interested)
        );
    }

    #[test]
    fn a_recorded_milestone_is_refused_without_mutating() {
        let mut state = drawn_to_him();
        state.trust_in_michael = Disposition::new(75);
        assert_eq!(
            state.record_milestone("milestone.test.open_invitation"),
            Some(RecruitmentStage::Committed)
        );
        let before = state.clone();
        assert_eq!(
            state.record_milestone("milestone.test.open_invitation"),
            None
        );
        assert_eq!(state, before);
    }

    #[test]
    fn the_projection_carries_a_stage_word_and_a_beat_and_nothing_else() {
        let mut state = drawn_to_him();
        assert_eq!(
            state.projection(),
            RecruitmentProjection {
                recruitment_stage: "interested",
                current_beat_id: None,
            }
        );
        state.record_milestone("milestone.test.open_invitation");
        let projection = state.projection();
        assert_eq!(projection.recruitment_stage, "interested");
        assert_eq!(
            projection.current_beat_id.as_deref(),
            Some("milestone.test.open_invitation")
        );
    }

    #[test]
    fn a_faction_proper_name_is_not_a_faction_id() {
        let mut state = RecruitmentState::new(BETTY);
        state.origin_faction_id = Some("guild.of.rope_makers".into());
        assert!(matches!(
            state.validate(),
            Err(ExpeditionError::InvalidStableId {
                field: "recruitment.origin_faction_id",
                ..
            })
        ));
        state.origin_faction_id = Some("faction.free_harbour".into());
        assert_eq!(state.validate(), Ok(()));
    }

    /// The `ExpeditionState` side of the lane: the map survives a save/load
    /// round trip byte-identically, and the command refuses before it mutates.
    mod through_the_expedition_state {
        use super::*;

        use crate::expedition::ExpeditionState;

        const AYLA: &str = "character.heroine.ayla";

        fn campaign() -> ExpeditionState {
            let mut state = ExpeditionState::new(
                42,
                vec![
                    "character.protagonist.captain".into(),
                    BETTY.into(),
                    AYLA.into(),
                ],
                "world.cell.black_beach",
            )
            .expect("fixture campaign is legal");
            let mut betty = drawn_to_him();
            betty.trust_in_michael = Disposition::new(75);
            state.recruitment.insert(BETTY.into(), betty);
            state
                .recruitment
                .insert(AYLA.into(), RecruitmentState::new(AYLA));
            state
        }

        #[test]
        fn recruitment_survives_a_save_load_round_trip() {
            let mut state = campaign();
            assert_eq!(
                state.record_recruitment_milestone(BETTY, "milestone.test.open_invitation"),
                Ok(RecruitmentStage::Committed)
            );
            let json = state.to_json();
            let reloaded = ExpeditionState::from_json(&json).expect("round trip reloads");
            assert_eq!(reloaded, state);
            assert_eq!(reloaded.to_json(), json);
            assert_eq!(
                reloaded.recruitment[BETTY].projection(),
                RecruitmentProjection {
                    recruitment_stage: "committed",
                    current_beat_id: Some("milestone.test.open_invitation".into()),
                }
            );
        }

        #[test]
        fn a_second_recording_is_rejected_without_mutating() {
            let mut state = campaign();
            state
                .record_recruitment_milestone(BETTY, "milestone.test.open_invitation")
                .expect("first recording is accepted");
            let before = state.to_json();
            assert_eq!(
                state.record_recruitment_milestone(BETTY, "milestone.test.open_invitation"),
                Err(ExpeditionError::MilestoneAlreadyRecorded {
                    character_id: BETTY.into(),
                    milestone_id: "milestone.test.open_invitation".into(),
                })
            );
            assert_eq!(state.to_json(), before);
        }

        #[test]
        fn an_unknown_woman_is_refused_and_never_created() {
            let mut state = campaign();
            let before = state.to_json();
            assert_eq!(
                state.record_recruitment_milestone(
                    "character.heroine.test",
                    "milestone.test.open_invitation"
                ),
                Err(ExpeditionError::UnknownRecruit {
                    character_id: "character.heroine.test".into(),
                })
            );
            assert_eq!(state.to_json(), before);
        }

        #[test]
        fn a_malformed_milestone_id_is_refused_before_anything_moves() {
            let mut state = campaign();
            let before = state.to_json();
            assert!(matches!(
                state.record_recruitment_milestone(BETTY, "Milestone Test"),
                Err(ExpeditionError::InvalidStableId {
                    field: "milestone_id",
                    ..
                })
            ));
            assert_eq!(state.to_json(), before);
        }

        /// The lane's whole point, at the command boundary: she is as drawn to
        /// him as the simulation can express and she does not move.
        #[test]
        fn attraction_alone_does_not_move_her_through_the_command() {
            let mut state = campaign();
            let betty = state.recruitment.get_mut(BETTY).expect("fixture recruit");
            betty.trust_in_michael = Disposition::new(10);
            betty.attraction_to_michael = Disposition::new(100);
            betty.romantic_interest = Disposition::new(100);
            assert_eq!(
                state.record_recruitment_milestone(BETTY, "milestone.test.open_invitation"),
                Ok(RecruitmentStage::Interested)
            );
        }
    }

    #[test]
    fn stage_words_are_distinct_and_ordered() {
        let ladder = [
            RecruitmentStage::Unaware,
            RecruitmentStage::Aware,
            RecruitmentStage::Interested,
            RecruitmentStage::Contact,
            RecruitmentStage::Committed,
            RecruitmentStage::Joined,
            RecruitmentStage::Integrated,
        ];
        let words: BTreeSet<&str> = ladder.iter().map(|stage| stage.as_str()).collect();
        assert_eq!(words.len(), ladder.len());
        for pair in ladder.windows(2) {
            assert!(pair[0].rank() < pair[1].rank());
        }
    }
}
