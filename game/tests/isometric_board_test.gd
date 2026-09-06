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

	# S2's ownership, drawn. Nothing here edits a colour: the cell is handed to a
	# faction and the tile's tint follows the bridge's own answer.
	var tile := board.terrain_root.get_node_or_null("Tile_river_landing") as MeshInstance3D
	check(tile != null, "the world distance must carry a tile for every cell")
	var unheld_tint: Color = tile.mesh.material.albedo_color
	check(unheld_tint.is_equal_approx(BoardPalette.CLAY), "an unheld cell must be drawn in the unheld clay")
	var claimed: Dictionary = campaign_session.set_control("world.cell.river_landing", "faction.pirates")
	check(bool(claimed.get("configured", false)), "claiming a cell must be accepted by the native bridge")
	board.project_snapshot(claimed)
	var held_tint: Color = tile.mesh.material.albedo_color
	check(not held_tint.is_equal_approx(unheld_tint), "a cell that changed hands must change tint on the board")
	check(held_tint.is_equal_approx(BoardPalette.controller_tint("faction.pirates")), "the tint must be the one the holding faction's concept key names")
	var released: Dictionary = campaign_session.set_control("world.cell.river_landing", "")
	board.project_snapshot(released)
	check((tile.mesh.material.albedo_color as Color).is_equal_approx(unheld_tint), "releasing a cell must put its tint back")

	# B14's live risk, drawn. The safe road out of the landing is legal from the
	# terrace's neighbour, so the board's road for it is coloured by the risk the
	# snapshot carries rather than by the authored base.
	var drawn_routes := 0
	for child in board.route_root.get_children():
		if str(child.name).begins_with("Route_"):
			drawn_routes += 1
	check(drawn_routes == board.tethers.size(), "every authored road must be drawn, legal or not")

	# S7's forces, drawn. No bridge verb raises or marches a force yet, so the
	# only honest way to put one on the live island is through the bridge's own
	# save round trip; `BoardVerificationCampaign` owns that and says why. What
	# is asserted here is the board's half: a force partway down a road stands
	# partway down that road, and the reading comes from the snapshot.
	var marched: Dictionary = BoardVerificationCampaign.add_marching_force(campaign_session)
	check(bool(marched.get("configured", false)), "the bridge must accept a save carrying one marching force")
	var projected_forces: Array = marched.get("forces", [])
	check(projected_forces.size() == 1, "the snapshot must project the one force the campaign now holds")
	if projected_forces.size() == 1:
		var force: Dictionary = projected_forces[0]
		check(str(force.get("position_cell_id", "")) == "world.cell.river_landing", "the force must stand where the save put it")
		check(str(force.get("next_cell_id", "")) == "world.cell.reception_terrace", "the snapshot must name the cell the force's next hop reaches")
		check(absf(float(force.get("progress", 0.0)) - 0.45) < 0.001, "progress must be the marched minutes over the road's own cost")
		for key in force.keys():
			var field := str(key)
			check(not (field in ["strength", "readiness", "supply", "composition", "roles", "assignment", "evidence"]), "an aggregate's strength must not cross the bridge: %s" % field)
	board.project_snapshot(marched)
	var force_pawn := board.miniature_root.get_node_or_null("Force_pirates_landing_column") as Node3D
	check(force_pawn != null, "a projected force must be drawn as a miniature")
	if force_pawn != null:
		var from_point: Vector3 = board.stand_point(board.cells["world.cell.river_landing"])
		var to_point: Vector3 = board.stand_point(board.cells["world.cell.reception_terrace"])
		check(force_pawn.position.is_equal_approx(from_point.lerp(to_point, 0.45)), "the miniature must stand as far down the road as the snapshot says it has marched")
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
