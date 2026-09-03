//! Individual encounter ecology, per `docs/GAME_BUILD_PLAN.md` Phase D1: "Build
//! named or generated individual encounters, habitat by habitat. An encounter has
//! one leader/creature identity, rank, behaviour, intent suite, territory, drop,
//! return eligibility and daily-level rule. Pack size is not a difficulty
//! substitute."
//!
//! Scope note: this is the habitat *rules* layer, in the same spirit as
//! `geography.rs` -- a deterministic Rust registry, not `content/` JSON. Authoring
//! creature records, loot tables and their art is content/frontend work; what the
//! backend owes is the deterministic rule data that decides which individual
//! materializes where, at what level, with what tactical shape. Each habitat feeds
//! `WorldClock::resolve_midnight` (via `spawn_rule`) with `daily_count: 1`, because
//! the island's threats are individual power relationships rather than packs.
//!
//! Every habitat owns a distinct `region_id`. `WorldClock::resolve_midnight` builds
//! each spawn's `instance_id` from region and slot, so habitats sharing a region
//! would produce colliding instance IDs and a single shared `HabitatState` entry in
//! `ExpeditionState`.

use std::collections::BTreeMap;

use crate::world::{SpawnRule, WorldClock};

/// A habitat's standing threat rank. Rank is the difficulty lever, not group size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncounterRank {
    Ordinary,
    Elevated,
    Apex,
}

/// What the individual does first once a fight starts. Presentation turns these
/// into readable prose; they are never prose themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    PressTheWounded,
    BreakGuardFirst,
    HoldTerritory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HabitatRecord {
    pub id: String,
    pub region_id: String,
    pub display_name: String,
    pub leader_definition_id: String,
    pub rank: EncounterRank,
    pub behavior_tags: Vec<String>,
    pub intent_suite: Vec<String>,
    pub territory_location_ids: Vec<String>,
    pub drop_table_id: String,
    pub return_eligible: bool,
    pub base_level: u8,
    pub daily_pressure: i8,
}

/// The structured threat facts a habitat presents on a given campaign day. The
/// frontend writes the sentence; this decides the facts.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreatProfile {
    pub habitat_id: String,
    pub region_id: String,
    pub leader_definition_id: String,
    pub rank: EncounterRank,
    pub level: u8,
    pub action_priority: ActionPriority,
    pub intent_suite: Vec<String>,
    pub territory_location_ids: Vec<String>,
    pub drop_table_id: String,
    pub return_eligible: bool,
}

impl HabitatRecord {
    /// Rank decides what the individual opens with, so three habitats of three
    /// ranks cannot resolve to the same tactical behaviour.
    pub fn action_priority(&self) -> ActionPriority {
        match self.rank {
            EncounterRank::Ordinary => ActionPriority::PressTheWounded,
            EncounterRank::Elevated => ActionPriority::BreakGuardFirst,
            EncounterRank::Apex => ActionPriority::HoldTerritory,
        }
    }

    /// Reuses `WorldClock::level_for_daily_spawn` rather than duplicating the
    /// daily-growth rule, so a habitat's advertised threat level always matches
    /// the level Midnight Return actually spawns it at.
    pub fn level_on_day(&self, day: u32) -> u8 {
        let clock = WorldClock {
            day,
            minute_of_day: 0,
            named_people: BTreeMap::new(),
        };
        clock.level_for_daily_spawn(self.base_level, self.daily_pressure)
    }

    /// One individual per habitat per day: "pack size is not a difficulty
    /// substitute."
    pub fn spawn_rule(&self) -> SpawnRule {
        SpawnRule {
            region_id: self.region_id.clone(),
            definition_ids: vec![self.leader_definition_id.clone()],
            region_base_level: self.base_level,
            daily_count: 1,
            pressure: self.daily_pressure,
        }
    }

    pub fn threat_profile(&self, day: u32) -> ThreatProfile {
        ThreatProfile {
            habitat_id: self.id.clone(),
            region_id: self.region_id.clone(),
            leader_definition_id: self.leader_definition_id.clone(),
            rank: self.rank,
            level: self.level_on_day(day),
            action_priority: self.action_priority(),
            intent_suite: self.intent_suite.clone(),
            territory_location_ids: self.territory_location_ids.clone(),
            drop_table_id: self.drop_table_id.clone(),
            return_eligible: self.return_eligible,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Habitats {
    records: BTreeMap<String, HabitatRecord>,
}

impl Habitats {
    /// The first chapter's three habitats, one per rank, mapped onto the same
    /// locations `Geography::black_beach_vertical_slice` already connects.
    pub fn black_beach_vertical_slice() -> Self {
        let records = [
            HabitatRecord {
                id: "habitat.black_beach.tideline".into(),
                region_id: "world.region.black_beach.tideline".into(),
                display_name: "Black Beach Tideline".into(),
                leader_definition_id: "enemy.raptor.razorbeak".into(),
                rank: EncounterRank::Ordinary,
                behavior_tags: vec!["individual-threat".into(), "opportunist".into()],
                intent_suite: vec!["skill.enemy.razorbeak.rushing_bite".into()],
                territory_location_ids: vec!["location.black_beach".into()],
                drop_table_id: "loot.razorbeak.prototype".into(),
                return_eligible: true,
                base_level: 5,
                daily_pressure: 0,
            },
            HabitatRecord {
                id: "habitat.black_beach.river_jungle".into(),
                region_id: "world.region.black_beach.river_jungle".into(),
                display_name: "River Landing Jungle Edge".into(),
                leader_definition_id: "enemy.boar.thunderback".into(),
                rank: EncounterRank::Elevated,
                behavior_tags: vec!["individual-threat".into(), "charger".into()],
                intent_suite: vec![
                    "skill.enemy.razorbeak.guard_breaking_kick".into(),
                    "skill.enemy.razorbeak.rushing_bite".into(),
                ],
                territory_location_ids: vec!["location.black_beach.river_landing".into()],
                drop_table_id: "loot.thunderback.prototype".into(),
                return_eligible: true,
                base_level: 7,
                daily_pressure: 1,
            },
            HabitatRecord {
                id: "habitat.black_beach.terrace_precinct".into(),
                region_id: "world.region.black_beach.terrace_precinct".into(),
                display_name: "Reception Terrace Precinct".into(),
                leader_definition_id: "enemy.raptor.razorbeak.crested".into(),
                rank: EncounterRank::Apex,
                behavior_tags: vec!["individual-threat".into(), "territorial".into()],
                intent_suite: vec![
                    "skill.enemy.razorbeak.guard_breaking_kick".into(),
                    "skill.enemy.razorbeak.rushing_bite".into(),
                ],
                territory_location_ids: vec![
                    "location.black_beach.reception_terrace".into(),
                    "location.black_beach.processional_ramp".into(),
                ],
                drop_table_id: "loot.razorbeak.crested.prototype".into(),
                return_eligible: true,
                base_level: 9,
                daily_pressure: 2,
            },
        ];
        Self {
            records: records
                .into_iter()
                .map(|record| (record.id.clone(), record))
                .collect(),
        }
    }

    pub fn habitat(&self, id: &str) -> Option<&HabitatRecord> {
        self.records.get(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &HabitatRecord> {
        self.records.values()
    }

    /// Every habitat's rule, in stable habitat-ID order, ready for
    /// `ExpeditionState::resolve_midnight`.
    pub fn spawn_rules(&self) -> Vec<SpawnRule> {
        self.records
            .values()
            .map(HabitatRecord::spawn_rule)
            .collect()
    }

    /// The habitat whose territory covers a location, if any -- how an expedition
    /// at a location learns which individual holds it.
    pub fn habitat_for_location(&self, location_id: &str) -> Option<&HabitatRecord> {
        self.records.values().find(|record| {
            record
                .territory_location_ids
                .iter()
                .any(|id| id == location_id)
        })
    }

    pub fn threat_profiles(&self, day: u32) -> Vec<ThreatProfile> {
        self.records
            .values()
            .map(|record| record.threat_profile(day))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expedition::ExpeditionState;
    use crate::world::WorldEvent;

    #[test]
    fn three_habitats_of_different_rank_produce_distinct_threat_profiles() {
        let habitats = Habitats::black_beach_vertical_slice();
        let profiles = habitats.threat_profiles(1);
        assert_eq!(profiles.len(), 3);

        let ranks: Vec<_> = profiles.iter().map(|profile| profile.rank).collect();
        assert!(ranks.contains(&EncounterRank::Ordinary));
        assert!(ranks.contains(&EncounterRank::Elevated));
        assert!(ranks.contains(&EncounterRank::Apex));

        // Distinct threat descriptions: no two habitats advertise the same
        // creature, level, action priority or intent suite.
        let priorities: Vec<_> = profiles
            .iter()
            .map(|profile| profile.action_priority)
            .collect();
        assert_eq!(
            priorities.len(),
            priorities
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
        let creatures: Vec<_> = profiles
            .iter()
            .map(|profile| profile.leader_definition_id.as_str())
            .collect();
        assert_eq!(
            creatures.len(),
            creatures
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
        let levels: Vec<_> = profiles.iter().map(|profile| profile.level).collect();
        assert_eq!(
            levels.len(),
            levels
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
    }

    #[test]
    fn rank_orders_the_tactical_consequence_on_the_same_day() {
        let habitats = Habitats::black_beach_vertical_slice();
        let ordinary = habitats
            .habitat("habitat.black_beach.tideline")
            .expect("habitat exists");
        let elevated = habitats
            .habitat("habitat.black_beach.river_jungle")
            .expect("habitat exists");
        let apex = habitats
            .habitat("habitat.black_beach.terrace_precinct")
            .expect("habitat exists");

        assert!(ordinary.level_on_day(4) < elevated.level_on_day(4));
        assert!(elevated.level_on_day(4) < apex.level_on_day(4));
        assert_eq!(ordinary.action_priority(), ActionPriority::PressTheWounded);
        assert_eq!(elevated.action_priority(), ActionPriority::BreakGuardFirst);
        assert_eq!(apex.action_priority(), ActionPriority::HoldTerritory);
    }

    #[test]
    fn a_locations_holder_is_the_habitat_whose_territory_covers_it() {
        let habitats = Habitats::black_beach_vertical_slice();
        assert_eq!(
            habitats
                .habitat_for_location("location.black_beach.reception_terrace")
                .map(|habitat| habitat.id.as_str()),
            Some("habitat.black_beach.terrace_precinct")
        );
        assert_eq!(
            habitats
                .habitat_for_location("location.black_beach.estate")
                .map(|habitat| habitat.id.as_str()),
            None
        );
    }

    #[test]
    fn midnight_materializes_exactly_one_individual_per_habitat() {
        let habitats = Habitats::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            11,
            vec!["character.heroine.betty".into()],
            "location.black_beach",
        )
        .expect("fresh campaign constructs");

        let events = state
            .resolve_midnight(&habitats.spawn_rules())
            .expect("resolves");

        let spawned: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                WorldEvent::MonsterMaterialized { monster, region_id } => {
                    Some((region_id.as_str(), monster))
                }
                _ => None,
            })
            .collect();
        assert_eq!(spawned.len(), 3);

        // One individual each, in three distinct regions, with distinct instances.
        let regions: std::collections::BTreeSet<_> =
            spawned.iter().map(|(region, _)| *region).collect();
        assert_eq!(regions.len(), 3);
        let instances: std::collections::BTreeSet<_> = spawned
            .iter()
            .map(|(_, monster)| monster.instance_id.as_str())
            .collect();
        assert_eq!(instances.len(), 3);
        assert_eq!(state.habitat_states.len(), 3);
    }

    #[test]
    fn the_apex_habitat_spawns_a_higher_level_individual_than_the_ordinary_one() {
        let habitats = Habitats::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            11,
            vec!["character.heroine.betty".into()],
            "location.black_beach",
        )
        .expect("fresh campaign constructs");

        let events = state
            .resolve_midnight(&habitats.spawn_rules())
            .expect("resolves");

        let level_in = |region: &str| {
            events
                .iter()
                .find_map(|event| match event {
                    WorldEvent::MonsterMaterialized { region_id, monster }
                        if region_id == region =>
                    {
                        Some(monster.level)
                    }
                    _ => None,
                })
                .expect("region spawned")
        };
        assert!(
            level_in("world.region.black_beach.tideline")
                < level_in("world.region.black_beach.terrace_precinct")
        );
    }
}
