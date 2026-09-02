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
var satchel: PaperBettyPiece
var ampoule_rack: PaperBettyPiece

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
	# Lower legs and boots are children of the thighs. A step changes the full
	# leg chain, so the boot cannot drift independently from its knee.
	left_boot = add_child_piece(left_leg, "boot", Vector2(26, 98), Vector2(0, 108), -4.0)
	right_boot = add_child_piece(right_leg, "boot", Vector2(26, 98), Vector2(0, 108), 5.0)
	left_arm = add_piece("arm", Vector2(20, 115), Vector2(210, 205), 18.0)
	right_arm = add_piece("arm", Vector2(20, 115), Vector2(309, 205), -28.0)
	# The mace is held from the right hand; a strike rotates the arm and the
	# complete weapon chain together. The off-hand remains free for medicine.
	weapon = add_child_piece(right_arm, "mace", Vector2(18, 174), Vector2(0, 101), -18.0)
	head = add_piece("head", Vector2(72, 86), Vector2(260, 107), 0.0)
	hair = add_child_piece(head, "hair", Vector2(92, 92), Vector2(0, -13), 0.0)
	# These props are identity-critical at game scale: they establish field
	# medicine before text, and retain stable sockets for all seven Betty skills.
	satchel = add_piece("satchel", Vector2(52, 56), Vector2(318, 282), -8.0)
	ampoule_rack = add_piece("ampoule_rack", Vector2(40, 44), Vector2(211, 270), 8.0)
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

func add_child_piece(parent_piece: PaperBettyPiece, kind: String, piece_size: Vector2, local_position: Vector2, rotation_degrees: float) -> PaperBettyPiece:
	var piece := PaperBettyPiece.new()
	piece.kind = kind
	piece.piece_size = piece_size
	piece.position = local_position
	piece.rotation_degrees = rotation_degrees
	piece.z_index = piece_z(kind)
	parent_piece.add_child(piece)
	return piece

func piece_z(kind: String) -> int:
	return {"coat_tail": 1, "leg": 2, "boot": 3, "torso": 4, "satchel": 5, "ampoule_rack": 5, "arm": 6, "mace": 7, "head": 8, "hair": 9}.get(kind, 0)

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
			torso.rotation_degrees = -8; left_arm.rotation_degrees = 42; right_arm.rotation_degrees = -72; weapon.rotation_degrees = -26
			left_leg.rotation_degrees = -11; right_leg.rotation_degrees = 15
		"mace_cross_body":
			torso.rotation_degrees = 5; left_arm.rotation_degrees = 20; right_arm.rotation_degrees = -60; weapon.rotation_degrees = -24
		_:
			torso.rotation_degrees = 0; left_leg.rotation_degrees = 3; right_leg.rotation_degrees = -4
			left_boot.rotation_degrees = -1; right_boot.rotation_degrees = 3
			left_arm.rotation_degrees = 18; right_arm.rotation_degrees = -28; weapon.rotation_degrees = -18
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
				# Open shoulders, blouse folds and fitted corset are separate visual
				# reads even in a simple paper proxy; do not substitute a box torso.
				draw_arc(Vector2(-43,-38),16,deg_to_rad(205),deg_to_rad(350),8,CREAM,5,true)
				draw_arc(Vector2(43,-38),16,deg_to_rad(190),deg_to_rad(335),8,CREAM,5,true)
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
				# A short heavy boarding mace, not a wizard staff. The body geometry
				# leaves a reinforced bronze neck and a luminous impact chamber clear.
				draw_line(Vector2.ZERO,Vector2(0,118),Color("76512e"),13,true)
				draw_circle(Vector2(0,132),31,BRONZE)
				draw_circle(Vector2(0,132),18,GLASS)
				draw_arc(Vector2(0,132),33,0,TAU,16,Color("e4c487"),2,true)
			"satchel":
				draw_rect(Rect2(-28,-24,56,47),Color("6e4d2c"),true)
				draw_rect(Rect2(-28,-24,56,47),BRONZE,false,3)
				draw_circle(Vector2(0,-1),9,Color("e6bf63"))
				draw_line(Vector2(-22,-28),Vector2(22,-42),Color("b78948"),4,true)
			"ampoule_rack":
				draw_line(Vector2(-16,8),Vector2(16,8),BRONZE,4,true)
				for x in [-12.0,0.0,12.0]:
					draw_rect(Rect2(x-4,-17,8,22),Color("d1e8dd"),true)
					draw_circle(Vector2(x,-18),4,GLASS)
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
