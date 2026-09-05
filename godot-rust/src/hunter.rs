//! Roaming hunters: pursuers who search for the party rather than waiting to be
//! found in a habitat. Unlike habitat holders (`habitat.rs`), a hunter is not tied
//! to one location -- it advances toward the party's current location every time
//! the world moves.
//!
//! "Both, unlocking in order": human/humanoid trackers first, then an undead
//! pursuer that never stops. `HunterKind::returns_after_defeat` is where that
//! distinction lives -- a beaten `HumanTracker` is gone; a beaten `Revenant`
//! returns at the next Midnight Return, which is the island's own law
//! (`GAME_BUILD_PLAN.md` section 2.4) rather than a special case invented for it.

use serde::{Deserialize, Serialize};

use crate::geography::Geography;
use crate::world::mix_seed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HunterKind {
    HumanTracker,
    Revenant,
}

impl HunterKind {
    /// Trackers first, then the undead pursuer that never stops.
    pub const fn available_from_day(self) -> u32 {
        match self {
            HunterKind::HumanTracker => 4,
            HunterKind::Revenant => 10,
        }
    }

    /// A defeated tracker can be reasoned with or simply beaten, and is gone
    /// afterward. A defeated revenant is the island's own law made visible: it
    /// returns at the next midnight, the same as any other eligible dead thing.
    pub const fn returns_after_defeat(self) -> bool {
        matches!(self, HunterKind::Revenant)
    }

    pub const fn definition_id(self) -> &'static str {
        match self {
            HunterKind::HumanTracker => "enemy.hunter.bounty_tracker",
            HunterKind::Revenant => "enemy.hunter.revenant",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hunter {
    pub id: String,
    pub definition_id: String,
    pub kind: HunterKind,
    pub current_location_id: String,
    pub spawned_on_day: u32,
    pub level: u8,
    /// Set on victory against this hunter, cleared at the next Midnight Return.
    /// A `HumanTracker` is removed from the world outright on defeat and never
    /// carries this; only a `Revenant` -- which stays in the world and simply
    /// stands down until it "returns" -- ever has it set.
    #[serde(default)]
    pub defeated_on_day: Option<u32>,
}

impl Hunter {
    pub fn is_defeated_today(&self) -> bool {
        self.defeated_on_day.is_some()
    }
}

/// One in four on an eligible day -- rare enough that a hunter joining the world
/// is an event, not a routine tax on every midnight.
const SPAWN_CHANCE_DENOMINATOR: u64 = 4;

/// Whether a new hunter of `kind` joins the world on `day`, and where.
///
/// Deterministic in seed, day and kind only -- no runtime randomness, matching the
/// handoff's "no random spawn without a recorded seed and a deterministic test."
/// A spawning hunter places itself at whichever location the graph puts furthest
/// from the party, so it always starts as a real, closeable distance rather than
/// immediately adjacent.
pub fn maybe_spawn_hunter(
    seed: u64,
    day: u32,
    kind: HunterKind,
    geography: &Geography,
    party_location_id: &str,
) -> Option<Hunter> {
    if day < kind.available_from_day() {
        return None;
    }
    let roll = mix_seed(seed, day, kind.definition_id(), 0);
    if roll % SPAWN_CHANCE_DENOMINATOR != 0 {
        return None;
    }
    let spawn_location_id = furthest_reachable_location(geography, party_location_id)?;
    Some(Hunter {
        id: format!(
            "hunter.{}.day{day}",
            kind.definition_id().trim_start_matches("enemy.hunter.")
        ),
        definition_id: kind.definition_id().to_owned(),
        kind,
        current_location_id: spawn_location_id,
        spawned_on_day: day,
        level: level_on_day(day),
        defeated_on_day: None,
    })
}

/// Advances every hunter exactly one step along a shortest path toward
/// `party_location_id`. A hunter already sharing the party's location, or with no
/// path to it, does not move -- the party outrunning a hunter by staying ahead of
/// it is the intended way to escape one.
pub fn advance_hunters(hunters: &mut [Hunter], geography: &Geography, party_location_id: &str) {
    for hunter in hunters.iter_mut() {
        if hunter.current_location_id == party_location_id {
            continue;
        }
        if let Some(step) =
            geography.next_step_toward(&hunter.current_location_id, party_location_id)
        {
            hunter.current_location_id = step.to_location_id.clone();
        }
    }
}

/// The first hunter standing at `location_id`, if any. `ExpeditionState` checks
/// this before falling back to a habitat's holder: a hunter that has caught up
/// takes precedence over whatever ordinarily lives there.
pub fn hunter_at<'a>(hunters: &'a [Hunter], location_id: &str) -> Option<&'a Hunter> {
    hunters
        .iter()
        .find(|hunter| hunter.current_location_id == location_id)
}

fn furthest_reachable_location(geography: &Geography, from: &str) -> Option<String> {
    geography
        .all_location_ids()
        .filter(|id| *id != from)
        .filter_map(|id| {
            geography
                .step_distance(from, id)
                .map(|distance| (id, distance))
        })
        .max_by_key(|(_, distance)| *distance)
        .map(|(id, _)| id.to_owned())
}

/// Authored, not the habitat daily-growth curve: a hunter is a deliberate threat,
/// not a wilderness encounter, so it climbs its own gentler line.
fn level_on_day(day: u32) -> u8 {
    (4 + day / 3).min(30) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geography() -> Geography {
        Geography::black_beach_vertical_slice()
    }

    #[test]
    fn nothing_spawns_before_a_kind_unlocks() {
        let geography = geography();
        assert!(
            maybe_spawn_hunter(
                1,
                3,
                HunterKind::HumanTracker,
                &geography,
                "world.cell.black_beach"
            )
            .is_none()
        );
        assert!(
            maybe_spawn_hunter(
                1,
                9,
                HunterKind::Revenant,
                &geography,
                "world.cell.black_beach"
            )
            .is_none()
        );
    }

    #[test]
    fn a_spawn_places_the_hunter_at_the_farthest_reachable_location() {
        let geography = geography();
        // Scan for a day this seed actually rolls a spawn on, rather than assuming
        // the exact day -- the rule under test is placement, not the roll itself.
        let hunter = (4..40)
            .find_map(|day| {
                maybe_spawn_hunter(
                    7,
                    day,
                    HunterKind::HumanTracker,
                    &geography,
                    "world.cell.black_beach",
                )
            })
            .expect("a seed this wide rolls a spawn eventually");
        let actual_distance = geography
            .step_distance("world.cell.black_beach", &hunter.current_location_id)
            .expect("reachable");
        let farthest_possible = geography
            .all_location_ids()
            .filter(|id| *id != "world.cell.black_beach")
            .filter_map(|id| geography.step_distance("world.cell.black_beach", id))
            .max()
            .expect("the graph has other locations");
        assert_eq!(actual_distance, farthest_possible);
    }

    #[test]
    fn the_same_seed_and_day_always_produce_the_same_answer() {
        let geography = geography();
        let first = maybe_spawn_hunter(
            7,
            20,
            HunterKind::Revenant,
            &geography,
            "world.cell.black_beach",
        );
        let second = maybe_spawn_hunter(
            7,
            20,
            HunterKind::Revenant,
            &geography,
            "world.cell.black_beach",
        );
        assert_eq!(first, second);
    }

    #[test]
    fn advancing_closes_exactly_one_step_and_eventually_arrives() {
        let geography = geography();
        let mut hunters = vec![Hunter {
            id: "hunter.test".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "world.cell.processional_ramp".into(),
            spawned_on_day: 4,
            level: 5,
            defeated_on_day: None,
        }];
        let destination = "world.cell.black_beach";
        let starting_distance = geography
            .step_distance(&hunters[0].current_location_id, destination)
            .unwrap();

        advance_hunters(&mut hunters, &geography, destination);

        let new_distance = geography
            .step_distance(&hunters[0].current_location_id, destination)
            .unwrap();
        assert_eq!(new_distance, starting_distance - 1);

        for _ in 0..new_distance {
            advance_hunters(&mut hunters, &geography, destination);
        }
        assert_eq!(hunters[0].current_location_id, destination);
    }

    #[test]
    fn a_hunter_already_with_the_party_does_not_move() {
        let geography = geography();
        let mut hunters = vec![Hunter {
            id: "hunter.test".into(),
            definition_id: HunterKind::Revenant.definition_id().to_owned(),
            kind: HunterKind::Revenant,
            current_location_id: "world.cell.black_beach".into(),
            spawned_on_day: 10,
            level: 8,
            defeated_on_day: None,
        }];
        advance_hunters(&mut hunters, &geography, "world.cell.black_beach");
        assert_eq!(hunters[0].current_location_id, "world.cell.black_beach");
    }

    #[test]
    fn hunter_at_finds_the_one_sharing_a_location_and_nothing_else() {
        let hunters = vec![Hunter {
            id: "hunter.test".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "world.cell.damaged_estate".into(),
            spawned_on_day: 4,
            level: 5,
            defeated_on_day: None,
        }];
        assert!(hunter_at(&hunters, "world.cell.damaged_estate").is_some());
        assert!(hunter_at(&hunters, "world.cell.black_beach").is_none());
    }

    #[test]
    fn a_tracker_is_gone_after_defeat_but_a_revenant_returns() {
        assert!(!HunterKind::HumanTracker.returns_after_defeat());
        assert!(HunterKind::Revenant.returns_after_defeat());
    }
}
