extends SceneTree

## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0

const PaperRazorbeakRigScript = preload("res://scripts/battle/paper_razorbeak_rig.gd")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var rig = PaperRazorbeakRigScript.new()
	get_root().add_child(rig)
	rig.build(Vector2(450, 455))
	check(rig.head.get_parent() == rig.neck, "Razorbeak head must remain attached to the neck")
	check(rig.jaw.get_parent() == rig.head, "Razorbeak jaw must remain attached to the head")
	var home: Vector2 = rig.home_position
	rig.animate_reaction("contact_lunge", .35)
	await create_timer(.12).timeout
	check(rig.position.x > home.x, "an individual Razorbeak must recoil from Betty's impact")
	check(rig.jaw.rotation_degrees > 10.0, "the hit reaction must visibly open the Razorbeak jaw")
	rig.set_pose("ready_idle")
	check(rig.position.is_equal_approx(home), "Razorbeak ready state must return to its stable action anchor")
	rig.queue_free()
	print("PaperRazorbeakRig tests passed.")
	quit(1 if failures > 0 else 0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
