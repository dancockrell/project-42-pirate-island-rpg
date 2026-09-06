class_name SettingsPanel
extends Control

## The accessibility and settings surface, and the first surface that pauses.
##
## Brief section 14: full-screen management, reading and accessibility
## interfaces pause by default, because the player must not be punished for
## reading slowly or studying the board. This panel therefore takes a GamePause
## reason while it is open and releases it when it closes; if a reading surface
## is open too, the game stays paused until both are gone.
##
## Three settings, persisted to `user://settings.cfg` through ConfigFile. They
## are stored and read back here and nowhere else yet: the B9 card asks for the
## surface and the persistence, and deliberately does not ask for text scale,
## contrast and motion to be threaded through every scene. When a screen is
## ready to honour one, it reads it from this panel's loaded values rather than
## keeping a second copy.

## The reason this surface gives GamePause while it is open. The surface owns
## the name, not the pause service: GamePause counts reasons and never learns
## what any of them is.
const PAUSE_REASON := "settings"

const SETTINGS_PATH := "user://settings.cfg"
const SECTION := "accessibility"

const MINIMUM_TEXT_SCALE := 0.75
const MAXIMUM_TEXT_SCALE := 2.0
const DEFAULT_TEXT_SCALE := 1.0

# The expedition shell's bronze-and-vellum grammar. Every screen in this project
# states its own palette (the battle prototype and the expedition prototype each
# do, and disagree deliberately on mood); there is no palette owner to reuse
# yet, and inventing one is B13's information-surface work, not this card's.
const DEEP := Color("081211")
const PANEL := Color("132321")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("9eb0a7")

var text_scale := DEFAULT_TEXT_SCALE
var high_contrast := false
var reduced_motion := false

var game_pause: Node
var text_scale_slider: HSlider
var text_scale_value: Label
var high_contrast_check: CheckBox
var reduced_motion_check: CheckBox
var close_button: Button


func _ready() -> void:
	# A settings panel that stopped processing when it paused the game could
	# never be closed again.
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	load_settings()
	build_screen()
	game_pause = get_node_or_null("/root/GamePause")
	if game_pause != null:
		game_pause.pause(PAUSE_REASON)


func _exit_tree() -> void:
	release_pause()


## Closes the surface: the pause reason is released first, then the panel goes.
func close() -> void:
	save_settings()
	release_pause()
	queue_free()


func release_pause() -> void:
	if game_pause == null:
		return
	game_pause.resume(PAUSE_REASON)


func load_settings() -> void:
	var config := ConfigFile.new()
	if config.load(SETTINGS_PATH) != OK:
		return
	set_text_scale(float(config.get_value(SECTION, "text_scale", DEFAULT_TEXT_SCALE)))
	high_contrast = bool(config.get_value(SECTION, "high_contrast", false))
	reduced_motion = bool(config.get_value(SECTION, "reduced_motion", false))


func save_settings() -> Error:
	var config := ConfigFile.new()
	config.set_value(SECTION, "text_scale", text_scale)
	config.set_value(SECTION, "high_contrast", high_contrast)
	config.set_value(SECTION, "reduced_motion", reduced_motion)
	return config.save(SETTINGS_PATH)


## Text scale is clamped rather than refused: a settings file edited by hand, or
## written by an older build, must never leave the game unreadable.
func set_text_scale(value: float) -> void:
	text_scale = clampf(value, MINIMUM_TEXT_SCALE, MAXIMUM_TEXT_SCALE)
	if text_scale_slider != null and not is_equal_approx(text_scale_slider.value, text_scale):
		text_scale_slider.value = text_scale
	if text_scale_value != null:
		text_scale_value.text = "%d%%" % roundi(text_scale * 100.0)


func set_high_contrast(value: bool) -> void:
	high_contrast = value
	if high_contrast_check != null and high_contrast_check.button_pressed != value:
		high_contrast_check.button_pressed = value


func set_reduced_motion(value: bool) -> void:
	reduced_motion = value
	if reduced_motion_check != null and reduced_motion_check.button_pressed != value:
		reduced_motion_check.button_pressed = value


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
	page.add_child(make_label("SETTINGS AND ACCESSIBILITY", 24, BRONZE))
	page.add_child(make_label("The island is held while this page is open. Read at your own pace.", 14, MUTED))

	var card := PanelContainer.new()
	card.name = "SettingsCard"
	card.add_theme_stylebox_override("panel", make_panel_box())
	page.add_child(card)

	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 14)
	card.add_child(rows)

	var text_row := HBoxContainer.new()
	text_row.add_theme_constant_override("separation", 12)
	rows.add_child(text_row)
	text_row.add_child(make_label("TEXT SCALE", 16, CREAM))
	text_scale_slider = HSlider.new()
	text_scale_slider.name = "TextScale"
	text_scale_slider.min_value = MINIMUM_TEXT_SCALE
	text_scale_slider.max_value = MAXIMUM_TEXT_SCALE
	text_scale_slider.step = 0.05
	text_scale_slider.custom_minimum_size = Vector2(260, 28)
	text_scale_slider.value = text_scale
	text_scale_slider.value_changed.connect(set_text_scale)
	text_row.add_child(text_scale_slider)
	text_scale_value = make_label("%d%%" % roundi(text_scale * 100.0), 16, TEAL)
	text_scale_value.name = "TextScaleValue"
	text_row.add_child(text_scale_value)

	high_contrast_check = make_check("HighContrast", "HIGH CONTRAST", high_contrast)
	high_contrast_check.toggled.connect(set_high_contrast)
	rows.add_child(high_contrast_check)

	reduced_motion_check = make_check("ReducedMotion", "REDUCED MOTION", reduced_motion)
	reduced_motion_check.toggled.connect(set_reduced_motion)
	rows.add_child(reduced_motion_check)

	close_button = Button.new()
	close_button.name = "CloseSettings"
	close_button.text = "CLOSE  •  RESUME THE ISLAND"
	close_button.custom_minimum_size.y = 48
	close_button.add_theme_font_size_override("font_size", 16)
	close_button.add_theme_stylebox_override("normal", make_route_box(Color("1d2a24"), BRONZE))
	close_button.add_theme_stylebox_override("hover", make_route_box(Color("2a3d33"), TEAL))
	close_button.pressed.connect(close)
	page.add_child(close_button)


func make_check(node_name: String, text: String, pressed: bool) -> CheckBox:
	var check_box := CheckBox.new()
	check_box.name = node_name
	check_box.text = text
	check_box.button_pressed = pressed
	check_box.add_theme_font_size_override("font_size", 16)
	check_box.add_theme_color_override("font_color", CREAM)
	return check_box


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
