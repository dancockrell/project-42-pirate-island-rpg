# Betty 3D Asset Contract

> **Historical prototype contract — superseded where it defines product direction (5 September 2026).** The [current build plan](GAME_BUILD_PLAN.md) governs the fixed isometric autonomous RTS, five controllable heroes, dual clocks, and companion-led progression. The side-view stage, card-to-active-fighter composition, and animation-first production sequence below are not current requirements. Preserve this body as implementation/reference provenance; reuse individual assets only after review for the current board.

## Current gate: visual authority reopened

**No new 3D request is currently authorized.** The earlier selected image and
the generic four-view request did not establish the actual Project 42 game
look. They remain below as historical provenance for the static candidate, not
as permission to generate another candidate. The controlling document is
`docs/VISUAL_AUTHORITY_AND_3D_ENTRY_GATE.md`.

The next step is a reviewed **Betty game-facing identity plate** derived from
the battle, motion and cast references named there. Only after the player
approves that plate may this contract receive a new canonical source and a
clean rig-input turnaround.

Once the plate is approved: rigging is a switch on the generator and is
expected to succeed; the deliverable at that point is a rigged GLB with the
named skeleton and sockets and **zero clips**. Animation is deferred to the
next-generation animation tool and is not hand-authored (`docs/SHIP_PLAN.md`
D4/D5). This is a generator path end to end; no outside artist is engaged for it.

## Historical canonical identity source for Candidate 01 only

Candidate 01 used this image as its single primary reference input:

`work/art/magnific/betty/betty-body-direction-gpt2.png`

It is no longer the approved Betty body-and-rendering direction. The temporary
Godot paper rig remains a technical pose scaffold only; it is prohibited as
visual source material for any model.

Supporting component references:

- `work/art/magnific/betty/betty-equipment-reference-seedream.jpg`: satchel,
  ampoules, teal shorts, split coat tails, freckles and equipment density.
- `work/art/magnific/betty/betty-palette-reference-seedream.png`: teal,
  cream and bronze palette; sunburst satchel emblem; luminous mace chamber.

## Retired Magnific 3D Generator input

Do not use the following previous generation brief. It is retained only so the
provenance of Candidate 01 can be understood:

```text
BETTY — adult anime-fantasy pirate field physician, identity-preserving 3D
character conversion. Preserve the reference image's face, auburn curly updo,
green eyes, freckles, slender slight adult body, modest small chest, narrow
waist, long lean legs, warm teasing confident expression, teal-and-cream
pirate medic outfit, open shoulders, slight midriff, fitted short field
shorts, tall brown boots, split teal coat tails, brown physician satchel,
glass ampoules, bandage roll, and short bronze-and-glass medical boarding
mace. Complete head-to-boot character with full weapon. Premium stylized anime
RPG quality. Neutral standing A-pose with feet apart enough to animate.

Do not change facial identity. Do not change her body to a pronounced bust,
heavy hourglass, wide hips, thick thighs or bulky physique. Do not give her a
long staff, a generic wand, a red-cross emblem, modern clothes, cropped feet,
cropped weapon, a background, text, or a second character.
```

## Required delivery package

| Item | Required condition |
| --- | --- |
| Model | GLB preferred; GLTF with separate textures acceptable; FBX only if skeleton and textures export cleanly |
| Mesh | Full body, complete boots, complete hands, complete mace, separate equipment where supported |
| Skeleton | Hips, spine, chest, neck, head, upper/lower arms, hands, upper/lower legs, feet; hair and coat-tail bones are preferred |
| Weapon | Separate mesh or named socketable object; not fused into the hand or body mesh |
| Materials | Albedo, normal, metallic/roughness where supplied; named and inspectable |
| Scale | Character stands upright at a predictable metre scale; root at ground between feet |
| Licensing/provenance | Magnific creation URL, model/version, date, reference asset IDs, export format, original download retained under `work/art/vendor/` |

## Immediate acceptance check

Reject the candidate before it reaches Godot if any condition below fails:

1. It is not recognizably the approved Betty in face, silhouette and costume.
2. The proportions drift toward any rejected anatomy listed above.
3. The mace reads as a long staff or cannot be separately attached to her hand.
4. Hair, boots, hands, satchel, ampoule rack or weapon are missing or fused in
   a way that blocks animation.
5. The model cannot produce a neutral idle, a forward step and a short
   cross-body mace strike without obvious mesh breakage.
6. The export is only a render, turntable or video rather than an editable 3D
   model file.

## Godot admission check

After a candidate passes identity review, import it into an isolated Betty
test scene. Do not replace the live battle actor yet. The test scene must
prove:

- skeleton and materials import without missing references;
- feet contact the ground at the active-fighter camera;
- camera frame contains hair, boots, mace and a 15% effect envelope;
- mace follows the right-hand socket in ready, step, anticipation, contact
  and recovery poses;
- the body can fit the existing card-to-active transition position;
- the model reads at the 1920×1080 battle camera before any final shader work.

Only then may the 3D model replace the temporary paper Betty in the Reception
Terrace scene.

## Candidate 01: static blocking export

The downloaded candidate is retained with its raw export and provenance at
`work/art/vendor/magnific/betty-3d/N2cYw4m6D9/`. Its exact engine review copy is
`game/assets/candidates/betty_3d/betty_candidate_v1.glb`.

Structural inspection found one textured mesh, one material, zero skins and
zero animation clips. It is therefore deliberately quarantined as a **static
blocking candidate**. The isolated Godot review scene at
`res://scenes/review/betty_3d_candidate_review.tscn` may test only identity
read, scale, silhouette and the active-fighter camera envelope. It cannot be
used for Betty's idle, step, weapon socket, seven skills or live battle actor.

The next 3D generation is blocked until the visual entry gate passes. After
that gate, this document must be rewritten around the approved game-facing
identity plate before a generator receives any new input.
