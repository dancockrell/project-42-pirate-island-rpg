# Project 42 3D Production Plan

## Production decision

Project 42 is now a **3D character-and-stage game with a 2D theatrical battle
interface**. The player sees dimensional, lit characters on a fixed side-view
battle plane. Party cards, skill diamonds, readable combat prose, targeting,
health and intent remain 2D `CanvasLayer` UI. This is not free-roaming
third-person action. The combat camera remains deliberate, legible and
fighting-game sized.

The existing paper dolls are temporary mechanical stand-ins. They are not
visual source, approved art or a fallback art direction. They leave the live
battle only when a corresponding 3D actor passes its rig, camera and skill
motion gates.

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
