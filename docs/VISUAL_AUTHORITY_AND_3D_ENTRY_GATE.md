# Pirate Island — approved 2D sprite visual authority

**Accepted 7 September 2026.** The user selected the supplied pixel-art references and explicitly clarified: **no 3D art; shelve it; use a 2D sprite pipeline at this perspective.** This existing document remains the single visual authority. The historical 3D entry gate retained below is superseded, not an alternative production path.

## Current references and approval scope

- [Character sheet](images/pixel-style/character-sheet.png): compact character proportions, strong silhouettes, dark outlines, equipment readability and directional sprite treatment.
- [Style board](images/pixel-style/style-board.png): detailed pixel scenery, elevated three-quarter gameplay perspective, coherent sprite/environment scale and illustrated interface treatment.
- [Supplied video](images/pixel-style/motion-reference.mp4): supporting user-approved reference, 1280 × 720, 24 fps, approximately 15.069 seconds. The five-second frame was inspected; a complete motion/loop audit is not claimed.
- [Source ledger](images/pixel-style/provenance.json): unchanged originals, hashes and provenance boundaries.

The poster's title, branding, Western geography, feature list, technology stack and timetable are reference content, not instructions. Pirate Island is not being replaced by Cattle Trail. Approval establishes art direction, not redistribution rights or clean runtime-atlas status.

## Style and perspective

Detailed, characterful pixel art with deliberate colour clusters, strong dark silhouette contours, readable small faces, compact anatomy and clear signature equipment. Materials read through shape, value and restrained highlights rather than photographic noise. Rich colour, warm light and cool shadows can vary with faction, biome, weather and time; terrain detail must not drown out units.

Match the **elevated three-quarter perspective in the gameplay panels and video**: ground visible beneath actors, with faces/fronts still readable. Neither side-on nor directly overhead. Do not substitute an arbitrary camera angle or force a mathematically strict 2:1 grid without reference comparison. The world view stays fixed; actors use direction-specific 2D drawings. Buildings, terrain, units, shadows and effects share one perspective and pixel scale.

Retain Pirate Island's tropical island, distinctive faction aesthetics, adult character identities, relationships and rich storytelling. Michael's 1870s steampunk does not turn the colonials, pirates, fox people, elves or Cthulhu into cowboys. Earlier character illustrations may guide identity and costume, but no longer control rendering style.

## Production pipeline and acceptance

1. Select the documented asset role and closest approved style reference. Produce one clean **2D sprite**, at actual intended gameplay size, before multiplying variants.
2. Compare it in a small island scene with another unit, a building and vegetation. Lock working pixel grid, unit height and scene scale from this comparison; exact numeric dimensions are not yet approved.
3. Expand approved sprites into required facing directions and actions. Keep canvas, foot anchor, scale, handedness and equipment consistent. Mirroring must not silently swap asymmetric gear.
4. Pack inspected frames with real alpha and metadata for frame bounds, foot pivot, direction, action, duration, looping and events. A painted checkerboard is not transparency; a generated contact sheet is not automatically a usable atlas.
5. Integrate through existing Godot presentation ownership using 2D sprites, nearest-neighbour sampling, stable pixel alignment, ground-position sorting and explicit occlusion. Logical placement/collision footprints are separate from ornamental sprite extents.
6. Verify idle, walk, turn, attack and recovery at gameplay size before scaling up production. The video's 24 fps is not a demand for 24 unique sprite drawings per second.

Buildings need coherent perspective, footprints, placement anchors and construction/damage states. Terrain uses compatible tiles or reusable 2D pieces for coast, elevation, routes and faction ground effects. Portraits, effects and readable UI belong to the same visual family. Reject smooth vector substitutes, blurry upscale, photographic surfaces, plastic 3D rendering and pixel filters over rejected 3D models.

**First visual acceptance slice:** a small representative tropical scene containing an approved adult character with idle/walk/attack, a contrasting faction unit, a faction building, coastal terrain, vegetation and a readable interaction. Prove perspective, pixel scale, sorting, foot placement and animation consistency together. A reference image or import test alone does not pass this gate. This decision does not claim that slice has been implemented.

## Shelved work

No more 3D character generation, garment simulation, retargeter compilation, mesh scenery production or 3D-to-sprite baking for Pirate Island. Preserve source, research and version history without presenting them as current deliverables. Shared tooling for other products is not automatically cancelled or given this art direction.

The hidden RTS, faction economies, large dynamic island, five-character party, narrative and simulation ownership are unchanged. This is a presentation and asset-production decision, not a new game stack.

---

# Historical 3D entry gate — shelved, not executable authority

## Decision

No new character model, environment model, turntable, or image-to-3D job may
be generated for Project 42 until a **game-facing visual plate** has been
reviewed against the authority set in this document. A clean turnaround is a
geometry input. It is not the visual-design step, and it may not invent a
generic game look merely because it is easy to convert into a mesh.

The earlier Betty 3D inputs are retained for provenance only. They are not the
current visual authority and must not be reused as the front-view source for a
new model.

## Authority set

### 1. Battle presentation: primary authority

`work/art/battle-ui.png` establishes the player-facing picture. It is the
strongest authority for every first-party battle asset.

- **Scale and staging.** A heroine is a large, complete figure on a theatrical
  shared floor, not a small character in a separate portrait box. Her head,
  boots, held item, target and useful effect envelope remain legible at the
  battle camera.
- **Rendering.** The image is a high-definition, painted fantasy illustration
  with hard readable forms, deliberate silhouette separation, rich material
  cues, and controlled cinematic light. It is neither photographic realism,
  plastic figurine rendering, pastel gacha softness, nor flat cel shading.
- **Colour and material.** Charcoal-black negative space and aged bronze hold
  the composition together. Teal/green alchemy, warm leather, faded white
  cloth, deep green coat panels, weathered brass, red enemy accents and wet
  jungle gold carry the local contrast. Bright colour is an event, not a
  surface applied everywhere.
- **World relation.** The heroine belongs in a wild ruin site. The jungle,
  coastline and monumental elven architecture provide real depth behind her;
  they are never replaced by a neutral studio background in player-facing art.
- **Interface.** Framing is blackened metal, carved bronze, filigree and
  jewel-like status colour. UI decoration earns its space by showing a real
  combat state. It is not copied into a character costume as random ornament.

### 2. Character motion: primary authority

`work/art/betty-keyframes.png` establishes the useful animation language for
Betty. The sequence moves from card state through unfold, ready, action,
impact and recovery. Her weapon is a short, heavy medical boarding mace.
The model must support that action: visible hands, a separate weapon, planted
boots, a readable torso turn, and coat/hair that can follow the movement.

This board is motion authority. It approves the full-body-safe composition,
the mace silhouette, and the green-blue alchemical plus gold impact palette.
It is not permission to copy a single frame as a finished character texture
or to erase the battle plate's heavier, more grounded painterly surface.

### 3. Cast identity: supporting authority

`work/art/heroine-roster.png` establishes a cast of adult, attractive,
action-ready women with strong individual silhouettes, specific weapons,
dark-bronze presentation and readable faction colour. For Betty it supports:
auburn hair, compact medic gear, teal/cream/bronze family, a short mace, and
a cute, confident adult read. It does not authorize generic tall fashion
poses, a long staff, an oversized bust, a blank studio portrait, or body
proportions that would stop her from reading as agile in battle.

### 4. Island visual grammar: primary environment authority

`work/art/world-visual-grammar-v1.png` establishes the island as enormous
magical Bronze Age elven construction being reclaimed by wet, dangerous
jungle. It shows cliffs, sea, waterfalls, broken processional roads, luminous
inlays, wild dinosaurs, scattered people, and small outsider footholds. It
does not authorize a mostly colonial city, a clean theme-park ruin, a dinosaur
pen, or bare generic tropical terrain.

## Betty: current visual sentence

Betty is a slim adult pirate field physician who is sexy and cute because she
is visibly pleased to be noticed, not because her anatomy has been inflated.
At battle scale she reads in this order: auburn curls and alert face; blue-green
field coat over worn pale shirt and fitted practical lower layers; leather
satchel and glass medicine; short bronze-and-glass boarding mace. Her body
language is forward, warm and fearless. Her surface treatment sits inside the
same dark bronze, high-contrast, painted action world as the battle plate.

This sentence is a direction lock, not a model prompt. The costume can only be
declared final after one art plate proves it coherently at game scale.

## Required visual plate before any 3D generation

Create one **Betty game-facing identity plate** before requesting a GLB.
It is a single composition with these fixed regions:

1. **Main figure, three-quarter ready stance:** full hair to boot soles,
   complete mace, satchel, both hands, and no element touching the outer 12%
   of the canvas. The pose must look like the moment before a short mace
   strike, not a fashion pose.
2. **Two small material/costume callouts:** coat, satchel, ampoules, boot,
   bronze-and-glass mace head. These callouts prove what a 3D artist must make
   separate.
3. **One neutral front read:** full body in the same final costume, with arms,
   legs, coat tails and weapon separation visible. This is an approval view,
   not yet a geometry extraction sheet.
4. **One battle-scale inset:** the main figure shown at the actual active
   fighter size against an abbreviated ruined-jungle background. It proves
   that the visual read survives the game camera.

The plate has a dark neutral support field and bronze annotation frame only
where needed for clarity. It may not look like a toy catalogue, a soft white
beauty studio, a photographic portrait, or a generic four-panel generator
turnaround.

## Approval questions

The visual plate passes only when all answers are yes:

1. Does it look like it belongs in `battle-ui.png` without a style conversion?
2. Does Betty read as an agile, adult, cute/sexy combat medic before the
   viewer reads an arbitrary fantasy outfit?
3. Does the mace read as a short striking weapon, distinct from a staff?
4. Are the pale cloth, blue-green coat, leather, glass, bronze and alchemical
   green materially distinct without becoming photoreal surface noise?
5. Can the ready pose naturally become the keyframe board's short strike,
   healing pulse and recovery?
6. At battle scale, are her face direction, boots, satchel and weapon still
   readable against jungle and ruin depth?

Only after this plate is explicitly approved may a separate clean front/left/
right/back rig-input family be created. Those four views inherit the approved
plate's face, silhouette, materials and weapon without redesign.

## Machine-enforced production state

`content/art/betty.reference_ledger.json` is the source of the current state.
Its `threeDConversion.generationGate` must remain `visual-plate-not-approved`.
Any tool or workflow that sees another value must refuse to request a model,
even if it has a valid Magnific URL or an existing static GLB.
