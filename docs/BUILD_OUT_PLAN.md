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
| `sites/` | 5 | **no** | Faction structures are also adventure sites |
| `factions/` | 5 | **no** | RTS faction contract: doctrine, elimination policy |
| `ai_scenarios/` | 5 | **no** | Replayable, explainable faction decisions |
| `characters/`, `skills/`, `encounters/` | 10 | **no** | Companions drive investigation (M2, M4) |

Seven domains, thirty-two records. The pattern is one thing, not seven: **the
island simulates its factions but not its campaign.** There is no clock counting
toward anything, no weather, no network beneath the walkable grid, no site to
enter, and — most consequential — **not one of the four women is on the island.**
The authority's third non-negotiable contract is that companions drive every
required mystery chain. Today the game has no mystery and no companion.

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
   Betty and Ayla exist as authored characters with skills and are not in the
   game. Put a heroine on the island as a recruitable, directable hero with her
   own competence domain; give her a lead that reads simulated evidence, offers
   two interpretations and asks for support; let the player's answer change board
   state, relationships and proof in one transaction. Milestone M2, and M4 begins
   here.
5. **Adventure sites.** A faction holding becomes enterable, its layout and
    defenders derived from the recorded composite seed the site records already
    describe, so a level-five cult shrine differs from a level-two fort.
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
