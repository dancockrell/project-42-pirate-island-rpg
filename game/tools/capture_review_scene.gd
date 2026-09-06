extends SceneTree
## Renders scenes of this project to PNGs through the root viewport, and refuses
## a frame that carries no picture.
##
## This is the one capture tool. tools/capture-scenes.sh and its PowerShell twin
## drive it over every scene in the project; a lane iterating on one scene calls
## it directly with that scene alone. There is deliberately no second renderer
## anywhere in the tree.
##
## It takes a LIST of scenes and renders all of them in one engine process. That
## is the owner's standing instruction about Godot: an engine start costs about
## ten seconds and a lot of memory, and a script that opened and closed one per
## scene would spend most of its life starting up while every other thread on
## the machine waited.
##
## Usage:
##   godot --path game --script res://tools/capture_review_scene.gd -- \
##       <res://scenes/a.tscn> [<res://scenes/b.tscn> ...] \
##       (--out-dir <absolute directory> | --output <absolute.png>) \
##       [--thumbnails] [--thumbnail <absolute.png>] \
##       [--frames N] [--size WxH] [--thumbnail-size WxH]
##
## With --out-dir each scene is written as <out-dir>/<scene stem>.png, and as
## <out-dir>/<scene stem>.thumb.png as well when --thumbnails is given. With
## --output (one scene only) the capture goes exactly where it is named, and
## --thumbnail names the thumbnail if one is wanted.
##
## Defaults: --frames 6, --size 1920x1080, --thumbnail-size 1280x720.
##
## Exit codes: 0 when every scene was captured or skipped for carrying nothing
## that can draw, 1 on any refusal -- bad arguments, a scene that will not load,
## a save error, or a frame that is uniform.

enum Verdict { CAPTURED, SKIPPED, FAILED }

const DEFAULT_FRAMES := 6
const DEFAULT_SIZE := Vector2i(1920, 1080)
const DEFAULT_THUMBNAIL_SIZE := Vector2i(1280, 720)

## The uniform-frame check. The captured image is downsampled to ANALYSIS_WIDTH
## across -- cheap, and it keeps single-pixel noise out of the verdict -- and
## the standard deviation of its luminance is measured on the 0..255 scale. A
## black frame, a frame that is nothing but the environment clear colour, and a
## frame whose camera sits inside geometry all measure at or near zero; a real
## render of the review scenes measures in the tens, and the thinnest picture in
## the project -- a wireframe collision proxy on flat grey -- measures 1.3. Half
## of that is far below any picture in this repository and far above rounding,
## and a capture under it is reported as a failure rather than written out as a
## pass.
const UNIFORM_STDDEV_THRESHOLD := 0.65
const ANALYSIS_WIDTH := 128

var _scene_paths: Array[String] = []
var _out_dir := ""
var _output_path := ""
var _thumbnail_path := ""
var _thumbnails := false
var _frames := DEFAULT_FRAMES
var _size := DEFAULT_SIZE
var _thumbnail_size := DEFAULT_THUMBNAIL_SIZE


func _init() -> void:
	call_deferred("capture_all")


func capture_all() -> void:
	if not _parse_arguments(OS.get_cmdline_user_args()):
		return

	if _size != Vector2i.ZERO:
		root.size = _size
		root.content_scale_size = _size

	var captured := 0
	var skipped: Array[String] = []
	for scene_path in _scene_paths:
		var verdict := await _capture_one(scene_path)
		match verdict:
			Verdict.FAILED:
				return
			Verdict.SKIPPED:
				skipped.append(scene_path)
			Verdict.CAPTURED:
				captured += 1

	print(
		(
			"Captured %d of %d scenes at %dx%d; %d skipped for carrying nothing that can draw."
			% [captured, _scene_paths.size(), _size.x, _size.y, skipped.size()]
		)
	)
	for scene_path in skipped:
		print("  skipped: %s" % scene_path)
	quit(0)


## Renders one scene and writes it out. Every scene starts from the same state:
## no leftover instance, no leftover camera, and debug drawing off.
func _capture_one(scene_path: String) -> Verdict:
	print("==> %s" % scene_path)
	debug_collisions_hint = false
	debug_navigation_hint = false

	var packed := load(scene_path) as PackedScene
	if packed == null:
		fail("Could not load scene: %s" % scene_path)
		return Verdict.FAILED

	var instance := packed.instantiate()
	root.add_child(instance)
	for _settle in range(_frames):
		await process_frame

	# A scene that drew nothing gets one second pass with the engine's collision
	# and navigation debug drawing turned on. black_beach's
	# reception_terrace_collision.tscn is nothing but proxy geometry, and the
	# proxy is what there is to look at in it; without this it captures an empty
	# frame. It is a second pass and not a guess up front because a scene like
	# reception_terrace.tscn builds its visuals from a script after it enters
	# the tree, so whether anything drew can only be answered by letting it run.
	# A scene that did draw is never captured with debug drawing on, so no red
	# wireframe lands over art under review. Collision shapes and navigation
	# meshes register their debug geometry as they enter the tree, which is why
	# the instance is rebuilt rather than reused.
	if instance is Node3D and not _has_visuals(instance):
		instance = await _rebuild_with_debug_drawing(packed, instance)
		print("  nothing drew; recaptured with collision and navigation debug drawing on")

	# A 3D scene with no camera renders the environment and nothing else. The
	# framing below is the capture tool's, never the scene's composition: it
	# fits the merged bounds of everything the scene can put on screen, from a
	# fixed three-quarter angle, exactly so a fragment scene can be looked at.
	# An authored camera always wins.
	var framing_camera: Camera3D = null
	if instance is Node3D and root.get_viewport().get_camera_3d() == null:
		framing_camera = _frame_automatically(instance)
		if framing_camera == null:
			# Not a failure: a scene can legitimately carry no geometry at all.
			# black_beach/reception_terrace_navigation.tscn is an empty
			# NavigationRegion3D and two markers, and says so in its own
			# metadata (authored_navigation_shell_pending_baked_walk_mesh).
			# There is no picture of it to take, so it is reported and skipped
			# rather than either failing the gate or writing out a grey
			# rectangle that would look like a capture. The moment it gains
			# geometry it is captured like anything else.
			print("  nothing in this scene can draw; skipped")
			_discard(instance, null)
			return Verdict.SKIPPED
		for _settle in range(_frames):
			await process_frame

	RenderingServer.force_sync()
	RenderingServer.force_draw(false)
	var image := root.get_viewport().get_texture().get_image()

	var uniformity := uniformity_of(image)
	print(
		(
			"  %dx%d, mean luminance %.2f, stddev %.3f"
			% [image.get_width(), image.get_height(), uniformity.x, uniformity.y]
		)
	)
	if uniformity.y < UNIFORM_STDDEV_THRESHOLD:
		_discard(instance, framing_camera)
		fail(
			(
				"Capture of %s is uniform (stddev %.3f below %.3f): a black or flat frame is a failure, not a pass."
				% [scene_path, uniformity.y, UNIFORM_STDDEV_THRESHOLD]
			)
		)
		return Verdict.FAILED

	var output := _output_for(scene_path)
	var error := image.save_png(output)
	if error != OK:
		_discard(instance, framing_camera)
		fail("Could not save PNG %s: error %s" % [output, error])
		return Verdict.FAILED
	print("  captured to %s" % output)

	var thumbnail_output := _thumbnail_for(scene_path)
	if thumbnail_output != "":
		var thumbnail := Image.new()
		thumbnail.copy_from(image)
		thumbnail.resize(_thumbnail_size.x, _thumbnail_size.y, Image.INTERPOLATE_LANCZOS)
		var thumbnail_error := thumbnail.save_png(thumbnail_output)
		if thumbnail_error != OK:
			_discard(instance, framing_camera)
			fail("Could not save thumbnail %s: error %s" % [thumbnail_output, thumbnail_error])
			return Verdict.FAILED
		print("  thumbnail %dx%d to %s" % [_thumbnail_size.x, _thumbnail_size.y, thumbnail_output])

	_discard(instance, framing_camera)
	return Verdict.CAPTURED


func _rebuild_with_debug_drawing(packed: PackedScene, instance: Node) -> Node:
	root.remove_child(instance)
	instance.queue_free()
	await process_frame
	debug_collisions_hint = true
	debug_navigation_hint = true
	var rebuilt := packed.instantiate()
	root.add_child(rebuilt)
	for _settle in range(_frames):
		await process_frame
	return rebuilt


func _discard(instance: Node, framing_camera: Camera3D) -> void:
	if framing_camera != null:
		root.remove_child(framing_camera)
		framing_camera.queue_free()
	root.remove_child(instance)
	instance.queue_free()


func _output_for(scene_path: String) -> String:
	if _output_path != "":
		return _output_path
	return _out_dir.path_join("%s.png" % scene_path.get_file().get_basename())


func _thumbnail_for(scene_path: String) -> String:
	if _output_path != "":
		return _thumbnail_path
	if not _thumbnails:
		return ""
	return _out_dir.path_join("%s.thumb.png" % scene_path.get_file().get_basename())


## Mean and standard deviation of luminance over a downsampled copy, both on the
## 0..255 scale, returned as (mean, stddev).
func uniformity_of(source: Image) -> Vector2:
	var sample := Image.new()
	sample.copy_from(source)
	if sample.get_width() > ANALYSIS_WIDTH:
		var height := maxi(
			1, int(round(float(sample.get_height()) * ANALYSIS_WIDTH / float(sample.get_width())))
		)
		sample.resize(ANALYSIS_WIDTH, height, Image.INTERPOLATE_BILINEAR)
	var count := sample.get_width() * sample.get_height()
	if count <= 0:
		return Vector2.ZERO
	var total := 0.0
	var total_squares := 0.0
	for y in range(sample.get_height()):
		for x in range(sample.get_width()):
			var pixel := sample.get_pixel(x, y)
			# Rec. 709 luminance, scaled to 0..255 so the threshold reads in the
			# units a person thinks about an 8-bit image in.
			var luminance := (0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b) * 255.0
			total += luminance
			total_squares += luminance * luminance
	var mean := total / count
	var variance := maxf(0.0, total_squares / count - mean * mean)
	return Vector2(mean, sqrt(variance))


func _parse_arguments(arguments: PackedStringArray) -> bool:
	var index := 0
	while index < arguments.size():
		var argument := arguments[index]
		match argument:
			"--frames":
				index += 1
				if index >= arguments.size() or not arguments[index].is_valid_int():
					fail("--frames needs a whole number of settle frames")
					return false
				_frames = maxi(1, arguments[index].to_int())
			"--size":
				index += 1
				if index >= arguments.size():
					fail("--size needs a WxH value, for example 1920x1080")
					return false
				var parsed_size := _parse_size(arguments[index])
				if parsed_size == Vector2i.ZERO:
					fail("--size must be WxH with both above zero, got: %s" % arguments[index])
					return false
				_size = parsed_size
			"--thumbnail-size":
				index += 1
				if index >= arguments.size():
					fail("--thumbnail-size needs a WxH value, for example 1280x720")
					return false
				var parsed_thumbnail := _parse_size(arguments[index])
				if parsed_thumbnail == Vector2i.ZERO:
					fail(
						(
							"--thumbnail-size must be WxH with both above zero, got: %s"
							% arguments[index]
						)
					)
					return false
				_thumbnail_size = parsed_thumbnail
			"--thumbnails":
				_thumbnails = true
			"--thumbnail":
				index += 1
				if index >= arguments.size():
					fail("--thumbnail needs an absolute .png path")
					return false
				_thumbnail_path = arguments[index]
			"--out-dir":
				index += 1
				if index >= arguments.size():
					fail("--out-dir needs an absolute directory")
					return false
				_out_dir = arguments[index]
			"--output":
				index += 1
				if index >= arguments.size():
					fail("--output needs an absolute .png path")
					return false
				_output_path = arguments[index]
			_:
				if argument.begins_with("--"):
					fail("Unknown option: %s" % argument)
					return false
				_scene_paths.append(argument)
		index += 1

	if _scene_paths.is_empty():
		fail("Expected at least one res:// scene path")
		return false
	for scene_path in _scene_paths:
		if not scene_path.begins_with("res://") or not scene_path.ends_with(".tscn"):
			fail("Scene must be a res:// .tscn path, got: %s" % scene_path)
			return false
	if _output_path != "":
		if _scene_paths.size() != 1:
			fail("--output names one file, so it takes one scene; use --out-dir for a list")
			return false
		if _out_dir != "":
			fail("--output and --out-dir name two different destinations; give one")
			return false
		if not _is_absolute_png(_output_path):
			fail("--output must be an absolute .png path, got: %s" % _output_path)
			return false
		if _thumbnail_path != "" and not _is_absolute_png(_thumbnail_path):
			fail("--thumbnail must be an absolute .png path, got: %s" % _thumbnail_path)
			return false
	else:
		if _out_dir == "":
			fail("Give either --out-dir <directory> or --output <file.png>")
			return false
		if not _out_dir.is_absolute_path():
			fail("--out-dir must be an absolute directory, got: %s" % _out_dir)
			return false
		if not DirAccess.dir_exists_absolute(_out_dir):
			fail("--out-dir does not exist: %s" % _out_dir)
			return false
		if _thumbnail_path != "":
			fail("--thumbnail names one file; with --out-dir use --thumbnails")
			return false
	return true


func _is_absolute_png(path: String) -> bool:
	return path.to_lower().ends_with(".png") and path.is_absolute_path()


func _parse_size(text: String) -> Vector2i:
	var parts := text.to_lower().split("x")
	if parts.size() != 2 or not parts[0].is_valid_int() or not parts[1].is_valid_int():
		return Vector2i.ZERO
	var width := parts[0].to_int()
	var height := parts[1].to_int()
	if width <= 0 or height <= 0:
		return Vector2i.ZERO
	return Vector2i(width, height)


## True when anything in the scene draws. Internal children are included
## deliberately: a node the engine adds for itself is still a thing on screen.
func _has_visuals(node: Node) -> bool:
	var pending: Array[Node] = [node]
	while not pending.is_empty():
		var current: Node = pending.pop_back()
		if current is VisualInstance3D or current is CanvasItem:
			return true
		for child in current.get_children(true):
			pending.append(child)
	return false


## The bounds of one node in its own space, or null when it contributes nothing
## to frame on. Collision shapes and navigation meshes count: the engine draws
## both through the rendering server rather than as child nodes, so a proxy-only
## scene has geometry on screen and no VisualInstance3D at all, and framing on
## the nodes is the only way to point a camera at it.
func _local_bounds_of(node: Node) -> Variant:
	if node is VisualInstance3D:
		return (node as VisualInstance3D).get_aabb()
	if node is CollisionShape3D:
		var shape := (node as CollisionShape3D).shape
		if shape == null:
			return null
		var debug_mesh := shape.get_debug_mesh()
		if debug_mesh == null:
			return null
		return debug_mesh.get_aabb()
	if node is NavigationRegion3D:
		var navigation_mesh := (node as NavigationRegion3D).navigation_mesh
		if navigation_mesh == null:
			return null
		var vertices := navigation_mesh.get_vertices()
		if vertices.is_empty():
			return null
		var box := AABB(vertices[0], Vector3.ZERO)
		for vertex in vertices:
			box = box.expand(vertex)
		return box
	return null


## Adds the capture tool's own camera over the merged bounds of everything the
## scene can put on screen, and returns it. Returns null when there is nothing
## to frame.
func _frame_automatically(node: Node) -> Camera3D:
	var bounds := AABB()
	var seen := false
	var pending: Array[Node] = [node]
	while not pending.is_empty():
		var current: Node = pending.pop_back()
		var local_box: Variant = _local_bounds_of(current)
		if local_box != null:
			var world_box: AABB = (current as Node3D).global_transform * (local_box as AABB)
			if seen:
				bounds = bounds.merge(world_box)
			else:
				bounds = world_box
				seen = true
		for child in current.get_children(true):
			pending.append(child)
	if not seen:
		return null

	var centre := bounds.get_center()
	var radius := maxf(1.0, bounds.get_longest_axis_size() * 0.5)
	var camera := Camera3D.new()
	camera.name = "CaptureFramingCamera"
	camera.fov = 55.0
	camera.far = maxf(1000.0, radius * 20.0)
	# A fixed three-quarter angle, so two runs of the same scene frame it the
	# same way and a diff of two captures is a diff of the scene.
	var direction := Vector3(0.75, 0.55, 1.0).normalized()
	var distance := radius / tan(deg_to_rad(camera.fov * 0.5)) * 1.35
	root.add_child(camera)
	camera.global_position = centre + direction * distance
	camera.look_at(centre, Vector3.UP)
	camera.current = true
	print("  no authored camera; framed by the capture tool over bounds %s" % [bounds])
	return camera


func fail(message: String) -> void:
	push_error(message)
	quit(1)
