## Current art production decision — 8 September 2026

All game artwork is 2D only: authored sprites, sprite animation, painted backgrounds, tiles, portraits and flat effects. Use the existing shared art folder at `C:/Users/Admin/Documents/Codex/shared-game-environment-library` for reusable 2D kits, with compatible perspective, pixel density, palette, anchors and animation metadata across Cattle Trail/Cattle Drive, DR Companion and Pirate Island. Preserve each game's characters, setting and gameplay identity.

Do not create, purchase, import, restore, archive for later reuse, or bake sprites from 3D models. Earlier model, rig, mesh, material, body-builder and six-month-resumption plans are retired. New generated artwork uses the user-authorized built-in image generator; no external paid generation APIs. Shared reuse does not make unreviewed art automatically approved.

Inspect true transparency, sequential poses, stable ground pivots, equipment handedness, native-size readability and actual motion before admission. Keep game state, navigation, combat rules, accessibility and persistence authoritative; changing artwork never changes legal actions. Existing engine API names and historical validation records may mention 3D without authorizing 3D artwork.

This is production authority, not a claim that every existing runtime or binary has been converted. Legacy-asset deletion is a separate operation: automatic review rejected deletion commands, so deletion remains unverified here. Do not restore those assets or present them as production options.

# Development runbook

> **Prototype tooling scope:** commands and contracts below describe existing implementation. Follow [GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md) for current product direction. A passing battle/animation fixture does not establish completion of the autonomous RTS, faction relations, dual clocks, or companion campaign.

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. F5 launches the current 2D island. F6 launches whichever scene is open, so it is not interchangeable with F5.

Expected current entry: tropical island, Michael alone, the colonial watch fort,
and three provisional autonomous factions producing units and fighting.
Click land to travel; Space pauses/resumes; F5 saves and F9 loads while the game
has keyboard focus. These in-game shortcuts are separate from editor shortcuts.
Use the mouse wheel to zoom toward the cursor, and middle-drag to pan. The
camera keeps the fixed elevated perspective and stays inside the map bounds.
The **Michael** button (Home) centers on him at the current zoom; **Island**
(End) restores the overview. **+ / −** also work without a mouse wheel.
These controls remain available while paused and never issue movement orders.
Camera framing is local presentation state, not part of the campaign save.

Shift-click a pirate woman to inspect her. **Approach** walks Michael and his
active party into talking range, following her movement along reachable land.
This order can be queued while paused and survives saving/loading. Click land
to choose another destination, or aim the carbine, to cancel the approach.
She keeps participating in her faction's war during the approach. On arrival,
the game pauses normally so she does not walk away before you can respond.
It does not automatically talk or recruit for you. Once Michael is close, Talk, then use
Join faction for the current authored offer. Choose an explicit companion slot;
joining the faction does not automatically fill one. Land clicks then move
Michael and living active companions through the same navigation system.
The current scene remains a development slice: standing sprites, incomplete
building art, provisional faction rules and one simple recruitment encounter.
Four-slot membership and dead identities save/load; full romance, diplomacy
and midnight resurrection are not implemented. See
[island implementation notes](../game/assets/island/README.md).

For a bounded low-resource check without opening an editor or visible window:

```powershell
./tools/verify-godot.ps1
```

The scene test resolves the configured main scene, then verifies native travel,
pause, collision, production/combat and persistence. This does not render an art
approval screenshot. Build the native extension after Rust or embedded building
contract changes. The verifier now runs only the current directional-sprite and
island suites, with a 30-second wall-clock limit per process and isolated test
profiles. It rejects script errors even when Godot returns exit code zero, and
requires a final completion message. It no longer opens an editor or loads
retired 3D fixtures. PowerShell 7 is required for the bounded process runner.

## Actual island render capture (explicit approval required)

Headless mode disables rendering, so it cannot provide screenshot evidence.
When a brief rendering session is approved, the existing project can capture
its real island viewport at 640x480 with a 15 FPS cap, after 60 deterministic
simulation ticks. This opens no editor but does require a rendering window.
Do not run it under the standing headless-only restriction without approval.

```powershell
& ./.local-tools/godot-4.7.2/Godot_v4.7.2-stable_win64_console.exe --path game --rendering-method gl_compatibility --audio-driver Dummy --script res://tests/island_render_capture.gd --quit-after 120
```

The PNG and adjacent JSON are saved to the Godot user directory. Override with
`-- --output=<fresh-absolute-path.png>`; existing evidence is never overwritten.
The JSON records simulation tick/state, engine, image hash and source-art hashes.
Inspect sprite scale, ground pivots, fort/wall placement, interface readability
and the actual pixel edges before recording a visual verdict. This capture is
not an animation proof. The tool deliberately exits with code 2 in headless mode.

## Historical battle fixture (retained)

Open `game/scenes/battle/battle_prototype.tscn` explicitly to inspect the retained
battle experiment. It is not the game entry or the current visual authority.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; a visible enemy-intent line; and all seven Betty skills in a two-row command grid. `Fatal Intercept` is visible but disabled because it is an automatic reaction. Manual skills enter the targeting session, prompt for legal targets in authored order, submit stable IDs through `SimulationPort`, play the authored action beats, then project the returned mechanical events.

## Shelved Betty 3D candidate review (historical)

Open `game/scenes/review/betty_3d_candidate_review.tscn` and press F6. This is
an isolated camera-and-silhouette review, not a second battle scene. It uses
the downloaded Magnific GLB at the same 1920×1080 active-fighter crop, fits
the character's geometric bounds to the floor and marks the fifteen-percent
effect envelope. The metadata panel is intentionally blunt: candidate 01 has
no skeleton and no animation clips, so it cannot replace the live Betty or
stand in for a weapon-socket/skill test. Use this scene only to decide whether
the 3D visual direction blocks better than the 2D proxy before commissioning a
rigged export.

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
