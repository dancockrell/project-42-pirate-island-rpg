//! The world exactly as Godot hands it to the bridge, driven through the same
//! sequence `game/tests/expedition_prototype_test.gd` drives on the engine.
//!
//! `native_expedition_port.gd` reads the content catalog and forwards cells,
//! anchors, portals and encounter triggers to `Project42ExpeditionBridge`.
//! This test performs that same translation from `content/world/` and
//! `content/encounters/` and then plays the prototype's opening: salvage the
//! wreck, climb to the estate, drop to the river, take the safe road, meet the
//! terrace's authored encounter. There is no Godot in the Rust toolchain, so
//! this is the nearest local proof that the engine-side slice holds together;
//! CI's Godot job is the real one.

use std::collections::BTreeMap;

use project42_sim::geography::{
    AuthoredCell, CONTESTED_RISK_MODIFIER, EncounterTriggerDefinition, PortalDefinition,
};
use project42_sim::strategy::building::{BuildingDefinition, BuildingDefinitions};
use project42_sim::strategy::faction::{ConceptKey, FactionDefinition, FactionDefinitions};
use project42_sim::*;

fn content(path: &str) -> String {
    format!("{}/../content/{path}", env!("CARGO_MANIFEST_DIR"))
}

fn json_files(directory: &str) -> Vec<serde_json::Value> {
    let mut records = Vec::new();
    for entry in std::fs::read_dir(content(directory)).expect("content directory is readable") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("record is readable");
        records.push(serde_json::from_str(&text).expect("record is JSON"));
    }
    records
}

/// The generated Godot bundle, as `ContentCatalog` loads it. Reading the
/// bundle rather than `content/factions/` is the point of this file: the
/// records only reach Godot if `build-content-bundle.mjs` carries their domain,
/// and CI holds the committed bundle fresh against the sources.
fn bundle_records(domain: &str) -> Vec<serde_json::Value> {
    let path = format!(
        "{}/../game/generated/content_bundle.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("the generated content bundle is readable");
    let bundle: serde_json::Value =
        serde_json::from_str(&text).expect("the generated content bundle is JSON");
    bundle["records"]
        .as_array()
        .expect("the bundle carries records")
        .iter()
        .filter(|record| record["domain"] == domain)
        .map(|record| record["value"].clone())
        .collect()
}

/// Mirrors `NativeExpeditionPort.configure_from_catalog`'s faction forwarding:
/// every `faction.*` record out of the catalog, verbatim, into the
/// `factions` array `ExpeditionConfiguration` reads. Verbatim is the whole
/// trick -- C9 authored the records in `FactionDefinition`'s own field names,
/// so the port renames nothing and the fields cannot drift apart in
/// translation the way the cell and portal fields above can.
fn faction_registry_as_godot_forwards_it() -> FactionDefinitions {
    let forwarded = serde_json::Value::Array(bundle_records("factions"));
    let definitions: Vec<FactionDefinition> =
        serde_json::from_value(forwarded).expect("every forwarded record is a FactionDefinition");
    let mut registry = FactionDefinitions::new();
    for definition in definitions {
        registry
            .insert(definition)
            .expect("the bridge admits every authored record");
    }
    registry
}

/// The six records reach Godot, survive the forwarding verbatim, and give the
/// bridge six distinct `faction.<concept_key>` IDs.
///
/// `strategy/faction.rs` already holds `content/factions/` equal to `ConceptKey`
/// on disk; this is the other half of the same claim and the half that was
/// missing -- that the bundle Godot actually reads carries them, that the
/// records survive the JSON round trip through it (their `displayName` and
/// `metadata` keys have no Rust field and must be ignored, not refused), and
/// that `FactionDefinitions::insert` validates every one on the way in. Before
/// B15 the builder's domain list omitted `factions` and this found nothing.
#[test]
fn the_six_authored_factions_reach_the_bridge_as_six_distinct_ids() {
    let registry = faction_registry_as_godot_forwards_it();
    assert_eq!(
        registry.len(),
        6,
        "the Godot bundle must carry all six authored faction records"
    );

    let ids: Vec<&str> = registry.ids().collect();
    // The registry iterates ascending by ID, like every other iteration in the
    // crate, so the expectation is sorted rather than in the brief's order.
    let mut expected: Vec<String> = ConceptKey::ALL
        .into_iter()
        .map(ConceptKey::faction_id)
        .collect();
    expected.sort();
    assert_eq!(
        ids,
        expected.iter().map(String::as_str).collect::<Vec<_>>(),
        "six distinct faction.<concept_key> IDs, one per concept, ascending"
    );
    for concept in ConceptKey::ALL {
        assert!(
            registry
                .by_concept(concept)
                .is_some_and(|record| record.id == concept.faction_id()),
            "{} did not reach the bridge",
            concept.faction_id()
        );
    }
}

/// Mirrors the same script's building forwarding, on the same terms: every
/// `building.*` record out of the catalog, verbatim, into the `buildings` array
/// `ExpeditionConfiguration` reads. C10 authored them in
/// `BuildingDefinition`'s own field names, so nothing is renamed here either.
fn building_registry_as_godot_forwards_it() -> BuildingDefinitions {
    let forwarded = serde_json::Value::Array(bundle_records("buildings"));
    let definitions: Vec<BuildingDefinition> =
        serde_json::from_value(forwarded).expect("every forwarded record is a BuildingDefinition");
    let mut registry = BuildingDefinitions::new();
    for definition in definitions {
        registry
            .insert(definition)
            .expect("the bridge admits every authored record");
    }
    registry
}

/// C10's three records reach Godot, survive the forwarding verbatim, and pass
/// `BuildingDefinition::validate` on the way into the bridge's registry.
///
/// `strategy/building.rs` already holds `content/buildings/` equal to the schema
/// on disk; this is the other half, and the half B16 needed: that the bundle
/// Godot actually reads carries the records, that the authoring-only keys
/// (`metadata`, the prose relationships) survive the JSON round trip through it,
/// and that the registry the bridge hands to the tick is the one the harness
/// runs on.
#[test]
fn the_authored_buildings_reach_the_bridge_and_validate() {
    let registry = building_registry_as_godot_forwards_it();
    let ids: Vec<&str> = registry.ids().collect();
    assert_eq!(
        ids,
        vec![
            "building.coast_watch_post",
            "building.machine_shop",
            "building.ritual_anchor"
        ],
        "the Godot bundle must carry C10's three building records, ascending by ID"
    );

    // The two halves of brief section 5.6 are authored, not asserted: exactly
    // one of these records is Michael's, and it makes machines rather than
    // people. `validate` refused the alternative on the way in.
    let michaels: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|id| {
            registry
                .get(id)
                .is_some_and(|record| record.faction_compatibility.contains(&ConceptKey::Michael))
        })
        .collect();
    assert_eq!(
        michaels,
        vec!["building.machine_shop"],
        "Michael's working core must reach the bridge"
    );
}

/// Mirrors `NativeExpeditionPort.configure_from_catalog` field for field. If
/// that script and this function ever disagree, one of them is lying to the
/// bridge, and the GDScript suite is the one that would catch which.
fn geography_as_godot_forwards_it() -> Geography {
    let battles_by_encounter: BTreeMap<String, String> = json_files("encounters")
        .into_iter()
        .map(|record| {
            (
                record["id"].as_str().expect("encounter id").to_owned(),
                record["battleId"].as_str().expect("battleId").to_owned(),
            )
        })
        .collect();

    let mut cells = Vec::new();
    let mut portals = Vec::new();
    let mut triggers = Vec::new();
    for cell in json_files("world")
        .into_iter()
        .filter(|record| record["kind"] == "world_cell")
    {
        let cell_id = cell["id"].as_str().expect("cell id").to_owned();
        let observation_ids: Vec<serde_json::Value> = cell["readableDescriptions"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|d| d["id"].clone())
            .collect();
        let anchors: Vec<serde_json::Value> = cell["anchors"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|anchor| {
                let mut forwarded = serde_json::json!({
                    "id": anchor["id"],
                    "kind": anchor["kind"],
                    "rations": anchor["rations"].as_u64().unwrap_or(0),
                    "medicine": anchor["medicine"].as_u64().unwrap_or(0),
                    "coin": anchor["coin"].as_u64().unwrap_or(0),
                    "once_per_day": anchor["oncePerDay"].as_bool().unwrap_or(false),
                });
                for (from, to) in [
                    ("requiresDiscoveryId", "requires_discovery_id"),
                    ("grantsDiscoveryId", "grants_discovery_id"),
                ] {
                    if let Some(value) = anchor[from].as_str() {
                        forwarded[to] = serde_json::Value::String(value.to_owned());
                    }
                }
                forwarded
            })
            .collect();
        // B7: the port forwards each battle entry's id, status and (when it
        // has one) habitat binding, and Rust derives eligibility from them.
        let battle_entries: Vec<serde_json::Value> = cell["battleEntries"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|entry| {
                let mut forwarded = serde_json::json!({
                    "id": entry["id"],
                    "status": entry["status"],
                });
                if let Some(habitat_id) = entry["habitatId"].as_str() {
                    forwarded["habitat_id"] = serde_json::Value::String(habitat_id.to_owned());
                }
                forwarded
            })
            .collect();
        let authored: AuthoredCell = serde_json::from_value(serde_json::json!({
            "id": cell_id,
            "region_id": cell["regionId"],
            "display_name": cell["displayName"],
            "observation_ids": observation_ids,
            "anchors": anchors,
            "battle_entries": battle_entries,
        }))
        .expect("the forwarded cell has the wire shape");
        cells.push(CellDefinition::try_from(authored).expect("known anchor kinds"));

        for portal in cell["portals"].as_array().into_iter().flatten() {
            let mut forwarded = serde_json::json!({
                "id": portal["id"],
                "from_location_id": cell_id,
                "target_location_id": portal["targetCellId"],
                "travel_mode": portal["travelMode"],
                "time_cost_minutes": portal["timeCostMinutes"].as_u64().unwrap_or(0),
                "supply_cost": portal["supplyCost"].as_u64().unwrap_or(0),
                "risk_level": portal["riskLevel"].as_u64().unwrap_or(0),
            });
            if let Some(gate) = portal["requiredDiscoveryId"].as_str() {
                forwarded["required_discovery_id"] = serde_json::Value::String(gate.to_owned());
            }
            let portal: PortalDefinition =
                serde_json::from_value(forwarded).expect("the forwarded portal has the wire shape");
            portals.push(portal);
        }
        for entry in cell["battleEntries"].as_array().into_iter().flatten() {
            if entry["status"] != "vertical_slice_encounter" {
                continue;
            }
            let encounter_id = entry["encounterId"]
                .as_str()
                .expect("encounterId")
                .to_owned();
            let battle_id = battles_by_encounter[&encounter_id].clone();
            triggers.push(EncounterTriggerDefinition {
                location_id: cell_id.clone(),
                encounter_id,
                battle_id,
                estate_upgrade_id: None,
            });
        }
    }
    Geography::from_authored(cells, portals, triggers).expect("the authored world builds")
}

#[test]
fn the_engine_side_slice_holds_together_on_the_authored_world() {
    let geography = geography_as_godot_forwards_it();
    let habitats = Habitats::black_beach_vertical_slice();
    let mut state = ExpeditionState::new(
        7,
        vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
        ],
        "world.cell.black_beach",
    )
    .expect("the prototype's party constructs");

    // Black Beach exposes exactly one legal route: the tidal cut is authored,
    // gated on the map table, and must not be offered before then.
    let legal = state
        .legal_route_commands(&geography)
        .expect("no encounter pending on the sand");
    assert_eq!(
        legal,
        vec!["travel:world.portal.black_beach_to_damaged_estate".to_owned()]
    );

    // Roads cost rations and the party lands with none. The wreck is an anchor
    // content declares, so through the same wire Godot uses it exists here.
    let salvage = state
        .use_anchor("anchor.black_beach.salvage_point", &geography)
        .expect("the wreck is salvageable through the authored world");
    assert!(
        salvage.rations_gained >= 4,
        "authored floor of four rations"
    );
    assert_eq!(
        state
            .legal_route_commands(&geography)
            .expect("still on the sand")
            .len(),
        1,
        "salvaging must not change the one legal departure"
    );

    state
        .travel("world.portal.black_beach_to_damaged_estate", &geography)
        .expect("the estate climb is free");
    state
        .travel("world.portal.damaged_estate_to_river_landing", &geography)
        .expect("the river gate");
    assert_eq!(
        state
            .legal_route_commands(&geography)
            .expect("no encounter")
            .len(),
        3,
        "River Landing projects its three legal routes"
    );
    let before = state.supplies.rations;
    state
        .travel(
            "world.portal.river_landing_to_reception_terrace_safe_road",
            &geography,
        )
        .expect("the safe road, paid for with salvaged rations");
    assert!(
        state.supplies.rations < before,
        "the authored road cost reached the simulation"
    );
    let encounter = state
        .begin_encounter(&geography, &habitats)
        .expect("the terrace arms its authored encounter")
        .clone();
    assert_eq!(
        encounter.encounter_id,
        "encounter.prototype.returning_names"
    );
    assert_eq!(encounter.battle_id, "battle.prototype.returning_names");
    assert!(
        state.legal_route_commands(&geography).is_err(),
        "a pending encounter blocks travel"
    );
}

#[test]
fn the_tidal_cut_opens_only_after_the_map_table_is_read() {
    let geography = geography_as_godot_forwards_it();
    let mut state = ExpeditionState::new(
        7,
        vec!["character.protagonist.captain".into()],
        "world.cell.black_beach",
    )
    .expect("constructs");
    state.supplies.rations = 10;
    assert!(
        state
            .travel(
                "world.portal.black_beach_to_reception_terrace_tidal_cut",
                &geography
            )
            .is_err(),
        "the tidal cut is closed until the map table grants its discovery"
    );
    // The map table's grant is authored on the estate's anchor and reaches the
    // simulation through the same wire; the Rust rule and the authored
    // record are held equal in geography's own tests.
    state
        .discoveries
        .insert("discovery.map_table.tidal_cut".into());
    state
        .travel(
            "world.portal.black_beach_to_reception_terrace_tidal_cut",
            &geography,
        )
        .expect("an opened gate is an open road");
    assert_eq!(state.active_location_id, "world.cell.reception_terrace");
}

/// B3's projection, proven on the world Godot actually forwards: the beach's
/// full legal-command list is what the expedition screen draws its controls
/// from, so a spent anchor has to leave that list and come back at midnight.
/// `Project42ExpeditionBridge` cannot be unit-tested without a live Godot, so
/// this holds the rule the bridge projects rather than the projection itself.
#[test]
fn the_salvage_anchor_leaves_the_legal_commands_when_spent_and_returns_at_midnight() {
    let geography = geography_as_godot_forwards_it();
    let habitats = Habitats::black_beach_vertical_slice();
    let mut state = ExpeditionState::new(
        42,
        vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
        ],
        "world.cell.black_beach",
    )
    .expect("the prototype's party constructs");

    let salvage = "anchor_action:anchor.black_beach.salvage_point".to_owned();
    assert!(
        state
            .legal_next_commands_with_geography(&geography)
            .contains(&salvage),
        "the wreck is a legal action on the sand before it is salvaged"
    );
    // Both authored observations are offered too: the action list is drawn
    // from this one list, not from the catalog.
    assert!(
        state
            .legal_next_commands_with_geography(&geography)
            .contains(&"inspect:observation.black_beach.wreck".to_owned())
    );

    state
        .use_anchor("anchor.black_beach.salvage_point", &geography)
        .expect("the wreck is salvageable");
    assert!(
        !state
            .legal_next_commands_with_geography(&geography)
            .contains(&salvage),
        "an anchor spent today must not be drawn as a button"
    );

    state
        .resolve_midnight_in(
            &geography,
            &habitats,
            &faction_registry_as_godot_forwards_it(),
            &building_registry_as_godot_forwards_it(),
        )
        .expect("no encounter is pending on the sand");
    assert!(
        state
            .legal_next_commands_with_geography(&geography)
            .contains(&salvage),
        "midnight makes the wreck salvageable again"
    );
}

/// B14's rule, on the world Godot forwards: the safe road out of the river
/// landing is authored risk 1, and while the landing is held by a party the
/// terrace is not, that road costs `CONTESTED_RISK_MODIFIER` more -- exactly
/// what the jungle edge beside it costs. Releasing the landing puts it back.
/// The bridge's `route_options` projection reads `Geography::effective_risk`
/// for every drawn road, so this is the number the expedition screen draws;
/// `Project42ExpeditionBridge` needs a live Godot, and the GDScript suite is
/// what proves the button says it.
#[test]
fn holding_the_river_landing_contests_the_safe_road_and_releasing_it_does_not() {
    let geography = geography_as_godot_forwards_it();
    let mut state = ExpeditionState::new(
        42,
        vec![
            "character.protagonist.captain".into(),
            "character.heroine.betty".into(),
        ],
        "world.cell.river_landing",
    )
    .expect("the prototype's party constructs");

    let safe_road = geography
        .route("world.portal.river_landing_to_reception_terrace_safe_road")
        .expect("the authored safe road")
        .clone();
    let authored_risk = safe_road.risk_level;
    let uncontested = geography.effective_risk(&safe_road, &state.ownership);
    assert_eq!(
        uncontested, authored_risk,
        "a fresh campaign holds nothing, so no road is contested"
    );

    let events = state
        .set_control(
            "world.cell.river_landing",
            Some("faction.pirates".into()),
            &geography,
        )
        .expect("the river landing is a cell the authored world declares");
    assert_eq!(events.len(), 1, "one handover, one event");
    assert_eq!(
        geography.effective_risk(&safe_road, &state.ownership),
        uncontested + CONTESTED_RISK_MODIFIER,
        "holding one endpoint contests the safe road"
    );
    assert_eq!(
        geography.effective_risk(&safe_road, &state.ownership),
        geography.effective_risk(
            geography
                .route("world.portal.river_landing_to_reception_terrace_jungle_edge")
                .expect("the authored jungle edge"),
            &state.ownership,
        ) - CONTESTED_RISK_MODIFIER,
        "the jungle edge is contested by the same claim, so the two move together"
    );
    assert_eq!(
        safe_road.risk_level, authored_risk,
        "no route record was edited: the sum is derived, never stored"
    );

    state
        .set_control("world.cell.river_landing", None, &geography)
        .expect("releasing a held cell");
    assert_eq!(
        geography.effective_risk(&safe_road, &state.ownership),
        uncontested,
        "releasing the landing puts the safe road back to its authored risk"
    );

    assert!(
        state
            .set_control(
                "world.cell.nowhere",
                Some("faction.pirates".into()),
                &geography
            )
            .is_err(),
        "a cell the authored world does not declare cannot be claimed"
    );
}
