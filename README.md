# Project 42: Pirate Island RPG

This repository is the executable companion to the design bible. It begins with a deliberately narrow vertical slice: one side-view battle stage, four party cards, one expanded active heroine, one individual monster, functional commands, descriptive combat text, and deterministic rules.

## Language ownership

| Boundary | Owner | Rule |
| --- | --- | --- |
| Godot scene tree, input, UI, animation, audio and projection | GDScript | May display simulation state; must not invent gameplay outcomes. |
| Combat rules, RNG, world-day transition, midnight repopulation, death memory and save invariants | Rust | Owns authoritative state and emits typed events. Must not refer to Godot node paths. |
| Content schemas, validators, asset ledgers, build reports and authoring transforms | TypeScript | Runs offline. Must not become a second gameplay runtime. |
| Characters, skills, enemies, locations, art requirements and localization references | JSON | Uses stable IDs and metadata. Never uses a scene path as identity. |

The initial Godot shell uses a clearly marked mock simulation adapter because Godot is not installed in the current environment and the native Rust bridge has not been compiled into a GDExtension yet. The mock implements the same command/event shapes as the Rust crate. Replacing it is a boundary task, not a rewrite of UI logic.

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

Godot runtime validation remains pending until a Godot 4 executable is installed or attached.

