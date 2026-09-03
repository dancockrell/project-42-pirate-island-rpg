//! Authoritative, engine-independent rules for Project 42.
//!
//! Godot may display these results but must not calculate replacements for them.

pub mod battle;
pub mod expedition;
pub mod geography;
pub mod habitat;
pub mod protocol;
pub mod world;

#[cfg(feature = "godot-ext")]
mod godot_bridge;

pub use battle::{
    Actor, ActorId, Battle, BattleError, BattleEvent, BattlePhase, BattleSnapshot,
    BattlefieldEffect, Faction, SkillCommand, StatusInstance, StatusKind,
};
pub use expedition::{
    CharacterState, EncounterOutcome, EncounterState, ExpeditionError, ExpeditionState,
    HabitatState, HouseholdProgress, RouteStep, SupplyState, TimeSegment, TravelOutcome,
};
pub use geography::{
    Geography, LocationRecord, PersistencePolicy, ReturnPolicy, RouteKind, RouteOption,
};
pub use habitat::{
    ActionPriority, CreatureFamily, EncounterRank, HabitatRecord, Habitats, RosterEntry,
    ThreatProfile,
};
pub use world::{DeathMemory, NamedPerson, SpawnRule, SpawnedMonster, WorldClock, WorldEvent};
