# Pirate Island — approved 2D sprite visual authority

**Accepted 7 September 2026.** The user selected the supplied pixel-art references and explicitly clarified: **2D sprite pipeline at this perspective.** This existing document remains the single visual authority.

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

1. Select the documented unit role and closest approved style reference. Deliver a coherent animated sprite sheet, not isolated standing poses.
2. Preserve full-resolution drawings and colors. Do not downsample or quantize source artwork. Camera scale is separate from source resolution.
3. Produce required facing directions and idle, walk, attack, hit and death actions. Keep canvas, foot anchor, scale, handedness and equipment consistent. Mirroring must not silently swap asymmetric gear.
4. Pack inspected frames with real alpha and metadata for frame bounds, foot pivot, direction, action, duration, looping and events. A painted checkerboard is not transparency; a generated contact sheet is not automatically a usable atlas.
5. Integrate through existing Godot presentation ownership using 2D sprites, nearest-neighbour sampling, stable pixel alignment, ground-position sorting and explicit occlusion. Logical placement/collision footprints are separate from ornamental sprite extents.
6. Verify idle, walk, turn, attack and recovery at gameplay size before scaling up production. The video's 24 fps is not a demand for 24 unique sprite drawings per second.

Buildings need coherent perspective, footprints, placement anchors and construction/damage states. Terrain uses compatible tiles or reusable 2D pieces for coast, elevation, routes and faction ground effects. Portraits, effects and readable UI belong to the same visual family. Reject smooth vector substitutes, blurry upscale, photographic surfaces and inconsistent rendering styles.

**First visual acceptance slice:** a small representative tropical scene containing an approved adult character with idle/walk/attack, a contrasting faction unit, a faction building, coastal terrain, vegetation and a readable interaction. Prove perspective, pixel scale, sorting, foot placement and animation consistency together. A reference image or import test alone does not pass this gate. This decision does not claim that slice has been implemented.
