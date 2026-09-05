//! Authoritative, engine-independent rules for Project 42.
//!
//! Godot may display these results but must not calculate replacements for them.

pub mod battle;
pub mod expedition;
pub mod geography;
pub mod habitat;
pub mod hunter;
pub mod protocol;
pub mod world;

#[cfg(feature = "godot-ext")]
mod godot_bridge;

pub use battle::{
    Actor, ActorId, Band, Battle, BattleError, BattleEvent, BattlePhase, BattleSnapshot,
    BattlefieldEffect, Faction, SkillCommand, StatusInstance, StatusKind, skill_rank,
};
pub use expedition::{
    AnchorOutcome, CharacterState, EncounterOutcome, EncounterResolution, EncounterState,
    ExpeditionError, ExpeditionState, HabitatState, HouseholdProgress, RouteStep, SupplyState,
    TimeSegment, TravelOutcome,
};
pub use geography::{
    AnchorDefinition, AnchorKind, CellDefinition, EncounterTriggerDefinition, Geography,
    LocationRecord, PersistencePolicy, PortalDefinition, ReturnPolicy, RouteKind, RouteOption,
};
pub use habitat::{
    ActionPriority, CreatureFamily, EncounterRank, HabitatRecord, Habitats, LootTable, RosterEntry,
    ThreatProfile,
};
pub use hunter::{Hunter, HunterKind};
pub use world::{DeathMemory, NamedPerson, SpawnRule, SpawnedMonster, WorldClock, WorldEvent};
