# Project 42 Ship Plan

The path from the current checkout to a product a stranger can buy. First
written 2026-09-04 from a full survey of this repository, every open branch,
the design bible, the production contracts in `docs/`, and the sibling
`dr-companion` repository; rebuilt the same day when the **Pirate Island
Continuation Brief** (`docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md`) arrived
and became the current design authority.

**What owns what.** `docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md` is the design
authority. `docs/GAME_BUILD_PLAN.md` is the **design contract** — the accepted
system contracts, the dependency order, and which older documents survive; it
was rewritten on `main` in parallel with this plan's first edition, and it
wins on direction. **This document owns the execution path only**: what is
actually built, in what order, by whom, with what proof. Where this plan and
the build plan disagree about direction, the build plan is right and this one
is corrected.

This is two documents in one. **Part I** (§1–§4) is the argument: what the
game now is, where the code actually is, what shipping means, the milestones
and estimates. **Part II** (§5–§7) is the work: every task, in lanes that
run in parallel, written so an agent with less context than the author can
pick one up, do it, prove it, and mark it done here without asking anyone.
**Part III** (§8–§9) is the bookkeeping and the parallel-work protocol
that keep this document true.

The brief's discipline applies throughout: **Accepted** decisions are
carried as written; anything this plan adds is **Provisional** until
approved; what the brief leaves open is listed as **Open**, never filled in
by invention. No faction proper names are used. The protagonist is **Captain
Michael**.

---

# Part I — The argument

## 1. What the game is now

### 1.1 The authority

`docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md` (5 September 2026) is the current
design authority. Its compact statement: *an autonomous RTS war experienced
through Captain Michael and four active female companions. The island is a
complex graph of playable rooms whose settlements, routes, buildings,
dungeons, terrain, weather, populations and ownership change as six factions
act. Captain Michael begins as a shipwreck survivor and creates a sixth
faction: a powerful alternate-1870s steampunk force. The faction plays mostly
automatically and is easy to direct. The player may pause normally, pursue an
organic path, and reach multiple victories, mixed outcomes, or genuine defeat.*

Accepted and load-bearing for this plan:

- **Five heroes:** Captain Michael and four women who are investigators,
  leaders, quest drivers, and companions — never passive. Betty and Ayla are
  established in this repository; **the other two are Open** (brief §20).
- **Six factions:** fox people, colonial powers, pirates, elves, the Cthulhu
  faction, and Michael's faction created during play. **No proper names.**
- **One map node is one playable room** with a full room contract (function,
  entrances/exits, camera, dimensions, circulation, slots, landmarks, spawn
  points, encounter space, material language, avoid-list). The island is a
  complex graph — routes, loops, rivers, bridges, sea routes, caves,
  chokepoints from geography. Not three lanes.
- **Factions act autonomously** — gather, build, spawn or recruit, route,
  trade, raid, defend, attack, relate, recover, expand, eliminate. Michael's
  faction takes strategic intent (Protect, Supply, Develop, Expand, Pressure,
  Attack, Support, Investigate, Avoid, Withdraw) and executes routine work
  itself; it explains a major directive in ordinary language before acting.
- **Buildings do not manufacture women.** Michael's buildings make machines,
  equipment, capacity and services; women join through recruitment,
  relationships, rescue, migration, factional change.
- **Dynamic difficulty from real board strength**, never global scaling.
  Dungeons generate from a `DungeonContext` (seed, node, owning faction,
  building tier, corruption, weather, epoch, budgets), never a palette swap.
- **Weather is tied to magic; Cthulhu corruption escalates**; confrontation
  begins by discovery, Day 100, or maximum hidden pressure.
- **Normal pause stops everything.** Calm UI; three information levels
  (ambient / notable / urgent). Strategic planning available while paused.
- **Faction elimination is persistent.** Organic paths; many victory, loss
  and mixed-ending families; a long campaign can still end in real defeat.
- **Fixed-view isometric presentation.** Blockouts first; standard
  building envelopes; **animation deferred**; assets future-ready (pivots,
  sockets, rigs, metadata).
- **Dual clocks** (`GAME_BUILD_PLAN.md`): world time and Cthulhu
  patience/heat are distinct state dimensions. Advancing time must not
  silently imply an equal heat increase, and each must be independently
  testable.
- **Persistence:** seed, graph, ownership, buildings, queues, resources,
  relationships, goals, forces, weather, corruption, hidden pressure,
  elimination, dungeon signatures, companion theories, recruitment and
  romance state; deterministic replay; pausing never changes outcomes.

Settled outside the brief and still standing: romance is a **headline
feature**, fade-to-black in the base game, with an optional **presentation
override pack** on adult storefronts (§7 C8/B10/E9); rigging is a switch on
the generator; animation waits for the next-generation tool; the game ships
on whatever Godot exports, desktop first.

### 1.2 What the brief supersedes in this repository

The design bible (`work/build_design_bible.py`) and `GAME_BUILD_PLAN.md`
were written for a linear, six-heroine, act-structured party RPG. The
brief keeps their character-scale combat, tomb grammar, Midnight Return,
household tone and asset discipline, and replaces their frame. Task H9/H10
have marked these as superseded in place:

| Was | Now (brief) |
|---|---|
| Six heroines: Betty, Ayla, Vix, Grisha, Isabella, Nara (`req.scope.cast.core` LOCKED) | Four women; Betty and Ayla established; two Open. Vix/Grisha/Isabella/Nara are **not confirmed**. |
| Prologue + four acts + finale; 20–30 h (`req.scope.campaign.duration`) | Organic path; no fixed duration; Day 100 is an in-world threshold, not a length. |
| Player-named protagonist with six backgrounds | Captain Michael. |
| One adaptive Champion as the main threat | A Cthulhu **faction** with hidden → emerging → dominant stages and a summoning endgame; the Champion concept is **Open** as to whether it survives as that faction's agent. |
| Static habitats as the only source of danger | Faction forces, convoys, patrols, refugees and monsters moving on the graph; habitats remain the wild-fauna layer. |
| Party of 1–4 (`party_ids: CharacterId[1..4]`, enforced in code) | Five (A11). |
| Side-view theatre as the only presentation | Fixed isometric board for the world (Accepted). **Battle presentation is Open** — see §4. |

Not superseded, and still LOCKED in the bible: five combat bands, Composure,
the seven-rank skill kits for established women, the reference ledger and
safe-frame rules, Midnight Return with death memory, the tomb grammar, the
household's romantic tone.

## 2. Where the code actually is

Every claim cites the file that proves it.

- **The character-scale simulation is deep and now unified.**
  `godot-rust/src/` on `backend/b0-expedition-state` (PR #3): `ExpeditionState`
  save/load, a location graph that already *is* "one node, one room"
  (`Geography`, now built from authored data by
  `from_authored(cells, portals, triggers)`), travel with time and supply
  cost, retreat, Midnight Return with death memory, one-individual-per-habitat
  ecology with day/night rosters, roaming hunters, the Tomb of Returning Names
  with a discovery-gated archive core and a recoverable failure branch, one
  estate action. The merge with `codex/battle-frontend-vertical-slice`
  **shipped** as `6b9d275`: one `ExpeditionState`, their Godot campaign bridge
  rewired onto `Geography`, defeat and retreat now resolved (their bridge
  resolved victory only). **110 unit + 1 slice test pass; fmt and
  `--features godot-ext` clean.**
- **The strategic layer does not exist.** No faction, building, ownership,
  resource, directive, force, weather, corruption or journal type exists in
  Rust, content, or Godot. This is the largest body of new work in the plan
  (Lane S).
- **Three ID namespaces for the same five places.** `world.cell.*` /
  `world.portal.*` in `content/world/*.world_cell.json` is canonical (it is
  what Godot renders); the Rust fixture's `location.black_beach.*` and PR #2's
  `location.road.reception` / `location.estate.infirmary` are retired by A2.
- **The economy never closes.** Rations and medicine are spent, nothing
  produces them, `coin` is untouched, zero rations travel free.
  `drop_table_id`, `loot_seed` and `interaction_anchor_ids` are declared and
  never read; the encounter's `lootTableId` names a record that does not
  exist and the validator does not check it.
- **Michael cannot act.** Listed in `partyActorIds`, never built as an actor,
  `skillIds: []`, drawn as a card reading `ECHO DECK / STANDBY`. Only Betty is
  playable; Ayla is 5/7 in the engine on PR #2 with no character record.
- **No shippable build.** No CI; Windows-only PowerShell gates; one export
  preset (`Web Preview`, `extensions_support=false`) that cannot load the
  GDExtension.
- **Every asset is a placeholder behind a closed gate.**
  `generationGate: "visual-plate-not-approved"`; Betty's and the razorbeak's
  plates are staged in `still_image_plan.json`, not yet generated; the one
  GLB has zero skins and clips; twelve placeholders, all `releaseLegal: false`.
- **The isometric board contract exists — in dr-companion.**
  `tools/isometric-board-layout.mjs` (`boardLayoutFor(cell)` → 5 m
  footprint, seven rig-ready spawn points with `rigSocket`, compass tether
  anchors; `classifyTether`), `src/types/worldPresentation.ts`
  (`WorldNodeProjection`, `ActorSpawnPoint`, `RoomTether`, `WorldProjection`),
  and `docs/THREE_D_WORLD_STRATEGY.md` (three distances, presentation slots,
  event-driven miniature theatre). Its README says these may be shared with
  Pirate Island. **There is no mechanism yet for this repository to consume
  them** (the submodule runs the other way); one owner is an Open item.

## 3. What "shippable" means

A build a stranger can install, play to a designed outcome — victory, loss,
or a recorded mixed ending — and pay for, with every asset release-legal and
every rule the one the simulation enforces.

The brief defines the first unit itself (§21.7): **two ordinary factions,
Michael's early machinery, one recruitable woman, one contested route, one
building upgrade, one faction-specific dungeon, and a save/reload proof.**
That is M4. Everything before it is foundation; everything after it is the
brief scaled up.

| | Demo | Early Access | 1.0 |
|---|---|---|---|
| Factions live | 2 ordinary + Michael's + Cthulhu hidden | 4 ordinary + Michael's + Cthulhu emerging | all six, Cthulhu to summoning |
| Women | Betty, Ayla | + the two Open women, defined | all four, portfolios assigned |
| Graph | one region: estate core, ~12 rooms, one contested route, one dungeon | two regions, ports, a bridge front | the island |
| Michael's faction | first core, one machine family, one building tier | three machine families, wagons, tier 2 | walkers, rockets, airships, full doctrine |
| Endings | one victory, one loss reachable | victory + loss + one mixed | all families |
| Art | blockouts + first rigged actors | dressed rooms for its region | atlas |
| Length | 1–3 h, organic | 8–15 h | open-ended; genuine defeat possible |

## 4. Milestones and estimates

Numbered because the order is a dependency order. Each is done when its
proof passed — a test, a build, a human sign-off — never a screenshot.
Task IDs in brackets are the lane tasks (§6–§7).

### M0 — One spine, one map · in progress
[A1 A2 A11 B1 B2 C2 H1–H10] Commit the staged merge; make `world.cell.*`
canonical and prove the Rust fixture equal to the authored cells; party cap
to five; fold PR #2 in; expose the depth through the bridge; every doc
consistent with the brief. **Done when** one `expedition.rs` exists, all
three branches' tests pass together, every `game/tests/*.gd` suite resolves,
the validator rejects a dangling location or loot ID, and the dangle sweep
(§8) prints nothing. **Estimate:** the rest of this week.

### M1 — The character slice holds together
[A3 A4 A5 A6 A7 A8 C1 C3 C4 B3 B4 B5] Economy that closes; estate actions
behind one mechanism; Michael playable; five bands and Composure; Ayla
playable. **Done when** the slice test plays a full day — salvage, travel,
tomb, an encounter in which Michael acts, return, an estate action, midnight,
a route that did not exist yesterday — and the bridge exposes every step.
**Estimate:** two weeks after M0.

### M2 — A build a stranger could install · parallel with M1
[E1–E7 B8 B9] CI, portable gates, desktop presets, nightly artifacts, save
discipline, settings and accessibility scaffolding, pause. **Done when** a
stranger downloads the nightly for their OS, plays the slice, quits, reloads,
and gets the same legal actions. **Estimate:** two weeks, alongside M1.

### M3 — The strategic layer exists
[S1–S13 B10 B11 C9 C10 C11] Factions as data; ownership and influence on the
graph; buildings with envelopes and tiers; a deterministic strategic tick
that pause stops; utility AI with strategic states; directives that explain
themselves; offscreen forces that materialise through real routes and
sockets; weather, corruption, hidden pressure and Day 100; dungeon context;
elimination with a recovery chain; an event journal in the save. **Done
when** a headless test runs 100 in-world days of two factions on the current
graph, reproduces byte-identically from the same seed, survives save/reload
at day 50, and one faction is eliminated with its recovery chain
demonstrably exhausted. **Estimate:** four to six weeks after M0, paced by
human approval of the two Provisional doctrines used (the brief's §6.1–6.4
are Provisional and must be approved before they are treated as accepted).

### M4 — The brief's first vertical slice (Demo)
[S14 S15 C5 C6 C12 B12 B13 D1–D4 D7 D8 F1–F3 G1 G2] Two ordinary factions,
Michael's first core and one machine family, one recruitable woman with a
real arc, one contested route whose control changes during play, one building
upgrade the player directs, one faction-specific dungeon generated from
context, save/reload proof, one reachable victory and one reachable loss.
Blockout art plus the first rigged actors; the isometric board for the world;
the encounter lens per the §4 decision. **Done when** external testers reach
an ending, the release profile passes, and the store page is live.
**Estimate:** eight to twelve weeks after M3, paced by two external playtest
rounds and the animation tool's arrival.

### M5 — Early Access · M6 — 1.0
Cards written when M4 is done, from what M4 taught. Shape: four ordinary
factions and Cthulhu emerging; the two Open women defined and playable;
two regions and a port front; Michael's second and third machine families;
the mixed-ending family. Then the island, all six factions, the summoning
endgame, walkers, rockets, airships, the atlas.

### Rolled up

| Milestone | Calendar from now | Confidence | Paced by |
|---|---|---|---|
| M0 One spine, one map | 1 week | high | merge conflicts, measured |
| M1 Character slice | 2–3 weeks | high | engineering |
| M2 Installable build | 2–3 weeks (parallel) | medium-high | CI runner, macOS signing |
| M3 Strategic layer | 6–9 weeks | medium | doctrine approval, determinism proofs |
| M4 Demo | 4–6 months | medium-low | playtests, animation tool |
| M5 Early Access | 9–12 months | low | two more factions, two Open women |
| M6 1.0 | 18+ months | low | everything, reshaped by EA |

### Decisions still open (human)

- **O1 — Battle presentation.** The brief accepts a fixed isometric world and
  says nothing about how an encounter is shown. The bible LOCKS the side-view
  card→actor theatre. **Provisional recommendation:** keep the theatre as the
  *encounter lens* opened from the board — it is character-scale, it is
  built, and it does not contradict the brief — and treat the dr-companion
  "miniature theatre" as the board's own event playback for fights the party
  only *witnesses*. Needed before B12.
- **O2 — The other two women.** Identities, origins, how they join, their
  portfolios (brief §20 Open). Needed before C3's sibling records and M5.
- **O3 — Board contract ownership.** The footprint/tether/socket data shapes
  live in dr-companion. Provisional: this repository authors its room cells
  with the same field names now (B11), and the two repositories agree one
  owner before M5.
- **O4 — Merge authority.** Who merges to `main`, and whether agents may.
- **O5 — Michael's origin.** Accepted: shipwreck survivor with Argentine and
  Texas history. Open: the year, the divergence, the history's substance.
  Affects C4's notes and the founding sequence (S14).

---

# Part II — The work

## 5. How to read a task card

Every task has an ID (`<Lane><n>`), a row in its lane's ledger, and a card.
The ledger is the thing you edit to claim or finish; the card is the thing you
read to do the work.

```
### A3 · Title
Status: open | claimed <who> <date> | done <short-sha> | blocked: <why>
Depends on: A1 — nothing else
Touches: exact file paths
Steps: numbered, concrete, in order
Traps: the Rule 0 mistakes this task invites
Done when: a command that ran and what it printed
```

**Start here** if you are a new agent:
1. Read `AGENTS.md` §0. Read `docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md`
   §"Instructions for Claude" and §20. Read `.agents/README.md` and every
   `.agents/claims/*.json` with `status: "active"`.
2. Pick the first `open` task in your lane whose `Depends on` are all `done`.
   Lanes are independent; two agents in one lane must not take adjacent tasks.
3. Claim it (§8). Do the steps. Run the `Done when` command. Mark it done (§8).
4. If a step is impossible as written, do not route around it. Mark the task
   `blocked: <one sentence>` in the ledger and stop; the next reader decides.
5. If a step would require inventing something the brief lists as Open — a
   faction name, a woman's identity, a resource list — stop and mark it
   `blocked: needs decision O<n>`. Never fill an Open with an invention.

Lanes: **A** character simulation (Rust) · **S** strategic simulation (Rust)
· **B** bridge, board and Godot · **C** content and validator · **D** art ·
**E** release engineering · **F** audio · **G** QA and playtest · **H** docs
and hygiene.

## 6. Lane ledgers

Edit these tables to claim and finish. Nothing else changes for bookkeeping.

**Status notation** — one token, then optional detail. Keep it to one line.

| Token | Means | Written as |
|---|---|---|
| `open` | nobody holds it; its dependencies may or may not be met | `open` |
| `claimed` | an agent holds it right now | `claimed <agent> <YYYY-MM-DD>` |
| `blocked` | cannot proceed; the condition is named | `blocked: <condition>` |
| `done` | code landed on the lane branch, proof ran | `done <short-sha>` |
| **`shipped`** | merged into `backend/b0-expedition-state`, proof re-ran green there | `shipped <short-sha> <YYYY-MM-DD>` |

`done` is the author's claim; **`shipped` is the project's**. A milestone
counts only `shipped` rows. A row goes `done` → `shipped` when its lane
branch is merged and the integration proof (§9) passes on the trunk — not
when the pull request opens.

**Milestone roll-up** — update these two lines whenever a row ships, so the
top of the document is never stale:

- **M0** shipped 7/13 · A1 A2 A11 B1 H6 H7 H9 · H2 and H10 superseded by
  `main`'s own rewrite · **remaining: C2, B2, H5, H8**
- **M1** shipped 7/12 — **C1, C2, C4** (loot, portal costs, Michael's
  commands), **A3** (the supply loop closes: anchors produce, rations bite,
  victory pays), **A4** (the estate's rooms are anchor actions; one way to act
  at a place), **A5** (five named bands, Composure, the Shaken gate) and
  **A6** (Michael is a playable actor: Weapon Attack, Guard, Reposition)
- **M2** shipped 6/9 — **E8** (both ledgers a CI gate), **E1** (CI live and green on GitHub), **E2** (Godot
  suites, first run executed and green), **E3** portable gates, **E4** desktop
  export presets, **E6** save-migration fixtures; E5 open with a hard
  build-before-export requirement; E7 and E8 open
- **M3** shipped 12/16 — **C10** (three building records, every Open number Open in the data), **S3** (buildings, with the Open numbers Open in the data), **S6** (directives, explained before confirmation), **S7** (forces walk the routes; materialisation waits on B11), **S5** (factions read the board and choose), **S8** (two clocks that never move each other), **S11** (the journal: bounded, saved, nothing lost), **S4** (the tick and the determinism harness), **C9** (six factions as content, unnamed by rule), **S1** (factions exist), **S2** (control and contested roads), **S12** (recruitment, never numbers) · **M4** not started (C6 and C8, its romance and pack seams, shipped ahead of it)
- Last updated 2026-09-06 against trunk `a579ff9`. If this line is older than
  the newest `shipped` row below, the row is right and this line is stale.

### Lane A — Character simulation (`godot-rust/src/`)

| ID | Task | Depends on | Status |
|---|---|---|---|
| A1 | Port the vertical-slice branch's concepts into `expedition.rs` | — | shipped 6b9d275 2026-09-04 |
| A2 | `world.cell.*` canonical; fixture ≡ authored cells | A1 | shipped c1804ea 2026-09-05 |
| A3 | Economy: anchors, salvage, loot, scarcity | A2 | shipped 9360f66 2026-09-05 — loot ownership reconciled with C1 on merge |
| A4 | Estate actions as anchor actions; delete `rest_at_estate` | A3 | shipped 047e0f4 2026-09-05 — pursuit made gate-aware in cae9337 on merge |
| A5 | Five named bands and Composure | A1 | shipped 906411c 2026-09-05 — skill_rank reconciled to authored bondRank on merge |
| A6 | Captain Michael as a battle actor: Weapon Attack, Guard, Reposition | A5, C4 | shipped f33f364 2026-09-05 — name and reposition record reconciled on merge |
| A7 | Ayla's Deny Activation and Override Tomb Rule via site rules | A2, H5 | open |
| A8 | Reconcile `hold_position` with the Guard decision | A6 | shipped 4572361 2026-09-06 |
| A9 | Faction-agent observation record (was "Champion") | A6, S5 | open |
| A10 | Bond rank D→C with a specified combat effect | A6, C6 | open |
| A11 | Party of five | — | shipped 6b9d275 2026-09-04 |
| A12 | Test fixtures that name unconfirmed women | O2 | blocked: needs decision O2 |

### Lane S — Strategic simulation (`godot-rust/src/strategy/`, new)

| ID | Task | Depends on | Status |
|---|---|---|---|
| S1 | `FactionDefinition` and faction resources | — | shipped 3614c34 2026-09-06 |
| S2 | Ownership and influence on `Geography` nodes | A2 | shipped 27d777b 2026-09-06 — contested road = authored base + 2 |
| S3 | Buildings: envelopes, sockets, tiers, capture and ruin | S1, S2 | shipped a1018dd 2026-09-06 — the three Open numbers are Open in the data too |
| S4 | Strategic tick, pause semantics, determinism | S1 | shipped fefed40 2026-09-06 — the draw sequence is under the save hash |
| S5 | Utility AI and strategic states | S4 | shipped 9b4765e 2026-09-06 — neutral weights until the bridge loads records |
| S6 | `StrategicDirective` vocabulary and plain-language explanation | S5 | shipped 7a63396 2026-09-06 |
| S7 | Offscreen forces and materialisation through routes and sockets | S4, S2 | shipped 092533f 2026-09-06 — movement and arrival; **materialisation blocked: needs B11** |
| S8 | Weather, corruption, and the dual clocks (world time vs patience/heat) | S4 | shipped ba1cb56 2026-09-06 — every tuning number marked needs decision |
| S9 | `DungeonContext` and generation signature | S3, S8 | open |
| S10 | Elimination and the recovery chain | S3, S7 | open |
| S11 | Event journal and the strategic save fields | S4 | shipped 8c79fde 2026-09-06 |
| S12 | `RecruitmentState` for women (not a numeric romance UI) | S1 | shipped 83658e5 2026-09-06 |
| S13 | Michael's faction production: machines, not people | S3 | open |
| S14 | Founding sequence: shipwreck to first strategic core | S13, O5 | open |
| S15 | Two Provisional doctrines approved and encoded | S1, human | blocked: needs approval of brief §6.x doctrines |

### Lane B — Bridge, board and Godot

| ID | Task | Depends on | Status |
|---|---|---|---|
| B1 | Rewire the campaign bridge from `RouteGraph` to `Geography` | A1 | shipped 6b9d275 2026-09-04 |
| B2 | `native_expedition_port.gd` sends cells and portal costs | B1, C2 | shipped 3c85451 2026-09-05 — landed inside C13, which found the port dropping every cost and gate |
| B3 | Expose midnight, anchors, inspect and full legal commands | B1, A3 | shipped 9090284 2026-09-06 — S2's control/risk surface is the next B item |
| B4 | Route board shows anchor and estate commands | B3 | shipped 9090284 2026-09-06 |
| B5 | Battle screen: Michael's card unfolds; bands and Composure drawn | A5, A6 | shipped ab81aee 2026-09-06 |
| B6 | World cells for the tomb interior | A2 | open |
| B7 | Battle-entry sockets bound to habitat holders | B3 | open |
| B8 | New game, save slots, continue | E6 | open |
| B9 | Settings, accessibility, and **pause** | — | open |
| B10 | `ContentPackRegistry` and the presentation override pack | C8 | open |
| B11 | Room cells carry board metadata (footprint, spawn sockets, tethers) | A2, O3 | open |
| B12 | The isometric board: world / route / room distances | B11, S2, O1 | open |
| B13 | Calm information surface: ambient / notable / urgent | S6, B12 | open |
| B14 | Control and risk surface: `set_control`, `controller_of`, `effective_risk`, `contested` on routes | S2, B3 | shipped 58fbdb3 2026-09-06 |
| B15 | The bridge loads the faction records and hands the registry down; S5's seam closed | C9, S5 | shipped 777a943 2026-09-06 — the seam is closed as an honest negative until a record carries a real weight |

### Lane C — Content and validator (`content/`, `tools/src/validate.mjs`)

| ID | Task | Depends on | Status |
|---|---|---|---|
| C1 | Loot records and the `lootTableId` reference check | — | shipped ab17bfa 2026-09-05 |
| C2 | Portal cost fields (the location IDs were fixed by A2) | — | shipped ab17bfa 2026-09-05 — its costs first reached the engine under C13; the port had dropped them |
| C3 | `ayla.json` and seven Ayla skill records | A7 | open |
| C4 | Captain Michael's skill records and a `self` target rule | — | shipped ab17bfa 2026-09-05 |
| C5 | The tomb as a faction-specific dungeon: twelve spaces | B6, S9 | open |
| C6 | Relationship scene records and schema | — | shipped 3857ffa 2026-09-06 — five scenes, fade-to-black by rule, the pack's seam left open |
| C7 | Placeholder deprecation migration | D4 | open |
| C8 | Scene `presentationLevel` and the pack manifest schema | C6 | shipped a579ff9 2026-09-06 |
| C9 | Faction records (no proper names) | S1 | shipped 3598628 2026-09-06 — the validator refuses a real name until one is approved |
| C10 | Building records with envelopes | S3 | shipped a579ff9 2026-09-06 |
| C11 | Room contract fields on world cells | B11 | open |
| C12 | One recruitable woman's arc (records only; identity per O2) | C6, S12, O2 | blocked: needs decision O2 |
| C13 | Content can declare a discovery ID; the tidal cut's gate authored | A4 | shipped 3c85451 2026-09-05 — plus the bridge wire it turned out to need |

### Lane D — Art (`content/art/`, `work/art/`, `game/assets/`)

| ID | Task | Depends on | Status |
|---|---|---|---|
| D1 | Generate Betty's identity plate from `still_image_plan.json` | — | open (human runs the generator) |
| D2 | Generate the razorbeak's identity plate | — | open (human) |
| D3 | Review both plates; flip `generationGate` | D1, D2 | open (human sign-off) |
| D4 | Betty rigged GLB via the rig switch; Godot admission check | D3 | open |
| D5 | Betty's five Guarded Strike clips | D4, animation tool | blocked: waiting on the animation tool |
| D6 | Razorbeak rigged GLB (open) and clips (blocked) | D3, animation tool | open / blocked |
| D7 | Reception Terrace dressed; first admitted shared assets | — | open |
| D8 | Record the measured per-actor asset bill | D4, D6 | open |
| D9 | Building blockout kit for Michael's faction (standard envelopes) | C10 | open |
| D10 | First machine family blockouts (one animal-form automaton, one steam wagon) | S13 | open |
| D11 | Card rail and command grid in the bronze-and-vellum grammar | — | open |
| D12 | Room blockouts for the Demo region | C11 | open |

### Lane E — Release engineering

| ID | Task | Depends on | Status |
|---|---|---|---|
| E1 | GitHub Actions: Rust and content checks | — | shipped e0b4051 2026-09-05 |
| E2 | Godot headless suites in CI | E1, E3 | shipped 7b8f698 2026-09-05 — executed and green; **hardened 5107113 2026-09-06**: the gate now fails any step that prints an ERROR line, after a suite's exit-code hole let a missing scene anchor through a green run |
| E3 | Shell equivalents of the two PowerShell gates | — | shipped 5344b7b 2026-09-05 |
| E4 | Desktop export presets for Windows, Linux, macOS | — | shipped 838824e 2026-09-05 |
| E5 | Nightly build artifacts per platform | E2, E4 | open — **must build the native library before exporting**, see its card |
| E6 | Save-version migration fixtures | — | shipped 85413f0 2026-09-05 — **blind spot closed 095f3ac 2026-09-06**: the expected key set is a literal now |
| E7 | Crash log with state snapshot; no silent telemetry | — | open |
| E8 | Claims enforcement in CI | E1 | shipped 528773b 2026-09-06 — both ledgers are a gate; it corrected 120-odd records on arrival |
| E9 | Pack build script and pack artifact | C8, E5 | open |

### Lane F — Audio · Lane G — QA

Cards at M4 from bible §14's backlog and the brief's calm-interface rules.
G1 is the acceptance script; G2/G3 two external rounds.

### Lane H — Docs and hygiene

| ID | Task | Depends on | Status |
|---|---|---|---|
| H1 | `docs/STATUS.md` adopted and current | — | done (this pass) |
| H2 | `GAME_BUILD_PLAN.md` §7 rewritten | — | superseded — `main` replaced the whole document (e77c9e6); its version is canonical and this plan defers to it |
| H3 | Bible wording: 3D; romance and the pack | — | done (this pass) |
| H4 | 3D production docs: rigging is a switch; clips wait for the tool | — | done (this pass) |
| H5 | Site-rule spec into `HEROINE_AYLA_DESIGN.md` | — | open |
| H6 | Retarget PR #3 to `main`; note PR #4; flag the stale status branch | — | shipped 34142a8 2026-09-05 — PR #3 now targets `main`, conflict resolved by merging it |
| H7 | dr-companion `.gitmodules` branch pointer | — | shipped (dr-companion e387d29) 2026-09-05 — the work landed in dr-companion, not here: merged to its `main` in its PR #276; no commit of that SHA exists in this repository |
| H8 | Regenerate the design bible `.docx` | H3, H9 | open (needs `python-docx`) |
| H9 | Bible: superseded entries marked; brief named as authority | — | done (this pass) |
| H10 | `GAME_BUILD_PLAN.md` C2 roster and E2 exit test reconciled to the brief | — | superseded — same rewrite; `main` states the roster contract directly |

## 7. Task cards

Cards for A1–A10, B1–B9, C1–C7, D1–D8, E1–E8 are unchanged from the first
edition except where the brief touched them; those edits are marked **(brief)**.
New cards follow.

### Lane A

### A1 · Port the vertical-slice branch's concepts into `expedition.rs`
Status: shipped 6b9d275 2026-09-04.
What was ported: `resolved_encounter_ids`; `EncounterState.estate_upgrade_id`;
`TravelBlockedByEncounter` (my `travel()` let the party walk away from a
pending fight); `DuplicatePortal` / `DuplicateEncounterTrigger`;
`legal_route_commands()`; an authored-trigger branch at the top of
`begin_encounter()`'s ladder (trigger → hunter → habitat holder).
`Geography` absorbed `RouteGraph`: `from_authored(cells, portals, triggers)`,
`CellDefinition`, `PortalDefinition`, `EncounterTriggerDefinition`,
`RouteKind::from_travel_mode`, `LocationRecord::authored`.
Done when: `cargo test` prints `110 passed` and `1 passed`; the merge commit exists.

### A2 · `world.cell.*` becomes canonical; fixture ≡ authored cells
Status: shipped c1804ea 2026-09-05 · Depends on: A1
Touches: `godot-rust/src/geography.rs`, `expedition.rs`, `habitat.rs`,
`hunter.rs`, `godot-rust/tests/vertical_slice.rs`, `content/world/*.world_cell.json`,
`content/encounters/returning_names.prototype.json`, `tools/src/validate.mjs`,
`docs/CLAUDE_BACKEND_HANDOFF.md` (its B1 graph lists the old IDs at lines 98
and 163–166 — update them in the same commit).
Steps:
1. Read the five files in `content/world/`. Note each `id`, `regionId`,
   `displayName`, `readableDescriptions[].id`, `interactionAnchors[].id`, and
   each portal's `id`, `targetCellId`, `travelMode`.
2. Rewrite `Geography::black_beach_vertical_slice()` to call
   `from_authored(...)` with `CellDefinition`s and `PortalDefinition`s whose
   IDs are exactly the authored ones. Carry the old costs onto the matching
   portals (safe road 60 min / 2 rations / risk 1; jungle edge 35 / 4 / 3;
   others as they were). `observation_ids` = `readableDescriptions[].id`;
   `interaction_anchor_ids` = `interactionAnchors[].id`;
   `encounter_eligible` = `battleEntries` non-empty.
3. Add the four tomb rooms as `world.cell.tomb_threshold`,
   `world.cell.tomb_reception`, `world.cell.tomb_archive_core`,
   `world.cell.tomb_service_passage` with portals `world.portal.<from>_to_<to>`.
   Keep the gate: the archive-core portal has
   `required_discovery_id: Some("observation.tomb_reception.true_name")`.
4. Replace every `location.black_beach.*` / `location.tomb.*` string in
   `godot-rust/` (`grep -rn "location\.\(black_beach\|tomb\)" godot-rust/`)
   with the canonical ID; `route.*` → `world.portal.*`;
   `ESTATE_LOCATION_ID` → `world.cell.damaged_estate`; habitat territories
   and every test follow.
5. Add the test `fixture_matches_the_authored_world_cells` in `geography.rs`:
   read `content/world/*.world_cell.json` via `env!("CARGO_MANIFEST_DIR")`
   `+ "/../content/world/"`, parse with `serde_json::Value`, assert the cell
   ID set equals the fixture's (skip `world.cell.tomb_*` until B6, with a
   comment naming B6), and every authored portal exists with the same from,
   to and `RouteKind::from_travel_mode(travelMode)`.
6. Encounter record: `locationId` → `world.cell.reception_terrace`;
   `defeat.returnLocationId` → `world.cell.damaged_estate`.
7. `validate.mjs` encounter block (~L174): `reference()` both fields. Prove
   the check bites with a deliberate typo, then revert.
Traps: no aliases; no old→new lookup table; the fixture is a test twin only.
Done when: `cargo test` green; `node tools/src/validate.mjs` green;
`grep -rn "location\.\(black_beach\|tomb\|road\|estate\)" godot-rust/ content/ docs/CLAUDE_BACKEND_HANDOFF.md` prints nothing.

### A3 · Economy: anchors, salvage, loot, scarcity
Status: shipped `9360f66` 2026-09-05 · Depends on: A2
Touches: `geography.rs`, `expedition.rs`, `habitat.rs`, `world.rs`, `lib.rs`
One mechanism for every "do something here" verb — salvage, loot, and (A4)
the estate rooms; **(brief)** later, building interactions (S3) reuse it.
- `geography.rs`: `pub enum AnchorKind { Salvage { rations: u32, coin: u32 }, LootCache { rations: u32, medicine: u32, coin: u32 }, Infirmary, Workshop, MapTable, Inspect }`,
  `pub struct AnchorDefinition { pub id: String, pub kind: AnchorKind, pub once_per_day: bool }`.
  `Geography.anchors: BTreeMap<String, AnchorDefinition>`; `CellDefinition`
  gains `#[serde(default)] anchors: Vec<AnchorDefinition>`; `from_authored`
  inserts them and lists their IDs in the cell's `interaction_anchor_ids`.
- `expedition.rs`: `#[serde(default)] pub anchor_uses: BTreeMap<String, u32>`
  (anchor → last day used). `pub fn use_anchor(&mut self, anchor_id, geography) -> Result<AnchorOutcome, ExpeditionError>`:
  reject before mutation if not at this location (`AnchorNotHere`), spent
  today (`AnchorSpentToday`), or an encounter is pending. Salvage yield is
  deterministic: `rations + (mix_seed(self.rng_seed, self.campaign_day as u64, anchor_id) % 2)`
  via `world.rs`'s existing `mix_seed`. `legal_next_commands_with_geography`
  emits `anchor_action:<id>` for every legal anchor here.
- Loot: `Habitats.loot_tables: BTreeMap<String, LootTable>` with
  `LootTable { id, rations, medicine, coin }` for the three `drop_table_id`s.
  In `resolve_encounter` on `Victory` against a habitat holder, add the
  table's yield plus `loot_seed % 3` coin. Return
  `EncounterResolution { outcome, loot: Option<LootTable> }` (change the return
  type; fix the callers — tests and the bridge).
- Scarcity: in `travel()`, after the discovery gate and before any mutation,
  reject with the existing `InsufficientSupplies { needed, available }` when
  `rations < cost`. The retreat leg keeps `saturating_sub`.
Tests: same salvage numbers for the same seed and day; second salvage today
rejected without mutation; travel with too few rations rejected, then
succeeds after salvaging; terrace victory adds loot; defeat adds none; the
slice test must now salvage before it travels.
Traps: no second estate-action method beside `use_anchor`; no `rand` crate.
Done when: `cargo test` green with the six new tests.

**Shipped.** Eleven tests rather than six. Salvage determinism is pinned to
real numbers derived from `mix_seed` independently before being asserted —
seed 42 yields `[5, 5, 4, 4, 4, 5, 5, 4]` rations on days 1–8 — and proven to
bite: replacing the mixed bonus with a constant flattens it to all fives and
fails. The slice test now starts the party at zero of everything, is refused
the safe road, walks back to the wreck, salvages, and only then goes inland;
the medicine the estate infirmary spends is medicine the terrace holder was
carrying. That is the loop closing rather than a test of it.

One out-of-lane edit, reviewed and accepted: six lines in `godot_bridge.rs`,
three arms in `expedition_error_code`, whose match over `ExpeditionError` is
exhaustive — the new variants cannot compile without them. Projecting anchor
commands to Godot remains B3's.

`Workshop` and `MapTable` ship with empty arms carrying A4's exact acceptance
criteria in-code, and nothing in the world declares an anchor of either kind,
so no reachable action depends on them. `Infirmary` calls `rest_at_estate`
rather than copying it — A4 inlines and deletes.

**Loot ownership, decided on merge.** A3 and C1 ran in the same round and each
wrote its own numbers for the same three drop tables; they disagreed on all
three. `content/loot/*.json` is now the owner, `Habitats` carries the same
values, and `fixture_matches_the_authored_loot_tables` fails if they drift —
the arrangement A2 established for geography, reused rather than reinvented.
That test also asserts every `drop_table_id` a habitat declares resolves,
which is C1's dangle check from the Rust side.

### A4 · Estate actions as anchor actions; delete `rest_at_estate`
Status: shipped `047e0f4` 2026-09-05 · Depends on: A3
Steps: the estate cell declares `anchor.estate.infirmary` → `Infirmary`,
`anchor.estate.workshop` → `Workshop`, `anchor.estate.map_table` → `MapTable`,
`anchor.estate.household_room` → `Inspect`. **Infirmary** = today's
`rest_at_estate` exactly. **Workshop** requires
`observation.black_beach.wreck_of_handsome_jack`; records
`estate.upgrade.workshop_field_rig`. **MapTable** requires
`observation.river_landing.elven_waymark`; inserts
`discovery.map_table.tidal_cut`. Add `effective_supply_cost(&self, route) -> u32`
(−1 with the field rig) as the single owner of route cost, used by `travel()`
and the retreat leg. Add the gated portal
`world.portal.black_beach_to_reception_terrace_tidal_cut` (40 min, 1 ration,
risk 2) to fixture and authored cell. Delete `rest_at_estate`,
`can_rest_at_estate`, `EstateRestOutcome`, unused `ESTATE_*` constants, the
`"rest_at_estate"` command string; move its five tests onto
`use_anchor("anchor.estate.infirmary", …)`. Extend the slice test through
the map table, midnight, and the tidal cut, with `assert_boundary_round_trips`
after the estate action. **(brief)** The estate is Michael's *first strategic
core*; S14 grows it into a faction. Nothing here is thrown away — the anchors
become the core's first building interactions.
Done when: `cargo test` green; `grep -rn rest_at_estate godot-rust/` prints nothing.

**Shipped.** There is now exactly one way to do something at a place:
`use_anchor`. `rest_at_estate` and everything that only existed to serve it
are deleted, the infirmary body inlined into its arm, and `anchor_refusal` is
the single owner of anchor legality for both the command and the legal-command
listing, so what is offered and what is accepted cannot drift.
`effective_supply_cost` is the single owner of route cost, called by `travel`
and the retreat leg; the field rig takes one ration off every road. The slice
test takes all three estate actions with round-trip assertions after each,
crosses midnight, and walks the tidal cut for zero rations.

The card named two gate observation IDs that exist nowhere. The lane used the
authored ones from `content/world/` — canonical since A2 — rather than mint
the card's spellings and recreate the two-namespaces drift A2 exists to kill.
The card was wrong, not the lane. `fixture_matches_the_authored_world_cells`
was extended to hold every portal's three costs, its gate, and each cell's
observation IDs equal, not merely the ID sets; proven to bite on one ration.

**Two findings the lane reported rather than reaching for, both real:**

- **Pursuit ignored gates.** `next_step_toward` and `step_distance` counted
  moves over the whole graph, so a hunter would chase through a door the party
  had no way to use. Already true of the tomb's true-name gate, and invisible;
  the tidal cut made every chase from the terrace arrive on the sand in one
  move through a passage the party had never found. The lane rewrote the
  distance expectations to the shortcut numbers and flagged the question. The
  integrator answered it in `cae9337`: a gate is closed for everyone until the
  party opens it and open for everyone afterward; pursuit consults the party's
  discoveries. The original distances return; a new test holds the other half
  (an opened gate is open for pursuit too). The other reading — that a natural
  passage is open to creatures who know the island — is defensible fiction and
  is not what shipped; if wanted, it is a per-portal property content authors,
  not a global change to pursuit.
- **Content cannot declare a discovery ID.** `validate.mjs` resolves a portal's
  `requiredDiscoveryId` against registered stable IDs and no record type
  declares one, so the tidal cut's gate lives only in the Rust fixture, under
  a named `GATES_CONTENT_CANNOT_YET_DECLARE` exception with its deletion
  condition. That is a content-schema gap, and it is now **C13**.

### A5 · Five named bands and Composure
Status: shipped `906411c` 2026-09-05 · Depends on: A1
`pub enum Band { PartyRear = 0, PartyFront = 1, Contested = 2, EnemyFront = 3, EnemyRear = 4 }`
with `from_index(i8) -> Option<Band>` and `name()`; keep `Actor.band: i8` as
storage. Fix the fixture (Betty PartyFront=1, Razorbeak EnemyFront=3).
`Actor.composure: u8` (fixture 10), `StatusKind::Shaken`,
`spend_composure()` applying Shaken at 0. `skill_rank(skill_id)` from the
seven `betty.*.json` (D…SSS) and Ayla's five; reject SS/SSS with
`BattleError::ShakenCannotUse` when Shaken. Bridge `actor_dictionary` adds
`band_name`, `composure`. **(brief)** Bands and Composure are the character
combat model and are unaffected by O1; keep them.
Done when: `cargo test` and `cargo check --features godot-ext` green.

**Shipped.** Naming the bands paid for itself immediately: two fixture values
were not merely unnamed but wrong. The prototype had the Razorbeak in
`Contested` rather than `EnemyFront`, and a party ally parked there too. A
magic number cannot be wrong on its face; a named band can, which is the
argument for the enum.

The card said "Ayla's five" — there are no `ayla.*` skill records in
`content/skills/` to read a rank from, so `skill_rank` covers the authored
skills and returns `None` elsewhere rather than inventing ranks. Correct call;
the card was wrong, not the work.

The careful part was the cleanse. `condition_cleanse` sorted statuses and
drained the top two unconditionally, so putting `Shaken` into that ordering
would have made a rank C skill hand Composure back — and contradicted the
authored `removalPriority`, which `validate.mjs` pins at exactly four entries.
`status_priority` became `cleanse_priority -> Option<u8>`: `Shaken` is `None`,
sorts last, and the drain stops before reaching it. The four authored
priorities are byte-identical.

**Rank ownership, decided on merge**, the same way loot was: `content/skills/`
owns `bondRank`, `skill_rank` mirrors it, and
`every_authored_skill_has_its_authored_rank` holds them equal. The table
arrived already missing Michael's two commands from C4 — both rank D, so
nothing failed, which is exactly how an SS skill would have slipped past the
Shaken gate unnoticed.

Two follow-ups A5 reported rather than reached for, both correct:
- `game/scripts/simulation/mock_simulation_port.gd` built actor dictionaries
  without `band_name` or `composure`, so the mock and the native bridge
  disagreed on the dict shape. **Closed by the integrator** rather than left
  for B5: the mock now emits both, its band names are the same five strings
  `Band::name()` returns, and its Razorbeak moved from `Contested` to
  `EnemyFront` — the identical category error A5 corrected in Rust, sitting
  in GDScript. Verified by CI's Godot job, not locally; there is no Godot in
  the integrator's environment and the file is not exercised by any test,
  since the mock is the fallback the anti-mock gate exists to forbid.
- No Composure cost is attached to any skill, and there is no Echo or Field
  Order. That is A6 and A9, not a gap in A5.

### A6 · Captain Michael as a battle actor
Status: shipped `f33f364` 2026-09-05 · Depends on: A5, C4
Build `character.protagonist.captain` — display name read from
`content/characters/captain.json`, which owns it (**Michael Corrigan**). The
brief's "use Captain Michael" supersedes Captain Jack; it does not remove the
surname, and the record's own notes reconcile the two — full canonical name
Captain Michael Corrigan, ordinary usage Michael, formal address Captain
Corrigan. Not player-named. — `Faction::Party`, level 3, vitality 90, guard 0,
initiative 10, band PartyRear. Allowlist `skill.captain.weapon_attack`
(default damage arm, one hostile target) and `skill.captain.reposition`
(zero targets; extract the band move from `resolve_rescue_charge` into
`move_actor_to_band(...)` and use it from both; PartyRear ↔ PartyFront only,
else `RepositionNotLegal`). Owner prefix `skill.captain.` mirrors
`skill.betty.`. Guard = `skill.system.hold_position` (no new code; A8 fixes
the doc). Echo and Field Order are not built here.
Done when: the slice test shows Michael acting; Betty submitting his skill
fails the owner check; both band moves share one helper.

**Shipped.** Nine tests, two of which read `content/skills/*.json` at test time
rather than restating its numbers — the lane applied that rule on its own,
having been told it once. Both were proven to bite: quietly falling back to the
generic damage arm fails the damage test, and permitting a move out of
`Contested` fails the band test with the illegal `ActorMoved` event printed.

The band move was extracted out of `resolve_rescue_charge` into
`move_actor_to_band` and shared, never copied. It takes `i8` rather than `Band`
because Rescue Charge copies whatever band its ally stands in, including a
value outside the five — which is exactly why `Band::from_index` returns
`Option`.

`prototype_vertical_slice()` was deliberately left alone, because
`native_simulation_port_test.gd` asserts it holds four actors and `game/` was
not this lane's to edit. Michael enters through the slice test instead, where
the razorbeak now survives the first round so the fight lasts long enough for
him to reposition and then land the killing attack.

**Two content contradictions the lane reported rather than papered over, both
resolved by the integrator:**

- `captain.reposition.json` authored `requiresUnoccupiedBand: true`, which
  nothing implemented and nothing validated. The five-band model cannot support
  it: A5's own fixture stands Betty and Vix both in `PartyFront`, so bands are
  zones rather than tiles, and honouring the flag would make Reposition's only
  legal destination unreachable whenever an ally happened to be standing in it.
  The field is removed. A flag that no code reads and no test checks is a
  promise the content makes and the game silently breaks.
- The actor's display name was written into Rust as "Captain Michael" while
  `content/characters/captain.json` authors "Michael Corrigan". Content owns it;
  the Rust now reads from the record and
  `the_captain_carries_his_authored_display_name` holds them equal. The A6 card
  above carried the wrong instruction and has been corrected — the card was
  wrong, not the lane.

### A7 · Ayla's Deny Activation and Override Tomb Rule via site rules
Status: open · Depends on: A2, H5
`CellDefinition`/`LocationRecord` gain `site_rule_ids`; the tomb cells declare
`site_rule.tomb.grave_watch` (hostiles regain 2 guard each round).
`ExpeditionState.suppressed_site_rules: BTreeSet<String>` `#[serde(default)]`.
`Battle::new` takes `site_rule_ids`; the bridge passes the cell's rules minus
suppressed. Override Tomb Rule (SSS) emits `SiteRuleOverridden { rule_id }`
and the bridge writes it to `suppressed_site_rules`. Deny Activation (SS)
applies `Stunned` for one turn via `apply_status` (arrives with PR #2).
**(brief)** Site rules are the first hook S9's `DungeonContext` will set per
owning faction; author them as data from the start.
Done when: two tests per skill; a suppressed rule survives save/reload.

### A8 · Reconcile `hold_position` with the Guard decision
Status: shipped `4572361` 2026-09-06 · Depends on: A6 — rewrite
`ARCHITECTURE.md`'s "admitted scaffolding" paragraph: `hold_position` is the
universal Guard verb.

**Shipped.** The paragraph said the verb "must disappear"; A6 had already
built on it staying (the Captain has no Guard of his own because this one
exists, and `the_captain_may_guard_with_the_universal_hold_position_verb`
proves he can spend it). The rewrite separates the two things the old text
conflated: the verb is permanent; the *driver's* use of it as a stand-in for
a heroine's unimplemented signature skills is the provisional part.

### A9 · Faction-agent observation record
Status: open · Depends on: A6, S5 · **(brief)** Was "Champion observation".
The Cthulhu faction's local agent (whatever the Champion becomes — Open)
adapts only through declared records: `ObservationRecord { day, observed,
countermeasure, tell }` in `ExpeditionState.faction_observations:
BTreeMap<String /*faction*/, Vec<ObservationRecord>>`; written on resolving
an encounter against that faction's `enemy.<faction-key>.*` actor; surfaced
by `legal_next_commands_with_geography` as `<faction>_briefing` before a
rematch. Done when: a test's second encounter briefing names the first's observation.

### A10 · Bond rank D→C with a specified combat effect
Status: open · Depends on: A6, C6
Touches: `godot-rust/src/battle.rs` (the rank gate), `godot-rust/src/expedition.rs`
(one appended `#[serde(default)] pub bond_ranks: BTreeMap<String, String>` and
its initialiser), `godot-rust/src/strategy/recruitment.rs` (one hook: a
recorded milestone whose scene says so raises the rank), `godot-rust/tests/save_migration.rs`
(key), `content/relationships/` (one field on an existing Betty scene)
Every skill record carries a `bondRank` (D…SSS) and `skill_rank` mirrors it,
but nothing yet says which rank a *woman* stands at, so every authored skill
is usable from the first fight. This card makes rank a fact of the
relationship: `bond_ranks[woman] = "D"` at the start; a scene record may
carry `raisesBondRankTo: "C"`; recording that scene's milestone raises the
rank; `Battle::submit` refuses a skill whose `bondRank` is above the actor's
current bond rank with a new `BattleError::BondRankTooLow { skill_id,
required, current }` — before mutation, beside the Shaken gate. Betty's
`condition_cleanse` is rank C; her first household scene
(`scene.betty.the_wreck_dead_are_buried` or the one the lane judges right)
raises her to C. One test each way: refused at D, allowed at C. The bridge's
`actor_dictionary` gains `bond_rank` (a letter, nothing numeric) so the card
rail can draw it; the mock port agrees.
Traps: only Betty and Ayla; rank never advances on a timer; the rank ladder
is the authored letters, no numbers; `every_authored_skill_has_its_authored_rank`
stays; the exhaustive error match in the bridge gains its arm.
Done when: both tests; the harness and migration suites green; the validator
accepts `raisesBondRankTo` only as one of the seven letters.

### A11 · Party of five
Status: shipped 6b9d275 2026-09-04 · **(brief)** "The player controls five heroes."
`expedition.rs` `validate()` accepts `1..=5`; the oversized-party test uses
six. `GAME_BUILD_PLAN.md` §4.1's `party_ids: CharacterId[1..4]` reads `[1..5]`.
Done when: `cargo test` green with the changed assertion.

### A12 · Test fixtures that name unconfirmed women
Status: blocked: needs decision O2. `Battle::prototype_vertical_slice()`
builds an actor named Vix, and `NATIVE_BRIDGE.md` gate 8 describes it. Vix is
not a confirmed member of the four. The fixture is test-only and stays until
O2 is decided; when it is, rename or replace the actor and the doc line in
one commit. Do not author any Vix/Grisha/Isabella/Nara content record.

### Lane S — Strategic simulation (`godot-rust/src/strategy/`, new)

The strategic layer is a second Rust module tree beside the character layer,
sharing `Geography` (the graph), `ExpeditionState` (the save) and `world.rs`'s
`mix_seed` (the only RNG). Everything is deterministic, seeded, `BTreeMap`-backed,
reject-before-mutation, and stops when paused. Nothing in it renders. Every
type below mirrors brief §19's "Derived" shapes by field name so the brief and
the code stay one vocabulary.

### S1 · `FactionDefinition` and faction resources
Status: shipped `3614c34` 2026-09-06 · Depends on: —
Touches: `godot-rust/src/strategy/faction.rs` (new), `lib.rs`, `content/factions/*.json` (C9)
Steps:
1. `pub struct FactionDefinition` with exactly brief §19's fields: `id`,
   `concept_key` (one of `fox_people`, `colonial_powers`, `pirates`, `elves`,
   `cthulhu`, `michael` — keys, **not names**), `doctrine`,
   `resource_priorities`, `building_priorities`,
   `recruitment_or_population_rules`, `movement_preferences`,
   `relationship_tendencies`, `board_position_behavior`, `victory_conditions`,
   `recovery_rules`, `elimination_rules`, `weather_preferences`,
   `terrain_influence`, `corruption_interactions`, `building_kit`,
   `actor_kit`, `dungeon_grammar`, `loot_grammar`. Sub-types are small
   enums/structs; unknown detail stays `String` notes until approved.
2. `pub struct FactionState { pub resources: BTreeMap<String, u32>, pub strategic_state: StrategicState, pub eliminated: bool, pub relationships: BTreeMap<String, Relationship> }`
   where `StrategicState ∈ {Desperate, Recovering, Contesting, Advantaged, Closing}`
   (brief §9) and `Relationship` carries brief §7's fields (trust, fear,
   hatred, grievance, dependence, trade_value, territorial_conflict,
   ideological_incompatibility, recent_aid, recent_aggression, treaty_state,
   known_betrayal, perceived_strength, perceived_opportunity) as `i16`.
3. **Resource categories are Open (brief §20).** Use string keys from the
   faction record; do not define an enum. `Habitats`' three `drop_table_id`s
   and A3's rations/medicine/coin are the character-scale currency and stay
   separate.
4. `ExpeditionState.factions: BTreeMap<String, FactionState>` `#[serde(default)]`.
Traps: no faction proper names anywhere — IDs are `faction.<concept_key>`.
Do not encode a Provisional doctrine (§6.1–6.4) as behaviour before S15.
Done when: round-trip test for a two-faction state; `cargo test` green.

**Shipped.** Brief §19's fields verbatim and in order; `ConceptKey` is a
closed six-variant enum and no type has a field a proper name could live in;
`validate` holds an ID to exactly `faction.<concept_key>` through the crate's
single stable-ID rule. `FactionDefinitions`, the registry S4's signature
names, lives here because loading C9's records is S1's half of that contract.
Resource categories stay Open by construction. `StrategicState::default()` is
`Contesting` only so pre-strategic saves load; S5 recomputes on first tick —
flagged for review, not decided. **C9 must author** one
`content/factions/<concept_key>.json` per faction whose `id` is exactly
`faction.<concept_key>`, with a `displayName` placeholder marked needs
decision that no Rust field reads; the validator block mirrors `validate`.
The lane's most useful finding is on E6's card.

### S2 · Ownership and influence on `Geography` nodes
Status: shipped `27d777b` 2026-09-06 · Depends on: A2
`LocationRecord` gains `owner_faction_id: Option<String>` and
`influence: BTreeMap<String, u16>`; `Geography` gains
`fn controller(&self, cell_id) -> Option<&str>`. `ExpeditionState.ownership:
BTreeMap<String, String>` is the mutable truth (the graph is static content);
a `ControlChanged { cell_id, from, to, day }` event. "A safe road becomes
contested" (brief §1) is `RouteOption.risk_level` derived at query time from
the two endpoints' controllers, not stored twice.
Done when: a test flips control of the river landing and the safe road's risk
changes without any route record being edited.

**Shipped, with the integrator's correction applied.** The authored
`risk_level` is the base; `Geography::effective_risk` is the single owner of
a road's live danger — base plus `CONTESTED_RISK_MODIFIER = 2` when the two
endpoints' effective controllers differ, unheld counting as a party of its
own (one comparison, no special case; a fresh campaign is uncontested). Two
is not arbitrary: the slice's safe road is risk 1 and its jungle edge 3, so a
contested safe road costs exactly what the jungle costs — "a safe road
becomes contested", mechanical. A test holds that equality.
`ControlChanged` went on the one `WorldEvent` enum in `world.rs`.
`set_control(cell, None)` releases the override rather than forcing unheld,
so the event never reports a cell as unowned while the graph still gives it
an owner. `influence` has no consumer yet and `effective_risk` deliberately
reads control only, so influence cannot become a second answer to who holds
a road; S4/S7 own it. **Bridge surface for B3, next round:** `set_control`,
`controller_of` (effective, never raw `ownership`), `effective_risk(portal)`,
route projections carrying effective risk plus `contested: bool` and never
the authored base as a second number, and a `ControlChanged` case in the
event projection.

### S3 · Buildings: envelopes, sockets, tiers, capture and ruin
Status: shipped `a1018dd` 2026-09-06 · Depends on: S1, S2
`BuildingDefinition` with brief §19's fields verbatim (`footprint_cells`,
`clearance_cells`, `height_class`, `entrance_sockets`, `road_sockets`,
`actor_sockets`, `delivery_sockets`, `allowed_terrain`, `maximum_slope`,
`construction_cost`, `construction_requirements`, `tier_states`, `production`,
`services`, `recruitment_support`, `capture_rules`, `ruin_state`,
`dungeon_relationship`, `loot_relationship`). `BuildingInstance { def_id,
cell_id, faction_id, tier, hp, state ∈ {UnderConstruction, Operational,
Damaged, Ruined, Captured} }` in `ExpeditionState.buildings`. Placement
rejects overlap of `footprint_cells ∪ clearance_cells` (brief §8: nothing
essential outside the envelope). **Michael's buildings produce machines,
capacity and services, never people** — enforce with a test that a
`concept_key == "michael"` building whose `production` names a human role is
rejected at load.
Done when: overlap rejection test; production-rule test; tier upgrade test.

**Shipped.** §19's fields in order; instances under their own
`building_instance.*` namespace; every method in `building.rs`. Placement
rejects overlap before mutation (proven to bite on exactly one test);
Michael's buildings never produce people, and no building produces a person
on a timer. **Three §20 items stay Open in the data:** `TIER_CAP = 3`,
`CELL_CAPACITY_CELLS = 12`, `CAPTURE_HP_RESTORED = 1` are named constants
marked `needs decision`, and `CaptureRules::NeedsDecision` /
`RuinState::NeedsDecision` are the *defaults*, refused at load by a message
naming the section — so the decision lands on each authored record, not on
this lane for every building at once. Dimensions are abstract shares of a
cell's capacity; B11/O3 own geometry. Damage at zero leaves a capturable
shell; ruin is the deliberate path and rubble still occupies ground.
`actor_sockets_at(cell, &definitions)` takes the registry, never the save —
the seam for B11 and S7's arrival. **C10 must author** `content/buildings/`
to the contract in S3's claim notes (concept keys only; `capture_rules` and
`ruin_state` explicitly chosen; sockets in their §19 field; a human-role
output only as recruitment support), plus the fixture-versus-records test.

### S4 · Strategic tick, pause semantics, determinism
Status: shipped `fefed40` 2026-09-06 · Depends on: S1
`pub fn strategic_tick(&mut self, geography, factions: &FactionDefinitions) -> Vec<StrategicEvent>`
advances one in-world hour of faction activity; `resolve_midnight_in` calls
it 24 times before the character-scale Midnight Return so the two clocks
stay one clock. `Paused` is not a state in the simulation: the bridge simply
does not call `tick` — a test proves that N ticks with any interleaving of
"pause" produce identical state (brief §17: pausing never changes outcomes).
Determinism: every choice draws from `mix_seed(rng_seed, day, hour, faction_id, purpose)`.
Done when: 2,400 ticks from seed 7 hash identically twice; a save at tick
1,200 and reload reproduces the same hash at 2,400.

**Shipped.** 2,400 hours from seed 7 through real midnights hash identically
twice; a save at hour 1,200 reloads into the same hour 2,400; pause is a
round trip between arbitrary ticks and is byte-identical to no pause; the
strategic hour and the campaign day agree after every midnight. **Why it can
bite before S5:** no faction acts yet, so a harness over the clock alone
passes a wall-clock seed and a `HashMap` order. `StrategicClock::draw_digest`
folds every draw ever made into the save hash; both sabotages then fail four
of seven tests. **The draw sequence S5 inherits**, per faction in `BTreeMap`
order (eliminated factions included, so removing one never shifts another),
per hour, from `tick::PURPOSES`: `board_position` (S5), `goal` (S5),
`directive` (S6), `economy` (S3/S13), `force` (S7), `relationship` (S6),
`recovery` (S10); `hour_draws(...)` is the accessor. The draw is `mix_seed`
composed twice, not a new mixer. **No journal field** — S11 owns it. **The
registry is unread and `resolve_midnight_in` passes an empty one**, because
the bridge has none; `the_registry_cannot_change_a_tick_yet` is written to be
deleted, and its failure the day S5 scores a record is the instruction to a
B card to load `content/factions/` and hand it down.

### S5 · Utility AI and strategic states
Status: shipped `9b4765e` 2026-09-06 · Depends on: S4
`fn choose_goals(faction, board) -> Vec<Goal>` scoring brief §9's
considerations (survival, threat, hatred, opportunity, strategic value,
supply, distance, route danger, relationship, territorial pressure, board
position, victory progress, recovery needs, player directives, bounded
personality variation) with weights from `FactionDefinition`. `StrategicState`
is recomputed each tick from position, never set by hand. Raw scores are
never exposed to the bridge (brief §9: "do not expose raw utility arithmetic").
Done when: a desperate faction chooses recovery goals and an advantaged one
chooses expansion, from the same code and different states.

**Shipped.** `BoardView` is a pure function of state and graph; the five
states are recomputed every hour from share of the island (floors 5 / 15 /
35 / 60 %, provisional, reasoning beside each), being surrounded costs
exactly one band, and an unclaimed board is `Contesting` for everyone. Goals
are six abstract verbs; the four considerations no lane can measure yet
(distance, route danger, victory progress, player directives — S7, S7,
C9/S15, S6) are named zero-weight terms and a test proves raising them
changes nothing. Done-when holds and bites both ways. Raw scores never leave
the module; personality is S4's goal draw, capped, with a compile-time
assertion it can never outweigh position. **The registry seam:** `run_hour`
scores with neutral weights because the bridge loads no faction records and
scoring them would make Godot's island differ from the harness's;
`GoalWeights::from_definition` exists, bounded, reads only resource and
relationship weights, never `concept_key` or doctrine (tested). The lane that
makes the bridge load `content/factions/` closes the seam and deletes
`the_registry_cannot_change_a_tick_yet` — that is a B card. **Decision
recorded:** the `board_position` draw is folded but unread; position is a
fact about the board, and a future perception model is where that draw
earns its meaning.

### S6 · `StrategicDirective` vocabulary and plain-language explanation
Status: shipped `7a63396` 2026-09-06 · Depends on: S5
`StrategicDirective` with brief §19's fields; `intent` ∈ {Protect, Supply,
Develop, Expand, Pressure, Attack, Support, Investigate, Avoid, Withdraw}.
`fn explain(&directive, board) -> Explanation { goal, why_target, resources,
blockers, withdrawal_conditions, party_could_help: bool }` produced **before**
confirmation; directives persist until completed, cancelled, superseded,
impossible, or returned. Directives are high-weight inputs to S5, not
commands.
Done when: `explain` for an `Attack` names a blocker when supply is short.

**Shipped.** The ten intents and six statuses; `issue`, `cancel`, `mark` and
`explain_directive` live in `directive.rs` as an impl on `ExpeditionState`, so
the shared file gained only the field. The explanation is words — goal, why
this target, resources, blockers, withdrawal conditions, whether the party
could help — computed from `BoardView` and the graph before confirmation; an
`Attack` short of supply names the blocker, a contested route names the road.
`DIRECTIVE_WEIGHT = 480` (provisional): one standing directive moves the
leading goal of a faction that has a choice, and `directive_signal` is zero
for a Desperate faction, so Recover still leads and the directive waits rather
than being discarded — an input, not a command.

### S7 · Offscreen forces and materialisation
Status: shipped `092533f` 2026-09-06 (movement and arrival) · materialisation `blocked: needs B11` · Depends on: S4, S2
`ForceRecord` with brief §10's fields (faction, roles, composition, strength,
readiness, supply, origin, route, destination, assignment, progress,
player-detectable evidence). Forces move along `Geography` routes one step
per tick-cost. A force reaching the party's cell or an adjacent one
**materialises through that cell's spawn sockets** (B11) as actors; never
teleports. Composition, damage, supply, leadership, equipment and recruited
identity survive the aggregate↔local transition.
Done when: a test moves a force three cells and asserts its arrival event
carries the same composition it left with.

**Shipped, movement and arrival.** `ForceRecord` carries §10's fields; a force
plans with the graph's own `next_step_toward` and moves one real hop as its
progress crosses each route's authored time cost, emitting `ForceDeparted`,
`ForceMoved`, `ForceArrived` (composition copied whole) and `ForceHalted` (zero
supply halts, never vanishes). Three cells out, the same composition in; a hop
that skips a cell fails the test. Every method lives in `force.rs`; the shared
file gained the field, its initialiser and one line in `strategic_tick`.
**Materialisation through spawn sockets is B11's and blocked on it** (and B11
on O3): this lane stops at `ForceArrived` and `forces_at(cell_id)`, the seam
B11 attaches to. A9's faction actor, when it exists, arrives through here.

### S8 · Weather, corruption, and the dual clocks
Status: shipped `ba1cb56` 2026-09-06 · Depends on: S4
`GAME_BUILD_PLAN.md` names this a system contract: **world time and Cthulhu
patience/heat are distinct dimensions**, advancing one must not silently
advance the other, and each must be independently testable. So: `campaign_day`
(existing) is world time; `cthulhu_heat: u32` is its own field with its own
inputs (rituals completed, corrupted cells held, party interference — not the
passage of days by itself). The required test is the contract's own wording:
advance world time by thirty days with no Cthulhu-relevant event and assert
heat is unchanged; then trigger one heat event on a single day and assert heat
moved while the day count did not jump.
Also: `WeatherState` per region; `corruption: BTreeMap<cell, u8>`;
confrontation trigger by discovery, Day 100, or maximum heat. Cthulhu assistance events are weighted and
state-gated (only when `strategic_state ∈ {Advantaged, Closing}`), never a
rescue when losing. Reversibility of corruption is **Open** — implement
accumulation only and mark decay `blocked: needs decision`.
Done when: pressure and day advance independently in a test; an assistance
event never fires for a Desperate Cthulhu.

**Shipped.** `cthulhu_heat` moves only through `record_heat_event`
(ritual completed, corrupted cell held, party interference); the passage of
days never touches it, and the contract's own test — thirty midnights, heat
unchanged; one event, heat moved, day still — is proven to bite on a single
`+= 1` inside midnight. Weather is per region per day inside midnight (under
the save hash, outside S4's draw digest); corruption accumulates only, with
**reversibility, summoning interruption and Cthulhu early elimination each
marked `blocked: needs decision`**; the confrontation trigger answers by
discovery (`CONFRONTATION_DISCOVERY_ID` is a hook constant that a C card must
author or rename), day 100, or maximum heat; the assistance *gate* is
non-zero only for Advantaged or Closing and zero for a Desperate Cthulhu —
the event itself is S7/S10's. Every tuning number (`MAX_HEAT = 1000`, the
three heat amounts, the two assistance weights) is provisional and says so;
`CONFRONTATION_DAY = 100` is the brief's. `corrupt_cell` takes the geography
so an unknown cell is refused before mutation; the weather step derives its
regions from the graph so a save cannot conjure one.

### S9 · `DungeonContext` and generation signature
Status: open · Depends on: S3, S8
`DungeonContext` with brief §19's fields; `fn dungeon_signature(ctx) -> u64`
folds every field so two contexts differing in owner or tier never collide;
the tomb (C5) is the first dungeon whose room set and site rules (A7) are
selected by the signature. Repeated farming: reward budget draws down the
building's stored value (S3) — no infinite loot.
Done when: same context → same rooms; owner change → different site rules.

### S10 · Elimination and the recovery chain
Status: open · Depends on: S3, S7
`fn recovery_chain(faction) -> Vec<RecoveryLink>` over brief §16's list; a
faction is eliminated only when the chain is empty; on elimination:
scheduling stops, queues end, buildings transition per `ruin_state`/
`capture_rules`, territory opens, others re-evaluate, **no respawn**.
Done when: a test exhausts every link and asserts elimination; a test with
one allied refuge left asserts survival.

### S11 · Event journal and the strategic save fields
Status: shipped `8c79fde` 2026-09-06 · Depends on: S4
`ExpeditionState.strategic_journal: Vec<StrategicEvent>` (append-only,
capped by a rolling window whose evicted prefix is folded into
`strategic_history_digest`) plus every field brief §17 lists that S1–S10
introduce. Save-boundary autosave includes them. `CURRENT_SAVE_VERSION`
stays 1 while every new field is `#[serde(default)]`.
Done when: E6's fixture for v1 still loads; a strategic save round-trips.

**Shipped.** A rolling window (`JOURNAL_WINDOW = 256`, provisional and said
so) of `JournalEntry` — a `StrategicEvent` plus day and hour, nothing else —
with every evicted entry folded into `history_digest` in the shape S4's
`draw_digest` uses. Window + digest + evicted count is the complete record;
two histories differing only before the window are different saves. Proven
to bite: an unfolded eviction fails four tests. The 2,400-hour harness
carries 2,400 pushes under the hash and still matches. `strategic_tick`
journals what `run_hour` returns, stamped before the clock advances.
`JournalEntry::event` deliberately has no serde default: an invented event
on a corrupt save is worse than a load failure. `CURRENT_SAVE_VERSION`
stays 1; the v1 fixture loads with an empty journal.

### S12 · `RecruitmentState` for women
Status: shipped `83658e5` 2026-09-06 · Depends on: S1
Brief §19's `RecruitmentState` verbatim as a struct in
`ExpeditionState.recruitment: BTreeMap<String, RecruitmentState>`; stages
advance only through authored milestone flags (C6/C12), never by a timer;
"attraction creates openings, not allegiance" — a test proves high
attraction with low trust does not advance the stage. **Never surfaced as
numbers** — the bridge exposes `recruitment_stage` and the current authored
beat only.
Done when: the stage test passes; the bridge dictionary has no numeric field.

**Shipped.** §19's fields verbatim plus three the behaviour needs and §19 does
not name, each marked: the milestone-rule table (the seam C6 replaces with
records), recorded milestones (replay refusal), and the current beat. One
predicate is the only place a stage can move; a milestone with no rule is
recorded and moves nothing. *Attraction creates openings, not allegiance* is
a test, proven to bite three ways. Never surfaced as numbers, structurally:
`Disposition` has no accessor outside the file, so a numeric romance
interface is unrepresentable in the crate; `projection()` yields only
`recruitment_stage` and `current_beat_id`, and that is all the bridge may
pass (B3). Only Betty and Ayla exist; there is no terminal never-joins rung
because naming one would author content — a woman who never joins stays at
`Contact`. Decide the rung when C6 lands.

### S13 · Michael's faction production: machines, not people
Status: open · Depends on: S3
`ActorKit` entries for Michael's faction are machines only; families from
brief §5.4 as keys (`mechanical_dog`, `mechanical_cavalry`, `mechanical_bear`,
`mechanical_elephant`, `walker`, `steam_wagon`, `rocket`, `airship`) with the
brief's asset fields (§18: footprint, navigation width, turning clearance,
max slope, valid route types, bridge requirements, crew/handler, fuel and
water, repair sockets, wreck footprint, salvage value). Doctrine §5.10:
machines take initial exposure; the utility AI (S5) weights human exposure
as a cost for this faction.
Done when: production of a human role by a Michael building is rejected (S3
test extended); a machine unit is produced from a building tier with fuel
deducted.

### S14 · Founding sequence: shipwreck to first strategic core
Status: open · Depends on: S13, O5
Authored, not generated: the estate (A4) becomes `faction.michael`'s core
building at tier 1 when the player takes the first Workshop anchor action;
the first machine (a `mechanical_dog`) is produced from salvage. Exact
founding location and first construction order are **Open** (brief §5.1) —
implement the estate-as-core path only and mark alternatives blocked.
Done when: the slice test's first day ends with `faction.michael` existing,
owning `world.cell.damaged_estate`, with one operational building.

### S15 · Two Provisional doctrines approved and encoded
Status: blocked: needs approval — the brief's §6.1–6.4 are Provisional. Pick
the two for the Demo (Provisional recommendation: pirates and colonial
powers — both coastal, both give the contested route and port front M4
needs), get explicit approval, then encode their `FactionDefinition`s (C9).

### Lane B — new cards

### B1 · Rewire the campaign bridge from `RouteGraph` to `Geography`
Status: shipped 6b9d275 2026-09-04 · Depends on: A1

### B2 · `native_expedition_port.gd` sends cells and portal costs
Status: shipped 3c85451 2026-09-05 · Depends on: B1, C2 — landed inside C13, which found the port dropping every cost and gate

### B3 · Expose midnight, anchors, inspect and full legal commands
Status: shipped `9090284` 2026-09-06 · Depends on: B1, A3
The bridge projects what `legal_next_commands_with_geography` already
produced — travel, inspect and anchor actions — as `legal_commands` beside the
unchanged `legal_route_commands`, plus `discoveries`; `inspect` and
`resolve_midnight` are bridge verbs, and midnight's `WorldEvent`s go through
an exhaustive projection.

**Shipped.** The exhaustive projection is how the merge caught that S2's
`ControlChanged` had landed while B3 was in flight: the tree would not
compile under `--features godot-ext` until the arm existed, and the
integrator added it in the shape S2 specified. The lane's own push ran green
through the hardened gate. **Flagged, left in its owner's file:**
`ExpeditionState::inspect` records every observation at the cell, not the one
named; the bridge validates the name against the legal list and then records
the cell. Per-observation recording is expedition.rs's rule to change — **changed by the integrator, ce32cc2**: `inspect_observation` records exactly one and refuses one not declared here; `inspect` (the whole cell) is its sum, kept for the household room and the slice tests; the bridge calls the single one.
**Next B item, from S2:** `set_control`, `controller_of` (effective, never
raw `ownership`), `effective_risk(portal)`, route projections carrying
effective risk plus `contested: bool` and never the authored base as a
second number, and the `ControlChanged` case is already projected.

### B4 · Route board shows anchor and estate commands
Status: shipped `9090284` 2026-09-06 · Depends on: B3
A `LegalActionList` below the route list: one button per `anchor_action:` and
per `inspect:` command in `legal_commands`, and a midnight control when no
encounter is pending.

**Shipped.** Nothing is drawn from the catalog alone — it supplies labels for
commands Rust already called legal — so a spent anchor or a locked door is
never a button, the invariant the route list already kept. The prototype
suite gained nine assertions and kept every existing one.

### B5 · Battle screen: Michael's card unfolds; bands and Composure drawn
Status: shipped `ab81aee` 2026-09-06 · Depends on: A5, A6
Touches: `game/scripts/battle/battle_prototype.gd`, `game/scripts/battle/paper_card.gd`,
`game/tests/prototype_turn_cycle_test.gd`, `game/tests/native_simulation_port_test.gd`,
`godot-rust/src/battle.rs` (one addition: Michael in `prototype_vertical_slice`)
A5 put `band_name` and `composure` on every actor dictionary and the mock
agrees; A6 made Michael a battle actor but left him out of
`Battle::prototype_vertical_slice()` because `native_simulation_port_test.gd`
asserts four actors. B5 puts him in and draws what the bridge already says.
- `prototype_vertical_slice()` gains Michael at A6's stats
  (`PartyRear`, level 3, vitality 90, initiative 10, his two authored
  commands); the four-actor assertion becomes five **with the reason in the
  message**.
- The screen groups cards by `band_name` in band order (party rear → enemy
  rear), draws Composure as ten pips from `composure`, and marks `Shaken`
  from `statuses`. Michael's card, when he is the active actor, unfolds his
  command grid (Weapon Attack, Guard via `skill.system.hold_position`,
  Reposition) through the same `SimulationPort` path Betty's uses — no new
  presentation-side rule, no mock.
- Suite: `prototype_turn_cycle_test.gd` keeps every existing check and adds:
  five cards; each card's band label equals its actor's `band_name`; Michael's
  grid is enabled on his turn and disabled on Betty's; a Reposition through
  the port moves his card to the other party band.
Traps: names come from content (`Michael Corrigan` is authored); nothing
numeric from utility on the screen; the anti-mock gate; `quit(1 if failures > 0 else 0)`.
Done when: the suite passes on CI's Godot job; `cargo test` green with the
five-actor fixture.

**Shipped.** Michael is in `prototype_vertical_slice`, and A6's Captain
fixture now reads him out of it rather than restating him — one definition,
so removing him fails thirteen tests. The rail is grouped by the bridge's
`band_name` (never re-derived from the integer; an unnamed band is an error);
ten Composure pips and a Shaken mark per card; the Razorbeak gets a hostile
portrait; the invented rank/role strings are gone. Michael's grid unfolds on
his turn through the same port path as Betty's; the automatic cycle stops at
any player-commanded actor and the return-to-Betty check is kept verbatim.
Green through the hardened gate on the lane's own push. **Two gaps reported,
not filled:** Composure/Shaken redraw only on a fresh snapshot — harmless
until something spends Composure (A9), then a bridge event or a port
`snapshot()` is needed; and `skill.system.hold_position` has no content
record, so "GUARD" is a screen string — a C-lane item (a `skill.system.*`
record, with `every_authored_skill_has_its_authored_rank` extended to it).

### B6 · World cells for the tomb interior
Status: open · Depends on: A2

### B7 · Battle-entry sockets bound to habitat holders
Status: open · Depends on: B3

### B8 · New game, save slots, continue
Status: open · Depends on: E6

### B9 · Settings, accessibility, and pause **(brief)**
Adds to the first edition: a global pause that stops the character scene,
the strategic tick (S4), construction, convoys, weather and pressure; every
full-screen management, reading and accessibility surface pauses by default;
the game does not progress while closed. Test: open settings → tick count
unchanged after N seconds.

### B10 · `ContentPackRegistry` and the presentation override pack
Status: open · Depends on: C8
Autoload scanning `user://packs/` and `res://packs/`;
`ProjectSettings.load_resource_pack()` per `.pck`; read `pack.json`; merge
overrides by scene ID in load order. The scene player asks the registry for
the highest `presentationLevel` available for a scene; absent a pack, base.
One headless test with a fixture pack: level switches with the pack present,
falls back without. The simulation never reads presentation.
Done when: the test passes both ways.

### B11 · Room cells carry board metadata
Status: open · Depends on: A2, O3
Each `world.cell` gains `board: { footprint: {…5 m…}, spawnPoints: [{ id, role ∈ {player, occupant, hostile, item}, position, rigSocket ∈ {humanoid-root, creature-root, item-root} }], tethers: [{ portalId, kind, anchor }] }`
using dr-companion's field names (`boardLayoutFor`, `classifyTether`,
`tetherAnchorFor`) so a single owner later is a rename, not a rewrite. The
validator requires ≥7 spawn points and one tether per portal. S7's forces
materialise into these.
Done when: validator green; B12 reads them.

### B12 · The isometric board: world / route / room distances
Status: open · Depends on: B11, S2, **O1**
One scene graph, three LODs (world / route / room) per
`THREE_D_WORLD_STRATEGY.md` §1.1; fixed orthographic camera; the party as
miniatures snapping between nodes on confirmed travel; ownership and
influence colour the world distance; forces and convoys visible at route
distance; the room distance shows spawn sockets, exits and the encounter
space. Entering an encounter opens the **lens O1 decides**. Blockout
primitives only (brief §18).
Done when: headless test drives travel across three cells and asserts the
party miniature's node each time; a screenshot at each distance.

### B13 · Calm information surface
Status: open · Depends on: S6, B12
Ambient (changes simply appear), Notable (a companion line, journal entry,
map update — no interruption), Urgent (interrupt only for party, major
relationship, critical core, or final-stage threat). No flashing territory
alerts, no red countdowns, no quest-log spam. Directive confirmation shows
S6's explanation. Test: 100 strategic events produce ≤1 Urgent.

### B14 · Control and risk surface
Status: shipped `58fbdb3` 2026-09-06 · Depends on: S2, B3
Touches: `godot-rust/src/godot_bridge.rs`, `game/scripts/simulation/native_expedition_port.gd`,
`game/scripts/campaign/campaign_session.gd`, `game/scripts/world/expedition_prototype.gd`,
`game/tests/expedition_prototype_test.gd`
What S2 built, projected — as S2's own report specified:
- `set_control(cell_id, faction_id)` → the `ControlChanged` event(s) or the
  error dictionary (`unknown_cell`, `invalid_stable_id`); empty `faction_id`
  releases.
- `controller_of(cell_id)` → the **effective** controller (`ownership` over
  the graph's owner), `""` for unheld. Godot must never read `ownership` raw.
- `effective_risk(portal_id)` → `Geography::effective_risk` for one route.
- Any route projection carries `risk_level` = effective risk and
  `contested: bool` (`effective != authored`), and **never** the authored base
  as a second number — storing the derived value in GDScript is the fork.
- The screen shows a road's live risk and marks a contested one; the
  prototype suite flips control of the river landing through `set_control`
  and asserts the safe road's drawn risk changed and reads contested.
Traps: no faction proper names in fixtures (`faction.pirates`, `faction.elves`);
the `ControlChanged` projection already exists — reuse it.
Done when: the suite asserts the flip; CI's Godot job green.

**Shipped.** Three bridge verbs and `route_options` on the state dictionary —
effective risk and a `contested` flag per legal route, never the authored
base beside it. The route button reads live risk and marks a contested road;
the test reads the number back out of the button's own text, so the control
and the assertion are one fact and GDScript stores no risk. The prototype
suite flips the river landing through the session and asserts the safe
road's drawn risk rose by exactly the modifier, one `control_changed` event,
`controller_of` and `effective_risk` agreeing with what was drawn, and a
clean release. The lane's own push ran green through the hardened gate.
**Two duplicates the lane flagged rather than hid:** the test declares its
own `CONTESTED_RISK_MODIFIER := 2` as the expected value (accepted — a fourth
`#[func]` to expose a constant would be worse); and the bridge composed
`ownership` over the graph's owner a second time beside the closure inside
`Geography::effective_risk` — unified by the integrator into one public
`Geography::held_by` immediately after the merge.

### B15 · The bridge loads the faction records and hands the registry down
Status: shipped `777a943` 2026-09-06 · Depends on: C9, S5
Touches: `tools/src/build-content-bundle.mjs`, `game/generated/content_bundle.json`
(regenerated), `game/scripts/simulation/native_expedition_port.gd`,
`godot-rust/src/godot_bridge.rs`, `godot-rust/src/expedition.rs`
(`resolve_midnight_in`'s signature and its callers only),
`godot-rust/src/strategy/tick.rs` (`run_hour` only),
`godot-rust/tests/strategic_determinism.rs`, `godot-rust/tests/authored_world.rs`
The seam three shipped cards describe from three sides: S4 passes an empty
registry because the bridge has none; S5 scores with neutral weights because
scoring records would make Godot's island differ from the harness's; C9's
records do not reach the Godot bundle because the builder's domain list omits
them. One lane closes all three at once:
- `factions` joins the bundle's domains; the port forwards every `faction.*`
  record verbatim; the bridge builds a validated `FactionDefinitions`
  (refusing configuration if any record fails `validate`) and passes it to
  `resolve_midnight_in`, whose signature gains the registry.
- `run_hour` uses `GoalWeights::from_definition` when the record exists.
- The harness loads `content/factions/*.json` and runs the same island Godot
  does; `the_registry_cannot_change_a_tick_yet` is **deleted** — it was
  written to be — and replaced by `the_authored_registry_changes_the_tick`:
  the 2,400-hour hash with the registry differs from the hash without, and
  both reproduce.
- The state dictionary carries `factions` with `id`, `strategic_state` and
  `current_goals` names only — nothing numeric from utility.
Traps: no widening of `from_definition` (it reads resource and relationship
weights only; doctrine never acts). Bundle freshness is a CI gate.
Done when: the new harness test passes; the Godot job green; the prototype
suite asserts six factions in the snapshot and no numeric score key.

**Shipped.** `factions` in the bundle; the port forwards records verbatim —
after its own CI caught Godot's `JSON.stringify` writing integers with a
decimal point and serde refusing every record; the port now coerces and says
why a configuration was refused. The bridge validates into `FactionDefinitions`
and `resolve_midnight_in` takes it; `run_hour` scores with
`from_definition`; the harness loads `content/factions/` and runs the island
Godot runs; `the_registry_cannot_change_a_tick_yet` is deleted.
**The seam is closed honestly, not by assertion:** today's records are
placeholder-neutral by rule (one Open resource key at weight zero, empty
relationship tendencies), so a loaded registry scores exactly as an empty
one and the card's literal "hashes differ" cannot hold without answering an
Open brief question. `the_authored_registry_changes_the_tick` pins that
equality as a negative and proves the wiring against a probe record with one
weighted signal; reverting `run_hour` fails it. The day content carries a
real weight, the negative flips on its own.

### Lane C — new cards

### C1 · Loot records and a reference check that bites · shipped ab17bfa
Status: shipped ab17bfa 2026-09-05 · Depends on: —
`content/loot/` did not exist, and `content/encounters/returning_names.prototype.json`
pointed `victory.lootTableId` at `loot.razorbeak.prototype` — a record that
was not there, which the validator never checked, so the dangle passed
silently every run. Three records now exist for the `drop_table_id` values
`habitat.rs` already declares, plus a `lootFile` validator block and a
`reference()` on the encounter field.

Proven twice over: retargeting the field at a nonexistent ID fails, **and**
moving the real `razorbeak.prototype.json` away reproduces the exact dangle
that used to pass. Bundle went 28 → 33 records.

**The yields are authored, not sourced, and that is flagged on purpose.** No
Rust code held resource numbers when this was written (A3 introduces them), so
the values were set to scale with each habitat's declared rank and base level:
ordinary razorbeak 2/1/6, elevated thunderback 4/1/10, apex crested razorbeak
3/2/18 (rations/medicine/coin). If A3's economy wants different numbers it
must **overwrite these**, never add a second table — two tables of loot values
is the fork.

### C2 · Portal costs in the authored world cells · shipped ab17bfa
Status: shipped ab17bfa 2026-09-05 · Depends on: —
The authored portals carried only identity and `travelMode`; the costs lived
solely in the Rust fixture, so content could not describe what a road actually
costs. All nine now carry `timeCostMinutes`, `supplyCost` and `riskLevel`,
plus an optional reference-checked `requiredDiscoveryId`, type-checked by the
world-cell validator.

**Every number was read verbatim from `Geography::black_beach_vertical_slice()`,
not invented** — 15/0/0 for the estate climb, 30/0/1 to the river landing,
60/2/1 for the safe road, 35/4/3 for the jungle edge, 45/1/2 back, 10/0/2 on
the ramp. No authored portal carries a discovery gate yet: the only gate in
the fixture is on the tomb's archive core, and the tomb cells stay
fixture-only until B6 authors them. The field is accepted and checked so B6
can use it. (The encounter's location IDs, listed here in an earlier draft,
were fixed by A2.)

### C3 · `ayla.json` and seven Ayla skill records
Status: open · Depends on: A7

### C4 · Captain Michael's two commands · shipped ab17bfa
Status: shipped ab17bfa 2026-09-05 · Depends on: —
`captain.json` had `skillIds: []` and `supportedTargetRules` had no
self-targeting rule, so the protagonist could not be given a command at all.
Added `"self"` to the rule set and authored `skill.captain.weapon_attack` and
`skill.captain.reposition` at the full schema the seven Betty records satisfy
— ordered beats, ≥3 event bindings, a framing naming the safe frame, one
resolvable presentation cue per beat. Both are listed in `captain.json`,
whose notes now say Echo and Field Order are authored later rather than that
the deck is unexposed. **A6 depends on exactly these two records.**

Proven by breaking four things at once — removing `self`, dropping a beat a
binding referenced, pointing a cue at a nonexistent camera, and listing an
unwritten skill — and watching all five failures appear.

### C5 · The tomb as a faction-specific dungeon **(brief)**
As the first edition (twelve spaces per bible §3.13) plus: the tomb's cells
carry `site_rule_ids` and a `dungeonContext` block; S9 selects rules by
owning faction (elves by default; corrupted variant when Cthulhu holds it).

### C6 · Relationship scene records and schema
Status: shipped `3857ffa` 2026-09-06 · Depends on: —
Touches: `content/relationships/*.json` (new), `tools/src/validate.mjs` (one
block), `godot-rust/src/strategy/recruitment.rs` (`MilestoneRule::from_authored`
and one equality test), `game/generated/content_bundle.json` (regenerated)
The romance is a key feature, fade-to-black in the base game with the adult
pack as a presentation override (C8/B10/E9). S12 built the seam: a woman's
stage moves only through authored milestones, and `RecruitmentState.milestone_rules`
is a data table waiting for records. **A scene record is that record.**
- `scene.<woman>.<slug>` with `womanId` ∈ the established two
  (`character.heroine.betty`, `character.heroine.ayla` — never a third),
  `grantsMilestoneId: "milestone.<woman>.<slug>"` (registered as a stable ID),
  `rule` mirroring `MilestoneRule` exactly: `advancesTo` (a `RecruitmentStage`
  name), `requiresStageAtLeast`, `requiresAtLeast` / `requiresAtMost` keyed by
  `Inclination` names, `blockedByConditions`, `clearsConditions`.
- `presentationLevel: "fade_to_black"` on every record — the only value the
  base game may carry (C8's pack overrides it; the validator refuses anything
  else here). `beats[]`: `{ id, kind ∈ {conversation, action, choice, fade},
  text }` — placeholder prose is fine and should say so; no explicit content
  in the base tree, ever.
- Validator: register `id` and `grantsMilestoneId`; `womanId` must be one of
  the two; stage and inclination names must be the Rust enums' serde
  spellings (read them from `recruitment.rs`, do not guess); a `fade` beat
  must be the last beat.
- Rust: `MilestoneRule::from_authored(&serde_json::Value)` (or a `Deserialize`
  wire struct with aliases, as `AuthoredAnchor` does) is the **one**
  translation; `every_authored_scene_rule_loads` reads `content/relationships/`
  and holds each rule to it. Content owns; Rust carries; a test holds them
  equal.
Traps: attraction alone must not advance a stage — author at least one scene
whose rule requires trust, and let S12's existing test keep proving it. No
timers. Two or three scenes per woman is enough to prove the shape.
Done when: the validator passes with the records; the equality test passes;
a scene with `presentationLevel: "explicit"` in the base tree fails the
validator (bite).

**Shipped.** Five scenes — three for Betty, two for Ayla — each granting one
milestone whose rule is S12's `MilestoneRule` itself (serde aliases for the
authored spelling; one type, not a copy). `adopt_authored_scenes` is the one
door into a woman's rule table; `every_authored_scene_rule_loads` holds the
table equal to the directory. Three rules rest on a `TrustInMichael` floor
and the validator refuses the directory if none does. `fade_to_black`
everywhere, refused otherwise — the adult pack (C8/B10/E9) overrides this
seam. Proven to bite three ways in both the validator and Rust. Prose is
marked placeholder and written as real moments that stop at the fade.
**Left, on purpose:** Ayla has no character record (C3 ← A7/H5), so the
validator carries a one-line named exception for her alone, deleted when C3
lands; the never-joins rung is still undecided, and a woman who never joins
stays at `Contact`; conditions are opaque `condition.*` IDs with no registry
yet. Stable IDs 200 → 210.

### C7 · Placeholder deprecation migration
Status: open · Depends on: D4

### C8 · Scene `presentationLevel` and the pack manifest schema
Status: shipped `a579ff9` 2026-09-06 · Depends on: C6
Every scene record: `"presentationLevel": "fade_to_black"`. Pack manifest
`packs/<id>/pack.json`: `{ "id": "pack.presentation.adult", "kind": "presentation_override", "targetLevel": "explicit", "overrides": { "<scene id>": { "beats": [...], "assetIds": [...] } } }`.
Validator `packFile`: every override targets an existing scene ID; every
asset ID resolves inside the pack; `targetLevel ∈ {fade_to_black, explicit}`;
a pack may not carry rules, skill, character, faction or building records.
Done when: validator green with a fixture pack; a pack carrying a skill fails.
**Shipped:** `packs/pack.presentation.fixture/pack.json` at `fade_to_black`
with two overrides; the validator's `packFile` block registers scene IDs
first, then refuses an override on a scene nobody wrote, a pack carrying a
`skill` key, and an asset ID outside the pack's own `assets` list. "Resolves
inside the pack" means a pack-relative path the validator `stat`s; pack asset
IDs are deliberately *not* repository stable IDs, so a base-game record can
never reference something that may not be installed. Two bites proven. The
pack is not in the bundle by design (B10 merges packs at load). No adult pack
exists yet — the seam does.

### C9 · Faction records
Status: shipped `3598628` 2026-09-06 · Depends on: S1
faction, `displayName` **left as a placeholder marked `needs decision`** (no
proper names). Validator block registers `faction.<key>` and checks every
S1 field exists.

**Shipped.** Six records, `id` exactly `faction.<concept_key>`, `displayName`
a placeholder the validator **requires** to contain "needs decision" — proven
to bite on a real-looking name. `resource_priorities` is one
`resource.open.needs_decision` key at weight zero and the validator refuses any
other namespace, so the Open resource list is Open in the data too. Doctrine
is Provisional prose no code reads. Every empty field's notes name the Open
item it waits on; political form has no field. Held equal to S1's types by
`every_authored_faction_record_loads_and_validates`. **Left, deliberately:**
`build-content-bundle.mjs`'s domain list omits `factions`, so they do not
reach the Godot bundle; nothing in Godot reads them. One line when a consumer
exists.

### C10 · Building records with envelopes
Status: shipped `a579ff9` 2026-09-06 · Depends on: S3
Touches: `content/buildings/*.json` (new), `tools/src/validate.mjs` (one
block), `tools/src/build-content-bundle.mjs` (`buildings` in the domain list),
`game/generated/content_bundle.json` (regenerated), one equality test in
`godot-rust/src/strategy/building.rs`
S3 shipped `BuildingDefinition` and the contract its claim notes spell out;
this card authors the first records to it. Shapes, not a catalogue: two or
three buildings — one for Michael's faction (a workshop or core that produces
a **machine**, never a person) and one or two for ordinary factions — enough
to prove every field and both refusals.
- `id` exactly `building.<slug>`; `faction_compatibility` as concept keys
  (never names); `tier_states` ascending from 1, no gaps, ≤ `TIER_CAP`;
  `capture_rules` **explicitly** `capturable` or `destroy_only` and
  `ruin_state` **explicitly** `clears_completely` or `leaves_rubble` — a
  record that omits either is refused by S3's `validate` naming brief §20, and
  the validator must mirror that refusal; sockets in their §19 field for
  their §8 kind with `offset_cells < footprint_cells`; `production` rules
  with `minimum_tier` ≤ the top tier; `height_class` a placeholder marked
  needs decision.
- Validator: register `id`; envelope integers; socket lists; the
  human-role rule (refused outright on a record listing `michael`; on the
  others only with `interval_hours: 0` and a non-empty `recruitment_support`).
- Rust: `every_authored_building_record_loads_and_validates` reads
  `content/buildings/` into `BuildingDefinitions` through `validate`, the
  shape S1/C9 set — content owns, Rust carries, a test holds them equal.
Traps: no resource categories invented — `construction_cost` keys under the
`resource.open.` namespace only, as C9 did; no proper faction names; no
doctrine.
Done when: validator and equality test green; a Michael record producing a
`human_role` fails both (bite); a record omitting `capture_rules` fails both.
**Shipped:** three records — `building.machine_shop` (Michael; a machine,
capacity and a service, `recruitment_support` empty on purpose),
`building.coast_watch_post` (three ordinary factions; the tree's one legal
`human_role` rule, interval 0 with recruitment support) and
`building.ritual_anchor` (`destroy_only` / `clears_completely`, empty socket
lists prove an empty field loads). Every Open item is Open in the data:
`height_class` a placeholder the validator *requires* to say needs decision,
costs and output keys under `resource.open.needs_decision`, empty terrain.
`every_authored_building_record_loads_and_validates` holds `building.rs`
equal to the directory. All four bites proven in both the validator and
Rust. Stable IDs 210 → 214; bundle 44 → 47 records. **Left, on purpose:**
`ProductionOutput::Machine` carries no machine family (S13 may extend);
`role.*`, `service.*` and `recruitment_support.*` have no authored vocabulary.

### C11 · Room contract fields on world cells — brief §3's list (function,
dimensions, circulation, slots, landmarks, encounter space, material
language, avoid-list) as required fields; the five existing cells filled in.

### C12 · One recruitable woman's arc — blocked: needs decision O2.

### C13 · Content can declare a discovery ID; the tidal cut's gate authored
Status: shipped `3c85451` 2026-09-05 · Depends on: A4
Touches: `tools/src/validate.mjs`, `content/world/black_beach.world_cell.json`,
`content/world/damaged_estate.world_cell.json`, `godot-rust/src/geography.rs`
The map table grants `discovery.map_table.tidal_cut` and the tidal cut requires
it, but only the Rust fixture can say so: `validate.mjs` resolves
`requiredDiscoveryId` against registered stable IDs and no content record
declares a discovery, so authoring the gate fails validation. The fixture
carries it under `GATES_CONTENT_CANNOT_YET_DECLARE` with the deletion condition.
- Give a world cell a way to declare the discoveries its anchors grant — an
  authored `anchors[]` block on the cell (`id`, `kind`, `grantsDiscoveryId`,
  `requiresObservationId`), registered as stable IDs the same way observations
  are. This is the content twin of A3's `CellDefinition.anchors`, which today
  exists only in the fixture; `from_authored` already consumes it.
- Do **not** add a `discovery.` entry to `intentionallyExternalPrefixes`. That
  lets content reference an ID nothing declares, which is the dangle C1 existed
  to close.
- Author the estate's four anchors and the map table's grant on
  `damaged_estate.world_cell.json`; author `requiredDiscoveryId` on the tidal
  cut in `black_beach.world_cell.json`.
- Extend `fixture_matches_the_authored_world_cells` to hold anchors equal
  between fixture and content, then **delete** `GATES_CONTENT_CANNOT_YET_DECLARE`
  and its branch.
Traps: the exception constant must not survive this card. A second place to
declare an anchor's ID is the fork.
Done when: the validator passes with the gate authored; the constant is gone;
`grep -rn GATES_CONTENT_CANNOT_YET_DECLARE godot-rust/` prints nothing.

**Shipped, as the fix for the first red Godot CI run.** The card as written
was one of four stacked faults. `native_expedition_port.gd` forwarded only a
portal's id, endpoints and travel mode, so **C2's costs and any gate had never
reached the engine** — every road was free in Godot and A3's scarcity had
never applied there. `legal_routes` had no gate filter, so a locked door was
drawn as a button. And `readableDescriptions[].id` — the observations the
Rust world gates on — were never registered as stable IDs. All four are
fixed. Forwarding costs then stranded a fresh party on the sand (zero rations,
no anchor in Godot to salvage from), so the **minimal core of B3** ships with
it: cells and anchors forwarded in their authored shape, `use_anchor` on the
bridge with an `anchor_outcome` block, session and scene wrappers, and the
prototype test salvaging the wreck before it travels. `tests/authored_world.rs`
performs the GDScript's translation field for field and drives the prototype's
opening on it — the nearest local proof; the Godot job is the real one.

### Lane D — new cards

### D1 · Generate Betty's identity plate from `still_image_plan.json`
Status: open (human runs the generator) · Depends on: —

### D2 · Generate the razorbeak's identity plate
Status: open (human) · Depends on: —

### D3 · Review both plates; flip `generationGate`
Status: open (human sign-off) · Depends on: D1, D2

### D4 · Betty rigged GLB via the rig switch; Godot admission check
Status: open · Depends on: D3

### D5 · Betty's five Guarded Strike clips
Status: blocked: waiting on the animation tool · Depends on: D4, animation tool

### D6 · Razorbeak rigged GLB (open) and clips (blocked)
Status: open / blocked · Depends on: D3, animation tool

### D7 · Reception Terrace dressed; first admitted shared assets
Status: open · Depends on: —

### D8 · Record the measured per-actor asset bill
Status: open · Depends on: D4, D6

### D9 · Building blockout kit — cube/multi-cube blockouts per C10's
envelopes; nothing outside the box (brief §8). Reviewed at gameplay distance.
### D10 · First machine family blockouts — one `mechanical_dog` and one
`steam_wagon`, riveted-iron material language (brief §5.3), future-ready
pivots/sockets, no animation.
### D11 · Card rail and command grid in the bronze-and-vellum grammar
Status: open · Depends on: —

### D12 · Room blockouts for the Demo region — from C11's contracts.

### Lane E — new card

### E1 · GitHub Actions: Rust and content checks
Status: shipped e0b4051 2026-09-05 · Depends on: —

### E3 · Shell equivalents of the two PowerShell gates
Status: shipped 5344b7b 2026-09-05 · Depends on: —

### E4 · Desktop export presets · shipped 838824e
Status: shipped 838824e 2026-09-05 · Depends on: —
`game/export_presets.cfg` held exactly one preset — `Web Preview`, with
`variant/extensions_support=false`. The whole simulation is a GDExtension, so
that preset **cannot load the game's own rules**, and no desktop preset
existed at all: there was no way to produce a build a person could play.
Added Windows Desktop, Linux Desktop and macOS, each including
`bin/project42_sim.gdextension` and that platform's two `expedition_v5`
libraries, with `exclude_filter=""` so `bin/` survives the export. `Web
Preview` is kept, and renamed so its own name says it is preview-only.

**Two findings, both real:**

1. **`game/bin/` contains only `windows/`.** The `.gdextension` declares six
   libraries across three platforms; two of those directories do not exist,
   so the Linux and macOS presets cannot yet produce a *loadable* build. The
   fix is **not** to commit more binaries — it is that whatever produces a
   release must build the library first. That is now a hard requirement on E5.
2. **`platform="Linux"` is the riskiest line in the file.** Godot renamed the
   Linux export platform from `Linux/X11` in the 4.3 era, and a preset whose
   `platform` string does not resolve is dropped *silently*. `verify-godot.ps1`
   pins 4.7.2, so `"Linux"` should be right — but whoever runs the first
   export must open the Export dialog, confirm all three presets appear, and
   flip the string if Linux is missing.

Unproven, and honestly so: no export was run, because Godot is not installed
in this container. The file was checked with `configparser` for structure,
`[preset.N]`/`[preset.N.options]` pairing and quoting, and every library path
was cross-checked against the `.gdextension` — but the first real
`--export-debug` is the actual proof. Expect Godot to rewrite the file and
drop its comments the first time the Export dialog saves; the preset *rename*
is the durable half of the warning.

### E2 · Godot headless suites in CI — what "cannot pass by accident" cost
Status: shipped `7b8f698`, hardened `5107113` 2026-09-06 · Depends on: E1, E3
The job's original proof was three-fold: the built `.so` at the declared
path, none of five fallback markers in the gate log, all three live-bridge
lines present. Still true, still necessary. It was not sufficient.

Run `826eb5a` was green with two `ERROR:` lines in its log: the reception
terrace scene had no node for the `entry.reception_terrace.tidal_cut` anchor
A4 authored, `world_cell_test.gd` said so, and the gate reported 17 suites
passed. The suite called `quit(1)` on the failed check and then reached an
unconditional `quit(0)`; `SceneTree.quit()` sets the exit code and returns,
so the last call wins. **Fifteen of the seventeen suites had that shape.**

Fixed at two levels, deliberately redundant. Both `tools/verify-godot.sh`
and its declared twin `verify-godot.ps1` now fail any step whose output
contains `ERROR:`, `SCRIPT ERROR:` or a GDScript backtrace, whatever the
exit code — legitimate runs print none; checked against the green logs
first. And the seven suites with the `push_error`-then-`quit(1)` shape now
count failures and decide once at the end.

Rule for whoever adds a suite: end with `quit(1 if failures > 0 else 0)`, or
use a `finish()` gate. Never a bare trailing `quit(0)` after checks.

Two lessons this card exists to keep. A green run is a claim; the log is the
evidence — twice in one day, reading a green log found something the tick
did not. And the shape of the *first* failing suite decided whether the gate
could see at all.

### E5 · Nightly build artifacts
Status: open · Depends on: E2, E4
A scheduled workflow that exports all three desktop presets and uploads them
as artifacts named with the commit SHA.

**Hard requirement, from E4's finding:** the job must run
`bash tools/build-native-bridge.sh release` **before** exporting. `game/bin/`
holds only the Windows libraries; Linux and macOS are built, never committed.
An export that skips that step produces a package with no simulation in it —
it will launch and be unable to load a single rule. Assert the library exists
at the path the `.gdextension` names before invoking the export, so this
fails loudly instead of shipping a hollow build.

Note the distinction from E2: E2 is genuinely unblocked, because its CI job
builds the `.so` itself at run time via the same script — confirmed by its
first green run, which built the library and loaded it. It is *release
packaging* that the missing committed binaries affect, not the test job.

### E6 · Save-version migration fixtures
Status: shipped `85413f0` 2026-09-05, blind spot closed `095f3ac` 2026-09-06
Two committed v1 fixtures (`v1_minimal.json`, the shape an early save really
had; `v1_current.json`, a generated full round trip) and a suite that loads
every fixture in `tests/saves/`, so a dropped `serde(default)` or a renamed
field fails here rather than in a player's save.

**Gap found by S1, 2026-09-06.** The added-fields test took its reference key
set from the serializer under test, so a field marked `skip_serializing`
passed it while vanishing from every save. S1's in-module round trip caught
it; this suite did not. **Closed:** `EXPECTED_CURRENT_KEYS` is a literal list
of the struct's fields in struct order, and
`the_current_save_writes_exactly_the_fields_the_struct_declares` holds the
serializer to it in both directions. Proven to bite: `skip_serializing` on
`ownership` fails it naming the field. Adding a field now means adding it to
the list and regenerating `v1_current.json`, on purpose.
Rule: a save fixture is generated by the code, never hand-written.


### E7 · Crash log with state snapshot; no silent telemetry
Status: open · Depends on: —

### E8 · Claims enforcement in CI
Status: shipped 528773b 2026-09-06 · Depends on: E1
Touches: `tools/src/check-claims.mjs` (new), `.github/workflows/verify.yml`
(one step in `rust-and-content`), `.agents/README.md` (one paragraph naming
the check)
The claims ledger and the ship-plan ledger close by discipline alone. This
branch found rows without cards five times and commit messages describing
edits that were not in the commit twice. Make both ledgers a gate.
- `check-claims.mjs` reads every `.agents/claims/*.json`: valid JSON;
  `task_id` equals the file stem; `status` ∈ {active, blocked, completed};
  ISO-8601 `claimed_at`/`updated_at`; `paths` non-empty. For `completed`:
  `completion.commit` non-null, `git cat-file -e` finds it, and it is an
  ancestor of `HEAD` (`git merge-base --is-ancestor`); `checks` non-empty and
  each check string ends in `passed`, `failed` or `not_run`. For `active` /
  `blocked`: `completion.commit` null.
- The same script reads `docs/SHIP_PLAN.md`: every ledger row `| X | … |
  shipped <sha> <date> …` must have a `### X ·` card whose `Status:` line says
  `shipped` and names the same sha, and `git cat-file -e` finds the sha. Every
  row that is not `shipped`/`superseded` must have a card (this is the
  row-without-card rule).
- CI: one step after "Validate content" running `node tools/src/check-claims.mjs`.
  It needs the full history: `actions/checkout` with `fetch-depth: 0` in that
  job (check what the job uses today).
Traps: never loosen a check to get green — fix the claim. If an existing
claim or row fails the new script, that is a finding: fix the record in the
same lane and say what was wrong.
Done when: the script passes on the current tree; a fixture claim with a
bogus SHA fails it (bite); CI green with the step present.

**Shipped.** `check-claims.mjs` runs after the validator with full history.
On arrival it found and corrected — as records, never by loosening a check —
94 check entries across 25 claims with no trailing verdict, one claim with
offsets instead of `Z`, A2's card saying open against a shipped row, four
cards with no `Status:` line, **twenty-two rows with no card** (each given
one restating the row), and H7 recording a dr-companion commit as if it were
here. Every correction is listed in the claim. From here the integrator runs
`node tools/src/check-claims.mjs` as the fifth standing proof before every
ledger commit.

### E9 · Pack build script and pack artifact — `tools/src/build-pack.mjs`
runs `godot --headless --export-pack` on `packs/<id>/`; CI uploads the `.pck`
beside the nightly; the adult-store build is base + pack, never a second build.

### Lane H — cards for this pass

### H5 · Site-rule spec into `HEROINE_AYLA_DESIGN.md`
Status: open · Depends on: —

### H6 · Retarget PR #3 to `main`; note PR #4; flag the stale status branch
Status: shipped 34142a8 2026-09-05 · Depends on: — · PR #3 now targets `main`, conflict resolved by merging it

### H8 · Regenerate the design bible `.docx`
Status: open (needs `python-docx`) · Depends on: H3, H9

### H9 · Bible: superseded entries marked · done — `req.scope.cast.core` and
`req.scope.campaign.duration` rewritten to the brief; protagonist hypothesis
→ Captain Michael; a "current authority" callout added at the top of the
generator naming the brief.
### H2 / H10 · superseded by `main`
While this plan's first edition was being written, `main` rewrote
`GAME_BUILD_PLAN.md` end to end (`e77c9e6`, "replace side-view build direction
with autonomous RTS contract") and stamped a prototype-scope banner into every
other document. That rewrite states the roster, the party size, the dual
clocks and the board contract directly, so the targeted edits H2 and H10
described no longer have anything to edit. Both sides reached the same
conclusion independently; `main`'s is canonical and was taken wholesale in the
merge. The edits that were *additive* rather than duplicative — the rigging
and animation-tool wording in the 3D contracts, the shared-asset platform's
presentation line, the RUNBOOK's generator wording — survived the merge and
stand.

---

# Part III — Bookkeeping

## 8. Claiming, finishing, and keeping this document true

**To claim a task:** set its ledger row to `claimed <agent> <YYYY-MM-DD>`.
Create `.agents/claims/<lane><n>-<slug>.json` from `.agents/README.md`'s
template with `status: "active"`, the card's `Touches` as `paths`, the task ID
in `summary`. Commit both: `Claim <ID>: <title>`.

**To finish:** run the card's `Done when` and keep its output. Set the row to
`done <short-sha>`. Set the claim `completed` with that SHA and the commands
you ran under `completion.checks`. Commit code, row and claim **together**.

**To block:** `blocked: <one sentence naming the exact condition>`; leave the
claim `active`; do not route around it. A block on a brief-Open item names
the decision (`blocked: needs decision O2`).

**To add:** next free number in its lane, a row, a card. Never renumber.

**Never edit** another agent's row or card except to add a dated note.

**The dangle sweep** — run before every docs commit; it must print nothing:
```
git grep -n -i -E "painted 2d|2d party|painterly 2d|commission(ing|ed) (a|an|the|from)|not yet commissioned|rating variant|prose throughput|RouteGraph|six heroines|first-six|player-named|Captain Jack|three-lane|CharacterId\[1\.\.4\]|rest_at_estate" -- docs README.md work/build_design_bible.py ':!docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md' ':!docs/STATUS.md' ':!docs/SHIP_PLAN.md'
```
(`STATUS.md` is a dated record and keeps its history; the brief is the
authority and names what it rejects; this plan quotes the superseded terms in
order to retire them. A naval *commission* in a character biography is not a
dangle -- the pattern is scoped to art-production usage.)

**Where truth lives:** the brief for design; this file for the plan;
`docs/STATUS.md` for what was last verified and when; `.agents/claims/` for
who is doing what. If they disagree, the brief wins on design, the claim
ledger on "who", `STATUS.md` on "what passed", and this file is corrected to
match — never the other way round.

## 9. Working in parallel

Lanes exist so several agents can work at once without meeting in a file.
The rules below are what make that safe. They are short on purpose.

### One lane, one clean checkout

Never work two lanes in one checkout, and never work a lane in the trunk
checkout. Take a git worktree — a real clean checkout sharing one object
store:

```bash
cd /home/user/project-42-pirate-island-rpg
git fetch origin
git worktree add ../p42-lane-A backend/b0-expedition-state   # then branch
cd ../p42-lane-A
git switch -c lane/A2-canonical-world-cell-ids
```

Branch name is `lane/<ID>-<slug>`, one branch per task, never per lane —
a task is the unit that ships. When the task is done:

```bash
cargo fmt --manifest-path godot-rust/Cargo.toml -- --check
cargo test --manifest-path godot-rust/Cargo.toml
cargo check --manifest-path godot-rust/Cargo.toml --features godot-ext
node tools/src/validate.mjs && node tools/src/build-content-bundle.mjs
git push -u origin lane/A2-canonical-world-cell-ids
```

Then merge to the trunk (`backend/b0-expedition-state`), re-run the same
four commands **on the trunk**, and only then write `shipped` in the ledger.
That second run is the integration proof; `done` without it is not shipping.
Remove the worktree when the task ships: `git worktree remove ../p42-lane-A`.

### Which lanes can run at once

Lanes touch different trees, so most pairs never conflict. The ones that do:

| Lane | Owns | Collides with |
|---|---|---|
| A | `godot-rust/src/{expedition,geography,habitat,hunter,battle}.rs` | S (shares `expedition.rs`'s struct), B (shares the bridge) |
| S | `godot-rust/src/strategy/**` (new tree) | A only where it adds `ExpeditionState` fields |
| B | `godot-rust/src/godot_bridge.rs`, `game/**` | A and S when they add bridge surface |
| C | `content/**`, `tools/src/validate.mjs` | D (both touch `content/art/`) |
| D | `work/art/**`, `game/assets/**` | C |
| E | `.github/**`, `tools/*.sh`, `game/export_presets.cfg` | nothing |
| F, G, H | audio, test scripts, `docs/**` | nothing |

**Safe to run fully in parallel today:** A2 · E1 · E3 · H5 · H6 · H7 · D1 ·
D2. Each touches a disjoint set. **Not** alongside A2: C1 and C2, because all
three edit `tools/src/validate.mjs` — run them in the round after A2 ships.

**The ledger is not a shared editing surface.** A worktree agent does not edit
`docs/SHIP_PLAN.md`: three agents rewriting the same table conflict every
time. An agent reports its result — task ID, branch, commit, the four proof
commands and their output — and whoever integrates writes the `done` or
`shipped` row in one place. The claim file in `.agents/claims/` *is* per-agent
and per-task, so it is written by the agent as normal.

**The `ExpeditionState` rule.** A and S both add fields to one struct. Any
new field is appended at the end, is `#[serde(default)]`, and never changes
an existing field's meaning. Two agents adding fields then merge cleanly; a
reordering does not. `CURRENT_SAVE_VERSION` bumps only when a field's
meaning changes, and only with a migration fixture (E6).

**Two environment facts, learned by getting both wrong first.**

*Worktree isolation cuts from the session's primary repository, not from the
one your task names — and attaching this repo does not change that.* The
session's primary repo is `dr-companion`; this one is attached and registered
as a second root, and agent isolation **still** produces a `dr-companion`
checkout (219 MB of it, per agent, under `.claude/worktrees/`). So do not rely
on `isolation: "worktree"` for work in this repository. Have the agent make
its own worktree from the real clone, and verify before it edits anything:

```bash
cd /home/user/project-42-pirate-island-rpg && git fetch origin
git worktree add -b lane/<ID>-<slug> /home/user/p42-lane-<ID> origin/backend/b0-expedition-state
cd /home/user/p42-lane-<ID>
git remote -v | grep -q project-42-pirate-island-rpg && ls godot-rust/src/geography.rs
```

Give every agent that check as step 0 and have it report the output. An agent
that finds itself in the wrong repository must stop, not improvise — one did
exactly that and was right to.

*The fetch refspec was single-branch, and lied about what was published.* This
clone was made with `--depth 1`, which sets
`+refs/heads/main:refs/remotes/origin/main`, so every `origin/<other-branch>`
tracking ref stayed frozen at clone time. Pushed work read as unpushed, and
`git log origin/backend/b0-expedition-state` showed hours-old history. It cost
one agent a wrong base branch and cost the integrator two contradictory wrong
conclusions in a row. Repaired to `+refs/heads/*:refs/remotes/origin/*`.

**The rule worth keeping: `git ls-remote origin <branch>` asks the server;
`git rev-parse origin/<branch>` asks a local cache that may be stale.** When
they disagree the server is right. Settle every "is it actually pushed?"
question with `ls-remote`.

### Spawning agents for lanes

Launch lane agents on **Opus 5 at medium reasoning**, one agent per task, each
in its own worktree (`isolation: "worktree"`). Give each agent exactly: the
task ID, its card from §7, `AGENTS.md` §0, the brief's Instructions section,
and the ledger rules from §8. Do not give an agent two tasks. Do not give an
agent a task whose dependencies are not `shipped`.

An agent that finds its card wrong or impossible marks the row `blocked:` and
stops. It does not redesign the task, and it never fills a brief-Open item
with an invention.
