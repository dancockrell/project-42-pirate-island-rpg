# Build-out plan: from the island that runs to the game the authority describes

**Status:** working plan, 9 September 2026. Subordinate to
[GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md) (the product authority),
[VISUAL_AUTHORITY.md](VISUAL_AUTHORITY.md) (art) and
[ARCHITECTURE.md](ARCHITECTURE.md). Where they disagree with this, they win.
This document is a route, not a new authority: it says what is missing, in what
order, and what proves each piece. Art is excluded by the owner's instruction —
sprites are theirs; everything else is ours.

## The gap, measured

The simulation is a real RTS: five factions, income, production queues,
dispatch by bounded utility, diplomacy, elimination, madness, repairs, salvage,
expansion, Michael's foothold and machinery, save and load, 93 tests, no
`rand` and no wall clock. The island runs and can be played.

But the repository has authored content for systems the simulation never reads.
Counting `content/` against the code that loads it:

| Domain | Files | Read by Rust | The authority section it serves |
|---|---|---|---|
| `campaign/` | 1 | **yes** | Three endgame triggers; Day 100; hidden heat (M3) |
| `world/` (island network) | 3 | **no** | One island, one board; complex network, not three lanes |
| `weather/` | 2 | **no** | Magical weather system |
| `sites/` | 5 | **wrong shape** | Faction structures are also adventure sites |
| `factions/` | 5 | **partly** | RTS faction contract: doctrine, elimination policy |
| `ai_scenarios/` | 5 | **no** | Replayable, explainable faction decisions |
| `characters/`, `skills/`, `encounters/` | 10 | **no** | Companions drive investigation (M2, M4) |

**Two rows corrected after being checked, not assumed, against the owner's
confirmed direction (2.5D, not 3D) and against the real dispatch code.**
`content/sites/*.json`'s `footprint` is a 3D module system —
`normalizedPosition`, volumetric bounds, `tetherSocketIds` — for a spatial
model this game does not have and, per the owner, will not: the board is 2.5D
pixel art on the flat `IslandPoint{x,y}` grid `world.rs` actually runs. The
site records' non-spatial fields (`production.spawnRuleIds`, `loot`,
`generation.seedInputs`) describe something real and still wanted; the
`footprint`/`hooks` block does not and needs re-authoring for a 2D scene, not
a 3D one, before anything reads it. `content/factions/*.json` is not uniformly
unread: every faction's `doctrine.dispatchScoreTerms` names the real
`DispatchScore` struct's nine fields exactly (plus one faction-specific extra
— `trade_opportunity`, `ritual_opportunity`, `restoration_opportunity` — that
has no field of its own yet), so that part is genuinely grounded. `doctrine.goal`,
`elimination.*` and `buildCycle.*` are still unread, though several already
have a bespoke, non-data-driven equivalent in `world.rs` (Cthulhu's undead
recovery via midnight return and salvage-paid restoration is, in substance,
an elimination recovery condition; it just isn't expressed as this record's
`recoveryConditions` field for any faction generically). One more finding from
the same pass: `ai_scenarios/*.json` fixtures target the same ungrounded
`network_node.*` names the reverted island-network commit adopted — those
IDs were never validated against real geography either, and nothing resolves
`targetNodeId` today, so the dangling reference was silent before and remains
silent now.

Seven domains, thirty-two records. The pattern is one thing, not seven: **the
island simulates its factions but not its campaign.** There is no clock counting
toward anything, no weather, no network beneath the walkable grid, no site to
enter, and — most consequential — **no companion investigates anything.**
Correcting a claim this document previously made without checking: generic
recruitable women already work — production spawns them, `talk`/`recruit`
work, a save round-trips a four-slot party — proven by a passing suite
(`island_scene_test.gd`'s recruitment test). What does not exist is Betty, the
one authored companion this repository actually has: `content/characters/betty.json`
and her seven skills are real, but every reference to her (`battle.rs`) is in
the side-view battle prototype the authority's provisional list marks
superseded, so she is in no `personas.json` pool and cannot be produced on the
island at all. "Ayla" is not a second authored companion — the name occurs
only as a bare, contentless actor ID inside that same dead prototype's test
fixtures, and nothing under `content/characters/` or anywhere else in this
repository defines her. The authority's third non-negotiable contract is that
companions drive every required mystery chain; today the game has generic
recruitment but no investigation, no lead, and no Betty on the board.

## The route

Each numbered item is one contract, one claim, and a pack-authored system: the
data lives in `content/scenarios/<id>/` per [SCENARIO_PACKS.md](SCENARIO_PACKS.md),
the rules live in `FactionWorld`, and a validator rule proves the two agree.
Order is by dependency, not by appetite.

### Landed

- **Scenario packs.** The island is data the simulation loads at runtime;
  `from_scenario` replaced the hard-coded island, proven equal at tick 0 and
  1,440.
- **Portable gate and CI.** The Godot gate and native build run on Linux and
  macOS; two jobs verify every pull request; a third builds the Windows library
  so a deleted verb cannot silently break the owner's platform.
- **The campaign clock.** `content/campaign/cthulhu_clocks.json` is read.
  Day 100 is a real deadline, heat is an irreversible ledger written by the
  island's own occurrences on the record's declared channels, and the three
  confrontation causes are arbitrated once at the settled tail of the tick with
  the authored first-wins priority. Milestone M3.
- **Resources.** The faction economy is authored: a catalogue on `ScenarioRules`
  replaces the invented income, cap and starting stock `from_scenario` used to
  fabricate, and every gain and spend goes through a checked pair that refuses
  an undeclared key instead of creating one on a typo.
- **Items.** The first inventory the simulation owns, with a closed effect set,
  surviving recruitment and save. Closed the real dangle it found: the
  captain's signature weapon was referenced by his character record and
  defined nowhere.
- **Triggers.** Deterministic condition→effect rules at one settled point in
  the tick, immediately before the campaign clock so an effect is visible to
  that tick's arbitration; a trigger is a transaction (every effect applies or
  none does), fires at most once unless it repeats, and is recorded in the
  save. `grant_item` gets its first declared source. The owner's decision of
  9 September holds: a trigger changes the world, it does not narrate it — no
  journal, no feed, only the `flags` a trigger has set, read like any other
  board fact.
- **Quests.** Stage machines whose transitions are triggers: `set_quest_stage`,
  one closed trigger effect, is the only thing that moves one, so completion
  and failure are effects exactly as the contract said, not a second mechanism.
  Every declared quest starts active on its own authored stage; a terminal
  stage locks a quest there; the snapshot carries each quest's current stage
  and objective text, so Godot shows state it does not invent. Every reserved
  key from schema version 1 (`resources`, `items`, `triggers`, `quests`) is now
  implemented.
- **Recruitment gets its first campaign consequence.** `loyal_companion_count`,
  a new trigger condition, reads `person.loyal_to_michael` — the field
  `recruit_island_person` already writes — so a pack can react to Michael no
  longer being alone. The main pack's `quest.not_alone_anymore` does exactly
  that: a small provisions grant and a flag once anyone is recruited, proven
  end to end through the real production, approach, talk and recruit path, not
  a synthetic flag flip. This is not the companion-and-lead system the
  authority describes — it is the smallest real step toward it, deliberately
  small after two mistakes this session (the reverted island network; a false
  claim about a nonexistent "Ayla") from building or documenting past what was
  actually verified.
- **A full household gets a second consequence.** `party_size`, a second new
  condition in the same shape, reads how many of the four `party` slots
  `assign_island_companion` has filled. The main pack's
  `quest.the_household_forms` resolves once all four are, proven the same
  way: four separate produced women, actually recruited and actually
  assigned, not four flags set by hand. Still not the lead system the
  authority describes; still no Betty, no geography, no invented identity.
- **A rival faction falling gets its first consequence.** `faction_eliminated`
  was already implemented and validator-checked — the elimination path itself
  predates this session — but unused by the main pack's triggers. The new
  `quest.the_island_grows_quiet` reacts to `faction.colonial_powers.prototype`
  falling with a small provisions grant and a flag, proven through the same
  siege path `siege_destroys_last_producer_and_elimination_survives_load`
  already exercises, not a synthetic `eliminated_factions.insert()`. This
  found something worth recording: the colonials already fall to the pirates
  within the unmanipulated main scenario's first day (tick 1440) with no
  player action at all, which the equality-proof fixture now reflects — this
  is the first trigger whose condition can go true from ordinary autonomous
  play rather than only from something the player did.

### Next, in order

1. **The island network.** `content/world/island_network.prototype.json`
   declares six nodes and eight tethers with a loop, a hub and a chokepoint; the
   simulation has a flat walkable grid and named destinations. Give the world the
   node/tether graph the authority requires, with tether state (blocked, hidden,
   conditional) changing reachability without moving geography. Regions become
   expressible, which is what location triggers were deferred for.
2. **Weather.** Two authored fronts with selection weights, movement across
   tethers, faction modifiers, terrain interactions and advance tells. Weather
   moves on the network from item 1 and modifies production, movement and sight
   through the same rules factions already use.
3. **Terrain influence.** Layered fields over cells sourced by structures,
   units and rituals, competing deterministically, changing movement cost, build
   legality and encounter grammar. Depends on the network.
4. **Companions and leads.** The heart of the product and the largest piece:
   Betty (`content/characters/betty.json`, seven real skills) is authored and
   reachable only from the dead battle.rs prototype, not producible on the
   island; the generic recruitment path already there gives a woman no
   competence domain, lead, or investigation. Put Betty on the island as a
   recruitable, directable hero with her own competence domain; give her a
   lead that reads simulated evidence, offers two interpretations and asks
   for support; let the player's answer change board state, relationships and
   proof in one transaction. Her seven skills need a home in the *island*
   simulation, not the side-view battle prototype the authority's provisional
   list marks superseded — a decision, addressed by item 6 below, this item
   depends on. Milestone M2, and M4 begins here.
5. **Adventure sites.** A faction holding becomes enterable, its layout and
    defenders derived from the recorded composite seed the site records already
    describe, so a level-five cult shrine differs from a level-two fort.
    **Confirmed by the owner: 2.5D, not 3D.** `content/sites/*.json`'s
    `footprint` (modules, `normalizedPosition`, `tetherSocketIds`) is a 3D
    system this game will not use; an entered site needs a 2D scene on the
    same sprite and grid conventions the island board already uses, so this
    contract re-authors `footprint`/`hooks` for that shape before any of it is
    read. The record's non-spatial fields — `production.spawnRuleIds`,
    `loot`, the seed inputs — describe something real and carry over.
6. **Battle, reconnected.** `battle.rs` compiles, has 26 tests, and is wired to
    nothing: the island tick never calls it. Either the board resolves combat
    where it stands or a bounded tactical view opens; the authority marks that
    choice provisional, so it needs a decision before it needs code.

### Deliberately not here

Art of any kind, per the owner. The sprite animation consumer, the visual
acceptance slice, and every asset remain the owner's to supply; the code that
consumes them is written when they exist.

## How each contract is proven

Unchanged from the scenario-pack work, because it has been catching real
defects: the full proof chain on the committed tree (`cargo fmt --check`,
`cargo test`, `cargo check --features godot-ext`, the validator, the bundle
staleness check, the sprite manifest tests, the headless Godot gate), a bite
that shows the new rule failing by name before it passes, CI green, and a claim
recording every command with its result — including the ones that failed.

Two invariants hold across all of it. **The main scenario keeps producing the
island it produces today**, so a change to how the game is described never
silently changes the game. And **nothing enters a pack that the simulation
cannot run**: each reserved key stays refused until its contract lands.
