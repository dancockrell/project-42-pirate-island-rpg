extends SceneTree

## P12's done-when: every authored building and machine record is built by its
## kit, the built envelope's extents are the record's own, and every socket and
## pivot the record names exists as a child by name.
##
## The records are read out of the generated content bundle, never out of a
## fixture written here: if C10 or C14 authors a fourth record tomorrow, this
## suite builds it without being edited, and if a record's envelope changes the
## assertion changes with it.

const BlockoutKitScript := preload("res://scripts/blockouts/blockout_kit.gd")
const BuildingBlockoutKitScript := preload("res://scripts/blockouts/building_blockout_kit.gd")
const MachineBlockoutKitScript := preload("res://scripts/blockouts/machine_blockout_kit.gd")
const ContentCatalogScript := preload("res://scripts/content/content_catalog.gd")

## Metres. Envelope arithmetic is float, so equality is compared at a tolerance
## far below anything a person can see and far above float noise.
const METRE_EPSILON := 0.001

var failures := 0


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var catalog = ContentCatalogScript.new()
	check(catalog.load_default() == OK, "the generated content bundle must load")

	var stage := Node3D.new()
	stage.name = "BlockoutKitStage"
	root.add_child(stage)

	check_open_dimensions_are_marked()
	check_buildings(catalog, stage)
	check_machines(catalog, stage)

	stage.queue_free()
	finish()


## Rule: no metre this project has not decided is written down without saying so.
## Both kits declare an `OPEN_DIMENSIONS` map, every entry of it says "needs
## decision", and every constant either kit invents is named in it.
func check_open_dimensions_are_marked() -> void:
	var invented := {
		BuildingBlockoutKitScript: [
			"CELL_CAPACITY_CELLS", "REFERENCE_ROOM_FOOTPRINT_ID", "STOREY_METRES",
			"TIER_SETBACK", "FOOTING_SHARE", "ROOF_CAP_SHARE",
			"SOCKET_MARKER_RADIUS_METRES", "SOCKET_MARKER_HEIGHT_METRES",
			"SOCKET_INNER_SHARE", "CLEARANCE_LINE_SHARE", "CORNICE_SHARE",
			"FOOTPRINT_LINE_METRES",
		],
		MachineBlockoutKitScript: [
			"FAMILY_ENVELOPE_METRES", "PIVOT_PLAN",
			"SOCKET_MARKER_RADIUS_METRES", "SOCKET_MARKER_HEIGHT_METRES",
		],
	}
	for kit in invented:
		var open: Dictionary = kit.OPEN_DIMENSIONS
		for constant_name in invented[kit]:
			check(
				open.has(constant_name),
				"every dimension a kit invents must be named in OPEN_DIMENSIONS: %s" % constant_name
			)
			check(
				str(open.get(constant_name, "")).begins_with("needs decision"),
				"%s must still be marked as a decision nobody has taken" % constant_name
			)
		check(
			open.size() == (invented[kit] as Array).size(),
			"OPEN_DIMENSIONS must name exactly the constants the kit invents, no more"
		)


func check_buildings(catalog, stage: Node3D) -> void:
	var room: Dictionary = BuildingBlockoutKitScript.reference_room_footprint(catalog)
	check(float(room.get("width_metres", 0.0)) > 0.0, "the O3 footprint table must give the reference room its metres")
	check(str(room.get("needs_decision", "")) == "O3", "the reference room's metres must still be marked as Open item O3")

	var ids: Array = catalog.ids_with_prefix("building.")
	check(ids.size() >= 3, "C10's three building records must all be built, not a chosen one")
	for building_id in ids:
		var record: Dictionary = catalog.get_record(building_id)
		var subject: Node3D = BuildingBlockoutKitScript.build(record, room)
		check(subject != null, "%s must build" % building_id)
		if subject == null:
			continue
		stage.add_child(subject)

		# The envelope the kit declares is the record's: its ground is the
		# record's share of the reference room, and it stands one storey per
		# authored tier.
		var envelope: Vector3 = BuildingBlockoutKitScript.envelope_metres(record, room)
		var tiers: int = (record.get("tier_states", []) as Array).size()
		check(
			absf(envelope.y - float(tiers) * BuildingBlockoutKitScript.STOREY_METRES) < METRE_EPSILON,
			"%s must stand one storey per authored tier" % building_id
		)
		var share := float(record.get("footprint_cells", 0)) / float(BuildingBlockoutKitScript.CELL_CAPACITY_CELLS)
		var expected_area := float(room.width_metres) * float(room.depth_metres) * share
		check(
			absf(envelope.x * envelope.z - expected_area) < METRE_EPSILON * 10.0,
			"%s must cover its own share of the reference room's ground" % building_id
		)

		# And the massing measures exactly that: nothing essential outside the
		# declared box, which is brief section 8's rule.
		var shell := subject.get_node_or_null("Envelope") as Node3D
		check(shell != null, "%s must carry its massing under Envelope" % building_id)
		if shell != null:
			var bounds: AABB = BlockoutKitScript.bounds_of(shell)
			check(
				absf(bounds.size.x - envelope.x) < METRE_EPSILON
				and absf(bounds.size.y - envelope.y) < METRE_EPSILON
				and absf(bounds.size.z - envelope.z) < METRE_EPSILON,
				"%s: the built envelope's extents must equal the record's declared envelope" % building_id
			)

		# Every socket the record names exists as a child, by name, inside the
		# footprint. Brief section 8 lists worker, defender, spawn and delivery
		# positions among the things that may not leave the box.
		var sockets := subject.get_node_or_null("Sockets") as Node3D
		check(sockets != null, "%s must carry its sockets" % building_id)
		var declared := 0
		for field in BuildingBlockoutKitScript.SOCKET_FIELDS:
			for socket in record.get(field, []) as Array:
				declared += 1
				var socket_id := str((socket as Dictionary).get("id", ""))
				var node := sockets.get_node_or_null(BlockoutKitScript.socket_node_name(socket_id)) as Node3D
				check(node != null, "%s must carry socket %s as a child by name" % [building_id, socket_id])
				if node == null:
					continue
				check(str(node.get_meta("socket_id", "")) == socket_id, "%s must carry its authored ID verbatim" % socket_id)
				check(
					absf(node.position.x) <= envelope.x * 0.5 + METRE_EPSILON
					and absf(node.position.z) <= envelope.z * 0.5 + METRE_EPSILON,
					"brief section 8: %s must stand inside the declared envelope" % socket_id
				)
		check(declared > 0 or building_id == "building.ritual_anchor", "%s declares sockets" % building_id)
		check(
			sockets != null and sockets.get_child_count() == declared,
			"%s must carry one socket node per authored socket and no others" % building_id
		)
		subject.queue_free()


func check_machines(catalog, stage: Node3D) -> void:
	var ids: Array = catalog.ids_with_prefix("machine.")
	check(ids.size() >= 2, "C14's two machine records must both be built")
	for machine_id in ids:
		var record: Dictionary = catalog.get_record(machine_id)
		check(MachineBlockoutKitScript.can_build(record), "%s must have a blockout for its family" % machine_id)
		var subject: Node3D = MachineBlockoutKitScript.build(record)
		check(subject != null, "%s must build" % machine_id)
		if subject == null:
			continue
		stage.add_child(subject)

		# The record's own dimensions are all zero and its open_dimensions map
		# names each decision, so the kit's envelope is the placeholder and the
		# record is what says it is one. Both halves are asserted, because a
		# record that quietly gained a real number would make this kit wrong.
		var envelope: Vector3 = MachineBlockoutKitScript.envelope_metres(record)
		check(envelope.x > 0.0 and envelope.y > 0.0 and envelope.z > 0.0, "%s must have an envelope to build inside" % machine_id)
		check(
			int(record.get("operational_footprint_cells", -1)) == 0,
			"%s still declares no operational footprint, which is why the kit's envelope is a placeholder" % machine_id
		)
		var open: Dictionary = record.get("open_dimensions", {})
		check(
			str(open.get("operational_footprint_cells", "")).begins_with("needs decision"),
			"%s must still name its operational footprint as a decision nobody has taken" % machine_id
		)

		# Nothing the kit builds leaves the envelope it declared.
		var chassis := subject.get_node_or_null("Chassis") as Node3D
		check(chassis != null, "%s must carry its geometry under Chassis" % machine_id)
		if chassis != null:
			var bounds: AABB = BlockoutKitScript.bounds_of(chassis)
			check(
				bounds.size.x <= envelope.x + METRE_EPSILON
				and bounds.size.y <= envelope.y + METRE_EPSILON
				and bounds.size.z <= envelope.z + METRE_EPSILON,
				"%s: nothing may stand outside the declared envelope" % machine_id
			)
			check(
				bounds.position.y >= -METRE_EPSILON,
				"%s must stand on the ground its origin sits on" % machine_id
			)

		# Every repair socket the record names, and every pivot derived from
		# one, exists as a child by name.
		var sockets := subject.get_node_or_null("Sockets") as Node3D
		check(sockets != null, "%s must carry its repair sockets" % machine_id)
		var authored: Array = record.get("repair_sockets", [])
		check(authored.size() > 0, "%s declares repair sockets" % machine_id)
		for socket_id in authored:
			var node := sockets.get_node_or_null(BlockoutKitScript.socket_node_name(str(socket_id))) as Node3D
			check(node != null, "%s must carry repair socket %s as a child by name" % [machine_id, socket_id])
			if node != null:
				check(bool(node.get_meta("placed", false)), "%s must be placed on the part it belongs to" % socket_id)

		var pivots := subject.get_node_or_null("Pivots") as Node3D
		check(pivots != null, "%s must carry its pivots" % machine_id)
		var pivot_ids: Array[String] = MachineBlockoutKitScript.pivot_ids(record)
		check(pivot_ids.size() > 0, "%s must keep future-ready pivots (brief section 18)" % machine_id)
		for pivot_id in pivot_ids:
			var node := pivots.get_node_or_null(BlockoutKitScript.socket_node_name(pivot_id)) as Node3D
			check(node != null, "%s must carry pivot %s as a child by name" % [machine_id, pivot_id])
			if node == null:
				continue
			var derived := str(node.get_meta("derived_from", ""))
			check(
				authored.has(derived),
				"%s must be derived from a repair socket the record actually names, not from thin air" % pivot_id
			)
		check(
			pivots != null and pivots.get_child_count() == pivot_ids.size(),
			"%s must carry one pivot node per planned pivot and no others" % machine_id
		)

		# Marked as a placeholder by name, as every P card requires.
		check(bool(subject.get_meta("blockout_placeholder", false)), "%s must be marked as the placeholder it is" % machine_id)
		check(str(subject.get_meta("replaced_by", "")).length() > 0, "%s must name what replaces it" % machine_id)
		subject.queue_free()

	# The mechanical dog stands on four pivots, which is what D10 asks for by
	# name; the wagon's are its axles.
	var dog: Dictionary = catalog.get_record("machine.mechanical_dog")
	check(MachineBlockoutKitScript.pivot_ids(dog).size() == 4, "the mechanical dog stands on four pivots")
	var wagon: Dictionary = catalog.get_record("machine.steam_wagon")
	var axles := 0
	for pivot_id in MachineBlockoutKitScript.pivot_ids(wagon):
		if pivot_id.contains("axle"):
			axles += 1
	check(axles == 2, "the steam wagon is built on its two axle sockets")


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Blockout kit tests passed.")
	quit(0)
