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

/// S1: `FactionDefinition`, `FactionState`, `StrategicState`, `Relationship`.
pub mod faction;
/// S12: `RecruitmentState` for the women -- stages, never numbers surfaced.
pub mod recruitment;
/// S4: the strategic tick, pause semantics and the determinism harness.
pub mod tick;
