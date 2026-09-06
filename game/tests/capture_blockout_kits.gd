extends SceneTree

## Renders P12's review scene in both of its views from ONE engine process.
##
## `game/tools/capture_review_scene.gd` remains the project's capture tool and
## this does not replace it: that tool answers "one scene, one image", and this
## card's evidence is two images of one scene in two states -- the buildings and
## the machines, which are two orders of size apart and cannot share a frame.
## P4's `capture_board_distances.gd` is the same shape for the same reason.
##
## Usage:
##   godot --path game --script res://tests/capture_blockout_kits.gd \
##     -- <output-directory> <prefix>
## writes <prefix>-buildings.png and <prefix>-machines.png.

const SETTLE_FRAMES := 6


func _init() -> void:
	call_deferred("capture")


func capture() -> void:
	var arguments := OS.get_cmdline_user_args()
	if arguments.size() != 2:
		fail("Expected an absolute output directory and a filename prefix")
		return
	var directory := arguments[0]
	var prefix := arguments[1]
	if not directory.is_absolute_path():
		fail("Output directory must be absolute")
		return
	var packed := load("res://scenes/review/blockout_kit_review.tscn") as PackedScene
	if packed == null:
		fail("Could not load res://scenes/review/blockout_kit_review.tscn")
		return
	var review := packed.instantiate() as Node3D
	root.add_child(review)
	await process_frame

	for view in review.VIEWS:
		review.show_view(view)
		for _frame in range(SETTLE_FRAMES):
			await process_frame
		RenderingServer.force_sync()
		RenderingServer.force_draw(false)
		var image := root.get_viewport().get_texture().get_image()
		var path := "%s/%s-%s.png" % [directory, prefix, view]
		var error := image.save_png(path)
		if error != OK:
			fail("Could not save %s: error %s" % [path, error])
			return
		print("Captured the %s view to %s" % [view, path])
	quit(0)


func fail(message: String) -> void:
	push_error(message)
	quit(1)
