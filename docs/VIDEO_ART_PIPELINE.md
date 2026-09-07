# Video art mining pipeline

## Current sprite-source inspection

### Rich-kit production priority — 7 September 2026

Adopt the approved-art-first method conveyed by the Cattle Trail task: preserve originals, extract reproducibly, author exact atlas/animation metadata, inspect actual pixels and native engine motion, then prove one playable scene before widening room or feature scope. Cattle Trail's METHOD.md, kits/DELIVERY.md, manifest structure and overview were inspected. Its reported 496 candidates and engine delivery are another project's evidence, not Pirate Island's counts or approval. No Western assets or cattle mechanics are imported. Its full playback was not independently audited here.

Aim for roughly sixteen times richer **useful** coverage against a measured baseline where one exists. Directions, actions, functional states and composable environment modules count; duplicate cells, mirrored copies, recolor padding and redundant exports do not. A numerical quota must not encourage filler. Keep planned, extracted, visually admitted and integrated counts separate. No automatic multiplication when the denominator is zero.

Current inspected baseline: `game/assets/` contains one PNG backdrop stand-in and no pixel-character atlas. `content/characters/` contains `character.protagonist.captain` and `character.heroine.betty`; `content/enemies/` contains `enemy.raptor.razorbeak.prototype`. These three records are the first bounded kit candidates, not a complete island cast inventory. The approved Cattle Trail images/video are style references, not island runtime art. Verified approved/integrated pixel-character frames in this scope: **0**. A sixteen-times ratio is therefore undefined. Wider source recovery remains open; this is not a claim that no other assets exist elsewhere.

First character pass: approve one source sprite for each of those existing identities in the reference perspective; then develop coherent facings, locomotion, role-specific attack/use, reaction, defeat and idle. Obtain actual needed poses instead of pretending mirrored facing or repeated idle frames fill gaps. Keep adult identity, period cut, equipment and faction readability stable. Inventory all other authored units, heroes, props and buildings through their existing content IDs before declaring full coverage; metadata alone does not count as completed art.

Each scenery family gets its own rich kit: tropical grass/ground cover; trees and palms; understory/vines; rocks and cliff edges; shoreline/water; roads/bridges; ruins; faction buildings; camp/harbour/workshop props; faction corruption and weather effects. Supply useful silhouette, maturity, density and condition variation. Trees require genuine canopy/trunk occlusion support, not counting two exports of one drawing as two variants. Roads, shores, walls and gates require authored ends, corners, joins, elevation transitions, anchors, collision footprints and open/broken states as applicable. A decorative atlas does not establish seamless or traversable connections. Faction variants must preserve each faction's own aesthetic within the common pixel language.

Stay on one representative tropical scene while kits expand. Prove interaction through existing simulation commands and persistent consequences, not a second gameplay model. No 3D work, paid generation or broad new room/feature production is authorized by this method handoff.

The inspector now reports `distinctDrawingsIgnoringHorizontalMirrors`: visible content is compared independently of transparent margins, hidden RGB values and horizontal reflection. Legal repeated animation holds remain allowed but do not increase this count. Distinct pixels still do not prove distinct useful states, correct facing or recolor-free coverage; `coverageAdmission` remains unassessed. Ten regression tests pass, including repeated-hold and mirrored-drawing accounting. No new art was extracted or admitted in this checkpoint.

Before treating a generated sheet as runtime sprites, run `python tools/inspect_sprite_source.py <source.png> --manifest <frames.json>` from the repository root, using a Python environment with Pillow. Run `python tools/test_sprite_source.py` for the synthetic regression suite. The inspector is read-only: it never cleans, repaints, slices or overwrites source art. Omitting the manifest produces a diagnostic report and a failed admission result, not guessed frame coordinates.

The authored JSON contract is `schemaVersion: 1`, `sourceSha256` for the exact PNG, and `frames`. Each frame requires a unique `id`, `action`, `direction`, integer pixel `rect: [x,y,width,height]`, frame-local integer `footPivot: [x,y]`, positive `durationSeconds` and explicit boolean `loop`. Example frame: `{"id":"idle.south.0","action":"idle","direction":"south","rect":[0,0,64,96],"footPivot":[32,96],"durationSeconds":0.125,"loop":true}`. These sample dimensions are illustrative, not approved sprite scale. Rectangles may be irregular; the tool does not assume the reference sheet uses a uniform grid.

Structural admission checks exact source identity, PNG bounds, actual transparency, visible frame content, valid pivots, timing and unique frame IDs. It does not certify character likeness, coherent action order, palette, pixel-grid quality, sorting or loop continuity. A structural pass always leaves `artAdmission: not-assessed`. Assets are not automatically added to runtime variation pools.

Checkpoint: the approved 1254 × 1254 character reference has **zero transparent pixels** and no authored frame metadata. It correctly fails cutout-atlas admission while remaining the approved style reference. Original bytes are preserved. Eight regression tests passed, including opaque-checkerboard rejection, invalid frame fields, empty frames and read-only source preservation. No new sprites, animation or playable scene are claimed by this tool.

**7 September 2026:** [Current visual authority](VISUAL_AUTHORITY_AND_3D_ENTRY_GATE.md) requires 2D pixel sprites and shelves 3D art. The older reel slate and rates below are historical tooling guidance, not approved current prompts or mandatory sprite frame counts. No 3D turns, model jobs or orbit-camera assets for Pirate Island. Generated frames require sprite identity, pivot, direction, pixel-grid and temporal checks before admission.

The canonical reel slate is `content/art/video_reel_plan.json`. Each `prompt` is copied into Magnific unchanged. Metadata stays outside the prompt.

## Production flow

1. Inspect the named source anchor at full resolution.
2. Reject an anchor containing text, signatures, watermarks, cropped required subjects or identity drift.
3. Generate one reel with the recorded duration and 16:9 frame.
4. Preserve the downloaded vendor source beneath `work/art/vendor/model/reel-id/`.
5. Record the creation URL, model, credit spend, generation date and source SHA-256.
6. Harvest with the reel's `extractionFps` and `maxFrames`.
7. Inspect `contact-sheet.jpg` and `manifest.json`.
8. Mark accepted frames `approved_source`, weak frames `rejected`, and malformed frames `quarantine`.
9. Copy approved source frames beneath `work/art/approved/reel-id/` without deleting the raw source.
10. Create engine derivatives beneath `game/assets/` with stable asset IDs and runtime manifests.

## Harvester command

FFmpeg and FFprobe must be on `PATH`. Use the bundled Python runtime with Pillow and NumPy:

```powershell
& 'C:\Users\Admin\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' 'C:\Users\Admin\.codex\skills\video-frame-harvester\scripts\harvest_frames.py' <source.mp4> <output-directory> --fps <6|12|24> --max-frames <limit> --source-creation <magnific-creation-url> --source-model <model>
```

Character action reels use 24 fps. Route, orbit and look-around environment reels use 12 fps. Atmospheric holds use 6 fps. The harvester performs triage, scoring and perceptual deduplication; it does not grant artistic approval.

## Admission rules

Character frames retain face, silhouette, proportions, hair, costume, hands, equipment, handedness and palette. Required body parts, weapons and effects remain inside the safe frame. Reject sliding feet, unstable pivots, deformed hands, weapon mutation, extra subjects, motion smear and inconsistent identity.

Environment frames retain geography, architecture, terrain, weather, light direction, landmarks and regional identity. Reject warped openings, duplicated structures, dissolving roads, discontinuous horizons, moving landmarks, text, signatures and generic scenery that contradicts the named location.

VFX plates contain one complete effect on black, with no subject, environment, text or edge crop. Store the black-background source unchanged. Alpha extraction and additive-blend derivatives are separate transformations with their own hashes.

## Reel-level rejection

Reject an environment reel when topology changes or more than one-third of candidate frames show motion damage. Reject a character reel when identity changes, the signature weapon mutates, required anatomy leaves frame, or the action cannot be read in three selected keyframes. Reject a loop when its first and last poses do not match closely enough for game playback.
# Runtime integration checkpoint — 7 September 2026

Michael's shared generated standing source now has an authored four-direction
manifest at `game/assets/sprites/michael/frames.json` and an actual Godot
presentation consumer at `game/scripts/world/directional_sprite.gd`.
The source is copied byte-for-byte from the shared library; its SHA256 is checked
before loading. This copy is a review payload, not a second source-art authority.

Run `game/scenes/review/sprite_review.tscn` for the four-facing scale review;
`game/tests/directional_sprite_test.gd` checks heading selection, frame regions,
nearest sampling, stationary heading retention and absence of simulation movement.
This is not walk animation or final visual admission. Broad partial alpha, equipment
side drift and camera/pixel-density differences remain explicit source defects.
The old battle scene remains the default until a playable island replacement works.
