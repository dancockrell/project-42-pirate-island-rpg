class_name BoardSurface
extends Control

## The isometric board, on the expedition screen.
##
## P4 built the board as a `Node3D`; the expedition screen is a `Control`. This
## is the one seam between them: a `SubViewport` with `isometric_board.tscn`
## inside it, a container that draws that viewport into the screen's layout, and
## the mouse.
##
## It exists so that exactly one thing knows the seam. The screen asks this node
## the three questions it used to ask the deleted 2D board --
## `portal_from_active_cell_to`, `set_party_selected`, `set_inspected_cell` --
## and gets the same answers from the 3D board behind it, so the screen's
## control code did not change meaning when the board underneath it changed.
## Nothing here decides legality, nothing here calls the bridge, and nothing
## here moves the party: a click is turned into a cell id and handed on.
##
## The two signals are P7's, unchanged:
##   left-click   -> `tile_selected`
##   right-click  -> `move_ordered`

signal tile_selected(cell_id: String)
signal move_ordered(cell_id: String)

const BOARD_SCENE_PATH := "res://scenes/board/isometric_board.tscn"

## The three distances, as the screen's camera modes, and the key that reaches
## each. Digits 1 to 9 are already the departures' hotkeys and Escape is already
## the pause menu's, so the function keys carry the camera. The legend below the
## board draws all three, because a hotkey nobody can see is not a control.
const CAMERA_MODE_KEYS := {
	KEY_F1: IsometricBoard.DISTANCE_WORLD,
	KEY_F2: IsometricBoard.DISTANCE_ROUTE,
	KEY_F3: IsometricBoard.DISTANCE_ROOM
}

## What each distance is called on the legend, in the words the player reads.
const DISTANCE_WORDS := {
	IsometricBoard.DISTANCE_WORLD: "THE ISLAND",
	IsometricBoard.DISTANCE_ROUTE: "THE ROADS",
	IsometricBoard.DISTANCE_ROOM: "THIS PLACE"
}

## How solid the plate behind a caption is, and its hairline. Framing choices:
## dark enough to read a caption over the room distance's bright sky, light
## enough not to become a second panel over the island.
const CAPTION_PLATE_ALPHA := 0.66
const CAPTION_PLATE_BORDER_ALPHA := 0.3

## How far a caption's plate sits in from the board's edge, in pixels.
const CAPTION_INSET := 12

var board: IsometricBoard
var board_viewport: SubViewport
var board_container: SubViewportContainer
var legend_label: Label
var distance_label: Label
var region_label: Label


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	build_surface()


func build_surface() -> void:
	board_container = SubViewportContainer.new()
	board_container.name = "BoardViewportContainer"
	# `stretch` makes the viewport exactly the size of the container that draws
	# it, and keeps it that size as the screen resizes. So a point in this
	# control's own coordinates is a point in the board's viewport, which is what
	# lets the camera's ray projection be the whole of the hit test -- and it is
	# why nothing here sets the viewport's size by hand.
	board_container.stretch = true
	# The container must not eat the click: this node owns the mouse, because it
	# is the one that knows how to turn a point into a cell.
	board_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	board_container.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(board_container)
	board_viewport = SubViewport.new()
	board_viewport.name = "BoardViewport"
	board_viewport.own_world_3d = true
	board_viewport.transparent_bg = false
	board_viewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	board_container.add_child(board_viewport)
	var scene := load(BOARD_SCENE_PATH) as PackedScene
	if scene == null:
		push_error("BoardSurface cannot load %s." % BOARD_SCENE_PATH)
		return
	board = scene.instantiate() as IsometricBoard
	board_viewport.add_child(board)
	connect_board_signals()
	build_legend()


## The board's own caption: where the player is looking, and what the controls
## on it are. P7 deleted the 2D board's legend of colours because the colours
## already said what they meant; this legend says what the *hands* do, which
## nothing else on the screen says.
func build_legend() -> void:
	region_label = make_legend_label("BoardRegion")
	add_child(plate_around(region_label, "BoardRegionPlate", Control.PRESET_TOP_LEFT))
	distance_label = make_legend_label("BoardDistance")
	add_child(plate_around(distance_label, "BoardDistancePlate", Control.PRESET_TOP_RIGHT))
	legend_label = make_legend_label("BoardLegend")
	legend_label.text = "LEFT-CLICK  LOOK      RIGHT-CLICK  MARCH      1-9  DEPARTURES      F1-F3  ISLAND / ROADS / PLACE"
	add_child(plate_around(legend_label, "BoardLegendPlate", Control.PRESET_BOTTOM_LEFT))
	refresh_captions()


## One caption, on a plate, in a corner. The plate is sized by the words rather
## than stretched across the board: a bar the width of the screen would be a
## second panel, and the board already has one panel beside it.
func plate_around(label: Label, plate_name: String, corner: int) -> PanelContainer:
	var plate := PanelContainer.new()
	plate.name = plate_name
	plate.mouse_filter = Control.MOUSE_FILTER_IGNORE
	plate.add_theme_stylebox_override("panel", make_caption_plate())
	plate.add_child(label)
	plate.set_anchors_and_offsets_preset(corner, Control.PRESET_MODE_MINSIZE, CAPTION_INSET)
	if corner == Control.PRESET_TOP_RIGHT:
		plate.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	if corner == Control.PRESET_BOTTOM_LEFT:
		plate.grow_vertical = Control.GROW_DIRECTION_BEGIN
	return plate


func make_legend_label(node_name: String) -> Label:
	var label := Label.new()
	label.name = node_name
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	label.add_theme_color_override("font_color", BoardPalette.cream())
	label.add_theme_font_size_override("font_size", ThemeTokens.font_size(ThemeTokens.active(self), "caption"))
	return label


func make_caption_plate() -> StyleBoxFlat:
	var plate := StyleBoxFlat.new()
	plate.bg_color = Color(BoardPalette.deep(), CAPTION_PLATE_ALPHA)
	plate.border_color = Color(BoardPalette.bronze(), CAPTION_PLATE_BORDER_ALPHA)
	plate.set_border_width_all(1)
	plate.set_corner_radius_all(4)
	plate.content_margin_left = 8
	plate.content_margin_right = 8
	plate.content_margin_top = 3
	plate.content_margin_bottom = 3
	return plate


# ---------------------------------------------------------------------------
# The three questions the screen asks, forwarded
# ---------------------------------------------------------------------------

func configure(catalog: ContentCatalog, snapshot: Dictionary) -> void:
	if board == null:
		return
	board.configure(catalog, snapshot)
	refresh_captions()


func portal_from_active_cell_to(cell_id: String) -> String:
	return "" if board == null else board.portal_from_active_cell_to(cell_id)


func is_reachable(cell_id: String) -> bool:
	return board != null and board.is_reachable(cell_id)


func set_party_selected(selected: bool) -> void:
	if board != null:
		board.set_party_selected(selected)


func set_inspected_cell(cell_id: String) -> void:
	if board != null:
		board.set_inspected_cell(cell_id)


func party_selected() -> bool:
	return board != null and board.party_selected


## The camera mode the screen is in. One of `IsometricBoard.DISTANCES`.
func distance() -> String:
	return IsometricBoard.DISTANCE_WORLD if board == null else board.distance


## Move the camera to one of the three distances. This is a camera move and
## nothing else: it never touches the party, never touches the snapshot, and
## never opens or closes a road.
func set_distance(next_distance: String) -> void:
	if board == null or not IsometricBoard.DISTANCES.has(next_distance):
		return
	board.set_distance(next_distance)
	refresh_captions()


## The distance a key reaches, or "" when the key is not one of the board's.
static func distance_for_key(keycode: Key) -> String:
	return str(CAMERA_MODE_KEYS.get(keycode, ""))


func refresh_captions() -> void:
	if board == null or distance_label == null:
		return
	distance_label.text = str(DISTANCE_WORDS.get(board.distance, "")).to_upper()
	if region_label != null and board.catalog != null:
		region_label.text = str(board.catalog.get_record(IsometricBoard.REGION_ID).get("displayName", "")).to_upper()


# ---------------------------------------------------------------------------
# The mouse
# ---------------------------------------------------------------------------

func _gui_input(event: InputEvent) -> void:
	if board == null:
		return
	var click := event as InputEventMouseButton
	if click != null and click.pressed:
		if click.button_index == MOUSE_BUTTON_LEFT:
			accept_event()
			board.click_at(click.position)
		elif click.button_index == MOUSE_BUTTON_RIGHT:
			accept_event()
			board.order_at(click.position)
		return
	var motion := event as InputEventMouseMotion
	if motion != null:
		board.hover_at(motion.position)


## The board's two signals are this node's two signals: the screen connects to
## one thing, and what is behind it is this node's business.
func connect_board_signals() -> void:
	if not board.tile_selected.is_connected(_on_tile_selected):
		board.tile_selected.connect(_on_tile_selected)
	if not board.move_ordered.is_connected(_on_move_ordered):
		board.move_ordered.connect(_on_move_ordered)


func _on_tile_selected(cell_id: String) -> void:
	tile_selected.emit(cell_id)


func _on_move_ordered(cell_id: String) -> void:
	move_ordered.emit(cell_id)
