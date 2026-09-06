extends SceneTree
## Captures any scene through its current camera, or straight off the viewport
## for a 2D screen.
## Usage:
##   godot --path game --script res://tools/capture_review_scene.gd -- \
##       <scene[,overlay...]> <absolute-output.png> [<scene> <output.png> ...]
##
## Arguments come in pairs, and every pair is rendered by the SAME engine
## process: opening and closing Godot once per image is unkind to a machine
## running several lanes at once.
##
## The first half of a pair may name more than one scene, comma-separated. They
## are instanced into the root in order, so a surface that exists over a running
## screen -- the pause menu over the expedition -- is captured over the screen it
## is drawn over, rather than over an empty viewport that would flatter it.

## Frames to settle before the shot. A screen builds itself in `_ready`, a
## threaded resource can land a frame later, and a tween needs one more.
const SETTLE_FRAMES := 4


func _init() -> void:
	call_deferred("capture")


func capture() -> void:
	var arguments := OS.get_cmdline_user_args()
	if arguments.size() < 2 or arguments.size() % 2 != 0:
		fail("Expected scene and absolute PNG output path pairs")
		return
	var index := 0
	while index < arguments.size():
		if not await capture_one(arguments[index], arguments[index + 1]):
			return
		index += 2
	quit(0)


func capture_one(scene_argument: String, output_path: String) -> bool:
	if not output_path.to_lower().ends_with(".png") or not output_path.is_absolute_path():
		fail("Output must be an absolute .png path: %s" % output_path)
		return false
	var instances: Array[Node] = []
	for scene_path in scene_argument.split(",", false):
		if not scene_path.begins_with("res://") or not scene_path.ends_with(".tscn"):
			fail("Scene must be a res:// .tscn path: %s" % scene_path)
			return false
		var packed := load(scene_path) as PackedScene
		if packed == null:
			fail("Could not load scene: %s" % scene_path)
			return false
		var instance := packed.instantiate()
		root.add_child(instance)
		instances.append(instance)
		await process_frame
	for frame in SETTLE_FRAMES:
		await process_frame
	RenderingServer.force_sync()
	RenderingServer.force_draw(false)
	var image := root.get_viewport().get_texture().get_image()
	var error := image.save_png(output_path)
	for instance in instances:
		instance.queue_free()
	await process_frame
	if error != OK:
		fail("Could not save PNG %s: error %s" % [output_path, error])
		return false
	print("Captured %s to %s" % [scene_argument, output_path])
	return true


func fail(message: String) -> void:
	push_error(message)
	quit(1)
