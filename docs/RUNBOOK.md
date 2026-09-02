# Development runbook

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. The main scene is already configured. Press F6/F5 to show the battle prototype.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; a visible enemy-intent line; and all seven Betty skills in a two-row command grid. `Fatal Intercept` is visible but disabled because it is an automatic reaction. Manual skills enter the targeting session, prompt for legal targets in authored order, submit stable IDs through `SimulationPort`, play the authored action beats, then project the returned mechanical events.

## Native simulation boundary

`Project42SimulationBridge` is the active authoritative simulation when its GDExtension is registered. Build it with `tools/build-native-bridge.ps1`, then run `tools/verify-godot.ps1`; the verifier performs an editor import pass before starting the scene and tests the real bridge through `NativeSimulationPort`.

`MockSimulationPort` remains a debug-only presentation fixture for work on machines that cannot build Rust. `BattlePrototype` selects it only when `OS.is_debug_build()` is true and the bridge class is unavailable. A release runtime exits with an error instead of silently running duplicate GDScript rules. See `docs/NATIVE_BRIDGE.md` for the pinned crate/API decision, exported class contract, library locations and acceptance gates.

## Validate before every commit

Run `cargo fmt --manifest-path godot-rust/Cargo.toml -- --check`, `cargo test --manifest-path godot-rust/Cargo.toml`, and `node tools/src/validate.mjs`. The content validator rejects missing stable-ID relationships, heroine rosters without exactly seven bond skills, animation records without explicit action beats or safe framing, penned dinosaurs, non-individual prototype monster groups, and encounters that abandon the card-rail/full-body-active presentation contract.

Run `.\tools\verify-godot.ps1` to import and register extensions, instantiate and advance the configured main scene, execute the presentation suites, then submit both rejected and accepted commands through the native bridge. The verifier redirects Godot's per-user directories to task-specific temporary folders and fails on a nonzero engine exit. Its default is the ignored local Godot 4.7.2 stable build; pass `-GodotExecutable` to check another Godot 4 build. A root-certificate-store warning can appear inside a restricted Windows sandbox. The prototype performs no runtime network access, so that warning is not a scene failure.

After validation, run `npm run build:content` from `tools/` or `node tools/src/build-content-bundle.mjs` from the repository root. Commit `game/generated/content_bundle.json` whenever its source records change. `ContentCatalog` loads only that bundle and returns defensive copies so callers cannot mutate the catalog's authoritative definitions.

Presentation registries under `content/presentation/` are bundled content. Camera entries use stable `presentation.camera.*` IDs and must define mode, zoom, focus subjects, lead, at least eight-percent safe padding and bounded transition times. Do not type a new camera name directly into a skill. Add or deliberately reuse a registry entry, reference it through `cameraId`, rebuild the bundle and let the validator prove the relationship.

VFX entries use stable `presentation.vfx.*` IDs. Every entry must define its communicative purpose, palette, anchor, layer, blend, safe-frame envelope, motion, lifetime, reduced-flash behavior and asset status. Do not type a new effect label directly into a skill. Add or reuse a registry entry, reference it through `vfxId`, rebuild the bundle and verify the live dummy cue shows the resolved anchor, envelope and fallback behavior. An effect whose art is unfinished remains `placeholder`; `not_required` is reserved for the deliberate no-effect record.

Magnific animation candidates use stable `presentation.motion.*` IDs. Preserve the creation URL and generation settings, record the intended usage and complete rejection gates, and leave the record as `remote_review_candidate` until the exact file is copied beneath `game/` and assigned a `res://` runtime path. Review the first frame, midpoint and last frame at battle scale. Reject identity drift, changing anatomy or costume, bent or duplicated weapons, camera motion, any crop, and a visible first-to-last loop jump. Only then change the import state to `imported`; approval is a separate reviewed decision.

Preserve downloaded H.264 originals under `work/art/` with their SHA-256 hashes. They are production sources, not runtime imports. Transcode an approved source to the runtime format selected for the target Godot build, place that derivative beneath `game/assets/`, set `runtimePath`, and change `importState` to `imported`. Never overwrite the Magnific original during transcode or cleanup.

## Definition of done for replacing dummy art

Do not merely replace the colored rectangle with any image. Open `content/art/placeholders.json`, satisfy every listed gate, change the content reference to the final stable asset ID, set the placeholder record to deprecated or remove it through a reviewed migration, run `npm run validate`, and capture a 1920x1080 screenshot proving body, weapon and VFX safe margins.
