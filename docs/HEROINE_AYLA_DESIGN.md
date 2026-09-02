# Ayla — design specification (pre-content)

This is a design spec plus a progress record, not validated content. It exists so Ayla's seven-skill kit can be tracked without re-deriving her canon from `outputs/Project_42_Pirate_Island_RPG_Design_Bible.docx` each time. It intentionally does **not** ship as `content/characters/ayla.json` yet — see "Why this isn't content yet" below.

## Canon (from the design bible, not invented)

The design bible names Ayla as the project's own next vertical-slice heroine (milestone B08: "Build vertical-slice content packages Betty, Ayla, estate, two routes, one tomb..."). Her fixed canon:

- **Character namespace ID:** `ayla` (`character.heroine.ayla`), one of the six core heroines (`betty, ayla, vix, grisha, isabella, nara`).
- **Role:** jungle-elf tomb warden from the Kesh River community, trained to maintain funeral roads, water seals and lineage doors around Veyra Tidehouse.
- **Signature weapon:** recurved bronze spear.
- **Backstory:** broke a prohibited seal to rescue three children during a flood, exposing a concealed containment channel; the council kept her in service but removed her authority to train apprentices. She wants to build a living custodial school that can question a rule before inheriting it.
- **Not established:** exact physical build, skin/hair/eye color, or palette. Betty's identity is locked in `content/art/betty.reference_ledger.json` through a dedicated identity-lock process before any skill art was authored; the same process needs to happen for Ayla before her placeholder art or `content/characters/ayla.json` art block can be filled in honestly. Nothing here invents that.

## Her seven-skill kit (bible's exact function/fiction)

| Rank | Skill | Function (from the bible) | Engine status |
| --- | --- | --- | --- |
| D | Reach Counter | Reaction: attack an enemy that enters the Contested band. | **Implemented** |
| C | Structural Scan | Reveal Armor, active construction tags and one valid counter on a construct, fortification or armored target. | **Implemented** |
| B | Safe Passage | Move the party through one hazardous band without triggering movement reactions. | **Implemented** |
| A | Ward Line | Place a ward between two adjacent bands; the first enemy crossing it takes damage and becomes Staggered. | Blocked |
| S | Curse Dispel | Remove one identified curse, funerary status or imposed authority effect. | **Implemented** |
| SS | Deny Activation | Once per site: cancel one understood machine, ritual or phase activation. | Blocked |
| SSS | Override Tomb Rule | Once per expedition: replace one discovered tomb rule with an approved alternate rule until the party leaves the site. | Blocked |

## Implemented (4/7), in `godot-rust/src/battle.rs`

The bible gives fiction and function, not battle-math constants — same as it did for Betty, whose exact numbers (12 base damage, 24 raw counter damage, etc.) were authored to fit the simulation rather than quoted from the bible. These four are implemented and covered by dedicated Rust unit tests (`curse_dispel_removes_every_negative_status_without_healing`, `structural_scan_reveals_guard_without_mutating_the_target`, `safe_passage_moves_adjacent_living_allies_into_aylas_band_in_stable_order`, `reach_counter_strikes_a_hostile_that_attacks_an_ally_sharing_aylas_band`, `reach_counter_does_not_trigger_when_ayla_does_not_share_the_targets_band`):

- **D Reach Counter** (`skill.ayla.reach_counter`) — automatic reaction inside `apply_damage`, reusing Fatal Intercept's `ReactionWindowOpened`/`ReactionTriggered` template. Rather than literal band-crossing movement (no hostile in this engine ever moves bands — the razorbeak only attacks), the trigger is **a hostile's declared attack targeting a party member who currently shares Ayla's band** — a faithful translation of "attack an enemy that enters the Contested band" using the concept the engine actually has (shared band = contested). Deals `10 + Ayla's level` raw damage to the attacker through Guard. Gated by presence of a `skill_uses_remaining["skill.ayla.reach_counter"]` entry (never decremented — genuinely unlimited-use, unlike Fatal Intercept's spent charge), so it stays inert for every existing test/fixture where Ayla appears as an inert party member without that entry.
- **C Structural Scan** (`skill.ayla.structural_scan`) — targets one hostile (reuses the existing `one_hostile`-shaped friendly-fire check). A new `BattleEvent::TargetInspected { actor_id, target_id, guard_revealed, counter_tag }` variant carries the reveal; the resolver only reads state, never mutates it. Implemented as a normal turn-consuming action — the bible doesn't require it to be free, and a "free action" turn economy doesn't exist anywhere in the engine, so it wasn't invented for one skill.
- **B Safe Passage** (`skill.ayla.safe_passage`) — zero-selection normal action, reusing the `expected_targets == 0` dispatch branch Mobile Infirmary established. Moves every living party member whose band is exactly one away from Ayla's band into her band, in stable actor-ID order (`BTreeMap` iteration order), emitting `ActorMoved` per mover. Allies already in her band, or two-or-more bands away, don't move.
- **S Curse Dispel** (`skill.ayla.curse_dispel`) — targets one living party member (same faction-match friendly-fire rule as Condition Cleanse). Removes **all** currently-applied negative statuses (reusing the existing `Bleeding, Poisoned, Burning, Stunned` vocabulary — the bible's "curse, funerary status, imposed authority effect" is fictional framing over that same mechanical pool), copying Condition Cleanse's sort-by-priority-then-drain pattern but with no cap and no healing, distinguishing it from Betty's version.

## Still blocked (3/7) — named, specific missing subsystems

- **A Ward Line** — needs hostile band movement, which doesn't exist for any actor in the simulation today, plus a `Staggered` status (no dynamic `apply_status` function exists anywhere — statuses today are only ever set at actor construction or removed by skills, never applied mid-battle).
- **SS Deny Activation** — the bible scopes this "once per site," but the engine has no concept of a site or encounter chain outside one `Battle`. There's also no override path for a computed `EnemyDecision` before it resolves.
- **SSS Override Tomb Rule** — same "once per expedition" scoping gap, plus it would need to pre-emptively intercept a *future* status application that, per the point above, no skill in the game currently performs dynamically.

## Why this isn't content yet

`tools/src/validate.mjs` requires every skill's `animation.eventBindings` to bind only to event kinds Rust's `Battle` actually emits, and requires `targetRule` to be one already registered with real Godot targeting behavior — and the character schema requires *exactly* seven working skill records to add a heroine at all, with no partial-roster path. The four implemented skills above could now honestly pass that bar; the three blocked ones still can't without the missing subsystems named above. Authoring all seven as content today, with the blocked three faked to fit the schema, would produce content that "validates" but is mechanically inert or misleading — the same failure mode the project's own placeholder-art discipline exists to prevent for art, applied to mechanics.

Once Ward Line, Deny Activation and Override Tomb Rule land in `battle.rs`, `content/characters/ayla.json` and her seven `content/skills/ayla.*.json` records can be authored the same way Betty's were, with real animation beats bound to real events, and this document can be retired in favor of the code and `docs/ARCHITECTURE.md`.
