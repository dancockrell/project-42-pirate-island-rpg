# Island development scene

Run res://scenes/world/island.tscn in the existing game project.
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
and all 45 Rust tests pass. Headless success does not prove rendered visual quality.
