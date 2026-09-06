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

use crate::geography::{AnchorDefinition, AnchorKind, Geography, RouteOption};
use crate::habitat::{Habitats, LootTable};
use crate::hunter::{self, Hunter, HunterKind};
use crate::strategy::clocks::{
    CONFRONTATION_DAY, CONFRONTATION_DISCOVERY_ID, CTHULHU_FACTION_ID, ConfrontationTrigger,
    HeatEvent, MAX_HEAT, WeatherState, assistance_weight, region_corruption_pressure,
};
use crate::strategy::directive::StrategicDirective;
use crate::strategy::faction::FactionDefinitions;
use crate::strategy::journal::{JournalEntry, StrategicJournal};
use crate::strategy::recruitment::{RecruitmentStage, RecruitmentState};
use crate::strategy::tick::{self, HOURS_PER_DAY, StrategicClock, StrategicEvent};
use crate::world::{
    DeathMemory, NamedPerson, SpawnRule, SpawnedMonster, WorldClock, WorldEvent, mix_seed,
};

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
    /// Optional authored household result applied when this encounter is won.
    /// The battle does not infer upgrades; the trigger carries the stable ID.
    #[serde(default)]
    pub estate_upgrade_id: Option<String>,
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
    /// Stable encounter IDs resolved in this campaign. Simulation state, not a
    /// UI flag: it stops an authored one-time encounter re-arming after a
    /// save/reload or a later return to its location.
    #[serde(default)]
    pub resolved_encounter_ids: BTreeSet<String>,
    /// The last campaign day each anchor was used on. A once-per-day anchor is
    /// spent when its entry names today; the entry is kept rather than cleared
    /// at midnight so the record survives a save/reload without a sweep.
    #[serde(default)]
    pub anchor_uses: BTreeMap<String, u32>,
    pub rng_seed: u64,
    /// S1: the strategic layer's mutable half, keyed by `faction.<concept_key>`.
    /// The authored `FactionDefinition` beside each of these is content and is
    /// reloaded rather than saved; only what a faction *has* lives here.
    /// `serde(default)` so every save written before the strategic layer
    /// existed still loads at the same `save_version`, with no factions.
    #[serde(default)]
    pub factions: BTreeMap<String, crate::strategy::faction::FactionState>,
    /// S12: where each woman's relationship to Captain Michael and his faction
    /// stands, keyed by character ID. Moved only by
    /// [`ExpeditionState::record_recruitment_milestone`] -- authored beats, never
    /// a timer. See `strategy/recruitment.rs` for the projection contract that
    /// keeps this out of the bridge as anything but a stage word and a beat ID.
    #[serde(default)]
    pub recruitment: BTreeMap<String, RecruitmentState>,
    /// S2: who holds each cell right now, keyed by `world.cell.*` ID and valued
    /// with a `faction.<concept_key>` ID. This is the mutable truth about
    /// control; the `Geography` is static content and carries only whatever an
    /// authored starting map declares. A cell absent from this map falls back to
    /// the graph's own `owner_faction_id`, and a cell held by nobody is simply
    /// not a key -- there is no "unowned" sentinel string to get misspelled.
    ///
    /// Nothing derived from it is stored: a road's live danger is
    /// `Geography::effective_risk`, recomputed from this map every time it is
    /// asked. `serde(default)` so saves written before this field existed load
    /// unchanged at the same `save_version`.
    #[serde(default)]
    pub ownership: BTreeMap<String, String>,
    /// S4: where the strategic simulation stands. The same clock as
    /// `campaign_day`, read at the hour rather than at the day: after any
    /// midnight `total_hours == (campaign_day - 1) * 24`, because
    /// [`ExpeditionState::resolve_midnight_in`] runs twenty-four strategic
    /// hours before the character-scale Midnight Return.
    ///
    /// There is no `paused` beside it, and there never will be: pausing is the
    /// bridge not calling [`ExpeditionState::strategic_tick`], so a pause has
    /// nothing here to change (brief section 17, "normal pausing must not
    /// change simulation outcomes"). `serde(default)` so a save written before
    /// the strategic clock existed resumes at hour zero of its own day.
    #[serde(default)]
    pub strategic_clock: StrategicClock,
    /// S11: what the island did, as the save keeps it. Brief section 17 lists
    /// "strategic event history" among the things a save must preserve; a
    /// literal append-only list of it would grow for as long as a campaign
    /// runs, so this is the bounded form -- a window of recent entries plus an
    /// order-sensitive digest of everything older, which together lose
    /// nothing that could distinguish two histories. Written only by
    /// [`ExpeditionState::strategic_tick`], through
    /// [`StrategicJournal::push`]. `serde(default)` so a save written before
    /// the journal existed loads with an empty one.
    #[serde(default)]
    pub strategic_journal: StrategicJournal,
    /// S8: Cthulhu patience and heat -- the *second* clock, and the whole
    /// point of it is that it is not this struct's first one.
    /// `docs/GAME_BUILD_PLAN.md`'s dual-clock contract: world time
    /// (`campaign_day`) and hidden pressure are distinct state dimensions, and
    /// "advancing time must not silently imply an identical heat increase".
    ///
    /// Nothing in `resolve_midnight_in`, `strategic_tick` or `travel` touches
    /// this field. It moves through
    /// [`ExpeditionState::record_heat_event`] and nowhere else, because a
    /// board event is the only thing entitled to move it. `serde(default)` so
    /// a save written before hidden pressure existed loads at zero pressure.
    #[serde(default)]
    pub cthulhu_heat: u32,
    /// S8: what the sky is doing over each region, keyed by `region_id` (the
    /// key on every `LocationRecord`), redrawn once per region per midnight.
    ///
    /// Stored rather than recomputed on demand only so a reader -- the bridge,
    /// a journal entry, a faction's `weather_preferences` -- gets one answer
    /// for the day without needing the seed. It is fully determined by
    /// [`WeatherState::draw`], so a lost entry is redrawn identically at the
    /// next midnight. `serde(default)`: an older save has no sky until its
    /// next midnight fills one in.
    #[serde(default)]
    pub weather: BTreeMap<String, WeatherState>,
    /// S8: how far corruption has gone in each cell, keyed by `world.cell.*`
    /// ID, `0..=255`. Absent means uncorrupted; there is no zero sentinel to
    /// misread.
    ///
    /// **Accumulation only. Reversibility is Open (brief section 20, "Which
    /// corruption effects are reversible") -- `blocked: needs decision`.**
    /// There is no decay, no cleanse, and no midnight sweep that lowers a
    /// value: [`ExpeditionState::corrupt_cell`] saturates upward and is the
    /// only writer. When the decision lands, it lands as a second method
    /// beside that one, not as a tuning constant here.
    #[serde(default)]
    pub corruption: BTreeMap<String, u8>,
    /// S6: the player's standing requests to the factions, keyed by the
    /// `directive.*` ID the caller supplied. Brief section 5.9: "Directions
    /// should persist until completed, cancelled, superseded, made impossible,
    /// or returned for reconsideration" -- so a directive lives here until one
    /// of those five endings, and stays afterwards as the record of what was
    /// asked and how it finished.
    ///
    /// A directive is an *input* to the strategic scoring, never a command:
    /// `BoardView::of` reads the Active ones and `choose_goals` weighs them
    /// heavily, and a faction fighting for its life still does not obey.
    /// Everything that reads or writes this map lives in
    /// `strategy/directive.rs`, including the methods on this struct.
    /// `serde(default)` so a save written before directives existed loads with
    /// none.
    #[serde(default)]
    pub directives: BTreeMap<String, StrategicDirective>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpeditionError {
    MalformedJson(String),
    FutureSaveVersion {
        found: u32,
        current: u32,
    },
    InvalidStableId {
        field: &'static str,
        value: String,
    },
    InvalidPartySize {
        found: usize,
    },
    IllegalRoute {
        route_id: String,
    },
    /// D3's truth-space gate: `route_id` requires `discovery_id` in
    /// `discoveries` and it isn't there yet.
    MissingDiscovery {
        route_id: String,
        discovery_id: String,
    },
    NoPendingEncounter,
    MidnightBlockedByPendingEncounter,
    InsufficientSupplies {
        needed: u32,
        available: u32,
    },
    DuplicatePortal {
        id: String,
    },
    /// Content named an anchor kind the simulation has no rule for. Refused at
    /// configuration rather than defaulted, so a typo cannot become a room
    /// that silently does nothing.
    UnknownAnchorKind {
        anchor_id: String,
        kind: String,
    },
    DuplicateAnchor {
        id: String,
    },
    DuplicateEncounterTrigger,
    /// The anchor exists, but not at the location the party is standing in.
    AnchorNotHere {
        anchor_id: String,
    },
    /// A once-per-day anchor already gave what it had today.
    AnchorSpentToday {
        anchor_id: String,
        used_on_day: u32,
    },
    /// The anchor needs something the party has not seen yet. Reading the
    /// estate's map table means nothing before the waymark has been looked at.
    AnchorRequiresDiscovery {
        anchor_id: String,
        discovery_id: String,
    },
    /// The anchor's one lasting result is already recorded, so working it again
    /// would gain nothing. This is what spends an estate room: not the clock,
    /// but the fact it exists to produce.
    AnchorAlreadyResolved {
        anchor_id: String,
        fact_id: String,
    },
    /// The party cannot walk away from a fight it has already been offered.
    TravelBlockedByEncounter {
        encounter_id: String,
    },
    /// S12: a milestone was recorded against a woman this campaign has no
    /// recruitment record for. Refused rather than created, because a woman is
    /// authored content -- buildings do not manufacture women, and neither does
    /// a stray milestone ID.
    UnknownRecruit {
        character_id: String,
    },
    /// S12: that authored beat has already played for her. Replaying it would
    /// be the one way to walk a stage forward without new authored content.
    MilestoneAlreadyRecorded {
        character_id: String,
        milestone_id: String,
    },
    /// S2: control was set on a cell the graph has never heard of. Refused
    /// before anything mutates, so a typo cannot invent a place by writing to
    /// it -- `ownership` is a map, and a bare insert would happily create one.
    UnknownCell {
        cell_id: String,
    },
    /// An `inspect:<id>` for an observation the current location does not
    /// declare. Refused before mutation: the legal list never offered it here.
    ObservationNotHere {
        observation_id: String,
        location_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TravelOutcome {
    pub arrived_at: String,
    pub time_cost_minutes: u32,
    pub supply_cost: u32,
    /// S2: how dangerous this road actually was when the party walked it --
    /// `Geography::effective_risk`, not the route's authored `risk_level`. The
    /// two differ exactly when the road was contested. Reported rather than
    /// stored, so the caller learns what it crossed without anyone keeping a
    /// second copy of a number that changes whenever control does.
    pub risk_level: u8,
}

/// What using an anchor actually produced. One outcome shape for every anchor
/// kind, so a caller reads the same fields whether the party salvaged a wreck,
/// opened a cache, or used a room at the estate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnchorOutcome {
    pub anchor_id: String,
    pub rations_gained: u32,
    pub medicine_gained: u32,
    pub coin_gained: u32,
    pub medicine_spent: u32,
    pub healed_character_ids: Vec<String>,
    pub discoveries_recorded: Vec<String>,
    /// Household facts this anchor recorded -- the workshop's field rig, the
    /// infirmary's recovery. Named rather than implied so a caller learns what
    /// changed without re-reading `household_progress`.
    pub upgrades_recorded: Vec<String>,
}

/// B2: how a battle ended, as reported back through `Battle`'s own `BattlePhase`
/// (`Victory` / `Defeat` / `Retreated`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterOutcome {
    Victory,
    Defeat,
    Retreat,
}

/// What resolving an encounter settled: how it ended, and what the beaten
/// individual was carrying. `loot` is `Some` only on a victory over the
/// individual that actually holds this habitat today -- a hunter carries
/// nothing, and neither does an encounter the party lost or fled.
#[derive(Clone, Debug, PartialEq)]
pub struct EncounterResolution {
    pub outcome: EncounterOutcome,
    pub loot: Option<LootTable>,
}

/// A4: what the estate's rooms cost and what they leave behind. Each room is an
/// anchor on the one `use_anchor` mechanism, so these are the rules of those
/// anchors rather than the constants of a separate estate subsystem. Which cell
/// they stand in is the graph's answer (`Geography::anchor_is_at`), not a
/// hard-coded location ID -- the reason `ESTATE_LOCATION_ID` is gone.
///
/// B4's infirmary rule, unchanged in substance: one dose of medicine, twenty
/// Vitality, one permanent household fact.
const INFIRMARY_MEDICINE_COST: u32 = 1;
const INFIRMARY_VITALITY_RESTORED: i32 = 20;
const INFIRMARY_UPGRADE_RESTED: &str = "estate.upgrade.infirmary_rested";
/// The workshop fits recovered parts to the party's kit. It needs the wreck to
/// have actually been looked at first, because the parts come off the Handsome
/// Jack.
///
/// The card wrote this gate as `observation.black_beach.wreck_of_handsome_jack`
/// and the river gate below as `observation.river_landing.elven_waymark`.
/// Neither string exists: `content/world/` -- canonical for world IDs since A2 --
/// declares `observation.black_beach.wreck` for the wreck of the Handsome Jack
/// and `observation.river_landing.road_marker` for the bronze-green elven marker
/// beside the jetty. Minting the card's spellings would have recreated exactly
/// the two-ID-sets-for-one-place drift A2 existed to kill, so the gates name the
/// authored observations instead.
pub(crate) const WORKSHOP_REQUIRED_DISCOVERY_ID: &str = "observation.black_beach.wreck";
const WORKSHOP_UPGRADE_FIELD_RIG: &str = "estate.upgrade.workshop_field_rig";
/// The map table reads a tidal cut along the shore out of the elven waymark's
/// roadwork. What it records is a route, so it lands in `discoveries` where the
/// portal gate looks for it, not in the household ledger.
pub(crate) const MAP_TABLE_REQUIRED_DISCOVERY_ID: &str = "observation.river_landing.road_marker";
pub(crate) const MAP_TABLE_DISCOVERY_TIDAL_CUT: &str = "discovery.map_table.tidal_cut";
/// What the field rig is worth on the road: one ration off every leg that
/// charges any, floored at free.
const FIELD_RIG_SUPPLY_SAVING: u32 = 1;

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
            resolved_encounter_ids: BTreeSet::new(),
            anchor_uses: BTreeMap::new(),
            rng_seed: seed,
            factions: BTreeMap::new(),
            recruitment: BTreeMap::new(),
            // S2: a fresh campaign starts with the island unclaimed. The static
            // graph carries whatever an authored starting map declares.
            ownership: BTreeMap::new(),
            // S4: the first hour of the first day. Nothing has been drawn yet,
            // so the determinism witness is still zero.
            strategic_clock: StrategicClock::new(),
            // S11: a fresh campaign has no history yet -- no entries, nothing
            // evicted, so the history digest is still zero.
            strategic_journal: StrategicJournal::new(),
            // S8: a fresh campaign starts with no hidden pressure, no drawn
            // sky, and an uncorrupted island. The first midnight draws the
            // weather; nothing draws pressure, ever.
            cthulhu_heat: 0,
            weather: BTreeMap::new(),
            corruption: BTreeMap::new(),
            // S6: nobody has asked the island for anything yet.
            directives: BTreeMap::new(),
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
        // Five heroes: Captain Michael and four women (continuation brief §1).
        if self.party_ids.is_empty() || self.party_ids.len() > 5 {
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
    /// The routes the party may actually take from here: every ungated route,
    /// plus a gated one whose discovery the party holds. A locked door the
    /// party does not know about is not a legal command -- offering it and
    /// then refusing it with `MissingDiscovery` would tell the player the
    /// door exists, which is exactly what the gate is for. A road the party
    /// cannot yet afford *is* listed: that refusal is information the player
    /// should have.
    pub fn legal_routes<'a>(&self, geography: &'a Geography) -> Vec<&'a RouteOption> {
        geography
            .routes_from(&self.active_location_id)
            .into_iter()
            .filter(|route| {
                route
                    .required_discovery_id
                    .as_ref()
                    .is_none_or(|discovery_id| self.discoveries.contains(discovery_id))
            })
            .collect()
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
        if let Some(encounter) = &self.pending_encounter {
            return Err(ExpeditionError::TravelBlockedByEncounter {
                encounter_id: encounter.encounter_id.clone(),
            });
        }
        let route = geography
            .route(route_id)
            .filter(|route| route.from_location_id == self.active_location_id)
            .ok_or_else(|| ExpeditionError::IllegalRoute {
                route_id: route_id.to_owned(),
            })?
            .clone();

        if let Some(discovery_id) = &route.required_discovery_id {
            if !self.discoveries.contains(discovery_id) {
                return Err(ExpeditionError::MissingDiscovery {
                    route_id: route_id.to_owned(),
                    discovery_id: discovery_id.clone(),
                });
            }
        }
        // A3: rations are a real constraint on where the party may go. A road
        // the party cannot feed itself along is refused outright, before
        // anything moves -- not walked for free by a `saturating_sub` that
        // quietly floors an empty pack at zero.
        let supply_cost = self.effective_supply_cost(&route);
        // S2: one owner for "how dangerous is this road right now". Read before
        // the move, from the control standing when the party set out.
        let risk_level = geography.effective_risk(&route, &self.ownership);
        if self.supplies.rations < supply_cost {
            return Err(ExpeditionError::InsufficientSupplies {
                needed: supply_cost,
                available: self.supplies.rations,
            });
        }

        self.advance_time(route.time_cost_minutes);
        self.supplies.rations -= supply_cost;
        self.route_history.push(RouteStep {
            location_id: route.to_location_id.clone(),
            arrived_on_day: self.campaign_day,
            arrived_segment: self.time_segment.clone(),
        });
        self.active_location_id = route.to_location_id.clone();
        // The world moves when the party does: every hunter closes one step
        // toward wherever the party now stands.
        hunter::advance_hunters(
            &mut self.hunters,
            geography,
            &self.active_location_id,
            &self.discoveries,
        );

        Ok(TravelOutcome {
            arrived_at: self.active_location_id.clone(),
            time_cost_minutes: route.time_cost_minutes,
            // What the party actually paid, not what the road lists: with the
            // field rig fitted these differ, and the caller wants the charge.
            supply_cost,
            risk_level,
        })
    }

    /// S2: hand a cell to a faction, or release it with `None`. The single way
    /// `ownership` changes.
    ///
    /// Rejects an unknown cell before any mutation, so a mistyped ID cannot
    /// invent a place by being written to. Setting the control a cell already
    /// has is a no-op that returns no events -- a change that changed nothing
    /// is not news, and an event stream that repeats itself is one a listener
    /// learns to distrust. Otherwise it returns the one
    /// [`WorldEvent::ControlChanged`] describing the handover.
    ///
    /// Nothing about risk is written here. Every road touching this cell simply
    /// answers `Geography::effective_risk` differently from the next call
    /// onward, with no route record edited.
    pub fn set_control(
        &mut self,
        cell_id: &str,
        faction_id: Option<String>,
        geography: &Geography,
    ) -> Result<Vec<WorldEvent>, ExpeditionError> {
        if geography.location(cell_id).is_none() {
            return Err(ExpeditionError::UnknownCell {
                cell_id: cell_id.to_owned(),
            });
        }
        if let Some(faction_id) = &faction_id {
            require_stable_id("faction_id", faction_id)?;
        }
        // The graph's authored owner is the standing answer for a cell that
        // `ownership` has never named, so it is what a first claim moves *from*
        // -- and what releasing a cell moves back *to*. `from` and `to` are
        // therefore the effective controller either side of the change, which
        // is the only reading under which a release is honest: dropping the
        // override on a cell the graph gives an owner hands it back to that
        // owner, and reporting `None` there would be a lie about the board.
        // (Content authors no owners today, so that is unheld everywhere.)
        let authored = geography.controller(cell_id).map(str::to_owned);
        let from = self
            .ownership
            .get(cell_id)
            .cloned()
            .or_else(|| authored.clone());
        let to = match &faction_id {
            Some(faction_id) => Some(faction_id.clone()),
            None => authored,
        };
        if from == to {
            return Ok(Vec::new());
        }
        match &faction_id {
            Some(faction_id) => {
                self.ownership
                    .insert(cell_id.to_owned(), faction_id.clone());
            }
            None => {
                self.ownership.remove(cell_id);
            }
        }
        Ok(vec![WorldEvent::ControlChanged {
            cell_id: cell_id.to_owned(),
            from,
            to,
            day: self.campaign_day,
        }])
    }

    /// S8: move Cthulhu patience and heat, and the **only** way it moves.
    ///
    /// `docs/GAME_BUILD_PLAN.md`'s dual-clock contract in one method: hidden
    /// pressure changes because a [`HeatEvent`] happened on the board, never
    /// because time passed. There is no `heat += 1` anywhere else in this
    /// crate, no elapsed-time variant of `HeatEvent` to pass here, and no way
    /// for a caller to name an amount of its own -- the amount belongs to the
    /// event (`HeatEvent::amount`), so tuning stays in
    /// `strategy/clocks.rs` and cannot drift per call site.
    ///
    /// Saturates at [`MAX_HEAT`], which is also the third confrontation route:
    /// pressure at its ceiling is what
    /// [`ExpeditionState::confrontation_is_triggered`] reads as "maximum
    /// hidden pressure". Returns the pressure after the event, so a caller
    /// that wants to know whether this was the event that filled the clock can
    /// see it without reading the field back.
    ///
    /// Nothing here is reversible and nothing decays. Hidden pressure is
    /// "irreversible hidden pressure" in the brief's own words (section 13).
    pub fn record_heat_event(&mut self, event: HeatEvent) -> u32 {
        self.cthulhu_heat = self
            .cthulhu_heat
            .saturating_add(event.amount())
            .min(MAX_HEAT);
        self.cthulhu_heat
    }

    /// S8: deepen the corruption of one cell, saturating at 255.
    ///
    /// Refused before anything mutates when the graph has never heard of the
    /// cell -- the same rule and the same [`ExpeditionError::UnknownCell`] as
    /// [`ExpeditionState::set_control`], for the same reason: `corruption` is
    /// a map, and a bare insert would happily invent a place by writing to it.
    ///
    /// **Accumulation only.** Whether corruption can be undone is Open (brief
    /// section 20) -- `blocked: needs decision` -- so there is no counterpart
    /// to this method, no `amount` a caller can make negative, and no sweep
    /// that lowers a cell over time. Corruption is a `u8`, so a decision to
    /// make it reversible costs a second method here and no migration.
    ///
    /// Returns the cell's corruption after the event.
    pub fn corrupt_cell(
        &mut self,
        cell_id: &str,
        amount: u8,
        geography: &Geography,
    ) -> Result<u8, ExpeditionError> {
        if geography.location(cell_id).is_none() {
            return Err(ExpeditionError::UnknownCell {
                cell_id: cell_id.to_owned(),
            });
        }
        let entry = self.corruption.entry(cell_id.to_owned()).or_insert(0);
        *entry = entry.saturating_add(amount);
        Ok(*entry)
    }

    /// S8: has the confrontation's opening condition been met, and by which of
    /// brief section 13's three routes?
    ///
    /// Derived, never stored: asked again a moment later it answers from the
    /// same state, so there is no flag that can disagree with the day count,
    /// the discovery set or the pressure. Reporting the route matters because
    /// the three are different stories -- the party walked into it, the island
    /// ran out of time, or the pressure filled -- and a bare `bool` would throw
    /// that away.
    ///
    /// Order is deliberate: a deliberate discovery is the route the party
    /// chose, so it outranks the two that happen *to* them.
    ///
    /// **Not modelled here:** what the party may do to a summoning once the
    /// confrontation begins. Summoning-interruption rules are Open (brief
    /// section 20) -- `blocked: needs decision`.
    pub fn confrontation_is_triggered(&self) -> Option<ConfrontationTrigger> {
        if self.discoveries.contains(CONFRONTATION_DISCOVERY_ID) {
            return Some(ConfrontationTrigger::Discovery);
        }
        if self.campaign_day >= CONFRONTATION_DAY {
            return Some(ConfrontationTrigger::DayHundred);
        }
        if self.cthulhu_heat >= MAX_HEAT {
            return Some(ConfrontationTrigger::MaximumHeat);
        }
        None
    }

    /// S8: how much weight a Cthulhu assistance event is entitled to right
    /// now. Zero means it does not fire.
    ///
    /// Brief section 13: assistance events are weighted and state-gated, and
    /// "must not simply rescue Cthulhu whenever it is losing". Non-zero only
    /// when the faction stands in `Advantaged` or `Closing`; zero for
    /// `Desperate`, `Recovering` and `Contesting`; zero when the save carries
    /// no Cthulhu faction at all.
    ///
    /// **The gate is the deliverable, not the event.** What an assistance
    /// event places on the board, and the visible board conditions it must
    /// arise from, are S7's and S10's. They call this first.
    pub fn cthulhu_assistance_weight(&self) -> u32 {
        assistance_weight(self.factions.get(CTHULHU_FACTION_ID))
    }

    /// S8: draw every region's weather for `day`.
    ///
    /// Private: weather is a consequence of a midnight, not a verb. The
    /// regions are the ones the board actually has -- read off the graph's
    /// cells -- so a region cannot get weather by being named in a save, and a
    /// region on the board cannot be missed.
    fn draw_weather_for_day(&mut self, geography: &Geography, day: u32) {
        let mut regions: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for location in geography.locations.values() {
            regions
                .entry(location.region_id.clone())
                .or_default()
                .push(location.id.as_str());
        }
        for (region_id, cell_ids) in regions {
            let pressure = region_corruption_pressure(&self.corruption, cell_ids.into_iter());
            let weather = WeatherState::draw(self.rng_seed, day, &region_id, pressure);
            self.weather.insert(region_id, weather);
        }
    }

    /// A3: the one "do something here" verb. Every anchor kind -- salvaging the
    /// wreck, opening a cache, and (A4) the estate's rooms -- goes through this
    /// method, so the game never grows a second interaction path per verb.
    ///
    /// Rejects before touching anything when the anchor is not at the current
    /// location, when a once-per-day anchor has already been used today, or when
    /// an encounter is still open (the same rule `travel` applies: nothing else
    /// happens while a fight is pending).
    ///
    /// A salvage yield is deterministic, not random: the declared rations are a
    /// floor and `world.rs`'s existing `mix_seed` decides, from the campaign
    /// seed, the day and the anchor's own ID, whether the day turns up one more.
    /// The same save on the same day always salvages the same amount, and no
    /// random-number crate enters the simulation.
    pub fn use_anchor(
        &mut self,
        anchor_id: &str,
        geography: &Geography,
    ) -> Result<AnchorOutcome, ExpeditionError> {
        if let Some(encounter) = &self.pending_encounter {
            return Err(ExpeditionError::TravelBlockedByEncounter {
                encounter_id: encounter.encounter_id.clone(),
            });
        }
        if !geography.anchor_is_at(anchor_id, &self.active_location_id) {
            return Err(ExpeditionError::AnchorNotHere {
                anchor_id: anchor_id.to_owned(),
            });
        }
        let anchor = geography
            .anchor(anchor_id)
            .expect("anchor_is_at proved the definition exists")
            .clone();
        if let Some(refusal) = self.anchor_refusal(&anchor) {
            return Err(refusal);
        }

        let mut outcome = AnchorOutcome {
            anchor_id: anchor_id.to_owned(),
            ..AnchorOutcome::default()
        };
        match anchor.kind {
            AnchorKind::Salvage { rations, coin } => {
                let bonus = (mix_seed(self.rng_seed, self.campaign_day, anchor_id, 0) % 2) as u32;
                outcome.rations_gained = rations + bonus;
                outcome.coin_gained = coin;
            }
            AnchorKind::LootCache {
                rations,
                medicine,
                coin,
            } => {
                outcome.rations_gained = rations;
                outcome.medicine_gained = medicine;
                outcome.coin_gained = coin;
            }
            AnchorKind::Infirmary => {
                // B4's infirmary rule itself, not a call out to a second copy
                // of it: A4 inlined the old standalone estate-rest method here
                // and deleted it, so treating the party's injuries has exactly
                // one owner.
                // `anchor_refusal` already proved the medicine is there.
                self.supplies.medicine -= INFIRMARY_MEDICINE_COST;
                outcome.medicine_spent = INFIRMARY_MEDICINE_COST;
                for (id, character) in self.character_states.iter_mut() {
                    character.vitality = (character.vitality + INFIRMARY_VITALITY_RESTORED)
                        .min(character.max_vitality);
                    character.injured = false;
                    outcome.healed_character_ids.push(id.clone());
                }
                self.household_progress
                    .estate_upgrades
                    .insert(INFIRMARY_UPGRADE_RESTED.to_owned());
                outcome
                    .upgrades_recorded
                    .push(INFIRMARY_UPGRADE_RESTED.to_owned());
            }
            AnchorKind::Workshop => {
                // Wreck parts become a field rig: a permanent household fact
                // that `effective_supply_cost` reads on every leg thereafter.
                self.household_progress
                    .estate_upgrades
                    .insert(WORKSHOP_UPGRADE_FIELD_RIG.to_owned());
                outcome
                    .upgrades_recorded
                    .push(WORKSHOP_UPGRADE_FIELD_RIG.to_owned());
            }
            AnchorKind::MapTable => {
                // A route, recorded where routes are gated from: the tidal cut
                // portal is illegal until this discovery exists.
                self.discoveries
                    .insert(MAP_TABLE_DISCOVERY_TIDAL_CUT.to_owned());
                outcome
                    .discoveries_recorded
                    .push(MAP_TABLE_DISCOVERY_TIDAL_CUT.to_owned());
            }
            AnchorKind::Inspect => {
                outcome.discoveries_recorded = self.inspect(geography);
            }
        }

        self.supplies.rations += outcome.rations_gained;
        self.supplies.medicine += outcome.medicine_gained;
        self.supplies.coin += outcome.coin_gained;
        self.anchor_uses
            .insert(anchor_id.to_owned(), self.campaign_day);
        Ok(outcome)
    }

    /// Why this anchor cannot be worked from this state, or `None` when it can.
    ///
    /// The single owner of anchor legality. `use_anchor` rejects with it before
    /// touching anything and `legal_next_commands_with_geography` filters with
    /// it, so the set of commands the game offers and the set it accepts cannot
    /// drift apart. Location is deliberately not its business: that is
    /// `Geography::anchor_is_at`, checked by the caller.
    fn anchor_refusal(&self, anchor: &AnchorDefinition) -> Option<ExpeditionError> {
        if anchor.once_per_day && self.anchor_uses.get(&anchor.id) == Some(&self.campaign_day) {
            return Some(ExpeditionError::AnchorSpentToday {
                anchor_id: anchor.id.clone(),
                used_on_day: self.campaign_day,
            });
        }
        // What the room needs to have been seen, and the one lasting fact it
        // exists to produce. A room whose fact is already recorded is spent for
        // good -- the estate's rooms are one-time strategic gains, not a daily
        // allowance, which is the rule B4 gave the infirmary and A4 keeps.
        let (required_discovery_id, granted_fact_id) = match anchor.kind {
            AnchorKind::Infirmary => (None, Some(INFIRMARY_UPGRADE_RESTED)),
            AnchorKind::Workshop => (
                Some(WORKSHOP_REQUIRED_DISCOVERY_ID),
                Some(WORKSHOP_UPGRADE_FIELD_RIG),
            ),
            AnchorKind::MapTable => (
                Some(MAP_TABLE_REQUIRED_DISCOVERY_ID),
                Some(MAP_TABLE_DISCOVERY_TIDAL_CUT),
            ),
            AnchorKind::Salvage { .. } | AnchorKind::LootCache { .. } | AnchorKind::Inspect => {
                (None, None)
            }
        };
        if let Some(discovery_id) = required_discovery_id {
            if !self.discoveries.contains(discovery_id) {
                return Some(ExpeditionError::AnchorRequiresDiscovery {
                    anchor_id: anchor.id.clone(),
                    discovery_id: discovery_id.to_owned(),
                });
            }
        }
        if let Some(fact_id) = granted_fact_id {
            if self.holds_fact(fact_id) {
                return Some(ExpeditionError::AnchorAlreadyResolved {
                    anchor_id: anchor.id.clone(),
                    fact_id: fact_id.to_owned(),
                });
            }
        }
        if anchor.kind == AnchorKind::Infirmary && self.supplies.medicine < INFIRMARY_MEDICINE_COST
        {
            return Some(ExpeditionError::InsufficientSupplies {
                needed: INFIRMARY_MEDICINE_COST,
                available: self.supplies.medicine,
            });
        }
        None
    }

    /// Whether the campaign already holds a lasting fact, wherever it is kept:
    /// a household upgrade lives in `household_progress`, a route the party
    /// learned lives in `discoveries`, and an anchor's gate should not have to
    /// know which of the two its own result is.
    fn holds_fact(&self, fact_id: &str) -> bool {
        self.household_progress.estate_upgrades.contains(fact_id)
            || self.discoveries.contains(fact_id)
    }

    /// A4: the single owner of what a leg of road actually costs to walk. The
    /// route's declared `supply_cost` is its price; the workshop's field rig
    /// makes the party pack lighter. Every path that charges rations --
    /// `travel` and the retreat leg of `resolve_encounter` -- asks this rather
    /// than reading `route.supply_cost`, so the discount cannot apply on the
    /// way out and go missing on the way back.
    pub fn effective_supply_cost(&self, route: &RouteOption) -> u32 {
        if self
            .household_progress
            .estate_upgrades
            .contains(WORKSHOP_UPGRADE_FIELD_RIG)
        {
            route.supply_cost.saturating_sub(FIELD_RIG_SUPPLY_SAVING)
        } else {
            route.supply_cost
        }
    }

    /// B1's observation query. Records every observation at the current location
    /// as a discovery (idempotent -- `discoveries` is a set) and returns their IDs.
    pub fn inspect(&mut self, geography: &Geography) -> Vec<String> {
        let Some(location) = geography.location(&self.active_location_id) else {
            return Vec::new();
        };
        // Looking around the whole place is the sum of reading each thing in
        // it; one rule records an observation, and this walks it.
        let observation_ids = location.observation_ids.clone();
        for observation_id in &observation_ids {
            self.inspect_observation(observation_id, geography)
                .expect("an observation the location declares is here");
        }
        observation_ids
    }

    /// Reads one thing: the `inspect:<observation_id>` command the legal list
    /// offers, one button per observation. Records exactly that observation
    /// and nothing else, so what the screen says the party read is what the
    /// party read. Refused before mutation when the observation is not one the
    /// current location declares -- the command was never legal here.
    pub fn inspect_observation(
        &mut self,
        observation_id: &str,
        geography: &Geography,
    ) -> Result<(), ExpeditionError> {
        let declared_here = geography
            .location(&self.active_location_id)
            .is_some_and(|location| {
                location
                    .observation_ids
                    .iter()
                    .any(|id| id == observation_id)
            });
        if !declared_here {
            return Err(ExpeditionError::ObservationNotHere {
                observation_id: observation_id.to_owned(),
                location_id: self.active_location_id.clone(),
            });
        }
        self.discoveries.insert(observation_id.to_owned());
        Ok(())
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
        // An authored one-time encounter is content's own claim on this place
        // and answers first -- once. After it resolves the ledger remembers it.
        if let Some(trigger) = geography
            .encounter_trigger_for(&self.active_location_id)
            .filter(|trigger| !self.resolved_encounter_ids.contains(&trigger.encounter_id))
        {
            self.pending_encounter = Some(EncounterState {
                encounter_id: trigger.encounter_id.clone(),
                battle_id: trigger.battle_id.clone(),
                estate_upgrade_id: trigger.estate_upgrade_id.clone(),
            });
            return self.pending_encounter.as_ref();
        }
        if let Some(hunter_id) = hunter::hunter_at(&self.hunters, &self.active_location_id)
            .filter(|hunter| !hunter.is_defeated_today())
            .map(|hunter| hunter.id.clone())
        {
            self.pending_encounter = Some(EncounterState {
                encounter_id: format!("encounter.{hunter_id}"),
                battle_id: format!("battle.{hunter_id}"),
                estate_upgrade_id: None,
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
            estate_upgrade_id: None,
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
    /// holds its territory.
    ///
    /// A3: victory over a habitat's actual holder also pays. The habitat's
    /// `drop_table_id` -- declared since D1 and read by nothing until now --
    /// resolves against `Habitats::loot_table`, and the individual's own
    /// `loot_seed`, likewise declared and never read, decides the coin it was
    /// personally carrying (`loot_seed % 3`) on top of the table. A hunter drops
    /// nothing: it came for the party, it does not hold territory.
    pub fn resolve_encounter(
        &mut self,
        outcome: EncounterOutcome,
        geography: &Geography,
        habitats: &Habitats,
    ) -> Result<EncounterResolution, ExpeditionError> {
        if self.pending_encounter.is_none() {
            return Err(ExpeditionError::NoPendingEncounter);
        }
        let mut loot = None;
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
                // Only the individual the party actually fought pays out, which
                // is why the seed is taken from the spawn record whose encounter
                // ID matches the pending one rather than from whatever currently
                // sits in the ledger for this region.
                let carried_coin = self
                    .daily_spawn_records
                    .get(&habitat.region_id)
                    .into_iter()
                    .chain(self.nightly_spawn_records.get(&habitat.region_id))
                    .find(|monster| {
                        pending_encounter_id.as_deref()
                            == Some(format!("encounter.{}", monster.instance_id).as_str())
                    })
                    .map(|monster| (monster.loot_seed % 3) as u32);
                if let (Some(carried_coin), Some(table)) = (
                    carried_coin,
                    habitats.loot_table(&habitat.drop_table_id).cloned(),
                ) {
                    self.supplies.rations += table.rations;
                    self.supplies.medicine += table.medicine;
                    self.supplies.coin += table.coin + carried_coin;
                    loot = Some(table);
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
                    // The same owner of route cost as `travel`, so the field rig
                    // is worth the same ration whichever direction it is walked.
                    // The `saturating_sub` stays: a retreat is never refused for
                    // an empty pack, it is simply paid for with what is left.
                    let supply_cost = self.effective_supply_cost(&return_route);
                    self.supplies.rations = self.supplies.rations.saturating_sub(supply_cost);
                    self.route_history.push(RouteStep {
                        location_id: return_route.to_location_id.clone(),
                        arrived_on_day: self.campaign_day,
                        arrived_segment: self.time_segment.clone(),
                    });
                    self.active_location_id = return_route.to_location_id;
                }
            }
        }
        let encounter = self
            .pending_encounter
            .take()
            .expect("checked pending above");
        self.resolved_encounter_ids
            .insert(encounter.encounter_id.clone());
        if outcome == EncounterOutcome::Victory {
            if let Some(estate_upgrade_id) = encounter.estate_upgrade_id {
                self.household_progress
                    .estate_upgrades
                    .insert(estate_upgrade_id);
            }
        }
        Ok(EncounterResolution { outcome, loot })
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

    /// Stable, player-addressable travel commands for the active location, or
    /// the reason none are legal. Ordered by route ID, never by authoring or UI
    /// order. This is what the bridge projects to Godot.
    pub fn legal_route_commands(
        &self,
        geography: &Geography,
    ) -> Result<Vec<String>, ExpeditionError> {
        if let Some(encounter) = &self.pending_encounter {
            return Err(ExpeditionError::TravelBlockedByEncounter {
                encounter_id: encounter.encounter_id.clone(),
            });
        }
        Ok(self
            .legal_routes(geography)
            .into_iter()
            .map(|route| format!("travel:{}", route.id))
            .collect())
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
        // Every legal "do something here" verb, in the order the cell lists its
        // anchors. An anchor already spent today is not a legal command.
        for anchor in geography.anchors_at(&self.active_location_id) {
            if self.anchor_refusal(anchor).is_some() {
                continue;
            }
            commands.push(format!("anchor_action:{}", anchor.id));
        }
        commands
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
        // S4: the day that is ending is twenty-four strategic hours long, and
        // they run *before* the character-scale Midnight Return so that the
        // hours belong to the day they happened in. After this loop
        // `strategic_clock.hour_of_day` is back to zero and `total_hours` has
        // grown by a day; `resolve_midnight` then turns `campaign_day`, and the
        // two halves of the one clock agree again.
        //
        // The registry is empty here because this call site has no faction
        // content: `content/factions/*.json` is C9's card and the bridge that
        // would load it is a later B card. That costs nothing today -- a
        // strategic hour reads only the faction states the save carries, which
        // is asserted by `the_registry_cannot_change_a_tick_yet` in
        // `godot-rust/tests/strategic_determinism.rs`. When S5 makes the
        // records matter, that test fails, and this line is what it is telling
        // the B card to fix.
        let definitions = FactionDefinitions::new();
        for _ in 0..HOURS_PER_DAY {
            self.strategic_tick(geography, &definitions);
        }

        // S8: the day's weather, one draw per region, before the
        // character-scale Midnight Return turns `campaign_day` -- so the sky
        // is drawn for the day that is about to begin and every reader on that
        // day sees the same one. This is the only new draw S8 adds and it is
        // per region per day, so S4's per-faction per-hour sequence above is
        // untouched.
        //
        // It moves `weather` and reads `corruption`. It does not move
        // `cthulhu_heat`, and no midnight ever will: the dual-clock contract
        // is that a day passing is not an event in the second clock.
        // `thirty_midnights_do_not_move_hidden_pressure` is the guard.
        self.draw_weather_for_day(geography, self.campaign_day + 1);

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
                &self.discoveries,
            ) {
                self.hunters.push(hunter);
            }
        }
        for hunter in &mut self.hunters {
            hunter.defeated_on_day = None;
        }
        hunter::advance_hunters(
            &mut self.hunters,
            geography,
            &self.active_location_id,
            &self.discoveries,
        );

        Ok(events)
    }

    /// S4: one in-world hour of faction activity.
    ///
    /// The whole strategic simulation is driven by repeating this call, and
    /// nothing else drives it -- which is what makes pause free. A paused game
    /// is a game whose bridge is not calling this method; there is no `paused`
    /// field to get out of step with, and no branch here that could behave
    /// differently for having been interrupted. Brief section 17: "normal
    /// pausing must not change simulation outcomes."
    ///
    /// The logic lives in [`crate::strategy::tick`] with the rest of the
    /// strategic layer; this is the method on the state that owns the data.
    /// `geography` and `factions` are the board and the authored records the
    /// utility AI will score against (S5); an hour today advances the clock,
    /// makes each present faction's fixed sequence of draws, and reports the
    /// hour that ran.
    pub fn strategic_tick(
        &mut self,
        geography: &Geography,
        factions: &FactionDefinitions,
    ) -> Vec<StrategicEvent> {
        // S11: the hour's events are recorded before they are handed back, so
        // the save's history does not depend on what the caller does with the
        // return value. The journal is bounded: entries older than its window
        // fold into its digest rather than accumulating, so a campaign of any
        // length writes a save of the same size.
        //
        // The stamp is read before the hour runs, because that is the hour
        // that ran: `run_hour` advances the clock, so afterwards
        // `hour_of_day` names the next hour. Reading it here rather than
        // matching the event's own fields also means a variant S5 adds needs
        // no change in this method to be journalled correctly.
        let day = self.campaign_day;
        let hour = self.strategic_clock.hour_of_day;
        let events = tick::run_hour(self, geography, factions);
        for event in &events {
            self.strategic_journal
                .push(JournalEntry::new(day, hour, event.clone()));
        }
        events
    }

    /// S12: plays one authored recruitment beat for one woman.
    ///
    /// Rejects before mutating on an unknown woman, a malformed milestone ID, a
    /// malformed recruitment record, or a milestone already recorded for her.
    /// On success returns the stage she stands at afterward -- which is very
    /// often the stage she was already at, because a milestone moves her only
    /// when her authored rule for it permits it. Attraction creates openings,
    /// not allegiance, and nothing here advances on a clock.
    pub fn record_recruitment_milestone(
        &mut self,
        woman_id: &str,
        milestone_id: &str,
    ) -> Result<RecruitmentStage, ExpeditionError> {
        require_stable_id("milestone_id", milestone_id)?;
        let recruit =
            self.recruitment
                .get_mut(woman_id)
                .ok_or_else(|| ExpeditionError::UnknownRecruit {
                    character_id: woman_id.to_owned(),
                })?;
        recruit.validate()?;
        recruit.record_milestone(milestone_id).ok_or_else(|| {
            ExpeditionError::MilestoneAlreadyRecorded {
                character_id: woman_id.to_owned(),
                milestone_id: milestone_id.to_owned(),
            }
        })
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

pub(crate) fn require_stable_id(field: &'static str, value: &str) -> Result<(), ExpeditionError> {
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
    use crate::geography::CONTESTED_RISK_MODIFIER;

    fn fixture() -> ExpeditionState {
        let mut state = ExpeditionState::new(
            42,
            vec![
                "character.protagonist.captain".into(),
                "character.heroine.betty".into(),
            ],
            "world.cell.black_beach",
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
            estate_upgrade_id: None,
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

    /// S2's Done-when end to end: control of the river landing flips, and the
    /// road the party would actually walk is more dangerous -- with the route
    /// record untouched, and the change reported through `travel` rather than
    /// stored anywhere.
    #[test]
    fn flipping_the_river_landing_changes_what_the_safe_road_costs_to_walk() {
        let geography = Geography::black_beach_vertical_slice();
        const SAFE_ROAD: &str = "world.portal.river_landing_to_reception_terrace_safe_road";
        let authored_risk = geography
            .route(SAFE_ROAD)
            .expect("the fixture's safe road")
            .risk_level;

        let walk = |state: &mut ExpeditionState| {
            state.active_location_id = "world.cell.river_landing".into();
            state.supplies.rations = 20;
            state
                .travel(SAFE_ROAD, &geography)
                .expect("the safe road is legal and affordable")
                .risk_level
        };

        let mut uncontested = fixture();
        assert_eq!(walk(&mut uncontested), authored_risk);

        let mut contested = fixture();
        let events = contested
            .set_control(
                "world.cell.river_landing",
                Some("faction.pirates".into()),
                &geography,
            )
            .expect("the river landing is a real cell");
        assert_eq!(
            events,
            vec![WorldEvent::ControlChanged {
                cell_id: "world.cell.river_landing".into(),
                from: None,
                to: Some("faction.pirates".into()),
                day: contested.campaign_day,
            }]
        );
        assert_eq!(
            walk(&mut contested),
            authored_risk + CONTESTED_RISK_MODIFIER
        );

        assert_eq!(
            geography
                .route(SAFE_ROAD)
                .expect("the safe road is still there")
                .risk_level,
            authored_risk,
            "no route record may be edited to make a road contested"
        );
    }

    #[test]
    fn setting_control_is_idempotent_and_releasable() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .set_control(
                "world.cell.river_landing",
                Some("faction.pirates".into()),
                &geography,
            )
            .expect("a real cell");
        assert!(
            state
                .set_control(
                    "world.cell.river_landing",
                    Some("faction.pirates".into()),
                    &geography,
                )
                .expect("a real cell")
                .is_empty(),
            "handing a cell to the faction already holding it is not news"
        );
        let events = state
            .set_control("world.cell.river_landing", None, &geography)
            .expect("a real cell");
        assert_eq!(
            events,
            vec![WorldEvent::ControlChanged {
                cell_id: "world.cell.river_landing".into(),
                from: Some("faction.pirates".into()),
                to: None,
                day: state.campaign_day,
            }]
        );
        assert!(
            state.ownership.is_empty(),
            "a released cell leaves no key behind"
        );
    }

    #[test]
    fn setting_control_rejects_an_unknown_cell_before_mutating() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let error = state
            .set_control(
                "world.cell.nowhere",
                Some("faction.pirates".into()),
                &geography,
            )
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::UnknownCell {
                cell_id: "world.cell.nowhere".into()
            }
        );
        assert!(state.ownership.is_empty(), "a rejected set wrote nothing");
    }

    #[test]
    fn ownership_survives_a_round_trip_and_older_saves_load_without_it() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .set_control(
                "world.cell.river_landing",
                Some("faction.pirates".into()),
                &geography,
            )
            .expect("a real cell");
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(restored.ownership, state.ownership);

        let without = state.to_json().replace(
            ",\"ownership\":{\"world.cell.river_landing\":\"faction.pirates\"}",
            "",
        );
        let older = ExpeditionState::from_json(&without).expect("a save without ownership loads");
        assert!(older.ownership.is_empty());
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
        // Five is the party (Captain Michael and four women); six is one too many.
        // The names past Betty and Ayla are test-only placeholders -- the other
        // two women are not yet decided (Ship Plan O2 / A12).
        let six = vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
            "character.heroine.ayla".into(),
            "character.test.third".into(),
            "character.test.fourth".into(),
            "character.test.fifth".into(),
        ];
        assert_eq!(
            ExpeditionState::new(1, six, "world.cell.black_beach").unwrap_err(),
            ExpeditionError::InvalidPartySize { found: 6 }
        );
    }

    #[test]
    fn inspect_records_the_current_locations_observations_as_discoveries() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        let observed = state.inspect(&geography);
        assert_eq!(
            observed,
            vec![
                "observation.black_beach.wreck".to_owned(),
                "observation.black_beach.boiler".to_owned()
            ]
        );
        assert!(state.discoveries.contains("observation.black_beach.wreck"));
        assert!(state.discoveries.contains("observation.black_beach.boiler"));
    }

    #[test]
    fn safe_road_and_jungle_edge_apply_distinct_time_and_supply_consequences() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();

        let mut via_safe_road = fixture();
        via_safe_road.supplies.rations = 10;
        via_safe_road
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");
        via_safe_road
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("legal route");
        let safe_outcome = via_safe_road
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .expect("legal route");

        let mut via_jungle_edge = fixture();
        via_jungle_edge.supplies.rations = 10;
        via_jungle_edge
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");
        via_jungle_edge
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("legal route");
        let jungle_outcome = via_jungle_edge
            .travel(
                "world.portal.river_landing_to_reception_terrace_jungle_edge",
                &geography,
            )
            .expect("legal route");

        assert_eq!(
            via_safe_road.active_location_id,
            "world.cell.reception_terrace"
        );
        assert_eq!(
            via_jungle_edge.active_location_id,
            "world.cell.reception_terrace"
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
        // Three legs each now: the climb off the beach to the estate, the
        // estate's river gate, and the river fork itself.
        assert_eq!(via_safe_road.route_history.len(), 3);
        assert_eq!(via_jungle_edge.route_history.len(), 3);
    }

    #[test]
    fn reception_terrace_is_encounter_eligible_and_black_beach_is_not() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        assert!(
            !geography
                .location("world.cell.black_beach")
                .unwrap()
                .encounter_eligible
        );
        assert!(
            geography
                .location("world.cell.reception_terrace")
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
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .expect_err("Black Beach cannot use a river-landing route directly");
        assert_eq!(
            error,
            ExpeditionError::IllegalRoute {
                route_id: "world.portal.river_landing_to_reception_terrace_safe_road".into()
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

    fn at_tomb_reception(geography: &Geography) -> ExpeditionState {
        let mut state = fixture();
        // Enough rations to reach the tomb: these tests are about D3's truth-space
        // gate, so the party is provisioned rather than tripping A3's supply gate
        // on the way in.
        state.supplies.rations = 10;
        for route_id in [
            "world.portal.black_beach_to_damaged_estate",
            "world.portal.damaged_estate_to_river_landing",
            "world.portal.river_landing_to_reception_terrace_safe_road",
            "world.portal.reception_terrace_to_processional_ramp",
            "world.portal.processional_ramp_to_tomb_threshold",
            "world.portal.tomb_threshold_to_tomb_reception",
        ] {
            state.travel(route_id, geography).expect("legal route");
        }
        assert_eq!(state.active_location_id, "world.cell.tomb_reception");
        state
    }

    #[test]
    fn the_archive_core_route_is_rejected_without_the_true_name_discovery() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = at_tomb_reception(&geography);
        let before = state.clone();

        let error = state
            .travel(
                "world.portal.tomb_reception_to_tomb_archive_core",
                &geography,
            )
            .unwrap_err();

        assert_eq!(
            error,
            ExpeditionError::MissingDiscovery {
                route_id: "world.portal.tomb_reception_to_tomb_archive_core".into(),
                discovery_id: "observation.tomb_reception.true_name".into(),
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn inspecting_the_reception_space_unlocks_the_archive_core_route() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = at_tomb_reception(&geography);

        let observed = state.inspect(&geography);
        assert!(observed.contains(&"observation.tomb_reception.true_name".to_owned()));

        state
            .travel(
                "world.portal.tomb_reception_to_tomb_archive_core",
                &geography,
            )
            .expect("the truth-space discovery unlocks the archive core");
        assert_eq!(state.active_location_id, "world.cell.tomb_archive_core");
    }

    #[test]
    fn the_service_passage_is_reachable_without_the_discovery_and_leads_back_out() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = at_tomb_reception(&geography);

        state
            .travel(
                "world.portal.tomb_reception_to_tomb_service_passage",
                &geography,
            )
            .expect("the ungated wrong turn is always legal");
        assert_eq!(state.active_location_id, "world.cell.tomb_service_passage");

        // The recoverable failure: a clear route leads back toward the
        // threshold rather than trapping the party in a dead end.
        state
            .travel(
                "world.portal.tomb_service_passage_to_tomb_threshold",
                &geography,
            )
            .expect("the service passage returns toward the threshold");
        assert_eq!(state.active_location_id, "world.cell.tomb_threshold");
    }

    #[test]
    fn the_service_passage_presents_the_same_territorial_guardian_as_the_terrace() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        let holder = state.daily_spawn_records["world.region.black_beach.terrace_precinct"]
            .instance_id
            .clone();
        state.active_location_id = "world.cell.tomb_service_passage".into();

        let encounter = state
            .begin_encounter(&geography, &habitats)
            .expect("the elven site's guardian answers the wrong turn too")
            .clone();

        assert_eq!(encounter.encounter_id, format!("encounter.{holder}"));
    }

    #[test]
    fn boundary_save_reload_preserves_legal_next_commands_after_a_route_choice() {
        let geography = crate::geography::Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.supplies.rations = 10;
        state
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");
        state
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("legal route");
        state
            .travel(
                "world.portal.river_landing_to_reception_terrace_jungle_edge",
                &geography,
            )
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
                estate_upgrade_id: None,
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
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");
        state
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("legal route");
        state
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .expect("legal route");
        assert_eq!(state.active_location_id, "world.cell.reception_terrace");

        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
            estate_upgrade_id: None,
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
        assert_eq!(state.active_location_id, "world.cell.river_landing");
        assert!(state.supplies.rations < rations_before_retreat);
        assert_eq!(
            state.route_history.last().unwrap().location_id,
            "world.cell.river_landing"
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
            estate_upgrade_id: None,
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

    const INFIRMARY: &str = "anchor.estate.infirmary";
    const WORKSHOP: &str = "anchor.estate.workshop";
    const MAP_TABLE: &str = "anchor.estate.map_table";

    /// The estate, with the party standing in it. A4 turned the estate's rooms
    /// into anchors on the one `use_anchor` mechanism, so every test below asks
    /// the graph where the room is rather than comparing a location ID.
    fn at_the_estate() -> (Geography, ExpeditionState) {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "world.cell.damaged_estate".into();
        (geography, state)
    }

    #[test]
    fn the_infirmary_spends_medicine_and_heals_without_exceeding_max() {
        let (geography, mut state) = at_the_estate();
        state.supplies.medicine = 3;
        state
            .character_states
            .insert("character.protagonist.captain".into(), wounded_captain());

        let outcome = state.use_anchor(INFIRMARY, &geography).expect("resolves");

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
    fn the_infirmary_never_heals_past_max_vitality() {
        let (geography, mut state) = at_the_estate();
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

        state.use_anchor(INFIRMARY, &geography).expect("resolves");

        assert_eq!(
            state.character_states["character.protagonist.captain"].vitality,
            60
        );
    }

    #[test]
    fn the_infirmary_away_from_the_estate_is_rejected_without_mutation() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.supplies.medicine = 3;
        let before = state.clone();

        let result = state.use_anchor(INFIRMARY, &geography);

        assert_eq!(
            result,
            Err(ExpeditionError::AnchorNotHere {
                anchor_id: INFIRMARY.into()
            })
        );
        assert_eq!(state, before);
    }

    #[test]
    fn the_infirmary_without_medicine_is_rejected_without_mutation() {
        let (geography, mut state) = at_the_estate();
        state.supplies.medicine = 0;
        let before = state.clone();

        let result = state.use_anchor(INFIRMARY, &geography);

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
    fn the_infirmary_is_a_one_time_material_fact_reflected_in_legal_state_data() {
        let (geography, mut state) = at_the_estate();
        state.supplies.medicine = 1;

        let before = state.legal_next_commands_with_geography(&geography);
        assert!(before.contains(&format!("anchor_action:{INFIRMARY}")));

        state.use_anchor(INFIRMARY, &geography).expect("resolves");

        let after = state.legal_next_commands_with_geography(&geography);
        assert!(!after.contains(&format!("anchor_action:{INFIRMARY}")));
        // And the command the game no longer offers is a command it no longer
        // accepts: an offered set and an accepted set that disagree is the bug
        // `anchor_refusal` exists to make impossible.
        assert_eq!(
            state.use_anchor(INFIRMARY, &geography),
            Err(ExpeditionError::AnchorAlreadyResolved {
                anchor_id: INFIRMARY.into(),
                fact_id: "estate.upgrade.infirmary_rested".into(),
            })
        );
    }

    #[test]
    fn the_estate_upgrade_fact_survives_save_and_reload() {
        let (geography, mut state) = at_the_estate();
        state.supplies.medicine = 1;
        state.use_anchor(INFIRMARY, &geography).expect("resolves");

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

    /// A4: an estate room is not a free action for standing in the right place.
    /// The workshop needs the wreck to have been looked at, because the parts it
    /// fits come off the Handsome Jack.
    #[test]
    fn the_workshop_is_refused_until_the_wreck_has_been_observed() {
        let (geography, mut state) = at_the_estate();
        let before = state.clone();

        assert_eq!(
            state.use_anchor(WORKSHOP, &geography),
            Err(ExpeditionError::AnchorRequiresDiscovery {
                anchor_id: WORKSHOP.into(),
                discovery_id: "observation.black_beach.wreck".into(),
            })
        );
        assert_eq!(state, before);
        assert!(
            !state
                .legal_next_commands_with_geography(&geography)
                .contains(&format!("anchor_action:{WORKSHOP}"))
        );

        state
            .discoveries
            .insert("observation.black_beach.wreck".into());
        let outcome = state.use_anchor(WORKSHOP, &geography).expect("resolves");
        assert_eq!(
            outcome.upgrades_recorded,
            vec!["estate.upgrade.workshop_field_rig".to_owned()]
        );
        assert!(
            state
                .household_progress
                .estate_upgrades
                .contains("estate.upgrade.workshop_field_rig")
        );
    }

    /// The field rig is the whole point of the workshop: it changes what the
    /// island costs to cross, on every leg, in both directions.
    #[test]
    fn the_field_rig_takes_a_ration_off_every_route_that_charges_one() {
        let geography = Geography::black_beach_vertical_slice();
        let safe_road = geography
            .route("world.portal.river_landing_to_reception_terrace_safe_road")
            .expect("the safe road exists");
        let free_climb = geography
            .route("world.portal.black_beach_to_damaged_estate")
            .expect("the estate climb exists");
        let mut state = fixture();

        assert_eq!(state.effective_supply_cost(safe_road), 2);
        assert_eq!(state.effective_supply_cost(free_climb), 0);

        state
            .household_progress
            .estate_upgrades
            .insert("estate.upgrade.workshop_field_rig".into());

        assert_eq!(state.effective_supply_cost(safe_road), 1);
        // A road that was already free does not start paying the party back.
        assert_eq!(state.effective_supply_cost(free_climb), 0);
    }

    /// And the discount is real at the command path, not only in the helper:
    /// two rations buy a road the party could not otherwise afford twice.
    #[test]
    fn travel_charges_the_effective_cost_rather_than_the_declared_one() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "world.cell.river_landing".into();
        state.supplies.rations = 1;
        state
            .household_progress
            .estate_upgrades
            .insert("estate.upgrade.workshop_field_rig".into());

        let outcome = state
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .expect("the field rig makes one ration enough for a two-ration road");
        assert_eq!(outcome.supply_cost, 1);
        assert_eq!(state.supplies.rations, 0);
    }

    /// The map table is the other half of the same bargain: it needs the elven
    /// waymark to have been read, and what it produces is the discovery the
    /// tidal cut portal is gated on.
    #[test]
    fn the_map_table_is_refused_until_the_waymark_has_been_observed_and_then_opens_the_tidal_cut() {
        let (geography, mut state) = at_the_estate();
        let before = state.clone();

        assert_eq!(
            state.use_anchor(MAP_TABLE, &geography),
            Err(ExpeditionError::AnchorRequiresDiscovery {
                anchor_id: MAP_TABLE.into(),
                discovery_id: "observation.river_landing.road_marker".into(),
            })
        );
        assert_eq!(state, before);

        state
            .discoveries
            .insert("observation.river_landing.road_marker".into());
        let outcome = state.use_anchor(MAP_TABLE, &geography).expect("resolves");
        assert_eq!(
            outcome.discoveries_recorded,
            vec!["discovery.map_table.tidal_cut".to_owned()]
        );

        // The gate the discovery actually opens.
        state.active_location_id = "world.cell.black_beach".into();
        state.supplies.rations = 4;
        state
            .travel(
                "world.portal.black_beach_to_reception_terrace_tidal_cut",
                &geography,
            )
            .expect("the map table's route is walkable once it has been read");
        assert_eq!(state.active_location_id, "world.cell.reception_terrace");
    }

    #[test]
    fn the_tidal_cut_is_illegal_before_the_map_table_has_been_read() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.supplies.rations = 4;
        let before = state.clone();

        assert_eq!(
            state.travel(
                "world.portal.black_beach_to_reception_terrace_tidal_cut",
                &geography,
            ),
            Err(ExpeditionError::MissingDiscovery {
                route_id: "world.portal.black_beach_to_reception_terrace_tidal_cut".into(),
                discovery_id: "discovery.map_table.tidal_cut".into(),
            })
        );
        assert_eq!(state, before);
    }

    /// The household room is the one estate anchor with no permanent fact to
    /// produce, so it is spent by the day rather than for good.
    #[test]
    fn the_household_room_records_the_estate_observations_once_a_day() {
        let (geography, mut state) = at_the_estate();

        let outcome = state
            .use_anchor("anchor.estate.household_room", &geography)
            .expect("resolves");
        assert_eq!(
            outcome.discoveries_recorded,
            vec![
                "observation.damaged_estate.veranda".to_owned(),
                "observation.damaged_estate.river_gate".to_owned(),
            ]
        );
        assert_eq!(
            state.use_anchor("anchor.estate.household_room", &geography),
            Err(ExpeditionError::AnchorSpentToday {
                anchor_id: "anchor.estate.household_room".into(),
                used_on_day: state.campaign_day,
            })
        );
    }

    const SALVAGE_POINT: &str = "anchor.black_beach.salvage_point";

    /// A3: the same campaign, on the same day, always salvages the same amount.
    /// The yield is not a constant either -- it moves with the day -- so this
    /// also proves the determinism is a seeded rule rather than a fixed number.
    #[test]
    fn salvaging_the_wreck_yields_the_same_supplies_for_the_same_seed_and_day() {
        let geography = Geography::black_beach_vertical_slice();

        let mut first = fixture();
        let mut second = fixture();
        let left = first
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("the wreck is on the beach");
        let right = second
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("the wreck is on the beach");
        assert_eq!(left, right);
        assert_eq!(first.supplies, second.supplies);
        assert_eq!(first, second);

        // The declared floor is four rations and two coin; the day decides the
        // fifth ration. Coin exists in the economy for the first time here.
        assert!((4..=5).contains(&left.rations_gained), "{left:?}");
        assert_eq!(left.coin_gained, 2);
        assert_eq!(first.supplies.rations, left.rations_gained);
        assert_eq!(first.supplies.coin, 2);

        // Run the same anchor forward day by day: identical replays, and not the
        // same answer every day, which is what makes it a seeded rule.
        let yields_over_a_week = |seed: u64| {
            let mut state = ExpeditionState::new(
                seed,
                vec!["character.protagonist.captain".into()],
                "world.cell.black_beach",
            )
            .expect("constructs");
            let mut yields = Vec::new();
            for day in 1..=8 {
                state.campaign_day = day;
                yields.push(
                    state
                        .use_anchor(SALVAGE_POINT, &geography)
                        .expect("a new day re-opens the wreck")
                        .rations_gained,
                );
            }
            yields
        };
        let week = yields_over_a_week(42);
        assert_eq!(
            week,
            yields_over_a_week(42),
            "the same seed replays exactly"
        );
        // Pinned, not merely self-consistent: these are the exact rations seed
        // 42 salvages on campaign days one to eight. They replay identically and
        // they move with the day, so the mixer is doing real work rather than
        // returning a constant dressed up as a rule.
        assert_eq!(week, vec![5, 5, 4, 4, 4, 5, 5, 4]);
        assert!(
            week.iter().any(|rations| *rations != week[0]),
            "the day has to move the yield, or the seed is doing nothing: {week:?}"
        );
    }

    #[test]
    fn a_second_salvage_on_the_same_day_is_rejected_without_mutation() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("the first day's work");
        let before = state.clone();

        let error = state.use_anchor(SALVAGE_POINT, &geography).unwrap_err();

        assert_eq!(
            error,
            ExpeditionError::AnchorSpentToday {
                anchor_id: SALVAGE_POINT.into(),
                used_on_day: state.campaign_day,
            }
        );
        assert_eq!(state, before, "a refused anchor changes nothing");

        // The wreck is not exhausted forever, only for today.
        state.campaign_day += 1;
        state
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("tomorrow the tide leaves more");
    }

    #[test]
    fn using_an_anchor_that_is_not_here_is_rejected_without_mutation() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "world.cell.river_landing".into();
        let before = state.clone();

        let error = state.use_anchor(SALVAGE_POINT, &geography).unwrap_err();

        assert_eq!(
            error,
            ExpeditionError::AnchorNotHere {
                anchor_id: SALVAGE_POINT.into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn an_open_encounter_blocks_the_anchor_the_same_way_it_blocks_travel() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.pending_encounter = Some(EncounterState {
            encounter_id: "encounter.prototype.returning_names".into(),
            battle_id: "battle.prototype.returning_names".into(),
            estate_upgrade_id: None,
        });
        let before = state.clone();

        let error = state.use_anchor(SALVAGE_POINT, &geography).unwrap_err();

        assert_eq!(
            error,
            ExpeditionError::TravelBlockedByEncounter {
                encounter_id: "encounter.prototype.returning_names".into()
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn the_anchor_action_is_a_legal_command_here_until_it_is_spent() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();

        let before = state.legal_next_commands_with_geography(&geography);
        assert!(before.contains(&format!("anchor_action:{SALVAGE_POINT}")));

        state
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("the wreck is here");

        let after = state.legal_next_commands_with_geography(&geography);
        assert!(!after.contains(&format!("anchor_action:{SALVAGE_POINT}")));

        // And the answer survives a save/reload, like every other legal command.
        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(
            after,
            restored.legal_next_commands_with_geography(&geography)
        );
    }

    #[test]
    fn a_loot_cache_gives_exactly_what_it_declares() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "world.cell.tomb_service_passage".into();

        let outcome = state
            .use_anchor(
                "anchor.tomb_service_passage.disturbed_grave_goods",
                &geography,
            )
            .expect("the grave goods are here");

        assert_eq!(outcome.rations_gained, 1);
        assert_eq!(outcome.medicine_gained, 1);
        assert_eq!(outcome.coin_gained, 6);
        assert_eq!(state.supplies.medicine, 1);
        assert_eq!(state.supplies.coin, 6);
    }

    #[test]
    fn a_save_written_before_anchors_existed_still_loads() {
        let mut without_anchor_uses: serde_json::Value =
            serde_json::from_str(&fixture().to_json()).expect("fixture parses");
        without_anchor_uses
            .as_object_mut()
            .expect("save is an object")
            .remove("anchor_uses")
            .expect("fixture wrote the field");

        let restored = ExpeditionState::from_json(&without_anchor_uses.to_string())
            .expect("an older save still loads");
        assert!(restored.anchor_uses.is_empty());
    }

    /// A3's whole point: the road costs food the party has to have found first.
    #[test]
    fn travel_is_refused_without_the_rations_it_costs_and_runs_after_salvaging() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();
        assert_eq!(state.supplies.rations, 0, "a fresh campaign starts empty");

        // The free legs off the sand are still free.
        state
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("the climb costs no rations");
        state
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("the river gate costs no rations");

        // The road inland does not move an empty pack, and refusing it changes
        // nothing at all -- not the clock, not the history, not the location.
        let before = state.clone();
        let error = state
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::InsufficientSupplies {
                needed: 2,
                available: 0
            }
        );
        assert_eq!(state, before);

        // Walk back to the wreck, work it, and the same road opens.
        state
            .travel("world.portal.river_landing_to_damaged_estate", &geography)
            .expect("legal route");
        state
            .travel("world.portal.damaged_estate_to_black_beach", &geography)
            .expect("legal route");
        let salvaged = state
            .use_anchor(SALVAGE_POINT, &geography)
            .expect("the wreck is on the beach");
        assert!(salvaged.rations_gained >= 2);
        state
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");
        state
            .travel("world.portal.damaged_estate_to_river_landing", &geography)
            .expect("legal route");

        let rations_before_the_road = state.supplies.rations;
        state
            .travel(
                "world.portal.river_landing_to_reception_terrace_safe_road",
                &geography,
            )
            .expect("the salvaged rations pay for the road");
        assert_eq!(state.active_location_id, "world.cell.reception_terrace");
        assert_eq!(state.supplies.rations, rations_before_the_road - 2);
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
        state.active_location_id = "world.cell.reception_terrace".into();
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
        state.active_location_id = "world.cell.damaged_estate".into();

        assert!(state.begin_encounter(&geography, &habitats).is_none());
        assert!(state.pending_encounter.is_none());
    }

    #[test]
    fn no_encounter_before_midnight_has_materialized_an_individual() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state.active_location_id = "world.cell.reception_terrace".into();

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
    fn victory_over_the_terrace_holder_pays_its_habitat_drop_table() {
        let (geography, habitats, mut state) = at_the_held_terrace();
        let holder = state.daily_spawn_records["world.region.black_beach.terrace_precinct"].clone();
        let supplies_before = state.supplies.clone();
        state
            .begin_encounter(&geography, &habitats)
            .expect("the terrace is held");

        let resolution = state
            .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
            .expect("resolves");

        let table = resolution
            .loot
            .expect("beating the holder pays its habitat's declared drop table");
        assert_eq!(table.id, "loot.razorbeak.crested.prototype");
        assert_eq!(
            state.supplies.rations,
            supplies_before.rations + table.rations
        );
        assert_eq!(
            state.supplies.medicine,
            supplies_before.medicine + table.medicine
        );
        // The table plus what this particular individual was carrying, which is
        // the first thing in the game ever to read `SpawnedMonster.loot_seed`.
        assert_eq!(
            state.supplies.coin,
            supplies_before.coin + table.coin + (holder.loot_seed % 3) as u32
        );
    }

    #[test]
    fn a_lost_or_abandoned_fight_pays_nothing() {
        for outcome in [EncounterOutcome::Defeat, EncounterOutcome::Retreat] {
            let (geography, habitats, mut state) = at_the_held_terrace();
            let supplies_before = state.supplies.clone();
            state
                .begin_encounter(&geography, &habitats)
                .expect("the terrace is held");

            let resolution = state
                .resolve_encounter(outcome, &geography, &habitats)
                .expect("resolves");

            assert_eq!(resolution.outcome, outcome);
            assert!(resolution.loot.is_none(), "{outcome:?} carries no loot");
            assert_eq!(
                state.supplies.medicine, supplies_before.medicine,
                "{outcome:?} gains nothing"
            );
            assert_eq!(
                state.supplies.coin, supplies_before.coin,
                "{outcome:?} gains nothing"
            );
            // A retreat still pays its own way home, so rations are the one
            // supply a lost fight can move -- downward.
            assert!(state.supplies.rations <= supplies_before.rations);
        }
    }

    #[test]
    fn a_beaten_hunter_carries_no_habitat_loot() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state.hunters.push(Hunter {
            id: "hunter.test.tracker".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "world.cell.black_beach".into(),
            spawned_on_day: 1,
            level: 5,
            defeated_on_day: None,
        });
        state
            .begin_encounter(&geography, &habitats)
            .expect("the hunter is here");

        let resolution = state
            .resolve_encounter(EncounterOutcome::Victory, &geography, &habitats)
            .expect("resolves");

        assert!(resolution.loot.is_none());
        assert_eq!(state.supplies.coin, 0);
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
            .step_distance(
                &hunter_location_before,
                &state.active_location_id,
                &state.discoveries,
            )
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
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
            .expect("legal route");

        let distance_after = geography
            .step_distance(
                &state.hunters[0].current_location_id,
                &state.active_location_id,
                &state.discoveries,
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
        let mut state = fixture();
        // Place a hunter as far away as the graph allows, by hand, so this test
        // does not depend on the spawn roll actually landing.
        state.hunters.push(Hunter {
            id: "hunter.test.pursuit".into(),
            definition_id: HunterKind::HumanTracker.definition_id().to_owned(),
            kind: HunterKind::HumanTracker,
            current_location_id: "world.cell.processional_ramp".into(),
            spawned_on_day: 1,
            level: 5,
            defeated_on_day: None,
        });
        let starting_distance = geography
            .step_distance(
                &state.hunters[0].current_location_id,
                &state.active_location_id,
                &state.discoveries,
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
            .travel("world.portal.black_beach_to_damaged_estate", &geography)
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
            current_location_id: "world.cell.black_beach".into(),
            spawned_on_day: 1,
            level: 5,
            defeated_on_day: None,
        });
        state.hunters.push(Hunter {
            id: "hunter.test.revenant".into(),
            definition_id: HunterKind::Revenant.definition_id().to_owned(),
            kind: HunterKind::Revenant,
            current_location_id: "world.cell.black_beach".into(),
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

    /// The tidal cut hangs directly off the beach but is gated on the map
    /// table's discovery. Until the party holds it, the route is not offered;
    /// once it does, it is. This is what the bridge projects to Godot, so a
    /// regression here is a locked door drawn as a button.
    #[test]
    fn a_gated_route_is_not_a_legal_command_until_its_discovery_is_held() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            3,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("constructs");
        let tidal_cut = "travel:world.portal.black_beach_to_reception_terrace_tidal_cut";
        let before = state
            .legal_route_commands(&geography)
            .expect("no encounter pending");
        assert!(
            !before.iter().any(|command| command == tidal_cut),
            "the tidal cut must not be offered before the map table opens it: {before:?}"
        );
        assert_eq!(
            before.len(),
            1,
            "the beach has exactly one ungated departure"
        );
        state
            .discoveries
            .insert(MAP_TABLE_DISCOVERY_TIDAL_CUT.to_owned());
        let after = state
            .legal_route_commands(&geography)
            .expect("no encounter pending");
        assert!(after.iter().any(|command| command == tidal_cut));
        assert_eq!(after.len(), 2);
    }

    /// B4 draws one button per observation; each must record only its own.
    #[test]
    fn inspecting_one_observation_records_that_one_and_nothing_else() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            3,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("constructs");
        let here = geography
            .location("world.cell.black_beach")
            .expect("the beach exists")
            .observation_ids
            .clone();
        assert!(
            here.len() >= 2,
            "the beach declares more than one observation"
        );
        state
            .inspect_observation(&here[0], &geography)
            .expect("declared here");
        assert!(state.discoveries.contains(&here[0]));
        assert!(
            !state.discoveries.contains(&here[1]),
            "reading one sign must not record the other"
        );
    }

    #[test]
    fn inspecting_an_observation_from_elsewhere_is_refused_without_mutation() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = ExpeditionState::new(
            3,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("constructs");
        let before = state.to_json();
        let error = state
            .inspect_observation("observation.river_landing.road_marker", &geography)
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::ObservationNotHere {
                observation_id: "observation.river_landing.road_marker".into(),
                location_id: "world.cell.black_beach".into(),
            }
        );
        assert_eq!(state.to_json(), before, "a refused inspect changes nothing");
    }
    // ---------------------------------------------------------------
    // S8: the dual clocks, weather and corruption.
    // ---------------------------------------------------------------

    /// The required test, in the contract's own wording
    /// (`docs/GAME_BUILD_PLAN.md`, "Dual clocks"; S8's card): advance world
    /// time by thirty days with no Cthulhu-relevant event and assert heat is
    /// unchanged; then trigger one heat event on a single day and assert heat
    /// moved while the day count did not jump.
    ///
    /// This is the sabotage detector for the whole lane. Anything that makes a
    /// day imply pressure -- a line in `resolve_midnight_in`, a decay-and-grow
    /// sweep, a "patience ticks up" convenience in a later lane -- fails here.
    #[test]
    fn thirty_midnights_do_not_move_hidden_pressure() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();

        let heat_before = state.cthulhu_heat;
        for _ in 0..30 {
            state
                .resolve_midnight_in(&geography, &habitats)
                .expect("a midnight resolves");
        }
        assert_eq!(
            state.campaign_day, 31,
            "thirty midnights must advance world time"
        );
        assert_eq!(
            state.cthulhu_heat, heat_before,
            "thirty days with no Cthulhu-relevant event must not move hidden pressure"
        );

        // Then one heat event on a single day.
        let day_before = state.campaign_day;
        let moved = state.record_heat_event(HeatEvent::RitualCompleted);
        assert!(
            moved > heat_before,
            "a ritual must move hidden pressure: {heat_before} then {moved}"
        );
        assert_eq!(
            state.campaign_day, day_before,
            "a heat event must not advance the day count"
        );
    }

    #[test]
    fn every_heat_input_moves_pressure_and_saturates_at_the_ceiling() {
        let mut state = fixture();
        let mut last = 0;
        for event in [
            HeatEvent::RitualCompleted,
            HeatEvent::CorruptedCellHeld {
                cell_id: "world.cell.river_landing".into(),
            },
            HeatEvent::PartyInterference,
        ] {
            let now = state.record_heat_event(event);
            assert!(now > last, "each named input must move pressure");
            last = now;
        }
        for _ in 0..1_000 {
            state.record_heat_event(HeatEvent::RitualCompleted);
        }
        assert_eq!(
            state.cthulhu_heat,
            crate::strategy::clocks::MAX_HEAT,
            "pressure saturates rather than wrapping"
        );
    }

    #[test]
    fn two_midnights_from_the_same_save_draw_the_same_weather() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let state = fixture();

        let mut first = ExpeditionState::from_json(&state.to_json()).expect("round trip");
        let mut again = ExpeditionState::from_json(&state.to_json()).expect("round trip");
        first
            .resolve_midnight_in(&geography, &habitats)
            .expect("a midnight resolves");
        again
            .resolve_midnight_in(&geography, &habitats)
            .expect("a midnight resolves");

        assert!(!first.weather.is_empty(), "a midnight draws the day's sky");
        assert_eq!(
            first.weather, again.weather,
            "weather is a deterministic draw, not a roll"
        );
        assert_eq!(
            first.to_json(),
            again.to_json(),
            "and so is everything else"
        );
    }

    #[test]
    fn weather_is_drawn_for_every_region_the_board_has() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("a midnight resolves");

        let regions: BTreeSet<&str> = geography
            .locations
            .values()
            .map(|location| location.region_id.as_str())
            .collect();
        let drawn: BTreeSet<&str> = state.weather.keys().map(String::as_str).collect();
        assert_eq!(drawn, regions, "every region on the board, and no other");
        for weather in state.weather.values() {
            assert_eq!(
                weather.drawn_on_day, state.campaign_day,
                "the sky is drawn for the day that is beginning"
            );
        }
    }

    #[test]
    fn corruption_accumulates_and_saturates_and_refuses_an_unknown_cell() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = fixture();

        assert_eq!(
            state
                .corrupt_cell("world.cell.river_landing", 10, &geography)
                .expect("a real cell"),
            10
        );
        assert_eq!(
            state
                .corrupt_cell("world.cell.river_landing", 20, &geography)
                .expect("a real cell"),
            30,
            "corruption accumulates; there is no decay (reversibility is Open)"
        );
        for _ in 0..20 {
            state
                .corrupt_cell("world.cell.river_landing", 200, &geography)
                .expect("a real cell");
        }
        assert_eq!(state.corruption["world.cell.river_landing"], 255);

        let error = state
            .corrupt_cell("world.cell.nowhere", 5, &geography)
            .unwrap_err();
        assert_eq!(
            error,
            ExpeditionError::UnknownCell {
                cell_id: "world.cell.nowhere".into()
            }
        );
        assert!(
            !state.corruption.contains_key("world.cell.nowhere"),
            "a rejected corruption wrote nothing"
        );
    }

    #[test]
    fn the_confrontation_opens_by_discovery_by_day_one_hundred_or_by_full_pressure() {
        let mut state = fixture();
        assert_eq!(state.confrontation_is_triggered(), None);

        let mut by_day = state.clone();
        by_day.campaign_day = CONFRONTATION_DAY;
        assert_eq!(
            by_day.confrontation_is_triggered(),
            Some(ConfrontationTrigger::DayHundred)
        );

        let mut by_pressure = state.clone();
        for _ in 0..1_000 {
            by_pressure.record_heat_event(HeatEvent::RitualCompleted);
        }
        assert_eq!(
            by_pressure.confrontation_is_triggered(),
            Some(ConfrontationTrigger::MaximumHeat)
        );

        state
            .discoveries
            .insert(CONFRONTATION_DISCOVERY_ID.to_owned());
        assert_eq!(
            state.confrontation_is_triggered(),
            Some(ConfrontationTrigger::Discovery),
            "the route the party chose outranks the two that happen to them"
        );
    }

    /// S8's Done-when: an assistance event never fires for a Desperate
    /// Cthulhu.
    #[test]
    fn assistance_is_weighted_only_for_a_winning_cthulhu() {
        use crate::strategy::faction::{FactionState, StrategicState};

        let mut state = fixture();
        assert_eq!(
            state.cthulhu_assistance_weight(),
            0,
            "a save with no such faction assists nobody"
        );

        for losing in [
            StrategicState::Desperate,
            StrategicState::Recovering,
            StrategicState::Contesting,
        ] {
            state.factions.insert(
                CTHULHU_FACTION_ID.to_owned(),
                FactionState {
                    strategic_state: losing,
                    ..FactionState::default()
                },
            );
            assert_eq!(
                state.cthulhu_assistance_weight(),
                0,
                "assistance must never rescue a losing Cthulhu: {losing:?}"
            );
        }
        for winning in [StrategicState::Advantaged, StrategicState::Closing] {
            state.factions.insert(
                CTHULHU_FACTION_ID.to_owned(),
                FactionState {
                    strategic_state: winning,
                    ..FactionState::default()
                },
            );
            assert!(state.cthulhu_assistance_weight() > 0, "{winning:?}");
        }
    }

    #[test]
    fn the_second_clock_survives_a_round_trip_and_older_saves_load_without_it() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = crate::habitat::Habitats::black_beach_vertical_slice();
        let mut state = fixture();
        state.record_heat_event(HeatEvent::PartyInterference);
        state
            .corrupt_cell("world.cell.river_landing", 90, &geography)
            .expect("a real cell");
        state
            .resolve_midnight_in(&geography, &habitats)
            .expect("a midnight resolves");

        let restored = ExpeditionState::from_json(&state.to_json()).expect("round trip parses");
        assert_eq!(restored.cthulhu_heat, state.cthulhu_heat);
        assert_eq!(restored.corruption, state.corruption);
        assert_eq!(restored.weather, state.weather);

        let mut older: serde_json::Value =
            serde_json::from_str(&state.to_json()).expect("save is JSON");
        let object = older.as_object_mut().expect("save is an object");
        for field in ["cthulhu_heat", "weather", "corruption"] {
            object.remove(field).expect("fixture wrote the field");
        }
        let loaded = ExpeditionState::from_json(&older.to_string()).expect("an older save loads");
        assert_eq!(loaded.cthulhu_heat, 0);
        assert!(loaded.weather.is_empty());
        assert!(loaded.corruption.is_empty());
    }
}
