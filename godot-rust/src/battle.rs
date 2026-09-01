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
    UnsupportedSkill(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BattleSnapshot {
    pub battle_id: String,
    pub round: u32,
    pub phase: BattlePhase,
    pub active_actor_id: Option<ActorId>,
    pub actors: Vec<Actor>,
}

#[derive(Clone, Debug)]
pub struct Battle {
    battle_id: String,
    actors: BTreeMap<ActorId, Actor>,
    turn_order: Vec<ActorId>,
    turn_index: usize,
    round: u32,
    phase: BattlePhase,
}

impl Battle {
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
        }
    }

    pub fn actor(&self, id: &ActorId) -> Option<&Actor> {
        self.actors.get(id)
    }
    pub fn active_actor_id(&self) -> Option<&ActorId> {
        self.turn_order.get(self.turn_index)
    }
    pub fn snapshot(&self) -> BattleSnapshot {
        BattleSnapshot {
            battle_id: self.battle_id.clone(),
            round: self.round,
            phase: self.phase.clone(),
            active_actor_id: self.active_actor_id().cloned(),
            actors: self.actors.values().cloned().collect(),
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
        if !matches!(
            command.skill_id.as_str(),
            "skill.betty.guarded_strike"
                | "skill.betty.condition_cleanse"
                | "skill.betty.rescue_charge"
                | "skill.betty.healing_impact"
                | "skill.enemy.razorbeak.rushing_bite"
        ) {
            return Err(BattleError::UnsupportedSkill(command.skill_id));
        }
        let expected_targets = match command.skill_id.as_str() {
            "skill.betty.rescue_charge" => 2,
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
            if !target.is_alive() {
                return Err(BattleError::ActorDefeated(target_id.clone()));
            }
        }
        let target_id = command.target_ids[0].clone();
        let target = self
            .actors
            .get(&target_id)
            .ok_or_else(|| BattleError::UnknownTarget(target_id.clone()))?;
        match command.skill_id.as_str() {
            "skill.betty.condition_cleanse" | "skill.betty.rescue_charge"
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
            "skill.betty.guarded_strike" | "skill.enemy.razorbeak.rushing_bite" => {}
            other => return Err(BattleError::UnsupportedSkill(other.to_owned())),
        }
        let mut target_id = command.target_ids[0].clone();
        if command.skill_id == "skill.enemy.razorbeak.rushing_bite" {
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
        let (raw_damage, guard_gain) = if command.skill_id == "skill.betty.guarded_strike" {
            (12 + i32::from(actor_level), 2)
        } else {
            (9 + i32::from(actor_level), 0)
        };
        let (damage, defeated) = {
            let target = self.actors.get_mut(&target_id).expect("target checked");
            let absorbed = target.guard.min(raw_damage);
            target.guard -= absorbed;
            let effective = raw_damage - absorbed;
            let before = target.vitality;
            target.vitality = (target.vitality - effective).max(0);
            let damage = before - target.vitality;
            (damage, !target.is_alive())
        };
        events.push(BattleEvent::DamageApplied {
            command_id: command.command_id.clone(),
            source_id: command.actor_id.clone(),
            target_id: target_id.clone(),
            amount: damage,
        });
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
        if defeated {
            events.push(BattleEvent::ActorDefeated {
                command_id: command.command_id.clone(),
                actor_id: target_id.clone(),
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
        let target = self.actors.get_mut(hostile_id).expect("hostile checked");
        let absorbed = target.guard.min(10);
        target.guard -= absorbed;
        let effective = 10 - absorbed;
        let before = target.vitality;
        target.vitality = (target.vitality - effective).max(0);
        let damage = before - target.vitality;
        events.push(BattleEvent::DamageApplied {
            command_id: command.command_id.clone(),
            source_id: command.actor_id.clone(),
            target_id: hostile_id.clone(),
            amount: damage,
        });
        if !target.is_alive() {
            events.push(BattleEvent::ActorDefeated {
                command_id: command.command_id.clone(),
                actor_id: hostile_id.clone(),
            });
        }
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
        let (damage, defeated) = {
            let hostile = self.actors.get_mut(hostile_id).expect("target checked");
            let absorbed = hostile.guard.min(raw_damage);
            hostile.guard -= absorbed;
            let effective = raw_damage - absorbed;
            let before = hostile.vitality;
            hostile.vitality = (hostile.vitality - effective).max(0);
            let damage = before - hostile.vitality;
            (damage, !hostile.is_alive())
        };
        events.push(BattleEvent::DamageApplied {
            command_id: command.command_id.clone(),
            source_id: command.actor_id.clone(),
            target_id: hostile_id.clone(),
            amount: damage,
        });
        if defeated {
            events.push(BattleEvent::ActorDefeated {
                command_id: command.command_id.clone(),
                actor_id: hostile_id.clone(),
            });
        }

        let healing = damage / 2;
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
        self.turn_index += 1;
        if self.turn_index >= self.turn_order.len() {
            self.turn_index = 0;
            self.round += 1;
            events.push(BattleEvent::RoundStarted { round: self.round });
        }
        self.skip_defeated_actors();
        self.phase = BattlePhase::AwaitingCommand;
        let actor_id = self.active_actor_id().expect("living actor exists").clone();
        events.push(BattleEvent::TurnStarted {
            round: self.round,
            actor_id: actor_id.clone(),
        });
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
        let target = self
            .actors
            .values()
            .filter(|a| a.faction == Faction::Party && a.is_alive())
            .min_by_key(|a| (a.vitality, &a.id))
            .expect("party target exists");
        BattleEvent::EnemyIntentDeclared {
            actor_id: actor_id.clone(),
            skill_id: "skill.enemy.razorbeak.rushing_bite".into(),
            target_ids: vec![target.id.clone()],
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
}
