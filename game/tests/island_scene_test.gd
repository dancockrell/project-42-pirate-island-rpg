extends SceneTree

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var scene = load("res://scenes/world/island.tscn").instantiate()
	root.add_child(scene)
	await process_frame
	scene.set_process(false)
	assert(scene.ready_ok)
	assert(not scene.request_move(Vector2i(0,0)))
	var start: Vector2 = scene.actor.position
	assert(scene.request_move(Vector2i(20,16)))
	scene.set_paused(true)
	scene.advance_tick()
	assert(scene.actor.position == start)
	scene.set_paused(false)
	for step in range(2):
		scene.advance_tick()
	assert(scene.actor.position != start)
	assert(scene.actor.position == Vector2(20.5,16.5) * 32)
	assert(scene.actor.facing == "ne")
	print("PASS: real island scene, authored land, sprite/native position agreement, travel and pause")
	quit()
