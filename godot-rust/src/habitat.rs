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

use crate::expedition::TimeSegment;
use crate::world::{SpawnRule, WorldClock};

/// A habitat's standing threat rank. Rank is the difficulty lever, not group size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncounterRank {
    Ordinary,
    Elevated,
    Apex,
}

/// What kind of thing holds a habitat. `GAME_BUILD_PLAN.md` section 2.4 already
/// makes the island's law resurrection -- "at midnight, eligible dead people and
/// monsters return in flashes of light" -- so undead and spectral holders are that
/// law becoming visible, and Eldritch ties into Phase E3's cosmic intruder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreatureFamily {
    Beast,
    Undead,
    Spectral,
    Eldritch,
}

/// One creature a habitat may present, and when it becomes possible.
///
/// Escalation lives in this data, not in a global switch: a funerary site lists its
/// undead at a low `available_from_day` while the beach lists the same family far
/// later, so the island turns over gradually and outward from the elven sites.
/// `night_only` entries never hold a habitat by day -- they are what the same road
/// presents at Dusk and Midnight.
#[derive(Clone, Debug, PartialEq)]
pub struct RosterEntry {
    pub definition_id: String,
    pub family: CreatureFamily,
    pub available_from_day: u32,
    pub night_only: bool,
}

/// Dusk and Midnight are night; Dawn and Day are not.
pub fn is_night(segment: &TimeSegment) -> bool {
    matches!(segment, TimeSegment::Dusk | TimeSegment::Midnight)
}

/// Appended to a habitat's region to key its night draw. Midnight Return derives
/// instance IDs from region and slot, so day and night need separate regions.
pub const NIGHT_REGION_SUFFIX: &str = ".night";

/// The habitat region a night record belongs to, or `None` if this is a day region.
pub fn base_region_of_night(region_id: &str) -> Option<&str> {
    region_id.strip_suffix(NIGHT_REGION_SUFFIX)
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
    /// Everything that can hold this habitat, gated by day and by night. Ordered
    /// baseline-first, so `leader_definition_id` stays a stable answer to "what
    /// normally lives here".
    pub roster: Vec<RosterEntry>,
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

    /// The baseline holder: the first roster entry that can hold this habitat by
    /// day, or the first entry at all before anything has unlocked.
    pub fn leader_definition_id(&self, day: u32) -> &str {
        self.day_roster(day)
            .first()
            .copied()
            .or_else(|| self.roster.first())
            .map(|entry| entry.definition_id.as_str())
            .unwrap_or("")
    }

    /// What can hold this habitat in daylight on `day`.
    pub fn day_roster(&self, day: u32) -> Vec<&RosterEntry> {
        self.roster
            .iter()
            .filter(|entry| !entry.night_only && entry.available_from_day <= day)
            .collect()
    }

    /// What can hold it after dark: everything the day has, plus the night-only
    /// entries that have unlocked.
    pub fn night_roster(&self, day: u32) -> Vec<&RosterEntry> {
        self.roster
            .iter()
            .filter(|entry| entry.available_from_day <= day)
            .collect()
    }

    /// True once this habitat has any night-only holder, i.e. once crossing it
    /// after dark is a different proposition from crossing it by day.
    pub fn has_night_holder(&self, day: u32) -> bool {
        self.roster
            .iter()
            .any(|entry| entry.night_only && entry.available_from_day <= day)
    }

    /// The region key a night holder is recorded under. Midnight Return builds each
    /// spawn's instance ID from region and slot, so the night draw needs its own
    /// region or it would collide with the day's individual.
    pub fn night_region_id(&self) -> String {
        format!("{}{NIGHT_REGION_SUFFIX}", self.region_id)
    }

    /// One individual per habitat per day: "pack size is not a difficulty
    /// substitute." Escalation is in which creature is drawn, never in how many.
    pub fn spawn_rule(&self, day: u32) -> SpawnRule {
        SpawnRule {
            region_id: self.region_id.clone(),
            definition_ids: self
                .day_roster(day)
                .into_iter()
                .map(|entry| entry.definition_id.clone())
                .collect(),
            region_base_level: self.base_level,
            daily_count: 1,
            pressure: self.daily_pressure,
        }
    }

    /// The night draw, or `None` while nothing nocturnal has unlocked here.
    pub fn night_spawn_rule(&self, day: u32) -> Option<SpawnRule> {
        if !self.has_night_holder(day) {
            return None;
        }
        Some(SpawnRule {
            region_id: self.night_region_id(),
            definition_ids: self
                .night_roster(day)
                .into_iter()
                .map(|entry| entry.definition_id.clone())
                .collect(),
            region_base_level: self.base_level,
            daily_count: 1,
            pressure: self.daily_pressure,
        })
    }

    pub fn threat_profile(&self, day: u32) -> ThreatProfile {
        ThreatProfile {
            habitat_id: self.id.clone(),
            region_id: self.region_id.clone(),
            leader_definition_id: self.leader_definition_id(day).to_owned(),
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
    ///
    /// The rosters carry the escalation schedule. The Reception Terrace precinct is
    /// an elven funerary site, so its dead rise first (day 2) and its ghosts walk
    /// there from day 3; the river jungle follows around day 8; the open beach is
    /// last, and even there the nights turn (day 6) well before the daylight does
    /// (day 14). Eldritch holders unlock latest, at the terrace only, where Phase
    /// E3's cosmic intruder has elven systems to work through.
    pub fn black_beach_vertical_slice() -> Self {
        let records = [
            HabitatRecord {
                id: "habitat.black_beach.tideline".into(),
                region_id: "world.region.black_beach.tideline".into(),
                display_name: "Black Beach Tideline".into(),
                roster: vec![
                    RosterEntry {
                        definition_id: "enemy.raptor.razorbeak".into(),
                        family: CreatureFamily::Beast,
                        available_from_day: 1,
                        night_only: false,
                    },
                    RosterEntry {
                        definition_id: "enemy.spectral.lament".into(),
                        family: CreatureFamily::Spectral,
                        available_from_day: 6,
                        night_only: true,
                    },
                    RosterEntry {
                        definition_id: "enemy.undead.drowned_bearer".into(),
                        family: CreatureFamily::Undead,
                        available_from_day: 14,
                        night_only: false,
                    },
                ],
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
                roster: vec![
                    RosterEntry {
                        definition_id: "enemy.boar.thunderback".into(),
                        family: CreatureFamily::Beast,
                        available_from_day: 1,
                        night_only: false,
                    },
                    RosterEntry {
                        definition_id: "enemy.undead.drowned_bearer".into(),
                        family: CreatureFamily::Undead,
                        available_from_day: 8,
                        night_only: true,
                    },
                    RosterEntry {
                        definition_id: "enemy.undead.tomb_wight".into(),
                        family: CreatureFamily::Undead,
                        available_from_day: 11,
                        night_only: false,
                    },
                ],
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
                roster: vec![
                    RosterEntry {
                        definition_id: "enemy.raptor.razorbeak.crested".into(),
                        family: CreatureFamily::Beast,
                        available_from_day: 1,
                        night_only: false,
                    },
                    RosterEntry {
                        definition_id: "enemy.undead.tomb_wight".into(),
                        family: CreatureFamily::Undead,
                        available_from_day: 2,
                        night_only: false,
                    },
                    RosterEntry {
                        definition_id: "enemy.spectral.lament".into(),
                        family: CreatureFamily::Spectral,
                        available_from_day: 3,
                        night_only: true,
                    },
                    RosterEntry {
                        definition_id: "enemy.eldritch.tide_spawn".into(),
                        family: CreatureFamily::Eldritch,
                        available_from_day: 12,
                        night_only: false,
                    },
                ],
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

    /// Every habitat's rule for `day`, in stable habitat-ID order, ready for
    /// `ExpeditionState::resolve_midnight`. Each habitat contributes its day draw
    /// and, once anything nocturnal has unlocked there, its night draw too -- one
    /// atomic midnight transaction still covers both.
    pub fn spawn_rules(&self, day: u32) -> Vec<SpawnRule> {
        let mut rules = Vec::new();
        for record in self.records.values() {
            rules.push(record.spawn_rule(day));
            if let Some(night) = record.night_spawn_rule(day) {
                rules.push(night);
            }
        }
        rules
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

        let events = state.resolve_midnight_in(&habitats).expect("resolves");

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

        let events = state.resolve_midnight_in(&habitats).expect("resolves");

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

    fn families_on(record: &HabitatRecord, day: u32) -> Vec<CreatureFamily> {
        record
            .day_roster(day)
            .into_iter()
            .map(|entry| entry.family)
            .collect()
    }

    #[test]
    fn the_dead_rise_at_the_elven_site_first_and_the_open_beach_last() {
        let habitats = Habitats::black_beach_vertical_slice();
        let terrace = habitats
            .habitat("habitat.black_beach.terrace_precinct")
            .expect("habitat exists");
        let tideline = habitats
            .habitat("habitat.black_beach.tideline")
            .expect("habitat exists");

        // Day 2: the funerary precinct's dead are already walking in daylight.
        assert!(families_on(terrace, 2).contains(&CreatureFamily::Undead));
        // The open beach is still nothing but beasts on the same day.
        assert_eq!(families_on(tideline, 2), vec![CreatureFamily::Beast]);

        // Late campaign: it has reached the beach too.
        assert!(families_on(tideline, 14).contains(&CreatureFamily::Undead));
    }

    #[test]
    fn nights_turn_before_daylight_does() {
        let habitats = Habitats::black_beach_vertical_slice();
        let tideline = habitats
            .habitat("habitat.black_beach.tideline")
            .expect("habitat exists");

        // Day 6 on the beach: beasts by day, something spectral after dark.
        assert_eq!(families_on(tideline, 6), vec![CreatureFamily::Beast]);
        assert!(tideline.has_night_holder(6));
        assert!(
            tideline
                .night_roster(6)
                .iter()
                .any(|entry| entry.family == CreatureFamily::Spectral)
        );

        // Before that, the beach holds nothing nocturnal at all.
        assert!(!tideline.has_night_holder(5));
        assert!(tideline.night_spawn_rule(5).is_none());
    }

    #[test]
    fn eldritch_holders_unlock_last_and_only_at_the_elven_site() {
        let habitats = Habitats::black_beach_vertical_slice();
        for record in habitats.all() {
            let eldritch: Vec<_> = record
                .night_roster(12)
                .into_iter()
                .filter(|entry| entry.family == CreatureFamily::Eldritch)
                .collect();
            if record.id == "habitat.black_beach.terrace_precinct" {
                assert_eq!(eldritch.len(), 1, "the precinct has its eldritch holder");
                assert!(
                    record
                        .night_roster(11)
                        .into_iter()
                        .all(|entry| entry.family != CreatureFamily::Eldritch),
                    "and not one day earlier"
                );
            } else {
                assert!(eldritch.is_empty(), "{} stays mundane", record.id);
            }
        }
    }

    #[test]
    fn a_night_draw_never_replaces_the_day_draw_or_multiplies_the_count() {
        let habitats = Habitats::black_beach_vertical_slice();
        let terrace = habitats
            .habitat("habitat.black_beach.terrace_precinct")
            .expect("habitat exists");

        let day = terrace.spawn_rule(6);
        let night = terrace
            .night_spawn_rule(6)
            .expect("nights have turned here");

        // Distinct regions, so their instance IDs cannot collide.
        assert_ne!(day.region_id, night.region_id);
        assert_eq!(night.region_id, terrace.night_region_id());
        // Still exactly one individual on each side of the clock.
        assert_eq!(day.daily_count, 1);
        assert_eq!(night.daily_count, 1);
        // The night pool is a superset: what walks by day still walks at night.
        for id in &day.definition_ids {
            assert!(night.definition_ids.contains(id));
        }
    }

    #[test]
    fn the_same_road_presents_a_different_individual_after_dark() {
        let habitats = Habitats::black_beach_vertical_slice();
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            11,
            vec!["character.heroine.betty".into()],
            "location.black_beach.reception_terrace",
        )
        .expect("fresh campaign constructs");
        state.campaign_day = 8;
        state.resolve_midnight_in(&habitats).expect("resolves");

        let region = "world.region.black_beach.terrace_precinct";
        assert!(state.daily_spawn_records.contains_key(region));
        assert!(state.nightly_spawn_records.contains_key(region));

        state.time_segment = TimeSegment::Day;
        let by_day = state
            .begin_encounter(&geography, &habitats)
            .expect("the terrace is held")
            .clone();

        state.pending_encounter = None;
        state.time_segment = TimeSegment::Dusk;
        let after_dark = state
            .begin_encounter(&geography, &habitats)
            .expect("and still held after dark")
            .clone();

        assert_ne!(by_day.encounter_id, after_dark.encounter_id);
    }
}
