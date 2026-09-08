# Project 42: Pirate Island RPG

**Current art direction: detailed 2D pixel-art sprites at the approved elevated three-quarter perspective.** See [visual authority](docs/VISUAL_AUTHORITY.md).

This repository implements a HaremLit adventure on top of a living, multi-faction RTS simulation of Pirate Island. Michael and four adult female companions are five controllable heroes on one large fixed-view isometric board. The women actively investigate the island, bring competing theories and personal quest lines, and ask the player to direct, equip, protect, and support their work.

The canonical product contract is [docs/GAME_BUILD_PLAN.md](docs/GAME_BUILD_PLAN.md). It supersedes the earlier side-view route-and-battle direction. Existing battle, character, native-bridge, asset work remains useful technical evidence where it does not conflict with the new authority; implementation already present does not overrule the product decision.

## Language ownership

| Boundary | Owner | Rule |
| --- | --- | --- |
| Godot scene tree, input, UI, animation, audio and projection | GDScript | May display simulation state; must not invent gameplay outcomes. |
| Combat rules, RNG, world-day transition, midnight repopulation, death memory and save invariants | Rust | Owns authoritative state and emits typed events. Must not refer to Godot node paths. |
| Content schemas, validators, asset ledgers, build reports and authoring transforms | TypeScript | Runs offline. Must not become a second gameplay runtime. |
| Characters, skills, enemies, locations, art requirements and localization references | JSON | Uses stable IDs and metadata. Never uses a scene path as identity. |

The Godot shell loads `Project42SimulationBridge`, the native Rust GDExtension, and submits commands through `NativeSimulationPort`. The bridge owns the authoritative prototype battle and projects typed snapshots and ordered events into Godot dictionaries. A clearly marked mock remains available only when a debug build cannot load the extension; release startup refuses that fallback.

The existing side-view presentation fixture exposes Betty's D-through-SSS skills. It is retained as a historical test bed for typed command/event projection , not as the current camera or campaign-loop authority. New production work targets the fixed-view isometric island slice in [docs/VERTICAL_SLICE_BUILD_CONTRACT.md](docs/VERTICAL_SLICE_BUILD_CONTRACT.md).

Betty's current Magnific images are stored as component references under `work/art/magnific/betty/`. Their exact approval boundaries live in `content/art/betty.reference_ledger.json`: one image controls body and rendering direction; two contribute equipment and palette only. None is marked as final production art.

## Workspace map

- `game/` — Godot 4 project and GDScript presentation.
- `godot-rust/` — deterministic Rust domain library.
- `content/` — canonical authored data.
- `tools/` — TypeScript validation and reporting.
- `docs/` — implementation contracts for humans and coding agents.

## Shared professional asset platform

The public [Shared Game Environment Library](https://github.com/dancockrell/shared-game-environment-library) owns shared CC0 source packs and catalogs. This repository retains Pirate Island consumer admission and project-specific asset work. It contains only the selected artwork needed by the game. `content/art/shared_asset_ledger.json` is the
machine-validated local record for shared candidates and consumer admission;
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
