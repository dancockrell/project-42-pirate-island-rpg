class_name ExpeditionRouteBoard
extends Control

## Read-only, data-driven route board for the Black Beach chapter. It does not
## decide where travel is legal. The native expedition snapshot supplies that
## fact; this node only draws the known cell graph and highlights current exits.

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

var catalog: ContentCatalog
var active_location_id := ""
var legal_portal_ids: Dictionary = {}
var cell_names: Dictionary = {}
var portals: Array[Dictionary] = []


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
	queue_redraw()


func _draw() -> void:
	var frame := Rect2(Vector2.ZERO, size)
	draw_rect(frame, DEEP, true)
	draw_rect(frame.grow(-3.0), BRONZE, false, 2.0)
	var island := Rect2(size * Vector2(0.07, 0.11), size * Vector2(0.86, 0.77))
	draw_style_box(make_island_box(), island)
	draw_river(island)
	for portal in portals:
		draw_portal(island, portal)
	for cell_id in NODE_POSITIONS:
		draw_cell(island, str(cell_id))
	draw_string(ThemeDB.fallback_font, Vector2(22, 30), "BLACK BEACH ROUTE BOARD", HORIZONTAL_ALIGNMENT_LEFT, -1, 17, BRONZE)
	draw_string(ThemeDB.fallback_font, Vector2(22, size.y - 20), "Solid teal route = legal now   •   bronze route = known connection", HORIZONTAL_ALIGNMENT_LEFT, -1, 12, CREAM)


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


func draw_portal(island: Rect2, portal: Dictionary) -> void:
	var from_id := str(portal.get("from_location_id", ""))
	var target_id := str(portal.get("targetCellId", ""))
	if not NODE_POSITIONS.has(from_id) or not NODE_POSITIONS.has(target_id):
		return
	var from: Vector2 = island.position + island.size * (NODE_POSITIONS[from_id] as Vector2)
	var target: Vector2 = island.position + island.size * (NODE_POSITIONS[target_id] as Vector2)
	var legal := legal_portal_ids.has(str(portal.get("id", "")))
	draw_line(from, target, TEAL if legal else BRONZE, 5.0 if legal else 2.0, true)
	if legal:
		var midpoint: Vector2 = from.lerp(target, 0.5)
		draw_circle(midpoint, 5.5, TEAL)


func draw_cell(island: Rect2, cell_id: String) -> void:
	var center: Vector2 = island.position + island.size * (NODE_POSITIONS[cell_id] as Vector2)
	var active := cell_id == active_location_id
	var tint := TEAL if active else BRONZE
	draw_circle(center, 18.0 if active else 13.0, Color("07100f"))
	draw_circle(center, 15.0 if active else 10.0, tint)
	draw_circle(center, 8.0 if active else 5.0, Color("e7d9ad") if active else Color("23342e"))
	var label := str(cell_names.get(cell_id, cell_id)).to_upper()
	draw_string(ThemeDB.fallback_font, center + Vector2(-62, 36), label, HORIZONTAL_ALIGNMENT_CENTER, 124, 12, CREAM if active else MUTED)
	if cell_id == "world.cell.reception_terrace":
		draw_string(ThemeDB.fallback_font, center + Vector2(-58, -27), "RAZORBEAK TERRITORY", HORIZONTAL_ALIGNMENT_CENTER, 116, 10, DANGER)
