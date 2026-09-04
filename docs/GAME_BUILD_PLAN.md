# Project 42: Pirate Island — Design Authority

**Status:** Accepted direction, 4 September 2026
**Authority:** This is the canonical product and production contract. If another document, prototype, scene, or asset conflicts with it, this document wins until an explicit recorded decision changes it.

## Product promise

Project 42 is a HaremLit adventure played on top of a living, multi-faction RTS simulation of a mysterious island. The player commands Michael and four adult female companions as five hero characters on one large, fixed-view isometric board. The island continues to build, spawn, raid, bargain, spread, and decay whether or not the heroes intervene.

The four women are investigators and quest drivers, not followers waiting for Michael to discover the plot. Each brings observations, theories, questions, requests, competing interpretations, and a personal quest line. The player decides which leads receive time and resources, equips and protects the women pursuing them, resolves disagreements, and chooses how their discoveries change the campaign. Michael leads by judgment and support; he does not replace their agency.

The central mystery is escalating, rule-governed weirdness caused by Cthulhu's plan. Following companion leads and playing reasonably well must be barely but reliably sufficient. Optimization earns control, preparedness, optional truths, allies, and better outcomes; it is never an undeclared entrance fee for a viable ending.

## Non-negotiable contracts

1. **One island, one board.** Regions, nodes, typed tethers, structures, props, actors, territory, influence, resources, spawns, and faction control exist on and are represented by the fixed-view isometric board. Menus inspect or command that board; they do not replace it with a separate route-game truth.
2. **Five controllable heroes.** Michael plus four women are the complete core hero roster. All five can be selected, directed, equipped, protected, and supported.
3. **Companions drive investigation.** Every required mystery chain originates in or materially advances through a companion's authored initiative. Michael cannot receive every objective and perform all intellectual work himself.
4. **Factions are real RTS factions behind the scenes.** Each has an economy, build cycle, construction and production queues, structures, units, named heroes, territory, supply constraints, and autonomous strategic action. This machinery is not exposed as a raw RTS dashboard to the characters or player. Narrative scripting may create pressures and opportunities; it may not fake the underlying RTS activity.
5. **The world acts without the player.** Factions construct, spawn, expand, contest, raid, reinforce, ally, betray, and pursue asymmetric goals using simulation-owned state.
6. **Three endgame triggers.** The Cthulhu confrontation starts at the first of deliberate discovery, Day 100 culmination, or irreversible player-created heat reaching its terminal condition.
7. **No numeric heat UI.** World day may be known. Cthulhu's hidden heat/patience state is communicated through inferable diegetic evidence, never an exact meter.
8. **Wrongness has rules.** Weather, time, and causality distort according to authored thresholds and causal state. Effects have prerequisites, tells, consequences, and clue links; they are never arbitrary random horror.
9. **Simulation truth is singular.** Rust owns authoritative world and combat state. Godot projects it and submits commands. Content records define authored possibilities. Presentation never invents outcomes.
10. **Fixed isometric, animation-ready, not animated.** This phase adds no animation content. Actors and structures are rigged and socketed so later animation does not require an asset or schema redesign.
11. **Toybox clarity over historicism.** Western fantasy, Bronze-Age mythic mashup, and eastern/wushu fantasy mashup are intentional broad families. Specialized faction overlays make them legible at board scale.

## Campaign clocks and confrontation

### World day

World time advances autonomously through deterministic simulation ticks and day transitions. Day 100 is a hard dramatic deadline, not an instant-loss screen. On Day 100, Cthulhu's plan becomes overt and the confrontation begins using the state the island and heroes produced.

### Hidden heat/patience

Heat is an irreversible consequence ledger, not a decaying suspicion gauge. Specific player actions add declared heat events: disrupting a protected rite, repeatedly assaulting cult logistics, exploiting an anomaly, exposing forbidden knowledge publicly, or probing Cthulhu's infrastructure. The simulation stores event IDs and severity so internal tools and saves can explain exactly why a threshold was crossed.

The player never sees the number. Each band unlocks curated signals such as altered patrol doctrine, impossible rain, shared dreams, changing book marginalia, NPC memory disagreements, tide errors, or shortened shadows. The signals remain consistent enough to learn across playthroughs.

### Deliberate discovery

The investigation reaches Cthulhu only after the party assembles sufficient proof and chooses to act on it. Proof is distributed among the companions' specialties and relationships. No single mandatory clue may be permanently lost; essential proof has an alternate source or companion-driven recovery scene.

### Trigger arbitration

At each authoritative tick, Rust evaluates a committed deliberate-reach command with valid proof, world day at or beyond 100, and heat at or beyond its terminal threshold. The first recorded trigger becomes immutable `confrontationCause`. If multiple conditions become true in one transaction, priority is deliberate discovery, then heat, then Day 100, preserving the most player-authored cause. All routes converge on confrontation but produce different opening conditions, allies, control, and available truths.

## Solvability budget

The baseline path assumes the player follows a majority of clearly presented companion leads, maintains functional equipment, answers a few obvious faction pressures, and does not repeatedly ignore explicit warnings. It must reach confrontation with minimum proof, at least one viable ally or territorial advantage, and a winnable preparation state before Day 100.

| Lane | Player behavior | Guaranteed campaign result |
| --- | --- | --- |
| Baseline | follows companion leads and plays reasonably | barely but reliably sufficient proof and preparation |
| Skilled | sequences leads, reads RTS pressures, manages territory and heat | more time, stronger alliances, optional discoveries, favorable setup |
| Reckless or neglectful | ignores warnings, abandons leads, destabilizes the island | early or poorly prepared confrontation, never an unexplained arbitrary failure |

Every required lead records time cost, prerequisites, fallback source, preparation reward, heat risk, and latest safe start. Automated campaign tests run baseline policies across bounded AI-wobble seeds; every approved seed must remain solvable.

## Hero and investigation loop

Each companion owns a competence domain, personal stake, updatable hypotheses, authored questions, actionable requests, competing interpretations, a personal quest line, and support needs expressed as equipment, escort, influence, access, time, or trust.

The recurring loop is:

1. The autonomous island advances and exposes readable changes.
2. Companions interpret those changes and propose leads.
3. The player compares theories, costs, faction effects, danger, and time.
4. The player assigns and equips heroes, negotiates access, or redirects forces.
5. Heroes act on the board while factions continue their RTS cycles.
6. Evidence, relationships, territory, heat events, and personal quests update.
7. Companions synthesize the outcome and propose the next questions.

Delegation never turns a heroine into an off-screen progress bar. Her position, route, exposure, escort, access, and task state remain inspectable. Critical decisions return to the player; routine execution can proceed once authorized.

## Island board grammar

The board is a stable isometric coordinate space. A region contains nodes and tiles. Typed tethers connect compatible sockets between nodes or structures and express roads, waterways, supply, ritual links, influence conduits, sightlines, or authored special relationships. A tether is a simulated relationship with state, capacity, ownership, visibility, and disruption rules—not decorative line art.

Structures declare hex footprints, construction stages, actor spawn points, tether sockets, influence emitters, state hooks, damage states, selection bounds, and visual-family metadata. Props use the same spatial contract at smaller scale. Territory and influence derive from placed, inspectable causes.

Faction structures are also adventure sites. Their faction, building archetype, upgrade level, board position, current condition, world-day band, and stable instance seed select a deterministic encounter layout, defenders, complications, discoveries, and loot table. Raising a building's level increases both the faction capability it provides and the risk and reward of attacking or infiltrating it. A level-five site should generally offer better loot than its lower-level form, but rewards remain faction- and archetype-specific rather than collapsing into a universal item curve.

Site generation uses a recorded composite seed derived from at least the world seed, stable faction ID, stable structure instance ID, structure archetype, structure level, and generation revision. A level-five cult dungeon and a level-two imperial fort therefore have different seeds, content families, defense grammar, and reward profiles. Re-entering an unchanged site reproduces its durable identity; construction, upgrading, capture, damage, corruption, or a declared refresh transition produces an auditable new generation state instead of arbitrary rerolling.

Encounter forces on every side derive from the same live faction state. Hostile strength and friendly reinforcements use the contributing faction's production buildings, upgrade levels, available units, named heroes, supply reach, losses, readiness, relationships, and local board position. An allied faction that prospers can field materially stronger help; a faction that has been raided, cut off, or nearly eliminated cannot supply an unchanged scripted ally squad. Difficulty therefore scales dynamically through the world simulation rather than by silently matching the player level.

## RTS faction contract

This is a conventional solved-problem domain. Use established RTS simulation patterns for economies, build orders, queues, influence, supply, targeting, and territorial replacement. Project-specific design effort belongs in asymmetric goals, readable world consequences, companion interpretation, and Cthulhu's hidden plan—not in inventing a novel substitute for basic RTS behavior.

Every ordinary faction has:

- stockpiles, income sources, storage limits, and upkeep;
- worker/build capacity or an explicitly different construction mechanism;
- a build catalogue with costs, prerequisites, footprints, build time, and purpose;
- structure levels and upgrade paths whose benefits, defenses, encounter grammar, and loot bands remain faction-specific;
- construction and unit-production queues that advance on world ticks;
- unit rosters, spawn rules, named-hero rules, rally points, and reinforcement paths;
- territory goals, supply reach, threat knowledge, and target valuation;
- independent bilateral reputation and relationship state with every relevant faction;
- doctrines for expansion, defense, raids, reinforcement, diplomacy, and retreat;
- asymmetric victory and survival goals.

The core faction loop is **gather or receive resources → evaluate needs and opportunities → reserve costs → build or produce → deploy → contest or support territory → learn from results → repeat**. Shortages, destroyed supply, queue contention, lost builders, and blocked footprints materially alter the cycle. A faction cannot conjure a scripted army without a declared spawn exception and visible causal source.

Faction survival and victory intent depend on board position. A secure faction may expand or pursue its asymmetric win condition; a pressured faction may consolidate, migrate, negotiate, raid for resources, or accept dependency; a doomed faction may spend its remaining capacity on escape, revenge, succession, or a last objective. These are utility responses to real state, not protected narrative roles.

Elimination is persistent. When a faction has no viable recovery path under its authored rules—no qualifying territory, population or production source, recoverable named hero, allied restoration route, or other explicit continuity condition—it becomes eliminated and is effectively gone from the island. It does not respawn because a later quest expects it. Its former territory, structures, resources, routes, and influence become abandoned, captured, dismantled, contested, corrupted, or reclaimed through ordinary simulation. Other factions grow or shrink into the vacuum according to proximity, supply, need, relationships, and utility. Authored quests tolerate this changing ecology through state-aware variants, successor actors, recoverable evidence, or honest closure.

At each decision window, legal RTS actions are scored by bounded utility:

`score = strategic value + goal progress + relationship value + personality bias - cost - risk + bounded wobble`

Wobble breaks brittle repetition without overriding dominant preferences. It is seeded, bounded, recorded for replay, and too small to make an obviously ruinous choice optimal unless personality or hidden goals justify it. Every autonomous action records candidates, score components, chosen action, wobble contribution, knowledge used, and public explanation tokens.

Full utility traces, stockpile numbers, queues, and hidden goals exist for simulation, debugging, and balance tests. The in-world experience exposes only what the heroes could perceive or learn: construction, troop movement, scarcity, smoke, damaged roads, changed patrols, abandoned holdings, rumors, negotiations, refugees, captured banners, and companion deductions. Inspection tools summarize known facts but never grant omniscient access to undiscovered faction state.

Cthulhu has a distinct economy and utility model. Its real objective is to win enough of the island struggle to complete the Day-100 summoning plan. Its build cycle may convert ritual control, dreams, sacrifices, corruption, or specific network states rather than ordinary lumber and coin. It may sacrifice territory, conventional resources, cult units, structures, and apparent victories when that advances the plan, misdirects investigators, or changes heat.

Cthulhu is deliberately weighted toward eventual victory in an unattended world. Authored escalation events give its faction causal advantages at known world-state or day thresholds: opened ritual routes, corrupted production, converted leaders, abnormal weather support, or other declared effects. These are simulation inputs with prerequisites and visible consequences, not free armies created off-screen. If the heroes and other factions do not disrupt that trajectory, Cthulhu's growing economy, board control, forces, and ritual network increasingly win the faction war and culminate in the Day-100 summoning of the big bad. Debug tooling may explain the full utility and boosts; the player earns that understanding through clues.

## Rule-governed wrongness

Wrongness effects require world-day and heat prerequisites; optional faction, location, weather, clue, or prior-effect prerequisites; a declared simulation consequence or `presentation_only`; an advance tell and aftermath clue; affected regions and duration; stacking/exclusion rules; accessibility substitutes; and a causal explanation for authoring tools.

Early signals are deniable. Mid-state effects make weather and behavior coordinated. Late-state effects bend timing and causality, but remain reconstructible: an effect cannot retroactively invalidate a decision without a prior signal and recovery response.

## Presentation and art production

The strategic island uses one fixed-view isometric presentation. Zoom and inspection may change detail density, but not camera angle or spatial truth. Selection silhouettes, faction colors, footprint outlines, tether states, alerts, and overlays remain readable without color alone.

The prior side-view battle prototype is retained as historical technical evidence for typed command/event projection and rig experimentation. It is not the current camera, world-loop, or vertical-slice authority. No further side-view-specific content should be produced unless a later recorded decision gives it a bounded role.

Tile and structure kits include stable IDs, isometric orientation, board scale, exact hex footprint and height envelope, construction/damage pieces, separate selection/collision bounds, spawn points, tether sockets, influence/state hooks, faction-overlay material slots, detail-density rules, provenance, and approval metadata.

Actor assets require a stable root, ground anchor, selection bounds, facing contract, equipment and effect sockets, separable rig parts or bones, and identity metadata. Animation is out of scope now; future attachment points are not.

Base art families are western fantasy, Bronze-Age mythic mashup, and eastern/wushu fantasy mashup. Mini-kits and overlays specialize elves, treefolk, cult forces, pirates, smugglers, imperial forces, and later factions. Art is admitted only after board-scale silhouette review, isometric fit, metadata validation, provenance review, and an in-context screenshot.

## Technical ownership

| Concern | Canonical owner |
| --- | --- |
| time, heat events, confrontation cause, faction economies/build queues/decisions, diplomacy, influence, spawns, combat, persistence | Rust simulation |
| board rendering, selection, input, overlays, feedback, audio, accessibility projection | Godot/GDScript |
| characters, leads, clues, factions, structures, tethers, wrongness effects, art metadata | validated JSON content |
| validation, deterministic bundle generation, reports, balance-harness orchestration | TypeScript tools |

Godot submits intents and renders snapshots/events. JSON describes authored options and constraints. TypeScript validates and packages content. None becomes a second gameplay runtime.

## Failure, recovery, persistence, accessibility

The game autosaves at each world day and before irreversible lead, diplomacy, heat, or confrontation transactions. Saves store world seed, simulation version, bundle hash, board entities, faction economies/queues/knowledge/relationships, day, heat-event ledger, clues and hypotheses, companion leads, and confrontation cause.

Required clues cannot be destroyed without a recovery source or clearly signaled alternate confrontation route. A heroine who is injured, captured, separated, or unavailable retains authorship of her ideas; another hero may recover her work but does not silently become its originator.

All important state has paired channels: color plus shape/pattern/text; weather visuals plus captions or ambience cues; audio direction plus board markers; motion or flashes plus reduced-motion/reduced-flash substitutes. Pausing stops player-facing advancement. Speed controls and event-log replay make simultaneous RTS activity understandable.

## First vertical slice

The first new-direction slice is one isometric board with two connected regions, three ordinary RTS factions plus Cthulhu's hidden faction, Michael, one fully authored companion investigator, and three rig-ready heroine stand-ins. It proves:

1. world ticks advance autonomously;
2. factions gather resources, progress real build/production queues, deploy units, and select explainable actions;
3. a structure occupies a validated footprint and exposes spawn, tether, and influence hooks;
4. independent faction relationships affect but do not replace RTS decisions;
5. a companion observes a board change, proposes two interpretations, and requests support;
6. the lead changes board state, relationships, proof, time, and possibly heat in one authoritative transaction;
7. an early wrongness signal follows an authored rule and has accessible evidence;
8. save/load and replay preserve the exact next decision;
9. a baseline policy remains on schedule for minimum Day-100 preparation across approved seeds.

It does not require final art, all companion arcs, broad content, complete combat replacement, or animation.

## Milestones

### M0 — Authority and schemas

Contradictory active documents are subordinated; core world, faction, structure, and campaign schemas validate representative records; the content bundle is deterministic.

### M1 — Autonomous RTS board

The board shows regions, tethers, resource flow, construction and production, deployments, territory/influence change, and replayable faction decisions while the player does nothing.

### M2 — Companion-led investigation

One companion creates, revises, and resolves a lead from simulated evidence; the player supports or redirects her; failure has a recoverable continuation.

### M3 — Clocks and wrongness

Deliberate discovery, Day 100, and terminal heat independently record the correct confrontation cause; each heat/world band exposes consistent diegetic signals without numeric leakage.

### M4 — Five-hero campaign slice

All five heroes are controllable, all four women own a playable lead and personal stake, faction diplomacy persists independently, and baseline balance simulations remain solvable.

## Approval ledger

### Accepted

- HaremLit adventure over an autonomous multi-faction RTS island.
- Michael plus four female companions as five controllable heroes.
- Companion-driven investigations and personal quests.
- Fixed-view isometric board as spatial and systemic presentation.
- Real faction economies, build cycles, production, deployment, and territory contest.
- First-of-three Cthulhu confrontation triggers.
- Day 100 deadline plus separate irreversible hidden heat ledger.
- Rule-governed, inferable weather/time/causality wrongness.
- Broad western, mythic Bronze-Age, and eastern/wushu art families with faction overlays.
- Rig now, animate later.

### Superseded

- Side-view party-card battle as governing presentation.
- Compact route screen instead of represented RTS island board.
- Unnamed adaptive cosmic intruder as central threat; Cthulhu is explicit.
- Michael as default sole investigator or quest initiator.
- Diplomacy scripts standing in for functioning faction economies and build cycles.
- Midnight respawn as the primary campaign mystery. It may survive only if later reconciled as a subordinate rule of Cthulhu's system.

### Provisional

- Exact hex size, tick length, heat thresholds, faction roster, resource catalogue, and companion identities beyond approved character work.
- Whether tactical combat resolves directly on the board or enters a bounded tactical presentation. Either choice preserves the board as world truth.
- Exact Day-100 confrontation and ending structure.

### Rejected

- Numeric heat display; pure-random AI or horror; mandatory optimization; animation during this phase; duplicate board, route, faction, or investigation runtimes; scripted faction activity that bypasses build-cycle constraints without an authored exceptional cause.

## Historical material policy

Older documents and prototypes remain evidence for character writing, simulation/presentation separation, deterministic combat, provenance, and rig/socket experiments. They do not retain authority merely because implementation exists. Every active implementation contract must link here and label incompatible assumptions historical, superseded, or provisional before further work builds on them.
