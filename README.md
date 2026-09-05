# Project 42: Pirate Island RPG

This repository is the executable companion to the design bible. The default entry is now the first chapter's authoritative Black Beach expedition route: Black Beach, Damaged Coastal Estate, River Landing, and Reception Terrace are projected from Rust campaign state. The existing side-view battle remains the Reception Terrace combat presentation slice: compact party cards, one expanded active heroine, one individual monster, functional commands, descriptive combat text, and deterministic rules.

## Language ownership

| Boundary | Owner | Rule |
| --- | --- | --- |
| Godot scene tree, input, UI, animation, audio and projection | GDScript | May display simulation state; must not invent gameplay outcomes. |
| Combat rules, RNG, world-day transition, midnight repopulation, death memory and save invariants | Rust | Owns authoritative state and emits typed events. Must not refer to Godot node paths. |
| Content schemas, validators, asset ledgers, build reports and authoring transforms | TypeScript | Runs offline. Must not become a second gameplay runtime. |
| Characters, skills, enemies, locations, art requirements and localization references | JSON | Uses stable IDs and metadata. Never uses a scene path as identity. |

The Godot shell loads native Rust GDExtensions and submits combat commands through `NativeSimulationPort` and expedition travel through `NativeExpeditionPort`. Rust owns the authoritative battle and campaign state, projecting typed snapshots into Godot dictionaries. The expedition screen has no gameplay mock fallback: if its bridge is unavailable, it displays a literal blocked-startup state rather than inventing a route.

The presentation fixture now exposes all seven of Betty's D-through-SSS skills. `Fatal Intercept` is visibly present but disabled because it is an automatic reaction, not a manual command. The remaining buttons drive deterministic mock event sequences so card focus, multi-target rescue, battlefield effects, revival and bonus-turn presentation can be built before the native bridge is attached. These fixtures are not a second rules implementation and are never release-authoritative.

Betty's current Magnific images are stored as component references under `work/art/magnific/betty/`. Their exact approval boundaries live in `content/art/betty.reference_ledger.json`: one image controls body and rendering direction; two contribute equipment and palette only. None is marked as final production art.

## Active implementation handoff

The project is being split deliberately rather than allowing backend work to
reshape the game screen. The systems/backend track owns deterministic campaign
state, command validation, saves, the Reception Terrace encounter, Midnight
Return, content validation and the first estate consequence. The frontend
track owns Godot scenes, theatrical battle composition, camera, input, 3D
asset review, animation, art and all player-facing layout.

The implementation-ready backend brief, dependency order and proof gates are
in [docs/CLAUDE_BACKEND_HANDOFF.md](docs/CLAUDE_BACKEND_HANDOFF.md). Start
there before changing simulation, content schemas, the native bridge or save
data. The brief deliberately prohibits fake UI state, visual redesign and
scene-owned authority in backend work.

## Workspace map

- `game/` — Godot 4 project and GDScript presentation.
- `godot-rust/` — deterministic Rust domain library.
- `content/` — canonical authored data.
- `tools/` — TypeScript validation and reporting.
- `docs/` — implementation contracts for humans and coding agents.

## Shared professional asset platform

This repository is also the home of the **shared tabletop asset platform** used
by Project 42, DR Companion, and future professional work. It is not a dumping
ground for downloaded models. `content/art/shared_asset_ledger.json` is the
machine-validated source of truth for every shared candidate and admitted asset;
`docs/SHARED_ASSET_PLATFORM.md` defines the legal, visual, technical, and
project-boundary rules.

The shared library concentrates on neutral, reusable physical vocabulary:
plants, terrain, stone, wood, roofs, roads, generic architecture, furniture,
travel props, materials, and neutral effects. Project-specific landmarks,
characters, race adapters, named weapons, story props, and local visual grammar
remain in their owning game. A paid source file is never assumed shareable merely
because both games can use the result.

## Validation

```powershell
cargo test --manifest-path godot-rust/Cargo.toml
./tools/build-native-bridge.ps1 -Configuration debug
cd tools
npm install
npm run validate
```

Godot 4.7.2 stable is installed as an ignored local tool under `.local-tools/`. Run `tools/verify-godot.ps1` to parse, instantiate and advance the main scene headlessly. The executable is not committed; pass `-GodotExecutable` to use another official Godot 4 build.
