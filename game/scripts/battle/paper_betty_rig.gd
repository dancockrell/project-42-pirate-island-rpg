class_name PaperBettyRig
extends Node2D

## A real articulated cut-paper proxy for Betty. Every visible body section is
## a child Node2D with its rotation pivot at the joint that owns it. This is
## intentionally simple art, yet it is a usable animation hierarchy rather
## than a single illustration translated around the battle plane.

const SKIN := Color("f1c9ad")
const AUBURN := Color("8e3d2e")
const TEAL := Color("237e76")
const DARK_TEAL := Color("123d40")
const CREAM := Color("f1e6cf")
const BOOT := Color("3a2928")
const BRONZE := Color("b78948")
const GLASS := Color("5bcabe")

var torso: PaperBettyPiece
var head: PaperBettyPiece
var hair: PaperBettyPiece
var left_arm: PaperBettyPiece
var right_arm: PaperBettyPiece
var left_leg: PaperBettyPiece
var right_leg: PaperBettyPiece
var left_boot: PaperBettyPiece
var right_boot: PaperBettyPiece
var coat_left: PaperBettyPiece
var coat_right: PaperBettyPiece
var weapon: PaperBettyPiece

func build(canvas: Vector2) -> void:
	for child in get_children():
		child.queue_free()
	# The rig uses a 520x560 design grid then scales as one object. This keeps
	# every bone location stable when actor panels or future cinematics resize.
	position = Vector2(canvas.x * .31, canvas.y * .17)
	scale = Vector2(canvas.x / 520.0, canvas.y / 560.0) * .76
	torso = add_piece("torso", Vector2(112, 154), Vector2(260, 190), 0.0)
	coat_left = add_piece("coat_tail", Vector2(70, 125), Vector2(225, 292), 8.0)
	coat_right = add_piece("coat_tail", Vector2(70, 125), Vector2(296, 292), -8.0)
	left_leg = add_piece("leg", Vector2(22, 118), Vector2(238, 308), 3.0)
	right_leg = add_piece("leg", Vector2(22, 118), Vector2(281, 308), -4.0)
	left_boot = add_piece("boot", Vector2(26, 98), Vector2(238, 420), -1.0)
	right_boot = add_piece("boot", Vector2(26, 98), Vector2(281, 420), 3.0)
	left_arm = add_piece("arm", Vector2(20, 115), Vector2(210, 205), 18.0)
	right_arm = add_piece("arm", Vector2(20, 115), Vector2(309, 205), -28.0)
	weapon = add_piece("mace", Vector2(18, 174), Vector2(335, 282), -34.0)
	head = add_piece("head", Vector2(72, 86), Vector2(260, 107), 0.0)
	hair = add_piece("hair", Vector2(92, 92), Vector2(260, 95), 0.0)
	set_pose("ready_idle")

func add_piece(kind: String, piece_size: Vector2, local_position: Vector2, rotation_degrees: float) -> PaperBettyPiece:
	var piece := PaperBettyPiece.new()
	piece.kind = kind
	piece.piece_size = piece_size
	piece.position = local_position
	piece.rotation_degrees = rotation_degrees
	piece.z_index = piece_z(kind)
	add_child(piece)
	return piece

func piece_z(kind: String) -> int:
	return {"coat_tail": 1, "leg": 2, "boot": 3, "torso": 4, "arm": 5, "mace": 6, "head": 7, "hair": 8}.get(kind, 0)

func set_pose(pose_name: String) -> void:
	if torso == null:
		return
	# All pose changes act on real pivots. The later animation director can use
	# the same named pieces for tweens, hit stops and hand-authored keyframes.
	match pose_name:
		"forward_step":
			left_leg.rotation_degrees = -18; right_leg.rotation_degrees = 12
			left_boot.rotation_degrees = -10; right_boot.rotation_degrees = 8
			left_arm.rotation_degrees = 28; right_arm.rotation_degrees = -48; weapon.rotation_degrees = -58
		"horizontal_mace_hit":
			torso.rotation_degrees = -8; left_arm.rotation_degrees = 42; right_arm.rotation_degrees = -72; weapon.rotation_degrees = -96
			left_leg.rotation_degrees = -11; right_leg.rotation_degrees = 15
		"mace_cross_body":
			torso.rotation_degrees = 5; left_arm.rotation_degrees = 20; right_arm.rotation_degrees = -60; weapon.rotation_degrees = -72
		_:
			torso.rotation_degrees = 0; left_leg.rotation_degrees = 3; right_leg.rotation_degrees = -4
			left_boot.rotation_degrees = -1; right_boot.rotation_degrees = 3
			left_arm.rotation_degrees = 18; right_arm.rotation_degrees = -28; weapon.rotation_degrees = -34
	queue_redraw()

class PaperBettyPiece:
	extends Node2D
	var kind := ""
	var piece_size := Vector2.ZERO

	func _draw() -> void:
		match kind:
			"torso":
				draw_colored_polygon(PackedVector2Array([Vector2(-56,-72),Vector2(56,-72),Vector2(45,18),Vector2(32,52),Vector2(-32,52),Vector2(-45,18)]), CREAM)
				draw_colored_polygon(PackedVector2Array([Vector2(-40,-20),Vector2(40,-20),Vector2(34,48),Vector2(-34,48)]), TEAL)
				draw_line(Vector2(0,-15),Vector2(0,40),BRONZE,3)
			"coat_tail":
				draw_colored_polygon(PackedVector2Array([Vector2(-30,-12),Vector2(30,-12),Vector2(42,108),Vector2(-12,96)]), Color("174f4d"))
			"leg":
				draw_line(Vector2.ZERO,Vector2(0,108),SKIN,22,true)
			"boot":
				draw_line(Vector2.ZERO,Vector2(0,85),BOOT,30,true)
				draw_line(Vector2(-8,88),Vector2(25,88),Color("1a1518"),12,true)
			"arm":
				draw_line(Vector2.ZERO,Vector2(0,98),SKIN,18,true)
				draw_circle(Vector2(0,101),8,SKIN)
			"mace":
				draw_line(Vector2.ZERO,Vector2(0,154),Color("76512e"),10,true)
				draw_circle(Vector2(0,164),29,BRONZE)
				draw_circle(Vector2(0,164),17,GLASS)
				draw_arc(Vector2(0,164),31,0,TAU,16,Color("e4c487"),2,true)
			"head":
				draw_paper_ellipse(Vector2(0,4),Vector2(38,48),SKIN)
				draw_line(Vector2(-16,3),Vector2(-5,3),Color("244a45"),3)
				draw_line(Vector2(5,3),Vector2(16,3),Color("244a45"),3)
				draw_circle(Vector2(-10,4),2.5,GLASS); draw_circle(Vector2(10,4),2.5,GLASS)
				draw_line(Vector2(-7,30),Vector2(8,30),Color("a94f50"),2)
			"hair":
				draw_arc(Vector2(0,0),48,deg_to_rad(195),deg_to_rad(345),16,AUBURN,13,true)
				draw_circle(Vector2(-22,-34),15,AUBURN); draw_circle(Vector2(20,-39),18,Color("a94f32"))

	func draw_paper_ellipse(center: Vector2, radii: Vector2, color: Color) -> void:
		var points := PackedVector2Array()
		for index in range(24):
			var angle := TAU * float(index) / 24.0
			points.append(center + Vector2(cos(angle)*radii.x, sin(angle)*radii.y))
		draw_colored_polygon(points, color)
