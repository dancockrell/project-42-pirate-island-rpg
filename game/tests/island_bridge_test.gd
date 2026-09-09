extends SceneTree

const Port = preload("res://scripts/simulation/native_simulation_port.gd")

func captain(snapshot: Dictionary) -> Dictionary:
	for actor in snapshot.actors:
		if actor.id == "character.protagonist.captain":
			return actor
	assert(false, "the scenario's captain is missing")
	return {}

func _initialize() -> void:
	var port = Port.new()
	assert(port.is_available())
	# The verb takes a scenario document and nothing else.
	assert(port.create_island_from_scenario("{}").has("error"))
	assert(port.create_island_from_scenario('{"format":"project42.scenario","version":1,"scenario":{}}').has("error"))
	var initial: Dictionary = port.create_island()
	assert(not initial.has("error"))
	# The main scenario places Michael alone and the five factions' holdings.
	assert(initial.actors.size() == 1)
	assert(initial.buildings.size() == 5)
	var start := Vector2i(captain(initial).x, captain(initial).y)
	var land := {}
	for cell in initial.land:
		land[cell] = true
	assert(land.has(start))
	# Four land cells in a straight line from wherever the scenario starts him.
	var step_direction := Vector2i.ZERO
	for direction in [Vector2i(-1,0), Vector2i(1,0), Vector2i(0,-1), Vector2i(0,1)]:
		var clear := true
		for distance in range(1, 5):
			clear = clear and land.has(start + direction * distance)
		if clear:
			step_direction = direction
			break
	assert(step_direction != Vector2i.ZERO)
	var target := start + step_direction * 4
	assert(not port.move_island_actor("unknown", target))
	assert(not port.move_island_actor("character.protagonist.captain", Vector2i(-1,-1)))
	assert(port.move_island_actor("character.protagonist.captain", target))
	assert(Vector2i(captain(port.island_snapshot()).x, captain(port.island_snapshot()).y) == start)
	port.pause_island(true)
	# Pause freezes the whole island, not just the captain. A paused tick adds
	# only its (empty) strike list to the snapshot it started from.
	var frozen := JSON.stringify(port.island_snapshot())
	var stepped: Dictionary = port.tick_island()
	assert(stepped.strikes.is_empty())
	stepped.erase("strikes")
	assert(JSON.stringify(stepped) == frozen)
	port.pause_island(false)
	for step in range(4):
		var state: Dictionary = port.tick_island()
		assert(Vector2i(captain(state).x, captain(state).y) == start + step_direction * (step + 1))
	assert(not captain(port.island_snapshot()).moving)
	# The campaign clock reaches Godot as authored state, and only as much of
	# it as the player may see: the authored record says heat is never a
	# number, so the snapshot carries signalling channels and no total.
	var campaign: Dictionary = port.island_snapshot().campaign
	assert(campaign.deadline_day == 100)
	assert(campaign.days_remaining == 99)
	assert(campaign.confrontation == "")
	assert(campaign.confrontation_day == 0)
	assert(campaign.heat_signals is Array)
	assert(not campaign.has("heat_severity"))
	# Triggers change the world, not a feed: what Godot reads is the flag
	# array, the same shape as any other board fact in the snapshot.
	assert(port.island_snapshot().flags is Array)
	print("PASS: actual Rust island bridge, scenario document, solo start, movement, ocean rejection, pause, arrival, the campaign clock and triggers")
	quit()
