# Project 42: Pirate Island RPG

This repository is the executable companion to the design bible. It begins with a deliberately narrow vertical slice: one side-view battle stage, four party cards, one expanded active heroine, one individual monster, functional commands, descriptive combat text, and deterministic rules.

## Language ownership

| Boundary | Owner | Rule |
| --- | --- | --- |
| Godot scene tree, input, UI, animation, audio and projection | GDScript | May display simulation state; must not invent gameplay outcomes. |
| Combat rules, RNG, world-day transition, midnight repopulation, death memory and save invariants | Rust | Owns authoritative state and emits typed events. Must not refer to Godot node paths. |
| Content schemas, validators, asset ledgers, build reports and authoring transforms | TypeScript | Runs offline. Must not become a second gameplay runtime. |
| Characters, skills, enemies, locations, art requirements and localization references | JSON | Uses stable IDs and metadata. Never uses a scene path as identity. |

The initial Godot shell uses a clearly marked mock simulation adapter because the native Rust bridge has not been compiled into a GDExtension yet. The mock implements the same command/event shapes as the Rust crate. Replacing it is a boundary task, not a rewrite of UI logic.

The presentation fixture now exposes all seven of Betty's D-through-SSS skills. `Fatal Intercept` is visibly present but disabled because it is an automatic reaction, not a manual command. The remaining buttons drive deterministic mock event sequences so card focus, multi-target rescue, battlefield effects, revival and bonus-turn presentation can be built before the native bridge is attached. These fixtures are not a second rules implementation and are never release-authoritative.

Betty's current Magnific images are stored as component references under `work/art/magnific/betty/`. Their exact approval boundaries live in `content/art/betty.reference_ledger.json`: one image controls body and rendering direction; two contribute equipment and palette only. None is marked as final production art.

## Workspace map

- `game/` — Godot 4 project and GDScript presentation.
- `godot-rust/` — deterministic Rust domain library.
- `content/` — canonical authored data.
- `tools/` — TypeScript validation and reporting.
- `docs/` — implementation contracts for humans and coding agents.

## Validation

```powershell
cargo test --manifest-path godot-rust/Cargo.toml
cd tools
npm install
npm run validate
```

Godot 4.7.2 stable is installed as an ignored local tool under `.local-tools/`. Run `tools/verify-godot.ps1` to parse, instantiate and advance the main scene headlessly. The executable is not committed; pass `-GodotExecutable` to use another official Godot 4 build.
