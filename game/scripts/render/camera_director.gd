class_name CameraDirector
extends Node3D
## The fixed isometric camera rig. The brief (section 18) accepts a
## fixed-view isometric presentation and asks for bold silhouettes judged at
## gameplay distance, so the heading never changes: only the projection and
## the framing distance do.
##
## The rig is a Node3D that carries the heading and a Camera3D child that
## carries the projection. Rotating the rig is not a supported move; the
## heading constants below are the view.

## The classic isometric family: a 45 degree yaw so both horizontal axes read
## at the same angle, and a 30 degree pitch, which is the 2:1 screen ratio
## every isometric tile set since the 1980s has been drawn for. Named, not
## sprinkled, so a later lane changes the view in one place or not at all.
const ISOMETRIC_YAW_DEGREES := 45.0
const ISOMETRIC_PITCH_DEGREES := 30.0

## Projection modes. Orthographic is the honest isometric projection.
## Near-orthographic is a very narrow perspective frustum pulled far back: it
## keeps parallel silhouettes almost parallel while giving back the small
## amount of depth cue that a shadowed, fogged 3D board reads better with.
enum ProjectionMode { ORTHOGONAL, NEAR_ORTHOGONAL }

## The narrow field of view that makes perspective read as near-orthographic.
const NEAR_ORTHOGONAL_FOV_DEGREES := 12.0

## The three distances the board is read at (P4 owns the board itself; this is
## the framing vocabulary it will call into).
enum Distance { WORLD, ROUTE, ROOM }

## Framing margin per distance: how much bigger than the framed bounds the
## visible field is, as a multiplier. World reads as a map and wants air;
## room reads as a stage and wants the setpiece to fill the plate. These are
## presentation values, not design decisions.
const MARGIN_WORLD := 1.24
const MARGIN_ROUTE := 1.12
const MARGIN_ROOM := 1.05

## How far behind the framed bounds the camera sits, in metres, before its own
## near plane. Enough that nothing in the bounds is clipped at any heading.
const DEPTH_CLEARANCE := 12.0

## Near and far planes. Near is small because an orthographic camera does not
## pay a precision cost for it; far is generous because the world distance
## frames a whole island.
const NEAR_PLANE := 0.05
const FAR_PLANE := 6000.0

## Fallback framing when a caller hands over an empty or degenerate AABB, so
## the rig never produces a zero-size camera that renders a uniform frame.
const MINIMUM_FRAMED_EXTENT := 1.0

## The child that carries the projection.
const CAMERA_NODE_NAME := "IsometricCamera"

## What to frame when the rig enters the tree. A scene that instances the rig
## points this at the subtree it wants filled and picks the distance; nothing
## else has to be written. Left empty, the rig frames its default box, which
## is what a bare instantiation in a suite gets.
@export var frame_target: NodePath
@export var frame_target_distance: Distance = Distance.ROOM
@export var frame_target_projection: ProjectionMode = ProjectionMode.ORTHOGONAL

## What the rig is currently framing, kept so a projection change re-frames
## the same thing instead of losing it.
var framed_bounds := AABB(Vector3.ZERO, Vector3.ONE * 8.0)
var framed_distance: Distance = Distance.ROOM
var projection: ProjectionMode = ProjectionMode.ORTHOGONAL

var _camera: Camera3D = null


func _ready() -> void:
	set_meta("render_contract", "Fixed isometric heading; only projection and framing distance vary.")
	projection = frame_target_projection
	apply_isometric_heading()
	frame_bounds(framed_bounds, frame_target_distance)
	if not frame_target.is_empty():
		# Deferred: a procedural setpiece builds its geometry in its own
		# _ready, so its bounds do not exist until every _ready has run.
		call_deferred("_frame_declared_target")


func _frame_declared_target() -> void:
	var subject := get_node_or_null(frame_target) as Node3D
	if subject == null:
		push_error("CameraDirector was pointed at a frame_target that is not a Node3D: %s" % frame_target)
		return
	if not frame_subtree(subject, frame_target_distance):
		push_error("CameraDirector found nothing to frame under: %s" % frame_target)


## The Camera3D this rig drives. Resolved lazily so framing works before the
## rig has entered the tree.
func camera() -> Camera3D:
	if is_instance_valid(_camera):
		return _camera
	_camera = get_node_or_null(CAMERA_NODE_NAME) as Camera3D
	if _camera == null:
		push_error("CameraDirector expects a Camera3D child named %s" % CAMERA_NODE_NAME)
	return _camera


## The fixed heading, as a direction pointing from the subject toward the
## camera. Unit length.
static func isometric_offset_direction() -> Vector3:
	var pitch := deg_to_rad(ISOMETRIC_PITCH_DEGREES)
	var yaw := deg_to_rad(ISOMETRIC_YAW_DEGREES)
	return Vector3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch)).normalized()


## The margin multiplier for a distance.
static func margin_for(distance: Distance) -> float:
	match distance:
		Distance.WORLD:
			return MARGIN_WORLD
		Distance.ROUTE:
			return MARGIN_ROUTE
		Distance.ROOM:
			return MARGIN_ROOM
	return MARGIN_ROOM


func apply_isometric_heading() -> void:
	var view := camera()
	if view == null:
		return
	view.rotation_degrees = Vector3(-ISOMETRIC_PITCH_DEGREES, ISOMETRIC_YAW_DEGREES, 0.0)
	view.near = NEAR_PLANE
	view.far = FAR_PLANE


## Switch projection and re-frame what was already framed, so the subject does
## not jump when the mode changes.
func set_projection_mode(mode: ProjectionMode) -> void:
	projection = mode
	frame_bounds(framed_bounds, framed_distance)


## Point the rig at a world-space box and fill the frame with it at the named
## distance. The heading is unchanged; only position and projection size move.
func frame_bounds(bounds: AABB, distance: Distance) -> void:
	framed_bounds = bounds
	framed_distance = distance
	var view := camera()
	if view == null:
		return
	apply_isometric_heading()
	var margin := margin_for(distance)
	var extent := _screen_extent(bounds, view.global_transform.basis)
	var half_height: float = maxf(extent.y, MINIMUM_FRAMED_EXTENT) * margin
	var half_width: float = maxf(extent.x, MINIMUM_FRAMED_EXTENT) * margin
	var aspect := viewport_aspect()
	# Godot keeps the vertical extent by default, so a wide subject has to be
	# converted into the height that contains it.
	half_height = maxf(half_height, half_width / maxf(aspect, 0.0001))
	var depth: float = maxf(extent.z, MINIMUM_FRAMED_EXTENT) + DEPTH_CLEARANCE
	var focus := bounds.get_center()
	match projection:
		ProjectionMode.ORTHOGONAL:
			view.projection = Camera3D.PROJECTION_ORTHOGONAL
			view.size = half_height * 2.0
			view.global_position = focus + isometric_offset_direction() * depth
			view.far = depth + maxf(extent.z, MINIMUM_FRAMED_EXTENT) + DEPTH_CLEARANCE
		ProjectionMode.NEAR_ORTHOGONAL:
			view.projection = Camera3D.PROJECTION_PERSPECTIVE
			view.fov = NEAR_ORTHOGONAL_FOV_DEGREES
			var pullback := half_height / tan(deg_to_rad(NEAR_ORTHOGONAL_FOV_DEGREES) * 0.5)
			var total: float = pullback + depth
			view.global_position = focus + isometric_offset_direction() * total
			view.far = minf(FAR_PLANE, total + maxf(extent.z, MINIMUM_FRAMED_EXTENT) * 2.0 + DEPTH_CLEARANCE)


## Frame a point with a radius, the common case for a party miniature or a
## single setpiece, expressed through the same framing rule.
func frame_point(centre: Vector3, radius: float, distance: Distance) -> void:
	var span := maxf(radius, MINIMUM_FRAMED_EXTENT)
	frame_bounds(AABB(centre - Vector3.ONE * span, Vector3.ONE * span * 2.0), distance)


## Frame everything a subtree draws. Returns false when the subtree has no
## visual bounds at all, so the caller can say so instead of framing nothing.
func frame_subtree(subject: Node3D, distance: Distance) -> bool:
	var bounds := subtree_bounds(subject)
	if bounds.size == Vector3.ZERO:
		return false
	frame_bounds(bounds, distance)
	return true


## The world-space AABB of every VisualInstance3D under a node, itself
## included. An empty AABB means nothing visual was found.
static func subtree_bounds(subject: Node3D) -> AABB:
	var bounds := AABB()
	var seen := false
	var pending: Array[Node] = [subject]
	while not pending.is_empty():
		var node: Node = pending.pop_back()
		for child in node.get_children():
			pending.append(child)
		var visual := node as VisualInstance3D
		if visual == null or not visual.is_inside_tree():
			continue
		var local := visual.get_aabb()
		if local.size == Vector3.ZERO:
			continue
		var world := visual.global_transform * local
		if seen:
			bounds = bounds.merge(world)
		else:
			bounds = world
			seen = true
	return bounds


## Half-extents of a box measured along a camera basis: x across the frame,
## y up the frame, z into it.
static func _screen_extent(bounds: AABB, basis: Basis) -> Vector3:
	var half := bounds.size * 0.5
	var right := basis.x.abs()
	var up := basis.y.abs()
	var back := basis.z.abs()
	return Vector3(
		right.dot(half),
		up.dot(half),
		back.dot(half),
	)


## Width over height of the frame this rig draws into. Falls back to the
## project's declared viewport when the rig is not in a tree yet, so framing is
## the same number in a headless suite as it is on screen.
func viewport_aspect() -> float:
	if is_inside_tree():
		var rect := get_viewport().get_visible_rect().size
		if rect.y > 0.0:
			return rect.x / rect.y
	var width := float(ProjectSettings.get_setting("display/window/size/viewport_width", 1920))
	var height := float(ProjectSettings.get_setting("display/window/size/viewport_height", 1080))
	if height <= 0.0:
		return 16.0 / 9.0
	return width / height
