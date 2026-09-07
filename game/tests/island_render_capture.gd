extends SceneTree

## Explicit rendered review, not a headless simulation test.
## Run only when a brief rendering session is authorized.
func _initialize() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("CAPTURE REFUSED: headless mode cannot render visual evidence.")
		quit(2)
		return
	call_deferred("capture")

func capture() -> void:
	var destination := "user://island-review.png"
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--output="):
			destination = arg.trim_prefix("--output=")
	if not destination.ends_with(".png") or FileAccess.file_exists(destination) or FileAccess.file_exists(destination + ".json"):
		printerr("CAPTURE REFUSED: provide a fresh PNG output path; prior evidence is preserved.")
		quit(3)
		return
	root.size = Vector2i(640, 480)
	root.content_scale_size = Vector2i(640, 480)
	Engine.max_fps = 15
	var scene = load(ProjectSettings.get_setting("application/run/main_scene")).instantiate()
	root.add_child(scene)
	scene.set_process(false)
	if not scene.ready_ok:
		printerr("CAPTURE FAILED: island scene did not initialize.")
		quit(4)
		return
	for tick in 60:
		scene.advance_tick()
	scene.set_paused(true)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var capture_image: Image = root.get_texture().get_image()
	if capture_image == null or capture_image.is_empty():
		printerr("CAPTURE FAILED: no rendered pixels.")
		quit(5)
		return
	if capture_image.save_png(destination) != OK:
		printerr("CAPTURE FAILED: PNG could not be saved.")
		quit(6)
		return
	var hashes := {}
	for path in ["res://assets/island/terrain.png", "res://assets/island/watch_fort.png",
			"res://assets/island/buildings.json", "res://assets/sprites/michael/source.png",
			"res://assets/island/troops/colonial.png", "res://assets/island/troops/pirate.png",
			"res://assets/island/troops/cultist.png"]:
		hashes[path] = FileAccess.get_sha256(path)
	var evidence := FileAccess.open(destination + ".json", FileAccess.WRITE)
	if evidence == null:
		printerr("CAPTURE INCOMPLETE: PNG saved but provenance could not be written.")
		quit(7)
		return
	evidence.store_string(JSON.stringify({
		"kind": "actual_godot_viewport", "status": "pending_visual_review",
		"engine": Engine.get_version_info().string,
		"display_driver": DisplayServer.get_name(),
		"size": [capture_image.get_width(), capture_image.get_height()],
		"tick": scene.snapshot.tick, "paused": scene.paused,
		"snapshot": scene.snapshot, "asset_sha256": hashes,
		"png_sha256": FileAccess.get_sha256(destination)
	}, "  "))
	evidence.close()
	print("CAPTURE SAVED: ", ProjectSettings.globalize_path(destination))
	quit()
