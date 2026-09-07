# Island development scene

Run res://scenes/world/island.tscn in the existing game project.
The scene now installs three preview factions at startup: colonials, pirates and
Cthulhu. They use existing content/production rules, with six-unit preview caps
and provisional economy budgets. Units rally toward a contested clearing.
The three preview factions are mutually hostile; Michael begins neutral.
Shift-click a person to inspect their name, faction and short biography;
Shift-click empty ground closes the panel without issuing movement. Selection
tracks the actor ID rather than the displayed name, which need not be unique.
personas.json supplies provisional fictional name/history pools for the three
currently admitted male troop definitions. Rust embeds it: rebuild the extension
after changing it. NamedPerson is stored on the existing produced actor and
moves with that actor into its casualty record; names and history are not
rerolled on save/load. Older actors without identity data remain unknown.
No female actor is inferred from a male sprite; matching female candidates are
under separate review. Biography text is not yet a playable personal quest.
Recruitment, faction transfer and the four-companion roster are still unfinished.
troops/appearances.json owns existing texture, pivot and scale by unit definition
and person sex, never current faction. Thus an allegiance change and sprite
recreation preserve costume. Explicit legacy-unknown mappings retain the three
old male appearances without inventing missing identity fields. Unsupported
definitions/sex variants have no arbitrary colonial fallback and the HUD reports
missing art. No female production is enabled until matching art is integrated.
Right-click a nearby unit to queue one steam-carbine shot. A hit provokes that
faction into retaliation. Range and line of sight are native checks; reload and
pause delay firing. A successful movement order cancels a queued shot, while an
invalid destination preserves it. No automatic pursuit or Echo skills yet.
Provisional Michael profile: 30 health, 4 damage, 4-cell range, 5-tick reload.
His health appears in the HUD. Death pauses with a fallen message, leaves a
native casualty record, and can be saved/reloaded. Dead Michael cannot move or
fire. This is not yet the midnight resurrection or companion rescue system.
On-island skirmishes now use distinct provisional health, damage, range and
cooldown profiles. Attacks resolve simultaneously and cannot shoot through
nonwalkable cells. Autonomous units hold whenever a living hostile is in range
and has line of sight, including during reload. All hold decisions use the same
pre-movement snapshot. Ranged troops therefore engage before melee troops stop;
the strategic route remains queued and resumes when the firing target is lost.
Movement and attacks share target legality. This is not yet pursuit, retreat,
formation spacing or final tactical AI. Player-directed factions without an
autonomous policy are not forced to hold by this behavior.
Each actual strike now emits immutable hit positions and attacker definition
through the native bridge. The scene draws a brief two-pixel effect from attacker
to target: warm shot, pale melee strike, or muted teal cultist effect. These last
one simulation tick and freeze with pause. No random cosmetic attacks are emitted.
Effects survive lethal target removal because they use event positions, not
lookups of surviving sprites. Successful campaign loading clears old effects.
Chest-height offsets are provisional; authored weapon sockets and animated
attack/recoil poses are still missing. Native render review remains outstanding.
Dead units leave saved casualty records and release their
population slots. Midnight conversion/resurrection, diplomacy changes, combat
animation and final faction balance remain unfinished.
Producers now have provisional 80-point health persisted in saves. Autonomous
units without an in-range troop target can strike a hostile producer's entrance
when in range and line of sight.
Units hold their position while a legal siege target exists, including reload;
once it falls they resume their retained strategic route. Movement and attacks
share one native siege-target rule rather than separate distance checks.
A destroyed building loses its queued resources, releases queued population and
removes its obstacle/policy entry. Losing the last
producer invokes persistent faction elimination. Rallied troops now choose
reachable hostile holdings, weighing route length, defenders and damage.
Finite-supply campaign tests reach and destroy a holding without teleportation;
unlimited replacement income can still sustain a stalemate. Building attack
orders for Michael, final durability and destruction art remain unfinished.
The colonial watch fort now has a provisional transparent sprite projected from
its native building position. Operating state dims inactive forts, and removed
buildings disappear on the next snapshot. Other producer archetypes remain
without art: they must not masquerade as colonial forts. The fort's five-cell
ground obstruction now participates in native movement and shot visibility;
the entrance remains open. Elimination removes its obstruction, while inactive
buildings continue blocking. No construction/damage animation yet.
The coarse ground footprint still needs rendered overlay review against the art.

buildings.json owns the sprite path, pivot, scale and blocked cell offsets.
Rust embeds this contract, so changes require rebuilding the native extension.
New saves preserve per-building obstacles. Historical saves without that field
reconstruct them through the same building contract used by new scenarios.
If an old actor position is now inside a wall, loading is rejected without
teleporting or overwriting the current campaign. Such a save still requires an
explicit recovery policy before release; no silent unit movement is performed.

watch_fort.png is the exact shared watch-fort-01/extracted/cell_00_00.png.
SHA256: 34c8f250350f73f4cb6ae78c82a7bf9aca4266040a221d60e40e9f2527494c56.
Source-size silhouette and alpha were inspected; full game-scale visual review
remains pending. Ground pivot (775,825) in the trimmed 1125x928 image marks
the entrance apron; displayed width is provisionally 150 map pixels.

troops/ contains exact copies of the shared troops-01/extracted male idle PNGs.
The existing Cattle Trail sprite_grid.py extractor produced six transparent
cutouts without rescaling, filtering components, or gutter warnings. All six
remain shared; only the three consumed images ship here. Source prompts,
original images, extraction parameters, bounds and hashes remain in the shared
library. Runtime uses binary-alpha PNGs directly, without a chroma shader.
The three male cutouts were inspected at source size: weapons and silhouettes
are intact. Rendered game-scale edge review is still required.
Only the three male idle appearances are used in this initial projection.
Female identities, directional poses and animations await proper NPC data.
Do not infer unit sex or recruitment eligibility from this temporary renderer.
Click land to travel; Space pauses; F5 saves and F9 loads.
No paid service is required to run it.

Save files use Godot's user data directory, pirate-island-save.json, plus a
last-good .bak. Rust serializes the complete current FactionWorld: tick, pause,
factions/resources/buildings/production queues, actors/provenance, navigation,
positions, travel orders and ID counters. Version 1 loads validate into a new
world before replacing live state. The loaded land must match this map.
Writing first flushes a .tmp file; corrupt primaries do not replace good backups.
F9 tries the backup if the primary is corrupt. This is manual persistence, not
autosave or a shipped migration guarantee; future mechanics must join this state.

Terrain source authority:
shared-game-environment-library/procedural/sprites/candidates/pirate-island/island-terrain-01/GENERATION.md.
Runtime terrain.png is an exact source copy, SHA256
5b5af8b60df58e402ec69f9d6788ee788c60238ebc68fd38d60654e7583cf976.

navigation.json is the authored physical navigation mask for this image.
Godot reads the asset contract once and submits its walkable cells to Rust
at initialization. Rust owns routes, movement, positions, travel orders and pause.
The API rejects map changes after ticks/orders begin. This is not room-node travel.

The island is now the configured game entry scene: launching the project starts
this working integration preview, not the historical battle fixture or a finished
game. Michael begins alone. Four companions
must be recruited rather than silently granted. Current pose frames are standing
art; movement is tick-stepped, not an approved walk animation. Foliage occlusion,
pixel-density matching, terrain-mask visual review, complete autonomous factions
and final island scale remain unfinished.

Checks:
Godot headless tests/island_scene_test.gd passes actual scene load, native position
projection, terrain rejection, travel and pause. The separate native bridge test
and all 57 Rust tests pass. Headless success does not prove rendered visual quality.
