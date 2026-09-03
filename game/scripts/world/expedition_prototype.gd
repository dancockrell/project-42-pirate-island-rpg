class_name ExpeditionPrototype
extends Control

## First chapter travel front end. Rust owns campaign state and legal portal
## commands. This screen resolves cell descriptions and route labels from the
## content catalog, then projects the native snapshot without adding rules.

const RouteBoardScript = preload("res://scripts/world/expedition_route_board.gd")

const DEEP := Color("081211")
const PANEL := Color("132321")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("9eb0a7")
const DANGER := Color("c24e45")
const SEED := 42

var catalog := ContentCatalog.new()
var campaign_session: Node
var latest_snapshot: Dictionary = {}
var title_label: Label
var day_label: Label
var location_label: Label
var description_label: RichTextLabel
var route_list: VBoxContainer
var status_label: Label
var route_board: ExpeditionRouteBoard


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	if catalog.load_default() != OK:
		show_startup_failure("The validated content bundle is unavailable. Rebuild content before running the expedition.")
		return
	campaign_session = get_node_or_null("/root/CampaignSession")
	if campaign_session == null:
		show_startup_failure("Campaign session is unavailable. This screen refuses to create a scene-local campaign state.")
		return
	latest_snapshot = campaign_session.begin_if_needed(catalog)
	if not bool(latest_snapshot.get("configured", false)):
		show_startup_failure("The expedition could not start: %s." % str(latest_snapshot.get("error", "unknown_error")))
		return
	build_screen()
	project_snapshot(latest_snapshot, "Michael and Betty reach the black shore below the wreck of the Handsome Jack.")


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
	status_label = make_label("", 14, MUTED)
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stack.add_child(status_label)
	return panel


func build_footer() -> Control:
	var footer := HBoxContainer.new()
	footer.custom_minimum_size.y = 52
	var party := make_label("PARTY  •  MICHAEL CORRIGAN  •  BETTY", 14, CREAM)
	party.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	footer.add_child(party)
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
	description_label.text = make_description(cell)
	route_board.configure(catalog, snapshot)
	populate_routes(cell, snapshot)
	status_label.text = message


func make_description(cell: Dictionary) -> String:
	var descriptions: Array = cell.get("readableDescriptions", [])
	if descriptions.is_empty():
		return "[color=#c24e45]AUTHORED OBSERVATION MISSING[/color]"
	var primary: Dictionary = descriptions[0]
	var secondary: Dictionary = descriptions[1] if descriptions.size() > 1 else {}
	var text := "[color=#b78a4b]OBSERVATION[/color]\n%s" % str(primary.get("text", ""))
	if not secondary.is_empty():
		text += "\n\n[color=#9eb0a7]%s[/color]" % str(secondary.get("text", ""))
	return text


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
	var visible_count := 0
	for portal in cell.get("portals", []):
		if not portal is Dictionary:
			continue
		var portal_id := str(portal.get("id", ""))
		if not legal.has(portal_id):
			continue
		visible_count += 1
		var target := catalog.get_record(str(portal.get("targetCellId", "")))
		var travel_mode := str(portal.get("travelMode", "on_foot")).replace("_", " ").to_upper()
		var button := Button.new()
		button.name = "Route_" + portal_id.replace(".", "_")
		button.custom_minimum_size.y = 64
		button.text = "%s\n%s  •  ARRIVAL SAVE" % [str(target.get("displayName", "UNKNOWN DESTINATION")).to_upper(), travel_mode]
		button.tooltip_text = "Authoritative portal: %s\nFrom: %s\nTo: %s\nTravel mode: %s" % [portal_id, str(cell.get("id", "")), str(target.get("id", "")), travel_mode]
		button.add_theme_font_size_override("font_size", 14)
		button.add_theme_stylebox_override("normal", make_route_box(Color("1a322e"), BRONZE))
		button.add_theme_stylebox_override("hover", make_route_box(Color("22443d"), TEAL))
		button.pressed.connect(func() -> void: request_travel(portal_id))
		route_list.add_child(button)
	if visible_count == 0:
		route_list.add_child(make_label("No legal departure. A pending encounter or unfinished state is blocking travel.", 14, DANGER))


func request_travel(portal_id: String) -> void:
	if campaign_session == null:
		return
	var result: Dictionary = campaign_session.travel(portal_id)
	if not bool(result.get("configured", false)):
		status_label.text = "TRAVEL REFUSED  •  %s" % str(result.get("error", "unknown_error")).to_upper()
		return
	var destination := catalog.get_record(str(result.get("active_location_id", "")))
	var pending_encounter: Dictionary = result.get("pending_encounter", {})
	var message := "ARRIVED  •  %s" % str(destination.get("displayName", "UNKNOWN LOCATION")).to_upper()
	if not pending_encounter.is_empty():
		message = "CONTACT  •  %s" % str(pending_encounter.get("encounter_id", "UNKNOWN ENCOUNTER")).replace("encounter.", "").replace("_", " ").to_upper()
	project_snapshot(result, message)


func enter_pending_battle() -> void:
	if campaign_session == null or not campaign_session.has_pending_encounter():
		status_label.text = "ENCOUNTER HANDOFF UNAVAILABLE"
		return
	get_tree().change_scene_to_file("res://scenes/battle/battle_prototype.tscn")


func get_authoritative_snapshot() -> Dictionary:
	return latest_snapshot.duplicate(true)


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
