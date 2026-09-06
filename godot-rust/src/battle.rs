use std::collections::BTreeMap;

use crate::strategy::site_rule::{ActiveSiteRule, SiteRuleEffect};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActorId(pub String);
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Faction {
    Party,
    Hostile,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattlePhase {
    AwaitingActor,
    AwaitingCommand,
    Resolving,
    Victory,
    Defeat,
    Retreated,
}

/// The five combat bands, LOCKED by the design bible (`req.combat.bands.five`).
/// `Actor.band` stays an `i8` so the save format does not change; every read of
/// it goes through [`Band::from_index`] so an out-of-range value is visible
/// rather than silently meaningful.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Band {
    PartyRear = 0,
    PartyFront = 1,
    Contested = 2,
    EnemyFront = 3,
    EnemyRear = 4,
}

impl Band {
    pub fn from_index(index: i8) -> Option<Band> {
        match index {
            0 => Some(Band::PartyRear),
            1 => Some(Band::PartyFront),
            2 => Some(Band::Contested),
            3 => Some(Band::EnemyFront),
            4 => Some(Band::EnemyRear),
            _ => None,
        }
    }

    pub fn index(&self) -> i8 {
        *self as i8
    }

    pub fn name(&self) -> &'static str {
        match self {
            Band::PartyRear => "party_rear",
            Band::PartyFront => "party_front",
            Band::Contested => "contested",
            Band::EnemyFront => "enemy_front",
            Band::EnemyRear => "enemy_rear",
        }
    }
}

/// Bond rank of an authored skill, mirroring the `bondRank` field of the
/// records under `content/skills/`. Used by the Composure gate: a Shaken actor
/// may not spend an SS or SSS command.
///
/// `content/skills/` is the owner and this is its twin, held equal by
/// `every_authored_skill_has_its_authored_rank`. That test exists because the
/// table arrived already incomplete: Michael's two commands were authored in
/// the same round and were missing here, and while both are rank D -- so the
/// gate behaved identically -- an SS skill added the same way would have
/// slipped past the Shaken gate in silence.
pub fn skill_rank(skill_id: &str) -> Option<&'static str> {
    match skill_id {
        "skill.captain.weapon_attack" => Some("D"),
        "skill.captain.reposition" => Some("D"),
        "skill.betty.guarded_strike" => Some("D"),
        "skill.betty.condition_cleanse" => Some("C"),
        "skill.betty.rescue_charge" => Some("B"),
        "skill.betty.healing_impact" => Some("A"),
        "skill.betty.fatal_intercept" => Some("S"),
        "skill.betty.mobile_infirmary" => Some("SS"),
        "skill.betty.combat_revival" => Some("SSS"),
        // A7: Ayla's seven, in the design bible's own order. The five PR #2
        // implemented and the two it could not, on the site-rule seam this
        // card built for them.
        "skill.ayla.reach_counter" => Some("D"),
        "skill.ayla.structural_scan" => Some("C"),
        "skill.ayla.safe_passage" => Some("B"),
        "skill.ayla.ward_line" => Some("A"),
        "skill.ayla.curse_dispel" => Some("S"),
        "skill.ayla.deny_activation" => Some("SS"),
        "skill.ayla.override_tomb_rule" => Some("SSS"),
        _ => None,
    }
}

/// The seven authored bond-rank letters, as an order. `D` is the floor every
/// woman starts at and `SSS` the top; there is no eighth rung and no numeric
/// spelling anywhere -- the ladder crosses the bridge as a letter (A10).
///
/// One table, two readers: the Composure gate (a Shaken actor may not spend a
/// command at `SS` or above) and the bond gate (a woman may not spend a command
/// ranked above where her relationship actually stands). A letter this does not
/// know is `None`, and both gates treat that as "not rank-governed" rather than
/// guessing -- the enemy vocabulary and `skill.system.*` have no bond rank at all.
pub fn rank_index(rank: &str) -> Option<u8> {
    match rank {
        "D" => Some(0),
        "C" => Some(1),
        "B" => Some(2),
        "A" => Some(3),
        "S" => Some(4),
        "SS" => Some(5),
        "SSS" => Some(6),
        _ => None,
    }
}

/// The rank at which the Composure gate closes a command to a Shaken actor.
const SHAKEN_CLOSES_AT_RANK: &str = "SS";

/// A7: Reach Counter's raw counter damage, before Ayla's level. Authored to
/// fit the simulation, as Betty's 12 and 24 were -- the design bible gives
/// fiction and function for a skill, never battle-math constants.
/// `content/skills/ayla.reach_counter.json` owns it and
/// `reach_counter_deals_its_authored_damage` holds the two equal.
const REACH_COUNTER_RAW_DAMAGE: i32 = 10;

/// A7: Ward Line's raw damage on trigger, before Ayla's level. A separate
/// number from Reach Counter's because the two reactions cost differently:
/// Reach Counter is free and unlimited, Ward Line costs a whole turn to place
/// and is spent after one hit.
const WARD_LINE_RAW_DAMAGE: i32 = 8;

/// The prefix every established woman's actor ID carries
/// (`character.heroine.<name>`). B17: it is what tells a battle actor whose
/// bond rank the campaign owns from one it does not -- Captain Michael, an
/// enemy and every `skill.system.*` user stand outside `bond_ranks` entirely,
/// and a battle built from a campaign leaves them exactly as the encounter
/// built them.
const WOMAN_ACTOR_ID_PREFIX: &str = "character.heroine.";

/// Whether this actor ID names one of the women whose bond rank
/// `ExpeditionState::bond_ranks` owns.
pub fn is_woman_actor_id(actor_id: &str) -> bool {
    actor_id.starts_with(WOMAN_ACTOR_ID_PREFIX)
}

/// Where a woman's bond stands before an authored scene has moved it. A10:
/// `ExpeditionState::bond_ranks` starts every established woman here, and an
/// `Actor` built without a rank is read as standing here too.
pub const STARTING_BOND_RANK: &str = "D";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusKind {
    Bleeding,
    Poisoned,
    Burning,
    Stunned,
    /// Composure has hit zero. The actor keeps acting, but SS and SSS commands
    /// (and, once it exists, Echo) are closed to them until it is restored.
    Shaken,
    /// A7: Ayla's Ward Line has caught this actor crossing it. Applied
    /// mid-battle by [`Battle::apply_status`] -- the first status in the engine
    /// that is neither set at actor construction nor only ever removed.
    Staggered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusInstance {
    pub id: String,
    pub kind: StatusKind,
    pub remaining_rounds: u8,
    pub source_id: ActorId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattlefieldEffect {
    pub instance_id: String,
    pub source_actor_id: ActorId,
    pub source_skill_id: String,
    pub remaining_pulses: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryOpening {
    pub actor_id: ActorId,
    pub source_skill_id: String,
    pub bonus_raw_damage: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Actor {
    pub id: ActorId,
    pub display_name: String,
    pub faction: Faction,
    pub level: u8,
    pub vitality: i32,
    pub max_vitality: i32,
    pub guard: i32,
    pub band: i8,
    /// Composure ranges 0-10. Costs and hostile effects reduce it; at 0 the
    /// actor gains `Shaken` and cannot use SS, SSS or Echo commands.
    pub composure: u8,
    pub initiative: i16,
    pub statuses: Vec<StatusInstance>,
    pub intercepts_for: Option<ActorId>,
    pub skill_uses_remaining: BTreeMap<String, u8>,
    /// A10: where this actor's bond actually stands, as one of the seven
    /// authored letters ([`rank_index`]). It is a fact of the *relationship*,
    /// not of the fight: `ExpeditionState::bond_ranks` owns it and authored
    /// scenes are the only thing that raise it. A command whose
    /// [`skill_rank`] sits above this is refused with
    /// [`BattleError::BondRankTooLow`] before anything mutates.
    ///
    /// [`STARTING_BOND_RANK`] is the floor; a letter outside the seven is read
    /// as the floor rather than trusted, so a malformed save cannot unlock a
    /// rank SSS command by spelling one wrong.
    pub bond_rank: String,
}
impl Actor {
    pub fn is_alive(&self) -> bool {
        self.vitality > 0
    }

    /// The actor's stored band index resolved to a named band, or `None` when
    /// the stored value is outside the five.
    pub fn band_kind(&self) -> Option<Band> {
        Band::from_index(self.band)
    }

    pub fn is_shaken(&self) -> bool {
        self.statuses
            .iter()
            .any(|status| status.kind == StatusKind::Shaken)
    }

    /// Reduce Composure, saturating at zero. Reaching zero pushes `Shaken`
    /// exactly once -- spending again while already at zero does not stack it.
    pub fn spend_composure(&mut self, amount: u8) {
        self.composure = self.composure.saturating_sub(amount);
        if self.composure == 0 && !self.is_shaken() {
            self.statuses.push(StatusInstance {
                id: format!("status.{}.shaken", self.id.0),
                kind: StatusKind::Shaken,
                remaining_rounds: 0,
                source_id: self.id.clone(),
            });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillCommand {
    pub command_id: String,
    pub actor_id: ActorId,
    pub skill_id: String,
    pub target_ids: Vec<ActorId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnemyDecision {
    pub actor_id: ActorId,
    pub skill_id: String,
    pub target_id: ActorId,
    pub rationale: String,
    pub guard_break_amount: i32,
    pub raw_damage: i32,
    pub guard_absorbed: i32,
    pub vitality_damage: i32,
    pub lethal: bool,
    pub interception_protector_id: Option<ActorId>,
    pub fatal_intercept_available: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BattleEvent {
    BattleStarted {
        round: u32,
    },
    TurnStarted {
        round: u32,
        actor_id: ActorId,
    },
    EnemyIntentDeclared {
        actor_id: ActorId,
        skill_id: String,
        target_ids: Vec<ActorId>,
        rationale: String,
        guard_break_amount: i32,
        raw_damage: i32,
        guard_absorbed: i32,
        vitality_damage: i32,
        lethal: bool,
        interception_protector_id: Option<ActorId>,
        fatal_intercept_available: bool,
    },
    CommandAccepted {
        command_id: String,
        actor_id: ActorId,
        skill_id: String,
    },
    ActorFocused {
        command_id: String,
        actor_id: ActorId,
    },
    DamageApplied {
        command_id: String,
        source_id: ActorId,
        target_id: ActorId,
        amount: i32,
    },
    GuardChanged {
        command_id: String,
        actor_id: ActorId,
        delta: i32,
        total: i32,
    },
    VitalityChanged {
        command_id: String,
        actor_id: ActorId,
        delta: i32,
        total: i32,
    },
    StatusRemoved {
        command_id: String,
        actor_id: ActorId,
        status_id: String,
        status_kind: StatusKind,
    },
    ActorMoved {
        command_id: String,
        actor_id: ActorId,
        from_band: i8,
        to_band: i8,
    },
    InterceptionSet {
        command_id: String,
        protector_id: ActorId,
        protected_id: ActorId,
    },
    InterceptionTriggered {
        command_id: String,
        protector_id: ActorId,
        protected_id: ActorId,
        attacker_id: ActorId,
    },
    ReactionWindowOpened {
        command_id: String,
        trigger: String,
        threatened_actor_id: ActorId,
    },
    ReactionTriggered {
        command_id: String,
        reactor_id: ActorId,
        skill_id: String,
        protected_id: ActorId,
    },
    DefeatPrevented {
        command_id: String,
        actor_id: ActorId,
        prevented_by_skill_id: String,
    },
    ActorRevived {
        command_id: String,
        actor_id: ActorId,
        vitality: i32,
    },
    BonusTurnGranted {
        command_id: String,
        actor_id: ActorId,
    },
    BattlefieldEffectCreated {
        command_id: String,
        effect_id: String,
        source_actor_id: ActorId,
        source_skill_id: String,
        total_pulses: u8,
    },
    BattlefieldEffectPulse {
        effect_id: String,
        source_actor_id: ActorId,
        pulses_remaining_after: u8,
    },
    BattlefieldEffectRemoved {
        effect_id: String,
        reason: String,
    },
    RecoveryOpeningCreated {
        command_id: String,
        actor_id: ActorId,
        source_skill_id: String,
        bonus_raw_damage: i32,
    },
    RecoveryOpeningConsumed {
        command_id: String,
        actor_id: ActorId,
        attacker_id: ActorId,
        bonus_raw_damage: i32,
    },
    RecoveryOpeningExpired {
        actor_id: ActorId,
    },
    ActorDefeated {
        command_id: String,
        actor_id: ActorId,
    },
    TurnEnded {
        round: u32,
        actor_id: ActorId,
    },
    RoundStarted {
        round: u32,
    },
    BattleEnded {
        victory: bool,
    },
    BattleRetreated {
        command_id: String,
        actor_id: ActorId,
    },
    /// A7 / Structural Scan: what the scan read off a target. The resolver only
    /// reads state; nothing in this event is a mutation.
    TargetInspected {
        command_id: String,
        actor_id: ActorId,
        target_id: ActorId,
        guard_revealed: i32,
        counter_tag: String,
    },
    /// A7: a status applied to a living actor after the battle has started.
    /// Every status before this was set at construction or removed by a skill;
    /// [`Battle::apply_status`] is the one writer of this event.
    StatusApplied {
        command_id: String,
        actor_id: ActorId,
        status_id: String,
        status_kind: StatusKind,
        source_id: ActorId,
    },
    WardLinePlaced {
        command_id: String,
        actor_id: ActorId,
        band: i8,
    },
    WardLineTriggered {
        command_id: String,
        attacker_id: ActorId,
        protected_id: ActorId,
    },
    /// A7 / Deny Activation: the hostile's declared action for this turn is
    /// cancelled. The bridge reads this to spend the site's one charge.
    ActivationDenied {
        command_id: String,
        actor_id: ActorId,
        target_id: ActorId,
    },
    /// A7 / Override Tomb Rule: this rule stops applying, here and for the rest
    /// of the expedition. The bridge reads this and writes `rule_id` into
    /// `ExpeditionState::suppressed_site_rules`.
    SiteRuleOverridden {
        command_id: String,
        actor_id: ActorId,
        rule_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum BattleError {
    BattleAlreadyEnded,
    WrongPhase {
        expected: BattlePhase,
        actual: BattlePhase,
    },
    NotActiveActor {
        expected: ActorId,
        actual: ActorId,
    },
    UnknownActor(ActorId),
    UnknownTarget(ActorId),
    ActorDefeated(ActorId),
    ActorIncapacitated {
        actor_id: ActorId,
        status: StatusKind,
    },
    SkillUnavailable {
        actor_id: ActorId,
        skill_id: String,
    },
    TargetMustBeDefeated(ActorId),
    FriendlyFire {
        actor_id: ActorId,
        target_id: ActorId,
    },
    IllegalSelfTarget {
        skill_id: String,
        actor_id: ActorId,
    },
    IllegalTargetCount {
        skill_id: String,
        expected: usize,
        actual: usize,
    },
    SkillOwnerMismatch {
        skill_id: String,
        expected_actor_id: ActorId,
        actual_actor_id: ActorId,
    },
    /// U3: the shared enemy skill vocabulary (`grasping_strike`, `dread_gaze`) is
    /// not locked to one exact actor the way Razorbeak's or a heroine's signature
    /// skills are -- any individual creature record may declare them -- but it is
    /// still owner-checked: only a hostile actor may submit one.
    HostileSkillUsedByNonHostile {
        skill_id: String,
        actor_id: ActorId,
    },
    /// A6: `skill.captain.reposition` is a one-band step between the two party
    /// bands. From anywhere else -- Contested or either enemy band -- there is
    /// no legal destination, and the command is refused before anything moves.
    RepositionNotLegal {
        actor_id: ActorId,
        from_band: i8,
    },
    /// Composure is spent: a Shaken actor may not spend an SS or SSS command.
    ShakenCannotUse {
        skill_id: String,
        actor_id: ActorId,
    },
    UnsupportedSkill(String),
    RetreatNotAllowed,
    IllegalRetreatActor(ActorId),
    /// A10: the command is ranked above where this woman's bond stands. Bond
    /// rank is a fact of the relationship -- it moves when an authored scene's
    /// milestone is recorded, never on a timer -- so this is not a cooldown to
    /// wait out. `required` and `current` are both authored letters.
    BondRankTooLow {
        skill_id: String,
        required: String,
        current: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct BattleSnapshot {
    pub battle_id: String,
    pub round: u32,
    pub phase: BattlePhase,
    pub active_actor_id: Option<ActorId>,
    pub actors: Vec<Actor>,
    pub effects: Vec<BattlefieldEffect>,
    pub recovery_openings: Vec<RecoveryOpening>,
    pub forced_next_actor_id: Option<ActorId>,
    pub forced_turn_resume: Option<(usize, u32)>,
}

#[derive(Clone, Debug)]
pub struct Battle {
    battle_id: String,
    actors: BTreeMap<ActorId, Actor>,
    turn_order: Vec<ActorId>,
    turn_index: usize,
    round: u32,
    phase: BattlePhase,
    resolving_reaction: bool,
    effects: Vec<BattlefieldEffect>,
    recovery_openings: BTreeMap<ActorId, RecoveryOpening>,
    forced_next_actor: Option<ActorId>,
    forced_turn_resume: Option<(usize, u32)>,
    retreat_allowed: bool,
    resolved_commands: BTreeMap<String, Vec<BattleEvent>>,
    /// A7: the site rules in force in this fight, in the cell's authored order.
    /// The campaign decides the list (the cell's `site_rule_ids` minus
    /// `ExpeditionState::suppressed_site_rules`); the battle only applies it.
    site_rules: Vec<ActiveSiteRule>,
    /// A7 / Ward Line: the single active ward, or `None`.
    active_ward: Option<WardLine>,
}

/// A7 / Ward Line: "place a ward between two adjacent bands; the first enemy
/// crossing it takes damage and becomes Staggered."
///
/// No hostile in this engine ever moves bands, so "crossing" is translated the
/// same way Reach Counter translates "entering the Contested band": the first
/// hostile attack targeting a party member who stands in the warded band. The
/// ward is spent on that first trigger.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WardLine {
    source_actor_id: ActorId,
    band: i8,
}

/// A7 / B17: everything the campaign says about a battle it is arming.
///
/// B17 built `prototype_vertical_slice_from_bond_ranks`, which took the one
/// campaign fact a battle then needed. A7 adds three more -- the site rules in
/// force and Ayla's two site-scoped charges -- and a four-argument constructor
/// with three of them positional would be a trap, so the parameter is this
/// struct and the constructor is its successor. There is still exactly one
/// campaign-shaped constructor; nothing was forked beside the old one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CampaignBattleSetup {
    /// `ExpeditionState::bond_ranks`, the sole owner of where each woman's
    /// bond stands (A10).
    pub bond_ranks: BTreeMap<String, String>,
    /// The cell's `site_rule_ids` minus `ExpeditionState::suppressed_site_rules`,
    /// resolved against the authored registry.
    pub site_rules: Vec<ActiveSiteRule>,
    /// Whether this site still has its one Deny Activation. Once per site: the
    /// count lives in `ExpeditionState::site_denial_charges`, keyed by site.
    pub deny_activation_charges: u8,
    /// Whether this expedition still has its one Override Tomb Rule. Once per
    /// expedition: it is spent when `suppressed_site_rules` stops being empty.
    pub override_tomb_rule_charges: u8,
}

#[derive(Clone, Debug, PartialEq)]
struct DamageOutcome {
    amount: i32,
    defeated: bool,
    prevented: bool,
}

impl Battle {
    pub fn prototype_vertical_slice() -> Self {
        let actor =
            |id: &str, display_name: &str, faction: Faction, level, vitality, guard, initiative| {
                Actor {
                    id: ActorId(id.into()),
                    display_name: display_name.into(),
                    faction,
                    level,
                    vitality,
                    max_vitality: vitality,
                    guard,
                    band: 0,
                    composure: 10,
                    initiative,
                    statuses: Vec::new(),
                    intercepts_for: None,
                    skill_uses_remaining: BTreeMap::new(),
                    // A10: the slice is a static fixture -- nothing builds it
                    // from an `ExpeditionState` -- so it carries the ranks that
                    // keep the vertical slice playing exactly as it did before
                    // the gate existed. Betty is raised below; everyone else
                    // stands at the floor, which is enough for every command
                    // they own (all rank D) and for the unranked enemy and
                    // system vocabulary.
                    bond_rank: STARTING_BOND_RANK.to_owned(),
                }
            };
        let mut betty = actor(
            "character.heroine.betty",
            "Betty",
            Faction::Party,
            3,
            100,
            0,
            12,
        );
        betty.band = Band::PartyFront.index();
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        // The slice demonstrates her whole authored kit, up to the rank SSS
        // Combat Revival, so the fixture Betty stands at the top of the ladder.
        // The campaign's Betty does not: `ExpeditionState::new` starts her at D
        // and only an authored scene raises her.
        "SSS".clone_into(&mut betty.bond_rank);

        let mut ayla = actor(
            "character.heroine.ayla",
            "Ayla",
            Faction::Party,
            3,
            80,
            0,
            9,
        );
        ayla.band = Band::PartyRear.index();
        // A7: Reach Counter is unlimited-use, so this entry is a presence gate
        // -- "Ayla has this skill" -- and is never decremented, unlike Fatal
        // Intercept's charge. She is down in the slice, and a defeated Ayla
        // never reacts, so the slice plays exactly as it did before.
        ayla.skill_uses_remaining
            .insert("skill.ayla.reach_counter".into(), 1);
        ayla.vitality = 0;
        ayla.statuses.push(StatusInstance {
            id: "status.ayla.bleeding.prototype".into(),
            kind: StatusKind::Bleeding,
            remaining_rounds: 2,
            source_id: ActorId("enemy.raptor.razorbeak".into()),
        });

        let mut vix = actor("character.heroine.vix", "Vix", Faction::Party, 3, 80, 0, 10);
        vix.vitality = 10;
        vix.band = Band::PartyFront.index();
        vix.statuses.push(StatusInstance {
            id: "status.vix.poisoned.prototype".into(),
            kind: StatusKind::Poisoned,
            remaining_rounds: 3,
            source_id: ActorId("enemy.raptor.razorbeak".into()),
        });

        let mut razorbeak = actor(
            "enemy.raptor.razorbeak",
            "Razorbeak",
            Faction::Hostile,
            7,
            70,
            3,
            11,
        );
        razorbeak.band = Band::EnemyFront.index();

        // B5: the Captain stands in the slice at A6's stats. A6 built him as a
        // battle actor but left him out of this fixture, because
        // `native_simulation_port_test.gd` asserted it held four actors; that
        // assertion now names five and says why. His display name is authored in
        // `content/characters/captain.json` and
        // `the_captain_carries_his_authored_display_name` holds the two equal.
        let mut michael = actor(
            "character.protagonist.captain",
            "Michael Corrigan",
            Faction::Party,
            3,
            90,
            0,
            10,
        );
        michael.band = Band::PartyRear.index();

        Self::new(
            "battle.prototype.returning_names",
            [betty, ayla, vix, razorbeak, michael],
        )
    }

    /// B17, extended by A7: the same fixture encounter, built for a campaign
    /// rather than for a review scene.
    ///
    /// There is one actor list -- [`Battle::prototype_vertical_slice`] above --
    /// and this delegates to it, then copies the campaign's facts over the
    /// fixture's: where each woman's bond stands, which site rules are in
    /// force here, and whether Ayla's two site-scoped commands still have their
    /// charge. A caller with an `ExpeditionState` uses this; the debug battle,
    /// which has no campaign at all, keeps the fixture and stands under no site
    /// rules.
    pub fn prototype_vertical_slice_from_campaign(setup: &CampaignBattleSetup) -> Self {
        let mut battle = Self::prototype_vertical_slice();
        battle.apply_bond_ranks(&setup.bond_ranks);
        battle.site_rules = setup.site_rules.clone();
        if let Some(ayla) = battle
            .actors
            .get_mut(&ActorId("character.heroine.ayla".into()))
        {
            ayla.skill_uses_remaining.insert(
                "skill.ayla.deny_activation".into(),
                setup.deny_activation_charges,
            );
            ayla.skill_uses_remaining.insert(
                "skill.ayla.override_tomb_rule".into(),
                setup.override_tomb_rule_charges,
            );
        }
        battle
    }

    /// The site rules in force in this fight, in the cell's authored order.
    pub fn site_rules(&self) -> &[ActiveSiteRule] {
        &self.site_rules
    }

    /// A7: the rules this battle stands under. Used by the tests and by any
    /// caller building a battle that is not the campaign's fixture encounter.
    pub fn with_site_rules(mut self, rules: Vec<ActiveSiteRule>) -> Self {
        self.site_rules = rules;
        self
    }

    /// B17: makes every woman in this battle stand where her bond actually
    /// stands. `ranks` is `ExpeditionState::bond_ranks`, the sole owner of that
    /// fact (A10); `Actor::bond_rank` is a copy of it and nothing else.
    ///
    /// A woman the map does not know falls to [`STARTING_BOND_RANK`] rather
    /// than keeping whatever the encounter fixture set -- a campaign that has
    /// never heard of her has not raised her, so the floor is the truthful
    /// answer and a fixture SSS must not survive into a fight the campaign
    /// built. Actors who are not women ([`is_woman_actor_id`]) are left alone:
    /// the map has no entry for them and no opinion about them.
    pub fn apply_bond_ranks(&mut self, ranks: &BTreeMap<String, String>) {
        for (actor_id, actor) in self.actors.iter_mut() {
            if !is_woman_actor_id(&actor_id.0) {
                continue;
            }
            actor.bond_rank = ranks
                .get(&actor_id.0)
                .cloned()
                .unwrap_or_else(|| STARTING_BOND_RANK.to_owned());
        }
    }

    pub fn new(battle_id: impl Into<String>, actors: impl IntoIterator<Item = Actor>) -> Self {
        let actors: BTreeMap<_, _> = actors
            .into_iter()
            .map(|actor| (actor.id.clone(), actor))
            .collect();
        let mut turn_order: Vec<_> = actors.values().map(|actor| actor.id.clone()).collect();
        turn_order.sort_by(|left, right| {
            let l = actors.get(left).expect("actor exists");
            let r = actors.get(right).expect("actor exists");
            r.initiative
                .cmp(&l.initiative)
                .then_with(|| left.cmp(right))
        });
        Self {
            battle_id: battle_id.into(),
            actors,
            turn_order,
            turn_index: 0,
            round: 1,
            phase: BattlePhase::AwaitingActor,
            resolving_reaction: false,
            effects: Vec::new(),
            recovery_openings: BTreeMap::new(),
            forced_next_actor: None,
            forced_turn_resume: None,
            retreat_allowed: true,
            resolved_commands: BTreeMap::new(),
            site_rules: Vec::new(),
            active_ward: None,
        }
    }

    /// B2: builder for encounters whose stable encounter rule forbids retreat.
    /// Every battle otherwise allows it, matching the vertical-slice contract's
    /// current encounters.
    pub fn with_retreat_allowed(mut self, allowed: bool) -> Self {
        self.retreat_allowed = allowed;
        self
    }

    pub fn actor(&self, id: &ActorId) -> Option<&Actor> {
        self.actors.get(id)
    }
    pub fn active_actor_id(&self) -> Option<&ActorId> {
        self.turn_order.get(self.turn_index)
    }
    pub fn recommended_enemy_command(&self, command_id: impl Into<String>) -> Option<SkillCommand> {
        let decision = self.enemy_decision()?;
        Some(SkillCommand {
            command_id: command_id.into(),
            actor_id: decision.actor_id,
            skill_id: decision.skill_id,
            target_ids: vec![decision.target_id],
        })
    }

    pub fn enemy_decision(&self) -> Option<EnemyDecision> {
        if self.phase != BattlePhase::AwaitingCommand {
            return None;
        }
        let actor_id = self.active_actor_id()?.clone();
        let attacker = self.actors.get(&actor_id)?;
        if attacker.faction != Faction::Hostile || !attacker.is_alive() {
            return None;
        }
        let target = self
            .actors
            .values()
            .filter(|candidate| candidate.faction == Faction::Party && candidate.is_alive())
            .min_by(|left, right| {
                compare_vitality_percentage(left, right)
                    .then_with(|| left.guard.cmp(&right.guard))
                    .then_with(|| left.id.cmp(&right.id))
            })?;
        let interception_protector_id = self
            .actors
            .values()
            .find(|candidate| {
                candidate.is_alive() && candidate.intercepts_for.as_ref() == Some(&target.id)
            })
            .map(|candidate| candidate.id.clone());
        let damage_recipient = interception_protector_id
            .as_ref()
            .and_then(|protector_id| self.actors.get(protector_id))
            .unwrap_or(target);
        let uses_guard_break = damage_recipient.guard >= 4;
        let guard_break_amount = if uses_guard_break {
            damage_recipient.guard.min(6)
        } else {
            0
        };
        let raw_damage = if uses_guard_break { 6 } else { 9 } + i32::from(attacker.level);
        let remaining_guard = damage_recipient.guard - guard_break_amount;
        let guard_absorbed = remaining_guard.min(raw_damage);
        let vitality_damage = raw_damage - guard_absorbed;
        let lethal = vitality_damage >= damage_recipient.vitality;
        let betty_id = ActorId("character.heroine.betty".into());
        let fatal_intercept_available = lethal
            && interception_protector_id.is_none()
            && self.actors.get(&betty_id).is_some_and(|betty| {
                betty.is_alive()
                    && !betty
                        .statuses
                        .iter()
                        .any(|status| status.kind == StatusKind::Stunned)
                    && betty
                        .skill_uses_remaining
                        .get("skill.betty.fatal_intercept")
                        .copied()
                        .unwrap_or(0)
                        > 0
            });
        Some(EnemyDecision {
            actor_id,
            skill_id: if uses_guard_break {
                "skill.enemy.razorbeak.guard_breaking_kick".into()
            } else {
                "skill.enemy.razorbeak.rushing_bite".into()
            },
            target_id: target.id.clone(),
            rationale: if uses_guard_break && interception_protector_id.is_some() {
                "break_guard_on_protector".into()
            } else if uses_guard_break {
                "break_guard_on_target".into()
            } else if lethal {
                "finish_most_wounded".into()
            } else {
                "pressure_most_wounded".into()
            },
            guard_break_amount,
            raw_damage,
            guard_absorbed,
            vitality_damage,
            lethal,
            interception_protector_id,
            fatal_intercept_available,
        })
    }
    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            battle_id: self.battle_id.clone(),
            round: self.round,
            phase: self.phase.clone(),
            active_actor_id: self.active_actor_id().cloned(),
            actors: self.actors.values().cloned().collect(),
            effects: self.effects.clone(),
            recovery_openings: self.recovery_openings.values().cloned().collect(),
            forced_next_actor_id: self.forced_next_actor.clone(),
            forced_turn_resume: self.forced_turn_resume,
        }
    }

    pub fn start(&mut self) -> Vec<BattleEvent> {
        if self.phase != BattlePhase::AwaitingActor || self.turn_order.is_empty() {
            return Vec::new();
        }
        self.skip_defeated_actors();
        self.phase = BattlePhase::AwaitingCommand;
        let actor_id = self.active_actor_id().expect("actor exists").clone();
        let mut events = vec![
            BattleEvent::BattleStarted { round: self.round },
            BattleEvent::TurnStarted {
                round: self.round,
                actor_id: actor_id.clone(),
            },
        ];
        if self.actor(&actor_id).expect("actor exists").faction == Faction::Hostile {
            events.push(self.enemy_intent(&actor_id));
        }
        events
    }

    /// B2: submitting the same `command_id` against the same authoritative state must
    /// not double-resolve. The first submission validates and mutates normally; every
    /// later submission of that exact `command_id` returns the recorded result without
    /// touching state again, regardless of what has happened to the battle since.
    pub fn submit(&mut self, command: SkillCommand) -> Result<Vec<BattleEvent>, BattleError> {
        if let Some(recorded) = self.resolved_commands.get(&command.command_id) {
            return Ok(recorded.clone());
        }
        let command_id = command.command_id.clone();
        let result = self.submit_uncached(command);
        if let Ok(events) = &result {
            self.resolved_commands.insert(command_id, events.clone());
        }
        result
    }

    fn submit_uncached(&mut self, command: SkillCommand) -> Result<Vec<BattleEvent>, BattleError> {
        if matches!(
            self.phase,
            BattlePhase::Victory | BattlePhase::Defeat | BattlePhase::Retreated
        ) {
            return Err(BattleError::BattleAlreadyEnded);
        }
        if self.phase != BattlePhase::AwaitingCommand {
            return Err(BattleError::WrongPhase {
                expected: BattlePhase::AwaitingCommand,
                actual: self.phase.clone(),
            });
        }
        let active = self.active_actor_id().expect("active actor exists").clone();
        if active != command.actor_id {
            return Err(BattleError::NotActiveActor {
                expected: active,
                actual: command.actor_id,
            });
        }
        let actor = self
            .actors
            .get(&command.actor_id)
            .ok_or_else(|| BattleError::UnknownActor(command.actor_id.clone()))?;
        if !actor.is_alive() {
            return Err(BattleError::ActorDefeated(command.actor_id));
        }
        if actor
            .statuses
            .iter()
            .any(|status| status.kind == StatusKind::Stunned)
        {
            return Err(BattleError::ActorIncapacitated {
                actor_id: command.actor_id,
                status: StatusKind::Stunned,
            });
        }
        // Both gates read the same ladder ([`rank_index`]) and both stand before
        // any mutation. A skill with no authored rank -- the enemy vocabulary,
        // `skill.system.*` -- is governed by neither.
        let required_rank = skill_rank(&command.skill_id);
        let required_index = required_rank.and_then(rank_index);
        if actor.is_shaken()
            && required_index.is_some_and(|required| {
                required >= rank_index(SHAKEN_CLOSES_AT_RANK).expect("SS is an authored rank")
            })
        {
            return Err(BattleError::ShakenCannotUse {
                skill_id: command.skill_id,
                actor_id: command.actor_id,
            });
        }
        // A10: a command ranked above where her bond actually stands is refused
        // here, before anything moves. An unrecognised stored letter is read as
        // the floor rather than trusted.
        if let (Some(required), Some(required_index)) = (required_rank, required_index) {
            let current_index = rank_index(&actor.bond_rank)
                .unwrap_or_else(|| rank_index(STARTING_BOND_RANK).expect("D is an authored rank"));
            if required_index > current_index {
                return Err(BattleError::BondRankTooLow {
                    skill_id: command.skill_id,
                    required: required.to_owned(),
                    current: actor.bond_rank.clone(),
                });
            }
        }
        if !matches!(
            command.skill_id.as_str(),
            "skill.captain.weapon_attack"
                | "skill.captain.reposition"
                | "skill.betty.guarded_strike"
                | "skill.betty.condition_cleanse"
                | "skill.betty.rescue_charge"
                | "skill.betty.healing_impact"
                | "skill.betty.mobile_infirmary"
                | "skill.betty.combat_revival"
                | "skill.ayla.structural_scan"
                | "skill.ayla.safe_passage"
                | "skill.ayla.ward_line"
                | "skill.ayla.curse_dispel"
                | "skill.ayla.deny_activation"
                | "skill.ayla.override_tomb_rule"
                | "skill.enemy.razorbeak.rushing_bite"
                | "skill.enemy.razorbeak.guard_breaking_kick"
                | "skill.enemy.undead.grasping_strike"
                | "skill.enemy.eldritch.dread_gaze"
                | "skill.system.hold_position"
                | "skill.system.retreat"
        ) {
            return Err(BattleError::UnsupportedSkill(command.skill_id));
        }
        let expected_owner = if command.skill_id.starts_with("skill.captain.") {
            Some(ActorId("character.protagonist.captain".into()))
        } else if command.skill_id.starts_with("skill.betty.") {
            Some(ActorId("character.heroine.betty".into()))
        } else if command.skill_id.starts_with("skill.ayla.") {
            Some(ActorId("character.heroine.ayla".into()))
        } else if command.skill_id.starts_with("skill.enemy.razorbeak.") {
            Some(ActorId("enemy.raptor.razorbeak".into()))
        } else {
            None
        };
        if let Some(expected_actor_id) = expected_owner {
            if command.actor_id != expected_actor_id {
                return Err(BattleError::SkillOwnerMismatch {
                    skill_id: command.skill_id,
                    expected_actor_id,
                    actual_actor_id: command.actor_id,
                });
            }
        }
        // The shared enemy skill vocabulary (U3) isn't locked to one exact actor
        // the way Razorbeak's or a heroine's signature skills are -- any hostile
        // creature record may declare `grasping_strike` or `dread_gaze` -- but it
        // is still owner-checked: only a hostile actor may submit one.
        if command.skill_id.starts_with("skill.enemy.")
            && !command.skill_id.starts_with("skill.enemy.razorbeak.")
            && actor.faction != Faction::Hostile
        {
            return Err(BattleError::HostileSkillUsedByNonHostile {
                skill_id: command.skill_id,
                actor_id: command.actor_id,
            });
        }
        // A charge-gated command. Combat Revival is once per battle; Ayla's
        // Deny Activation is once per *site* and her Override Tomb Rule once
        // per *expedition*, and both arrive already spent or unspent from the
        // campaign (`CampaignBattleSetup`) -- inside one battle the gate is the
        // same one Combat Revival established, so there is one gate and not
        // three.
        if matches!(
            command.skill_id.as_str(),
            "skill.betty.combat_revival"
                | "skill.ayla.deny_activation"
                | "skill.ayla.override_tomb_rule"
        ) && actor
            .skill_uses_remaining
            .get(&command.skill_id)
            .copied()
            .unwrap_or(0)
            == 0
        {
            return Err(BattleError::SkillUnavailable {
                actor_id: command.actor_id,
                skill_id: command.skill_id,
            });
        }
        let expected_targets = match command.skill_id.as_str() {
            "skill.betty.rescue_charge" => 2,
            "skill.captain.reposition"
            | "skill.betty.mobile_infirmary"
            | "skill.ayla.safe_passage"
            | "skill.ayla.ward_line"
            | "skill.ayla.override_tomb_rule"
            | "skill.system.hold_position"
            | "skill.system.retreat" => 0,
            _ => 1,
        };
        if command.target_ids.len() != expected_targets {
            return Err(BattleError::IllegalTargetCount {
                skill_id: command.skill_id,
                expected: expected_targets,
                actual: command.target_ids.len(),
            });
        }
        for target_id in &command.target_ids {
            let target = self
                .actors
                .get(target_id)
                .ok_or_else(|| BattleError::UnknownTarget(target_id.clone()))?;
            if command.skill_id == "skill.betty.combat_revival" && target.is_alive() {
                return Err(BattleError::TargetMustBeDefeated(target_id.clone()));
            }
            if command.skill_id != "skill.betty.combat_revival" && !target.is_alive() {
                return Err(BattleError::ActorDefeated(target_id.clone()));
            }
        }
        if command.skill_id == "skill.captain.reposition"
            && reposition_destination(actor.band).is_none()
        {
            return Err(BattleError::RepositionNotLegal {
                actor_id: command.actor_id,
                from_band: actor.band,
            });
        }
        if command.skill_id == "skill.system.retreat" {
            if !self.retreat_allowed {
                return Err(BattleError::RetreatNotAllowed);
            }
            if actor.faction != Faction::Party {
                return Err(BattleError::IllegalRetreatActor(command.actor_id));
            }
            self.phase = BattlePhase::Retreated;
            return Ok(vec![
                BattleEvent::CommandAccepted {
                    command_id: command.command_id.clone(),
                    actor_id: command.actor_id.clone(),
                    skill_id: command.skill_id.clone(),
                },
                BattleEvent::BattleRetreated {
                    command_id: command.command_id.clone(),
                    actor_id: command.actor_id,
                },
            ]);
        }
        if matches!(
            command.skill_id.as_str(),
            "skill.captain.reposition"
                | "skill.betty.mobile_infirmary"
                | "skill.ayla.safe_passage"
                | "skill.ayla.ward_line"
                | "skill.ayla.override_tomb_rule"
                | "skill.system.hold_position"
        ) {
            self.phase = BattlePhase::Resolving;
            let mut events = vec![
                BattleEvent::CommandAccepted {
                    command_id: command.command_id.clone(),
                    actor_id: command.actor_id.clone(),
                    skill_id: command.skill_id.clone(),
                },
                BattleEvent::ActorFocused {
                    command_id: command.command_id.clone(),
                    actor_id: command.actor_id.clone(),
                },
            ];
            self.resolve_skill(&command, &mut events)?;
            events.push(BattleEvent::TurnEnded {
                round: self.round,
                actor_id: command.actor_id,
            });
            self.advance_turn(&mut events);
            return Ok(events);
        }
        let target_id = command.target_ids[0].clone();
        let target = self
            .actors
            .get(&target_id)
            .ok_or_else(|| BattleError::UnknownTarget(target_id.clone()))?;
        match command.skill_id.as_str() {
            "skill.betty.condition_cleanse"
            | "skill.betty.rescue_charge"
            | "skill.betty.combat_revival"
            | "skill.ayla.curse_dispel"
                if actor.faction != target.faction =>
            {
                return Err(BattleError::FriendlyFire {
                    actor_id: command.actor_id,
                    target_id,
                });
            }
            "skill.captain.weapon_attack"
            | "skill.betty.guarded_strike"
            | "skill.betty.healing_impact"
            | "skill.ayla.structural_scan"
            | "skill.ayla.deny_activation"
            | "skill.enemy.razorbeak.rushing_bite"
            | "skill.enemy.razorbeak.guard_breaking_kick"
            | "skill.enemy.undead.grasping_strike"
            | "skill.enemy.eldritch.dread_gaze"
                if actor.faction == target.faction =>
            {
                return Err(BattleError::FriendlyFire {
                    actor_id: command.actor_id,
                    target_id,
                });
            }
            _ => {}
        }
        if command.skill_id == "skill.betty.rescue_charge" && command.actor_id == target_id {
            return Err(BattleError::IllegalSelfTarget {
                skill_id: command.skill_id,
                actor_id: command.actor_id,
            });
        }
        if command.skill_id == "skill.betty.combat_revival" && command.actor_id == target_id {
            return Err(BattleError::IllegalSelfTarget {
                skill_id: command.skill_id,
                actor_id: command.actor_id,
            });
        }

        self.phase = BattlePhase::Resolving;
        let mut events = vec![
            BattleEvent::CommandAccepted {
                command_id: command.command_id.clone(),
                actor_id: command.actor_id.clone(),
                skill_id: command.skill_id.clone(),
            },
            BattleEvent::ActorFocused {
                command_id: command.command_id.clone(),
                actor_id: command.actor_id.clone(),
            },
        ];
        self.resolve_skill(&command, &mut events)?;
        events.push(BattleEvent::TurnEnded {
            round: self.round,
            actor_id: command.actor_id,
        });
        self.advance_turn(&mut events);
        Ok(events)
    }

    fn resolve_skill(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let actor_level = self
            .actors
            .get(&command.actor_id)
            .expect("actor checked")
            .level;
        match command.skill_id.as_str() {
            "skill.betty.condition_cleanse" => {
                return self.resolve_condition_cleanse(command, events);
            }
            "skill.betty.rescue_charge" => return self.resolve_rescue_charge(command, events),
            "skill.captain.reposition" => return self.resolve_reposition(command, events),
            "skill.betty.healing_impact" => return self.resolve_healing_impact(command, events),
            "skill.betty.mobile_infirmary" => {
                self.resolve_mobile_infirmary(command, events);
                return Ok(());
            }
            "skill.betty.combat_revival" => {
                self.resolve_combat_revival(command, events);
                return Ok(());
            }
            "skill.ayla.curse_dispel" => return self.resolve_curse_dispel(command, events),
            "skill.ayla.structural_scan" => return self.resolve_structural_scan(command, events),
            "skill.ayla.safe_passage" => {
                self.resolve_safe_passage(command, events);
                return Ok(());
            }
            "skill.ayla.ward_line" => {
                self.resolve_ward_line(command, events);
                return Ok(());
            }
            "skill.ayla.deny_activation" => {
                self.resolve_deny_activation(command, events);
                return Ok(());
            }
            "skill.ayla.override_tomb_rule" => {
                self.resolve_override_tomb_rule(command, events);
                return Ok(());
            }
            "skill.system.hold_position" => {
                let actor = self
                    .actors
                    .get_mut(&command.actor_id)
                    .expect("actor checked");
                actor.guard += 2;
                events.push(BattleEvent::GuardChanged {
                    command_id: command.command_id.clone(),
                    actor_id: command.actor_id.clone(),
                    delta: 2,
                    total: actor.guard,
                });
                return Ok(());
            }
            "skill.captain.weapon_attack"
            | "skill.betty.guarded_strike"
            | "skill.enemy.razorbeak.rushing_bite"
            | "skill.enemy.razorbeak.guard_breaking_kick"
            | "skill.enemy.undead.grasping_strike"
            | "skill.enemy.eldritch.dread_gaze" => {}
            other => return Err(BattleError::UnsupportedSkill(other.to_owned())),
        }
        let mut target_id = command.target_ids[0].clone();
        // Any hostile attack in this shared shape can be redirected onto an
        // active interceptor, not only Razorbeak's own two skills.
        if command.skill_id.starts_with("skill.enemy.") {
            let protected_id = target_id.clone();
            let protector_id = self
                .actors
                .values()
                .find(|candidate| {
                    candidate.is_alive() && candidate.intercepts_for.as_ref() == Some(&protected_id)
                })
                .map(|candidate| candidate.id.clone());
            if let Some(protector_id) = protector_id {
                self.actors
                    .get_mut(&protector_id)
                    .expect("protector exists")
                    .intercepts_for = None;
                events.push(BattleEvent::InterceptionTriggered {
                    command_id: command.command_id.clone(),
                    protector_id: protector_id.clone(),
                    protected_id,
                    attacker_id: command.actor_id.clone(),
                });
                target_id = protector_id;
            }
        }
        // Dread Gaze reuses the same "strip Guard, then hit through what's left"
        // shape Guard-Breaking Kick already established, at a smaller amount and
        // without a Recovery Opening -- a dreadful stare unravels composure, it
        // doesn't leave a physical opening to punish.
        let guard_strip = match command.skill_id.as_str() {
            "skill.enemy.razorbeak.guard_breaking_kick" => 6,
            "skill.enemy.eldritch.dread_gaze" => 4,
            _ => 0,
        };
        if guard_strip > 0 {
            let target = self.actors.get_mut(&target_id).expect("target checked");
            let guard_lost = target.guard.min(guard_strip);
            target.guard -= guard_lost;
            events.push(BattleEvent::GuardChanged {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
                delta: -guard_lost,
                total: target.guard,
            });
        }
        let (raw_damage, guard_gain) = if command.skill_id == "skill.captain.weapon_attack" {
            // `content/skills/captain.weapon_attack.json` owns these:
            // damageBase 10, damageLevelScale 1, guardGain 0. Held equal by
            // `captain_weapon_attack_deals_its_authored_damage`.
            (10 + i32::from(actor_level), 0)
        } else if command.skill_id == "skill.betty.guarded_strike" {
            (12 + i32::from(actor_level), 2)
        } else if command.skill_id == "skill.enemy.razorbeak.guard_breaking_kick" {
            (6 + i32::from(actor_level), 0)
        } else {
            (9 + i32::from(actor_level), 0)
        };
        self.apply_damage(
            &command.command_id,
            &command.actor_id,
            &target_id,
            raw_damage,
            true,
            events,
        );
        if command.skill_id == "skill.enemy.razorbeak.guard_breaking_kick" {
            let opening = RecoveryOpening {
                actor_id: command.actor_id.clone(),
                source_skill_id: command.skill_id.clone(),
                bonus_raw_damage: 6,
            };
            self.recovery_openings
                .insert(command.actor_id.clone(), opening.clone());
            events.push(BattleEvent::RecoveryOpeningCreated {
                command_id: command.command_id.clone(),
                actor_id: opening.actor_id,
                source_skill_id: opening.source_skill_id,
                bonus_raw_damage: opening.bonus_raw_damage,
            });
        }
        if guard_gain > 0 {
            let actor = self
                .actors
                .get_mut(&command.actor_id)
                .expect("actor checked");
            actor.guard += guard_gain;
            events.push(BattleEvent::GuardChanged {
                command_id: command.command_id.clone(),
                actor_id: command.actor_id.clone(),
                delta: guard_gain,
                total: actor.guard,
            });
        }
        Ok(())
    }

    fn resolve_condition_cleanse(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let target_id = &command.target_ids[0];
        let target = self.actors.get_mut(target_id).expect("target checked");
        target.statuses.sort_by(|left, right| {
            cleanse_priority(&left.kind)
                .unwrap_or(u8::MAX)
                .cmp(&cleanse_priority(&right.kind).unwrap_or(u8::MAX))
                .then_with(|| left.id.cmp(&right.id))
        });
        // Statuses the cleanse cannot remove sort last, so the drain stops at
        // the first of them rather than reaching into them.
        let remove_count = target
            .statuses
            .iter()
            .take_while(|status| cleanse_priority(&status.kind).is_some())
            .count()
            .min(2);
        for status in target.statuses.drain(0..remove_count).collect::<Vec<_>>() {
            events.push(BattleEvent::StatusRemoved {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
                status_id: status.id,
                status_kind: status.kind,
            });
        }
        let before = target.vitality;
        target.vitality = (target.vitality + 8).min(target.max_vitality);
        events.push(BattleEvent::VitalityChanged {
            command_id: command.command_id.clone(),
            actor_id: target_id.clone(),
            delta: target.vitality - before,
            total: target.vitality,
        });
        Ok(())
    }

    // ---- A7: Ayla's seven --------------------------------------------------

    /// **S Curse Dispel.** Removes every negative status the target is
    /// carrying, and heals nothing -- which is what distinguishes it from
    /// Betty's rank C Condition Cleanse (two statuses and eight Vitality).
    ///
    /// `Shaken` is not a curse. It is Composure at zero (A5), restored by
    /// Composure and not by a dispel, so it is the one status this leaves
    /// standing -- the same status `cleanse_priority` already refuses to
    /// remove, read through that same table rather than a second one.
    fn resolve_curse_dispel(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let target_id = &command.target_ids[0];
        let target = self.actors.get_mut(target_id).expect("target checked");
        target.statuses.sort_by(|left, right| {
            cleanse_priority(&left.kind)
                .unwrap_or(u8::MAX)
                .cmp(&cleanse_priority(&right.kind).unwrap_or(u8::MAX))
                .then_with(|| left.id.cmp(&right.id))
        });
        let removed: Vec<StatusInstance> = {
            let (removable, kept) = target
                .statuses
                .drain(..)
                .partition(|status| status.kind != StatusKind::Shaken);
            target.statuses = kept;
            removable
        };
        for status in removed {
            events.push(BattleEvent::StatusRemoved {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
                status_id: status.id,
                status_kind: status.kind,
            });
        }
        Ok(())
    }

    /// **C Structural Scan.** Reads Guard and one valid counter off a hostile
    /// and mutates nothing. A turn-consuming normal action: a free-action turn
    /// economy does not exist anywhere in this engine and was not invented for
    /// one skill.
    fn resolve_structural_scan(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let target_id = &command.target_ids[0];
        let target = self.actors.get(target_id).expect("target checked");
        let guard_revealed = target.guard;
        let counter_tag = if guard_revealed > 0 {
            "break_guard_before_striking"
        } else {
            "exploit_open_guard"
        };
        events.push(BattleEvent::TargetInspected {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            target_id: target_id.clone(),
            guard_revealed,
            counter_tag: counter_tag.to_owned(),
        });
        Ok(())
    }

    /// **B Safe Passage.** Brings every living ally standing one band away into
    /// Ayla's band, in stable actor-ID order, without triggering a movement
    /// reaction -- there are none in this engine, so the "safe" half is free
    /// and the passage is the whole of it. Zero-selection, like Mobile
    /// Infirmary.
    fn resolve_safe_passage(&mut self, command: &SkillCommand, events: &mut Vec<BattleEvent>) {
        let ayla_id = command.actor_id.clone();
        let ayla_band = self.actors.get(&ayla_id).expect("actor checked").band;
        let movers: Vec<ActorId> = self
            .actors
            .values()
            .filter(|actor| {
                actor.faction == Faction::Party
                    && actor.is_alive()
                    && actor.id != ayla_id
                    && actor.band != ayla_band
                    && (actor.band - ayla_band).abs() == 1
            })
            .map(|actor| actor.id.clone())
            .collect();
        for actor_id in movers {
            self.move_actor_to_band(&command.command_id, &actor_id, ayla_band, events);
        }
    }

    /// **A Ward Line.** Places (or replaces) the single ward at Ayla's own
    /// band. The trigger lives in [`Battle::apply_damage`], beside Reach
    /// Counter's structurally identical check; see [`WardLine`] for why
    /// "crossing" is spelled that way.
    fn resolve_ward_line(&mut self, command: &SkillCommand, events: &mut Vec<BattleEvent>) {
        let band = self
            .actors
            .get(&command.actor_id)
            .expect("actor checked")
            .band;
        self.active_ward = Some(WardLine {
            source_actor_id: command.actor_id.clone(),
            band,
        });
        events.push(BattleEvent::WardLinePlaced {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            band,
        });
    }

    /// **SS Deny Activation.** Cancels one hostile's declared action for the
    /// turn: the target is `Stunned` for one round, which is exactly what
    /// `submit_uncached` already refuses a command from
    /// ([`BattleError::ActorIncapacitated`]). The bible's "one understood
    /// machine, ritual or phase activation" is that refusal; the engine has no
    /// other declared action to cancel.
    ///
    /// Once per *site*: the charge arrives from
    /// `ExpeditionState::site_denial_charges` through [`CampaignBattleSetup`],
    /// is spent here, and the bridge writes the site's count down when it sees
    /// [`BattleEvent::ActivationDenied`].
    fn resolve_deny_activation(&mut self, command: &SkillCommand, events: &mut Vec<BattleEvent>) {
        let target_id = command.target_ids[0].clone();
        let ayla = self
            .actors
            .get_mut(&command.actor_id)
            .expect("acting Ayla checked");
        *ayla
            .skill_uses_remaining
            .get_mut(&command.skill_id)
            .expect("availability checked") -= 1;
        events.push(BattleEvent::ActivationDenied {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            target_id: target_id.clone(),
        });
        self.apply_status(
            &command.command_id,
            &target_id,
            StatusInstance {
                id: format!("status.deny_activation.stunned.{}", command.command_id),
                kind: StatusKind::Stunned,
                remaining_rounds: 1,
                source_id: command.actor_id.clone(),
            },
            events,
        );
    }

    /// **SSS Override Tomb Rule.** Takes the first site rule still in force out
    /// of this fight and emits [`BattleEvent::SiteRuleOverridden`]; the bridge
    /// writes that ID into `ExpeditionState::suppressed_site_rules`, which is
    /// what makes it hold "until the party leaves the site" and beyond.
    ///
    /// *First* in the cell's authored order, not chosen: the player picks no
    /// rule because no screen exists to pick one from, and a deterministic
    /// order is the honest stand-in for a choice nobody can make yet. Once per
    /// expedition; standing under no rules at all, the command spends its
    /// charge and overrides nothing, which is the truthful outcome rather than
    /// a refusal invented here.
    fn resolve_override_tomb_rule(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) {
        let ayla = self
            .actors
            .get_mut(&command.actor_id)
            .expect("acting Ayla checked");
        *ayla
            .skill_uses_remaining
            .get_mut(&command.skill_id)
            .expect("availability checked") -= 1;
        if self.site_rules.is_empty() {
            return;
        }
        let overridden = self.site_rules.remove(0);
        events.push(BattleEvent::SiteRuleOverridden {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            rule_id: overridden.id,
        });
    }

    /// A7: the engine's one writer of a status onto a living actor after the
    /// battle has started. Every status before this was set at construction or
    /// only ever removed.
    fn apply_status(
        &mut self,
        command_id: &str,
        target_id: &ActorId,
        status: StatusInstance,
        events: &mut Vec<BattleEvent>,
    ) {
        events.push(BattleEvent::StatusApplied {
            command_id: command_id.to_owned(),
            actor_id: target_id.clone(),
            status_id: status.id.clone(),
            status_kind: status.kind.clone(),
            source_id: status.source_id.clone(),
        });
        if let Some(target) = self.actors.get_mut(target_id) {
            target.statuses.push(status);
        }
    }

    /// A7: the site rules in force, applied at the round boundary.
    ///
    /// `guard_regen_per_round` is the only effect that acts today;
    /// `needs_decision` is a decision nobody has made and does nothing, loudly
    /// (`strategy/site_rule.rs`). Guard arrives through the same
    /// `GuardChanged` event the presentation layer already binds, so a site
    /// rule needs no new spelling on the Godot side.
    fn apply_site_rules_at_round_boundary(&mut self, events: &mut Vec<BattleEvent>) {
        let regen: i32 = self
            .site_rules
            .iter()
            .map(|rule| match rule.effect {
                SiteRuleEffect::GuardRegenPerRound(amount) => amount,
                SiteRuleEffect::NeedsDecision(_) => 0,
            })
            .sum();
        if regen == 0 {
            return;
        }
        let command_id = format!("site_rule.round.{}", self.round);
        for actor in self
            .actors
            .values_mut()
            .filter(|actor| actor.faction == Faction::Hostile && actor.is_alive())
        {
            actor.guard += regen;
            events.push(BattleEvent::GuardChanged {
                command_id: command_id.clone(),
                actor_id: actor.id.clone(),
                delta: regen,
                total: actor.guard,
            });
        }
    }

    /// A7: a round turns. One place, so a site rule cannot be applied on one
    /// path through [`Battle::advance_turn`] and skipped on the other.
    fn begin_round(&mut self, events: &mut Vec<BattleEvent>) {
        events.push(BattleEvent::RoundStarted { round: self.round });
        self.apply_site_rules_at_round_boundary(events);
    }

    fn resolve_rescue_charge(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let ally_id = &command.target_ids[0];
        let hostile_id = &command.target_ids[1];
        let actor_faction = self
            .actors
            .get(&command.actor_id)
            .expect("actor checked")
            .faction
            .clone();
        let hostile = self
            .actors
            .get(hostile_id)
            .ok_or_else(|| BattleError::UnknownTarget(hostile_id.clone()))?;
        if hostile.faction == actor_faction {
            return Err(BattleError::FriendlyFire {
                actor_id: command.actor_id.clone(),
                target_id: hostile_id.clone(),
            });
        }
        let ally_band = self.actors.get(ally_id).expect("ally checked").band;
        self.move_actor_to_band(&command.command_id, &command.actor_id, ally_band, events);
        self.actors
            .get_mut(&command.actor_id)
            .expect("actor checked")
            .intercepts_for = Some(ally_id.clone());
        events.push(BattleEvent::InterceptionSet {
            command_id: command.command_id.clone(),
            protector_id: command.actor_id.clone(),
            protected_id: ally_id.clone(),
        });
        self.apply_damage(
            &command.command_id,
            &command.actor_id,
            hostile_id,
            10,
            true,
            events,
        );
        Ok(())
    }

    /// The one band move in the game. Rescue Charge (a heroine crossing to the
    /// band her ally is standing in) and the Captain's Reposition (a one-band
    /// step between the two party bands) are the same movement with different
    /// destinations, so they are one function: the caller decides where, this
    /// decides what moving means and emits the `ActorMoved` the presentation
    /// layer binds its `actor_moved` beat to.
    ///
    /// The destination is an `i8` rather than a [`Band`] because Rescue Charge
    /// copies whatever band its ally is standing in, including a stored value
    /// outside the five; `Band`-typed callers pass `Band::index()`.
    fn move_actor_to_band(
        &mut self,
        command_id: &str,
        actor_id: &ActorId,
        to_band: i8,
        events: &mut Vec<BattleEvent>,
    ) {
        let actor = self.actors.get_mut(actor_id).expect("moving actor checked");
        let from_band = actor.band;
        actor.band = to_band;
        events.push(BattleEvent::ActorMoved {
            command_id: command_id.into(),
            actor_id: actor_id.clone(),
            from_band,
            to_band,
        });
    }

    /// `skill.captain.reposition`: a one-band step between `PartyRear` and
    /// `PartyFront`, taking no target and ending the turn. Legality was decided
    /// in `submit_uncached` before anything mutated, so the destination is
    /// known to exist by the time this runs.
    ///
    /// `content/skills/captain.reposition.json` owns the numbers -- movementBands
    /// 1, guardGain 1, damageBase 0 -- and
    /// `captain_reposition_moves_one_band_and_gains_its_authored_guard` holds
    /// them equal.
    fn resolve_reposition(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let from_band = self
            .actors
            .get(&command.actor_id)
            .expect("actor checked")
            .band;
        let destination =
            reposition_destination(from_band).ok_or_else(|| BattleError::RepositionNotLegal {
                actor_id: command.actor_id.clone(),
                from_band,
            })?;
        self.move_actor_to_band(
            &command.command_id,
            &command.actor_id,
            destination.index(),
            events,
        );
        let actor = self
            .actors
            .get_mut(&command.actor_id)
            .expect("actor checked");
        actor.guard += 1;
        events.push(BattleEvent::GuardChanged {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            delta: 1,
            total: actor.guard,
        });
        Ok(())
    }

    fn resolve_healing_impact(
        &mut self,
        command: &SkillCommand,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let hostile_id = &command.target_ids[0];
        let actor_level = self
            .actors
            .get(&command.actor_id)
            .expect("actor checked")
            .level;
        let raw_damage = 18 + i32::from(actor_level);
        let outcome = self.apply_damage(
            &command.command_id,
            &command.actor_id,
            hostile_id,
            raw_damage,
            true,
            events,
        );
        let healing = outcome.amount / 2;
        let recipient_id = self
            .actors
            .values()
            .filter(|candidate| candidate.faction == Faction::Party && candidate.is_alive())
            .min_by(|left, right| {
                compare_vitality_percentage(left, right).then_with(|| left.id.cmp(&right.id))
            })
            .map(|candidate| candidate.id.clone())
            .expect("acting party has a living member");
        let recipient = self
            .actors
            .get_mut(&recipient_id)
            .expect("recipient exists");
        let before = recipient.vitality;
        recipient.vitality = (recipient.vitality + healing).min(recipient.max_vitality);
        events.push(BattleEvent::VitalityChanged {
            command_id: command.command_id.clone(),
            actor_id: recipient_id,
            delta: recipient.vitality - before,
            total: recipient.vitality,
        });
        Ok(())
    }

    fn resolve_mobile_infirmary(&mut self, command: &SkillCommand, events: &mut Vec<BattleEvent>) {
        let replaced: Vec<_> = self
            .effects
            .iter()
            .filter(|effect| effect.source_skill_id == command.skill_id)
            .map(|effect| effect.instance_id.clone())
            .collect();
        for effect_id in replaced {
            self.remove_effect(&effect_id, "replaced", events);
        }
        let effect_id = format!("effect.{}.mobile_infirmary", command.command_id);
        self.effects.push(BattlefieldEffect {
            instance_id: effect_id.clone(),
            source_actor_id: command.actor_id.clone(),
            source_skill_id: command.skill_id.clone(),
            remaining_pulses: 3,
        });
        events.push(BattleEvent::BattlefieldEffectCreated {
            command_id: command.command_id.clone(),
            effect_id: effect_id.clone(),
            source_actor_id: command.actor_id.clone(),
            source_skill_id: command.skill_id.clone(),
            total_pulses: 3,
        });
        self.pulse_effect(&effect_id, events);
    }

    fn resolve_combat_revival(&mut self, command: &SkillCommand, events: &mut Vec<BattleEvent>) {
        let target_id = command.target_ids[0].clone();
        let betty = self
            .actors
            .get_mut(&command.actor_id)
            .expect("acting Betty checked");
        *betty
            .skill_uses_remaining
            .get_mut(&command.skill_id)
            .expect("availability checked") -= 1;

        let target = self.actors.get_mut(&target_id).expect("target checked");
        target.vitality = ((target.max_vitality * 40) + 99) / 100;
        target.guard = 0;
        let removed_statuses = std::mem::take(&mut target.statuses);
        for status in removed_statuses {
            events.push(BattleEvent::StatusRemoved {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
                status_id: status.id,
                status_kind: status.kind,
            });
        }
        events.push(BattleEvent::ActorRevived {
            command_id: command.command_id.clone(),
            actor_id: target_id.clone(),
            vitality: target.vitality,
        });
        events.push(BattleEvent::BonusTurnGranted {
            command_id: command.command_id.clone(),
            actor_id: target_id.clone(),
        });
        self.forced_next_actor = Some(target_id);
    }

    fn pulse_effects_for_source(
        &mut self,
        source_actor_id: &ActorId,
        events: &mut Vec<BattleEvent>,
    ) {
        let effect_ids: Vec<_> = self
            .effects
            .iter()
            .filter(|effect| &effect.source_actor_id == source_actor_id)
            .map(|effect| effect.instance_id.clone())
            .collect();
        for effect_id in effect_ids {
            self.pulse_effect(&effect_id, events);
        }
    }

    fn pulse_effect(&mut self, effect_id: &str, events: &mut Vec<BattleEvent>) {
        let Some(index) = self
            .effects
            .iter()
            .position(|effect| effect.instance_id == effect_id)
        else {
            return;
        };
        let source_actor_id = self.effects[index].source_actor_id.clone();
        self.effects[index].remaining_pulses -= 1;
        let remaining = self.effects[index].remaining_pulses;
        events.push(BattleEvent::BattlefieldEffectPulse {
            effect_id: effect_id.into(),
            source_actor_id,
            pulses_remaining_after: remaining,
        });
        for actor in self
            .actors
            .values_mut()
            .filter(|actor| actor.faction == Faction::Party && actor.is_alive())
        {
            let before = actor.vitality;
            actor.vitality = (actor.vitality + 10).min(actor.max_vitality);
            events.push(BattleEvent::VitalityChanged {
                command_id: effect_id.into(),
                actor_id: actor.id.clone(),
                delta: actor.vitality - before,
                total: actor.vitality,
            });
            actor.guard += 2;
            events.push(BattleEvent::GuardChanged {
                command_id: effect_id.into(),
                actor_id: actor.id.clone(),
                delta: 2,
                total: actor.guard,
            });
        }
        if remaining == 0 {
            self.remove_effect(effect_id, "duration_completed", events);
        }
    }

    fn remove_effect(&mut self, effect_id: &str, reason: &str, events: &mut Vec<BattleEvent>) {
        if let Some(index) = self
            .effects
            .iter()
            .position(|effect| effect.instance_id == effect_id)
        {
            self.effects.remove(index);
            events.push(BattleEvent::BattlefieldEffectRemoved {
                effect_id: effect_id.into(),
                reason: reason.into(),
            });
        }
    }

    fn remove_effects_from_source(
        &mut self,
        source_actor_id: &ActorId,
        events: &mut Vec<BattleEvent>,
    ) {
        let effect_ids: Vec<_> = self
            .effects
            .iter()
            .filter(|effect| &effect.source_actor_id == source_actor_id)
            .map(|effect| effect.instance_id.clone())
            .collect();
        for effect_id in effect_ids {
            self.remove_effect(&effect_id, "source_defeated", events);
        }
    }

    fn apply_damage(
        &mut self,
        command_id: &str,
        source_id: &ActorId,
        target_id: &ActorId,
        raw_damage: i32,
        allow_reactions: bool,
        events: &mut Vec<BattleEvent>,
    ) -> DamageOutcome {
        let mut raw_damage = raw_damage;
        let source_is_party = self
            .actors
            .get(source_id)
            .is_some_and(|source| source.faction == Faction::Party);
        if source_is_party {
            if let Some(opening) = self.recovery_openings.remove(target_id) {
                raw_damage += opening.bonus_raw_damage;
                events.push(BattleEvent::RecoveryOpeningConsumed {
                    command_id: command_id.into(),
                    actor_id: target_id.clone(),
                    attacker_id: source_id.clone(),
                    bonus_raw_damage: opening.bonus_raw_damage,
                });
            }
        }
        let target = self.actors.get(target_id).expect("damage target checked");
        let absorbed = target.guard.min(raw_damage);
        let predicted_damage = (raw_damage - absorbed).min(target.vitality);
        let would_defeat = predicted_damage >= target.vitality;
        let target_faction = target.faction.clone();
        let target_band = target.band;
        let source_is_hostile = self
            .actors
            .get(source_id)
            .is_some_and(|source| source.faction == Faction::Hostile);

        // A7 / **D Reach Counter.** The bible's "attack an enemy that enters
        // the Contested band" translated with the concept the engine actually
        // has: no hostile in this engine ever moves bands, and a shared band
        // *is* contested, so the trigger is a hostile's attack on a party
        // member standing where Ayla stands. Unlimited-use: the
        // `skill_uses_remaining` entry is a presence gate and is never
        // decremented, unlike Fatal Intercept's charge, so an Ayla without the
        // entry -- every fixture written before this card -- stays inert.
        let ayla_id = ActorId("character.heroine.ayla".into());
        let reach_counter_eligible = allow_reactions
            && !self.resolving_reaction
            && source_is_hostile
            && target_faction == Faction::Party
            && self.actors.get(&ayla_id).is_some_and(|ayla| {
                ayla.is_alive()
                    && !ayla
                        .statuses
                        .iter()
                        .any(|status| status.kind == StatusKind::Stunned)
                    && ayla.band == target_band
                    && ayla
                        .skill_uses_remaining
                        .get("skill.ayla.reach_counter")
                        .copied()
                        .unwrap_or(0)
                        > 0
            });
        if reach_counter_eligible {
            let ayla_level = self
                .actors
                .get(&ayla_id)
                .expect("eligible Ayla exists")
                .level;
            events.push(BattleEvent::ReactionWindowOpened {
                command_id: command_id.into(),
                trigger: "hostile_attack_in_aylas_band".into(),
                threatened_actor_id: target_id.clone(),
            });
            events.push(BattleEvent::ReactionTriggered {
                command_id: command_id.into(),
                reactor_id: ayla_id.clone(),
                skill_id: "skill.ayla.reach_counter".into(),
                protected_id: target_id.clone(),
            });
            self.resolving_reaction = true;
            self.apply_damage(
                command_id,
                &ayla_id,
                source_id,
                REACH_COUNTER_RAW_DAMAGE + i32::from(ayla_level),
                false,
                events,
            );
            self.resolving_reaction = false;
        }

        // A7 / **A Ward Line.** The same translation of "crossing", one skill
        // up: the first hostile attack on a party member in the warded band.
        // The ward spends itself on that trigger and does not fire again until
        // it is re-placed.
        let ward_triggered = allow_reactions
            && !self.resolving_reaction
            && source_is_hostile
            && target_faction == Faction::Party
            && self
                .active_ward
                .as_ref()
                .is_some_and(|ward| ward.band == target_band);
        if ward_triggered {
            let ward = self.active_ward.take().expect("checked above");
            let ayla_level = self
                .actors
                .get(&ward.source_actor_id)
                .map(|ayla| ayla.level)
                .unwrap_or(0);
            events.push(BattleEvent::WardLineTriggered {
                command_id: command_id.into(),
                attacker_id: source_id.clone(),
                protected_id: target_id.clone(),
            });
            self.resolving_reaction = true;
            self.apply_damage(
                command_id,
                &ward.source_actor_id,
                source_id,
                WARD_LINE_RAW_DAMAGE + i32::from(ayla_level),
                false,
                events,
            );
            self.apply_status(
                command_id,
                source_id,
                StatusInstance {
                    id: format!("status.ward_line.staggered.{command_id}"),
                    kind: StatusKind::Staggered,
                    remaining_rounds: 1,
                    source_id: ward.source_actor_id,
                },
                events,
            );
            self.resolving_reaction = false;
        }

        let opens_reaction = allow_reactions
            && !self.resolving_reaction
            && would_defeat
            && source_is_hostile
            && target_faction == Faction::Party
            && target_id.0 != "character.heroine.betty";
        if opens_reaction {
            events.push(BattleEvent::ReactionWindowOpened {
                command_id: command_id.into(),
                trigger: "party_member_would_be_defeated".into(),
                threatened_actor_id: target_id.clone(),
            });
            let betty_id = ActorId("character.heroine.betty".into());
            let can_react = self.actors.get(&betty_id).is_some_and(|betty| {
                betty.is_alive()
                    && !betty
                        .statuses
                        .iter()
                        .any(|status| status.kind == StatusKind::Stunned)
                    && betty
                        .skill_uses_remaining
                        .get("skill.betty.fatal_intercept")
                        .copied()
                        .unwrap_or(0)
                        > 0
            });
            if can_react {
                let betty = self
                    .actors
                    .get_mut(&betty_id)
                    .expect("eligible Betty exists");
                *betty
                    .skill_uses_remaining
                    .get_mut("skill.betty.fatal_intercept")
                    .expect("reaction charge exists") -= 1;
                events.push(BattleEvent::ReactionTriggered {
                    command_id: command_id.into(),
                    reactor_id: betty_id.clone(),
                    skill_id: "skill.betty.fatal_intercept".into(),
                    protected_id: target_id.clone(),
                });
                events.push(BattleEvent::DefeatPrevented {
                    command_id: command_id.into(),
                    actor_id: target_id.clone(),
                    prevented_by_skill_id: "skill.betty.fatal_intercept".into(),
                });
                self.resolving_reaction = true;
                self.apply_damage(command_id, &betty_id, source_id, 24, false, events);
                self.resolving_reaction = false;
                return DamageOutcome {
                    amount: 0,
                    defeated: false,
                    prevented: true,
                };
            }
        }

        let target = self
            .actors
            .get_mut(target_id)
            .expect("damage target checked");
        let absorbed = target.guard.min(raw_damage);
        target.guard -= absorbed;
        let effective = raw_damage - absorbed;
        let before = target.vitality;
        target.vitality = (target.vitality - effective).max(0);
        let amount = before - target.vitality;
        let defeated = !target.is_alive();
        events.push(BattleEvent::DamageApplied {
            command_id: command_id.into(),
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            amount,
        });
        if defeated {
            events.push(BattleEvent::ActorDefeated {
                command_id: command_id.into(),
                actor_id: target_id.clone(),
            });
            self.remove_effects_from_source(target_id, events);
        }
        DamageOutcome {
            amount,
            defeated,
            prevented: false,
        }
    }

    /// A7: an actor who cannot act does not hold the fight up.
    ///
    /// Before this, `Stunned` was only ever set at actor construction and no
    /// live battle ever produced one, so a stunned actor's turn simply arrived
    /// and every command for it was refused with
    /// [`BattleError::ActorIncapacitated`] -- a fight nobody could continue.
    /// Ayla's Deny Activation is the first thing in the game that stuns a
    /// living actor mid-fight, so the turn cycle has to answer for it: the turn
    /// starts, the stun is spent, the turn ends, and play moves on. That *is*
    /// "cancel one hostile's declared action for the turn".
    ///
    /// The loop is bounded by the turn order, so a battle in which every actor
    /// is stunned stops rather than spinning.
    fn advance_turn(&mut self, events: &mut Vec<BattleEvent>) {
        for _ in 0..=self.turn_order.len() {
            if !self.step_to_next_actor(events) {
                return;
            }
        }
    }

    /// One step of [`Battle::advance_turn`]. `true` when the turn it opened was
    /// skipped and the caller must step again; `false` when the battle has
    /// ended or the new active actor may act.
    fn step_to_next_actor(&mut self, events: &mut Vec<BattleEvent>) -> bool {
        let party_alive = self
            .actors
            .values()
            .any(|a| a.faction == Faction::Party && a.is_alive());
        let hostiles_alive = self
            .actors
            .values()
            .any(|a| a.faction == Faction::Hostile && a.is_alive());
        if !hostiles_alive {
            self.phase = BattlePhase::Victory;
            events.push(BattleEvent::BattleEnded { victory: true });
            return false;
        }
        if !party_alive {
            self.phase = BattlePhase::Defeat;
            events.push(BattleEvent::BattleEnded { victory: false });
            return false;
        }
        if let Some(forced_actor_id) = self.forced_next_actor.take() {
            let mut resume_index = self.turn_index + 1;
            let mut resume_round = self.round;
            if resume_index >= self.turn_order.len() {
                resume_index = 0;
                resume_round += 1;
            }
            self.forced_turn_resume = Some((resume_index, resume_round));
            self.turn_index = self
                .turn_order
                .iter()
                .position(|actor_id| actor_id == &forced_actor_id)
                .expect("forced actor belongs to turn order");
        } else if let Some((resume_index, resume_round)) = self.forced_turn_resume.take() {
            self.turn_index = resume_index;
            if resume_round > self.round {
                self.round = resume_round;
                self.begin_round(events);
            }
        } else {
            self.turn_index += 1;
            if self.turn_index >= self.turn_order.len() {
                self.turn_index = 0;
                self.round += 1;
                self.begin_round(events);
            }
        }
        self.skip_defeated_actors();
        self.phase = BattlePhase::AwaitingCommand;
        let actor_id = self.active_actor_id().expect("living actor exists").clone();
        if self.recovery_openings.remove(&actor_id).is_some() {
            events.push(BattleEvent::RecoveryOpeningExpired {
                actor_id: actor_id.clone(),
            });
        }
        events.push(BattleEvent::TurnStarted {
            round: self.round,
            actor_id: actor_id.clone(),
        });
        self.pulse_effects_for_source(&actor_id, events);
        if self.spend_incapacitation(&actor_id, events) {
            events.push(BattleEvent::TurnEnded {
                round: self.round,
                actor_id,
            });
            return true;
        }
        if self.actor(&actor_id).expect("actor exists").faction == Faction::Hostile {
            events.push(self.enemy_intent(&actor_id));
        }
        false
    }

    /// A7: whether this actor's turn is cancelled, spending one round of the
    /// `Stunned` that cancels it. The status is removed once it is used up, so
    /// a one-round stun costs exactly one turn.
    fn spend_incapacitation(&mut self, actor_id: &ActorId, events: &mut Vec<BattleEvent>) -> bool {
        let Some(actor) = self.actors.get_mut(actor_id) else {
            return false;
        };
        let Some(index) = actor
            .statuses
            .iter()
            .position(|status| status.kind == StatusKind::Stunned)
        else {
            return false;
        };
        actor.statuses[index].remaining_rounds =
            actor.statuses[index].remaining_rounds.saturating_sub(1);
        if actor.statuses[index].remaining_rounds == 0 {
            let status = actor.statuses.remove(index);
            events.push(BattleEvent::StatusRemoved {
                command_id: format!("stun.spent.{}", actor_id.0),
                actor_id: actor_id.clone(),
                status_id: status.id,
                status_kind: status.kind,
            });
        }
        true
    }

    fn skip_defeated_actors(&mut self) {
        for _ in 0..self.turn_order.len() {
            if self
                .actors
                .get(&self.turn_order[self.turn_index])
                .is_some_and(Actor::is_alive)
            {
                return;
            }
            self.turn_index = (self.turn_index + 1) % self.turn_order.len();
        }
    }

    fn enemy_intent(&self, actor_id: &ActorId) -> BattleEvent {
        let decision = self
            .enemy_decision()
            .expect("active hostile has a decision");
        debug_assert_eq!(&decision.actor_id, actor_id);
        BattleEvent::EnemyIntentDeclared {
            actor_id: decision.actor_id,
            skill_id: decision.skill_id,
            target_ids: vec![decision.target_id],
            rationale: decision.rationale,
            guard_break_amount: decision.guard_break_amount,
            raw_damage: decision.raw_damage,
            guard_absorbed: decision.guard_absorbed,
            vitality_damage: decision.vitality_damage,
            lethal: decision.lethal,
            interception_protector_id: decision.interception_protector_id,
            fatal_intercept_available: decision.fatal_intercept_available,
        }
    }
}

/// Sort key for `skill.betty.condition_cleanse`, whose `removalPriority` is
/// authored in `content/skills/betty.condition_cleanse.json` as exactly
/// `["stunned", "burning", "poisoned", "bleeding"]` and pinned by
/// `tools/src/validate.mjs`. `Shaken` is deliberately absent: Composure is not
/// restored by a rank C cleanse, so `None` marks a status the cleanse sorts
/// last and never removes.
fn cleanse_priority(kind: &StatusKind) -> Option<u8> {
    match kind {
        StatusKind::Stunned => Some(0),
        StatusKind::Burning => Some(1),
        StatusKind::Poisoned => Some(2),
        StatusKind::Bleeding => Some(3),
        // Neither is one of Condition Cleanse's four authored statuses.
        // `Staggered` sorts last and is left standing by the rank C cleanse;
        // Ayla's rank S Curse Dispel is what takes it off.
        StatusKind::Staggered | StatusKind::Shaken => None,
    }
}

/// Where `skill.captain.reposition` may step from a given stored band. The
/// Captain's Reposition is a party-line manoeuvre: `PartyRear` and `PartyFront`
/// are each other's only destination, and from any other band -- Contested,
/// either enemy band, or a stored value outside the five -- there is none, which
/// is [`BattleError::RepositionNotLegal`].
fn reposition_destination(from_band: i8) -> Option<Band> {
    match Band::from_index(from_band)? {
        Band::PartyRear => Some(Band::PartyFront),
        Band::PartyFront => Some(Band::PartyRear),
        Band::Contested | Band::EnemyFront | Band::EnemyRear => None,
    }
}

fn compare_vitality_percentage(left: &Actor, right: &Actor) -> std::cmp::Ordering {
    let left_scaled = i64::from(left.vitality) * i64::from(right.max_vitality);
    let right_scaled = i64::from(right.vitality) * i64::from(left.max_vitality);
    left_scaled.cmp(&right_scaled)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor(
        id: &str,
        faction: Faction,
        level: u8,
        vitality: i32,
        guard: i32,
        initiative: i16,
    ) -> Actor {
        Actor {
            id: ActorId(id.into()),
            display_name: id.into(),
            faction,
            level,
            vitality,
            max_vitality: vitality,
            guard,
            band: 0,
            composure: 10,
            initiative,
            statuses: Vec::new(),
            intercepts_for: None,
            skill_uses_remaining: BTreeMap::new(),
            // A10: the shared test fixture stands at the top of the ladder so
            // every test written before the bond gate existed still exercises
            // what it was written to exercise. The two tests that are *about*
            // the gate set the rank they mean.
            bond_rank: "SSS".to_owned(),
        }
    }
    fn prototype() -> Battle {
        Battle::new(
            "battle.prototype.returning_names",
            [
                actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12),
                actor("enemy.raptor.razorbeak", Faction::Hostile, 7, 70, 3, 8),
            ],
        )
    }

    #[test]
    fn guarded_strike_advances_to_declared_enemy_intent() {
        let mut battle = prototype();
        let start = battle.start();
        assert!(matches!(start[1], BattleEvent::TurnStarted { .. }));
        let events = battle
            .submit(SkillCommand {
                command_id: "command.test.1".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .unwrap()
                .vitality,
            58
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .guard,
            2
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::EnemyIntentDeclared { .. }))
        );
        assert_eq!(battle.snapshot().phase, BattlePhase::AwaitingCommand);
    }

    #[test]
    fn enemy_decision_selects_the_most_wounded_percentage_and_exposes_reaction_facts() {
        let mut battle = Battle::prototype_vertical_slice();
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "open.enemy.turn".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        let decision = battle.enemy_decision().expect("razorbeak is active");
        assert_eq!(decision.target_id, ActorId("character.heroine.vix".into()));
        assert_eq!(decision.rationale, "finish_most_wounded");
        assert_eq!(decision.raw_damage, 16);
        assert_eq!(decision.guard_absorbed, 0);
        assert_eq!(decision.vitality_damage, 16);
        assert!(decision.lethal);
        assert!(decision.interception_protector_id.is_none());
        assert!(decision.fatal_intercept_available);
        let command = battle
            .recommended_enemy_command("command.enemy.recommended")
            .expect("decision converts to a command");
        assert_eq!(command.actor_id, decision.actor_id);
        assert_eq!(command.skill_id, decision.skill_id);
        assert_eq!(command.target_ids, vec![decision.target_id]);
    }

    #[test]
    fn enemy_projection_uses_the_interceptor_as_the_damage_recipient() {
        let mut battle = Battle::prototype_vertical_slice();
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "set.interception".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.vix".into()),
                    ActorId("enemy.raptor.razorbeak".into()),
                ],
            })
            .unwrap();
        let decision = battle.enemy_decision().expect("razorbeak is active");
        assert_eq!(decision.target_id, ActorId("character.heroine.vix".into()));
        assert_eq!(
            decision.interception_protector_id,
            Some(ActorId("character.heroine.betty".into()))
        );
        assert_eq!(decision.vitality_damage, 16);
        assert!(!decision.lethal);
        assert!(!decision.fatal_intercept_available);
    }

    #[test]
    fn enemy_selects_guard_breaking_kick_and_projects_both_guard_steps() {
        let mut battle = Battle::prototype_vertical_slice();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.vix".into()))
            .expect("Vix exists")
            .guard = 10;
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "open.enemy.turn.with.guard".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();

        let decision = battle.enemy_decision().expect("razorbeak is active");
        assert_eq!(
            decision.skill_id,
            "skill.enemy.razorbeak.guard_breaking_kick"
        );
        assert_eq!(decision.rationale, "break_guard_on_target");
        assert_eq!(decision.guard_break_amount, 6);
        assert_eq!(decision.raw_damage, 13);
        assert_eq!(decision.guard_absorbed, 4);
        assert_eq!(decision.vitality_damage, 9);
        assert!(!decision.lethal);
    }

    #[test]
    fn enemy_selects_guard_breaking_kick_from_the_interceptors_guard() {
        let mut battle = Battle::prototype_vertical_slice();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.betty".into()))
            .expect("Betty exists")
            .guard = 8;
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "set.guarded.interception".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.vix".into()),
                    ActorId("enemy.raptor.razorbeak".into()),
                ],
            })
            .unwrap();

        let decision = battle.enemy_decision().expect("razorbeak is active");
        assert_eq!(
            decision.skill_id,
            "skill.enemy.razorbeak.guard_breaking_kick"
        );
        assert_eq!(decision.rationale, "break_guard_on_protector");
        assert_eq!(decision.guard_break_amount, 6);
        assert_eq!(decision.guard_absorbed, 2);
        assert_eq!(
            decision.interception_protector_id,
            Some(ActorId("character.heroine.betty".into()))
        );
    }

    #[test]
    fn guard_breaking_kick_redirects_then_breaks_guard_then_deals_damage() {
        let mut battle = Battle::prototype_vertical_slice();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.betty".into()))
            .expect("Betty exists")
            .guard = 8;
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "set.interception.for.kick".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.vix".into()),
                    ActorId("enemy.raptor.razorbeak".into()),
                ],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "enemy.guard.break".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.guard_breaking_kick".into(),
                target_ids: vec![ActorId("character.heroine.vix".into())],
            })
            .unwrap();

        let intercept_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::InterceptionTriggered { .. }))
            .expect("interception event");
        let break_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::GuardChanged { actor_id, delta: -6, total: 2, .. } if actor_id.0 == "character.heroine.betty"))
            .expect("guard break event");
        let damage_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::DamageApplied { target_id, amount: 11, .. } if target_id.0 == "character.heroine.betty"))
            .expect("damage event");
        assert!(intercept_index < break_index && break_index < damage_index);
        let betty = battle
            .actor(&ActorId("character.heroine.betty".into()))
            .expect("Betty remains");
        assert_eq!(betty.guard, 0);
        assert_eq!(betty.vitality, 89);
        assert!(betty.intercepts_for.is_none());
    }

    #[test]
    fn guard_breaking_kick_creates_a_recovery_opening_consumed_by_the_next_party_hit() {
        let mut battle = Battle::prototype_vertical_slice();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.vix".into()))
            .expect("Vix exists")
            .guard = 10;
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "open.enemy.turn.for.recovery".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        let kick_events = battle
            .submit(SkillCommand {
                command_id: "create.recovery.opening".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.guard_breaking_kick".into(),
                target_ids: vec![ActorId("character.heroine.vix".into())],
            })
            .unwrap();
        assert!(kick_events.iter().any(|event| matches!(
            event,
            BattleEvent::RecoveryOpeningCreated {
                actor_id,
                bonus_raw_damage: 6,
                ..
            } if actor_id.0 == "enemy.raptor.razorbeak"
        )));
        assert_eq!(battle.snapshot().recovery_openings.len(), 1);

        battle
            .submit(SkillCommand {
                command_id: "vix.waits".into(),
                actor_id: ActorId("character.heroine.vix".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .unwrap();
        // B5: the Captain stands in the slice and takes his turn between Vix and
        // Betty. He waits, so the opening is still Betty's to punish.
        battle
            .submit(SkillCommand {
                command_id: "michael.waits".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .unwrap();
        assert_eq!(battle.snapshot().recovery_openings.len(), 1);

        let punish_events = battle
            .submit(SkillCommand {
                command_id: "betty.punishes.recovery".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        let consumed_index = punish_events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    BattleEvent::RecoveryOpeningConsumed {
                        bonus_raw_damage: 6,
                        ..
                    }
                )
            })
            .expect("recovery opening consumed");
        let damage_index = punish_events
            .iter()
            .position(|event| matches!(event, BattleEvent::DamageApplied { target_id, amount: 21, .. } if target_id.0 == "enemy.raptor.razorbeak"))
            .expect("bonus damage applied");
        assert!(consumed_index < damage_index);
        assert!(battle.snapshot().recovery_openings.is_empty());
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .expect("Razorbeak remains")
                .vitality,
            37
        );
    }

    #[test]
    fn unused_recovery_opening_expires_before_razorbeaks_next_intent() {
        let mut battle = Battle::prototype_vertical_slice();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.vix".into()))
            .expect("Vix exists")
            .guard = 10;
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "open.enemy.turn.for.expiry".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        battle
            .submit(SkillCommand {
                command_id: "create.expiring.opening".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.guard_breaking_kick".into(),
                target_ids: vec![ActorId("character.heroine.vix".into())],
            })
            .unwrap();
        battle
            .submit(SkillCommand {
                command_id: "vix.does.not.punish".into(),
                actor_id: ActorId("character.heroine.vix".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .unwrap();
        // B5: the Captain's turn now sits between Vix's and Betty's; he does not
        // punish the opening either.
        battle
            .submit(SkillCommand {
                command_id: "michael.does.not.punish".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "betty.does.not.punish".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .unwrap();
        let expired_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::RecoveryOpeningExpired { actor_id } if actor_id.0 == "enemy.raptor.razorbeak"))
            .expect("recovery opening expired");
        let intent_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::EnemyIntentDeclared { .. }))
            .expect("next enemy intent declared");
        assert!(expired_index < intent_index);
        assert!(battle.snapshot().recovery_openings.is_empty());
    }
    #[test]
    fn rejects_out_of_turn_commands_without_mutation() {
        let mut battle = prototype();
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "bad".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::NotActiveActor { .. }));
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn rejects_unsupported_skills_without_mutation() {
        let mut battle = prototype();
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "unsupported".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.not_real".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::UnsupportedSkill("skill.betty.not_real".into())
        );
        assert_eq!(battle.snapshot(), before);
    }
    #[test]
    fn ends_when_last_hostile_falls() {
        let mut battle = Battle::new(
            "battle.test",
            [
                actor("character.heroine.betty", Faction::Party, 20, 100, 0, 12),
                actor("enemy.test", Faction::Hostile, 1, 5, 0, 8),
            ],
        );
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "finish".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .unwrap();
        assert!(events.contains(&BattleEvent::BattleEnded { victory: true }));
        assert_eq!(battle.snapshot().phase, BattlePhase::Victory);
    }

    #[test]
    fn condition_cleanse_removes_two_most_urgent_statuses_and_heals_without_overflow() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 95, 0, 10);
        ayla.max_vitality = 100;
        ayla.statuses = vec![
            StatusInstance {
                id: "status.ayla.bleeding".into(),
                kind: StatusKind::Bleeding,
                remaining_rounds: 3,
                source_id: ActorId("enemy.test".into()),
            },
            StatusInstance {
                id: "status.ayla.stunned".into(),
                kind: StatusKind::Stunned,
                remaining_rounds: 1,
                source_id: ActorId("enemy.test".into()),
            },
            StatusInstance {
                id: "status.ayla.poisoned".into(),
                kind: StatusKind::Poisoned,
                remaining_rounds: 4,
                source_id: ActorId("enemy.test".into()),
            },
        ];
        betty.band = Band::PartyFront.index();
        ayla.band = Band::PartyFront.index();
        let mut battle = Battle::new(
            "battle.cleanse",
            [
                betty,
                ayla,
                actor("enemy.test", Faction::Hostile, 3, 50, 0, 8),
            ],
        );
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "cleanse".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.condition_cleanse".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        let ayla = battle
            .actor(&ActorId("character.heroine.ayla".into()))
            .unwrap();
        assert_eq!(ayla.vitality, 100);
        assert_eq!(ayla.statuses.len(), 1);
        assert_eq!(ayla.statuses[0].kind, StatusKind::Bleeding);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, BattleEvent::StatusRemoved { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn rescue_charge_moves_betty_sets_interception_and_hits_named_hostile() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyRear.index();
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 10);
        ayla.band = Band::PartyFront.index();
        let enemy = actor("enemy.test", Faction::Hostile, 4, 50, 3, 8);
        let mut battle = Battle::new("battle.rescue", [betty, ayla, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "rescue".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.ayla".into()),
                    ActorId("enemy.test".into()),
                ],
            })
            .unwrap();
        let betty = battle
            .actor(&ActorId("character.heroine.betty".into()))
            .unwrap();
        assert_eq!(betty.band_kind(), Some(Band::PartyFront));
        assert_eq!(
            betty.intercepts_for,
            Some(ActorId("character.heroine.ayla".into()))
        );
        assert_eq!(
            battle
                .actor(&ActorId("enemy.test".into()))
                .unwrap()
                .vitality,
            43
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::InterceptionSet { .. }))
        );
    }

    #[test]
    fn rescue_interception_redirects_exactly_one_enemy_attack() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyRear.index();
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 8);
        ayla.band = Band::PartyFront.index();
        let enemy = actor("enemy.raptor.razorbeak", Faction::Hostile, 4, 50, 0, 10);
        let mut battle = Battle::new("battle.intercept", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "rescue".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.ayla".into()),
                    ActorId("enemy.raptor.razorbeak".into()),
                ],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            80
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .vitality,
            87
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .intercepts_for,
            None
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .skill_uses_remaining["skill.betty.fatal_intercept"],
            1
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, BattleEvent::ReactionTriggered { .. }))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::InterceptionTriggered { .. }))
        );
    }

    #[test]
    fn healing_impact_uses_actual_post_guard_damage_and_lowest_percentage() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 60, 0, 12);
        betty.max_vitality = 100;
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 40, 0, 10);
        ayla.max_vitality = 80;
        let enemy = actor("enemy.test", Faction::Hostile, 4, 70, 5, 8);
        let mut battle = Battle::new("battle.healing_impact", [betty, ayla, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "impact".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.healing_impact".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("enemy.test".into()))
                .unwrap()
                .vitality,
            54
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            48
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .vitality,
            60
        );
        let damage_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::DamageApplied { .. }))
            .unwrap();
        let healing_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::VitalityChanged { .. }))
            .unwrap();
        assert!(damage_index < healing_index);
    }

    #[test]
    fn healing_impact_heals_before_victory_and_uses_floor_rounding() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 50, 0, 12);
        betty.max_vitality = 100;
        let enemy = actor("enemy.test", Faction::Hostile, 1, 5, 0, 8);
        let mut battle = Battle::new("battle.healing_impact_victory", [betty, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "impact.finish".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.healing_impact".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .vitality,
            52
        );
        let heal_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::VitalityChanged { .. }))
            .unwrap();
        let victory_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::BattleEnded { victory: true }))
            .unwrap();
        assert!(heal_index < victory_index);
    }

    #[test]
    fn fatal_intercept_cancels_lethal_damage_spends_charge_and_counters() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        let ayla = actor("character.heroine.ayla", Faction::Party, 3, 10, 0, 8);
        let enemy = actor("enemy.raptor.razorbeak", Faction::Hostile, 4, 50, 0, 10);
        let mut battle = Battle::new("battle.fatal_intercept", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "betty.turn".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "lethal.bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            10
        );
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .unwrap()
                .vitality,
            11
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .skill_uses_remaining["skill.betty.fatal_intercept"],
            0
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::ReactionWindowOpened { .. }))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::DefeatPrevented { .. }))
        );
    }

    #[test]
    fn fatal_intercept_does_not_trigger_while_betty_is_stunned() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        betty.statuses.push(StatusInstance {
            id: "status.betty.stunned".into(),
            kind: StatusKind::Stunned,
            remaining_rounds: 1,
            source_id: ActorId("enemy.test".into()),
        });
        let ayla = actor("character.heroine.ayla", Faction::Party, 3, 10, 0, 8);
        let enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 10);
        let mut battle = Battle::new("battle.stunned_reaction", [betty, ayla, enemy]);
        let mut events = Vec::new();
        let outcome = battle.apply_damage(
            "test.lethal",
            &ActorId("enemy.test".into()),
            &ActorId("character.heroine.ayla".into()),
            20,
            true,
            &mut events,
        );
        assert!(outcome.defeated);
        assert!(!outcome.prevented);
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            0
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, BattleEvent::ReactionTriggered { .. }))
        );
    }

    #[test]
    fn fatal_intercept_is_spent_and_second_lethal_hit_resolves_normally() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        let ayla = actor("character.heroine.ayla", Faction::Party, 3, 10, 0, 8);
        let enemy = actor("enemy.test", Faction::Hostile, 4, 100, 0, 10);
        let mut battle = Battle::new("battle.single_reaction", [betty, ayla, enemy]);
        let mut first_events = Vec::new();
        let first = battle.apply_damage(
            "first",
            &ActorId("enemy.test".into()),
            &ActorId("character.heroine.ayla".into()),
            20,
            true,
            &mut first_events,
        );
        assert!(first.prevented);
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            10
        );
        let mut second_events = Vec::new();
        let second = battle.apply_damage(
            "second",
            &ActorId("enemy.test".into()),
            &ActorId("character.heroine.ayla".into()),
            20,
            true,
            &mut second_events,
        );
        assert!(!second.prevented);
        assert!(second.defeated);
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            0
        );
        assert!(
            !second_events
                .iter()
                .any(|event| matches!(event, BattleEvent::ReactionTriggered { .. }))
        );
    }

    #[test]
    fn mobile_infirmary_pulses_immediately_and_on_two_future_betty_turns() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 50, 0, 12);
        betty.max_vitality = 100;
        let enemy = actor("enemy.raptor.razorbeak", Faction::Hostile, 1, 200, 0, 8);
        let mut battle = Battle::new("battle.mobile_infirmary", [betty, enemy]);
        battle.start();
        let cast_events = battle
            .submit(SkillCommand {
                command_id: "infirmary.cast".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.mobile_infirmary".into(),
                target_ids: vec![],
            })
            .unwrap();
        assert_eq!(battle.snapshot().effects[0].remaining_pulses, 2);
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .vitality,
            60
        );
        assert!(matches!(
            cast_events
                .iter()
                .find(|event| matches!(event, BattleEvent::BattlefieldEffectPulse { .. })),
            Some(BattleEvent::BattlefieldEffectPulse {
                pulses_remaining_after: 2,
                ..
            })
        ));

        battle
            .submit(SkillCommand {
                command_id: "enemy.one".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap();
        assert_eq!(battle.snapshot().effects[0].remaining_pulses, 1);

        battle
            .submit(SkillCommand {
                command_id: "betty.two".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap();
        let final_pulse_events = battle
            .submit(SkillCommand {
                command_id: "enemy.two".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap();
        assert!(battle.snapshot().effects.is_empty());
        assert!(final_pulse_events.iter().any(|event| matches!(
            event,
            BattleEvent::BattlefieldEffectRemoved { reason, .. }
                if reason == "duration_completed"
        )));
    }

    #[test]
    fn mobile_infirmary_affects_each_living_party_member_in_actor_id_order() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 50, 0, 12);
        betty.max_vitality = 100;
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 30, 1, 10);
        ayla.max_vitality = 80;
        let defeated = actor("character.heroine.vix", Faction::Party, 3, 0, 0, 9);
        let enemy = actor("enemy.test", Faction::Hostile, 1, 100, 0, 8);
        let mut battle = Battle::new("battle.mobile_party", [betty, ayla, defeated, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "infirmary.party".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.mobile_infirmary".into(),
                target_ids: vec![],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .vitality,
            40
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .guard,
            3
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.vix".into()))
                .unwrap()
                .vitality,
            0
        );
        let healed: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                BattleEvent::VitalityChanged { actor_id, .. } => Some(actor_id.0.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            healed,
            vec!["character.heroine.ayla", "character.heroine.betty"]
        );
    }

    #[test]
    fn stunned_actor_cannot_submit_a_command_and_state_does_not_mutate() {
        let mut battle = prototype();
        battle
            .actors
            .get_mut(&ActorId("character.heroine.betty".into()))
            .unwrap()
            .statuses
            .push(StatusInstance {
                id: "status.stunned".into(),
                kind: StatusKind::Stunned,
                remaining_rounds: 1,
                source_id: ActorId("enemy.test".into()),
            });
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "stunned.command".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::ActorIncapacitated { .. }));
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn actor_cannot_submit_another_actors_signature_skill() {
        let betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 8);
        let ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 12);
        let enemy = actor("enemy.test", Faction::Hostile, 1, 100, 0, 6);
        let mut battle = Battle::new("battle.skill_owner", [betty, ayla, enemy]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "wrong.owner".into(),
                actor_id: ActorId("character.heroine.ayla".into()),
                skill_id: "skill.betty.mobile_infirmary".into(),
                target_ids: vec![],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::SkillOwnerMismatch { .. }));
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn combat_revival_restores_ceiling_percent_clears_statuses_and_spends_use() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 0, 7, 10);
        ayla.max_vitality = 83;
        ayla.statuses.push(StatusInstance {
            id: "status.ayla.poisoned".into(),
            kind: StatusKind::Poisoned,
            remaining_rounds: 3,
            source_id: ActorId("enemy.test".into()),
        });
        let enemy = actor("enemy.test", Faction::Hostile, 1, 100, 0, 8);
        let mut battle = Battle::new("battle.combat_revival", [betty, ayla, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "revive.ayla".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        let ayla = battle
            .actor(&ActorId("character.heroine.ayla".into()))
            .unwrap();
        assert_eq!(ayla.vitality, 34);
        assert_eq!(ayla.guard, 0);
        assert!(ayla.statuses.is_empty());
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .skill_uses_remaining["skill.betty.combat_revival"],
            0
        );
        assert_eq!(
            battle.active_actor_id(),
            Some(&ActorId("character.heroine.ayla".into()))
        );
        let removed_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::StatusRemoved { .. }))
            .unwrap();
        let revived_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::ActorRevived { .. }))
            .unwrap();
        let bonus_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::BonusTurnGranted { .. }))
            .unwrap();
        let turn_index = events
            .iter()
            .position(|event| matches!(event, BattleEvent::TurnStarted { actor_id, .. } if actor_id.0 == "character.heroine.ayla"))
            .unwrap();
        assert!(removed_index < revived_index);
        assert!(revived_index < bonus_index);
        assert!(bonus_index < turn_index);
    }

    #[test]
    fn combat_revival_bonus_turn_resumes_the_natural_successor_and_round() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 0, 0, 10);
        ayla.max_vitality = 80;
        let enemy = actor("enemy.test", Faction::Hostile, 1, 100, 0, 8);
        let mut battle = Battle::new("battle.bonus_resume", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "revive.bonus".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        assert_eq!(battle.snapshot().forced_turn_resume, Some((1, 1)));
        let mut events = Vec::new();
        battle.advance_turn(&mut events);
        assert_eq!(battle.round, 1);
        assert_eq!(
            battle.active_actor_id(),
            Some(&ActorId("character.heroine.ayla".into()))
        );

        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 5);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 0, 0, 15);
        ayla.max_vitality = 80;
        let enemy = actor("enemy.raptor.razorbeak", Faction::Hostile, 1, 100, 0, 10);
        let mut battle = Battle::new("battle.bonus_wrap", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "enemy.first".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap();
        battle
            .submit(SkillCommand {
                command_id: "revive.wrap".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        let mut resume_events = Vec::new();
        battle.advance_turn(&mut resume_events);
        assert_eq!(battle.round, 2);
        assert!(
            resume_events
                .iter()
                .any(|event| matches!(event, BattleEvent::RoundStarted { round: 2 }))
        );
    }

    #[test]
    fn combat_revival_rejects_living_targets_and_missing_use_without_mutation() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        let ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 10);
        let enemy = actor("enemy.test", Faction::Hostile, 1, 100, 0, 8);
        let mut battle = Battle::new("battle.revival_rejection", [betty, ayla, enemy]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "revive.living".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::TargetMustBeDefeated(_)));
        assert_eq!(battle.snapshot(), before);

        battle
            .actors
            .get_mut(&ActorId("character.heroine.ayla".into()))
            .unwrap()
            .vitality = 0;
        battle
            .actors
            .get_mut(&ActorId("character.heroine.betty".into()))
            .unwrap()
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 0);
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "revive.spent".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::SkillUnavailable { .. }));
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn resubmitting_the_same_command_id_returns_the_recorded_result_without_double_resolving() {
        let mut battle = prototype();
        battle.start();
        let command = SkillCommand {
            command_id: "command.idempotent.1".into(),
            actor_id: ActorId("character.heroine.betty".into()),
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
        };
        let first = battle.submit(command.clone()).unwrap();
        let vitality_after_first = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .unwrap()
            .vitality;
        let second = battle.submit(command).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .unwrap()
                .vitality,
            vitality_after_first
        );
    }

    #[test]
    fn retreat_ends_the_battle_for_an_eligible_party_actor() {
        let mut battle = prototype();
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "command.retreat.1".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.system.retreat".into(),
                target_ids: vec![],
            })
            .unwrap();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::BattleRetreated { .. }))
        );
        assert_eq!(battle.snapshot().phase, BattlePhase::Retreated);
        let error = battle
            .submit(SkillCommand {
                command_id: "command.after_retreat".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak".into())],
            })
            .unwrap_err();
        assert_eq!(error, BattleError::BattleAlreadyEnded);
    }

    #[test]
    fn retreat_is_rejected_when_the_encounter_forbids_it_without_mutation() {
        let mut battle = prototype().with_retreat_allowed(false);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "command.retreat.forbidden".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.system.retreat".into(),
                target_ids: vec![],
            })
            .unwrap_err();
        assert_eq!(error, BattleError::RetreatNotAllowed);
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn retreat_is_rejected_for_a_hostile_actor_without_mutation() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 20);
        let enemy = actor("enemy.raptor.razorbeak", Faction::Hostile, 7, 70, 3, 30);
        betty.initiative = 1;
        let mut battle = Battle::new("battle.retreat_hostile", [betty, enemy]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "command.retreat.hostile".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.system.retreat".into(),
                target_ids: vec![],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::IllegalRetreatActor(ActorId("enemy.raptor.razorbeak".into()))
        );
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn grasping_strike_deals_the_shared_enemy_strike_damage_through_guard() {
        let betty = actor("character.heroine.betty", Faction::Party, 3, 100, 5, 12);
        let wight = actor(
            "enemy.undead.tomb_wight.prototype",
            Faction::Hostile,
            6,
            50,
            0,
            20,
        );
        let mut battle = Battle::new("battle.grasping_strike", [wight, betty]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "grasp".into(),
                actor_id: ActorId("enemy.undead.tomb_wight.prototype".into()),
                skill_id: "skill.enemy.undead.grasping_strike".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap();
        // 9 + level(6) = 15 raw damage; 5 Guard absorbed, 10 gets through.
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::DamageApplied { target_id, amount: 10, .. }
                if target_id.0 == "character.heroine.betty"
        )));
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .vitality,
            90
        );
    }

    #[test]
    fn dread_gaze_strips_up_to_four_guard_then_hits_through_the_remainder() {
        let betty = actor("character.heroine.betty", Faction::Party, 3, 100, 6, 12);
        let tide_spawn = actor(
            "enemy.eldritch.tide_spawn.prototype",
            Faction::Hostile,
            9,
            60,
            0,
            20,
        );
        let mut battle = Battle::new("battle.dread_gaze", [tide_spawn, betty]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "gaze".into(),
                actor_id: ActorId("enemy.eldritch.tide_spawn.prototype".into()),
                skill_id: "skill.enemy.eldritch.dread_gaze".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap();
        // Guard 6, stripped by 4, leaves 2. Raw damage 9 + level(9) = 18; the
        // remaining 2 Guard absorbs 2 (and is itself spent doing so), 16 gets
        // through.
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::GuardChanged {
                delta: -4,
                total: 2,
                ..
            }
        )));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::DamageApplied { amount: 16, .. }))
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .guard,
            0
        );
    }

    #[test]
    fn a_party_actor_cannot_submit_a_shared_enemy_skill_without_mutation() {
        let betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        let wight = actor(
            "enemy.undead.tomb_wight.prototype",
            Faction::Hostile,
            6,
            50,
            0,
            5,
        );
        let mut battle = Battle::new("battle.hostile_lock", [betty, wight]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "illegal".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.enemy.undead.grasping_strike".into(),
                target_ids: vec![ActorId("enemy.undead.tomb_wight.prototype".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::HostileSkillUsedByNonHostile {
                skill_id: "skill.enemy.undead.grasping_strike".into(),
                actor_id: ActorId("character.heroine.betty".into()),
            }
        );
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn a_shared_enemy_skill_can_be_redirected_onto_an_active_interceptor() {
        let betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 30);
        let vix = actor("character.heroine.vix", Faction::Party, 3, 80, 0, 1);
        let wight = actor(
            "enemy.undead.tomb_wight.prototype",
            Faction::Hostile,
            6,
            50,
            0,
            20,
        );
        let mut battle = Battle::new("battle.grasp_redirect", [betty, vix, wight]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "set.interception".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.vix".into()),
                    ActorId("enemy.undead.tomb_wight.prototype".into()),
                ],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "grasp.redirect".into(),
                actor_id: ActorId("enemy.undead.tomb_wight.prototype".into()),
                skill_id: "skill.enemy.undead.grasping_strike".into(),
                target_ids: vec![ActorId("character.heroine.vix".into())],
            })
            .unwrap();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::InterceptionTriggered { .. }))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::DamageApplied { target_id, .. } if target_id.0 == "character.heroine.betty"
        )));
    }

    #[test]
    fn every_band_round_trips_through_from_index_and_has_a_name() {
        let bands = [
            (0i8, Band::PartyRear, "party_rear"),
            (1, Band::PartyFront, "party_front"),
            (2, Band::Contested, "contested"),
            (3, Band::EnemyFront, "enemy_front"),
            (4, Band::EnemyRear, "enemy_rear"),
        ];
        for (index, band, name) in bands {
            assert_eq!(Band::from_index(index), Some(band));
            assert_eq!(band.index(), index);
            assert_eq!(band.name(), name);
        }
        assert_eq!(Band::from_index(-1), None);
        assert_eq!(Band::from_index(5), None);
        assert_eq!(Band::from_index(i8::MAX), None);
    }

    #[test]
    fn prototype_fixture_places_every_actor_in_a_named_band() {
        let battle = Battle::prototype_vertical_slice();
        let band_of = |id: &str| {
            battle
                .actor(&ActorId(id.into()))
                .expect("fixture actor exists")
                .band_kind()
        };
        assert_eq!(band_of("character.heroine.betty"), Some(Band::PartyFront));
        assert_eq!(band_of("character.heroine.vix"), Some(Band::PartyFront));
        assert_eq!(band_of("character.heroine.ayla"), Some(Band::PartyRear));
        assert_eq!(band_of("enemy.raptor.razorbeak"), Some(Band::EnemyFront));
        assert_eq!(
            band_of("character.protagonist.captain"),
            Some(Band::PartyRear)
        );
    }

    /// B5: the slice holds five actors because the Captain stands in it. The
    /// count is asserted here, and in `native_simulation_port_test.gd`, so that
    /// dropping him from the fixture is a failure with a reason rather than a
    /// silently smaller battle.
    #[test]
    fn the_prototype_fixture_stands_five_actors_including_the_captain() {
        let battle = Battle::prototype_vertical_slice();
        assert_eq!(
            battle.snapshot().actors.len(),
            5,
            "the slice holds Betty, Ayla, Vix, the Razorbeak and Captain Michael"
        );
        assert!(
            battle
                .actor(&ActorId("character.protagonist.captain".into()))
                .is_some(),
            "B5 stands Captain Michael in the prototype vertical slice"
        );
    }

    #[test]
    fn prototype_fixture_starts_every_actor_at_full_composure() {
        let battle = Battle::prototype_vertical_slice();
        for actor in battle.snapshot().actors {
            assert_eq!(actor.composure, 10, "{}", actor.id.0);
            assert!(!actor.is_shaken(), "{}", actor.id.0);
        }
    }

    #[test]
    fn spending_composure_to_zero_applies_shaken_exactly_once() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        assert_eq!(betty.composure, 10);

        betty.spend_composure(4);
        assert_eq!(betty.composure, 6);
        assert!(!betty.is_shaken());

        betty.spend_composure(6);
        assert_eq!(betty.composure, 0);
        assert_eq!(
            betty
                .statuses
                .iter()
                .filter(|status| status.kind == StatusKind::Shaken)
                .count(),
            1
        );

        // Saturates at zero, and does not stack a second Shaken.
        betty.spend_composure(200);
        assert_eq!(betty.composure, 0);
        assert_eq!(
            betty
                .statuses
                .iter()
                .filter(|status| status.kind == StatusKind::Shaken)
                .count(),
            1
        );
    }

    #[test]
    fn skill_rank_reads_the_authored_bond_ranks() {
        assert_eq!(skill_rank("skill.betty.guarded_strike"), Some("D"));
        assert_eq!(skill_rank("skill.betty.condition_cleanse"), Some("C"));
        assert_eq!(skill_rank("skill.betty.rescue_charge"), Some("B"));
        assert_eq!(skill_rank("skill.betty.healing_impact"), Some("A"));
        assert_eq!(skill_rank("skill.betty.fatal_intercept"), Some("S"));
        assert_eq!(skill_rank("skill.betty.mobile_infirmary"), Some("SS"));
        assert_eq!(skill_rank("skill.betty.combat_revival"), Some("SSS"));
        assert_eq!(skill_rank("skill.system.hold_position"), None);
        assert_eq!(skill_rank("skill.betty.not_real"), None);
    }

    fn shaken_betty_battle() -> Battle {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyFront.index();
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        betty.spend_composure(10);
        assert!(betty.is_shaken());
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 3, 8);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.shaken", [betty, enemy]);
        battle.start();
        battle
    }

    #[test]
    fn shaken_betty_cannot_submit_an_sss_command() {
        let mut battle = shaken_betty_battle();
        let mut fallen = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 4);
        fallen.vitality = 0;
        let error = battle
            .submit(SkillCommand {
                command_id: "revive".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::ShakenCannotUse {
                skill_id: "skill.betty.combat_revival".into(),
                actor_id: ActorId("character.heroine.betty".into()),
            }
        );
    }

    #[test]
    fn shaken_betty_cannot_submit_an_ss_command() {
        let mut battle = shaken_betty_battle();
        let error = battle
            .submit(SkillCommand {
                command_id: "infirmary".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.mobile_infirmary".into(),
                target_ids: Vec::new(),
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::ShakenCannotUse { .. }));
    }

    #[test]
    fn shaken_betty_can_still_submit_her_rank_d_command() {
        let mut battle = shaken_betty_battle();
        let events = battle
            .submit(SkillCommand {
                command_id: "strike".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .expect("a rank D command survives Shaken");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::DamageApplied { .. }))
        );
    }

    #[test]
    fn an_unshaken_betty_may_still_use_her_sss_command() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);
        assert!(!betty.is_shaken());
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 4);
        ayla.vitality = 0;
        let enemy = actor("enemy.test", Faction::Hostile, 4, 50, 3, 8);
        let mut battle = Battle::new("battle.unshaken", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "revive".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.combat_revival".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .expect("Composure intact, so the SSS command is legal");
    }

    #[test]
    fn condition_cleanse_leaves_shaken_alone() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyFront.index();
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 100, 0, 10);
        ayla.band = Band::PartyFront.index();
        ayla.spend_composure(10);
        ayla.statuses.push(StatusInstance {
            id: "status.ayla.bleeding".into(),
            kind: StatusKind::Bleeding,
            remaining_rounds: 2,
            source_id: ActorId("enemy.test".into()),
        });
        let mut battle = Battle::new(
            "battle.cleanse.shaken",
            [
                betty,
                ayla,
                actor("enemy.test", Faction::Hostile, 3, 50, 0, 8),
            ],
        );
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "cleanse".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.condition_cleanse".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap();
        let ayla = battle
            .actor(&ActorId("character.heroine.ayla".into()))
            .unwrap();
        assert!(
            ayla.is_shaken(),
            "a rank C cleanse does not restore Composure"
        );
        assert_eq!(ayla.statuses.len(), 1);
        assert_eq!(ayla.statuses[0].kind, StatusKind::Shaken);
    }

    /// A10: a battle in which Betty may cleanse Ayla, with Betty's bond rank
    /// left for the caller to set. Everything else is identical between the two
    /// tests below, so the only thing that differs is where her bond stands.
    fn cleanse_battle(betty_bond_rank: &str) -> Battle {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyFront.index();
        betty_bond_rank.clone_into(&mut betty.bond_rank);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 100, 0, 10);
        ayla.band = Band::PartyFront.index();
        ayla.statuses.push(StatusInstance {
            id: "status.ayla.bleeding".into(),
            kind: StatusKind::Bleeding,
            remaining_rounds: 2,
            source_id: ActorId("enemy.test".into()),
        });
        let mut battle = Battle::new(
            "battle.bond_rank",
            [
                betty,
                ayla,
                actor("enemy.test", Faction::Hostile, 3, 50, 0, 8),
            ],
        );
        battle.start();
        battle
    }

    /// A10, the refusal. Condition Cleanse is authored at rank C; a Betty whose
    /// bond still stands at D cannot spend it, and the refusal lands before
    /// anything moves -- Ayla still bleeds and the turn is still Betty's.
    #[test]
    fn betty_at_bond_rank_d_cannot_spend_her_rank_c_command() {
        let mut battle = cleanse_battle("D");
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "cleanse".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.condition_cleanse".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::BondRankTooLow {
                skill_id: "skill.betty.condition_cleanse".into(),
                required: "C".into(),
                current: "D".into(),
            }
        );
        let after = battle.snapshot();
        assert_eq!(before.actors, after.actors, "the refusal mutated an actor");
        assert_eq!(before.round, after.round);
        assert_eq!(before.phase, after.phase);
        assert_eq!(before.active_actor_id, after.active_actor_id);
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .statuses
                .len(),
            1,
            "the bleed the refused cleanse would have removed is still there"
        );
        // Her rank D command is untouched by the gate: it is the rank C one
        // that is above her, not the fight.
        battle
            .submit(SkillCommand {
                command_id: "strike".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .expect("a rank D command is hers from the first fight");
    }

    /// A10, the other way. The same command, the same battle, once an authored
    /// scene has carried her to C.
    #[test]
    fn betty_at_bond_rank_c_may_spend_her_rank_c_command() {
        let mut battle = cleanse_battle("C");
        battle
            .submit(SkillCommand {
                command_id: "cleanse".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.condition_cleanse".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .expect("a rank C command resolves once her bond stands at C");
        assert!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .unwrap()
                .statuses
                .is_empty(),
            "the cleanse resolved exactly as it did before the gate existed"
        );
    }

    /// The ladder is the seven authored letters and nothing else, and an
    /// unrecognised stored letter reads as the floor rather than as the top --
    /// a malformed save must not unlock a rank SSS command.
    #[test]
    fn the_bond_ladder_is_the_seven_authored_letters() {
        let ladder = ["D", "C", "B", "A", "S", "SS", "SSS"];
        for (index, letter) in ladder.iter().enumerate() {
            assert_eq!(rank_index(letter), Some(index as u8), "{letter}");
        }
        assert_eq!(rank_index("E"), None);
        assert_eq!(rank_index("SSSS"), None);
        assert_eq!(rank_index("1"), None);
        assert_eq!(rank_index("d"), None);
        assert_eq!(rank_index(STARTING_BOND_RANK), Some(0));

        let mut battle = cleanse_battle("SSSS");
        let error = battle
            .submit(SkillCommand {
                command_id: "cleanse".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.condition_cleanse".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .unwrap_err();
        assert!(matches!(error, BattleError::BondRankTooLow { .. }));
    }

    /// The Composure gate decides which commands a Shaken actor may spend, and
    /// it decides on rank. `content/skills/*.json` authors `bondRank`;
    /// [`skill_rank`] mirrors it so the gate does not read the disk mid-battle.
    /// Two tables answering one question is the fork AGENTS.md section 0
    /// forbids, so this holds them equal in both directions: every authored
    /// skill must have a rank here, and it must be the authored one.
    ///
    /// It is not hypothetical. The table shipped already missing Michael's two
    /// commands, authored in the same parallel round. Both are rank D, so the
    /// gate behaved the same and nothing failed -- which is exactly why an SS
    /// skill added the same way would have slipped through in silence.
    #[test]
    fn every_authored_skill_has_its_authored_rank() {
        let skills_directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/skills/");
        let mut checked = 0;
        for entry in std::fs::read_dir(skills_directory).expect("content/skills/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("the skill file is readable");
            let record: serde_json::Value =
                serde_json::from_str(&text).expect("the skill file is JSON");
            let id = record["id"]
                .as_str()
                .expect("an authored skill declares a string id");
            let authored = record["bondRank"]
                .as_str()
                .unwrap_or_else(|| panic!("{id} declares a string bondRank"));
            assert_eq!(
                skill_rank(id),
                Some(authored),
                "{id} is authored at rank {authored}; skill_rank disagrees. \
                 content/skills/ is the owner -- add or correct the arm."
            );
            checked += 1;
        }
        assert!(
            checked >= 9,
            "only {checked} skill records were read; the content path is wrong"
        );
    }

    // ---- A6: Captain Michael as a battle actor -------------------------------

    /// The Captain as the brief describes him: `character.protagonist.captain`,
    /// Party, level 3, 90 vitality, no guard, initiative 10, standing in
    /// `PartyRear`.
    ///
    /// B5: read out of `prototype_vertical_slice()` rather than restated here.
    /// A6 had to write his stats a second time because he was not in the slice;
    /// now that he is, two copies would be free to drift, so there is one --
    /// which is also why removing him from the slice fails every A6 test below.
    fn captain() -> Actor {
        Battle::prototype_vertical_slice()
            .actor(&ActorId("character.protagonist.captain".into()))
            .expect("prototype_vertical_slice() stands the Captain in the slice")
            .clone()
    }

    /// The authored record behind a skill id. `content/skills/` owns every
    /// number the two Captain commands use; these tests read it rather than
    /// restating it, so a change to the record fails here instead of drifting.
    fn authored_skill(skill_id: &str) -> serde_json::Value {
        let file = skill_id
            .strip_prefix("skill.")
            .expect("a skill id starts with skill.");
        let path = format!(
            "{}/../content/skills/{file}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{path} is readable: {error}"));
        serde_json::from_str(&text).expect("the skill record is JSON")
    }

    /// Michael alone against one hostile, with initiative arranged so that the
    /// Captain has the first turn.
    fn captain_battle() -> Battle {
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 8);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.captain", [captain(), enemy]);
        battle.start();
        battle
    }

    #[test]
    fn captain_weapon_attack_deals_its_authored_damage() {
        let record = authored_skill("skill.captain.weapon_attack");
        assert_eq!(record["ownerId"], "character.protagonist.captain");
        assert_eq!(record["targetRule"], "one_hostile");
        let base = record["rules"]["damageBase"]
            .as_i64()
            .expect("damageBase is authored") as i32;
        let level_scale = record["rules"]["damageLevelScale"]
            .as_i64()
            .expect("damageLevelScale is authored") as i32;
        let guard_gain = record["rules"]["guardGain"]
            .as_i64()
            .expect("guardGain is authored") as i32;

        let mut battle = captain_battle();
        let michael_id = ActorId("character.protagonist.captain".into());
        let enemy_id = ActorId("enemy.test".into());
        let level = i32::from(battle.actor(&michael_id).unwrap().level);
        let enemy_before = battle.actor(&enemy_id).unwrap().vitality;
        let events = battle
            .submit(SkillCommand {
                command_id: "captain.attack".into(),
                actor_id: michael_id.clone(),
                skill_id: "skill.captain.weapon_attack".into(),
                target_ids: vec![enemy_id.clone()],
            })
            .expect("the Captain's baseline command is legal");

        let expected = base + level_scale * level;
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::DamageApplied { amount, .. } if *amount == expected
        )));
        assert_eq!(
            battle.actor(&enemy_id).unwrap().vitality,
            enemy_before - expected
        );
        assert_eq!(battle.actor(&michael_id).unwrap().guard, guard_gain);
    }

    #[test]
    fn captain_reposition_moves_one_band_and_gains_its_authored_guard() {
        let record = authored_skill("skill.captain.reposition");
        assert_eq!(record["ownerId"], "character.protagonist.captain");
        assert_eq!(record["targetRule"], "self");
        assert_eq!(record["rules"]["endsTurn"], true);
        let movement_bands = record["rules"]["movementBands"]
            .as_i64()
            .expect("movementBands is authored") as i8;
        let guard_gain = record["rules"]["guardGain"]
            .as_i64()
            .expect("guardGain is authored") as i32;
        let damage_base = record["rules"]["damageBase"]
            .as_i64()
            .expect("damageBase is authored") as i32;
        assert_eq!(damage_base, 0, "Reposition is authored to deal no damage");

        let michael_id = ActorId("character.protagonist.captain".into());
        let enemy_id = ActorId("enemy.test".into());
        let mut battle = captain_battle();
        let enemy_before = battle.actor(&enemy_id).unwrap().vitality;
        let events = battle
            .submit(SkillCommand {
                command_id: "captain.reposition.forward".into(),
                actor_id: michael_id.clone(),
                skill_id: "skill.captain.reposition".into(),
                target_ids: Vec::new(),
            })
            .expect("PartyRear steps to PartyFront");
        let michael = battle.actor(&michael_id).unwrap();
        assert_eq!(michael.band_kind(), Some(Band::PartyFront));
        assert_eq!(michael.guard, guard_gain);
        assert_eq!(
            (michael.band - Band::PartyRear.index()).abs(),
            movement_bands,
            "the authored movementBands is how far Reposition actually moves"
        );
        assert_eq!(
            battle.actor(&enemy_id).unwrap().vitality,
            enemy_before,
            "Reposition deals no damage"
        );
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::ActorMoved { from_band, to_band, .. }
                if *from_band == Band::PartyRear.index() && *to_band == Band::PartyFront.index()
        )));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::TurnEnded { .. }))
        );

        // ...and back the other way, which is the only other legal step.
        let mut michael = captain();
        michael.band = Band::PartyFront.index();
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 8);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.captain.back", [michael, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "captain.reposition.back".into(),
                actor_id: michael_id.clone(),
                skill_id: "skill.captain.reposition".into(),
                target_ids: Vec::new(),
            })
            .expect("PartyFront steps back to PartyRear");
        assert_eq!(
            battle.actor(&michael_id).unwrap().band_kind(),
            Some(Band::PartyRear)
        );
    }

    #[test]
    fn captain_reposition_from_a_forbidden_band_is_rejected_without_mutation() {
        for forbidden in [Band::Contested, Band::EnemyFront, Band::EnemyRear] {
            let mut michael = captain();
            michael.band = forbidden.index();
            let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 8);
            enemy.band = Band::EnemyFront.index();
            let mut battle = Battle::new("battle.captain.illegal", [michael, enemy]);
            battle.start();
            let before = battle.snapshot();
            let error = battle
                .submit(SkillCommand {
                    command_id: "captain.reposition.illegal".into(),
                    actor_id: ActorId("character.protagonist.captain".into()),
                    skill_id: "skill.captain.reposition".into(),
                    target_ids: Vec::new(),
                })
                .unwrap_err();
            assert_eq!(
                error,
                BattleError::RepositionNotLegal {
                    actor_id: ActorId("character.protagonist.captain".into()),
                    from_band: forbidden.index(),
                },
                "{} is not one of the two party bands",
                forbidden.name()
            );
            assert_eq!(
                battle.snapshot(),
                before,
                "a rejected command mutates nothing"
            );
        }
    }

    #[test]
    fn captain_reposition_takes_no_target() {
        let mut battle = captain_battle();
        let error = battle
            .submit(SkillCommand {
                command_id: "captain.reposition.targeted".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.captain.reposition".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::IllegalTargetCount {
                skill_id: "skill.captain.reposition".into(),
                expected: 0,
                actual: 1,
            }
        );
    }

    #[test]
    fn betty_cannot_submit_the_captains_command() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyFront.index();
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 8);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.captain.owner", [betty, captain(), enemy]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "betty.borrows.the.carbine".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.captain.weapon_attack".into(),
                target_ids: vec![ActorId("enemy.test".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::SkillOwnerMismatch {
                skill_id: "skill.captain.weapon_attack".into(),
                expected_actor_id: ActorId("character.protagonist.captain".into()),
                actual_actor_id: ActorId("character.heroine.betty".into()),
            }
        );
        assert_eq!(battle.snapshot(), before);
    }

    #[test]
    fn captain_weapon_attack_cannot_be_pointed_at_an_ally() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 4);
        betty.band = Band::PartyFront.index();
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 2);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.captain.friendly", [captain(), betty, enemy]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(SkillCommand {
                command_id: "captain.shoots.betty".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.captain.weapon_attack".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .unwrap_err();
        assert_eq!(
            error,
            BattleError::FriendlyFire {
                actor_id: ActorId("character.protagonist.captain".into()),
                target_id: ActorId("character.heroine.betty".into()),
            }
        );
        assert_eq!(battle.snapshot(), before);
    }

    /// Both band moves in the game go through `Battle::move_actor_to_band`, so
    /// they report movement identically: one `ActorMoved` carrying the band the
    /// actor left and the band it now occupies. This is the behavioural half of
    /// "one helper"; the structural half is that there is exactly one function
    /// that writes `Actor.band` during a command.
    #[test]
    fn rescue_charge_and_reposition_report_the_same_band_move() {
        let moved_events = |events: &[BattleEvent]| -> Vec<(i8, i8)> {
            events
                .iter()
                .filter_map(|event| match event {
                    BattleEvent::ActorMoved {
                        from_band, to_band, ..
                    } => Some((*from_band, *to_band)),
                    _ => None,
                })
                .collect()
        };

        let mut battle = captain_battle();
        let reposition = battle
            .submit(SkillCommand {
                command_id: "captain.reposition.shared".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.captain.reposition".into(),
                target_ids: Vec::new(),
            })
            .expect("legal step");
        assert_eq!(
            moved_events(&reposition),
            vec![(Band::PartyRear.index(), Band::PartyFront.index())]
        );

        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = Band::PartyFront.index();
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 4);
        ayla.band = Band::PartyRear.index();
        let mut enemy = actor("enemy.test", Faction::Hostile, 4, 50, 0, 2);
        enemy.band = Band::EnemyFront.index();
        let mut battle = Battle::new("battle.rescue.shared", [betty, ayla, enemy]);
        battle.start();
        let rescue = battle
            .submit(SkillCommand {
                command_id: "betty.rescue.shared".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.ayla".into()),
                    ActorId("enemy.test".into()),
                ],
            })
            .expect("legal rescue");
        assert_eq!(
            moved_events(&rescue),
            vec![(Band::PartyFront.index(), Band::PartyRear.index())],
            "Rescue Charge reports its move in exactly the shape Reposition does"
        );
    }

    #[test]
    fn the_captain_may_guard_with_the_universal_hold_position_verb() {
        let mut battle = captain_battle();
        battle
            .submit(SkillCommand {
                command_id: "captain.holds".into(),
                actor_id: ActorId("character.protagonist.captain".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: Vec::new(),
            })
            .expect("Guard is the system verb, not a per-character skill");
        assert_eq!(
            battle
                .actor(&ActorId("character.protagonist.captain".into()))
                .unwrap()
                .guard,
            2
        );
    }

    /// The protagonist's name is authored in `content/characters/captain.json`,
    /// whose own notes reconcile it: the full canonical name is Captain Michael
    /// Corrigan, ordinary usage is Michael, and `Captain Corrigan` is the formal
    /// address. The brief's instruction to "use Captain Michael" supersedes
    /// Captain Jack; it does not remove the surname.
    ///
    /// A6 built the actor with a name written into Rust, which disagreed with
    /// the record. Content owns it and this holds the two equal, the same way
    /// loot yields and bond ranks are held.
    #[test]
    fn the_captain_carries_his_authored_display_name() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/characters/captain.json"
        );
        let text = std::fs::read_to_string(path).expect("the captain record is readable");
        let record: serde_json::Value =
            serde_json::from_str(&text).expect("the captain record is JSON");
        let authored = record["displayName"]
            .as_str()
            .expect("the captain record declares a string displayName");
        assert_eq!(
            captain().display_name,
            authored,
            "content/characters/captain.json owns the protagonist's display name"
        );
        assert_eq!(captain().id.0, "character.protagonist.captain");
    }

    /// B17, the copy. The prototype fixture stands Betty at the top of the
    /// ladder so a review scene can show her whole kit; a battle built for a
    /// campaign that has never raised her must not inherit that.
    #[test]
    fn a_battle_built_from_a_campaign_stands_each_woman_where_her_bond_stands() {
        let fixture = Battle::prototype_vertical_slice();
        assert_eq!(
            fixture
                .actor(&ActorId("character.heroine.betty".into()))
                .unwrap()
                .bond_rank,
            "SSS",
            "the review fixture is unchanged by this card"
        );

        let ranks = BTreeMap::from([
            ("character.heroine.betty".to_owned(), "C".to_owned()),
            ("character.heroine.ayla".to_owned(), "D".to_owned()),
        ]);
        let campaign = Battle::prototype_vertical_slice_from_campaign(&CampaignBattleSetup {
            bond_ranks: ranks.clone(),
            ..CampaignBattleSetup::default()
        });
        let rank_of = |id: &str| {
            campaign
                .actor(&ActorId(id.into()))
                .unwrap_or_else(|| panic!("{id} stands in the slice"))
                .bond_rank
                .clone()
        };
        assert_eq!(
            rank_of("character.heroine.betty"),
            "C",
            "the campaign's letter, not the fixture's"
        );
        assert_eq!(rank_of("character.heroine.ayla"), "D");
        assert_eq!(
            rank_of("character.heroine.vix"),
            STARTING_BOND_RANK,
            "a woman no campaign has heard of stands at the floor"
        );
    }

    /// B17: the map is only ever asked about women. Captain Michael and the
    /// Razorbeak have no bond rank to raise, and a map that names one anyway
    /// does not move them.
    #[test]
    fn a_campaign_battle_leaves_every_actor_who_is_not_a_woman_alone() {
        assert!(is_woman_actor_id("character.heroine.betty"));
        assert!(!is_woman_actor_id("character.protagonist.captain"));
        assert!(!is_woman_actor_id("enemy.raptor.razorbeak"));

        let ranks = BTreeMap::from([
            ("character.protagonist.captain".to_owned(), "SSS".to_owned()),
            ("enemy.raptor.razorbeak".to_owned(), "SSS".to_owned()),
        ]);
        let campaign = Battle::prototype_vertical_slice_from_campaign(&CampaignBattleSetup {
            bond_ranks: ranks.clone(),
            ..CampaignBattleSetup::default()
        });
        for id in ["character.protagonist.captain", "enemy.raptor.razorbeak"] {
            assert_eq!(
                campaign.actor(&ActorId(id.into())).unwrap().bond_rank,
                STARTING_BOND_RANK,
                "{id} is not a woman: the campaign's map has no say over him"
            );
        }
    }
    // ---- A7: Ayla's seven ---------------------------------------------------
    //
    // PR #2 (`feature/ayla-bridge-art`) implemented five of these on a battle
    // engine that A5 (bands, Composure), A6 (Michael) and A10 (bond ranks) have
    // since rewritten, so the branch cannot merge and its diff is the
    // specification instead. These are its eight tests, re-landed on the current
    // engine against the current fixture, plus the tests for the two skills it
    // had to leave blocked and for the site-rule seam that unblocked them.

    fn ayla(band: i8, vitality: i32) -> Actor {
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, vitality, 0, 30);
        ayla.band = band;
        ayla
    }

    fn razorbeak(vitality: i32, guard: i32) -> Actor {
        let mut enemy = actor(
            "enemy.raptor.razorbeak",
            Faction::Hostile,
            4,
            vitality,
            guard,
            20,
        );
        enemy.band = Band::EnemyFront.index();
        enemy
    }

    fn ayla_command(command_id: &str, skill_id: &str, targets: Vec<&str>) -> SkillCommand {
        SkillCommand {
            command_id: command_id.into(),
            actor_id: ActorId("character.heroine.ayla".into()),
            skill_id: skill_id.into(),
            target_ids: targets.into_iter().map(|id| ActorId(id.into())).collect(),
        }
    }

    fn grave_watch() -> Vec<ActiveSiteRule> {
        vec![ActiveSiteRule {
            id: crate::strategy::site_rule::GRAVE_WATCH.into(),
            effect: SiteRuleEffect::GuardRegenPerRound(2),
        }]
    }

    // ---- S Curse Dispel ------------------------------------------------------

    #[test]
    fn curse_dispel_removes_every_negative_status_without_healing() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.vitality = 60;
        for (slug, kind, rounds) in [
            ("stunned", StatusKind::Stunned, 1),
            ("poisoned", StatusKind::Poisoned, 3),
            ("bleeding", StatusKind::Bleeding, 2),
        ] {
            betty.statuses.push(StatusInstance {
                id: format!("status.betty.{slug}"),
                kind,
                remaining_rounds: rounds,
                source_id: ActorId("enemy.raptor.razorbeak".into()),
            });
        }
        let mut battle = Battle::new(
            "battle.curse_dispel",
            [betty, ayla(0, 60), razorbeak(70, 3)],
        );
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.curse_dispel",
                "skill.ayla.curse_dispel",
                vec!["character.heroine.betty"],
            ))
            .expect("Ayla dispels her ally's curses");
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, BattleEvent::StatusRemoved { .. }))
                .count(),
            3
        );
        let betty = battle
            .actor(&ActorId("character.heroine.betty".into()))
            .expect("Betty stands in this fight");
        assert!(betty.statuses.is_empty());
        assert_eq!(
            betty.vitality, 60,
            "a dispel is not a heal; Betty's own Condition Cleanse is the one that heals"
        );
    }

    /// Composure is not a curse. `Shaken` is A5's Composure-at-zero status, and
    /// it is restored by Composure rather than lifted by a rank S dispel -- the
    /// same status `cleanse_priority` already refuses to remove.
    #[test]
    fn curse_dispel_lifts_a_stagger_and_leaves_shaken_standing() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.statuses.push(StatusInstance {
            id: "status.betty.staggered".into(),
            kind: StatusKind::Staggered,
            remaining_rounds: 1,
            source_id: ActorId("enemy.raptor.razorbeak".into()),
        });
        betty.spend_composure(10);
        assert!(
            betty.is_shaken(),
            "the fixture Betty is Shaken to begin with"
        );
        let mut battle = Battle::new(
            "battle.curse_dispel_shaken",
            [betty, ayla(0, 60), razorbeak(70, 3)],
        );
        battle.start();
        battle
            .submit(ayla_command(
                "ayla.curse_dispel",
                "skill.ayla.curse_dispel",
                vec!["character.heroine.betty"],
            ))
            .expect("Ayla dispels what a dispel can reach");
        let betty = battle
            .actor(&ActorId("character.heroine.betty".into()))
            .expect("Betty stands in this fight");
        assert_eq!(
            betty
                .statuses
                .iter()
                .map(|status| status.kind.clone())
                .collect::<Vec<_>>(),
            vec![StatusKind::Shaken]
        );
    }

    // ---- C Structural Scan ---------------------------------------------------

    #[test]
    fn structural_scan_reveals_guard_without_mutating_the_target() {
        let mut battle = Battle::new("battle.structural_scan", [ayla(0, 60), razorbeak(70, 5)]);
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.scan",
                "skill.ayla.structural_scan",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect("Ayla reads the construct");
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::TargetInspected {
                guard_revealed: 5,
                counter_tag,
                ..
            } if counter_tag == "break_guard_before_striking"
        )));
        let enemy = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .expect("the razorbeak stands in this fight");
        assert_eq!(enemy.guard, 5);
        assert_eq!(enemy.vitality, 70);
        assert!(enemy.statuses.is_empty());
    }

    #[test]
    fn structural_scan_on_an_open_guard_names_the_other_counter() {
        let mut battle = Battle::new(
            "battle.structural_scan_open",
            [ayla(0, 60), razorbeak(70, 0)],
        );
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.scan",
                "skill.ayla.structural_scan",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect("Ayla reads the construct");
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::TargetInspected {
                guard_revealed: 0,
                counter_tag,
                ..
            } if counter_tag == "exploit_open_guard"
        )));
    }

    // ---- B Safe Passage ------------------------------------------------------

    #[test]
    fn safe_passage_moves_adjacent_living_allies_into_aylas_band_in_stable_order() {
        let mut near_ally = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        near_ally.band = 0;
        let mut far_ally = actor("character.heroine.vix", Faction::Party, 3, 80, 0, 10);
        far_ally.band = 2;
        let mut already_there = actor("character.heroine.grisha", Faction::Party, 3, 80, 0, 9);
        already_there.band = 1;
        let mut defeated_ally = actor("character.heroine.nara", Faction::Party, 3, 0, 0, 8);
        defeated_ally.band = 0;
        let mut battle = Battle::new(
            "battle.safe_passage",
            [
                ayla(1, 60),
                near_ally,
                far_ally,
                already_there,
                defeated_ally,
                razorbeak(70, 3),
            ],
        );
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.safe_passage",
                "skill.ayla.safe_passage",
                vec![],
            ))
            .expect("Ayla walks the party through");
        let moved: Vec<String> = events
            .iter()
            .filter_map(|event| match event {
                BattleEvent::ActorMoved { actor_id, .. } => Some(actor_id.0.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            moved,
            vec!["character.heroine.betty", "character.heroine.vix"],
            "stable actor-ID order; the ally already in her band and the ally two bands away do not move"
        );
        let band_of = |id: &str| {
            battle
                .actor(&ActorId(id.into()))
                .unwrap_or_else(|| panic!("{id} stands in this fight"))
                .band
        };
        assert_eq!(band_of("character.heroine.betty"), 1);
        assert_eq!(band_of("character.heroine.vix"), 1);
        assert_eq!(band_of("character.heroine.grisha"), 1);
        assert_eq!(
            band_of("character.heroine.nara"),
            0,
            "a defeated ally is not carried anywhere"
        );
    }

    #[test]
    fn safe_passage_with_nobody_adjacent_moves_nobody_and_still_ends_the_turn() {
        let mut far_ally = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        far_ally.band = 3;
        let mut battle = Battle::new(
            "battle.safe_passage_alone",
            [ayla(1, 60), far_ally, razorbeak(70, 3)],
        );
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.safe_passage",
                "skill.ayla.safe_passage",
                vec![],
            ))
            .expect("the command is legal with nobody to carry");
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, BattleEvent::ActorMoved { .. }))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::TurnEnded { .. }))
        );
    }

    // ---- D Reach Counter -----------------------------------------------------

    #[test]
    fn reach_counter_strikes_a_hostile_that_attacks_an_ally_sharing_aylas_band() {
        let mut ayla = ayla(2, 60);
        ayla.skill_uses_remaining
            .insert("skill.ayla.reach_counter".into(), 1);
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = 2;
        let mut enemy = razorbeak(50, 0);
        enemy.initiative = 40;
        let mut battle = Battle::new("battle.reach_counter", [ayla, betty, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "enemy.bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the razorbeak bites");
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::ReactionTriggered { skill_id, .. }
                if skill_id == "skill.ayla.reach_counter"
        )));
        let enemy = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .expect("the razorbeak stands in this fight");
        assert_eq!(
            enemy.vitality,
            50 - (REACH_COUNTER_RAW_DAMAGE + 3),
            "ten raw damage plus Ayla's level, through no Guard"
        );
        assert_eq!(
            battle
                .actor(&ActorId("character.heroine.ayla".into()))
                .expect("Ayla stands in this fight")
                .skill_uses_remaining["skill.ayla.reach_counter"],
            1,
            "the entry is a presence gate, not a charge; it is never spent"
        );
    }

    #[test]
    fn reach_counter_does_not_trigger_when_ayla_does_not_share_the_targets_band() {
        let mut ayla = ayla(0, 60);
        ayla.skill_uses_remaining
            .insert("skill.ayla.reach_counter".into(), 1);
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12);
        betty.band = 2;
        let mut enemy = razorbeak(50, 0);
        enemy.initiative = 40;
        let mut battle = Battle::new("battle.reach_counter_out_of_band", [ayla, betty, enemy]);
        battle.start();
        let events = battle
            .submit(SkillCommand {
                command_id: "enemy.bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the razorbeak bites");
        assert!(!events.iter().any(|event| matches!(
            event,
            BattleEvent::ReactionTriggered { skill_id, .. }
                if skill_id == "skill.ayla.reach_counter"
        )));
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .expect("the razorbeak stands in this fight")
                .vitality,
            50
        );
    }

    // ---- A Ward Line ---------------------------------------------------------

    #[test]
    fn ward_line_damages_and_staggers_the_first_hostile_attacking_the_warded_band() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 1);
        betty.band = 1;
        let mut battle = Battle::new("battle.ward_line", [ayla(1, 60), betty, razorbeak(50, 0)]);
        battle.start();
        let placed = battle
            .submit(ayla_command("place_ward", "skill.ayla.ward_line", vec![]))
            .expect("Ayla places the ward at her own band");
        assert!(
            placed
                .iter()
                .any(|event| matches!(event, BattleEvent::WardLinePlaced { band: 1, .. }))
        );
        let events = battle
            .submit(SkillCommand {
                command_id: "enemy.bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the razorbeak crosses the ward");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::WardLineTriggered { .. }))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::StatusApplied {
                status_kind: StatusKind::Staggered,
                ..
            }
        )));
        let enemy = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .expect("the razorbeak stands in this fight");
        assert_eq!(enemy.vitality, 50 - (WARD_LINE_RAW_DAMAGE + 3));
        assert!(
            enemy
                .statuses
                .iter()
                .any(|status| status.kind == StatusKind::Staggered)
        );
    }

    #[test]
    fn ward_line_is_spent_after_its_first_trigger_and_does_not_retrigger() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 1);
        betty.band = 1;
        let mut battle = Battle::new(
            "battle.ward_line_spent",
            [ayla(1, 60), betty, razorbeak(50, 0)],
        );
        battle.start();
        battle
            .submit(ayla_command("place_ward", "skill.ayla.ward_line", vec![]))
            .expect("the ward is placed");
        battle
            .submit(SkillCommand {
                command_id: "first_bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the first crossing");
        let after_first = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .expect("the razorbeak stands in this fight")
            .clone();
        battle
            .submit(SkillCommand {
                command_id: "betty_holds".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.system.hold_position".into(),
                target_ids: vec![],
            })
            .expect("Betty holds");
        battle
            .submit(ayla_command(
                "ayla_holds",
                "skill.system.hold_position",
                vec![],
            ))
            .expect("Ayla holds");
        let second = battle
            .submit(SkillCommand {
                command_id: "second_bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the second crossing");
        assert!(
            !second
                .iter()
                .any(|event| matches!(event, BattleEvent::WardLineTriggered { .. }))
        );
        let after_second = battle
            .actor(&ActorId("enemy.raptor.razorbeak".into()))
            .expect("the razorbeak stands in this fight");
        assert_eq!(after_second.vitality, after_first.vitality);
        assert_eq!(after_second.statuses.len(), after_first.statuses.len());
    }

    #[test]
    fn ward_line_does_not_trigger_when_no_ward_is_active() {
        let mut betty = actor("character.heroine.betty", Faction::Party, 3, 100, 0, 1);
        betty.band = 1;
        let mut battle = Battle::new("battle.no_ward", [ayla(1, 60), betty, razorbeak(50, 0)]);
        battle.start();
        battle
            .submit(ayla_command(
                "ayla_holds",
                "skill.system.hold_position",
                vec![],
            ))
            .expect("Ayla holds instead of warding");
        let events = battle
            .submit(SkillCommand {
                command_id: "bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.betty".into())],
            })
            .expect("the razorbeak bites");
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, BattleEvent::WardLineTriggered { .. }))
        );
    }

    // ---- SS Deny Activation --------------------------------------------------

    fn deny_activation_battle() -> Battle {
        let mut ayla = ayla(1, 60);
        ayla.skill_uses_remaining
            .insert("skill.ayla.deny_activation".into(), 1);
        Battle::new("battle.deny_activation", [ayla, razorbeak(50, 0)])
    }

    #[test]
    fn deny_activation_cancels_the_hostiles_declared_action_for_the_turn() {
        let mut battle = deny_activation_battle();
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.deny",
                "skill.ayla.deny_activation",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect("Ayla denies the activation");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, BattleEvent::ActivationDenied { .. })),
            "the bridge reads this event to spend the site's one charge"
        );
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::StatusApplied {
                status_kind: StatusKind::Stunned,
                ..
            }
        )));
        // The razorbeak's turn came and went without it acting, and the stun is
        // spent: the fight is on round two with Ayla to move again.
        assert_eq!(
            battle.active_actor_id(),
            Some(&ActorId("character.heroine.ayla".into()))
        );
        assert_eq!(battle.snapshot().round, 2);
        assert!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak".into()))
                .expect("the razorbeak stands in this fight")
                .statuses
                .is_empty(),
            "one turn, and no longer"
        );
    }

    #[test]
    fn deny_activation_is_once_per_site_and_the_second_is_refused() {
        let mut battle = deny_activation_battle();
        battle.start();
        battle
            .submit(ayla_command(
                "ayla.deny",
                "skill.ayla.deny_activation",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect("the site's one denial");
        let before = battle.snapshot();
        let error = battle
            .submit(ayla_command(
                "ayla.deny.again",
                "skill.ayla.deny_activation",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect_err("the charge is spent");
        assert!(matches!(error, BattleError::SkillUnavailable { .. }));
        assert_eq!(battle.snapshot(), before, "a refusal mutates nothing");
    }

    // ---- SSS Override Tomb Rule ----------------------------------------------

    fn override_battle() -> Battle {
        let mut ayla = ayla(1, 60);
        ayla.skill_uses_remaining
            .insert("skill.ayla.override_tomb_rule".into(), 1);
        Battle::new("battle.override", [ayla, razorbeak(50, 0)]).with_site_rules(grave_watch())
    }

    #[test]
    fn override_tomb_rule_names_a_rule_in_force_and_takes_it_out_of_the_fight() {
        let mut battle = override_battle();
        battle.start();
        let events = battle
            .submit(ayla_command(
                "ayla.override",
                "skill.ayla.override_tomb_rule",
                vec![],
            ))
            .expect("Ayla overrides the tomb's rule");
        assert!(events.iter().any(|event| matches!(
            event,
            BattleEvent::SiteRuleOverridden { rule_id, .. }
                if rule_id == crate::strategy::site_rule::GRAVE_WATCH
        )));
        assert!(
            battle.site_rules().is_empty(),
            "the rule stops applying here and now; the bridge makes it stick"
        );
    }

    #[test]
    fn override_tomb_rule_is_once_per_expedition_and_the_second_is_refused() {
        let mut battle = override_battle();
        battle.start();
        battle
            .submit(ayla_command(
                "ayla.override",
                "skill.ayla.override_tomb_rule",
                vec![],
            ))
            .expect("the expedition's one override");
        battle
            .submit(SkillCommand {
                command_id: "enemy_bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .expect("the razorbeak answers");
        let error = battle
            .submit(ayla_command(
                "ayla.override.again",
                "skill.ayla.override_tomb_rule",
                vec![],
            ))
            .expect_err("the charge is spent");
        assert!(matches!(error, BattleError::SkillUnavailable { .. }));
    }

    // ---- The site-rule seam --------------------------------------------------

    /// A7's card in one assertion: `site_rule.tomb.grave_watch` gives every
    /// living hostile two Guard at the round boundary. Remove the regen from
    /// `apply_site_rules_at_round_boundary` and this is what fails.
    #[test]
    fn grave_watch_regenerates_two_guard_on_every_hostile_at_the_round_boundary() {
        let mut battle = Battle::new("battle.grave_watch", [ayla(1, 60), razorbeak(50, 0)])
            .with_site_rules(grave_watch());
        battle.start();
        assert_eq!(guard_of(&battle, "enemy.raptor.razorbeak"), 0);
        play_one_round(&mut battle);
        assert_eq!(battle.snapshot().round, 2);
        assert_eq!(
            guard_of(&battle, "enemy.raptor.razorbeak"),
            2,
            "the tomb watches its own dead: two Guard back at every round boundary"
        );
        play_one_round(&mut battle);
        assert_eq!(guard_of(&battle, "enemy.raptor.razorbeak"), 4);
    }

    #[test]
    fn a_battle_under_no_site_rules_regenerates_nothing() {
        let mut battle = Battle::new("battle.no_rules", [ayla(1, 60), razorbeak(50, 0)]);
        battle.start();
        play_one_round(&mut battle);
        assert_eq!(battle.snapshot().round, 2);
        assert_eq!(guard_of(&battle, "enemy.raptor.razorbeak"), 0);
    }

    /// The other half of the suppression seam, inside the battle: a rule the
    /// campaign has overridden is not in the list the battle is built with, so
    /// it regenerates nothing. `a_suppressed_site_rule_survives_a_save_and_is_still_not_in_force`
    /// in `expedition.rs` is the campaign-side half.
    #[test]
    fn an_overridden_rule_stops_regenerating_guard_for_the_rest_of_the_fight() {
        let mut ayla = ayla(1, 60);
        ayla.skill_uses_remaining
            .insert("skill.ayla.override_tomb_rule".into(), 1);
        let mut battle = Battle::new("battle.override_regen", [ayla, razorbeak(50, 0)])
            .with_site_rules(grave_watch());
        battle.start();
        battle
            .submit(ayla_command(
                "ayla.override",
                "skill.ayla.override_tomb_rule",
                vec![],
            ))
            .expect("Ayla overrides the tomb's rule");
        battle
            .submit(SkillCommand {
                command_id: "enemy_bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak".into()),
                skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
                target_ids: vec![ActorId("character.heroine.ayla".into())],
            })
            .expect("the razorbeak answers, and the round turns");
        assert_eq!(battle.snapshot().round, 2);
        assert_eq!(
            guard_of(&battle, "enemy.raptor.razorbeak"),
            0,
            "the rule was overridden before the boundary, so nothing regenerated"
        );
    }

    /// A10's gate applies to Ayla exactly as it applies to Betty: her rank C
    /// Structural Scan is not hers at bond rank D.
    #[test]
    fn ayla_at_bond_rank_d_cannot_spend_her_rank_c_structural_scan() {
        let mut ayla = ayla(1, 60);
        STARTING_BOND_RANK.clone_into(&mut ayla.bond_rank);
        let mut battle = Battle::new("battle.ayla_bond_gate", [ayla, razorbeak(50, 5)]);
        battle.start();
        let before = battle.snapshot();
        let error = battle
            .submit(ayla_command(
                "ayla.scan",
                "skill.ayla.structural_scan",
                vec!["enemy.raptor.razorbeak"],
            ))
            .expect_err("her bond has not reached C");
        assert_eq!(
            error,
            BattleError::BondRankTooLow {
                skill_id: "skill.ayla.structural_scan".into(),
                required: "C".into(),
                current: "D".into(),
            }
        );
        assert_eq!(battle.snapshot(), before);
    }

    fn guard_of(battle: &Battle, id: &str) -> i32 {
        battle
            .actor(&ActorId(id.into()))
            .unwrap_or_else(|| panic!("{id} stands in this fight"))
            .guard
    }

    /// One full round: the party holds position, the hostiles take the command
    /// the engine recommends. Nobody's Guard moves except by the site rule --
    /// holding position grants Guard, which is why the hostiles attack instead.
    fn play_one_round(battle: &mut Battle) {
        let round = battle.snapshot().round;
        let mut step = 0;
        while battle.snapshot().round == round {
            step += 1;
            assert!(step < 16, "the round did not turn");
            let actor_id = battle
                .active_actor_id()
                .expect("an actor is active")
                .clone();
            let command_id = format!("round.{round}.step.{step}");
            let command = battle
                .recommended_enemy_command(&command_id)
                .unwrap_or(SkillCommand {
                    command_id,
                    actor_id,
                    skill_id: "skill.system.hold_position".into(),
                    target_ids: vec![],
                });
            battle.submit(command).expect("the command is legal");
        }
    }
    /// `content/skills/` owns every number Ayla's two reactions use; the Rust
    /// constants are copies of it. Held equal here, exactly as
    /// `captain_weapon_attack_deals_its_authored_damage` holds Michael's.
    #[test]
    fn reach_counter_and_ward_line_deal_their_authored_damage() {
        let reach = authored_skill("skill.ayla.reach_counter");
        assert_eq!(
            reach["rules"]["counterRawDamage"].as_i64(),
            Some(i64::from(REACH_COUNTER_RAW_DAMAGE))
        );
        assert_eq!(reach["rules"]["counterDamageLevelScale"].as_i64(), Some(1));
        let ward = authored_skill("skill.ayla.ward_line");
        assert_eq!(
            ward["rules"]["triggerRawDamage"].as_i64(),
            Some(i64::from(WARD_LINE_RAW_DAMAGE))
        );
        assert_eq!(ward["rules"]["triggerDamageLevelScale"].as_i64(), Some(1));
        assert_eq!(ward["rules"]["appliesStatus"].as_str(), Some("staggered"));
        assert_eq!(ward["rules"]["statusRounds"].as_i64(), Some(1));
    }

    /// Her seven records name her, in the design bible's own rank order, and
    /// `content/characters/ayla.json` lists exactly them. Delete one record and
    /// this says which.
    #[test]
    fn aylas_authored_deck_is_her_seven_skills_in_rank_order() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/characters/ayla.json"
        );
        let text = std::fs::read_to_string(path).expect("Ayla's record is readable");
        let ayla: serde_json::Value = serde_json::from_str(&text).expect("her record is JSON");
        let declared: Vec<&str> = ayla["skillIds"]
            .as_array()
            .expect("she declares a skill list")
            .iter()
            .map(|id| id.as_str().expect("a string skill id"))
            .collect();
        assert_eq!(
            declared,
            vec![
                "skill.ayla.reach_counter",
                "skill.ayla.structural_scan",
                "skill.ayla.safe_passage",
                "skill.ayla.ward_line",
                "skill.ayla.curse_dispel",
                "skill.ayla.deny_activation",
                "skill.ayla.override_tomb_rule",
            ]
        );
        for (skill_id, rank) in declared.iter().zip(["D", "C", "B", "A", "S", "SS", "SSS"]) {
            let record = authored_skill(skill_id);
            assert_eq!(record["ownerId"].as_str(), Some("character.heroine.ayla"));
            assert_eq!(
                record["bondRank"].as_str(),
                Some(rank),
                "{skill_id} is her rank {rank} command"
            );
            assert_eq!(skill_rank(skill_id), Some(rank));
        }
    }
}
