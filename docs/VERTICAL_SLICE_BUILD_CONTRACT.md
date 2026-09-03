# Project 42: First Vertical Slice Build Contract

> **Rule 0 — never fork.** A problem is to be solved, never dodged. Fix the thing,
> replace it outright, or delete the feature — those are the only three moves.
> Never leave two answers to one question standing side by side, and never route
> a parallel path around something you did not want to touch. That is a noodle to
> nowhere, and it is the most serious thing you can do to this codebase.
> Full rule: [`CLAUDE.md`](../CLAUDE.md).

## Purpose

This is the execution order for the first complete playable proof. It is
derived from the Design Bible, not from the current prototype screen. The
slice starts at the Black Beach, reaches the damaged estate, sends a party to
the Reception Terrace, resolves one dangerous encounter, returns home, and
persists the result. A feature belongs in this slice only when it strengthens
that loop.

## Non-negotiable player promise

The player leads Michael and selected adult heroines through a wild island
whose danger is readable. Inactive party members remain as compact cards. The
acting character expands into the same shared battle plane as the enemies,
performs a clear full-body action, then returns to a card state when the
combat state permits. Text provides actionable observation, not anonymous
narration or decorative filler.

## Build order

### 1. Establish one authoritative expedition state

Implement one `ExpeditionState` that owns location, time, selected party,
supplies, injuries, route state, discoveries, encounter state and aftermath.
The estate, route, site and battle may project this state. No scene is allowed
to invent a separate copy of the truth.

**Proof:** a saved state restored at the estate, route, camp, target-selection
or post-battle boundary produces the same next legal commands and the same
seeded outcome.

### 2. Build the exact playable geography

Implement the first connected graph: Black Beach → damaged coastal estate →
river landing → Reception Terrace → processional ramp. Each space has a
stable ID, a visible blockout, entry points, exits, interaction anchors,
authored text, a return rule and persistence policy.

**Proof:** the player can choose the safe road or jungle edge, inspect one
actionable observation, camp or press onward, and return with a changed
expedition state.

### 3. Build the battle composition before final art

The battle is one painted side-view field. The acting heroine occupies the
foreground at a readable but interaction-safe scale. Individual enemies share
the same floor and may occupy the five positional bands: Party Rear, Party
Front, Contested, Enemy Front and Enemy Rear. Four active party members appear
as real cards when inactive; no empty roster slots are drawn.

The UI contains only live information:

- location and time;
- current enemy intent and known damage/risk;
- compact party cards with vitality, guard and readiness;
- the acting character's authored seven-skill D→SSS grid;
- targeting prompts, retreat rule and event consequences when those systems
  exist.

No top progress rail, limit-break crest, unlabelled diamond, ornamental corner
or fake command appears without a stored rule, input behavior and validation.

**Proof:** a screenshot and input recording show Michael plus three heroines
as cards, one selected actor expanded, one enemy intent, one targetable
command and complete bodies/weapons/VFX inside the eight-percent safe frame.

### 4. Build paper rigs as production blockouts

Every temporary actor is a hierarchy of independently movable cut-paper pieces
with a named root, pivots, draw order, socket list and safe frame. Geometry is
allowed to be simple. It must still communicate the character described in the
Bible and reference ledger.

Betty requires head, auburn hair, blouse/corset torso, arms, hands, split coat
tails, satchel, brass ampoule rack, thighs, lower legs, boots and short
bronze-and-glass boarding mace. Her mace follows her hand; boots follow legs;
hair follows head. No static beauty image may substitute for the rig.

**Proof:** ready, step, anticipation, contact and recovery poses use the same
pieces; a pose test demonstrates a parent movement carrying its child pieces.

### 5. Prove the first combat encounter

Use the Reception Terrace encounter with one individual Razorbeak. The first
implemented player command is Betty's D-rank **Guarded Strike**: sweep across
the protected ally line, catch the attack, strike one adjacent enemy, gain
2 Guard for the threatened ally in the same band, then recover in guard.

The encounter includes target selection, legal-target preview, committed
resolution, visible enemy intent, hit/miss, guard result, damage, enemy reply,
victory, defeat and retreat handling. The encounter begins and ends in the
same `ExpeditionState`.

**Proof:** one deterministic browser run demonstrates all command phases and
the result persists after the return to exploration.

### 6. Prove estate consequence and Midnight Return

The estate presents recovery, an infirmary action, one household beat and a
changed object or resident response after the expedition. At midnight, the
Return transaction revives eligible dead named people, records death memory
and counters, and repopulates active habitats with individually generated
monsters. These are campaign transactions, never scene reload effects.

**Proof:** kill or down a named fixture, advance to midnight, observe the
return flash and restore its memory/counter from save data.

### 7. Polish only after loop proof

Final illustrations, animation loops, VFX, sound, expanded cast content and
more locations follow the proof. Every asset package records identity,
camera, parts/pivots, pose, framing, source, admission test and replacement
gate. Every hero skill requires entry, anticipation, contact, consequence and
recovery boards at game scale.

**Proof:** no visual asset is accepted merely because it is attractive at full
resolution. It must read at the actual player camera and preserve its complete
body, weapon, target and effect envelope.

## Current implementation rule

Work only on the earliest incomplete step that blocks the next one. When a
step changes code or authored data: validate content, run Godot verification,
inspect the live browser build when the change is visual, commit the focused
result and push it before proceeding.
