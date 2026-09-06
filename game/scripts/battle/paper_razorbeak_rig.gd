class_name PaperRazorbeakRig
extends Node2D

## An articulated individual hostile for the prototype battle plane. This is a
## deliberately neutral paper construction, not a claim that final creature art
## exists. Its joints give the contact sequence a real target before a painted
## or 3D creature package is admitted.

const HIDE := Color("78362f")
const HIDE_LIGHT := Color("a74636")
const BELLY := Color("d4b071")
const CLAW := Color("e3c783")
const INK := Color("271b1b")

var body: RazorbeakPiece
var neck: RazorbeakPiece
var head: RazorbeakPiece
var jaw: RazorbeakPiece
var tail: RazorbeakPiece
var left_leg: RazorbeakPiece
var right_leg: RazorbeakPiece
var home_position := Vector2.ZERO
var reaction_tween: Tween
## The design grid the rig is laid out on, and where its feet and its footprint
## sit inside that grid. `build` scales the whole grid as one object, so these
## three constants are what lets the stage put the creature's claws on the
## plate's floor line instead of leaving it hanging in the air.
const DESIGN_GRID := Vector2(450.0, 455.0)
const GROUND_CENTRE := Vector2(287.0, 386.0)
const GROUND_WIDTH := 205.0
var uniform_scale := 1.0


func build(canvas: Vector2) -> void:
	for child in get_children():
		child.queue_free()
	uniform_scale = minf(canvas.x / DESIGN_GRID.x, canvas.y / DESIGN_GRID.y) * .83
	# Mirrored on the x axis: the grid is drawn facing right, and a hostile that
	# faces away from the party reads as scenery. It occupies the same span of
	# the canvas either way, so the flip is the sign of the scale plus the grid
	# width added back to the origin.
	var grid_left := (canvas.x - DESIGN_GRID.x * uniform_scale) * .52
	scale = Vector2(-uniform_scale, uniform_scale)
	position = Vector2(grid_left + DESIGN_GRID.x * uniform_scale, canvas.y * .22)
	home_position = position
	body = add_piece("body", Vector2(230, 112), Vector2(255, 228), 0.0)
	tail = add_piece("tail", Vector2(205, 80), Vector2(154, 214), -7.0)
	left_leg = add_piece("leg", Vector2(48, 132), Vector2(245, 292), 4.0)
	right_leg = add_piece("leg", Vector2(48, 132), Vector2(325, 292), -3.0)
	neck = add_piece("neck", Vector2(90, 94), Vector2(358, 183), -17.0)
	head = add_child_piece(neck, "head", Vector2(104, 72), Vector2(36, -26), 16.0)
	jaw = add_child_piece(head, "jaw", Vector2(73, 34), Vector2(48, 28), 0.0)
	set_pose("ready_idle")


func add_piece(kind: String, piece_size: Vector2, local_position: Vector2, rotation_degrees: float) -> RazorbeakPiece:
	var piece := RazorbeakPiece.new()
	piece.kind = kind
	piece.piece_size = piece_size
	piece.position = local_position
	piece.rotation_degrees = rotation_degrees
	piece.z_index = {"tail": 1, "leg": 2, "body": 3, "neck": 4, "head": 5, "jaw": 6}.get(kind, 0)
	add_child(piece)
	return piece


func add_child_piece(parent_piece: RazorbeakPiece, kind: String, piece_size: Vector2, local_position: Vector2, rotation_degrees: float) -> RazorbeakPiece:
	var piece := RazorbeakPiece.new()
	piece.kind = kind
	piece.piece_size = piece_size
	piece.position = local_position
	piece.rotation_degrees = rotation_degrees
	piece.z_index = 6
	parent_piece.add_child(piece)
	return piece


## The same shade the heroine rigs take: legs and tail stand in the terrace's
## shadow while the back and head keep the sky, and every piece renders through
## the doll's lighting material.
func apply_lighting(shade: float) -> void:
	var low := Color.WHITE.darkened(shade)
	var mid := Color.WHITE.darkened(shade * .5)
	for piece in [left_leg, right_leg, tail]:
		if piece != null:
			piece.self_modulate = low
	for piece in [body]:
		if piece != null:
			piece.self_modulate = mid
	for piece in get_children():
		mark_child_material(piece)


func mark_child_material(piece: Node) -> void:
	if piece is CanvasItem:
		(piece as CanvasItem).use_parent_material = true
	for child in piece.get_children():
		mark_child_material(child)


## Where the creature's claws meet the floor, in the parent doll's coordinates.
func ground_point() -> Vector2:
	return position + GROUND_CENTRE * scale


## How wide its footprint is on the floor, for the contact shadow.
func ground_width() -> float:
	return GROUND_WIDTH * uniform_scale


func set_pose(pose_name: String) -> void:
	if reaction_tween != null:
		reaction_tween.kill()
	apply_pose(pose_name)


func animate_reaction(motion: String, shake: float) -> void:
	if reaction_tween != null:
		reaction_tween.kill()
	var target := pose_values("hit_recoil" if motion == "contact_lunge" and shake > 0.0 else "ready_idle")
	reaction_tween = create_tween().set_parallel(true)
	reaction_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	reaction_tween.tween_property(self, "position", target.root_position, .10)
	for property_name in target.rotations:
		var entry: Dictionary = target.rotations[property_name]
		reaction_tween.tween_property(entry.get("node"), "rotation_degrees", entry.get("value"), .10)


func apply_pose(pose_name: String) -> void:
	var target := pose_values(pose_name)
	position = target.root_position
	for property_name in target.rotations:
		var entry: Dictionary = target.rotations[property_name]
		entry.get("node").rotation_degrees = entry.get("value")


func pose_values(pose_name: String) -> Dictionary:
	var rotations := {
		"body": {"node": body, "value": 0.0},
		"tail": {"node": tail, "value": -7.0},
		"left_leg": {"node": left_leg, "value": 4.0},
		"right_leg": {"node": right_leg, "value": -3.0},
		"neck": {"node": neck, "value": -17.0},
		"head": {"node": head, "value": 16.0},
		"jaw": {"node": jaw, "value": 0.0}
	}
	var root_position := home_position
	if pose_name == "hit_recoil":
		root_position += Vector2(27, -8)
		rotations.body.value = 7.0; rotations.tail.value = -26.0
		rotations.left_leg.value = 15.0; rotations.right_leg.value = -13.0
		rotations.neck.value = 11.0; rotations.head.value = -8.0; rotations.jaw.value = 19.0
	return {"root_position": root_position, "rotations": rotations}


class RazorbeakPiece:
	extends Node2D
	var kind := ""
	var piece_size := Vector2.ZERO

	func _draw() -> void:
		match kind:
			"body":
				draw_paper_ellipse(Vector2(0, 4), Vector2(112, 58), HIDE)
				draw_paper_ellipse(Vector2(25, 18), Vector2(76, 30), BELLY)
				for x in [-55.0, -20.0, 15.0, 50.0]: draw_line(Vector2(x, -35), Vector2(x + 18, -54), HIDE_LIGHT, 6)
			"tail":
				draw_colored_polygon(PackedVector2Array([Vector2(8,-27),Vector2(-174,-53),Vector2(-201,-4),Vector2(4,31)]), HIDE)
				for x in [-120.0, -80.0, -40.0]: draw_line(Vector2(x, -22), Vector2(x + 10, -46), HIDE_LIGHT, 5)
			"leg":
				draw_line(Vector2.ZERO, Vector2(1, 91), HIDE_LIGHT, 25, true)
				draw_line(Vector2(1, 91), Vector2(32, 94), CLAW, 11, true)
			"neck":
				draw_line(Vector2.ZERO, Vector2(45, -38), HIDE_LIGHT, 45, true)
			"head":
				draw_colored_polygon(PackedVector2Array([Vector2(-28,-25),Vector2(60,-18),Vector2(75,4),Vector2(28,28),Vector2(-33,19)]), HIDE_LIGHT)
				draw_circle(Vector2(25,-5), 6, Color("ffd166")); draw_circle(Vector2(27,-5), 2, INK)
				for x in [7.0, 20.0, 33.0, 46.0]: draw_line(Vector2(x,14),Vector2(x+4,22),CLAW,3)
			"jaw":
				draw_colored_polygon(PackedVector2Array([Vector2(-21,-5),Vector2(54,-4),Vector2(30,20),Vector2(-20,10)]), Color("8c4338"))

	func draw_paper_ellipse(center: Vector2, radii: Vector2, color: Color) -> void:
		var points := PackedVector2Array()
		for index in range(24):
			var angle := TAU * float(index) / 24.0
			points.append(center + Vector2(cos(angle) * radii.x, sin(angle) * radii.y))
		draw_colored_polygon(points, color)
