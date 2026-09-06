class_name BattlePrototype
extends Control

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")
const PaperDollScript = preload("res://scripts/battle/paper_doll.gd")
const PaperCardScript = preload("res://scripts/battle/paper_card.gd")
const PaperSkillDiamondScript = preload("res://scripts/battle/paper_skill_diamond.gd")
const BattleStageScript = preload("res://scripts/battle/battle_stage.gd")
const DamageNumberScript = preload("res://scripts/battle/damage_number.gd")
const PaletteScript = preload("res://scripts/battle/battle_palette.gd")
const CampaignEncounterSimulationPortScript = preload("res://scripts/simulation/campaign_encounter_simulation_port.gd")

## Presentation-only prototype. Gameplay truth comes through SimulationPort.
## All generated shapes and labels are explicit placeholders registered in
## content/art/placeholders.json.
##
## P6 moved every positional decision out of this file and into
## `battle_stage.gd`: the plate, the floor line, the five bands, the actors
## standing on them and the resolution of a VFX record's socket. What is left
## here is the screen -- the HUD, the rail, the command dock -- and the wiring
## from a simulation event to the stage beat that shows it.

## The palette has one owner. Until `game/themes/bronze_vellum.tres` exists,
## that owner is `battle_palette.gd`, and these six names are aliases of it so
## the rest of this file reads as it always did.
const BRONZE := PaletteScript.BRONZE
const DEEP := PaletteScript.DEEP
const PANEL := PaletteScript.PANEL
const CREAM := PaletteScript.CREAM
const TEAL := PaletteScript.TEAL
const DANGER := PaletteScript.DANGER

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
## Betty's seven authored commands, D through SSS, in rank order.
const BETTY_COMMANDS := [
	["skill.betty.guarded_strike", "D", "Guarded Strike"],
	["skill.betty.condition_cleanse", "C", "Condition Cleanse"],
	["skill.betty.rescue_charge", "B", "Rescue Charge"],
	["skill.betty.healing_impact", "A", "Healing Impact"],
	["skill.betty.fatal_intercept", "S", "Fatal Intercept"],
	["skill.betty.mobile_infirmary", "SS", "Mobile Infirmary"],
	["skill.betty.combat_revival", "SSS", "Combat Revival"]
]
## The rail's geometry. One card per actor the simulation reports, standing in
## its band's column; a card eases to its new column when the actor moves.
const CARD_SIZE := Vector2(196, 124)
const RAIL_ORIGIN := Vector2(34.0, 772.0)
const RAIL_COLUMN_WIDTH := 212.0
const RAIL_ROW_HEIGHT := 132.0
const RAIL_HEADING_HEIGHT := 20.0
const CARD_MOVE_SECONDS := 0.30
## The accessibility settings this screen honours, read from the same file the
## settings surface writes (B9 owns that surface; this screen owns reading it).
## `reduced_flash` is not a checkbox yet -- the surface offers text scale,
## contrast and motion -- so flashing follows reduced motion until it gets one,
## and the key is read here so that adding it needs no change to this file.
const SETTINGS_PATH := "user://settings.cfg"
const SETTINGS_SECTION := "accessibility"

var simulation: SimulationPort
var catalog := ContentCatalog.new()
var targeting := TargetingSession.new()
var animation_director: SkillAnimationDirector
var placeholder_presenter: Node
var text_renderer := CombatTextRenderer.new()
var stage: BattleStage
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
var band_rail: Control
var band_headings: Dictionary = {}
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
var reduced_motion := false
var reduced_flash := false
## The camera record of the beat that is currently framing the action, so the
## settle at the end of an action uses that record's own transition out.
var last_camera_record: Dictionary = {}
var defeated_actor_ids: Dictionary = {}
var actor_display_names := {
	"character.protagonist.captain": "MICHAEL CORRIGAN",
	"character.heroine.betty": "BETTY",
	"character.heroine.ayla": "AYLA",
	"character.heroine.vix": "VIX",
	"character.heroine.grisha": "GRISHA"
}

func _ready() -> void:
	load_accessibility_settings()
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


## Reduced motion is the surface's own setting. Reduced flash follows it until
## the settings surface offers flashing its own switch; the key is read either
## way, so the day it appears this screen already obeys it.
func load_accessibility_settings() -> void:
	var config := ConfigFile.new()
	if config.load(SETTINGS_PATH) != OK:
		return
	reduced_motion = bool(config.get_value(SETTINGS_SECTION, "reduced_motion", false))
	reduced_flash = bool(config.get_value(SETTINGS_SECTION, "reduced_flash", reduced_motion))


func build_screen() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(make_color_rect(PaletteScript.VOID, "BackgroundBase"))
	# The Bible's battle screen is one shared theatrical plane. The stage owns
	# the plate and everything standing on it; the HUD sits above and is never
	# scaled by the camera, because text that scales with a punch-in is text the
	# player cannot read at the moment it matters most.
	stage = BattleStageScript.new()
	stage.build(reduced_motion, reduced_flash)
	add_child(stage)
	var hud := Control.new()
	hud.name = "BattleHud"
	hud.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hud.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(hud)
	build_actors()
	build_hud(hud)


func build_actors() -> void:
	actor_panel = make_actor_placeholder("presentation.paper_doll.betty.active", TEAL)
	actor_panel.size = Vector2(520, 600)
	stage.add_body(BETTY_ID, actor_panel, actor_panel.get_meta("paper_doll"), "party_front")
	enemy_panel = make_actor_placeholder("presentation.paper_doll.razorbeak.active", DANGER)
	enemy_panel.size = Vector2(560, 460)
	enemy_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	enemy_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	enemy_panel.gui_input.connect(on_enemy_gui_input)
	stage.add_body("enemy.raptor.razorbeak", enemy_panel, enemy_panel.get_meta("paper_doll"), "enemy_front")


func build_hud(hud: Control) -> void:
	# The only persistent information at the top is information the player can
	# use: location/time and the current enemy intent. No ornamental meter.
	round_label = make_label("DAY 18  •  16:40", 16, CREAM)
	round_label.position = Vector2(38, 30)
	round_label.size = Vector2(280, 28)
	hud.add_child(round_label)
	description_label = RichTextLabel.new()
	description_label.name = "CombatDescription"
	description_label.bbcode_enabled = true
	description_label.fit_content = false
	description_label.add_theme_font_size_override("normal_font_size", 15)
	description_label.add_theme_color_override("default_color", CREAM)
	description_label.add_theme_color_override("font_outline_color", PaletteScript.VOID)
	description_label.add_theme_constant_override("outline_size", 5)
	description_label.text = "[color=#b78a4b]RECEPTION ROAD[/color]  •  ELVEN GATE  •  LATE AFTERNOON"
	description_label.position = Vector2(38, 58)
	description_label.size = Vector2(880, 64)
	description_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hud.add_child(description_label)
	intent_label = make_label("RAZORBEAK  •  RUSHING BITE  •  16 DAMAGE", 16, Color("f0938a"))
	intent_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	intent_label.position = Vector2(1120, 30)
	intent_label.size = Vector2(760, 30)
	hud.add_child(intent_label)
	action_cue_label = make_label("BETTY IS READY", 15, PaletteScript.GOLD)
	action_cue_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	action_cue_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	action_cue_label.position = Vector2(1120, 736)
	action_cue_label.size = Vector2(766, 28)
	hud.add_child(action_cue_label)
	# Every actor the simulation reports receives one card, standing in the band
	# the simulation puts it in. The prototype does not invent empty roster slots
	# and does not draw a band nobody occupies.
	band_rail = Control.new()
	band_rail.name = "BandRail"
	band_rail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	band_rail.position = RAIL_ORIGIN
	band_rail.size = Vector2(RAIL_COLUMN_WIDTH * BAND_ORDER.size(), 260)
	hud.add_child(band_rail)
	command_dock = build_command_dock()
	command_dock.position = Vector2(1120, 772)
	command_dock.size = Vector2(766, 284)
	hud.add_child(command_dock)
	return_to_expedition_button = Button.new()
	return_to_expedition_button.name = "ReturnToExpedition"
	return_to_expedition_button.text = "RETURN TO RECEPTION TERRACE"
	return_to_expedition_button.tooltip_text = "Return to the expedition after the authoritative encounter outcome has been recorded."
	return_to_expedition_button.add_theme_font_size_override("font_size", 15)
	return_to_expedition_button.add_theme_stylebox_override("normal", make_command_box(Color("1a322e"), TEAL))
	return_to_expedition_button.position = Vector2(1400, 676)
	return_to_expedition_button.size = Vector2(460, 72)
	return_to_expedition_button.visible = false
	return_to_expedition_button.pressed.connect(return_to_expedition)
	hud.add_child(return_to_expedition_button)


## One dock, whichever actor is commanding. Betty's seven rank diamonds and
## Michael's three verbs share it: two command surfaces in two places was the
## baseline screen's other legibility fault -- his grid was folded into a
## portrait card too small to read a word in.
func build_command_dock() -> Control:
	var dock := PanelContainer.new()
	dock.name = "CommandDock"
	dock.mouse_filter = Control.MOUSE_FILTER_PASS
	var box := StyleBoxFlat.new()
	box.bg_color = Color(PANEL.r, PANEL.g, PANEL.b, 0.82)
	box.border_color = Color(BRONZE, 0.55)
	box.set_border_width_all(2)
	box.set_corner_radius_all(12)
	box.content_margin_left = 16
	box.content_margin_right = 16
	box.content_margin_top = 10
	box.content_margin_bottom = 10
	dock.add_theme_stylebox_override("panel", box)
	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 4)
	dock.add_child(rows)
	target_label = make_label("BETTY  •  D-RANK", 14, BRONZE)
	rows.add_child(target_label)
	var first_row := HBoxContainer.new()
	first_row.alignment = BoxContainer.ALIGNMENT_CENTER
	first_row.add_theme_constant_override("separation", 6)
	var second_row := HBoxContainer.new()
	second_row.alignment = BoxContainer.ALIGNMENT_CENTER
	second_row.add_theme_constant_override("separation", 6)
	# Seven diamonds correspond exactly to Betty's D through SSS authored skill
	# records. Six are locked because their named milestones have not occurred;
	# they are future capabilities, not decorative empty inventory slots.
	for index in BETTY_COMMANDS.size():
		var command: Array = BETTY_COMMANDS[index]
		var gem := PaperSkillDiamondScript.new()
		gem.configure(command[0], command[1], command[2], index == 0)
		gem.command_pressed.connect(func(skill_id: String) -> void: begin_skill_targeting(skill_id))
		if index < 4:
			first_row.add_child(gem)
		else:
			second_row.add_child(gem)
		command_buttons[command[0]] = gem
	rows.add_child(first_row)
	rows.add_child(second_row)
	captain_command_grid = make_captain_command_grid()
	rows.add_child(captain_command_grid)
	return dock


## Rebuild the rail from the actors the simulation reports. Cards are grouped by
## the `band_name` the bridge sends, in `BAND_ORDER`; a band nobody stands in is
## not drawn. Cards are created once and eased to their column afterwards, so an
## actor crossing a band is a move the player can follow rather than a redraw.
func rebuild_band_rail() -> void:
	if band_rail == null:
		return
	var drawn := 0
	var seen: Dictionary = {}
	for band_index in BAND_ORDER.size():
		var band: String = str(BAND_ORDER[band_index])
		var occupants := actors_in_band(band)
		set_band_heading(band, band_index, not occupants.is_empty())
		for row in occupants.size():
			var actor: Dictionary = occupants[row]
			var id := str(actor.get("id", ""))
			seen[id] = true
			place_band_card(actor, band_index, row)
			drawn += 1
	for id in card_buttons.keys():
		if not seen.has(id):
			var stale := card_buttons[id] as Control
			card_buttons.erase(id)
			card_portraits.erase(id)
			status_labels.erase(id)
			if stale != null:
				stale.queue_free()
	if drawn != snapshot_actors.size():
		push_error("The simulation reported %d actors but only %d stand in a named band." % [snapshot_actors.size(), drawn])
	refresh_command_availability()


func actors_in_band(band: String) -> Array:
	var occupants: Array = []
	for actor in snapshot_actors:
		if str(actor.get("band_name", "")) == band:
			occupants.append(actor)
	return occupants


func set_band_heading(band: String, band_index: int, occupied: bool) -> void:
	if not band_headings.has(band):
		var heading := make_label(band.replace("_", " ").to_upper(), 12, BRONZE)
		heading.name = "Heading_" + band
		heading.mouse_filter = Control.MOUSE_FILTER_IGNORE
		heading.position = Vector2(RAIL_COLUMN_WIDTH * float(band_index), 0.0)
		heading.size = Vector2(CARD_SIZE.x, RAIL_HEADING_HEIGHT)
		band_rail.add_child(heading)
		band_headings[band] = heading
	(band_headings[band] as Label).visible = occupied


## Place one actor's card in its band's column. An existing card eases across;
## a new one appears where it belongs.
func place_band_card(actor: Dictionary, band_index: int, row: int) -> void:
	var id := str(actor.get("id", ""))
	var destination := Vector2(RAIL_COLUMN_WIDTH * float(band_index), RAIL_HEADING_HEIGHT + RAIL_ROW_HEIGHT * float(row))
	var holder := card_buttons.get(id) as Control
	if holder == null:
		holder = make_band_card(actor)
		holder.position = destination
		band_rail.add_child(holder)
	elif not holder.position.is_equal_approx(destination):
		if reduced_motion:
			holder.position = destination
		else:
			var tween := holder.create_tween()
			tween.set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_IN_OUT)
			tween.tween_property(holder, "position", destination, CARD_MOVE_SECONDS)
	var portrait := card_portraits.get(id) as PaperCard
	if portrait != null:
		portrait.configure(str(actor.get("display_name", id)).to_upper(), str(actor.get("band_name", "")), accent_for(actor), int(actor.get("composure", 0)), actor_is_shaken(actor), portrait_kind_for(actor))
	update_actor_status(actor)


func make_band_card(actor: Dictionary) -> Control:
	var id := str(actor.get("id", ""))
	var band := str(actor.get("band_name", ""))
	var display_name := str(actor.get("display_name", id)).to_upper()
	var holder := Control.new()
	holder.name = "Card_" + id.replace(".", "_")
	holder.size = CARD_SIZE
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	holder.tooltip_text = "PAPER PORTRAIT — DEVELOPMENT BLOCKOUT\nStable actor ID: %s\nBand: %s\nFuture portrait must preserve card crop, face, hair, outfit palette and readiness overlay." % [id, band]
	var portrait := PaperCardScript.new()
	portrait.name = "Portrait"
	portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	portrait.size = CARD_SIZE
	portrait.configure(display_name, band, accent_for(actor), int(actor.get("composure", 0)), actor_is_shaken(actor), portrait_kind_for(actor))
	holder.add_child(portrait)
	var copy := VBoxContainer.new()
	copy.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.position = Vector2(74, 16)
	copy.size = Vector2(CARD_SIZE.x - 82, CARD_SIZE.y - 52)
	copy.add_theme_constant_override("separation", 0)
	var name_label := make_label(display_name, 13, CREAM)
	name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	name_label.clip_text = true
	name_label.custom_minimum_size = Vector2(CARD_SIZE.x - 82, 18)
	copy.add_child(name_label)
	var status_label := make_label("", 11, Color("d7e0d7"))
	status_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	copy.add_child(status_label)
	holder.add_child(copy)
	card_buttons[id] = holder
	card_portraits[id] = portrait
	status_labels[id] = status_label
	return holder


## Captain Michael's command grid, unfolded in the shared dock when he is the
## active actor. Every entry is submitted through the same SimulationPort path
## Betty's commands use; the simulation remains the authority on whether it is
## legal.
func make_captain_command_grid() -> Control:
	var grid := HBoxContainer.new()
	grid.name = "CaptainCommandGrid"
	grid.visible = false
	grid.alignment = BoxContainer.ALIGNMENT_CENTER
	grid.add_theme_constant_override("separation", 8)
	for entry in CAPTAIN_COMMANDS:
		var skill_id: String = entry[0]
		var button := Button.new()
		button.name = "Command_" + skill_id.replace(".", "_")
		button.text = captain_command_label(skill_id, entry[1])
		button.custom_minimum_size = Vector2(190, 44)
		button.add_theme_font_size_override("font_size", 14)
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


func actor_is_hostile(actor_id: String) -> bool:
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == actor_id:
			return str(actor.get("faction", "")) == "hostile"
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
	panel.name = "Actor_" + spec_id.replace(".", "_")
	# The panel remains the interaction owner for targeting and animation. Its
	# background is explicitly transparent: a stage actor must not stand inside
	# a fake poster card.
	panel.add_theme_stylebox_override("panel", StyleBoxEmpty.new())
	var stack := VBoxContainer.new()
	stack.alignment = BoxContainer.ALIGNMENT_CENTER
	stack.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var doll := PaperDollScript.new()
	doll.name = "PaperDoll"
	doll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	doll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	doll.mouse_filter = Control.MOUSE_FILTER_IGNORE
	doll.configure(spec, accent)
	# The registry's native canvas is the rig's design grid, not a minimum size
	# on the screen: leaving it as a minimum forced every panel to at least
	# 520x560 and made the figures overpower the plate they stand on. The stage
	# sizes the panel; the rig scales itself to whatever it is given.
	doll.custom_minimum_size = Vector2.ZERO
	stack.add_child(doll)
	# Actor identity and state belong on their compact rail card and in the
	# useful battle labels. The stage itself shows bodies and action, not a
	# redundant poster title underneath each figure.
	panel.add_child(stack)
	panel.set_meta("paper_doll", doll)
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
		var id := str(actor.get("id", ""))
		if stage != null and stage.bodies.has(id):
			stage.bands[id] = str(actor.get("band_name", ""))
	rebuild_band_rail()


func project_event(event: Dictionary) -> void:
	var narration := text_renderer.render(event)
	apply_narration(narration)
	var kind := str(event.get("kind", ""))
	match kind:
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
			show_number(DamageNumberScript.DAMAGE, int(event.payload.get("amount", 0)), target_id, str(event.subjects[0]))
		"guard_changed":
			var guarded_id := str(event.subjects[0])
			update_guard_values(guarded_id, event.payload.total)
			# The site rule, made visible. The bridge writes `site_rule.round.<n>`
			# into the command id of the Guard a rule hands out at the round
			# boundary, and nothing else does; that is how the screen knows grave
			# watch is in force in this battle without inventing a snapshot key.
			if stage != null and BattleStageScript.is_site_rule_command(str(event.get("command_id", ""))):
				stage.pulse_site_rule(guarded_id)
			elif int(event.payload.get("delta", 0)) < 0:
				show_number(DamageNumberScript.GUARD, int(event.payload.delta), guarded_id, active_actor_id)
		"actor_moved":
			update_actor_band(event.subjects[0], int(event.payload.to_band))
		"vitality_changed":
			update_status_values(event.subjects[0], event.payload.total)
			if int(event.payload.get("delta", 0)) > 0:
				show_number(DamageNumberScript.HEAL, int(event.payload.delta), str(event.subjects[0]), active_actor_id)
		"recovery_opening_consumed":
			show_number(DamageNumberScript.PUNISH, int(event.payload.get("bonus_raw_damage", 0)), str(event.subjects[0]), str(event.subjects[-1]))
		"ward_line_placed":
			place_ward_line(int(event.payload.get("band", 0)))
		"ward_line_triggered":
			release_ward_line()
		"actor_defeated":
			var fallen := str(event.subjects[0])
			defeated_actor_ids[fallen] = true
			if stage != null:
				stage.dissolve_actor(fallen)
		"actor_revived":
			update_status_values(event.subjects[0], event.payload.vitality)
		"battle_ended":
			set_commands_enabled(false)
			set_captain_commands_enabled(false)
			command_dock.visible = false
			action_cue_label.visible = false
			if stage != null:
				stage.release_all_effects()
			intent_label.add_theme_color_override("font_color", TEAL if event.payload.victory else DANGER)
			intent_label.text = "RAZORBEAK WITHDRAWS" if event.payload.victory else "BETTY IS DOWN"
			description_label.text = "[color=#4fc7b4]The razorbeak breaks away into the jungle.[/color] The road is quiet for the moment." if event.payload.victory else "[color=#c24e45]Betty falls beneath the elven gate.[/color] The expedition must recover before midnight."
			if is_campaign_encounter:
				return_to_expedition_button.visible = true
		_: pass


func show_number(variant: String, amount: int, actor_id: String, from_actor_id: String) -> void:
	if stage != null and amount != 0:
		stage.show_number(variant, amount, actor_id, from_actor_id)


## Ayla's ward stands on a band boundary the bridge named. The line itself is
## `presentation.vfx.ayla.ward_line_drawn`, built by the same factory as every
## other effect, because the record owns what the effect looks like.
func place_ward_line(band: int) -> void:
	if stage == null:
		return
	stage.place_ward_line(catalog.get_registry_entry("presentation.vfx.ayla.ward_line_drawn"), band_name_for_index(band), BAND_ORDER)


func release_ward_line() -> void:
	if stage == null:
		return
	stage.spawn_effect(catalog.get_registry_entry("presentation.vfx.ayla.ward_line_break"), active_actor_id, active_actor_id, actor_is_hostile(active_actor_id))
	stage.release_effect("presentation.vfx.ayla.ward_line_drawn")


func apply_narration(narration: Dictionary) -> void:
	match narration.get("mode", "none"):
		"replace": description_label.text = narration.get("text", "")
		"append": description_label.text += narration.get("text", "")


func on_animation_beat(skill_id: String, _beat_index: int, beat: Dictionary) -> void:
	var readable_name := str(beat.get("name", "unnamed_beat")).replace("_", " ").to_upper()
	target_label.text = "%s  •  %s" % [skill_id.trim_prefix("skill.").replace(".", " ").replace("_", " ").to_upper(), readable_name]


func on_animation_event_cued(_skill_id: String, _beat_name: String, event: Dictionary) -> void:
	project_event(event)


## An authored cue arrives. The presenter still owns the rigs' reaction to it --
## pose, lunge, recoil -- and the stage owns the camera beat, the effect, the
## hit-stop and the shake. Both read the same cue; neither reads a skill id.
func on_presentation_cue(_skill_id: String, cue: Dictionary) -> void:
	var resolved := cue.duplicate(true)
	var camera_id := str(cue.get("cameraId", ""))
	resolved.cameraRecord = catalog.get_registry_entry(camera_id)
	var vfx_id := str(cue.get("vfxId", ""))
	resolved.vfxRecord = catalog.get_registry_entry(vfx_id)
	last_camera_record = resolved.cameraRecord
	placeholder_presenter.present(resolved)
	if stage != null:
		stage.play_cue(resolved, active_actor_id, primary_target_id(), actor_is_hostile(active_actor_id))
		var hit_stop := int(cue.get("hitStopMs", 0))
		if hit_stop > 0:
			await stage.camera.hit_stop(hit_stop, get_tree())
			stage.camera.hold()


## The actor a cue's effects point at: the target the player chose if there is
## one, otherwise the first living hostile, because a beat with no chosen target
## still has a direction on the floor.
func primary_target_id() -> String:
	if not targeting.selected_ids.is_empty():
		return str(targeting.selected_ids[0])
	for actor in snapshot_actors:
		var id := str(actor.get("id", ""))
		if id == active_actor_id or defeated_actor_ids.has(id):
			continue
		if str(actor.get("faction", "")) == ("party" if actor_is_hostile(active_actor_id) else "hostile"):
			return id
	return active_actor_id


func on_animation_finished(_skill_id: String) -> void:
	placeholder_presenter.reset()
	if stage != null:
		stage.settle_camera(last_camera_record)


func update_actor_status(actor: Dictionary) -> void:
	var id := str(actor.get("id", ""))
	if not status_labels.has(id):
		return
	var status := status_labels[id] as Label
	var state := "DEFEATED" if int(actor.get("vitality", 0)) <= 0 else "READY"
	status.text = "VIT %d/%d  •  GRD %d  •  %s" % [int(actor.get("vitality", 0)), int(actor.get("max_vitality", 0)), int(actor.get("guard", 0)), state]


## An actor changed band. The destination's name is the one the bridge gave for
## that stored index, so the card carries the simulation's own word for it, and
## the body on the stage walks to that band's place on the floor.
func update_actor_band(id: String, band: int) -> void:
	var destination := band_name_for_index(band)
	for actor in snapshot_actors:
		if str(actor.get("id", "")) == id:
			actor.band = band
			actor.band_name = destination
			break
	if stage != null and not destination.is_empty():
		stage.move_to_band(id, destination)
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
	# Every label on this screen sits over a painted plate whose brightness the
	# screen does not control. An outline is what keeps text legible over the
	# terrace stone and over the sky alike.
	label.add_theme_color_override("font_outline_color", PaletteScript.VOID)
	label.add_theme_constant_override("outline_size", 5)
	return label
