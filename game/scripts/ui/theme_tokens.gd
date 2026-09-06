class_name ThemeTokens
extends RefCounted

## The code-side door onto `res://themes/bronze_vellum.tres`.
##
## Card P5 asks for one owner of the palette. That owner is the Theme resource,
## not this script: every colour, every type-scale step and every StyleBox is
## read from the `.tres` and nothing here declares a hex. What this file adds is
## the vocabulary (the token names), the reader (`color`, `font_size`,
## `style`), and the one transformation the accessibility settings need
## (`build`, which returns a Theme with B9's text scale, high contrast and
## reduced motion already applied).
##
## The transformation is data-driven on purpose. `build` reads the base palette
## and the high-contrast palette out of the same resource, makes a colour-for-
## colour substitution map between them, and walks every colour and every
## StyleBox the Theme carries. So a StyleBox added to the `.tres` tomorrow is
## recoloured correctly without a line changing here, and there is no table in
## GDScript restating which box is painted in which token -- restating it is
## how the second owner would appear.

## The resource that owns the grammar.
const THEME_PATH := "res://themes/bronze_vellum.tres"

## Theme type names inside that resource. `PALETTE` is what every reader asks
## for; `HIGH_CONTRAST_PALETTE` is only ever read by `build`.
const PALETTE := "Palette"
const HIGH_CONTRAST_PALETTE := "PaletteHighContrast"
const TYPE_SCALE := "TypeScale"
const MOTION := "Motion"
const FONTS := "Fonts"

## The palette's token names, in the order the resource lists them. Names, not
## values: the values are in the `.tres`.
const TOKENS: PackedStringArray = [
	"deep", "panel", "panel_raised", "panel_sunken", "panel_hover", "panel_active",
	"bronze", "bronze_bright", "rule", "rule_faint", "teal", "teal_deep",
	"cream", "muted", "danger", "danger_soft", "focus", "ink",
]

## The type scale's steps. `mono` is the one that carries a stable ID.
const STEPS: PackedStringArray = ["display", "title", "body", "caption", "mono"]

## Text scale bounds. The settings panel clamps to these too; they are stated
## here because the Theme is what the scale is applied to.
const MINIMUM_TEXT_SCALE := 0.75
const MAXIMUM_TEXT_SCALE := 2.0

## The smallest font the scale may produce, so a 75% scale on the caption step
## still leaves a readable line rather than a smear.
const MINIMUM_FONT_SIZE := 9

static var _base: Theme = null


## The Theme actually in force for `control`: its own if it carries one,
## otherwise the nearest ancestor's, otherwise the resource as authored. Every
## component in `scripts/ui/` asks this way, so one assignment at the top of a
## screen reaches all of them and no component keeps a Theme of its own.
static func active(control: Control) -> Theme:
	var walker: Node = control
	while walker != null:
		if walker is Control and (walker as Control).theme != null:
			return (walker as Control).theme
		walker = walker.get_parent()
	return base_theme()


## The resource as authored: base palette, no scaling, no contrast swap. Cached,
## and never handed out for mutation -- `build` duplicates before it touches
## anything.
static func base_theme() -> Theme:
	if _base == null:
		_base = load(THEME_PATH) as Theme
	return _base


## The Theme a screen should actually use: the grammar with B9's three settings
## applied. Returns a fresh Theme every call, so two surfaces with different
## settings cannot corrupt each other, and assigning it to a Control's `theme`
## is the whole of "the settings changed the Theme live".
static func build(text_scale := 1.0, high_contrast := false, reduced_motion := false) -> Theme:
	var base := base_theme()
	if base == null:
		push_error("ThemeTokens: %s did not load as a Theme." % THEME_PATH)
		return Theme.new()
	var theme: Theme = base.duplicate(true)
	if high_contrast:
		_swap_palette(base, theme)
	_scale_type(theme, clampf(text_scale, MINIMUM_TEXT_SCALE, MAXIMUM_TEXT_SCALE))
	theme.set_constant("reduced_motion", MOTION, 1 if reduced_motion else 0)
	theme.set_constant("text_scale_percent", MOTION, roundi(clampf(text_scale, MINIMUM_TEXT_SCALE, MAXIMUM_TEXT_SCALE) * 100.0))
	return theme


## One palette token, by name. Falls back to the base resource when the caller
## has no Theme of its own, so a component drawn before it is parented still
## draws in the grammar rather than in a guessed colour.
static func color(theme: Theme, token: String) -> Color:
	var source := theme if theme != null and theme.has_color(token, PALETTE) else base_theme()
	if source == null or not source.has_color(token, PALETTE):
		push_error("ThemeTokens: no palette token named '%s'." % token)
		return Color.MAGENTA
	return source.get_color(token, PALETTE)


## One step of the type scale, already carrying the reader's text scale because
## `build` scaled the resource itself.
static func font_size(theme: Theme, step: String) -> int:
	var source := theme if theme != null and theme.has_font_size(step, TYPE_SCALE) else base_theme()
	if source == null or not source.has_font_size(step, TYPE_SCALE):
		push_error("ThemeTokens: no type-scale step named '%s'." % step)
		return MINIMUM_FONT_SIZE
	return source.get_font_size(step, TYPE_SCALE)


## One StyleBox from the grammar. `theme_type` and `style` name a slot the
## `.tres` fills, e.g. ("VellumCard", "selected").
static func style(theme: Theme, theme_type: String, style_name: String) -> StyleBox:
	var source := theme if theme != null and theme.has_stylebox(style_name, theme_type) else base_theme()
	if source == null or not source.has_stylebox(style_name, theme_type):
		push_error("ThemeTokens: no style '%s' on type '%s'." % [style_name, theme_type])
		return StyleBoxEmpty.new()
	return source.get_stylebox(style_name, theme_type)


## The monospace face stable IDs are set in. Null when the platform offers no
## monospace family, in which case a caller draws in the default font -- an ID
## in the wrong face is still readable, and a missing font is not an error.
static func mono_font(theme: Theme) -> Font:
	var source := theme if theme != null and theme.has_font("mono", FONTS) else base_theme()
	if source == null or not source.has_font("mono", FONTS):
		return null
	return source.get_font("mono", FONTS)


## Whether motion is to be suppressed. Components ask this instead of reading
## the settings file, so one answer reaches every animation.
static func reduced_motion(theme: Theme) -> bool:
	var source := theme if theme != null and theme.has_constant("reduced_motion", MOTION) else base_theme()
	if source == null or not source.has_constant("reduced_motion", MOTION):
		return false
	return source.get_constant("reduced_motion", MOTION) != 0


## Replace every base-palette colour in `theme` with its high-contrast twin.
##
## The substitution is by value: whatever `deep` is in the base palette becomes
## whatever `deep` is in the high-contrast palette, wherever it appears --- in a
## Label's font colour, in a StyleBox's fill, in a border. That is why no table
## of "which box is painted in which token" exists in this file.
static func _swap_palette(base: Theme, theme: Theme) -> void:
	var substitution := {}
	for token in TOKENS:
		if base.has_color(token, PALETTE) and base.has_color(token, HIGH_CONTRAST_PALETTE):
			substitution[base.get_color(token, PALETTE)] = base.get_color(token, HIGH_CONTRAST_PALETTE)
	for theme_type in theme.get_color_type_list():
		if theme_type == PALETTE or theme_type == HIGH_CONTRAST_PALETTE:
			continue
		for color_name in theme.get_color_list(theme_type):
			var existing := theme.get_color(color_name, theme_type)
			if substitution.has(existing):
				theme.set_color(color_name, theme_type, substitution[existing])
	for theme_type in theme.get_stylebox_type_list():
		for style_name in theme.get_stylebox_list(theme_type):
			var box := theme.get_stylebox(style_name, theme_type) as StyleBoxFlat
			if box == null:
				continue
			if substitution.has(box.bg_color):
				box.bg_color = substitution[box.bg_color]
			if substitution.has(box.border_color):
				box.border_color = substitution[box.border_color]
	# The palette itself moves last, so the walk above compared against the
	# colours that were actually in force while it ran.
	for token in TOKENS:
		if base.has_color(token, HIGH_CONTRAST_PALETTE):
			theme.set_color(token, PALETTE, base.get_color(token, HIGH_CONTRAST_PALETTE))


## Multiply every font size the Theme carries. Every one, including the ones
## Godot's own controls read, so a scaled Theme scales a Button's label as
## surely as it scales a title.
static func _scale_type(theme: Theme, scale: float) -> void:
	theme.default_font_size = maxi(MINIMUM_FONT_SIZE, roundi(float(theme.default_font_size) * scale))
	for theme_type in theme.get_font_size_type_list():
		for size_name in theme.get_font_size_list(theme_type):
			var scaled := roundi(float(theme.get_font_size(size_name, theme_type)) * scale)
			theme.set_font_size(size_name, theme_type, maxi(MINIMUM_FONT_SIZE, scaled))
