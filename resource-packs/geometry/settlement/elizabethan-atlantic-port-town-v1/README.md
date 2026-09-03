# Elizabethan Atlantic Port Town v1

This pack is a construction system for a complete stylized port town. It is
not a collection of unrelated houses. Each recipe states the player function,
street relationship, readable landmarks, interaction anchors, character roles,
and canonical source assets required to assemble one part of town.

The functional inventory was checked against the Crossing map family in DR
Companion. Crossing supplies evidence that a durable game town needs more than
shops: a connective street hierarchy, public space, specialist services,
training institutions, worship, housing, transport edges, civic authority,
social interiors, and concealed routes. No DragonRealms names, lore, factions,
or proprietary geometry are copied into this pack.

## Visual contract

- Genre: romantic adventure, Atlantic fantasy port, circa 1550-1650 silhouette
  language with controlled anachronistic steampunk additions at project level.
- Render: stylized 3D, high-quality hand-painted target, clean shape grouping,
  readable from a 35-55 degree elevated gameplay camera.
- Palette: warm cream limewash, dark oak framing, oxblood tile, weathered slate,
  sea-green paint, verdigris copper, warm brass, saturated market cloth.
- Scale: one Godot unit is one metre; ordinary doors are 2.1 m; ground floors
  are 3.0 m; streets are 4-7 m; alleys are 1.5-2.5 m.
- Density: every street view must show one destination landmark, one route
  continuation, one human-scale activity cluster, and one environmental story.
- Composition: buildings make outdoor rooms. Empty plazas, freestanding facade
  rows, and evenly scattered props fail review.
- Repetition: reuse geometry freely, then vary footprint, roofline, frontage,
  sign, awning, color accent, clutter story, and relationship to the street.
- Characters: named heroes require their approved identity assets. Town-set
  recipes contain spawn roles and rig sockets only; they never substitute a
  generic body for a named character.

## Contents

- `town-functions.json`: setting-neutral town functions and mandatory physical
  signals, including generalized martial and wilderness institutions.
- `set-recipes.json`: twelve buildable sets with dimensions, adjacency,
  navigation, encounters, population roles, camera requirements, and dressing.
- `resource-pack.json`: searchable pack identity, provenance, technical rules,
  canonical source selections, and review state.
- `discovery/*/catalog.json`: alternate indexes by physical type. These are
  references to canonical assets, not duplicate binaries.
- `game/scenes/review/elizabethan_port_town_set_review.tscn`: Godot proof scene
  that blocks the town at final metre scale with explicit replacement metadata.
- `docs/images/elizabethan-port-town-blockout-v1.png`: GPU-rendered review proof
  generated from that exact scene; its sidecar records the capture provenance.

## Assembly rules

1. Start with a recipe. Preserve its function, footprint, landmark, public
   approach, service entrance, navigation widths, and gameplay anchors.
2. Resolve every `assetRef` through `resource-pack.json`; never copy a binary
   into a second category to improve discovery.
3. Put collision, navigation, interaction anchors, spawn slots, and semantic
   IDs in the consuming scene. Imported art owns appearance only.
4. Replace primitive proof geometry by named section. A replacement passes only
   if the same silhouette, footprint, entrances, occlusion budget, and anchors
   survive at the gameplay camera.
5. Record source, license, transformation, review image, engine import result,
   and final runtime path before promoting any external asset.

## Completion gate

The pack becomes runtime-approved only after all twelve recipes have an
in-engine camera review, collision and navigation proof, character-scale proof,
day and night readability check, and provenance-complete admitted art. Until
then it is a production-grade blockout contract, not final scenery.
