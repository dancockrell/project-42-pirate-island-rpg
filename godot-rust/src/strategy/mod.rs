//! The strategic simulation: the autonomous island the brief describes, in
//! which factions act on their own clock and Captain Michael's party moves
//! through the result. Lane S of `docs/SHIP_PLAN.md`.
//!
//! This module is declared once, here, so that the lanes building it in
//! parallel each own exactly one file and never race on this one. A lane
//! adds nothing to this file; it fills the file named for it.
//!
//! Rules that hold for every file under this module, from the continuation
//! brief and `AGENTS.md` section 0:
//! - Factions have concept keys, never proper names. IDs are
//!   `faction.<concept_key>`.
//! - Resource categories are Open. String keys from content; no enum.
//! - Provisional doctrines (brief section 6.x) are not behaviour until S15.
//! - Every random draw goes through `crate::world::mix_seed`; `BTreeMap` and
//!   `BTreeSet` everywhere a map or set is serialized.
//! - Pause is not a state. The bridge simply does not call the tick.

/// S3: buildings -- envelopes, sockets, tiers, capture and ruin. Shapes are
/// the deliverable; every Open number is a named constant marked needs decision.
pub mod building;
/// S8: the dual clocks -- world time against Cthulhu patience and heat --
/// weather, and corruption. Advancing one clock never advances the other.
pub mod clocks;
/// S6: `StrategicDirective` -- the player's high-weight request to a faction,
/// with its plain-language explanation produced before confirmation.
pub mod directive;
/// S9: `DungeonContext` and the generation signature -- the same context
/// yields the same rooms; a different owner or tier never collides.
pub mod dungeon;
/// C5: the authored dungeon record -- the design bible's twelve tomb spaces,
/// and the site rules each owning faction brings, selected by S9's context.
pub mod dungeon_content;
/// S10: elimination and the recovery chain -- a faction is gone only when
/// every link is exhausted, and it never respawns.
pub mod elimination;
/// S1: `FactionDefinition`, `FactionState`, `StrategicState`, `Relationship`.
pub mod faction;
/// S7: offscreen forces -- aggregate bodies that move along routes and
/// materialise through a cell's sockets, never teleporting.
pub mod force;
/// S11: the strategic event journal and its digest -- bounded, saved.
pub mod journal;
/// S13: Michael's faction produces machines, capacity and services -- never
/// people. Actor kits for his faction are machine families only.
pub mod production;
/// S12: `RecruitmentState` for the women -- stages, never numbers surfaced.
pub mod recruitment;
/// S4: the strategic tick, pause semantics and the determinism harness.
pub mod tick;
/// S5: utility scoring and the five strategic states, recomputed each tick.
pub mod utility;
