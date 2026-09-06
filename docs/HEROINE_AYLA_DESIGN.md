# Ayla — design specification and site-rule spec

This document exists so Ayla's seven-skill kit can be read without re-deriving her canon from `outputs/Project_42_Pirate_Island_RPG_Design_Bible.docx` every time, and so the site-rule system her two highest commands rest on has one written spec (ship-plan card H5, written inside A7 beside the code).

Its first edition, on `feature/ayla-bridge-art` (PR #2), was a *pre-content* progress record: five of seven skills implemented, two blocked, and no `content/characters/ayla.json`. **That is no longer the status.** All seven are executable, all seven are authored content, and the two blocked ones are unblocked by the site-rule seam specified below. PR #2 itself could never merge — A5 (bands, Composure), A6 (Captain Michael) and A10 (bond ranks) rewrote `godot-rust/src/battle.rs` under it — so its diff was treated as the specification and its eight tests were re-landed on the current engine against the current fixture.

## Canon (from the design bible, not invented)

- **Character namespace ID:** `character.heroine.ayla`, one of the six core heroines (`betty, ayla, vix, grisha, isabella, nara`).
- **Role:** jungle-elf tomb warden from the Kesh River community, trained to maintain funeral roads, water seals and lineage doors around Veyra Tidehouse.
- **Signature weapon:** recurved bronze spear.
- **Backstory:** she broke a prohibited seal to rescue three children during a flood, exposing a concealed containment channel. The council kept her in service but removed her authority to train apprentices. She wants to build a living custodial school that can question a rule before inheriting it.

**Not established — `blocked: needs decision`.** Her exact physical build, her skin, hair and eye colour, and her palette. Betty's identity was locked in `content/art/betty.reference_ledger.json` by a dedicated process *before* any of her art or skill VFX was authored; that process has not happened for Ayla. Nothing in this repository invents it:

- `content/characters/ayla.json` carries an accessibility description that says out loud that it is a placeholder and describes only what is canon — an adult jungle-elf tomb warden with a recurved bronze spear.
- Her two `content/art/placeholders.json` entries are gated on `approved Ayla identity sheet from the identity-lock process (not yet performed)`.
- Her ten `presentation.vfx.ayla.*` entries carry the single palette entry `palette_unestablished_pending_ayla_identity_lock`. No colour is named anywhere for her.

That gate is the *only* thing outstanding for her. It is an art decision, not an engineering one.

## Her seven-skill kit: seven of seven

| Rank | Skill | Function (from the bible) | How it is translated |
| --- | --- | --- | --- |
| D | Reach Counter | Reaction: attack an enemy that enters the Contested band. | Automatic reaction inside `apply_damage`, on Fatal Intercept's `ReactionWindowOpened`/`ReactionTriggered` template. No hostile in this engine ever moves bands, and a shared band **is** a contested one, so the trigger is a hostile's attack on a party member standing where Ayla stands. `10 + level` raw damage through Guard. Unlimited use: the `skill_uses_remaining` entry is a presence gate, never decremented. |
| C | Structural Scan | Reveal Armor, active construction tags and one valid counter. | One hostile. `TargetInspected` carries the Guard it read and one of two counter tags. The resolver reads state and writes none. A turn-consuming normal action — a free-action turn economy does not exist in this engine and was not invented for one skill. |
| B | Safe Passage | Move the party through one hazardous band without triggering movement reactions. | Zero-selection action in Mobile Infirmary's shape. Every living ally exactly one band away moves into Ayla's band, in stable actor-ID order. "Without triggering movement reactions" is free here: there are none, and the record says so rather than implying a system that does not exist. |
| A | Ward Line | Place a ward between two adjacent bands; the first enemy crossing it takes damage and becomes Staggered. | Zero-selection action placing one ward at Ayla's band. "Crossing" is translated exactly as Reach Counter translates "entering": the first hostile attack on a party member in the warded band. `8 + level` raw damage back through Guard, plus a one-round `Staggered` through `apply_status`. The ward is spent on that first trigger. |
| S | Curse Dispel | Remove one identified curse, funerary status or imposed authority effect. | One living party member. Removes **every** negative status and heals nothing — which is what distinguishes it from Betty's rank C Condition Cleanse (two statuses, eight Vitality). `Shaken` is deliberately left standing: Composure at zero is A5's status, restored by Composure and not lifted by a dispel. |
| SS | Deny Activation | Once per site: cancel one understood machine, ritual or phase activation. | One hostile. Applies a one-round `Stunned` and emits `ActivationDenied`. Once per **site** — see below. |
| SSS | Override Tomb Rule | Once per expedition: replace one discovered tomb rule with an approved alternate rule until the party leaves the site. | Zero-selection. Takes the first site rule still in force out of the fight and emits `SiteRuleOverridden`; the bridge writes the ID into `ExpeditionState::suppressed_site_rules`, which is what makes the override outlast the fight. Once per **expedition** — see below. |

Numbers (`10 + level`, `8 + level`) are authored to fit the simulation, exactly as Betty's 12 and 24 were: the design bible gives fiction and function for a skill, never battle-math constants. `content/skills/ayla.*.json` owns them and `reach_counter_and_ward_line_deal_their_authored_damage` holds Rust equal to the records.

Rank letters are the seven-rung ladder A10 owns (`battle::rank_index`, `validate.mjs`'s `bondRanks`). A10's bond gate applies to Ayla exactly as it does to Betty: at bond rank D her rank C Structural Scan is refused before anything mutates (`ayla_at_bond_rank_d_cannot_spend_her_rank_c_structural_scan`).

## The site-rule spec (H5)

### The problem PR #2 named

PR #2 could not implement the last two skills, and said why: "the bible scopes this 'once per site,' but the engine has no concept of a site or encounter chain outside one `Battle`", and "'until the party leaves the site' is expedition-level state, which is `ExpeditionState`'s domain, not `Battle`'s. Implementing it means designing how an expedition-level rule override reaches into a `Battle` instance's resolution — a real architectural decision this document isn't making unilaterally."

This is that decision.

### A site rule is an authored record

C5 registered twenty-six `site_rule.*` IDs from `content/dungeons/tomb_of_returning_names.json` and treated them as **opaque**, saying plainly that A7 owned what one does. They are no longer opaque:

```json
{
  "id": "site_rule.tomb.grave_watch",
  "displayName": "Grave Watch (placeholder: no site rule is named for players yet)",
  "effect": { "guard_regen_per_round": 2 },
  "metadata": { "implementationOwner": "rust-simulation", "maturity": "executable-core", "notes": [ … ] }
}
```

`content/site_rules/<slug>.json` is now the owner of every one of those IDs; the dungeon record and the tomb's world cells *reference* them like any other stable ID. The file's own name is the ID's slug, and a Rust test says so by name if it ever stops being.

**The effect vocabulary is closed.** It is `SiteRuleEffect` in `godot-rust/src/strategy/site_rule.rs`, and there are exactly two shapes:

- `{ "guard_regen_per_round": <positive integer> }` — while this rule is in force, every living hostile regains that much Guard at each round boundary. `site_rule.tomb.grave_watch` is `2`: the tomb keeps its own account, and it keeps its own dead.
- `{ "needs_decision": true }` — the rule is registered and what it does has not been decided. This is the honest answer for **twenty-five of the twenty-six**, and it is written down rather than defaulted, so a rule cannot quietly become a no-op by omission.

There is no catch-all variant and no `Unknown` arm: a vocabulary that accepts anything is not a vocabulary. A record whose `effect` names neither key fails to deserialize in Rust *and* fails `node tools/src/validate.mjs`, in the same words. `{"needs_decision": false}` is refused too — it would claim the decision is made and then name no mechanic. New effects join by adding a variant here and a branch in `Battle`, never by a record inventing a key.

The registry is held equal to the directory in both directions, and against the dungeon record that selects the rules: an authored rule no dungeon selects is a noodle to nowhere, and a selected rule with no record is a mechanic nobody wrote (`the_site_rule_registry_equals_the_authored_directory`, `every_site_rule_the_tomb_selects_has_a_record_and_the_other_way_round`, plus the JS check on the same pair).

### A place carries its rules

`CellDefinition` and `LocationRecord` gain `site_rule_ids` and `dungeon_id`, read out of C5's `dungeonContext` block through the one constructor, `Geography::from_authored`. The Rust fixture and `content/world/*.world_cell.json` are held equal on both fields, **in order** — the order is load-bearing, because Override Tomb Rule takes the first rule still in force (`fixture_matches_the_authored_world_cells`).

`LocationRecord::site_id()` is the whole definition of "site": the cell's dungeon where it declares one, and the cell itself where it does not. The whole tomb is one site, so walking into the next room does not refresh a once-per-site charge; a cell outside every dungeon is its own site. There is no third answer and no "unknown site".

### The campaign arms the battle

B17 built `Battle::prototype_vertical_slice_from_bond_ranks`, the one campaign-shaped constructor. Its successor is `prototype_vertical_slice_from_campaign(&CampaignBattleSetup)`, which carries four campaign facts instead of one:

| Field | Owner | Meaning |
| --- | --- | --- |
| `bond_ranks` | `ExpeditionState::bond_ranks` (A10) | where each woman's bond stands |
| `site_rules` | the encounter's location's `site_rule_ids`, minus `suppressed_site_rules`, resolved against the registry | what holds here |
| `deny_activation_charges` | `ExpeditionState::site_denial_charges`, keyed by `site_id()` | whether this site still has its one denial |
| `override_tomb_rule_charges` | `suppressed_site_rules.is_empty()` | whether this expedition still has its one override |

`ExpeditionState::battle_setup` builds it; `Project42ExpeditionBridge::begin_pending_battle` carries it. The bridge decides nothing.

The last row is deliberate: "once per expedition" and "has anything been suppressed" are the same fact, so there is one field and not a second flag beside it free to disagree.

### The battle applies them, and writes back

`Battle` holds the active rules in the cell's authored order and applies them in one place, `apply_site_rules_at_round_boundary`, called from `begin_round` — so a rule cannot be applied on one path through `advance_turn` and skipped on the other. Guard arrives through the same `GuardChanged` event the presentation layer already binds, so a site rule needs no new spelling on the Godot side.

Two events travel back out. `ExpeditionState::record_battle_site_events` is the single reader of both, so the bridge carries no rule of its own about what an event means:

- `SiteRuleOverridden { rule_id }` → inserted into `suppressed_site_rules`. Nothing removes an entry: whether an override can be revoked is not a decision this card makes, and when it is made it arrives as a method beside the writer.
- `ActivationDenied` → the current site's charge is written down.

Both survive a save: the two fields are appended `#[serde(default)]` members of `ExpeditionState` with their keys in `godot-rust/tests/save_migration.rs`, and `a_suppressed_site_rule_survives_a_save_and_is_still_not_in_force` proves the round trip.

### One thing the turn cycle had to answer for

Ayla's Deny Activation is the first thing in the game that stuns a *living* actor mid-fight. Before it, `Stunned` was only ever set at actor construction, and no live battle ever produced one — so a stunned actor's turn simply arrived, every command for it was refused with `ActorIncapacitated`, and the fight could not continue. `advance_turn` now spends one round of the stun when the turn opens and moves on, bounded by the turn order. That *is* "cancel one hostile's declared action for the turn"; without it the skill would have deadlocked the battle it was supposed to interrupt.

### What is open

- **Twenty-five of the twenty-six rules' mechanics** — `needs decision`. Each record says so in its own `effect`, and each display name is an explicit placeholder derived from the ID's own slug. No rule has been named for players.
- **Which rule Override Tomb Rule replaces, and what it is replaced with** — `needs decision`. There is no screen for the player to choose a rule from, so the engine takes the first still in force in the cell's authored order: a deterministic stand-in for a choice nobody can make yet, not a design claim. The bible's "approved alternate rule" is authored fiction and no mechanic; nothing here pretends it is one.
- **A third owner's rule set** — still C5's open item. The tomb record carries two sets, and `select_site_rules` gives every owner that is not the corrupted variant the default set.
- **Ayla's physical identity** — as above, and it gates her art and nothing else.

## Where the code is

| Concern | File |
| --- | --- |
| The effect vocabulary, the record, the registry | `godot-rust/src/strategy/site_rule.rs` |
| The seven skills, `apply_status`, the round boundary, `CampaignBattleSetup` | `godot-rust/src/battle.rs` |
| `site_rule_ids`, `dungeon_id`, `site_id()` | `godot-rust/src/geography.rs` |
| `suppressed_site_rules`, `site_denial_charges`, `battle_setup`, `record_battle_site_events` | `godot-rust/src/expedition.rs` |
| Registry loading, event write-back, the wire | `godot-rust/src/godot_bridge.rs`, `game/scripts/simulation/native_expedition_port.gd` |
| Her records | `content/characters/ayla.json`, `content/skills/ayla.*.json`, `content/site_rules/` |
