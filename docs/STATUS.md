# Status

Running record of what has actually been verified in this repository, updated as of each evaluation rather than left to go stale. Update this file when re-running these checks; do not assume a prior entry still holds.

## 2026-09-02 (updated)

`main` moved substantially during this session (the actual maintainer pushing directly, in parallel): the native bridge went from "wired" to fully built and Godot-verified, and the battle engine gained enemy-intent authority, interception redirection, full turn cycles, structured combat text, and a new "recovery opening" mechanic. This entry reflects the state after merging that in and adding this session's own work on top.

- **Native Godot↔Rust bridge: done, and now Godot-verified on the host that built it.** `docs/NATIVE_BRIDGE.md`'s acceptance gates 1–12 all report passing there, including the native library build, `.gdextension` load, and a live Godot-to-Rust command round-trip. Re-verified independently in this Linux container (no Godot/Windows toolchain here, so only the engine-independent checks apply): `cargo test` (36/36 pass, up from 24 as of last entry — the maintainer's turn-cycle/interception work and this session's 5 new Ayla-mechanic tests both landed), `cargo check --features godot-ext` (passes), `cargo fmt -- --check` (clean).
- `node tools/src/validate.mjs`: passes — 106 stable IDs, 7 skills, 37 presentation cues, 15 video reels, 2 still-image plates, 3 locations, 3 tracked placeholders.
- `node tools/src/build-content-bundle.mjs`: regenerates `game/generated/content_bundle.json` with no unexpected diff.
- **Art:** `content/art/still_image_plan.json` (added earlier this session) has generation-ready Magnific prompts for the 3 still-open placeholders — Betty's production identity sheet and a new razorbeak production identity sheet backed by `content/art/razorbeak.reference_ledger.json`. Still queued, not yet commissioned.
- **Ayla (second heroine): 4 of 7 skills now implemented in `godot-rust/src/battle.rs`** — Reach Counter, Structural Scan, Safe Passage, Curse Dispel — each with a dedicated Rust test. She's still not `content/characters/ayla.json`: the remaining 3 skills (Ward Line, Deny Activation, Override Tomb Rule) need engine subsystems that don't exist for any character yet (hostile band movement, a `Staggered` status, world/site-persistent state across encounters), and the character schema requires all seven or none. Full detail, including exactly which subsystem blocks which skill, is in `docs/HEROINE_AYLA_DESIGN.md`.
- **World/location content:** added `content/locations/` (`location.road.reception`, `location.estate.infirmary`, `location.tomb.returning_names`) with validator support. This also closed a real gap: `encounter.prototype.returning_names`'s `locationId` and `defeat.returnLocationId` fields were previously unvalidated free-form strings; they're now checked stable-ID references like every other cross-domain link in the schema. Location records intentionally stay out of `game/generated/content_bundle.json`, consistent with how `content/art/*` planning files are also excluded — confirmed by re-running the bundle build and seeing no diff.

**Summary:** the native bridge milestone is fully complete and Godot-verified. Ayla is 4/7 implemented at the engine level (not yet playable — no content exists for her). The next real increments are: (1) the 3 remaining Ayla mechanics, each blocked on a named missing subsystem; (2) commissioning the 2 queued production identity sheets to retire Betty's and the razorbeak's placeholders; (3) authoring `content/characters/ayla.json` once (1) lands.

## 2026-09-03 (updated)

Synced this branch with `main` again (main gained the 3D world-cell/battle-staging pipeline and Betty's identity-locked art in parallel; merged cleanly, no conflicts) and unblocked one more Ayla skill.

- **Ayla (second heroine): 5 of 7 skills now implemented in `godot-rust/src/battle.rs`** — Reach Counter, Structural Scan, Safe Passage, Curse Dispel, and now **Ward Line**. Ward Line needed two new engine primitives that genuinely didn't exist before: a general-purpose `apply_status` helper (the engine's first *dynamic* mid-battle status application — every prior status was only ever set at actor construction or removed by a skill) and a new `Staggered` `StatusKind`. "The first enemy crossing" a placed ward is translated the same way Reach Counter translated "an enemy entering the Contested band" — no hostile in this engine ever moves bands, so the trigger is a hostile's declared attack targeting a party member sharing the warded band. 3 new dedicated tests (`ward_line_damages_and_staggers_the_first_hostile_attacking_the_warded_band`, `ward_line_is_spent_after_its_first_trigger_and_does_not_retrigger`, `ward_line_does_not_trigger_when_no_ward_is_active`). Still not `content/characters/ayla.json`: the remaining 2 skills (Deny Activation, Override Tomb Rule) need an expedition/site-scoped concept spanning multiple encounters, which is a real architectural decision (touching `ExpeditionState`, not just `Battle`) rather than a same-battle translation problem like Ward Line was. Full detail in `docs/HEROINE_AYLA_DESIGN.md`.
- `cargo test --manifest-path godot-rust/Cargo.toml`: 39/39 pass. `cargo check --features godot-ext`: passes. `cargo fmt -- --check`: clean.
- `node tools/src/validate.mjs`: passes — 117 stable IDs, 7 skills, 37 presentation cues, 15 video reels, 2 still-image plates, 3 locations, 6 tracked placeholders (counts moved because `main`'s merged-in 3D pipeline work added its own content records).

**Summary:** Ayla is 5/7 implemented at the engine level (still not playable — no content exists for her). The next real increments are: (1) design and implement the expedition/site-scoped concept Deny Activation and Override Tomb Rule both need; (2) commissioning the 2 queued production identity sheets to retire Betty's and the razorbeak's placeholders; (3) authoring `content/characters/ayla.json` once (1) lands.

## 2026-09-04

This branch (`backend/b0-expedition-state`, PR #3) is now the unified spine. The full plan from here to a shippable product is `docs/SHIP_PLAN.md`; its lane ledgers are the live task list.

- **Two independent `expedition.rs` implementations reconciled into one.** `codex/battle-frontend-vertical-slice` had written its own `ExpeditionState`, `RouteGraph`/`PortalDefinition`, and the Godot campaign bridge; this branch had the deep simulation (geography, habitats, day/night rosters, hunters, the Tomb of Returning Names, Midnight Return, one estate action) with no campaign bridge. Merged here: this branch's state and `Geography` survive; their `resolved_encounter_ids`, `EncounterState.estate_upgrade_id`, `TravelBlockedByEncounter`, authored encounter triggers and the whole campaign bridge survive, rewired onto `Geography::from_authored(cells, portals, triggers)`. The bridge now resolves defeat and retreat too (the old one resolved victory only and left a lost fight pending forever).
- `cargo test --manifest-path godot-rust/Cargo.toml`: **110/110 unit + 1/1 integration pass**. `cargo check --features godot-ext`: passes. `cargo fmt -- --check`: clean.
- `node tools/src/validate.mjs`: passes — 169 stable IDs, 7 skills, 37 presentation cues, 15 video reels, 6 creature still-image plates, 5 shared asset records, 33 source collections, 12 tracked placeholders.
- **Known and scheduled (see `SHIP_PLAN.md` A2/C2):** three ID namespaces exist for the same five places — `world.cell.*` (authored world cells, canonical), `location.black_beach.*` (this branch's Rust fixture), and `location.road.reception` / `location.estate.infirmary` (this file's 2026-09-02 entry, and `content/encounters/returning_names.prototype.json` on `main`). The `content/locations/` records the 2026-09-02 entry added are retired by A2 in favour of world cells; their validator reference checks on the encounter record are kept and repointed.
- **Known and scheduled (see A3/C1):** nothing produces rations, medicine or coin; `drop_table_id`, `loot_seed` and `interaction_anchor_ids` are declared and never read; the encounter record's `lootTableId` names a record that does not exist and the validator does not check it.
- **Ayla:** 5/7 in the engine on PR #2 (not yet merged here — P0.5). Deny Activation and Override Tomb Rule are unblocked by the tomb's site-scoped state; A7/H5 specify the mechanism.
- **Art:** unchanged — gate closed pending plate approval (D1–D3). Rigging is a generator switch; animation waits for the next-generation tool and is not attempted by hand.
- **Not yet:** CI, desktop export presets, a Linux-runnable Godot gate (E1–E4).

**Summary:** one spine, compiling, green. Next increments in order: commit the merge (A1/B1), canonical IDs (A2/C2), fold PR #2 in, then M1 (economy, estate, protagonist, bands, Ayla) with M2 (CI, presets) in parallel.

### 2026-09-04, later — the continuation brief

`docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md` arrived and is the current design authority: an autonomous RTS experienced from inside by Captain Michael and four women; six factions; one map node is one playable room; fixed isometric presentation; normal pause; organic paths; genuine loss. `docs/SHIP_PLAN.md` was rebuilt around it (new Lane S for the strategic simulation; milestones re-based on the brief's own first vertical slice).

- **Code:** `ExpeditionState::validate` now accepts a party of five (was four). Tests still 110/110 + 1/1 after the change. The strategic layer (factions, buildings, ownership, directives, forces, weather, corruption, journal) does **not** exist yet in Rust, content or Godot — see Ship Plan Lane S.
- **Docs reconciled in this pass:** `GAME_BUILD_PLAN.md` C2 roster and E2 exit test (four women; Betty and Ayla established; two open), `§4.1` party cap; the bible's `req.scope.cast.core` and `req.scope.campaign.duration` marked SUPERSEDED, protagonist = Captain Michael, a current-authority callout at its top; `THREE_D_PRODUCTION_PLAN.md` and `BETTY_3D_ASSET_CONTRACT.md` say rigging is a generator switch and clips wait for the animation tool; `RUNBOOK.md` no longer says "commissioning". The design bible `.docx` in `outputs/` is **stale** until regenerated (needs `python-docx`; Ship Plan H8).
- **Not decided (brief §20 / Ship Plan §4):** the other two women; battle presentation (side-view theatre as the encounter lens is the Provisional recommendation); board-contract ownership between this repository and dr-companion; merge authority; Michael's exact history.

## 2026-09-05 and 2026-09-06

Three parallel rounds on `backend/b0-expedition-state` (PR #3), each lane in
its own worktree with a `.agents/claims/` manifest, integrated only after the
four proofs were re-run on the trunk. The live ledger is `docs/SHIP_PLAN.md`;
this entry records what was *verified*, not what was planned.

- **CI exists and is trusted.** `.github/workflows/verify.yml` runs two jobs:
  Rust and content (fmt, tests, `--features godot-ext`, validator, bundle
  freshness) and the Godot headless gate (editor import, main scene, three
  review scenes, seventeen suites) against the real GDExtension, with an
  anti-mock step that fails unless three live-bridge lines are present. Both
  are green on trunk `205fec2`. **Two ways the gate could pass falsely were
  found by reading green logs and closed:** the anti-mock proof (E2), and on
  2026-09-06 a suite-shape hole — fifteen suites ended in an unconditional
  `quit(0)` that erased an earlier `quit(1)` — closed at two levels: both
  gate scripts now fail any step that prints an `ERROR:` line, and the seven
  suites with that shape count failures. The same day the migration suite was
  found to take its expected keys from the serializer under test; it is a
  literal list now (E6).
- **The slice holds through the engine, not only in Rust.** The port had
  forwarded only a portal's id, endpoints and travel mode, so C2's authored
  road costs and any gate never reached Godot. Found when A4's authored tidal
  cut turned the Godot job red; fixed under C13 — costs, gates, cells and
  anchors forwarded, `use_anchor` on the bridge, the prototype and campaign
  suites salvaging the wreck before they walk, as the Rust slice does.
  `godot-rust/tests/authored_world.rs` performs the port's translation field
  for field and drives the prototype's opening on it.
- **Character simulation (M1 7/12):** the supply loop closes (A3: anchors
  produce, rations bite, victory pays; loot values owned by
  `content/loot/`, held equal by a test); the estate's rooms are anchor
  actions and `rest_at_estate` is gone (A4); five named bands and Composure
  with the Shaken gate (A5; bond ranks owned by content, held equal); Captain
  Michael is a battle actor with Weapon Attack, Guard and Reposition (A6; his
  display name owned by `content/characters/captain.json`); Hold Position is
  the universal Guard verb and the architecture doc says so (A8).
- **Strategic simulation (M3 3/16), which did not exist before 2026-09-06:**
  `godot-rust/src/strategy/` — `FactionDefinition`/`FactionState` with a
  closed six-key concept enum and no field a proper name could live in (S1);
  ownership and influence on the graph with `effective_risk` as the single
  owner of a road's live danger, contested = authored base + 2 (S2);
  `RecruitmentState` moved only by authored milestones, never surfaced as
  numbers — structurally, `Disposition` has no accessor outside its file
  (S12). Pursuit respects gates: a hunter never takes a door the party has
  not opened (found by A4, fixed on merge).
- **Bridge and screen:** the full legal-command list, `inspect`,
  `resolve_midnight`, `use_anchor`; the screen draws anchor, inspect and
  midnight controls only from the legal list, so a spent anchor or a locked
  door is never a button (B3, B4). `inspect` records one observation, not the
  whole cell. Cells, anchors and portal costs forwarded (B2, inside C13).
- `cargo test --manifest-path godot-rust/Cargo.toml`: **183 unit + 3
  authored-world + 5 save-migration + 1 integration pass**. `cargo check
  --features godot-ext`: passes, no warnings. `cargo fmt -- --check`: clean.
- `node tools/src/validate.mjs`: passes — **194 stable IDs** (178 → 194: ten
  observation IDs, five anchors and one discovery it previously could not
  see), 9 skills, 47 presentation cues, 15 video reels, 6 creature plates, 5
  shared asset records, 33 source collections, 12 tracked placeholders.
- **Recurring defect class, three times in one day:** a lane mirrored
  authored content in Rust with nothing holding the two equal (loot yields,
  bond ranks, the Captain's name). Each is now content-owned with an
  equality test; the rule is in every lane brief. **Rows without cards** in
  the ledger were found and fixed four times (C1/C2/C4, E5, E6, B3/B4).
- **Not decided, unchanged (brief §20):** the other two women (O2); battle
  presentation (O1); resource categories; faction display names (C9 authors
  placeholders that the validator requires to say "needs decision").
- **In flight at the time of writing:** S4 (strategic tick and the
  determinism harness), C9 (faction records), B14 (control and risk through
  the bridge).

**Summary:** one spine, green through both CI jobs, the first-chapter slice
playable through the engine with real costs and gates, and the strategic
layer begun. Next: S4/S8 (the tick and the dual clocks), S5 (utility AI)
once S4 lands, and C6/C12 when O2 is decided.

### 2026-09-06, later — the strategic layer thinks

Three more strategic rounds on `backend/b0-expedition-state`, each lane in its
own worktree, integrated only after the four proofs re-ran on the trunk. Trunk
`777a943`, green on both CI jobs at every integration commit.

- **Factions decide (S5):** the five strategic states recomputed every hour
  from share of the island and pressure on the border, goals scored over the
  brief's considerations, raw scores never leaving the module. **Directives
  (S6):** the player's request is a high-weight input a faction weighs, never
  a command, explained in words before confirmation. **Forces (S7):**
  aggregate bodies walk the routes one real hop at a time and arrive with the
  composition they left with; materialisation waits on B11. **The tick (S4)**
  drives it all deterministically, with the draw sequence under the save
  hash; **the journal (S11)** keeps a bounded window plus a digest that loses
  nothing; **the dual clocks (S8)** never move each other, proven in the
  contract's own words. **Six factions exist as content (C9)** with a validator
  rule that refuses a real name until one is approved; **the bridge loads them
  (B15)** and the Rust harness runs the same island Godot runs.
- `cargo test --manifest-path godot-rust/Cargo.toml`: **260 unit + 5
  authored-world + 5 save-migration + 7 strategic-determinism + 1 integration
  pass.** `cargo check --features godot-ext`: passes. `cargo fmt -- --check`:
  clean. `node tools/src/validate.mjs`: 200 stable IDs. Bundle fresh.
- **Honest negatives recorded rather than papered over:** the authored faction
  records are neutral by rule, so a loaded registry scores as an empty one
  until content carries a real weight; every tuning number in S5, S6, S8 is a
  named constant marked `needs decision`.
- **Not decided, unchanged:** O1, O2, O3 (which now blocks B11 and therefore
  force materialisation), resource categories, faction names, doctrines (S15).

**Summary:** M1 7/12, M2 5/9, **M3 10/16**. Next: S3 (buildings, shapes
only), C6 (relationship scenes), B5 (Michael's card on the battle screen), E8
(claims enforced in CI).

