class_name MachineBlockoutKit
extends RefCounted

## D10: the first machine family blockouts, built from a C14 record alone.
##
## Brief section 5.3 gives the material language -- riveted iron and steel, cast
## housings, structural timber, brass valves and gauges, boiler tanks, exposed
## connecting rods, repair plates, bold silhouettes readable from the fixed
## isometric camera -- and section 18 asks that machines keep future-ready
## pivots, sockets and metadata while animation is deferred. So: no animation,
## no rig, and every repair socket the record names plus every pivot a rig will
## need as an empty `Node3D` a later card can find by name.
##
## **Every dimension here is a named constant marked needs decision, and that is
## the record's own doing.** C14 authored `machine.mechanical_dog` and
## `machine.steam_wagon` with every dimension held at zero and an
## `open_dimensions` map naming each decision, because brief section 20 leaves
## the exact standard dimensions Open and a number invented in a content file
## would read as a decision somebody had taken. The same rule applies here: the
## two envelopes below are this kit's placeholders, they say so, and the suite
## holds them to saying so. Every other measurement in this file is a
## proportion of its family's envelope, so when the dimensions are decided the
## two constants change and the blockouts follow.

const BlockoutKitScript := preload("res://scripts/blockouts/blockout_kit.gd")
const SetpieceMeshFactoryScript := preload("res://scripts/world/setpiece_mesh_factory.gd")
const BoardPaletteScript := preload("res://scripts/board/board_palette.gd")

## **needs decision** -- brief section 20, exact standard dimensions, named in
## each record's own `open_dimensions` as `operational_footprint_cells`,
## `navigation_width_cells` and `turning_clearance_cells`. The overall envelope
## a family's blockout is built inside, as (width, height, length) in metres,
## with length along +Z, which is the direction the machine faces.
const FAMILY_ENVELOPE_METRES := {
	"mechanical_dog": Vector3(1.05, 1.55, 2.30),
	"steam_wagon": Vector3(2.60, 3.05, 5.40),
}

## **needs decision.** The pivots a rig will need and no record names. C14's
## records carry `repair_sockets` -- where a machine is *worked on* -- and brief
## section 18 asks separately for future-ready pivots; there is no authored rig
## vocabulary anywhere in the tree to take names from, so the names are derived
## from the record's own sockets: a dog's two leg linkages become four leg
## pivots, left and right, and a wagon's drive train becomes two axles with a
## wheel pivot at each end. `derived_from` names the authored socket each pivot
## answers to, so when a rig vocabulary is authored this map is replaced by it
## and the derivation is on record rather than in somebody's head.
const PIVOT_PLAN := {
	"mechanical_dog": [
		{"suffix": "fore_left", "derived_from": "fore_leg_linkage"},
		{"suffix": "fore_right", "derived_from": "fore_leg_linkage"},
		{"suffix": "hind_left", "derived_from": "hind_leg_linkage"},
		{"suffix": "hind_right", "derived_from": "hind_leg_linkage"},
	],
	"steam_wagon": [
		{"suffix": "axle_front", "derived_from": "drive_train"},
		{"suffix": "axle_rear", "derived_from": "drive_train"},
		{"suffix": "wheel_front_left", "derived_from": "drive_train"},
		{"suffix": "wheel_front_right", "derived_from": "drive_train"},
		{"suffix": "wheel_rear_left", "derived_from": "drive_train"},
		{"suffix": "wheel_rear_right", "derived_from": "drive_train"},
	],
}

## **needs decision.** How big a repair socket's review marker is, in metres. A
## repair socket is an empty transform in the game; this is what a capture draws
## at it so it can be seen.
const SOCKET_MARKER_RADIUS_METRES := 0.14
const SOCKET_MARKER_HEIGHT_METRES := 0.20

## Every metre this kit writes down because no record carries one. The suite
## asserts that this dictionary covers each of them and that each still says it
## is a decision nobody has taken.
const OPEN_DIMENSIONS := {
	"FAMILY_ENVELOPE_METRES": "needs decision: brief section 20, exact standard dimensions -- every C14 record holds operational footprint, navigation width, turning clearance, maximum slope and wreck footprint at zero and names each in its own open_dimensions map",
	"PIVOT_PLAN": "needs decision: brief section 18 asks for future-ready pivots and no authored rig vocabulary exists, so a pivot's name is derived from the repair socket it answers to",
	"SOCKET_MARKER_RADIUS_METRES": "needs decision: review-only marker size for a repair socket, which is an empty transform in the game",
	"SOCKET_MARKER_HEIGHT_METRES": "needs decision: review-only marker size for a repair socket, which is an empty transform in the game",
}

## Structural timber and canvas (brief section 5.3), for a wagon's bed and
## boards. Presentation, not a decision.
const TIMBER := Color("6d5a44")
const CANVAS := Color("9a9079")


## The family this record belongs to, which the record states outright.
static func family_of(record: Dictionary) -> String:
	return str(record.get("family", ""))


## The envelope a family's blockout is built inside, in metres. Zero when the
## family has no authored blockout yet -- six of the eight families are
## unauthored in content and none of them has one here either.
static func envelope_metres(record: Dictionary) -> Vector3:
	return FAMILY_ENVELOPE_METRES.get(family_of(record), Vector3.ZERO)


## True when this kit can draw the record's family. Six families are unauthored
## in `content/machines/` and this refuses them by name rather than drawing a
## box and calling it a machine.
static func can_build(record: Dictionary) -> bool:
	return FAMILY_ENVELOPE_METRES.has(family_of(record))


## The full pivot IDs a record's family gets, in the plan's order.
static func pivot_ids(record: Dictionary) -> Array[String]:
	var machine_id := str(record.get("id", ""))
	var result: Array[String] = []
	for pivot in PIVOT_PLAN.get(family_of(record), []):
		result.append("pivot.%s.%s" % [machine_id.trim_prefix("machine."), str((pivot as Dictionary).get("suffix", ""))])
	return result


## Build one authored machine record.
##
## The returned `Node3D` stands with its origin on the ground under the centre
## of the machine, facing +Z. `Chassis` carries the geometry; `Sockets` carries
## one empty `Node3D` per authored repair socket; `Pivots` carries one per
## derived pivot. Nothing is animated and nothing is rigged: brief section 18
## defers animation and asks only that the attachment points exist.
static func build(record: Dictionary) -> Node3D:
	var machine_id := str(record.get("id", ""))
	var family := family_of(record)
	if machine_id.is_empty() or not can_build(record):
		push_error("MachineBlockoutKit has no blockout for family: %s" % family)
		return null
	var envelope: Vector3 = FAMILY_ENVELOPE_METRES[family]

	var root_node := Node3D.new()
	root_node.name = BlockoutKitScript.socket_node_name(machine_id)
	root_node.set_meta("machine_id", machine_id)
	root_node.set_meta("family", family)
	root_node.set_meta("envelope_metres", envelope)
	BlockoutKitScript.mark(root_node, "the machine's model and rig, to brief section 18's asset contract (%s)" % machine_id)

	var chassis := Node3D.new()
	chassis.name = "Chassis"
	root_node.add_child(chassis)
	BlockoutKitScript.mark(chassis, "the machine's model (%s)" % machine_id)
	match family:
		"mechanical_dog":
			_build_dog(chassis, envelope)
		"steam_wagon":
			_build_wagon(chassis, envelope)

	_build_sockets(root_node, record, envelope)
	_build_pivots(root_node, record, envelope)
	return root_node


## A small four-legged automaton: a riveted-iron body over a boiler slung along
## its length, a sensing head, and four legs of exposed connecting rods standing
## on the pivots a rig will drive.
static func _build_dog(chassis: Node3D, envelope: Vector3) -> void:
	var iron := BlockoutKitScript.iron()
	var body_height := envelope.y * 0.34
	var body_y := envelope.y * 0.74
	var body := SetpieceMeshFactoryScript.box(
		chassis, "Body", Vector3(envelope.x * 0.78, body_height, envelope.z * 0.60), Vector3(0.0, body_y, 0.0), iron
	)
	BlockoutKitScript.mark(body, "the automaton's cast body housing")

	# The boiler: the machine's reason for existing, carried where a spine goes.
	var boiler := SetpieceMeshFactoryScript.cylinder(
		chassis,
		"Boiler",
		envelope.x * 0.28,
		envelope.x * 0.28,
		envelope.z * 0.46,
		Vector3(0.0, body_y + body_height * 0.12, -envelope.z * 0.06),
		iron,
		12,
		Vector3(90.0, 0.0, 0.0)
	)
	BlockoutKitScript.mark(boiler, "the automaton's boiler tank")

	var stack := SetpieceMeshFactoryScript.cylinder(
		chassis,
		"ExhaustStack",
		envelope.x * 0.09,
		envelope.x * 0.11,
		envelope.y * 0.16,
		Vector3(0.0, envelope.y - envelope.y * 0.08, -envelope.z * 0.24),
		BlockoutKitScript.brass(),
		10
	)
	BlockoutKitScript.mark(stack, "the automaton's exhaust and its brass fittings")

	var head := SetpieceMeshFactoryScript.box(
		chassis,
		"SensorHead",
		Vector3(envelope.x * 0.52, envelope.y * 0.24, envelope.z * 0.19),
		Vector3(0.0, body_y + body_height * 0.34, envelope.z * 0.38),
		iron
	)
	BlockoutKitScript.mark(head, "the automaton's head housing, where a lamp and an alarm sit")

	# Repair plates: brief section 5.3 asks for hard use, and a plate on a flank
	# is the cheapest honest way for a blockout to read as a repaired machine.
	for side in [-1.0, 1.0]:
		var plate := SetpieceMeshFactoryScript.box(
			chassis,
			"RepairPlate%s" % ("L" if side < 0.0 else "R"),
			Vector3(envelope.x * 0.03, body_height * 0.62, envelope.z * 0.22),
			Vector3(side * envelope.x * 0.37, body_y, -envelope.z * 0.10),
			iron
		)
		BlockoutKitScript.mark(plate, "a riveted repair plate on the automaton's flank")

	# Four legs, standing where the pivots stand.
	for place in _dog_leg_places(envelope):
		var leg := Node3D.new()
		leg.name = "Leg%s" % str(place.name)
		leg.position = place.position as Vector3
		chassis.add_child(leg)
		BlockoutKitScript.mark(leg, "the automaton's leg, driven by the pivot of the same name")
		var hip_height: float = place.position.y
		var upper_length: float = hip_height * 0.56
		var upper := SetpieceMeshFactoryScript.cylinder(
			leg,
			"UpperRod",
			envelope.x * 0.10,
			envelope.x * 0.11,
			upper_length,
			Vector3(0.0, -upper_length * 0.5, 0.0),
			iron,
			8,
			Vector3(0.0, 0.0, float(place.lean) * 12.0)
		)
		BlockoutKitScript.mark(upper, "the automaton's upper connecting rod")
		var lower_length: float = hip_height - upper_length
		var lower := SetpieceMeshFactoryScript.cylinder(
			leg,
			"LowerRod",
			envelope.x * 0.08,
			envelope.x * 0.09,
			lower_length,
			Vector3(float(place.lean) * envelope.x * 0.04, -upper_length - lower_length * 0.5, 0.0),
			iron,
			8
		)
		BlockoutKitScript.mark(lower, "the automaton's lower connecting rod")
		var foot := SetpieceMeshFactoryScript.cylinder(
			leg,
			"Foot",
			envelope.x * 0.13,
			envelope.x * 0.13,
			envelope.y * 0.05,
			Vector3(float(place.lean) * envelope.x * 0.07, -hip_height + envelope.y * 0.025, 0.0),
			BlockoutKitScript.clay(TIMBER),
			10
		)
		BlockoutKitScript.mark(foot, "the automaton's foot pad, leather over iron")


## Where the four legs stand, and which way each leans. The suffixes are the
## pivot plan's, so a leg and its pivot always agree.
static func _dog_leg_places(envelope: Vector3) -> Array:
	var hip := envelope.y * 0.56
	return [
		{"name": "ForeLeft", "position": Vector3(-envelope.x * 0.30, hip, envelope.z * 0.28), "lean": -1.0},
		{"name": "ForeRight", "position": Vector3(envelope.x * 0.30, hip, envelope.z * 0.28), "lean": 1.0},
		{"name": "HindLeft", "position": Vector3(-envelope.x * 0.30, hip, -envelope.z * 0.30), "lean": -1.0},
		{"name": "HindRight", "position": Vector3(envelope.x * 0.30, hip, -envelope.z * 0.30), "lean": 1.0},
	]


## A steam-driven freight wagon: a boiler and stack over the drive train at the
## front, a timber bed behind it, a water tank across the frame, and four wheels
## on two axles.
static func _build_wagon(chassis: Node3D, envelope: Vector3) -> void:
	var iron := BlockoutKitScript.iron()
	var wheel := _wagon_wheel_metrics(envelope)
	var frame_y: float = wheel.rear_radius * 1.25
	var frame := SetpieceMeshFactoryScript.box(
		chassis,
		"Frame",
		Vector3(envelope.x * 0.82, envelope.y * 0.09, envelope.z * 0.88),
		Vector3(0.0, frame_y, 0.0),
		iron
	)
	BlockoutKitScript.mark(frame, "the wagon's riveted iron frame")

	var bed_height := envelope.y * 0.30
	var bed := SetpieceMeshFactoryScript.box(
		chassis,
		"Bed",
		Vector3(envelope.x * 0.80, bed_height, envelope.z * 0.46),
		Vector3(0.0, frame_y + bed_height * 0.5, -envelope.z * 0.21),
		BlockoutKitScript.clay(TIMBER)
	)
	BlockoutKitScript.mark(bed, "the wagon's timber bed")
	for side in [-1.0, 1.0]:
		var board := SetpieceMeshFactoryScript.box(
			chassis,
			"BedBoard%s" % ("L" if side < 0.0 else "R"),
			Vector3(envelope.x * 0.04, bed_height * 0.75, envelope.z * 0.46),
			Vector3(side * envelope.x * 0.40, frame_y + bed_height * 1.05, -envelope.z * 0.21),
			BlockoutKitScript.clay(CANVAS)
		)
		BlockoutKitScript.mark(board, "the wagon's side board, timber and canvas")

	var boiler_height := envelope.y * 0.34
	var boiler := SetpieceMeshFactoryScript.cylinder(
		chassis,
		"Boiler",
		envelope.x * 0.27,
		envelope.x * 0.29,
		boiler_height,
		Vector3(0.0, frame_y + boiler_height * 0.5, envelope.z * 0.28),
		iron,
		14
	)
	BlockoutKitScript.mark(boiler, "the wagon's boiler")
	var stack := SetpieceMeshFactoryScript.cylinder(
		chassis,
		"Stack",
		envelope.x * 0.10,
		envelope.x * 0.13,
		envelope.y - (frame_y + boiler_height),
		Vector3(0.0, (frame_y + boiler_height + envelope.y) * 0.5, envelope.z * 0.28),
		BlockoutKitScript.brass(),
		12
	)
	BlockoutKitScript.mark(stack, "the wagon's stack and its brass fittings")

	var tank := SetpieceMeshFactoryScript.cylinder(
		chassis,
		"WaterTank",
		envelope.x * 0.21,
		envelope.x * 0.21,
		envelope.x * 0.66,
		Vector3(0.0, frame_y + envelope.y * 0.12, envelope.z * 0.06),
		iron,
		12,
		Vector3(0.0, 0.0, 90.0)
	)
	BlockoutKitScript.mark(tank, "the wagon's water tank, across the frame behind the boiler")

	for axle in _wagon_axle_places(envelope, wheel):
		var bar := SetpieceMeshFactoryScript.cylinder(
			chassis,
			"Axle%s" % str(axle.name),
			envelope.x * 0.05,
			envelope.x * 0.05,
			envelope.x * 0.90,
			Vector3(0.0, float(axle.radius), float(axle.z)),
			iron,
			8,
			Vector3(0.0, 0.0, 90.0)
		)
		BlockoutKitScript.mark(bar, "the wagon's axle, driven by the pivot of the same name")
		for side in [-1.0, 1.0]:
			var tyre := SetpieceMeshFactoryScript.cylinder(
				chassis,
				"Wheel%s%s" % [str(axle.name), "L" if side < 0.0 else "R"],
				float(axle.radius),
				float(axle.radius),
				envelope.x * 0.12,
				Vector3(side * envelope.x * 0.44, float(axle.radius), float(axle.z)),
				iron,
				16,
				Vector3(0.0, 0.0, 90.0)
			)
			BlockoutKitScript.mark(tyre, "the wagon's wheel, iron tyre over a timber rim")


static func _wagon_wheel_metrics(envelope: Vector3) -> Dictionary:
	return {"front_radius": envelope.y * 0.20, "rear_radius": envelope.y * 0.28}


static func _wagon_axle_places(envelope: Vector3, wheel: Dictionary) -> Array:
	return [
		{"name": "Front", "z": envelope.z * 0.30, "radius": float(wheel.front_radius)},
		{"name": "Rear", "z": -envelope.z * 0.30, "radius": float(wheel.rear_radius)},
	]


## Every repair socket the record names, as an empty `Node3D` on the part it
## belongs to. A socket the kit has no place for stands at the machine's centre
## of mass rather than being dropped: a record's socket always exists.
static func _build_sockets(root_node: Node3D, record: Dictionary, envelope: Vector3) -> void:
	var group := Node3D.new()
	group.name = "Sockets"
	root_node.add_child(group)
	BlockoutKitScript.mark(group, "no model: a repair socket is an empty transform a rig and a repair action find by name")
	var places := _socket_places(family_of(record), envelope)
	for socket_id in record.get("repair_sockets", []) as Array:
		var suffix := str(socket_id).get_slice(".", str(socket_id).get_slice_count(".") - 1)
		var node := BlockoutKitScript.socket_node(
			group, str(socket_id), "repair", places.get(suffix, Vector3(0.0, envelope.y * 0.5, 0.0))
		)
		node.set_meta("placed", places.has(suffix))
		BlockoutKitScript.socket_marker(
			node, SOCKET_MARKER_RADIUS_METRES, SOCKET_MARKER_HEIGHT_METRES, BoardPaletteScript.BRONZE
		)


## Where each authored repair socket sits, by its own last name segment. A
## family this kit cannot draw has no places, and every socket falls back to the
## centre of mass -- which is still an existing, findable transform.
static func _socket_places(family: String, envelope: Vector3) -> Dictionary:
	match family:
		"mechanical_dog":
			return {
				"boiler": Vector3(0.0, envelope.y * 0.86, -envelope.z * 0.06),
				"fore_leg_linkage": Vector3(0.0, envelope.y * 0.58, envelope.z * 0.28),
				"hind_leg_linkage": Vector3(0.0, envelope.y * 0.58, -envelope.z * 0.30),
			}
		"steam_wagon":
			var wheel := _wagon_wheel_metrics(envelope)
			return {
				"boiler": Vector3(0.0, envelope.y * 0.55, envelope.z * 0.28),
				"drive_train": Vector3(0.0, float(wheel.front_radius), envelope.z * 0.30),
				"bed_frame": Vector3(0.0, float(wheel.rear_radius) * 1.25, -envelope.z * 0.21),
				"water_tank": Vector3(0.0, float(wheel.rear_radius) * 1.25 + envelope.y * 0.12, envelope.z * 0.06),
			}
	return {}


## The pivots a rig will need, each an empty `Node3D` under `Pivots`, each
## carrying the authored socket its name was derived from.
static func _build_pivots(root_node: Node3D, record: Dictionary, envelope: Vector3) -> void:
	var group := Node3D.new()
	group.name = "Pivots"
	root_node.add_child(group)
	BlockoutKitScript.mark(group, "no rig: brief section 18 defers animation and asks only that the pivots exist and are findable")
	var family := family_of(record)
	var places := _pivot_places(family, envelope)
	var plan: Array = PIVOT_PLAN.get(family, [])
	var ids := pivot_ids(record)
	for index in range(plan.size()):
		var entry: Dictionary = plan[index]
		var suffix := str(entry.get("suffix", ""))
		var node := BlockoutKitScript.socket_node(
			group, ids[index], "pivot", places.get(suffix, Vector3(0.0, envelope.y * 0.5, 0.0))
		)
		node.set_meta("derived_from", "socket.%s.%s" % [str(record.get("id", "")), str(entry.get("derived_from", ""))])
		node.set_meta("needs_decision", OPEN_DIMENSIONS["PIVOT_PLAN"])


static func _pivot_places(family: String, envelope: Vector3) -> Dictionary:
	match family:
		"mechanical_dog":
			var places := {}
			for place in _dog_leg_places(envelope):
				places[str(place.name).to_snake_case()] = place.position
			return places
		"steam_wagon":
			var wheel := _wagon_wheel_metrics(envelope)
			var axles := _wagon_axle_places(envelope, wheel)
			var result := {}
			for axle in axles:
				var lower := str(axle.name).to_lower()
				result["axle_%s" % lower] = Vector3(0.0, float(axle.radius), float(axle.z))
				result["wheel_%s_left" % lower] = Vector3(-envelope.x * 0.44, float(axle.radius), float(axle.z))
				result["wheel_%s_right" % lower] = Vector3(envelope.x * 0.44, float(axle.radius), float(axle.z))
			return result
	return {}
