extends SceneTree

# Bounded observation of the real island, without changing its faction state.
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
	for step in range(601):
		var state: Dictionary = scene.port.island_snapshot()
		var holdings: Array = []
		for building in state.buildings:
			if building.faction == "faction.colonial_powers.prototype":
				holdings.append({"id":building.id, "x":building.x, "y":building.y,
					"operational":building.operational, "level":building.level,
					"construction":building.get("construction_remaining",0)})
		var signature := JSON.stringify(holdings)
		if step <= 120 and step % 20 == 0:
			var saved: Dictionary = JSON.parse_string(scene.port.save_island())
			var faction: Dictionary = saved.world.factions.get("faction.colonial_powers.prototype", {})
			print("ISLAND economy step=",step," resources=",faction.get("resources",{})," population=",faction.get("population_used",0))
		if signature != previous and (step % 20 == 0 or not signature.contains("expansion_fort") or not previous.contains("expansion_fort")):
			print("ISLAND step=", step, " colonial_holdings=", signature)
			previous = signature
		if step < 600:
			scene.port.tick_island()
		if step % 20 == 0:
			await process_frame
	print("ISLAND bounded campaign observation complete")
	scene.queue_free()
	await process_frame
	quit()
