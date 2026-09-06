//! S17: the hour's goals become acts. `docs/SHIP_PLAN.md` section 7, card S17.
//!
//! S4 fixed the draw sequence, S5 turned the board into a [`Goal`] list, S6
//! explained directives, S7 taught forces to march, S13 and S16 taught yards to
//! produce -- and `run_hour` still ended at `faction.current_goals = goals`.
//! This file is the line between wanting and doing, and nothing else in the
//! crate is.
//!
//! **What this file is not.** It is not a doctrine. Brief section 6's four
//! faction doctrines are Provisional and S15 owns them; nothing here reads
//! `ConceptKey`, `doctrine`, `FactionDefinition` or any per-faction table, and
//! the branch below is keyed on [`Goal`] alone. Six goals, six branches, the
//! same six for everybody on the island. The one place a concept key is read is
//! the refusal at the top of [`act_on_goals`], and it is the brief's, not a
//! doctrine: section 5.9 gives Captain Michael's faction to the **player**.
//! His faction executes routine work itself and the player directs its
//! strategic intent, so an autonomous act taken on his behalf would be the
//! simulation playing his half of the game. It takes none.
//!
//! **No new draw.** [`PURPOSES`](crate::strategy::tick::PURPOSES) is the hash
//! contract, and this card adds nothing to it. The table there already reserves
//! `strategic.economy` for construction and `strategic.force` for movement;
//! both are made and folded into `StrategicClock::draw_digest` every hour for
//! every faction whether or not anything acts, so a campaign saved before this
//! card and one saved after it make the same numbers in the same order. What
//! changes is that two of them are now *spent* -- on the identity of what the
//! hour raises -- rather than drawn and dropped.
//!
//! **Every branch answers, and a refusal is journalled.** A goal that cannot be
//! acted on emits [`StrategicEvent::ActionSkipped`] naming the goal and the
//! reason. A faction that wants to build and has nowhere to put a building says
//! so once an hour; it does not fail quietly and it does not invent somewhere.
//!
//! ## Goal has no `Supply`
//!
//! Card S17 is written as "Supply/Recover gather a per-hour trickle". `Supply`
//! is S6's [`Intent`](crate::strategy::directive::Intent) vocabulary, not S5's:
//! [`Goal`] carries six words and `Supply` is not one of them, and adding it
//! here would be a second answer to "what can a faction want". So gathering is
//! [`Goal::Recover`]'s branch, and all six goals are covered exactly once.
//!
//! ## What this card deliberately does not do
//!
//! * **It does not materialise.** A force arrives and stands, exactly as S7
//!   left it: turning an arrival into actors on the ground is B11's spawn
//!   sockets and O3's room metadata. Nothing here spawns anything.
//! * **It does not take ground.** Nothing in this file writes
//!   `ExpeditionState::ownership`. A force sent at a rival's cell marches there
//!   and stands on it; who *holds* a cell changes through
//!   `ExpeditionState::set_control`, and the lane that resolves an arrival into
//!   a change of control is the lane that materialises one. This is why the M3
//!   probe in `tests/strategic_determinism.rs` asserts what is true rather than
//!   an elimination the rules cannot yet reach; it says so in as many words.
//! * **It does not finish a building.** `advance_construction` still has no
//!   caller on a clock -- S3 wrote that deliberately and no card has moved it --
//!   so a building this file places stands `UnderConstruction` until something
//!   spends the hours. Placing is what the card asks for and placing is what it
//!   does; the gap is named here rather than papered over with a second,
//!   quieter construction clock.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::expedition::ExpeditionState;
use crate::geography::Geography;
use crate::strategy::building::{BuildingDefinitions, CELL_CAPACITY_CELLS};
use crate::strategy::faction::{ConceptKey, FACTION_ID_PREFIX};
use crate::strategy::tick::StrategicEvent;
use crate::strategy::utility::{BoardView, Goal};

/// **needs decision** -- brief section 20 leaves the resource list Open, and
/// with it every rate that produces one. How much one held cell yields a
/// faction in one hour on a [`Goal::Recover`] hour: the smallest number that is
/// not nothing, so that "a faction with ground recovers and a faction without
/// ground does not" is *visible* in a campaign without this lane pretending to
/// know an economy's shape. Tuning it is an edit to this line.
pub const GATHER_PER_HELD_CELL_PER_HOUR: u32 = 1;

/// The stockpile key a gathered trickle lands under.
///
/// `resource.open.*` is the namespace content already uses for a category the
/// brief has not decided -- `content/buildings/` authors
/// `resource.open.needs_decision` and the validator admits that namespace for
/// exactly this reason. **No resource category is invented here**: this is one
/// open key meaning "what a faction picked up", and the lane that answers brief
/// section 20 replaces it with the categories it decides on.
pub const GATHERED_RESOURCE_KEY: &str = "resource.open.gathered";

/// **needs decision** -- what a faction sends when it reaches for ground.
///
/// Content owns actor IDs and authors no faction rank-and-file yet, so this is
/// an open placeholder in the same shape [`GATHERED_RESOURCE_KEY`] is: a real
/// stable ID, in an `open` namespace, naming the decision rather than an
/// invented soldier. `ForceRecord::composition` is content's keys everywhere
/// else in the crate and stays content's keys here -- the moment an authored
/// actor kit exists, the lane that owns it replaces this constant and nothing
/// else in this file moves.
pub const FORCE_COMPOSITION_ACTOR_KEY: &str = "actor.open.needs_decision";

/// **needs decision** -- how many of them. Provisional: big enough to be over
/// [`RECOVERY_FORCE_STRENGTH_FLOOR`](crate::strategy::elimination::RECOVERY_FORCE_STRENGTH_FLOOR)
/// and so to count as a way back, small enough that nothing reads it as an
/// army. Brief section 11 owns what strength eventually means.
pub const FORCE_COMPOSITION_HEADS: u32 = 6;

/// **needs decision** -- how many bodies one faction keeps on the board at
/// once.
///
/// One, and the reason is honesty rather than balance: nothing in this crate
/// disbands, merges or destroys a force, so a faction allowed to raise one an
/// hour would fill a hundred-day save with thousands of standing armies that
/// nothing can ever remove. A faction therefore *re-uses* the body it has --
/// which is what a faction with one field army does -- and asks for another
/// only when it has none. The lane that gives a force a way to stand down owns
/// raising this number.
pub const MAX_STANDING_FORCES_PER_FACTION: usize = 1;

/// The assignment string a dispatched force carries, per goal. `assignment` is
/// the caller's own word (S7 interprets none), so the goal's own name is the
/// honest one: a reader of the save can see what the body was sent to do.
fn assignment_of(goal: Goal) -> &'static str {
    match goal {
        Goal::Expand => "goal.expand",
        Goal::Pressure => "goal.pressure",
        Goal::Recover => "goal.recover",
        Goal::Consolidate => "goal.consolidate",
        Goal::Develop => "goal.develop",
        Goal::Withdraw => "goal.withdraw",
    }
}

/// Why a goal did not become an act this hour.
///
/// A closed vocabulary rather than a free string, following
/// [`ProductionSkipReason`](crate::strategy::production::ProductionSkipReason):
/// a journal a reader can filter is worth more than a sentence, and a reason
/// that has to be *added here* cannot be invented at a call site.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionSkipReason {
    /// [`Goal::Consolidate`] and [`Goal::Withdraw`]. Card S17: these two "do
    /// nothing new yet and say so". Making held ground defensible needs a
    /// defence model and giving ground up deliberately needs a way to *lose* a
    /// cell, and this card owns neither -- so the hour records the intent it
    /// could not act on, once, rather than dropping it.
    NoActionAuthoredYet,
    /// The faction holds no cell, so there is nothing to build on, gather from
    /// or march out of.
    NoHeldCell,
    /// The registry carries no building this faction's concept may raise.
    NoCompatibleBuilding,
    /// Every held cell is full: no envelope fits in what is left of any of them
    /// ([`CELL_CAPACITY_CELLS`]).
    NoRoomToBuild,
    /// The stockpile does not cover the building's `construction_cost`. Named
    /// key, held and needed, the way S16 names a production shortfall.
    InsufficientResource { key: String, held: u32, needed: u32 },
    /// Nothing to march at: no unheld cell touches this faction's ground
    /// ([`Goal::Expand`]), or no rival's cell does ([`Goal::Pressure`]).
    NoReachableTarget,
    /// This faction is already at [`MAX_STANDING_FORCES_PER_FACTION`] and every
    /// body it has is on the road. It marches with what it has or it waits.
    NoIdleForce,
    /// The order was refused before anything moved -- an unreachable
    /// destination through the gates a faction knows (none), a malformed ID, a
    /// cell the graph lost. S7 refuses rather than accepting an order that can
    /// never arrive, and the refusal is recorded rather than swallowed.
    OrderRefused,
    /// The placement was refused before anything was raised. Every rule
    /// `place_building` enforces is checked above it, so this is the honest
    /// catch for one that changes underneath: no building stands and the hour
    /// says so.
    PlacementRefused,
}

/// **S17: one hour of one faction's acting.** Called once per faction per hour
/// from [`run_hour`](crate::strategy::tick::run_hour), inside the loop that
/// makes that faction's draws and after
/// [`advance_production`](crate::strategy::production::advance_production) --
/// the standing economy of what already exists runs first, and what the hour
/// newly commits to runs second, so a yard raised this hour is not asked to
/// produce in the hour it was founded.
///
/// `draws` is that faction's whole hour, exactly as `hour_draws` made it and
/// `run_hour` folded it. Two entries are read: `strategic.economy` for what
/// this hour builds and `strategic.force` for what it sends, per the
/// [`PURPOSES`](crate::strategy::tick::PURPOSES) table. Both are read whether
/// or not the matching branch acts -- they were made and folded before this
/// function was called -- so a faction that acts and a faction that skips leave
/// the determinism witness in exactly the same place.
///
/// The goals acted on are the ones `run_hour` has just written, in the order
/// [`choose_goals`](crate::strategy::utility::choose_goals) ranked them: an
/// hour acts on its leading intent first, and a goal that scored its way onto
/// the list is acted on even when a goal above it already spent the hour. There
/// is no budget yet, and inventing one -- an action-points economy, a
/// one-act-per-hour rule -- would be this lane deciding a pacing question no
/// card has asked.
pub(crate) fn act_on_goals(
    state: &mut ExpeditionState,
    faction_id: &str,
    geography: &Geography,
    buildings: &BuildingDefinitions,
    draws: &BTreeMap<&'static str, u64>,
) -> Vec<StrategicEvent> {
    // Brief section 5.9. The player directs this faction's strategic intent,
    // so the simulation takes no strategic act on its behalf. Its buildings
    // still produce (S16 runs before this, for every faction), its forces still
    // march (S7 runs after, for every force): what it does not do is *decide*.
    if faction_id == ConceptKey::Michael.faction_id() {
        return Vec::new();
    }
    let Some(faction) = state.factions.get(faction_id) else {
        return Vec::new();
    };
    // Brief section 16: an eliminated faction stays in the record because the
    // recovery chain and the island's reaction still read it. It does not act.
    // Its going is already journalled, once, by S10's `FactionEliminated`;
    // repeating "it did nothing again" every hour for the rest of the campaign
    // would be noise, not a record.
    if faction.eliminated {
        return Vec::new();
    }
    let goals = faction.current_goals.clone();
    if goals.is_empty() {
        return Vec::new();
    }

    let economy_draw = draws
        .get("strategic.economy")
        .copied()
        .expect("PURPOSES reserves strategic.economy and hour_draws makes every purpose");
    let force_draw = draws
        .get("strategic.force")
        .copied()
        .expect("PURPOSES reserves strategic.force and hour_draws makes every purpose");

    let day = state.campaign_day;
    let hour = state.strategic_clock.hour_of_day;
    let mut events = Vec::new();

    for goal in goals {
        // The board is read once per goal rather than once per hour: a building
        // placed by the goal above this one has taken room out of a cell, and
        // an act must see the board it is acting on. `BoardView::of` is the one
        // owner of that reading and is called here rather than copied.
        let view = BoardView::of(faction_id, state, geography);
        let outcome = match goal {
            Goal::Develop => develop(state, &view, geography, buildings, day, hour, economy_draw),
            Goal::Recover => gather(state, &view, day),
            Goal::Expand => march(
                state,
                &view,
                geography,
                goal,
                hour,
                force_draw,
                Target::Unheld,
            ),
            Goal::Pressure => march(
                state,
                &view,
                geography,
                goal,
                hour,
                force_draw,
                Target::Rival,
            ),
            Goal::Consolidate | Goal::Withdraw => Err(ActionSkipReason::NoActionAuthoredYet),
        };
        match outcome {
            Ok(mut acted) => events.append(&mut acted),
            Err(reason) => events.push(StrategicEvent::ActionSkipped {
                faction_id: faction_id.to_owned(),
                goal,
                reason,
                day,
            }),
        }
    }
    events
}

/// [`Goal::Develop`]: raise the first building this faction may raise, on the
/// first of its cells with room for it.
///
/// *First* is [`BuildingDefinitions::ids`]' `BTreeMap` order and
/// [`BoardView::held`]'s `BTreeSet` order -- total, identical on every machine,
/// and deliberately not a preference: choosing *which* building a faction wants
/// is a doctrine, and brief section 6 is Provisional.
///
/// The cost is paid from the faction's stockpile, every key or none, before the
/// building is placed. A shortfall is a skip naming the key, not a debt.
#[allow(clippy::too_many_arguments)]
fn develop(
    state: &mut ExpeditionState,
    view: &BoardView,
    geography: &Geography,
    buildings: &BuildingDefinitions,
    day: u32,
    hour: u8,
    draw: u64,
) -> Result<Vec<StrategicEvent>, ActionSkipReason> {
    if view.held.is_empty() {
        return Err(ActionSkipReason::NoHeldCell);
    }
    let concept = view
        .faction_id
        .strip_prefix(FACTION_ID_PREFIX)
        .and_then(ConceptKey::from_key)
        .ok_or(ActionSkipReason::NoCompatibleBuilding)?;
    let def_id = buildings
        .ids()
        .find(|id| {
            buildings
                .get(id)
                .is_some_and(|definition| definition.faction_compatibility.contains(&concept))
        })
        .ok_or(ActionSkipReason::NoCompatibleBuilding)?
        .to_owned();
    let definition = buildings
        .get(&def_id)
        .expect("the ID came from this registry");
    let envelope = definition.envelope_cells();
    let cost = definition.construction_cost.clone();

    let cell_id = view
        .held
        .iter()
        .find(|cell_id| {
            state
                .envelope_cells_used(cell_id, buildings)
                .saturating_add(envelope)
                <= CELL_CAPACITY_CELLS
        })
        .ok_or(ActionSkipReason::NoRoomToBuild)?
        .clone();

    // Every key is checked before any key is spent -- the same order
    // `charge_production_cost` uses, and for the same reason: a half-paid
    // building is a stockpile that went down for nothing.
    let faction = state
        .factions
        .get(&view.faction_id)
        .ok_or(ActionSkipReason::NoHeldCell)?;
    for (key, needed) in &cost {
        let held = faction.resources.get(key).copied().unwrap_or(0);
        if held < *needed {
            return Err(ActionSkipReason::InsufficientResource {
                key: key.clone(),
                held,
                needed: *needed,
            });
        }
    }

    let instance_id = building_instance_id(&view.faction_id, day, hour, draw);
    state
        .place_building(
            &instance_id,
            &def_id,
            &cell_id,
            &view.faction_id,
            geography,
            buildings,
        )
        .map_err(|_| ActionSkipReason::PlacementRefused)?;

    let faction = state
        .factions
        .get_mut(&view.faction_id)
        .expect("the same faction was just read");
    for (key, needed) in &cost {
        let entry = faction.resources.entry(key.clone()).or_insert(0);
        *entry = entry.saturating_sub(*needed);
    }

    Ok(vec![StrategicEvent::BuildingStarted {
        faction_id: view.faction_id.clone(),
        building_instance_id: instance_id,
        def_id,
        cell_id,
        day,
    }])
}

/// The ID of the building this faction founds this hour: the faction, the hour
/// it was founded in, and the hour's economy draw.
///
/// The draw is in the ID for the reason S16 put it in a machine's: the hour
/// reserves a number for construction, and spending it on the identity of what
/// was built means a change to the draw sequence shows up in the save as a
/// differently named building rather than nowhere. Two buildings founded by one
/// faction in one hour would collide, which is exactly why nothing here founds
/// two: [`Goal`] carries one `Develop`.
fn building_instance_id(faction_id: &str, day: u32, hour: u8, draw: u64) -> String {
    let concept = faction_id
        .strip_prefix(FACTION_ID_PREFIX)
        .unwrap_or(faction_id);
    format!("building_instance.s17.{concept}.d{day}h{hour}.{draw:016x}")
}

/// [`Goal::Recover`]: the per-hour trickle. Card S17's "gather a per-hour
/// trickle from held cells into the open-keyed stockpile".
///
/// Ground is what a faction gathers from, so the yield is
/// [`GATHER_PER_HELD_CELL_PER_HOUR`] per held cell -- brief section 11's
/// "strength derives from the board" at its smallest. A faction holding nothing
/// gathers nothing and says so, which is the whole of the difference between a
/// faction that is behind and one that is finished.
fn gather(
    state: &mut ExpeditionState,
    view: &BoardView,
    day: u32,
) -> Result<Vec<StrategicEvent>, ActionSkipReason> {
    if view.held.is_empty() {
        return Err(ActionSkipReason::NoHeldCell);
    }
    let amount = (view.held.len() as u32).saturating_mul(GATHER_PER_HELD_CELL_PER_HOUR);
    if amount == 0 {
        return Err(ActionSkipReason::NoHeldCell);
    }
    let faction = state
        .factions
        .get_mut(&view.faction_id)
        .ok_or(ActionSkipReason::NoHeldCell)?;
    let entry = faction
        .resources
        .entry(GATHERED_RESOURCE_KEY.to_owned())
        .or_insert(0);
    *entry = entry.saturating_add(amount);

    Ok(vec![StrategicEvent::Gathered {
        faction_id: view.faction_id.clone(),
        resource_key: GATHERED_RESOURCE_KEY.to_owned(),
        amount,
        day,
    }])
}

/// Which frontier a march is aimed at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    /// [`Goal::Expand`]: ground nobody holds.
    Unheld,
    /// [`Goal::Pressure`]: ground a rival holds.
    Rival,
}

/// [`Goal::Expand`] and [`Goal::Pressure`]: send a body at the nearest cell of
/// the chosen kind, along real roads, through
/// [`ExpeditionState::dispatch_force`].
///
/// *Nearest* is the frontier: [`BoardView`] computes the cells one road from
/// this faction's ground, so every candidate is one step away and the tie-break
/// is the `BTree*` order the view is built in. The route itself is S7's --
/// `dispatch_force` plans it with [`Geography::next_step_toward`] and refuses
/// an order it cannot plan, so a force never sets off toward somewhere it
/// cannot reach.
///
/// The body is the one this faction already has, if it is standing; otherwise
/// one is raised, up to [`MAX_STANDING_FORCES_PER_FACTION`]. A candidate the
/// marching body is already standing on is passed over -- sending a force where
/// it already is would be a departure event every hour and no movement at all.
#[allow(clippy::too_many_arguments)]
fn march(
    state: &mut ExpeditionState,
    view: &BoardView,
    geography: &Geography,
    goal: Goal,
    hour: u8,
    draw: u64,
    target: Target,
) -> Result<Vec<StrategicEvent>, ActionSkipReason> {
    if view.held.is_empty() {
        return Err(ActionSkipReason::NoHeldCell);
    }
    let candidates: Vec<&str> = match target {
        Target::Unheld => view.open_frontier.iter().map(String::as_str).collect(),
        Target::Rival => view.contested_frontier.keys().map(String::as_str).collect(),
    };
    if candidates.is_empty() {
        return Err(ActionSkipReason::NoReachableTarget);
    }

    // The body: this faction's first force with no road left, in force-ID
    // order. A force that is still marching is not re-tasked -- its orders
    // stand until it arrives or halts, which is what makes `assignment` mean
    // anything.
    let standing: Vec<(String, String)> = state
        .forces
        .values()
        .filter(|force| force.faction_id == view.faction_id)
        .map(|force| (force.id.to_string(), force.position_cell_id.clone()))
        .collect();
    let idle: Option<(String, String)> = state
        .forces
        .values()
        .filter(|force| force.faction_id == view.faction_id && !force.is_marching())
        .map(|force| (force.id.to_string(), force.position_cell_id.clone()))
        .next();

    // Where it would set off from: where the standing body is, or the first
    // cell this faction holds if one has to be raised.
    let from_cell_id = match &idle {
        Some((_, position)) => position.clone(),
        None => {
            if standing.len() >= MAX_STANDING_FORCES_PER_FACTION {
                return Err(ActionSkipReason::NoIdleForce);
            }
            view.held
                .iter()
                .next()
                .expect("the held set was checked non-empty above")
                .clone()
        }
    };
    // The destination is chosen **before** anything is raised, so an hour with
    // nowhere to go leaves the board exactly as it found it rather than
    // standing up a body it then cannot send.
    let destination = candidates
        .into_iter()
        .find(|cell_id| *cell_id != from_cell_id)
        .ok_or(ActionSkipReason::NoReachableTarget)?
        .to_owned();

    let force_id = match idle {
        Some((id, _)) => id,
        None => {
            let id = force_id(&view.faction_id, goal, state.campaign_day, hour, draw);
            state
                .raise_force(
                    &id,
                    &view.faction_id,
                    &from_cell_id,
                    Default::default(),
                    BTreeMap::from([(
                        FORCE_COMPOSITION_ACTOR_KEY.to_owned(),
                        FORCE_COMPOSITION_HEADS,
                    )]),
                    assignment_of(goal),
                    geography,
                )
                .map_err(|_| ActionSkipReason::OrderRefused)?;
            id
        }
    };

    let departed = state
        .dispatch_force(&force_id, &destination, geography)
        .map_err(|_| ActionSkipReason::OrderRefused)?;
    if let Some(force) = state.forces.get_mut(&force_id) {
        force.assignment = assignment_of(goal).to_owned();
    }
    Ok(vec![departed])
}

/// The ID of the body this faction raises this hour. Same shape and same
/// reasoning as [`building_instance_id`], plus the goal it was raised for, so
/// that an hour reaching for open ground and an hour reaching at a rival cannot
/// name the same force.
fn force_id(faction_id: &str, goal: Goal, day: u32, hour: u8, draw: u64) -> String {
    let concept = faction_id
        .strip_prefix(FACTION_ID_PREFIX)
        .unwrap_or(faction_id);
    let purpose = assignment_of(goal)
        .strip_prefix("goal.")
        .unwrap_or("unknown");
    format!("force.s17.{concept}.{purpose}.d{day}h{hour}.{draw:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geography::Geography;
    use crate::strategy::building::{
        BuildingDefinition, BuildingState, CaptureRules, RuinState, TierState,
    };
    use crate::strategy::faction::FactionState;
    use crate::strategy::production::MachineDefinitions;
    use crate::strategy::tick::hour_draws;
    use std::collections::BTreeSet;

    const BEACH: &str = "world.cell.black_beach";
    const ESTATE: &str = "world.cell.damaged_estate";

    fn pirates() -> String {
        ConceptKey::Pirates.faction_id()
    }

    /// A campaign with the slice under it and every concept on the board.
    fn a_campaign() -> ExpeditionState {
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        for concept in ConceptKey::ALL {
            state
                .factions
                .insert(concept.faction_id(), FactionState::new());
        }
        state
    }

    /// A record the test namespace owns, compatible with one concept, cheap
    /// enough to fit a cell twice and priced in one open key.
    fn a_definition(
        id: &str,
        concept: ConceptKey,
        cost: BTreeMap<String, u32>,
    ) -> BuildingDefinition {
        BuildingDefinition {
            id: id.to_owned(),
            faction_compatibility: BTreeSet::from([concept]),
            function: "a test record".into(),
            footprint_cells: 4,
            clearance_cells: 2,
            construction_cost: cost,
            tier_states: vec![TierState {
                tier: 1,
                construction_hours: 4,
                hit_points: 10,
                ..TierState::default()
            }],
            capture_rules: CaptureRules::DestroyOnly,
            ruin_state: RuinState::ClearsCompletely,
            ..BuildingDefinition::default()
        }
    }

    fn a_registry(cost: BTreeMap<String, u32>) -> BuildingDefinitions {
        let mut definitions = BuildingDefinitions::new();
        definitions
            .insert(a_definition(
                "building.test.pirate_post",
                ConceptKey::Pirates,
                cost,
            ))
            .expect("the fixture record loads");
        definitions
    }

    fn draws_for(state: &ExpeditionState, faction_id: &str) -> BTreeMap<&'static str, u64> {
        hour_draws(
            state.rng_seed,
            state.campaign_day,
            state.strategic_clock.hour_of_day,
            faction_id,
        )
    }

    fn act(
        state: &mut ExpeditionState,
        faction_id: &str,
        geography: &Geography,
        buildings: &BuildingDefinitions,
    ) -> Vec<StrategicEvent> {
        let draws = draws_for(state, faction_id);
        act_on_goals(state, faction_id, geography, buildings, &draws)
    }

    fn with_goals(state: &mut ExpeditionState, faction_id: &str, goals: Vec<Goal>) {
        state
            .factions
            .get_mut(faction_id)
            .expect("the campaign carries this faction")
            .current_goals = goals;
    }

    /// Brief section 5.9: the player directs Captain Michael's faction, so the
    /// simulation never acts for it. Every goal, on a board where every other
    /// faction would act, and not one event.
    #[test]
    fn michaels_faction_takes_no_autonomous_action() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let michael = ConceptKey::Michael.faction_id();
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(michael.clone()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &michael, Goal::ALL.to_vec());

        let events = act(&mut state, &michael, &geography, &buildings);
        assert!(
            events.is_empty(),
            "the player directs this faction; the simulation acted for it: {events:?}"
        );
        assert!(state.buildings.is_empty());
        assert!(state.forces.is_empty());
        assert!(state.factions[&michael].resources.is_empty());
    }

    /// Develop raises the first compatible record on the first held cell with
    /// room, pays for it, and reports it.
    #[test]
    fn develop_places_a_compatible_building_and_pays_for_it() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::from([(
            "resource.open.gathered".to_owned(),
            2u32,
        )]));
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        state
            .factions
            .get_mut(&pirates())
            .expect("the campaign carries this faction")
            .resources
            .insert(GATHERED_RESOURCE_KEY.to_owned(), 5);
        with_goals(&mut state, &pirates(), vec![Goal::Develop]);

        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert_eq!(events.len(), 1, "{events:?}");
        let StrategicEvent::BuildingStarted {
            faction_id,
            building_instance_id,
            def_id,
            cell_id,
            day,
        } = &events[0]
        else {
            panic!("Develop must report the building it started: {events:?}");
        };
        assert_eq!(faction_id, &pirates());
        assert_eq!(def_id, "building.test.pirate_post");
        assert_eq!(cell_id, BEACH);
        assert_eq!(*day, state.campaign_day);
        assert_eq!(
            state.buildings[building_instance_id].state,
            BuildingState::UnderConstruction,
            "a building is placed, not finished"
        );
        assert_eq!(
            state.factions[&pirates()].resources[GATHERED_RESOURCE_KEY],
            3,
            "the construction cost came out of the stockpile"
        );
    }

    /// The three refusals Develop owns, each journalled rather than silent, and
    /// none of them leaving anything on the board.
    #[test]
    fn develop_skips_with_a_reason_and_builds_nothing() {
        let geography = Geography::black_beach_vertical_slice();
        let priced = a_registry(BTreeMap::from([(
            "resource.open.gathered".to_owned(),
            2u32,
        )]));
        let free = a_registry(BTreeMap::new());

        // No held cell.
        let mut state = a_campaign();
        with_goals(&mut state, &pirates(), vec![Goal::Develop]);
        let events = act(&mut state, &pirates(), &geography, &free);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, goal, .. }]
                    if *reason == ActionSkipReason::NoHeldCell && *goal == Goal::Develop
            ),
            "{events:?}"
        );

        // No compatible record: the registry carries one, for another concept.
        let mut state = a_campaign();
        let elves = ConceptKey::Elves.faction_id();
        state
            .set_control(BEACH, Some(elves.clone()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &elves, vec![Goal::Develop]);
        let events = act(&mut state, &elves, &geography, &free);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::NoCompatibleBuilding
            ),
            "{events:?}"
        );

        // Short of the cost.
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &pirates(), vec![Goal::Develop]);
        let events = act(&mut state, &pirates(), &geography, &priced);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::InsufficientResource {
                        key: "resource.open.gathered".into(),
                        held: 0,
                        needed: 2,
                    }
            ),
            "{events:?}"
        );
        assert!(state.buildings.is_empty(), "a refused hour builds nothing");

        // No room: the cell fills at two of these envelopes (six cells each).
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &pirates(), vec![Goal::Develop]);
        for _ in 0..2 {
            let events = act(&mut state, &pirates(), &geography, &free);
            assert!(matches!(events[0], StrategicEvent::BuildingStarted { .. }));
            state.strategic_clock.hour_of_day += 1;
        }
        let events = act(&mut state, &pirates(), &geography, &free);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::NoRoomToBuild
            ),
            "{events:?}"
        );
        assert_eq!(state.buildings.len(), 2, "a full cell takes no third");
    }

    /// The trickle is per held cell, lands under the one open key, and a
    /// faction with no ground gathers nothing and says so.
    #[test]
    fn recover_gathers_a_trickle_from_held_ground_only() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let mut state = a_campaign();
        with_goals(&mut state, &pirates(), vec![Goal::Recover]);

        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::NoHeldCell
            ),
            "{events:?}"
        );

        for cell in [BEACH, ESTATE] {
            state
                .set_control(cell, Some(pirates()), &geography)
                .expect("both are real cells");
        }
        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert_eq!(
            events,
            vec![StrategicEvent::Gathered {
                faction_id: pirates(),
                resource_key: GATHERED_RESOURCE_KEY.into(),
                amount: 2 * GATHER_PER_HELD_CELL_PER_HOUR,
                day: state.campaign_day,
            }]
        );
        assert_eq!(
            state.factions[&pirates()].resources[GATHERED_RESOURCE_KEY],
            2 * GATHER_PER_HELD_CELL_PER_HOUR
        );
    }

    /// Expand raises one body and sends it at unheld ground along a real route.
    /// Pressure sends the same body at a rival's. Neither takes the ground:
    /// materialisation is B11's and nothing here writes ownership.
    #[test]
    fn expand_and_pressure_dispatch_a_force_along_a_real_route() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &pirates(), vec![Goal::Expand]);

        let events = act(&mut state, &pirates(), &geography, &buildings);
        let StrategicEvent::ForceDeparted {
            force_id,
            faction_id,
            from_cell_id,
            destination_cell_id,
            steps,
        } = &events[0]
        else {
            panic!("Expand must dispatch: {events:?}");
        };
        assert_eq!(faction_id, &pirates());
        assert_eq!(from_cell_id, BEACH);
        assert!(*steps >= 1, "a dispatch walks at least one real road");
        assert_eq!(
            geography.held_by(destination_cell_id, &state.ownership),
            None,
            "Expand reaches for ground nobody holds"
        );
        let record = &state.forces[force_id.as_str()];
        assert_eq!(record.assignment, "goal.expand");
        assert_eq!(
            record.composition[FORCE_COMPOSITION_ACTOR_KEY],
            FORCE_COMPOSITION_HEADS
        );
        assert_eq!(
            state.ownership.get(destination_cell_id),
            None,
            "an order given is not ground taken"
        );

        // A rival on the far cell, so Pressure has somewhere to reach.
        state
            .set_control(ESTATE, Some(ConceptKey::Elves.faction_id()), &geography)
            .expect("the estate is a real cell");

        // Still marching: the body is not re-tasked and nothing is raised
        // beside it.
        let raised = state.forces.len();
        with_goals(&mut state, &pirates(), vec![Goal::Pressure]);
        state.strategic_clock.hour_of_day += 1;
        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::NoIdleForce
            ),
            "{events:?}"
        );
        assert_eq!(state.forces.len(), raised, "one body, re-used");

        // Bring it home, and the same body is sent at the rival's ground.
        let force = state
            .forces
            .get_mut(force_id.as_str())
            .expect("the force was just raised");
        force.route.clear();
        force.destination_cell_id = None;
        force.position_cell_id = BEACH.to_owned();
        state.strategic_clock.hour_of_day += 1;
        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ForceDeparted { destination_cell_id, .. }]
                    if geography.held_by(destination_cell_id, &state.ownership)
                        == Some(ConceptKey::Elves.faction_id().as_str())
            ),
            "Pressure reaches at a rival's ground: {events:?}"
        );
    }

    /// A faction with no frontier of the kind it wants says so.
    #[test]
    fn a_march_with_nowhere_to_go_says_so() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &pirates(), vec![Goal::Pressure]);

        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert!(
            matches!(
                &events[..],
                [StrategicEvent::ActionSkipped { reason, .. }]
                    if *reason == ActionSkipReason::NoReachableTarget
            ),
            "nobody else holds anything, so there is nothing to push on: {events:?}"
        );
    }

    /// The two goals this card does not act on are recorded, not dropped.
    #[test]
    fn consolidate_and_withdraw_journal_a_skip_rather_than_going_quiet() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(
            &mut state,
            &pirates(),
            vec![Goal::Consolidate, Goal::Withdraw],
        );

        let events = act(&mut state, &pirates(), &geography, &buildings);
        assert_eq!(
            events,
            vec![
                StrategicEvent::ActionSkipped {
                    faction_id: pirates(),
                    goal: Goal::Consolidate,
                    reason: ActionSkipReason::NoActionAuthoredYet,
                    day: state.campaign_day,
                },
                StrategicEvent::ActionSkipped {
                    faction_id: pirates(),
                    goal: Goal::Withdraw,
                    reason: ActionSkipReason::NoActionAuthoredYet,
                    day: state.campaign_day,
                },
            ]
        );
    }

    /// An eliminated faction is still in the record and still draws (S4), and
    /// it does not act. Brief section 16.
    #[test]
    fn an_eliminated_faction_does_not_act() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        with_goals(&mut state, &pirates(), vec![Goal::Recover, Goal::Develop]);
        state
            .factions
            .get_mut(&pirates())
            .expect("the campaign carries this faction")
            .eliminated = true;

        assert!(act(&mut state, &pirates(), &geography, &buildings).is_empty());
        assert!(state.buildings.is_empty());
        assert!(state.factions[&pirates()].resources.is_empty());
    }

    /// The hour acts through the one entry point the game uses, and the acts it
    /// takes are in the hour's own events -- so `run_hour` calling this is a
    /// fact the journal carries rather than a claim about the code.
    #[test]
    fn the_hour_carries_the_acts_it_took() {
        let geography = Geography::black_beach_vertical_slice();
        let buildings = a_registry(BTreeMap::new());
        let factions = crate::strategy::faction::FactionDefinitions::new();
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");

        let mut acted = false;
        for _ in 0..24 {
            let events = state.strategic_tick(
                &geography,
                &factions,
                &buildings,
                &MachineDefinitions::new(),
            );
            acted |= events.iter().any(|event| {
                matches!(
                    event,
                    StrategicEvent::BuildingStarted { .. }
                        | StrategicEvent::Gathered { .. }
                        | StrategicEvent::ForceDeparted { .. }
                        | StrategicEvent::ActionSkipped { .. }
                )
            });
        }
        assert!(
            acted,
            "a day of a faction holding ground produced no act at all"
        );
    }
}
