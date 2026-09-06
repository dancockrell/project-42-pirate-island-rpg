class_name BoardDevelopment
extends RefCounted

## P13: what S17 raises and S16 builds, standing on the board.
##
## The board already drew the island, the roads, the party and the marching
## forces. It did not draw a single thing anybody had *built* -- a faction acted
## on `Goal::Develop`, a yard turned out a machine, and the island looked
## exactly the same. This is that half.
##
## It draws and it does not decide. Every fact here comes from one of two places
## and from nowhere else:
##
## * the **snapshot**, for what is standing: `building_instances` and
##   `machine_instances`, the two read-only keys P13 put on the state dictionary
##   beside B16's and B19's registry arrays. Each instance says which record it
##   was raised from, which cell it stands on, whose it is, and where it is in
##   its life. Nothing else crosses -- no hit points, no fuel, no cost;
## * the **content catalog**, for what a record looks like: the same authored
##   `building.*` and `machine.*` records P12's kits already build from. The
##   envelope, the tiers and the sockets are read from the record, exactly once,
##   through `BuildingBlockoutKit` and `MachineBlockoutKit`. **No geometry is
##   written here.** This file calls P12's `build` seam and places what it
##   returns; a metre of a building's shape never appears in this file, because
##   two files that both knew how tall a machine shop stands would be two
##   answers to one question.
##
## ## What is a decision, and what is not
##
## Where a thing stands *inside* a cell is nobody's decision yet. Brief section
## 4 gives a room a footprint and spawn sockets; it does not say where a faction
## puts its second shed, and O3 (exact standard building dimensions) is Open, so
## the arithmetic that would settle it does not exist. Every constant below is
## therefore named and marked `needs decision`, and the suite asserts that they
## still say so. What is *not* a decision, and is read rather than chosen: which
## cell a thing stands on (the instance says), whose it is (the instance says),
## how big it is (the record and the kit say), and which authored socket it
## takes at the room distance (the cell's own board block says).

const BlockoutKitScript := preload("res://scripts/blockouts/blockout_kit.gd")
const BuildingBlockoutKitScript := preload("res://scripts/blockouts/building_blockout_kit.gd")
const MachineBlockoutKitScript := preload("res://scripts/blockouts/machine_blockout_kit.gd")
const BoardBlockoutScript := preload("res://scripts/board/board_blockout.gd")
const BoardPaletteScript := preload("res://scripts/board/board_palette.gd")
const SetpieceMeshFactoryScript := preload("res://scripts/world/setpiece_mesh_factory.gd")

## The two kinds of thing this file stands on a cell, and the node-name prefix
## each takes. A node is named from the *instance* ID and never from the record:
## two sheds raised from one record are two things on the board, and a board
## that named them both after the record would draw one.
const BUILDING := "building"
const MACHINE := "machine"
const BUILDING_PREFIX := "Building_"
const MACHINE_PREFIX := "Machine_"

## `BuildingState`, spelled as `godot_bridge.rs` hands it over. The board reads
## two things off it and nothing more: whether the thing is finished, and whether
## it is standing at all.
const STATE_UNDER_CONSTRUCTION := "under_construction"
const STATE_RUINED := "ruined"

## The tier argument `BuildingBlockoutKit.build` takes for the two ways a
## building is drawn. Zero is the kit's own word for "every authored tier", so a
## finished building stands at its full height; one is its first tier alone,
## which is what a building that has not been finished has actually got built.
const TIER_FULL := 0
const TIER_FIRST := 1

## **needs decision (O3).** Where the things a faction has built stand inside a
## cell, and how far apart.
##
## They stand in rows laid along the cell's width, one row per kind: the
## buildings across the back of the room, and the machines on open ground in
## front of them. The party's miniature stands at the cell's centre and a
## marching column stands at the road mouth, so those are the two places a row
## must keep off.
##
## Two rows rather than one, and this is the part that is a decision: a
## `mechanical_dog` is 2.3 m long and `building.machine_shop` is over twenty
## metres wide, so a machine standing in line with the sheds is a machine
## standing behind one. A yard puts its buildings at the back and parks what they
## turned out in front of them, and until somebody decides otherwise that is what
## this draws. `ROW_DEPTH_SHARE` is how far each kind's row stands from the
## cell's centre as a share of the footprint's depth -- negative is toward the
## front -- and `ROW_GAP_METRES` is the clear ground between two things in a row.
##
## This is placement, not architecture: how a building is *shaped* comes from
## the record through P12's kit, and how a room is shaped comes from B11's
## footprint. Where a second shed goes when a faction raises one is a design
## decision nobody has taken, and O3 is the Open item it belongs to.
const ROW_DEPTH_SHARE := {BUILDING: 0.28, MACHINE: -0.22}
const ROW_GAP_METRES := 3.0

## **needs decision (O3).** The order things stand in the row: every building
## first, then every machine, each group in the order the snapshot gives (which
## is Rust's `BTreeMap` order over the instance IDs -- total, and identical on
## every machine, so a board built twice from one snapshot is the same board).
## A faction laying its yard out by function rather than by name is a decision
## nobody has taken.
const ORDER_KINDS: Array[String] = [BUILDING, MACHINE]

## **needs decision (O3).** Which authored spawn role a thing occupies at the
## room distance. B11 gives a room four roles -- player, occupant, hostile,
## item -- and none of them is "the shed in the corner", so a building is placed
## as an occupant of its room and a machine as an item standing in it. When the
## instances outnumber the sockets of their role the row wraps around them,
## because a room that has run out of authored places for a building is a
## content question and this file must not answer it by inventing a place.
const ROOM_ROLE := {BUILDING: "occupant", MACHINE: "item"}

## **needs decision (O3).** The apron every standing thing is put down on: how
## far it reaches past what stands on it, how small it is allowed to get, and how
## thick it is drawn, all in metres.
##
## The apron is where a thing's *faction* is said, and it is under everything
## rather than only under the machines because the material language P12 owns is
## not a faction's to repaint: `building.machine_shop` is iron walls and an iron
## roof by C10's record, so a wash applied to its surfaces would say nothing at
## all, and a machine is iron and brass whoever built it. The ground a thing is
## standing on can carry the wash without any of that being touched.
##
## The minimum is what makes a machine visible at all: a `mechanical_dog` is
## 2.3 m long and the world distance frames an island 280 m across, so a machine
## drawn at its own size is two pixels of a capture and a board that draws
## nothing is a board that lies about what is standing on it.
const APRON_MARGIN_METRES := 1.6
const APRON_MINIMUM_METRES := 6.0
const APRON_THICKNESS_METRES := 0.35

## Every metre and share this file writes down because no record and no decision
## carries one. The suite asserts that this covers each of them and that each
## still says it is a decision nobody has taken.
const OPEN_DIMENSIONS := {
	"ROW_DEPTH_SHARE": "needs decision: O3, where inside a cell a faction stands what it has built -- the sheds at the back, the machines parked in front of them",
	"ROW_GAP_METRES": "needs decision: O3, the clear ground between two things standing in one cell",
	"ORDER_KINDS": "needs decision: O3, the order things stand in a cell's row",
	"ROOM_ROLE": "needs decision: O3, which authored spawn role a building and a machine occupy at the room distance",
	"APRON_MARGIN_METRES": "needs decision: O3, how far the ground a thing is standing on reaches past it",
	"APRON_MINIMUM_METRES": "needs decision: O3, the smallest mark a thing may leave on the board, so a two-metre machine can be seen at all",
	"APRON_THICKNESS_METRES": "needs decision: O3, how thick the ground a thing is standing on is drawn",
}


## The node one instance is drawn as, named from its own stable ID. Godot
## rewrites a "." in a node name, so the ID is spelled through
## `BoardBlockout.node_name` exactly as every other board node is -- one
## function, so the board and the suite look one thing up and not two.
static func node_name_for(kind: String, instance_id: String) -> String:
	if kind == MACHINE:
		return BoardBlockoutScript.node_name(MACHINE_PREFIX, instance_id, "machine_instance.")
	return BoardBlockoutScript.node_name(BUILDING_PREFIX, instance_id, "building_instance.")


## What a thing looks like right now, as one string. When this changes the node
## is rebuilt from the record; when it does not, the node is only moved. A
## building that finishes changes from the first tier in the placeholder
## material to the full kit, and that is exactly what this catches.
static func signature_of(kind: String, instance: Dictionary) -> String:
	return "%s|%s|%s|%s|%s" % [
		kind,
		str(instance.get("def_id", "")),
		str(instance.get("state", "")),
		str(instance.get("tier", 0)),
		str(instance.get("faction_id", "")),
	]


## Whether this instance is drawn at all. A ruin is not: brief section 8's
## envelope belongs to a building that is standing, `BuildingState::Ruined` is
## terminal, and drawing a ruin as a whole building would be the board saying
## something the simulation did not.
static func is_standing(instance: Dictionary) -> bool:
	return str(instance.get("state", "")) != STATE_RUINED


## One instance, built from its authored record by P12's kit and washed in the
## colour of the faction that holds it. Returns null when the catalog has no
## record for the `def_id` the save names, which is a content error and not
## something to draw a box for.
##
## The two states a building can be drawn in are the card's: **under
## construction** is the kit's first tier alone, wearing the deliberately ugly
## placeholder material, because that is what is true -- one storey of work has
## been done and what will stand there is not there yet; **standing** is the
## full kit at every authored tier, washed in its faction's colour.
static func build_instance(kind: String, instance: Dictionary, record: Dictionary, room_footprint: Dictionary) -> Node3D:
	if record.is_empty():
		return null
	var wash: Color = BoardPaletteScript.controller_tint(str(instance.get("faction_id", "")))
	var node: Node3D = null
	if kind == MACHINE:
		if not MachineBlockoutKitScript.can_build(record):
			return null
		node = MachineBlockoutKitScript.build(record)
		if node == null:
			return null
	else:
		var under_construction := str(instance.get("state", "")) == STATE_UNDER_CONSTRUCTION
		node = BuildingBlockoutKitScript.build(record, room_footprint, TIER_FIRST if under_construction else TIER_FULL)
		if node == null:
			return null
		if under_construction:
			_wear_placeholder(node)
		else:
			_wash(node, BuildingBlockoutKitScript.tint_of(record), wash)
	_stand_on_apron(node, wash, apron_metres(kind, record, room_footprint))
	node.name = node_name_for(kind, str(instance.get("id", "")))
	node.set_meta("instance_id", str(instance.get("id", "")))
	node.set_meta("instance_kind", kind)
	node.set_meta("def_id", str(instance.get("def_id", "")))
	node.set_meta("cell_id", str(instance.get("cell_id", "")))
	node.set_meta("faction_id", str(instance.get("faction_id", "")))
	node.set_meta("instance_state", str(instance.get("state", "")))
	node.set_meta("signature", signature_of(kind, instance))
	return node


## The places one cell's things stand on, at the world and route distances: one
## row per kind laid along the cell's width, each thing given its own apron's
## width and `ROW_GAP_METRES` of clear ground, at the depth `ROW_DEPTH_SHARE`
## names for its kind.
##
## `grounds` is each thing's own apron in metres and `kinds` what each of them
## is, both in the order they stand, so the rows are measured off the records
## rather than off a slot size this file would have had to invent, and each place
## is drawn back inside the cell's footprint the same way a room-distance socket
## is. The answer is in the order it was asked, whichever row each entry fell in.
static func row_places(cell: Dictionary, grounds: Array, kinds: Array) -> Array[Vector3]:
	var places: Array[Vector3] = []
	places.resize(grounds.size())
	if grounds.is_empty():
		return places
	var centre: Vector3 = BoardBlockoutScript.centre_of(cell)
	var footprint: Dictionary = cell.get("footprint", {})
	var depth := float(footprint.get("depth_metres", 0.0))
	for row_kind in ORDER_KINDS:
		var indices: Array[int] = []
		var total := 0.0
		for index in grounds.size():
			if str(kinds[index]) != row_kind:
				continue
			indices.append(index)
			total += (grounds[index] as Vector2).x
		if indices.is_empty():
			continue
		total += ROW_GAP_METRES * float(indices.size() - 1)
		var cursor := centre.x - total * 0.5
		var back := centre.z - depth * float(ROW_DEPTH_SHARE.get(row_kind, 0.0))
		for index in indices:
			var own: Vector2 = grounds[index]
			places[index] = _drawn_inside(cell, Vector3(cursor + own.x * 0.5, centre.y, back), own)
			cursor += own.x + ROW_GAP_METRES
	return places


## Where one thing stands at the room distance: on the authored spawn socket its
## role names, in the cell's own authored order, wrapping when there are more
## things than sockets, skipping a socket something in `taken` is already
## standing on, and drawn back inside the footprint where what stands on it
## would otherwise cross the edge. The socket is the intent and content owns it:
## a thing stands where content put it, not where a layout rule put it, and the
## only things that move it are the room's own boundary and ground somebody else
## is already standing on.
##
## Falls back to the row place when the cell declares no socket of that role,
## because a room with no `item` socket is a content shape this file must not
## repair by inventing a position.
static func room_place(cell: Dictionary, kind: String, index: int, ground: Vector2 = Vector2.ZERO, taken: Array = []) -> Vector3:
	var role := str(ROOM_ROLE.get(kind, ""))
	var matching: Array[Vector3] = []
	for spawn in cell.get("spawn_points", []):
		if str((spawn as Dictionary).get("role", "")) == role:
			matching.append((spawn as Dictionary).get("position_metres", Vector3.ZERO) as Vector3)
	if matching.is_empty():
		return Vector3.INF
	var centre: Vector3 = BoardBlockoutScript.centre_of(cell)
	var lift := BoardBlockoutScript.TILE_THICKNESS_METRES * 0.5
	var first := Vector3.ZERO
	for step in matching.size():
		var seat := (index + step) % matching.size()
		var point := _drawn_inside(cell, centre + matching[seat] + Vector3(0.0, lift, 0.0), ground)
		if step == 0:
			first = point
		if not _overlaps(point, ground, taken):
			return point
	# Every authored socket of this role is already under something. The room is
	# fuller than content gave it places for, which is a content question -- how
	# much a cell may hold is `CELL_CAPACITY_CELLS`, S3's -- so this stands on the
	# socket it was asked for rather than inventing a place that is not authored.
	return first


## The ground a thing would take up, as the rectangle the two checks above use.
static func ground_rect(point: Vector3, ground: Vector2) -> Rect2:
	return Rect2(point.x - ground.x * 0.5, point.z - ground.y * 0.5, ground.x, ground.y)


static func _overlaps(point: Vector3, ground: Vector2, taken: Array) -> bool:
	if ground == Vector2.ZERO:
		return false
	var mine := ground_rect(point, ground)
	for other in taken:
		if mine.intersects(other as Rect2):
			return true
	return false


## The socket the record names, drawn back inside the room where what stands on
## it would otherwise cross the footprint's edge.
##
## Brief section 18 is plain that nothing essential may reach outside the
## declared box, and an authored spawn socket was authored for a person standing
## on it, not for a building four twelfths of the room wide: `building.machine_shop`
## on the terrace's first occupant socket would hang over the kerb by metres. So
## the socket stays the intent and the footprint stays the boundary, and where
## the two disagree the boundary wins. A room too small to hold what somebody
## built on it is centred rather than clipped -- that is a content question (how
## much a cell can hold is `CELL_CAPACITY_CELLS`, S3's) and this file must not
## answer it by pushing geometry off the island.
static func _drawn_inside(cell: Dictionary, point: Vector3, ground: Vector2) -> Vector3:
	if ground == Vector2.ZERO:
		return point
	var centre: Vector3 = BoardBlockoutScript.centre_of(cell)
	var footprint: Dictionary = cell.get("footprint", {})
	var room := Vector2(float(footprint.get("width_metres", 0.0)), float(footprint.get("depth_metres", 0.0)))
	var slack := Vector2(maxf((room.x - ground.x) * 0.5, 0.0), maxf((room.y - ground.y) * 0.5, 0.0))
	return Vector3(
		clampf(point.x, centre.x - slack.x, centre.x + slack.x),
		point.y,
		clampf(point.z, centre.z - slack.y, centre.z + slack.y)
	)


## The ground one instance stands on, in metres, measured off the envelope its
## record and P12's kit give it. Never smaller than `APRON_MINIMUM_METRES`.
static func apron_metres(kind: String, record: Dictionary, room_footprint: Dictionary) -> Vector2:
	var envelope: Vector3 = MachineBlockoutKitScript.envelope_metres(record) if kind == MACHINE \
		else BuildingBlockoutKitScript.envelope_metres(record, room_footprint, TIER_FULL)
	return Vector2(
		maxf(envelope.x + APRON_MARGIN_METRES * 2.0, APRON_MINIMUM_METRES),
		maxf(envelope.z + APRON_MARGIN_METRES * 2.0, APRON_MINIMUM_METRES)
	)


## How wide one instance stands, in metres: its apron, which is never narrower
## than the thing on it, so a row spaced by this never overlaps.
static func width_of(kind: String, record: Dictionary, room_footprint: Dictionary) -> float:
	if record.is_empty():
		return 0.0
	return apron_metres(kind, record, room_footprint).x


## The faction's wash, said under the thing rather than over it. See
## `APRON_MARGIN_METRES` for why it is the ground and not the walls.
static func _stand_on_apron(node: Node3D, wash: Color, ground: Vector2) -> void:
	var apron := SetpieceMeshFactoryScript.box(
		node,
		"FactionApron",
		Vector3(ground.x, APRON_THICKNESS_METRES, ground.y),
		Vector3(0.0, APRON_THICKNESS_METRES * 0.5, 0.0),
		BlockoutKitScript.clay(wash)
	)
	BlockoutKitScript.mark(apron, "the worked ground a building or a machine stands on, which a room's finished floor will carry")


## Repaint the surfaces the kit drew in the record's own colour, and only those.
##
## `BuildingBlockoutKit.tint_of` is the colour the kit used for a record's clay,
## taken from the first concept key in the record's `faction_compatibility`;
## every other surface it drew is worked stone or iron, which a faction does not
## own. So this replaces exactly the surfaces wearing that one colour and leaves
## the material language P12 owns alone. The instance's faction is the one on the
## save, never the record's compatibility list: a captured building flies the
## flag of whoever holds it.
static func _wash(node: Node3D, record_tint: Color, wash: Color) -> void:
	for mesh_instance in _mesh_instances(node):
		var surface := mesh_instance.mesh.material as ShaderMaterial
		if surface == null:
			continue
		var albedo: Variant = surface.get_shader_parameter("albedo")
		if albedo is Color and (albedo as Color).is_equal_approx(record_tint):
			mesh_instance.mesh.material = BlockoutKitScript.clay(wash)


## The deliberately ugly material over everything the kit built, which is what a
## building that is not finished is: hours of work remain and what will stand
## there is not there yet. `BlockoutKit.placeholder` is the same material P12
## reserved for metadata made visible, and it is used here for the same reason --
## it must be impossible to mistake for architecture.
static func _wear_placeholder(node: Node3D) -> void:
	for mesh_instance in _mesh_instances(node):
		mesh_instance.mesh.material = BlockoutKitScript.placeholder()


static func _mesh_instances(node: Node3D) -> Array[MeshInstance3D]:
	var found: Array[MeshInstance3D] = []
	var pending: Array[Node] = [node]
	while not pending.is_empty():
		var current: Node = pending.pop_back()
		if current is MeshInstance3D and (current as MeshInstance3D).mesh != null:
			found.append(current as MeshInstance3D)
		for child in current.get_children():
			pending.append(child)
	return found
