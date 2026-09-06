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
## Three settings, persisted to `user://settings.cfg` through ConfigFile. **P5
## wired them.** Every change is handed to `InformationSurface.apply_settings`,
## which rebuilds `themes/bronze_vellum.tres` with the text scale multiplied
## through the type scale, the high-contrast token set swapped in, and the
## reduced-motion constant set; this panel then wears the result immediately, so
## the player sees the setting they are choosing while they choose it. B9's
## "recorded, not wired" note is closed by that path.
##
## The panel declares no colour and no font size of its own. The Theme is the
## one owner of both after P5; the hexes this file used to carry were moved into
## the resource unchanged.

## The reason this surface gives GamePause while it is open. The surface owns
## the name, not the pause service: GamePause counts reasons and never learns
## what any of them is.
const PAUSE_REASON := "settings"

const SETTINGS_PATH := "user://settings.cfg"
const SECTION := "accessibility"

## The readable range, owned by `ThemeTokens` because the Theme is what a scale
## is applied to. Named here as well so the constant callers already use keeps
## working and cannot drift from the one the Theme enforces.
const MINIMUM_TEXT_SCALE := ThemeTokens.MINIMUM_TEXT_SCALE
const MAXIMUM_TEXT_SCALE := ThemeTokens.MAXIMUM_TEXT_SCALE
const DEFAULT_TEXT_SCALE := 1.0

## The settings card is a column of controls, not a banner: it stops at a
## readable measure however wide the window is.
const CARD_WIDTH := 860
const CLOSE_BUTTON_WIDTH := 360

var text_scale := DEFAULT_TEXT_SCALE
var high_contrast := false
var reduced_motion := false

var game_pause: Node
var information_surface: Node
var text_scale_slider: HSlider
var text_scale_value: Label
var high_contrast_check: CheckBox
var reduced_motion_check: CheckBox
var close_button: Button

var _background: ColorRect


func _ready() -> void:
	# A settings panel that stopped processing when it paused the game could
	# never be closed again.
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	information_surface = get_node_or_null("/root/InformationSurface")
	load_settings()
	build_screen()
	apply_to_theme()
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
	set_high_contrast(bool(config.get_value(SECTION, "high_contrast", false)))
	set_reduced_motion(bool(config.get_value(SECTION, "reduced_motion", false)))


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
	apply_to_theme()


func set_high_contrast(value: bool) -> void:
	high_contrast = value
	if high_contrast_check != null and high_contrast_check.button_pressed != value:
		high_contrast_check.button_pressed = value
	apply_to_theme()


func set_reduced_motion(value: bool) -> void:
	reduced_motion = value
	if reduced_motion_check != null and reduced_motion_check.button_pressed != value:
		reduced_motion_check.button_pressed = value
	apply_to_theme()


## Rebuild the grammar from the three settings and wear it. The rebuilt Theme
## goes to `InformationSurface` when the autoload is present, so every other
## screen changes at the same instant; without it the panel still restyles
## itself, which is what a review scene and a suite need.
func apply_to_theme() -> void:
	var built: Theme
	if information_surface != null:
		built = information_surface.apply_settings(text_scale, high_contrast, reduced_motion)
	else:
		built = ThemeTokens.build(text_scale, high_contrast, reduced_motion)
	theme = built
	if _background != null:
		_background.color = ThemeTokens.color(built, "deep")


func build_screen() -> void:
	_background = ColorRect.new()
	_background.name = "Ground"
	_background.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_background.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_background)

	var frame := MarginContainer.new()
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 96)
	frame.add_theme_constant_override("margin_right", 96)
	frame.add_theme_constant_override("margin_top", 72)
	frame.add_theme_constant_override("margin_bottom", 56)
	add_child(frame)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 20)
	frame.add_child(page)
	page.add_child(make_label("Settings and accessibility", "DisplayLabel"))
	page.add_child(make_label("The island is held while this page is open. Read at your own pace.", "MutedLabel"))

	var card := VellumPanel.new()
	card.name = "SettingsCard"
	card.heading = "Reading and motion"
	card.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	card.custom_minimum_size.x = CARD_WIDTH
	page.add_child(card)
	var rows := card.body()
	rows.add_theme_constant_override("separation", 16)

	var text_row := HBoxContainer.new()
	text_row.add_theme_constant_override("separation", 16)
	rows.add_child(text_row)
	var text_caption := make_label("TEXT SCALE", "BodyLabel")
	text_caption.custom_minimum_size.x = 220
	text_row.add_child(text_caption)
	text_scale_slider = HSlider.new()
	text_scale_slider.name = "TextScale"
	text_scale_slider.min_value = MINIMUM_TEXT_SCALE
	text_scale_slider.max_value = MAXIMUM_TEXT_SCALE
	text_scale_slider.step = 0.05
	text_scale_slider.custom_minimum_size = Vector2(360, 24)
	text_scale_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text_scale_slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	text_scale_slider.value = text_scale
	text_scale_slider.value_changed.connect(set_text_scale)
	text_row.add_child(text_scale_slider)
	text_scale_value = make_label("%d%%" % roundi(text_scale * 100.0), "AccentLabel")
	text_scale_value.name = "TextScaleValue"
	text_scale_value.custom_minimum_size.x = 72
	text_scale_value.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	text_row.add_child(text_scale_value)

	high_contrast_check = make_check("HighContrast", "HIGH CONTRAST", high_contrast)
	high_contrast_check.toggled.connect(set_high_contrast)
	rows.add_child(high_contrast_check)

	reduced_motion_check = make_check("ReducedMotion", "REDUCED MOTION", reduced_motion)
	reduced_motion_check.toggled.connect(set_reduced_motion)
	rows.add_child(reduced_motion_check)

	# A live specimen of what the choices do. It is a real `VellumJournalEntry`
	# and a real body line wearing the Theme this panel just built, so the
	# player is looking at the interface rather than at a description of it.
	var preview_rule := Panel.new()
	preview_rule.name = "PreviewRule"
	preview_rule.custom_minimum_size = Vector2(0, 1)
	preview_rule.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var hairline := StyleBoxFlat.new()
	hairline.bg_color = ThemeTokens.color(ThemeTokens.active(self), "rule_faint")
	preview_rule.add_theme_stylebox_override("panel", hairline)
	rows.add_child(preview_rule)
	rows.add_child(make_label("HOW IT WILL READ", "CaptionLabel"))
	var preview := VellumJournalEntry.new()
	preview.name = "Preview"
	rows.add_child(preview)
	preview.configure({
		"day": 12, "hour": 11, "level": InformationLevel.Level.NOTABLE,
		"prose": "A force of faction.pirates reached world.cell.black_beach.landing.",
	})

	var spacer := Control.new()
	spacer.size_flags_vertical = Control.SIZE_EXPAND_FILL
	page.add_child(spacer)

	var footer := HBoxContainer.new()
	footer.name = "Footer"
	footer.add_theme_constant_override("separation", 24)
	page.add_child(footer)
	var note := make_label(
		"These three are read through the Theme, so every screen changes as you choose them.",
		"CaptionLabel")
	note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	note.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	footer.add_child(note)

	close_button = Button.new()
	close_button.name = "CloseSettings"
	close_button.text = "CLOSE  ·  RESUME THE ISLAND"
	close_button.custom_minimum_size = Vector2(CLOSE_BUTTON_WIDTH, 52)
	close_button.pressed.connect(close)
	footer.add_child(close_button)


func make_check(node_name: String, text: String, pressed: bool) -> CheckBox:
	var check_box := CheckBox.new()
	check_box.name = node_name
	check_box.text = text
	check_box.button_pressed = pressed
	return check_box


## A label in the grammar. It names a Theme *variation* -- `DisplayLabel`,
## `BodyLabel`, `MutedLabel` -- rather than a size and a colour, so a rebuilt
## Theme restyles it where it stands and the panel never restates the type
## scale.
func make_label(text: String, variation: String) -> Label:
	var label := Label.new()
	label.text = text
	label.theme_type_variation = variation
	return label
