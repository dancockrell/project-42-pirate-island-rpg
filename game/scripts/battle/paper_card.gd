class_name PaperCard
extends Control

## A readable card-sized stand-in for the future illustrated portrait package.
## The Button remains the input owner; this control owns only visual blocking.

var display_name := "BETTY"
var role := "FIELD MEDIC"
var accent := Color("2d7770")
var card_rank := "D"

func configure(next_name: String, next_role: String, next_accent: Color, next_rank: String) -> void:
	display_name = next_name
	role = next_role
	accent = next_accent
	card_rank = next_rank
	queue_redraw()

func _draw() -> void:
	var s := size
	# Dark-and-bronze frame, a physical card rather than a generic button.
	draw_style_box(make_box(Color("14201e"), Color("b78a4b"), 2, 10), Rect2(Vector2.ZERO, s))
	draw_rect(Rect2(8, 8, 54, s.y - 16), Color(accent, .46))
	draw_circle(Vector2(35, 34), 21, Color("d8c29e"))
	draw_circle(Vector2(32, 28), 24, accent.darkened(.25))
	draw_circle(Vector2(35, 34), 17, Color("d8c29e"))
	# Compact cut-paper torso and weapon mark communicate the card's combat job.
	draw_colored_polygon(PackedVector2Array([Vector2(22,55),Vector2(48,55),Vector2(54,100),Vector2(16,100)]), accent)
	draw_line(Vector2(48,66), Vector2(59,96), Color("d0a85f"), 5)
	draw_circle(Vector2(60,98), 6, Color("c79a55"))
	draw_string(ThemeDB.fallback_font, Vector2(s.x - 32, 29), card_rank, HORIZONTAL_ALIGNMENT_LEFT, -1, 20, Color("e4c487"))
	draw_string(ThemeDB.fallback_font, Vector2(15, s.y - 10), "PAPER PORTRAIT • %s" % display_name, HORIZONTAL_ALIGNMENT_LEFT, -1, 9, Color("9ca9a1"))

func make_box(background: Color, border: Color, width: int, radius: int) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = background
	box.border_color = border
	box.set_border_width_all(width)
	box.set_corner_radius_all(radius)
	return box
