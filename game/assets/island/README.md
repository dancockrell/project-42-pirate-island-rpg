# Island development scene

Run res://scenes/world/island.tscn in the existing game project.
The scene now installs three preview factions at startup: colonials, pirates and
Cthulhu. They use existing content/production rules, with six-unit preview caps
and provisional economy budgets. Units rally toward a contested clearing.
The three preview factions are mutually hostile; Michael remains neutral.
On-island skirmishes now use distinct provisional health, damage, range and
cooldown profiles. Attacks resolve simultaneously and cannot shoot through
nonwalkable cells. Dead units leave saved casualty records and release their
population slots. Midnight conversion/resurrection, diplomacy changes, combat
animation, building damage and final faction balance remain unfinished.
Buildings exist in simulation but do not yet have visible building art.

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

The original battle scene remains default; this island scene is a working
integration preview, not the full game. Michael begins alone. Four companions
must be recruited rather than silently granted. Current pose frames are standing
art; movement is tick-stepped, not an approved walk animation. Foliage occlusion,
pixel-density matching, terrain-mask visual review, autonomous faction integration
and final island scale remain unfinished.

Checks:
Godot headless tests/island_scene_test.gd passes actual scene load, native position
projection, terrain rejection, travel and pause. The separate native bridge test
and all 52 Rust tests pass. Headless success does not prove rendered visual quality.
