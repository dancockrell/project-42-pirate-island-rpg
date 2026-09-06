extends SceneTree

const BoardVerificationCampaign = preload("res://tests/board_verification_campaign.gd")

## B12's done-when, through the live Rust bridge: the board is driven across
## three cells and the party miniature is asserted to be standing on the cell
## the snapshot names, each time. Then S2's control is handed to a faction and
## the cell's tint is asserted to have changed with it.
##
## The board is not asked where the party is. The snapshot is, and the board is
## held to it -- which is the point of the card: the miniature snaps to
## `active_location_id` and moves on nothing else.

const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")

var failures := 0
var campaign_session: Node


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Isometric board test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	campaign_session = root.get_node_or_null("CampaignSession")
	if campaign_session == null:
		campaign_session = CampaignSessionScript.new()
		campaign_session.name = "CampaignSession"
		root.add_child(campaign_session)
	campaign_session.reset_for_test()
	BoardPalette.use_authored_theme()
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the generated content bundle must load")
	var snapshot: Dictionary = campaign_session.begin_if_needed(catalog)
	check(bool(snapshot.get("configured", false)), "the campaign must configure through the native bridge")

	var scene := load("res://scenes/board/isometric_board.tscn") as PackedScene
	check(scene != null, "the isometric board scene must load")
	if scene == null:
		finish()
		return
	var board := scene.instantiate() as IsometricBoard
	root.add_child(board)
	await process_frame

	# B11 reached the board: nine authored rooms, each with a footprint out of
	# the one constant table, at least seven sockets and one tether per portal.
	check(board.cells.size() == 9, "the board must build every authored world cell, not only the five the 2D board drew")
	for cell_id in board.sorted_cell_ids():
		var cell: Dictionary = board.cells[cell_id]
		var footprint: Dictionary = cell.get("footprint", {})
		check(float(footprint.get("width_metres", 0.0)) > 0.0 and float(footprint.get("depth_metres", 0.0)) > 0.0, "%s must resolve its footprint to metres" % cell_id)
		check(str(footprint.get("needs_decision", "")) == "O3", "%s footprint must still be marked as the open decision it is" % cell_id)
		check((cell.get("spawn_points", []) as Array).size() >= 7, "%s must carry at least seven spawn sockets" % cell_id)
		var portal_count: int = (catalog.get_record(cell_id).get("portals", []) as Array).size()
		check((cell.get("tethers", []) as Array).size() == portal_count, "%s must carry one tether per portal" % cell_id)

	# Travel across three cells, salvaging first exactly as the prototype suite
	# does: the party lands with no rations and every road costs.
	campaign_session.use_anchor(BoardVerificationCampaign.OPENING_SALVAGE)
	board.project_snapshot(campaign_session.latest_snapshot)
	check(board.party_cell_id() == "world.cell.black_beach", "a fresh campaign must begin at Black Beach")
	check_miniature_on(board, "world.cell.black_beach", "the miniature must stand on the beach it began on")

	var arrivals := ["world.cell.damaged_estate", "world.cell.river_landing", "world.cell.reception_terrace"]
	for index in BoardVerificationCampaign.OPENING_TRAVEL.size():
		var step := [BoardVerificationCampaign.OPENING_TRAVEL[index], arrivals[index]]
		var travelled: Dictionary = campaign_session.travel(str(step[0]))
		check(bool(travelled.get("configured", false)), "travel along %s must be accepted by the native bridge" % step[0])
		board.project_snapshot(travelled)
		check(board.party_cell_id() == str(step[1]), "the snapshot must place the party in %s" % step[1])
		check_miniature_on(board, str(step[1]), "the miniature must snap to %s" % step[1])

	# A click is not a travel. Asking the board to look somewhere else moves the
	# camera and leaves the party exactly where the simulation put it.
	var standing := board.party_miniature.position
	board.set_distance(IsometricBoard.DISTANCE_ROOM)
	check(board.party_miniature.position == standing, "changing distance must not move the party miniature")
	board.set_distance(IsometricBoard.DISTANCE_WORLD)

	# The room distance draws what brief section 4 requires of a room: sockets,
	# exits and an encounter space.
	board.set_distance(IsometricBoard.DISTANCE_ROOM)
	var sockets := 0
	var exits := 0
	var encounter_spaces := 0
	for child in board.room_root.get_children():
		var child_name := str(child.name)
		if child_name.begins_with("Socket_"):
			sockets += 1
		elif child_name.begins_with("Exit_"):
			exits += 1
		elif child_name.begins_with("EncounterSpace_"):
			encounter_spaces += 1
	check(sockets >= 7, "the room distance must draw every spawn socket the cell declares")
	check(exits == 2, "the room distance must draw the terrace's two authored exits")
	check(encounter_spaces == 1, "the room distance must mark the cell's authored encounter space")
	check(EncounterLens.lens_scene() != null, "the stand-in encounter lens must resolve the battle screen the project already ships")
	board.set_distance(IsometricBoard.DISTANCE_WORLD)

	# P7's grammar needs an open road to point at, and arriving at the terrace
	# arms its authored encounter, which makes every road illegal -- a board
	# asked about reachability there would truthfully answer "nowhere". So the
	# campaign is replayed one leg short, through the same opening
	# `BoardVerificationCampaign` owns, and the grammar is asserted from the
	# river landing where three roads are open.
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	board.project_snapshot(BoardVerificationCampaign.play_opening(campaign_session, 2))
	check(board.party_cell_id() == "world.cell.river_landing", "the replayed opening must stop the party at the river landing")

	# P7's legality vocabulary, on this board. The three states are a reading of
	# the snapshot's own legal list and of nothing else, so the cell the party is
	# on is active, the cell its one legal road reaches is reachable, and a cell
	# on the far side of the island is neither.
	var here := board.party_cell_id()
	check(board.cell_state(here) == BoardPalette.CELL_ACTIVE, "the cell the party stands on must read as the active cell")
	var reachable_cells: Array[String] = []
	for cell_id in board.sorted_cell_ids():
		if board.is_reachable(str(cell_id)):
			reachable_cells.append(str(cell_id))
	check(not reachable_cells.is_empty(), "the terrace must have at least one legal road out of it on the board")
	for cell_id in reachable_cells:
		check(not board.portal_from_active_cell_to(cell_id).is_empty(), "a reachable cell must name the portal that reaches it")
		check(board.cell_state(cell_id) == BoardPalette.CELL_REACHABLE, "a cell a legal road reaches must read as reachable")
	check(board.cell_state("world.cell.black_beach") == BoardPalette.CELL_DISTANT, "a cell no legal road reaches from here must read as distant")

	# The mouse lands on tiles. The camera is fixed and orthographic, so the
	# centre of a room projected onto the viewport must pick that room back --
	# this is the whole of "a click on the board is a click on a place".
	for cell_id in board.sorted_cell_ids():
		var centre: Vector3 = BoardBlockout.centre_of(board.cells[cell_id])
		var point := board.camera.unproject_position(centre)
		check(board.cell_at_viewport_point(point) == str(cell_id), "a click at the middle of %s must pick %s" % [cell_id, cell_id])
	# Open water picks nothing, and the room distance -- which draws no island --
	# picks nothing either.
	check(board.cell_at_viewport_point(Vector2(-4000.0, -4000.0)).is_empty(), "a click off the island must pick no cell")
	board.set_distance(IsometricBoard.DISTANCE_ROOM)
	var room_centre: Vector3 = BoardBlockout.centre_of(board.cells[here])
	check(board.cell_at_viewport_point(board.camera.unproject_position(room_centre)).is_empty(), "no tile is pickable at the room distance, where the island is not drawn")
	board.set_distance(IsometricBoard.DISTANCE_WORLD)

	# The two rings. The selection ring stands on the party's cell and a look
	# rings the place that was looked at, and neither is on the other's cell.
	board.set_party_selected(true)
	check(board.party_ring.visible, "selecting the party must ring its cell")
	check(board.party_ring.position.is_equal_approx(board.ring_point(board.cells[here])), "the selection ring must stand on the cell the snapshot names")
	board.set_inspected_cell("world.cell.black_beach")
	check(board.inspect_ring.visible, "a look must ring the place that was looked at")
	check(board.inspect_ring.position.is_equal_approx(board.ring_point(board.cells["world.cell.black_beach"])), "the look ring must stand on the cell that was looked at")
	board.set_inspected_cell(here)
	check(not board.inspect_ring.visible, "looking at the party's own cell is a selection, not a look, and leaves no second ring")


	# S2's ownership, drawn. Nothing here edits a colour: the cell is handed to a
	# faction and the tile's tint follows the bridge's own answer.
	var tile := board.terrain_root.get_node_or_null("Tile_river_landing") as MeshInstance3D
	check(tile != null, "the world distance must carry a tile for every cell")
	# P10: a cell's tile carries two facts at once -- who holds it and whether a
	# legal road reaches it -- so the expected colour is asked of the palette with
	# both, and a change to either must move it.
	var landing_state := board.cell_state("world.cell.river_landing")
	var unheld_tint: Color = tile.mesh.material.get_shader_parameter("albedo")
	check(unheld_tint.is_equal_approx(BoardPalette.cell_tint("", landing_state)), "an unheld cell must be drawn in the unheld clay, at the reach the snapshot gives it")
	var claimed: Dictionary = campaign_session.set_control("world.cell.river_landing", "faction.pirates")
	check(bool(claimed.get("configured", false)), "claiming a cell must be accepted by the native bridge")
	board.project_snapshot(claimed)
	var held_tint: Color = tile.mesh.material.get_shader_parameter("albedo")
	check(not held_tint.is_equal_approx(unheld_tint), "a cell that changed hands must change tint on the board")
	check(held_tint.is_equal_approx(BoardPalette.cell_tint("faction.pirates", landing_state)), "the tint must be the one the holding faction's concept key names")
	var released: Dictionary = campaign_session.set_control("world.cell.river_landing", "")
	board.project_snapshot(released)
	check((tile.mesh.material.get_shader_parameter("albedo") as Color).is_equal_approx(unheld_tint), "releasing a cell must put its tint back")

	# B14's live risk, drawn. The safe road out of the landing is legal from the
	# terrace's neighbour, so the board's road for it is coloured by the risk the
	# snapshot carries rather than by the authored base.
	var drawn_routes := 0
	for child in board.route_root.get_children():
		if str(child.name).begins_with("Route_"):
			drawn_routes += 1
	check(drawn_routes == board.tethers.size(), "every authored road must be drawn, legal or not")

	# S7's forces, drawn. S18 gave the bridge `raise_force` and `dispatch_force`,
	# so the column is raised and sent through those verbs;
	# `BoardVerificationCampaign` owns that campaign and says why. A dispatch
	# plans the whole road and marches none of it -- the hours belong to the
	# strategic clock and no verb here turns one -- so the force stands at the
	# head of its road, and what is asserted is the board's half: it stands
	# where the snapshot says it stands, and the reading comes from the
	# snapshot.
	var marched: Dictionary = BoardVerificationCampaign.add_marching_force(campaign_session)
	check(bool(marched.get("configured", false)), "the bridge must raise and dispatch the column through its own verbs: %s" % str(marched.get("error", "")))
	var projected_forces: Array = marched.get("forces", [])
	check(projected_forces.size() == 1, "the snapshot must project the one force the campaign now holds")
	if projected_forces.size() == 1:
		var force: Dictionary = projected_forces[0]
		check(str(force.get("position_cell_id", "")) == "world.cell.river_landing", "the force must stand where the save put it")
		check(str(force.get("next_cell_id", "")) == "world.cell.reception_terrace", "the snapshot must name the cell the force's next hop reaches")
		check(absf(float(force.get("progress", 0.0))) < 0.001, "a dispatched force has marched none of its road until an hour runs")
		for key in force.keys():
			var field := str(key)
			check(not (field in ["strength", "readiness", "supply", "composition", "roles", "assignment", "evidence"]), "an aggregate's strength must not cross the bridge: %s" % field)
	board.project_snapshot(marched)
	var force_pawn := board.miniature_root.get_node_or_null("Force_pirates_landing_column") as Node3D
	check(force_pawn != null, "a projected force must be drawn as a miniature")
	if force_pawn != null:
		var from_point: Vector3 = board.stand_point(board.cells["world.cell.river_landing"])
		var to_point: Vector3 = board.stand_point(board.cells["world.cell.reception_terrace"])
		check(force_pawn.position.is_equal_approx(from_point.lerp(to_point, 0.0)), "the miniature must stand as far down the road as the snapshot says it has marched")
		check(force_pawn.position != board.party_miniature.position, "a force is not the party and does not stand on it")

	board.queue_free()
	await process_frame
	finish()


## The miniature stands on the named cell's tile: same ground position, lifted
## by the tile's own thickness.
func check_miniature_on(board: IsometricBoard, cell_id: String, message: String) -> void:
	var expected: Vector3 = board.stand_point(board.cells[cell_id])
	check(board.party_miniature.position.is_equal_approx(expected), message)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Isometric board tests passed.")
	quit(0)
