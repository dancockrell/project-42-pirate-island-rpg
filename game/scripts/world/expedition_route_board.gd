class_name ExpeditionRouteBoard
extends Control

## The playable board for the Black Beach chapter. It does not decide where
## travel is legal: the native expedition snapshot supplies that fact and this
## node draws it and takes orders on it.
##
## P7 (B18 resumed). The board used to draw one line per authored portal, a teal
## dot at the middle of every legal one and a legend explaining the colours. All
## three are gone rather than hidden, because the thing they encoded -- "this
## road is open right now" -- is a property of a *tile* from where the party
## stands, and drawing it twice meant two answers to one question. A tile whose
## portal from the active cell is in `legal_route_commands` is lit; every other
## tile is dim. That is the whole legality vocabulary on the board.
##
## The grammar is the RTS one, and the board only reports it:
##   left-click   -> `tile_selected`  (the party's own tile selects it, any
##                   other tile is a look, never a move)
##   right-click  -> `move_ordered`   (the screen turns it into the one
##                   `request_travel` call, or refuses it in the status line)
## The board never calls the bridge and never judges legality a second time.

## Left-click on a tile. The screen decides whether that is "select the party"
## or "look at that place"; the board only says which tile was clicked.
signal tile_selected(cell_id: String)

## Right-click on a tile: the move order, in board terms. The screen resolves it
## to a portal and to the single native travel call.
signal move_ordered(cell_id: String)

const DEEP := Color("0b1514")
const LAND := Color("18352b")
const RIVER := Color("235b68")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("52625b")
const DANGER := Color("c24e45")

# Tile geometry, in pixels at the board's drawn size. An isometric top face is
# twice as wide as it is tall; the body is the extrusion under it.
const TILE_HALF_WIDTH := 52.0
const TILE_HALF_HEIGHT := 26.0
const TILE_DEPTH := 13.0

# The selection ring's gentle breath. The amplitude is a fraction of the ring's
# radius and the period is in seconds. Both are switched off under reduced
# motion, where the ring is drawn at rest.
const RING_PULSE_AMPLITUDE := 0.055
const RING_PULSE_SECONDS := 2.6

const NODE_POSITIONS := {
	"world.cell.black_beach": Vector2(0.16, 0.72),
	"world.cell.damaged_estate": Vector2(0.31, 0.48),
	"world.cell.river_landing": Vector2(0.51, 0.36),
	"world.cell.reception_terrace": Vector2(0.73, 0.52),
	"world.cell.processional_ramp": Vector2(0.86, 0.26)
}

var catalog: ContentCatalog
var active_location_id := ""
var legal_portal_ids: Dictionary = {}
var cell_names: Dictionary = {}
var portals: Array[Dictionary] = []

## Whether the party is currently selected. It starts selected because the party
## is the only unit on this board and an RTS that opens with nothing selected
## just costs the player a click; a left-click on the party's tile or on its
## card re-states it, and looking at another tile never takes it away.
var party_selected := true

## The tile the player last looked at, or "" for none. A look is not a
## selection: it outlines one tile and writes one status line, nothing else.
var inspected_cell_id := ""

var hovered_cell_id := ""

## Seconds of engine time accumulated while the ring is breathing. Presentation
## only: nothing on this board is read by the simulation, and the accumulator is
## frozen (and the ring drawn at rest) whenever reduced motion is on.
var ring_phase := 0.0
var reduced_motion := false


func _ready() -> void:
	reduced_motion = read_reduced_motion()
	set_process(not reduced_motion)


## Reduced motion belongs to the settings surface. This reads that surface's own
## file through that surface's own constants rather than inventing a second name
## for it. A player who has never opened settings gets the default, false.
func read_reduced_motion() -> bool:
	var config := ConfigFile.new()
	if config.load(SettingsPanel.SETTINGS_PATH) != OK:
		return false
	return bool(config.get_value(SettingsPanel.SECTION, "reduced_motion", false))


func _process(delta: float) -> void:
	if reduced_motion or not party_selected:
		return
	ring_phase = fmod(ring_phase + delta, RING_PULSE_SECONDS)
	queue_redraw()


func configure(next_catalog: ContentCatalog, snapshot: Dictionary) -> void:
	catalog = next_catalog
	active_location_id = str(snapshot.get("active_location_id", ""))
	legal_portal_ids.clear()
	for command in snapshot.get("legal_route_commands", []):
		var command_text := str(command)
		if command_text.begins_with("travel:"):
			legal_portal_ids[command_text.trim_prefix("travel:")] = true
	portals.clear()
	cell_names.clear()
	var region := catalog.get_record("world.region.black_beach")
	for cell_id in region.get("worldCellIds", []):
		var cell := catalog.get_record(str(cell_id))
		cell_names[str(cell_id)] = str(cell.get("displayName", cell_id))
		for portal in cell.get("portals", []):
			if portal is Dictionary:
				var record: Dictionary = portal.duplicate(true)
				record["from_location_id"] = str(cell.get("id", ""))
				portals.append(record)
	if inspected_cell_id == active_location_id:
		inspected_cell_id = ""
	queue_redraw()


# ---------------------------------------------------------------------------
# What the board knows about a tile
# ---------------------------------------------------------------------------

## The authored portal that leaves the active cell for `cell_id` and that the
## native snapshot currently calls legal, or "" when there is no such road.
##
## This is a lookup inside `legal_route_commands`, not a rule: the board never
## decides that a road is open, it only finds the one the simulation already
## named. A cell joined by a portal Rust did not list reads as unreachable here,
## which is exactly the intent.
func portal_from_active_cell_to(cell_id: String) -> String:
	for portal in portals:
		if str(portal.get("from_location_id", "")) != active_location_id:
			continue
		if str(portal.get("targetCellId", "")) != cell_id:
			continue
		var portal_id := str(portal.get("id", ""))
		if legal_portal_ids.has(portal_id):
			return portal_id
	return ""


func is_reachable(cell_id: String) -> bool:
	return not portal_from_active_cell_to(cell_id).is_empty()


func display_name_of(cell_id: String) -> String:
	return str(cell_names.get(cell_id, cell_id))


## The tile under a point in this control's coordinates, or "" for open ground.
## Where two glyphs overlap the nearer one wins, which is the one the player can
## actually see.
func tile_at_point(point: Vector2) -> String:
	var best_cell := ""
	var best_depth := -INF
	for cell_id in NODE_POSITIONS:
		var center := tile_center(str(cell_id))
		if not point_is_on_tile(point, center):
			continue
		if center.y > best_depth:
			best_depth = center.y
			best_cell = str(cell_id)
	return best_cell


func point_is_on_tile(point: Vector2, center: Vector2) -> bool:
	var offset := point - center
	if diamond_contains(offset):
		return true
	return diamond_contains(offset - Vector2(0.0, TILE_DEPTH))


func diamond_contains(offset: Vector2) -> bool:
	return absf(offset.x) / TILE_HALF_WIDTH + absf(offset.y) / TILE_HALF_HEIGHT <= 1.0


func island_rect() -> Rect2:
	return Rect2(size * Vector2(0.07, 0.11), size * Vector2(0.86, 0.77))


func tile_center(cell_id: String) -> Vector2:
	var island := island_rect()
	return island.position + island.size * (NODE_POSITIONS[cell_id] as Vector2)


# ---------------------------------------------------------------------------
# Orders
# ---------------------------------------------------------------------------

func _gui_input(event: InputEvent) -> void:
	var click := event as InputEventMouseButton
	if click != null and click.pressed:
		var cell_id := tile_at_point(click.position)
		if cell_id.is_empty():
			return
		if click.button_index == MOUSE_BUTTON_LEFT:
			accept_event()
			tile_selected.emit(cell_id)
		elif click.button_index == MOUSE_BUTTON_RIGHT:
			accept_event()
			move_ordered.emit(cell_id)
		return
	var motion := event as InputEventMouseMotion
	if motion != null:
		var hovered := tile_at_point(motion.position)
		if hovered != hovered_cell_id:
			hovered_cell_id = hovered
			queue_redraw()


## Called by the screen when the party is selected, from a click on its tile or
## on its card in the interface. The ring restarts at rest so the breath begins
## where the player's eye lands.
func set_party_selected(selected: bool) -> void:
	party_selected = selected
	ring_phase = 0.0
	queue_redraw()


func set_inspected_cell(cell_id: String) -> void:
	inspected_cell_id = cell_id
	queue_redraw()


# ---------------------------------------------------------------------------
# Drawing
# ---------------------------------------------------------------------------

func _draw() -> void:
	var frame := Rect2(Vector2.ZERO, size)
	draw_rect(frame, DEEP, true)
	draw_rect(frame.grow(-3.0), BRONZE, false, 2.0)
	var island := island_rect()
	draw_style_box(make_island_box(), island)
	draw_river(island)
	# Painter's order: a tile further down the board is nearer the eye and must
	# cover the one behind it.
	var drawn: Array[String] = []
	for cell_id in NODE_POSITIONS:
		drawn.append(str(cell_id))
	drawn.sort_custom(func(a: String, b: String) -> bool: return tile_center(a).y < tile_center(b).y)
	for cell_id in drawn:
		draw_cell(cell_id)
	draw_string(ThemeDB.fallback_font, Vector2(22, 30), "BLACK BEACH", HORIZONTAL_ALIGNMENT_LEFT, -1, 17, BRONZE)
	draw_string(ThemeDB.fallback_font, Vector2(22, size.y - 20), "LEFT-CLICK  LOOK      RIGHT-CLICK  MARCH      1-9  DEPARTURES", HORIZONTAL_ALIGNMENT_LEFT, -1, 12, MUTED)


func make_island_box() -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = LAND
	box.border_color = Color("496147")
	box.set_border_width_all(2)
	box.set_corner_radius_all(18)
	return box


## The river as a soft ribbon: four stacked passes, widest and faintest first,
## so the water has a bank that fades into the land instead of a hard edge.
func draw_river(island: Rect2) -> void:
	var points := PackedVector2Array([
		island.position + island.size * Vector2(0.48, 0.03),
		island.position + island.size * Vector2(0.505, 0.12),
		island.position + island.size * Vector2(0.52, 0.20),
		island.position + island.size * Vector2(0.485, 0.30),
		island.position + island.size * Vector2(0.47, 0.40),
		island.position + island.size * Vector2(0.505, 0.50),
		island.position + island.size * Vector2(0.535, 0.62),
		island.position + island.size * Vector2(0.52, 0.78),
		island.position + island.size * Vector2(0.50, 0.96)
	])
	draw_polyline(points, Color(RIVER, 0.20), 40.0, true)
	draw_polyline(points, Color(RIVER, 0.55), 27.0, true)
	draw_polyline(points, RIVER, 16.0, true)
	draw_polyline(points, Color(Color("68aab0"), 0.75), 3.0, true)


## One place on the island, as a small isometric tile.
##
## Three states, and they are the snapshot's three states: the tile the party
## stands on, a tile a legal portal reaches from there, and a tile it does not.
## Nothing here consults anything but `legal_route_commands`.
func draw_cell(cell_id: String) -> void:
	var center := tile_center(cell_id)
	var active := cell_id == active_location_id
	var reachable := is_reachable(cell_id)
	var top_color := LAND.lerp(TEAL, 0.5) if active else (LAND.lerp(BRONZE, 0.34) if reachable else LAND.lerp(DEEP, 0.4))
	var edge_color := CREAM if active else (BRONZE if reachable else MUTED)
	if cell_id == hovered_cell_id:
		top_color = top_color.lerp(CREAM, 0.12)
	var top := diamond_points(center)
	var body := body_points(center)
	draw_colored_polygon(body, top_color.darkened(0.55))
	draw_colored_polygon(top, top_color)
	draw_polyline(closed(top), Color(edge_color, 0.9 if (active or reachable) else 0.45), 2.0, true)
	if active and party_selected:
		draw_selection_ring(center)
	if cell_id == inspected_cell_id and not active:
		draw_ring(center + Vector2(0.0, TILE_DEPTH * 0.4), 1.2, Color(CREAM, 0.5), 1.5)
	if active:
		draw_party_marker(center)
	draw_cell_label(cell_id, center, active, reachable)


func diamond_points(center: Vector2) -> PackedVector2Array:
	return PackedVector2Array([
		center + Vector2(0.0, -TILE_HALF_HEIGHT),
		center + Vector2(TILE_HALF_WIDTH, 0.0),
		center + Vector2(0.0, TILE_HALF_HEIGHT),
		center + Vector2(-TILE_HALF_WIDTH, 0.0)
	])


func body_points(center: Vector2) -> PackedVector2Array:
	var depth := Vector2(0.0, TILE_DEPTH)
	return PackedVector2Array([
		center + Vector2(-TILE_HALF_WIDTH, 0.0),
		center + Vector2(0.0, TILE_HALF_HEIGHT),
		center + Vector2(TILE_HALF_WIDTH, 0.0),
		center + Vector2(TILE_HALF_WIDTH, 0.0) + depth,
		center + Vector2(0.0, TILE_HALF_HEIGHT) + depth,
		center + Vector2(-TILE_HALF_WIDTH, 0.0) + depth
	])


func closed(points: PackedVector2Array) -> PackedVector2Array:
	var loop := PackedVector2Array(points)
	loop.append(points[0])
	return loop


func draw_selection_ring(center: Vector2) -> void:
	var breath := 0.0
	if not reduced_motion:
		breath = sin(ring_phase / RING_PULSE_SECONDS * TAU) * RING_PULSE_AMPLITUDE
	var foot := center + Vector2(0.0, TILE_DEPTH * 0.4)
	draw_ring(foot, 1.26 + breath, Color(TEAL, 0.85), 2.0)
	draw_ring(foot, 1.36 + breath, Color(TEAL, 0.28), 1.0)


## An isometric ring: the tile's own diamond proportions, swept as an ellipse.
func draw_ring(center: Vector2, ring_scale: float, color: Color, width: float) -> void:
	var radius := Vector2(TILE_HALF_WIDTH, TILE_HALF_HEIGHT) * ring_scale
	var points := PackedVector2Array()
	for step in 49:
		var angle := TAU * float(step) / 48.0
		points.append(center + Vector2(cos(angle) * radius.x, sin(angle) * radius.y))
	draw_polyline(points, color, width, true)


## Michael and Betty, as the two standing marks an RTS puts on the selected
## unit's tile. Procedural placeholder geometry, drawn in code and generated
## nowhere else: it is replaced by the owner's models when they arrive.
func draw_party_marker(center: Vector2) -> void:
	for offset in [Vector2(-9.0, -1.0), Vector2(9.0, 3.0)]:
		var foot: Vector2 = center + offset
		draw_line(foot, foot + Vector2(0.0, -15.0), Color(CREAM, 0.9), 3.0, true)
		draw_circle(foot + Vector2(0.0, -19.0), 4.0, CREAM)


func draw_cell_label(cell_id: String, center: Vector2, active: bool, reachable: bool) -> void:
	var font := ThemeDB.fallback_font
	var label := display_name_of(cell_id).to_upper()
	var font_size := 12
	var width := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
	var origin := center + Vector2(-width * 0.5, TILE_HALF_HEIGHT + TILE_DEPTH + 22.0)
	# A plate under the name, so a place stays readable where the river or a
	# neighbouring tile runs behind it.
	draw_rect(Rect2(origin + Vector2(-8.0, -13.0), Vector2(width + 16.0, 19.0)), Color(DEEP, 0.72), true)
	draw_string(font, origin, label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, CREAM if active else (Color(CREAM, 0.82) if reachable else MUTED))
	if cell_id == "world.cell.reception_terrace":
		draw_string(font, center + Vector2(-58.0, -TILE_HALF_HEIGHT - 16.0), "RAZORBEAK TERRITORY", HORIZONTAL_ALIGNMENT_CENTER, 116, 10, DANGER)
