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

### 2026-09-06, evening — the round that made the ledgers a gate

Four lanes, integrated the established way. Trunk `00e3e28`.

- **S3 buildings:** §19's definition in full, instances in their own
  namespace, overlap refused before mutation, Michael's buildings never
  produce people. The three §20 numbers stay Open *in the data*: named
  constants marked `needs decision`, and `capture_rules` / `ruin_state`
  default to a `NeedsDecision` that is refused at load by a message naming
  the section — the decision lands on each authored record.
- **C6 relationship scenes:** five fade-to-black scenes for the two women who
  exist, each granting a milestone whose rule *is* S12's `MilestoneRule`;
  three rest on a trust floor and the validator refuses the directory if none
  does. The adult pack overrides this seam (C8/B10/E9); the base tree may
  carry nothing but `fade_to_black`, proven to bite. Stable IDs 200 → 210.
- **B5 battle screen:** Michael in the debug battle; the rail grouped by the
  bridge's band names, Composure as pips, Shaken marked; his grid unfolds on
  his turn through the same port path as Betty's. Green through the hardened
  gate on the trunk's own run.
- **E8 ledgers as a gate:** `tools/src/check-claims.mjs` runs in CI. On
  arrival it corrected 94 check verdicts across 25 claims, 22 rows without
  cards, four cards without a status, and one row citing a commit from the
  other repository — every one as a record, none by loosening a check.
- `cargo test`: **271 unit + 5 + 5 + 7 + 1 pass.** `--features godot-ext`,
  `fmt --check`, validator (210 IDs), bundle freshness and
  `check-claims` all green. Milestones **M1 7/12, M2 6/9, M3 11/16.**
- **Honest gaps:** Composure redraws only on a fresh snapshot (nothing spends
  it yet); `skill.system.hold_position` has no content record; Ayla has no
  character record (C3 ← A7/H5) and C6 carries a one-line named exception for
  her; force materialisation waits on B11 ← O3.

**Summary:** the strategic layer has buildings, the romance has beats, the
battle screen shows the whole party, and both ledgers are machine-checked.
Next: S9 (dungeon context), S10 (elimination), S13 (Michael's machines),
C10+C8 (building records and the pack manifest), A10 (bond rank as a fact of
the relationship).


## 2026-09-06, night — the strategic layer closes, one row short

Trunk `backend/b0-expedition-state` at the commit this entry ships in. Five
lanes ran in parallel worktrees off `e96e7cb` and were merged serially, each
proven on the trunk after its merge.

- **C10 + C8:** three building records (Michael's machine shop, a coast watch
  post for three ordinary factions, a ritual anchor that can only be
  destroyed), every Open number Open in the data; a presentation pack fixture
  whose overrides resolve only inside the pack. Stable IDs 210 → 214, bundle
  44 → 47 records.
- **S9:** a dungeon is a signature over its context — owner, tier, banded
  corruption, heat and epoch — and rewards draw down a building's stored
  value. Nothing refills that value yet.
- **S10:** brief §16's nine recovery links in order; a faction is eliminated
  only once it has held something and lost every link; no respawn; Cthulhu
  cannot be removed while §20's rules are Open. The 2,400-hour harness is
  untouched.
- **S13:** `MachineFamily` closed at the brief's eight; `produce_machine`
  deducts the cost the rule names and refuses a person on a Michael kit.
- **A10:** a woman's higher-ranked commands open as her arc advances;
  `bond_ranks` has one owner and moves only upward, only by an authored scene.
- **One defect found at a merge and fixed there.** S13 turned the machine
  output into a struct carrying a family; C10's record and S10's fixture
  still spoke the bare string. The record now names `mechanical_dog`, the
  validator refuses the bare string exactly as serde does (two refusals
  proven to bite), and the fixture carries the default family.
- `cargo test`: **303 unit + 5 + 5 + 7 + 1 pass.** `--features godot-ext`,
  `fmt --check`, validator (214 IDs), bundle freshness and `check-claims`
  (40 claims, 86 rows, 80 cards) all green on the trunk. Milestones
  **M1 8/12, M2 6/9, M3 15/16.**
- **Honest gaps:** S14 (the founding sequence) is M3's last row and waits on
  decision O5; the elimination sweep runs with an empty building registry
  until the real one is threaded through the tick (it can only delay an
  elimination); no `Battle` is yet built from `ExpeditionState`, so bond
  ranks reach a fight through the fixture only; `content/machines/` does not
  exist; nothing calls `produce_machine` on a timer.

**Summary:** every strategic card that did not need a human decision is
shipped. What remains in M3 is a decision, not code.

## 2026-09-06, small hours — the game can be saved, paused, packed and continued

Trunk `backend/b0-expedition-state` at the commit this entry ships in. Seven
lanes ran in parallel worktrees off `2f51e77`, each merged serially and
proven on the trunk; CI green on every integration commit.

- **B17:** a battle armed by the campaign carries the campaign's bond ranks;
  the debug battle keeps the review fixture because that bridge holds no
  campaign; no setter was added — an authored beat is the only thing that
  moves a bond.
- **B6:** the four tomb cells are content, read from the fixture verbatim;
  the equality test now holds the whole graph and bites from both sides.
  Stable IDs 214 → 223.
- **E7:** a crash log on the player's own disk keyed by in-world day and
  segment; the no-telemetry rule is a test that scans every script.
- **B16:** the building registry reaches the tick and the elimination
  sweep through the bridge, B15's shape repeated; the honest bite is cell
  capacity, not the core link, and the card says so.
- **B9:** pause is one Godot fact with a set of reasons; the midnight call
  refuses while paused; settings persist but are read by no scene yet.
  Proven to bite in CI itself.
- **B10:** packs merge by scene ID in load order; the `.pck` branch waits
  on E9 to build one.
- **B8:** save slots and continue through two appended bridge functions;
  newest is the save's own clock; a broken slot is listed as broken.
- **Two defects found at merges and fixed there:** B8's `load_json` called
  the state dictionary without B16's registry parameter (caught by
  `cargo check --features godot-ext` on the trunk); and the M2 roll-up had
  counted E8, which sits outside the bracket — corrected to 8/9 with E5 open.
- `cargo test`: **306 unit + 6 + 5 + 8 + 1 pass.** Validator 223 IDs, bundle
  51 records fresh, `check-claims` 47 claims / 88 rows / 82 cards.
  Milestones **M1 8/12, M2 8/9, M3 15/16.**
- **Honest gaps:** E5 (nightly artifacts) is M2's last row; S14 waits on O5;
  the service passage's habitat eligibility has no content expression; the
  three accessibility settings are read by nothing; `content/machines/` does
  not exist.

**Summary:** a stranger could now start, pause, save, quit, continue and
get the same legal actions — through the real bridge, proven in CI.

## 2026-09-06, afternoon — the game gets a face

Integrated in order, each with the full proof chain and the headless gate
green on the trunk before its ledger row moved: **P8** (the shell: title,
pause, slots, one `SceneFlow`), **P5** (one Theme, the UI grammar, the calm
information surface, the `strategic_surface` bridge verb), **B19** (machines
through the bridge), **S17** (goals become acts), **E11** (a universal macOS
export with the simulation inside, proven by a nightly run), **P2** (Forward+
with a declared fallback, one island environment, the isometric rig, the
material library), **P4** (the isometric board: world / route / room), **P6**
(the battle reads as a fight). Rows B11, B12, B13 and D11 are marked shipped
by the P cards that delivered them.

- **Three integration defects found on the trunk and fixed there.** P5's
  exhaustive journal prose refused S16's two production events, then S17's
  three; both given words and a level at the bridge rather than an
  `_ => ""`. P4 typed its clay as `StandardMaterial3D` and P2 had already
  made the factory return the library's `ShaderMaterial`; the board now
  wears P2's clay as its own header promised.
- **The gate's timed suites were racing engine start-up.** The first process
  frame's delta carries initialisation (0.133 s with today's autoloads) and
  the tree steps timers before tweens, so three suites that waited on a
  0.12 s wall-clock timer for a 0.10 s tween checked before the motion took
  one step. `paper_razorbeak_rig_test` failed by name on trunk (CI runs 274
  and 279; B19 reproduced it on `a1869b8`), and
  `placeholder_action_presenter_test` halted on a bare `assert()` and hung
  a ten-minute run. All three now await the tween they test; no assertion
  weakened. The gate gained a per-step ceiling (`GODOT_STEP_TIMEOUT_SECONDS`,
  default 240) so a halted suite fails in minutes and says why; proven to
  bite at a 1 s ceiling. The same class of defect as P9's synth in
  `_ready`: `InformationSurface` built the whole Theme in `_ready` and named
  two `class_name` globals from an autoload; both fixed the way P9 was.
- **Looked at, not only committed:** the title (a seeded shader sky, a
  cutter on a warm sunline), the terrace at dusk through P2's environment
  and P3's atmosphere, P4's three board distances, P6's opening and hit
  frames. The battle now reads as a fight: grounded rigs with contact
  shadows, effects built from their registry records, a camera beat, damage
  weight, a legible command dock. The board is a clean clay blockout with
  the party as a miniature and a marching force on a tether.
- `cargo test` **364 + 7 + 5 + 11 + 1**, validator 518 IDs, bundle 233
  records fresh, `check-claims` 67 claims / 105 rows / 99 cards, gate
  4 review scenes / 28 suites in about 25 s.
- **Honest gaps, all recorded on their cards:** S17's M3 100-day
  elimination is `failed` because nothing writes `ownership` from a tick
  (B11/O3 materialisation); no bridge verb raises or dispatches a force, so
  a live campaign's `forces` array is empty and P4 drives its proof through
  a save round trip; Ayla's kit cannot be submitted in the debug battle
  (bond rank D); `spend_composure` has no caller so no fight produces
  Shaken; the battle and the shell still restate the palette until they
  adopt the Theme; the 2D route board still owns the expedition screen and
  P4's board waits for the card that retires it; the engine has no
  attribution record and the credits say so.

**Summary:** a stranger can now open the title, start, walk the island on a
lit board, fight with effects and a camera, pause, save and continue, and
the island's factions build, gather and march while they do it.

## 2026-09-06, late afternoon — five lanes in one round, and the island fights back

Integrated with the full chain and the gate green on trunk before each row
moved: **E12** (22 third-party components in a ledger the credits read;
eight notices honestly pending), **P12** (procedural building and machine
kits from the C10 and C14 records; every metre a named `needs decision`),
**S18** (an arriving force takes unheld or undefended ground; `raise_force`
and `dispatch_force` through the bridge; the M3 100-day elimination now
passes for a faction that does not act), **P10** (the expedition screen is
the isometric board, P7's controls on it, the 2D route board deleted,
`BoardPalette` with no hex), **P11** (one palette owner adopted by the
battle, the shell and the settings; a scan of 71 scripts finds zero
unmarked colour literals; high contrast and 1.3× text reach the title's
weather and the battle's dock).

- **Four cross-lane seams found at merge and fixed on trunk:** P12's kits
  read P4's palette constants that P10 had replaced with Theme reads; a
  freshly dispatched column (S18: marches nothing until an hour runs)
  stood on the party's miniature, so a waiting column now stands at the
  road mouth; E12's credits lines used the label signature P11 retired;
  P11's scan read P12's iron, timber and canvas as UI colours, so they
  carry the game-colour marker with their reasons, and the scan now covers
  `board/` too.
- **Rows D9, D10, B11, B12, B13 and D11 are closed by the P cards that
  delivered them.** The gate is 5 review scenes / 30 suites in about 30 s.
- **Honest gaps, all on their cards:** an autonomous faction still cannot
  be eliminated because nothing lowers a stockpile (S19 next); the board
  shows no building or machine yet (P13 next); a roster card's status line
  overflows and the diamond labels truncate at 1.3× (P14 next); eight
  licence notices are pending (E13 next); C11's room contracts are
  unauthored; Ayla's colours and kit submission stay Open; `spend_composure`
  has no caller and the brief gives no rule for one.

**Summary:** the game opens on a title, walks a lit board with the party
selected and a column marching, fights with effects, and factions take
ground from each other while the player reads about it in one calm
surface.

## 2026-09-06, evening — the island spends, builds and shows it

Integrated after a container restart, each with the full chain and the gate
green on trunk: **E13** (seven of eight licence notices read from upstream
files, Mesa alone pending behind the proxy), **S19** (machines drink the
fuel and water their records name, construction runs on the clock, and an
autonomous faction can now be eliminated — the M3 second claim flipped),
**P13** (building and machine instances cross the bridge and P12's kits
stand on the board where the simulation put them), **P14** (the battle's
cards and dock are measured from the type scale; nothing clips at 1.3×),
**C11** (brief §3's room contract on all nine cells, every value sourced
from what the repository already says, twenty-one validator bites).

- The owner pushed a CI change directly to trunk mid-round (one run per
  pull request, in-progress runs cancelled by a newer push, 7-day
  artifacts); fast-forwarded under the lanes without conflict.
- Gate: 5 review scenes / 32 suites. Ledger: 78 claims, 114 rows, 108 cards.
- **Honest gaps:** nothing on the authored island produces fuel or water,
  so every authored machine starves the hour after it is built; no
  authored campaign raises a building or machine through the live bridge
  inside 60 days, so P13's proof drives instances through a save round
  trip and says so; five non-tomb rooms have no owning faction so their
  building slots are null; Mesa's notice; Ayla's colours and kit; a
  Composure rule the brief does not give.

**Summary:** the simulation now closes its own loop — take ground, build,
feed or starve, fall — and the board shows the buildings and machines it
produces.
