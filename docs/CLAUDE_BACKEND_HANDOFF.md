# Claude backend handoff — Project 42: Pirate Island RPG

> **Rule 0 — never fork.** A problem is to be solved, never dodged. Fix the thing,
> replace it outright, or delete the feature — those are the only three moves.
> Never leave two answers to one question standing side by side, and never route
> a parallel path around something you did not want to touch. That is a noodle to
> nowhere, and it is the most serious thing you can do to this codebase.
> Full rule: [`CLAUDE.md`](../CLAUDE.md).

## Read this first

You own the **authoritative game systems**, save data, content validation, and deterministic simulation. You do **not** own visual layout, camera composition, UI styling, character art direction, animation timing, or world set dressing. Those remain on the Godot/frontend track because the project must look and play like a theatrical 3D party RPG rather than a systems demo.

Work in small vertical slices. A feature is not done because a type, class, JSON record, or UI button exists. It is done only when the authoritative state can save, reload, and produce the same legal next commands and ordered results. Every coherent change must include a focused deterministic test, `npm run build:content`, the relevant Godot test, `tools/verify-godot.ps1`, and one commit. Do not rewrite or erase unrelated work.

The authoritative sources are, in order:

1. `docs/ARCHITECTURE.md` — simulation, command/event, targeting, and event-order contract.
2. `docs/VERTICAL_SLICE_BUILD_CONTRACT.md` — exact first playable loop and proof gates.
3. `docs/GAME_BUILD_PLAN.md` — product direction, build order, and fixed player promises.
4. `docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md` — language and scene boundaries.
5. JSON records under `content/` plus schemas and validators — executable authored-data contract.

When prose and executable schema disagree, do not silently choose one. Preserve the current stable ID, document the conflict in the change, and make the smallest compatible correction.

## The game you are implementing

Project 42 is a 3D-presented, side-view tactical party RPG. Captain Michael Corrigan and adult heroines explore a wild tropical island built primarily on reclaimed magical Bronze Age elven infrastructure. Sparse colonial, fox-folk, and orc footholds exist, but the island must never become a built-up colonial map. Dinosaurs are wild. The party reads dense authored observation text, chooses dangerous routes, fights individual high-presence threats, returns to a household estate, and advances the day.

The first playable chapter is the only current gameplay target:

```text
Black Beach -> damaged coastal estate -> river landing -> Reception Terrace
-> one Razorbeak encounter -> return estate -> one estate consequence -> save
```

The player starts with Michael and Betty. Later heroines are not a reason to add fake UI slots, default party members, placeholder combat actions, or incomplete systems now.

## Architecture boundary

### Claude owns

- `ExpeditionState`: location, clock/day, selected party, supplies, injuries, route state, discoveries, encounter reference, aftermath, and durable death memory.
- Campaign save/load and versioned migrations.
- Legal commands and deterministic encounter state.
- Ordered event emission and replay-safe snapshots.
- Target validation, skill cost/use tracking, statuses, persistent battlefield effects, victory/defeat/retreat outcomes, loot, time advancement.
- Individual encounter ecology and deterministic Midnight Return transaction.
- Content schemas, JSON validation, stable-ID integrity, and deterministic bundle generation.
- Test fixtures and tests that prove the systems above.

### Frontend/Godot track owns

- Scene composition, input affordances, battle layout, camera, lighting, VFX, audio, animation, art, and visual safe-frame enforcement.
- Turning returned structured events into visual effects and human-readable text.
- 3D model inspection, rigging, import, materials, socket setup, world blockout, collision and navigation layers.
- Presentation-side targeting prompts. These may filter obvious invalid choices, but cannot replace backend validation.

### Strict prohibitions

- No authoritative mutation in a Godot scene script, animation callback, or UI signal.
- No parsing combat prose to determine mechanical results.
- No human-readable display strings as stable identifiers.
- No random combat, spawn, loot, or target result without a recorded seed/input and a deterministic test.
- No presentation code advancing initiative, granting Guard, deciding targets, or resolving damage.
- No Rust expansion unless profiling shows GDScript cannot meet a stated budget. GDScript is the ordinary runtime language. TypeScript is offline tooling. C/C++ is not authored project code.

## Existing contracts to preserve

### Commands and events

Godot sends only this conceptual command shape:

```json
{
  "command_id": "stable request identity",
  "battle_id": "stable encounter identity",
  "actor_id": "stable actor ID",
  "kind": "skill or system action ID",
  "target_ids": ["ordered stable target IDs"],
  "payload": {}
}
```

The simulation returns ordered events with `event_id`, `command_id`, increasing `sequence`, `kind`, `subjects`, and structured primitive `payload`. Authoritative events drive presentation; they are never reconstructed from the UI.

Keep the encounter state machine exact:

```text
AwaitingActor -> AwaitingCommand -> Resolving -> AwaitingCommand
                                           -> Victory | Defeat
```

The actor selected by initiative is the only actor who may submit a normal command. Rejections happen before mutation and include enough structured facts for the presentation to explain the error.

### Stable IDs and data locality

Stable IDs are lowercase dotted strings. Examples already in use:

```text
character.heroine.betty
character.captain.michael
skill.betty.guarded_strike
enemy.raptor.razorbeak
location.black_beach.reception_terrace
```

Do not rename these casually. Add a record beside the data it describes and extend the content bundle through `tools/src/build-content-bundle.mjs`; do not crawl `content/` dynamically in Godot.

### First encounter: Razorbeak

The Reception Terrace encounter uses one individual Razorbeak. Its visible intent is authoritative. The target decision is deterministic: living party members ordered by lowest vitality percentage, then lowest Guard, then stable actor ID. The decision exposes target, raw damage, projected Guard absorption, projected vitality loss, lethality, current protector, Fatal Intercept availability, and a stable rationale code. Godot may phrase those facts; it must not decide them.

### Betty’s seven authored skills

All seven need authoritative implementations and deterministic tests. Their UI may remain visibly locked until the frontend has the matching target flow and presentation path.

| Rank | Stable record | Literal player-facing skill | Backend result |
| --- | --- | --- | --- |
| D | `skill.betty.guarded_strike` | Guarded Strike | Strike one legal adjacent enemy and give the threatened ally in the same band 2 Guard. |
| C | `skill.betty.condition_cleanse` | Condition Cleanse | Target one living ally; remove up to two negative statuses in urgency order, then restore 8 vitality without exceeding maximum. |
| B | `skill.betty.rescue_charge` | Rescue Charge | Ordered targets: threatened ally, then threatening hostile. Betty moves to ally band, deals 10 normal damage, then intercepts the next hostile attack aimed at that ally. |
| A | `skill.betty.healing_impact` | Healing Impact | Target one living hostile; deal `18 + Betty level` normal damage. Heal the living party member with lowest vitality percentage by half the actual vitality removed. |
| S | `skill.betty.fatal_intercept` | Fatal Intercept | Automatic reaction, one use. When an unredirected hostile attack would defeat an eligible ally, cancel it and counter for 24 normal damage. No nested reaction loop. |
| SS | `skill.betty.mobile_infirmary` | Mobile Infirmary | Zero manual targets. Create one effect; immediately pulse then pulse at Betty’s next two turn starts. Each pulse heals living party members up to 10 and gives 2 Guard. Recast replaces previous effect. |
| SSS | `skill.betty.combat_revival` | Combat Revival | Once per battle, target one defeated ally other than Betty. Remove negative statuses, revive at ceil(40% max vitality), zero Guard, grant immediate bonus turn, resume stored normal initiative continuation. |

Do not "improve" those names with poetic titles. Skill records need literal outcome names because they are input contracts, save-game facts, accessibility labels, test names, and player-facing controls.

## Highest-priority work packages

Complete these in order. Do not start the next package until the previous package has its listed proof.

### B0 — Establish versioned ExpeditionState

Create one explicit versioned campaign object which contains:

```text
save_version
world_seed
day_index
time_of_day
current_location_id
selected_party_ids
supplies
injuries/statuses
route_state
discoveries
active_encounter reference or snapshot
post_battle_aftermath
named_person_death_counters
named_person_death_memory
daily_spawn_records
```

Requirements:

- Create, serialize, deserialize, and migrate the state through a narrow API.
- Reject malformed, future-version, or unresolvable stable-ID records safely; never create silent defaults for unknown characters, skills, locations, or seeds.
- Restore at estate, route, target selection, combat boundary, and aftermath boundary without changing the next legal command or generated daily encounter.
- Keep presentation-only values out of this object: screen coordinates, animation time, card selection glow, camera state, current VFX, and browser/UI state.

**Proof:** a fixture is saved at each boundary, reloaded, queried for legal commands, and compared with the pre-save result. The test also verifies stable JSON ordering so equivalent state emits identical bytes.

### B1 — Connect the first geography as data

Use the current world records as the starting point. Complete the connected graph:

```text
location.black_beach -> location.black_beach.estate
-> location.black_beach.river_landing
-> location.black_beach.reception_terrace
-> location.black_beach.processional_ramp
```

Each location record must define stable ID, region, visible exits, entry anchors, observation records, interaction anchors, route cost/time, return policy, persistence policy, and encounter eligibility. Each exit points to a stable destination and has one clear availability predicate.

At the river approach, implement two actual options: **safe road** and **jungle edge**. They must have distinct time/supply/risk consequences that are persisted and available to future observation text. Do not make their difference cosmetic.

**Proof:** beginning at Black Beach, deterministic commands can inspect one observation, travel either route, enter Reception Terrace, retreat or return, and preserve changed expedition state.

### B2 — Finish the Guarded Strike encounter loop

The Rust/native slice has useful prototype mechanics. Audit it against the contracts and expose only its structured result through the `SimulationPort` boundary. Keep compatibility with the existing GDScript mock port and its tests.

Required command lifecycle:

```text
enemy decision -> visible intent facts -> legal target query -> command submit
-> CommandAccepted -> actor focus -> damage/guard/status events -> defeat if any
-> turn end -> next turn -> next visible enemy intent
```

Add command idempotency: submitting the same `command_id` against the same authoritative snapshot must not double-resolve. Return the recorded result or a structured duplicate result according to the established protocol.

Add explicit retreat rules for the first encounter. Retreat is a legal system command only when its stable encounter rule says so. It cannot be a UI back button that resets the scene. Retreat outcome must update `ExpeditionState`, consume declared time/cost, and retain any declared injuries or discoveries.

**Proof:** a deterministic test covers legal Guarded Strike, invalid actor, invalid target, duplicate command, hostile turn, Guard absorption, victory, defeat, and legal/illegal retreat. A save at a command boundary reloads to the same next decision.

### B3 — Individual ecology and Midnight Return

Implement Midnight as a single atomic transaction. It is campaign simulation, never a level reload or visual trick.

```text
validate transition to midnight
increment day once
restore eligible named people to alive state
preserve every death counter and death-memory fact
for each active habitat, emit deterministic daily individual spawns
record each spawn’s stable instance, level, physical variant, condition,
purpose, loot seed, region, and source day
queue acknowledgement facts for affected people/locations
commit once or roll back entirely on failure
```

Requirements:

- A named person killed earlier remembers it and retains the count. Their return does not erase the counter.
- Spawn groups are one by default. The island has individual D&D-style power relationships, not disposable packs.
- Same world seed + day + region + slot must reproduce the same spawn record.
- New day changes only the records the transaction declares. It must not reset unrelated route progress, discovery, active relationship progression, or ordinary save facts.

**Proof:** kill a named fixture, advance to midnight, save/reload, then verify alive state, counter, memory fact, and deterministic individual habitat records. Repeat with same seed/day fixture and compare byte-for-byte.

### B4 — First estate consequence

Implement one small but real estate transaction after Reception Terrace. The return record must make a discovery available, then allow exactly one estate action such as an infirmary recovery or Michael workshop adjustment. It must write a material tactical or route change into `ExpeditionState` for tomorrow.

Do not build a generic base-building tree. Implement one actual consequence, one actual cost, one actual changed fact, and one save/load proof.

**Proof:** survive/retreat from the terrace, return estate, take the action, save/reload, and show its changed tactical/route fact in legal-state data.

## Content and validation work

Extend the schemas instead of allowing ad-hoc JSON fields. For every new record:

- include ownership and lifecycle;
- list dependencies using stable IDs;
- include `source_status`, `art_state`, and `accessibility_description` when a record has presentation meaning;
- validate relationships and unique IDs;
- add the record to the deterministic content bundle;
- include a failing fixture/test for malformed data where the schema alone cannot prove the relationship.

The TypeScript toolchain is the authoritative offline validator. Useful commands:

```powershell
Set-Location tools
npm run build:content
npm run validate
```

Then run the relevant Godot test and `./tools/verify-godot.ps1` from the repository root. Do not claim a visual test passed merely because the backend tests passed.

## Save and replay requirements

Every authoritative snapshot must be sufficient to reconstruct:

- current phase and active actor;
- ordered turn/round continuation, including Combat Revival’s forced bonus turn;
- actor vitality, Guard, statuses, positions/bands, skill uses, and defeat state;
- persistent battlefield effects and remaining pulses;
- deterministic seeds and commands already resolved;
- named death counters/memory and daily spawn ledger;
- current expedition location/time and return aftermath.

Use integers for simulation values. Do not rely on floating-point percentage comparison: use integer cross multiplication with stable actor-ID tie breaks. Put random input into a seed/stream record and test it. Never rely on JSON object insertion order for a game result.

## Frontend integration contract

Provide narrow, typed facts for these frontend consumers:

| Consumer | Backend must provide | Backend must not provide |
| --- | --- | --- |
| `TargetingSession` | `targetRule`, target list, target ordering, reason codes | screen positions, highlight colours, UI copy |
| battle screen | active actor, legal commands, snapshot, ordered events, enemy intent facts | camera focus, animation duration, VFX art |
| `CombatTextRenderer` | primitive event payload facts and stable subject IDs | prewritten prose as authority |
| world scene | stable location/exit/interactions and persisted consequence facts | meshes, collision, navigation, light placement |
| estate scene | actionable estate transactions and resulting durable facts | dialogue layout, relationship scene staging |

Return error/result codes as stable lower-case dotted IDs or compact enums, never only English sentences. The frontend can map them to concise player-facing text and accessibility narration.

## Visual and asset status

Betty now has a new four-view Magnific candidate generated from the approved leather-and-lace/tartan design. It is **not yet an accepted runtime asset**. The frontend track will download it, inspect mesh structure, test scale/face/hands/satchel/mace, record provenance, and decide whether it can be rigged. Do not add a hard backend dependency on a GLB, animation clip, render path, or texture name.

The old `Candidate 01` under `work/art/vendor/magnific/betty-3d/N2cYw4m6D9/` remains a known static blocking candidate with zero skins and zero animation clips. It must not become an active battle actor.

## Definition of done for this handoff

The handoff has succeeded when the following can be demonstrated in one deterministic test run:

1. Start at Black Beach with Michael and Betty.
2. Travel by either river option and persist its actual consequence.
3. Enter Reception Terrace and resolve Guarded Strike against the individual Razorbeak through the command/event boundary.
4. Persist victory, defeat, or legal retreat; return to estate.
5. Take one estate action that changes a durable tactical or route fact.
6. Advance Midnight, restore an eligible named person without losing death memory, and regenerate individual habitat encounters from the day seed.
7. Save and reload at any listed boundary with unchanged legal next commands and deterministic results.

When a package is complete, commit only that coherent package and push it. In the commit body or PR description, state the exit test, exact commands run, and any remaining frontend-only work. Do not bundle art changes, visual redesigns, or speculative new characters into backend work.
