class_name ExpeditionRouteBoard
extends Control

## Read-only, data-driven route board for the Black Beach chapter. It does not
## decide where travel is legal. The native expedition snapshot supplies that
## fact; this node only draws the known cell graph and tints the tiles the party
## can reach from where it stands.
##
## B18, the owner's direction: this is an RTS under the hood, so it takes
## RTS-style orders. The drawn route markers are gone -- no portal lines, no
## teal dot halfway along each of them, no legend -- because a marker is a second
## drawing of a fact the tile itself already carries. What the snapshot says is
## "these portals are legal now", and that reads on the board as "these tiles
## are reachable", tinted once. Clicking is the grammar: left-click selects a
## tile and never moves, right-click orders a move to it. The board issues both
## as signals and judges neither: the screen above it owns the order and the
## native legal-command list owns the legality.

## Left-click: this tile is now the selection. The party's own tile means the
## party is selected; any other tile is an inspected target.
signal tile_selected(cell_id: String)

## Right-click: move the party to this tile. Nothing here has checked whether
## that is possible -- the screen asks the native legal-command list.
signal move_ordered(cell_id: String)

const DEEP := Color("0b1514")
const LAND := Color("18352b")
const RIVER := Color("235b68")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("52625b")
const DANGER := Color("c24e45")

const NODE_POSITIONS := {
	"world.cell.black_beach": Vector2(0.16, 0.72),
	"world.cell.damaged_estate": Vector2(0.31, 0.48),
	"world.cell.river_landing": Vector2(0.51, 0.36),
	"world.cell.reception_terrace": Vector2(0.73, 0.52),
	"world.cell.processional_ramp": Vector2(0.86, 0.26)
}

## How near a click must land to a tile's centre to count as that tile. Tiles
## draw at radius 15 (the party's) or 10, and the label sits below; this is the
## generous hit target an order deserves, and it is still well under half the
## closest distance between two node positions.
const TILE_CLICK_RADIUS := 34.0

var catalog: ContentCatalog
var active_location_id := ""
var legal_portal_ids: Dictionary = {}
var cell_names: Dictionary = {}
var portals: Array[Dictionary] = []
var selected_cell_id := ""

## Target cell ID -> the legal portal that reaches it from the active cell. Not
## a legality rule of its own: it is the native `legal_route_commands` list,
## re-keyed by where each of those portals lands, so a click on a tile can name
## one of them.
var reachable_portal_ids: Dictionary = {}


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
	rebuild_reachability()
	if not NODE_POSITIONS.has(selected_cell_id):
		selected_cell_id = active_location_id
	queue_redraw()


## The legal departures, seen from the tiles they arrive at. One portal per
## destination: the authored graph carries no two legal roads from one cell to
## the same cell, and were that ever authored, the first in authored order would
## answer a tile order while both stay in the route list -- which is the place a
## road is chosen by name.
func rebuild_reachability() -> void:
	reachable_portal_ids.clear()
	for portal in portals:
		if str(portal.get("from_location_id", "")) != active_location_id:
			continue
		var portal_id := str(portal.get("id", ""))
		if not legal_portal_ids.has(portal_id):
			continue
		var target_id := str(portal.get("targetCellId", ""))
		if target_id.is_empty() or reachable_portal_ids.has(target_id):
			continue
		reachable_portal_ids[target_id] = portal_id


## The legal portal that reaches `cell_id` from where the party stands, or ""
## when no legal departure lands there.
func portal_to_cell(cell_id: String) -> String:
	return str(reachable_portal_ids.get(cell_id, ""))


func is_reachable(cell_id: String) -> bool:
	return reachable_portal_ids.has(cell_id)


func set_selected_cell(cell_id: String) -> void:
	selected_cell_id = cell_id
	queue_redraw()


func _gui_input(event: InputEvent) -> void:
	var click := event as InputEventMouseButton
	if click == null or not click.pressed:
		return
	if click.button_index != MOUSE_BUTTON_LEFT and click.button_index != MOUSE_BUTTON_RIGHT:
		return
	var cell_id := tile_at_point(click.position)
	if cell_id.is_empty():
		return
	accept_event()
	if click.button_index == MOUSE_BUTTON_LEFT:
		tile_selected.emit(cell_id)
		return
	move_ordered.emit(cell_id)


## The tile under a point in this control's own coordinates, or "" for open
## water. The nearest centre within the click radius wins, so two hit circles
## that ever overlapped could still not make a click ambiguous.
func tile_at_point(point: Vector2) -> String:
	var island := island_rect()
	var best_id := ""
	var best_distance := TILE_CLICK_RADIUS
	for cell_id in NODE_POSITIONS:
		var center: Vector2 = island.position + island.size * (NODE_POSITIONS[cell_id] as Vector2)
		var distance := center.distance_to(point)
		if distance <= best_distance:
			best_distance = distance
			best_id = str(cell_id)
	return best_id


## The one definition of where the island sits inside this control. Drawing and
## hit-testing both read it here, so a click can never land somewhere other than
## what the player is looking at.
func island_rect() -> Rect2:
	return Rect2(size * Vector2(0.07, 0.11), size * Vector2(0.86, 0.77))


func _draw() -> void:
	var frame := Rect2(Vector2.ZERO, size)
	draw_rect(frame, DEEP, true)
	draw_rect(frame.grow(-3.0), BRONZE, false, 2.0)
	var island := island_rect()
	draw_style_box(make_island_box(), island)
	draw_river(island)
	for cell_id in NODE_POSITIONS:
		draw_cell(island, str(cell_id))
	draw_string(ThemeDB.fallback_font, Vector2(22, 30), "BLACK BEACH ROUTE BOARD", HORIZONTAL_ALIGNMENT_LEFT, -1, 17, BRONZE)


func make_island_box() -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = LAND
	box.border_color = Color("496147")
	box.set_border_width_all(2)
	box.set_corner_radius_all(18)
	return box


func draw_river(island: Rect2) -> void:
	var points := PackedVector2Array([
		island.position + island.size * Vector2(0.48, 0.03),
		island.position + island.size * Vector2(0.52, 0.18),
		island.position + island.size * Vector2(0.47, 0.39),
		island.position + island.size * Vector2(0.54, 0.60),
		island.position + island.size * Vector2(0.50, 0.96)
	])
	draw_polyline(points, RIVER, 25.0, true)
	draw_polyline(points, Color("68aab0"), 3.0, true)


func draw_cell(island: Rect2, cell_id: String) -> void:
	var center: Vector2 = island.position + island.size * (NODE_POSITIONS[cell_id] as Vector2)
	var active := cell_id == active_location_id
	var reachable := is_reachable(cell_id)
	var lit := active or reachable
	draw_circle(center, 18.0 if active else 13.0, Color("07100f"))
	draw_circle(center, 15.0 if active else 10.0, TEAL if lit else MUTED)
	draw_circle(center, 8.0 if active else 5.0, Color("e7d9ad") if active else Color("23342e"))
	if cell_id == selected_cell_id:
		draw_arc(center, 24.0 if active else 19.0, 0.0, TAU, 40, CREAM, 2.0, true)
	var label := str(cell_names.get(cell_id, cell_id)).to_upper()
	var label_color := MUTED
	if active:
		label_color = CREAM
	elif reachable:
		label_color = TEAL
	draw_string(ThemeDB.fallback_font, center + Vector2(-62, 36), label, HORIZONTAL_ALIGNMENT_CENTER, 124, 12, label_color)
	if cell_id == "world.cell.reception_terrace":
		draw_string(ThemeDB.fallback_font, center + Vector2(-58, -27), "RAZORBEAK TERRITORY", HORIZONTAL_ALIGNMENT_CENTER, 116, 10, DANGER)
