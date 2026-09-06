extends SceneTree

## Exercises the actual GDScript screen against the native Rust expedition
## bridge. The test proves that the route buttons are a projection of legal
## native commands and that selected travel updates the rendered location.

## What a contested road adds to its authored risk. The number is
## `geography.rs`'s `CONTESTED_RISK_MODIFIER` and that constant remains its
## owner; this is the expected value the suite holds the drawn button to, so a
## change to the rule that did not reach the screen fails here.
const CONTESTED_RISK_MODIFIER := 2

## The one road out of Black Beach, and where it ends. P7 compares the three
## RTS controls against each other, so the road and the destination are named
## once here rather than three times below.
const BEACH_ROAD := "world.portal.black_beach_to_damaged_estate"
const RTS_DESTINATION := "world.cell.damaged_estate"

var failures := 0


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Expedition prototype native test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	var scene := load("res://scenes/world/expedition_prototype.tscn") as PackedScene
	check(scene != null, "expedition prototype scene must load")
	if scene == null:
		finish()
		return
	var expedition := scene.instantiate() as ExpeditionPrototype
	root.add_child(expedition)
	await process_frame
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "fresh expedition must begin at Black Beach")
	check(expedition.get_legal_route_count() == 1, "Black Beach must expose exactly one legal route")
	# The tidal cut also leaves the beach, but it is gated on the map table's
	# discovery and must not be offered until then: a locked door drawn as a
	# button tells the player the door exists.
	# Roads cost rations and the party lands with none, so the slice through
	# the engine begins the way the Rust slice does: salvage the wreck first.
	# B4: the action list is a projection of the native legal-command list, so
	# the beach draws the wreck it can salvage and both observations it can read.
	check(expedition.get_action_commands().has("anchor_action:anchor.black_beach.salvage_point"), "the beach must draw its authored salvage anchor as an action")
	check(expedition.get_action_commands().has("inspect:observation.black_beach.wreck"), "the beach must draw its authored observations as actions")
	check(expedition.get_action_commands().has("resolve_midnight"), "a midnight control must be offered while no encounter is pending")
	# B15: C9's six faction records reached the bundle, the port forwarded them
	# and the bridge accepted them, so the strategic hours the engine runs score
	# the same registry the Rust harness scores. The snapshot says who is on the
	# island and what each is doing -- and says nothing numeric: brief section 9
	# forbids exposing raw utility arithmetic, so a score or a weight appearing
	# on this array is the failure, not a bonus.
	var projected_factions: Array = expedition.get_authoritative_snapshot().get("factions", [])
	check(projected_factions.size() == 6, "the bridge must project the six authored faction records it loaded")
	for projected in projected_factions:
		var entry := projected as Dictionary
		check(str(entry.get("id", "")).begins_with("faction."), "every projected faction must carry its stable ID")
		check(entry.has("strategic_state") and entry.has("current_goals"), "every projected faction must carry its board position and its goals")
		for key in entry.keys():
			var name := str(key)
			check(not ("score" in name or "weight" in name), "no utility score or weight may cross the bridge: %s" % name)
	# B16: C10's three building records made the same trip. The snapshot carries
	# the loaded registry's IDs and nothing else from a record -- a tier's hit
	# points or a production interval crossing here would be the record being
	# read through the wrong door.
	var projected_buildings: Array = expedition.get_authoritative_snapshot().get("buildings", [])
	check(projected_buildings.size() == 3, "the bridge must project the three authored building records it loaded")
	for projected_building in projected_buildings:
		check(str(projected_building).begins_with("building."), "every projected building must be a stable record ID")
	# B19: and C14's two machine records made the same trip, on the same terms --
	# registry IDs only, so a crew requirement or a fuel count never crosses here.
	var projected_machines: Array = expedition.get_authoritative_snapshot().get("machines", [])
	check(projected_machines.size() == 2 and str(projected_machines[0]) == "machine.mechanical_dog" and str(projected_machines[1]) == "machine.steam_wagon", "the bridge must project the two authored machine record IDs it loaded")
	var salvage: Dictionary = expedition.request_anchor("anchor.black_beach.salvage_point")
	check(bool(salvage.get("configured", false)), "the wreck must be salvageable through the native bridge")
	check(int((salvage.get("anchor_outcome", {}) as Dictionary).get("rations_gained", 0)) >= 4, "salvaging the wreck must yield its authored floor of four rations")
	check(expedition.get_legal_route_count() == 1, "salvaging must not change the one legal departure")
	check(not expedition.get_action_commands().has("anchor_action:anchor.black_beach.salvage_point"), "an anchor spent today must leave the drawn action list")
	var inspected: Dictionary = expedition.request_inspect("observation.black_beach.wreck")
	check(bool(inspected.get("configured", false)), "an observation offered here must be inspectable through the native bridge")
	var discoveries: Array = inspected.get("discoveries", [])
	check(discoveries.has("observation.black_beach.wreck"), "inspecting must record the authored observation as a native discovery")
	var refused: Dictionary = expedition.request_inspect("observation.damaged_estate.veranda")
	check(not bool(refused.get("configured", true)), "an observation that belongs to another cell must be refused, not recorded")
	var midnight: Dictionary = expedition.request_midnight()
	check(bool(midnight.get("configured", false)), "midnight must resolve through the native bridge while no encounter is pending")
	check(int(midnight.get("campaign_day", 0)) == 2, "midnight must turn the campaign day")
	check(expedition.get_action_commands().has("anchor_action:anchor.black_beach.salvage_point"), "midnight must make the spent anchor drawable again")
	expedition.request_travel("world.portal.black_beach_to_damaged_estate")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.damaged_estate", "travel must use the native portal result")
	expedition.request_travel("world.portal.damaged_estate_to_river_landing")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.river_landing", "estate departure must arrive at River Landing")
	check(expedition.get_legal_route_count() == 3, "River Landing must project its three legal native routes")
	# B14: the safe road's danger is control, drawn. Nothing here edits a route
	# record; the claim changes who holds the landing and the button changes
	# with it, which is the brief's "a safe road becomes contested" on screen.
	var session: Node = expedition.campaign_session
	var safe_road := "world.portal.river_landing_to_reception_terrace_safe_road"
	var uncontested_risk := expedition.get_drawn_route_risk(safe_road)
	check(uncontested_risk > 0, "the safe road must draw the risk the native snapshot gives it")
	check(not expedition.get_drawn_route_is_contested(safe_road), "a campaign that holds nothing must draw no contested road")
	var claimed: Dictionary = session.set_control("world.cell.river_landing", "faction.pirates")
	check(bool(claimed.get("configured", false)), "claiming an authored cell must be accepted by the native bridge")
	var control_events: Array = claimed.get("events", [])
	check(control_events.size() == 1, "one handover must project exactly one control_changed event")
	check(str((control_events[0] as Dictionary).get("kind", "")) == "control_changed", "the projected event must be the control change")
	check(session.controller_of("world.cell.river_landing") == "faction.pirates", "the effective controller must be the party that took the cell")
	expedition.project_snapshot(claimed, "CONTROL CHANGED")
	check(expedition.get_drawn_route_risk(safe_road) == uncontested_risk + CONTESTED_RISK_MODIFIER, "holding one endpoint must raise the safe road's drawn risk by the contested modifier")
	check(expedition.get_drawn_route_is_contested(safe_road), "a contested road must be marked contested on its button")
	check(int(session.effective_risk(safe_road)) == uncontested_risk + CONTESTED_RISK_MODIFIER, "the bridge's single-route risk query must agree with the drawn number")
	var released: Dictionary = session.set_control("world.cell.river_landing", "")
	check(bool(released.get("configured", false)), "releasing a held cell must be accepted by the native bridge")
	check(session.controller_of("world.cell.river_landing") == "", "a released cell must read as unheld")
	expedition.project_snapshot(released, "CONTROL RELEASED")
	check(expedition.get_drawn_route_risk(safe_road) == uncontested_risk, "releasing the landing must put the safe road's drawn risk back")
	check(not expedition.get_drawn_route_is_contested(safe_road), "a released road must lose its contested marker")
	check(expedition.get_legal_route_count() == 3, "control changes must not add or remove a legal departure")
	expedition.request_travel("world.portal.river_landing_to_reception_terrace_safe_road")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.reception_terrace", "safe road must arrive at Reception Terrace")
	var pending_encounter: Dictionary = expedition.get_authoritative_snapshot().get("pending_encounter", {})
	check(pending_encounter.get("encounter_id") == "encounter.prototype.returning_names", "Reception Terrace must arm its authored Razorbeak encounter")
	check(pending_encounter.get("battle_id") == "battle.prototype.returning_names", "Encounter handoff must retain the authored battle ID")
	check(expedition.get_legal_route_count() == 2, "Pending encounter must replace route controls with its handoff record and one real engage action")
	check(expedition.route_list.get_node_or_null("EngagePendingEncounter") != null, "Pending encounter must expose its authored engage action")
	expedition.queue_free()
	await process_frame
	var reloaded_expedition := scene.instantiate() as ExpeditionPrototype
	root.add_child(reloaded_expedition)
	await process_frame
	check(reloaded_expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.reception_terrace", "Scene reload must retain the one native campaign location")
	check(not (reloaded_expedition.get_authoritative_snapshot().get("pending_encounter", {}) as Dictionary).is_empty(), "Scene reload must retain the pending native encounter")
	var terrace := catalog_record(reloaded_expedition, "world.cell.reception_terrace")
	var resolved_copy := reloaded_expedition.get_authoritative_snapshot()
	resolved_copy.pending_encounter = {}
	resolved_copy.resolved_encounter_ids = ["encounter.prototype.returning_names"]
	check("AFTERMATH" in reloaded_expedition.make_description(terrace, resolved_copy), "A resolved authored encounter must project its content-authored aftermath")
	var estate := catalog_record(reloaded_expedition, "world.cell.damaged_estate")
	var estate_copy := resolved_copy.duplicate(true)
	estate_copy.active_location_id = "world.cell.damaged_estate"
	estate_copy.estate_upgrades = ["estate_upgrade.river_gate_alarm"]
	check("HOUSEHOLD RESULT" in reloaded_expedition.make_description(estate, estate_copy), "The Estate must project its authored response to the native household upgrade")
	check("RIVER GATE ALARM" in reloaded_expedition.initial_status_message(estate_copy), "The Estate status must name the unlocked household change")
	reloaded_expedition.queue_free()
	await process_frame
	await check_rts_controls(scene)
	await check_the_screen_is_the_board(scene)
	finish()


## P10's done-when: the screen the player uses is the isometric board, and the
## RTS grammar on it is the one P7 shipped.
##
## Three cells are crossed by select-then-order on the 3D board, through the
## live bridge, and at every stop two things are held: the miniature stands on
## the cell the snapshot names, and the commands the screen draws are exactly
## the commands the snapshot calls legal. The second is the one that says the
## board swap changed nothing about legality -- the drawn departures are the
## native `legal_route_commands` and the drawn actions are the native
## `legal_commands`, which is what the deleted 2D board's own suite asserted.
func check_the_screen_is_the_board(scene: PackedScene) -> void:
	var expedition: ExpeditionPrototype = await fresh_expedition(scene)
	var board: BoardSurface = expedition.board
	check(board != null, "the expedition screen must host a board")
	check(board.board != null, "the hosted board must be the isometric board scene")
	check(board.get_parent() != null and board.is_inside_tree(), "the board must sit in the screen's own layout")
	check(board.board.get_viewport() is SubViewport, "the 3D board must be drawn through the screen's board viewport")
	check_drawn_commands_match_the_snapshot(expedition, "world.cell.black_beach")

	var crossings := [
		["world.cell.damaged_estate", "world.portal.black_beach_to_damaged_estate"],
		["world.cell.river_landing", "world.portal.damaged_estate_to_river_landing"],
		["world.cell.reception_terrace", "world.portal.river_landing_to_reception_terrace_safe_road"]
	]
	for crossing in crossings:
		var destination := str((crossing as Array)[0])
		var portal_id := str((crossing as Array)[1])
		var origin := str(expedition.get_authoritative_snapshot().get("active_location_id", ""))
		# Select first, exactly as a player does: a left-click on the party's own
		# tile. It selects and it does not travel.
		expedition.select_tile(origin)
		check(board.party_selected(), "a left-click on the party's tile must select the party on the 3D board")
		check(str(expedition.get_authoritative_snapshot().get("active_location_id", "")) == origin, "selecting must never move the party")
		# Then order: the right-click on the destination tile.
		var ordered: Dictionary = expedition.order_move_to_tile(destination)
		check(bool(ordered.get("ordered", false)), "a right-click on %s must issue the order" % destination)
		check(str(ordered.get("portal_id", "")) == portal_id, "the order must run along the authored portal %s" % portal_id)
		check(str(expedition.get_authoritative_snapshot().get("active_location_id", "")) == destination, "confirmed travel must land the party at %s" % destination)
		# The miniature moved because the simulation did, and it stands where the
		# snapshot says. This is the assertion the deleted 2D board could not make.
		var miniature_cell := board.board.party_cell_id()
		check(miniature_cell == destination, "the miniature must stand in %s after confirmed travel" % destination)
		check((board.board.party_miniature.position as Vector3).is_equal_approx(board.board.stand_point(board.board.cells[destination])), "the miniature must snap onto %s's own tile" % destination)
		check_drawn_commands_match_the_snapshot(expedition, destination)

	# An order to a place no legal road reaches is refused in the status line and
	# never reaches the bridge, on this board as on the last one.
	var before_refusal := JSON.stringify(expedition.get_authoritative_snapshot())
	var refusal: Dictionary = expedition.order_move_to_tile("world.cell.black_beach")
	check(not bool(refusal.get("ordered", true)), "an order to an unreachable cell must be refused")
	check(str(refusal.get("reason", "")) == "no_legal_route", "the refusal must name the missing road")
	check(JSON.stringify(expedition.get_authoritative_snapshot()) == before_refusal, "a refused order must leave the native snapshot untouched")
	check(not expedition.status_label.text.is_empty(), "a refused order must say so in the status line")
	# And the refusal that matters most: the terrace's armed encounter makes every
	# road out of it illegal, but the road back to the landing is still *authored*
	# and still drawn on the island. So this is the case where the graph says yes
	# and the snapshot says no, and the snapshot is the one the board obeys --
	# without it, an order could be legal because a road exists.
	var terrace_snapshot := expedition.get_authoritative_snapshot()
	check(not (terrace_snapshot.get("pending_encounter", {}) as Dictionary).is_empty(), "the terrace must have armed its authored encounter")
	check(expedition.board.board.tethers.has("world.portal.reception_terrace_to_river_landing"), "the road back to the landing must still be drawn on the board")
	check(expedition.board.portal_from_active_cell_to("world.cell.river_landing").is_empty(), "a road the snapshot does not call legal must not be found by the board")
	var held_back: Dictionary = expedition.order_move_to_tile("world.cell.river_landing")
	check(not bool(held_back.get("ordered", true)), "an order along an authored road the snapshot does not call legal must be refused")
	check(JSON.stringify(expedition.get_authoritative_snapshot()) == before_refusal, "that refusal must not reach the bridge either")

	# The three distances are camera modes: they move the camera and nothing
	# else. The party does not move and the snapshot does not change.
	var standing: Vector3 = board.board.party_miniature.position
	var unchanged := JSON.stringify(expedition.get_authoritative_snapshot())
	for keycode in [KEY_F1, KEY_F2, KEY_F3]:
		check(expedition.press_camera_hotkey(keycode), "F%d must be one of the board's camera keys" % (int(keycode) - int(KEY_F1) + 1))
	check(board.distance() == IsometricBoard.DISTANCE_ROOM, "F3 must leave the camera at the room distance")
	check((board.board.party_miniature.position as Vector3).is_equal_approx(standing), "a camera mode must never move the party")
	check(JSON.stringify(expedition.get_authoritative_snapshot()) == unchanged, "a camera mode must never touch the snapshot")
	check(not expedition.press_camera_hotkey(KEY_F5), "a key that is not one of the three must not be swallowed as a camera mode")
	expedition.press_camera_hotkey(KEY_F1)
	expedition.queue_free()
	await process_frame


## The controls the screen is drawing are the commands the snapshot calls legal:
## the departures are `legal_route_commands`, the actions are `legal_commands`
## plus the midnight the native transaction always accepts. Nothing is drawn
## that the simulation did not name, and nothing it named is missing.
func check_drawn_commands_match_the_snapshot(expedition: ExpeditionPrototype, where: String) -> void:
	var snapshot := expedition.get_authoritative_snapshot()
	if not (snapshot.get("pending_encounter", {}) as Dictionary).is_empty():
		return
	var legal_routes: Array[String] = []
	for command in snapshot.get("legal_route_commands", []):
		legal_routes.append(str(command).trim_prefix("travel:"))
	legal_routes.sort()
	var drawn_routes: Array[String] = []
	for portal_id in expedition.route_hotkey_portal_ids():
		drawn_routes.append(str(portal_id))
	drawn_routes.sort()
	check(drawn_routes == legal_routes, "at %s the drawn departures must be the native legal route commands" % where)
	var expected_actions: Array[String] = []
	for command in snapshot.get("legal_commands", []):
		var command_text := str(command)
		if command_text.begins_with("anchor_action:") or command_text.begins_with("inspect:"):
			expected_actions.append(command_text)
	expected_actions.append("resolve_midnight")
	expected_actions.sort()
	var drawn_actions: Array[String] = expedition.get_action_commands()
	drawn_actions.sort()
	check(drawn_actions == expected_actions, "at %s the drawn actions must be the native legal commands" % where)


## P7 (B18 resumed): the board is an RTS board. One unit, one order, three ways
## to give it -- a right-click on the destination tile, the digit drawn on the
## departure, and the words of the departure itself -- and all three must land
## the party on the same cell. Each pass starts from a fresh campaign so the
## three are compared against the same opening position and not against each
## other's leftovers.
func check_rts_controls(scene: PackedScene) -> void:
	var by_right_click: ExpeditionPrototype = await fresh_expedition(scene)
	check(by_right_click.get_legal_route_count() == 1, "the salvaged beach must still offer exactly one departure")
	# Left-click is a look and a selection, never a move. Both the party's own
	# tile and another one are tried, because the two branches of select_tile
	# are the two ways a click could have travelled by mistake.
	by_right_click.select_tile("world.cell.damaged_estate")
	check(by_right_click.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a left-click on another tile must look at it and never travel")
	by_right_click.select_tile("world.cell.black_beach")
	check(by_right_click.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a left-click on the party's own tile must select it and never travel")
	check(by_right_click.board.party_selected(), "a left-click on the party's tile must select the party")
	# An order to a tile no legal portal reaches is refused here and never
	# reaches the bridge, so the whole snapshot must come back identical.
	var before_refusal := JSON.stringify(by_right_click.get_authoritative_snapshot())
	var refusal: Dictionary = by_right_click.order_move_to_tile("world.cell.processional_ramp")
	check(not bool(refusal.get("ordered", true)), "a right-click on an unreachable tile must refuse the order")
	check(str(refusal.get("reason", "")) == "no_legal_route", "the refusal must name the missing road")
	check(JSON.stringify(by_right_click.get_authoritative_snapshot()) == before_refusal, "a refused order must leave the native snapshot untouched")
	var right_clicked: Dictionary = by_right_click.order_move_to_tile("world.cell.damaged_estate")
	check(bool(right_clicked.get("ordered", false)), "a right-click on a reachable tile must issue the order")
	check(str(right_clicked.get("portal_id", "")) == BEACH_ROAD, "the order must run along the authored portal from the active cell")
	check(by_right_click.get_authoritative_snapshot().get("active_location_id") == RTS_DESTINATION, "a right-click order must move the party through the native bridge")
	by_right_click.queue_free()
	await process_frame

	var by_hotkey: ExpeditionPrototype = await fresh_expedition(scene)
	var numbered := by_hotkey.route_hotkey_portal_ids()
	check(numbered.size() == 1 and numbered[0] == BEACH_ROAD, "the departure list must number the one legal road as departure one")
	check("[1]" in by_hotkey.get_route_button(BEACH_ROAD).text, "a hotkey the player cannot see is not a control: the digit must be drawn on the departure")
	var no_such_departure: Dictionary = by_hotkey.press_route_hotkey(4)
	check(not bool(no_such_departure.get("ordered", true)), "a digit with no departure behind it must refuse")
	check(by_hotkey.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a refused hotkey must not move the party")
	var hotkeyed: Dictionary = by_hotkey.press_route_hotkey(1)
	check(bool(hotkeyed.get("ordered", false)), "the departure's digit must issue the order")
	check(by_hotkey.get_authoritative_snapshot().get("active_location_id") == RTS_DESTINATION, "the hotkey must land the party where the right-click did")
	by_hotkey.queue_free()
	await process_frame

	var by_words: ExpeditionPrototype = await fresh_expedition(scene)
	# Brief section 14: a held game takes no orders. The pause is the owner's
	# verdict, asked here through the same autoload the screen asks.
	var game_pause: Node = root.get_node_or_null("/root/GamePause")
	check(game_pause != null, "the pause autoload must be available to the expedition screen")
	game_pause.pause("expedition_prototype_test")
	var while_paused: Dictionary = by_words.order_move_to_tile(RTS_DESTINATION)
	check(not bool(while_paused.get("ordered", true)), "a held game must take no move order")
	check(str(while_paused.get("reason", "")) == "game_paused", "a refusal while paused must say so")
	check(by_words.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a held game must leave the party where it stands")
	game_pause.resume("expedition_prototype_test")
	by_words.get_route_button(BEACH_ROAD).pressed.emit()
	check(by_words.get_authoritative_snapshot().get("active_location_id") == RTS_DESTINATION, "the words must land the party where the right-click and the hotkey did")
	by_words.queue_free()
	await process_frame


## A campaign at its opening position with the wreck already salvaged. Roads
## cost rations and the party lands with none, so this is the earliest point at
## which any of the three controls can be compared with the others.
func fresh_expedition(scene: PackedScene) -> ExpeditionPrototype:
	var session: Node = root.get_node_or_null("/root/CampaignSession")
	check(session != null, "the campaign session autoload must be available")
	if session != null:
		session.reset_for_test()
	var expedition := scene.instantiate() as ExpeditionPrototype
	root.add_child(expedition)
	await process_frame
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a reset campaign must open on Black Beach again")
	var salvage: Dictionary = expedition.request_anchor("anchor.black_beach.salvage_point")
	check(bool(salvage.get("configured", false)), "the reset campaign must be able to salvage the wreck for the rations a road costs")
	return expedition


func catalog_record(expedition: ExpeditionPrototype, record_id: String) -> Dictionary:
	return expedition.catalog.get_record(record_id)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Expedition prototype tests passed.")
	quit(0)
