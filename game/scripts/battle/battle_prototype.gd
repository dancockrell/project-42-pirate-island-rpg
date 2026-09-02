class_name BattlePrototype
extends Control

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")
const PaperDollScript = preload("res://scripts/battle/paper_doll.gd")
const PaperStageScript = preload("res://scripts/battle/paper_stage.gd")
const PaperCardScript = preload("res://scripts/battle/paper_card.gd")
const PaperSkillDiamondScript = preload("res://scripts/battle/paper_skill_diamond.gd")

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
var command_dock: Control
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
	actor_panel.position = Vector2(250, 190)
	actor_panel.size = Vector2(420, 560)
	plane.add_child(actor_panel)
	enemy_panel = make_actor_placeholder("presentation.paper_doll.razorbeak.active", DANGER)
	enemy_panel.position = Vector2(1030, 285)
	enemy_panel.size = Vector2(385, 420)
	enemy_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	enemy_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	enemy_panel.gui_input.connect(on_enemy_gui_input)
	plane.add_child(enemy_panel)
	action_cue_label = make_label("BETTY IS READY", 14, Color("e4b75e"))
	action_cue_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	action_cue_label.position = Vector2(400, 755)
	action_cue_label.size = Vector2(220, 28)
	plane.add_child(action_cue_label)
	# Only present party members receive cards. The prototype does not invent
	# empty roster slots merely to imitate a reference composition.
	var cards := HBoxContainer.new()
	cards.name = "PartyCardRail"
	cards.position = Vector2(32, 794)
	cards.size = Vector2(420, 172)
	cards.add_theme_constant_override("separation", 12)
	for item in [
		["character.protagonist.captain", "MICHAEL", "STEAM CUTTER CAPTAIN", Color("536c79"), "MC", "captain"],
		["character.heroine.betty", "BETTY", "FIELD MEDIC", Color("2d7770"), "D"]
	]:
		var card := make_party_card(item[0], item[1], item[2], item[3], item[4], item[5] if item.size() > 5 else "heroine")
		cards.add_child(card)
		card_buttons[item[0]] = card
	plane.add_child(cards)
	command_dock = build_footer()
	command_dock.position = Vector2(650, 805)
	command_dock.size = Vector2(560, 175)
	plane.add_child(command_dock)
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

func make_party_card(id: String, display_name: String, role: String, accent: Color, rank: String, portrait_kind := "heroine") -> Control:
	var card := Control.new()
	card.name = display_name + "Card"
	card.custom_minimum_size = Vector2(190, 146)
	card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	card.tooltip_text = "PAPER PORTRAIT — DEVELOPMENT BLOCKOUT\nStable actor ID: %s\nRole: %s\nFuture portrait must preserve card crop, role read, face, hair, outfit palette and readiness overlay." % [id, role]
	var portrait := PaperCardScript.new()
	portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	portrait.configure(display_name, role, accent, rank, portrait_kind)
	card.add_child(portrait)
	var copy := VBoxContainer.new()
	copy.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.set_anchors_preset(Control.PRESET_FULL_RECT)
	copy.offset_left = 70
	copy.offset_top = 20
	copy.offset_right = -10
	copy.offset_bottom = -18
	var name_label := make_label(display_name, 16, CREAM)
	name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(name_label)
	var role_label := make_label(role, 10, Color("caa66a"))
	role_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(role_label)
	var initial_status := "ECHO DECK\nSTANDBY" if id == "character.protagonist.captain" else "VIT 84/100  •  GRD 0\nREADY"
	var status_label := make_label(initial_status, 11, Color("d7e0d7"))
	status_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(status_label)
	card.add_child(copy)
	if id != "character.protagonist.captain":
		status_labels[id] = status_label
	return card

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
			command_dock.visible = false
			action_cue_label.visible = false
			intent_label.add_theme_color_override("font_color", TEAL if event.payload.victory else DANGER)
			intent_label.text = "RAZORBEAK WITHDRAWS" if event.payload.victory else "BETTY IS DOWN"
			description_label.text = "[color=#4fc7b4]The razorbeak breaks away into the jungle.[/color] The road is quiet for the moment." if event.payload.victory else "[color=#c24e45]Betty falls beneath the elven gate.[/color] The expedition must recover before midnight."
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
		var status := status_labels[id] as Label
		var name: String = actor_display_names.get(id, actor.display_name.to_upper())
		var state := "DEFEATED" if actor.vitality <= 0 else "READY"
		status.text = "VIT %d/%d  •  GRD %d\n%s" % [actor.vitality, actor.max_vitality, actor.guard, state]

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

func set_commands_enabled(enabled: bool) -> void:
	for skill_id in command_buttons:
		var button := command_buttons[skill_id] as PaperSkillDiamond
		var has_legal_targets := catalog.has(skill_id) and targeting.has_legal_targets(catalog.get_record(skill_id), snapshot_actors)
		var unlocks_in_opening: bool = str(skill_id) == "skill.betty.guarded_strike"
		button.set_command_enabled(enabled and unlocks_in_opening and has_legal_targets)
		if enabled and not has_legal_targets:
			button.tooltip_text = "Unavailable in the current authoritative encounter: no legal target exists. Stable skill ID: %s" % skill_id

func is_command_enabled(skill_id: String) -> bool:
	var diamond := command_buttons.get(skill_id) as PaperSkillDiamond
	return diamond != null and diamond.command_button != null and not diamond.command_button.disabled

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
