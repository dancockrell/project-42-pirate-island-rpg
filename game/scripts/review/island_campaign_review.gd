extends SceneTree

# Bounded observation of the real island, without changing its faction state.
# Optional `talk` drives actual Approach/Talk controls to a produced adult man.
# Headless output is simulation evidence, never a rendered art approval.
func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var scene = load("res://scenes/world/island.tscn").instantiate()
	root.add_child(scene)
	await process_frame
	scene.set_process(false)
	if not scene.ready_ok:
		push_error("Island could not initialize")
		quit(1)
		return
	var previous := ""
	var previous_wars := ""
	var steps := 600
	var demonstrate_talk := false
	var talk_target := ""
	var conversation_complete := false
	for argument in OS.get_cmdline_user_args():
		if argument == "talk":
			demonstrate_talk = true
		if argument.begins_with("ticks="):
			steps = clampi(argument.trim_prefix("ticks=").to_int(), 1, 2880)
	for step in range(steps + 1):
		var state: Dictionary = scene.port.island_snapshot()
		if demonstrate_talk and not conversation_complete:
			scene.refresh_snapshot()
			if not talk_target.is_empty() and not state.actors.any(func(person): return person.id == talk_target):
				talk_target = ""
			if talk_target.is_empty():
				for person in state.actors:
					if person.id != scene.MICHAEL and person.sex == "male" and int(person.get("age",0)) >= 18:
						scene.inspect_actor(person.id)
						scene.approach_action()
						if scene.conversation_notice.is_empty() or scene.port.can_talk_island_person(person.id):
							talk_target = person.id
							break
			if not talk_target.is_empty() and scene.port.can_talk_island_person(talk_target):
				scene.talk_action()
				print("ISLAND conversation step=", step, " name=", scene.inspected_person.name,
					" talk=", scene.talk_button.visible, " join=", scene.recruit_button.visible,
					" news=", scene.inspected_person.get("news",""))
				conversation_complete = true
				scene.set_paused(false)
		var holdings: Array = []
		for building in state.buildings:
			if building.faction == "faction.colonial_powers.prototype":
				holdings.append({"id":building.id, "x":building.x, "y":building.y,
					"operational":building.operational, "level":building.level,
					"construction":building.get("construction_remaining",0)})
		var signature := JSON.stringify(holdings)
		if step % 32 == 0 or step == steps:
			var saved: Dictionary = JSON.parse_string(scene.port.save_island())
			var wars := JSON.stringify(saved.world.hostilities)
			if wars != previous_wars:
				print("ISLAND diplomacy step=", step, " wars=", wars)
				previous_wars = wars
			if step % 128 == 0 or step == steps:
				var populations := {}
				for faction in saved.world.factions.values():
					populations[faction.id] = {"population": faction.population_used, "holdings": faction.buildings.size()}
				print("ISLAND population step=", step, " factions=", JSON.stringify(populations))
		if signature != previous and (step % 20 == 0 or not signature.contains("expansion_fort") or not previous.contains("expansion_fort")):
			print("ISLAND step=", step, " colonial_holdings=", signature)
			previous = signature
		if step < steps:
			if demonstrate_talk:
				scene.advance_tick()
			else:
				scene.port.tick_island()
		if step % 20 == 0:
			await process_frame
	print("ISLAND bounded campaign observation complete")
	if demonstrate_talk and not conversation_complete:
		print("ISLAND conversation not reached in this bounded run")
	scene.queue_free()
	await process_frame
	quit()
