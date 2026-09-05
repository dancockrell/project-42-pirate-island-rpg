extends SceneTree

## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0

## Presentation-only test. It proves that the active paper actor is a rig with
## dependent pieces and that authored Guarded Strike poses move inside the
## designated canvas rather than swapping one static illustration for another.

const PaperBettyRigScript = preload("res://scripts/battle/paper_betty_rig.gd")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var rig: PaperBettyRig = PaperBettyRigScript.new()
	get_root().add_child(rig)
	rig.build(Vector2(500, 615))
	check(rig.weapon.get_parent() == rig.right_arm, "Betty's mace must remain parented to her weapon arm")
	check(rig.left_boot.get_parent() == rig.left_leg, "Betty's left boot must remain parented to her left leg")
	check(rig.right_boot.get_parent() == rig.right_leg, "Betty's right boot must remain parented to her right leg")
	var home := rig.home_position
	rig.set_pose("mace_low_guard")
	check(is_equal_approx(rig.right_arm.rotation_degrees, -42.0), "low guard must bring the mace arm across Betty's body")
	rig.animate_to_pose("forward_step", "card_to_battle_plane")
	await create_timer(.20).timeout
	check(rig.position.x > home.x, "card-to-plane motion must advance Betty into the shared action field")
	check(is_equal_approx(rig.weapon.rotation_degrees, -58.0), "forward step must carry the whole weapon chain")
	rig.animate_to_pose("horizontal_mace_hit", "contact_lunge")
	await create_timer(.12).timeout
	check(rig.position.x > home.x + 40.0, "impact lunge must remain a readable forward action")
	check(is_equal_approx(rig.right_arm.rotation_degrees, -72.0), "impact must use the authored striking shoulder pose")
	rig.animate_to_pose("card_ready", "battle_plane_to_card")
	await create_timer(.22).timeout
	check(rig.position.is_equal_approx(home), "recall must return Betty to her stable ready anchor")
	rig.queue_free()
	print("PaperBettyRig tests passed.")
	quit(1 if failures > 0 else 0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
