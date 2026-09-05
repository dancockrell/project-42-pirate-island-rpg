# Pirate Island: Current Game Direction and Build Plan

Status: **current design authority, reaffirmed 5 September 2026**. This document describes the accepted target. It does not certify that the existing prototype implements it.

## The game being built

Pirate Island is a HaremLit multi-faction RTS simulation experienced through Captain Michael and four adult female companions. All five are controllable heroes inside the island world. The companions are investigators, leaders, and quest drivers with their own theories, requests, competencies, and relationships. Their quest lines reveal the island and drive campaign progression.

The world is presented on a fixed isometric board. The player directs the heroes and establishes strategic intent; autonomous factions carry out routine world activity. The game is not defined by an overhead unit-micromanagement interface or by the earlier side-view active-fighter/card presentation.

## Accepted system contracts

| System | Current contract |
| --- | --- |
| Heroes | Captain Michael plus four female companions are controllable participants. Companion agency must affect investigation, choices, faction opportunities, and campaign progress. |
| Factions | Multiple factions pursue goals, resources, territory, survival, and opportunities autonomously. Their relations with one another matter independently of their relation to the player. |
| Strategic behavior | Decisions respond to incentives, constraints, alliances, rival actions, and changes in the world. Game-theoretic behavior must produce understandable consequences rather than a fixed sequence of player-triggered encounters. |
| Dual clocks | World time and Cthulhu patience/heat are distinct state dimensions. Advancing time must not silently imply an identical heat increase. The exact rules and presentation must follow accepted design decisions and remain independently testable. |
| Board | One fixed isometric presentation makes places, connections, heroes, and faction changes readable. A layout or art asset cannot invent a legal connection or simulation outcome. |
| Art | Broad reusable fantasy art families support the island's different cultures, factions, terrain, and sites. Local visual references remain local; one port or ruin reference does not dictate the whole island. |
| Character production | Preserve rigs, rest poses, sockets, identity, scale, and provenance now. Animation is a later production stage; static readability and usable controls come first. |
| Story progression | Companion investigations and personal quest lines change knowledge, access, relationships, and campaign possibilities. Companions do not wait passively for Michael to discover everything. |

Normal pause and a calm, readable interface remain part of the accepted continuation brief. Exact faction names, founding sequence, clock tuning, and unapproved victory details must not be invented by implementation or promoted from provisional notes.

## One authoritative implementation

Keep the existing ownership split: Rust owns simulation state, commands, validation, saves, deterministic decisions, and typed outcomes; GDScript owns Godot presentation, input, and projection; TypeScript owns offline content validation and authoring transforms; authored records use stable IDs.

Extend or replace the existing implementation in place. Existing expedition, combat, world-clock, and save work is useful evidence and reusable implementation where compatible. A prototype's turn order, midnight law, camera, or party-card behavior does not by itself make that behavior a requirement of the new RTS design.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the existing technical boundary. Verify implementation and tests before reporting a feature as complete.

## Work already in progress

At this audit, [PR #3](https://github.com/dancockrell/project-42-pirate-island-rpg/pull/3) contains expedition/backend work and the longer [Pirate Island continuation brief](https://github.com/dancockrell/project-42-pirate-island-rpg/blob/84ddb4720e965fd920767a41f20e365927bf4986/docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md). That brief distinguishes accepted decisions from provisional proposals; this plan consolidates the current contract on the default branch. Its old instruction to locate a repository is historical context: this repository has now been inspected.

[PR #4](https://github.com/dancockrell/project-42-pirate-island-rpg/pull/4) contains the earlier systems handoff and [PR #2](https://github.com/dancockrell/project-42-pirate-island-rpg/pull/2) contains character/content work. These links identify preserved work, not merge or validation approval. Before integrating them, reconcile their README, build-plan, ship-plan, and handoff claims with this direction. Do not restore side-view promises simply to resolve a documentation conflict.

## Dependency order and proof

1. **Reconcile the simulation spine.** Identify the actual campaign owner, legal commands, persistence, and pending branch work. Prove that one save and event sequence owns the outcome across presentation changes.
2. **Prove the static board and five-hero control.** Show current locations, selection, legal interactions, and confirmed transitions with readable rigged stand-ins. Clearly mark prototype art.
3. **Prove autonomous faction behavior.** Exercise resource use, competing goals, and at least one relation between non-player factions that changes a visible world opportunity. Record the rules and a reproducible scenario.
4. **Prove the two clocks independently.** Save and restore both; exercise changes to one without assuming the other changed; trace consequences back to the relevant rule.
5. **Connect a companion investigation.** A companion raises a question, pursues evidence, and changes a campaign option. Persist the consequence and make the next action understandable.
6. **Admit art at gameplay distance.** Validate sources and license records, silhouettes, scale, sockets, and board readability before expanding content. Add animation only after the static interaction and simulation contracts hold.

These are acceptance milestones, not claims of completed systems. Detailed tuning and additional systems remain provisional until their design is accepted.

## Documentation to retain without reviving obsolete scope

- [Character authoring](CHARACTER_AND_HAREMLIT_AUTHORING.md) retains character depth and relationship guidance under the current companion-agency contract.
- [Shared asset platform](SHARED_ASSET_PLATFORM.md) and [integration protocol](SHARED_ASSET_INTEGRATION_PROTOCOL.md) retain provenance and consumer admission rules.
- The earlier [vertical slice](VERTICAL_SLICE_BUILD_CONTRACT.md), [3D production plan](THREE_D_PRODUCTION_PLAN.md), [visual authority](VISUAL_AUTHORITY_AND_3D_ENTRY_GATE.md), and [Betty asset contract](BETTY_3D_ASSET_CONTRACT.md) retain prototype and asset-review evidence. Their side-view staging and animation-first gates are superseded.
- [RUNBOOK.md](RUNBOOK.md) and [NATIVE_BRIDGE.md](NATIVE_BRIDGE.md) document existing tooling and prototype interfaces. They are not proof of an implemented RTS campaign.

The prior side-view build plan is preserved in Git history. Source assets, approved reference records, runtime code, and open PRs remain intact. This fantasy island does not inherit the alternate-WW2 lore of the separately retained Project 42 worldbuilding and World Aflame repositories merely because the names overlap.
