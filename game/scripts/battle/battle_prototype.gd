class_name BattlePrototype
extends Control

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")

## Presentation-only prototype. Gameplay truth comes through SimulationPort.
## All generated shapes and labels are explicit placeholders registered in
## content/art/placeholders.json.

const BRONZE := Color("b78a4b")
const DEEP := Color("101817")
const PANEL := Color("182321")
const CREAM := Color("eadfca")
const TEAL := Color("4fc7b4")
const DANGER := Color("c24e45")

var simulation: SimulationPort
var catalog := ContentCatalog.new()
var targeting := TargetingSession.new()
var animation_director: SkillAnimationDirector
var placeholder_presenter: Node
var text_renderer := CombatTextRenderer.new()
var active_actor_id := "character.heroine.betty"
var selected_skill_id := ""
var snapshot_actors: Array = []
var description_label: RichTextLabel
var actor_panel: PanelContainer
var actor_name: Label
var enemy_panel: PanelContainer
var card_buttons: Dictionary = {}
var status_labels: Dictionary = {}
var intent_label: Label
var round_label: Label
var target_label: Label
var action_cue_label: Label
var command_buttons: Dictionary = {}
var actor_display_names := {
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
	if OS.has_feature("web") and OS.is_debug_build():
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
	safe.add_theme_constant_override("margin_left", 38)
	safe.add_theme_constant_override("margin_right", 38)
	safe.add_theme_constant_override("margin_top", 30)
	safe.add_theme_constant_override("margin_bottom", 30)
	add_child(safe)
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 14)
	safe.add_child(root)
	root.add_child(build_header())
	root.add_child(build_battle_plane())
	root.add_child(build_footer())

func build_header() -> Control:
	var bar := HBoxContainer.new()
	bar.custom_minimum_size.y = 62
	var title := make_label("RETURNING NAMES — RECEPTION ROAD", 25, BRONZE)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(title)
	round_label = make_label("DAY 18  •  16:40  •  ROUND 1", 18, CREAM)
	bar.add_child(round_label)
	return bar

func build_battle_plane() -> Control:
	var plane := HBoxContainer.new()
	plane.size_flags_vertical = Control.SIZE_EXPAND_FILL
	plane.add_theme_constant_override("separation", 18)
	var cards := VBoxContainer.new()
	cards.custom_minimum_size.x = 280
	cards.add_theme_constant_override("separation", 9)
	for item in [
		["character.heroine.betty", "BETTY", "FIELD MEDIC", Color("2d7770")],
		["character.heroine.ayla", "AYLA", "TOMB WARDEN", Color("648243")],
		["character.heroine.vix", "VIX", "CORSAIR", Color("a05242")],
		["character.heroine.grisha", "GRISHA", "OFFICER", Color("704339")]
	]:
		var card := make_party_card(item[0], item[1], item[2], item[3])
		cards.add_child(card)
		card_buttons[item[0]] = card
	plane.add_child(cards)
	var stage := VBoxContainer.new()
	stage.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stage.add_theme_constant_override("separation", 12)
	var visual := HBoxContainer.new()
	visual.size_flags_vertical = Control.SIZE_EXPAND_FILL
	visual.add_theme_constant_override("separation", 18)
	actor_panel = make_actor_placeholder("BETTY", "DUMMY FULL-BODY ACTOR\nREPLACE: art.character.betty.battle.base", TEAL)
	actor_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	visual.add_child(actor_panel)
	enemy_panel = make_actor_placeholder("RAZORBEAK", "DUMMY ENEMY ACTOR\nLEVEL 7 • INDIVIDUAL THREAT", DANGER)
	enemy_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	enemy_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	enemy_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	enemy_panel.gui_input.connect(on_enemy_gui_input)
	visual.add_child(enemy_panel)
	stage.add_child(visual)
	action_cue_label = make_label("DUMMY ACTION CUE — WAITING FOR AUTHORED BEAT", 13, Color("e4b75e"))
	action_cue_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	action_cue_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	action_cue_label.custom_minimum_size.y = 42
	stage.add_child(action_cue_label)
	intent_label = make_label("INTENT: OBSERVING", 18, DANGER)
	intent_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	stage.add_child(intent_label)
	description_label = RichTextLabel.new()
	description_label.name = "CombatDescription"
	description_label.bbcode_enabled = true
	description_label.fit_content = false
	description_label.custom_minimum_size.y = 104
	description_label.add_theme_font_size_override("normal_font_size", 19)
	description_label.add_theme_color_override("default_color", CREAM)
	description_label.text = "[color=#b78a4b]OBSERVED[/color] The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush."
	stage.add_child(description_label)
	plane.add_child(stage)
	return plane

func build_footer() -> Control:
	var footer := VBoxContainer.new()
	footer.add_theme_constant_override("separation", 8)
	target_label = make_label("COMMAND READY", 16, BRONZE)
	target_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	footer.add_child(target_label)
	var commands := GridContainer.new()
	commands.columns = 4
	commands.custom_minimum_size.y = 156
	commands.add_theme_constant_override("h_separation", 12)
	commands.add_theme_constant_override("v_separation", 10)
	for command in [
		["skill.betty.guarded_strike", "D  GUARDED STRIKE", true],
		["skill.betty.condition_cleanse", "C  CONDITION CLEANSE", true],
		["skill.betty.rescue_charge", "B  RESCUE CHARGE", true],
		["skill.betty.healing_impact", "A  HEALING IMPACT", true],
		["skill.betty.fatal_intercept", "S  FATAL INTERCEPT\n[AUTOMATIC REACTION]", false],
		["skill.betty.mobile_infirmary", "SS  MOBILE INFIRMARY", true],
		["skill.betty.combat_revival", "SSS  COMBAT REVIVAL", true]
	]:
		var button := Button.new()
		button.text = command[1]
		button.custom_minimum_size = Vector2(238, 66)
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.add_theme_font_size_override("font_size", 15)
		button.disabled = not command[2]
		button.tooltip_text = "Stable skill ID: %s%s" % [command[0], " — triggers automatically; it is never a manual command" if not command[2] else ""]
		button.pressed.connect(func() -> void: begin_skill_targeting(command[0]))
		commands.add_child(button)
		command_buttons[command[0]] = button
	footer.add_child(commands)
	return footer

func make_party_card(id: String, display_name: String, role: String, accent: Color) -> Button:
	var card := Button.new()
	card.name = display_name + "Card"
	card.custom_minimum_size = Vector2(280, 150)
	card.text = "%s\n%s\nVIT 84/100  •  READY\n[DUMMY PORTRAIT]" % [display_name, role]
	card.tooltip_text = "Placeholder portrait. Stable actor ID: %s" % id
	card.add_theme_color_override("font_color", CREAM)
	card.add_theme_color_override("font_hover_color", Color.WHITE)
	card.add_theme_color_override("font_pressed_color", accent)
	card.pressed.connect(func() -> void: on_party_card_pressed(id, display_name, accent))
	status_labels[id] = card
	return card

func make_actor_placeholder(display_name: String, replacement: String, accent: Color) -> PanelContainer:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(480, 560)
	var stack := VBoxContainer.new()
	stack.alignment = BoxContainer.ALIGNMENT_CENTER
	stack.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var silhouette := ColorRect.new()
	silhouette.custom_minimum_size = Vector2(250, 390)
	silhouette.color = Color(accent, 0.52)
	silhouette.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stack.add_child(silhouette)
	var label := make_label(display_name, 32, accent)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stack.add_child(label)
	var dummy := make_label(replacement, 15, Color("e4b75e"))
	dummy.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	dummy.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	dummy.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stack.add_child(dummy)
	panel.add_child(stack)
	return panel

func select_actor(id: String, display_name: String, accent: Color) -> void:
	var stack := actor_panel.get_child(0) as VBoxContainer
	(stack.get_child(0) as ColorRect).color = Color(accent, 0.52)
	(stack.get_child(1) as Label).text = display_name
	description_label.text = "[color=#b78a4b]INSPECT[/color] %s is previewed from the card rail. The simulation still owns whose turn it is." % display_name

func on_party_card_pressed(id: String, display_name: String, accent: Color) -> void:
	if targeting.active:
		accept_target(id)
	else:
		select_actor(id, display_name, accent)

func on_enemy_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		accept_target("enemy.raptor.razorbeak.prototype")

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

func run_automatic_turns() -> void:
	while active_actor_id != "character.heroine.betty":
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
		if active_actor_id == "enemy.raptor.razorbeak.prototype":
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
		update_actor_status(actor)

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
			set_commands_enabled(actor_id == "character.heroine.betty")
		"command_accepted": pass
		"actor_focused": set_card_state(event.subjects[0], "ACTIVE")
		"enemy_intent_declared":
			intent_label.text = narration.get("intent", "INTENT: UNKNOWN")
		"damage_applied":
			var target_id: String = event.subjects[-1]
			update_status_values(target_id, event.payload.remaining_vitality)
		"guard_changed": update_guard_values(event.subjects[0], event.payload.total)
		"vitality_changed":
			update_status_values(event.subjects[0], event.payload.total)
		"actor_revived":
			update_status_values(event.subjects[0], event.payload.vitality)
		"battle_ended":
			set_commands_enabled(false)
			intent_label.text = "VICTORY" if event.payload.victory else "DEFEAT — MIDNIGHT RETURN PENDING"
		_: pass

func apply_narration(narration: Dictionary) -> void:
	match narration.get("mode", "none"):
		"replace": description_label.text = narration.get("text", "")
		"append": description_label.text += narration.get("text", "")

func on_animation_beat(skill_id: String, _beat_index: int, beat: Dictionary) -> void:
	var readable_name := str(beat.get("name", "unnamed_beat")).replace("_", " ").to_upper()
	target_label.text = "%s  •  %s" % [skill_id.trim_prefix("skill.betty.").replace("_", " ").to_upper(), readable_name]

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
	var id: String = actor.id
	if status_labels.has(id):
		var card := status_labels[id] as Button
		var name: String = actor_display_names.get(id, actor.display_name.to_upper())
		var state := "DEFEATED" if actor.vitality <= 0 else "READY"
		card.text = "%s\nVIT %d/%d  •  GRD %d  •  %s\n[DUMMY PORTRAIT]" % [name, actor.vitality, actor.max_vitality, actor.guard, state]

func update_status_values(id: String, vitality: int) -> void:
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.vitality = vitality
			update_actor_status(actor)
			break
	if id == "enemy.raptor.razorbeak.prototype":
		var stack := enemy_panel.get_child(0) as VBoxContainer
		(stack.get_child(2) as Label).text = "DUMMY ENEMY ACTOR\nVIT %d/70 • INDIVIDUAL THREAT" % vitality

func update_guard_values(id: String, guard: int) -> void:
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.guard = guard
			update_actor_status(actor)
			return

func set_card_state(id: String, state: String) -> void:
	for card_id in card_buttons:
		var card := card_buttons[card_id] as Button
		card.modulate = Color.WHITE if card_id == id else Color("9aa5a2")
		if card_id == id:
			card.tooltip_text = "Card state: %s. Stable actor ID: %s" % [state, id]

func set_commands_enabled(enabled: bool) -> void:
	for skill_id in command_buttons:
		var button := command_buttons[skill_id] as Button
		var has_legal_targets := catalog.has(skill_id) and targeting.has_legal_targets(catalog.get_record(skill_id), snapshot_actors)
		button.disabled = not enabled or not has_legal_targets
		if enabled and not has_legal_targets:
			button.tooltip_text = "Unavailable in the current authoritative encounter: no legal target exists. Stable skill ID: %s" % skill_id

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
