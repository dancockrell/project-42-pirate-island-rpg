use crate::battle::{ActorId, BattleError, BattleEvent, BattleSnapshot, SkillCommand};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct CommandEnvelope {
    pub protocol_version: u16,
    pub command_id: String,
    pub battle_id: String,
    pub actor_id: String,
    pub kind: CommandKind,
    pub skill_id: String,
    pub target_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandKind {
    UseSkill,
}

impl CommandEnvelope {
    pub fn into_skill_command(self) -> Result<SkillCommand, ProtocolError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion {
                expected: PROTOCOL_VERSION,
                actual: self.protocol_version,
            });
        }
        if self.command_id.is_empty()
            || self.battle_id.is_empty()
            || self.actor_id.is_empty()
            || self.skill_id.is_empty()
        {
            return Err(ProtocolError::MissingRequiredId);
        }
        Ok(SkillCommand {
            command_id: self.command_id,
            actor_id: ActorId(self.actor_id),
            skill_id: self.skill_id,
            target_ids: self.target_ids.into_iter().map(ActorId).collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolError {
    UnsupportedVersion { expected: u16, actual: u16 },
    MissingRequiredId,
    BattleNotCreated,
    SimulationRejected(BattleError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventBatch {
    pub protocol_version: u16,
    pub battle_id: String,
    pub events: Vec<BattleEvent>,
    pub snapshot: BattleSnapshot,
}

impl EventBatch {
    pub fn new(
        battle_id: impl Into<String>,
        events: Vec<BattleEvent>,
        snapshot: BattleSnapshot,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            battle_id: battle_id.into(),
            events,
            snapshot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(version: u16) -> CommandEnvelope {
        CommandEnvelope {
            protocol_version: version,
            command_id: "command.test.1".into(),
            battle_id: "battle.prototype.returning_names".into(),
            actor_id: "character.heroine.betty".into(),
            kind: CommandKind::UseSkill,
            skill_id: "skill.betty.guarded_strike".into(),
            target_ids: vec!["enemy.raptor.razorbeak.prototype".into()],
        }
    }

    #[test]
    fn conversion_preserves_stable_ids() {
        let converted = command(PROTOCOL_VERSION).into_skill_command().unwrap();
        assert_eq!(
            converted.actor_id,
            ActorId("character.heroine.betty".into())
        );
        assert_eq!(
            converted.target_ids,
            vec![ActorId("enemy.raptor.razorbeak.prototype".into())]
        );
    }

    #[test]
    fn rejects_unknown_versions_before_simulation() {
        assert_eq!(
            command(9).into_skill_command().unwrap_err(),
            ProtocolError::UnsupportedVersion {
                expected: 1,
                actual: 9
            }
        );
    }

    #[test]
    fn rejects_missing_ids_before_simulation() {
        let mut invalid = command(PROTOCOL_VERSION);
        invalid.skill_id.clear();
        assert_eq!(
            invalid.into_skill_command().unwrap_err(),
            ProtocolError::MissingRequiredId
        );
    }
}
