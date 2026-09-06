class_name PaperCard
extends Control

## A readable card-sized stand-in for the future illustrated portrait package.
## The Button remains the input owner; this control owns only visual blocking.
##
## B5: the card draws what the simulation says about the actor and nothing else.
## `band_label` is the `band_name` string the bridge sends -- the card never
## spells a band name of its own -- and the ten Composure pips and the Shaken
## mark come from `composure` and `statuses` on the same actor dictionary.

## The one door onto the palette and the type scale (card P11). The card's
## frame, its Composure well and its Shaken tab are interface and come from the
## Theme; the portrait below them is a placeholder illustration and its colours
## are marked as the game colours they are.
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")
## The one owner of the screen's sizes (card P14). The portrait gutter and the
## Composure well are measured there, so the card and the copy column beside it
## cannot disagree about where one ends and the other begins.
const BattleMetricsScript = preload("res://scripts/battle/battle_metrics.gd")

const COMPOSURE_PIPS := 10

## The cut-paper portrait's own palette. It stands in for an illustrated
## portrait package and is replaced with it, not restyled with the interface.
# game colour: a placeholder illustration's authored palette -- skin, hair,
# cloth, leather, brass, beak and eye. None of these is a UI colour; every one
# of them goes when `content/art/placeholders.json` names a real portrait.
const PORTRAIT := {
	"skin": Color("d8c29e"),
	"captain_hair": Color("26312e"),
	"captain_coat": Color("2a3b42"),
	"captain_shirt": Color("eadfca"),
	"captain_hat": Color("4a3928"),
	"blade": Color("b78948"),
	"brass": Color("d0a85f"),
	"brass_dark": Color("9b6f3d"),
	"brass_soft": Color("c79a55"),
	"beak": Color("d8a24a"),
	"eye_white": Color("f4e6c8"),
	"eye_dark": Color("2b0f0f"),
	"heroine_eye_white": Color("f7f3de"),
	"heroine_iris": Color("3b9b82"),
	"mouth": Color("a94f50")
}

var display_name := "BETTY"
var band_label := "party_front"
# not a colour: the stage calls `configure` with the actor's own accent before
# the first draw; this is what a card wears if it is drawn without one.
var accent := Color.WHITE
var composure := COMPOSURE_PIPS
var shaken := false
var portrait_kind := "heroine"

func configure(next_name: String, next_band_label: String, next_accent: Color, next_composure: int, next_shaken: bool, next_portrait_kind := "heroine") -> void:
	display_name = next_name
	band_label = next_band_label
	accent = next_accent
	composure = next_composure
	shaken = next_shaken
	portrait_kind = next_portrait_kind
	queue_redraw()

func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		queue_redraw()


func _draw() -> void:
	var s := size
	var theme := ThemeTokensScript.active(self)
	var danger := ThemeTokensScript.color(theme, "danger")
	# Dark-and-bronze frame, a physical card rather than a generic button. A
	# Shaken actor's frame carries the danger colour, so the state reads before
	# the word does.
	draw_style_box(make_box(
		ThemeTokensScript.color(theme, "panel"),
		danger if shaken else ThemeTokensScript.color(theme, "bronze"),
		2, 10), Rect2(Vector2.ZERO, s))
	draw_rect(Rect2(8, 8, 54, s.y - 16), Color(accent, .46))
	draw_circle(Vector2(35, 34), 21, PORTRAIT["skin"])
	draw_circle(Vector2(32, 28), 24, PORTRAIT["captain_hair"] if portrait_kind == "captain" else accent.darkened(.25))
	draw_circle(Vector2(35, 34), 17, PORTRAIT["skin"])
	# Compact cut-paper torso and weapon mark communicate the card's combat job.
	if portrait_kind == "captain":
		draw_colored_polygon(PackedVector2Array([Vector2(17,56),Vector2(53,56),Vector2(61,101),Vector2(10,101)]), PORTRAIT["captain_coat"])
		draw_rect(Rect2(27, 57, 11, 31), PORTRAIT["captain_shirt"])
		draw_rect(Rect2(16, 17, 36, 7), PORTRAIT["captain_hat"])
		draw_rect(Rect2(24, 7, 20, 14), PORTRAIT["captain_hat"])
		draw_line(Vector2(19, 68), Vector2(56, 56), PORTRAIT["blade"], 4)
		draw_circle(Vector2(48, 76), 6, PORTRAIT["brass"])
		draw_circle(Vector2(18, 72), 7, PORTRAIT["brass_dark"])
		draw_arc(Vector2(18, 72), 9, deg_to_rad(200), deg_to_rad(340), 8, PORTRAIT["brass"], 2)
	elif portrait_kind == "hostile":
		# A hostile actor is not drawn as a heroine. Beak, eye and raised claw
		# are the raptor read; the illustrated package replaces all three.
		draw_colored_polygon(PackedVector2Array([Vector2(20,56),Vector2(52,56),Vector2(58,101),Vector2(12,101)]), accent.darkened(.35))
		draw_colored_polygon(PackedVector2Array([Vector2(46,30),Vector2(62,37),Vector2(46,42)]), PORTRAIT["beak"])
		draw_circle(Vector2(36, 30), 4, PORTRAIT["eye_white"])
		draw_circle(Vector2(36, 30), 2, PORTRAIT["eye_dark"])
		draw_line(Vector2(20, 74), Vector2(10, 88), PORTRAIT["brass"], 4)
		draw_line(Vector2(20, 74), Vector2(14, 92), PORTRAIT["brass"], 4)
	else:
		draw_colored_polygon(PackedVector2Array([Vector2(22,55),Vector2(48,55),Vector2(54,100),Vector2(16,100)]), accent)
		draw_line(Vector2(48,66), Vector2(59,96), PORTRAIT["brass"], 5)
		draw_circle(Vector2(60,98), 6, PORTRAIT["brass_soft"])
		draw_circle(Vector2(31, 33), 5, PORTRAIT["heroine_eye_white"])
		draw_circle(Vector2(40, 33), 5, PORTRAIT["heroine_eye_white"])
		draw_circle(Vector2(31, 33), 2.5, PORTRAIT["heroine_iris"])
		draw_circle(Vector2(40, 33), 2.5, PORTRAIT["heroine_iris"])
		draw_arc(Vector2(35, 36), 9, deg_to_rad(20), deg_to_rad(160), 8, PORTRAIT["mouth"], 2)
	draw_composure(s, theme)

func draw_composure(s: Vector2, theme: Theme) -> void:
	# Ten pips, one for each point of Composure. A count, not a bar and not a
	# percentage: the player must be able to see how many are left at a glance,
	# so each pip sits in its own dark well and the spent ones stay visible as
	# empty sockets rather than fading into the card.
	#
	# Card P14: the well is one caption line tall with air around it, so it
	# grows with the text scale instead of staying a 24-pixel bar under type
	# that has grown past it, and it runs from the edge of the portrait gutter
	# to the card's inset rather than from a stated x.
	var filled := clampi(composure, 0, COMPOSURE_PIPS)
	var well_height := BattleMetricsScript.well_height(theme)
	var left := BattleMetricsScript.PORTRAIT_GUTTER - 4.0
	var trough := Rect2(left, s.y - well_height - 6.0, s.x - left - 8.0, well_height)
	var spacing := (trough.size.x - 16.0) / float(COMPOSURE_PIPS)
	var origin := Vector2(trough.position.x + 8.0 + spacing * .5, trough.position.y + trough.size.y * .5)
	var radius := minf(5.0, trough.size.y * .28)
	var lit := ThemeTokensScript.color(theme, "danger_soft" if shaken else "teal")
	var well := ThemeTokensScript.color(theme, "night")
	draw_style_box(make_box(Color(well, .82), Color(lit, .35), 1, 8), trough)
	for index in COMPOSURE_PIPS:
		var centre := origin + Vector2(spacing * float(index), 0.0)
		if index < filled:
			draw_circle(centre, radius * 1.4, Color(lit, .30))
			draw_circle(centre, radius, lit)
		else:
			draw_arc(centre, radius, 0, TAU, 14, ThemeTokensScript.color(theme, "muted").darkened(0.42), 1.6, true)


## Card P14: the Shaken mark used to be drawn here, as a 68-pixel tab pinned to
## the card's top-right corner -- which is inside the copy column, over the
## actor's name, and which stayed 68 pixels wide while the word inside it grew
## with the text scale. It is a Label in the copy column now, laid out beside
## the name and sized from the caption step like every other line on this
## screen; the frame's danger colour, which is drawn above, is what this card
## still says about it.


func make_box(background: Color, border: Color, width: int, radius: int) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = background
	box.border_color = border
	box.set_border_width_all(width)
	box.set_corner_radius_all(radius)
	return box
