class_name ShellStyle
extends RefCounted

## The bronze-and-vellum grammar the shell draws with.
##
## These are the constants the two prototypes and the settings panel already
## carry (`DEEP`, `PANEL`, `BRONZE`, `TEAL`, `CREAM`, `MUTED`, `DANGER`),
## gathered here so the title, the pause menu and the credits state them once
## instead of three times. They are stated in ONE place on purpose and they are
## not the palette's owner: `game/themes/bronze_vellum.tres` (card P5) becomes
## that owner, and when it lands this file's values are deleted and every
## reference here resolves through the Theme instead. Nothing new is invented
## here beyond the two shell-only tones marked below.

const DEEP := Color("081211")
const PANEL := Color("132321")
const BRONZE := Color("b78a4b")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const MUTED := Color("9eb0a7")
const DANGER := Color("c24e45")

## Two tones the shell needs and the prototypes have no name for: the ground a
## full-screen menu sits on, which is darker than DEEP so a screen behind it
## reads as held rather than merely dimmed, and the hairline the shell rules
## its columns with. Both belong to the Theme when P5 lands.
const NIGHT := Color("050d0c")
const HAIRLINE := Color("2a3d33")


## A menu row: no fill, a bronze bar down its left edge, cream text. The bar is
## what moves on focus, so keyboard, gamepad and mouse all show the same thing
## in the same place.
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


## A framed card: the panel fill inside a bronze hairline.
static func card_box(border: Color = BRONZE, fill: Color = PANEL) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = fill
	box.border_color = border
	box.set_border_width_all(1)
	box.set_corner_radius_all(3)
	box.content_margin_left = 20
	box.content_margin_right = 20
	box.content_margin_top = 16
	box.content_margin_bottom = 16
	return box


static func label(text: String, font_size: int, color: Color) -> Label:
	var made := Label.new()
	made.text = text
	made.add_theme_font_size_override("font_size", font_size)
	made.add_theme_color_override("font_color", color)
	return made


## A hairline rule. Godot has no thin separator that takes a colour without a
## Theme, so the shell draws one as a one-pixel ColorRect.
static func rule(color: Color, width: float, thickness: float = 1.0) -> ColorRect:
	var made := ColorRect.new()
	made.color = color
	made.custom_minimum_size = Vector2(width, thickness)
	made.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return made


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
