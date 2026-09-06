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


## The unit vector the fixed camera looks along.
static func view_direction() -> Vector3:
	var basis_value := Basis.from_euler(Vector3(deg_to_rad(PITCH_DEGREES), deg_to_rad(YAW_DEGREES), 0.0))
	return -basis_value.z
