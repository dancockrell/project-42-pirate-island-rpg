class_name TitleScreen
extends Control

## The front door.
##
## Six controls and nothing else on the glass: New Game, Continue, Load,
## Settings, Credits, Quit. Continue is live only when a save exists and always
## means the newest campaign by the save's own clock (card B8), never the newest
## file. Load lists every slot, including the ones that are broken, because a
## save that has gone bad is news the player is owed rather than a row that
## quietly disappears.
##
## This screen holds no campaign. `CampaignSession` is the one holder; the title
## asks it to start, to continue, or to load a slot, and then asks `SceneFlow`
## to move the glass. Nothing here decides a game result.

## The session's own script, preloaded for its static slot comparison: an
## autoload's name resolves to the node, and a static function is reached
## through the script rather than through the instance.
const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")

## The flow's script, for its one static reader. The autoload itself is reached
## as a node, the way every other autoload in this project is: an autoload name
## is not a compile-time identifier in a `--script` run, and the suites are
## `--script` runs.
const SceneFlowScript = preload("res://scripts/shell/scene_flow.gd")

const NEW_GAME := "new_game"
const CONTINUE := "continue"
const LOAD := "load"
const SETTINGS := "settings"
const CREDITS := "credits"
const QUIT := "quit"

const MENU_ROWS := [
	[NEW_GAME, "NEW GAME", "Make landfall on the island for the first time."],
	[CONTINUE, "CONTINUE", "Resume the most recent campaign."],
	[LOAD, "LOAD", "Choose a campaign from the recorded slots."],
	[SETTINGS, "SETTINGS", "Text size, contrast and motion."],
	[CREDITS, "CREDITS", "Attribution for everything admitted to this build."],
	[QUIT, "QUIT", "Leave the island where it stands."]
]

var catalog := ContentCatalog.new()
var campaign_session: Node
var scene_flow: Node
var atmosphere: TitleAtmosphere
var menu_buttons: Dictionary = {}
var continue_detail: Label
var status_label: Label
var slot_panel: Control
var credits_panel: Control


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	campaign_session = get_node_or_null("/root/CampaignSession")
	scene_flow = get_node_or_null("/root/SceneFlow")
	if catalog.load_default() != OK:
		# Loud, but not an engine error: the shell still draws so the player can
		# read why and quit, which a hard failure at this point would deny them.
		build_screen()
		report("THE VALIDATED CONTENT BUNDLE IS UNAVAILABLE  •  REBUILD CONTENT BEFORE SAILING")
		set_menu_enabled(NEW_GAME, false)
		set_menu_enabled(CONTINUE, false)
		set_menu_enabled(LOAD, false)
		return
	build_screen()
	refresh_saves()
	# The first control takes focus on open, so a keyboard or a gamepad can move
	# through the menu without a mouse having touched it first.
	restore_focus()


# ---------------------------------------------------------------------------
# The screen.
# ---------------------------------------------------------------------------


func build_screen() -> void:
	atmosphere = TitleAtmosphere.new()
	atmosphere.name = "Atmosphere"
	atmosphere.motion_enabled = not SceneFlowScript.reduced_motion()
	add_child(atmosphere)

	var frame := MarginContainer.new()
	frame.name = "Frame"
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 132)
	frame.add_theme_constant_override("margin_right", 132)
	frame.add_theme_constant_override("margin_top", 96)
	frame.add_theme_constant_override("margin_bottom", 72)
	frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(frame)

	# One column, hard left, top-weighted: the title, a rule, the menu. The
	# right two thirds are left to the weather on purpose -- the composition is
	# the island, and the type is what stands in front of it.
	var column := VBoxContainer.new()
	column.name = "Column"
	column.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	column.custom_minimum_size.x = 620
	column.add_theme_constant_override("separation", 0)
	column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	frame.add_child(column)

	var eyebrow := ShellStyle.label(ShellStyle.tracked("PROJECT 42"), 18, ShellStyle.BRONZE)
	eyebrow.name = "Eyebrow"
	column.add_child(eyebrow)
	column.add_child(spacer(18))

	var wordmark := ShellStyle.label(ShellStyle.tracked("PIRATE"), 86, ShellStyle.CREAM)
	wordmark.name = "Wordmark"
	column.add_child(wordmark)
	var wordmark_second := ShellStyle.label(ShellStyle.tracked("ISLAND"), 86, ShellStyle.CREAM)
	wordmark_second.name = "WordmarkSecond"
	column.add_child(wordmark_second)

	column.add_child(spacer(26))
	column.add_child(ShellStyle.rule(ShellStyle.BRONZE, 470.0, 2.0))
	column.add_child(spacer(18))

	# The brief's own sentence for what this is, quoted rather than written for
	# the box: docs/PIRATE_ISLAND_CONTINUATION_BRIEF.md section 1.
	var promise := ShellStyle.label("A character-scale RPG taking place inside a\nliving autonomous RTS simulation.", 19, ShellStyle.MUTED)
	promise.name = "Promise"
	column.add_child(promise)

	column.add_child(spacer(46))

	var menu := VBoxContainer.new()
	menu.name = "Menu"
	menu.add_theme_constant_override("separation", 2)
	column.add_child(menu)
	continue_detail = ShellStyle.label("", 14, ShellStyle.TEAL)
	continue_detail.name = "ContinueDetail"
	for row in MENU_ROWS:
		menu.add_child(make_menu_button(str(row[0]), str(row[1]), str(row[2])))
		# The line that says which campaign Continue would resume belongs under
		# Continue, not at the foot of the menu.
		if str(row[0]) == CONTINUE:
			menu.add_child(continue_detail)
	wire_focus_chain()

	status_label = ShellStyle.label("", 15, ShellStyle.DANGER)
	status_label.name = "Status"
	status_label.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_LEFT)
	status_label.offset_left = 132.0
	status_label.offset_top = -64.0
	status_label.offset_right = 1400.0
	status_label.offset_bottom = -40.0
	add_child(status_label)

	var footer := ShellStyle.label("ARROWS OR STICK TO MOVE  •  ENTER OR A TO CHOOSE", 13, ShellStyle.HAIRLINE.lightened(0.35))
	footer.name = "Footer"
	footer.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	footer.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	footer.offset_left = -520.0
	footer.offset_top = -64.0
	footer.offset_right = -132.0
	footer.offset_bottom = -40.0
	add_child(footer)


func make_menu_button(action: String, text: String, hint: String) -> Button:
	var button := Button.new()
	button.name = action.to_pascal_case()
	button.text = text
	button.tooltip_text = hint
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.custom_minimum_size = Vector2(470, 52)
	button.focus_mode = Control.FOCUS_ALL
	button.add_theme_font_size_override("font_size", 22)
	button.add_theme_color_override("font_color", ShellStyle.CREAM)
	button.add_theme_color_override("font_hover_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_focus_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_pressed_color", ShellStyle.TEAL)
	button.add_theme_color_override("font_disabled_color", ShellStyle.HAIRLINE.lightened(0.3))
	button.add_theme_stylebox_override("normal", ShellStyle.menu_box(ShellStyle.BRONZE, Color(0, 0, 0, 0)))
	button.add_theme_stylebox_override("hover", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.09)))
	button.add_theme_stylebox_override("focus", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.13)))
	button.add_theme_stylebox_override("pressed", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.18)))
	button.add_theme_stylebox_override("disabled", ShellStyle.menu_box(ShellStyle.HAIRLINE, Color(0, 0, 0, 0)))
	button.pressed.connect(choose.bind(action))
	menu_buttons[action] = button
	return button


## Keyboard and gamepad walk the column and stop at both ends rather than
## wrapping: a menu that wraps makes the player overshoot the row they wanted.
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


# ---------------------------------------------------------------------------
# The saves this title knows about.
# ---------------------------------------------------------------------------


## Re-reads the slot list and states plainly whether there is a campaign to come
## back to. Called on open and after anything that could have changed the disk.
func refresh_saves() -> void:
	var slots := readable_slots()
	set_menu_enabled(CONTINUE, not slots.is_empty())
	set_menu_enabled(LOAD, not all_slots().is_empty())
	if continue_detail == null:
		return
	if slots.is_empty():
		continue_detail.text = "        No campaign recorded yet."
		continue_detail.add_theme_color_override("font_color", ShellStyle.MUTED)
		return
	var newest: Dictionary = slots[0]
	for entry in slots:
		if CampaignSessionScript.slot_is_newer(entry, newest):
			newest = entry
	continue_detail.text = "        %s" % slot_line(newest)
	continue_detail.add_theme_color_override("font_color", ShellStyle.TEAL)


func all_slots() -> Array[Dictionary]:
	if campaign_session == null:
		return []
	return campaign_session.list_slots()


func readable_slots() -> Array[Dictionary]:
	var readable: Array[Dictionary] = []
	for entry in all_slots():
		if bool(entry.get("readable", false)):
			readable.append(entry)
	return readable


## One slot as a line a player can tell from the next one: the day, the hour on
## the strategic clock, and where the party is standing. The location crosses
## the session as a stable cell ID and becomes a name here, through the same
## content catalog every other screen reads.
func slot_line(entry: Dictionary) -> String:
	if not bool(entry.get("readable", false)):
		return "%s  •  UNREADABLE  •  %s" % [str(entry.get("slot", "")).to_upper(), str(entry.get("error", "unknown")).to_upper()]
	var cell := catalog.get_record(str(entry.get("active_location_id", "")))
	var place := str(cell.get("displayName", "")).to_upper()
	if place.is_empty():
		place = "UNKNOWN LOCATION"
	return "%s  •  DAY %d  •  %02d:00  •  %s" % [
		str(entry.get("slot", "")).to_upper(),
		int(entry.get("campaign_day", 0)),
		int(entry.get("hour_of_day", 0)),
		place
	]


func set_menu_enabled(action: String, enabled: bool) -> void:
	var button := menu_buttons.get(action) as Button
	if button == null:
		return
	button.disabled = not enabled


# ---------------------------------------------------------------------------
# The six choices.
# ---------------------------------------------------------------------------


func choose(action: String) -> void:
	match action:
		NEW_GAME:
			start_new_game()
		CONTINUE:
			continue_newest()
		LOAD:
			open_slot_panel()
		SETTINGS:
			open_settings()
		CREDITS:
			open_credits()
		QUIT:
			get_tree().quit()


## A new game drops whatever the session holds and configures the island again.
## `discard_campaign` is the session's own name for "this session now holds
## nothing", so there is no second idea of an empty session here.
func start_new_game() -> void:
	if campaign_session == null:
		report("CAMPAIGN SESSION UNAVAILABLE  •  THE TITLE REFUSES TO CREATE A SCENE-LOCAL CAMPAIGN")
		return
	campaign_session.discard_campaign()
	var started: Dictionary = campaign_session.begin_if_needed(catalog)
	if not bool(started.get("configured", false)):
		report("THE EXPEDITION COULD NOT START  •  %s" % str(started.get("error", "unknown_error")).to_upper())
		return
	scene_flow.enter_expedition()


func continue_newest() -> void:
	load_campaign(func() -> Dictionary: return campaign_session.continue_newest())


func continue_slot(slot: String) -> void:
	load_campaign(func() -> Dictionary: return campaign_session.continue_slot(slot))


## The one load path on this screen. Both Continue and a chosen row hand a
## bridge call in and get the same treatment: a refusal is reported with the
## reason the bridge gave and leaves the title exactly where it stood.
func load_campaign(call_bridge: Callable) -> void:
	if campaign_session == null:
		report("CAMPAIGN SESSION UNAVAILABLE")
		return
	var ready_state: Dictionary = campaign_session.begin_if_needed(catalog)
	if not bool(ready_state.get("configured", false)):
		report("THE BRIDGE IS UNAVAILABLE  •  %s" % str(ready_state.get("error", "unknown_error")).to_upper())
		return
	var loaded: Dictionary = call_bridge.call()
	if not bool(loaded.get("configured", false)):
		report("THAT CAMPAIGN COULD NOT BE OPENED  •  %s" % str(loaded.get("error", "unknown_error")).to_upper())
		refresh_saves()
		return
	scene_flow.enter_expedition()


func open_settings() -> void:
	var packed := load("res://scenes/ui/settings_panel.tscn") as PackedScene
	if packed == null:
		return
	var panel := packed.instantiate()
	panel.tree_exited.connect(on_settings_closed)
	add_child(panel)


## Motion is the one setting this screen shows immediately, because it is the
## one the player changed while looking straight at it.
func on_settings_closed() -> void:
	if atmosphere != null:
		atmosphere.set_motion_enabled(not SceneFlowScript.reduced_motion())
	restore_focus()


func open_credits() -> void:
	if is_instance_valid(credits_panel):
		return
	credits_panel = CreditsScreen.new()
	credits_panel.closed.connect(restore_focus)
	add_child(credits_panel)


func open_slot_panel() -> void:
	if is_instance_valid(slot_panel):
		return
	var rows: Array[Dictionary] = all_slots()
	var lines: Array[String] = []
	var slots: Array[String] = []
	for entry in rows:
		lines.append(slot_line(entry))
		slots.append(str(entry.get("slot", "")) if bool(entry.get("readable", false)) else "")
	slot_panel = ShellChooser.new()
	slot_panel.configure("LOAD A CAMPAIGN", "A slot that cannot be read is listed and marked, never hidden.", lines, slots)
	slot_panel.chosen.connect(continue_slot)
	slot_panel.closed.connect(restore_focus)
	add_child(slot_panel)


func restore_focus() -> void:
	var button := menu_buttons.get(NEW_GAME) as Button
	if button != null and button.is_inside_tree():
		button.grab_focus()


func report(message: String) -> void:
	if status_label != null:
		status_label.text = message
