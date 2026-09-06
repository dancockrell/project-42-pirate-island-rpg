extends SceneTree
## Captures review scenes through their current Camera3D, in one engine run.
##
## Usage, one scene as authored:
##   godot --path game --script res://tools/capture_review_scene.gd -- <scene> <absolute-output.png>
##
## Usage, several captures in a single process (P3): repeating triples of
## scene, atmosphere snapshot and output, with `-` for "as authored".
##   ... -- <scene> <snapshot|-> <out.png> [<scene> <snapshot|-> <out.png> ...]
##
## The batch form exists because opening and closing the engine once per image
## is unkind to a machine running several lanes; one process renders the whole
## set. The snapshot names are `AtmosphereReviewSnapshots.NAMES`, and applying
## one drives the `Atmosphere` autoload exactly as a campaign snapshot would --
## the same `apply_immediately` call, the same authored tables, no capture-only
## path through the sky.


func _init() -> void:
	call_deferred("capture")


func capture() -> void:
	var arguments := OS.get_cmdline_user_args()
	var jobs := parse(arguments)
	if jobs.is_empty():
		fail(
			"Expected <scene> <output.png>, or repeating <scene> <snapshot|-> <output.png> triples"
		)
		return
	for job in jobs:
		if not await capture_one(job):
			return
	quit(0)


## The argument list as a list of jobs, or an empty array when it is neither
## shape. Two arguments is the original contract and stays exactly what it was.
func parse(arguments: PackedStringArray) -> Array[Dictionary]:
	var jobs: Array[Dictionary] = []
	if arguments.size() == 2:
		jobs.append({"scene": arguments[0], "snapshot": "", "output": arguments[1]})
		return jobs
	if arguments.size() < 3 or arguments.size() % 3 != 0:
		return jobs
	for index in range(0, arguments.size(), 3):
		var snapshot := arguments[index + 1]
		jobs.append(
			{
				"scene": arguments[index],
				"snapshot": "" if snapshot == "-" else snapshot,
				"output": arguments[index + 2]
			}
		)
	return jobs


func capture_one(job: Dictionary) -> bool:
	var scene_path: String = job.scene
	var output_path: String = job.output
	var snapshot_name: String = job.snapshot
	if not scene_path.begins_with("res://") or not scene_path.ends_with(".tscn"):
		fail("Review scene must be a res:// .tscn path")
		return false
	if not output_path.to_lower().ends_with(".png") or not output_path.is_absolute_path():
		fail("Output must be an absolute .png path")
		return false
	var packed := load(scene_path) as PackedScene
	if packed == null:
		fail("Could not load review scene: %s" % scene_path)
		return false
	var instance := packed.instantiate()
	root.add_child(instance)
	await process_frame
	if not snapshot_name.is_empty() and not apply_atmosphere(snapshot_name, instance):
		instance.queue_free()
		return false
	await process_frame
	await process_frame
	RenderingServer.force_sync()
	RenderingServer.force_draw(false)
	var image := root.get_viewport().get_texture().get_image()
	var error := image.save_png(output_path)
	instance.queue_free()
	await process_frame
	if error != OK:
		fail("Could not save PNG %s: error %s" % [output_path, error])
		return false
	print("Captured %s to %s" % [scene_path, output_path])
	return true


## Drives the `Atmosphere` autoload with one authored snapshot, with no travel:
## a capture wants the sky it asked for this frame, not two seconds of it.
func apply_atmosphere(snapshot_name: String, scene: Node) -> bool:
	var atmosphere := root.get_node_or_null("Atmosphere")
	if atmosphere == null:
		fail("Atmosphere autoload is not registered; a snapshot cannot be applied")
		return false
	# The night's lamps need somewhere to stand, and the board that will say
	# where is lane P4's. For a review capture the scene's own entry markers
	# stand in, marked as review-only in AtmosphereReviewBoard; in the game
	# there is no provider and the lamps stay unplaced.
	var board := AtmosphereReviewBoard.new(scene)
	atmosphere.board_anchor_provider = board if board.has_anchors() else null
	var snapshot := AtmosphereReviewSnapshots.of(snapshot_name)
	if snapshot.is_empty():
		fail(
			(
				"Unknown atmosphere snapshot %s; the authored names are %s"
				% [snapshot_name, ", ".join(AtmosphereReviewSnapshots.NAMES)]
			)
		)
		return false
	var applied: Dictionary = atmosphere.apply_immediately(snapshot)
	if applied.is_empty():
		fail("Atmosphere refused the %s snapshot: %s" % [snapshot_name, atmosphere.refusal()])
		return false
	var target: Object = atmosphere.target
	print(
		(
			"Atmosphere %s applied through %s: %s weather, %s grade, %s shift, horizon %s"
			% [
				snapshot_name,
				"nothing" if target == null else target.describes(),
				applied.weather_kind,
				applied.grade.palette_name,
				applied.grade.heat_shift_name,
				applied.sun.sky_horizon_colour.to_html(false)
			]
		)
	)
	return true


func fail(message: String) -> void:
	push_error(message)
	quit(1)
