extends Node3D

## P12's review scene: every authored building and machine record, built by the
## kits and stood side by side on one ground under the island's own environment
## and its own fixed isometric camera.
##
## Brief section 18: "visual changes must be reviewed at actual gameplay
## distance", and "a readable blockout is not proof of finished production art".
## So nothing here is a showroom turntable: the light is
## `res://render/world_environment.tscn`, the view is `res://render/camera_rig.tscn`
## at the room distance the board plays at, and every subject is exactly what
## `BuildingBlockoutKit.build` and `MachineBlockoutKit.build` return for a
## record -- this scene adds no geometry to a subject and edits none.
##
## It has two views because the two families of subject are two orders of size
## apart and a frame that held both would be a picture of neither: `buildings`
## frames the three C10 records, `machines` the two C14 records. Both are built
## either way; only the framing changes. `game/tests/capture_blockout_kits.gd`
## renders one of each from a single engine process.

const BlockoutKitScript := preload("res://scripts/blockouts/blockout_kit.gd")
const BuildingBlockoutKitScript := preload("res://scripts/blockouts/building_blockout_kit.gd")
const MachineBlockoutKitScript := preload("res://scripts/blockouts/machine_blockout_kit.gd")
const BoardPaletteScript := preload("res://scripts/board/board_palette.gd")
const SetpieceMeshFactoryScript := preload("res://scripts/world/setpiece_mesh_factory.gd")
const CameraDirectorScript := preload("res://scripts/render/camera_director.gd")

const BUILDINGS_VIEW := "buildings"
const MACHINES_VIEW := "machines"
const VIEWS: Array[String] = [BUILDINGS_VIEW, MACHINES_VIEW]

## Review staging, all of it: the gap between two subjects, how far the socket
## plan row stands behind the massing row, and how far the ground reaches past
## the subjects on it. None of these is a game dimension and none is a decision:
## they are how a contact sheet is laid out.
const SUBJECT_GAP_METRES := 9.0

## The fixed isometric heading is 45 degrees of yaw, so a row laid along a world
## axis projects as a diagonal and every subject stands in front of the next
## one. The row is laid along the direction that reads as horizontal on screen
## instead, and the second row along the one that reads as vertical, with each
## subject left square to the world -- which is how a building stands on the
## board. Review staging, and the only reason this scene knows the heading.
const ROW_AXIS := Vector3(1.0, 0.0, -1.0)
const DEPTH_AXIS := Vector3(1.0, 0.0, 1.0)
const MACHINE_GAP_METRES := 3.2

## How far in front of the buildings the machines stand, in metres. Far enough
## that the camera framing a two-metre machine is never standing inside a
## twenty-metre building, and close enough that the buildings are behind them
## in the plate and give them their scale.
const MACHINE_ROW_CLEARANCE_METRES := 26.0

## How much of the reference room the machines view holds, as a share. The
## board reads a room at its whole footprint; two machines two metres across in
## a forty-metre room are a hundred pixels of a capture and there would be
## nothing to judge. This frames a working corner of that room instead, with
## the buildings still behind them for scale. Review staging, not a dimension.
const MACHINE_VIEW_ROOM_SHARE := 0.36
const PLAN_ROW_OFFSET_SHARE := 1.9

## Which view the scene opens in. The gate captures the scene as it is
## committed, so the committed value is the one a reviewer sees first.
@export var view: String = BUILDINGS_VIEW

var catalog: ContentCatalog
var buildings: Node3D
var machines: Node3D
var subjects_by_id: Dictionary = {}
var row_depth := 0.0
var row_separation := 0.0


func _ready() -> void:
	set_meta(
		"review_purpose",
		"D9 and D10 at gameplay distance: every authored building and machine record built by the blockout kits, under the island environment and the fixed isometric camera."
	)
	set_meta(
		"review_staging",
		"The ground slab and the spacing between subjects are review staging, generated in-repo as procedural primitives. No subject is edited by this scene; each is exactly what its kit returns for its record."
	)
	catalog = ContentCatalog.new()
	if catalog.load_default() != OK:
		push_error("The blockout kit review needs the generated content bundle")
		return
	_build_buildings()
	_build_machines()
	_build_ground()
	show_view(view)


## Frame one of the two views. Called by the capture driver between frames, so
## both pictures come out of one engine process.
func show_view(name_of_view: String) -> void:
	view = name_of_view
	var rig := get_node_or_null("ReviewCamera") as Node3D
	if rig == null:
		return
	if name_of_view == MACHINES_VIEW:
		if machines == null:
			return
		var bounds: AABB = CameraDirectorScript.subtree_bounds(machines)
		if bounds.size == Vector3.ZERO:
			push_error("The blockout kit review found nothing to frame in the machines view")
			return
		var room := BuildingBlockoutKitScript.reference_room_footprint(catalog)
		var span := float(room.get("width_metres", 0.0)) * MACHINE_VIEW_ROOM_SHARE
		# Widened on the ground only: a taller box would tilt the frame off the
		# machines standing in it.
		var centre := bounds.get_center()
		var size := Vector3(maxf(bounds.size.x, span), bounds.size.y, maxf(bounds.size.z, span))
		rig.frame_bounds(
			AABB(Vector3(centre.x - size.x * 0.5, bounds.position.y, centre.z - size.z * 0.5), size),
			CameraDirectorScript.Distance.ROOM
		)
		return
	if buildings == null:
		return
	if not rig.frame_subtree(buildings, CameraDirectorScript.Distance.ROOM):
		push_error("The blockout kit review found nothing to frame in the %s view" % name_of_view)


## The three C10 records: the massing row, and behind it the same three records
## with the massing hidden, so the sockets brief section 8 keeps *inside* the
## envelope can be seen at all. Both rows are the kit's own output.
func _build_buildings() -> void:
	buildings = Node3D.new()
	buildings.name = "Buildings"
	add_child(buildings)
	var room := BuildingBlockoutKitScript.reference_room_footprint(catalog)
	var ids: Array = catalog.ids_with_prefix("building.")

	# One pass to measure, so the row is laid out on the reserved ground each
	# record declares rather than on its walls: two clearance outlines that
	# overlapped would be a picture of this scene's spacing, not of the records.
	var widths: Array[float] = []
	row_depth = 0.0
	var total := 0.0
	for building_id in ids:
		var record: Dictionary = catalog.get_record(building_id)
		var envelope: Vector3 = BuildingBlockoutKitScript.envelope_metres(record, room)
		var reserved: Vector2 = BuildingBlockoutKitScript.clearance_metres(record, room)
		var width: float = maxf(maxf(envelope.x, reserved.x), maxf(envelope.z, reserved.y))
		widths.append(width)
		row_depth = maxf(row_depth, maxf(envelope.z, reserved.y))
		total += width + SUBJECT_GAP_METRES

	row_separation = row_depth * PLAN_ROW_OFFSET_SHARE
	var separation := row_separation
	var cursor := 0.0
	for index in range(ids.size()):
		var building_id := str(ids[index])
		var record: Dictionary = catalog.get_record(building_id)
		var subject: Node3D = BuildingBlockoutKitScript.build(record, room)
		if subject == null:
			continue
		buildings.add_child(subject)
		subject.position = ROW_AXIS.normalized() * (cursor + widths[index] * 0.5) + DEPTH_AXIS.normalized() * separation * 0.5
		subjects_by_id[building_id] = subject

		# The same record again, with the envelope hidden. A building's sockets
		# stand inside its box because brief section 8 says nothing essential
		# may stand outside it, so the only honest way to look at them is to
		# take the walls away rather than to move the sockets out.
		var plan: Node3D = BuildingBlockoutKitScript.build(record, room)
		plan.name = "%s_SocketPlan" % subject.name
		buildings.add_child(plan)
		plan.position = ROW_AXIS.normalized() * (cursor + widths[index] * 0.5) - DEPTH_AXIS.normalized() * separation * 0.5
		(plan.get_node("Envelope") as Node3D).visible = false
		plan.set_meta("review_staging", "the same record with its massing hidden, so the sockets inside the envelope can be seen")
		cursor += widths[index] + SUBJECT_GAP_METRES
	buildings.position = -ROW_AXIS.normalized() * (total - SUBJECT_GAP_METRES) * 0.5


## The two C14 records, in a row of their own, at the size their kit says they
## are. They stand on the same ground and under the same light as the buildings:
## a machine that only reads on a plinth of its own does not read.
func _build_machines() -> void:
	machines = Node3D.new()
	machines.name = "Machines"
	add_child(machines)
	var cursor := 0.0
	for machine_id in catalog.ids_with_prefix("machine."):
		var record := catalog.get_record(machine_id)
		if not MachineBlockoutKitScript.can_build(record):
			continue
		var envelope: Vector3 = MachineBlockoutKitScript.envelope_metres(record)
		var subject: Node3D = MachineBlockoutKitScript.build(record)
		if subject == null:
			continue
		machines.add_child(subject)
		subject.position = ROW_AXIS.normalized() * (cursor + envelope.x * 0.5)
		subjects_by_id[machine_id] = subject
		cursor += envelope.x + MACHINE_GAP_METRES
	# On the same ground and under the same light as the buildings, in front of
	# the row: a machine that only reads on a plinth of its own does not read.
	machines.position = (
		-ROW_AXIS.normalized() * (cursor - MACHINE_GAP_METRES) * 0.5
		+ DEPTH_AXIS.normalized() * (row_separation * 0.5 + MACHINE_ROW_CLEARANCE_METRES)
	)


## The ground everything stands on: review staging, and the one thing in this
## scene that is not the kits' output. Wet stone out of P2's library, sized to
## whatever the two rows came out as, so no subject ever stands on an edge.
func _build_ground() -> void:
	var bounds := CameraDirectorScript.subtree_bounds(self)
	var margin := 24.0
	var size := Vector3(bounds.size.x + margin * 2.0, 1.2, bounds.size.z + margin * 2.0)
	var centre := bounds.get_center()
	var ground := SetpieceMeshFactoryScript.box(
		self,
		"ReviewGround",
		size,
		Vector3(centre.x, -size.y * 0.5, centre.z),
		BlockoutKitScript.stone()
	)
	ground.set_meta(
		"review_staging",
		"A procedural slab, generated in-repo, so the subjects are lit and shadowed on a surface rather than floating. It is not part of any record."
	)
