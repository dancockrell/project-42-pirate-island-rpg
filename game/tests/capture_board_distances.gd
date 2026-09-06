extends SceneTree

const BoardVerificationCampaign = preload("res://tests/board_verification_campaign.gd")

## Renders the expedition screen -- which since P10 *is* the isometric board --
## in five states from ONE engine process.
##
## `game/tools/capture_review_scene.gd` remains the project's capture tool and
## this does not replace it: that tool answers "one scene, one image", and this
## card's evidence is five images of one screen in five states that only the
## screen's own controls can put it in. Opening and closing the engine five
## times to get them is what this exists to avoid.
##
## P4 pointed this script at `isometric_board.tscn` on its own, because the
## board was not yet on any screen. It is now, so photographing the bare scene
## would be a picture of something no player sees; the states below are reached
## by calling the very functions the player's hand calls.
##
## Usage:
##   godot --path game --script res://tests/capture_board_distances.gd \
##     -- <output-directory> <prefix>
## writes <prefix>-{world,route,room,selected,refused}.png.

const SETTLE_FRAMES := 4

## How wide a written capture is. The ledger's rule is a picture that can be
## *looked at* and a file under 400 KB; this board's sky is a dithered gradient
## and a 1920-wide PNG of it runs past that on its own, so the ledger plate is
## written at this width and CI's `captures` job remains the full-size render.
## There is no image tool in the container, so the engine's own `Image.resize`
## does it -- rule 15's named substitute.
const LEDGER_WIDTH := 880

## The cell the refused order is aimed at. From the river landing no authored
## road reaches the black beach, so the order is refused by the native legal
## list rather than by anything this script arranged.
const UNREACHABLE_CELL := "world.cell.black_beach"


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
	var packed := load("res://scenes/world/expedition_prototype.tscn") as PackedScene
	if packed == null:
		fail("Could not load res://scenes/world/expedition_prototype.tscn")
		return
	var screen := packed.instantiate() as ExpeditionPrototype
	root.add_child(screen)
	await process_frame
	# The screen is photographed with something on it: the authored opening
	# played to the river landing, two cells held by the factions the fiction
	# gives them, and one force on the road. `BoardVerificationCampaign` owns
	# that campaign and the suite runs the same one, so the picture and the
	# assertion are of one island.
	if screen.campaign_session != null and NativeExpeditionPort.bridge_is_registered():
		BoardVerificationCampaign.play_opening(screen.campaign_session, 2)
		screen.project_snapshot(BoardVerificationCampaign.take_control(screen.campaign_session), "The pirates hold the landing. The safe road is not safe now.")
		var marched: Dictionary = BoardVerificationCampaign.add_marching_force(screen.campaign_session)
		if bool(marched.get("configured", false)):
			screen.project_snapshot(marched, "A pirate column is on the safe road, most of the way to the terrace.")
	for distance in IsometricBoard.DISTANCES:
		screen.press_camera_hotkey(camera_key_for(distance))
		if not await save_state(screen, directory, prefix, distance):
			return
	# Selected: the left-click on the party's own tile, which is what an RTS
	# player does first. The ring under the party is the whole of the state.
	screen.press_camera_hotkey(KEY_F1)
	screen.select_tile(str(screen.get_authoritative_snapshot().get("active_location_id", "")))
	if not await save_state(screen, directory, prefix, "selected"):
		return
	# Refused: a right-click on a place no legal road reaches. The status line
	# says so and no call is made -- that refusal is the picture.
	screen.order_move_to_tile(UNREACHABLE_CELL)
	if not await save_state(screen, directory, prefix, "refused"):
		return
	quit(0)


## The key the screen's own camera-mode hotkey handler reads for one distance,
## so this script reaches the three distances by the control the player uses
## rather than by a second path into the board.
func camera_key_for(distance: String) -> Key:
	for keycode in BoardSurface.CAMERA_MODE_KEYS:
		if str(BoardSurface.CAMERA_MODE_KEYS[keycode]) == distance:
			return keycode
	return KEY_F1


func save_state(screen: ExpeditionPrototype, directory: String, prefix: String, state: String) -> bool:
	for _frame in range(SETTLE_FRAMES):
		await process_frame
	RenderingServer.force_sync()
	RenderingServer.force_draw(false)
	var image := root.get_viewport().get_texture().get_image()
	if image.get_width() > LEDGER_WIDTH:
		image.resize(LEDGER_WIDTH, roundi(float(image.get_height()) * float(LEDGER_WIDTH) / float(image.get_width())), Image.INTERPOLATE_LANCZOS)
	var path := "%s/%s-%s.png" % [directory, prefix, state]
	var error := image.save_png(path)
	if error != OK:
		fail("Could not save %s: error %s" % [path, error])
		return false
	print("Captured the %s state to %s" % [state, path])
	return true


func fail(message: String) -> void:
	push_error(message)
	quit(1)
