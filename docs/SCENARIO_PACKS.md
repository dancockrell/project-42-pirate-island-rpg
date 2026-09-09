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
  "items": "items.json",
  "triggers": [],
  "quests": []
}
```

Every string value under `geography.navigation`, `factions[].tuning`,
`factions[].production`, `rules.*`, `buildings` and `personas` is a path
relative to the pack directory; the referenced document is **embedded verbatim**
by the bundle builder. The manifest never restates a rule file: one owner per
document, referenced, not copied. `items` names the pack's item catalogue the
same way, or stays `[]` when the pack carries none ([Items](#items) below).
`resources`, `triggers` and `quests` are reserved for the next contracts (below)
and must be empty arrays in schema version 1; the validator refuses anything
else so no pack can carry data the simulation does not yet run.

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

## Items

Implemented. The manifest's `items` key is either the empty array — this pack
carries none, and its world is exactly the world without items — or a path to
the pack's catalogue document, embedded verbatim like `buildings` and
`personas`:

```
content/scenarios/<scenario-id>/items.json
```

a keyed document of item records:

```json
{
  "weapon.captain.handsome_jack_steam_carbine": {
    "display_name": "Handsome Jack steam carbine",
    "stack": "unique",
    "effect": {"kind": "combat_bonus", "health": 0, "damage": 2, "range": 2}
  }
}
```

A record's key is a stable ID whose **prefix names its domain**, as
[ARCHITECTURE.md](ARCHITECTURE.md) requires of every identifier: a weapon uses
`weapon.`, and a record that already exists elsewhere in the repository keeps
the ID it already has rather than gaining an `item.` twin. `stack` is
`"unique"` (at most one per actor) or `{"stackable": n}` for `n` in 1 to 64.

`effect` names **one of a closed set the simulation implements**. There are two
and a pack may use no others:

- `{"kind": "grant_resource", "resource": "<resource id>", "amount": n}` —
  credited once to the holder's faction when the item is granted, through the
  same storage clamp the tick's income uses. The resource key is a plain string
  until the resource catalogue contract owns it.
- `{"kind": "combat_bonus", "health": n, "damage": n, "range": n}` — raises the
  holder's own combat numbers for as long as the item is held.

`ScenarioRules` carries the catalogue, so a save remembers the items it was
playing and a mod's items travel with its save. Inventories are a side table on
`FactionWorld` keyed by actor id, like `positions` and `unit_combat`: an actor's
items therefore move with the person, and neither recruitment
(`recruit_island_person`) nor leaving the active party can strip her equipment,
which is what [the character contract](CHARACTER_AND_HAREMLIT_AUTHORING.md)
requires. An empty inventory writes nothing to the save. Load refuses, as
`invalid_saved_inventory`, an inventory over 4,096 actors or 32 items per actor,
a key naming an actor the world does not have living or fallen, an item no
catalogue entry defines, or a stack over its record's rule.

`IslandCombatProfile` is keyed by **definition**, so `combat_bonus` cannot
change it. `FactionWorld::actor_combat_profile` resolves one actor's numbers on
read — the definition's profile raised by what that actor carries. It is the
only answer to an actor's combat numbers, so nothing holds a second copy that
could fall out of step with the inventory.

`FactionWorld::grant_item(actor, item)` and `FactionWorld::actor_inventory(actor)`
are the only doors. Items reach the world **only through a declared source**;
today that source is a direct grant, and triggers become a source of their own
under their own contract. `grant_item` refuses by name: `unknown_item_actor`,
`unknown_item`, `item_stack_full`, `inventory_full`. The validator proves before
bundling that every record's ID, stack rule and effect are ones the simulation
runs, and that a character a scenario starts has its `signatureWeaponId` defined
in that scenario's catalogue, so a signature weapon cannot dangle unreferenced.

The per-actor snapshot entry carries `inventory` — an array of `{id, name}` in
the order the items were granted — and `max_health` is the resolved profile's,
so an inspection panel reads equipment and its effect from the snapshot without
keeping a second table of either.

## Reserved: resources, triggers, quests

Each is its own contract and its own claim; this document reserves the keys so
the manifest shape does not change again.

- **Resources.** A catalogue of resource IDs (`resource.<name>`) with display
  names and storage rules; faction income, storage caps and costs reference the
  catalogue instead of free strings.
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

Balance numbers stay where their files say (provisional) — including the
carbine's bonus magnitudes and the world's inventory caps, which are the item
contract's own provisional numbers. Whether the land
polygon moves to Rust. The in-game builder's interface. Save migration for
scenarios with differing rules across versions (version 1 saves carry the rules
they were made with; a differing rule set on load is refused, not coerced).
