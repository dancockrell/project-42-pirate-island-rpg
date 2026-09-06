class_name PaperCard
extends Control

## A readable card-sized stand-in for the future illustrated portrait package.
## The Button remains the input owner; this control owns only visual blocking.
##
## B5: the card draws what the simulation says about the actor and nothing else.
## `band_label` is the `band_name` string the bridge sends -- the card never
## spells a band name of its own -- and the ten Composure pips and the Shaken
## mark come from `composure` and `statuses` on the same actor dictionary.

const COMPOSURE_PIPS := 10

var display_name := "BETTY"
var band_label := "party_front"
var accent := Color("2d7770")
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

func _draw() -> void:
	var s := size
	# Dark-and-bronze frame, a physical card rather than a generic button. A
	# Shaken actor's frame carries the danger colour, so the state reads before
	# the word does.
	draw_style_box(make_box(Color("14201e"), Color("c24e45") if shaken else Color("b78a4b"), 2, 10), Rect2(Vector2.ZERO, s))
	draw_rect(Rect2(8, 8, 54, s.y - 16), Color(accent, .46))
	draw_circle(Vector2(35, 34), 21, Color("d8c29e"))
	draw_circle(Vector2(32, 28), 24, Color("26312e") if portrait_kind == "captain" else accent.darkened(.25))
	draw_circle(Vector2(35, 34), 17, Color("d8c29e"))
	# Compact cut-paper torso and weapon mark communicate the card's combat job.
	if portrait_kind == "captain":
		draw_colored_polygon(PackedVector2Array([Vector2(17,56),Vector2(53,56),Vector2(61,101),Vector2(10,101)]), Color("2a3b42"))
		draw_rect(Rect2(27, 57, 11, 31), Color("eadfca"))
		draw_rect(Rect2(16, 17, 36, 7), Color("4a3928"))
		draw_rect(Rect2(24, 7, 20, 14), Color("4a3928"))
		draw_line(Vector2(19, 68), Vector2(56, 56), Color("b78948"), 4)
		draw_circle(Vector2(48, 76), 6, Color("d0a85f"))
		draw_circle(Vector2(18, 72), 7, Color("9b6f3d"))
		draw_arc(Vector2(18, 72), 9, deg_to_rad(200), deg_to_rad(340), 8, Color("d0a85f"), 2)
	elif portrait_kind == "hostile":
		# A hostile actor is not drawn as a heroine. Beak, eye and raised claw
		# are the raptor read; the illustrated package replaces all three.
		draw_colored_polygon(PackedVector2Array([Vector2(20,56),Vector2(52,56),Vector2(58,101),Vector2(12,101)]), accent.darkened(.35))
		draw_colored_polygon(PackedVector2Array([Vector2(46,30),Vector2(62,37),Vector2(46,42)]), Color("d8a24a"))
		draw_circle(Vector2(36, 30), 4, Color("f4e6c8"))
		draw_circle(Vector2(36, 30), 2, Color("2b0f0f"))
		draw_line(Vector2(20, 74), Vector2(10, 88), Color("d0a85f"), 4)
		draw_line(Vector2(20, 74), Vector2(14, 92), Color("d0a85f"), 4)
	else:
		draw_colored_polygon(PackedVector2Array([Vector2(22,55),Vector2(48,55),Vector2(54,100),Vector2(16,100)]), accent)
		draw_line(Vector2(48,66), Vector2(59,96), Color("d0a85f"), 5)
		draw_circle(Vector2(60,98), 6, Color("c79a55"))
		draw_circle(Vector2(31, 33), 5, Color("f7f3de"))
		draw_circle(Vector2(40, 33), 5, Color("f7f3de"))
		draw_circle(Vector2(31, 33), 2.5, Color("3b9b82"))
		draw_circle(Vector2(40, 33), 2.5, Color("3b9b82"))
		draw_arc(Vector2(35, 36), 9, deg_to_rad(20), deg_to_rad(160), 8, Color("a94f50"), 2)
	draw_composure(s)
	if shaken:
		draw_string(ThemeDB.fallback_font, Vector2(6, s.y - 5), "SHAKEN", HORIZONTAL_ALIGNMENT_CENTER, 58, 10, Color("e08079"))

func draw_composure(s: Vector2) -> void:
	# Ten pips, one for each point of Composure. A count, not a bar and not a
	# percentage: the player must be able to see how many are left.
	var filled := clampi(composure, 0, COMPOSURE_PIPS)
	var origin := Vector2(74, s.y - 16)
	for index in COMPOSURE_PIPS:
		var centre := origin + Vector2(index * 11.5, 0)
		if index < filled:
			draw_circle(centre, 3.5, Color("c24e45") if shaken else Color("4fc7b4"))
		else:
			draw_arc(centre, 3.5, 0, TAU, 12, Color("6d7c78"), 1.5, true)

func make_box(background: Color, border: Color, width: int, radius: int) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = background
	box.border_color = border
	box.set_border_width_all(width)
	box.set_corner_radius_all(radius)
	return box
