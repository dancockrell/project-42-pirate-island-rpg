extends SceneTree

const Port = preload("res://scripts/simulation/native_simulation_port.gd")

func _initialize() -> void:
	var port = Port.new()
	assert(port.is_available())
	var initial: Dictionary = port.create_island()
	assert(initial.actors.size() == 1)
	assert(initial.actors[0].x == 8)
	assert(not port.move_island_actor("unknown", Vector2i(12,16)))
	assert(not port.move_island_actor("character.protagonist.captain", Vector2i(-1,-1)))
	assert(port.move_island_actor("character.protagonist.captain", Vector2i(12,16)))
	assert(port.island_snapshot().actors[0].x == 8)
	port.pause_island(true)
	var frozen: Dictionary = port.island_snapshot()
	assert(port.tick_island() == frozen)
	port.pause_island(false)
	for step in range(4):
		var state: Dictionary = port.tick_island()
		assert(state.actors[0].x == 9 + step)
	assert(not port.island_snapshot().actors[0].moving)
	print("PASS: actual Rust island bridge, solo start, movement, ocean rejection, pause and arrival")
	quit()
