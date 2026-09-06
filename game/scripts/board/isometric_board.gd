class_name IsometricBoard
extends Node3D

## B12: one scene graph, three distances -- world, route, room.
##
## The board draws. It does not decide. Every fact on screen comes from one of
## two places and from nowhere else:
##
## * the **content catalog**, through `NativeExpeditionPort.board_for`, for what
##   a room is: its footprint, its spawn sockets, its tethers, where it stands
##   on the island. B11 authored those and this reads them;
## * the **snapshot**, for what is true right now: `active_location_id` for
##   where the party is, `route_options` for which roads are legal and what they
##   are worth today, `forces` for who is marching, and `controller_of` for who
##   holds a cell.
##
## The party miniature snaps to `active_location_id` on every snapshot and moves
## on nothing else. There is no click-to-move here: a click cannot move the
## party, because a click is not a confirmed travel and the simulation is the
## only thing that says the party has gone anywhere. B18/P7 owns selection and
## orders; when it lands it issues travel through `CampaignSession` and this
## board hears about it through the next snapshot, exactly as it hears about
## everything else.
##
## This board does not replace `expedition_route_board.gd` in this pass. That
## 2D board is still the one the expedition screen draws; a later card retires
## it against this one rather than leaving two boards drawing the island.

const DISTANCE_WORLD := "world"
const DISTANCE_ROUTE := "route"
const DISTANCE_ROOM := "room"
const DISTANCES := [DISTANCE_WORLD, DISTANCE_ROUTE, DISTANCE_ROOM]

## The region whose cells this board draws. One region is authored.
const REGION_ID := "world.region.black_beach"

## **needs decision.** How much smaller a force miniature stands than the party's.
## A framing choice: the party is the one the player follows.
const FORCE_MINIATURE_SCALE := 0.72
## **needs decision.** How far above a tile a miniature's feet sit, in metres.
const MINIATURE_LIFT_METRES := 0.4
## **needs decision.** How far above the terrace a drawn road rides, in metres.
const ROUTE_LIFT_METRES := 2.4
## **needs decision.** How far apart two roads between the same two rooms are
## drawn, in metres.
const ROUTE_PARALLEL_SPACING_METRES := 7.0
## **needs decision.** How wide the encounter space a battle entry marks is, in
## metres. Brief section 4 requires a room to declare one; how big it is is a
## design decision nobody has taken.
const ENCOUNTER_SPACE_RADIUS_METRES := 7.0
## **needs decision.** Where the sea stands, and how high the coastal shelf that
## carries the rooms sits above it. Both in metres, both framing choices: the
## shelf must be below the lowest room's floor, and the sea below the shelf, or
## the island stops reading as an island.
const SEA_LEVEL_METRES := -7.0
const SHELF_TOP_METRES := -1.5
## **needs decision.** How far the coastal shelf reaches past a room's footprint,
## as a multiple of it, and how wide the shelf is along a road, in metres.
const SHELF_MARGIN := 2.1
const SHELF_STRIP_WIDTH_METRES := 46.0

var catalog: ContentCatalog
## The autoloaded campaign owner, when one is in the tree. The board asks it for
## nothing but the port it holds and the snapshots it produces.
var campaign_session: Node
var port: NativeExpeditionPort
var distance := DISTANCE_WORLD
var snapshot: Dictionary = {}

## cell id -> the board block `board_for` returned. Built once; content does not
## change while the game runs.
var cells: Dictionary = {}
## portal id -> { from_cell_id, to_cell_id, from_point, to_point }.
var tethers: Dictionary = {}

var camera: BoardCamera
var terrain_root: Node3D
var route_root: Node3D
var miniature_root: Node3D
var room_root: Node3D
var party_miniature: Node3D
var force_miniatures: Dictionary = {}
## The port used purely as the catalog's board reader when no campaign is live.
var reader: NativeExpeditionPort


func _ready() -> void:
	if catalog == null:
		catalog = ContentCatalog.new()
		catalog.load_default()
	if campaign_session == null:
		campaign_session = get_tree().root.get_node_or_null("CampaignSession")
	if port == null and campaign_session != null:
		campaign_session.begin_if_needed(catalog)
		port = campaign_session.expedition
	build_scene_graph()
	if port != null:
		project_snapshot(campaign_session.latest_snapshot)
	else:
		# Content alone is enough to stand the island up; only the live facts
		# are missing. Drawing the rooms with no campaign is honest -- and it is
		# what the capture path uses when no bridge is present.
		apply_distance()


## The one build. Rooms, roads and sockets are content, so they are made once.
func build_scene_graph() -> void:
	camera = BoardCamera.new()
	camera.name = "BoardCamera"
	add_child(camera)
	add_child(make_environment())
	add_child(make_key_light())
	add_child(make_fill_light())
	terrain_root = make_layer("Terrain")
	route_root = make_layer("Routes")
	room_root = make_layer("Room")
	miniature_root = make_layer("Miniatures")
	load_cells()
	build_sea()
	for cell_id in sorted_cell_ids():
		BoardBlockout.tile(terrain_root, cells[cell_id], BoardPalette.CLAY)
	build_tethers()
	build_shelf()
	space_parallel_tethers()
	build_causeways()
	party_miniature = BoardBlockout.miniature(miniature_root, "PartyMiniature", BoardPalette.CREAM)


func make_layer(layer_name: String) -> Node3D:
	var layer := Node3D.new()
	layer.name = layer_name
	add_child(layer)
	return layer


## Reads every cell of the authored region through the port. Sorted, so the
## scene graph is built in the same order every run.
func load_cells() -> void:
	cells.clear()
	var region := catalog.get_record(REGION_ID)
	for cell_id in region.get("worldCellIds", []):
		var block := board_block(str(cell_id))
		if block.is_empty():
			push_error("World cell %s carries no board block; B11 requires one." % cell_id)
			continue
		cells[str(cell_id)] = block


## The board block for one cell. `NativeExpeditionPort.board_for` is the one
## reader of the authored block; without a live port the board falls back to the
## same port class used purely as that reader, so there is still one resolution
## of the footprint and no second copy of the shape.
func board_block(cell_id: String) -> Dictionary:
	if port != null:
		return port.board_for(cell_id)
	if reader == null:
		reader = NativeExpeditionPort.new()
		reader.catalog = catalog
	return reader.board_for(cell_id)


func sorted_cell_ids() -> Array:
	var ids := cells.keys()
	ids.sort()
	return ids


## Every portal of every loaded cell, as a segment from this room's tether
## anchor to the target room's tether anchor back. One tether per portal is
## B11's rule and the validator holds it, so a missing one is a content error
## rather than something to invent a line for.
func build_tethers() -> void:
	tethers.clear()
	for cell_id in sorted_cell_ids():
		var cell: Dictionary = cells[cell_id]
		var centre := BoardBlockout.centre_of(cell)
		for tether in cell.get("tethers", []):
			var portal_id := str(tether.get("portal_id", ""))
			var anchor: Vector3 = tether.get("anchor_metres", Vector3.ZERO)
			var to_cell_id := portal_target(portal_id)
			if to_cell_id.is_empty() or not cells.has(to_cell_id):
				continue
			var target_cell: Dictionary = cells[to_cell_id]
			tethers[portal_id] = {
				"from_cell_id": cell_id,
				"to_cell_id": to_cell_id,
				"kind": str(tether.get("kind", "")),
				"from_point": centre + anchor,
				"to_point": BoardBlockout.centre_of(target_cell) + facing_anchor(target_cell, centre)
			}


## Two roads between the same pair of rooms leave by the same authored gate --
## the river landing's safe road and its jungle edge both leave by the reception
## road -- so their tethers are the same segment. They are drawn side by side
## rather than on top of each other: one line for two roads is one road the
## player cannot choose between, and coincident geometry flickers besides.
func space_parallel_tethers() -> void:
	var by_pair: Dictionary = {}
	var portal_ids := tethers.keys()
	portal_ids.sort()
	for portal_id in portal_ids:
		var tether: Dictionary = tethers[portal_id]
		var pair: Array = [str(tether["from_cell_id"]), str(tether["to_cell_id"])]
		pair.sort()
		var key := "%s|%s" % pair
		if not by_pair.has(key):
			by_pair[key] = []
		(by_pair[key] as Array).append(portal_id)
	for key in by_pair:
		var group: Array = by_pair[key]
		for index in group.size():
			var tether: Dictionary = tethers[group[index]]
			var span: Vector3 = (tether["to_point"] as Vector3) - (tether["from_point"] as Vector3)
			var sideways := Vector3(span.z, 0.0, -span.x).normalized()
			tether["parallel_offset"] = sideways * (float(index) - float(group.size() - 1) * 0.5) * ROUTE_PARALLEL_SPACING_METRES


## Which cell a portal reaches, read from the authored cell that declares it.
func portal_target(portal_id: String) -> String:
	for cell_id in sorted_cell_ids():
		var record := catalog.get_record(cell_id)
		for portal in record.get("portals", []):
			if str(portal.get("id", "")) == portal_id:
				return str(portal.get("targetCellId", ""))
	return ""


## The tether anchor of `cell` that faces `from_point`. A road arrives at the
## room's own gate rather than at its middle; when the far room declares no
## tether back (a one-way authored portal) the road ends at its edge, computed
## the same way B11's anchors are.
func facing_anchor(cell: Dictionary, from_point: Vector3) -> Vector3:
	var centre := BoardBlockout.centre_of(cell)
	var toward := Vector2(from_point.x - centre.x, from_point.z - centre.z)
	var best := Vector3.ZERO
	var best_dot := -1.0
	for tether in cell.get("tethers", []):
		var anchor: Vector3 = tether.get("anchor_metres", Vector3.ZERO)
		var dot := Vector2(anchor.x, anchor.z).normalized().dot(toward.normalized())
		if dot > best_dot:
			best_dot = dot
			best = anchor
	if best_dot > 0.0:
		return best
	var footprint: Dictionary = cell.get("footprint", {})
	var half := Vector2(float(footprint.get("width_metres", 2.0)) * 0.5, float(footprint.get("depth_metres", 2.0)) * 0.5)
	var direction := toward.normalized()
	var scale_value := minf(
		half.x / maxf(absf(direction.x), 0.0001),
		half.y / maxf(absf(direction.y), 0.0001)
	)
	return Vector3(direction.x * scale_value, 0.0, direction.y * scale_value)


## The sea the island stands in: one plane under everything, so the world
## distance reads as an island rather than as tiles floating in a void.
func build_sea() -> void:
	var bounds := island_bounds()
	var mesh := PlaneMesh.new()
	mesh.size = bounds.size * 7.0
	mesh.material = BoardBlockout.clay(BoardPalette.SEA, 0.08)
	var plane := MeshInstance3D.new()
	plane.name = "Sea"
	plane.mesh = mesh
	plane.position = Vector3(bounds.get_center().x, SEA_LEVEL_METRES, bounds.get_center().y)
	BoardBlockout.mark(plane, "P2's stylised water material and the island's baked terrain")
	terrain_root.add_child(plane)


## The island's coastal shelf: one apron of land around each room and one along
## each road between rooms. Overlapping aprons make a coastline that follows
## where the island is actually inhabited, instead of a plate the rooms sit in
## the middle of -- the shape of the island is the shape of the route graph,
## which is what "one node equals one playable room" means on a map.
func build_shelf() -> void:
	var top := SHELF_TOP_METRES
	var height := top + BoardBlockout.SEA_FLOOR_METRES * 1.6
	for cell_id in sorted_cell_ids():
		var cell: Dictionary = cells[cell_id]
		var footprint: Dictionary = cell.get("footprint", {})
		var centre := BoardBlockout.centre_of(cell)
		var apron := SetpieceMeshFactory.box(
			terrain_root,
			BoardBlockout.node_name("Apron_", cell_id, "world.cell."),
			Vector3(float(footprint.get("width_metres", 1.0)) * SHELF_MARGIN, height, float(footprint.get("depth_metres", 1.0)) * SHELF_MARGIN),
			Vector3(centre.x, top - height * 0.5, centre.z),
			BoardBlockout.clay(BoardBlockout.LAND)
		)
		BoardBlockout.mark(apron, "the island's baked terrain and coastline around %s" % cell_id)
	var portal_ids := tethers.keys()
	portal_ids.sort()
	for portal_id in portal_ids:
		var tether: Dictionary = tethers[portal_id]
		var from: Vector3 = tether["from_point"]
		var to: Vector3 = tether["to_point"]
		var flat_from := Vector3(from.x, top, from.z)
		var flat_to := Vector3(to.x, top, to.z)
		var span := maxf(flat_from.distance_to(flat_to), 0.01)
		var strip := SetpieceMeshFactory.box(
			terrain_root,
			BoardBlockout.node_name("Shelf_", str(portal_id), "world.portal."),
			Vector3(SHELF_STRIP_WIDTH_METRES, height, span),
			Vector3.ZERO,
			BoardBlockout.clay(BoardBlockout.LAND)
		)
		strip.position = flat_from.lerp(flat_to, 0.5) - Vector3(0.0, height * 0.5, 0.0)
		strip.look_at(Vector3(flat_to.x, strip.global_position.y, flat_to.z), Vector3.UP)
		BoardBlockout.mark(strip, "the island's baked terrain between two rooms")


## The land between the rooms. Built once, from the tethers, because a causeway
## is where a road *can* run and not where one is legal today: an island does not
## rearrange itself when a door opens.
func build_causeways() -> void:
	var portal_ids := tethers.keys()
	portal_ids.sort()
	var drawn: Dictionary = {}
	for portal_id in portal_ids:
		var tether: Dictionary = tethers[portal_id]
		var pair: Array = [str(tether["from_cell_id"]), str(tether["to_cell_id"])]
		pair.sort()
		var key := "%s|%s" % pair
		if drawn.has(key):
			continue
		drawn[key] = true
		BoardBlockout.causeway(terrain_root, BoardBlockout.node_name("Causeway_", str(portal_id), "world.portal."), tether["from_point"], tether["to_point"], BoardBlockout.ROCK)


## The rectangle every room's footprint fits inside, on the ground plane.
func island_bounds() -> Rect2:
	var bounds := Rect2()
	var first := true
	for cell_id in sorted_cell_ids():
		var cell: Dictionary = cells[cell_id]
		var island: Vector2 = cell.get("island_position_metres", Vector2.ZERO)
		var footprint: Dictionary = cell.get("footprint", {})
		var half := Vector2(float(footprint.get("width_metres", 0.0)) * 0.5, float(footprint.get("depth_metres", 0.0)) * 0.5)
		var box := Rect2(island - half, half * 2.0)
		bounds = box if first else bounds.merge(box)
		first = false
	return bounds


## One snapshot, drawn. Called on every snapshot the campaign produces.
func project_snapshot(next_snapshot: Dictionary) -> void:
	snapshot = next_snapshot.duplicate(true)
	apply_ownership()
	apply_routes()
	apply_party()
	apply_forces()
	apply_distance()


## S2's control, drawn. `controller_of` is the only answer to who holds a cell;
## the board never reads a raw ownership map and never derives a controller.
func apply_ownership() -> void:
	for cell_id in sorted_cell_ids():
		var tile := terrain_root.get_node_or_null(BoardBlockout.node_name("Tile_", cell_id, "world.cell."))
		if tile == null:
			continue
		var controller := "" if port == null else port.controller_of(cell_id)
		(tile as MeshInstance3D).mesh.material = BoardBlockout.clay(BoardPalette.controller_tint(controller))


## Every known road drawn, and the legal ones drawn by the danger the snapshot
## says they carry today. The risk is `route_options[].risk_level`, which is
## `Geography::effective_risk` -- so a road that became contested because a
## faction took one of its ends changes colour here without this file knowing
## what "contested" means.
func apply_routes() -> void:
	for child in route_root.get_children():
		child.queue_free()
	var risk_by_portal: Dictionary = {}
	for option in snapshot.get("route_options", []):
		risk_by_portal[str((option as Dictionary).get("portal_id", ""))] = int((option as Dictionary).get("risk_level", 0))
	var portal_ids := tethers.keys()
	portal_ids.sort()
	for portal_id in portal_ids:
		var tether: Dictionary = tethers[portal_id]
		var legal: bool = risk_by_portal.has(portal_id)
		var tint: Color = BoardPalette.risk_tint(int(risk_by_portal.get(portal_id, 0))) if legal else BoardPalette.MUTED
		var from: Vector3 = (tether["from_point"] as Vector3) + Vector3(0.0, ROUTE_LIFT_METRES, 0.0) + (tether["parallel_offset"] as Vector3)
		var to: Vector3 = (tether["to_point"] as Vector3) + Vector3(0.0, ROUTE_LIFT_METRES, 0.0) + (tether["parallel_offset"] as Vector3)
		BoardBlockout.route(route_root, BoardBlockout.node_name("Route_", str(portal_id), "world.portal."), from, to, tint, 2.4 if legal else 1.0)


## The party miniature snaps to the cell the snapshot says the party is in.
## This is the whole of the party's motion rule.
func apply_party() -> void:
	var cell_id := party_cell_id()
	if cell_id.is_empty() or not cells.has(cell_id):
		return
	party_miniature.position = stand_point(cells[cell_id])


## Where the snapshot says the party is.
func party_cell_id() -> String:
	return str(snapshot.get("active_location_id", ""))


## S7's forces, at the route distance: each stands in the cell it occupies, or
## partway along the tether it is marching down, by the progress the snapshot
## carries. Nothing here advances a force: a miniature between two cells is a
## reading of `progress`, not a tween the board invented.
func apply_forces() -> void:
	var seen: Dictionary = {}
	for entry in snapshot.get("forces", []):
		var force: Dictionary = entry
		var force_id := str(force.get("id", ""))
		var position_cell_id := str(force.get("position_cell_id", ""))
		if not cells.has(position_cell_id):
			continue
		seen[force_id] = true
		if not force_miniatures.has(force_id):
			force_miniatures[force_id] = BoardBlockout.miniature(
				miniature_root,
				BoardBlockout.node_name("Force_", force_id, "force."),
				BoardPalette.faction_colour(str(force.get("faction_id", ""))),
				FORCE_MINIATURE_SCALE
			)
		var here := stand_point(cells[position_cell_id])
		var next_cell_id := str(force.get("next_cell_id", ""))
		var point := here
		if cells.has(next_cell_id):
			point = here.lerp(stand_point(cells[next_cell_id]), clampf(float(force.get("progress", 0.0)), 0.0, 1.0))
		(force_miniatures[force_id] as Node3D).position = point
	for force_id in force_miniatures.keys():
		(force_miniatures[force_id] as Node3D).visible = seen.has(force_id)


## Where a miniature stands in a cell: on the tile, at its centre.
func stand_point(cell: Dictionary) -> Vector3:
	return BoardBlockout.centre_of(cell) + Vector3(0.0, MINIATURE_LIFT_METRES, 0.0)


## Switch distance. The scene graph is one graph: a distance decides what is
## visible and where the fixed camera stands, never which board is loaded.
func set_distance(next_distance: String) -> void:
	if not DISTANCES.has(next_distance):
		push_error("Unknown board distance %s." % next_distance)
		return
	distance = next_distance
	apply_distance()


func apply_distance() -> void:
	var focus_cell_id := party_cell_id()
	if not cells.has(focus_cell_id):
		focus_cell_id = sorted_cell_ids()[0] if not cells.is_empty() else ""
	room_root.visible = distance == DISTANCE_ROOM
	terrain_root.visible = distance != DISTANCE_ROOM
	route_root.visible = distance != DISTANCE_ROOM
	miniature_root.visible = distance != DISTANCE_ROOM
	match distance:
		DISTANCE_ROOM:
			build_room(focus_cell_id)
			camera.frame(distance, BoardBlockout.centre_of(cells[focus_cell_id]) if cells.has(focus_cell_id) else Vector3.ZERO)
		DISTANCE_ROUTE:
			camera.frame(distance, BoardBlockout.centre_of(cells[focus_cell_id]) if cells.has(focus_cell_id) else Vector3.ZERO)
		_:
			var bounds := island_bounds()
			camera.frame(distance, Vector3(bounds.get_center().x, 12.0, bounds.get_center().y))


## The room distance: one cell's footprint, its spawn sockets by role, the gates
## its tethers leave by, and the encounter space its battle entry declares.
## Brief section 4 asks a room for defined entrances and exits, spawn points and
## an encounter space; this is those three, drawn.
func build_room(cell_id: String) -> void:
	for child in room_root.get_children():
		room_root.remove_child(child)
		child.queue_free()
	if not cells.has(cell_id):
		return
	var cell: Dictionary = cells[cell_id]
	var centre := BoardBlockout.centre_of(cell)
	var footprint: Dictionary = cell.get("footprint", {})
	var width := float(footprint.get("width_metres", 1.0))
	var depth := float(footprint.get("depth_metres", 1.0))
	var floor_slab := SetpieceMeshFactory.box(room_root, "RoomFloor", Vector3(width, BoardBlockout.TILE_THICKNESS_METRES, depth), centre, BoardBlockout.clay(BoardPalette.CLAY))
	BoardBlockout.mark(floor_slab, "the cell's baked visual shell (%s)" % cell_id)
	# The footprint's own edge, so the declared box is visible as a box rather
	# than implied. Nothing essential may extend outside it (brief section 18).
	for edge in [Vector3(0, 0, depth * 0.5), Vector3(0, 0, -depth * 0.5), Vector3(width * 0.5, 0, 0), Vector3(-width * 0.5, 0, 0)]:
		var along_x := absf(edge.z) > 0.0
		var kerb := SetpieceMeshFactory.box(
			room_root,
			"RoomEdge%d_%d" % [int(edge.x), int(edge.z)],
			Vector3(width if along_x else 1.1, 2.4, 1.1 if along_x else depth),
			centre + edge + Vector3(0.0, 1.5, 0.0),
			BoardBlockout.clay(BoardPalette.MUTED)
		)
		BoardBlockout.mark(kerb, "the room's authored boundary geometry")
	for spawn in cell.get("spawn_points", []):
		var role := str((spawn as Dictionary).get("role", ""))
		var tint: Color = BoardPalette.ROLE_TINT.get(role, BoardPalette.MUTED)
		BoardBlockout.socket(room_root, BoardBlockout.node_name("Socket_", str((spawn as Dictionary).get("id", "")), "spawn."), centre + ((spawn as Dictionary).get("position_metres", Vector3.ZERO) as Vector3) + Vector3(0.0, BoardBlockout.TILE_THICKNESS_METRES * 0.5, 0.0), tint, float(BoardPalette.ROLE_POST_HEIGHT.get(role, 0.0)))
	for tether in cell.get("tethers", []):
		var portal_id := str((tether as Dictionary).get("portal_id", ""))
		var anchor: Vector3 = (tether as Dictionary).get("anchor_metres", Vector3.ZERO)
		# Two roads that leave by the same authored gate are spaced apart here
		# the same way they are spaced apart on the world board, and for the same
		# reason: a room with three ways out must show three.
		var offset: Vector3 = (tethers[portal_id] as Dictionary).get("parallel_offset", Vector3.ZERO) if tethers.has(portal_id) else Vector3.ZERO
		var gate := SetpieceMeshFactory.box(
			room_root,
			BoardBlockout.node_name("Exit_", portal_id, "world.portal."),
			Vector3(2.6, 3.8, 2.6),
			centre + anchor + offset + Vector3(0.0, 1.9 + BoardBlockout.TILE_THICKNESS_METRES * 0.5, 0.0),
			BoardBlockout.clay(BoardPalette.BRONZE, 0.28)
		)
		BoardBlockout.mark(gate, "the authored exit's baked gate geometry")
	# The encounter space: where the cell's battle entry says a fight stands. The
	# lens that opens when the party enters it is O1's and is not decided here;
	# `EncounterLens` is the stand-in and says so.
	var record := catalog.get_record(cell_id)
	for battle_entry in record.get("battleEntries", []):
		var position := NativeExpeditionPort.as_vector3((battle_entry as Dictionary).get("worldPositionMetres", []))
		var ring := BoardBlockout.encounter_ring(
			room_root,
			BoardBlockout.node_name("EncounterSpace_", str((battle_entry as Dictionary).get("id", "")), "battle_entry."),
			centre + position + Vector3(0.0, BoardBlockout.TILE_THICKNESS_METRES * 0.5 + 0.4, 0.0),
			ENCOUNTER_SPACE_RADIUS_METRES,
			BoardPalette.DANGER
		)
		ring.set_meta("encounter_lens", EncounterLens.OPEN_DECISION)


## The board's environment. **P2 owns `world_environment.tres`**; this is the
## local stand-in until that lane lands, and it is the same shape as the
## terrace setpiece's lighting kit so the two can be replaced together.
func make_environment() -> WorldEnvironment:
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = BoardPalette.DEEP
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color("4c6f74")
	environment.ambient_light_energy = 0.7
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_exposure = 1.05
	environment.fog_enabled = true
	environment.fog_light_color = Color("32545a")
	environment.fog_light_energy = 0.4
	environment.fog_density = 0.0016
	var world_environment := WorldEnvironment.new()
	world_environment.name = "BoardEnvironment"
	world_environment.environment = environment
	return world_environment


func make_key_light() -> DirectionalLight3D:
	var sun := DirectionalLight3D.new()
	sun.name = "BoardKey"
	sun.rotation_degrees = Vector3(-52.0, -34.0, 0.0)
	sun.light_color = Color("ffd9a3")
	sun.light_energy = 1.45
	sun.shadow_enabled = true
	return sun


func make_fill_light() -> DirectionalLight3D:
	var fill := DirectionalLight3D.new()
	fill.name = "BoardFill"
	fill.rotation_degrees = Vector3(-26.0, 146.0, 0.0)
	fill.light_color = Color("6fc8bd")
	fill.light_energy = 0.5
	return fill
