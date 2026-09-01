use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActorId(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Actor {
    pub id: ActorId,
    pub display_name: String,
    pub level: u8,
    pub vitality: i32,
    pub max_vitality: i32,
    pub guard: i32,
    pub band: i8,
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
    CommandAccepted { command_id: String, actor_id: ActorId, skill_id: String },
    DamageApplied { command_id: String, source_id: ActorId, target_id: ActorId, amount: i32 },
    GuardChanged { command_id: String, actor_id: ActorId, delta: i32, total: i32 },
    ActorDefeated { command_id: String, actor_id: ActorId },
}

#[derive(Clone, Debug, PartialEq)]
pub enum BattleError {
    UnknownActor(ActorId),
    UnknownTarget(ActorId),
    ActorDefeated(ActorId),
    IllegalTargetCount { skill_id: String, expected: usize, actual: usize },
    UnsupportedSkill(String),
}

#[derive(Clone, Debug)]
pub struct Battle {
    actors: BTreeMap<ActorId, Actor>,
}

impl Battle {
    pub fn new(actors: impl IntoIterator<Item = Actor>) -> Self {
        Self { actors: actors.into_iter().map(|actor| (actor.id.clone(), actor)).collect() }
    }

    pub fn actor(&self, id: &ActorId) -> Option<&Actor> { self.actors.get(id) }

    pub fn submit(&mut self, command: SkillCommand) -> Result<Vec<BattleEvent>, BattleError> {
        let actor = self.actors.get(&command.actor_id)
            .ok_or_else(|| BattleError::UnknownActor(command.actor_id.clone()))?;
        if actor.vitality <= 0 { return Err(BattleError::ActorDefeated(command.actor_id)); }
        if command.target_ids.len() != 1 {
            return Err(BattleError::IllegalTargetCount {
                skill_id: command.skill_id,
                expected: 1,
                actual: command.target_ids.len(),
            });
        }
        let target_id = command.target_ids[0].clone();
        if !self.actors.contains_key(&target_id) { return Err(BattleError::UnknownTarget(target_id)); }

        let mut events = vec![BattleEvent::CommandAccepted {
            command_id: command.command_id.clone(),
            actor_id: command.actor_id.clone(),
            skill_id: command.skill_id.clone(),
        }];
        match command.skill_id.as_str() {
            "skill.betty.guarded_strike" => {
                let raw_damage = 12 + i32::from(actor.level);
                let (damage, target_defeated) = {
                    let target = self.actors.get_mut(&target_id).expect("target checked above");
                    let absorbed = target.guard.min(raw_damage);
                    target.guard -= absorbed;
                    let damage = raw_damage - absorbed;
                    target.vitality = (target.vitality - damage).max(0);
                    (damage, target.vitality == 0)
                };
                events.push(BattleEvent::DamageApplied {
                    command_id: command.command_id.clone(), source_id: command.actor_id.clone(),
                    target_id: target_id.clone(), amount: damage,
                });
                let actor = self.actors.get_mut(&command.actor_id).expect("actor checked above");
                actor.guard += 2;
                events.push(BattleEvent::GuardChanged {
                    command_id: command.command_id.clone(), actor_id: command.actor_id,
                    delta: 2, total: actor.guard,
                });
                if target_defeated {
                    events.push(BattleEvent::ActorDefeated { command_id: command.command_id, actor_id: target_id });
                }
            }
            other => return Err(BattleError::UnsupportedSkill(other.to_owned())),
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(id: &str, level: u8, vitality: i32, guard: i32) -> Actor {
        Actor { id: ActorId(id.into()), display_name: id.into(), level, vitality,
            max_vitality: vitality, guard, band: 0 }
    }

    #[test]
    fn guarded_strike_is_deterministic_and_emits_projection_events() {
        let betty = actor("character.heroine.betty", 3, 100, 0);
        let raptor = actor("enemy.raptor.razorbeak.prototype", 7, 70, 3);
        let mut battle = Battle::new([betty, raptor]);
        let events = battle.submit(SkillCommand {
            command_id: "command.test.1".into(), actor_id: ActorId("character.heroine.betty".into()),
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec![ActorId("enemy.raptor.razorbeak.prototype".into())],
        }).expect("command should be legal");
        assert_eq!(battle.actor(&ActorId("enemy.raptor.razorbeak.prototype".into())).unwrap().vitality, 58);
        assert_eq!(battle.actor(&ActorId("character.heroine.betty".into())).unwrap().guard, 2);
        assert_eq!(events.len(), 3);
    }
}
