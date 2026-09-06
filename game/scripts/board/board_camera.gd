class_name BoardCamera
extends Camera3D

## The board's fixed isometric view, at the three distances B12 names.
##
## **This class is P2's to absorb.** P2 (`game/scripts/render/camera_director.gd`)
## owns the project's camera director and its framing rules; that lane had not
## landed on `origin` when this board was written, so the angles live here.
## The two named angles below are the ones a director must keep -- true
## isometric, 45 degrees of yaw and the 35.264 degrees of pitch that makes the
## three axes equal on screen -- and when the director exists this class becomes
## the three `size` values and the framing target it hands over, not a second
## camera. Nothing here reads the simulation.

## The brief's fixed view: the camera never orbits and the player never turns it.
const YAW_DEGREES := 45.0
## atan(1 / sqrt(2)) in degrees. The true isometric pitch, so a cube's three
## visible faces are equal and a blockout reads as a solid rather than a plan.
const PITCH_DEGREES := -35.264

## **needs decision.** Orthographic frame height, in metres, at each distance.
## These are framing choices, not simulation facts: the world distance has to
## hold an island roughly 280 m across, the route distance a cell and its
## neighbours, the room distance one footprint with air around it. P2's director
## owns the final numbers.
const WORLD_SIZE_METRES := 232.0
const ROUTE_SIZE_METRES := 118.0
const ROOM_SIZE_METRES := 56.0

## How far back along the view axis the camera stands. Orthographic, so this
## only decides what is clipped, never how large anything looks.
const STANDOFF_METRES := 400.0


func _ready() -> void:
	projection = PROJECTION_ORTHOGONAL
	near = 1.0
	far = 1200.0
	current = true


## Frame `target` (a point in board metres) at one of the three named distances.
func frame(distance: String, target: Vector3) -> void:
	size = size_for(distance)
	rotation_degrees = Vector3(PITCH_DEGREES, YAW_DEGREES, 0.0)
	position = target - view_direction() * STANDOFF_METRES


## The orthographic frame height for a named distance. An unknown name frames
## the world, which is the distance that can always be drawn.
static func size_for(distance: String) -> float:
	match distance:
		"route":
			return ROUTE_SIZE_METRES
		"room":
			return ROOM_SIZE_METRES
		_:
			return WORLD_SIZE_METRES


## **needs decision.** How much air is left around the island at the world
## distance, as a multiple of the frame that just contains it. A framing choice;
## P2's director owns the final number.
const WORLD_FIT_MARGIN := 1.12


## Widen the frame until every one of `points` is inside it, and never narrow it
## below the named distance size.
##
## The world distance has to hold an island whose width is content -- nine
## authored rooms at authored positions, and a tenth tomorrow -- so a single
## constant frame height can only ever be right for the island that existed the
## day it was written. This measures instead: each point is taken into the
## camera's own space, and the frame is the larger of what the height needs and
## what the width needs at this viewport's aspect. Nothing here reads the
## simulation; a footprint is content and a viewport is a window.
func fit(points: PackedVector3Array) -> void:
	if points.is_empty():
		return
	var into_camera := global_transform.affine_inverse()
	var lowest := Vector2(INF, INF)
	var highest := Vector2(-INF, -INF)
	for point in points:
		var local := into_camera * point
		lowest = Vector2(minf(lowest.x, local.x), minf(lowest.y, local.y))
		highest = Vector2(maxf(highest.x, local.x), maxf(highest.y, local.y))
	var span := highest - lowest
	var viewport_size := get_viewport().get_visible_rect().size
	var aspect := viewport_size.x / maxf(viewport_size.y, 1.0)
	size = maxf(size, maxf(span.y, span.x / maxf(aspect, 0.0001)) * WORLD_FIT_MARGIN)
	# And then centred on what it is holding. Framing the middle of the island's
	# ground-plane rectangle is not the same as framing the middle of the picture
	# once the rooms are at nine different elevations, so the camera slides in its
	# own plane to put the measured middle in the middle. It slides; it never
	# turns -- the view stays the fixed one the brief asks for.
	var middle := (lowest + highest) * 0.5
	position += global_transform.basis * Vector3(middle.x, middle.y, 0.0)


## The unit vector the fixed camera looks along.
static func view_direction() -> Vector3:
	var basis_value := Basis.from_euler(Vector3(deg_to_rad(PITCH_DEGREES), deg_to_rad(YAW_DEGREES), 0.0))
	return -basis_value.z
