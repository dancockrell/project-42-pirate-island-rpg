class_name BoardBlockout
extends RefCounted

## Procedural blockout geometry for the board, and the one marked placeholder
## material it is made of.
##
## Brief section 18: buildings begin as cubes or multi-cube blockouts, and a
## readable blockout is not proof of finished production art. Every mesh this
## class makes is a Godot primitive built at runtime -- no mesh, texture or
## material file is added to the repository by this lane -- and every node it
## makes carries `blockout_placeholder` metadata naming what will replace it, so
## a model arriving later can be found by name rather than by eye.
##
## The material is P2's clay (`game/render/materials/clay_blockout.tres`, the
## matte clay with an edge-light rim that blockouts inherit), asked for through
## `SetpieceMeshFactory.material`, which is the one adapter over the library.
## This class holds no material of its own.

## **needs decision.** How thick a cell tile stands, in metres. A framing choice.
const TILE_THICKNESS_METRES := 1.4
## **needs decision.** How far below sea level the island's rock is carried, in
## metres. A room's terrace is a prism from here up to its authored elevation,
## which is what makes a set of rooms read as one island rather than as tiles.
const SEA_FLOOR_METRES := 9.0
## **needs decision.** How wide the causeway of land under a road is, in metres.
const CAUSEWAY_WIDTH_METRES := 17.0
## **needs decision.** How wide a drawn road is, in metres.
const ROUTE_WIDTH_METRES := 3.6
## **needs decision.** How tall one miniature stands, in metres. Big enough to
## read at the world distance, small enough not to hide the cell it stands on.
const MINIATURE_HEIGHT_METRES := 10.0
## The island's rock, under every room's ground. One colour: P5's Theme owns it.
const ROCK := Color("32403a")
## The island's own ground between the rooms -- jungle over volcanic soil.
const LAND := Color("4a5b48")

## **needs decision.** The radius of a spawn socket's marker, in metres.
const SOCKET_RADIUS_METRES := 1.15


## The board's clay. `SetpieceMeshFactory.material` is the one adapter over the
## material library in this project and this uses it rather than a second one.
## The tint is the shader's `albedo` parameter; read it back with
## `get_shader_parameter("albedo")`.
static func clay(color: Color, emission_energy: float = 0.0) -> Material:
	return SetpieceMeshFactory.material(color, 0.0, 0.86, color, emission_energy)


## A node name built from a stable ID. Godot forbids "." in a node name and
## silently rewrites one, so the trimmed ID is spelled with underscores here
## rather than left to the engine to mangle -- a suite that looks a node up by
## name is looking one thing up, not two.
static func node_name(prefix: String, stable_id: String, drop: String) -> String:
	return "%s%s" % [prefix, stable_id.trim_prefix(drop).replace(".", "_")]


## Mark a node as procedural blockout standing in for a real asset.
static func mark(node: Node, replaced_by: String) -> void:
	node.set_meta("blockout_placeholder", true)
	node.set_meta("replaced_by", replaced_by)
	node.set_meta("generated", "procedural primitive, built at runtime; no asset file")


## One cell as a terrace: a prism the size of the room's footprint, standing from
## the sea floor up to the room's authored elevation. A slab would have been
## enough to say where a room is; a prism says the island has a shape, which is
## what the world distance is for.
static func tile(parent: Node3D, cell: Dictionary, tint: Color) -> MeshInstance3D:
	var footprint: Dictionary = cell.get("footprint", {})
	var slug := String(cell.get("cell_id", "")).trim_prefix("world.cell.").replace(".", "_")
	var elevation := float(cell.get("elevation_metres", 0.0))
	var width := float(footprint.get("width_metres", 1.0))
	var depth := float(footprint.get("depth_metres", 1.0))
	var centre := centre_of(cell)
	# The rock the room stands on. One colour for every room, because the island
	# is one island: what differs between rooms is the ground on top of it.
	var plinth_height := elevation + SEA_FLOOR_METRES - TILE_THICKNESS_METRES
	var plinth := SetpieceMeshFactory.box(
		parent,
		"Plinth_%s" % slug,
		Vector3(width * 0.94, maxf(plinth_height, 0.5), depth * 0.94),
		Vector3(centre.x, elevation - TILE_THICKNESS_METRES - maxf(plinth_height, 0.5) * 0.5, centre.z),
		clay(ROCK)
	)
	mark(plinth, "the island's baked terrain under %s" % cell.get("cell_id", ""))
	# The room's own ground: the face the player reads, and the one that carries
	# who holds this cell.
	var terrace := SetpieceMeshFactory.box(
		parent,
		"Tile_%s" % slug,
		Vector3(width, TILE_THICKNESS_METRES, depth),
		Vector3(centre.x, elevation - TILE_THICKNESS_METRES * 0.5, centre.z),
		clay(tint)
	)
	mark(terrace, "the cell's baked visual shell (%s)" % cell.get("cell_id", ""))
	return terrace


## Where a cell's centre stands in board metres: its island position on the
## ground plane, lifted to its authored elevation.
static func centre_of(cell: Dictionary) -> Vector3:
	var island: Vector2 = cell.get("island_position_metres", Vector2.ZERO)
	return Vector3(island.x, float(cell.get("elevation_metres", 0.0)), island.y)


## The land under one road: a level causeway from the sea floor up to the lower
## of the two rooms it joins, so the island is continuous ground instead of
## terraces floating apart. Yaw only -- rock does not tilt.
static func causeway(parent: Node3D, node_name: String, from: Vector3, to: Vector3, tint: Color) -> MeshInstance3D:
	var flat_from := Vector3(from.x, 0.0, from.z)
	var flat_to := Vector3(to.x, 0.0, to.z)
	var length := maxf(flat_from.distance_to(flat_to), 0.01)
	var top := minf(from.y, to.y)
	var height := top + SEA_FLOOR_METRES
	var bar := SetpieceMeshFactory.box(parent, node_name, Vector3(CAUSEWAY_WIDTH_METRES, height, length), Vector3.ZERO, clay(tint))
	bar.position = Vector3(flat_from.lerp(flat_to, 0.5).x, top - height * 0.5, flat_from.lerp(flat_to, 0.5).z)
	bar.look_at(Vector3(flat_to.x, bar.global_position.y, flat_to.z), Vector3.UP)
	mark(bar, "the island's baked terrain between two rooms")
	return bar


## One road between two points, drawn as a flat bar. `width_scale` lets a legal
## road read heavier than a merely known one without a second mesh kind.
static func route(parent: Node3D, node_name: String, from: Vector3, to: Vector3, tint: Color, width_scale: float) -> MeshInstance3D:
	var span := to - from
	var length := maxf(span.length(), 0.01)
	var bar := SetpieceMeshFactory.box(
		parent,
		node_name,
		Vector3(ROUTE_WIDTH_METRES * width_scale, 0.95, length),
		Vector3.ZERO,
		clay(tint, 0.22)
	)
	bar.position = from.lerp(to, 0.5)
	# A road between two rooms stacked one above the other -- a stair, a shaft --
	# runs straight up, and `up` may not be colinear with it.
	var direction := span / length
	bar.look_at(to, Vector3.FORWARD if absf(direction.dot(Vector3.UP)) > 0.999 else Vector3.UP)
	mark(bar, "the authored road's baked geometry")
	return bar


## One miniature: a standing pawn. The party, a force and a convoy are the same
## silhouette at different scales and tints, which is the point -- the board
## says *where*, and the room distance says *what*.
static func miniature(parent: Node3D, node_name: String, tint: Color, scale_value: float = 1.0) -> Node3D:
	var pawn := Node3D.new()
	pawn.name = node_name
	parent.add_child(pawn)
	var height := MINIATURE_HEIGHT_METRES * scale_value
	var surface := clay(tint, 0.35)
	SetpieceMeshFactory.cylinder(pawn, "Base", height * 0.34, height * 0.42, height * 0.12, Vector3(0.0, height * 0.06, 0.0), surface, 12)
	SetpieceMeshFactory.cylinder(pawn, "Body", height * 0.14, height * 0.27, height * 0.62, Vector3(0.0, height * 0.43, 0.0), surface, 10)
	SetpieceMeshFactory.sphere(pawn, "Head", Vector3.ONE * height * 0.2, Vector3(0.0, height * 0.84, 0.0), surface)
	mark(pawn, "the actor's rigged model, through the character asset contract in brief section 18")
	return pawn


## One spawn socket, marked by role.
## The encounter space: a ring on the floor, drawn as a ring rather than a disc
## so the sockets inside it stay visible. A fight happens *in* the room's spawn
## points, not on top of them.
static func encounter_ring(parent: Node3D, ring_name: String, position: Vector3, radius: float, tint: Color) -> MeshInstance3D:
	var mesh := TorusMesh.new()
	mesh.inner_radius = radius * 0.9
	mesh.outer_radius = radius
	mesh.rings = 32
	mesh.ring_segments = 6
	mesh.material = clay(tint, 0.34)
	var ring := MeshInstance3D.new()
	ring.name = ring_name
	ring.mesh = mesh
	ring.position = position
	parent.add_child(ring)
	mark(ring, "no model: the encounter space is a volume, and the lens over it is Open decision O1")
	return ring


## One spawn socket: a pad on the floor with a post standing in it. The post's
## height is the role, so the four kinds are told apart by shape and not by
## colour alone -- the same rule the rest of the interface follows.
static func socket(parent: Node3D, marker_name: String, position: Vector3, tint: Color, post_height: float) -> Node3D:
	var group := Node3D.new()
	group.name = marker_name
	group.position = position
	parent.add_child(group)
	var surface := clay(tint, 0.45)
	SetpieceMeshFactory.cylinder(group, "Pad", SOCKET_RADIUS_METRES, SOCKET_RADIUS_METRES, 0.22, Vector3(0.0, 0.11, 0.0), surface, 14)
	if post_height > 0.0:
		SetpieceMeshFactory.cylinder(group, "Post", SOCKET_RADIUS_METRES * 0.34, SOCKET_RADIUS_METRES * 0.42, post_height, Vector3(0.0, 0.22 + post_height * 0.5, 0.0), surface, 8)
	mark(group, "no model: a socket is where a model is placed, and stays a marker")
	return group
