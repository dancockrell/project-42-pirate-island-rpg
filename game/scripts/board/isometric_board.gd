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
## only thing that says the party has gone anywhere.
##
## ## P10: this is the expedition screen's board
##
## P4 said the 2D route board under `scripts/world/` would stay the one the
## expedition screen drew until a later card retired it. This is that card: that
## board is deleted, this one is what the screen hosts, and P7's control grammar
## moved onto it unchanged in meaning --
##
##   left-click   -> `tile_selected`  (the party's own tile selects it, any
##                   other tile is a look, never a move)
##   right-click  -> `move_ordered`   (the screen turns it into the one
##                   `request_travel` call, or refuses it in the status line)
##
## The board still never calls the bridge and never judges legality a second
## time: `portal_from_active_cell_to` is a lookup inside the snapshot's own
## `legal_route_commands`, exactly as it was on the 2D board, and the three
## states a tile can be in -- the party stands here, a legal road reaches here,
## no road reaches here -- are that list read out loud in clay.

## Left-click on a tile. The screen decides whether that is "select the party"
## or "look at that place"; the board only says which tile was clicked.
signal tile_selected(cell_id: String)

## Right-click on a tile: the move order, in board terms. The screen resolves it
## to a portal and to the single native travel call.
signal move_ordered(cell_id: String)

const DISTANCE_WORLD := "world"
const DISTANCE_ROUTE := "route"
const DISTANCE_ROOM := "room"
const DISTANCES := [DISTANCE_WORLD, DISTANCE_ROUTE, DISTANCE_ROOM]

## The region whose cells this board draws. One region is authored.
const REGION_ID := "world.region.black_beach"

## P2's one lighting kit: the island's environment, its key sun and its sea
## fill. The board instances it rather than declaring a second one.
const LIGHTING_KIT_PATH := "res://render/world_environment.tscn"

## **needs decision.** How much smaller a force miniature stands than the party's.
## A framing choice: the party is the one the player follows.
const FORCE_MINIATURE_SCALE := 0.72
## **needs decision.** How far above a tile a miniature's feet sit, in metres.
const MINIATURE_LIFT_METRES := 0.4
## **needs decision.** How far along its road a column that has not yet marched
## stands from the cell's centre, in metres. A dispatched force marches none of
## its road until an hour runs (S18), and the party's miniature stands at the
## centre, so a waiting column stands at the road mouth rather than on the party.
const FORCE_STAND_OFFSET_METRES := 3.0
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
## **needs decision.** How far past a room's footprint the selection ring stands,
## as a multiple of the footprint's smaller side, and how thick the ring is drawn
## in metres. Framing choices: the ring must read as a ring around the room at
## the world distance without swallowing the road that leaves it.
const RING_FOOTPRINT_MARGIN := 0.62
const RING_THICKNESS_METRES := 1.6
## **needs decision.** How far above a room's floor a ring is drawn, in metres.
const RING_LIFT_METRES := 0.9

## The selection ring's gentle breath, carried over from the 2D board unchanged:
## the amplitude is a fraction of the ring's radius and the period is in seconds.
## Both are switched off under reduced motion, where the ring stands at rest.
const RING_PULSE_AMPLITUDE := 0.055
const RING_PULSE_SECONDS := 2.6

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
## P13: what has been *built* on the island. One layer, drawn at every distance:
## the same nodes stand on the tile at the world and route distances and inside
## the room's footprint at the room distance, because a second set of nodes for
## the close view would be two boards and the one this file draws is one board.
var development_root: Node3D
var room_root: Node3D
var party_miniature: Node3D
var force_miniatures: Dictionary = {}
## P13: node name -> the `Node3D` one building or machine instance is standing
## as, and the order the snapshot gave them in. The order is kept because a
## cell's row is laid out in it and `Dictionary` iteration order is insertion
## order, which the rebuild of a single node would otherwise disturb.
var development_nodes: Dictionary = {}
var development_order: Array[String] = []
## The port used purely as the catalog's board reader when no campaign is live.
var reader: NativeExpeditionPort

# -- P7's grammar, on this board ---------------------------------------------

## The portals the snapshot currently calls legal, as a set. Read from
## `legal_route_commands` and from nothing else.
var legal_portal_ids: Dictionary = {}

## Whether the party is currently selected. It starts selected because the party
## is the only unit on this board and an RTS that opens with nothing selected
## just costs the player a click; a left-click on the party's tile or on its
## card re-states it, and looking at another tile never takes it away.
var party_selected := true

## The tile the player last looked at, or "" for none. A look is not a
## selection: it rings one tile and writes one status line, nothing else.
var inspected_cell_id := ""

var hovered_cell_id := ""

var party_ring: MeshInstance3D
var inspect_ring: MeshInstance3D

## Seconds of engine time accumulated while the ring is breathing. Presentation
## only: nothing on this board is read by the simulation, and the accumulator is
## frozen (and the ring drawn at rest) whenever reduced motion is on.
var ring_phase := 0.0


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
	add_child(make_lighting_kit())
	terrain_root = make_layer("Terrain")
	route_root = make_layer("Routes")
	room_root = make_layer("Room")
	miniature_root = make_layer("Miniatures")
	development_root = make_layer("Development")
	load_cells()
	build_sea()
	for cell_id in sorted_cell_ids():
		BoardBlockout.tile(terrain_root, cells[cell_id], BoardPalette.clay())
	build_tethers()
	build_shelf()
	space_parallel_tethers()
	build_causeways()
	party_miniature = BoardBlockout.miniature(miniature_root, "PartyMiniature", BoardPalette.cream())
	party_ring = BoardBlockout.ring(miniature_root, "PartySelectionRing", RING_THICKNESS_METRES, RING_THICKNESS_METRES, BoardPalette.teal())
	inspect_ring = BoardBlockout.ring(miniature_root, "InspectedCellRing", RING_THICKNESS_METRES, RING_THICKNESS_METRES * 0.6, BoardPalette.cream())
	inspect_ring.visible = false
	set_process(true)


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
	mesh.material = BoardBlockout.clay(BoardPalette.sea(), 0.08)
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
			BoardBlockout.clay(BoardPalette.land())
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
			BoardBlockout.clay(BoardPalette.land())
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
		BoardBlockout.causeway(terrain_root, BoardBlockout.node_name("Causeway_", str(portal_id), "world.portal."), tether["from_point"], tether["to_point"], BoardPalette.rock())


## Every corner of every room's footprint, at the room's own elevation. What the
## world distance has to hold in frame.
func island_corners() -> PackedVector3Array:
	var corners := PackedVector3Array()
	for cell_id in sorted_cell_ids():
		var cell: Dictionary = cells[cell_id]
		var centre := BoardBlockout.centre_of(cell)
		var footprint: Dictionary = cell.get("footprint", {})
		var half_width := float(footprint.get("width_metres", 0.0)) * 0.5
		var half_depth := float(footprint.get("depth_metres", 0.0)) * 0.5
		for corner in [Vector2(-1.0, -1.0), Vector2(1.0, -1.0), Vector2(1.0, 1.0), Vector2(-1.0, 1.0)]:
			corners.append(centre + Vector3(corner.x * half_width, 0.0, corner.y * half_depth))
	return corners


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
	read_legal_routes()
	apply_cells()
	apply_routes()
	apply_party()
	apply_forces()
	apply_development()
	apply_rings()
	apply_distance()


## The one door the screen projects a snapshot through, kept to the signature
## the 2D board carried so the expedition screen's call site did not have to
## learn a second shape when the board underneath it changed. The catalog is
## content and is taken once; everything else is the snapshot.
func configure(next_catalog: ContentCatalog, next_snapshot: Dictionary) -> void:
	if next_catalog != null and catalog != next_catalog:
		catalog = next_catalog
	project_snapshot(next_snapshot)


## The portals the snapshot calls legal right now. This is a read of
## `legal_route_commands` and nothing more: the board holds no opinion about
## which road is open, and a road the simulation stopped naming stops being
## reachable here on the very next projection.
func read_legal_routes() -> void:
	legal_portal_ids.clear()
	for command in snapshot.get("legal_route_commands", []):
		var command_text := str(command)
		if command_text.begins_with("travel:"):
			legal_portal_ids[command_text.trim_prefix("travel:")] = true
	if inspected_cell_id == party_cell_id():
		inspected_cell_id = ""


## S2's control and P7's reachability, drawn on the one surface a cell has.
##
## `controller_of` is the only answer to who holds a cell; the board never reads
## a raw ownership map and never derives a controller. `cell_state` is the only
## answer to whether the party can go there, and it is a lookup inside the
## snapshot's legal list. `BoardPalette.cell_tint` puts the two together, so a
## cell that changes hands changes colour whether it is reachable or not, and a
## road opening lights the place it reaches without repainting who holds it.
func apply_cells() -> void:
	for cell_id in sorted_cell_ids():
		var tile := terrain_root.get_node_or_null(BoardBlockout.node_name("Tile_", cell_id, "world.cell."))
		if tile == null:
			continue
		var live_port := current_port()
		var controller := "" if live_port == null else live_port.controller_of(cell_id)
		(tile as MeshInstance3D).mesh.material = BoardBlockout.clay(BoardPalette.cell_tint(controller, cell_state(cell_id)))


## Where one cell stands to the party: the tile it is on, a tile a legal road
## reaches, or a tile no road reaches. P7's whole legality vocabulary.
## The port the live campaign is holding right now. `CampaignSession` is the one
## owner of that, and it replaces the port it holds whenever a campaign is
## discarded and begun again -- a reset, a new game, a loaded slot. So this asks
## the session each time rather than keeping the port it was handed when the
## scene graph was built, which after any of those would answer for a campaign
## that no longer exists.
func current_port() -> NativeExpeditionPort:
	if campaign_session != null and campaign_session.expedition != null:
		port = campaign_session.expedition
	return port


func cell_state(cell_id: String) -> String:
	if cell_id == party_cell_id():
		return BoardPalette.CELL_ACTIVE
	if is_reachable(cell_id):
		return BoardPalette.CELL_REACHABLE
	return BoardPalette.CELL_DISTANT


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
		var tint: Color = BoardPalette.risk_tint(int(risk_by_portal.get(portal_id, 0))) if legal else BoardPalette.faint_structure()
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
		var point := here + Vector3.RIGHT * FORCE_STAND_OFFSET_METRES
		if cells.has(next_cell_id):
			var there := stand_point(cells[next_cell_id])
			var progress := clampf(float(force.get("progress", 0.0)), 0.0, 1.0)
			point = here.lerp(there, progress)
			if progress <= 0.0:
				point = here + (there - here).normalized() * FORCE_STAND_OFFSET_METRES
		(force_miniatures[force_id] as Node3D).position = point
	for force_id in force_miniatures.keys():
		(force_miniatures[force_id] as Node3D).visible = seen.has(force_id)


## P13: what S17 raised and S16 built, standing on the cells that hold it.
##
## The two read-only snapshot keys are read and nothing else is: an instance
## names the record it was raised from, the cell it stands on, whose it is and
## where it is in its life, and `BoardDevelopment` turns those into a node built
## by P12's kit. Nothing here decides anything -- a building appears because the
## snapshot carries it and stops standing the moment the snapshot does not, which
## is the same rule the party miniature and the force miniatures already follow.
##
## A node is rebuilt only when what it looks like has changed (its record, its
## state, its tier or its holder); otherwise it is moved. That is what makes a
## building finishing visible: `under_construction` is the first tier in the
## placeholder material and `operational` is the full kit, and the signature
## catches the change between two snapshots.
func apply_development() -> void:
	var wanted: Dictionary = {}
	var order: Array[String] = []
	for kind in BoardDevelopment.ORDER_KINDS:
		var key := "building_instances" if kind == BoardDevelopment.BUILDING else "machine_instances"
		for entry in snapshot.get(key, []):
			var instance: Dictionary = entry
			if not cells.has(str(instance.get("cell_id", ""))) or not BoardDevelopment.is_standing(instance):
				continue
			var node_name := BoardDevelopment.node_name_for(str(kind), str(instance.get("id", "")))
			wanted[node_name] = {"kind": str(kind), "instance": instance}
			order.append(node_name)
	# Gone from the snapshot, or standing as something else now: the node goes.
	for node_name in development_nodes.keys():
		var standing: Node3D = development_nodes[node_name]
		var still := wanted.has(node_name)
		if still:
			var entry: Dictionary = wanted[node_name]
			still = str(standing.get_meta("signature", "")) == BoardDevelopment.signature_of(str(entry["kind"]), entry["instance"])
		if still:
			continue
		development_root.remove_child(standing)
		standing.queue_free()
		development_nodes.erase(node_name)
	for node_name in order:
		if development_nodes.has(node_name):
			continue
		var entry: Dictionary = wanted[node_name]
		var kind := str(entry["kind"])
		var instance: Dictionary = entry["instance"]
		var def_id := str(instance.get("def_id", ""))
		if catalog == null or not catalog.has(def_id):
			push_error("The save stands %s on %s and content declares no such record." % [def_id, instance.get("cell_id", "")])
			continue
		var record: Dictionary = catalog.get_record(def_id)
		# The room a thing stands in is the room its share of the ground is
		# measured against, so the kit is handed the holding cell's own
		# footprint rather than a reference one: a shed in a court is a court's
		# worth of shed.
		var room_footprint: Dictionary = (cells[str(instance.get("cell_id", ""))] as Dictionary).get("footprint", {})
		var node := BoardDevelopment.build_instance(kind, instance, record, room_footprint)
		if node == null:
			continue
		var apron: Vector2 = BoardDevelopment.apron_metres(kind, record, room_footprint)
		node.set_meta("row_width", apron.x)
		node.set_meta("row_depth", apron.y)
		development_root.add_child(node)
		development_nodes[node_name] = node
	development_order = order
	place_development()


## Where each standing thing is put, which is the one thing that changes with
## the distance. At the world and route distances a cell's things stand in one
## row across the back of its tile; at the room distance the same nodes stand on
## the authored spawn sockets of the cell being looked at, and the things
## standing on every other cell are not drawn -- the room distance is one room.
func place_development() -> void:
	var focus := party_cell_id()
	if not cells.has(focus):
		focus = str(sorted_cell_ids()[0]) if not cells.is_empty() else ""
	var by_cell: Dictionary = {}
	for node_name in development_order:
		if not development_nodes.has(node_name):
			continue
		var node: Node3D = development_nodes[node_name]
		var cell_id := str(node.get_meta("cell_id", ""))
		if not by_cell.has(cell_id):
			by_cell[cell_id] = []
		(by_cell[cell_id] as Array).append(node)
	for cell_id in by_cell:
		var group: Array = by_cell[cell_id]
		var cell: Dictionary = cells[cell_id]
		var grounds: Array = []
		var kinds: Array = []
		for node in group:
			grounds.append(Vector2(float((node as Node3D).get_meta("row_width", 0.0)), float((node as Node3D).get_meta("row_depth", 0.0))))
			kinds.append(str((node as Node3D).get_meta("instance_kind", "")))
		var places: Array[Vector3] = BoardDevelopment.row_places(cell, grounds, kinds)
		var seats: Dictionary = {}
		# What is already standing in this room, so the next thing does not take a
		# socket underneath it. Buildings are placed before machines, so a yard
		# takes its socket and the dog it turned out takes the first free one.
		var taken: Array = []
		for index in group.size():
			var node: Node3D = group[index]
			var kind := str(node.get_meta("instance_kind", ""))
			if distance != DISTANCE_ROOM:
				node.visible = true
				node.position = places[index]
				continue
			node.visible = cell_id == focus
			if not node.visible:
				continue
			var seat := int(seats.get(kind, 0))
			seats[kind] = seat + 1
			var ground := Vector2(float(node.get_meta("row_width", 0.0)), float(node.get_meta("row_depth", 0.0)))
			var socket_point: Vector3 = BoardDevelopment.room_place(cell, kind, seat, ground, taken)
			node.position = places[index] if socket_point == Vector3.INF else socket_point
			taken.append(BoardDevelopment.ground_rect(node.position, ground))


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
	# P13: and what is built stands at every distance, so the same node is
	# re-placed rather than a second one being drawn close up.
	place_development()
	match distance:
		DISTANCE_ROOM:
			build_room(focus_cell_id)
			camera.frame(distance, BoardBlockout.centre_of(cells[focus_cell_id]) if cells.has(focus_cell_id) else Vector3.ZERO)
		DISTANCE_ROUTE:
			camera.frame(distance, BoardBlockout.centre_of(cells[focus_cell_id]) if cells.has(focus_cell_id) else Vector3.ZERO)
		_:
			var bounds := island_bounds()
			camera.frame(distance, Vector3(bounds.get_center().x, 12.0, bounds.get_center().y))
			# The world distance must hold the whole island, and how wide the
			# island is is content: nine authored rooms at authored positions.
			# So the constant frame height is a floor and the actual frame is
			# measured off the rooms, which is why adding a tenth room does not
			# crop the board.
			camera.fit(island_corners())


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
	var floor_slab := SetpieceMeshFactory.box(room_root, "RoomFloor", Vector3(width, BoardBlockout.TILE_THICKNESS_METRES, depth), centre, BoardBlockout.clay(BoardPalette.clay()))
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
			BoardBlockout.clay(BoardPalette.faint_structure())
		)
		BoardBlockout.mark(kerb, "the room's authored boundary geometry")
	for spawn in cell.get("spawn_points", []):
		var role := str((spawn as Dictionary).get("role", ""))
		var tint: Color = BoardPalette.role_tint(role)
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
			BoardBlockout.clay(BoardPalette.bronze(), 0.28)
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
			BoardPalette.danger()
		)
		ring.set_meta("encounter_lens", EncounterLens.OPEN_DECISION)


## The board's light. **P2 owns it**, and this instances P2's kit rather than
## keeping the local stand-in P4 wrote while that lane was in flight: the same
## `world_environment.tscn` the terrace and the port town instance, so the island
## is lit by the one environment the project declares and a change to it reaches
## every screen at once. Three hand-written nodes -- an Environment, a key light
## and a fill -- came out of this file to put it there.
## The kit's root is a `WorldEnvironment`, which is a `Node` and not a `Node3D`
## -- the lights are its children -- so this returns a `Node`.
func make_lighting_kit() -> Node:
	var packed := load(LIGHTING_KIT_PATH) as PackedScene
	if packed == null:
		push_error("The board cannot load the project's lighting kit at %s." % LIGHTING_KIT_PATH)
		return Node.new()
	var kit := packed.instantiate()
	kit.name = "BoardLight"
	return kit


# ---------------------------------------------------------------------------
# P7's grammar, on the 3D board
#
# The three hooks the 2D board offered the screen, on this one, with the same
# meanings: `portal_from_active_cell_to` is the lookup, `set_party_selected` and
# `set_inspected_cell` are what a click leaves behind. The screen is unchanged
# in what it asks for; only what answers has changed.
#
# The board still emits rather than acts: `tile_selected` and `move_ordered`
# carry a cell id out, and every order that follows goes through the screen's
# one `request_travel`. No line below calls the bridge.
# ---------------------------------------------------------------------------

## The authored portal that leaves the active cell for `cell_id` and that the
## native snapshot currently calls legal, or "" when there is no such road.
##
## Two roads can join the same pair of rooms -- the river landing's safe road
## and its jungle edge both reach the terrace -- so the order this walks in
## decides which one a right-click orders. It walks the active cell's portals in
## the order the content authors them, which is the order the departure list
## draws them in, so the tile and the first digit give the same road. The 2D
## board resolved it the same way and this keeps that meaning.
func portal_from_active_cell_to(cell_id: String) -> String:
	var active := party_cell_id()
	# Before the first snapshot there is no active cell, and asking the catalog
	# for the record named "" is a content error rather than a question.
	if catalog == null or active.is_empty() or cell_id.is_empty():
		return ""
	for portal in catalog.get_record(active).get("portals", []):
		if not portal is Dictionary:
			continue
		if str((portal as Dictionary).get("targetCellId", "")) != cell_id:
			continue
		var portal_id := str((portal as Dictionary).get("id", ""))
		if legal_portal_ids.has(portal_id):
			return portal_id
	return ""


func is_reachable(cell_id: String) -> bool:
	return not portal_from_active_cell_to(cell_id).is_empty()


## A cell's authored name, for a status line. The board never invents a place
## name; a cell the catalog does not carry reads as its own id.
func display_name_of(cell_id: String) -> String:
	if catalog == null:
		return cell_id
	var record := catalog.get_record(cell_id)
	var display := str(record.get("displayName", ""))
	return display if not display.is_empty() else cell_id


## The cell under a point in the board viewport's own pixels, or "" for open
## water. The camera is orthographic and fixed, so this is an exact ray against
## each room's top face rather than a hit test against drawn pixels: the answer
## does not depend on what happens to be drawn over the tile.
##
## At the room distance the island is not on screen at all, so nothing is
## pickable there -- a place the player cannot see is not a place they can order
## a march to, and the words and the digits still can.
func cell_at_viewport_point(point: Vector2) -> String:
	if camera == null or not terrain_root.visible:
		return ""
	var origin := camera.project_ray_origin(point)
	var direction := camera.project_ray_normal(point)
	var best_cell := ""
	var best_distance := INF
	for cell_id in sorted_cell_ids():
		var cell: Dictionary = cells[cell_id]
		var top := float(cell.get("elevation_metres", 0.0))
		if absf(direction.y) < 0.00001:
			continue
		var travel := (top - origin.y) / direction.y
		if travel <= 0.0:
			continue
		var hit := origin + direction * travel
		var centre := BoardBlockout.centre_of(cell)
		var footprint: Dictionary = cell.get("footprint", {})
		if absf(hit.x - centre.x) > float(footprint.get("width_metres", 0.0)) * 0.5:
			continue
		if absf(hit.z - centre.z) > float(footprint.get("depth_metres", 0.0)) * 0.5:
			continue
		if travel < best_distance:
			best_distance = travel
			best_cell = cell_id
	return best_cell


## Left-click, in board terms. The board reports which tile; the screen decides
## whether that was a selection or a look.
func click_at(point: Vector2) -> String:
	var cell_id := cell_at_viewport_point(point)
	if not cell_id.is_empty():
		tile_selected.emit(cell_id)
	return cell_id


## Right-click, in board terms: the move order. The screen resolves it to a
## portal and to the one native travel call, or refuses it.
func order_at(point: Vector2) -> String:
	var cell_id := cell_at_viewport_point(point)
	if not cell_id.is_empty():
		move_ordered.emit(cell_id)
	return cell_id


## The tile the pointer is over, kept so the board can say so. Returns true when
## the answer changed, which is what a caller redraws on.
func hover_at(point: Vector2) -> bool:
	var hovered := cell_at_viewport_point(point)
	if hovered == hovered_cell_id:
		return false
	hovered_cell_id = hovered
	return true


## Called by the screen when the party is selected, from a click on its tile or
## on its card in the interface. The ring restarts at rest so the breath begins
## where the player's eye lands.
func set_party_selected(selected: bool) -> void:
	party_selected = selected
	ring_phase = 0.0
	apply_rings()


func set_inspected_cell(cell_id: String) -> void:
	inspected_cell_id = "" if cell_id == party_cell_id() else cell_id
	apply_rings()


## How wide a ring stands on one room: half the room's shorter side, plus the
## named margin. A ring is drawn around the *place* the party is in, so it is
## the size of that place -- one radius for every room on the island would have
## to be the biggest room's, and the biggest room here is wide enough that the
## ring reached across three of its neighbours.
func ring_radius_for(cell: Dictionary) -> float:
	var footprint: Dictionary = cell.get("footprint", {})
	var shorter := minf(float(footprint.get("width_metres", 2.0)), float(footprint.get("depth_metres", 2.0)))
	return maxf(shorter, 1.0) * 0.5 * (1.0 + RING_FOOTPRINT_MARGIN)


## Put a ring on one cell: its size and its place, in one call, because a ring
## that moved without resizing would be the wrong ring on the right room.
func place_ring(torus: MeshInstance3D, cell: Dictionary, thickness: float) -> void:
	var radius := ring_radius_for(cell)
	var mesh := torus.mesh as TorusMesh
	mesh.outer_radius = radius
	mesh.inner_radius = maxf(radius - thickness, 0.05)
	torus.position = ring_point(cell)


## The two rings put where the snapshot and the player's last click say they go.
## Neither ring moves the party and neither is read by anything: they are the
## board saying what is selected and what was looked at.
func apply_rings() -> void:
	if party_ring == null or inspect_ring == null:
		return
	var party_cell := party_cell_id()
	party_ring.visible = party_selected and cells.has(party_cell)
	if party_ring.visible:
		place_ring(party_ring, cells[party_cell], RING_THICKNESS_METRES)
	inspect_ring.visible = not inspected_cell_id.is_empty() and cells.has(inspected_cell_id) and inspected_cell_id != party_cell
	if inspect_ring.visible:
		place_ring(inspect_ring, cells[inspected_cell_id], RING_THICKNESS_METRES * 0.6)
	apply_ring_breath()


func ring_point(cell: Dictionary) -> Vector3:
	return BoardBlockout.centre_of(cell) + Vector3(0.0, RING_LIFT_METRES, 0.0)


## The breath, and the reduced-motion substitute for it. B9's setting reaches
## the board through the Theme the screen built, which is the same door every
## other surface asks through; under reduced motion the ring is drawn at rest
## rather than removed, because the ring is what says the party is selected.
func apply_ring_breath() -> void:
	if party_ring == null:
		return
	var breath := 0.0
	if not BoardPalette.reduced_motion():
		breath = sin(ring_phase / RING_PULSE_SECONDS * TAU) * RING_PULSE_AMPLITUDE
	party_ring.scale = Vector3.ONE * (1.0 + breath)


func _process(delta: float) -> void:
	if not party_selected or party_ring == null or not party_ring.visible or BoardPalette.reduced_motion():
		return
	ring_phase = fmod(ring_phase + delta, RING_PULSE_SECONDS)
	apply_ring_breath()
