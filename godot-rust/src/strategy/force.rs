//! S7: offscreen forces -- the island's bodies of troops while nobody is
//! looking at them. `docs/SHIP_PLAN.md` section 7, card S7; brief section 10,
//! "Full local simulation and offscreen simulation".
//!
//! A force is an *aggregate*: one record standing for a body of actors that is
//! too far away to simulate individually. Brief section 10 lists what such a
//! record may retain, and [`ForceRecord`] carries that list field for field --
//! faction, roles, composition, strength, readiness, supply, origin, route,
//! destination, assignment, progress, player-detectable evidence.
//!
//! ## The one rule this file exists to keep
//!
//! **A force never teleports.** Brief section 10: "It must not teleport into
//! the room merely because the simulation decided reinforcements exist." So
//! there is no method here that sets a force's position, and none that takes a
//! cell ID and puts a force in it. The only thing that changes a force's
//! position is [`ExpeditionState::advance_forces`] taking one hop along one
//! real [`RouteOption`] that the graph says leads out of the cell the force is
//! actually standing in, once it has accrued that route's own
//! `time_cost_minutes` of marching. A route the graph does not carry is a halt,
//! not a jump.
//!
//! ## Where this lane stops: materialisation is B11's
//!
//! When a force reaches the party's cell or one next to it, brief section 10
//! says it materialises *through that cell's spawn sockets* as actors. Spawn
//! sockets are B11's card and are **not built here** -- `blocked: needs B11`.
//! What this lane delivers instead is the seam B11 attaches to:
//!
//! * [`StrategicEvent::ForceArrived`] carries the arriving force's **full
//!   composition**, so the materialiser is handed what to build rather than
//!   having to guess it from a strength number;
//! * [`ExpeditionState::forces_at`] answers "which forces are standing in this
//!   cell right now", which is the query a socket-filler and the bridge's
//!   projection both need;
//! * the record itself is untouched by arrival, so the aggregate is still
//!   there to be reconciled against whatever B11 spawns. Brief section 10
//!   requires composition, damage, supply, leadership, equipment and recruited
//!   identity to survive the aggregate-to-local transition; keeping the
//!   aggregate intact across the event is what makes that possible instead of
//!   merely intended.
//!
//! ## S18: what an arrival *does* mean
//!
//! Materialisation is still B11's. What card S18 added is the other half of an
//! arrival, the one that needs no actors on the ground:
//! [`ExpeditionState::resolve_arrivals`] turns a force reaching a cell into a
//! change of who **holds** that cell -- unheld or undefended ground changes
//! hands through `ExpeditionState::set_control`, defended ground does not.
//! That is a fact about the board, not a fact about the room, so it can be true
//! before a single actor exists. It is also the line card S17 recorded as
//! `failed`: nothing wrote `ExpeditionState::ownership` from a tick, so no
//! faction could ever be pushed off its ground and brief section 16's recovery
//! chain could never run out.
//!
//! ## Whose map a force walks
//!
//! [`Geography::next_step_toward`] takes the set of open gates -- the
//! discoveries that make a gated route legal. [`ExpeditionState::dispatch_force`]
//! passes an **empty** set, deliberately: a force plans with its own faction's
//! knowledge, and the party's `discoveries` are the *party's* knowledge. The
//! tidal cut in the vertical slice is exactly this case -- the party reads the
//! estate's map table and a shore shortcut opens for the party, and a faction
//! marching inland has no reason to know a door opened because Captain Michael
//! opened it. A faction's own gate knowledge, when a later card gives factions
//! one, arrives here as a set read off the faction rather than as a second
//! pathfinder.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, ExpeditionState, require_stable_id};
use crate::geography::{Geography, RouteOption};
use crate::strategy::tick::{StrategicEvent, hour_draws};

/// The prefix every force ID carries, as `faction.` is to a faction.
pub const FORCE_ID_PREFIX: &str = "force.";

/// Minutes of marching one strategic hour buys a force. The strategic tick is
/// one in-world hour ([`crate::strategy::tick::HOURS_PER_DAY`]), and a route's
/// `time_cost_minutes` is authored in minutes, so this is the conversion
/// between the two and the only place it is written.
pub const MINUTES_PER_STRATEGIC_HOUR: u32 = 60;

/// Readiness a freshly raised force stands at, and its ceiling. Readiness is a
/// percentage of "fit to fight now", nothing finer.
pub const READINESS_FULL: u8 = 100;

/// **Provisional.** What one hop costs a force in readiness. Marching tires a
/// body of troops; the number is a placeholder chosen so that the slice's
/// longest march is felt without being crippling, and brief section 11
/// ("Dynamic difficulty") owns the eventual relationship between readiness,
/// strength and equipment. Nothing branches on the exact value.
pub const READINESS_COST_PER_HOP: u8 = 2;

/// **Provisional.** How far the hour's `strategic.force` draw may move
/// readiness on a hop, in either direction: a march goes a little better or a
/// little worse than the constant says. Capped hard, so the draw colours the
/// march and never decides it.
pub const READINESS_DRAW_BAND: u8 = 1;

/// **Provisional.** What one hop costs a force in supply *on top of* the
/// route's own authored `supply_cost`. The authored number is the road's
/// demand and is already the right owner of "this way is hungrier"; this is
/// the flat cost of being a body of troops on the move at all, so that a force
/// on a slice road that charges the party nothing still eats.
pub const SUPPLY_COST_PER_HOP: u32 = 1;

/// **Provisional.** Supply a force is raised with. Enough for a long march
/// across the authored slice and not enough to march forever, which is what
/// makes [`StrategicEvent::ForceHalted`] a real outcome rather than a branch
/// nothing reaches. S3/S13 own supply as an economy; this is the stand-in
/// until a force can be provisioned from one.
pub const SUPPLY_ON_RAISE: u32 = 24;

/// **needs decision** -- whether a force arriving on ground its owner is
/// standing on can take that ground.
///
/// `false`, and the falseness is the point rather than a balance choice. Brief
/// section 20 leaves force-versus-force resolution undecided: there is no
/// combat model for two aggregates, no casualties, no morale and no retreat,
/// so every way of resolving a contested arrival here -- strength wins,
/// attacker wins, defender wins -- would be this lane authoring the rule the
/// brief has not written. So the arrival is recorded
/// ([`StrategicEvent::ArrivalContested`]) and the board does not move. The lane
/// that decides how two forces meet replaces this constant with that rule; it
/// is a named `false` rather than an absent branch precisely so it can be
/// found.
pub const CONTESTED_ARRIVAL_RESOLVES_CONTROL: bool = false;

/// The stable ID of one force: `force.<something>`, in the crate's one stable-ID
/// shape (lowercase, dotted, no spaces).
///
/// A newtype rather than a bare `String` so that "the ID of a force" cannot be
/// confused at a call site with the faction ID, the cell ID or the assignment
/// string that stand beside it in every signature in this file.
/// `serde(transparent)`: on disk it is the plain string, so
/// `ExpeditionState::forces` is a map of string to record exactly as every
/// other keyed map in the save is.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ForceId(String);

impl ForceId {
    /// Validates shape and the `force.` prefix. Refuses rather than repairs: an
    /// ID that drifted is the first place a faction proper name would appear.
    pub fn new(id: &str) -> Result<Self, ForceError> {
        require_stable_id("force_id", id).map_err(ForceError::MalformedId)?;
        if !id.starts_with(FORCE_ID_PREFIX) {
            return Err(ForceError::IdIsNotAForceId {
                found: id.to_owned(),
            });
        }
        Ok(Self(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ForceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// What the party could find of a force that is not in the room: the abstract
/// categories, with no faction flavour in any of them. Which evidence a given
/// march actually leaves is content's decision (brief section 10 names the
/// field and nothing else), so movement in this file never changes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForceEvidence {
    /// Marks on the ground: something passed this way.
    #[default]
    Tracks,
    /// Something is burning, or cooking, where it should not be.
    Smoke,
    /// Somebody said so. The least reliable and the most common.
    Rumour,
    /// Seen, at distance, by somebody who could describe it.
    Sighting,
}

/// What went wrong raising or dispatching a force.
///
/// A module-local error vocabulary, following [`ForceError::MalformedId`]'s
/// own carrier `ExpeditionError` and `FactionError`: one owner per failure
/// vocabulary, and ID *shape* is not re-implemented here -- it is the crate's
/// single `require_stable_id` rule, quoted.
#[derive(Clone, Debug, PartialEq)]
pub enum ForceError {
    /// Not a stable ID at all (empty, uppercase, no dot, stray punctuation).
    MalformedId(ExpeditionError),
    /// Well-shaped, but not a `force.<...>` ID.
    IdIsNotAForceId { found: String },
    /// Two forces would claim the same ID. One force, one record.
    DuplicateForce { id: String },
    /// Orders were given to a force this campaign has no record of. Refused
    /// rather than created: a stray ID must not be able to raise an army.
    UnknownForce { id: String },
    /// The origin, destination or standing cell is not on the graph. Refused
    /// before anything mutates, so a typo cannot invent a place to march to.
    UnknownCell { cell_id: String },
    /// The graph offers no path from where the force stands to where it was
    /// told to go, through the gates that force knows about. Refused, because
    /// the alternative -- accepting the order and quietly never arriving -- is
    /// the shape of bug that looks like a balance problem for a month.
    Unreachable { from: String, to: String },
    /// A composition with no actors in it is not a force. Refused at the point
    /// of raising, so an empty body can never arrive somewhere and materialise
    /// into nothing.
    EmptyComposition { id: String },
}

/// One offscreen force, exactly as brief section 10 lists it, plus the one
/// thing that list leaves implicit and movement cannot do without.
///
/// Brief section 10's list is of what a force *retains* -- it names `origin`,
/// `route`, `destination` and `progress` but no standing position, because at
/// the resolution the brief is written at, "where it is" is a thing you read
/// off the route. Storing it ([`ForceRecord::position_cell_id`]) rather than
/// deriving it keeps [`ExpeditionState::forces_at`] a question the save can
/// answer on its own, without the graph; the hop is the single writer, so the
/// stored cell and the route cannot drift apart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForceRecord {
    /// This force's own stable ID; equal to its key in
    /// [`ExpeditionState::forces`].
    pub id: ForceId,
    /// Whose force it is: `faction.<concept_key>`. Never a proper name.
    pub faction_id: String,
    /// What this body is *for*, as authored role keys. Open set of strings from
    /// content (brief section 20 leaves the list Open), so no enum.
    #[serde(default)]
    pub roles: BTreeSet<String>,
    /// **What it is made of**: actor-definition ID to how many of them.
    /// Content's keys; nothing here invents an actor. This is the field brief
    /// section 10 requires to survive the aggregate-to-local transition intact,
    /// and it is the one the arrival event carries whole.
    #[serde(default)]
    pub composition: BTreeMap<String, u32>,
    /// The aggregate weight of the force. Derived from [`Self::composition`] by
    /// [`ForceRecord::recompute_strength`], which is its only writer, so there
    /// are never two answers to "how strong is it". Today that is the head
    /// count; brief section 11 ("Dynamic difficulty") owns making equipment and
    /// experience count, and it will do so inside that one function.
    #[serde(default)]
    pub strength: u32,
    /// Fitness to fight now, `0..=`[`READINESS_FULL`]. Falls with marching.
    #[serde(default)]
    pub readiness: u8,
    /// What it has left to eat and burn. At zero the force halts where it
    /// stands rather than vanishing.
    #[serde(default)]
    pub supply: u32,
    /// The cell it was raised in, or last dispatched from. Not where it is:
    /// that is [`Self::position_cell_id`].
    pub origin_cell_id: String,
    /// Where it actually stands. Written only by a hop.
    pub position_cell_id: String,
    /// The portal IDs still to be walked, in order, front first. Empty means
    /// the force is not marching. Each entry is a real `RouteOption` ID on the
    /// `Geography`; a force cannot hold a route the graph does not carry.
    #[serde(default)]
    pub route: Vec<String>,
    /// Where it is going, while [`Self::route`] is non-empty.
    #[serde(default)]
    pub destination_cell_id: Option<String>,
    /// What it was told to do when it gets there, as the caller's own string.
    /// Assignments are the strategic layer's vocabulary (S5's goals, S6's
    /// directives) and this lane does not interpret one.
    #[serde(default)]
    pub assignment: String,
    /// Minutes of marching accrued toward the next route's `time_cost_minutes`.
    /// Never as large as that cost after a hop: the remainder carries forward,
    /// so a long chain of short roads costs exactly what the roads cost.
    #[serde(default)]
    pub progress_minutes: u32,
    /// What the party could find of it. See [`ForceEvidence`].
    #[serde(default)]
    pub evidence: ForceEvidence,
}

impl ForceRecord {
    /// The one writer of [`Self::strength`]. Called wherever composition
    /// changes; today that is only at raising, because nothing in this lane
    /// takes casualties.
    pub fn recompute_strength(&mut self) {
        self.strength = self
            .composition
            .values()
            .fold(0u32, |total, count| total.saturating_add(*count));
    }

    /// Whether this force is marching -- it has somewhere to be and road left.
    pub fn is_marching(&self) -> bool {
        !self.route.is_empty()
    }
}

/// Why a force stopped short of where it was sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HaltReason {
    /// It ran out of supply on the road. It is still there, and still whole:
    /// brief section 10's aggregate does not evaporate because the ledger did.
    OutOfSupply,
    /// The route it was holding is not on the graph any more. A force may not
    /// step onto a road that does not exist, and it may not skip it either, so
    /// it stops and says so.
    RouteMissing,
}

impl ExpeditionState {
    /// Raise a force: the point at which a body of actors becomes one aggregate
    /// record. It stands where it is raised and has no orders yet.
    ///
    /// Rejects before mutating on a malformed or non-`force.` ID, an ID this
    /// campaign already carries, a cell the graph does not know, or an empty
    /// composition.
    pub fn raise_force(
        &mut self,
        force_id: &str,
        faction_id: &str,
        cell_id: &str,
        roles: BTreeSet<String>,
        composition: BTreeMap<String, u32>,
        assignment: &str,
        geography: &Geography,
    ) -> Result<ForceId, ForceError> {
        let id = ForceId::new(force_id)?;
        require_stable_id("faction_id", faction_id).map_err(ForceError::MalformedId)?;
        if self.forces.contains_key(id.as_str()) {
            return Err(ForceError::DuplicateForce { id: id.to_string() });
        }
        if geography.location(cell_id).is_none() {
            return Err(ForceError::UnknownCell {
                cell_id: cell_id.to_owned(),
            });
        }
        if composition.is_empty() || composition.values().all(|count| *count == 0) {
            return Err(ForceError::EmptyComposition { id: id.to_string() });
        }

        let mut record = ForceRecord {
            id: id.clone(),
            faction_id: faction_id.to_owned(),
            roles,
            composition,
            strength: 0,
            readiness: READINESS_FULL,
            supply: SUPPLY_ON_RAISE,
            origin_cell_id: cell_id.to_owned(),
            position_cell_id: cell_id.to_owned(),
            route: Vec::new(),
            destination_cell_id: None,
            assignment: assignment.to_owned(),
            progress_minutes: 0,
            evidence: ForceEvidence::default(),
        };
        record.recompute_strength();
        self.forces.insert(id.to_string(), record);
        Ok(id)
    }

    /// Send a force somewhere, planning the whole road now.
    ///
    /// The plan is [`Geography::next_step_toward`] applied repeatedly from each
    /// cell the previous step lands in, so the stored route is a chain of real
    /// `RouteOption` IDs whose ends meet -- which is what makes a hop a hop and
    /// not a jump. Planning the whole road at dispatch rather than one step per
    /// hour is deliberate: it is the moment a bad order can be *refused*, and a
    /// force that cannot get there never sets off.
    ///
    /// **Open gates: none.** The empty set passed to `next_step_toward` is the
    /// faction's own ignorance of doors the party has opened; see this module's
    /// header for why, and for where a faction's own gate knowledge will attach
    /// when a card gives factions one.
    ///
    /// Rejects before mutating on an unknown force, an unknown destination, or
    /// a destination nothing connects to. Returns the departure event for the
    /// caller to journal; dispatching is an order given from outside the hour,
    /// so it does not journal itself.
    pub fn dispatch_force(
        &mut self,
        force_id: &str,
        destination_cell_id: &str,
        geography: &Geography,
    ) -> Result<StrategicEvent, ForceError> {
        let force = self
            .forces
            .get(force_id)
            .ok_or_else(|| ForceError::UnknownForce {
                id: force_id.to_owned(),
            })?;
        if geography.location(destination_cell_id).is_none() {
            return Err(ForceError::UnknownCell {
                cell_id: destination_cell_id.to_owned(),
            });
        }

        let route = plan_route(geography, &force.position_cell_id, destination_cell_id)?;
        let departed = StrategicEvent::ForceDeparted {
            force_id: force.id.clone(),
            faction_id: force.faction_id.clone(),
            from_cell_id: force.position_cell_id.clone(),
            destination_cell_id: destination_cell_id.to_owned(),
            steps: route.len() as u32,
        };

        let force = self
            .forces
            .get_mut(force_id)
            .expect("the same key was just read");
        force.origin_cell_id = force.position_cell_id.clone();
        force.destination_cell_id = Some(destination_cell_id.to_owned());
        force.route = route;
        force.progress_minutes = 0;
        Ok(departed)
    }

    /// One hour of marching for every force that has road left.
    ///
    /// Called by
    /// [`ExpeditionState::strategic_tick`](crate::expedition::ExpeditionState::strategic_tick)
    /// once per strategic hour, after the hour's faction draws have been made.
    /// Each marching force accrues [`MINUTES_PER_STRATEGIC_HOUR`] and then
    /// takes as many hops as it has paid for -- a force on ten-minute roads
    /// crosses several in an hour, and one on a sixty-minute road crosses it in
    /// exactly one, because the accrual is in the road's own minutes rather
    /// than in steps.
    ///
    /// Every hop pays [`READINESS_COST_PER_HOP`] in readiness and
    /// [`SUPPLY_COST_PER_HOP`] plus the road's own `supply_cost` in supply. A
    /// force that cannot pay the supply halts where it stands with
    /// [`StrategicEvent::ForceHalted`], keeping its orders, its composition and
    /// its position; it does not vanish and it does not arrive.
    ///
    /// **The hour's `strategic.force` draw is read**, through
    /// [`hour_draws`] -- the pure accessor S4 reserves for this lane -- and
    /// spent on one thing: moving readiness by at most
    /// [`READINESS_DRAW_BAND`] on a hop, so two identical marches are not
    /// identically fresh at the end of them. It is read, never made: nothing
    /// here calls `mix_seed`, nothing folds into `draw_digest` (that is
    /// `run_hour`'s, and only for the draws it makes), and the draw belongs to
    /// the *faction* and the hour, so two forces of one faction hopping in the
    /// same hour are perturbed alike. The hour read is the one standing on the
    /// clock when this runs, which is the hour `run_hour` has just advanced
    /// *to*; that is a stable fact of the sequence, not an accident, and it
    /// stays stable because this is the only reader.
    pub fn advance_forces(&mut self, geography: &Geography) -> Vec<StrategicEvent> {
        let day = self.campaign_day;
        let hour = self.strategic_clock.hour_of_day;
        let rng_seed = self.rng_seed;
        let mut events = Vec::new();

        for force in self.forces.values_mut() {
            if !force.is_marching() {
                continue;
            }
            let draw = hour_draws(rng_seed, day, hour, &force.faction_id)
                .get("strategic.force")
                .copied()
                .expect("PURPOSES reserves strategic.force and hour_draws makes every purpose");
            events.extend(march_one_hour(force, geography, draw));
        }
        events
    }

    /// Which forces are standing in this cell right now.
    ///
    /// **The seam B11 reads.** Materialisation through a cell's spawn sockets
    /// is B11's card and is not built in this lane (`blocked: needs B11`):
    /// when a force arrives at the party's cell or one adjacent to it, this
    /// lane emits [`StrategicEvent::ForceArrived`] with the composition intact
    /// and stops. B11 asks this question of the cell it is filling, reads
    /// `composition` off each record, and spawns through the sockets -- which
    /// is the brief's "materializes through valid routes and spawn sockets"
    /// rather than an actor appearing because the simulation decided one
    /// existed.
    ///
    /// A force in transit stands in the cell it last hopped into, so it is
    /// never in two cells and never in none.
    pub fn forces_at(&self, cell_id: &str) -> Vec<&ForceRecord> {
        self.forces
            .values()
            .filter(|force| force.position_cell_id == cell_id)
            .collect()
    }

    /// **S18: what an arrival means for who holds the ground.** The hour's
    /// marching is handed back in, and every
    /// [`StrategicEvent::ForceArrived`] in it is resolved against the board the
    /// force landed on.
    ///
    /// Called from
    /// [`ExpeditionState::strategic_tick`](crate::expedition::ExpeditionState::strategic_tick)
    /// immediately after [`ExpeditionState::advance_forces`] and before S10's
    /// elimination sweep -- so a faction pushed off its last cell this hour is
    /// noticed this hour, which is the whole of what card S17 recorded as
    /// `failed` and this card closes.
    ///
    /// Three rules, and they are the card's:
    ///
    /// 1. **Ground nobody holds is taken.** The arriving force's faction takes
    ///    it through [`ExpeditionState::set_control`] -- the one writer of
    ///    `ownership`, called rather than copied -- and
    ///    [`StrategicEvent::ControlTaken`] says so with `from: None`.
    /// 2. **Ground another faction holds with nothing of theirs standing on it
    ///    changes hands the same way**, and `from` names the faction that lost
    ///    it. A cell is not defended by being owned.
    /// 3. **Ground another faction holds with a force of theirs standing on it
    ///    does not change hands**, and [`StrategicEvent::ArrivalContested`]
    ///    names the holder and every defending body. See
    ///    [`CONTESTED_ARRIVAL_RESOLVES_CONTROL`] for why this lane refuses to
    ///    decide it.
    ///
    /// A force arriving on ground its own faction already holds resolves to
    /// nothing at all, and emits nothing: coming home is not news.
    ///
    /// **Captain Michael's cells and forces obey all three**, exactly as every
    /// other faction's do. Brief section 5.9 gives the player his faction's
    /// *decisions*, not an exemption from the board: a column of his that
    /// marches onto unheld ground takes it, and a rival's column that reaches
    /// an undefended cell of his takes that.
    ///
    /// **Buildings stand as they were.** `building_instance_ids` on the event
    /// is `buildings_at` read the instant before the handover; nothing here
    /// captures, ruins, damages or reassigns one, because capture-versus-
    /// destruction by building type is Open (brief section 20). The event
    /// carries the inputs so the lane that closes that decision has them.
    ///
    /// Deterministic and draw-free: the order is the order `advance_forces`
    /// produced (`BTreeMap` over forces), the defenders and the buildings come
    /// out of `BTreeMap`s, and nothing here reads a draw, a clock or a seed.
    /// No purpose is added to
    /// [`PURPOSES`](crate::strategy::tick::PURPOSES); an arrival is a
    /// consequence of hops that were already paid for.
    pub fn resolve_arrivals(
        &mut self,
        geography: &Geography,
        marching: &[StrategicEvent],
    ) -> Vec<StrategicEvent> {
        let day = self.campaign_day;
        let mut events = Vec::new();
        for event in marching {
            let StrategicEvent::ForceArrived {
                force_id,
                faction_id,
                cell_id,
                ..
            } = event
            else {
                continue;
            };
            let holder = geography
                .held_by(cell_id, &self.ownership)
                .map(str::to_owned);
            if holder.as_deref() == Some(faction_id.as_str()) {
                continue;
            }
            // Defence is a body standing here, not a claim on the map. Read
            // before anything moves, and read through `forces_at`, so "who is
            // in this cell" has one owner.
            let defender_force_ids: Vec<ForceId> = match &holder {
                Some(holder) => self
                    .forces_at(cell_id)
                    .into_iter()
                    .filter(|force| &force.faction_id == holder)
                    .map(|force| force.id.clone())
                    .collect(),
                None => Vec::new(),
            };
            if !defender_force_ids.is_empty() && !CONTESTED_ARRIVAL_RESOLVES_CONTROL {
                events.push(StrategicEvent::ArrivalContested {
                    force_id: force_id.clone(),
                    faction_id: faction_id.clone(),
                    cell_id: cell_id.clone(),
                    held_by: holder.expect("a defender was found, so somebody holds this cell"),
                    defender_force_ids,
                    day,
                });
                continue;
            }
            let building_instance_ids: Vec<String> = self
                .buildings_at(cell_id)
                .into_iter()
                .map(|building| building.id.clone())
                .collect();
            match self.set_control(cell_id, Some(faction_id.clone()), geography) {
                Ok(_) => events.push(StrategicEvent::ControlTaken {
                    force_id: force_id.clone(),
                    faction_id: faction_id.clone(),
                    cell_id: cell_id.clone(),
                    from: holder,
                    building_instance_ids,
                    day,
                }),
                // Unreachable in practice -- a force is standing in this cell,
                // so the graph carries it, and the faction ID came off a record
                // `raise_force` validated. Swallowed rather than panicked on:
                // an hour of the island must not be able to abort.
                Err(_) => continue,
            }
        }
        events
    }
}

/// The chain of real routes from `from` to `to`, or the reason there is none.
///
/// Repeated [`Geography::next_step_toward`] with **no gates open**: see
/// [`ExpeditionState::dispatch_force`]. Each step is planned from the cell the
/// previous step lands in, so consecutive entries meet end to end and no cell
/// on the way is skipped.
fn plan_route(geography: &Geography, from: &str, to: &str) -> Result<Vec<String>, ForceError> {
    let faction_knows_no_open_gates: BTreeSet<String> = BTreeSet::new();
    let mut route = Vec::new();
    let mut standing = from.to_owned();
    while standing != to {
        let step = geography
            .next_step_toward(&standing, to, &faction_knows_no_open_gates)
            .ok_or_else(|| ForceError::Unreachable {
                from: from.to_owned(),
                to: to.to_owned(),
            })?;
        route.push(step.id.clone());
        standing = step.to_location_id.clone();
    }
    Ok(route)
}

/// One force's hour. Pure in everything but the record it is handed.
fn march_one_hour(
    force: &mut ForceRecord,
    geography: &Geography,
    draw: u64,
) -> Vec<StrategicEvent> {
    let mut events = Vec::new();
    force.progress_minutes = force
        .progress_minutes
        .saturating_add(MINUTES_PER_STRATEGIC_HOUR);

    while let Some(route_id) = force.route.first().cloned() {
        let Some(route) = geography.route(&route_id) else {
            // The road is gone. Stopping is the only honest answer: stepping
            // to the far end without a road is the teleport this file exists
            // to make impossible, and dropping the leg would skip a cell.
            events.push(StrategicEvent::ForceHalted {
                force_id: force.id.clone(),
                faction_id: force.faction_id.clone(),
                cell_id: force.position_cell_id.clone(),
                reason: HaltReason::RouteMissing,
            });
            return events;
        };
        if force.progress_minutes < route.time_cost_minutes {
            break;
        }
        let cost = supply_cost_of(route);
        if force.supply < cost {
            events.push(StrategicEvent::ForceHalted {
                force_id: force.id.clone(),
                faction_id: force.faction_id.clone(),
                cell_id: force.position_cell_id.clone(),
                reason: HaltReason::OutOfSupply,
            });
            return events;
        }

        force.progress_minutes -= route.time_cost_minutes;
        force.supply -= cost;
        force.readiness = readiness_after_a_hop(force.readiness, draw);
        force.position_cell_id = route.to_location_id.clone();
        force.route.remove(0);

        events.push(StrategicEvent::ForceMoved {
            force_id: force.id.clone(),
            faction_id: force.faction_id.clone(),
            route_id: route.id.clone(),
            from_cell_id: route.from_location_id.clone(),
            to_cell_id: route.to_location_id.clone(),
        });

        if force.route.is_empty() {
            // Arrival. The composition is *copied* into the event, not moved
            // out of the record: brief section 10 requires the aggregate to
            // survive the transition, and B11 reconciles what it spawns
            // against a record that is still there.
            force.progress_minutes = 0;
            force.destination_cell_id = None;
            events.push(StrategicEvent::ForceArrived {
                force_id: force.id.clone(),
                faction_id: force.faction_id.clone(),
                cell_id: force.position_cell_id.clone(),
                composition: force.composition.clone(),
                strength: force.strength,
                readiness: force.readiness,
                supply: force.supply,
            });
            break;
        }
    }
    events
}

/// What a hop off this road costs in supply: the road's authored demand plus
/// the flat cost of moving a body of troops at all.
fn supply_cost_of(route: &RouteOption) -> u32 {
    route.supply_cost.saturating_add(SUPPLY_COST_PER_HOP)
}

/// Readiness after one hop: the constant cost, then the hour's draw inside
/// [`READINESS_DRAW_BAND`]. Saturating at both ends, so readiness is always a
/// percentage and a long enough march ends at zero rather than wrapping.
fn readiness_after_a_hop(readiness: u8, draw: u64) -> u8 {
    let after_marching = readiness.saturating_sub(READINESS_COST_PER_HOP);
    match draw % 3 {
        0 => after_marching.saturating_sub(READINESS_DRAW_BAND),
        1 => after_marching,
        _ => after_marching
            .saturating_add(READINESS_DRAW_BAND)
            .min(READINESS_FULL),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::building::BuildingDefinitions;
    use crate::strategy::faction::ConceptKey;
    use crate::strategy::production::MachineDefinitions;

    const BEACH: &str = "world.cell.black_beach";
    const ESTATE: &str = "world.cell.damaged_estate";
    const LANDING: &str = "world.cell.river_landing";
    const TERRACE: &str = "world.cell.reception_terrace";

    /// Content owns actor IDs. These are the test namespace's, not invented
    /// content: nothing outside this module reads them.
    fn a_composition() -> BTreeMap<String, u32> {
        BTreeMap::from([
            ("actor.test.spearman".to_owned(), 6u32),
            ("actor.test.scout".to_owned(), 2u32),
        ])
    }

    fn a_campaign() -> ExpeditionState {
        ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
            .expect("a fresh campaign constructs")
    }

    fn raise(state: &mut ExpeditionState, geography: &Geography, at: &str) -> ForceId {
        state
            .raise_force(
                "force.test.column",
                &ConceptKey::Pirates.faction_id(),
                at,
                BTreeSet::from(["role.test.line".to_owned()]),
                a_composition(),
                "assignment.test.march",
                geography,
            )
            .expect("a force raises on a real cell with a real composition")
    }

    #[test]
    fn a_raised_force_stands_where_it_was_raised_with_no_orders() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        let id = raise(&mut state, &geography, BEACH);

        let force = &state.forces[id.as_str()];
        assert_eq!(force.position_cell_id, BEACH);
        assert_eq!(force.origin_cell_id, BEACH);
        assert!(force.route.is_empty());
        assert!(!force.is_marching());
        assert_eq!(force.destination_cell_id, None);
        assert_eq!(force.readiness, READINESS_FULL);
        assert_eq!(force.supply, SUPPLY_ON_RAISE);
        assert_eq!(force.strength, 8, "strength is derived from composition");
        assert_eq!(state.forces_at(BEACH).len(), 1);
        assert!(state.forces_at(TERRACE).is_empty());
    }

    #[test]
    fn raising_refuses_a_bad_id_a_duplicate_an_unknown_cell_and_an_empty_body() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();

        assert!(matches!(
            state.raise_force(
                "Force.Test.Column",
                &ConceptKey::Pirates.faction_id(),
                BEACH,
                BTreeSet::new(),
                a_composition(),
                "",
                &geography,
            ),
            Err(ForceError::MalformedId(_))
        ));
        assert!(matches!(
            state.raise_force(
                "band.test.column",
                &ConceptKey::Pirates.faction_id(),
                BEACH,
                BTreeSet::new(),
                a_composition(),
                "",
                &geography,
            ),
            Err(ForceError::IdIsNotAForceId { .. })
        ));
        assert!(matches!(
            state.raise_force(
                "force.test.column",
                &ConceptKey::Pirates.faction_id(),
                "world.cell.nowhere",
                BTreeSet::new(),
                a_composition(),
                "",
                &geography,
            ),
            Err(ForceError::UnknownCell { .. })
        ));
        assert!(matches!(
            state.raise_force(
                "force.test.column",
                &ConceptKey::Pirates.faction_id(),
                BEACH,
                BTreeSet::new(),
                BTreeMap::new(),
                "",
                &geography,
            ),
            Err(ForceError::EmptyComposition { .. })
        ));
        assert!(
            state.forces.is_empty(),
            "every refusal came before mutation"
        );

        raise(&mut state, &geography, BEACH);
        assert!(matches!(
            state.raise_force(
                "force.test.column",
                &ConceptKey::Pirates.faction_id(),
                BEACH,
                BTreeSet::new(),
                a_composition(),
                "",
                &geography,
            ),
            Err(ForceError::DuplicateForce { .. })
        ));
        assert_eq!(state.forces.len(), 1);
    }

    #[test]
    fn dispatch_refuses_an_unknown_force_an_unknown_cell_and_an_unreachable_one() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);

        assert!(matches!(
            state.dispatch_force("force.test.absent", TERRACE, &geography),
            Err(ForceError::UnknownForce { .. })
        ));
        assert!(matches!(
            state.dispatch_force("force.test.column", "world.cell.nowhere", &geography),
            Err(ForceError::UnknownCell { .. })
        ));
        // The archive core is behind D3's gate, and the force knows no gates:
        // unreachable, and refused rather than accepted and never arrived at.
        state
            .discoveries
            .insert("observation.tomb_reception.true_name".to_owned());
        assert!(matches!(
            state.dispatch_force(
                "force.test.column",
                "world.cell.tomb_archive_core",
                &geography
            ),
            Err(ForceError::Unreachable { .. })
        ));
        assert!(
            state.forces["force.test.column"].route.is_empty(),
            "a refused order left no orders behind"
        );
    }

    /// The plan is a chain of real roads whose ends meet, and the gated tidal
    /// cut is not one of them however many discoveries the *party* has made.
    #[test]
    fn a_dispatched_force_plans_a_chain_of_real_roads_and_ignores_the_partys_doors() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        state
            .discoveries
            .insert("discovery.map_table.tidal_cut".to_owned());
        raise(&mut state, &geography, BEACH);

        let departed = state
            .dispatch_force("force.test.column", TERRACE, &geography)
            .expect("the terrace is reachable from the beach");
        let StrategicEvent::ForceDeparted { steps, .. } = &departed else {
            panic!("dispatch reports a departure, got {departed:?}");
        };
        assert_eq!(*steps, 3, "the party's shore shortcut is not the force's");

        let force = &state.forces["force.test.column"];
        let mut standing = BEACH.to_owned();
        for route_id in &force.route {
            let route = geography.route(route_id).expect("a planned road is real");
            assert_eq!(
                route.from_location_id, standing,
                "the roads meet end to end"
            );
            standing = route.to_location_id.clone();
        }
        assert_eq!(standing, TERRACE);
    }

    /// The card's Done-when. Three cells, and what arrives is what left.
    #[test]
    fn a_force_marched_three_cells_arrives_carrying_exactly_what_it_left_with() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        let departed_with = state.forces["force.test.column"].composition.clone();
        assert_eq!(departed_with, a_composition());

        state
            .dispatch_force("force.test.column", TERRACE, &geography)
            .expect("the terrace is reachable");

        let mut seen: Vec<StrategicEvent> = Vec::new();
        for _ in 0..12 {
            seen.extend(state.strategic_tick(
                &geography,
                &definitions,
                &BuildingDefinitions::new(),
                &MachineDefinitions::new(),
            ));
            if seen
                .iter()
                .any(|event| matches!(event, StrategicEvent::ForceArrived { .. }))
            {
                break;
            }
        }

        let arrival = seen
            .iter()
            .find_map(|event| match event {
                StrategicEvent::ForceArrived {
                    cell_id,
                    composition,
                    ..
                } => Some((cell_id.clone(), composition.clone())),
                _ => None,
            })
            .expect("the force arrives within the hours the road costs");
        assert_eq!(arrival.0, TERRACE);
        assert_eq!(
            arrival.1, departed_with,
            "the arrival event carries exactly the composition that left"
        );
        assert_eq!(
            state.forces["force.test.column"].composition, departed_with,
            "and the record still carries it -- the aggregate survives arrival"
        );

        // Three cells crossed, in order, by three hops along three real roads.
        let hops: Vec<(String, String)> = seen
            .iter()
            .filter_map(|event| match event {
                StrategicEvent::ForceMoved {
                    from_cell_id,
                    to_cell_id,
                    ..
                } => Some((from_cell_id.clone(), to_cell_id.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            hops,
            vec![
                (BEACH.to_owned(), ESTATE.to_owned()),
                (ESTATE.to_owned(), LANDING.to_owned()),
                (LANDING.to_owned(), TERRACE.to_owned()),
            ],
            "every cell on the way was stood in; nothing was skipped"
        );
        let force = &state.forces["force.test.column"];
        assert!(force.route.is_empty());
        assert_eq!(force.position_cell_id, TERRACE);
        assert!(state.forces_at(TERRACE).len() == 1);
    }

    /// The same march, saved and reloaded between every hop, is the same march.
    #[test]
    fn a_save_and_reload_between_hops_changes_nothing() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();

        let mut straight_through = a_campaign();
        raise(&mut straight_through, &geography, BEACH);
        straight_through
            .dispatch_force("force.test.column", TERRACE, &geography)
            .expect("the terrace is reachable");
        let mut round_tripped = ExpeditionState::from_json(&straight_through.to_json())
            .expect("a campaign with a marching force round-trips");

        let mut straight_events = Vec::new();
        let mut tripped_events = Vec::new();
        for _ in 0..12 {
            straight_events.extend(straight_through.strategic_tick(
                &geography,
                &definitions,
                &BuildingDefinitions::new(),
                &MachineDefinitions::new(),
            ));
            tripped_events.extend(round_tripped.strategic_tick(
                &geography,
                &definitions,
                &BuildingDefinitions::new(),
                &MachineDefinitions::new(),
            ));
            round_tripped = ExpeditionState::from_json(&round_tripped.to_json())
                .expect("the save reloads between every hop");
        }

        assert_eq!(straight_events, tripped_events);
        assert_eq!(straight_through.to_json(), round_tripped.to_json());
    }

    /// Marching costs, and a force that cannot pay stops where it is with its
    /// composition intact rather than disappearing off the board.
    #[test]
    fn a_force_out_of_supply_halts_where_it_stands_and_keeps_everything() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        state
            .dispatch_force("force.test.column", TERRACE, &geography)
            .expect("the terrace is reachable");
        state
            .forces
            .get_mut("force.test.column")
            .expect("the force was raised")
            .supply = 0;

        let mut seen = Vec::new();
        for _ in 0..4 {
            seen.extend(state.strategic_tick(
                &geography,
                &definitions,
                &BuildingDefinitions::new(),
                &MachineDefinitions::new(),
            ));
        }

        assert!(
            seen.iter().any(|event| matches!(
                event,
                StrategicEvent::ForceHalted {
                    reason: HaltReason::OutOfSupply,
                    ..
                }
            )),
            "a force with nothing left halts"
        );
        assert!(
            !seen
                .iter()
                .any(|event| matches!(event, StrategicEvent::ForceMoved { .. })),
            "and it does not move first"
        );
        let force = &state.forces["force.test.column"];
        assert_eq!(force.position_cell_id, BEACH);
        assert_eq!(force.composition, a_composition());
        assert_eq!(force.route.len(), 3, "it still holds its orders");
    }

    /// Readiness falls with marching, inside the band the draw may move it.
    #[test]
    fn marching_costs_readiness_and_supply() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        state
            .dispatch_force("force.test.column", ESTATE, &geography)
            .expect("the estate is one road from the beach");
        state.strategic_tick(
            &geography,
            &definitions,
            &BuildingDefinitions::new(),
            &MachineDefinitions::new(),
        );

        let force = &state.forces["force.test.column"];
        assert_eq!(force.position_cell_id, ESTATE);
        let expected = READINESS_FULL - READINESS_COST_PER_HOP;
        assert!(
            force.readiness >= expected - READINESS_DRAW_BAND
                && force.readiness <= expected + READINESS_DRAW_BAND,
            "readiness {} is outside the draw's band around {expected}",
            force.readiness
        );
        assert_eq!(force.supply, SUPPLY_ON_RAISE - SUPPLY_COST_PER_HOP);
    }

    /// A force that is not marching is not touched by an hour, and an hour with
    /// no forces in it produces no force events.
    #[test]
    fn an_hour_leaves_a_force_with_no_orders_exactly_as_it_was() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        let before = state.forces["force.test.column"].clone();

        let events = state.strategic_tick(
            &geography,
            &definitions,
            &BuildingDefinitions::new(),
            &MachineDefinitions::new(),
        );
        assert_eq!(state.forces["force.test.column"], before);
        assert!(
            !events
                .iter()
                .any(|event| !matches!(event, StrategicEvent::HourPassed { .. })),
            "a standing force is not an event"
        );
    }

    /// Progress is in the road's own minutes: a sixty-minute road takes an hour
    /// and the remainder carries forward rather than being lost or doubled.
    #[test]
    fn progress_accrues_in_minutes_and_the_remainder_carries_forward() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        state
            .dispatch_force("force.test.column", LANDING, &geography)
            .expect("the landing is two roads from the beach");

        // Beach to estate is fifteen minutes, estate to landing is thirty: an
        // hour buys both, with fifteen minutes left over and nowhere to spend
        // them, so arrival clears them.
        let events = state.strategic_tick(
            &geography,
            &definitions,
            &BuildingDefinitions::new(),
            &MachineDefinitions::new(),
        );
        let moves = events
            .iter()
            .filter(|event| matches!(event, StrategicEvent::ForceMoved { .. }))
            .count();
        assert_eq!(moves, 2, "one hour crossed both short roads");
        let force = &state.forces["force.test.column"];
        assert_eq!(force.position_cell_id, LANDING);
        assert_eq!(force.progress_minutes, 0);
    }

    // -----------------------------------------------------------------------
    // S18: what an arrival means for who holds the ground
    // -----------------------------------------------------------------------

    /// Marches the one raised force to `destination` and hands back the hour's
    /// events, arrival resolution included -- through `strategic_tick`, which
    /// is the path the game itself runs, so these tests cannot pass on a call
    /// order the island never uses.
    fn march_to(
        state: &mut ExpeditionState,
        geography: &Geography,
        force_id: &str,
        destination: &str,
    ) -> Vec<StrategicEvent> {
        state
            .dispatch_force(force_id, destination, geography)
            .expect("the destination is on the slice");
        let definitions = crate::strategy::faction::FactionDefinitions::new();
        let mut events = Vec::new();
        for _ in 0..8 {
            events.extend(state.strategic_tick(
                &geography.clone(),
                &definitions,
                &BuildingDefinitions::new(),
                &MachineDefinitions::new(),
            ));
            if state.forces[force_id].position_cell_id == destination {
                break;
            }
        }
        assert_eq!(
            state.forces[force_id].position_cell_id, destination,
            "the force never got there, so there is no arrival to resolve"
        );
        events
    }

    fn control_taken(events: &[StrategicEvent]) -> Vec<&StrategicEvent> {
        events
            .iter()
            .filter(|event| matches!(event, StrategicEvent::ControlTaken { .. }))
            .collect()
    }

    /// Rule (a): ground nobody holds is taken by the force that reaches it.
    #[test]
    fn an_arrival_on_unheld_ground_takes_it() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        raise(&mut state, &geography, BEACH);
        assert_eq!(geography.held_by(ESTATE, &state.ownership), None);

        let events = march_to(&mut state, &geography, "force.test.column", ESTATE);

        assert_eq!(
            geography.held_by(ESTATE, &state.ownership),
            Some(ConceptKey::Pirates.faction_id().as_str()),
            "the force that reached unheld ground holds it"
        );
        let taken = control_taken(&events);
        assert_eq!(taken.len(), 1, "one arrival, one handover: {events:?}");
        let StrategicEvent::ControlTaken {
            faction_id,
            cell_id,
            from,
            building_instance_ids,
            ..
        } = taken[0]
        else {
            unreachable!("filtered above")
        };
        assert_eq!(*faction_id, ConceptKey::Pirates.faction_id());
        assert_eq!(cell_id, ESTATE);
        assert_eq!(*from, None, "nobody lost ground nobody held");
        assert!(building_instance_ids.is_empty(), "nothing stands there");
    }

    /// Rule (b): a cell is not defended by being owned. It changes hands, the
    /// loser is named, and the buildings standing on it **stand as they were**
    /// -- capture versus destruction is Open (brief section 20), so the event
    /// carries their IDs and nothing else happens to them.
    #[test]
    fn an_arrival_on_an_undefended_rival_cell_takes_it_and_leaves_the_buildings_standing() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        let colonial = ConceptKey::ColonialPowers.faction_id();
        state
            .set_control(ESTATE, Some(colonial.clone()), &geography)
            .expect("the estate is on the slice");
        let standing = crate::strategy::building::BuildingInstance {
            id: "building_instance.test.post".to_owned(),
            def_id: "building.coast_watch_post".to_owned(),
            cell_id: ESTATE.to_owned(),
            faction_id: colonial.clone(),
            tier: 1,
            hp: 40,
            state: crate::strategy::building::BuildingState::Operational,
            ..Default::default()
        };
        state
            .buildings
            .insert(standing.id.clone(), standing.clone());
        raise(&mut state, &geography, BEACH);

        let events = march_to(&mut state, &geography, "force.test.column", ESTATE);

        assert_eq!(
            geography.held_by(ESTATE, &state.ownership),
            Some(ConceptKey::Pirates.faction_id().as_str())
        );
        let taken = control_taken(&events);
        assert_eq!(taken.len(), 1, "{events:?}");
        let StrategicEvent::ControlTaken {
            from,
            building_instance_ids,
            ..
        } = taken[0]
        else {
            unreachable!("filtered above")
        };
        assert_eq!(
            *from,
            Some(colonial.clone()),
            "the event names the faction that lost the cell"
        );
        assert_eq!(
            *building_instance_ids,
            vec!["building_instance.test.post".to_owned()],
            "the event carries what stands on the ground that changed hands"
        );
        assert_eq!(
            state.buildings["building_instance.test.post"], standing,
            "a building on a cell that changed hands is not captured, ruined or              reassigned: capture versus destruction is Open"
        );
    }

    /// Rule (c): a force of the holder's standing on the cell means nothing
    /// changes hands. See [`CONTESTED_ARRIVAL_RESOLVES_CONTROL`].
    #[test]
    fn an_arrival_on_a_defended_cell_changes_nothing_and_says_so() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        let colonial = ConceptKey::ColonialPowers.faction_id();
        state
            .set_control(ESTATE, Some(colonial.clone()), &geography)
            .expect("the estate is on the slice");
        state
            .raise_force(
                "force.test.garrison",
                &colonial,
                ESTATE,
                BTreeSet::new(),
                a_composition(),
                "assignment.test.hold",
                &geography,
            )
            .expect("a garrison raises on the cell its faction holds");
        raise(&mut state, &geography, BEACH);

        let events = march_to(&mut state, &geography, "force.test.column", ESTATE);

        assert_eq!(
            geography.held_by(ESTATE, &state.ownership),
            Some(colonial.as_str()),
            "a defended cell does not change hands"
        );
        assert!(
            control_taken(&events).is_empty(),
            "nothing was taken: {events:?}"
        );
        let contested: Vec<&StrategicEvent> = events
            .iter()
            .filter(|event| matches!(event, StrategicEvent::ArrivalContested { .. }))
            .collect();
        assert_eq!(contested.len(), 1, "{events:?}");
        let StrategicEvent::ArrivalContested {
            faction_id,
            cell_id,
            held_by,
            defender_force_ids,
            ..
        } = contested[0]
        else {
            unreachable!("filtered above")
        };
        assert_eq!(*faction_id, ConceptKey::Pirates.faction_id());
        assert_eq!(cell_id, ESTATE);
        assert_eq!(*held_by, colonial);
        assert_eq!(
            *defender_force_ids,
            vec![ForceId::new("force.test.garrison").expect("a well-shaped force ID")],
            "the record names who was standing there"
        );
        assert!(
            !CONTESTED_ARRIVAL_RESOLVES_CONTROL,
            "the day this constant is decided, this test is rewritten to the              rule that decides it"
        );
    }

    /// Brief section 5.9 gives the player Captain Michael's *decisions*, not an
    /// exemption from the board. Both directions, in one test: a rival's column
    /// takes an undefended cell of his, and a column of his takes unheld ground
    /// exactly as anybody else's does.
    #[test]
    fn michaels_cells_and_forces_obey_the_same_three_rules() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        let michael = ConceptKey::Michael.faction_id();
        state
            .set_control(ESTATE, Some(michael.clone()), &geography)
            .expect("the estate is on the slice");
        raise(&mut state, &geography, BEACH);

        let events = march_to(&mut state, &geography, "force.test.column", ESTATE);
        let taken = control_taken(&events);
        assert_eq!(taken.len(), 1, "{events:?}");
        let StrategicEvent::ControlTaken { from, .. } = taken[0] else {
            unreachable!("filtered above")
        };
        assert_eq!(
            *from,
            Some(michael.clone()),
            "an undefended cell of Michael's falls like any other"
        );

        state
            .raise_force(
                "force.test.michaels_column",
                &michael,
                ESTATE,
                BTreeSet::new(),
                a_composition(),
                "assignment.test.march",
                &geography,
            )
            .expect("Michael's faction raises a body like any other");
        let events = march_to(
            &mut state,
            &geography,
            "force.test.michaels_column",
            LANDING,
        );
        let taken = control_taken(&events);
        assert_eq!(taken.len(), 1, "{events:?}");
        let StrategicEvent::ControlTaken {
            faction_id,
            cell_id,
            ..
        } = taken[0]
        else {
            unreachable!("filtered above")
        };
        assert_eq!(*faction_id, michael);
        assert_eq!(
            cell_id, LANDING,
            "and his column takes ground like any other"
        );
    }

    /// Coming home is not news: an arrival on ground the force's own faction
    /// already holds resolves to nothing and emits nothing.
    #[test]
    fn an_arrival_on_ground_the_faction_already_holds_reports_nothing() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        let pirates = ConceptKey::Pirates.faction_id();
        state
            .set_control(ESTATE, Some(pirates.clone()), &geography)
            .expect("the estate is on the slice");
        raise(&mut state, &geography, BEACH);

        let events = march_to(&mut state, &geography, "force.test.column", ESTATE);

        assert!(control_taken(&events).is_empty(), "{events:?}");
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, StrategicEvent::ArrivalContested { .. })),
            "{events:?}"
        );
        assert_eq!(
            geography.held_by(ESTATE, &state.ownership),
            Some(pirates.as_str())
        );
    }
}
