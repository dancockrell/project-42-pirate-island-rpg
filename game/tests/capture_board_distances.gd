extends SceneTree

const BoardVerificationCampaign = preload("res://tests/board_verification_campaign.gd")

## Renders the isometric board at all three distances from ONE engine process.
##
## `game/tools/capture_review_scene.gd` remains the project's capture tool and
## this does not replace it: that tool answers "one scene, one image", and this
## card's evidence is three images of one scene in three states. Opening and
## closing the engine three times to get them is what this exists to avoid.
##
## Usage:
##   godot --path game --script res://tests/capture_board_distances.gd \
##     -- <output-directory> <prefix>
## writes <prefix>-world.png, <prefix>-route.png and <prefix>-room.png.

const SETTLE_FRAMES := 4


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
	var packed := load("res://scenes/board/isometric_board.tscn") as PackedScene
	if packed == null:
		fail("Could not load res://scenes/board/isometric_board.tscn")
		return
	var board := packed.instantiate() as IsometricBoard
	root.add_child(board)
	await process_frame
	# The board is photographed with something on it: the authored opening
	# played, two cells held by the factions the fiction gives them, and one
	# force on the road. `BoardVerificationCampaign` owns that campaign and the
	# suite runs the same one, so the picture and the assertion are of one island.
	if board.campaign_session != null and NativeExpeditionPort.bridge_is_registered():
		BoardVerificationCampaign.play_opening(board.campaign_session, 2)
		board.project_snapshot(BoardVerificationCampaign.take_control(board.campaign_session))
		var marched: Dictionary = BoardVerificationCampaign.add_marching_force(board.campaign_session)
		if bool(marched.get("configured", false)):
			board.project_snapshot(marched)
	for distance in IsometricBoard.DISTANCES:
		board.set_distance(distance)
		for _frame in range(SETTLE_FRAMES):
			await process_frame
		RenderingServer.force_sync()
		RenderingServer.force_draw(false)
		var image := root.get_viewport().get_texture().get_image()
		var path := "%s/%s-%s.png" % [directory, prefix, distance]
		var error := image.save_png(path)
		if error != OK:
			fail("Could not save %s: error %s" % [path, error])
			return
		print("Captured the %s distance to %s" % [distance, path])
	quit(0)


func fail(message: String) -> void:
	push_error(message)
	quit(1)
