## Current art production decision — 8 September 2026

All game artwork is 2D only: authored sprites, sprite animation, painted backgrounds, tiles, portraits and flat effects. Use the existing shared art folder at `C:/Users/Admin/Documents/Codex/shared-game-environment-library` for reusable 2D kits, with compatible perspective, pixel density, palette, anchors and animation metadata across Cattle Trail/Cattle Drive, DR Companion and Pirate Island. Preserve each game's characters, setting and gameplay identity.

Do not create, purchase, import, restore, archive for later reuse, or bake sprites from 3D models. Earlier model, rig, mesh, material, body-builder and six-month-resumption plans are retired. New generated artwork uses the user-authorized built-in image generator; no external paid generation APIs. Shared reuse does not make unreviewed art automatically approved.

Inspect true transparency, sequential poses, stable ground pivots, equipment handedness, native-size readability and actual motion before admission. Keep game state, navigation, combat rules, accessibility and persistence authoritative; changing artwork never changes legal actions. Existing engine API names and historical validation records may mention 3D without authorizing 3D artwork.

This is production authority, not a claim that every existing runtime or binary has been converted. Legacy-asset deletion is a separate operation: automatic review rejected deletion commands, so deletion remains unverified here. Do not restore those assets or present them as production options.

# Shared Asset Integration Protocol

Status: **working production contract**

This protocol turns the local shared resource library into something a game
team can actually use. It prevents a common failure mode: a folder full of
attractive models that nobody can select confidently, whose scale, materials,
licenses, and gameplay role change every time a scene is assembled.

The library is deliberately **not** a project folder. It holds a physical
vocabulary: terrain, vegetation, construction, furnishing, transport, props,
materials, rig parts, effects, and source collections. Project 42 and DR
Companion each decide their own lore, UI, actors, encounter rules, and visual
identity while drawing from that vocabulary through an explicit profile.

## 1. Two inventories, one discovery surface

There are two places to look, and they mean different things.

| Surface | Location | Purpose | May a game ship it? |
| --- | --- | --- | --- |
| Shared resource library | `resource-packs/` | Repository-backed inventory of raw CC0 source collections and curated derivatives. Search `library.local.json`. | No. A hit is a candidate, not automatic runtime permission. |
| Committed provenance catalog | `content/art/shared_source_collections.json` and `content/art/shared_asset_ledger.json` | Durable proof of source, license, archive hash, review status, and consumer scope. | Only when the individual asset record has passed every admission gate. |

Do not make `game/assets/shared/` the discovery surface. A consuming project
copies, imports, or references only a named admitted derivative for a specific
scene. The source collection remains available locally so the next game can
make a different decision without inheriting another game's imports.

## 2. The consumer profile comes before selection

Before anyone selects a pack, the consuming team writes one short profile for
the first proof scene. This is intentionally concrete. “Fantasy,” “good
lighting,” or “a 3D view” is not enough to make a repeatable choice.

```ts
type SharedAssetConsumerProfile = {
  id: string
  consumer: 'project42' | 'dr-companion'
  proofScene: string
  camera: {
    mode: 'fixed-three-quarter' | 'free-orbit' | 'room-to-overworld'
    elevationDegrees: number
    targetViewportHeightForAdult?: number
    closestPlayDistanceMetres: number
    farthestPlayDistanceMetres: number
  }
  worldScale: {
    units: 'metres'
    baselineDoorHeightMetres: number
    baselineAdultHeightMetres: number
    gridOrCellMetres?: number
  }
  look: {
    geometry: string
    materialLanguage: string
    lighting: string
    palette: string
    prohibited: string[]
  }
  performance: {
    targetPlatform: string
    visibleInstanceBudget: number
    heroAssetTriangleBudget?: number
    backgroundAssetTriangleBudget?: number
  }
  gameplayBoundaries: {
    navigationAuthority: string
    collisionAuthority: string
    selectionAuthority: string
    decorationMayNeverDo: string[]
  }
}
```

The present shared baseline is a deliberately geometric tabletop world:
chunky, readable silhouettes; restrained matte or lightly plastic materials;
simple colored ground planes; strong value separation; and no attempt at
photorealism. That baseline is not a literal felt table and is not a demand
that Project 42 and DragonRealms look identical. It makes raw geometry useful
while each game retains its own palette, lighting, character treatment, UI,
and named world language.

## 3. The selection and admission loop

1. **Declare a proof scene.** Name its gameplay purpose, not just its visual
   theme. For example: an overworld route with a bridge and river crossing, a
   compact settlement approach, a tomb encounter lane, or a DragonRealms room
   node seen from both close room view and distant world view.
2. **Search the local index.** Filter by physical type, family, authoring
   lineage, style tags, source collection, format, and review state. Start
   with an existing derivative pack whenever one fits; do not blindly duplicate
   a source model.
3. **Assemble only a disposable proof.** Place the selected models at the
   declared camera and scale. The proof has separately authored route, portal,
   collision, and selection data. Geometry may decorate these systems but may
   not quietly become them.
4. **Review in the actual game camera.** Record a visual decision, technical
   decision, scale/pivot outcome, material decision, and whether the asset
   remains legible behind active characters, effects, and UI.
5. **Create or update the resource-pack record.** A derivative must say
   exactly what changed: curated subset, material adapter, pivot correction,
   collision proxy, LOD, mesh edit, or other transformation. Preserve original
   source path, archive hash, creator, and license.
6. **Admit the exact derivative, not a vague source folder.** Add an asset
   ledger record with a stable ID and explicitly list its consumers. If it is
   only useful for a named DragonRealms guild or Project 42 faction, it remains
   project-only.
7. **Return evidence to the library.** Store screenshot/camera evidence and
   performance notes with the review record. Rejected choices stay tagged as
   rejected so a future team does not repeat the same mistake.

## 4. Runtime boundaries: what an asset is allowed to decide

| System | Owner | Rule |
| --- | --- | --- |
| World connectivity, legal travel and room state | Game simulation/map data | A visual road, door, bridge, or staircase never creates a valid traversal link. |
| DR Companion room tethering and MUD event truth | DR Companion bridge and map model | A character is visually attached to its authoritative room node; animation may interpolate, but it never asserts a new room. |
| Project 42 encounter, combat, and mission truth | Project 42 simulation | Godot presents a validated state and does not invent combat results from contact animation. |
| Navigation and collision | Per-scene authored collision/navigation layers | A rendered mesh is not trusted as a navigation surface or blocker until the scene declares it. |
| Selection and interaction | Per-scene interaction data | Decorative props cannot consume pointer/target input by accident. |
| Materials and lighting | Consumer profile plus derivative pack | The same neutral rock can use a different palette adapter in two games without losing its source lineage. |

For DR Companion specifically, the target is a single Godot viewer that can
zoom between a readable world network and a room-scale presentation. Its MUD
remains authoritative: the viewer is an animated spatial explanation of room
nodes, known routes, people, creatures, items, and event results. For Project
42, the world-cell contract remains authoritative: visual shell, terrain and
collision, navigation, occlusion, interactives, encounter volumes, ambience,
and camera rails remain separate scene concerns.

## 5. What to request from the library team

Every request should use this compact form:

```md
Consumer: dr-companion | project42
Proof scene: <named gameplay situation>
Need: <physical types and count ranges>
Camera: <mode, elevation, close/far distance>
Scale: <adult, door, cell/grid convention>
Look: <geometry, materials, lighting, prohibited treatments>
Gameplay separation: <where routes/collision/selection are authored>
Budget: <target platform and visible-instance target>
Decision needed: <source candidates | derivative pack | new material adapter | store candidate search>
```

The library team responds with stable local resource IDs/paths, provenance,
license scope, a recommended derivative action, and the exact proof needed
before admission. It does not respond by silently dumping hundreds of models
into a consumer repository.

## 6. First shared proof families

The two teams should converge through small, reusable proofs rather than a
city-sized import.

| Priority | Shared family | Proof it must support | Existing local source direction |
| --- | --- | --- | --- |
| 1 | Weathered stone, routes, bridges, water edge | A route remains readable close-up and zoomed-out; collision/nav remain separate. | Nature, modular cave, city roads, watercraft kits. |
| 2 | Vegetation and terrain dressing | Repeated plants create variety without hiding actors, routes, or interactions. | Nature and mini forest kits. |
| 3 | Neutral construction and settlement dressing | Doors, roofs, walls, carts, crates, lamps, tables, and market clutter read as a place without imposing named lore. | Building, modular buildings, furniture, survival, pirate kits. |
| 4 | Tomb/cave/interior shell | A room can be dressed as a discrete encounter/exploration space without using baked geometry as gameplay logic. | Modular dungeon and cave kits. |
| 5 | Characters and effects | Establish only sockets, scale, visibility rules, and animation/VFX interfaces. Premium bodies, heads, costumes, rigs, and performances have their own licensed admission path. | No raw pack is automatically approved for this role. |

## 7. Review states and stopping rules

* `source_cc0` means downloaded and traceable, not visually approved.
* `project_derivative` means a source-backed local transformation exists; it
  remains draft until its consumer and camera review pass.
* `approved_shared` requires legal, technical, visual, and consumer decisions.
* `approved_project_only` remains reusable only inside its named license and
  product boundary.
* `rejected` is useful historical information. Retain why it failed: wrong
  silhouette, noisy material, bad scale, overlap hazards, unclear license, or
  unsuitable performance.

Stop a source intake batch when it does not close a declared proof-scene gap.
Broad coverage is valuable; untargeted accumulation is not. Marketplace and
paid candidates must enter the same review loop, but their raw source remains
in an access-controlled local vendor cache unless the recorded license grants
every intended form of sharing.
