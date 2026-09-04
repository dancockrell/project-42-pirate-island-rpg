# Project 42: Autonomous Island Vertical Slice

This contract implements the first slice in [GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md). That design authority wins over the older side-view battle slice this file replaces.

## Slice promise

On one fixed-view isometric board, the player can witness the legible in-world consequences of factions acting autonomously, support a companion-led investigation, and see that choice change the same simulated island. RTS internals remain behind the scenes. The proof is deterministic, recoverable, and animation-free.

The player experience must read as an RTS with five embodied characters inside it: the board is broad and systemic, while every direct player action has a hero source, world position, knowledge basis, and legitimate scope of authority.

## Required content

- Two connected regions whose visible network contains a loop, a chokepoint, an alternate route, and at least five typed tethers. It must not form three fixed lanes.
- Michael, one fully authored female investigator, and three selectable rig-ready heroine stand-ins.
- Four ordinary factions—eastern/wushu fox people, colonial powers, pirates, and elves—with resource flow, build and production queues, supply, territory, units, and independent bilateral relationships.
- One hidden Cthulhu faction whose utility and unconventional build cycle advance its Day-100 plan.
- One standard-cube-volume structure per visible ordinary faction.
- Two generated adventure sites proving faction identity and structure level change deterministic defenses and loot.
- One encounter whose hostile and allied contributions both derive from current faction capacity rather than player level.
- One companion lead with two defensible interpretations and two support choices.
- One early wrongness effect with prerequisites, advance tell, consequence, aftermath clue, and accessibility substitute.
- One faction terrain influence that spreads across eligible adjacency and mechanically changes ground state.
- Two magical weather fronts with different faction affinities, including one early Cthulhu-aligned or necromantic front.

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
- The island graph permits branching, loops, cutoffs, and state-dependent alternate routes; tower-defense pressure is local behavior, not global topology.
- Ordinary RTS mechanics use established patterns rather than a bespoke narrative substitute.
- Utility traces reproduce for the same seed and separate personality weights from bounded wobble.
- Faction relationships exist independently of player reputation.
- A faction can be permanently eliminated; former holdings naturally become abandoned, captured, dismantled, contested, corrupted, or reclaimed.
- Faction, building archetype, level, and stable instance seed deterministically produce distinct site and loot profiles.
- Quests encountering an eliminated faction choose a validated successor, recovery, or closure path; they do not respawn it.
- Cthulhu may rationally lose conventional board value while advancing hidden plan state.
- Cthulhu's authored advantage event has prerequisites, changes real faction state, and exposes a diegetic consequence; it does not spawn an unexplained force.
- Hostile and friendly encounter forces change when their source factions' buildings, upgrades, supply, losses, or local positions change.
- Terrain conversion changes at least traversal, supply, encounter composition, and visible ground treatment from one authoritative influence state.
- Weather movement and effects reproduce for the same state and seed; changed magical or faction influence can change the next front.
- Every weather faction bonus or penalty is inspectable in debug traces and perceptible in-world without exposing hidden numeric state.
- The player never sees raw stockpiles, queues, utility traces, hidden objectives, or an exact heat value unless that information has been learned in-world.
- Every player command identifies the acting hero and passes position, knowledge, equipment, relationship, and authority checks; the camera does not grant omniscience.
- Every structure expands to a unique set of standard cubes; overlapping procedural placements fail atomically.
- Visual, collision, selection, props, spawn points, tether sockets, influence hooks, and state hooks remain inside reserved modules.
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
