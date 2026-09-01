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
var active_actor_id := "character.heroine.betty"
var description_label: RichTextLabel
var actor_panel: PanelContainer
var actor_name: Label
var enemy_panel: PanelContainer
var card_buttons: Dictionary = {}
var status_labels: Dictionary = {}
var intent_label: Label
var round_label: Label
var command_buttons: Array[Button] = []

func _ready() -> void:
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
	var commands := HBoxContainer.new()
	commands.custom_minimum_size.y = 94
	commands.alignment = BoxContainer.ALIGNMENT_CENTER
	commands.add_theme_constant_override("separation", 16)
	for command in [
		["skill.betty.guarded_strike", "GUARDED STRIKE", true],
		["skill.betty.condition_cleanse", "CONDITION CLEANSE", true],
		["skill.betty.rescue_charge", "RESCUE CHARGE", false],
		["skill.betty.healing_impact", "HEALING IMPACT", false]
	]:
		var button := Button.new()
		button.text = command[1] if command[2] else "%s\n[LOCKED IN PROTOTYPE]" % command[1]
		button.custom_minimum_size = Vector2(260, 70)
		button.add_theme_font_size_override("font_size", 17)
		button.disabled = not command[2]
		button.tooltip_text = "Stable skill ID: %s" % command[0]
		button.pressed.connect(func() -> void: submit_skill(command[0]))
		commands.add_child(button)
		command_buttons.append(button)
	return commands

func make_party_card(id: String, display_name: String, role: String, accent: Color) -> Button:
	var card := Button.new()
	card.name = display_name + "Card"
	card.custom_minimum_size = Vector2(280, 150)
	card.text = "%s\n%s\nVIT 84/100  •  READY\n[DUMMY PORTRAIT]" % [display_name, role]
	card.tooltip_text = "Placeholder portrait. Stable actor ID: %s" % id
	card.add_theme_color_override("font_color", CREAM)
	card.add_theme_color_override("font_hover_color", Color.WHITE)
	card.add_theme_color_override("font_pressed_color", accent)
	card.pressed.connect(func() -> void: select_actor(id, display_name, accent))
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
	active_actor_id = id
	var stack := actor_panel.get_child(0) as VBoxContainer
	(stack.get_child(0) as ColorRect).color = Color(accent, 0.52)
	(stack.get_child(1) as Label).text = display_name
	description_label.text = "[color=#b78a4b]FOCUS[/color] %s leaves the card rail and occupies the active battle plane. Simulation state is unchanged until a command is accepted." % display_name

func submit_skill(skill_id: String) -> void:
	var command := {
		"command_id": "command.debug.%s" % Time.get_ticks_msec(),
		"battle_id": "battle.prototype.returning_names",
		"actor_id": active_actor_id,
		"kind": "use_skill",
		"skill_id": skill_id,
		"target_ids": ["character.heroine.betty"] if skill_id == "skill.betty.condition_cleanse" else ["enemy.raptor.razorbeak.prototype"],
		"payload": {}
	}
	for event in simulation.submit(command):
		project_event(event)

func project_snapshot(snapshot: Dictionary) -> void:
	description_label.text = "[color=#b78a4b]OBSERVED[/color] %s" % snapshot.get("description", "No description supplied.")
	for actor in snapshot.get("actors", []):
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
		"guard_changed": description_label.text += " Betty gains %d Guard." % event.payload.delta
		"vitality_changed":
			description_label.text += " Betty restores %d Vitality and now has %d." % [event.payload.delta, event.payload.total]
			update_status_values(event.subjects[0], event.payload.total)
		"round_started": description_label.text += " [color=#b78a4b]ROUND %d[/color] Both survivors reset their stance." % event.payload.round
		"battle_ended":
			set_commands_enabled(false)
			intent_label.text = "VICTORY" if event.payload.victory else "DEFEAT — MIDNIGHT RETURN PENDING"
		"command_rejected": description_label.text = "[color=#c24e45]REJECTED[/color] %s" % event.payload.reason
		_: description_label.text += " [Unknown event: %s]" % event.get("kind", "missing")

func update_actor_status(actor: Dictionary) -> void:
	var id: String = actor.id
	if status_labels.has(id):
		var card := status_labels[id] as Button
		card.text = "BETTY\nFIELD MEDIC\nVIT %d/%d  •  GRD %d\n[DUMMY PORTRAIT]" % [actor.vitality, actor.max_vitality, actor.guard]

func update_status_values(id: String, vitality: int) -> void:
	if id == "character.heroine.betty" and status_labels.has(id):
		var card := status_labels[id] as Button
		card.text = "BETTY\nFIELD MEDIC\nVIT %d/100  •  RESOLVING\n[DUMMY PORTRAIT]" % vitality
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
	for index in command_buttons.size():
		command_buttons[index].disabled = not enabled or index > 1

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
