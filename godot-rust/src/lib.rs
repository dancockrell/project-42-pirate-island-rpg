//! Authoritative, engine-independent rules for Project 42.
//!
//! Godot may display these results but must not calculate replacements for them.

pub mod battle;
pub mod expedition;
pub mod protocol;
pub mod world;

#[cfg(feature = "godot-ext")]
mod godot_bridge;

pub use battle::{
    Actor, ActorId, Battle, BattleError, BattleEvent, BattlePhase, BattleSnapshot,
    BattlefieldEffect, Faction, SkillCommand, StatusInstance, StatusKind,
};
pub use expedition::{
    CharacterState, EncounterState, ExpeditionError, ExpeditionState, HabitatState,
    HouseholdProgress, RouteStep, SupplyState, TimeSegment,
};
pub use world::{DeathMemory, NamedPerson, SpawnRule, SpawnedMonster, WorldClock, WorldEvent};
