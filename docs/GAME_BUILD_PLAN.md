# Project 42: Pirate Island RPG — Build Plan

> **Rule 0 — never fork.** A problem is to be solved, never dodged. Fix the thing,
> replace it outright, or delete the feature — those are the only three moves.
> Never leave two answers to one question standing side by side, and never route
> a parallel path around something you did not want to touch. That is a noodle to
> nowhere, and it is the most serious thing you can do to this codebase.
> Full rule: [`CLAUDE.md`](../CLAUDE.md).

## 1. The game being built

Project 42 is a party-based, side-view tactical RPG about Captain Michael
Corrigan and the adult women who choose to join his household on a lost,
dangerous tropical island. Michael is the captain and inventor who survived
the destruction of the steam-refitted tea cutter *Handsome Jack*. The island
is predominantly ancient elven magical Bronze Age infrastructure reclaimed by
jungle. Colonial ports, forts, fox-folk docks and orc institutions exist as
small foreign footholds; they do not turn the island into a built-up colonial
city map. Dinosaurs are wild across the island. The party explores, reads the
world through detailed authored observation text, enters dangerous sites,
fights individual high-presence threats, returns to an estate, gains allies,
and changes what can be attempted tomorrow.

This plan builds one coherent game, in dependency order. A phase is complete
only when its exit test passes in the running game and its changed files are
committed and pushed. A sketch, an attractive screenshot or a disconnected
screen is not completion.

## 2. Fixed player-facing promises

### 2.1 The first playable chapter

The first chapter begins at Black Beach after the wreck of the *Handsome
Jack*. Michael reaches the damaged coastal estate, makes it defensible enough
to function, then leads a party along the river approach to Reception Terrace,
the first open elven processional site. The chapter ends when the party has
survived the terrace encounter, returned to the estate, and carried a real
change back into household life.

### 2.2 What every battle looks like

The battle camera is one broad, theatrical side-view field. It uses the
reference battle composition as the spatial target: deep jungle and enormous
elven ruin at left and in the distance; a clear ground plane through the
middle; the active heroine in the party foreground; an individual enemy at
the enemy front; compact party cards only at the lower left; actual commands
only at the lower centre. The card-to-active transition is the central visual
rule. A party member is compressed into her card until she is selected or
acts. She then unfolds into a complete full-body fighter on the shared battle
floor. The camera never cuts off her head, boots, weapon, target, or the
meaningful part of her effect envelope.

The initial encounter contains Betty, Michael, and one Razorbeak. It does not
pretend that a fourth heroine or an orc is present merely because the reference
composition has room for them. As authored characters and enemies become live,
they occupy those already-defined spaces.

### 2.3 The relationship fantasy

This is adult haremlit with adult characters. Michael is admired because he is
capable, decent, brave and increasingly able to protect the people who choose
him. The household is stable. Its women are interested in Michael and in one
another, and the fantasy is affectionate expansion, not betrayal, surprise
romantic rivals or an analysis of relationship failure. The player is never
asked to force a woman, manage hidden outside affairs, or discover that a
committed heroine has been taken away by a romance twist. Romance scenes and
household scenes earn new skills, scenes, outfit variants and tactical links
through authored story progress.

### 2.4 The island’s strange daily law

At midnight, eligible dead people and monsters return in flashes of light.
Named people remember their deaths and keep a death counter. They may mention
it in conversation. They are used to the horror in the practical, uneasy way
people become used to a fact that cannot be changed. Monsters repopulate their
habitats individually, at the day’s encounter level. The result is a dense
island of memorable foes, not waves of disposable one-shot enemies.

## 3. Reference ledger and non-negotiable visual reading

The following files are production references, not decorative mood boards:

| Reference | It approves | It does not approve |
| --- | --- | --- |
| `work/art/battle-ui.png` | one large active fighter, individual threat scale, compact party rail, bronze-dark UI, rich ruin/jungle depth | a fake top progress track, anonymous corner crests, a second enemy that is not simulated, dead UI ornaments |
| `work/art/betty-keyframes.png` | readable pose sequence: card state, unfold, ready, action, impact, recovery; full body in frame | a static portrait standing in for a rig, effects that obscure the actor |
| `work/art/world-visual-grammar-v1.png` | immense magical Bronze Age elven ruins, wild jungle, wild dinosaurs, sparse outsider settlements | an island built mostly from colonial blocks, captive dinosaur pens |
| `content/art/betty.reference_ledger.json` | adult, slender, small-busted, auburn curls, green eyes, medicine gear, short boarding mace, cute/sexy confident silhouette | bulky realism, oversized bust, wizard staff, cropped body or weapon |

Every temporary visual asset must carry machine-readable metadata: stable ID,
purpose, camera, full-body-safe-frame, layer order, required body parts,
weapon socket, replacement source, and test that proves it can be removed
without changing gameplay. No dummy visual is allowed to masquerade as an
approved final asset.

## 4. Runtime architecture before content expansion

### 4.1 Single source of truth

`ExpeditionState` owns campaign truth:

```text
ExpeditionState
  campaign_day: int
  time_segment: Dawn | Day | Dusk | Midnight
  party_ids: CharacterId[1..4]
  active_location_id: LocationId
  route_history: RouteStep[]
  supplies: SupplyState
  character_states: Map<CharacterId, CharacterState>
  named_person_memory: Map<PersonId, DeathMemory>
  habitat_states: Map<HabitatId, HabitatState>
  discoveries: Set<DiscoveryId>
  household_progress: HouseholdProgress
  pending_encounter: EncounterState | null
  rng_seed: int
```

Scenes render this state and send commands to it. A scene must not keep a
second health value, encounter result, day counter or romance-progress value.
The battle receives a snapshot and emits an authoritative result transaction.

### 4.2 Presentation boundary

Game rules use data IDs and records. Godot presentation receives only the
resolved state it needs: actor identity, visible health/guard, selected pose,
action timing, target IDs, safe frame and art contract. Presentation cannot
silently alter damage, unlock a skill, revive a character or advance time.

### 4.3 Save boundary

Save at four boundaries only: entering a location, choosing a route, beginning
an encounter and resolving an encounter/estate action. Autosave writes the
same serializable `ExpeditionState`; it never serializes Godot nodes.

## 5. Build sequence

## Phase A — Make the first chapter navigable

### A1. Campaign bootstrap

Build the start menu, new-game seed, save slot, and first `ExpeditionState`.
New game creates Michael, Betty, a damaged estate, Black Beach as the current
location, and a single available route toward Reception Terrace. The title
card says **Michael Corrigan**, captain of the *Handsome Jack*.

**Exit test:** start a fresh game, quit at the estate, reload, and receive the
same legal actions, character state and seeded Razorbeak encounter.

### A2. Black Beach and estate

Build the shore scene and estate scene as separate playable spaces. Black
Beach gives the wreck, salvage choice and first observation. The estate gives
four immediately useful anchors: workshop, infirmary, map table and household
room. Each anchor has one current action and one textual observation that
changes when state changes. No decorative button exists without a command.

**Exit test:** salvage at the beach changes supplies; return to the estate;
the workshop and infirmary report the changed state after reload.

### A3. Route screen

Build a compact island route presentation, not a full open-world map. It shows
the river landing, safe road, jungle edge, discovered sites, travel cost,
known risk and return path. The visible art follows the world reference:
elven ruins dominate the terrain, vegetation breaks up every route, one small
outsider landmark may appear only where its location data says it exists.

**Exit test:** choose road or jungle edge; advance time; receive distinct
observation text and an encounter seed tied to that route choice.

## Phase B — Make Reception Terrace a real combat location

### B1. Location blockout and authored text

Build Reception Terrace as a two-layer scene: exploration entrance and combat
field. The exploration entrance has the broken elven processional ramp, a
collapsed reception arch, one obvious observation point, one loot point, one
retreat path and the Razorbeak’s territorial sign. The combat field preserves
the same left ruin, distant terraces, jungle depth, and stone floor so combat
is recognizably occurring in that place.

**Exit test:** the player can identify where the party is standing, where they
can retreat, and why the Razorbeak is here without opening a codex panel.

### B2. Exact battle screen composition

Use a 1920×1080 logical canvas. Reserve the 8% safe frame around all
full-body action.

```text
0–150 px     only location/time and current enemy intent; no progress rail
150–780 px   shared battle plane and all active actors
780–1060 px  party cards at lower-left; active skill grid lower-centre

Party foreground:  x 230–650
Contested space:   x 650–1020
Enemy foreground:  x 1020–1430
Enemy rear:        x 1430–1810
```

Current first encounter layout:

- Michael: compact card, bottom left; present as party leader, not expanded.
- Betty: selected active fighter at party foreground; 420×560 interaction box;
  full body, satchel, short mace and effects remain inside safe frame.
- Razorbeak: one active enemy at enemy foreground; 385×420 interaction box;
  full body and bite envelope remain inside safe frame.
- Party rail: only Michael and Betty cards until additional party members are
  authored and active. No empty portrait frames.
- Commands: Betty’s actual seven skills as a centred four-over-three diamond
  grid. D-rank `Guarded Strike` is actionable. C through SSS identify the
  real authored skill and are locked until their actual bond milestone. No
  invented eighth diamond.

**Exit test:** a screenshot at 1920×1080 contains complete actors, weapon,
target and effect envelopes. Every visible card, bar, text label and diamond
has a data source and a live purpose.

### B3. 3D actor standard

The production presentation is a 3D fighter and stage viewed through a fixed
side-view battle camera, with the card rail and command grid remaining 2D.
Temporary paper rigs remain technical blocking tools only. They do not define
the visual source and cannot replace an approved rigged model.

Every production actor has a root, named skeleton, named weapon socket,
separate held weapon, material slots and literal clips. Betty’s minimum 3D
hierarchy is:

```text
BettyRoot → Hips → Spine → Chest → Neck → Head
  LeftUpperArm → LeftLowerArm → LeftHand
  RightUpperArm → RightLowerArm → RightHand → Socket_Weapon_R → boarding_mace
  LeftUpperLeg → LeftLowerLeg → LeftFoot
  RightUpperLeg → RightLowerLeg → RightFoot
  optional: hair, coat-tail, satchel and ampoule-rack secondary bones
```

The active camera must allow pose changes without clipping. The required first
clip set is `Idle_Ready`, `Step_Forward`, `GuardedStrike_Anticipation`,
`GuardedStrike_Contact` and `GuardedStrike_Recovery`. Betty is not a generic
nurse or a large-busted fantasy pin-up: her reference ledger controls the mesh
silhouette, source plate, materials and final animation work.

**Exit test:** a GLB inspection finds the required skeleton, weapon socket and
five named clips. Five camera screenshots of one action use the same mesh and
show no crop. The model cannot be admitted as an animated fighter merely
because it is a good static render.

### B4. First complete command

Implement `skill.betty.guarded_strike` exactly before adding another skill.

```text
Input: choose Guarded Strike → choose legal enemy.
Validation: one adjacent enemy and one threatened party ally share a legal band relation.
Presentation: Betty steps across the ally line, raises her short mace,
  catches the incoming threat, strikes Razorbeak, then plants the mace in a
  recovery guard pose.
Resolution: damage target; grant 2 Guard to threatened ally; log result;
  update cards and enemy intent.
```

This is a five-pose sequence: ready, forward step, anticipation, contact,
recovery. It must be clear at the game camera before any VFX are added.

**Exit test:** player command, target preview, resolution, card update,
Razorbeak reply, victory, defeat and retreat all execute deterministically in
the browser.

## Phase C — Turn combat into a party game

### C1. Active-card transition system

Build the transition that changes a compact card into an active battle actor
and returns that actor to a card. It uses the same `CharacterId`, health,
guard, readiness, pose rig and target context. The transition is gameplay
readable: selected card glows, the active stage box opens, actor enters,
command grid binds to that actor, actor resolves, card updates.

**Exit test:** Betty, Michael and the next authored heroine can each take one
turn using the same transition path. No actor remains visible on the stage
after her state says she is inactive.

### C2. Party roster order

Author the first six heroines in this order so every addition proves a new
combat/job pattern instead of adding cast without playable purpose:

1. **Betty** — combat surgeon; short boarding mace; guard/heal/rescue.
2. **Ayla** — jungle elf scout; mobile spear and living-ruin traversal.
3. **Vix** — fox-folk duelist; pistol/curved blade; position and tempo.
4. **Grisha** — orc officer; polearm; command, force and formation.
5. **Isabella** — pirate aristocrat; rapier; interrupts, marks and social access.
6. **Nara** — tomb scholar; ritual focus; warding, relic logic and alien threat reading.

For each heroine, complete this exact package before adding the next:

- one canonical reference ledger;
- one card state, one clean 3D identity plate and one complete rigged GLB;
- one recruitment scene and estate presence;
- one signature weapon and seven literal D→SSS skills;
- seven action boards with entry, anticipation, contact, consequence,
  recovery and safe-frame notes;
- seven data definitions, legal-target rules and deterministic tests;
- one relationship beat that opens a practical game advantage.

**Exit test:** each heroine’s D-rank skill alone demonstrates her combat role.
No roster entry is a portrait with unimplemented promises.

## Phase D — Make the island persistent and dangerous

### D1. Individual encounter ecology

Build named or generated individual encounters, habitat by habitat. An
encounter has one leader/creature identity, rank, behaviour, intent suite,
territory, drop, return eligibility and daily-level rule. Pack size is not a
difficulty substitute. A high-rank lone animal can control a trail; a named
orc patrol can dictate who enters a ruin.

**Exit test:** three habitat encounters of different rank produce distinct
threat descriptions, action priorities and tactical consequences.

### D2. Midnight Return

At midnight run one explicit campaign transaction:

```text
for each eligible named person: restore life state; increment death memory if dead today
for each habitat: create its daily individual encounter set from its seed and day level
for each changed resident/location: queue its authored acknowledgement text
advance day; save ExpeditionState; request return-flash presentation where visible
```

**Exit test:** defeat a named fixture, reach midnight, reload the next day;
the fixture exists, retains counter/memory, and a habitat contains its correct
new individual encounter set.

### D3. Tomb architecture

Every tomb uses an elven public-purpose plan before it becomes a dungeon:
approach, ceremonial threshold, reception/truth space, burial or archive core,
service/passages, failure condition, exit/return logic. Puzzles state what the
player can observe, what action is possible, what it changes, and what danger
responds. Tombs are not anonymous corridor generators.

**Exit test:** the first tomb has a readable purpose, at least one observation
that changes a choice, a recoverable failure, and a clear return path.

## Phase E — Make the household and story carry progression

### E1. Estate as operating base

Workshop upgrades Michael’s steampunk kit. Infirmary converts supplies and
recovery into expedition readiness. Map room expands route knowledge. Rooms
and people change after real expeditions. Michael’s Echo abilities are
mechanically related to a heroine’s learned pattern, never a copied skill.

**Exit test:** return from Reception Terrace with a discovery; choose one
estate action; begin the next day with a material tactical or route change.

### E2. Haremlit progression

The household’s story structure is direct and positive. Recruitment occurs
through competence, kindness, attraction, shared danger and a clear choice to
join. Once a heroine commits, she stays within the household’s romantic
future. Her relationship scenes with Michael and other women strengthen the
household and unlock practical content. There is no cheating subplot, no
outside male romance lane, no bait-and-switch breakup system, and no coercive
player command.

**Exit test:** every first-six heroine has a recruitment, commitment, household
scene, one girl-with-girl connection and a bond unlock whose exact combat
effect is specified.

### E3. Main threat

The cosmic intelligence is an alien intruder using ancient elven systems to
learn from the island. Its adaptive Champion changes tactics only through
declared observable adaptation records: what party behaviour it observed,
what countermeasure became available, and how the player can identify it.
It never secretly invalidates a build.

**Exit test:** a Champion rematch displays its previous observation and its
new response before combat begins; the player can choose a counter-plan.

## 6. Build discipline

For every implementation pass:

1. State the phase and exit test being built.
2. Read the reference ledger and the relevant implementation contract.
3. Make the smallest coherent change that completes the defined slice, not a
   disconnected component.
4. Validate content and run Godot checks.
5. Open the exported browser build if the work is visual or interactive.
6. Compare the screen to its named reference and list the mismatch honestly.
7. Commit the finished coherent change and push it.
8. Start the next blocked phase.

If a task cannot prove its exit test, it remains unfinished. It is not carried
forward as an invisible assumption.

## 7. Current position

The current code contains early battle simulation, an early paper Betty rig,
and an in-progress first battle screen. It is not yet a valid implementation
of Phase B. The next implementation target is therefore singular: complete
the Reception Terrace encounter screen and Guarded Strike loop to Phase B’s
exit tests. Do not add more locations, heroines, systems, buttons or final art
until that one screen reads like the specified game and works like the
specified game.
