extends SceneTree

## Exercises the actual GDScript screen against the native Rust expedition
## bridge. The test proves that the route buttons are a projection of legal
## native commands and that selected travel updates the rendered location.

## What a contested road adds to its authored risk. The number is
## `geography.rs`'s `CONTESTED_RISK_MODIFIER` and that constant remains its
## owner; this is the expected value the suite holds the drawn button to, so a
## change to the rule that did not reach the screen fails here.
const CONTESTED_RISK_MODIFIER := 2

## The one legal departure from Black Beach on day one, and where it lands. The
## RTS block below orders that same move three different ways and holds all
## three to this destination.
const BEACH_DEPARTURE := "world.portal.black_beach_to_damaged_estate"
const BEACH_DESTINATION := "world.cell.damaged_estate"
## No road leaves the beach for the terrace: it is two cells away. Ordering a
## move there is the refusal case.
const UNREACHABLE_TILE := "world.cell.reception_terrace"

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
	# B18: RTS controls. The same order -- move to the Damaged Estate -- given
	# three ways, from three fresh campaigns, must land on the same cell. Each
	# run resets the session so the party is back on the beach with the same one
	# legal departure, which is what makes "the same order" a fair comparison.
	var by_words: ExpeditionPrototype = await fresh_expedition(scene)
	check(by_words.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a reset session must put the party back on the beach")
	var words_button := by_words.get_route_button(BEACH_DEPARTURE)
	check(words_button != null, "the beach departure must be drawn as a route entry")
	check(words_button != null and words_button.text.begins_with("[1]"), "a route entry must show the digit that issues it")
	if words_button != null:
		words_button.pressed.emit()
	check(by_words.get_authoritative_snapshot().get("active_location_id") == BEACH_DESTINATION, "the words must issue the move order")
	by_words.queue_free()
	await process_frame

	var by_hotkey: ExpeditionPrototype = await fresh_expedition(scene)
	var hotkey_portals: Array[String] = by_hotkey.route_hotkey_portal_ids()
	check(hotkey_portals.size() == 1 and hotkey_portals[0] == BEACH_DEPARTURE, "the hotkey order must be the route list's own order")
	var hotkey_result: Dictionary = by_hotkey.press_route_hotkey(1)
	check(bool(hotkey_result.get("configured", false)), "hotkey 1 must be accepted by the native bridge")
	check(by_hotkey.get_authoritative_snapshot().get("active_location_id") == BEACH_DESTINATION, "the hotkey must land where the words land")
	var no_such_key: Dictionary = by_hotkey.press_route_hotkey(9)
	check(not bool(no_such_key.get("configured", true)), "a digit with no route on it must be refused, not guessed at")
	check(by_hotkey.get_authoritative_snapshot().get("active_location_id") == BEACH_DESTINATION, "a refused hotkey must not move the party")
	by_hotkey.queue_free()
	await process_frame

	var by_tile: ExpeditionPrototype = await fresh_expedition(scene)
	var board: ExpeditionRouteBoard = by_tile.route_board
	check(board.tile_at_point(tile_point(board, BEACH_DESTINATION)) == BEACH_DESTINATION, "a click on a tile must hit that tile")
	check(board.tile_at_point(tile_point(board, UNREACHABLE_TILE)) == UNREACHABLE_TILE, "two tiles must not answer for one another")
	check(board.is_reachable(BEACH_DESTINATION), "the tile the one legal departure lands on must tint as reachable")
	check(not board.is_reachable(UNREACHABLE_TILE), "a tile no legal departure reaches must not tint as reachable")
	check(board.portal_to_cell(BEACH_DESTINATION) == BEACH_DEPARTURE, "the tile order must resolve to the legal portal, not to a rule of its own")
	# The whole snapshot as text: Dictionary equality in GDScript is not a
	# content comparison, and "nothing moved" has to mean every field.
	var standing := JSON.stringify(by_tile.get_authoritative_snapshot())
	# Left-click never travels: not on an unreachable tile, not on a reachable
	# one, and not on the party's own tile.
	click_tile(board, UNREACHABLE_TILE, MOUSE_BUTTON_LEFT)
	click_tile(board, BEACH_DESTINATION, MOUSE_BUTTON_LEFT)
	click_tile(board, "world.cell.black_beach", MOUSE_BUTTON_LEFT)
	check(by_tile.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "a left-click must never travel")
	check(by_tile.selected_cell_id == "world.cell.black_beach", "a left-click must leave the tile it hit selected")
	# A right-click on a tile no legal road reaches is refused and the snapshot
	# is untouched -- the bridge is never called at all.
	click_tile(board, UNREACHABLE_TILE, MOUSE_BUTTON_RIGHT)
	check(JSON.stringify(by_tile.get_authoritative_snapshot()) == standing, "a right-click on an unreachable tile must leave the snapshot exactly as it was")
	var refused_order: Dictionary = by_tile.order_move_to_tile(UNREACHABLE_TILE)
	check(not bool(refused_order.get("configured", true)), "an unreachable tile must be refused by name")
	click_tile(board, BEACH_DESTINATION, MOUSE_BUTTON_RIGHT)
	check(by_tile.get_authoritative_snapshot().get("active_location_id") == BEACH_DESTINATION, "a right-click on a reachable tile must land where the words and the hotkey land")
	check(by_tile.selected_cell_id == BEACH_DESTINATION, "arriving must put the selection back on the party")
	by_tile.queue_free()
	await process_frame
	session_owner().reset_for_test()
	finish()


## A campaign that has just begun, on a screen that has just been built. The
## session is dropped first so every RTS run starts from the same day one.
func fresh_expedition(scene: PackedScene) -> ExpeditionPrototype:
	session_owner().reset_for_test()
	var expedition := scene.instantiate() as ExpeditionPrototype
	root.add_child(expedition)
	await process_frame
	return expedition


func session_owner() -> Node:
	return root.get_node("CampaignSession")


## Where a tile is drawn, in the board's own coordinates. Read from the board's
## island rectangle rather than recomputed here, so the suite clicks exactly
## where the player sees the tile whatever size the board was laid out at.
func tile_point(board: ExpeditionRouteBoard, cell_id: String) -> Vector2:
	var island: Rect2 = board.island_rect()
	return island.position + island.size * (ExpeditionRouteBoard.NODE_POSITIONS[cell_id] as Vector2)


## A real mouse button on the board, through the board's own input handler.
func click_tile(board: ExpeditionRouteBoard, cell_id: String, button_index: int) -> void:
	var event := InputEventMouseButton.new()
	event.position = tile_point(board, cell_id)
	event.button_index = button_index
	event.pressed = true
	board._gui_input(event)


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
