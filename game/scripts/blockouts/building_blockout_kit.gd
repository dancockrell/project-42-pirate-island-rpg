class_name BuildingBlockoutKit
extends RefCounted

## D9: a building's cube or multi-cube blockout, built from a C10 record alone.
##
## Brief section 8: a building is its declared envelope and nothing essential
## may extend outside it -- not walls, not roof masses, not stairs, not worker,
## defender, spawn or delivery positions. That rule is what this kit draws. The
## massing is one box per authored tier, stepped back as it rises so the record's
## `tier_states` are visible as a shape rather than as a number, and every socket
## the record declares stands inside the footprint as an empty `Node3D` a rig or
## a placement system can find by name.
##
## Every metre comes from the record or from the O3 footprint table by ID. A
## record declares its envelope in the abstract cells `CELL_CAPACITY_CELLS`
## counts, not in metres, so the metres are the record's *share of a room* --
## `footprint_cells / CELL_CAPACITY_CELLS` of the reference room's area out of
## `content/presentation/board.registry.json`, in that room's own aspect. Where
## no number exists at all -- a storey's height, because `height_class` is a
## needs-decision placeholder in all three records -- the constant below says so
## and `OPEN_DIMENSIONS` names the decision it is standing in for.
##
## Placement on the board is not this card's: the kit builds a subtree with its
## origin on the ground at the centre of the envelope, and hands it back.

const BlockoutKitScript := preload("res://scripts/blockouts/blockout_kit.gd")
const SetpieceMeshFactoryScript := preload("res://scripts/world/setpiece_mesh_factory.gd")
const BoardPaletteScript := preload("res://scripts/board/board_palette.gd")

## **needs decision** -- brief section 20, "Exact standard building dimensions".
## The share pool a record's `footprint_cells` and `clearance_cells` are counted
## against. It mirrors `CELL_CAPACITY_CELLS` in
## `godot-rust/src/strategy/building.rs`, which is where the number is decided;
## nothing exposes it through the bridge, so this is a mirror and not a second
## owner. A bridge verb that reads it would delete this constant.
const CELL_CAPACITY_CELLS := 12

## **needs decision** -- brief section 20, "Exact standard building dimensions".
## Which room out of the O3 table a building's cell share is measured against.
## The court is the entry the table describes as "an outdoor approach, landing
## or terrace with a road and a battle lane in it", which is the room a building
## is placed in; that it is *this* entry rather than another is a framing choice
## nobody has taken. When O3 closes, the six numbers move in the registry and
## this ID keeps pointing at one of them.
const REFERENCE_ROOM_FOOTPRINT_ID := "presentation.board.footprint.court"

## **needs decision** -- brief section 20, "Exact standard building dimensions".
## How tall one authored tier stands, in metres. Every C10 record's
## `height_class` is a needs-decision placeholder string, so there is no height
## in the data to read; a tier is a storey and a storey is this.
const STOREY_METRES := 4.6

## **needs decision.** How far each tier above the first is stepped in, as a
## share of the envelope. A framing choice: it makes a tier-three building read
## as three tiers from the fixed isometric camera without any tier leaving the
## declared box.
const TIER_SETBACK := 0.16

## **needs decision.** How thick the footing course and the roof cap are, as a
## share of a storey.
const FOOTING_SHARE := 0.18
const ROOF_CAP_SHARE := 0.12

## **needs decision.** How high a socket marker stands, in metres, and how wide.
## Review geometry: big enough to see at gameplay distance, small enough that a
## marker never reads as a wall.
const SOCKET_MARKER_RADIUS_METRES := 0.55
const SOCKET_MARKER_HEIGHT_METRES := 0.9

## **needs decision.** Where a socket sits between the centre of the footprint
## and its edge when the record's `offset_cells` is zero. A record counts a
## socket's offset in cells and `offset_cells < footprint_cells` is the rule
## S3 enforces, so the offset is read as a share of the footprint's half-extent
## -- and an offset of zero is the centre, which two sockets would share.
const SOCKET_INNER_SHARE := 0.24

## **needs decision.** How wide the line that draws the declared clearance is,
## as a share of the reserved ground's shorter side.
const CLEARANCE_LINE_SHARE := 0.045

## **needs decision.** How deep the course between two storeys stands, as a
## share of a storey.
const CORNICE_SHARE := 0.10

## **needs decision.** How wide the line that draws the declared footprint on
## the ground is, in metres.
const FOOTPRINT_LINE_METRES := 0.35

## Every metre this kit writes down because no record and no table carries one.
## The suite asserts that this dictionary covers each of them and that each one
## still says it is a decision nobody has taken.
const OPEN_DIMENSIONS := {
	"CELL_CAPACITY_CELLS": "needs decision: brief section 20, exact standard building dimensions -- a mirror of CELL_CAPACITY_CELLS in godot-rust/src/strategy/building.rs, which no bridge verb exposes",
	"REFERENCE_ROOM_FOOTPRINT_ID": "needs decision: brief section 20 and O3 -- which room out of content/presentation/board.registry.json a building's cell share is measured against",
	"STOREY_METRES": "needs decision: brief section 20, exact standard building dimensions -- every C10 record's height_class is a placeholder, so no record carries a height",
	"TIER_SETBACK": "needs decision: how far a tier above the first is stepped in, so tiers read from the fixed isometric camera",
	"FOOTING_SHARE": "needs decision: how thick a footing course stands, as a share of a storey",
	"ROOF_CAP_SHARE": "needs decision: how thick a roof cap stands, as a share of a storey",
	"SOCKET_MARKER_RADIUS_METRES": "needs decision: review-only marker size for a socket, which is an empty transform in the game",
	"SOCKET_MARKER_HEIGHT_METRES": "needs decision: review-only marker size for a socket, which is an empty transform in the game",
	"SOCKET_INNER_SHARE": "needs decision: where a socket authored at offset_cells 0 stands inside the footprint",
	"CLEARANCE_LINE_SHARE": "needs decision: how wide the line that draws the declared clearance is drawn",
	"CORNICE_SHARE": "needs decision: how deep the course between two storeys stands",
	"FOOTPRINT_LINE_METRES": "needs decision: how wide the line that draws the declared footprint on the ground is drawn",
}

## **needs decision.** Which library material each authored building wears.
## C10's records carry no material field: brief section 3's material language is
## a *room* contract (C11) and no building record was asked for one. The names
## are the library's own -- `wet_stone`, `clay`, and the metal the machines wear
## through `BlockoutKit.iron()` -- so when a record gains a material-language
## field this map is deleted and the field is read instead.
const MATERIAL_LANGUAGE := {
	"building.machine_shop": {"walls": "iron", "base": BlockoutKitScript.WET_STONE, "roof": "iron"},
	"building.coast_watch_post": {"walls": BlockoutKitScript.CLAY, "base": BlockoutKitScript.WET_STONE, "roof": BlockoutKitScript.CLAY},
	"building.ritual_anchor": {"walls": BlockoutKitScript.WET_STONE, "base": BlockoutKitScript.WET_STONE, "roof": BlockoutKitScript.WET_STONE},
}
## What a record this map does not name wears. Clay is the blockout grammar.
const DEFAULT_MATERIAL_LANGUAGE := {"walls": BlockoutKitScript.CLAY, "base": BlockoutKitScript.WET_STONE, "roof": BlockoutKitScript.CLAY}

## The clay tints. Concept-key faction colour is P4's `BoardPalette` and is not
## restated: a building takes the wash of the first concept key its
## `faction_compatibility` names, which is the only faction fact a C10 record
## carries. **needs decision** on nothing: this is presentation.
const TIMBER := Color("6d5a44")

## The socket kinds a record declares, in the field each is authored in, and the
## order the kit reads them in. Brief section 8's list; S3 owns the refusal.
const SOCKET_FIELDS: Array[String] = [
	"entrance_sockets", "road_sockets", "actor_sockets", "delivery_sockets"
]


## The reference room out of the O3 table, normalised to the shape
## `native_expedition_port.board_for` hands a footprint down in. The catalog is
## the only reader of `content/presentation/board.registry.json` here; no metre
## is copied out of it into this file.
static func reference_room_footprint(catalog: ContentCatalog) -> Dictionary:
	if catalog == null or not catalog.has_registry_entry(REFERENCE_ROOM_FOOTPRINT_ID):
		push_error("The O3 footprint table does not declare %s" % REFERENCE_ROOM_FOOTPRINT_ID)
		return {}
	var entry := catalog.get_registry_entry(REFERENCE_ROOM_FOOTPRINT_ID)
	return {
		"size_id": REFERENCE_ROOM_FOOTPRINT_ID,
		"width_metres": float(entry.get("widthMetres", 0.0)),
		"depth_metres": float(entry.get("depthMetres", 0.0)),
		"needs_decision": str(entry.get("needsDecision", "")),
	}


## The authored tiers, ascending. S3 requires them ascending from 1 with no
## gaps, so this is the record's own order and not a sort of the kit's choosing.
static func tiers_of(record: Dictionary) -> Array:
	return record.get("tier_states", []) as Array


## How many tiers stand when a building is built to `tier`. `tier` 0 means the
## record's top tier, which is what a review scene wants.
static func standing_tiers(record: Dictionary, tier: int) -> int:
	var authored := tiers_of(record).size()
	if authored <= 0:
		return 1
	if tier <= 0:
		return authored
	return clampi(tier, 1, authored)


## The declared envelope in metres: the record's share of the reference room's
## ground, in that room's aspect, standing one storey per tier built.
static func envelope_metres(record: Dictionary, room_footprint: Dictionary, tier: int = 0) -> Vector3:
	var ground := _ground_metres(int(record.get("footprint_cells", 1)), room_footprint)
	return Vector3(ground.x, float(standing_tiers(record, tier)) * STOREY_METRES, ground.y)


## The clearance ground the record reserves around itself, in metres: the same
## share arithmetic over `footprint_cells + clearance_cells`. Brief section 8
## counts required combat clearance as part of what a building declares, so it
## is drawn -- as ground, never as architecture.
static func clearance_metres(record: Dictionary, room_footprint: Dictionary) -> Vector2:
	var cells := int(record.get("footprint_cells", 1)) + int(record.get("clearance_cells", 0))
	return _ground_metres(cells, room_footprint)


static func _ground_metres(cells: int, room_footprint: Dictionary) -> Vector2:
	var room_width := float(room_footprint.get("width_metres", 0.0))
	var room_depth := float(room_footprint.get("depth_metres", 0.0))
	if room_width <= 0.0 or room_depth <= 0.0:
		push_error("A building envelope needs the O3 room footprint in metres; none was given")
		return Vector2.ONE
	var share := float(maxi(cells, 1)) / float(CELL_CAPACITY_CELLS)
	var area := room_width * room_depth * share
	var width := sqrt(area * room_width / room_depth)
	return Vector2(width, area / width)


## Build one authored building record.
##
## The returned `Node3D` stands with its origin on the ground at the centre of
## the envelope. Its `Envelope` child carries the massing and nothing else, so
## the declared box can be measured; `Clearance` carries the reserved ground,
## which is the one thing that is allowed outside the box because the record
## declares it; `Sockets` carries one empty `Node3D` per authored socket, each
## inside the footprint, named by `BlockoutKit.socket_node_name`.
static func build(record: Dictionary, room_footprint: Dictionary, tier: int = 0) -> Node3D:
	var building_id := str(record.get("id", ""))
	if building_id.is_empty():
		push_error("A building blockout needs a record with an id")
		return null
	var tiers := standing_tiers(record, tier)
	var envelope := envelope_metres(record, room_footprint, tier)

	var root_node := Node3D.new()
	root_node.name = BlockoutKitScript.socket_node_name(building_id)
	root_node.set_meta("building_id", building_id)
	root_node.set_meta("envelope_metres", envelope)
	root_node.set_meta("tier", tiers)
	root_node.set_meta("footprint_source", str(room_footprint.get("size_id", "")))
	BlockoutKitScript.mark(root_node, "the building's model, to brief section 8's declared envelope (%s)" % building_id)

	_build_clearance(root_node, record, room_footprint, envelope)
	var shell := Node3D.new()
	shell.name = "Envelope"
	root_node.add_child(shell)
	shell.set_meta("envelope_metres", envelope)
	BlockoutKitScript.mark(shell, "the building's massing (%s)" % building_id)
	_build_massing(shell, record, envelope, tiers)
	_build_footprint(root_node, record, envelope)
	_build_sockets(root_node, record, envelope)
	return root_node


## The declared footprint, drawn as a line on the ground in the record's own
## tint. It is a separate child from the massing so it stays visible when a
## review hides the walls to look at the sockets inside them, and it sits on the
## envelope's own edge, so it says exactly where brief section 8's box ends.
static func _build_footprint(root_node: Node3D, record: Dictionary, envelope: Vector3) -> void:
	var group := Node3D.new()
	group.name = "Footprint"
	root_node.add_child(group)
	group.set_meta("envelope_metres", envelope)
	BlockoutKitScript.mark(group, "no model: the declared footprint is a boundary, not a surface")
	var tint := tint_of(record)
	var line := FOOTPRINT_LINE_METRES
	var height := 0.14
	var edges := {
		"North": {"size": Vector3(envelope.x, height, line), "at": Vector3(0.0, height * 0.5, -envelope.z * 0.5 + line * 0.5)},
		"South": {"size": Vector3(envelope.x, height, line), "at": Vector3(0.0, height * 0.5, envelope.z * 0.5 - line * 0.5)},
		"West": {"size": Vector3(line, height, envelope.z), "at": Vector3(-envelope.x * 0.5 + line * 0.5, height * 0.5, 0.0)},
		"East": {"size": Vector3(line, height, envelope.z), "at": Vector3(envelope.x * 0.5 - line * 0.5, height * 0.5, 0.0)},
	}
	for edge in edges:
		var bar := SetpieceMeshFactoryScript.box(
			group, "Footprint%s" % edge, edges[edge].size, edges[edge].at, BlockoutKitScript.clay(tint, 0.28)
		)
		bar.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF


## The tint a record's clay takes: the wash P4's palette gives the first concept
## key the record is compatible with. A record compatible with nobody -- none is
## authored -- falls back to the unheld blockout clay.
static func tint_of(record: Dictionary) -> Color:
	var compatibility: Array = record.get("faction_compatibility", [])
	for key in compatibility:
		if BoardPaletteScript.FACTION_HUE_TURN.has(str(key)):
			return BoardPaletteScript.faction_hue(str(key))
	return BoardPaletteScript.clay()


static func _surface(language_key: String, tint: Color) -> Material:
	if language_key == "iron":
		return BlockoutKitScript.iron()
	if language_key == BlockoutKitScript.WET_STONE:
		return BlockoutKitScript.stone()
	return BlockoutKitScript.clay(tint)


static func _language_of(record: Dictionary) -> Dictionary:
	return MATERIAL_LANGUAGE.get(str(record.get("id", "")), DEFAULT_MATERIAL_LANGUAGE)


## One box per tier, stepped in as it rises, plus a footing course at the ground
## and a cap over the top tier. The first tier is the full envelope, so the
## massing's extents are exactly the declared envelope's and nothing above it
## reaches past the box.
static func _build_massing(shell: Node3D, record: Dictionary, envelope: Vector3, tiers: int) -> void:
	var language := _language_of(record)
	var tint := tint_of(record)
	var footing_height := STOREY_METRES * FOOTING_SHARE
	var footing := SetpieceMeshFactoryScript.box(
		shell,
		"Footing",
		Vector3(envelope.x, footing_height, envelope.z),
		Vector3(0.0, footing_height * 0.5, 0.0),
		_surface(str(language.get("base", BlockoutKitScript.WET_STONE)), tint)
	)
	BlockoutKitScript.mark(footing, "the building's baked footing course")

	var authored := tiers_of(record)
	for index in range(tiers):
		var shrink := 1.0 - TIER_SETBACK * float(index)
		var tier_record: Dictionary = authored[index] if index < authored.size() else {}
		var box := SetpieceMeshFactoryScript.box(
			shell,
			"Tier%d" % int(tier_record.get("tier", index + 1)),
			Vector3(envelope.x * shrink, STOREY_METRES, envelope.z * shrink),
			Vector3(0.0, STOREY_METRES * (float(index) + 0.5), 0.0),
			_surface(str(language.get("walls", BlockoutKitScript.CLAY)), tint)
		)
		box.set_meta("tier", int(tier_record.get("tier", index + 1)))
		box.set_meta("hit_points", int(tier_record.get("hit_points", 0)))
		BlockoutKitScript.mark(box, "tier %d of the building's model" % int(tier_record.get("tier", index + 1)))

		# A cornice where one tier meets the next. It is what makes a stack of
		# cubes read as storeys from the fixed isometric camera, and it is
		# clamped to the envelope so a wider ledge never leaves the box.
		var cornice_shrink: float = minf(shrink * 1.05, 1.0)
		var cornice_height := STOREY_METRES * CORNICE_SHARE
		var cornice := SetpieceMeshFactoryScript.box(
			shell,
			"Cornice%d" % int(tier_record.get("tier", index + 1)),
			Vector3(envelope.x * cornice_shrink, cornice_height, envelope.z * cornice_shrink),
			Vector3(0.0, STOREY_METRES * float(index + 1) - cornice_height * 0.5, 0.0),
			_surface(str(language.get("base", BlockoutKitScript.WET_STONE)), tint)
		)
		BlockoutKitScript.mark(cornice, "the course between two of the building's storeys")

	var top_shrink := 1.0 - TIER_SETBACK * float(tiers - 1)
	var cap_shrink := minf(top_shrink * 1.08, 1.0)
	var cap_height := STOREY_METRES * ROOF_CAP_SHARE
	var cap := SetpieceMeshFactoryScript.box(
		shell,
		"RoofCap",
		Vector3(envelope.x * cap_shrink, cap_height, envelope.z * cap_shrink),
		Vector3(0.0, envelope.y - cap_height * 0.5, 0.0),
		_surface(str(language.get("roof", BlockoutKitScript.CLAY)), tint)
	)
	BlockoutKitScript.mark(cap, "the building's baked roof mass, which brief section 8 keeps inside the envelope")

	# One structural line down each face, so a multi-tier blockout reads as a
	# built thing and not as a stack of cubes. Timber (brief section 5.3,
	# "structural timber"), inset so nothing leaves the box.
	var post := Vector3(envelope.x * 0.055, envelope.y, envelope.z * 0.055)
	var inset := Vector3(envelope.x * 0.5 - post.x * 0.5, 0.0, envelope.z * 0.5 - post.z * 0.5)
	for corner in [Vector3(1, 0, 1), Vector3(1, 0, -1), Vector3(-1, 0, 1), Vector3(-1, 0, -1)]:
		var column := SetpieceMeshFactoryScript.box(
			shell,
			"Column%s%s" % ["P" if corner.x > 0.0 else "N", "P" if corner.z > 0.0 else "N"],
			post,
			Vector3(corner.x * inset.x, envelope.y * 0.5, corner.z * inset.z),
			BlockoutKitScript.clay(TIMBER)
		)
		BlockoutKitScript.mark(column, "the building's baked structural frame")


## The declared clearance, drawn as an outline on the ground in the deliberately
## ugly placeholder material: it is the record's reserved metadata made visible,
## not a surface anybody walks on in the finished game. An outline rather than a
## filled pad, because the reserved ground is a *boundary* and a filled slab of
## checkerboard would be the loudest thing in every picture of a building.
static func _build_clearance(root_node: Node3D, record: Dictionary, room_footprint: Dictionary, envelope: Vector3) -> void:
	var reserved := clearance_metres(record, room_footprint)
	if reserved.x <= envelope.x and reserved.y <= envelope.z:
		return
	var group := Node3D.new()
	group.name = "Clearance"
	root_node.add_child(group)
	group.set_meta("clearance_metres", reserved)
	var thickness: float = maxf(minf(reserved.x, reserved.y) * CLEARANCE_LINE_SHARE, 0.2)
	var height := 0.10
	var edges := {
		"North": {"size": Vector3(reserved.x, height, thickness), "at": Vector3(0.0, height * 0.5, -reserved.y * 0.5 + thickness * 0.5)},
		"South": {"size": Vector3(reserved.x, height, thickness), "at": Vector3(0.0, height * 0.5, reserved.y * 0.5 - thickness * 0.5)},
		"West": {"size": Vector3(thickness, height, reserved.y), "at": Vector3(-reserved.x * 0.5 + thickness * 0.5, height * 0.5, 0.0)},
		"East": {"size": Vector3(thickness, height, reserved.y), "at": Vector3(reserved.x * 0.5 - thickness * 0.5, height * 0.5, 0.0)},
	}
	for edge in edges:
		var bar := SetpieceMeshFactoryScript.box(
			group, "Clearance%s" % edge, edges[edge].size, edges[edge].at, BlockoutKitScript.placeholder()
		)
		bar.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	BlockoutKitScript.mark(group, "no model: brief section 8's declared clearance is reserved ground, never architecture")


## Every socket the record declares, as an empty `Node3D` inside the footprint.
##
## A record counts a socket's place as `offset_cells`, and S3's rule is
## `offset_cells < footprint_cells`; so the offset is read as a share of the
## footprint's half-extent, and the sockets in one field are spread around the
## building in the record's own order. Nothing lands outside the envelope,
## which is brief section 8's rule about spawn, worker, defender and delivery
## points in particular.
static func _build_sockets(root_node: Node3D, record: Dictionary, envelope: Vector3) -> void:
	var group := Node3D.new()
	group.name = "Sockets"
	root_node.add_child(group)
	BlockoutKitScript.mark(group, "no model: sockets are empty transforms a rig or a placement system finds")
	var footprint_cells := maxf(float(record.get("footprint_cells", 1)), 1.0)
	var index := 0
	var total := 0
	for field in SOCKET_FIELDS:
		total += (record.get(field, []) as Array).size()
	for field in SOCKET_FIELDS:
		for socket in record.get(field, []) as Array:
			var offset := float((socket as Dictionary).get("offset_cells", 0))
			var reach: float = lerpf(SOCKET_INNER_SHARE, 1.0, clampf(offset / footprint_cells, 0.0, 1.0))
			var angle := TAU * float(index) / float(maxi(total, 1))
			var place := Vector3(
				sin(angle) * envelope.x * 0.5 * reach, 0.0, cos(angle) * envelope.z * 0.5 * reach
			)
			var node := BlockoutKitScript.socket_node(
				group,
				str((socket as Dictionary).get("id", "")),
				str((socket as Dictionary).get("kind", "")),
				place
			)
			node.set_meta("offset_cells", int(offset))
			BlockoutKitScript.socket_marker(
				node,
				SOCKET_MARKER_RADIUS_METRES,
				SOCKET_MARKER_HEIGHT_METRES,
				BoardPaletteScript.cream()
			)
			index += 1
