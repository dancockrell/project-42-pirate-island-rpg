class_name PaperBettyRig
extends Node2D

## A real articulated cut-paper proxy for Betty. Every visible body section is
## a child Node2D with its rotation pivot at the joint that owns it. This is
## intentionally simple art, yet it is a usable animation hierarchy rather
## than a single illustration translated around the battle plane.

const SKIN := Color("f1c9ad")
const AUBURN := Color("8e3d2e")
const LEATHER := Color("6d3c29")
const DARK_LEATHER := Color("34201d")
const LACE := Color("f1e6cf")
const TARTAN_GREEN := Color("34533d")
const TARTAN_NAVY := Color("1f3442")
const TARTAN_WINE := Color("784446")
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
var home_position := Vector2.ZERO
var pose_tween: Tween

func build(canvas: Vector2) -> void:
	for child in get_children():
		child.queue_free()
	# The rig uses a 520x560 design grid then scales as one object. This keeps
	# every bone location stable when actor panels or future cinematics resize.
	# Betty must feel like a fighting-game lead, not a small token on a huge
	# field. Scale uniformly from the design grid so her adult silhouette stays
	# proportioned, while every boot, mace head and coat-tail still clears the
	# eight-percent safe frame.
	var uniform_scale := minf(canvas.x / 520.0, canvas.y / 560.0) * .94
	position = Vector2((canvas.x - 520.0 * uniform_scale) * .50, canvas.y * .035)
	scale = Vector2.ONE * uniform_scale
	home_position = position
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
	if pose_tween != null:
		pose_tween.kill()
	apply_pose(pose_name)

func animate_to_pose(pose_name: String, motion := "") -> void:
	if torso == null:
		return
	if pose_tween != null:
		pose_tween.kill()
	var target := pose_values(pose_name)
	pose_tween = create_tween().set_parallel(true)
	var duration := 0.11
	if motion == "card_to_battle_plane":
		duration = 0.16
	elif motion == "contact_lunge":
		duration = 0.08
	elif motion == "battle_plane_to_card":
		duration = 0.18
	pose_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	pose_tween.tween_property(self, "position", target.root_position, duration)
	for property_name in target.rotations:
		var entry: Dictionary = target.rotations[property_name]
		pose_tween.tween_property(entry.get("node"), "rotation_degrees", entry.get("value"), duration)
	if pose_name == "horizontal_mace_hit":
		# The contact pose is held long enough to read, then the next authored
		# beat takes control. This is presentation timing only; it never delays
		# or changes the simulation event sequence.
		pose_tween.tween_property(self, "modulate", Color("fff6df"), duration * .55)
		pose_tween.tween_property(self, "modulate", Color.WHITE, duration)

func apply_pose(pose_name: String) -> void:
	var target := pose_values(pose_name)
	position = target.root_position
	for property_name in target.rotations:
		var entry: Dictionary = target.rotations[property_name]
		entry.get("node").rotation_degrees = entry.get("value")
	queue_redraw()

func pose_values(pose_name: String) -> Dictionary:
	# This table is the blocking contract for the first live combat action. The
	# result exposes every transform the director is allowed to own, which keeps
	# the fighter reproducible when paper is replaced by a 2D or 3D rig later.
	var rotations := {
		"torso": {"node": torso, "value": 0.0},
		"coat_left": {"node": coat_left, "value": 8.0},
		"coat_right": {"node": coat_right, "value": -8.0},
		"left_leg": {"node": left_leg, "value": 3.0},
		"right_leg": {"node": right_leg, "value": -4.0},
		"left_boot": {"node": left_boot, "value": -1.0},
		"right_boot": {"node": right_boot, "value": 3.0},
		"left_arm": {"node": left_arm, "value": 18.0},
		"right_arm": {"node": right_arm, "value": -28.0},
		"weapon": {"node": weapon, "value": -18.0},
		"head": {"node": head, "value": 0.0},
		"hair": {"node": hair, "value": 0.0}
	}
	var root_position := home_position
	match pose_name:
		"mace_low_guard":
			rotations.left_arm.value = 24.0; rotations.right_arm.value = -42.0; rotations.weapon.value = -36.0
			rotations.left_leg.value = -3.0; rotations.right_leg.value = 6.0
		"forward_step":
			root_position += Vector2(20, -4)
			rotations.left_leg.value = -18.0; rotations.right_leg.value = 12.0
			rotations.left_boot.value = -10.0; rotations.right_boot.value = 8.0
			rotations.left_arm.value = 28.0; rotations.right_arm.value = -48.0; rotations.weapon.value = -58.0
			rotations.coat_left.value = 19.0; rotations.coat_right.value = -18.0
		"horizontal_mace_hit":
			root_position += Vector2(55, -15)
			rotations.torso.value = -8.0; rotations.left_arm.value = 42.0; rotations.right_arm.value = -72.0; rotations.weapon.value = -26.0
			rotations.left_leg.value = -11.0; rotations.right_leg.value = 15.0
			rotations.coat_left.value = 27.0; rotations.coat_right.value = -27.0
		"mace_cross_body":
			root_position += Vector2(29, -3)
			rotations.torso.value = 5.0; rotations.left_arm.value = 20.0; rotations.right_arm.value = -60.0; rotations.weapon.value = -24.0
			rotations.left_leg.value = -4.0; rotations.right_leg.value = 8.0
			rotations.coat_left.value = 15.0; rotations.coat_right.value = -13.0
	return {"root_position": root_position, "rotations": rotations}

class PaperBettyPiece:
	extends Node2D
	var kind := ""
	var piece_size := Vector2.ZERO

	func _draw() -> void:
		match kind:
			"torso":
				# Approved Betty blockout: fitted tobacco leather over ivory lace,
				# dark leather shorts, and one green/navy tartan hip sash. The paper
				# model preserves costume identity without pretending to be final art.
				draw_colored_polygon(PackedVector2Array([Vector2(-56,-72),Vector2(56,-72),Vector2(49,-16),Vector2(42,24),Vector2(30,52),Vector2(-30,52),Vector2(-42,24),Vector2(-49,-16)]), LACE)
				draw_colored_polygon(PackedVector2Array([Vector2(-42,-18),Vector2(42,-18),Vector2(35,34),Vector2(24,42),Vector2(-24,42),Vector2(-35,34)]), LEATHER)
				draw_colored_polygon(PackedVector2Array([Vector2(-40,35),Vector2(40,35),Vector2(34,55),Vector2(-34,55)]), DARK_LEATHER)
				draw_rect(Rect2(-42,26,84,13), TARTAN_GREEN, true)
				draw_line(Vector2(-36,30),Vector2(34,30),TARTAN_NAVY,3)
				draw_line(Vector2(-28,38),Vector2(30,38),TARTAN_WINE,2)
				for x in [-25.0, 0.0, 25.0]: draw_line(Vector2(x,26),Vector2(x+7,39),TARTAN_NAVY,2)
				draw_line(Vector2(0,-13),Vector2(0,38),BRONZE,3)
				draw_line(Vector2(-24,7),Vector2(24,7),Color("2d1d1b"),2)
				# Open shoulders, blouse folds and fitted corset are separate visual
				# reads even in a simple paper proxy; do not substitute a box torso.
				draw_arc(Vector2(-43,-38),16,deg_to_rad(205),deg_to_rad(350),8,LACE,5,true)
				draw_arc(Vector2(43,-38),16,deg_to_rad(190),deg_to_rad(335),8,LACE,5,true)
			"coat_tail":
				draw_colored_polygon(PackedVector2Array([Vector2(-30,-12),Vector2(30,-12),Vector2(42,108),Vector2(-12,96)]), Color("4a2b25"))
				draw_line(Vector2(-24,4),Vector2(26,4),BRONZE,2)
			"leg":
				draw_line(Vector2.ZERO,Vector2(0,108),SKIN,24,true)
				draw_line(Vector2(-7,18),Vector2(7,18),Color("e2a98e"),2)
			"boot":
				draw_line(Vector2.ZERO,Vector2(0,85),BOOT,32,true)
				draw_line(Vector2(-7,10),Vector2(7,10),BRONZE,2)
				draw_line(Vector2(-8,88),Vector2(26,88),Color("1a1518"),13,true)
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
				draw_paper_ellipse(Vector2(0,5),Vector2(37,49),SKIN)
				draw_line(Vector2(-17,1),Vector2(-6,0),Color("5d382d"),2)
				draw_line(Vector2(6,0),Vector2(17,1),Color("5d382d"),2)
				draw_line(Vector2(-16,5),Vector2(-6,5),Color("244a45"),3)
				draw_line(Vector2(6,5),Vector2(16,5),Color("244a45"),3)
				draw_circle(Vector2(-10,6),2.8,GLASS); draw_circle(Vector2(10,6),2.8,GLASS)
				for x in [-21.0,-16.0,16.0,21.0]: draw_circle(Vector2(x,20),1.7,Color("c36f53"))
				draw_arc(Vector2(0,23),11,deg_to_rad(20),deg_to_rad(160),8,Color("a94f50"),2,true)
			"hair":
				draw_arc(Vector2(0,0),48,deg_to_rad(195),deg_to_rad(345),20,AUBURN,14,true)
				draw_circle(Vector2(-22,-34),16,AUBURN); draw_circle(Vector2(20,-39),19,Color("a94f32"))
				draw_arc(Vector2(-18,-8),25,deg_to_rad(125),deg_to_rad(250),8,Color("b05a39"),4,true)

	func draw_paper_ellipse(center: Vector2, radii: Vector2, color: Color) -> void:
		var points := PackedVector2Array()
		for index in range(24):
			var angle := TAU * float(index) / 24.0
			points.append(center + Vector2(cos(angle)*radii.x, sin(angle)*radii.y))
		draw_colored_polygon(points, color)
