extends SceneTree

## P13's done-when: what S17 raises and S16 builds is standing on the board.
##
## The bridge is live. The campaign is begun through `CampaignSession` exactly
## as every other board proof begins one, the island is the authored island, and
## the two read-only keys this card put on the state dictionary --
## `building_instances` and `machine_instances` -- are read off a real snapshot
## and asserted node by node.
##
## ## Why the island is developed through a save round trip, and what replaces it
##
## The card asks for a campaign in which a building is raised through the live
## bridge and the player's own yard produces a machine, and says plainly what to
## do if no authored opening reaches one inside the suite's budget. None does,
## and the reason is not budget -- no number of hours fixes either half:
##
## * **Nothing chooses a goal.** S17's `act_on_goals` acts only on a faction's
##   `current_goals`, and S5 scores a goal out of stockpiles, relationships and
##   held ground. A campaign begun through the bridge has an empty stockpile and
##   no relationship for every faction, so every goal scores zero and
##   `current_goals` is empty for all six. Measured, not assumed: 60 in-world
##   days (1,440 strategic hours) driven through `resolve_midnight`, with four
##   cells handed out through `set_control` first, and `current_goals` is still
##   empty for all six at the end. The determinism harness reaches S17's Develop
##   branch because `a_campaign()` in `godot-rust/tests/strategic_determinism.rs`
##   *seeds* the stockpiles and the pairwise relationships across the signal
##   range; no bridge verb seeds either.
## * **Nothing finishes a building.** S17's own shipped paragraph records it:
##   `advance_construction` has no caller on a clock, so a building S17 did place
##   would stand `UnderConstruction` for ever -- and `produce_machine` refuses a
##   building that is not `Operational`. A machine therefore cannot be reached
##   through the tick at all, however the goals were seeded.
##
## So the island is developed the way P4 developed its force before S18 gave the
## bridge a verb: the campaign's own save is taken from the bridge, two building
## instances and one machine instance are spliced into it, and it is loaded back
## **through the bridge**, which parses it, validates it and projects it. What is
## asserted is still the live bridge's own projection of its own state, and
## `BoardVerificationCampaign.develop_island` is the one place it happens.
##
## **It goes the day a verb lands.** Either would do it: one that seeds or reads
## back a faction's stockpile and relationships so S5 can score a goal, or one
## that spends construction hours on the clock (S17's recorded gap). When one
## arrives, the splice is replaced by the verb and this note comes out with it --
## nothing else here changes, because every assertion below is about the board
## and reads the snapshot, not the splice.

const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")
const BoardVerificationCampaign = preload("res://tests/board_verification_campaign.gd")

## The development this suite asserts is `BoardVerificationCampaign`'s, not this
## file's. That file is the one owner of the campaign both of the board's proofs
## run -- the suite that asserts and the capture that photographs -- so a picture
## and an assertion can never be of two different islands, and the three
## instances, the cells they stand on and the save round trip that puts them
## there are all declared there and read here.
##
## Three instances, chosen so that every branch this board has is exercised on a
## real authored record: Captain Michael's yard on the terrace, standing and
## washed in his faction's colour; a pirate watch post on the landing,
## `under_construction` with hours still on it, which is the first tier alone in
## the placeholder material; and the dog the yard turned out, standing at the
## yard's own cell.

var failures := 0
var campaign_session: Node


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Board development test skipped: bridge is not registered in this running Godot process.")
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
	var opening: Dictionary = campaign_session.begin_if_needed(catalog)
	check(bool(opening.get("configured", false)), "the campaign must configure through the native bridge")

	# The two keys exist on every snapshot the bridge gives, and a fresh island
	# has nothing standing on it.
	check(opening.has("building_instances"), "the state dictionary must carry building_instances")
	check(opening.has("machine_instances"), "the state dictionary must carry machine_instances")
	check((opening.get("building_instances", []) as Array).is_empty(), "a fresh campaign has raised nothing")
	check((opening.get("machine_instances", []) as Array).is_empty(), "a fresh campaign has built nothing")

	var scene := load("res://scenes/board/isometric_board.tscn") as PackedScene
	check(scene != null, "the isometric board scene must load")
	if scene == null:
		finish()
		return
	var board := scene.instantiate() as IsometricBoard
	root.add_child(board)
	await process_frame
	check(board.development_root != null, "the board must carry a layer for what has been built")
	check(board.development_nodes.is_empty(), "an island nobody has built on stands nothing")

	# The island is developed, through the bridge's own save round trip.
	var developed: Dictionary = BoardVerificationCampaign.develop_island(campaign_session)
	check(bool(developed.get("configured", false)), "the bridge must accept the developed save: %s" % str(developed.get("error", "")))
	var buildings: Array = developed.get("building_instances", [])
	var machines: Array = developed.get("machine_instances", [])
	check(buildings.size() == 2, "the snapshot must project both building instances")
	check(machines.size() == 1, "the snapshot must project the machine the yard built")

	# The six fields a building crosses on, and the five a machine crosses on --
	# and nothing else. A hit point, a fuel count or a stored value reaching the
	# board would be the record and the save being read through the wrong door.
	for entry in buildings:
		var building: Dictionary = entry
		check(building.keys().size() == 7, "a building instance crosses on seven fields and no more: %s" % str(building.keys()))
		for field in ["id", "def_id", "cell_id", "faction_id", "tier", "state", "construction_hours_remaining"]:
			check(building.has(field), "a building instance must carry %s" % field)
		for withheld in ["hp", "stored_value", "machines_produced", "hit_points"]:
			check(not building.has(withheld), "a building's %s must not cross the bridge" % withheld)
	var dog: Dictionary = machines[0] if machines.size() == 1 else {}
	check(dog.keys().size() == 5, "a machine instance crosses on five fields and no more: %s" % str(dog.keys()))
	for field in ["id", "def_id", "faction_id", "cell_id", "state"]:
		check(dog.has(field), "a machine instance must carry %s" % field)
	for withheld in ["fuel_remaining", "water_remaining", "damage", "built_by_building_instance_id"]:
		check(not dog.has(withheld), "a machine's %s must not cross the bridge" % withheld)
	check(str(dog.get("state", "")) == "whole", "an undamaged machine reads as whole")

	var yard_state := ""
	var post_state := ""
	for entry in buildings:
		var building: Dictionary = entry
		if str(building.get("id", "")) == BoardVerificationCampaign.YARD_ID:
			yard_state = str(building.get("state", ""))
		if str(building.get("id", "")) == BoardVerificationCampaign.POST_ID:
			post_state = str(building.get("state", ""))
	check(yard_state == "operational", "the yard must cross as operational")
	check(post_state == "under_construction", "the half-built post must cross as under_construction")

	# The board's half: one node per instance, named from the instance and not
	# from the record, standing on the cell the snapshot names.
	board.project_snapshot(developed)
	var yard := node_for(board, BoardDevelopment.BUILDING, BoardVerificationCampaign.YARD_ID)
	var post := node_for(board, BoardDevelopment.BUILDING, BoardVerificationCampaign.POST_ID)
	var machine := node_for(board, BoardDevelopment.MACHINE, BoardVerificationCampaign.DOG_ID)
	check(yard != null, "the yard must stand on the board as its own node")
	check(post != null, "the half-built post must stand on the board as its own node")
	check(machine != null, "the machine must stand on the board as its own node")
	if yard == null or post == null or machine == null:
		board.queue_free()
		finish()
		return

	check(on_cell(board, yard, BoardVerificationCampaign.YARD_CELL), "the yard must stand on the cell the snapshot names")
	check(on_cell(board, machine, BoardVerificationCampaign.YARD_CELL), "the machine must stand on the cell its yard stands on")
	check(on_cell(board, post, BoardVerificationCampaign.POST_CELL), "the post must stand on the cell the snapshot names")
	check(yard.position.distance_to(machine.position) > 1.0, "two things standing in one cell must not stand in the same place")
	check(machine.position.z > yard.position.z, "a machine is parked in front of the sheds, not in line behind one twenty metres wide")

	# P12's kit built them, and the record decided the size. Nothing here draws
	# a box of its own.
	check(bool(yard.get_meta("blockout_placeholder", false)), "everything standing on the island is marked procedural blockout")
	check(str(yard.get_meta("def_id", "")) == "building.machine_shop", "the node must remember the record it was raised from")
	check(yard.get_node_or_null("Envelope") != null, "a standing building is P12's whole kit")
	check(yard.get_node_or_null("Sockets") != null, "a standing building keeps the record's sockets")
	check(machine.get_node_or_null("Chassis") != null, "a machine is P12's machine kit")
	check(machine.get_node_or_null("Pivots") != null, "a machine keeps the pivots a rig will need")
	check(machine.get_node_or_null("FactionApron") != null, "a machine stands on its faction's apron, which is where its faction is said and what makes a two-metre machine visible at all")
	check(yard.get_node_or_null("FactionApron") != null, "a building stands on its faction's apron")
	check(wears(yard.get_node("FactionApron") as Node3D, BoardPalette.controller_tint("faction.michael")), "the apron under a building is washed in the colour of the faction that holds it")
	check(wears(machine.get_node("FactionApron") as Node3D, BoardPalette.controller_tint("faction.michael")), "the apron under a machine is washed in the colour of the faction that built it")

	# A building under construction is the first tier alone, in the placeholder
	# material; a standing one is every authored tier, washed in the colour of
	# the faction holding it.
	var record: Dictionary = catalog.get_record("building.machine_shop")
	var post_record: Dictionary = catalog.get_record("building.coast_watch_post")
	var yard_footprint: Dictionary = (board.cells[BoardVerificationCampaign.YARD_CELL] as Dictionary).get("footprint", {})
	var post_footprint: Dictionary = (board.cells[BoardVerificationCampaign.POST_CELL] as Dictionary).get("footprint", {})
	var full_height: float = BuildingBlockoutKit.envelope_metres(record, yard_footprint, BoardDevelopment.TIER_FULL).y
	var first_height: float = BuildingBlockoutKit.envelope_metres(post_record, post_footprint, BoardDevelopment.TIER_FIRST).y
	check(absf(BlockoutKit.bounds_of(yard).size.y - full_height) < 0.6, "a standing building stands at every authored tier")
	check(absf(BlockoutKit.bounds_of(post).size.y - first_height) < 0.6, "a building under construction stands at its first tier alone")
	# The massing is what the two states differ in. The faction apron under a
	# building is the *ground* it stands on and is washed either way -- a
	# half-built shed is on somebody's plot from the hour it is started.
	check(wears_placeholder(post.get_node("Envelope") as Node3D), "a building under construction wears the placeholder material over the whole of its massing")
	check(not wears_placeholder(yard.get_node("Envelope") as Node3D), "a finished building does not wear the placeholder material")
	check(not wears(yard, BoardPalette.controller_tint("faction.pirates")), "a building does not wear another faction's wash")

	# The room distance: the same nodes, on the authored sockets of the room
	# being looked at, and nothing standing on any other cell.
	board.set_distance(IsometricBoard.DISTANCE_ROOM)
	check(board.party_cell_id() == "world.cell.black_beach", "a fresh campaign is looked at from the beach it began on")
	check(not yard.visible, "the room distance is one room: what stands on another cell is not drawn")
	check(not machine.visible, "the room distance is one room: what stands on another cell is not drawn")
	board.set_distance(IsometricBoard.DISTANCE_WORLD)

	# Walk the party to the terrace so the room being looked at is the one the
	# yard stands on, and the same node is asserted on its authored socket.
	var arrived := walk_to_terrace(board, developed)
	check(board.party_cell_id() == BoardVerificationCampaign.YARD_CELL, "the opening must walk the party to the terrace")
	check(bool(arrived.get("configured", false)), "the walk to the terrace must be accepted by the native bridge")
	board.set_distance(IsometricBoard.DISTANCE_ROOM)
	check(yard.visible, "the room being looked at draws what stands in it")
	check(machine.visible, "the room being looked at draws what stands in it")
	check(yard.position.is_equal_approx(BoardDevelopment.room_place(board.cells[BoardVerificationCampaign.YARD_CELL], BoardDevelopment.BUILDING, 0, Vector2(yard.get_meta("row_width"), yard.get_meta("row_depth")))), "at the room distance a building stands on the authored socket its role names")
	# The machine takes the first socket of its own role that the yard is not
	# already standing on, which is what `taken` is for.
	check(machine.position.is_equal_approx(BoardDevelopment.room_place(board.cells[BoardVerificationCampaign.YARD_CELL], BoardDevelopment.MACHINE, 0, apron_of(machine), [BoardDevelopment.ground_rect(yard.position, apron_of(yard))])), "at the room distance a machine stands on the authored socket its role names")
	check(not BoardDevelopment.ground_rect(machine.position, apron_of(machine)).intersects(BoardDevelopment.ground_rect(yard.position, apron_of(yard))), "a machine does not stand on ground a building is already standing on")
	# Brief section 18: nothing essential outside the box. The whole ground a
	# thing takes up is held inside the room, not only the point it stands on.
	check(inside_footprint(board.cells[BoardVerificationCampaign.YARD_CELL], yard.position, apron_of(yard)), "the whole ground a building takes up stands inside the room's footprint")
	check(inside_footprint(board.cells[BoardVerificationCampaign.YARD_CELL], machine.position, apron_of(machine)), "the whole ground a machine takes up stands inside the room's footprint")
	board.set_distance(IsometricBoard.DISTANCE_WORLD)
	check(yard.position.is_equal_approx(BoardDevelopment.row_places(board.cells[BoardVerificationCampaign.YARD_CELL], [apron_of(yard), apron_of(machine)], [BoardDevelopment.BUILDING, BoardDevelopment.MACHINE])[0]), "back at the world distance the same node stands on the tile again")

	# An instance the snapshot stops carrying stops standing on the island, and
	# the node is freed rather than hidden.
	var cleared: Dictionary = BoardVerificationCampaign.develop_island(campaign_session, BoardVerificationCampaign.HALF_CLEARED)
	check(bool(cleared.get("configured", false)), "the bridge must accept the cleared save: %s" % str(cleared.get("error", "")))
	board.project_snapshot(cleared)
	await process_frame
	check(node_for(board, BoardDevelopment.BUILDING, BoardVerificationCampaign.YARD_ID) != null, "a building the snapshot still carries goes on standing")
	check(node_for(board, BoardDevelopment.BUILDING, BoardVerificationCampaign.POST_ID) == null, "a building the snapshot no longer carries stops standing on the island")
	check(node_for(board, BoardDevelopment.MACHINE, BoardVerificationCampaign.DOG_ID) == null, "a machine the snapshot no longer carries stops standing on the island")
	check(not board.development_nodes.has(BoardDevelopment.node_name_for(BoardDevelopment.BUILDING, BoardVerificationCampaign.POST_ID)), "the freed node must leave the board's own record of what is standing")

	# Every metre and share this lane wrote down is still marked as the open
	# decision it is. O3 is Open; a constant that quietly stopped saying so would
	# be a decision taken by nobody.
	for key in ["ROW_DEPTH_SHARE", "ROW_GAP_METRES", "ORDER_KINDS", "ROOM_ROLE", "APRON_MARGIN_METRES", "APRON_MINIMUM_METRES", "APRON_THICKNESS_METRES"]:
		check(BoardDevelopment.OPEN_DIMENSIONS.has(key), "%s must be declared as an open dimension" % key)
		check(str(BoardDevelopment.OPEN_DIMENSIONS.get(key, "")).begins_with("needs decision"), "%s must still say it is a decision nobody has taken" % key)

	board.queue_free()
	await process_frame
	finish()


## Walks the party from the beach to the terrace along the authored opening,
## projecting each snapshot, so the room distance can be asked about the cell the
## yard stands on.
func walk_to_terrace(board: IsometricBoard, from_snapshot: Dictionary) -> Dictionary:
	var snapshot := from_snapshot
	campaign_session.use_anchor(BoardVerificationCampaign.OPENING_SALVAGE)
	for portal_id in BoardVerificationCampaign.OPENING_TRAVEL:
		snapshot = campaign_session.travel(str(portal_id))
		board.project_snapshot(snapshot)
	return snapshot


func node_for(board: IsometricBoard, kind: String, instance_id: String) -> Node3D:
	return board.development_root.get_node_or_null(BoardDevelopment.node_name_for(kind, instance_id)) as Node3D


## The node stands on the named cell: same ground rectangle, at the cell's own
## elevation. The board is asked where the cell is; the node is held to it.
func on_cell(board: IsometricBoard, node: Node3D, cell_id: String) -> bool:
	var cell: Dictionary = board.cells[cell_id]
	return absf(node.position.y - BoardBlockout.centre_of(cell).y) < 0.001 and inside_footprint(cell, node.position)


func apron_of(node: Node3D) -> Vector2:
	return Vector2(float(node.get_meta("row_width", 0.0)), float(node.get_meta("row_depth", 0.0)))


## The ground `point` takes up, `ground` metres across, is inside the cell's
## authored footprint. With no ground given it is the point alone.
func inside_footprint(cell: Dictionary, point: Vector3, ground: Vector2 = Vector2.ZERO) -> bool:
	var centre: Vector3 = BoardBlockout.centre_of(cell)
	var footprint: Dictionary = cell.get("footprint", {})
	return absf(point.x - centre.x) + ground.x * 0.5 <= float(footprint.get("width_metres", 0.0)) * 0.5 + 0.001 \
		and absf(point.z - centre.z) + ground.y * 0.5 <= float(footprint.get("depth_metres", 0.0)) * 0.5 + 0.001


## Every surface of the node wears the project's own placeholder shader --
## `render/materials/placeholder.tres`, the deliberately ugly one P12 reserved
## for "this has not been art-directed yet". The shader is what is compared,
## because that resource is duplicated per surface by
## `SetpieceMeshFactory.library_material` and two duplicates are not equal.
func wears_placeholder(node: Node3D) -> bool:
	var placeholder := BlockoutKit.placeholder() as ShaderMaterial
	if placeholder == null:
		return false
	var seen := false
	for mesh_instance in mesh_instances(node):
		var surface := mesh_instance.mesh.material as ShaderMaterial
		if surface == null or surface.shader != placeholder.shader:
			return false
		seen = true
	return seen


func wears(node: Node3D, tint: Color) -> bool:
	for mesh_instance in mesh_instances(node):
		var surface := mesh_instance.mesh.material as ShaderMaterial
		if surface == null:
			continue
		var albedo: Variant = surface.get_shader_parameter("albedo")
		if albedo is Color and (albedo as Color).is_equal_approx(tint):
			return true
	return false


func mesh_instances(node: Node3D) -> Array[MeshInstance3D]:
	var found: Array[MeshInstance3D] = []
	var pending: Array[Node] = [node]
	while not pending.is_empty():
		var current: Node = pending.pop_back()
		if current is MeshInstance3D and (current as MeshInstance3D).mesh != null:
			found.append(current as MeshInstance3D)
		for child in current.get_children():
			pending.append(child)
	return found


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Board development tests passed.")
	quit(0)
