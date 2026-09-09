# Scenario packs

**Status:** implementation contract, 9 September 2026. Subordinate to
[GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md) and [ARCHITECTURE.md](ARCHITECTURE.md);
where they conflict, those win.

## Why

Pirate Island is one scenario of a game meant to run many: the same five-faction
RTS, the same embodied hero loop, the same simulation, on different islands with
different factions present, different starts, resources, items, triggers and
quests. Today the main scenario is compiled into the simulation: `world.rs`
reads its tuning through `include_str!`, `prototype_island()` hard-codes the
captain, and `install_preview_factions()` hard-codes which factions exist, where
their holdings stand and which production rules they run. A second scenario
would mean editing Rust. A mod would be impossible.

A **scenario pack** is the unit of authoring. The repository's own scenarios,
the scenario builder, and player-made mods all produce the same thing: a
directory of validated JSON that the simulation loads **at runtime** through the
bridge. The simulation stays one owner of the rules (`FactionWorld`); packs
supply the data those rules run on. Nothing in a pack is executable and nothing
in a pack can invent an outcome the simulation does not implement.

## The pack

```
content/scenarios/<scenario-id>/
  scenario.json              the manifest (below)
  ... referenced files       any file the manifest names, relative to this directory
```

`scenario.json`:

```json
{
  "schemaVersion": 1,
  "id": "scenario.pirate_island",
  "title": "Pirate Island",
  "summary": "The main scenario: five factions on one tropical island, Michael alone on the beach.",
  "geography": {
    "terrainTexture": "res://assets/island/terrain.png",
    "navigation": "navigation.json"
  },
  "start": {
    "captainId": "character.protagonist.captain",
    "combatProfile": { "health": 30, "damage": 4, "range": 4, "cooldown_ticks": 5 }
  },
  "factions": [
    {
      "id": "faction.pirates.prototype",
      "seed": [35, 15],
      "tuning": "factions/pirates.tuning.json",
      "production": "../../production/tide_quay_deckhands.json"
    }
  ],
  "rules": {
    "cthulhuMadness": "../../island/cthulhu_madness.json",
    "colonialExpansion": "../../island/colonial_expansion.json",
    "holdingRepairs": "../../island/holding_repairs.json",
    "holdingSalvage": "../../island/holding_salvage.json",
    "holdingDevelopment": "../../island/holding_development.json",
    "michaelFoothold": "../../island/michael_foothold.json",
    "michaelMachinery": "../../island/michael_machinery.json",
    "michaelMachineProduction": "../../production/michael_field_workshop_mechanical_dogs.json",
    "survivalDiplomacy": "../../island/survival_diplomacy.json",
    "initialDiplomacy": "../../diplomacy/initial_relationships.prototype.json"
  },
  "buildings": "../../../game/assets/island/buildings.json",
  "personas": "../../../game/assets/island/personas.json",
  "resources": [],
  "items": [],
  "triggers": [],
  "quests": []
}
```

Every string value under `geography.navigation`, `factions[].tuning`,
`factions[].production`, `rules.*`, `buildings` and `personas` is a path
relative to the pack directory; the referenced document is **embedded verbatim**
by the bundle builder. The manifest never restates a rule file: one owner per
document, referenced, not copied. `resources`, `items`, `triggers` and `quests`
are reserved for the next contracts (below) and must be empty arrays in schema
version 1; the validator refuses anything else so no pack can carry data the
simulation does not yet run.

## The bundle

`node tools/src/build-content-bundle.mjs` (extended, not a second tool) reads
every `content/scenarios/*/scenario.json`, resolves every path, embeds each
document, and writes one file per scenario:

```
game/generated/scenarios/<scenario-id>.json
```

with `format: "project42.scenario"`, `version: 1`, the resolved manifest, and a
`contentHash` over the canonical embedded document, exactly as the content
bundle does. The validator (`tools/src/validate.mjs`, one appended block) proves
before bundling that: every path resolves; every faction `id` exists in
`content/factions/`; every production rule's `producerArchetypeId` is a key of
the embedded `buildings` document and its `outputDefinitionId` a key of the
embedded `personas` document; every seed is inside the navigation `size`; no two
factions share a seed; the reserved arrays are empty; the scenario `id` is a
stable ID with the `scenario.` prefix and is registered like every other.

## The runtime

Rust gains one type, `ScenarioDefinition` (in `world.rs`, beside `FactionWorld`),
deserialized from the generated scenario document, and one constructor,
`FactionWorld::from_scenario(&ScenarioDefinition) -> Result<Self, String>`,
which does everything `prototype_island()` and `install_preview_factions()` do
today, from the definition instead of from `include_str!`. Every rule the
simulation reads at tick time (madness, expansion, repairs, salvage,
development, foothold, machinery, survival diplomacy, footprints, personas)
becomes a field of the world's saved state or of a `ScenarioRules` value the
world carries, so a save remembers which rules it was playing and a mod's rules
travel with its save. `include_str!` of scenario data leaves `world.rs`; the
tests that used it load the main scenario document from
`game/generated/scenarios/scenario.pirate_island.json` instead.

The bridge gains `create_island_from_scenario(payload: String) -> Dictionary`
(the scenario document text; returns the first snapshot or an `error`).
`create_island()` remains and means "the main scenario": the port reads
`res://generated/scenarios/scenario.pirate_island.json` and calls the new verb,
so no caller changes. `install_preview_factions()` is deleted with its callers
repaired: the scenario places the factions. Land rasterisation (polygon and
blocked rects to cells) stays where it is today, in GDScript feeding
`configure_island_land`, until a later contract moves it into Rust; the scene
reads the polygon from the scenario document's embedded navigation rather than
from `game/assets/island/navigation.json` directly, so the asset file becomes
the pack's referenced document and nothing else.

**Equality is the proof.** A test builds the world from the generated main
scenario and asserts it equals, field for field, the world the hard-coded path
built before the change, at tick zero and after 1,440 ticks, byte-identical
saves. The main scenario is then the only path and the hard-coded one is
deleted.

## Reserved: resources, items, triggers, quests

Each is its own contract and its own claim; this document reserves the keys so
the manifest shape does not change again.

- **Resources.** A catalogue of resource IDs (`resource.<name>`) with display
  names and storage rules; faction income, storage caps and costs reference the
  catalogue instead of free strings.
- **Items.** Records (`item.<name>`) with stack rules, an owner (an actor or a
  holding), and effects the simulation implements (a health bonus, a carbine
  upgrade, a key that unlocks a trigger). No item carries logic.
- **Triggers.** Deterministic rules evaluated in the island tick: a condition
  over world state (day, tick, a faction eliminated, a holding reaching a
  level, an actor entering a cell region, a recruitment, a flag) and an effect
  the simulation implements (a news line, a flag, an item granted, a diplomacy
  change, a spawn through a declared producer, a quest stage). Every trigger
  fires at most once unless it says `repeat`, and every firing is journalled
  with its cause.
- **Quests.** Stage machines whose transitions are triggers; a quest exposes
  its current stage and objective text through the snapshot; completion and
  failure are effects.

## The builder

`node tools/src/scaffold-scenario.mjs <scenario-id>` writes a pack skeleton
that already validates (every reserved array empty, the main scenario's rule
files referenced, a two-faction default). An in-game builder is a later
contract; the CLI is its data layer. A mod is a pack loaded from
`user://packs/<id>/` through the same bundle format; the bridge accepts the
document text and nothing else, so the loading path is one.

## What this contract does not decide

Balance numbers stay where their files say (provisional). Whether the land
polygon moves to Rust. The in-game builder's interface. Save migration for
scenarios with differing rules across versions (version 1 saves carry the rules
they were made with; a differing rule set on load is refused, not coerced).
