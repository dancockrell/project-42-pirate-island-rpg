use std::collections::BTreeMap;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusKind {
    Bleeding,
    Poisoned,
    Burning,
    Stunned,
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
    pub initiative: i16,
    pub statuses: Vec<StatusInstance>,
    pub intercepts_for: Option<ActorId>,
    pub skill_uses_remaining: BTreeMap<String, u8>,
}
impl Actor {
    pub fn is_alive(&self) -> bool {
        self.vitality > 0
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
    UnsupportedSkill(String),
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
                    initiative,
                    statuses: Vec::new(),
                    intercepts_for: None,
                    skill_uses_remaining: BTreeMap::new(),
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
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        betty
            .skill_uses_remaining
            .insert("skill.betty.combat_revival".into(), 1);

        let mut ayla = actor(
            "character.heroine.ayla",
            "Ayla",
            Faction::Party,
            3,
            80,
            0,
            9,
        );
        ayla.vitality = 0;
        ayla.statuses.push(StatusInstance {
            id: "status.ayla.bleeding.prototype".into(),
            kind: StatusKind::Bleeding,
            remaining_rounds: 2,
            source_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
        });

        let mut vix = actor("character.heroine.vix", "Vix", Faction::Party, 3, 80, 0, 10);
        vix.vitality = 10;
        vix.band = 1;
        vix.statuses.push(StatusInstance {
            id: "status.vix.poisoned.prototype".into(),
            kind: StatusKind::Poisoned,
            remaining_rounds: 3,
            source_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
        });

        let mut razorbeak = actor(
            "enemy.raptor.razorbeak.prototype",
            "Razorbeak",
            Faction::Hostile,
            7,
            70,
            3,
            11,
        );
        razorbeak.band = 2;

        Self::new(
            "battle.prototype.returning_names",
            [betty, ayla, vix, razorbeak],
        )
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
        }
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

    pub fn submit(&mut self, command: SkillCommand) -> Result<Vec<BattleEvent>, BattleError> {
        if matches!(self.phase, BattlePhase::Victory | BattlePhase::Defeat) {
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
        if !matches!(
            command.skill_id.as_str(),
            "skill.betty.guarded_strike"
                | "skill.betty.condition_cleanse"
                | "skill.betty.rescue_charge"
                | "skill.betty.healing_impact"
                | "skill.betty.mobile_infirmary"
                | "skill.betty.combat_revival"
                | "skill.enemy.razorbeak.rushing_bite"
                | "skill.enemy.razorbeak.guard_breaking_kick"
                | "skill.system.hold_position"
        ) {
            return Err(BattleError::UnsupportedSkill(command.skill_id));
        }
        let expected_owner = if command.skill_id.starts_with("skill.betty.") {
            Some(ActorId("character.heroine.betty".into()))
        } else if command.skill_id.starts_with("skill.enemy.razorbeak.") {
            Some(ActorId("enemy.raptor.razorbeak.prototype".into()))
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
        if command.skill_id == "skill.betty.combat_revival"
            && actor
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
            "skill.betty.mobile_infirmary" | "skill.system.hold_position" => 0,
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
        if matches!(
            command.skill_id.as_str(),
            "skill.betty.mobile_infirmary" | "skill.system.hold_position"
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
                if actor.faction != target.faction =>
            {
                return Err(BattleError::FriendlyFire {
                    actor_id: command.actor_id,
                    target_id,
                });
            }
            "skill.betty.guarded_strike"
            | "skill.betty.healing_impact"
            | "skill.enemy.razorbeak.rushing_bite"
            | "skill.enemy.razorbeak.guard_breaking_kick"
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
            "skill.betty.healing_impact" => return self.resolve_healing_impact(command, events),
            "skill.betty.mobile_infirmary" => {
                self.resolve_mobile_infirmary(command, events);
                return Ok(());
            }
            "skill.betty.combat_revival" => {
                self.resolve_combat_revival(command, events);
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
            "skill.betty.guarded_strike"
            | "skill.enemy.razorbeak.rushing_bite"
            | "skill.enemy.razorbeak.guard_breaking_kick" => {}
            other => return Err(BattleError::UnsupportedSkill(other.to_owned())),
        }
        let mut target_id = command.target_ids[0].clone();
        if command.skill_id.starts_with("skill.enemy.razorbeak.") {
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
        if command.skill_id == "skill.enemy.razorbeak.guard_breaking_kick" {
            let target = self.actors.get_mut(&target_id).expect("target checked");
            let guard_lost = target.guard.min(6);
            target.guard -= guard_lost;
            events.push(BattleEvent::GuardChanged {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
                delta: -guard_lost,
                total: target.guard,
            });
        }
        let (raw_damage, guard_gain) = if command.skill_id == "skill.betty.guarded_strike" {
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
            status_priority(&left.kind)
                .cmp(&status_priority(&right.kind))
                .then_with(|| left.id.cmp(&right.id))
        });
        let remove_count = target.statuses.len().min(2);
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
        let actor = self
            .actors
            .get_mut(&command.actor_id)
            .expect("actor checked");
        let from_band = actor.band;
        actor.band = ally_band;
        actor.intercepts_for = Some(ally_id.clone());
        events.push(BattleEvent::ActorMoved {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            from_band,
            to_band: ally_band,
        });
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
        let source_is_hostile = self
            .actors
            .get(source_id)
            .is_some_and(|source| source.faction == Faction::Hostile);
        let opens_reaction = allow_reactions
            && !self.resolving_reaction
            && would_defeat
            && source_is_hostile
            && target.faction == Faction::Party
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

    fn advance_turn(&mut self, events: &mut Vec<BattleEvent>) {
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
            return;
        }
        if !party_alive {
            self.phase = BattlePhase::Defeat;
            events.push(BattleEvent::BattleEnded { victory: false });
            return;
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
                events.push(BattleEvent::RoundStarted { round: self.round });
            }
        } else {
            self.turn_index += 1;
            if self.turn_index >= self.turn_order.len() {
                self.turn_index = 0;
                self.round += 1;
                events.push(BattleEvent::RoundStarted { round: self.round });
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
        if self.actor(&actor_id).expect("actor exists").faction == Faction::Hostile {
            events.push(self.enemy_intent(&actor_id));
        }
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

fn status_priority(kind: &StatusKind) -> u8 {
    match kind {
        StatusKind::Stunned => 0,
        StatusKind::Burning => 1,
        StatusKind::Poisoned => 2,
        StatusKind::Bleeding => 3,
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
            initiative,
            statuses: Vec::new(),
            intercepts_for: None,
            skill_uses_remaining: BTreeMap::new(),
        }
    }
    fn prototype() -> Battle {
        Battle::new(
            "battle.prototype.returning_names",
            [
                actor("character.heroine.betty", Faction::Party, 3, 100, 0, 12),
                actor(
                    "enemy.raptor.razorbeak.prototype",
                    Faction::Hostile,
                    7,
                    70,
                    3,
                    8,
                ),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
            })
            .unwrap();
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak.prototype".into()))
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
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
                    ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
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
                    ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                    ActorId("enemy.raptor.razorbeak.prototype".into()),
                ],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "enemy.guard.break".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
            })
            .unwrap();
        let kick_events = battle
            .submit(SkillCommand {
                command_id: "create.recovery.opening".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
            } if actor_id.0 == "enemy.raptor.razorbeak.prototype"
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
        assert_eq!(battle.snapshot().recovery_openings.len(), 1);

        let punish_events = battle
            .submit(SkillCommand {
                command_id: "betty.punishes.recovery".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
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
            .position(|event| matches!(event, BattleEvent::DamageApplied { target_id, amount: 21, .. } if target_id.0 == "enemy.raptor.razorbeak.prototype"))
            .expect("bonus damage applied");
        assert!(consumed_index < damage_index);
        assert!(battle.snapshot().recovery_openings.is_empty());
        assert_eq!(
            battle
                .actor(&ActorId("enemy.raptor.razorbeak.prototype".into()))
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
            })
            .unwrap();
        battle
            .submit(SkillCommand {
                command_id: "create.expiring.opening".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
            .position(|event| matches!(event, BattleEvent::RecoveryOpeningExpired { actor_id } if actor_id.0 == "enemy.raptor.razorbeak.prototype"))
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
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
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
        betty.band = 1;
        ayla.band = 1;
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
        betty.band = 0;
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 10);
        ayla.band = 2;
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
        assert_eq!(betty.band, 2);
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
        betty.band = 0;
        betty
            .skill_uses_remaining
            .insert("skill.betty.fatal_intercept".into(), 1);
        let mut ayla = actor("character.heroine.ayla", Faction::Party, 3, 80, 0, 8);
        ayla.band = 2;
        let enemy = actor(
            "enemy.raptor.razorbeak.prototype",
            Faction::Hostile,
            4,
            50,
            0,
            10,
        );
        let mut battle = Battle::new("battle.intercept", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "rescue".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.rescue_charge".into(),
                target_ids: vec![
                    ActorId("character.heroine.ayla".into()),
                    ActorId("enemy.raptor.razorbeak.prototype".into()),
                ],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
        let enemy = actor(
            "enemy.raptor.razorbeak.prototype",
            Faction::Hostile,
            4,
            50,
            0,
            10,
        );
        let mut battle = Battle::new("battle.fatal_intercept", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "betty.turn".into(),
                actor_id: ActorId("character.heroine.betty".into()),
                skill_id: "skill.betty.guarded_strike".into(),
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
            })
            .unwrap();
        let events = battle
            .submit(SkillCommand {
                command_id: "lethal.bite".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                .actor(&ActorId("enemy.raptor.razorbeak.prototype".into()))
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
        let enemy = actor(
            "enemy.raptor.razorbeak.prototype",
            Faction::Hostile,
            1,
            200,
            0,
            8,
        );
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
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
            })
            .unwrap();
        let final_pulse_events = battle
            .submit(SkillCommand {
                command_id: "enemy.two".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
                target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
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
        let enemy = actor(
            "enemy.raptor.razorbeak.prototype",
            Faction::Hostile,
            1,
            100,
            0,
            10,
        );
        let mut battle = Battle::new("battle.bonus_wrap", [betty, ayla, enemy]);
        battle.start();
        battle
            .submit(SkillCommand {
                command_id: "enemy.first".into(),
                actor_id: ActorId("enemy.raptor.razorbeak.prototype".into()),
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
}
