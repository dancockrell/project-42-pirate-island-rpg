extends SceneTree
## Captures review scenes through their current Camera3D.
##
## One engine process renders every pair it is given, in order. Starting a
## Godot process per scene costs more than the captures do and is unkind to
## whatever else is running on the machine, so batch rather than loop.
##
## Usage:
##   godot --path game --script res://tools/capture_review_scene.gd -- \
##       [WIDTHxHEIGHT] [--frames N] <scene.tscn> <absolute-out.png> [more pairs...]
##
## WIDTHxHEIGHT sets the render size for every capture in the run (default:
## the project's declared viewport). --frames sets how many frames each scene
## is allowed to settle for before the pixels are read; procedural setpieces
## build their geometry in _ready and the camera rig frames them one frame
## later, so the default is deliberately more than one.
##
## A single pair still works exactly as it did, so every existing caller is
## unchanged.

const DEFAULT_SETTLE_FRAMES := 8


func _init() -> void:
	call_deferred("capture_all")


func capture_all() -> void:
	var arguments := OS.get_cmdline_user_args()
	var settle_frames := DEFAULT_SETTLE_FRAMES
	var size := Vector2i.ZERO
	var pairs: Array[String] = []
	var index := 0
	while index < arguments.size():
		var argument := arguments[index]
		if argument == "--frames":
			if index + 1 >= arguments.size():
				fail("--frames needs a count")
				return
			settle_frames = maxi(1, int(arguments[index + 1]))
			index += 2
			continue
		var parsed := _parse_size(argument)
		if parsed != Vector2i.ZERO:
			size = parsed
			index += 1
			continue
		pairs.append(argument)
		index += 1

	if pairs.is_empty() or pairs.size() % 2 != 0:
		fail("Expected one or more <scene.tscn> <absolute-out.png> pairs")
		return

	if size != Vector2i.ZERO:
		var window := root as Window
		if window != null:
			window.size = size

	var pair_index := 0
	while pair_index < pairs.size():
		var scene_path := pairs[pair_index]
		var output_path := pairs[pair_index + 1]
		pair_index += 2
		if not scene_path.begins_with("res://") or not scene_path.ends_with(".tscn"):
			fail("Review scene must be a res:// .tscn path, got: %s" % scene_path)
			return
		if not output_path.to_lower().ends_with(".png") or not output_path.is_absolute_path():
			fail("Output must be an absolute .png path, got: %s" % output_path)
			return
		var packed := load(scene_path) as PackedScene
		if packed == null:
			fail("Could not load review scene: %s" % scene_path)
			return
		var instance := packed.instantiate()
		root.add_child(instance)
		for frame in range(settle_frames):
			await process_frame
		RenderingServer.force_sync()
		RenderingServer.force_draw(false)
		var image := root.get_viewport().get_texture().get_image()
		var error := image.save_png(output_path)
		root.remove_child(instance)
		instance.queue_free()
		# One more frame so the freed scene is gone before the next one is
		# added: two setpieces in the tree at once would be captured together.
		await process_frame
		if error != OK:
			fail("Could not save PNG %s: error %s" % [output_path, error])
			return
		print("Captured %s to %s (%dx%d)" % [scene_path, output_path, image.get_width(), image.get_height()])
	quit(0)


## "1280x720" -> Vector2i(1280, 720). Anything else -> Vector2i.ZERO.
func _parse_size(argument: String) -> Vector2i:
	var parts := argument.split("x", false)
	if parts.size() != 2:
		return Vector2i.ZERO
	if not parts[0].is_valid_int() or not parts[1].is_valid_int():
		return Vector2i.ZERO
	return Vector2i(int(parts[0]), int(parts[1]))


func fail(message: String) -> void:
	push_error(message)
	quit(1)
