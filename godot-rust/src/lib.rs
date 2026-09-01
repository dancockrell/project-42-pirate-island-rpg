//! Authoritative, engine-independent rules for Project 42.
//!
//! Godot may display these results but must not calculate replacements for them.

pub mod battle;
pub mod world;

pub use battle::{Actor, ActorId, Battle, BattleError, BattleEvent, SkillCommand};
pub use world::{DeathMemory, NamedPerson, SpawnedMonster, WorldClock};

