//! The generated main-scenario document, and the land the Rust proofs run on.
//!
//! `tools/src/build-content-bundle.mjs` writes the document from the validated
//! pack under `content/scenarios/pirate_island/`; the tests read that generated
//! file rather than a second copy of the island. Land is separate on purpose:
//! the shipped island's land comes from the scenario's navigation polygon,
//! rasterised in GDScript, so the Rust proofs supply their own deterministic
//! land instead of carrying a second rasteriser.

use crate::world::{IslandPoint, ScenarioDefinition};
use std::collections::BTreeSet;

/// The generated main-scenario document, verbatim.
pub fn main_scenario_document() -> &'static str {
    include_str!("../../game/generated/scenarios/scenario.pirate_island.json")
}

/// The main scenario with the land the Rust proofs run on.
pub fn main_scenario() -> ScenarioDefinition {
    let mut definition = ScenarioDefinition::from_document(main_scenario_document())
        .expect("generated main scenario document");
    assert!(definition.set_land(prototype_land(), IslandPoint { x: 8, y: 16 }));
    definition
}

/// The small elliptical island the simulation's proofs have always used. The
/// shipped island's land comes from the scenario's navigation polygon,
/// rasterised in GDScript; this fixture keeps the Rust proofs deterministic
/// without a second rasteriser.
pub fn prototype_land() -> BTreeSet<IslandPoint> {
    let mut land = BTreeSet::new();
    for x in 0..48 {
        for y in 0..32 {
            if (x - 24_i32).pow(2) * 196 + (y - 16_i32).pow(2) * 484 < 484 * 196 {
                land.insert(IslandPoint { x, y });
            }
        }
    }
    land
}

fn faction(id: &str, seed: [i32; 2], production: &str) -> serde_json::Value {
    let roster: serde_json::Value =
        serde_json::from_str(include_str!("../../content/island/faction_roster.json"))
            .expect("authored faction roster");
    serde_json::json!({
        "id": id,
        "seed": seed,
        "tuning": roster[id].clone(),
        "production": serde_json::from_str::<serde_json::Value>(production).expect("authored production rule"),
    })
}

macro_rules! document {
    ($path:literal) => {
        serde_json::from_str::<serde_json::Value>(include_str!($path)).expect("authored document")
    };
}

fn document(path: &str) -> serde_json::Value {
    match path {
        "../../game/assets/island/navigation.json" => {
            document!("../../game/assets/island/navigation.json")
        }
        "../../content/island/cthulhu_madness.json" => {
            document!("../../content/island/cthulhu_madness.json")
        }
        "../../content/island/colonial_expansion.json" => {
            document!("../../content/island/colonial_expansion.json")
        }
        "../../content/island/holding_repairs.json" => {
            document!("../../content/island/holding_repairs.json")
        }
        "../../content/island/holding_salvage.json" => {
            document!("../../content/island/holding_salvage.json")
        }
        "../../content/island/holding_development.json" => {
            document!("../../content/island/holding_development.json")
        }
        "../../content/island/michael_foothold.json" => {
            document!("../../content/island/michael_foothold.json")
        }
        "../../content/island/michael_machinery.json" => {
            document!("../../content/island/michael_machinery.json")
        }
        "../../content/production/michael_field_workshop_mechanical_dogs.json" => {
            document!("../../content/production/michael_field_workshop_mechanical_dogs.json")
        }
        "../../content/island/survival_diplomacy.json" => {
            document!("../../content/island/survival_diplomacy.json")
        }
        "../../content/diplomacy/initial_relationships.prototype.json" => {
            document!("../../content/diplomacy/initial_relationships.prototype.json")
        }
        "../../game/assets/island/buildings.json" => {
            document!("../../game/assets/island/buildings.json")
        }
        "../../game/assets/island/personas.json" => {
            document!("../../game/assets/island/personas.json")
        }
        other => panic!("unknown scenario document {other}"),
    }
}
