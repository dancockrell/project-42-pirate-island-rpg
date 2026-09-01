# Development runbook

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. The main scene is already configured. Press F6/F5 to show the battle prototype.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; a visible enemy-intent line; and all seven Betty skills in a two-row command grid. `Fatal Intercept` is visible but disabled because it is an automatic reaction. Manual skills enter the targeting session, prompt for legal targets in authored order, submit stable IDs through `SimulationPort`, play the authored action beats, then project the returned mechanical events.

## Current deliberate mock

`MockSimulationPort` is development-only. It exists because the native Rust bridge is a separate boundary milestone. It must be removed from release exports. Rust tests already own the first real `Guarded Strike` rule; the next bridge task serializes Godot commands into the Rust library and maps `BattleEvent` values back into Dictionaries.

`NativeSimulationPort` now defines the exact engine-side adapter and fails closed when the extension class is unavailable. See `docs/NATIVE_BRIDGE.md` for the pinned crate/API decision, exported class contract, library locations and acceptance gates. Do not switch the battle scene to the native adapter until those gates pass.

## Validate before every commit

Run `cargo fmt --manifest-path godot-rust/Cargo.toml -- --check`, `cargo test --manifest-path godot-rust/Cargo.toml`, and `node tools/src/validate.mjs`. The content validator rejects missing stable-ID relationships, heroine rosters without exactly seven bond skills, animation records without explicit action beats or safe framing, penned dinosaurs, non-individual prototype monster groups, and encounters that abandon the card-rail/full-body-active presentation contract.

Run `.\tools\verify-godot.ps1` to parse, instantiate and advance the configured main scene for three headless frames, then execute the Godot targeting-session test suite. The verifier redirects Godot's per-user directories to task-specific temporary folders and fails on a nonzero engine exit. Its default is the ignored local Godot 4.7.2 stable build; pass `-GodotExecutable` to check another Godot 4 build. A root-certificate-store warning can appear inside a restricted Windows sandbox. The prototype performs no runtime network access, so that warning is not a scene failure.

After validation, run `npm run build:content` from `tools/` or `node tools/src/build-content-bundle.mjs` from the repository root. Commit `game/generated/content_bundle.json` whenever its source records change. `ContentCatalog` loads only that bundle and returns defensive copies so callers cannot mutate the catalog's authoritative definitions.

Presentation registries under `content/presentation/` are bundled content. Camera entries use stable `presentation.camera.*` IDs and must define mode, zoom, focus subjects, lead, at least eight-percent safe padding and bounded transition times. Do not type a new camera name directly into a skill. Add or deliberately reuse a registry entry, reference it through `cameraId`, rebuild the bundle and let the validator prove the relationship.

## Definition of done for replacing dummy art

Do not merely replace the colored rectangle with any image. Open `content/art/placeholders.json`, satisfy every listed gate, change the content reference to the final stable asset ID, set the placeholder record to deprecated or remove it through a reviewed migration, run `npm run validate`, and capture a 1920x1080 screenshot proving body, weapon and VFX safe margins.
