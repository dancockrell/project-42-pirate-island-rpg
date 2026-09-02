# Project 42 3D Production Plan

## Production decision

Project 42 is now a **fully 3D island game with a 2D theatrical interface**.
Exploration locations, battle stages, playable characters, monsters,
architecture, props and distant scenery are dimensional scenes. Party cards,
skill diamonds, readable combat prose, targeting, health and intent remain 2D
`CanvasLayer` UI. This is not a loose collection of generated beauty shots.
The combat camera remains deliberate, legible and fighting-game sized.

The existing paper dolls and painted environment plate are temporary mechanical
stand-ins. They are not visual source, approved art or a fallback art direction.
They leave the live game only when corresponding 3D actors and world cells pass
their separate camera, collision, navigation and motion gates.

## Island world-cell architecture

The island is not one giant generated mesh. It is a directed graph of authored
**world cells**. A cell is a bounded 3D playable scene with one stable entry
and exit contract. A region contains several cells; the island map connects
regions. This lets a generated or hand-built scene become playable only after
its physical and narrative layers are separately defined.

```text
Island Map
  Region: Black Beach
    Cell: Shipwreck Shore
    Cell: Smuggler Trail
    Cell: Reception Terrace
  Region: Elven Interior
    Cell: Sunken Road
    Cell: Tomb Entrance
    Cell: Bronze-Age Burial Hall

Each WorldCell (Node3D)
  VisualShell            imported baked GLB/GLTF and distant sky/card backdrop
  TerrainAndCollision    hand-authored simple walkable and blocking geometry
  Navigation             authored navigation regions and explicit portal paths
  Occlusion              camera obstruction volumes and transparency rules
  Interactives           named doors, chests, readable objects and NPC anchors
  EncounterVolumes       authored encounter entries and retreat exits
  Ambient                weather, sound anchors, day/night material parameters
  CameraRails            exploration, conversation and battle-entry camera marks
```

A visual scene can be elaborate. Its collision, navigation and encounter
meaning cannot be guessed from its triangles. Every imported environment gets
simple separate proxy geometry. Actors walk on authored `TerrainAndCollision`,
not on arbitrary generated decoration. The same rule keeps the player from
getting trapped in roots, stairs, ruins or foliage simply because an AI scene
looks good in a still image.

### Exploration camera and scale

Exploration uses a fixed elevated three-quarter camera, not a free rotating
camera. The camera travels on explicit rails in a cell and keeps the player,
near interaction targets and local path exits readable. The player is normally
shown head-to-boot at about 20–32% of viewport height. On approach to a battle
volume, the world cell passes a stable entry anchor and facing direction to the
battle scene; the battle then uses its fixed side camera. The two scenes share
the same named physical location, not two unrelated paintings.

World scale uses metres. One normal adult is 1.65–1.85 metres; a normal cell
is 45–90 metres across; a local interaction reads at 1.5–3 metres; a battle
entry lane is 8–12 metres wide. Do not make a 300-metre plaza only because a
generated image feels grand. The player must cross the cell at a believable
walk time and be able to understand where every route leads.

### Baked-scene intake

A baked world scene is a source layer, not a finished playable level. The
ingestion package for every cell contains:

| Layer | Required asset or data | Purpose |
| --- | --- | --- |
| Source | raw vendor GLB/GLTF, images and creation URL | preserves provenance and rerun information |
| Visual shell | cleaned, material-budgeted engine copy | visible architecture, foliage, prop density and distant depth |
| Collision | low-complexity ground, wall, climb, water and no-go volumes | reliable walking and combat placement |
| Navigation | explicit nav region and portal path graph | deterministic movement and NPC routes |
| Occlusion | camera hide/fade volumes and ceiling cutaways | camera never loses the player behind a wall |
| Semantics | stable object IDs and readable descriptions | makes prose, quests and interaction logic refer to real places |
| Lighting | baked/parameterized day-night material inputs | keeps region palette coherent without runtime light chaos |
| QA | arrival, exit, camera, collision, encounter and browser performance result | admission evidence |

The baked visual shell must never embed functional collision, a navmesh, quest
state, NPC identity or a spawn rule. Those are authored Godot/data layers.

### World-cell manifest

Each cell has an explicit record before implementation:

```json
{
  "id": "world.cell.reception_terrace",
  "regionId": "world.region.black_beach",
  "visualShell": "res://assets/world/black_beach/reception_terrace.glb",
  "collisionScene": "res://scenes/world/black_beach/reception_terrace_collision.tscn",
  "entryAnchors": ["entry.shipwreck_trail", "entry.estate_gate"],
  "portals": ["world.portal.reception_to_shipwreck_trail"],
  "battleEntries": ["encounter.reception_razorbeak"],
  "cameraRail": "camera.exploration.reception_terrace",
  "descriptions": ["location.reception_terrace.arrival", "location.reception_terrace.ruin"],
  "timeLightingProfile": "lighting.black_beach.storm_gold"
}
```

All fields use stable IDs. No dialogue, quest, battle or save system is
permitted to point directly at a Godot node path or a vendor asset filename.

### First 3D world-cell proof

`world.cell.reception_terrace` is the first complete cell. It has a wild
Bronze-Age elven gate, jungle encroachment, a broken approach, clear ground
for the Razorbeak encounter, and a visible path toward the estate. It is not a
colonial city block, zoo, dinosaur pen or generic arena. Its implementation
test is intentionally concrete:

1. Player arrives from Shipwreck Trail at the named entry anchor.
2. Camera frames player, gate and usable route without foliage clipping.
3. The player can walk to estate gate, battle entry and return route.
4. A Razorbeak encounter locks to the named battle entry, then returns the
   party to the correct world anchor after victory, retreat or defeat.
5. Every visible landmark has a short, authored readable description.
6. The exported browser build keeps a responsive frame rate with its visual
   shell, collision proxies, active actor, monster and interface present.

## First playable 3D shot

Reception Terrace stays the first battle, at a 1920×1080 logical canvas.

```text
2D UI safe area
  top 0–150        location, active turn and actual enemy intent only
  centre 150–780   one 3D battle plane; full Betty; full Razorbeak
  lower 780–1060   party cards left; seven-skill diamond grid centre

3D world coordinates
  party actor anchor       (-2.65, 0.00, 0.00)
  contested action point  ( 0.00, 0.00, 0.00)
  enemy actor anchor      ( 2.65, 0.00, 0.10)
  camera focus            ( 0.00, 1.65, 0.00)
  camera position         ( 0.00, 1.85, 9.25)
  camera FOV              31 degrees
```

Normal turns use a fixed camera. A skill may use a bounded 0.16–0.28-second
focus shift to its named action point. It does not orbit a character, zoom
through the floor or remove a weapon, feet, target or UI from view. The full
actor, held weapon, target and at least fifteen percent VFX envelope must be
visible for every authored skill.

## Engine boundary

The 3D stage is presentation only. Rust still owns combat state. GDScript
projects typed events into UI and animation. A 3D clip may never infer damage,
choose a target, advance initiative or decide that an attack landed.

```text
Battle3DStaging (Node3D)
  WorldEnvironment / KeyLight / RimLight / GroundPlane
  BackdropLayer                     3D or 2.5D environment depth
  PartyAnchors → BettyAnchor / MichaelAnchor
  EnemyAnchors → RazorbeakAnchor
  EffectAnchors                     actor, weapon, ground and target sockets
  BattleCamera                      fixed fight camera

BattleUI (CanvasLayer)
  party card rail / enemy intent / combat prose / seven skills / targeting
```

The browser-compatible `gl_compatibility` renderer remains pinned until a
shipped browser build proves another backend works. Use readable directional
and spot lighting, restrained emissive magic and painted/baked environment
depth. Never choose a visual effect that the browser build cannot ship.

## Rig-input plate rule

A scenic beauty shot is not an economical rig input. A 3D generator needs a
clean full-body image that makes limbs, hands, boots, costume layers and prop
separation unambiguous.

Prepare one **3D identity plate** per playable character before generation:

| Field | Required value |
| --- | --- |
| Canvas | 2048×2048 or larger; full hair, boot soles, hands and weapon; 12% empty margin |
| Camera | near-orthographic front three-quarter; readable shoulders and waist; no dramatic lens |
| Pose | relaxed A-pose; separated feet; elbows clear of torso; visible hands; low diagonal weapon |
| Background | flat neutral charcoal; no scenery, cast shadow, text, UI, VFX or second figure |
| Costume | final silhouette and palette; every layer readable as a separate form |
| Weapon | complete and visibly separate from torso and legs; short mace, never staff or wand |
| Output | fully riggable adult game character, not a poster, portrait or action illustration |

The generator receives the identity plate as the front view with **Rig for
animation enabled before generation**. Add left, right and back plates only
when they are the same approved body, costume and weapon. Never repair a poor
side view by mixing in a different character image.

Betty’s generation-ready four-view contract is
`work/art/magnific/betty/rig_input_plate_family_v1.json`. It produces a single
clean turnaround sheet first, then extracts the Front, Left, Right and Back
panels without repainting or mixing generation runs. The material target is
high-definition stylized anime 3D: clear fabric, bronze, leather and glass
separation under studio light, with painterly stylized skin and face rather
than photographic pores or a real-person likeness.

## Rig delivery contract

A candidate must deliver an editable GLB or GLTF with these literal,
machine-inspectable requirements:

```text
Required bones
  Root / Hips / Spine / Chest / Neck / Head
  LeftUpperArm / LeftLowerArm / LeftHand
  RightUpperArm / RightLowerArm / RightHand
  LeftUpperLeg / LeftLowerLeg / LeftFoot / LeftToe
  RightUpperLeg / RightLowerLeg / RightFoot / RightToe

Preferred secondary bones
  Hair_01...n / CoatTail_L_01...n / CoatTail_R_01...n / Satchel / AmpouleRack

Required sockets
  Socket_Weapon_R, Socket_Weapon_L, Socket_Ground,
  Socket_Chest, Socket_Head, Socket_Target

Required clips
  Idle_Ready, Step_Forward, GuardedStrike_Anticipation,
  GuardedStrike_Contact, GuardedStrike_Recovery
```

The held weapon must be a separate mesh or a child of `Socket_Weapon_R`; it
may not be fused into a hand or body mesh. It must remain readable through
ready, step, anticipation, contact and recovery.

## Betty’s first package

Betty is first because `Guarded Strike` already has complete gameplay rules,
targets and action beats. Her identity plate preserves the approved ledger:

- adult woman in her mid-twenties; auburn curls, green eyes and freckles;
- slight agile adult build, modest small chest, narrow hips, fine waist and
  long lean legs;
- teal, cream and bronze pirate-field-medic costume; open shoulders, modest
  midriff, fitted field shorts, tall boots, satchel and ampoules;
- a short, heavy bronze-and-glass boarding mace with a visible impact head;
- direct, confident, teasing expression; no second subject;
- never a giant bust, heavy hourglass, wide hips, thick thighs, staff-like
  weapon, cropped limbs, cropped weapon, red-cross emblem, text or scenery.

Her first in-engine proof is a five-beat `Guarded Strike` loop:

1. `Idle_Ready`: both boots grounded, mace below shoulder height.
2. `Step_Forward`: one short step reaches the contested action point.
3. `GuardedStrike_Anticipation`: weight drops and mace chambers across body.
4. `GuardedStrike_Contact`: one short horizontal strike reaches the enemy
   chest-height contact point.
5. `GuardedStrike_Recovery`: Betty returns to party foreground in a guard
   that can blend to `Idle_Ready`.

The camera must retain Betty, Razorbeak, mace, both feet and impact envelope
for all five beats. The seven skill packages come only after this loop works.

## Candidate gates and provenance

Every download is recorded under
`work/art/vendor/magnific/<character>/<creation-id>/`. The manifest contains
input-plate hash, creation URL, model version, export hash, skeleton count,
clips, materials, visual review, in-engine review and decision.

| Gate | Pass condition | Failure action |
| --- | --- | --- |
| Identity | approved face, body direction, costume and weapon | reject; do not repair through runtime tweaks |
| Structure | skeleton, named bones, separate weapon and named clips | reject as animation source; static blocking only |
| Camera | head, boots, weapon, target and 15% VFX margin visible | change staging or reject model scale |
| Motion | no collapsed joints, sliding feet, weapon drift or face distortion | reject rig or regenerate; no camera concealment |
| Performance | active fighter, enemy, lights, stage and UI run in browser export | reduce mesh/material complexity |
| Provenance | source, hash and settings retained | quarantine; never silently admit |

`betty_candidate_v1.glb` passed only static-blocking import. It has zero skins
and zero clips. It is a 3D scale-and-lighting reference, not an animated actor.

## Production order

1. Make and approve Betty’s clean 3D identity plate.
2. Generate one rig-enabled Betty GLB from that plate.
3. Inspect skeleton, clips, materials and weapon separation.
4. Use the isolated review scene for the five-beat camera test.
5. Build `Battle3DStaging` and connect its anchors to the authoritative battle
   snapshot already feeding the UI.
6. Replace only Betty’s paper proxy after the five-beat test passes.
7. Repeat for Razorbeak, Michael and the next heroine.

No stage-wide conversion happens before one heroine visibly completes one real
command from selection through recovery.
