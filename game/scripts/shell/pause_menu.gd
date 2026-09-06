class_name PauseMenu
extends Control

## Escape, on a screen with a game running behind it.
##
## The menu does not invent pausing: it takes a `GamePause` reason while it is
## open and releases it when it closes, which is the same contract the settings
## panel already has, so a settings panel opened from here holds a second reason
## and the island stays held until both are gone. While the reason stands, the
## one call in the tree that advances the strategic clock refuses (card B9), so
## the clock does not move behind this menu.
##
## `SceneFlow` opens and closes it. Nothing else instances it, so there is one
## pause menu and one place that decides when it may exist.

## The reason this surface gives GamePause. The surface owns the name; the pause
## service counts reasons and never learns what any of them means.
const PAUSE_REASON := "pause_menu"

const RESUME := "resume"
const SAVE := "save"
const SETTINGS := "settings"
const TITLE := "title"
const QUIT := "quit"

const MENU_ROWS := [
	[RESUME, "RESUME"],
	[SAVE, "SAVE"],
	[SETTINGS, "SETTINGS"],
	[TITLE, "RETURN TO TITLE"],
	[QUIT, "QUIT"]
]

## The slots the pause menu offers. A fixed, named set rather than free text:
## `CampaignSession` restricts slot names to letters, digits, underscore and
## hyphen so a name can never address a file outside the save directory, and a
## text field on a pause menu is a way to find that rule the hard way.
const OFFERED_SLOTS: Array[String] = ["slot_one", "slot_two", "slot_three"]

## How wide the menu's plate is, in the project's 1920-wide design space. Wide
## enough to hold the column and its confirmations, narrow enough that the board
## behind it is still there.
const PLATE_WIDTH := 800.0

## Cards P5 and B13 forbid a red countdown or a flashing alert; a confirmation
## is the one place the shell may raise its voice, and it does it in words.
const CONFIRMATIONS := {
	TITLE: "Return to the title? Anything since your last save is lost.",
	QUIT: "Quit to the desktop? Anything since your last save is lost."
}

var catalog := ContentCatalog.new()
var campaign_session: Node
var game_pause: Node
var scene_flow: Node
var menu_buttons: Dictionary = {}
var status_label: Label
var confirm_row: HBoxContainer
var pending_confirmation := ""


func _init() -> void:
	# A pause menu that stopped processing when it paused the game could never
	# be closed again.
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	campaign_session = get_node_or_null("/root/CampaignSession")
	scene_flow = get_node_or_null("/root/SceneFlow")
	catalog.load_default()
	build()
	game_pause = get_node_or_null("/root/GamePause")
	if game_pause != null:
		game_pause.pause(PAUSE_REASON)
	refresh()


func _exit_tree() -> void:
	release_pause()


func release_pause() -> void:
	if game_pause == null:
		return
	game_pause.resume(PAUSE_REASON)
	game_pause = null


# ---------------------------------------------------------------------------
# The screen. A heavy scrim, because the island behind this menu is held: the
# player should read the menu, not keep watching the board.
# ---------------------------------------------------------------------------


func build() -> void:
	var scrim := ColorRect.new()
	scrim.name = "Scrim"
	scrim.color = Color(ShellStyle.NIGHT, 0.90)
	scrim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(scrim)

	# The island stays faintly visible behind the scrim -- it is held, not gone --
	# so the menu itself stands on a plate rather than floating over a board the
	# player can still half-read.
	var plate := ColorRect.new()
	plate.name = "Plate"
	plate.color = Color(ShellStyle.NIGHT, 0.97)
	plate.set_anchors_and_offsets_preset(Control.PRESET_LEFT_WIDE)
	plate.offset_right = PLATE_WIDTH
	plate.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(plate)

	var plate_edge := ColorRect.new()
	plate_edge.name = "PlateEdge"
	plate_edge.color = Color(ShellStyle.BRONZE, 0.55)
	plate_edge.set_anchors_and_offsets_preset(Control.PRESET_LEFT_WIDE)
	plate_edge.offset_left = PLATE_WIDTH
	plate_edge.offset_right = PLATE_WIDTH + 2.0
	plate_edge.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(plate_edge)

	var frame := MarginContainer.new()
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 180)
	frame.add_theme_constant_override("margin_right", 180)
	frame.add_theme_constant_override("margin_top", 150)
	frame.add_theme_constant_override("margin_bottom", 120)
	add_child(frame)

	var column := VBoxContainer.new()
	column.name = "Column"
	column.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	column.custom_minimum_size.x = 560
	column.add_theme_constant_override("separation", 0)
	frame.add_child(column)

	column.add_child(ShellStyle.label(ShellStyle.tracked("HELD"), 20, ShellStyle.BRONZE))
	column.add_child(spacer(10))
	var heading := ShellStyle.label("THE ISLAND IS STOPPED", 46, ShellStyle.CREAM)
	heading.name = "Heading"
	column.add_child(heading)
	column.add_child(spacer(14))
	column.add_child(ShellStyle.rule(ShellStyle.BRONZE, 430.0, 2.0))
	column.add_child(spacer(12))
	status_label = ShellStyle.label("", 15, ShellStyle.TEAL)
	status_label.name = "Status"
	column.add_child(status_label)
	column.add_child(spacer(24))

	var menu := VBoxContainer.new()
	menu.name = "Menu"
	menu.add_theme_constant_override("separation", 2)
	column.add_child(menu)
	for row in MENU_ROWS:
		menu.add_child(make_menu_button(str(row[0]), str(row[1])))
	wire_focus_chain()

	confirm_row = HBoxContainer.new()
	confirm_row.name = "Confirm"
	confirm_row.add_theme_constant_override("separation", 12)
	confirm_row.visible = false
	column.add_child(spacer(18))
	column.add_child(confirm_row)

	# The foot of the plate says what pause actually does here, because "the
	# island is stopped" is a promise and the player is owed the terms of it.
	var foot := VBoxContainer.new()
	foot.name = "Foot"
	foot.add_theme_constant_override("separation", 6)
	foot.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_LEFT)
	foot.offset_left = 180.0
	foot.offset_top = -152.0
	foot.offset_right = PLATE_WIDTH - 60.0
	foot.offset_bottom = -64.0
	add_child(foot)
	foot.add_child(ShellStyle.rule(ShellStyle.HAIRLINE, 430.0))
	foot.add_child(spacer(8))
	foot.add_child(ShellStyle.label("The strategic clock does not turn while this menu is open.", 14, ShellStyle.MUTED))
	foot.add_child(ShellStyle.label("ESCAPE CLOSES THIS MENU", 13, ShellStyle.HAIRLINE.lightened(0.45)))

	menu_buttons[RESUME].grab_focus()


func make_menu_button(action: String, text: String) -> Button:
	var button := Button.new()
	button.name = action.to_pascal_case()
	button.text = text
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.custom_minimum_size = Vector2(430, 50)
	button.add_theme_font_size_override("font_size", 21)
	button.add_theme_color_override("font_color", ShellStyle.CREAM)
	button.add_theme_color_override("font_hover_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_focus_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_disabled_color", ShellStyle.HAIRLINE.lightened(0.3))
	button.add_theme_stylebox_override("normal", ShellStyle.menu_box(ShellStyle.BRONZE, Color(0, 0, 0, 0)))
	button.add_theme_stylebox_override("hover", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.09)))
	button.add_theme_stylebox_override("focus", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.13)))
	button.add_theme_stylebox_override("pressed", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.18)))
	button.add_theme_stylebox_override("disabled", ShellStyle.menu_box(ShellStyle.HAIRLINE, Color(0, 0, 0, 0)))
	button.pressed.connect(choose.bind(action))
	menu_buttons[action] = button
	return button


func wire_focus_chain() -> void:
	var order: Array[Button] = []
	for row in MENU_ROWS:
		order.append(menu_buttons[str(row[0])] as Button)
	for index in order.size():
		var button := order[index]
		var above := order[maxi(index - 1, 0)]
		var below := order[mini(index + 1, order.size() - 1)]
		button.focus_neighbor_top = button.get_path_to(above)
		button.focus_neighbor_bottom = button.get_path_to(below)
		button.focus_previous = button.get_path_to(above)
		button.focus_next = button.get_path_to(below)


func spacer(height: int) -> Control:
	var made := Control.new()
	made.custom_minimum_size = Vector2(0, height)
	made.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return made


## The line under the rule: where the party is standing and what day it is, so
## the player knows which campaign they are about to save or leave.
func refresh() -> void:
	if status_label == null:
		return
	if campaign_session == null:
		status_label.text = "NO CAMPAIGN SESSION"
		menu_buttons[SAVE].disabled = true
		return
	var snapshot: Dictionary = campaign_session.latest_snapshot
	if not bool(snapshot.get("configured", false)):
		status_label.text = "NO CAMPAIGN IS RUNNING"
		menu_buttons[SAVE].disabled = true
		return
	var cell := catalog.get_record(str(snapshot.get("active_location_id", "")))
	var place := str(cell.get("displayName", "")).to_upper()
	status_label.text = "DAY %d  •  %s  •  %s" % [
		int(snapshot.get("campaign_day", 0)),
		str(snapshot.get("time_segment", "")).to_upper(),
		place if not place.is_empty() else "UNKNOWN LOCATION"
	]


# ---------------------------------------------------------------------------
# The five choices.
# ---------------------------------------------------------------------------


func choose(action: String) -> void:
	clear_confirmation()
	match action:
		RESUME:
			close()
		SAVE:
			open_save_picker()
		SETTINGS:
			open_settings()
		TITLE:
			ask(TITLE)
		QUIT:
			ask(QUIT)


func open_settings() -> void:
	var packed := load("res://scenes/ui/settings_panel.tscn") as PackedScene
	if packed == null:
		return
	var panel := packed.instantiate()
	panel.tree_exited.connect(func() -> void: restore_focus(SETTINGS))
	add_child(panel)


## The slot picker is the shell's one list-and-pick surface, filled with the
## offered slots and what each currently holds.
func open_save_picker() -> void:
	if campaign_session == null:
		return
	var existing := {}
	for entry in campaign_session.list_slots():
		existing[str(entry.get("slot", ""))] = entry
	var lines: Array[String] = []
	var payloads: Array[String] = []
	for slot in OFFERED_SLOTS:
		payloads.append(slot)
		if existing.has(slot):
			lines.append("%s  •  OVERWRITE  •  %s" % [slot.to_upper(), describe(existing[slot])])
		else:
			lines.append("%s  •  EMPTY" % slot.to_upper())
	var chooser := ShellChooser.new()
	chooser.configure("SAVE THE CAMPAIGN", "Saving writes the running campaign exactly as the simulation holds it.", lines, payloads)
	chooser.chosen.connect(save_to_slot)
	chooser.closed.connect(func() -> void: restore_focus(SAVE))
	add_child(chooser)


func describe(entry: Dictionary) -> String:
	if not bool(entry.get("readable", false)):
		return "UNREADABLE  •  %s" % str(entry.get("error", "unknown")).to_upper()
	var cell := catalog.get_record(str(entry.get("active_location_id", "")))
	var place := str(cell.get("displayName", "")).to_upper()
	return "DAY %d  •  %02d:00  •  %s" % [
		int(entry.get("campaign_day", 0)),
		int(entry.get("hour_of_day", 0)),
		place if not place.is_empty() else "UNKNOWN LOCATION"
	]


func save_to_slot(slot: String) -> void:
	if campaign_session == null:
		return
	var saved: Dictionary = campaign_session.save_to_slot(slot)
	if bool(saved.get("configured", false)):
		status_label.text = "SAVED TO %s  •  DAY %d" % [slot.to_upper(), int(saved.get("campaign_day", 0))]
		status_label.add_theme_color_override("font_color", ShellStyle.TEAL)
		return
	status_label.text = "THE CAMPAIGN COULD NOT BE SAVED  •  %s" % str(saved.get("error", "unknown_error")).to_upper()
	status_label.add_theme_color_override("font_color", ShellStyle.DANGER)


# ---------------------------------------------------------------------------
# Confirmation. Two choices that throw work away ask first, in the same row,
# with the consequence written out rather than implied.
# ---------------------------------------------------------------------------


func ask(action: String) -> void:
	pending_confirmation = action
	for child in confirm_row.get_children():
		child.queue_free()
	var question := ShellStyle.label(str(CONFIRMATIONS[action]), 15, ShellStyle.CREAM)
	question.custom_minimum_size.x = 470
	question.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 8)
	stack.add_child(question)
	var buttons := HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 10)
	var yes := confirm_button("Confirm", ShellStyle.DANGER)
	yes.pressed.connect(commit)
	buttons.add_child(yes)
	var no := confirm_button("Cancel", ShellStyle.MUTED)
	no.pressed.connect(cancel_confirmation)
	buttons.add_child(no)
	stack.add_child(buttons)
	confirm_row.add_child(stack)
	confirm_row.visible = true
	yes.grab_focus()


func confirm_button(text: String, bar: Color) -> Button:
	var button := Button.new()
	button.name = text
	button.text = text.to_upper()
	button.custom_minimum_size = Vector2(210, 44)
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.add_theme_font_size_override("font_size", 17)
	button.add_theme_color_override("font_color", ShellStyle.CREAM)
	button.add_theme_color_override("font_focus_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_hover_color", ShellStyle.TEAL)
	button.add_theme_stylebox_override("normal", ShellStyle.menu_box(bar, Color(0, 0, 0, 0)))
	button.add_theme_stylebox_override("hover", ShellStyle.menu_box(bar, Color(bar, 0.12)))
	button.add_theme_stylebox_override("focus", ShellStyle.menu_box(bar, Color(bar, 0.16)))
	button.add_theme_stylebox_override("pressed", ShellStyle.menu_box(bar, Color(bar, 0.2)))
	return button


func cancel_confirmation() -> void:
	var action := pending_confirmation
	clear_confirmation()
	restore_focus(action)


func clear_confirmation() -> void:
	pending_confirmation = ""
	if confirm_row == null:
		return
	confirm_row.visible = false
	for child in confirm_row.get_children():
		child.queue_free()


func commit() -> void:
	var action := pending_confirmation
	clear_confirmation()
	match action:
		TITLE:
			# The campaign is dropped before the title opens: the title's own
			# Continue reads the disk, and a session still holding a campaign
			# would let a player walk back into an unsaved one they had just
			# agreed to leave.
			if campaign_session != null:
				campaign_session.discard_campaign()
			scene_flow.go_to_title()
		QUIT:
			get_tree().quit()


func restore_focus(action: String) -> void:
	var button := menu_buttons.get(action) as Button
	if button != null and button.is_inside_tree():
		button.grab_focus()


func _unhandled_input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return
	get_viewport().set_input_as_handled()
	if not pending_confirmation.is_empty():
		cancel_confirmation()
		return
	close()


## Closing releases the pause reason first, then the menu goes -- the same order
## the settings panel uses, so the island is running again before the surface
## that held it stops existing.
func close() -> void:
	release_pause()
	queue_free()
