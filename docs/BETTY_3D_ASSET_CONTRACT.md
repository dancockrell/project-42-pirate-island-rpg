# Betty 3D Asset Contract

## Canonical identity source

Use this image as the single primary reference input:

`work/art/magnific/betty/betty-body-direction-gpt2.png`

This is the approved Betty body-and-rendering direction. The generated 3D
model must be a conversion of this identity. It is not a new character-design
attempt. The temporary Godot paper rig is a technical pose scaffold only; it
is prohibited as visual source material for the model.

Supporting component references:

- `work/art/magnific/betty/betty-equipment-reference-seedream.jpg`: satchel,
  ampoules, teal shorts, split coat tails, freckles and equipment density.
- `work/art/magnific/betty/betty-palette-reference-seedream.png`: teal,
  cream and bronze palette; sunburst satchel emblem; luminous mace chamber.

## Required Magnific 3D Generator input

Use the canonical identity source as image reference. Enter this as the
generation brief:

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
