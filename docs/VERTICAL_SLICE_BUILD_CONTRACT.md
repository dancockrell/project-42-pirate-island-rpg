# Project 42: Autonomous Island Vertical Slice

This contract implements the first slice in [GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md). That design authority wins over the older side-view battle slice this file replaces.

## Slice promise

On one fixed-view isometric board, the player can witness the legible in-world consequences of factions acting autonomously, support a companion-led investigation, and see that choice change the same simulated island. RTS internals remain behind the scenes. The proof is deterministic, recoverable, and animation-free.

## Required content

- Two connected regions with visible nodes and at least three typed tethers.
- Michael, one fully authored female investigator, and three selectable rig-ready heroine stand-ins.
- Three ordinary factions with resource flow, build and production queues, supply, territory, units, and independent bilateral relationships.
- One hidden Cthulhu faction whose utility and unconventional build cycle advance its Day-100 plan.
- One footprint-valid structure per visible ordinary faction.
- Two generated adventure sites proving faction identity and structure level change deterministic defenses and loot.
- One companion lead with two defensible interpretations and two support choices.
- One early wrongness effect with prerequisites, advance tell, consequence, aftermath clue, and accessibility substitute.

## Demonstrable flow

1. Load a seeded world and inspect only hero-known territory, structures, relationships, and positions.
2. Advance at least three decision windows without issuing a command; factions collect resources, progress queues, deploy, and act.
3. Show those hidden systems through world evidence: a new structure, changed patrol, contested route, shortage, raid aftermath, or abandoned holding—not raw utility numbers.
4. A board event causes the companion to update a hypothesis and propose a lead.
5. The player assigns heroes, equipment, access, or political support to one interpretation.
6. The action consumes time and resolves through Rust into board, relationship, evidence, and heat-ledger changes.
7. The companion reports what her theory explained, what remains uncertain, and what she wants next.
8. Save, reload, and replay from the preceding boundary to reproduce the decision and outcome.

## Exit gates

- The board remains the sole spatial truth; no route screen duplicates it.
- Ordinary RTS mechanics use established patterns rather than a bespoke narrative substitute.
- Utility traces reproduce for the same seed and separate personality weights from bounded wobble.
- Faction relationships exist independently of player reputation.
- A faction can be permanently eliminated; former holdings naturally become abandoned, captured, dismantled, contested, corrupted, or reclaimed.
- Faction, building archetype, level, and stable instance seed deterministically produce distinct site and loot profiles.
- Quests encountering an eliminated faction choose a validated successor, recovery, or closure path; they do not respawn it.
- Cthulhu may rationally lose conventional board value while advancing hidden plan state.
- The player never sees raw stockpiles, queues, utility traces, hidden objectives, or an exact heat value unless that information has been learned in-world.
- Structure footprints, spawn points, tether sockets, influence hooks, and selection bounds validate.
- Rebuilding the same unchanged site reproduces its layout and loot seed; changing faction, archetype, or level produces a distinct content profile.
- The companion, not Michael, authors the lead and interpretations.
- A baseline policy remains on pace for minimum viable confrontation preparation.
- Godot verification, Rust tests, content validation, and deterministic bundle regeneration pass.

## Explicit non-goals

- animation;
- final art;
- a novel RTS algorithm;
- the complete faction roster;
- all four companion quest lines;
- the final Cthulhu encounter;
- a broad rewrite of proven command/event combat code.

The old battle prototype remains a test bed for existing combat and presentation boundaries. It is not the product slice and receives no new side-view-specific content under this contract.
