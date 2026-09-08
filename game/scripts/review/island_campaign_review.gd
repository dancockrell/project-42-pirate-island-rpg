extends SceneTree

# Bounded observation of the real island, without changing its faction state.
# Optional `talk` drives actual Approach/Talk controls to a produced adult man.
# Optional `reclaim` instead seeks a living madness-converted woman and joins her.
# Optional `workshop` drives salvage, travel, workshop construction and a machine order.
# Optional `salvage` walks to a real AI-destroyed holding and recovers its materials.
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
	var seen_conversions := {}
	var seen_repairs := {}
	var steps := 600
	var demonstrate_talk := false
	var reclaim_converted := false
	var talk_target := ""
	var conversation_complete := false
	var demonstrate_workshop := false
	var workshop_stage := 0
	var workshop_destination := Vector2i(19,17)
	var machine_seen := false
	var demonstrate_salvage := false
	var seen_caches := {}
	var salvage_target := ""
	var salvage_complete := false
	for argument in OS.get_cmdline_user_args():
		if argument == "talk":
			demonstrate_talk = true
		if argument == "workshop":
			demonstrate_workshop = true
		if argument == "salvage":
			demonstrate_salvage = true
		if argument == "reclaim":
			demonstrate_talk = true
			reclaim_converted = true
		if argument.begins_with("ticks="):
			steps = clampi(argument.trim_prefix("ticks=").to_int(), 1, 2880)
	for step in range(steps + 1):
		var state: Dictionary = scene.port.island_snapshot()
		for cache in state.get("salvage_caches", []):
			if not seen_caches.has(cache.id):
				seen_caches[cache.id] = true
				print("ISLAND salvage appeared step=", step, " cache=", cache)
			if demonstrate_salvage and not salvage_complete and int(cache.level) > 0 and int(cache.remaining) > 0:
				if salvage_target.is_empty():
					var destination := Vector2i(cache.x,cache.y)
					if scene.port.move_island_party(destination):
						salvage_target = cache.id
						print("ISLAND salvage approach step=", step, " cache=", cache.id)
				if cache.id == salvage_target and cache.collectible:
					scene.refresh_snapshot()
					var before := int(scene.snapshot.salvage)
					scene.salvage_action()
					print("ISLAND salvage collected step=", step, " cache=", cache.id, " gained=", int(scene.snapshot.salvage)-before, " notice=", scene.save_notice, " marker_removed=", not scene.salvage_markers.has(cache.id))
					var payload: String = scene.port.save_island()
					print("ISLAND salvage saved and loaded=", scene.port.load_island(payload))
					scene.refresh_snapshot()
					print("ISLAND salvage after reload treasury=", scene.snapshot.salvage, " cache=", scene.snapshot.salvage_caches.filter(func(c): return c.id == salvage_target))
					salvage_complete = true
					# Reinvest the recovered materials through the same workshop controls.
					workshop_destination = Vector2i(cache.x,cache.y)
					demonstrate_workshop = true
					workshop_stage = 1
					scene.port.move_island_party(workshop_destination)
		if demonstrate_workshop:
			scene.refresh_snapshot()
			if workshop_stage == 0:
				scene.salvage_action()
				print("ISLAND workshop salvage=", scene.snapshot.salvage, " notice=", scene.save_notice)
				scene.port.move_island_party(workshop_destination)
				workshop_stage = 1
			elif workshop_stage == 1:
				for person in state.actors:
					if person.id == scene.MICHAEL and person.x == workshop_destination.x and person.y == workshop_destination.y:
						scene.workshop_action()
						print("ISLAND workshop construction step=", step, " notice=", scene.save_notice)
						workshop_stage = 2
			elif workshop_stage == 2:
				for building in state.buildings:
					if building.faction == "faction.michael" and building.operational:
						scene.machine_action()
						print("ISLAND machine ordered step=", step, " salvage=", scene.snapshot.salvage, " notice=", scene.save_notice)
						workshop_stage = 3
			elif workshop_stage == 3:
				for person in state.actors:
					if person.definition == "actor_def.michael.mechanical_dog":
						print("ISLAND machine born step=", step, " unit=", person, " appearance=", not scene.appearance_for(person).is_empty(), " party=", state.party)
						var saved_machine: String = scene.port.save_island()
						print("ISLAND machine saved and loaded=", scene.port.load_island(saved_machine))
						var escort_destination := Vector2i(20,18)
						print("ISLAND escort order=", scene.port.move_island_party(escort_destination), " reason=", scene.port.party_move_failure(escort_destination))
						workshop_stage = 4
						machine_seen = true
			if workshop_stage == 4 and step % 20 == 0:
				for person in state.actors:
					if person.id == scene.MICHAEL or person.definition == "actor_def.michael.mechanical_dog":
						print("ISLAND escort step=", step, " unit=", person.id, " x=", person.x, " y=", person.y, " health=", person.health)
		for person in state.actors:
			if person.get("madness_stage", "") == "converted" and not seen_conversions.has(person.id):
				seen_conversions[person.id] = true
				print("ISLAND madness step=", step, " id=", person.id, " name=", person.name,
					" faction=", person.faction, " undead=", person.undead)
		if demonstrate_talk and not conversation_complete:
			scene.refresh_snapshot()
			if not talk_target.is_empty() and not state.actors.any(func(person): return person.id == talk_target):
				talk_target = ""
			if talk_target.is_empty():
				for person in state.actors:
					var desired_person: bool = person.sex == "female" and person.get("madness_stage", "") == "converted" if reclaim_converted else person.sex == "male"
					if person.id != scene.MICHAEL and desired_person and int(person.get("age",0)) >= 18:
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
				if reclaim_converted:
					scene.recruit_action()
					scene.assign_action(0)
					print("ISLAND reclamation step=", step, " person=", talk_target,
						" faction=",scene.inspected_person.get("faction",""),
						" madness_stage=",scene.inspected_person.get("madness_stage",""),
						" loyal=",scene.inspected_person.get("loyal_to_michael",false),
						" party=",scene.snapshot.get("party",[]))
				conversation_complete = true
				scene.set_paused(false)
		var holdings: Array = []
		for building in state.buildings:
			var remaining := int(building.get("repair_remaining", 0))
			if remaining > 0 and not seen_repairs.has(building.id):
				seen_repairs[building.id] = true
				print("ISLAND repair started step=", step, " holding=", building.id, " health=", building.health, " remaining=", remaining)
			elif remaining == 0 and seen_repairs.has(building.id):
				seen_repairs.erase(building.id)
				print("ISLAND repair finished step=", step, " holding=", building.id, " health=", building.health)
			if building.faction == "faction.colonial_powers.prototype":
				holdings.append({"id":building.id, "x":building.x, "y":building.y,
					"operational":building.operational, "level":building.level,
					"construction":building.get("construction_remaining",0)})
		var signature := JSON.stringify(holdings)
		if step <= 96 and step % 8 == 0:
			var early: Dictionary = JSON.parse_string(scene.port.save_island())
			var pressure := 0
			for person in early.world.actors.values():
				pressure = maxi(pressure, int(person.get("madness",0)))
				if str(person.get("current_assignment_id", "")).begins_with("repair."):
					print("ISLAND repair duty step=", step, " actor=", person.instance_id, " position=", early.world.positions.get(person.instance_id), " order=", early.world.travel_orders.get(person.instance_id))
			var shrine_health := 0
			for building in early.world.factions.get("faction.cthulhu.prototype",{}).get("buildings",{}).values():
				shrine_health += int(building.health)
			print("ISLAND ritual step=",step," shrine_health=",shrine_health," max_exposure=",pressure)
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
	if demonstrate_workshop and not machine_seen:
		print("ISLAND workshop machine not reached in this bounded run")
	if demonstrate_salvage and not salvage_complete:
		print("ISLAND ruined holding salvage not reached in this bounded run")
	scene.queue_free()
	await process_frame
	quit()
