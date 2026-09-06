class_name BlockoutKit
extends RefCounted

## What the building and the machine blockout kits share: the material language,
## the socket and pivot node vocabulary, and the one place a metre this project
## has not decided is written down.
##
## Brief section 18: buildings may begin as cubes or multi-cube blockouts, a
## readable blockout is not proof of finished production art, and characters and
## machines keep future-ready pivots, sockets and metadata. Everything the two
## kits build is a Godot primitive made at runtime -- no mesh, texture or
## material file is added to the repository by this lane -- and every node is
## marked `blockout_placeholder` through P4's `BoardBlockout.mark`, which is the
## project's one marking vocabulary and is called here rather than copied.
##
## Materials come only from P2's library, and only through the one door,
## `SetpieceMeshFactory`. This class holds no material resource of its own.

## P4 owns the placeholder marking and the "a node name may not carry a dot"
## rule. Preloaded by path rather than named by `class_name`, so this file also
## compiles inside a `--script` run, where the global class cache does not exist
## (`capture_review_scene.gd` records the same finding).
const BoardBlockoutScript := preload("res://scripts/board/board_blockout.gd")
## The one material door, preloaded for the same reason.
const SetpieceMeshFactoryScript := preload("res://scripts/world/setpiece_mesh_factory.gd")

## The material language, in the library's own names. A building or machine
## record carries no material field -- brief section 3's material language is a
## *room* contract (C11), and C10 and C14 author neither -- so which library
## material a subject wears is named here, per record ID, and marked below as
## the decision it is standing in for.
const CLAY := SetpieceMeshFactoryScript.CLAY
const WET_STONE := SetpieceMeshFactoryScript.WET_STONE
const BRONZE := SetpieceMeshFactoryScript.BRONZE
const PLACEHOLDER := SetpieceMeshFactoryScript.PLACEHOLDER

## Riveted iron (brief section 5.3: "riveted iron and steel", "cast housings",
## "repair plates") is the machines' material language, and P2's library has no
## riveted iron in it. It is *not* added here as a ninth material file: iron is
## the library's own metal shader carrying iron's values -- a cold grey metal,
## an oxide rather than a verdigris patina, and almost no polish -- set through
## `SetpieceMeshFactoryScript.library_material`, which is the single door. If a
## `riveted_iron.tres` ever joins the library, this dictionary becomes one key
## and nothing else in either kit changes.
const IRON_PARAMETERS := {
	"metal_color": Color(0.322, 0.353, 0.396, 1.0),
	"patina_color": Color(0.412, 0.267, 0.180, 1.0),
	"tarnish_color": Color(0.121, 0.129, 0.145, 1.0),
	"patina_amount": 0.10,
	"patina_bias": 0.05,
	"patina_scale": 3.4,
	"polish": 0.34,
	"metal_roughness": 0.54,
	"patina_roughness": 0.94,
	"rim_color": Color(0.706, 0.788, 0.851, 1.0),
	"rim_strength": 0.72,
	"rim_sharpness": 2.8,
}

## Brass valves, gauges and fittings (brief section 5.3), which is what the
## library's bronze already is: the same shader with its own values, untouched.
const BRASS_PARAMETERS := {}


## Riveted iron. One duplicate per surface, as the factory hands out.
static func iron() -> Material:
	return _metal(IRON_PARAMETERS)


## Brass fittings: the library's bronze as P2 tuned it.
static func brass() -> Material:
	return _metal(BRASS_PARAMETERS)


static func _metal(parameters: Dictionary) -> Material:
	var surface := SetpieceMeshFactoryScript.library_material(BRONZE)
	if surface == null:
		return null
	for key in parameters:
		surface.set_shader_parameter(key, parameters[key])
	return surface


## Matte clay, tinted: the blockout grammar every procedural box in the game
## already wears. `SetpieceMeshFactory.material` is the one adapter over it.
static func clay(color: Color, emission_energy: float = 0.0) -> Material:
	return SetpieceMeshFactoryScript.material(color, 0.0, 0.88, color, emission_energy)


## Worked stone: footings, plinths and a cut anchor.
static func stone() -> Material:
	return SetpieceMeshFactoryScript.library_material(WET_STONE)


## The deliberately ugly material. Only the declared *clearance* wears it: the
## ground a record reserves around itself is metadata made visible, not
## architecture, and it should be impossible to mistake for either.
static func placeholder() -> Material:
	return SetpieceMeshFactoryScript.library_material(PLACEHOLDER)


## Mark a node as procedural blockout standing in for something else. P4's rule,
## called rather than repeated.
static func mark(node: Node, replaced_by: String) -> void:
	BoardBlockoutScript.mark(node, replaced_by)


## The node name one authored socket ID gets. Godot forbids "." in a node name
## and silently rewrites one, so the ID is spelled with underscores here rather
## than left to the engine to mangle -- and the kit and the suite look a socket
## up through this one function, so they are looking up one thing and not two.
static func socket_node_name(socket_id: String) -> String:
	return BoardBlockoutScript.node_name("", socket_id, "")


## An empty `Node3D` at a socket's place: what a rig finds later, carrying the
## authored ID verbatim in `socket_id` because the node name has had its dots
## replaced. `kind` is the record's own socket kind, or "pivot" for a pivot a
## rig needs and no record names.
static func socket_node(parent: Node3D, socket_id: String, kind: String, location: Vector3) -> Node3D:
	var node := Node3D.new()
	node.name = socket_node_name(socket_id)
	node.position = location
	parent.add_child(node)
	node.set_meta("socket_id", socket_id)
	node.set_meta("socket_kind", kind)
	mark(node, "no model: a socket is where a model or a rig bone is attached, and stays an empty transform")
	return node


## A small visible marker under a socket, so a socket can be seen in a review
## capture. It is a *child* of the socket rather than the socket itself: the
## socket stays an empty transform whatever is drawn at it.
static func socket_marker(socket: Node3D, radius: float, height: float, tint: Color) -> MeshInstance3D:
	var marker := SetpieceMeshFactoryScript.cylinder(
		socket, "Marker", radius, radius, height, Vector3(0.0, height * 0.5, 0.0), clay(tint, 0.45), 10
	)
	mark(marker, "no model: the socket's marker is review geometry and never ships in a room")
	return marker


## The merged bounds of everything under a node that can draw, in that node's
## own space. The suite measures a built envelope with this.
static func bounds_of(node: Node3D) -> AABB:
	var bounds := AABB()
	var seen := false
	var pending: Array[Node] = [node]
	while not pending.is_empty():
		var current: Node = pending.pop_back()
		if current is VisualInstance3D:
			var visual := current as VisualInstance3D
			var box: AABB = node.global_transform.affine_inverse() * (visual.global_transform * visual.get_aabb())
			bounds = bounds.merge(box) if seen else box
			seen = true
		for child in current.get_children():
			pending.append(child)
	return bounds
