//! S10: the recovery chain, and the one door out of the simulation.
//! `docs/SHIP_PLAN.md` section 7, card S10.
//!
//! Brief section 16 is short and absolute: *"A faction should be eliminated
//! only when it has no viable recovery chain"*, and *"the eliminated faction
//! does not automatically respawn"*. Everything in this file follows from
//! reading those two sentences as a rule rather than as flavour.
//!
//! **The chain is the check, not a health bar.** [`RecoveryLink`] has one
//! variant per bullet of section 16's list, in the brief's order, and
//! [`recovery_chain`] returns the ones a faction still holds. Elimination is
//! the single question "is that list empty", asked by
//! [`ExpeditionState::eliminate_if_exhausted`]. There is no threshold, no
//! score, and no "mostly dead": a faction with one link left is alive, and a
//! faction with one link left is exactly what the card's second Done-when
//! test asserts survives.
//!
//! **Having nothing is not the same as having lost everything.** A fresh
//! campaign authors no ownership, so on hour one every faction on the board
//! holds no cell, no building and no force. Calling that exhaustion would
//! eliminate the whole island before it started -- which is the same mistake
//! `recompute_strategic_state` refuses when it calls an unclaimed board
//! `Contesting` for everyone. So elimination requires a *loss*: a faction is
//! a candidate only once it has held at least one link
//! ([`FactionState::has_ever_held`]), and the link it lost is named by
//! [`StrategicEvent::RecoveryLinkLost`] as it goes
//! ([`FactionState::recovery_links_held`] is the previous reading the loss is
//! measured against). The 2,400-hour determinism harness is the standing
//! proof: its six factions hold nothing on hour one and are all still on the
//! board on hour 2,400.
//!
//! **Two links have no state to read yet, and are not faked.**
//! [`RecoveryLink::RemainingPopulation`] and
//! [`RecoveryLink::AuthoredRecoveryEvent`] are in the enum because they are in
//! the brief, and [`recovery_chain`] never reports them because nothing in the
//! crate models a population count or an authored recovery event. Reporting
//! them anyway -- always present -- would make elimination unreachable for
//! everyone; inventing a proxy would be this file answering a Provisional
//! design question. They are `blocked` and say so on the variant.
//!
//! **Cthulhu is Open and is therefore untouchable here.** Brief section 20
//! lists "Cthulhu early-elimination rules" among the undecided items, so
//! [`recovery_chain`] appends [`RecoveryLink::NeedsDecision`] to that
//! faction's chain and the chain is never empty. Nothing in this file can
//! remove that faction from the board until the decision lands.
//!
//! **"What happens if all ordinary factions collapse" is Open too** (brief
//! section 20, same list), and this file deliberately emits nothing special
//! for it. Six eliminations are six [`StrategicEvent::FactionEliminated`]
//! events and no seventh event of any kind. There is no last-faction branch to
//! be found here and later mistaken for the decision.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::expedition::ExpeditionState;
use crate::geography::Geography;
use crate::strategy::building::{BuildingDefinitions, CELL_CAPACITY_CELLS, CaptureRules};
use crate::strategy::faction::{ConceptKey, StrategicState};
use crate::strategy::tick::StrategicEvent;

/// Total stock, summed over whatever open resource keys a faction carries, at
/// or above which its reserve is a link back.
///
/// **needs decision** -- brief section 20 leaves the exact resource list Open,
/// so there is no category here to weigh and no meaningful quantity to name.
/// One is the only honest placeholder: it means "anything at all in the
/// stores", which is a fact about the save rather than a guess about an
/// economy nobody has specified. When the resource list lands, this becomes a
/// per-category question and the constant is replaced, not tuned.
pub const RESOURCE_RESERVE_FLOOR: u32 = 1;

/// Aggregate strength at or above which a surviving body of a faction's own
/// counts as a mobile recovery force.
///
/// **needs decision** -- `ForceRecord::strength` is a head count today and
/// brief section 11 owns making equipment and experience count. One means "a
/// force that still has anybody in it", which `raise_force` already treats as
/// the difference between a force and nothing.
pub const RECOVERY_FORCE_STRENGTH_FLOOR: u32 = 1;

/// Trust, in one faction's live half of a pairwise relationship, at or above
/// which a *living* neighbour counts as an allied refuge.
///
/// **needs decision** -- brief section 7's pairwise pressures are Provisional
/// and brief section 20 leaves the treaty vocabulary Open, so there is no
/// approved band on this axis to point at. One means "some trust has actually
/// formed", which is the only reading that does not treat
/// `Relationship::default()` -- zero throughout, the brief's "no history yet"
/// -- as a friendship. It is deliberately a *positive* floor: a faction is not
/// sheltered by a neighbour it has no history with.
pub const ALLY_REFUGE_TRUST_FLOOR: i16 = 1;

/// One way back, from brief section 16's list of recovery paths.
///
/// The variants are that list, in the brief's order and one per bullet, so the
/// design document and the code can be diffed by eye. They carry no payload on
/// purpose: the question this type answers is *which kinds of way back exist*,
/// and a link that has just been lost has no instance ID left to name, so
/// [`StrategicEvent::RecoveryLinkLost`] could not carry one honestly.
///
/// `Ord` follows declaration order, so a `BTreeSet<RecoveryLink>` iterates in
/// the brief's order -- which is what makes the saved reading in
/// [`FactionState::recovery_links_held`] and the events derived from it
/// deterministic without a sort anywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryLink {
    /// "Operational core building." A building of this faction's that is
    /// standing well enough to work (`BuildingInstance::is_working`).
    ///
    /// *Core* is not defined anywhere in the brief and no field distinguishes
    /// a core building from any other, so this reads every working building of
    /// theirs. Narrowing it would require inventing the distinction.
    OperationalCoreBuilding,
    /// "Remaining population."
    ///
    /// **blocked: needs decision** -- brief section 20 leaves the population
    /// model Open (the five established factions' populations are Provisional
    /// in section 4, and Michael's people are S12's recruitment, never a
    /// building timer). Nothing in the crate counts a faction's people, so
    /// [`recovery_chain`] never reports this link. The variant exists because
    /// a link that cannot be *named* cannot be *filled in* by the lane that
    /// gets the decision.
    RemainingPopulation,
    /// "Worker production or recruitment." A working building of this
    /// faction's whose record authors any production rule or any
    /// recruitment support.
    ///
    /// A building whose definition is not in the registry passed to
    /// [`recovery_chain`] counts: the registry is the owner of what a building
    /// makes, and a save cannot be read as proof that a building makes
    /// nothing. Reading an unlookupable building as productive can only ever
    /// *delay* an elimination, which is the direction section 16 errs in.
    WorkerProductionOrRecruitment,
    /// "Resource reserve." Stock summed over every open resource key the
    /// faction carries, at or above [`RESOURCE_RESERVE_FLOOR`]. Open keys, so
    /// the sum names no category (brief section 20).
    ResourceReserve,
    /// "Controlled settlement." A cell this faction effectively holds --
    /// `Geography::held_by`, which is the save's override where there is one
    /// and the authored owner otherwise.
    ControlledSettlement,
    /// "Mobile recovery force." A force of this faction's whose strength is at
    /// or above [`RECOVERY_FORCE_STRENGTH_FLOOR`], wherever it stands. Brief
    /// section 10's aggregate does not have to be near home to be a way back.
    MobileRecoveryForce,
    /// "Allied refuge." A *living* faction the save carries toward which this
    /// faction's trust is at or above [`ALLY_REFUGE_TRUST_FLOOR`].
    ///
    /// Living is load-bearing: an eliminated neighbour shelters nobody, so the
    /// last two factions on the board cannot hold each other up after one of
    /// them is gone.
    AlliedRefuge,
    /// "Valid construction site." A held cell with envelope capacity left
    /// (`ExpeditionState::envelope_cells_used` below
    /// [`CELL_CAPACITY_CELLS`]) -- ground this faction could still build on.
    ///
    /// It is deliberately *not* narrowed to "a site big enough for something
    /// in this faction's building kit": the kit lives on the authored
    /// `FactionDefinition` and the smallest envelope in it is a question for
    /// the lane that spends hours building, not for the lane that asks whether
    /// anything is left.
    ValidConstructionSite,
    /// "Authored recovery event."
    ///
    /// **blocked: needs content** -- brief section 16 lists it, and no
    /// authored recovery events exist: `content/` carries no such record and
    /// no registry of them reaches this crate. [`recovery_chain`] never
    /// reports it. The lane that authors them fills this arm in.
    AuthoredRecoveryEvent,
    /// Not a link from the brief's list: the standing refusal to eliminate a
    /// faction whose elimination rules are undecided.
    ///
    /// **blocked: needs decision** -- brief section 20, "Cthulhu
    /// early-elimination rules". [`recovery_chain`] appends this to
    /// [`ConceptKey::Cthulhu`]'s chain and to no other faction's, so that
    /// chain is never empty and [`ExpeditionState::eliminate_if_exhausted`]
    /// can never fire for it. It is removed by the lane that receives the
    /// decision, and removing it is the whole of that change.
    NeedsDecision,
}

impl RecoveryLink {
    /// Every link, in brief section 16's order, with `NeedsDecision` last
    /// because it is not one of the brief's.
    pub const ALL: [RecoveryLink; 10] = [
        RecoveryLink::OperationalCoreBuilding,
        RecoveryLink::RemainingPopulation,
        RecoveryLink::WorkerProductionOrRecruitment,
        RecoveryLink::ResourceReserve,
        RecoveryLink::ControlledSettlement,
        RecoveryLink::MobileRecoveryForce,
        RecoveryLink::AlliedRefuge,
        RecoveryLink::ValidConstructionSite,
        RecoveryLink::AuthoredRecoveryEvent,
        RecoveryLink::NeedsDecision,
    ];
}

/// The ways back `faction_id` still has, in brief section 16's order.
///
/// Pure: it reads the board and answers, and nothing here writes. An empty
/// result is "no viable recovery chain" in the brief's words -- but it is not
/// on its own a reason to eliminate anybody, because a faction that never held
/// anything has an empty chain too. [`ExpeditionState::eliminate_if_exhausted`]
/// is where that distinction lives.
///
/// The card names the third argument `definitions`; it is two registries here,
/// because two different owners answer two of the links. Held ground is the
/// graph's answer (`Geography::held_by` composes the save's override with the
/// authored owner, and is the one place that composition lives), and buildable
/// room is the building registry's. Reading ownership out of
/// `ExpeditionState::ownership` alone would be a second, quieter answer to
/// "who holds this cell", which is exactly the drift `held_by` exists to stop.
pub fn recovery_chain(
    state: &ExpeditionState,
    faction_id: &str,
    geography: &Geography,
    definitions: &BuildingDefinitions,
) -> Vec<RecoveryLink> {
    let Some(faction) = state.factions.get(faction_id) else {
        return Vec::new();
    };
    let mut chain = Vec::new();

    let working: Vec<&crate::strategy::building::BuildingInstance> = state
        .buildings
        .values()
        .filter(|building| building.faction_id == faction_id && building.is_working())
        .collect();

    if !working.is_empty() {
        chain.push(RecoveryLink::OperationalCoreBuilding);
    }

    // RemainingPopulation: blocked, see the variant. Nothing counts people.

    let produces = working.iter().any(|building| {
        match definitions.get(&building.def_id) {
            Some(definition) => {
                !definition.production.is_empty() || !definition.recruitment_support.is_empty()
            }
            // The registry owns what a building makes; a missing record is
            // ignorance, not a proof of idleness.
            None => true,
        }
    });
    if produces {
        chain.push(RecoveryLink::WorkerProductionOrRecruitment);
    }

    let stock = faction
        .resources
        .values()
        .fold(0u32, |total, amount| total.saturating_add(*amount));
    if stock >= RESOURCE_RESERVE_FLOOR {
        chain.push(RecoveryLink::ResourceReserve);
    }

    let held: Vec<&str> = geography
        .all_location_ids()
        .filter(|cell_id| geography.held_by(cell_id, &state.ownership) == Some(faction_id))
        .collect();
    if !held.is_empty() {
        chain.push(RecoveryLink::ControlledSettlement);
    }

    if state.forces.values().any(|force| {
        force.faction_id == faction_id && force.strength >= RECOVERY_FORCE_STRENGTH_FLOOR
    }) {
        chain.push(RecoveryLink::MobileRecoveryForce);
    }

    let sheltered = faction
        .relationships
        .iter()
        .any(|(other_id, relationship)| {
            other_id != faction_id
                && relationship.trust >= ALLY_REFUGE_TRUST_FLOOR
                && state
                    .factions
                    .get(other_id)
                    .is_some_and(|other| !other.eliminated)
        });
    if sheltered {
        chain.push(RecoveryLink::AlliedRefuge);
    }

    if held
        .iter()
        .any(|cell_id| state.envelope_cells_used(cell_id, definitions) < CELL_CAPACITY_CELLS)
    {
        chain.push(RecoveryLink::ValidConstructionSite);
    }

    // AuthoredRecoveryEvent: blocked, see the variant. No such content exists.

    if faction_id == ConceptKey::Cthulhu.faction_id() {
        chain.push(RecoveryLink::NeedsDecision);
    }

    chain
}

impl ExpeditionState {
    /// Ask whether `faction_id` has run out of ways back, and remove it if it
    /// has. **The one caller of [`ExpeditionState::eliminate_faction`] that
    /// decides anything.**
    ///
    /// Three things happen here, in this order:
    ///
    /// 1. the chain is read;
    /// 2. the reading is compared with the one the save carries, and every
    ///    link that was there and is not now is reported as
    ///    [`StrategicEvent::RecoveryLinkLost`], in brief order;
    /// 3. if the chain is empty *and* this faction has ever held a link, it is
    ///    eliminated.
    ///
    /// Step 3's second condition is the "never held anything yet" rule stated
    /// once: an unclaimed board is not exhaustion. A faction that has never
    /// held a cell, raised a force, built anything, stocked anything or made a
    /// friend has [`FactionState::has_ever_held`] `false` and is not
    /// eliminated by the absence of those things -- it is eliminated only by
    /// having *lost* them, which is precisely what the flag records.
    ///
    /// Idempotent: called twice in a row it reports nothing the second time,
    /// because the saved reading now matches the board.
    pub fn eliminate_if_exhausted(
        &mut self,
        faction_id: &str,
        geography: &Geography,
        definitions: &BuildingDefinitions,
    ) -> Vec<StrategicEvent> {
        if !self.factions.contains_key(faction_id) {
            return Vec::new();
        }
        let chain: BTreeSet<RecoveryLink> =
            recovery_chain(self, faction_id, geography, definitions)
                .into_iter()
                .collect();

        let faction = self
            .factions
            .get_mut(faction_id)
            .expect("the faction was found above");
        let lost: Vec<RecoveryLink> = faction
            .recovery_links_held
            .difference(&chain)
            .copied()
            .collect();
        faction.recovery_links_held = chain.clone();
        faction.has_ever_held |= !chain.is_empty();
        let exhausted = chain.is_empty() && faction.has_ever_held && !faction.eliminated;

        let mut events: Vec<StrategicEvent> = lost
            .into_iter()
            .map(|link| StrategicEvent::RecoveryLinkLost {
                faction_id: faction_id.to_owned(),
                link,
            })
            .collect();
        if exhausted {
            events.extend(self.eliminate_faction(faction_id, geography, definitions));
        }
        events
    }

    /// Take `faction_id` off the board, whatever the chain says.
    ///
    /// The consequences are brief section 16's "After elimination" list, in
    /// the card's order, and they all happen or none of them do -- every step
    /// below is infallible over data already validated, so there is no partial
    /// elimination to unwind:
    ///
    /// * `eliminated` is set, which is what stops strategic scheduling: S5's
    ///   `recompute_strategic_state` reads the flag and answers
    ///   [`StrategicState::Desperate`] for the rest of the campaign;
    /// * `current_goals` is cleared and the state set `Desperate` now, so the
    ///   save is right immediately rather than at the next hour;
    /// * every building of theirs transitions **per its own record**: a
    ///   `DestroyOnly` one is ruined, and a `Capturable` one is left exactly
    ///   where it stands as an unowned shell for whoever walks in. No captor is
    ///   invented -- `capture_building` is the only way a building changes
    ///   hands and it needs an attacker;
    /// * every cell they hold is released;
    /// * every force of theirs halts where it stands, whole (brief section 10:
    ///   an aggregate that cannot march does not evaporate);
    /// * [`StrategicEvent::FactionEliminated`] is emitted.
    ///
    /// **No respawn.** Calling this on an eliminated faction is a no-op that
    /// returns no events, and nothing anywhere in the crate sets `eliminated`
    /// back to `false` -- there is no revive, no reset, and no clear. Brief
    /// section 16: "the eliminated faction does not automatically respawn",
    /// and "any rare story exception must be explicit", which a method that
    /// does not exist cannot be mistaken for.
    pub fn eliminate_faction(
        &mut self,
        faction_id: &str,
        geography: &Geography,
        definitions: &BuildingDefinitions,
    ) -> Vec<StrategicEvent> {
        let Some(faction) = self.factions.get_mut(faction_id) else {
            return Vec::new();
        };
        if faction.eliminated {
            return Vec::new();
        }
        faction.eliminated = true;
        faction.current_goals.clear();
        faction.strategic_state = StrategicState::Desperate;

        // Buildings, each by its own record. `ruin_building` refuses a wreck,
        // so the already-ruined are skipped rather than re-ruined; a
        // `Capturable` one is deliberately untouched.
        let theirs: Vec<String> = self
            .buildings
            .values()
            .filter(|building| building.faction_id == faction_id)
            .map(|building| building.id.clone())
            .collect();
        for instance_id in theirs {
            let ruin = match self.buildings.get(&instance_id) {
                Some(building) => match definitions.get(&building.def_id) {
                    Some(definition) => definition.capture_rules == CaptureRules::DestroyOnly,
                    // Unlookupable: `damage_building` reads a building nobody
                    // said was capturable as not captured, and this agrees
                    // with it rather than inventing a second reading.
                    None => true,
                },
                None => continue,
            };
            if ruin {
                let _ = self.ruin_building(&instance_id);
            }
        }

        // Territory, back to whoever the graph says holds it -- which is
        // nobody, everywhere content authors no owner. `set_control` is S2's
        // single writer of `ownership` and this does not go around it.
        let held: Vec<String> = geography
            .all_location_ids()
            .filter(|cell_id| geography.held_by(cell_id, &self.ownership) == Some(faction_id))
            .map(str::to_owned)
            .collect();
        for cell_id in held {
            self.set_control(&cell_id, None, geography)
                .expect("the cell IDs came from the graph this call is passed");
        }

        // Forces halt where they stand, keeping composition, supply and
        // readiness. Nothing is disbanded: brief section 16 lets survivors
        // flee, surrender, defect or disappear "according to content", and
        // there is no content saying which, so this lane does not choose.
        for force in self.forces.values_mut() {
            if force.faction_id != faction_id {
                continue;
            }
            force.route.clear();
            force.destination_cell_id = None;
            force.progress_minutes = 0;
        }

        // The reading is now whatever the board says after all of that, so a
        // later call reports nothing stale.
        let chain: BTreeSet<RecoveryLink> =
            recovery_chain(self, faction_id, geography, definitions)
                .into_iter()
                .collect();
        let day = self.campaign_day;
        let faction = self
            .factions
            .get_mut(faction_id)
            .expect("the faction was found above");
        faction.recovery_links_held = chain;

        vec![StrategicEvent::FactionEliminated {
            faction_id: faction_id.to_owned(),
            day,
        }]
    }

    /// The hourly sweep: ask [`ExpeditionState::eliminate_if_exhausted`] of
    /// every faction the save carries, in `BTreeMap` order.
    ///
    /// A faction does not run out of ways back on a clock -- it runs out
    /// because a building fell, a force died or a cell changed hands -- so
    /// this is how the loss gets *noticed*, in the hour it happened, which is
    /// what makes brief section 16's "other factions immediately reevaluate
    /// the board" true: the flag is set before the next hour's
    /// `recompute_strategic_state` reads it.
    ///
    /// `definitions` is the building registry, and B16 made it the real one:
    /// [`ExpeditionState::strategic_tick`](crate::expedition::ExpeditionState::strategic_tick)
    /// takes it from its caller, the expedition bridge builds it from the
    /// `building.*` records Godot forwards out of the content bundle, and the
    /// Rust harness builds it from the same `content/buildings/` files. So the
    /// sweep now reads C10's authored records: what a building makes, how much
    /// of a cell its envelope takes, and whether its wreck still occupies
    /// ground.
    ///
    /// The conservative reading of an *unlookupable* building stays, because it
    /// is still reachable and is still the right answer where it is: a save
    /// carrying an instance whose record has been withdrawn, or a caller that
    /// passes `BuildingDefinitions::new()` deliberately, gets a building read
    /// as standing, productive and not to be ruined. That can only delay an
    /// elimination, never cause one. Nothing else in this file changed when the
    /// registry became real, which is what the parameter was for.
    pub fn eliminate_exhausted_factions(
        &mut self,
        geography: &Geography,
        definitions: &BuildingDefinitions,
    ) -> Vec<StrategicEvent> {
        let faction_ids: Vec<String> = self.factions.keys().cloned().collect();
        let mut events = Vec::new();
        for faction_id in &faction_ids {
            events.extend(self.eliminate_if_exhausted(faction_id, geography, definitions));
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    use crate::habitat::Habitats;
    use crate::strategy::building::{
        BuildingDefinition, BuildingSocket, BuildingState, FIRST_TIER, ProductionOutput,
        ProductionRule, RuinState, SocketKind, TierState,
    };
    use crate::strategy::faction::{FactionDefinitions, FactionState, Relationship};
    use crate::strategy::production::MachineFamily;

    const BEACH: &str = "world.cell.black_beach";

    /// Content owns building IDs; `content/buildings/` is C10's card and does
    /// not exist yet, so these are the test namespace's and are read by
    /// nothing outside this module.
    const PRODUCER: &str = "building.test.producer";
    const PLAIN: &str = "building.test.plain";
    const SCAFFOLD: &str = "building.test.scaffold";
    const DEMOLISHED: &str = "building.test.destroy_only";

    fn pirates() -> String {
        ConceptKey::Pirates.faction_id()
    }

    fn elves() -> String {
        ConceptKey::Elves.faction_id()
    }

    /// Envelope four (footprint three plus clearance one), so three of them
    /// fit exactly inside `CELL_CAPACITY_CELLS` and two leave room for a
    /// third -- which is what makes "a valid construction site" a thing this
    /// fixture can take away without taking the cell away.
    fn a_definition(id: &str, construction_hours: u32) -> BuildingDefinition {
        BuildingDefinition {
            id: id.to_owned(),
            faction_compatibility: BTreeSet::from([ConceptKey::Pirates, ConceptKey::Elves]),
            function: "a test building".into(),
            footprint_cells: 3,
            clearance_cells: 1,
            height_class: "needs decision".into(),
            entrance_sockets: vec![BuildingSocket {
                id: "socket.door".into(),
                kind: SocketKind::Entrance,
                offset_cells: 0,
            }],
            tier_states: vec![TierState {
                tier: FIRST_TIER,
                construction_hours,
                construction_requirements: BTreeSet::new(),
                hit_points: 100,
                notes: String::new(),
            }],
            capture_rules: CaptureRules::Capturable,
            ruin_state: RuinState::LeavesRubble { footprint_cells: 4 },
            ..BuildingDefinition::default()
        }
    }

    fn a_producing_definition() -> BuildingDefinition {
        BuildingDefinition {
            production: vec![ProductionRule {
                id: "rule.test.output".into(),
                output: ProductionOutput::Machine {
                    family: MachineFamily::default(),
                },
                output_key: "machine.example".into(),
                amount: 1,
                interval_hours: 6,
                minimum_tier: FIRST_TIER,
                cost: BTreeMap::new(),
            }],
            ..a_definition(PRODUCER, 0)
        }
    }

    fn registry() -> BuildingDefinitions {
        let mut registry = BuildingDefinitions::new();
        for definition in [
            a_producing_definition(),
            a_definition(PLAIN, 0),
            a_definition(SCAFFOLD, 8),
            BuildingDefinition {
                capture_rules: CaptureRules::DestroyOnly,
                ..a_definition(DEMOLISHED, 0)
            },
        ] {
            registry
                .insert(definition)
                .expect("the fixture definitions load");
        }
        registry
    }

    /// A campaign with two factions in it and nothing held by either. The
    /// second exists so an allied refuge has somebody living to point at.
    fn a_campaign() -> ExpeditionState {
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        state.factions.insert(pirates(), FactionState::new());
        state.factions.insert(elves(), FactionState::new());
        state
    }

    fn trust(state: &mut ExpeditionState, from: &str, toward: &str, trust: i16) {
        state
            .factions
            .get_mut(from)
            .expect("the fixture carries this faction")
            .relationships
            .insert(
                toward.to_owned(),
                Relationship {
                    trust,
                    ..Relationship::default()
                },
            );
    }

    fn chain(
        state: &ExpeditionState,
        faction_id: &str,
        geography: &Geography,
    ) -> Vec<RecoveryLink> {
        recovery_chain(state, faction_id, geography, &registry())
    }

    /// The list is brief section 16's, bullet for bullet and in its order,
    /// with the one variant that is not the brief's last and named for what it
    /// is. If section 16 gains a way back, this test is where the code notices.
    #[test]
    fn the_links_are_brief_section_sixteens_list_in_the_briefs_order() {
        assert_eq!(
            RecoveryLink::ALL,
            [
                RecoveryLink::OperationalCoreBuilding,
                RecoveryLink::RemainingPopulation,
                RecoveryLink::WorkerProductionOrRecruitment,
                RecoveryLink::ResourceReserve,
                RecoveryLink::ControlledSettlement,
                RecoveryLink::MobileRecoveryForce,
                RecoveryLink::AlliedRefuge,
                RecoveryLink::ValidConstructionSite,
                RecoveryLink::AuthoredRecoveryEvent,
                RecoveryLink::NeedsDecision,
            ]
        );
        // Declaration order is the brief's order, and `BTreeSet` iteration
        // follows it -- which is what makes the saved reading deterministic.
        let set: BTreeSet<RecoveryLink> = RecoveryLink::ALL.into_iter().collect();
        assert_eq!(set.into_iter().collect::<Vec<_>>(), RecoveryLink::ALL);
    }

    /// The card's first Done-when: every link the chain can hold, taken away
    /// one at a time, each loss reported, and the last one ending the faction.
    ///
    /// Bite proof: deleting the `chain.is_empty()` guard in
    /// `eliminate_if_exhausted` eliminates the faction at step one; deleting
    /// the `has_ever_held` guard eliminates it before step one.
    #[test]
    fn every_link_is_exhausted_one_at_a_time_and_the_last_one_ends_the_faction() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        // A faction holding every link this crate can model: a cell, two
        // working buildings (one of them productive), room to build a third,
        // a stocked store, a force, and a neighbour it trusts.
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        for (instance, def_id) in [
            ("building_instance.test.works", PRODUCER),
            ("building_instance.test.hut", PLAIN),
        ] {
            state
                .place_building(
                    instance,
                    def_id,
                    BEACH,
                    &pirates(),
                    &geography,
                    &definitions,
                )
                .expect("the fixture envelopes fit");
        }
        state
            .factions
            .get_mut(&pirates())
            .expect("the fixture carries this faction")
            .resources
            .insert("resource.example".into(), 5);
        state
            .raise_force(
                "force.test.column",
                &pirates(),
                BEACH,
                BTreeSet::new(),
                BTreeMap::from([("actor.example".to_owned(), 3u32)]),
                "",
                &geography,
            )
            .expect("a force raises on a real cell");
        trust(&mut state, &pirates(), &elves(), ALLY_REFUGE_TRUST_FLOOR);

        assert_eq!(
            chain(&state, &pirates(), &geography),
            vec![
                RecoveryLink::OperationalCoreBuilding,
                RecoveryLink::WorkerProductionOrRecruitment,
                RecoveryLink::ResourceReserve,
                RecoveryLink::ControlledSettlement,
                RecoveryLink::MobileRecoveryForce,
                RecoveryLink::AlliedRefuge,
                RecoveryLink::ValidConstructionSite,
            ],
            "seven of section 16's nine links; the other two have no state to read"
        );
        // The first reading only records what is held. Nothing is lost yet.
        assert_eq!(
            state.eliminate_if_exhausted(&pirates(), &geography, &definitions),
            Vec::new()
        );

        let lost = |state: &mut ExpeditionState, link: RecoveryLink| {
            assert_eq!(
                state.eliminate_if_exhausted(&pirates(), &geography, &definitions),
                vec![StrategicEvent::RecoveryLinkLost {
                    faction_id: pirates(),
                    link,
                }],
                "exactly one link goes, and it is named"
            );
            assert!(
                !state.factions[&pirates()].eliminated,
                "a faction with links left is not eliminated"
            );
        };

        // 1. The stores run out.
        state
            .factions
            .get_mut(&pirates())
            .expect("the fixture carries this faction")
            .resources
            .clear();
        lost(&mut state, RecoveryLink::ResourceReserve);

        // 2. The column is gone.
        state.forces.clear();
        lost(&mut state, RecoveryLink::MobileRecoveryForce);

        // 3. The neighbour cools below the floor.
        trust(
            &mut state,
            &pirates(),
            &elves(),
            ALLY_REFUGE_TRUST_FLOOR - 1,
        );
        lost(&mut state, RecoveryLink::AlliedRefuge);

        // 4. The works come down; the hut still stands, so the faction still
        //    has a working building -- it has lost the ability to *make*.
        state
            .ruin_building("building_instance.test.works")
            .expect("a standing building can be brought down");
        lost(&mut state, RecoveryLink::WorkerProductionOrRecruitment);

        // 5. The hut comes down too.
        state
            .ruin_building("building_instance.test.hut")
            .expect("a standing building can be brought down");
        lost(&mut state, RecoveryLink::OperationalCoreBuilding);

        // 6. The last of the cell's capacity is spoken for: two wrecks and a
        //    half-built third leave no room to raise anything. Ground held is
        //    not the same as ground you can build on.
        state
            .place_building(
                "building_instance.test.scaffold",
                SCAFFOLD,
                BEACH,
                &pirates(),
                &geography,
                &definitions,
            )
            .expect("the third envelope is the last that fits");
        assert_eq!(
            state.envelope_cells_used(BEACH, &definitions),
            CELL_CAPACITY_CELLS
        );
        lost(&mut state, RecoveryLink::ValidConstructionSite);

        // 7. And the ground itself. Nothing is left, and the chain is empty.
        state
            .set_control(BEACH, None, &geography)
            .expect("the beach is a real cell");
        assert_eq!(chain(&state, &pirates(), &geography), Vec::new());
        assert_eq!(
            state.eliminate_if_exhausted(&pirates(), &geography, &definitions),
            vec![
                StrategicEvent::RecoveryLinkLost {
                    faction_id: pirates(),
                    link: RecoveryLink::ControlledSettlement,
                },
                StrategicEvent::FactionEliminated {
                    faction_id: pirates(),
                    day: state.campaign_day,
                },
            ]
        );

        let gone = &state.factions[&pirates()];
        assert!(gone.eliminated);
        assert_eq!(gone.strategic_state, StrategicState::Desperate);
        assert!(gone.current_goals.is_empty(), "scheduling stops");
        // Asked again, it says nothing: elimination happens once.
        assert_eq!(
            state.eliminate_if_exhausted(&pirates(), &geography, &definitions),
            Vec::new()
        );
    }

    /// The card's second Done-when: one link is a chain, and a chain is life.
    ///
    /// Bite proof: making `AlliedRefuge` count as exhausted -- deleting its
    /// `chain.push` in `recovery_chain` -- fails this test on the
    /// `assert!(!eliminated)` below, and on the empty chain above it.
    #[test]
    fn one_allied_refuge_is_the_whole_chain_and_the_faction_survives() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        // It once held ground, so it is a candidate for elimination...
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        trust(&mut state, &pirates(), &elves(), ALLY_REFUGE_TRUST_FLOOR);
        state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        assert!(state.factions[&pirates()].has_ever_held);

        // ...and now it holds nothing at all except a neighbour that will take
        // it in.
        state
            .set_control(BEACH, None, &geography)
            .expect("the beach is a real cell");
        assert_eq!(
            chain(&state, &pirates(), &geography),
            vec![RecoveryLink::AlliedRefuge],
            "brief section 16: an allied refuge is a viable recovery path"
        );

        let events = state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, StrategicEvent::FactionEliminated { .. })),
            "a faction with one link left is not eliminated"
        );
        assert!(!state.factions[&pirates()].eliminated);
    }

    /// A refuge is a *living* faction. The last two on the board cannot hold
    /// each other up once one of them is gone.
    #[test]
    fn an_eliminated_neighbour_shelters_nobody() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        trust(&mut state, &pirates(), &elves(), ALLY_REFUGE_TRUST_FLOOR);
        state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        state
            .set_control(BEACH, None, &geography)
            .expect("the beach is a real cell");
        assert_eq!(
            chain(&state, &pirates(), &geography),
            vec![RecoveryLink::AlliedRefuge]
        );

        state
            .factions
            .get_mut(&elves())
            .expect("the fixture carries this faction")
            .eliminated = true;
        assert_eq!(chain(&state, &pirates(), &geography), Vec::new());
        assert!(
            state
                .eliminate_if_exhausted(&pirates(), &geography, &definitions)
                .contains(&StrategicEvent::FactionEliminated {
                    faction_id: pirates(),
                    day: state.campaign_day,
                })
        );
    }

    /// The rule the 2,400-hour harness depends on, stated as a test: an
    /// unclaimed board is not exhaustion. A faction that has never held a
    /// cell, built anything or raised a force is not eliminated by the absence
    /// of those things; elimination requires having *lost* them.
    #[test]
    fn a_faction_that_has_never_held_anything_is_never_eliminated() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        assert_eq!(chain(&state, &pirates(), &geography), Vec::new());
        for _ in 0..HOURS_IN_A_DAY {
            assert_eq!(
                state.eliminate_if_exhausted(&pirates(), &geography, &definitions),
                Vec::new(),
                "an empty chain that was always empty is not a loss"
            );
        }
        assert!(!state.factions[&pirates()].eliminated);
        assert!(!state.factions[&pirates()].has_ever_held);

        // One hour of holding is all it takes to become a candidate.
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        assert!(state.factions[&pirates()].has_ever_held);
        state
            .set_control(BEACH, None, &geography)
            .expect("the beach is a real cell");
        assert!(
            state
                .eliminate_if_exhausted(&pirates(), &geography, &definitions)
                .contains(&StrategicEvent::FactionEliminated {
                    faction_id: pirates(),
                    day: state.campaign_day,
                })
        );
    }

    const HOURS_IN_A_DAY: u32 = 24;

    /// Buildings transition by their own records and by nothing else: a
    /// `DestroyOnly` one is ruined, a `Capturable` one is left standing as an
    /// unowned shell for whoever walks in. No captor is invented.
    #[test]
    fn elimination_transitions_each_building_by_its_own_record() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        for (instance, def_id) in [
            ("building_instance.test.hut", PLAIN),
            ("building_instance.test.magazine", DEMOLISHED),
        ] {
            state
                .place_building(
                    instance,
                    def_id,
                    BEACH,
                    &pirates(),
                    &geography,
                    &definitions,
                )
                .expect("the fixture envelopes fit");
        }

        state.eliminate_faction(&pirates(), &geography, &definitions);

        let hut = &state.buildings["building_instance.test.hut"];
        assert_eq!(
            hut.state,
            BuildingState::Operational,
            "a capturable building is left standing; nobody has taken it"
        );
        assert_eq!(
            hut.faction_id,
            pirates(),
            "the record of who raised it survives; capture_building is the only way it changes hands"
        );
        assert_eq!(
            state.buildings["building_instance.test.magazine"].state,
            BuildingState::Ruined,
            "a destroy-only building goes down with its faction"
        );
        assert!(
            !state.ownership.contains_key(BEACH),
            "territory becomes available to competitors"
        );
    }

    /// Forces halt where they stand, whole. Brief section 10's aggregate does
    /// not evaporate because its faction did.
    #[test]
    fn elimination_halts_the_forces_without_disbanding_them() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        state
            .raise_force(
                "force.test.column",
                &pirates(),
                BEACH,
                BTreeSet::new(),
                BTreeMap::from([("actor.example".to_owned(), 3u32)]),
                "",
                &geography,
            )
            .expect("a force raises on a real cell");
        let destination = geography
            .all_location_ids()
            .find(|cell_id| *cell_id != BEACH)
            .expect("the slice has more than one cell")
            .to_owned();
        let marching = state
            .dispatch_force("force.test.column", &destination, &geography)
            .is_ok();

        state.eliminate_faction(&pirates(), &geography, &definitions);

        let force = &state.forces["force.test.column"];
        assert!(!force.is_marching(), "an eliminated faction's forces stop");
        assert_eq!(force.destination_cell_id, None);
        assert_eq!(force.strength, 3, "the body is still whole");
        assert_eq!(force.composition["actor.example"], 3);
        assert!(
            marching || force.route.is_empty(),
            "the fixture either marched and was halted, or never left"
        );
    }

    /// **No respawn.** The flag survives a midnight and a save round trip, and
    /// eliminating an eliminated faction is a no-op that reports nothing.
    ///
    /// This is the test that stands in for a grep: nothing in the crate sets
    /// `eliminated` back to `false`, so there is nothing to call here that
    /// would undo it -- including the two calls that most look like they might.
    #[test]
    fn elimination_survives_a_midnight_and_a_save_round_trip_and_never_reverses() {
        let geography = Geography::black_beach_vertical_slice();
        let habitats = Habitats::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();

        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        // The neighbour takes the ground rather than it falling vacant, so the
        // board is still claimed afterwards: S5 calls an *unclaimed* board
        // Contesting for everybody, and this test is about the eliminated
        // faction's position, not that rule.
        state
            .set_control(BEACH, Some(elves()), &geography)
            .expect("the beach is a real cell");
        state.eliminate_if_exhausted(&pirates(), &geography, &definitions);
        assert!(state.factions[&pirates()].eliminated);

        // Twenty-four strategic hours and the character-scale day turning.
        state
            .resolve_midnight_in(
                &geography,
                &habitats,
                &FactionDefinitions::new(),
                &BuildingDefinitions::new(),
            )
            .expect("a midnight resolves");
        assert!(
            state.factions[&pirates()].eliminated,
            "a day of simulation does not bring a faction back"
        );
        assert_eq!(
            state.factions[&pirates()].strategic_state,
            StrategicState::Desperate,
            "S5 recomputes an eliminated faction as Desperate every hour"
        );

        let reloaded = ExpeditionState::from_json(&state.to_json()).expect("the save round-trips");
        assert!(reloaded.factions[&pirates()].eliminated);
        assert_eq!(
            reloaded.factions[&pirates()].recovery_links_held,
            state.factions[&pirates()].recovery_links_held
        );
        assert!(reloaded.factions[&pirates()].has_ever_held);

        let mut reloaded = reloaded;
        assert_eq!(
            reloaded.eliminate_faction(&pirates(), &geography, &definitions),
            Vec::new(),
            "eliminating an eliminated faction is a no-op returning no events"
        );
        assert!(reloaded.factions[&pirates()].eliminated);
    }

    /// Brief section 20 leaves Cthulhu's early-elimination rules Open, so this
    /// code cannot remove that faction at all: its chain carries
    /// `NeedsDecision` and is therefore never empty.
    #[test]
    fn cthulhu_cannot_be_eliminated_while_its_rules_are_open() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state = a_campaign();
        let cthulhu = ConceptKey::Cthulhu.faction_id();
        state.factions.insert(cthulhu.clone(), FactionState::new());

        assert_eq!(
            chain(&state, &cthulhu, &geography),
            vec![RecoveryLink::NeedsDecision],
            "blocked: needs decision -- brief section 20, Cthulhu early-elimination rules"
        );
        state
            .set_control(BEACH, Some(cthulhu.clone()), &geography)
            .expect("the beach is a real cell");
        state.eliminate_if_exhausted(&cthulhu, &geography, &definitions);
        state
            .set_control(BEACH, None, &geography)
            .expect("the beach is a real cell");

        let events = state.eliminate_if_exhausted(&cthulhu, &geography, &definitions);
        assert_eq!(
            events,
            vec![
                StrategicEvent::RecoveryLinkLost {
                    faction_id: cthulhu.clone(),
                    link: RecoveryLink::ControlledSettlement,
                },
                StrategicEvent::RecoveryLinkLost {
                    faction_id: cthulhu.clone(),
                    link: RecoveryLink::ValidConstructionSite,
                },
            ],
            "it loses links like anyone else -- ground and the room to build on it go together -- it just never runs out"
        );
        assert!(!state.factions[&cthulhu].eliminated);

        // And no other faction is handed the same immunity.
        assert!(!chain(&state, &pirates(), &geography).contains(&RecoveryLink::NeedsDecision));
    }

    /// The two links section 16 lists that no state in this crate can answer
    /// are never reported, and are never quietly faked as present.
    #[test]
    fn the_two_unbacked_links_are_never_reported() {
        let geography = Geography::black_beach_vertical_slice();
        let mut state = a_campaign();
        state
            .set_control(BEACH, Some(pirates()), &geography)
            .expect("the beach is a real cell");
        state
            .factions
            .get_mut(&pirates())
            .expect("the fixture carries this faction")
            .resources
            .insert("resource.example".into(), 100);

        let held = chain(&state, &pirates(), &geography);
        assert!(!held.contains(&RecoveryLink::RemainingPopulation));
        assert!(!held.contains(&RecoveryLink::AuthoredRecoveryEvent));
    }

    /// "What happens if all ordinary factions collapse" is Open (brief section
    /// 20), so six eliminations are six events and no seventh of any kind.
    /// There is no last-faction branch here to be mistaken for that decision.
    #[test]
    fn a_board_that_empties_emits_nothing_beyond_the_eliminations() {
        let geography = Geography::black_beach_vertical_slice();
        let definitions = registry();
        let mut state =
            ExpeditionState::new(7, vec!["character.protagonist.captain".into()], BEACH)
                .expect("a fresh campaign constructs");
        // Every concept but Cthulhu, whose rules are Open.
        let ordinary: Vec<String> = ConceptKey::ALL
            .into_iter()
            .filter(|concept| *concept != ConceptKey::Cthulhu)
            .map(ConceptKey::faction_id)
            .collect();
        for faction_id in &ordinary {
            state
                .factions
                .insert(faction_id.clone(), FactionState::new());
            state
                .factions
                .get_mut(faction_id)
                .expect("just inserted")
                .resources
                .insert("resource.example".into(), 1);
        }
        state.eliminate_exhausted_factions(&geography, &definitions);

        for faction_id in &ordinary {
            state
                .factions
                .get_mut(faction_id)
                .expect("the fixture carries this faction")
                .resources
                .clear();
        }
        let events = state.eliminate_exhausted_factions(&geography, &definitions);
        let eliminations = events
            .iter()
            .filter(|event| matches!(event, StrategicEvent::FactionEliminated { .. }))
            .count();
        assert_eq!(eliminations, ordinary.len());
        assert_eq!(
            events.len(),
            ordinary.len() * 2,
            "one lost link and one elimination each, and nothing invented for the empty board"
        );
    }
}
