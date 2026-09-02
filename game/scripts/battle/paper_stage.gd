class_name PaperStage
extends Control

## Reception Terrace is a tracked provisional environment plate. The active
## party and enemies remain separate articulated paper rigs. The stage owns no
## combat state; its sole job is to establish the correct shared floor, depth
## and ancient-elven geography at the player camera.

const RECEPTION_TERRACE_BACKDROP := preload("res://assets/standins/reception_terrace/reception_terrace_backdrop_v1.png")

func _draw() -> void:
	var bounds := Rect2(Vector2.ZERO, size)
	# Use the entire 16:9 environment plate. The generated asset deliberately
	# contains no cast, monsters, UI or text, so it can never contradict live
	# simulation data and can be replaced by a production-painted layer stack.
	draw_texture_rect(RECEPTION_TERRACE_BACKDROP, bounds, false)
	# A restrained theatre wash lets the living UI and paper rigs read without
	# turning the painted background into a dim generic rectangle.
	draw_rect(bounds, Color(0.025, 0.065, 0.055, 0.18))
	# Exact shared standing floor. These marks are presentation-only guides for
	# current and future rig placement, retained at low contrast so the stone
	# terrace remains the visual surface.
	var floor_y := size.y * .755
	draw_line(Vector2(size.x * .16, floor_y), Vector2(size.x * .86, floor_y), Color("d9bd72", .34), 2.0)
	draw_arc(Vector2(size.x * .29, floor_y), size.x * .095, deg_to_rad(198), deg_to_rad(342), 20, Color("4fc7b4", .40), 2.0, true)
	draw_arc(Vector2(size.x * .66, floor_y), size.x * .095, deg_to_rad(198), deg_to_rad(342), 20, Color("c24e45", .40), 2.0, true)
