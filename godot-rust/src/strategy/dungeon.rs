//! S9: [`DungeonContext`] and the generation signature -- the same context
//! always yields the same rooms, and a different owner or tier never collides.
//! `docs/SHIP_PLAN.md` section 7, card S9.
//!
//! Brief section 12 is the whole argument for this file: "Dungeon generation
//! depends on more than a numerical level", and "a level-five dungeon
//! controlled by one faction must not be the same dungeon with a palette swap
//! as a level-two dungeon controlled by another faction." So the unit of
//! generation is not a level. It is a *context* -- brief section 19's
//! `DungeonContext` block, field for field -- and a signature folded over the
//! whole of it.
//!
//! ## What this lane does and does not build
//!
//! It builds the context, the fold, and the seed the fold hands on. It builds
//! **no rooms**. C5 authors the tomb's twelve spaces and A7 authors its site
//! rules; both are *selected by* [`room_seed`] from a signature computed here,
//! which is why [`room_seed`] is public and documented as a contract while
//! nothing in this crate yet calls it. That is a seam with two named owners,
//! not a noodle to nowhere.
//!
//! ## Why the fold is a serialization
//!
//! [`dungeon_signature`] is FNV-1a over `serde_json`'s bytes for the whole
//! struct -- the shape [`StrategicJournal`](crate::strategy::journal::StrategicJournal)
//! already uses for its history digest, and for the same reason: `std`'s
//! `DefaultHasher` is explicitly not stable across releases, so a determinism
//! witness built on it is not a witness. Every map inside is a `BTreeMap` and
//! every struct serializes in declaration order, so the bytes are canonical.
//!
//! Folding the *serialization* rather than a hand-written list of fields is
//! the point of doing it this way: a field added to [`DungeonContext`] joins
//! the fold by existing. It cannot be forgotten, and
//! `every_single_field_changes_the_signature` fails by name the moment one is
//! excluded from the bytes.
//!
//! ## What is provisional here
//!
//! Every band threshold and every budget coefficient below is **provisional,
//! `needs decision`**. `docs/GAME_BUILD_PLAN.md` forbids implementation
//! inventing tuning, and nothing accepted fixes a scale for corruption bands,
//! heat bands, world epochs, or encounter and hazard budgets. What S9 ships is
//! the *shape*: named bands rather than raw numbers in the signature, so that
//! one point of corruption does not silently regenerate a dungeon, and
//! retuning is editing constants rather than behaviour.
//!
//! ## What is blocked
//!
//! * **Refilling a reward budget** -- `blocked: needs decision`.
//!   [`BuildingInstance::stored_value`] only ever goes down here. Brief
//!   section 12 says rewards "must correspond to actual stored value,
//!   production, reinforcements, abandonment, and recovery"; production and
//!   reinforcement are S13's and S3's, so this lane implements the draw-down
//!   and no refill at all. A dungeon farmed dry stays dry until one of those
//!   lanes decides what fills it.
//! * **Per-instance upgrade history** -- `needs decision`.
//!   [`DungeonContext::upgrade_signature`] is brief section 12's "upgrade
//!   history", and no such history is stored: [`BuildingInstance`] carries a
//!   current tier and nothing about the road to it. What is folded instead is
//!   the tier ladder the building has actually climbed, read from its
//!   definition -- the honest reading of the state that exists. When S13 gives
//!   an instance a real history, this field reads it and nothing else moves.

use serde::{Deserialize, Serialize};

use crate::expedition::ExpeditionState;
use crate::geography::Geography;
use crate::strategy::building::{BuildingDefinitions, BuildingError, BuildingInstance};
use crate::strategy::clocks::{CONFRONTATION_DAY, WeatherCondition};
use crate::world::mix_seed;

/// The FNV-1a 64-bit prime, as
/// [`StrategicJournal`](crate::strategy::journal::StrategicJournal) and
/// [`mix_seed`] both use it.
const FNV_PRIME: u64 = 0x100_0000_01B3;

/// The FNV-1a 64-bit offset basis. The journal's digest starts from its own
/// running value and so never needed this; a signature is folded from nothing
/// each time and does.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The `dungeon_relationship` (S3's authored field) of a building that is not
/// a dungeon at all.
///
/// A building presents a dungeon when that field names a grammar. Empty means
/// "not authored yet" and this word means "authored, and deliberately not one"
/// -- both answer [`presents_a_dungeon`] with `false`, and the difference is
/// C10's to record, not this lane's to guess.
pub const NOT_A_DUNGEON: &str = "none";

/// The key [`room_seed`] mixes under. Named once so the room seeds of every
/// dungeon in every campaign agree on what they are derived from.
const ROOM_SEED_KEY: &str = "dungeon.room";

// -- Bands ------------------------------------------------------------------
//
// Every threshold in this block is provisional, `needs decision`.

/// Lowest cell corruption that counts as [`CorruptionBand::Touched`].
/// **Provisional, `needs decision`.**
pub const CORRUPTION_BAND_TOUCHED: u8 = 1;
/// Lowest cell corruption that counts as [`CorruptionBand::Spreading`].
/// **Provisional, `needs decision`.**
pub const CORRUPTION_BAND_SPREADING: u8 = 64;
/// Lowest cell corruption that counts as [`CorruptionBand::Consumed`].
/// **Provisional, `needs decision`.**
pub const CORRUPTION_BAND_CONSUMED: u8 = 192;

/// Lowest hidden pressure that counts as [`HeatBand::Stirring`].
/// **Provisional, `needs decision`.**
pub const HEAT_BAND_STIRRING: u32 = 100;
/// Lowest hidden pressure that counts as [`HeatBand::Rising`].
/// **Provisional, `needs decision`.**
pub const HEAT_BAND_RISING: u32 = 400;
/// Lowest hidden pressure that counts as [`HeatBand::Imminent`].
/// **Provisional, `needs decision`.**
pub const HEAT_BAND_IMMINENT: u32 = 800;

/// First campaign day of [`WorldEpoch::Establishment`]. **Provisional,
/// `needs decision`.**
pub const EPOCH_ESTABLISHMENT_DAY: u32 = 25;
/// First campaign day of [`WorldEpoch::Pressure`]. **Provisional,
/// `needs decision`.**
pub const EPOCH_PRESSURE_DAY: u32 = 60;
/// First campaign day of [`WorldEpoch::Endgame`].
///
/// The one threshold here that is *not* invented: it is
/// [`CONFRONTATION_DAY`], so the epoch a dungeon generates in turns over on
/// exactly the day brief section 13 says the confrontation can begin, rather
/// than on a second number that would drift away from it.
pub const EPOCH_ENDGAME_DAY: u32 = CONFRONTATION_DAY;

/// Encounters a dungeon is entitled to per building tier. **Provisional,
/// `needs decision`.**
pub const ENCOUNTER_BUDGET_PER_TIER: u32 = 3;
/// Hazards a dungeon is entitled to per building tier. **Provisional,
/// `needs decision`.**
pub const HAZARD_BUDGET_PER_TIER: u32 = 2;
/// Extra hazards per step up the corruption ladder. **Provisional,
/// `needs decision`.**
pub const HAZARD_BUDGET_PER_CORRUPTION_BAND: u32 = 1;

/// How far corruption has gone at the dungeon's cell, as a band rather than a
/// number.
///
/// A band and not the raw `u8` on purpose. Corruption accumulates a point at a
/// time (S8), and a signature folded over the raw value would regenerate the
/// whole dungeon every time one point landed -- which is the "arbitrary global
/// scaling disconnected from the board" brief section 11 rejects, arriving
/// from the other direction. Four bands mean the dungeon changes when the
/// *situation* changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorruptionBand {
    /// Clean ground.
    #[default]
    Untouched,
    Touched,
    Spreading,
    /// Nothing of the original place is left.
    Consumed,
}

impl CorruptionBand {
    /// The band a cell's corruption falls in.
    pub fn of(corruption: u8) -> Self {
        if corruption >= CORRUPTION_BAND_CONSUMED {
            CorruptionBand::Consumed
        } else if corruption >= CORRUPTION_BAND_SPREADING {
            CorruptionBand::Spreading
        } else if corruption >= CORRUPTION_BAND_TOUCHED {
            CorruptionBand::Touched
        } else {
            CorruptionBand::Untouched
        }
    }

    /// How many steps up the ladder this band is, which is what the hazard
    /// budget reads. Zero for [`CorruptionBand::Untouched`].
    pub fn steps(self) -> u32 {
        match self {
            CorruptionBand::Untouched => 0,
            CorruptionBand::Touched => 1,
            CorruptionBand::Spreading => 2,
            CorruptionBand::Consumed => 3,
        }
    }
}

/// Hidden pressure as a band.
///
/// Brief section 13 is explicit that pressure is *hidden*, so this is not a
/// projection and nothing in the bridge shows it. It is in the context because
/// brief section 12 wants generation to depend on the state of the world, and
/// a dungeon raided while the island is about to end is not the same dungeon.
/// Banding it also keeps the hidden number hidden: a player who could see a
/// dungeon change would learn at most which of four bands the world is in, and
/// only at the thresholds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatBand {
    #[default]
    Dormant,
    Stirring,
    Rising,
    Imminent,
}

impl HeatBand {
    /// The band a campaign's `cthulhu_heat` falls in.
    pub fn of(heat: u32) -> Self {
        if heat >= HEAT_BAND_IMMINENT {
            HeatBand::Imminent
        } else if heat >= HEAT_BAND_RISING {
            HeatBand::Rising
        } else if heat >= HEAT_BAND_STIRRING {
            HeatBand::Stirring
        } else {
            HeatBand::Dormant
        }
    }
}

/// Brief section 19's `world_epoch`: the campaign day as a band.
///
/// The same argument as [`CorruptionBand`]. World time advances every
/// midnight, and a dungeon that regenerated nightly would be noise rather than
/// history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldEpoch {
    /// The wreck and the first weeks.
    #[default]
    Landfall,
    Establishment,
    Pressure,
    /// [`EPOCH_ENDGAME_DAY`] onward: the confrontation can begin.
    Endgame,
}

impl WorldEpoch {
    /// The epoch a campaign day falls in.
    pub fn of(campaign_day: u32) -> Self {
        if campaign_day >= EPOCH_ENDGAME_DAY {
            WorldEpoch::Endgame
        } else if campaign_day >= EPOCH_PRESSURE_DAY {
            WorldEpoch::Pressure
        } else if campaign_day >= EPOCH_ESTABLISHMENT_DAY {
            WorldEpoch::Establishment
        } else {
            WorldEpoch::Landfall
        }
    }
}

/// Whether a building definition presents a dungeon at all.
///
/// S3 stores `dungeon_relationship` as authored text and says S9 owns the
/// meaning; this is that meaning, in one place. A named grammar is a dungeon;
/// nothing authored, or [`NOT_A_DUNGEON`], is not.
pub fn presents_a_dungeon(dungeon_relationship: &str) -> bool {
    !dungeon_relationship.is_empty() && dungeon_relationship != NOT_A_DUNGEON
}

/// What went wrong reading or raiding a dungeon.
///
/// [`BuildingError`] already names every failure this file shares with S3 --
/// an instance the campaign does not carry, a definition the registry does
/// not, a cell the graph does not know -- so those are carried through rather
/// than re-spelled. Only the one condition S3 has no word for gets a variant.
#[derive(Clone, Debug, PartialEq)]
pub enum DungeonError {
    /// A failure S3 already names. Wrapped, not copied.
    Building(BuildingError),
    /// The building exists and is not a dungeon: its definition's
    /// `dungeon_relationship` does not name a grammar.
    NotADungeon { id: String },
}

impl From<BuildingError> for DungeonError {
    fn from(error: BuildingError) -> Self {
        DungeonError::Building(error)
    }
}

/// Brief section 19's `DungeonContext` block, field for field and in order,
/// with one field appended that S9's card requires and section 19 has no slot
/// for.
///
/// **Every field participates in [`dungeon_signature`].** That is not a
/// convention held by hand: the fold is over this struct's serialization, so a
/// field joins by existing, and `every_single_field_changes_the_signature`
/// walks a list of single-field mutations that a new field must be added to.
///
/// ## Reading order: read, then sign, then seal
///
/// Section 19's last four fields (`generated_seed`, `generated_room_ids`, and
/// with them `revision`) are the *record of a generation*, not inputs to one.
/// [`DungeonContext::of`] therefore returns the **pre-generation** context:
/// generated seed zero, no rooms, revision zero. A generator reads that
/// context, folds it with [`dungeon_signature`], draws its rooms through
/// [`room_seed`], and only then fills the outputs in. Folding them too is
/// deliberate and correct: a context that has already generated is a different
/// context from one that has not, so a deliberate regeneration at a bumped
/// `revision` is a different dungeon rather than a silent overwrite of the
/// same one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonContext {
    /// The campaign's `rng_seed`. Brief section 12's "campaign seed".
    pub campaign_seed: u64,
    /// The cell the dungeon stands on. Brief section 12's "node".
    pub node_id: String,
    /// The `building_instance.*` ID whose building is the way in. Brief
    /// section 12's "entrance building".
    pub entrance_building_id: String,
    /// `faction.<concept_key>`, from [`Geography::held_by`] -- the S2 answer,
    /// the save's override where there is one and the authored owner
    /// otherwise. `None` is unheld ground, which is a real answer and folds as
    /// one: an abandoned dungeon is not the same dungeon as a held one.
    pub owning_faction_id: Option<String>,
    /// The `building.*` definition ID. Brief section 12's "building
    /// archetype".
    pub building_archetype_id: String,
    /// The instance's current tier. Brief section 12: tier "increases the
    /// value and danger of attacking the location".
    pub building_tier: u32,
    /// Brief section 12's "upgrade history", folded. See the module docs:
    /// no per-instance history is stored yet, so this is a fold of the tier
    /// ladder the building has actually climbed, read from its definition.
    pub upgrade_signature: u64,
    /// Corruption at [`DungeonContext::node_id`], banded.
    pub corruption_band: CorruptionBand,
    /// The sky over the dungeon's region.
    ///
    /// The *condition* only, deliberately not S8's whole
    /// [`WeatherState`](crate::strategy::clocks::WeatherState): that struct
    /// carries the day it was drawn on, and folding a day stamp would mean
    /// "the same context" could never recur, which is precisely what this
    /// lane's Done-when forbids. The campaign day is in the context already,
    /// banded, as `world_epoch`.
    pub weather_state: WeatherCondition,
    /// The campaign day, banded.
    pub world_epoch: WorldEpoch,
    /// Filled by the generator from [`dungeon_signature`]; zero until then.
    /// See the struct docs on reading order.
    pub generated_seed: u64,
    /// The grammar to generate against: the building definition's authored
    /// `dungeon_relationship` (S3). Brief section 19 gives each faction a
    /// `dungeon_grammar` too; when S1's definitions carry one, it composes
    /// here rather than replacing this.
    pub grammar_id: String,
    /// How much fighting the dungeon is entitled to. Provisional; see
    /// [`ENCOUNTER_BUDGET_PER_TIER`].
    pub encounter_budget: u32,
    /// How much hazard the dungeon is entitled to. Provisional; see
    /// [`HAZARD_BUDGET_PER_TIER`].
    pub hazard_budget: u32,
    /// What is actually in there to take: the building's
    /// [`BuildingInstance::stored_value`], not a number derived from tier.
    /// Brief section 12: "Rewards must correspond to actual stored value."
    /// This is why repeated farming cannot pay twice --
    /// [`ExpeditionState::draw_dungeon_reward`] lowers the same field this
    /// reads.
    pub reward_budget: u32,
    /// The rooms a generator produced, in order. Empty until it has. C5 owns
    /// what the IDs mean.
    pub generated_room_ids: Vec<String>,
    /// Bumped by a deliberate regeneration. Zero for a context read off the
    /// board.
    pub revision: u32,
    /// Hidden pressure, banded.
    ///
    /// **Beyond brief section 19's block**, and appended after it rather than
    /// inserted into it so that section 19's fields stay verbatim and in
    /// order. S9's card requires it: heat is one of the two clocks S8 made a
    /// dimension of its own, and a dungeon generated while the island is about
    /// to end is not the dungeon generated in week one. Section 12's list does
    /// not name it and section 19 has no slot for it, so the field says so
    /// here rather than pretending to be one of theirs.
    pub heat_band: HeatBand,
}

impl DungeonContext {
    /// The context for the dungeon `building_instance_id` is the way into, read
    /// off the board.
    ///
    /// A pure read: it mutates nothing, draws nothing, and gives the same
    /// answer twice from the same state. Refuses -- before doing anything --
    /// an instance the campaign does not carry, a definition the registry does
    /// not carry, and a building whose definition does not name a dungeon
    /// grammar.
    ///
    /// Returns the **pre-generation** context; see the struct docs.
    pub fn of(
        state: &ExpeditionState,
        geography: &Geography,
        building_instance_id: &str,
        definitions: &BuildingDefinitions,
    ) -> Result<Self, DungeonError> {
        let instance = state.buildings.get(building_instance_id).ok_or_else(|| {
            DungeonError::Building(BuildingError::UnknownBuilding {
                id: building_instance_id.to_owned(),
            })
        })?;
        let definition = definitions.require(&instance.def_id)?;
        if !presents_a_dungeon(&definition.dungeon_relationship) {
            return Err(DungeonError::NotADungeon {
                id: building_instance_id.to_owned(),
            });
        }

        let corruption_band = CorruptionBand::of(
            state
                .corruption
                .get(&instance.cell_id)
                .copied()
                .unwrap_or(0),
        );

        // The sky over the region the cell sits in. A cell the graph does not
        // know, or a region no midnight has drawn for yet, is ordinary weather
        // rather than a failure: a dungeon is not less real because nobody has
        // looked at the sky.
        let weather_state = geography
            .location(&instance.cell_id)
            .and_then(|location| state.weather.get(&location.region_id))
            .map(|weather| weather.condition)
            .unwrap_or_default();

        Ok(Self {
            campaign_seed: state.rng_seed,
            node_id: instance.cell_id.clone(),
            entrance_building_id: instance.id.clone(),
            owning_faction_id: geography
                .held_by(&instance.cell_id, &state.ownership)
                .map(str::to_owned),
            building_archetype_id: instance.def_id.clone(),
            building_tier: instance.tier,
            upgrade_signature: upgrade_signature(instance, definition),
            corruption_band,
            weather_state,
            world_epoch: WorldEpoch::of(state.campaign_day),
            generated_seed: 0,
            grammar_id: definition.dungeon_relationship.clone(),
            encounter_budget: instance.tier.saturating_mul(ENCOUNTER_BUDGET_PER_TIER),
            hazard_budget: instance
                .tier
                .saturating_mul(HAZARD_BUDGET_PER_TIER)
                .saturating_add(corruption_band.steps() * HAZARD_BUDGET_PER_CORRUPTION_BAND),
            reward_budget: instance.stored_value,
            generated_room_ids: Vec::new(),
            revision: 0,
            heat_band: HeatBand::of(state.cthulhu_heat),
        })
    }

    /// This context's signature: [`dungeon_signature`] of `self`, as a method
    /// for readability at call sites. One implementation, not two.
    pub fn signature(&self) -> u64 {
        dungeon_signature(self)
    }
}

/// Brief section 12's "upgrade history", as far as the state honestly knows
/// it: a fold of the tier ladder this building has actually climbed.
///
/// Two buildings at tier three whose definitions author different tiers got
/// there by different roads and are different dungeons; two at tier one and
/// tier three of the same definition fold differently because the second
/// walked further. When S13 stores a real per-instance history, this reads it
/// and no caller changes.
fn upgrade_signature(
    instance: &BuildingInstance,
    definition: &crate::strategy::building::BuildingDefinition,
) -> u64 {
    let climbed: Vec<_> = definition
        .tier_states
        .iter()
        .filter(|tier_state| tier_state.tier <= instance.tier)
        .collect();
    fnv1a(&serde_json::to_vec(&climbed).expect("tier states always serialize"))
}

/// The one fold: FNV-1a over `bytes`.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut digest = FNV_OFFSET_BASIS;
    for byte in bytes {
        digest = (digest ^ u64::from(*byte)).wrapping_mul(FNV_PRIME);
    }
    digest
}

/// The generation signature: FNV-1a over the context's canonical
/// serialization.
///
/// Pure, total, and stable across machines and toolchain releases -- no
/// hashing crate, no `DefaultHasher`, the crate's own `serde_json` bytes and
/// the same prime the journal digest uses. Two contexts that differ in any
/// field differ here; the same context gives the same number forever, which is
/// what makes "the same context yields the same rooms" a fact about the data
/// rather than a hope about the generator.
pub fn dungeon_signature(context: &DungeonContext) -> u64 {
    fnv1a(&serde_json::to_vec(context).expect("a DungeonContext always serializes"))
}

/// **The contract C5 and A7 generate against.**
///
/// The seed for the `room_index`-th room of the dungeon whose context folded
/// to `signature`. Two guarantees, and they are the whole of what this lane
/// promises the content lanes:
///
/// 1. **The same signature yields the same sequence of room seeds, forever.**
///    So C5's authored room set, selected by these seeds, is the same set on
///    every machine and after every reload -- S9's Done-when, "same context →
///    same rooms".
/// 2. **A different signature yields a different sequence.** So a tomb held by
///    one faction selects different rooms and different A7 site rules from the
///    same authored pool than the same tomb held by another -- S9's Done-when,
///    "owner change → different site rules". A7 selects a cell's
///    `site_rule_ids` from the seed of the room they belong to, by the same
///    call; there is no second mixer for rules.
///
/// Composed from [`mix_seed`], the crate's single mixing function, exactly as
/// [`strategic_draw`](crate::strategy::tick::strategic_draw) and
/// [`weather_draw`](crate::strategy::clocks::weather_draw) are. The room index
/// travels in `mix_seed`'s day position because that position is the one that
/// takes a `u32` counter; nothing about it means a day here.
pub fn room_seed(signature: u64, room_index: u32) -> u64 {
    mix_seed(signature, room_index, ROOM_SEED_KEY, 0)
}

impl ExpeditionState {
    /// The context for the dungeon this cell presents, or `None`.
    ///
    /// A cell presents a dungeon when a building standing on it has a
    /// definition whose `dungeon_relationship` names a grammar
    /// ([`presents_a_dungeon`]). Buildings are a `BTreeMap`, so where a cell
    /// somehow carries two, the first by instance ID wins and does so on every
    /// machine.
    ///
    /// `None` covers every way there is no dungeon here -- no such cell, no
    /// buildings, none of them a dungeon -- because they are the same answer
    /// to the question asked. A caller that needs to know *why* asks
    /// [`DungeonContext::of`] about a particular building and reads the
    /// [`DungeonError`].
    pub fn dungeon_context_at(
        &self,
        cell_id: &str,
        geography: &Geography,
        definitions: &BuildingDefinitions,
    ) -> Option<DungeonContext> {
        self.buildings
            .values()
            .filter(|instance| instance.cell_id == cell_id)
            .find_map(|instance| {
                DungeonContext::of(self, geography, &instance.id, definitions).ok()
            })
    }

    /// Take up to `requested` from a dungeon's reward budget, and report what
    /// was actually there.
    ///
    /// **This is brief section 12's "repeated farming must not generate
    /// infinite high-tier loot", and it is the only mutator of
    /// [`BuildingInstance::stored_value`].** The draw is
    /// `min(requested, stored_value)`; the budget saturates at zero and never
    /// passes it. Nothing in this lane puts anything back: refill from
    /// production, reinforcement, abandonment and recovery is S13's and S3's
    /// decision, `blocked: needs decision`. So a dungeon raided three times
    /// for four pays four, four, and two, and then pays nothing at all, for as
    /// long as this round lasts.
    ///
    /// Refuses **before mutating** on an instance the campaign does not carry,
    /// a definition the registry does not carry, and a building that is not a
    /// dungeon -- a store of value is not loot until something says the place
    /// can be raided.
    pub fn draw_dungeon_reward(
        &mut self,
        building_instance_id: &str,
        requested: u32,
        definitions: &BuildingDefinitions,
    ) -> Result<u32, DungeonError> {
        // Every refusal is decided against an immutable borrow, before the
        // mutable one exists. There is no path from here that leaves a budget
        // half-spent.
        let instance = self.buildings.get(building_instance_id).ok_or_else(|| {
            DungeonError::Building(BuildingError::UnknownBuilding {
                id: building_instance_id.to_owned(),
            })
        })?;
        let definition = definitions.require(&instance.def_id)?;
        if !presents_a_dungeon(&definition.dungeon_relationship) {
            return Err(DungeonError::NotADungeon {
                id: building_instance_id.to_owned(),
            });
        }

        let instance = self
            .buildings
            .get_mut(building_instance_id)
            .expect("the instance was just read under an immutable borrow");
        let drawn = requested.min(instance.stored_value);
        instance.stored_value -= drawn;
        Ok(drawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::building::{
        BuildingDefinition, BuildingDefinitions, BuildingSocket, CaptureRules, FIRST_TIER,
        RuinState, SocketKind, TierState,
    };
    use crate::strategy::clocks::HeatEvent;
    use crate::strategy::faction::ConceptKey;
    use std::collections::{BTreeSet, HashSet};

    const BEACH: &str = "world.cell.black_beach";
    const TERRACE: &str = "world.cell.reception_terrace";

    /// C10 owns `content/buildings/`; these IDs are this module's own and are
    /// read by nothing outside it.
    const TOMB: &str = "building.test.tomb";
    const SHED: &str = "building.test.shed";
    const INSTANCE: &str = "building_instance.test.tomb";

    fn pirates() -> String {
        ConceptKey::Pirates.faction_id()
    }

    fn elves() -> String {
        ConceptKey::Elves.faction_id()
    }

    fn tiers(count: u32) -> Vec<TierState> {
        (FIRST_TIER..FIRST_TIER + count)
            .map(|tier| TierState {
                tier,
                construction_hours: 0,
                construction_requirements: BTreeSet::new(),
                hit_points: 100 * tier,
                notes: String::new(),
            })
            .collect()
    }

    /// A definition that loads, with `dungeon_relationship` under this test's
    /// control so both sides of [`presents_a_dungeon`] can be exercised.
    fn a_definition(id: &str, dungeon_relationship: &str) -> BuildingDefinition {
        BuildingDefinition {
            id: id.to_owned(),
            faction_compatibility: BTreeSet::from([ConceptKey::Pirates, ConceptKey::Elves]),
            function: "a test building".into(),
            footprint_cells: 2,
            clearance_cells: 1,
            height_class: "needs decision".into(),
            entrance_sockets: vec![BuildingSocket {
                id: "socket.door".into(),
                kind: SocketKind::Entrance,
                offset_cells: 0,
            }],
            tier_states: tiers(3),
            capture_rules: CaptureRules::Capturable,
            ruin_state: RuinState::ClearsCompletely,
            dungeon_relationship: dungeon_relationship.to_owned(),
            ..BuildingDefinition::default()
        }
    }

    fn registry() -> BuildingDefinitions {
        let mut registry = BuildingDefinitions::new();
        registry
            .insert(a_definition(TOMB, "dungeon_grammar.test.tomb"))
            .expect("the tomb definition loads");
        registry
            .insert(a_definition(SHED, NOT_A_DUNGEON))
            .expect("the shed definition loads");
        registry
    }

    /// A campaign holding the beach, with a tomb standing on it.
    fn a_campaign(geography: &Geography, holder: &str) -> ExpeditionState {
        let definitions = registry();
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        state
            .set_control(BEACH, Some(holder.to_owned()), geography)
            .expect("the beach is a real cell");
        state
            .place_building(INSTANCE, TOMB, BEACH, holder, geography, &definitions)
            .expect("the tomb is placed on ground its holder holds");
        state
    }

    fn a_context() -> DungeonContext {
        let geography = Geography::black_beach_vertical_slice();
        let state = a_campaign(&geography, &pirates());
        DungeonContext::of(&state, &geography, INSTANCE, &registry())
            .expect("the tomb presents a dungeon")
    }

    /// S9's Done-when, first half: the same context yields the same rooms.
    ///
    /// Rooms are C5's; what this lane can prove is the whole of what rooms are
    /// selected from -- the signature and the seed sequence it hands on.
    #[test]
    fn the_same_context_always_signs_and_seeds_the_same() {
        let one = a_context();
        let two = a_context();
        assert_eq!(
            one, two,
            "two reads of the same board give the same context"
        );
        assert_eq!(
            dungeon_signature(&one),
            dungeon_signature(&two),
            "the same context folds to the same signature"
        );

        let rooms_once: Vec<u64> = (0..12).map(|i| room_seed(one.signature(), i)).collect();
        let rooms_again: Vec<u64> = (0..12).map(|i| room_seed(two.signature(), i)).collect();
        assert_eq!(
            rooms_once, rooms_again,
            "the same signature yields the same twelve room seeds"
        );
        assert_eq!(
            rooms_once.iter().collect::<HashSet<_>>().len(),
            rooms_once.len(),
            "twelve rooms of one dungeon draw twelve different seeds"
        );
    }

    /// Every single field of [`DungeonContext`] changes the signature.
    ///
    /// Written as a loop over single-field mutations rather than a field list
    /// inside a fold, so that a field added to the struct cannot quietly stay
    /// outside the signature: it has to be added to this list, and a field
    /// dropped from the fold fails here by name.
    ///
    /// Bite proof: `#[serde(skip)]` on `building_tier` fails this test naming
    /// `building_tier`, and nothing else.
    #[test]
    fn every_single_field_changes_the_signature() {
        let base = a_context();
        let baseline = dungeon_signature(&base);

        let mutations: Vec<(&str, fn(&mut DungeonContext))> = vec![
            ("campaign_seed", |c| c.campaign_seed ^= 0xFFFF),
            ("node_id", |c| c.node_id = TERRACE.to_owned()),
            ("entrance_building_id", |c| {
                c.entrance_building_id = "building_instance.test.other".to_owned()
            }),
            ("owning_faction_id", |c| c.owning_faction_id = Some(elves())),
            ("building_archetype_id", |c| {
                c.building_archetype_id = SHED.to_owned()
            }),
            ("building_tier", |c| c.building_tier += 1),
            ("upgrade_signature", |c| c.upgrade_signature ^= 0xFFFF),
            ("corruption_band", |c| {
                c.corruption_band = CorruptionBand::Consumed
            }),
            ("weather_state", |c| {
                c.weather_state = WeatherCondition::Unnatural
            }),
            ("world_epoch", |c| c.world_epoch = WorldEpoch::Endgame),
            ("generated_seed", |c| c.generated_seed ^= 0xFFFF),
            ("grammar_id", |c| {
                c.grammar_id = "dungeon_grammar.test.other".to_owned()
            }),
            ("encounter_budget", |c| c.encounter_budget += 1),
            ("hazard_budget", |c| c.hazard_budget += 1),
            ("reward_budget", |c| c.reward_budget += 1),
            ("generated_room_ids", |c| {
                c.generated_room_ids
                    .push("dungeon_room.test.antechamber".to_owned())
            }),
            ("revision", |c| c.revision += 1),
            ("heat_band", |c| c.heat_band = HeatBand::Imminent),
        ];

        let mutation_count = mutations.len();
        let mut seen: HashSet<u64> = HashSet::from([baseline]);
        for (field, mutate) in mutations {
            let mut mutated = base.clone();
            mutate(&mut mutated);
            assert_ne!(
                mutated, base,
                "the mutation for {field} must actually change the context"
            );
            let signature = dungeon_signature(&mutated);
            assert_ne!(
                signature, baseline,
                "changing {field} alone must change the signature"
            );
            assert!(
                seen.insert(signature),
                "changing {field} collided with another field's signature"
            );
            assert_ne!(
                room_seed(signature, 0),
                room_seed(baseline, 0),
                "changing {field} must change the first room's seed"
            );
        }

        // ...and the list covers the whole struct. Counted from the
        // serialization rather than typed out, so a field added without a
        // mutation beside it fails here rather than passing unnoticed. The
        // check comes *after* the loop deliberately: a field dropped from the
        // fold is named by its own assertion above, and only a field missing
        // from the list at all reaches this one.
        let field_count = match serde_json::to_value(&base).expect("a context serializes") {
            serde_json::Value::Object(fields) => fields.len(),
            other => panic!("a DungeonContext serializes as an object, not {other:?}"),
        };
        assert_eq!(
            mutation_count, field_count,
            "every field of DungeonContext must appear in this list"
        );
    }

    /// S9's Done-when, second half: an owner change is a different dungeon, and
    /// so is a tier change. The card's two named collisions, held apart.
    #[test]
    fn owner_and_tier_never_collide() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();

        let held_by_pirates = a_campaign(&geography, &pirates());
        let mut held_by_elves = a_campaign(&geography, &elves());

        let pirate_context =
            DungeonContext::of(&held_by_pirates, &geography, INSTANCE, &definitions)
                .expect("the tomb presents a dungeon");
        let elf_context = DungeonContext::of(&held_by_elves, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(
            pirate_context.owning_faction_id.as_deref(),
            Some(pirates().as_str())
        );
        assert_eq!(
            elf_context.owning_faction_id.as_deref(),
            Some(elves().as_str())
        );
        assert_ne!(
            pirate_context.signature(),
            elf_context.signature(),
            "the same tomb under a different owner is a different dungeon"
        );

        // The same building, one tier higher: also a different dungeon, and
        // different from both of the above.
        held_by_elves
            .upgrade_building(INSTANCE, &BTreeSet::new(), &definitions)
            .expect("the tomb authors a second tier with no requirements");
        let upgraded = DungeonContext::of(&held_by_elves, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(upgraded.building_tier, FIRST_TIER + 1);
        let signatures = HashSet::from([
            pirate_context.signature(),
            elf_context.signature(),
            upgraded.signature(),
        ]);
        assert_eq!(
            signatures.len(),
            3,
            "owner and tier are independent dimensions of the same signature"
        );
    }

    /// Brief section 12: "Repeated farming must not generate infinite
    /// high-tier loot." Three draws on a budget of ten yield ten, and then
    /// nothing.
    ///
    /// Bite proof: replacing `requested.min(instance.stored_value)` with
    /// `requested` makes the total eighteen and this test fail.
    #[test]
    fn repeated_farming_draws_the_budget_down_and_stops() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign(&geography, &pirates());
        state
            .buildings
            .get_mut(INSTANCE)
            .expect("the tomb was placed")
            .stored_value = 10;

        let drawn: Vec<u32> = (0..3)
            .map(|_| {
                state
                    .draw_dungeon_reward(INSTANCE, 6, &definitions)
                    .expect("the tomb is a dungeon")
            })
            .collect();
        assert_eq!(drawn, vec![6, 4, 0], "the third raid finds an empty tomb");
        assert_eq!(drawn.iter().sum::<u32>(), 10, "a budget of ten pays ten");
        assert_eq!(
            state.buildings[INSTANCE].stored_value, 0,
            "the budget saturates at zero and never passes it"
        );

        // Nothing in this round refills it: S13 and S3 decide what production,
        // reinforcement and recovery put back. A hundred more raids pay
        // nothing.
        for _ in 0..100 {
            assert_eq!(
                state
                    .draw_dungeon_reward(INSTANCE, 6, &definitions)
                    .expect("the tomb is still a dungeon"),
                0
            );
        }

        // The reward budget a context reports is that same field, not a number
        // derived from tier -- so a farmed-out dungeon reads as empty.
        let context = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(context.reward_budget, 0);
    }

    /// A draw refuses before it mutates: an unknown instance, and a building
    /// that is not a dungeon.
    #[test]
    fn a_draw_refuses_before_mutating() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign(&geography, &pirates());
        state
            .place_building(
                "building_instance.test.shed",
                SHED,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("a shed fits beside the tomb");
        state
            .buildings
            .get_mut("building_instance.test.shed")
            .expect("the shed was placed")
            .stored_value = 50;

        assert_eq!(
            state.draw_dungeon_reward("building_instance.test.nowhere", 1, &definitions),
            Err(DungeonError::Building(BuildingError::UnknownBuilding {
                id: "building_instance.test.nowhere".to_owned()
            }))
        );
        assert_eq!(
            state.draw_dungeon_reward("building_instance.test.shed", 1, &definitions),
            Err(DungeonError::NotADungeon {
                id: "building_instance.test.shed".to_owned()
            }),
            "a store of value is not loot until something says the place is a dungeon"
        );
        assert_eq!(
            state.buildings["building_instance.test.shed"].stored_value, 50,
            "a refused draw takes nothing"
        );
    }

    /// The cell-level read: a cell presents the dungeon standing on it, and
    /// nothing where there is none.
    #[test]
    fn a_cell_presents_only_the_dungeon_standing_on_it() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign(&geography, &pirates());

        let context = state
            .dungeon_context_at(BEACH, &geography, &definitions)
            .expect("the beach carries a tomb");
        assert_eq!(context.entrance_building_id, INSTANCE);
        assert_eq!(context.node_id, BEACH);
        assert_eq!(context.grammar_id, "dungeon_grammar.test.tomb");

        assert!(
            state
                .dungeon_context_at(TERRACE, &geography, &definitions)
                .is_none(),
            "an empty cell presents no dungeon"
        );
        assert!(
            state
                .dungeon_context_at("world.cell.nowhere", &geography, &definitions)
                .is_none(),
            "a cell the graph does not know presents no dungeon"
        );

        // A cell carrying only a shed presents no dungeon either: it is the
        // definition's `dungeon_relationship` that decides, not the presence of
        // a building.
        state
            .set_control(TERRACE, Some(pirates()), &geography)
            .expect("the terrace is a real cell");
        state
            .place_building(
                "building_instance.test.shed",
                SHED,
                TERRACE,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("a shed stands on the terrace");
        assert!(
            state
                .dungeon_context_at(TERRACE, &geography, &definitions)
                .is_none(),
            "a shed is a building, not a dungeon"
        );
    }

    /// The three world dimensions the context reads are banded, not raw: one
    /// point of corruption, one day, or one point of hidden pressure does not
    /// regenerate a dungeon, but crossing a threshold does.
    #[test]
    fn world_state_reaches_the_context_as_bands() {
        assert_eq!(CorruptionBand::of(0), CorruptionBand::Untouched);
        assert_eq!(
            CorruptionBand::of(CORRUPTION_BAND_TOUCHED),
            CorruptionBand::Touched
        );
        assert_eq!(
            CorruptionBand::of(CORRUPTION_BAND_CONSUMED),
            CorruptionBand::Consumed
        );
        assert_eq!(HeatBand::of(0), HeatBand::Dormant);
        assert_eq!(HeatBand::of(HEAT_BAND_IMMINENT), HeatBand::Imminent);
        assert_eq!(WorldEpoch::of(1), WorldEpoch::Landfall);
        assert_eq!(WorldEpoch::of(EPOCH_ENDGAME_DAY), WorldEpoch::Endgame);

        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign(&geography, &pirates());
        let before = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(before.heat_band, HeatBand::Dormant);
        assert_eq!(before.corruption_band, CorruptionBand::Untouched);

        // One heat event, well below the first threshold: same band, same
        // dungeon.
        state.record_heat_event(HeatEvent::PartyInterference);
        let nudged = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(
            nudged.signature(),
            before.signature(),
            "hidden pressure inside a band does not regenerate the dungeon"
        );

        // Enough events to cross a threshold: a different dungeon.
        for _ in 0..HEAT_BAND_IMMINENT {
            state.record_heat_event(HeatEvent::PartyInterference);
        }
        let crossed = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(crossed.heat_band, HeatBand::Imminent);
        assert_ne!(
            crossed.signature(),
            before.signature(),
            "crossing a band is a different dungeon"
        );

        // Corruption reaches the context through the same banding, and lifts
        // the hazard budget without touching the encounter budget.
        state
            .corrupt_cell(BEACH, CORRUPTION_BAND_CONSUMED, &geography)
            .expect("the beach is a real cell");
        let corrupted = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(corrupted.corruption_band, CorruptionBand::Consumed);
        assert_eq!(corrupted.encounter_budget, before.encounter_budget);
        assert!(corrupted.hazard_budget > before.hazard_budget);
    }

    /// A context read off the board is the pre-generation record, and the
    /// signature is stable across a save and reload.
    #[test]
    fn a_context_read_off_the_board_has_not_generated_yet() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let state = a_campaign(&geography, &pirates());
        let context = DungeonContext::of(&state, &geography, INSTANCE, &definitions)
            .expect("the tomb presents a dungeon");
        assert_eq!(context.generated_seed, 0);
        assert!(context.generated_room_ids.is_empty());
        assert_eq!(context.revision, 0);

        let json = state.to_json();
        let reloaded = ExpeditionState::from_json(&json).expect("the campaign reloads");
        let after = DungeonContext::of(&reloaded, &geography, INSTANCE, &definitions)
            .expect("the tomb still presents a dungeon");
        assert_eq!(
            after.signature(),
            context.signature(),
            "a save and reload is the same dungeon"
        );
    }
}
