# Development runbook

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. The main scene is already configured. Press F6/F5 to show the battle prototype.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; a visible enemy-intent line; and four Betty command buttons. `Guarded Strike` is functional. The other three visible commands are honestly marked as locked until their Rust resolvers exist. Clicking the active command sends a stable-ID command through `SimulationPort`, projects the ordered player action, shows the enemy's declared `Rushing Bite`, resolves it, then begins the next round.

## Current deliberate mock

`MockSimulationPort` is development-only. It exists because the native Rust bridge is a separate boundary milestone. It must be removed from release exports. Rust tests already own the first real `Guarded Strike` rule; the next bridge task serializes Godot commands into the Rust library and maps `BattleEvent` values back into Dictionaries.

`NativeSimulationPort` now defines the exact engine-side adapter and fails closed when the extension class is unavailable. See `docs/NATIVE_BRIDGE.md` for the pinned crate/API decision, exported class contract, library locations and acceptance gates. Do not switch the battle scene to the native adapter until those gates pass.

## Validate before every commit

Run `cargo fmt --manifest-path godot-rust/Cargo.toml -- --check`, `cargo test --manifest-path godot-rust/Cargo.toml`, and `node tools/src/validate.mjs`. The content validator rejects missing stable-ID relationships, heroine rosters without exactly seven bond skills, animation records without explicit action beats or safe framing, penned dinosaurs, non-individual prototype monster groups, and encounters that abandon the card-rail/full-body-active presentation contract.

After validation, run `npm run build:content` from `tools/` or `node tools/src/build-content-bundle.mjs` from the repository root. Commit `game/generated/content_bundle.json` whenever its source records change. `ContentCatalog` loads only that bundle and returns defensive copies so callers cannot mutate the catalog's authoritative definitions.

## Definition of done for replacing dummy art

Do not merely replace the colored rectangle with any image. Open `content/art/placeholders.json`, satisfy every listed gate, change the content reference to the final stable asset ID, set the placeholder record to deprecated or remove it through a reviewed migration, run `npm run validate`, and capture a 1920x1080 screenshot proving body, weapon and VFX safe margins.
