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
| `campaign/` | 1 | **no** | Three endgame triggers; Day 100; hidden heat (M3) |
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

### In flight

1. **Resources.** The faction economy becomes authored: today `from_scenario`
   gives every faction the same invented income, cap and starting stock, so a
   second scenario cannot have a different economy. Adds a catalogue and refuses
   an undeclared resource key instead of creating one on a typo.
2. **Items.** The first inventory the simulation owns, with a closed effect set,
   surviving recruitment and save. Closes a real dangle: the captain's signature
   weapon is referenced by his character record and defined nowhere.

### Next, in order

3. **Triggers.** Deterministic condition→effect rules at one point in the tick,
   fired at most once, recorded in the save. The owner's decision of 9 September:
   a trigger changes the world, it does not narrate it. No journal, no feed.
4. **Quests.** Stage machines whose transitions are triggers, their stage in the
   snapshot so Godot shows state it does not invent.
5. **The campaign clock.** `content/campaign/cthulhu_clocks.json` exists and is
   read by nothing. Day 100 as a real deadline, heat as an irreversible event
   ledger (IDs and severity, never a number the player sees), and the three
   confrontation triggers with `firstTriggerWins` arbitration recording an
   immutable cause. This is milestone M3 and it is entirely unbuilt.
6. **The island network.** `content/world/island_network.prototype.json`
   declares six nodes and eight tethers with a loop, a hub and a chokepoint; the
   simulation has a flat walkable grid and named destinations. Give the world the
   node/tether graph the authority requires, with tether state (blocked, hidden,
   conditional) changing reachability without moving geography. Regions become
   expressible, which is what location triggers were deferred for.
7. **Weather.** Two authored fronts with selection weights, movement across
   tethers, faction modifiers, terrain interactions and advance tells. Weather
   moves on the network from item 6 and modifies production, movement and sight
   through the same rules factions already use.
8. **Terrain influence.** Layered fields over cells sourced by structures,
   units and rituals, competing deterministically, changing movement cost, build
   legality and encounter grammar. Depends on the network.
9. **Companions and leads.** The heart of the product and the largest piece:
   Betty and Ayla exist as authored characters with skills and are not in the
   game. Put a heroine on the island as a recruitable, directable hero with her
   own competence domain; give her a lead that reads simulated evidence, offers
   two interpretations and asks for support; let the player's answer change board
   state, relationships and proof in one transaction. Milestone M2, and M4 begins
   here.
10. **Adventure sites.** A faction holding becomes enterable, its layout and
    defenders derived from the recorded composite seed the site records already
    describe, so a level-five cult shrine differs from a level-two fort.
11. **Battle, reconnected.** `battle.rs` compiles, has 26 tests, and is wired to
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
