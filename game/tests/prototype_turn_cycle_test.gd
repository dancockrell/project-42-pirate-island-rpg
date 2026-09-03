extends SceneTree

var failures := 0

func _init() -> void:
	call_deferred("run")

func run() -> void:
	var prototype := BattlePrototype.new()
	root.add_child(prototype)
	await process_frame
	check(prototype.active_actor_id == "character.heroine.betty", "Betty must own the opening player turn")
	await prototype.submit_skill("skill.betty.guarded_strike", ["enemy.raptor.razorbeak.prototype"])
	check(prototype.active_actor_id == "character.heroine.betty", "automatic enemy and support turns must return control to Betty")
	check(prototype.is_command_enabled("skill.betty.guarded_strike"), "Betty's legal commands must re-enable after the full initiative cycle")
	check("VIX" in prototype.intent_label.text and "LETHAL" in prototype.intent_label.text, "visible enemy intent must name the authoritative target and lethal projection")
	prototype.queue_free()
	if failures > 0:
		quit(1)
		return
	print("BattlePrototype turn-cycle test passed.")
	quit(0)

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
