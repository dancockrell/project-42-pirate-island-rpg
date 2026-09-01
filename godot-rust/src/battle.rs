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
        if command.target_ids.len() != 1 {
            return Err(BattleError::IllegalTargetCount {
                skill_id: command.skill_id,
                expected: 1,
                actual: command.target_ids.len(),
            });
        }
        let target_id = command.target_ids[0].clone();
        let target = self
            .actors
            .get(&target_id)
            .ok_or_else(|| BattleError::UnknownTarget(target_id.clone()))?;
        if actor.faction == target.faction {
            return Err(BattleError::FriendlyFire {
                actor_id: command.actor_id,
                target_id,
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
        self.resolve_skill(&command, &target_id, &mut events)?;
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
        target_id: &ActorId,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let actor = self.actors.get(&command.actor_id).expect("actor checked");
        let (raw_damage, guard_gain) = match command.skill_id.as_str() {
            "skill.betty.guarded_strike" => (12 + i32::from(actor.level), 2),
            "skill.enemy.razorbeak.rushing_bite" => (9 + i32::from(actor.level), 0),
            other => return Err(BattleError::UnsupportedSkill(other.to_owned())),
        };
        let (damage, defeated) = {
            let target = self.actors.get_mut(target_id).expect("target checked");
            let absorbed = target.guard.min(raw_damage);
            target.guard -= absorbed;
            let damage = raw_damage - absorbed;
            target.vitality = (target.vitality - damage).max(0);
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
}
