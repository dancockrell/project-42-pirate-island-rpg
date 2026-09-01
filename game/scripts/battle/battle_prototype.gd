class_name BattlePrototype
extends Control

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
var command_buttons: Dictionary = {}
var actor_display_names := {
	"character.heroine.betty": "BETTY",
	"character.heroine.ayla": "AYLA",
	"character.heroine.vix": "VIX",
	"character.heroine.grisha": "GRISHA"
}

func _ready() -> void:
	if catalog.load_default() != OK:
		push_error("The battle prototype requires the validated generated content bundle.")
	animation_director = SkillAnimationDirector.new()
	animation_director.name = "SkillAnimationDirector"
	animation_director.beat_started.connect(on_animation_beat)
	add_child(animation_director)
	simulation = MockSimulationPort.new()
	build_screen()
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
	var silhouette := ColorRect.new()
	silhouette.custom_minimum_size = Vector2(250, 390)
	silhouette.color = Color(accent, 0.52)
	stack.add_child(silhouette)
	var label := make_label(display_name, 32, accent)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	stack.add_child(label)
	var dummy := make_label(replacement, 15, Color("e4b75e"))
	dummy.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	dummy.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
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

func project_command_events(events: Array[Dictionary]) -> void:
	for event in events:
		project_event(event)
		if event.get("kind", "") == "command_accepted":
			var skill_id: String = str(event.payload.get("skill_id", ""))
			if catalog.has(skill_id):
				await animation_director.play(catalog.get_record(skill_id))
	target_label.text = "COMMAND READY"
	targeting.cancel()
	selected_skill_id = ""

func project_snapshot(snapshot: Dictionary) -> void:
	description_label.text = "[color=#b78a4b]OBSERVED[/color] %s" % snapshot.get("description", "No description supplied.")
	snapshot_actors = snapshot.get("actors", []).duplicate(true)
	for actor in snapshot_actors:
		update_actor_status(actor)

func project_event(event: Dictionary) -> void:
	match event.get("kind", ""):
		"battle_started": description_label.text = "[color=#b78a4b]BATTLE STARTED[/color] The card rail is dormant until the simulation names an active actor."
		"turn_started":
			var actor_id: String = event.subjects[0]
			active_actor_id = actor_id
			round_label.text = "DAY 18  •  16:40  •  ROUND %d" % event.payload.round
			set_card_state(actor_id, "FOCUSED")
			set_commands_enabled(actor_id == "character.heroine.betty")
		"command_accepted": description_label.text = "[color=#4fc7b4]ACCEPTED[/color] The simulation accepted %s." % event.payload.skill_id
		"actor_focused": set_card_state(event.subjects[0], "ACTIVE")
		"enemy_intent_declared":
			intent_label.text = "INTENT: RUSHING BITE → BETTY"
			description_label.text += " [color=#c24e45]INTENT[/color] It lowers its skull and commits to a straight rushing bite."
		"damage_applied":
			var target_id: String = event.subjects[-1]
			description_label.text += " The hit deals %d damage; %d vitality remains." % [event.payload.amount, event.payload.remaining_vitality]
			update_status_values(target_id, event.payload.remaining_vitality)
		"guard_changed":
			var guarded_name: String = actor_display_names.get(event.subjects[0], event.subjects[0])
			description_label.text += " %s gains %d Guard." % [guarded_name, event.payload.delta]
		"vitality_changed":
			var healed_name: String = actor_display_names.get(event.subjects[0], event.subjects[0])
			description_label.text += " %s restores %d Vitality and now has %d." % [healed_name, event.payload.delta, event.payload.total]
			update_status_values(event.subjects[0], event.payload.total)
		"actor_moved": description_label.text += " Betty crosses from band %d to band %d." % [event.payload.from_band, event.payload.to_band]
		"interception_set": description_label.text += " Betty takes position in front of Vix and will intercept the next attack aimed at her."
		"interception_triggered": description_label.text += " Betty receives the attack meant for Vix; the interception is now spent."
		"reaction_window_opened": description_label.text += " [color=#c24e45]LETHAL HIT DETECTED[/color] The simulation opens a reaction window."
		"reaction_triggered": description_label.text += " Betty breaks out of the card rail and triggers Fatal Intercept."
		"defeat_prevented": description_label.text += " The incoming hit is cancelled completely."
		"battlefield_effect_created": description_label.text += " Betty plants her mace and unrolls the Mobile Infirmary."
		"battlefield_effect_pulse": description_label.text += " [color=#4fc7b4]INFIRMARY PULSE[/color] Medicine and guard reach every living party member. %d pulse(s) remain." % event.payload.pulses_remaining_after
		"battlefield_effect_removed": description_label.text += " The Mobile Infirmary folds away: %s." % event.payload.reason
		"actor_revived":
			description_label.text += " [color=#4fc7b4]COMBAT REVIVAL[/color] Ayla returns with %d Vitality, zero Guard and no negative conditions." % event.payload.vitality
			update_status_values(event.subjects[0], event.payload.vitality)
		"bonus_turn_granted": description_label.text += " Ayla receives an immediate bonus turn; normal initiative will resume after it."
		"round_started": description_label.text += " [color=#b78a4b]ROUND %d[/color] Both survivors reset their stance." % event.payload.round
		"battle_ended":
			set_commands_enabled(false)
			intent_label.text = "VICTORY" if event.payload.victory else "DEFEAT — MIDNIGHT RETURN PENDING"
		"command_rejected": description_label.text = "[color=#c24e45]REJECTED[/color] %s" % event.payload.reason
		_: description_label.text += " [Unknown event: %s]" % event.get("kind", "missing")

func on_animation_beat(skill_id: String, _beat_index: int, beat: Dictionary) -> void:
	var readable_name := str(beat.get("name", "unnamed_beat")).replace("_", " ").to_upper()
	target_label.text = "%s  •  %s" % [skill_id.trim_prefix("skill.betty.").replace("_", " ").to_upper(), readable_name]

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
			break
	if id == "character.heroine.betty" and status_labels.has(id):
		var card := status_labels[id] as Button
		card.text = "BETTY\nFIELD MEDIC\nVIT %d/100  •  RESOLVING\n[DUMMY PORTRAIT]" % vitality
	elif status_labels.has(id):
		var card := status_labels[id] as Button
		var state := "DEFEATED" if vitality <= 0 else "READY"
		card.text = "%s\nVIT %d  •  %s\n[DUMMY PORTRAIT]" % [actor_display_names.get(id, id), vitality, state]
	elif id == "enemy.raptor.razorbeak.prototype":
		var stack := enemy_panel.get_child(0) as VBoxContainer
		(stack.get_child(2) as Label).text = "DUMMY ENEMY ACTOR\nVIT %d/70 • INDIVIDUAL THREAT" % vitality

func set_card_state(id: String, state: String) -> void:
	for card_id in card_buttons:
		var card := card_buttons[card_id] as Button
		card.modulate = Color.WHITE if card_id == id else Color("9aa5a2")
		if card_id == id:
			card.tooltip_text = "Card state: %s. Stable actor ID: %s" % [state, id]

func set_commands_enabled(enabled: bool) -> void:
	for skill_id in command_buttons:
		var button := command_buttons[skill_id] as Button
		button.disabled = not enabled or skill_id == "skill.betty.fatal_intercept"

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
