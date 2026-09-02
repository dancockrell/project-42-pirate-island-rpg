# Ayla — design specification (pre-implementation)

This is a design spec, not validated content. It exists so Ayla's seven-skill kit can be implemented later without re-deriving her canon from `outputs/Project_42_Pirate_Island_RPG_Design_Bible.docx`. It intentionally does **not** ship as `content/characters/ayla.json` yet — see "Why this isn't content yet" below.

## Canon (from the design bible, not invented)

The design bible names Ayla as the project's own next vertical-slice heroine (milestone B08: "Build vertical-slice content packages Betty, Ayla, estate, two routes, one tomb..."). Her fixed canon:

- **Character namespace ID:** `ayla` (`character.heroine.ayla`), one of the six core heroines (`betty, ayla, vix, grisha, isabella, nara`).
- **Role:** jungle-elf tomb warden from the Kesh River community, trained to maintain funeral roads, water seals and lineage doors around Veyra Tidehouse.
- **Signature weapon:** recurved bronze spear.
- **Backstory:** broke a prohibited seal to rescue three children during a flood, exposing a concealed containment channel; the council kept her in service but removed her authority to train apprentices. She wants to build a living custodial school that can question a rule before inheriting it.
- **Not established:** exact physical build, skin/hair/eye color, or palette. Betty's identity is locked in `content/art/betty.reference_ledger.json` through a dedicated identity-lock process before any skill art was authored; the same process (an identity-lock production sheet, reviewed and approved) needs to happen for Ayla before her placeholder art or `content/characters/ayla.json` art block can be filled in honestly. Nothing here invents that.

## Her seven-skill kit (bible's exact function/fiction)

| Rank | Skill | Function (from the bible) |
| --- | --- | --- |
| D | Reach Counter | Reaction: attack an enemy that enters the Contested band. |
| C | Structural Scan | Reveal Armor, active construction tags and one valid counter on a construct, fortification or armored target. |
| B | Safe Passage | Move the party through one hazardous band without triggering movement reactions. |
| A | Ward Line | Place a ward between two adjacent bands; the first enemy crossing it takes damage and becomes Staggered. |
| S | Curse Dispel | Remove one identified curse, funerary status or imposed authority effect. |
| SS | Deny Activation | Once per site: cancel one understood machine, ritual or phase activation. |
| SSS | Override Tomb Rule | Once per expedition: replace one discovered tomb rule with an approved alternate rule until the party leaves the site. |

## Proposed deterministic translation (for implementation)

The bible gives fiction and function, not battle-math constants — same as it did for Betty, whose exact numbers (12 base damage, 24 raw counter damage, etc.) were authored to fit the simulation rather than quoted from the bible. The following is a first-pass proposal in the same style, for whoever implements her in `godot-rust/src/battle.rs`:

- **D Reach Counter** — reaction. Triggers the instant a hostile actor's `band` becomes equal to Ayla's `band` by any move (the "Contested band" is simply whichever band they now share). Deals `10 + Ayla's level` raw damage through Guard to that hostile. Unlimited uses; does not consume a turn.
- **C Structural Scan** — targets one hostile (`targetRule: one_hostile`). A pure information reveal: exposes that hostile's current Guard value and one authored counter-tag string to the presentation layer. No damage, no state mutation beyond the normal turn cost — the closest existing precedent is a read-only variant of `actor_focused`.
- **B Safe Passage** — zero-selection normal action (`targetRule: all_living_party_members`, matching Mobile Infirmary's convention). Moves every living party member in Ayla's band or an adjacent band into Ayla's band, in stable actor-ID order, without opening any reaction window.
- **A Ward Line** — names two adjacent bands. Creates a `BattlefieldEffect` anchored between them that persists until triggered once: the first hostile whose move crosses that boundary takes 16 raw damage through Guard and gains a new `Staggered` status for 2 rounds, then the ward removes itself. **Staggered does not exist yet** — it needs a new `StatusKind` variant and a defined mechanical effect (e.g., that actor's next skill use is skipped), analogous to how `Stunned` already blocks command submission.
- **S Curse Dispel** — targets one living party member (`targetRule: one_living_party_member`). Removes all currently-applied negative statuses (reusing the existing `Bleeding, Poisoned, Burning, Stunned` vocabulary — the bible's "curse, funerary status, imposed authority effect" is fictional framing over that same mechanical pool). Restores no Vitality, distinguishing it from Betty's Condition Cleanse.
- **SS Deny Activation** — the bible scopes this "once per site," but the current engine has no concept of a site or encounter chain outside one `Battle`; a faithful multi-encounter version needs a persistent overworld/site-state layer that doesn't exist yet. Proposed engine-scope translation: **once per battle**, zero-selection, cancels the enemy's declared intent for its next turn only (that turn resolves as no action), then the use is spent.
- **SSS Override Tomb Rule** — same "once per expedition" scoping problem as above. Proposed translation: **once per battle**, zero-selection, grants a single-use exemption that cancels the next `Stunned` status application to any living party member this battle.

## Why this isn't content yet

`tools/src/validate.mjs` requires every skill's `animation.eventBindings` to bind only to event kinds Rust's `Battle` actually emits (the `bindableBattleEvents` allowlist), and requires `targetRule` to be one already registered with real Godot targeting behavior. Reach Counter, Ward Line, Structural Scan and both SS/SSS skills need engine concepts that don't exist yet (a `Staggered` status, a band-contest reaction trigger, a non-mutating reveal event, ward battlefield effects, intent-cancellation). Authoring `content/characters/ayla.json` with full animation/presentation records against events Rust never emits would produce content that "validates" but is inert in the actual game — the same failure mode the project's own placeholder-art discipline exists to prevent for art, applied to mechanics. And the character schema requires *exactly* seven working skill records to add a heroine at all — there's no partial-roster path.

Once `battle.rs` implements these five-plus mechanics (most reusable from patterns already in the file: `BattlefieldEffect`, band moves, reaction windows, `StatusKind`), `content/characters/ayla.json` and her seven `content/skills/ayla.*.json` records can be authored the same way Betty's were, with real animation beats bound to real events, and this document can be retired in favor of the code and `docs/ARCHITECTURE.md`.
