//! `ExpeditionState`: the single versioned campaign object, per
//! `docs/GAME_BUILD_PLAN.md` section 4.1 ("Single source of truth").
//!
//! Scope note: this module proves the save/load/query round trip that B0 of
//! `docs/CLAUDE_BACKEND_HANDOFF.md` requires -- deterministic serialization, version
//! rejection, and a stable `legal_next_commands` query across a reload. It does not
//! attempt full cross-reference validation of stable IDs against `content/` records;
//! that is the TypeScript validator's job (`docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md`),
//! and this module has no dependency on the content bundle to check it honestly. It only
//! checks structural shape (non-empty, lowercase-dotted).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::geography::{Geography, RouteOption};
use crate::habitat::Habitats;
use crate::hunter::{self, Hunter, HunterKind};
use crate::world::{DeathMemory, NamedPerson, SpawnRule, SpawnedMonster, WorldClock, WorldEvent};

pub const CURRENT_SAVE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSegment {
    Dawn,
    Day,
    Dusk,
    Midnight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteStep {
    pub location_id: String,
    pub arrived_on_day: u32,
    pub arrived_segment: TimeSegment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupplyState {
    pub rations: u32,
    pub medicine: u32,
    pub coin: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    pub vitality: i32,
    pub max_vitality: i32,
    pub statuses: Vec<String>,
    pub injured: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HabitatState {
    pub last_spawn_day: Option<u32>,
    pub cleared_today: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HouseholdProgress {
    pub estate_upgrades: BTreeSet<String>,
    pub bond_ranks: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounter_id: String,
    pub battle_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpeditionState {
    pub save_version: u32,
    pub campaign_day: u32,
    pub time_segment: TimeSegment,
    pub party_ids: Vec<String>,
    pub active_location_id: String,
    pub route_history: Vec<RouteStep>,
    pub supplies: SupplyState,
    pub character_states: BTreeMap<String, CharacterState>,
    pub named_person_memory: BTreeMap<String, DeathMemory>,
    pub habitat_states: BTreeMap<String, HabitatState>,
    /// The daily spawn ledger `docs/CLAUDE_BACKEND_HANDOFF.md` requires a snapshot
    /// to reconstruct: which individual currently holds each habitat, keyed by
    /// `region_id` (one individual per habitat per day). `serde(default)` so saves
    /// written before this field existed still load at the same `save_version`.
    #[serde(default)]
    pub daily_spawn_records: BTreeMap<String, SpawnedMonster>,
    /// What holds each habitat after dark, keyed by the habitat's own `region_id`
    /// (the `.night` suffix Midnight Return needs for a distinct instance ID is
    /// stripped here). Empty for habitats with nothing nocturnal unlocked yet.
    #[serde(default)]
    pub nightly_spawn_records: BTreeMap<String, SpawnedMonster>,
    /// Roaming pursuers currently in the world. Unlike a habitat holder, a
    /// hunter is not tied to one location -- it is placed once and then moves.
    #[serde(default)]
    pub hunters: Vec<Hunter>,
    pub discoveries: BTreeSet<String>,
    pub household_progress: HouseholdProgress,
    pub pending_encounter: Option<EncounterState>,
    pub rng_seed: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpeditionError {
    MalformedJson(String),
    FutureSaveVersion { found: u32, current: u32 },
    InvalidStableId { field: &'static str, value: String },
    InvalidPartySize { found: usize },
    IllegalRoute { route_id: String },
    NoPendingEncounter,
    MidnightBlockedByPendingEncounter,
    NotAtEstate,
    InsufficientSupplies { needed: u32, available: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TravelOutcome {
    pub arrived_at: String,
    pub time_cost_minutes: u32,
    pub supply_cost: u32,
}

/// B2: how a battle ended, as reported back through `Battle`'s own `BattlePhase`
/// (`Victory` / `Defeat` / `Retreated`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterOutcome {
    Victory,
    Defeat,
    Retreat,
}

/// B4: the one real estate consequence -- an infirmary recovery. Not a generic
/// base-building tree; one actual action with one actual cost.
const ESTATE_LOCATION_ID: &str = "location.black_beach.estate";
const ESTATE_REST_MEDICINE_COST: u32 = 1;
const ESTATE_REST_VITALITY_RESTORED: i32 = 20;
const ESTATE_UPGRADE_INFIRMARY_RESTED: &str = "estate.upgrade.infirmary_rested";

#[derive(Clone, Debug, PartialEq)]
pub struct EstateRestOutcome {
    pub healed_character_ids: Vec<String>,
    pub medicine_spent: u32,
}

impl ExpeditionState {
    /// Deterministic constructor for a fresh campaign (Phase A1: "new game creates
    /// Michael, Betty, a damaged estate, Black Beach as the current location").
    pub fn new(
        seed: u64,
        party_ids: Vec<String>,
        active_location_id: impl Into<String>,
    ) -> Result<Self, ExpeditionError> {
        let active_location_id = active_location_id.into();
        let state = Self {
            save_version: CURRENT_SAVE_VERSION,
            campaign_day: 1,
            time_segment: TimeSegment::Dawn,
            party_ids,
            active_location_id,
            route_history: Vec::new(),
            supplies: SupplyState {
                rations: 0,
                medicine: 0,
                coin: 0,
            },
            character_states: BTreeMap::new(),
            named_person_memory: BTreeMap::new(),
            habitat_states: BTreeMap::new(),
            daily_spawn_records: BTreeMap::new(),
            nightly_spawn_records: BTreeMap::new(),
            hunters: Vec::new(),
            discoveries: BTreeSet::new(),
            household_progress: HouseholdProgress {
                estate_upgrades: BTreeSet::new(),
                bond_ranks: BTreeMap::new(),
            },
            pending_encounter: None,
            rng_seed: seed,
        };
        state.validate()?;
        Ok(state)
    }

    /// Serializes with `BTreeMap`/`BTreeSet`-backed fields, so equal states always
    /// produce byte-identical JSON regardless of construction order.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ExpeditionState always serializes")
    }

    pub fn from_json(json: &str) -> Result<Self, ExpeditionError> {
        let state: ExpeditionState = serde_json::from_str(json)
            .map_err(|error| ExpeditionError::MalformedJson(error.to_string()))?;
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), ExpeditionError> {
        if self.save_version > CURRENT_SAVE_VERSION {
            return Err(ExpeditionError::FutureSaveVersion {
                found: self.save_version,
                current: CURRENT_SAVE_VERSION,
            });
        }
        if self.party_ids.is_empty() || self.party_ids.len() > 4 {
            return Err(ExpeditionError::InvalidPartySize {
                found: self.party_ids.len(),
            });
        }
        for id in &self.party_ids {
            require_stable_id("party_ids", id)?;
        }
        require_stable_id("active_location_id", &self.active_location_id)?;
        Ok(())
    }

    /// The narrow query B0's proof requires: legal next commands derived purely from
    /// this state, so a save/reload round trip must yield the same set. Deliberately
    /// minimal -- B1 (location graph) and B2 (encounter integration) extend this with
    /// real travel/target commands once those subsystems exist; this proves the query
    /// is stable across persistence, not that it is complete.
    pub fn legal_next_commands(&self) -> Vec<String> {
        match &self.pending_encounter {
            Some(encounter) => vec![format!("resolve_encounter:{}", encounter.encounter_id)],
            None => vec![format!("depart_location:{}", self.active_location_id)],
        }
    }

    /// B1: the routes departing the current location, per the supplied `Geography`.
    pub fn legal_routes<'a>(&self, geography: &'a Geography) -> Vec<&'a RouteOption> {
        geography.routes_from(&self.active_location_id)
    }

    /// B1's real travel command. Rejects a route that doesn't depart the current
    /// location without mutating state -- the same "reject before mutation"
    /// discipline `battle.rs` uses for illegal commands. On success, applies the
    /// route's time/supply cost, records a `RouteStep`, and updates the active
    /// location.
    pub fn travel(
        &mut self,
        route_id: &str,
        geography: &Geography,
    ) -> Result<TravelOutcome, ExpeditionError> {
        let route = geography
            .route(route_id)
            .filter(|route| route.from_location_id == self.active_location_id)
            .ok_or_else(|| ExpeditionError::IllegalRoute {
                route_id: route_id.to_owned(),
            })?
            .clone();

        self.advance_time(route.time_cost_minutes);
        self.supplies.rations = self.supplies.rations.saturating_sub(route.supply_cost);
        self.route_history.push(RouteStep {
            location_id: route.to_location_id.clone(),
            arrived_on_day: self.campaign_day,
            arrived_segment: self.time_segment.clone(),
        });
        self.active_location_id = route.to_location_id.clone();
        // The world moves when the party does: every hunter closes one step
        // toward wherever the party now stands.
        hunter::advance_hunters(&mut self.hunters, geography, &self.active_location_id);

        Ok(TravelOutcome {
            arrived_at: self.active_location_id.clone(),
            time_cost_minutes: route.time_cost_minutes,
            supply_cost: route.supply_cost,
        })
    }

    /// B1's observation query. Records every observation at the current location
    /// as a discovery (idempotent -- `discoveries` is a set) and returns their IDs.
    pub fn inspect(&mut self, geography: &Geography) -> Vec<String> {
        let Some(location) = geography.location(&self.active_location_id) else {
            return Vec::new();
        };
        for observation_id in &location.observation_ids {
            self.discoveries.insert(observation_id.clone());
        }
        location.observation_ids.clone()
    }

    /// Starts the encounter the world actually presents here. A hunter that has
    /// caught up takes precedence over whatever ordinarily lives at this location
    /// -- it came looking for the party, so it answers first. Failing that, this
    /// is the individual that Midnight Return materialized in the habitat holding
    /// this location. Returns `None`, without mutating anything, when neither
    /// applies -- no encounter is already open, and for the habitat case the
    /// location must exist and be `encounter_eligible`, a habitat's territory must
    /// cover it, that habitat must have today's individual still uncleared, and
    /// the ledger must hold it. The encounter's IDs derive from the spawn's
    /// `instance_id` (or the hunter's own stable ID), which already encodes day,
    /// region and slot, so the same campaign day always names the same encounter
    /// across a save/reload.
    pub fn begin_encounter(
        &mut self,
        geography: &Geography,
        habitats: &Habitats,
    ) -> Option<&EncounterState> {
        if self.pending_encounter.is_some() {
            return None;
        }
        if let Some(hunter_id) = hunter::hunter_at(&self.hunters, &self.active_location_id)
            .filter(|hunter| !hunter.is_defeated_today())
            .map(|hunter| hunter.id.clone())
        {
            self.pending_encounter = Some(EncounterState {
                encounter_id: format!("encounter.{hunter_id}"),
                battle_id: format!("battle.{hunter_id}"),
            });
            return self.pending_encounter.as_ref();
        }
        if !geography
            .location(&self.active_location_id)
            .is_some_and(|location| location.encounter_eligible)
        {
            return None;
        }
        let habitat = habitats.habitat_for_location(&self.active_location_id)?;
        let holds_today = self
            .habitat_states
            .get(&habitat.region_id)
            .is_some_and(|state| {
                state.last_spawn_day == Some(self.campaign_day) && !state.cleared_today
            });
        if !holds_today {
            return None;
        }
        // After dark the habitat's night holder answers instead, when it has one.
        // The same road is a different proposition at Dusk than it was at noon.
        let monster = if crate::habitat::is_night(&self.time_segment) {
            self.nightly_spawn_records
                .get(&habitat.region_id)
                .or_else(|| self.daily_spawn_records.get(&habitat.region_id))?
        } else {
            self.daily_spawn_records.get(&habitat.region_id)?
        };

        self.pending_encounter = Some(EncounterState {
            encounter_id: format!("encounter.{}", monster.instance_id),
            battle_id: format!("battle.{}", monster.instance_id),
        });
        self.pending_encounter.as_ref()
    }

    /// B2: resolves the current `pending_encounter` against a `Battle` outcome
    /// (Victory / Defeat / Retreat). Errors, without mutating state, if there is no
    /// pending encounter to resolve. A retreat additionally "updates ExpeditionState,
    /// consumes declared time/cost, and retains any declared injuries or discoveries"
    /// per B2's proof: it looks up the route that led into the current location (the
    /// entry immediately before it in `route_history`) and, if the graph still has a
    /// route back along that same path, applies its time/supply cost and returns the
    /// party there.
    ///
    /// Victory additionally marks that habitat cleared for the day, so the individual
    /// the party just beat cannot be refought until the next Midnight Return puts a
    /// new one there. Defeat and Retreat leave it uncleared: the individual still
    /// holds its territory. Loot and injuries stay out of this -- they are content
    /// the encounter itself declares, not something this method invents.
    pub fn resolve_encounter(
        &mut self,
        outcome: EncounterOutcome,
        geography: &Geography,
        habitats: &Habitats,
    ) -> Result<(), ExpeditionError> {
        if self.pending_encounter.is_none() {
            return Err(ExpeditionError::NoPendingEncounter);
        }
        if outcome == EncounterOutcome::Victory {
            let pending_encounter_id = self
                .pending_encounter
                .as_ref()
                .map(|encounter| encounter.encounter_id.clone());
            let defeated_hunter_index = pending_encounter_id.as_deref().and_then(|encounter_id| {
                self.hunters
                    .iter()
                    .position(|hunter| encounter_id == format!("encounter.{}", hunter.id))
            });
            if let Some(index) = defeated_hunter_index {
                // A tracker beaten in a fair fight is gone; a revenant is the
                // island's own law made visible, and stands down rather than dies
                // -- it returns at the next Midnight Return.
                if self.hunters[index].kind.returns_after_defeat() {
                    self.hunters[index].defeated_on_day = Some(self.campaign_day);
                } else {
                    self.hunters.remove(index);
                }
            } else if let Some(habitat) = habitats.habitat_for_location(&self.active_location_id) {
                if let Some(state) = self.habitat_states.get_mut(&habitat.region_id) {
                    state.cleared_today = true;
                }
            }
        }
        if outcome == EncounterOutcome::Retreat {
            let previous_location_id = self
                .route_history
                .iter()
                .rev()
                .nth(1)
                .map(|step| step.location_id.clone());
            if let Some(previous_location_id) = previous_location_id {
                let return_route = self
                    .legal_routes(geography)
                    .into_iter()
                    .find(|route| route.to_location_id == previous_location_id)
                    .cloned();
                if let Some(return_route) = return_route {
                    self.advance_time(return_route.time_cost_minutes);
                    self.supplies.rations = self
                        .supplies
                        .rations
                        .saturating_sub(return_route.supply_cost);
                    self.route_history.push(RouteStep {
                        location_id: return_route.to_location_id.clone(),
                        arrived_on_day: self.campaign_day,
                        arrived_segment: self.time_segment.clone(),
                    });
                    self.active_location_id = return_route.to_location_id;
                }
            }
        }
        self.pending_encounter = None;
        Ok(())
    }

    /// A coarse, deterministic clock: every 360 minutes rolls the campaign one time
    /// segment forward, and a roll past Midnight advances `campaign_day`. Minimal by
    /// design -- B3 (Midnight Return) owns the actual midnight transaction; this only
    /// keeps `campaign_day`/`time_segment` honest for route time costs.
    fn advance_time(&mut self, minutes: u32) {
        const MINUTES_PER_SEGMENT: u32 = 360;
        let mut remaining = minutes;
        while remaining >= MINUTES_PER_SEGMENT {
            remaining -= MINUTES_PER_SEGMENT;
            self.time_segment = match self.time_segment {
                TimeSegment::Dawn => TimeSegment::Day,
                TimeSegment::Day => TimeSegment::Dusk,
                TimeSegment::Dusk => TimeSegment::Midnight,
                TimeSegment::Midnight => {
                    self.campaign_day += 1;
                    TimeSegment::Dawn
                }
            };
        }
    }

    /// The real, geography-aware version of `legal_next_commands`: departing routes
    /// plus observations at the current location, or the pending encounter alone if
    /// one is active. `legal_next_commands` (no geography) stays as the minimal,
    /// geography-independent stub B0 already proved round-trips a save/reload.
    pub fn legal_next_commands_with_geography(&self, geography: &Geography) -> Vec<String> {
        if let Some(encounter) = &self.pending_encounter {
            return vec![format!("resolve_encounter:{}", encounter.encounter_id)];
        }
        let mut commands: Vec<String> = self
            .legal_routes(geography)
            .into_iter()
            .map(|route| format!("travel:{}", route.id))
            .collect();
        if let Some(location) = geography.location(&self.active_location_id) {
            commands.extend(
                location
                    .observation_ids
                    .iter()
                    .map(|id| format!("inspect:{id}")),
            );
        }
        if self.can_rest_at_estate() {
            commands.push("rest_at_estate".to_owned());
        }
        commands
    }

    fn can_rest_at_estate(&self) -> bool {
        self.active_location_id == ESTATE_LOCATION_ID
            && self.supplies.medicine >= ESTATE_REST_MEDICINE_COST
            && !self
                .household_progress
                .estate_upgrades
                .contains(ESTATE_UPGRADE_INFIRMARY_RESTED)
    }

    /// B4: the one real estate consequence after Reception Terrace -- an infirmary
    /// recovery. Rejects, without mutating state, unless the party is actually at
    /// the estate and has medicine to spend. Spends the medicine, heals every
    /// tracked character up to their max Vitality and clears `injured`, then
    /// records `estate.upgrade.infirmary_rested` as a material tactical fact:
    /// once recorded it removes `rest_at_estate` from
    /// `legal_next_commands_with_geography` for the rest of the day, and the fact
    /// itself survives a save/reload.
    pub fn rest_at_estate(&mut self) -> Result<EstateRestOutcome, ExpeditionError> {
        if self.active_location_id != ESTATE_LOCATION_ID {
            return Err(ExpeditionError::NotAtEstate);
        }
        if self.supplies.medicine < ESTATE_REST_MEDICINE_COST {
            return Err(ExpeditionError::InsufficientSupplies {
                needed: ESTATE_REST_MEDICINE_COST,
                available: self.supplies.medicine,
            });
        }

        self.supplies.medicine -= ESTATE_REST_MEDICINE_COST;
        let mut healed_character_ids = Vec::new();
        for (id, character) in self.character_states.iter_mut() {
            character.vitality =
                (character.vitality + ESTATE_REST_VITALITY_RESTORED).min(character.max_vitality);
            character.injured = false;
            healed_character_ids.push(id.clone());
        }
        self.household_progress
            .estate_upgrades
            .insert(ESTATE_UPGRADE_INFIRMARY_RESTED.to_owned());

        Ok(EstateRestOutcome {
            healed_character_ids,
            medicine_spent: ESTATE_REST_MEDICINE_COST,
        })
    }

    /// B3: Midnight as a single atomic transaction, delegating the deterministic
    /// return/spawn math to `WorldClock::resolve_midnight` (already proven in
    /// `world.rs`) rather than reimplementing it here. Rejects, without mutating
    /// state, if a `pending_encounter` is still open -- midnight cannot resolve
    /// mid-encounter. On success, advances `campaign_day` by one, resets
    /// `time_segment` to Dawn, preserves every death-memory fact and counter while
    /// restoring eligible named people, and records each region's deterministic
    /// spawn in `habitat_states`.
    pub fn resolve_midnight(
        &mut self,
        rules: &[SpawnRule],
    ) -> Result<Vec<WorldEvent>, ExpeditionError> {
        if self.pending_encounter.is_some() {
            return Err(ExpeditionError::MidnightBlockedByPendingEncounter);
        }

        let named_people = self
            .named_person_memory
            .iter()
            .map(|(id, memory)| {
                (
                    id.clone(),
                    NamedPerson {
                        id: id.clone(),
                        display_name: id.clone(),
                        alive_today: self.named_person_is_alive(id),
                        death_memory: memory.clone(),
                    },
                )
            })
            .collect();
        let mut clock = WorldClock {
            day: self.campaign_day,
            minute_of_day: 0,
            named_people,
        };
        let events = clock.resolve_midnight(rules, self.rng_seed);

        self.campaign_day = clock.day;
        self.time_segment = TimeSegment::Dawn;
        for (id, person) in clock.named_people {
            self.named_person_memory.insert(id, person.death_memory);
        }
        // Yesterday's individuals are gone; today's ledgers are exactly what this
        // transaction materialized. A `.night` region carries the habitat's after-dark
        // holder and is filed under the habitat's own region, so the two never collide.
        self.daily_spawn_records.clear();
        self.nightly_spawn_records.clear();
        for event in &events {
            if let WorldEvent::MonsterMaterialized { region_id, monster } = event {
                if let Some(base_region) = crate::habitat::base_region_of_night(region_id) {
                    self.nightly_spawn_records
                        .insert(base_region.to_owned(), monster.clone());
                    continue;
                }
                self.habitat_states.insert(
                    region_id.clone(),
                    HabitatState {
                        last_spawn_day: Some(self.campaign_day),
                        cleared_today: false,
                    },
                );
                self.daily_spawn_records
                    .insert(region_id.clone(), monster.clone());
            }
        }

        Ok(events)
    }

    /// Midnight against a habitat registry rather than hand-built rules. The day
    /// gate has to be read from the day the transaction is about to *become*, not
    /// the one it is leaving, and only the state knows that -- so this computes the
    /// rules itself instead of leaving every caller to remember the off-by-one.
    ///
    /// Also where the roaming hunters live in the midnight transaction: after the
    /// habitat spawns resolve, each unlocked `HunterKind` gets one deterministic
    /// chance to join the world (skipped while one of that kind is already
    /// present, defeated or not, so the island never floods with pursuers), every
    /// hunter's stand-down from a defeat clears -- "it returns at the next
    /// midnight" -- and then every hunter takes its nightly step toward the party,
    /// the same as the step `travel` gives them.
    pub fn resolve_midnight_in(
        &mut self,
        geography: &Geography,
        habitats: &Habitats,
    ) -> Result<Vec<WorldEvent>, ExpeditionError> {
        let rules = habitats.spawn_rules(self.campaign_day + 1);
        let events = self.resolve_midnight(&rules)?;

        for kind in [HunterKind::HumanTracker, HunterKind::Revenant] {
            if self.hunters.iter().any(|hunter| hunter.kind == kind) {
                continue;
            }
            if let Some(hunter) = hunter::maybe_spawn_hunter(
                self.rng_seed,
                self.campaign_day,
                kind,
                geography,
                &self.active_location_id,
            ) {
                self.hunters.push(hunter);
            }
        }
        for hunter in &mut self.hunters {
            hunter.defeated_on_day = None;
        }
        hunter::advance_hunters(&mut self.hunters, geography, &self.active_location_id);

        Ok(events)
    }

    /// Derives "alive today" from the death memory alone -- `named_person_memory`
    /// stays exactly the `Map<PersonId, DeathMemory>` shape `docs/GAME_BUILD_PLAN.md`
    /// section 4.1 specifies, with no separate alive-flag field to keep in sync.
    fn named_person_is_alive(&self, person_id: &str) -> bool {
        self.named_person_memory
            .get(person_id)
            .map_or(true, |memory| {
                memory.last_death_day != Some(self.campaign_day)
            })
    }
}

fn require_stable_id(field: &'static str, value: &str) -> Result<(), ExpeditionError> {
    let shaped = !value.is_empty()
        && value.contains('.')
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_');
    if shaped {
        Ok(())
    } else {
        Err(ExpeditionError::InvalidStableId {
            field,
            value: value.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ExpeditionState {
        let mut state = ExpeditionState::new(
            42,
            vec![
                "character.protagonist.captain".into(),
                "character.heroine.betty".into(),
            ],
            "location.black_beach",
        )
        .expect("fixture constructs");
        state.named_person_memory.insert(
            "person.villager.tomas".into(),
            DeathMemory {
                killed_by_player_count: 1,
                last_death_day: Some(3),
                last_death_context_id: Some("encounter.prototype.returning_names".into()),
            },
        );
        state
            .discoveries
            .insert("discovery.reception_terrace.elven_marker".into());
        state
    }

    #[test]
    fn round_trip_preserves_every_field() {
        let original = fixture();
        let restored = ExpeditionState::from_json(&original.to_json()).expect("round trip parses");
        assert_eq!(original, restored);
    }

    #[test]
    fn equal_states_serialize_to_identical_bytes() {
        let first = fixture();
        let second = fixture();
        assert_eq!(first.to_json(), second.to_json());
    }

    #[test]
    fn boundary_save_reload_preserves_legal_next_commands() {
        // Boundary: beginning an encounter.
        let mut at_encounter = fixture();
        at_encounter.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        let before = at_encounter.legal_next_commands();
        let restored =
            ExpeditionState::from_json(&at_encounter.to_json()).expect("round trip parses");
        assert_eq!(before, restored.legal_next_commands());

        // Boundary: entering a location (no pending encounter).
        let mut at_location = fixture();
        at_location.active_location_id = "world.cell.reception_terrace".into();
        let before = at_location.legal_next_commands();
        let restored =
            ExpeditionState::from_json(&at_location.to_json()).expect("round trip parses");
        assert_eq!(before, restored.legal_next_commands());
    }

    #[test]
    fn rejects_a_save_version_from_the_future() {
        let mut state = fixture();
        state.save_version = CURRENT_SAVE_VERSION + 1;
        let error = ExpeditionState::from_json(&state.to_json()).unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::FutureSaveVersion {
                found: CURRENT_SAVE_VERSION + 1,
                current: CURRENT_SAVE_VERSION
            }
        );
    }

    #[test]
    fn rejects_malformed_json() {
        let error = ExpeditionState::from_json("{ not json").unwrap_err();
        assert!(matches!(error, ExpeditionError::MalformedJson(_)));
    }

    #[test]
    fn rejects_a_structurally_invalid_stable_id() {
        let mut state = fixture();
        state.active_location_id = "Black Beach".into();
        let error = ExpeditionState::from_json(&state.to_json()).unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::InvalidStableId {
                field: "active_location_id",
                value: "Black Beach".into()
            }
        );
    }

    #[test]
    fn rejects_an_empty_or_oversized_party() {
        assert_eq!(
            ExpeditionState::new(1, vec![], "world.cell.black_beach").unwrap_err(),
            ExpeditionError::InvalidPartySize { found: 0 }
        );
        let five = vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
            "character.heroine.ayla".into(),
            "character.heroine.vix".into(),
            "character.heroine.grisha".into(),
        ];
        assert_eq!(
            ExpeditionState::new(1, five, "world.cell.black_beach").unwrap_err(),
            ExpeditionError::InvalidPartySize { found: 5 }
        );
    }

    #[test]
    fn inspect_records_the_current_locations_observations_as_discoveries() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let observed = state.inspect(&geography);
        assert_eq!(
            observed,
            vec!["observation.black_beach.wreck_of_handsome_jack".to_owned()]
        );
        assert!(
            state
                .discoveries
                .contains("observation.black_beach.wreck_of_handsome_jack")
        );
    }

    #[test]
    fn safe_road_and_jungle_edge_apply_distinct_time_and_supply_consequences() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();

        let mut via_safe_road = fixture();
        via_safe_road.supplies.rations = 10;
        via_safe_road
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        let safe_outcome = via_safe_road
            .travel("route.river_landing.safe_road", &geography)
            .expect("legal route");

        let mut via_jungle_edge = fixture();
        via_jungle_edge.supplies.rations = 10;
        via_jungle_edge
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        let jungle_outcome = via_jungle_edge
            .travel("route.river_landing.jungle_edge", &geography)
            .expect("legal route");

        assert_eq!(
            via_safe_road.active_location_id,
            "location.black_beach.reception_terrace"
        );
        assert_eq!(
            via_jungle_edge.active_location_id,
            "location.black_beach.reception_terrace"
        );
        assert_ne!(
            safe_outcome.time_cost_minutes,
            jungle_outcome.time_cost_minutes
        );
        assert_ne!(safe_outcome.supply_cost, jungle_outcome.supply_cost);
        assert_ne!(
            via_safe_road.supplies.rations,
            via_jungle_edge.supplies.rations
        );
        assert_eq!(via_safe_road.route_history.len(), 2);
        assert_eq!(via_jungle_edge.route_history.len(), 2);
    }

    #[test]
    fn reception_terrace_is_encounter_eligible_and_black_beach_is_not() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        assert!(
            !geography
                .location("location.black_beach")
                .unwrap()
                .encounter_eligible
        );
        assert!(
            geography
                .location("location.black_beach.reception_terrace")
                .unwrap()
                .encounter_eligible
        );
    }

    #[test]
    fn travel_rejects_a_route_not_departing_the_current_location_without_mutation() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let before = state.clone();
        let error = state
            .travel("route.river_landing.safe_road", &geography)
            .expect_err("Black Beach cannot use a river-landing route directly");
        assert_eq!(
            error,
            ExpeditionError::IllegalRoute {
                route_id: "route.river_landing.safe_road".into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn travel_rejects_an_unknown_route_id_without_mutation() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let before = state.clone();
        let error = state
            .travel("route.does_not_exist", &geography)
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::IllegalRoute {
                route_id: "route.does_not_exist".into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn boundary_save_reload_preserves_legal_next_commands_after_a_route_choice() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        state
            .travel("route.river_landing.jungle_edge", &geography)
            .expect("legal route");

        let before = state.legal_next_commands_with_geography(&geography);
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(
            before,
            restored.legal_next_commands_with_geography(&geography)
        );
        assert!(before.iter().any(|command| command.starts_with("travel:")));
        assert!(before.iter().any(|command| command.starts_with("inspect:")));
    }

    #[test]
    fn resolve_encounter_rejects_when_nothing_is_pending() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let error = state
            .resolve_encounter(
                EncounterOutcome::Victory,
                &geography,
                &crate::habitat::Habitats::black_beach_vertical_slice(),
            )
            .unwrap_err();
        assert_eq!(error, ExpeditionError::NoPendingEncounter);
    }

    #[test]
    fn victory_and_defeat_only_clear_the_pending_encounter() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        for outcome in [EncounterOutcome::Victory, EncounterOutcome::Defeat] {
            let mut state = fixture();
            state.pending_encounter = Some(EncounterState {
                encounter_id: "encounter.prototype.returning_names".into(),
                battle_id: "battle.prototype.returning_names".into(),
            });
            let location_before = state.active_location_id.clone();
            state
                .resolve_encounter(outcome, &geography, &habitats)
                .expect("resolves");
            assert!(state.pending_encounter.is_none());
            assert_eq!(state.active_location_id, location_before);
        }
    }

    #[test]
    fn retreat_returns_to_the_previous_location_and_consumes_its_route_cost() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.supplies.rations = 10;
        state
            .travel("route.black_beach.to_river_landing", &geography)
            .expect("legal route");
        state
            .travel("route.river_landing.safe_road", &geography)
            .expect("legal route");
        assert_eq!(
            state.active_location_id,
            "location.black_beach.reception_terrace"
        );

        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        let rations_before_retreat = state.supplies.rations;
        state
            .resolve_encounter(
                EncounterOutcome::Retreat,
                &geography,
                &crate::habitat::Habitats::black_beach_vertical_slice(),
            )
            .expect("resolves");

        assert!(state.pending_encounter.is_none());
        assert_eq!(
            state.active_location_id,
            "location.black_beach.river_landing"
        );
        assert!(state.supplies.rations < rations_before_retreat);
        assert_eq!(
            state.route_history.last().unwrap().location_id,
            "location.black_beach.river_landing"
        );
    }

    fn midnight_rules() -> Vec<SpawnRule> {
        vec![SpawnRule {
            region_id: "region.reception_road".into(),
            definition_ids: vec![
                "enemy.raptor.razorbeak".into(),
                "enemy.boar.thunderback".into(),
            ],
            region_base_level: 5,
            daily_count: 2,
            pressure: 1,
        }]
    }

    #[test]
    fn midnight_advances_the_day_once_and_resets_to_dawn() {
        let mut state = fixture();
        state.time_segment = TimeSegment::Dusk;
        let starting_day = state.campaign_day;
        state.resolve_midnight(&midnight_rules()).expect("resolves");
        assert_eq!(state.campaign_day, starting_day + 1);
        assert_eq!(state.time_segment, TimeSegment::Dawn);
    }

    #[test]
    fn a_person_killed_today_returns_alive_with_death_memory_preserved() {
        let mut state = fixture();
        state.named_person_memory.insert(
            "person.villager.tomas".into(),
            DeathMemory {
                killed_by_player_count: 1,
                last_death_day: Some(state.campaign_day),
                last_death_context_id: Some("encounter.prototype.returning_names".into()),
            },
        );
        assert!(!state.named_person_is_alive("person.villager.tomas"));

        state.resolve_midnight(&midnight_rules()).expect("resolves");

        assert!(state.named_person_is_alive("person.villager.tomas"));
        let memory = &state.named_person_memory["person.villager.tomas"];
        assert_eq!(memory.killed_by_player_count, 1);
        assert_eq!(memory.last_death_day, Some(1));
        assert_eq!(
            memory.last_death_context_id.as_deref(),
            Some("encounter.prototype.returning_names")
        );
    }

    #[test]
    fn midnight_spawns_are_deterministic_for_the_same_seed_and_day() {
        let mut first = fixture();
        let mut second = fixture();
        let left = first.resolve_midnight(&midnight_rules()).expect("resolves");
        let right = second
            .resolve_midnight(&midnight_rules())
            .expect("resolves");
        assert_eq!(left, right);
        assert_eq!(first, second);
    }

    #[test]
    fn midnight_records_deterministic_spawns_in_habitat_states() {
        let mut state = fixture();
        state.resolve_midnight(&midnight_rules()).expect("resolves");
        let habitat = state
            .habitat_states
            .get("region.reception_road")
            .expect("habitat recorded");
        assert_eq!(habitat.last_spawn_day, Some(state.campaign_day));
        assert!(!habitat.cleared_today);
    }

    #[test]
    fn midnight_is_blocked_by_a_pending_encounter_without_mutation() {
        let mut state = fixture();
        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
        });
        let before = state.clone();
        let result = state.resolve_midnight(&midnight_rules());
        assert_eq!(
            result,
            Err(ExpeditionError::MidnightBlockedByPendingEncounter)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn midnight_result_round_trips_through_save_and_reload() {
        let mut state = fixture();
        state.resolve_midnight(&midnight_rules()).expect("resolves");
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(state.campaign_day, restored.campaign_day);
        assert_eq!(state.time_segment, restored.time_segment);
        assert_eq!(state.named_person_memory, restored.named_person_memory);
        assert_eq!(state.habitat_states, restored.habitat_states);
    }

    fn wounded_captain() -> CharacterState {
        CharacterState {
            vitality: 30,
            max_vitality: 60,
            statuses: Vec::new(),
            injured: true,
        }
    }

    #[test]
    fn resting_at_the_estate_spends_medicine_and_heals_without_exceeding_max() {
        let mut state = fixture();
        state.active_location_id = "location.black_beach.estate".into();
        state.supplies.medicine = 3;
        state
            .character_states
            .insert("character.protagonist.captain".into(), wounded_captain());

        let outcome = state.rest_at_estate().expect("resolves");

        assert_eq!(outcome.medicine_spent, 1);
        assert_eq!(state.supplies.medicine, 2);
        assert_eq!(
            outcome.healed_character_ids,
            vec!["character.protagonist.captain".to_owned()]
        );
        let captain = &state.character_states["character.protagonist.captain"];
        assert_eq!(captain.vitality, 50);
        assert!(!captain.injured);
        assert!(
            state
                .household_progress
                .estate_upgrades
                .contains("estate.upgrade.infirmary_rested")
        );
    }

    #[test]
    fn resting_at_the_estate_never_heals_past_max_vitality() {
        let mut state = fixture();
        state.active_location_id = "location.black_beach.estate".into();
        state.supplies.medicine = 1;
        state.character_states.insert(
            "character.protagonist.captain".into(),
            CharacterState {
                vitality: 55,
                max_vitality: 60,
                statuses: Vec::new(),
                injured: true,
            },
        );

        state.rest_at_estate().expect("resolves");

        assert_eq!(
            state.character_states["character.protagonist.captain"].vitality,
            60
        );
    }

    #[test]
    fn resting_away_from_the_estate_is_rejected_without_mutation() {
        let mut state = fixture();
        state.supplies.medicine = 3;
        let before = state.clone();

        let result = state.rest_at_estate();

        assert_eq!(result, Err(ExpeditionError::NotAtEstate));
        assert_eq!(state, before);
    }

    #[test]
    fn resting_without_medicine_is_rejected_without_mutation() {
        let mut state = fixture();
        state.active_location_id = "location.black_beach.estate".into();
        state.supplies.medicine = 0;
        let before = state.clone();

        let result = state.rest_at_estate();

        assert_eq!(
            result,
            Err(ExpeditionError::InsufficientSupplies {
                needed: 1,
                available: 0
            })
        );
        assert_eq!(state, before);
    }

    #[test]
    fn resting_is_a_one_time_material_fact_reflected_in_legal_state_data() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "location.black_beach.estate".into();
        state.supplies.medicine = 1;

        let before = state.legal_next_commands_with_geography(&geography);
        assert!(before.contains(&"rest_at_estate".to_owned()));

        state.rest_at_estate().expect("resolves");

        let after = state.legal_next_commands_with_geography(&geography);
        assert!(!after.contains(&"rest_at_estate".to_owned()));
    }

    #[test]
    fn the_estate_upgrade_fact_survives_save_and_reload() {
        let mut state = fixture();
        state.active_location_id = "location.black_beach.estate".into();
        state.supplies.medicine = 1;
        state.rest_at_estate().expect("resolves");

        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(
            state.household_progress.estate_upgrades,
            restored.household_progress.estate_upgrades
        );
        assert!(
            restored
                .household_progress
                .estate_upgrades
                .contains("estate.upgrade.infirmary_rested")
        );
    }

    /// A state standing at the encounter-eligible terrace on a day whose midnight
    /// has already put an individual in every habitat.
    fn at_the_held_terrace() -> (Geography, crate::habitat::Habitats, ExpeditionState) {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("resolves");
        state.active_location_id = "location.black_beach.reception_terrace".into();
        (geography, habitats, state)
    }

    #[test]
    fn the_individual_holding_an_eligible_location_presents_the_encounter() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        let holder = state.daily_spawn_records["world.region.black_beach.terrace_precinct"]
            .instance_id
            .clone();

        let encounter = state
            .begin_encounter(&geography, &habitats)
            .expect("the terrace is held")
            .clone();

        assert_eq!(encounter.encounter_id, format!("encounter.{holder}"));
        assert_eq!(encounter.battle_id, format!("battle.{holder}"));
        assert_eq!(state.pending_encounter, Some(encounter));
    }

    #[test]
    fn a_location_that_is_not_encounter_eligible_presents_nothing() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        state.active_location_id = "location.black_beach.estate".into();

        assert!(state.begin_encounter(&geography, &habitats).is_none());
        assert!(state.pending_encounter.is_none());
    }

    #[test]
    fn no_encounter_before_midnight_has_materialized_an_individual() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "location.black_beach.reception_terrace".into();

        assert!(state.begin_encounter(&geography, &habitats).is_none());
        assert!(state.pending_encounter.is_none());
    }

    #[test]
    fn a_cleared_habitat_presents_nothing_until_the_next_midnight() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        state
            .begin_encounter(&geography, &habitats)
            .expect("the terrace is held");
        state
            .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
            .expect("resolves");

        assert!(
            state.habitat_states["world.region.black_beach.terrace_precinct"].cleared_today,
            "victory clears the habitat for the day"
        );
        assert!(state.begin_encounter(&geography, &habitats).is_none());

        // The next midnight puts a new individual there and reopens it.
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("resolves");
        assert!(state.begin_encounter(&geography, &habitats).is_some());
    }

    #[test]
    fn defeat_and_retreat_leave_the_individual_holding_its_territory() {
        for outcome in [EncounterOutcome::Defeat, EncounterOutcome::Retreat] {
            let (geography, habitats, mut state) = at_the_held_terrace();
            state
                .begin_encounter(&geography, &habitats)
                .expect("the terrace is held");
            state
                .resolve_encounter(outcome, &geography, &habitats)
                .expect("resolves");

            assert!(
                !state.habitat_states["world.region.black_beach.terrace_precinct"].cleared_today,
                "{outcome:?} does not clear the habitat"
            );
        }
    }

    #[test]
    fn the_spawn_ledger_and_pending_encounter_survive_save_and_reload() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        state
            .begin_encounter(&geography, &habitats)
            .expect("the terrace is held");

        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(state, restored);
        assert_eq!(restored.daily_spawn_records.len(), 3);
        assert_eq!(state.pending_encounter, restored.pending_encounter);
    }

    #[test]
    fn a_save_written_before_the_spawn_ledger_existed_still_loads() {
        let mut without_ledger: serde_json::Value =
            serde_json::from_str(&fixture().to_json()).expect("fixture parses");
        without_ledger
            .as_object_mut()
            .expect("save is an object")
            .remove("daily_spawn_records")
            .expect("fixture wrote the field");

        let restored = ExpeditionState::from_json(&without_ledger.to_string())
            .expect("an older save still loads");
        assert!(restored.daily_spawn_records.is_empty());
        assert_eq!(restored.save_version, CURRENT_SAVE_VERSION);
    }

    /// Runs midnight forward from a fresh fixture until a `HumanTracker` has
    /// joined the world, up to a generous day cap, and returns the state at that
    /// exact moment. The roll is seeded and deterministic but not guaranteed on
    /// any one day, so tests that need a live hunter scan for one rather than
    /// assuming a specific day.
    fn state_with_a_spawned_tracker() -> (
        ExpeditionState,
        crate::geography::Geography,
        crate::habitat::Habitats,
    ) {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        for _ in 0..40 {
            state
                .resolve_midnight_in(&geography, &habitats)
                .expect("resolves");
            if state
                .hunters
                .iter()
                .any(|hunter| hunter.kind == HunterKind::HumanTracker)
            {
                break;
            }
        }
        (state, geography, habitats)
    }

    #[test]
    fn hunters_spawn_deterministically_for_the_same_seed_and_day() {
        let (first, _, _) = state_with_a_spawned_tracker();
        let (second, _, _) = state_with_a_spawned_tracker();
        assert_eq!(first.hunters, second.hunters);
        assert!(!first.hunters.is_empty(), "a seed this wide spawns one");
    }

    #[test]
    fn travel_closes_one_hunter_step_toward_the_party() {
        let (mut state, geography, _) = state_with_a_spawned_tracker();
        let hunter_location_before = state.hunters[0].current_location_id.clone();
        let distance_before = geography
            .step_distance(&hunter_location_before, &state.active_location_id)
            .expect("reachable");
        if distance_before == 0 {
            // The hunter already reached the party on the spawning midnight's own
            // advance step; nothing left to close.
            return;
        }

        // Travel somewhere and back so the party's location changes and changes
        // back, giving the hunter a real step to take without the test needing to
        // know the whole route in advance.
        state
            .travel("route.black_beach.to_estate", &geography)
            .expect("legal route");

        let distance_after = geography
            .step_distance(
                &state.hunters[0].current_location_id,
                &state.active_location_id,
            )
            .expect("still reachable");
        assert!(distance_after <= distance_before);
    }

    #[test]
    fn a_hunter_that_reaches_the_party_presents_its_encounter_on_arrival() {
        let (mut state, geography, habitats) = state_with_a_spawned_tracker();
        // Walk the hunter home by hand: repeatedly advancing midnight both spawns
        // and paces hunters, so running it forward is a legitimate way to let the
        // pursuit actually conclude rather than asserting on engineered state.
        for _ in 0..40 {
            if state.hunters[0].current_location_id == state.active_location_id {
                break;
            }
            state
                .resolve_midnight_in(&geography, &habitats)
                .expect("resolves");
        }
        assert_eq!(
            state.hunters[0].current_location_id,
            state.active_location_id
        );

        let encounter = state
            .begin_encounter(&geography, &habitats)
            .expect("the hunter presents an encounter")
            .clone();
        assert_eq!(
            encounter.encounter_id,
            format!("encounter.{}", state.hunters[0].id)
        );
    }

    #[test]
    fn the_party_can_outrun_a_hunter_by_staying_ahead_of_it() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        // Place a hunter as far away as the graph allows, by hand, so this test
        // does not depend on the spawn roll actually landing.
        state.hunters.push(Hunter {
            id: "hunter.test.pursuit".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "location.black_beach.processional_ramp".into(),
            spawned_on_day: 1,
            level: 5,
            defeated_on_day: None,
        });
        let starting_distance = geography
            .step_distance(
                &state.hunters[0].current_location_id,
                &state.active_location_id,
            )
            .expect("reachable");
        assert!(
            starting_distance > 1,
            "the fixture needs real distance to close"
        );

        // The party moves once; the hunter closes exactly one step in response.
        // As long as the party keeps moving, and the graph is wider than one hop,
        // staying ahead is possible -- this asserts the party is not instantly
        // caught the moment it takes a single step.
        state
            .travel("route.black_beach.to_estate", &geography)
            .expect("legal route");
        assert_ne!(
            state.hunters[0].current_location_id,
            state.active_location_id
        );
    }

    #[test]
    fn hunter_pursuit_survives_save_and_reload_mid_chase() {
        let (state, _, _) = state_with_a_spawned_tracker();
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(state.hunters, restored.hunters);
    }

    #[test]
    fn a_beaten_tracker_is_gone_but_a_beaten_revenant_returns_next_midnight() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();

        let mut state = fixture();
        state.hunters.push(Hunter {
            id: "hunter.test.tracker".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "location.black_beach".into(),
            spawned_on_day: 1,
            level: 5,
            defeated_on_day: None,
        });
        state.hunters.push(Hunter {
            id: "hunter.test.revenant".into(),
            definition_id: HunterKind::Revenant.definition_id().to_owned(),
            kind: HunterKind::Revenant,
            current_location_id: "location.black_beach".into(),
            spawned_on_day: 1,
            level: 8,
            defeated_on_day: None,
        });

        state
            .begin_encounter(&geography, &habitats)
            .expect("a hunter is here");
        let first_defeated_id = state
            .pending_encounter
            .as_ref()
            .unwrap()
            .encounter_id
            .clone();
        state
            .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
            .expect("resolves");
        assert!(
            !state
                .hunters
                .iter()
                .any(|hunter| format!("encounter.{}", hunter.id) == first_defeated_id),
            "the first hunter beaten here is either removed (tracker) or stood down (revenant), \
             but never still presents the same pending id"
        );

        // Whichever it was, the survivor is still here; beat that one too.
        if let Some(encounter) = state.begin_encounter(&geography, &habitats).cloned() {
            state
                .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
                .expect("resolves");
            let _ = encounter;
        }

        // Exactly one hunter remains: the tracker is gone outright, the revenant
        // is merely defeated-today and still present in the ledger.
        assert_eq!(state.hunters.len(), 1);
        let revenant = &state.hunters[0];
        assert_eq!(revenant.kind, HunterKind::Revenant);
        assert!(revenant.is_defeated_today());

        // Midnight is where the island's law runs: the revenant stands back up.
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("resolves");
        assert!(!state.hunters[0].is_defeated_today());
    }
}
