# Development runbook

> **Prototype tooling scope:** commands and contracts below describe existing implementation. Follow [GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md) for current product direction. A passing battle/animation fixture does not establish completion of the autonomous RTS, faction relations, dual clocks, or companion campaign.

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. The main scene is already configured. Press F6/F5 to show the battle prototype.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; a visible enemy-intent line; and all seven Betty skills in a two-row command grid. `Fatal Intercept` is visible but disabled because it is an automatic reaction. Manual skills enter the targeting session, prompt for legal targets in authored order, submit stable IDs through `SimulationPort`, play the authored action beats, then project the returned mechanical events.

## Betty 3D candidate review

Open `game/scenes/review/betty_3d_candidate_review.tscn` and press F6. This is
an isolated camera-and-silhouette review, not a second battle scene. It uses
the downloaded Magnific GLB at the same 1920×1080 active-fighter crop, fits
the character's geometric bounds to the floor and marks the fifteen-percent
effect envelope. The metadata panel is intentionally blunt: candidate 01 has
no skeleton and no animation clips, so it cannot replace the live Betty or
stand in for a weapon-socket/skill test. Use this scene only to decide whether
the 3D visual direction blocks better than the 2D proxy before generating a
rigged export.

## Native simulation boundary

`Project42SimulationBridge` is the active authoritative simulation when its GDExtension is registered. Build it with `tools/build-native-bridge.ps1` on Windows or `tools/build-native-bridge.sh` on Linux and macOS, then run the matching verifier (`tools/verify-godot.ps1` or `tools/verify-godot.sh`); the verifier performs an editor import pass before starting the scene and tests the real bridge through `NativeSimulationPort`.

`MockSimulationPort` remains a debug-only presentation fixture for work on machines that cannot build Rust. `BattlePrototype` selects it only when `OS.is_debug_build()` is true and the bridge class is unavailable. A release runtime exits with an error instead of silently running duplicate GDScript rules. See `docs/NATIVE_BRIDGE.md` for the pinned crate/API decision, exported class contract, library locations and acceptance gates.

## Validate before every commit

Run `cargo fmt --manifest-path godot-rust/Cargo.toml -- --check`, `cargo test --manifest-path godot-rust/Cargo.toml`, and `node tools/src/validate.mjs`. The content validator rejects missing stable-ID relationships, heroine rosters without exactly seven bond skills, animation records without explicit action beats or safe framing, penned dinosaurs, non-individual prototype monster groups, and encounters that abandon the card-rail/full-body-active presentation contract.

### The two verification gates

There are two gates and one contract. Both run the same sequence against the same
project; use the one that matches the machine.

| Gate | Where | Build step | Verifier |
|---|---|---|---|
| PowerShell | Windows developer machines | `.\tools\build-native-bridge.ps1` | `.\tools\verify-godot.ps1` |
| Bash | Linux, macOS, and CI | `tools/build-native-bridge.sh` | `tools/verify-godot.sh` |

They are twins, not alternatives: the same commands in the same order, the same
failure behaviour, the same messages. Change one and change the other in the same
commit, or they will drift and one of them will be lying.

**The sequence both verifiers run**, stopping at the first non-zero engine exit and
exiting with that engine's exit code:

1. `--headless --editor --path game --quit` — import resources and register extensions.
2. `--headless --path game --quit-after 3` — instantiate and advance the configured main scene.
3. one `--headless --path game --scene res://scenes/review/<name>.tscn --quit-after 2` per review scene.
4. one `--headless --path game --script res://tests/<name>.gd` per suite under `game/tests/`.

The shell verifier enumerates steps 3 and 4 by globbing `game/scenes/review/*_review.tscn`
and `game/tests/*_test.gd`, and echoes each file as it runs, so a new scene or suite is
covered the day it lands instead of the day someone remembers to edit a list. Exit
status is zero only when every step passed.

**Finding Godot.** The shell verifier uses `$GODOT` if set, otherwise the first of
`godot4` or `godot` on `PATH`. With no engine it exits non-zero with a message naming
`GODOT`; it never passes silently. The PowerShell verifier instead defaults to the
ignored local `.local-tools/godot-4.7.2` build and takes `-GodotExecutable`.

**Per-user directories.** Both verifiers redirect Godot's per-user directories to
task-specific temporary folders so a run cannot inherit or pollute a developer
profile — `APPDATA`/`LOCALAPPDATA` on Windows, the `XDG_*` variables on Linux and macOS.

**Building the bridge.** Both build scripts run
`cargo build --manifest-path godot-rust/Cargo.toml --features godot-ext` (add `release`
as the argument, or `-Configuration release`, for the release profile) and copy the
produced library into `game/bin/<platform>/`. The library differs per platform:
`project42_sim.dll` on Windows, `libproject42_sim.so` on Linux,
`libproject42_sim.dylib` on macOS. Note that `game/bin/project42_sim.gdextension`
currently declares `windows.*` libraries only, so on Linux and macOS the library is
built and installed but Godot does not yet register the extension; the suites then run
against `MockSimulationPort` in a debug build. Adding the non-Windows entries is E2's
work, not the build script's.

A root-certificate-store warning can appear inside a restricted Windows sandbox. The
prototype performs no runtime network access, so that warning is not a scene failure.

After validation, run `npm run build:content` from `tools/` or `node tools/src/build-content-bundle.mjs` from the repository root. Commit `game/generated/content_bundle.json` whenever its source records change. `ContentCatalog` loads only that bundle and returns defensive copies so callers cannot mutate the catalog's authoritative definitions.

Presentation registries under `content/presentation/` are bundled content. Camera entries use stable `presentation.camera.*` IDs and must define mode, zoom, focus subjects, lead, at least eight-percent safe padding and bounded transition times. Do not type a new camera name directly into a skill. Add or deliberately reuse a registry entry, reference it through `cameraId`, rebuild the bundle and let the validator prove the relationship.

VFX entries use stable `presentation.vfx.*` IDs. Every entry must define its communicative purpose, palette, anchor, layer, blend, safe-frame envelope, motion, lifetime, reduced-flash behavior and asset status. Do not type a new effect label directly into a skill. Add or reuse a registry entry, reference it through `vfxId`, rebuild the bundle and verify the live dummy cue shows the resolved anchor, envelope and fallback behavior. An effect whose art is unfinished remains `placeholder`; `not_required` is reserved for the deliberate no-effect record.

Magnific animation candidates use stable `presentation.motion.*` IDs. Preserve the creation URL and generation settings, record the intended usage and complete rejection gates, and leave the record as `remote_review_candidate` until the exact file is copied beneath `game/` and assigned a `res://` runtime path. Review the first frame, midpoint and last frame at battle scale. Reject identity drift, changing anatomy or costume, bent or duplicated weapons, camera motion, any crop, and a visible first-to-last loop jump. Only then change the import state to `imported`; approval is a separate reviewed decision.

Preserve downloaded H.264 originals under `work/art/` with their SHA-256 hashes. They are production sources, not runtime imports. Transcode an approved source to the runtime format selected for the target Godot build, place that derivative beneath `game/assets/`, set `runtimePath`, and change `importState` to `imported`. Never overwrite the Magnific original during transcode or cleanup.

## Definition of done for replacing dummy art

Do not merely replace the colored rectangle with any image. Open `content/art/placeholders.json`, satisfy every listed gate, change the content reference to the final stable asset ID, set the placeholder record to deprecated or remove it through a reviewed migration, run `npm run validate`, and capture a 1920x1080 screenshot proving body, weapon and VFX safe margins.
