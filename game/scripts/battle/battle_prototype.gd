class_name BattlePrototype
extends Control

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")
const PaperDollScript = preload("res://scripts/battle/paper_doll.gd")
const PaperStageScript = preload("res://scripts/battle/paper_stage.gd")
const PaperCardScript = preload("res://scripts/battle/paper_card.gd")
const PaperSkillDiamondScript = preload("res://scripts/battle/paper_skill_diamond.gd")
const CampaignEncounterSimulationPortScript = preload("res://scripts/simulation/campaign_encounter_simulation_port.gd")

## Presentation-only prototype. Gameplay truth comes through SimulationPort.
## All generated shapes and labels are explicit placeholders registered in
## content/art/placeholders.json.

const BRONZE := Color("b78a4b")
const DEEP := Color("101817")
const PANEL := Color("182321")
const CREAM := Color("eadfca")
const TEAL := Color("4fc7b4")
const DANGER := Color("c24e45")

const BETTY_ID := "character.heroine.betty"
const CAPTAIN_ID := "character.protagonist.captain"
## The five bands A5 named, in the order the board reads: the party's rear rank
## through the enemy's. The screen groups cards by the `band_name` string the
## bridge sends and uses this list only for the order they are shown in.
const BAND_ORDER := ["party_rear", "party_front", "contested", "enemy_front", "enemy_rear"]
## The actors the player commands. Every other actor's turn is played out
## automatically; these two stop the cycle and wait for a command.
const PLAYER_COMMANDED_ACTOR_IDS := [BETTY_ID, CAPTAIN_ID]
## Captain Michael's command grid: his two authored commands and the universal
## Guard verb. `skill.system.hold_position` has no record under `content/skills/`
## (A8 owns that gap), so its label is written here; the other two take the
## authored `displayName`.
const CAPTAIN_COMMANDS := [
	["skill.captain.weapon_attack", "WEAPON ATTACK"],
	["skill.system.hold_position", "GUARD"],
	["skill.captain.reposition", "REPOSITION"]
]
const CARD_SIZE := Vector2(178, 124)

var simulation: SimulationPort
var catalog := ContentCatalog.new()
var targeting := TargetingSession.new()
var animation_director: SkillAnimationDirector
var placeholder_presenter: Node
var text_renderer := CombatTextRenderer.new()
var active_actor_id := BETTY_ID
var selected_skill_id := ""
var snapshot_actors: Array = []
var description_label: RichTextLabel
var actor_panel: PanelContainer
var actor_name: Label
var enemy_panel: PanelContainer
var card_buttons: Dictionary = {}
var card_portraits: Dictionary = {}
var status_labels: Dictionary = {}
var band_rail: HBoxContainer
var band_names_by_index: Dictionary = {}
var captain_command_buttons: Dictionary = {}
var captain_command_grid: Control
var intent_label: Label
var round_label: Label
var target_label: Label
var action_cue_label: Label
var command_dock: Control
var return_to_expedition_button: Button
var is_campaign_encounter := false
var command_buttons: Dictionary = {}
var actor_display_names := {
	"character.protagonist.captain": "MICHAEL CORRIGAN",
	"character.heroine.betty": "BETTY",
	"character.heroine.ayla": "AYLA",
	"character.heroine.vix": "VIX",
	"character.heroine.grisha": "GRISHA"
}

func _ready() -> void:
	text_renderer.configure(actor_display_names)
	if catalog.load_default() != OK:
		push_error("The battle prototype requires the validated generated content bundle.")
	animation_director = SkillAnimationDirector.new()
	animation_director.name = "SkillAnimationDirector"
	animation_director.beat_started.connect(on_animation_beat)
	animation_director.presentation_cue_started.connect(on_presentation_cue)
	animation_director.event_cued.connect(on_animation_event_cued)
	animation_director.action_finished.connect(on_animation_finished)
	add_child(animation_director)
	var campaign_session := get_node_or_null("/root/CampaignSession")
	if campaign_session != null and campaign_session.has_method("has_pending_encounter") and campaign_session.has_pending_encounter():
		simulation = CampaignEncounterSimulationPortScript.new(campaign_session)
		is_campaign_encounter = true
	elif OS.has_feature("web") and OS.is_debug_build():
		# Browser previews cannot load the Windows Rust extension. They use the
		# presentation fixture deliberately, without asking ClassDB for a class
		# that cannot exist in this export.
		simulation = MockSimulationPort.new()
	elif ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS):
		simulation = NativeSimulationPort.new()
	elif OS.is_debug_build():
		push_warning("Rust GDExtension unavailable; using the development-only mock simulation.")
		simulation = MockSimulationPort.new()
	else:
		push_error("Release startup refused: Project42SimulationBridge is unavailable and mock gameplay is forbidden.")
		get_tree().quit(78)
		return
	build_screen()
	placeholder_presenter = PlaceholderActionPresenterScript.new()
	placeholder_presenter.name = "PlaceholderActionPresenter"
	add_child(placeholder_presenter)
	placeholder_presenter.configure(actor_panel, enemy_panel, action_cue_label)
	project_snapshot(simulation.create_debug_battle())
	for event in simulation.start():
		project_event(event)

func build_screen() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(make_color_rect(Color("07100f"), "BackgroundBase"))
	var safe := MarginContainer.new()
	safe.name = "SafeFrame"
	safe.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	safe.add_theme_constant_override("margin_left", 26)
	safe.add_theme_constant_override("margin_right", 26)
	safe.add_theme_constant_override("margin_top", 18)
	safe.add_theme_constant_override("margin_bottom", 18)
	add_child(safe)
	# The Bible's battle screen is one shared theatrical plane. Cards and commands
	# sit on its lower edge; actors share one floor. There is no decorative top
	# rail and no slot, diamond or crest without a live game rule behind it.
	var root := Control.new()
	root.name = "BattlefieldComposition"
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	safe.add_child(root)
	root.add_child(build_battle_plane())

func build_header() -> Control:
	var bar := HBoxContainer.new()
	bar.custom_minimum_size.y = 46
	var title := make_label("RETURNING NAMES  /  RECEPTION ROAD", 22, BRONZE)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(title)
	round_label = make_label("DAY 18  •  16:40  •  ROUND 1", 15, CREAM)
	bar.add_child(round_label)
	return bar

func build_battle_plane() -> Control:
	var plane := Control.new()
	plane.name = "BattlePlane"
	plane.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	var paper_stage := PaperStageScript.new()
	paper_stage.name = "PaperStage"
	paper_stage.mouse_filter = Control.MOUSE_FILTER_IGNORE
	paper_stage.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	plane.add_child(paper_stage)
	# The only persistent information at the top is information the player can
	# use: location/time and the current enemy intent. No ornamental meter.
	round_label = make_label("DAY 18  •  16:40", 15, CREAM)
	round_label.position = Vector2(34, 28)
	round_label.size = Vector2(250, 28)
	plane.add_child(round_label)
	description_label = RichTextLabel.new()
	description_label.name = "CombatDescription"
	description_label.bbcode_enabled = true
	description_label.fit_content = false
	description_label.add_theme_font_size_override("normal_font_size", 14)
	description_label.add_theme_color_override("default_color", CREAM)
	description_label.text = "[color=#b78a4b]RECEPTION ROAD[/color]  •  ELVEN GATE  •  LATE AFTERNOON"
	description_label.position = Vector2(34, 54)
	description_label.size = Vector2(520, 34)
	description_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	plane.add_child(description_label)
	intent_label = make_label("RAZORBEAK  •  RUSHING BITE  •  16 DAMAGE", 15, DANGER)
	intent_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	intent_label.position = Vector2(1170, 48)
	intent_label.size = Vector2(650, 30)
	plane.add_child(intent_label)
	# A foreground active actor, an individual raptor, and a later enemy socket
	# occupy the same painted floor. Their panels are not half-screen columns.
	actor_panel = make_actor_placeholder("presentation.paper_doll.betty.active", TEAL)
	actor_panel.position = Vector2(330, 175)
	actor_panel.size = Vector2(500, 615)
	plane.add_child(actor_panel)
	enemy_panel = make_actor_placeholder("presentation.paper_doll.razorbeak.active", DANGER)
	enemy_panel.position = Vector2(1080, 330)
	enemy_panel.size = Vector2(450, 455)
	enemy_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	enemy_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	enemy_panel.gui_input.connect(on_enemy_gui_input)
	plane.add_child(enemy_panel)
	action_cue_label = make_label("BETTY IS READY", 14, Color("e4b75e"))
	action_cue_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	action_cue_label.position = Vector2(470, 760)
	action_cue_label.size = Vector2(220, 28)
	plane.add_child(action_cue_label)
	# Every actor the simulation reports receives one card, standing in the band
	# the simulation puts it in. The prototype does not invent empty roster slots
	# and does not draw a band nobody occupies.
	band_rail = HBoxContainer.new()
	band_rail.name = "BandRail"
	band_rail.position = Vector2(32, 712)
	band_rail.size = Vector2(600, 320)
	band_rail.add_theme_constant_override("separation", 14)
	plane.add_child(band_rail)
	command_dock = build_footer()
	command_dock.position = Vector2(650, 805)
	command_dock.size = Vector2(560, 175)
	plane.add_child(command_dock)
	return_to_expedition_button = Button.new()
	return_to_expedition_button.name = "ReturnToExpedition"
	return_to_expedition_button.text = "RETURN TO RECEPTION TERRACE"
	return_to_expedition_button.tooltip_text = "Return to the expedition after the authoritative encounter outcome has been recorded."
	return_to_expedition_button.add_theme_font_size_override("font_size", 14)
	return_to_expedition_button.add_theme_stylebox_override("normal", make_command_box(Color("1a322e"), TEAL))
	return_to_expedition_button.position = Vector2(1260, 840)
	return_to_expedition_button.size = Vector2(450, 72)
	return_to_expedition_button.visible = false
	return_to_expedition_button.pressed.connect(return_to_expedition)
	plane.add_child(return_to_expedition_button)
	return plane

func build_footer() -> Control:
	var footer := HBoxContainer.new()
	footer.custom_minimum_size.y = 166
	footer.add_theme_constant_override("separation", 10)
	target_label = make_label("BETTY\nD-RANK", 12, BRONZE)
	target_label.custom_minimum_size.x = 120
	target_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	footer.add_child(target_label)
	var command_rows := VBoxContainer.new()
	command_rows.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	command_rows.add_theme_constant_override("separation", 0)
	var first_row := HBoxContainer.new()
	first_row.alignment = BoxContainer.ALIGNMENT_CENTER
	first_row.add_theme_constant_override("separation", 4)
	var second_row := HBoxContainer.new()
	second_row.alignment = BoxContainer.ALIGNMENT_CENTER
	second_row.add_theme_constant_override("separation", 4)
	# Seven diamonds correspond exactly to Betty's D through SSS authored skill
	# records. Six are locked because their named milestones have not occurred;
	# they are future capabilities, not decorative empty inventory slots.
	var commands := [
		["skill.betty.guarded_strike", "D", "Guarded Strike", true],
		["skill.betty.condition_cleanse", "C", "Condition Cleanse", false],
		["skill.betty.rescue_charge", "B", "Rescue Charge", false],
		["skill.betty.healing_impact", "A", "Healing Impact", false],
		["skill.betty.fatal_intercept", "S", "Fatal Intercept", false],
		["skill.betty.mobile_infirmary", "SS", "Mobile Infirmary", false],
		["skill.betty.combat_revival", "SSS", "Combat Revival", false]
	]
	for index in commands.size():
		var command: Array = commands[index]
		var gem := PaperSkillDiamondScript.new()
		gem.configure(command[0], command[1], command[2], command[3])
		gem.command_pressed.connect(func(skill_id: String) -> void: begin_skill_targeting(skill_id))
		if index < 4:
			first_row.add_child(gem)
		else:
			second_row.add_child(gem)
		command_buttons[command[0]] = gem
	command_rows.add_child(first_row)
	command_rows.add_child(second_row)
	footer.add_child(command_rows)
	return footer

func make_stage_box() -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = Color("0d1a19")
	box.border_color = Color("735d39")
	box.set_border_width_all(2)
	box.set_corner_radius_all(10)
	box.content_margin_left = 8
	box.content_margin_right = 8
	box.content_margin_top = 8
	box.content_margin_bottom = 8
	return box

## Rebuild the rail from the actors the simulation reports. Cards are grouped by
## the `band_name` the bridge sends, in `BAND_ORDER`; a band nobody stands in is
## not drawn. Called on every snapshot and whenever an actor changes band.
func rebuild_band_rail() -> void:
	if band_rail == null:
		return
	for child in band_rail.get_children():
		band_rail.remove_child(child)
		child.queue_free()
	card_buttons.clear()
	card_portraits.clear()
	status_labels.clear()
	captain_command_buttons.clear()
	captain_command_grid = null
	var drawn := 0
	for band in BAND_ORDER:
		var occupants := actors_in_band(band)
		if occupants.is_empty():
			continue
		band_rail.add_child(make_band_column(band, occupants))
		drawn += occupants.size()
	if drawn != snapshot_actors.size():
		push_error("The simulation reported %d actors but only %d stand in a named band." % [snapshot_actors.size(), drawn])
	refresh_command_availability()

func actors_in_band(band: String) -> Array:
	var occupants: Array = []
	for actor in snapshot_actors:
		if str(actor.get("band_name", "")) == band:
			occupants.append(actor)
	return occupants

func make_band_column(band: String, occupants: Array) -> Control:
	var column := VBoxContainer.new()
	column.name = "BandColumn_" + band
	column.add_theme_constant_override("separation", 6)
	var heading := make_label(band.replace("_", " ").to_upper(), 12, BRONZE)
	heading.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(heading)
	for actor in occupants:
		column.add_child(make_band_card(actor))
	return column

func make_band_card(actor: Dictionary) -> Control:
	var id := str(actor.get("id", ""))
	var band := str(actor.get("band_name", ""))
	var display_name := str(actor.get("display_name", id)).to_upper()
	var holder := VBoxContainer.new()
	holder.name = "Card_" + id.replace(".", "_")
	holder.add_theme_constant_override("separation", 4)
	var card := Control.new()
	card.custom_minimum_size = CARD_SIZE
	card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	card.tooltip_text = "PAPER PORTRAIT — DEVELOPMENT BLOCKOUT\nStable actor ID: %s\nBand: %s\nFuture portrait must preserve card crop, face, hair, outfit palette and readiness overlay." % [id, band]
	var portrait := PaperCardScript.new()
	portrait.name = "Portrait"
	portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	portrait.configure(display_name, band, accent_for(actor), int(actor.get("composure", 0)), actor_is_shaken(actor), portrait_kind_for(actor))
	card.add_child(portrait)
	var copy := VBoxContainer.new()
	copy.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.set_anchors_preset(Control.PRESET_FULL_RECT)
	copy.offset_left = 70
	copy.offset_top = 14
	copy.offset_right = -8
	copy.offset_bottom = -22
	var name_label := make_label(display_name, 14, CREAM)
	name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(name_label)
	# The band label is the string the bridge sent for this actor. The screen
	# never spells a band name of its own.
	var band_label := make_label(band, 10, Color("caa66a"))
	band_label.name = "BandLabel"
	band_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(band_label)
	var status_label := make_label("", 11, Color("d7e0d7"))
	status_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(status_label)
	card.add_child(copy)
	holder.add_child(card)
	if id == CAPTAIN_ID:
		captain_command_grid = make_captain_command_grid()
		holder.add_child(captain_command_grid)
	card_buttons[id] = holder
	card_portraits[id] = portrait
	status_labels[id] = status_label
	update_actor_status(actor)
	return holder

## Captain Michael's command grid, unfolded on his card when he is the active
## actor. Every entry is submitted through the same SimulationPort path Betty's
## commands use; the simulation remains the authority on whether it is legal.
func make_captain_command_grid() -> Control:
	var grid := VBoxContainer.new()
	grid.name = "CaptainCommandGrid"
	grid.visible = false
	grid.add_theme_constant_override("separation", 3)
	for entry in CAPTAIN_COMMANDS:
		var skill_id: String = entry[0]
		var button := Button.new()
		button.name = "Command_" + skill_id.replace(".", "_")
		button.text = captain_command_label(skill_id, entry[1])
		button.add_theme_font_size_override("font_size", 12)
		button.add_theme_stylebox_override("normal", make_command_box(Color("1a322e"), TEAL))
		button.tooltip_text = "Captain Michael's command. Stable skill ID: %s" % skill_id
		button.pressed.connect(func() -> void: on_captain_command(skill_id))
		grid.add_child(button)
		captain_command_buttons[skill_id] = button
	return grid

func captain_command_label(skill_id: String, fallback_label: String) -> String:
	if not catalog.has(skill_id):
		return fallback_label
	return str(catalog.get_record(skill_id).get("displayName", fallback_label)).to_upper()

func on_captain_command(skill_id: String) -> void:
	if animation_director.playing:
		return
	if command_selects_another_actor(skill_id):
		begin_skill_targeting(skill_id)
	else:
		submit_skill(skill_id, [])

## The authored record decides whether a command points at another actor.
## Reposition is `targetRule: self` and Hold Position is the universal verb with
## no record; neither opens a targeting session.
func command_selects_another_actor(skill_id: String) -> bool:
	if not catalog.has(skill_id):
		return false
	return str(catalog.get_record(skill_id).get("targetRule", "")) != "self"

func accent_for(actor: Dictionary) -> Color:
	if str(actor.get("id", "")) == CAPTAIN_ID:
		return Color("536c79")
	return DANGER if str(actor.get("faction", "")) == "hostile" else Color("2d7770")

func portrait_kind_for(actor: Dictionary) -> String:
	if str(actor.get("id", "")) == CAPTAIN_ID:
		return "captain"
	return "hostile" if str(actor.get("faction", "")) == "hostile" else "heroine"

func actor_is_shaken(actor: Dictionary) -> bool:
	for status in actor.get("statuses", []):
		if status is Dictionary and str(status.get("kind", "")) == "shaken":
			return true
	return false

## The band name the bridge gave for a stored band index, learned from the
## actors in the snapshots it sends. The screen never derives a name from the
## integer itself.
func band_name_for_index(band: int) -> String:
	if band_names_by_index.has(band):
		return str(band_names_by_index[band])
	push_error("The simulation has not named band %d; the screen does not invent band names." % band)
	return ""

func card_band_name(actor_id: String) -> String:
	var portrait := card_portraits.get(actor_id) as PaperCard
	return "" if portrait == null else portrait.band_label

func drawn_card_ids() -> Array:
	return card_buttons.keys()

func make_command_box(background: Color, border: Color) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = background
	box.border_color = border
	box.set_border_width_all(2)
	box.set_corner_radius_all(7)
	box.shadow_color = Color(0, 0, 0, .35)
	box.shadow_size = 4
	box.content_margin_left = 10
	box.content_margin_right = 10
	return box

func make_actor_placeholder(spec_id: String, accent: Color) -> PanelContainer:
	var spec := catalog.get_registry_entry(spec_id)
	var panel := PanelContainer.new()
	# The panel remains the interaction owner for targeting and animation. Its
	# background is explicitly transparent: a stage actor must not stand inside
	# a fake poster card.
	var transparent_panel := StyleBoxEmpty.new()
	panel.add_theme_stylebox_override("panel", transparent_panel)
	panel.custom_minimum_size = Vector2(320, 360)
	var stack := VBoxContainer.new()
	stack.alignment = BoxContainer.ALIGNMENT_CENTER
	stack.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var doll := PaperDollScript.new()
	doll.name = "PaperDoll"
	doll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	doll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	doll.mouse_filter = Control.MOUSE_FILTER_IGNORE
	doll.configure(spec, accent)
	stack.add_child(doll)
	# Actor identity and state belong on their compact rail card and in the
	# useful battle labels. The stage itself shows bodies and action, not a
	# redundant poster title underneath each figure.
	panel.add_child(stack)
	return panel

func select_actor(id: String, display_name: String, accent: Color) -> void:
	description_label.text = "[color=#b78a4b]INSPECT[/color] %s is previewed from the card rail. The simulation still owns whose turn it is." % display_name

func on_party_card_pressed(id: String, display_name: String, accent: Color) -> void:
	if targeting.active:
		accept_target(id)
	else:
		select_actor(id, display_name, accent)

func on_enemy_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		accept_target("enemy.raptor.razorbeak")

func begin_skill_targeting(skill_id: String) -> void:
	if animation_director.playing:
		return
	selected_skill_id = skill_id
	var skill_record := catalog.get_record(skill_id)
	var result := targeting.begin(skill_record, snapshot_actors)
	match result.get("status", ""):
		"ready": submit_skill(skill_id, result.target_ids)
		"selecting":
			target_label.text = result.prompt
			description_label.text = "[color=#b78a4b]TARGETING[/color] %s — %s." % [skill_record.get("displayName", skill_id), result.prompt]
		"automatic":
			target_label.text = "AUTOMATIC REACTION — NO MANUAL TARGET"

func accept_target(actor_id: String) -> void:
	if not targeting.active:
		return
	var result := targeting.select(actor_id)
	match result.get("status", ""):
		"ready": submit_skill(selected_skill_id, result.target_ids)
		"selecting": target_label.text = result.prompt
		"illegal": description_label.text = "[color=#c24e45]ILLEGAL TARGET[/color] %s. %s." % [result.reason, result.prompt]

func submit_skill(skill_id: String, targets: Array) -> void:
	target_label.text = "RESOLVING %s" % skill_id.to_upper()
	set_commands_enabled(false)
	set_captain_commands_enabled(false)
	var command := {
		"command_id": "command.debug.%s" % Time.get_ticks_msec(),
		"battle_id": "battle.prototype.returning_names",
		"actor_id": active_actor_id,
		"kind": "use_skill",
		"skill_id": skill_id,
		"target_ids": targets,
		"payload": {}
	}
	await project_command_events(simulation.submit(command))

func project_command_events(events: Array[Dictionary], run_followups := true) -> void:
	var index := 0
	while index < events.size():
		var event: Dictionary = events[index]
		project_event(event)
		index += 1
		if event.get("kind", "") == "command_accepted":
			var skill_id: String = str(event.payload.get("skill_id", ""))
			var action_events: Array[Dictionary] = []
			while index < events.size() and events[index].get("kind", "") != "turn_started":
				action_events.append(events[index])
				index += 1
			if catalog.has(skill_id):
				await animation_director.play(catalog.get_record(skill_id), action_events)
			else:
				for deferred_event in action_events:
					project_event(deferred_event)
	target_label.text = "COMMAND READY"
	targeting.cancel()
	selected_skill_id = ""
	if run_followups:
		await run_automatic_turns()

## Play out every turn the player does not own. Betty and Captain Michael both
## stop the cycle: an actor with a command grid is not held automatically.
func run_automatic_turns() -> void:
	while not PLAYER_COMMANDED_ACTOR_IDS.has(active_actor_id):
		var command_id := "command.prototype.auto.%s" % Time.get_ticks_usec()
		var automatic_command := {
			"command_id": command_id,
			"battle_id": "battle.prototype.returning_names",
			"actor_id": active_actor_id,
			"kind": "use_skill",
			"skill_id": "skill.system.hold_position",
			"target_ids": [],
			"payload": {}
		}
		if active_actor_id == "enemy.raptor.razorbeak":
			automatic_command = simulation.recommended_enemy_command(command_id)
			if not automatic_command.get("available", false):
				description_label.text += " [color=#c24e45]ENEMY DECISION FAILED[/color] %s." % automatic_command.get("reason", "unknown_reason")
				return
		var events: Array[Dictionary] = simulation.submit(automatic_command)
		if events.is_empty() or events[0].get("kind", "") == "command_rejected":
			for event in events:
				project_event(event)
			return
		await project_command_events(events, false)

func project_snapshot(snapshot: Dictionary) -> void:
	description_label.text = "[color=#b78a4b]OBSERVED[/color] %s" % snapshot.get("description", "No description supplied.")
	snapshot_actors = snapshot.get("actors", []).duplicate(true)
	for actor in snapshot_actors:
		band_names_by_index[int(actor.get("band", -1))] = str(actor.get("band_name", ""))
	rebuild_band_rail()

func project_event(event: Dictionary) -> void:
	var narration := text_renderer.render(event)
	apply_narration(narration)
	match event.get("kind", ""):
		"battle_started": pass
		"turn_started":
			var actor_id: String = event.subjects[0]
			active_actor_id = actor_id
			round_label.text = "DAY 18  •  16:40  •  ROUND %d" % event.payload.round
			set_card_state(actor_id, "FOCUSED")
			refresh_command_availability()
		"command_accepted": pass
		"actor_focused": set_card_state(event.subjects[0], "ACTIVE")
		"enemy_intent_declared":
			intent_label.text = narration.get("intent", "INTENT: UNKNOWN")
		"damage_applied":
			var target_id: String = event.subjects[-1]
			update_status_values(target_id, event.payload.remaining_vitality)
		"guard_changed": update_guard_values(event.subjects[0], event.payload.total)
		"actor_moved": update_actor_band(event.subjects[0], int(event.payload.to_band))
		"vitality_changed":
			update_status_values(event.subjects[0], event.payload.total)
		"actor_revived":
			update_status_values(event.subjects[0], event.payload.vitality)
		"battle_ended":
			set_commands_enabled(false)
			set_captain_commands_enabled(false)
			command_dock.visible = false
			action_cue_label.visible = false
			intent_label.add_theme_color_override("font_color", TEAL if event.payload.victory else DANGER)
			intent_label.text = "RAZORBEAK WITHDRAWS" if event.payload.victory else "BETTY IS DOWN"
			description_label.text = "[color=#4fc7b4]The razorbeak breaks away into the jungle.[/color] The road is quiet for the moment." if event.payload.victory else "[color=#c24e45]Betty falls beneath the elven gate.[/color] The expedition must recover before midnight."
			if is_campaign_encounter:
				return_to_expedition_button.visible = true
		_: pass

func apply_narration(narration: Dictionary) -> void:
	match narration.get("mode", "none"):
		"replace": description_label.text = narration.get("text", "")
		"append": description_label.text += narration.get("text", "")

func on_animation_beat(skill_id: String, _beat_index: int, beat: Dictionary) -> void:
	var readable_name := str(beat.get("name", "unnamed_beat")).replace("_", " ").to_upper()
	target_label.text = "%s  •  %s" % [skill_id.trim_prefix("skill.").replace(".", " ").replace("_", " ").to_upper(), readable_name]

func on_animation_event_cued(_skill_id: String, _beat_name: String, event: Dictionary) -> void:
	project_event(event)

func on_presentation_cue(_skill_id: String, cue: Dictionary) -> void:
	var resolved := cue.duplicate(true)
	var camera_id := str(cue.get("cameraId", ""))
	resolved.cameraRecord = catalog.get_registry_entry(camera_id)
	var vfx_id := str(cue.get("vfxId", ""))
	resolved.vfxRecord = catalog.get_registry_entry(vfx_id)
	placeholder_presenter.present(resolved)

func on_animation_finished(_skill_id: String) -> void:
	placeholder_presenter.reset()

func update_actor_status(actor: Dictionary) -> void:
	var id := str(actor.get("id", ""))
	if not status_labels.has(id):
		return
	var status := status_labels[id] as Label
	var state := "DEFEATED" if int(actor.get("vitality", 0)) <= 0 else "READY"
	status.text = "VIT %d/%d  •  GRD %d\n%s" % [int(actor.get("vitality", 0)), int(actor.get("max_vitality", 0)), int(actor.get("guard", 0)), state]

## An actor changed band. The destination's name is the one the bridge gave for
## that stored index, so the card carries the simulation's own word for it.
func update_actor_band(id: String, band: int) -> void:
	var destination := band_name_for_index(band)
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.band = band
			actor.band_name = destination
			break
	rebuild_band_rail()
	set_card_state(active_actor_id, "ACTIVE")

func update_status_values(id: String, vitality: int) -> void:
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.vitality = vitality
			update_actor_status(actor)
			break

func update_guard_values(id: String, guard: int) -> void:
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.guard = guard
			update_actor_status(actor)
			return

func set_card_state(id: String, state: String) -> void:
	for card_id in card_buttons:
		var card := card_buttons[card_id] as Control
		card.modulate = Color.WHITE if card_id == id else Color("9aa5a2")
		if card_id == id:
			card.tooltip_text = "Card state: %s. Stable actor ID: %s" % [state, id]

func refresh_command_availability() -> void:
	set_commands_enabled(active_actor_id == BETTY_ID)
	set_captain_commands_enabled(active_actor_id == CAPTAIN_ID)

func set_commands_enabled(enabled: bool) -> void:
	for skill_id in command_buttons:
		var button := command_buttons[skill_id] as PaperSkillDiamond
		var has_legal_targets := catalog.has(skill_id) and targeting.has_legal_targets(catalog.get_record(skill_id), snapshot_actors)
		var unlocks_in_opening: bool = str(skill_id) == "skill.betty.guarded_strike"
		button.set_command_enabled(enabled and unlocks_in_opening and has_legal_targets)
		if enabled and not has_legal_targets:
			button.tooltip_text = "Unavailable in the current authoritative encounter: no legal target exists. Stable skill ID: %s" % skill_id

## Michael's grid unfolds on his turn and is folded away and disabled on anyone
## else's. Legality is not decided here: a command that points at another actor
## is offered when the authored target rule has a legal target, exactly as
## Betty's are, and the simulation may still reject the submitted command.
func set_captain_commands_enabled(enabled: bool) -> void:
	if captain_command_grid != null:
		captain_command_grid.visible = enabled
	for skill_id in captain_command_buttons:
		var button := captain_command_buttons[skill_id] as Button
		var has_legal_targets := true
		if command_selects_another_actor(skill_id):
			has_legal_targets = targeting.has_legal_targets(catalog.get_record(skill_id), snapshot_actors)
		button.disabled = not (enabled and has_legal_targets)
		if enabled and not has_legal_targets:
			button.tooltip_text = "Unavailable in the current authoritative encounter: no legal target exists. Stable skill ID: %s" % skill_id

func is_command_enabled(skill_id: String) -> bool:
	if captain_command_buttons.has(skill_id):
		var captain_button := captain_command_buttons[skill_id] as Button
		return captain_button != null and not captain_button.disabled
	var diamond := command_buttons.get(skill_id) as PaperSkillDiamond
	return diamond != null and diamond.command_button != null and not diamond.command_button.disabled


func return_to_expedition() -> void:
	if not is_campaign_encounter:
		return
	get_node("/root/SceneFlow").return_to_expedition()

func make_color_rect(color: Color, node_name: String) -> ColorRect:
	var rect := ColorRect.new()
	rect.name = node_name
	rect.color = color
	rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	return rect

func make_label(value: String, size: int, color: Color) -> Label:
	var label := Label.new()
	label.text = value
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", color)
	return label
