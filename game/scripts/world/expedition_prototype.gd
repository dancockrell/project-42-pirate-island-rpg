class_name ExpeditionPrototype
extends Control

## First chapter travel front end. Rust owns campaign state and legal portal
## commands. This screen resolves cell descriptions and route labels from the
## content catalog, then projects the native snapshot without adding rules.
##
## P7 (B18 resumed) makes the controls RTS controls. There is one unit -- the
## party -- and three ways to give it the same order:
##   * right-click a tile on the board,
##   * press the digit shown on a departure in the list,
##   * click the departure itself, the words.
## All three end in `order_move_along`, which ends in `request_travel`, which is
## the only call to the bridge. Left-click never travels: on the party's own
## tile (or its card in the footer) it selects the party, and anywhere else it
## is a look -- the place's name and what the road there would cost, read out of
## the snapshot's `route_options`. Legality is never decided here; an order for
## a tile the native `legal_route_commands` does not reach is refused in the
## status line and no call is made. While `GamePause` holds the game, every one
## of these is ignored.

const RouteBoardScript = preload("res://scripts/world/expedition_route_board.gd")

const DEEP := Color("081211")
const PANEL := Color("132321")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("9eb0a7")
const DANGER := Color("c24e45")
const SEED := 42

## Departures are numbered from one in the order the list draws them, and only
## the first nine can carry a digit. A tenth road would still be clickable and
## still be right-clickable on its tile; it simply has no key.
const MAXIMUM_ROUTE_HOTKEYS := 9

var catalog := ContentCatalog.new()
var campaign_session: Node
var latest_snapshot: Dictionary = {}
var title_label: Label
var day_label: Label
var location_label: Label
var description_label: RichTextLabel
var route_list: VBoxContainer
var action_list: VBoxContainer
var status_label: Label
var party_card: Button
var route_board: ExpeditionRouteBoard
var game_pause: Node


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	if catalog.load_default() != OK:
		show_startup_failure("The validated content bundle is unavailable. Rebuild content before running the expedition.")
		return
	game_pause = get_node_or_null("/root/GamePause")
	campaign_session = get_node_or_null("/root/CampaignSession")
	if campaign_session == null:
		show_startup_failure("Campaign session is unavailable. This screen refuses to create a scene-local campaign state.")
		return
	latest_snapshot = campaign_session.begin_if_needed(catalog)
	if not bool(latest_snapshot.get("configured", false)):
		show_startup_failure("The expedition could not start: %s." % str(latest_snapshot.get("error", "unknown_error")))
		return
	build_screen()
	project_snapshot(latest_snapshot, initial_status_message(latest_snapshot))


func build_screen() -> void:
	add_child(make_rect(DEEP))
	var frame := MarginContainer.new()
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 40)
	frame.add_theme_constant_override("margin_right", 40)
	frame.add_theme_constant_override("margin_top", 30)
	frame.add_theme_constant_override("margin_bottom", 28)
	add_child(frame)
	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 16)
	frame.add_child(page)
	page.add_child(build_header())
	var content_row := HBoxContainer.new()
	content_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	content_row.add_theme_constant_override("separation", 22)
	page.add_child(content_row)
	route_board = RouteBoardScript.new()
	route_board.name = "ExpeditionRouteBoard"
	route_board.custom_minimum_size = Vector2(1060, 700)
	route_board.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	route_board.tile_selected.connect(select_tile)
	route_board.move_ordered.connect(order_move_to_tile)
	content_row.add_child(route_board)
	content_row.add_child(build_location_panel())
	page.add_child(build_footer())


func build_header() -> Control:
	var header := HBoxContainer.new()
	header.custom_minimum_size.y = 62
	title_label = make_label("MICHAEL CORRIGAN  /  BLACK BEACH EXPEDITION", 23, BRONZE)
	title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title_label)
	day_label = make_label("DAY 1  •  DAWN", 16, CREAM)
	day_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	day_label.custom_minimum_size.x = 230
	header.add_child(day_label)
	return header


func build_location_panel() -> Control:
	var panel := PanelContainer.new()
	panel.name = "LocationPanel"
	panel.custom_minimum_size = Vector2(560, 700)
	panel.add_theme_stylebox_override("panel", make_panel_box())
	var padding := MarginContainer.new()
	padding.add_theme_constant_override("margin_left", 22)
	padding.add_theme_constant_override("margin_right", 22)
	padding.add_theme_constant_override("margin_top", 20)
	padding.add_theme_constant_override("margin_bottom", 20)
	panel.add_child(padding)
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 14)
	padding.add_child(stack)
	location_label = make_label("", 28, CREAM)
	stack.add_child(location_label)
	description_label = RichTextLabel.new()
	description_label.name = "LocationDescription"
	description_label.bbcode_enabled = true
	description_label.fit_content = false
	description_label.custom_minimum_size.y = 205
	description_label.add_theme_font_size_override("normal_font_size", 17)
	description_label.add_theme_color_override("default_color", CREAM)
	description_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stack.add_child(description_label)
	var route_heading := make_label("LEGAL DEPARTURES", 14, BRONZE)
	stack.add_child(route_heading)
	route_list = VBoxContainer.new()
	route_list.name = "LegalRouteList"
	route_list.add_theme_constant_override("separation", 9)
	route_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	stack.add_child(route_list)
	var action_heading := make_label("LEGAL ACTIONS HERE", 14, BRONZE)
	stack.add_child(action_heading)
	action_list = VBoxContainer.new()
	action_list.name = "LegalActionList"
	action_list.add_theme_constant_override("separation", 9)
	stack.add_child(action_list)
	status_label = make_label("", 14, MUTED)
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stack.add_child(status_label)
	return panel


func build_footer() -> Control:
	var footer := HBoxContainer.new()
	footer.custom_minimum_size.y = 52
	# The party's card in the interface. Clicking it selects the party, exactly
	# as clicking its tile does -- the owner's "or by clicking on the words in
	# the interface", applied to the unit as well as to the roads.
	party_card = Button.new()
	party_card.name = "PartyCard"
	party_card.text = "PARTY  •  MICHAEL CORRIGAN  •  BETTY"
	party_card.tooltip_text = "Selects the party. Right-click a tile, click a departure, or press its number to march."
	party_card.flat = true
	party_card.alignment = HORIZONTAL_ALIGNMENT_LEFT
	party_card.add_theme_font_size_override("font_size", 14)
	party_card.add_theme_color_override("font_color", CREAM)
	party_card.add_theme_color_override("font_hover_color", TEAL)
	party_card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	party_card.pressed.connect(func() -> void: select_tile(str(latest_snapshot.get("active_location_id", ""))))
	footer.add_child(party_card)
	var authority := make_label("RUST EXPEDITION STATE  •  ARRIVAL SAVE BOUNDARY", 13, TEAL)
	authority.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	footer.add_child(authority)
	return footer


func project_snapshot(snapshot: Dictionary, message: String) -> void:
	latest_snapshot = snapshot.duplicate(true)
	var cell := catalog.get_record(str(snapshot.get("active_location_id", "")))
	var day := int(snapshot.get("campaign_day", 1))
	var segment := str(snapshot.get("time_segment", "dawn")).to_upper()
	day_label.text = "DAY %d  •  %s" % [day, segment]
	location_label.text = str(cell.get("displayName", "UNKNOWN LOCATION")).to_upper()
	description_label.text = make_description(cell, snapshot)
	route_board.configure(catalog, snapshot)
	populate_routes(cell, snapshot)
	populate_actions(cell, snapshot)
	status_label.text = message


func make_description(cell: Dictionary, snapshot: Dictionary) -> String:
	var descriptions: Array = cell.get("readableDescriptions", [])
	if descriptions.is_empty():
		return "[color=#c24e45]AUTHORED OBSERVATION MISSING[/color]"
	var primary: Dictionary = descriptions[0]
	var secondary: Dictionary = descriptions[1] if descriptions.size() > 1 else {}
	var text := "[color=#b78a4b]OBSERVATION[/color]\n%s" % str(primary.get("text", ""))
	if not secondary.is_empty():
		text += "\n\n[color=#9eb0a7]%s[/color]" % str(secondary.get("text", ""))
	for entry in cell.get("battleEntries", []):
		if not entry is Dictionary:
			continue
		var encounter_id := str(entry.get("encounterId", ""))
		if resolved_encounter_ids(snapshot).has(encounter_id):
			var aftermath := str(entry.get("aftermathDescription", ""))
			if not aftermath.is_empty():
				text += "\n\n[color=#55c9ac]AFTERMATH[/color]\n%s" % aftermath
	for response in cell.get("estateConsequenceResponses", []):
		if not response is Dictionary:
			continue
		var estate_upgrade_id := str(response.get("requiresEstateUpgradeId", ""))
		if estate_upgrades(snapshot).has(estate_upgrade_id):
			var response_text := str(response.get("text", ""))
			if not response_text.is_empty():
				text += "\n\n[color=#55c9ac]HOUSEHOLD RESULT[/color]\n%s" % response_text
	return text


func resolved_encounter_ids(snapshot: Dictionary) -> Dictionary:
	var ids: Dictionary = {}
	for encounter_id in snapshot.get("resolved_encounter_ids", []):
		ids[str(encounter_id)] = true
	return ids


func estate_upgrades(snapshot: Dictionary) -> Dictionary:
	var ids: Dictionary = {}
	for estate_upgrade_id in snapshot.get("estate_upgrades", []):
		ids[str(estate_upgrade_id)] = true
	return ids


## What the screen says when it opens on a place. P7: the status line is the
## party's voice, not the transaction log -- it says what happened on the island
## rather than which native call returned. The one shouted phrase left is the
## name of an authored household upgrade, which is a thing in the world with a
## name of its own.
func initial_status_message(snapshot: Dictionary) -> String:
	if resolved_encounter_ids(snapshot).has("encounter.prototype.returning_names") and str(snapshot.get("active_location_id", "")) == "world.cell.reception_terrace":
		return "The terrace is quiet. The razorbeaks are driven off and the roads out of it are open again."
	if estate_upgrades(snapshot).has("estate_upgrade.river_gate_alarm") and str(snapshot.get("active_location_id", "")) == "world.cell.damaged_estate":
		return "The household has been at work: the RIVER GATE ALARM is strung and the water is watched."
	if str(snapshot.get("active_location_id", "")) == "world.cell.black_beach":
		return "Michael and Betty reach the black shore below the wreck of the Handsome Jack."
	return "The expedition stands at %s. Select the party, then order a march." % place_name(str(snapshot.get("active_location_id", "")))


## An authored place's name for the status line, in the words the content gives
## it. A cell the catalog does not carry falls back to its own ID made readable,
## which is a missing record on screen rather than a blank sentence.
func place_name(cell_id: String) -> String:
	var record := catalog.get_record(cell_id)
	var display := str(record.get("displayName", ""))
	if not display.is_empty():
		return display
	return cell_id.replace("world.cell.", "").replace("_", " ").capitalize()


func populate_routes(cell: Dictionary, snapshot: Dictionary) -> void:
	for child in route_list.get_children():
		child.queue_free()
	var pending_encounter: Dictionary = snapshot.get("pending_encounter", {})
	if not pending_encounter.is_empty():
		var encounter_id := str(pending_encounter.get("encounter_id", ""))
		var battle_id := str(pending_encounter.get("battle_id", ""))
		var pending := make_label("ENCOUNTER PENDING\n%s\n%s" % [encounter_id.to_upper(), battle_id.to_upper()], 14, DANGER)
		pending.name = "PendingEncounter"
		pending.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		pending.custom_minimum_size.y = 78
		pending.tooltip_text = "Authoritative encounter handoff. Travel is blocked until the named battle resolves.\nEncounter: %s\nBattle: %s" % [encounter_id, battle_id]
		route_list.add_child(pending)
		var engage := Button.new()
		engage.name = "EngagePendingEncounter"
		engage.custom_minimum_size.y = 62
		engage.text = "ENGAGE  •  %s" % encounter_id.trim_prefix("encounter.").replace("_", " ").to_upper()
		engage.tooltip_text = "Enter the authored battle declared by this native campaign state.\nEncounter: %s\nBattle: %s" % [encounter_id, battle_id]
		engage.add_theme_font_size_override("font_size", 14)
		engage.add_theme_stylebox_override("normal", make_route_box(Color("41251f"), DANGER))
		engage.add_theme_stylebox_override("hover", make_route_box(Color("61332a"), BRONZE))
		engage.pressed.connect(enter_pending_battle)
		route_list.add_child(engage)
		return
	var legal: Dictionary = {}
	for command in snapshot.get("legal_route_commands", []):
		var command_text := str(command)
		if command_text.begins_with("travel:"):
			legal[command_text.trim_prefix("travel:")] = true
	# B14: what each road costs the party *right now*. The native snapshot
	# recomputes risk from control every time it is built, so the screen reads
	# it here and keeps nothing: a risk number remembered in GDScript would be
	# a second answer to how dangerous a road is, and it would be the stale one.
	var options := route_options(snapshot)
	var visible_count := 0
	for portal in cell.get("portals", []):
		if not portal is Dictionary:
			continue
		var portal_id := str(portal.get("id", ""))
		if not legal.has(portal_id):
			continue
		visible_count += 1
		var option: Dictionary = options.get(portal_id, {})
		var target := catalog.get_record(str(portal.get("targetCellId", "")))
		var travel_mode := str(portal.get("travelMode", "on_foot")).replace("_", " ").to_upper()
		var risk_level := int(option.get("risk_level", 0))
		var contested := bool(option.get("contested", false))
		var risk_text := "RISK %d" % risk_level
		if contested:
			risk_text += "  •  CONTESTED"
		var button := Button.new()
		button.name = "Route_" + portal_id.replace(".", "_")
		button.set_meta("portal_id", portal_id)
		button.custom_minimum_size.y = 64
		# The digit is drawn on the entry because a hotkey nobody can see is not
		# a control. It is the entry's position in this list and nothing else,
		# so `route_hotkey_portal_ids` reads it back off the drawn list rather
		# than keeping a second numbering beside it.
		var key_prefix := "[%d]  " % visible_count if visible_count <= MAXIMUM_ROUTE_HOTKEYS else ""
		button.text = "%s%s\n%s  •  %s  •  ARRIVAL SAVE" % [key_prefix, str(target.get("displayName", "UNKNOWN DESTINATION")).to_upper(), travel_mode, risk_text]
		var contested_note := "  (contested: the endpoints are held by different parties)" if contested else ""
		button.tooltip_text = "Authoritative portal: %s\nFrom: %s\nTo: %s\nTravel mode: %s\nRisk now: %d%s" % [portal_id, str(cell.get("id", "")), str(target.get("id", "")), travel_mode, risk_level, contested_note]
		button.add_theme_font_size_override("font_size", 14)
		button.add_theme_stylebox_override("normal", make_route_box(Color("1a322e"), BRONZE))
		button.add_theme_stylebox_override("hover", make_route_box(Color("22443d"), TEAL))
		button.pressed.connect(func() -> void: order_move_along(portal_id))
		route_list.add_child(button)
	if visible_count == 0:
		route_list.add_child(make_label("No road leaves this place right now. Something here has to be settled first.", 14, DANGER))


## B4: every "do something here" control, drawn from the native legal-command
## list and from nothing else. An anchor already spent today, or an observation
## that belongs to another cell, is simply absent from that list and so is
## never drawn -- the same invariant the route list keeps for a gated door. The
## catalog is consulted only for the authored label of a command Rust already
## called legal.
func populate_actions(cell: Dictionary, snapshot: Dictionary) -> void:
	for child in action_list.get_children():
		child.queue_free()
	var pending_encounter: Dictionary = snapshot.get("pending_encounter", {})
	if not pending_encounter.is_empty():
		return
	var anchor_kinds: Dictionary = {}
	for anchor in cell.get("anchors", []):
		if anchor is Dictionary:
			anchor_kinds[str(anchor.get("id", ""))] = str(anchor.get("kind", ""))
	var observation_texts: Dictionary = {}
	for description in cell.get("readableDescriptions", []):
		if description is Dictionary:
			observation_texts[str(description.get("id", ""))] = str(description.get("text", ""))
	for command in snapshot.get("legal_commands", []):
		var command_text := str(command)
		if command_text.begins_with("anchor_action:"):
			var anchor_id := command_text.trim_prefix("anchor_action:")
			var kind := str(anchor_kinds.get(anchor_id, "")).replace("_", " ").to_upper()
			var anchor_button := make_action_button("Anchor_" + anchor_id.replace(".", "_"), command_text, "%s\n%s" % [readable_id(anchor_id.trim_prefix("anchor.")), kind], "Authoritative anchor action: %s\nKind: %s" % [anchor_id, kind])
			anchor_button.pressed.connect(func() -> void: request_anchor(anchor_id))
			action_list.add_child(anchor_button)
		elif command_text.begins_with("inspect:"):
			var observation_id := command_text.trim_prefix("inspect:")
			var inspect_button := make_action_button("Inspect_" + observation_id.replace(".", "_"), command_text, "INSPECT  •  %s" % readable_id(observation_id.trim_prefix("observation.")), "Records this authored observation as a discovery.\nObservation: %s\n%s" % [observation_id, str(observation_texts.get(observation_id, ""))])
			inspect_button.pressed.connect(func() -> void: request_inspect(observation_id))
			action_list.add_child(inspect_button)
	# Midnight is not a per-place command, so it is not in the legal list: it is
	# offered whenever no encounter is pending, which is exactly when the native
	# transaction will accept it.
	var midnight_button := make_action_button("ResolveMidnight", "resolve_midnight", "HOLD FOR MIDNIGHT\nDAY %d ENDS" % int(snapshot.get("campaign_day", 1)), "Ends the day through the native midnight transaction: the island repopulates and every anchor spent today can be used again.")
	midnight_button.pressed.connect(request_midnight)
	action_list.add_child(midnight_button)


func make_action_button(node_name: String, command: String, text: String, tooltip: String) -> Button:
	var button := Button.new()
	button.name = node_name
	button.set_meta("command", command)
	button.custom_minimum_size.y = 54
	button.text = text
	button.tooltip_text = tooltip
	button.add_theme_font_size_override("font_size", 14)
	button.add_theme_stylebox_override("normal", make_route_box(Color("1d2a24"), BRONZE))
	button.add_theme_stylebox_override("hover", make_route_box(Color("2a3d33"), TEAL))
	return button


func readable_id(stable_id: String) -> String:
	return stable_id.replace(".", " ").replace("_", " ").strip_edges().to_upper()


## The one call to the bridge that moves the party. Every control on this screen
## -- the right-click on a tile, the digit, the words in the departure list --
## arrives here through `order_move_along` and nowhere else, so there is exactly
## one place where travel happens and exactly one place it can be refused.
func request_travel(portal_id: String) -> Dictionary:
	if campaign_session == null:
		return {"configured": false, "error": "campaign_session_unavailable"}
	var result: Dictionary = campaign_session.travel(portal_id)
	if not bool(result.get("configured", false)):
		status_label.text = "The party does not take that road: %s." % readable_refusal(str(result.get("error", "unknown_error")))
		return result
	var destination := catalog.get_record(str(result.get("active_location_id", "")))
	var pending_encounter: Dictionary = result.get("pending_encounter", {})
	var message := "The party comes up at %s." % str(destination.get("displayName", "an unnamed place"))
	if not pending_encounter.is_empty():
		message = "Contact at %s — %s. Nothing else moves until it is settled." % [str(destination.get("displayName", "an unnamed place")), readable_id(str(pending_encounter.get("encounter_id", "")).trim_prefix("encounter.")).capitalize()]
	project_snapshot(result, message)
	return result


## A machine-readable refusal, said out loud. The reason itself is the bridge's
## and is never rewritten here; this only turns its underscores into a sentence
## so the player reads a sentence.
func readable_refusal(reason: String) -> String:
	return reason.replace("_", " ")


## The one "do something here" verb, driven through the authoritative
## session. Returns the bridge's result so a caller (or a test) can read the
## `anchor_outcome` block. B3 draws the control; this is the wire.
func request_anchor(anchor_id: String) -> Dictionary:
	if campaign_session == null:
		return {"configured": false, "error": "campaign_session_unavailable"}
	var result: Dictionary = campaign_session.use_anchor(anchor_id)
	if not bool(result.get("configured", false)):
		status_label.text = "That cannot be done here: %s." % readable_refusal(str(result.get("error", "unknown_error")))
		return result
	var outcome: Dictionary = result.get("anchor_outcome", {})
	project_snapshot(result, "The party works the %s: %d rations, %d medicine, %d coin." % [readable_id(str(outcome.get("anchor_id", "")).trim_prefix("anchor.")).to_lower(), int(outcome.get("rations_gained", 0)), int(outcome.get("medicine_gained", 0)), int(outcome.get("coin_gained", 0))])
	return result


## Reading a place. The bridge refuses an observation that is not legal here,
## so the screen never has to decide what may be looked at.
func request_inspect(observation_id: String) -> Dictionary:
	if campaign_session == null:
		return {"configured": false, "error": "campaign_session_unavailable"}
	var result: Dictionary = campaign_session.inspect(observation_id)
	if not bool(result.get("configured", false)):
		status_label.text = "There is nothing of that here: %s." % readable_refusal(str(result.get("error", "unknown_error")))
		return result
	project_snapshot(result, "Noted: %s." % readable_id(observation_id.trim_prefix("observation.")).to_lower())
	return result


## Ending the day. Rust owns the whole transaction; the screen only reports how
## many events it produced.
func request_midnight() -> Dictionary:
	if campaign_session == null:
		return {"configured": false, "error": "campaign_session_unavailable"}
	var result: Dictionary = campaign_session.resolve_midnight()
	if not bool(result.get("configured", false)):
		status_label.text = "The day will not close yet: %s." % readable_refusal(str(result.get("error", "unknown_error")))
		return result
	var events: Array = result.get("events", [])
	project_snapshot(result, "The night passes. Day %d, and the island moved %d times in the dark." % [int(result.get("campaign_day", 1)), events.size()])
	return result


func enter_pending_battle() -> void:
	if campaign_session == null or not campaign_session.has_pending_encounter():
		status_label.text = "There is nothing waiting to be fought."
		return
	get_tree().change_scene_to_file("res://scenes/battle/battle_prototype.tscn")


# ---------------------------------------------------------------------------
# P7: the RTS controls.
#
# Three verbs and one order. `select_tile` is the left-click, `order_move_to_tile`
# is the right-click, `press_route_hotkey` is the digit -- and the mouse and key
# handlers call these very functions, so a suite that calls them is exercising
# the same code the player's hand does rather than a parallel test path. Every
# order funnels into `order_move_along`, and that is the only caller of
# `request_travel`. Nothing here decides legality: `portal_from_active_cell_to`
# is a lookup inside the native `legal_route_commands` list.
# ---------------------------------------------------------------------------

## Whether the game is held. Brief section 14 pauses local movement, so while a
## reason stands, none of these controls does anything at all: no order, no
## selection, no status line. `GamePause` is the one owner of the verdict and it
## is asked, never mirrored.
func controls_are_held() -> bool:
	return game_pause != null and game_pause.is_paused()


## Left-click. The party's own tile (or its card in the footer) selects the
## party; any other tile is a look at that place and never a move.
func select_tile(cell_id: String) -> void:
	if controls_are_held() or route_board == null or cell_id.is_empty():
		return
	if cell_id == str(latest_snapshot.get("active_location_id", "")):
		route_board.set_party_selected(true)
		route_board.set_inspected_cell("")
		status_label.text = "Michael and Betty stand ready at %s. Right-click a place to march, or press its number." % place_name(cell_id)
		return
	route_board.set_inspected_cell(cell_id)
	status_label.text = describe_tile(cell_id)


## What a look at a place says. The road's cost is the snapshot's own
## `route_options` entry, read at the moment of the look; the screen keeps no
## copy of a risk, so what the player is told and what the simulation would
## charge are the same number by construction.
func describe_tile(cell_id: String) -> String:
	var here := place_name(str(latest_snapshot.get("active_location_id", "")))
	var there := place_name(cell_id)
	var portal_id := route_board.portal_from_active_cell_to(cell_id)
	if portal_id.is_empty():
		return "%s. No road runs there from %s." % [there, here]
	var option: Dictionary = route_options(latest_snapshot).get(portal_id, {})
	var line := "%s. The road from %s runs at risk %d" % [there, here, int(option.get("risk_level", 0))]
	if bool(option.get("contested", false)):
		line += ", and its far end is held against us"
	return line + ". Right-click to march."


## Right-click on a tile: the move order. Refused, in the status line and with
## no call to the bridge, when the native command list does not carry a road
## from where the party stands to that place.
func order_move_to_tile(cell_id: String) -> Dictionary:
	if controls_are_held():
		return refused_order("game_paused")
	if route_board == null or cell_id.is_empty():
		return refused_order("no_such_tile")
	var portal_id := route_board.portal_from_active_cell_to(cell_id)
	if portal_id.is_empty():
		status_label.text = "No road runs from %s to %s." % [place_name(str(latest_snapshot.get("active_location_id", ""))), place_name(cell_id)]
		return refused_order("no_legal_route")
	return order_move_along(portal_id)


## The digit shown on a departure, one-based and in the order the list draws
## them. Out of range is refused the same way an unreachable tile is.
func press_route_hotkey(index: int) -> Dictionary:
	if controls_are_held():
		return refused_order("game_paused")
	var portal_ids := route_hotkey_portal_ids()
	if index < 1 or index > portal_ids.size():
		status_label.text = "There is no departure %d." % index
		return refused_order("no_such_departure")
	return order_move_along(portal_ids[index - 1])


## The one order. The words, the digit and the right-click all end here, and
## this is the only caller of `request_travel`.
func order_move_along(portal_id: String) -> Dictionary:
	if controls_are_held():
		return refused_order("game_paused")
	if portal_id.is_empty():
		return refused_order("no_legal_route")
	request_travel(portal_id)
	return {"ordered": true, "portal_id": portal_id, "reason": ""}


func refused_order(reason: String) -> Dictionary:
	return {"ordered": false, "portal_id": "", "reason": reason}


## The portals the departure list is drawing, in the order it drew them. The
## drawn controls are the numbering: a second list kept beside them would drift
## the moment a road opened or closed.
func route_hotkey_portal_ids() -> Array[String]:
	var portal_ids: Array[String] = []
	if route_list == null:
		return portal_ids
	for child in route_list.get_children():
		if child.is_queued_for_deletion():
			continue
		var portal_id := str(child.get_meta("portal_id", ""))
		if not portal_id.is_empty():
			portal_ids.append(portal_id)
		if portal_ids.size() >= MAXIMUM_ROUTE_HOTKEYS:
			break
	return portal_ids


func _unhandled_key_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or key.echo:
		return
	var index := route_hotkey_index(key.keycode)
	if index == 0:
		return
	get_viewport().set_input_as_handled()
	press_route_hotkey(index)


## Digits one to nine, from the number row or the keypad, as a one-based
## departure index. Anything else is 0, meaning "not a departure key".
func route_hotkey_index(keycode: Key) -> int:
	if keycode >= KEY_1 and keycode <= KEY_9:
		return int(keycode) - int(KEY_1) + 1
	if keycode >= KEY_KP_1 and keycode <= KEY_KP_9:
		return int(keycode) - int(KEY_KP_1) + 1
	return 0


func get_authoritative_snapshot() -> Dictionary:
	return latest_snapshot.duplicate(true)


## The commands the action list is currently drawing, in list order. A control
## and its command are the same fact, so a test can hold the drawn buttons
## against the native legal-command list directly.
func get_action_commands() -> Array[String]:
	var commands: Array[String] = []
	if action_list == null:
		return commands
	for child in action_list.get_children():
		if child.is_queued_for_deletion():
			continue
		commands.append(str(child.get_meta("command", "")))
	return commands


## The native route projection, keyed by portal. One entry per legal road,
## carrying the risk the party would run now and whether the road is contested.
## The authored base risk is deliberately absent from it: one road, one number.
func route_options(snapshot: Dictionary) -> Dictionary:
	var options: Dictionary = {}
	for option in snapshot.get("route_options", []):
		if option is Dictionary:
			options[str(option.get("portal_id", ""))] = option
	return options


## The route button currently drawn for one portal, or null. A button freed by
## the previous projection is still a child until the frame ends, so the
## deletion check is what keeps a stale control from answering for a live one --
## the same guard `get_legal_route_count` keeps.
func get_route_button(portal_id: String) -> Button:
	if route_list == null:
		return null
	for child in route_list.get_children():
		if child.is_queued_for_deletion():
			continue
		var button := child as Button
		if button != null and str(button.get_meta("portal_id", "")) == portal_id:
			return button
	return null


## The risk a drawn route button is currently showing the player, read back out
## of the button's own text, or -1 when no such button is drawn. A control and
## the number on it are the same fact, so a test can hold the screen to it.
func get_drawn_route_risk(portal_id: String) -> int:
	var button := get_route_button(portal_id)
	if button == null:
		return -1
	for part in button.text.replace("\n", "  •  ").split("  •  "):
		var field := str(part).strip_edges()
		if field.begins_with("RISK "):
			return int(field.trim_prefix("RISK "))
	return -1


## Whether a drawn route button is marked contested.
func get_drawn_route_is_contested(portal_id: String) -> bool:
	var button := get_route_button(portal_id)
	if button == null:
		return false
	return "CONTESTED" in button.text


func get_legal_route_count() -> int:
	if route_list == null:
		return 0
	var count := 0
	for child in route_list.get_children():
		if not child.is_queued_for_deletion():
			count += 1
	return count


func show_startup_failure(reason: String) -> void:
	add_child(make_rect(DEEP))
	var label := make_label("EXPEDITION STARTUP BLOCKED\n\n%s" % reason, 20, DANGER)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	label.offset_left = 180
	label.offset_right = -180
	add_child(label)


func make_rect(color: Color) -> ColorRect:
	var rect := ColorRect.new()
	rect.color = color
	rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return rect


func make_label(text: String, font_size: int, color: Color) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", font_size)
	label.add_theme_color_override("font_color", color)
	return label


func make_panel_box() -> StyleBoxFlat:
	return make_route_box(PANEL, BRONZE)


func make_route_box(background: Color, border: Color) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = background
	box.border_color = border
	box.set_border_width_all(2)
	box.set_corner_radius_all(8)
	box.content_margin_left = 12
	box.content_margin_right = 12
	box.content_margin_top = 8
	box.content_margin_bottom = 8
	return box
