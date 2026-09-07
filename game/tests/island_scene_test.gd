extends SceneTree

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var scene = load("res://scenes/world/island.tscn").instantiate()
	root.add_child(scene)
	await process_frame
	scene.set_process(false)
	assert(scene.ready_ok)
	assert(scene.troop_textures.size() == 3)
	for texture in scene.troop_textures:
		assert(texture.get_image().detect_alpha() == Image.ALPHA_BIT)
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
	var path := "user://island-scene-persistence-test.json"
	scene.set_paused(true)
	assert(scene.save_campaign(path))
	var saved_position: Vector2 = scene.actor.position
	scene.set_paused(false)
	assert(scene.request_move(Vector2i(20,18)))
	scene.advance_tick()
	assert(scene.actor.position != saved_position)
	assert(scene.load_campaign(path))
	assert(scene.actor.position == saved_position and scene.paused)
	assert(scene.save_campaign(path))
	var corrupt := FileAccess.open(path, FileAccess.WRITE)
	corrupt.store_string("{broken")
	corrupt.close()
	assert(scene.load_campaign(path)) # backup survives corrupt primary
	var before: Dictionary = scene.port.island_snapshot()
	assert(not scene.port.load_island("{broken"))
	assert(scene.port.island_snapshot() == before)
	for suffix in ["", ".bak", ".tmp"]:
		if FileAccess.file_exists(path + suffix):
			DirAccess.remove_absolute(path + suffix)
	print("PASS: save/load restores native position and pause; corrupt primary falls back; invalid load preserves live state")
	scene.set_paused(false)
	for step in range(60):
		scene.advance_tick()
	assert(scene.snapshot.actors.size() <= 19)
	assert(scene.snapshot.casualties > 0)
	assert(scene.troop_sprites.size() == scene.snapshot.actors.size() - 1)
	var factions := {}
	for unit in scene.snapshot.actors:
		if unit.id != scene.MICHAEL:
			factions[unit.faction] = true
			assert(scene.troop_sprites[unit.id].material == null)
			assert(scene.troop_sprites[unit.id].position == (Vector2(unit.x,unit.y) + Vector2.ONE * 0.5) * 32)
	assert(factions.size() >= 1)
	var survivors: int = scene.troop_sprites.size()
	var payload: String = scene.port.save_island()
	assert(scene.port.load_island(payload))
	scene.refresh_snapshot()
	assert(scene.troop_sprites.size() == survivors)
	print("PASS: autonomous skirmish casualties, capped survivors, removed dead sprites, roster preserved through load")
	print("PASS: real island scene, authored land, sprite/native position agreement, travel and pause")
	quit()
