# Video art mining pipeline

> **Historical prototype contract — superseded where it defines product direction (5 September 2026).** The [current build plan](GAME_BUILD_PLAN.md) governs the fixed isometric autonomous RTS, five controllable heroes, dual clocks, and companion-led progression. The side-view stage, card-to-active-fighter composition, and animation-first production sequence below are not current requirements. Preserve this body as implementation/reference provenance; reuse individual assets only after review for the current board.

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
