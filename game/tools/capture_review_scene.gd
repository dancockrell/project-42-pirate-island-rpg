extends SceneTree
## Captures any review scene through its current Camera3D.
## Usage:
##   godot --path game --script res://tools/capture_review_scene.gd -- <scene> <absolute-output.png>


func _init() -> void:
	call_deferred("capture")


func capture() -> void:
	var arguments := OS.get_cmdline_user_args()
	if arguments.size() != 2:
		fail("Expected review scene path and absolute PNG output path")
		return
	var scene_path := arguments[0]
	var output_path := arguments[1]
	if not scene_path.begins_with("res://") or not scene_path.ends_with(".tscn"):
		fail("Review scene must be a res:// .tscn path")
		return
	if not output_path.to_lower().ends_with(".png") or not output_path.is_absolute_path():
		fail("Output must be an absolute .png path")
		return
	var packed := load(scene_path) as PackedScene
	if packed == null:
		fail("Could not load review scene: %s" % scene_path)
		return
	var instance := packed.instantiate()
	root.add_child(instance)
	await process_frame
	await process_frame
	await process_frame
	RenderingServer.force_sync()
	RenderingServer.force_draw(false)
	var image := root.get_viewport().get_texture().get_image()
	var error := image.save_png(output_path)
	if error != OK:
		fail("Could not save PNG %s: error %s" % [output_path, error])
		return
	print("Captured %s to %s" % [scene_path, output_path])
	quit(0)


func fail(message: String) -> void:
	push_error(message)
	quit(1)
