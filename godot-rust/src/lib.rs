//! Authoritative, engine-independent rules for Project 42.
//!
//! Godot may display these results but must not calculate replacements for them.

pub mod battle;
pub mod protocol;
pub mod world;

#[cfg(feature = "godot-ext")]
mod godot_bridge;

pub use battle::{
    Actor, ActorId, Battle, BattleError, BattleEvent, BattlePhase, BattleSnapshot,
    BattlefieldEffect, Faction, SkillCommand, StatusInstance, StatusKind,
};
pub use world::{
    ActorProductionProvenance, BuildingVolume, CubeModule, DeathMemory, DispatchCandidate,
    DispatchScore, FactionBuilding, FactionState, FactionWorld, FactionWorldError,
    FactionWorldEvent, GridCube, MapPlacement, MapPlacementError, NamedPerson, PlacedBuilding,
    ProducedActor, ProductionOrder, ProductionRule, SpawnRule, SpawnedMonster, WorldClock,
    WorldEvent,
};
