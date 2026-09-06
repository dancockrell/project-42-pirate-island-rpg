class_name ShellStyle
extends RefCounted

## How the shell says a thing in the bronze-and-vellum grammar.
##
## Card P11: this file used to restate the seven prototype tones and the two
## shell-only ones, which made it a second owner of the palette beside
## `game/themes/bronze_vellum.tres`. It declares no colour now. The seven were
## already in the Theme; the two that were not (`night`, `hairline`) were added
## to it, and every value below is read back out of the Theme through
## `ThemeTokens`. What is left here is the shell's small vocabulary of shapes --
## a menu row's bar, a framed card, a hairline rule, a display line's tracking
## -- and each of them takes the Theme it is to be drawn in.
##
## A screen calls `ThemeTokens.adopt(self)` once and the Theme reaches every
## Label and every `MenuRow` Button below it without another line of code, which
## is why B9's text scale and high contrast now reach the title and the pause
## menu at all.

const ThemeTokensScript := preload("res://scripts/ui/theme_tokens.gd")

## Theme type variations the shell writes in, so a screen names a role rather
## than a size and a colour. All seven live in the `.tres`.
const WORDMARK := &"WordmarkLabel"
const DISPLAY := &"DisplayLabel"
const TITLE := &"TitleLabel"
const SUBTITLE := &"SubtitleLabel"
const BODY := &"BodyLabel"
const CAPTION := &"CaptionLabel"
const ACCENT := &"AccentLabel"
const MUTED := &"MutedLabel"
const BRONZE := &"BronzeLabel"
const DANGER := &"DangerLabel"
const FAINT := &"FaintLabel"
## The Button variation that carries the shell's menu row: no fill, a bronze bar
## down the left edge that moves and lights on focus. Its five StyleBoxes are in
## the Theme, so high contrast repaints them with no code here.
const MENU_ROW := &"MenuRow"


## A menu row's box in a bar colour the Theme does not have a variation for --
## the destructive Confirm, the Cancel beside it, the row for a slot that will
## not load. The standard row uses the `MenuRow` variation instead and builds
## nothing.
static func menu_box(bar: Color, fill: Color) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = fill
	box.border_color = bar
	box.border_width_left = 4
	box.content_margin_left = 22
	box.content_margin_right = 18
	box.content_margin_top = 12
	box.content_margin_bottom = 12
	return box


## A menu row whose bar is a named palette token rather than the standard
## bronze: the destructive Confirm in danger, the Cancel beside it in muted, the
## row for a slot that will not load. A StyleBox built in code is the one thing
## the Theme cannot restyle where it stands, so the token is remembered on the
## button and `restyle_bar_buttons` puts it back after a rebuild.
static func bar_button(button: Button, theme: Theme, token: String) -> Button:
	button.set_meta("bar_token", token)
	var bar := ThemeTokensScript.color(theme, token)
	button.add_theme_font_size_override("font_size", ThemeTokensScript.font_size(theme, "subtitle"))
	button.add_theme_color_override("font_color", ThemeTokensScript.color(theme, "cream"))
	button.add_theme_color_override("font_focus_color", ThemeTokensScript.color(theme, "teal"))
	button.add_theme_color_override("font_hover_color", ThemeTokensScript.color(theme, "teal"))
	button.add_theme_color_override("font_disabled_color", ThemeTokensScript.color(theme, "hairline"))
	button.add_theme_stylebox_override("normal", menu_box(bar, Color(bar, 0.0)))
	button.add_theme_stylebox_override("hover", menu_box(bar, Color(bar, 0.12)))
	button.add_theme_stylebox_override("focus", menu_box(ThemeTokensScript.color(theme, "focus"), Color(bar, 0.16)))
	button.add_theme_stylebox_override("pressed", menu_box(bar, Color(bar, 0.2)))
	button.add_theme_stylebox_override("disabled", menu_box(ThemeTokensScript.color(theme, "hairline"), Color(bar, 0.0)))
	return button


## Put every bar button under `root` back in the grammar after a rebuild.
static func restyle_bar_buttons(root: Node, theme: Theme) -> void:
	for node in root.find_children("*", "Button", true, false):
		var button := node as Button
		if button != null and button.has_meta("bar_token"):
			bar_button(button, theme, str(button.get_meta("bar_token")))


## A framed card: the panel fill inside a bronze hairline. The Theme's
## `VellumPanel` is the same shape and a PanelContainer gets it for free; this
## exists for the two places the shell needs the box itself.
static func card_box(theme: Theme) -> StyleBoxFlat:
	return ThemeTokensScript.style(theme, "VellumPanel", "panel")


## A label in a named role. It carries no size and no colour of its own: the
## variation is the whole of its appearance, so the Theme restyles it where it
## stands when the player moves the text-scale slider.
static func label(text: String, variation: StringName) -> Label:
	var made := Label.new()
	made.text = text
	made.theme_type_variation = variation
	return made


## A hairline rule in a palette token. Godot has no thin separator that takes a
## colour without a Theme, so the shell draws one as a one-pixel ColorRect -- and
## because a ColorRect's colour is a property rather than a theme item, the
## screen that owns it repaints it in `_on_theme_rebuilt`.
static func rule(theme: Theme, token: String, width: float, thickness: float = 1.0) -> ColorRect:
	var made := ColorRect.new()
	made.color = ThemeTokensScript.color(theme, token)
	made.set_meta("palette_token", token)
	made.custom_minimum_size = Vector2(width, thickness)
	made.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return made


## Repaint every ColorRect under `root` that `rule` (or a screen) marked with a
## palette token. One call in `_on_theme_rebuilt` puts the whole screen's flat
## fills back in the grammar, high contrast included.
static func repaint_marked(root: Node, theme: Theme) -> void:
	for node in root.find_children("*", "ColorRect", true, false):
		var rect := node as ColorRect
		if rect == null or not rect.has_meta("palette_token"):
			continue
		var alpha := float(rect.get_meta("palette_alpha", 1.0))
		var painted := ThemeTokensScript.color(theme, str(rect.get_meta("palette_token")))
		rect.color = Color(painted, alpha)


## Mark a ColorRect the screen made itself, so `repaint_marked` finds it.
static func paint(rect: ColorRect, theme: Theme, token: String, alpha := 1.0) -> ColorRect:
	rect.set_meta("palette_token", token)
	rect.set_meta("palette_alpha", alpha)
	rect.color = Color(ThemeTokensScript.color(theme, token), alpha)
	return rect


## Display lettering, spaced out by hand.
##
## Godot's Label has no tracking and the project admits no display font -- a
## font is an asset with a licence, and nothing may enter this tree without a
## provenance record -- so the shell's one piece of display typography is the
## default face opened up with figure spaces. It is a typographic choice made
## in text rather than an asset, which is why it can be made at all today.
static func tracked(text: String) -> String:
	var out := ""
	for index in text.length():
		if index > 0:
			out += " "
		out += text[index]
	return out
