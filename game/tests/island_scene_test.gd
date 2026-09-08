extends SceneTree

# Godot parses every JSON number as float; the Rust save schema uses integers.
func save_integers(value: Variant) -> Variant:
	if value is float:
		assert(value == floor(value))
		return int(value)
	if value is Dictionary:
		for key in value:
			value[key] = save_integers(value[key])
	elif value is Array:
		for index in value.size():
			value[index] = save_integers(value[index])
	return value

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var entry: String = ProjectSettings.get_setting("application/run/main_scene")
	assert(entry == "res://scenes/world/island.tscn")
	var scene = load(entry).instantiate()
	root.add_child(scene)
	await process_frame
	scene.set_process(false)
	assert(scene.ready_ok)
	var fresh_campaign: String = scene.port.save_island()
	# Camera is presentation-only: wheel, drag, focus and resize never move actors,
	# advance the world or change a queued order, including while paused.
	scene.set_paused(true)
	var camera_save: String = scene.port.save_island()
	var viewport_center: Vector2 = scene.get_viewport_rect().size * 0.5
	assert(is_equal_approx(scene.camera_zoom, 2.0))
	scene.show_island()
	scene.zoom_camera(2.0, viewport_center)
	assert(is_equal_approx(scene.camera_zoom, 2.0))
	var anchor := viewport_center + Vector2(40,20)
	var land_under_cursor: Vector2 = scene.world_at_screen(anchor)
	var wheel := InputEventMouseButton.new()
	wheel.button_index = MOUSE_BUTTON_WHEEL_UP
	wheel.pressed = true
	wheel.position = anchor
	scene._unhandled_input(wheel)
	assert(scene.world_at_screen(anchor).distance_to(land_under_cursor) < 0.001)
	var drag_start := InputEventMouseButton.new()
	drag_start.button_index = MOUSE_BUTTON_MIDDLE
	drag_start.pressed = true
	scene._unhandled_input(drag_start)
	assert(scene.camera_dragging)
	var drag := InputEventMouseMotion.new()
	drag.button_mask = MOUSE_BUTTON_MASK_MIDDLE
	drag.relative = Vector2(30,20)
	var center_before_drag: Vector2 = scene.camera_center
	var expected_center: Vector2 = center_before_drag - drag.relative / scene.map_root.scale.x
	scene._input(drag)
	assert(scene.camera_center.distance_to(expected_center) < 0.001)
	drag_start.pressed = false
	scene._input(drag_start)
	assert(not scene.camera_dragging)
	scene._input(drag)
	assert(scene.camera_center.distance_to(expected_center) < 0.001)
	scene.center_button.pressed.emit()
	assert(scene.camera_center.distance_to(scene.actor.position) < 0.001)
	# Picking uses the same transformed ground coordinates at any camera setting.
	var zoom_inspect := InputEventMouseButton.new()
	zoom_inspect.button_index = MOUSE_BUTTON_LEFT
	zoom_inspect.pressed = true
	zoom_inspect.shift_pressed = true
	zoom_inspect.position = scene.map_root.get_global_transform_with_canvas() * (scene.actor.position - Vector2(0,12))
	scene._unhandled_input(zoom_inspect)
	assert(scene.inspected_id == scene.MICHAEL)
	scene.inspect_actor("")
	scene.zoom_camera(100.0, viewport_center)
	assert(scene.camera_zoom == 4.0 and scene.zoom_in_button.disabled)
	scene.pan_camera(Vector2(100000,100000))
	assert(scene.map_root.position.x <= 0.001 and scene.map_root.position.y <= 0.001)
	scene.pan_camera(Vector2(-200000,-200000))
	var far_corner: Vector2 = scene.map_root.position + scene.map_size * scene.map_root.scale
	assert(far_corner.x >= scene.get_viewport_rect().size.x - 0.001)
	assert(far_corner.y >= scene.get_viewport_rect().size.y - 0.001)
	var camera_before_invalid: Vector2 = scene.camera_center
	scene.zoom_camera(NAN, viewport_center)
	scene.pan_camera(Vector2(INF,0))
	assert(scene.camera_center == camera_before_invalid and scene.camera_zoom == 4.0)
	var original_size: Vector2i = root.size
	root.size = Vector2i(960,720)
	await process_frame
	scene._fit()
	assert(scene.camera_zoom == 4.0)
	assert(scene.map_root.scale.is_finite() and scene.map_root.position.is_finite())
	root.size = original_size
	await process_frame
	scene.overview_button.pressed.emit()
	assert(scene.camera_zoom == 1.0 and scene.zoom_out_button.disabled)
	assert(scene.camera_center == scene.map_size * 0.5)
	drag_start.pressed = true
	scene._unhandled_input(drag_start)
	scene._notification(Node.NOTIFICATION_WM_WINDOW_FOCUS_OUT)
	assert(not scene.camera_dragging)
	assert(scene.port.save_island() == camera_save)
	scene.set_paused(false)
	assert(scene.port.save_island() == fresh_campaign)
	print("PASS: bounded cursor zoom, drag, focus, resize and transformed picking preserve the paused native world")
	assert(not scene.inspection.visible)
	var inspect_click := InputEventMouseButton.new()
	inspect_click.button_index = MOUSE_BUTTON_LEFT
	inspect_click.pressed = true
	inspect_click.shift_pressed = true
	inspect_click.position = scene.map_root.get_global_transform_with_canvas() * (scene.actor.position - Vector2(0,12))
	var before_inspection: String = scene.port.save_island()
	scene._unhandled_input(inspect_click)
	assert(scene.inspected_id == scene.MICHAEL and scene.inspection.visible)
	assert(scene.inspected_person.name == "Michael")
	assert(scene.inspection.text.contains(scene.inspected_person.biography))
	assert(scene.port.save_island() == before_inspection)
	inspect_click.position = scene.map_root.get_global_transform_with_canvas() * Vector2.ZERO
	scene._unhandled_input(inspect_click)
	assert(scene.inspected_id.is_empty() and not scene.inspection.visible)
	assert(scene.port.save_island() == before_inspection)
	assert(scene.pause_button.text == "Pause")
	scene.pause_button.pressed.emit()
	assert(scene.paused and scene.pause_button.text == "Resume")
	scene.campaign_path = "user://island-hud-controls-test.json"
	scene.save_button.pressed.emit()
	assert(scene.save_notice == "Saved")
	scene.pause_button.pressed.emit()
	assert(not scene.paused)
	scene.load_button.pressed.emit()
	assert(scene.paused and scene.save_notice == "Loaded")
	scene.pause_button.pressed.emit()
	assert(not scene.paused)
	for suffix in ["", ".bak", ".tmp"]:
		if FileAccess.file_exists(scene.campaign_path + suffix):
			DirAccess.remove_absolute(scene.campaign_path + suffix)
	assert(scene.pause_button.mouse_filter == Control.MOUSE_FILTER_STOP)
	assert(scene.hud_panel.size.x <= scene.get_viewport_rect().size.x)
	print("PASS: visible pause/save/load actions share native campaign state and preserve test-only files")
	assert(scene.snapshot.buildings.size() == 5)
	assert(scene.building_sprites.size() == 5)
	for building in scene.snapshot.buildings:
		assert(building.operational)
		assert(building.queued == 0)
		if scene.building_art.has(building.archetype):
			assert(scene.request_move(Vector2i(building.x, building.y)))
			for offset in scene.building_art[building.archetype].blocked_offsets:
				assert(not scene.request_move(Vector2i(building.x + offset[0], building.y + offset[1])))
			assert(scene.building_sprites[building.id].position == (Vector2(building.x,building.y) + Vector2.ONE * 0.5) * 32)
			assert(scene.building_sprites[building.id].texture.get_image().detect_alpha() == Image.ALPHA_BIT)
			if building.archetype == "site_archetype.pirates.tide_quay":
				assert(Vector2i(building.x,building.y) == Vector2i(34,22))
				assert(scene.building_sprites[building.id].texture == scene.building_textures["res://assets/island/tide_quay.png"])
			if building.archetype == "site_archetype.cthulhu.drowned_shrine":
				assert(Vector2i(building.x,building.y) == Vector2i(27,16))
				assert(scene.building_sprites[building.id].texture == scene.building_textures["res://assets/island/drowned_shrine.png"])
			if building.archetype == "site_archetype.elven.heart_grove":
				assert(Vector2i(building.x,building.y) == Vector2i(15,24))
				assert(scene.building_sprites[building.id].texture == scene.building_textures["res://assets/island/heart_grove.png"])
	assert(scene.troop_textures.size() == 9)
	assert(scene.troop_textures.has("res://assets/island/troops/mechanical-dog.png"))
	assert(scene.troop_textures.has("res://assets/island/troops/elven-bow-warden.png"))
	assert(scene.troop_textures.has("res://assets/island/troops/cultist-female.png"))
	assert(scene.troop_textures.has("res://assets/island/troops/colonial-female.png"))
	for texture in scene.troop_textures.values():
		assert(texture.get_image().detect_alpha() == Image.ALPHA_BIT)
	assert(not scene.request_move(Vector2i(0,0)))
	assert(not scene.save_notice.is_empty() and scene.status.text.contains(scene.save_notice))
	var start: Vector2 = scene.actor.position
	assert(scene.request_move(Vector2i(20,16)))
	scene.set_paused(true)
	scene.advance_tick()
	assert(scene.actor.position == start)
	scene.set_paused(false)
	for step in range(2):
		scene.advance_tick()
	assert(scene.actor.position != start)
	assert(scene.actor.position == Vector2(20.5,16.5) * 32)
	assert(scene.actor.facing == "ne")
	var path := "user://island-scene-persistence-test.json"
	scene.set_paused(true)
	assert(scene.save_campaign(path))
	var saved_position: Vector2 = scene.actor.position
	scene.set_paused(false)
	assert(scene.request_move(Vector2i(20,18)))
	scene.advance_tick()
	assert(scene.actor.position != saved_position)
	assert(scene.load_campaign(path))
	assert(scene.actor.position == saved_position and scene.paused)
	assert(scene.save_campaign(path))
	var corrupt := FileAccess.open(path, FileAccess.WRITE)
	corrupt.store_string("{broken")
	corrupt.close()
	assert(scene.load_campaign(path)) # backup survives corrupt primary
	var before: Dictionary = scene.port.island_snapshot()
	assert(not scene.port.load_island("{broken"))
	assert(scene.port.island_snapshot() == before)
	for suffix in ["", ".bak", ".tmp"]:
		if FileAccess.file_exists(path + suffix):
			DirAccess.remove_absolute(path + suffix)
	print("PASS: save/load restores native position and pause; corrupt primary falls back; invalid load preserves live state")
	scene.set_paused(false)
	var observed_strikes := 0
	for step in range(60):
		scene.advance_tick()
		assert(scene.hit_effects.size() == scene.last_strikes.size())
		observed_strikes += scene.last_strikes.size()
		for index in scene.last_strikes.size():
			var strike: Dictionary = scene.last_strikes[index]
			assert(strike.damage > 0)
			assert(scene.hit_effects[index].points[1] == (Vector2(strike.destination) + Vector2.ONE * 0.5) * 32 - Vector2(0,12))
	assert(observed_strikes > 0)
	scene.set_paused(true)
	var paused_effects: Array = scene.hit_effects.duplicate()
	scene.advance_tick()
	assert(scene.hit_effects == paused_effects)
	scene.set_paused(false)
	assert(scene.snapshot.actors.size() <= 19)
	assert(scene.snapshot.casualties > 0)
	assert(scene.troop_sprites.size() == scene.snapshot.actors.size() - 1)
	var factions := {}
	for unit in scene.snapshot.actors:
		if unit.id != scene.MICHAEL:
			factions[unit.faction] = true
			assert(scene.troop_sprites[unit.id].material == null)
			assert(scene.troop_sprites[unit.id].position == (Vector2(unit.x,unit.y) + Vector2.ONE * 0.5) * 32)
	assert(factions.size() >= 1)
	var survivors: int = scene.troop_sprites.size()
	var inspected_unit: Dictionary = {}
	for unit in scene.snapshot.actors:
		if unit.id != scene.MICHAEL:
			inspected_unit = unit
			break
	assert(not inspected_unit.is_empty())
	assert(not inspected_unit.name.is_empty() and not inspected_unit.biography.is_empty())
	scene.inspect_actor(inspected_unit.id)
	assert(scene.inspection.text.contains(inspected_unit.name))
	assert(scene.inspection.text.contains(inspected_unit.biography))
	var payload: String = scene.port.save_island()
	# Ownership never determines costume, even when a sprite must be recreated.
	var appearance_before: Dictionary = scene.appearance_for(inspected_unit)
	var allegiance_probe: Dictionary = save_integers(JSON.parse_string(payload))
	allegiance_probe.world.actors[inspected_unit.id].faction_id = "faction.michael"
	var moved_population: int = allegiance_probe.world.unit_combat[inspected_unit.id].population_use
	allegiance_probe.world.factions[inspected_unit.faction].population_used -= moved_population
	allegiance_probe.world.factions["faction.michael"].population_used += moved_population
	allegiance_probe.world.factions["faction.michael"].population_capacity = maxi(allegiance_probe.world.factions["faction.michael"].population_capacity, allegiance_probe.world.factions["faction.michael"].population_used)
	assert(scene.port.load_island(JSON.stringify(allegiance_probe)))
	scene.troop_sprites[inspected_unit.id].queue_free()
	scene.troop_sprites.erase(inspected_unit.id)
	scene.refresh_snapshot()
	assert(scene.troop_sprites[inspected_unit.id].texture == scene.troop_textures[appearance_before.texture])
	assert(scene.troop_sprites[inspected_unit.id].offset == -Vector2(appearance_before.pivot[0], appearance_before.pivot[1]))
	var unadmitted: Dictionary = inspected_unit.duplicate(true)
	unadmitted.sex = "female"
	unadmitted.definition = "actor_def.colonial.line_marine"
	assert(scene.appearance_for(unadmitted).texture == "res://assets/island/troops/colonial-female.png")
	unadmitted.definition = "actor_def.cthulhu.drowned_cultist"
	assert(scene.appearance_for(unadmitted).texture == "res://assets/island/troops/cultist-female.png")
	unadmitted.sex = "other"
	assert(scene.appearance_for(unadmitted).is_empty()) # no unadmitted identity fallback
	unadmitted.definition = "actor_def.colonial.line_marine"
	unadmitted.sex = "unknown"
	assert(scene.appearance_for(unadmitted).texture == "res://assets/island/troops/colonial.png") # explicit legacy presentation only
	unadmitted.definition = "actor_def.not_admitted"
	assert(scene.appearance_for(unadmitted).is_empty())
	print("PASS: character art survives allegiance change and sprite recreation; unadmitted identities have no false fallback")
	assert(scene.port.load_island(payload))
	scene.refresh_snapshot()
	assert(scene.troop_sprites.size() == survivors)
	assert(scene.inspected_id == inspected_unit.id and scene.inspected_person == inspected_unit)
	print("PASS: Shift-click inspects native identities without movement, selection survives snapshot and save restoration")
	# Building projection is restored from state, including removal and operation.
	var building_count_before: int = scene.building_sprites.size()
	var saved: Dictionary = save_integers(JSON.parse_string(payload))
	var colonial: Dictionary = saved.world.factions["faction.colonial_powers.prototype"]
	var fort_id: String = colonial.buildings.keys()[0]
	colonial.buildings[fort_id].operational = false
	assert(scene.port.load_island(JSON.stringify(saved)))
	scene.refresh_snapshot()
	assert(scene.building_sprites[fort_id].self_modulate != Color.WHITE)
	saved.world.policies.erase("faction.colonial_powers.prototype")
	colonial.buildings.clear()
	saved.world.navigation.building_obstacles.erase(fort_id)
	assert(scene.port.load_island(JSON.stringify(saved)))
	scene.refresh_snapshot()
	assert(not scene.building_sprites.has(fort_id))
	assert(scene.building_sprites.size() == building_count_before - 1)
	assert(scene.port.load_island(payload))
	scene.refresh_snapshot()
	assert(scene.building_sprites.size() == building_count_before)
	print("PASS: native building positions, transparent fort, operating state and removal/load projection")
	print("PASS: native strike events project one effect per hit at recorded target positions; pause preserves effects")
	print("PASS: autonomous skirmish casualties, capped survivors, removed dead sprites, roster preserved through load")
	print("PASS: real island scene, authored land, sprite/native position agreement, travel and pause")
	var combat_save: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	var enemy: Dictionary = {}
	for unit in scene.snapshot.actors:
		if unit.id != scene.MICHAEL:
			enemy = unit
			break
	assert(not enemy.is_empty())
	combat_save.world.positions[scene.MICHAEL] = {"x": int(enemy.x), "y": int(enemy.y)}
	assert(scene.port.load_island(JSON.stringify(combat_save)))
	scene.refresh_snapshot()
	assert(scene.port.aim_island_carbine(enemy.id))
	scene.advance_tick()
	assert(scene.last_strikes.any(func(hit): return hit.attacker == scene.MICHAEL))
	assert(scene.status.text.contains("Michael HP"))
	print("PASS: player carbine command crosses native bridge and produces authoritative hit feedback")
	var lethal: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	# The first target may have died from Michael's shot; select a survivor.
	for unit in scene.snapshot.actors:
		if unit.id != scene.MICHAEL:
			enemy = unit
			break
	lethal.world.hostilities = [[enemy.faction, "faction.michael"]]
	lethal.world.unit_combat[scene.MICHAEL].health = 1
	lethal.world.unit_combat[enemy.id].next_attack_tick = 0
	lethal.world.positions[enemy.id] = lethal.world.positions[scene.MICHAEL].duplicate()
	lethal.world.travel_orders.erase(enemy.id)
	lethal.world.travel_orders.erase(scene.MICHAEL)
	lethal.world.policies.clear()
	assert(scene.port.load_island(JSON.stringify(lethal)))
	scene.refresh_snapshot()
	scene.inspect_actor(scene.MICHAEL)
	var remembered_name: String = scene.inspected_person.name
	scene.advance_tick()
	assert(scene.inspected_id == scene.MICHAEL)
	assert(scene.inspection.text.contains(remembered_name) and scene.inspection.text.contains("Dead or departed"))
	assert(scene.paused and not scene.actor.visible)
	assert(scene.pause_button.disabled and scene.pause_button.text == "Fallen")
	assert(scene.status.text.contains("Michael has fallen"))
	var defeat_path := "user://island-scene-defeat-test.json"
	assert(scene.save_campaign(defeat_path))
	assert(scene.port.load_island(payload))
	scene.paused = false
	scene.refresh_snapshot()
	assert(scene.actor.visible)
	assert(not scene.inspection.text.contains("Dead or departed"))
	assert(scene.load_campaign(defeat_path))
	assert(scene.paused and not scene.actor.visible)
	assert(scene.inspection.text.contains(remembered_name) and scene.inspection.text.contains("Dead or departed"))
	assert(not scene.request_move(Vector2i(20,18)))
	assert(not scene.port.aim_island_carbine(enemy.id))
	for suffix in ["", ".bak", ".tmp"]:
		if FileAccess.file_exists(defeat_path + suffix):
			DirAccess.remove_absolute(defeat_path + suffix)
	print("PASS: actual lethal retaliation, paused defeat, save/reload defeat, no dead movement or attacks")
	if not check_recruitment(scene, fresh_campaign):
		quit(1)
		return
	if not check_fresh_recruitment(scene, fresh_campaign):
		quit(1)
		return
	if not check_holding_development(scene, fresh_campaign):
		return
	if not check_coastal_save_upgrade(scene, fresh_campaign):
		quit(1)
		return
	print("PASS: island scene suite complete")
	quit()

func check_recruitment(scene: Node, fresh_campaign: String) -> bool:
	assert(scene.port.load_island(fresh_campaign))
	scene.paused = false
	scene.refresh_snapshot()
	assert(scene.snapshot.party == ["", "", "", ""])
	var woman: Dictionary = {}
	for step in range(30):
		scene.advance_tick()
		for entry in scene.snapshot.actors:
			if entry.sex == "female":
				woman = entry
				break
		if not woman.is_empty():
			break
	assert(not woman.is_empty()) # actual production, not an invented companion
	var woman_sprite: Sprite2D = scene.troop_sprites[woman.id]
	var woman_bar: ProgressBar = woman_sprite.get_node("Health")
	assert((woman_bar.scale * woman_sprite.scale).distance_to(Vector2.ONE) < 0.001)
	assert(woman_bar.size == Vector2(32,4))
	assert(woman_bar.position.y * woman_sprite.scale.y < woman_sprite.offset.y * woman_sprite.scale.y)
	for sprite in scene.troop_sprites.values():
		var bar: ProgressBar = sprite.get_node("Health")
		assert((bar.scale * sprite.scale).distance_to(Vector2.ONE) < 0.001)
		assert(bar.size == woman_bar.size)
	scene.inspect_actor(woman.id)
	assert(scene.talk_button.visible and not scene.recruit_button.visible)
	assert(scene.talk_button.mouse_filter == Control.MOUSE_FILTER_STOP)
	var before_talk: String = scene.port.save_island()
	assert(not scene.port.recruit_island_person(woman.id))
	assert(scene.port.save_island() == before_talk)
	scene.talk_button.pressed.emit() # initial producer is outside talking range
	assert(not scene.recruit_button.visible)
	assert(scene.conversation_status.text.contains("closer"))
	# Isolate this generated encounter from the ongoing war and put Michael nearby.
	var nearby: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	nearby.world.positions[scene.MICHAEL] = {"x": int(woman.x), "y": int(woman.y)}
	nearby.world.policies.clear()
	nearby.world.hostilities.clear()
	nearby.world.travel_orders.clear()
	assert(scene.port.load_island(JSON.stringify(nearby)))
	scene.refresh_snapshot()
	scene.set_paused(true)
	var encounter_tick: int = scene.snapshot.tick
	scene.talk_button.pressed.emit()
	assert(scene.inspected_person.discussed and scene.recruit_button.visible)
	assert(not scene.inspected_person.recruitment_offer.is_empty())
	assert(scene.inspection.text.contains(scene.inspected_person.recruitment_offer))
	var before_join: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	var roster_count: int = scene.snapshot.actors.size()
	scene.recruit_button.pressed.emit()
	assert(scene.snapshot.actors.size() == roster_count)
	assert(scene.inspected_person.id == woman.id and scene.inspected_person.name == woman.name)
	assert(scene.inspected_person.biography == woman.biography and scene.inspected_person.health == woman.health)
	assert(scene.inspected_person.faction == "faction.michael" and scene.inspected_person.loyal_to_michael)
	assert(scene.paused and scene.snapshot.tick == encounter_tick)
	assert(not scene.recruit_button.visible and scene.party_controls.visible)
	assert(scene.talk_button.visible and scene.approach_button.visible)
	var companion_line: String = scene.inspected_person.dialogue
	assert(not companion_line.is_empty() and companion_line != scene.inspected_person.recruitment_offer)
	assert(scene.inspection.text.contains(companion_line))
	assert(not scene.inspection.text.contains(scene.inspected_person.recruitment_offer))
	scene.talk_button.pressed.emit()
	assert(scene.inspected_person.dialogue == companion_line)
	assert(scene.snapshot.tick == encounter_tick and scene.snapshot.actors.size() == roster_count)
	assert(scene.snapshot.party == ["", "", "", ""]) # faction membership is not a party slot
	var after_join: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	assert(after_join.world.factions[woman.faction].population_used == before_join.world.factions[woman.faction].population_used - 1)
	assert(after_join.world.factions["faction.michael"].population_used == before_join.world.factions["faction.michael"].population_used + 1)
	var older_companion: Dictionary = after_join.duplicate(true)
	older_companion.world.actors[woman.id].person.erase("companion_response")
	older_companion.world.actors[woman.id].person.recruitment_offer = ""
	assert(scene.port.load_island(JSON.stringify(older_companion)))
	scene.refresh_snapshot()
	assert(scene.talk_button.visible and scene.approach_button.visible)
	assert(scene.port.can_talk_island_person(woman.id))
	assert(scene.port.talk_island_person(woman.id) == "I'm with you, Michael.")
	assert(scene.port.load_island(JSON.stringify(after_join)))
	scene.refresh_snapshot()
	scene.party_buttons[0].pressed.emit()
	assert(scene.snapshot.party[0] == woman.id)
	assert(scene.party_buttons[0].text.contains(woman.name))
	assert(scene.request_move(Vector2i(20,18)))
	assert(scene.snapshot.tick == encounter_tick)
	scene.set_paused(false)
	var party_save: String = scene.port.save_island()
	var ordered: Dictionary = save_integers(JSON.parse_string(party_save))
	assert(ordered.world.travel_orders.has(woman.id) and ordered.world.travel_orders.has(scene.MICHAEL))
	scene.advance_tick()
	assert(scene.port.save_island() != party_save)
	assert(Vector2i(scene.inspected_person.x, scene.inspected_person.y) != Vector2i(woman.x, woman.y))
	assert(scene.port.load_island(party_save))
	scene.refresh_snapshot()
	assert(scene.snapshot.party[0] == woman.id and scene.inspected_person.loyal_to_michael)
	assert(scene.inspected_person.name == woman.name and scene.inspected_person.discussed)
	assert(scene.inspected_person.dialogue == companion_line and scene.talk_button.visible)
	assert(scene.party_heading.text.contains("1/4 active"))
	scene.dismiss_buttons[0].pressed.emit()
	assert(scene.snapshot.party[0].is_empty() and scene.inspected_person.faction == "faction.michael")
	assert(scene.talk_button.visible and scene.inspected_person.dialogue == companion_line)
	assert(scene.port.load_island(party_save))
	scene.refresh_snapshot()
	var lethal: Dictionary = save_integers(JSON.parse_string(party_save))
	var attacker: Dictionary = {}
	for entry in scene.snapshot.actors:
		if entry.faction != "faction.michael":
			attacker = entry
			break
	assert(not attacker.is_empty())
	lethal.world.positions[scene.MICHAEL] = {"x": 20, "y": 18}
	lethal.world.positions[attacker.id] = lethal.world.positions[woman.id].duplicate()
	lethal.world.travel_orders.clear()
	lethal.world.unit_combat[woman.id].health = 1
	lethal.world.unit_combat[attacker.id].next_attack_tick = 0
	lethal.world.hostilities = [[attacker.faction, "faction.michael"]]
	assert(scene.port.load_island(JSON.stringify(lethal)))
	scene.refresh_snapshot()
	scene.advance_tick()
	assert(scene.snapshot.party[0] == woman.id and scene.snapshot.party_names[0] == woman.name)
	assert(scene.party_heading.text.contains("0/4 active"))
	assert(scene.party_buttons[0].text.contains(woman.name) and scene.party_buttons[0].text.contains("Fallen"))
	var fallen_save: String = scene.port.save_island()
	assert(scene.port.load_island(fallen_save))
	scene.refresh_snapshot()
	assert(scene.party_buttons[0].text.contains(woman.name))
	# Accelerated boundary fixture after an actual combat death, not a fake return.
	var before_midnight: Dictionary = save_integers(JSON.parse_string(fallen_save))
	before_midnight.world.tick = 1439
	before_midnight.world.hostilities.clear()
	before_midnight.world.travel_orders.clear()
	assert(scene.port.load_island(JSON.stringify(before_midnight)))
	scene.refresh_snapshot()
	assert(scene.snapshot.day == 1 and scene.snapshot.minute_of_day == 1439)
	scene.set_paused(true)
	scene.advance_tick()
	assert(scene.snapshot.minute_of_day == 1439)
	scene.set_paused(false)
	scene.advance_tick()
	assert(scene.snapshot.day == 2 and scene.snapshot.minute_of_day == 0)
	assert(scene.inspected_person.id == woman.id and scene.inspected_person.undead)
	assert(scene.inspected_person.faction == "faction.cthulhu.prototype")
	assert(scene.party_buttons[0].text.contains("With Cthulhu"))
	assert(scene.party_heading.text.contains("0/4 active"))
	assert(scene.troop_sprites[woman.id].self_modulate != Color.WHITE)
	assert(scene.inspection.text.contains("Undead"))
	var returned: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	returned.world.positions[scene.MICHAEL] = returned.world.positions[woman.id].duplicate()
	assert(scene.port.load_island(JSON.stringify(returned)))
	scene.refresh_snapshot()
	scene.set_paused(true)
	scene.talk_button.pressed.emit()
	scene.recruit_button.pressed.emit()
	assert(scene.inspected_person.faction == "faction.michael")
	assert(scene.inspected_person.id == woman.id and scene.inspected_person.name == woman.name)
	assert(scene.snapshot.party[0] == woman.id)
	assert(scene.party_heading.text.contains("1/4 active"))
	assert(scene.inspected_person.undead) # allegiance recovery is not bodily resurrection
	print("PASS: actual produced female dialogue, explicit same-identity recruitment, population transfer, four-slot party, group order and save restore")
	assert(check_foothold(scene, woman.id))
	return true

func check_foothold(scene: Node, woman_id: String) -> bool:
	# Continue the real death/midnight/reclaim scenario above. No salvage or
	# building is injected: walk to the finite cache and build through UI commands.
	assert(scene.snapshot.salvage == 0 and scene.inspected_person.undead)
	var cache: Dictionary = scene.snapshot.salvage_caches.filter(func(c): return c.id == "salvage.wreck")[0]
	assert(cache.remaining == 20 and scene.salvage_markers.has(cache.id))
	assert(scene.request_move(Vector2i(cache.x, cache.y)))
	scene.set_paused(false)
	for step in range(70):
		scene.advance_tick()
	assert(scene.actor.position == (Vector2(cache.x, cache.y) + Vector2.ONE * 0.5) * scene.cell_size)
	scene.set_paused(true)
	scene.salvage_button.pressed.emit()
	assert(scene.snapshot.salvage == 20 and not scene.salvage_markers.has(cache.id))
	var gathered: String = scene.port.save_island()
	scene.salvage_action()
	assert(scene.port.save_island() == gathered)
	assert(scene.request_move(Vector2i(19,17)))
	scene.set_paused(false)
	for step in range(12):
		scene.advance_tick()
	assert(scene.actor.position == Vector2(19.5,17.5) * scene.cell_size)
	scene.set_paused(true)
	scene.workshop_button.pressed.emit()
	assert(scene.snapshot.salvage == 8)
	var sites: Array = scene.snapshot.buildings.filter(func(b): return b.id == "site.michael.field_workshop")
	assert(sites.size() == 1 and sites[0].construction_remaining == 40 and not sites[0].operational)
	assert(scene.building_sprites.has(sites[0].id))
	var paused_work: String = scene.port.save_island()
	scene.advance_tick()
	assert(scene.port.save_island() == paused_work)
	assert(scene.port.load_island(paused_work))
	scene.refresh_snapshot()
	scene.set_paused(false)
	for step in range(40):
		scene.advance_tick()
	sites = scene.snapshot.buildings.filter(func(b): return b.id == "site.michael.field_workshop")
	assert(sites[0].operational and sites[0].construction_remaining == 0 and sites[0].level == 1)
	scene.inspect_actor(woman_id)
	assert(scene.inspected_person.undead and scene.restore_button.visible)
	scene.set_paused(true)
	var before_restore: Dictionary = scene.inspected_person.duplicate(true)
	scene.restore_button.pressed.emit()
	assert(not scene.inspected_person.undead and scene.snapshot.salvage == 4)
	assert(scene.inspected_person.id == before_restore.id and scene.inspected_person.name == before_restore.name)
	assert(scene.inspected_person.faction == "faction.michael" and scene.snapshot.party[0] == woman_id)
	assert(scene.troop_sprites[woman_id].self_modulate == Color.WHITE)
	assert(not scene.restore_button.visible)
	var completed: String = scene.port.save_island()
	assert(scene.port.load_island(completed))
	assert(scene.port.save_island() == completed)
	print("PASS: walk to finite salvage, paid workshop construction with pause/save, then explicit paid restoration of the same reclaimed undead companion")
	return true

func check_fresh_recruitment(scene: Node, fresh_campaign: String) -> bool:
	# Begin with the untouched initial campaign. No coordinate edits, invented
	# people, disabled wars, hostility changes or resources added in this path.
	assert(scene.port.load_island(fresh_campaign))
	scene.paused = false
	scene.refresh_snapshot()
	var woman: Dictionary = {}
	for step in range(12):
		scene.advance_tick()
		for entry in scene.snapshot.actors:
			if entry.sex == "female":
				woman = entry.duplicate(true)
				break
		if not woman.is_empty():
			break
	assert(not woman.is_empty())
	scene.inspect_actor(woman.id)
	assert(scene.approach_button.visible)
	scene.set_paused(true)
	var start: Vector2 = scene.actor.position
	scene.approach_button.pressed.emit()
	assert(scene.snapshot.approach_target == woman.id)
	assert(scene.approach_button.disabled)
	assert(scene.conversation_status.text.contains(woman.name))
	scene.advance_tick()
	assert(scene.actor.position == start)
	var approaching_save: String = scene.port.save_island()
	assert(scene.port.load_island(approaching_save))
	scene.refresh_snapshot()
	assert(scene.snapshot.approach_target == woman.id)
	scene.set_paused(false)
	var travelled := 0
	for step in range(40):
		var before: Vector2 = scene.actor.position
		scene.advance_tick()
		assert(scene.actor.visible)
		var delta: Vector2 = (scene.actor.position - before) / scene.cell_size
		assert(absf(delta.x) + absf(delta.y) <= 1.001)
		if not delta.is_zero_approx():
			travelled += 1
		if scene.snapshot.approach_target.is_empty():
			break
	assert(travelled > 0)
	assert(scene.snapshot.approach_target.is_empty())
	assert(scene.paused and scene.conversation_status.text.contains("Within talking range"))
	assert(not scene.inspected_person.discussed)
	scene.set_paused(true)
	scene.talk_button.pressed.emit()
	assert(scene.inspected_person.discussed and scene.recruit_button.visible)
	scene.recruit_button.pressed.emit()
	assert(scene.inspected_person.faction == "faction.michael")
	assert(scene.inspected_person.id == woman.id and scene.inspected_person.name == woman.name)
	scene.party_buttons[0].pressed.emit()
	assert(scene.snapshot.party[0] == woman.id)
	assert(scene.request_move(Vector2i(20,18)))
	scene.set_paused(false)
	for step in range(40):
		scene.advance_tick()
		if scene.actor.position == Vector2(20.5,18.5) * scene.cell_size:
			break
	assert(scene.actor.position == Vector2(20.5,18.5) * scene.cell_size)
	assert(scene.inspected_person.faction == "faction.michael")
	assert(scene.party_heading.text.contains("1/4 active"))
	print("PASS: untouched campaign produces a woman; Approach walks to her through the live war; talk/recruit/assign and return to start without fixture edits")
	return true

func check_holding_development(scene: Node, fresh_campaign: String) -> bool:
	assert(scene.port.load_island(fresh_campaign))
	scene.paused = false
	scene.refresh_snapshot()
	var saw_work := false
	var saw_completion := false
	for step in range(240):
		scene.advance_tick()
		for building in scene.snapshot.buildings:
			assert(building.level >= 1 and building.level <= 5)
			if building.development_remaining > 0:
				saw_work = true
				if scene.building_sprites.has(building.id):
					var structure: Sprite2D = scene.building_sprites[building.id]
					var bar: ProgressBar = structure.get_node("Development")
					assert((bar.scale * structure.scale).distance_to(Vector2.ONE) < 0.001)
					assert(bar.value == building.development_ticks - building.development_remaining)
			if building.level > 1:
				saw_completion = true
				assert(building.max_health > 80)
	assert(saw_work and saw_completion)
	var saved: String = scene.port.save_island()
	assert(scene.port.load_island(saved))
	assert(scene.port.save_island() == saved)
	print("PASS: untouched island factions fund and finish holding development during their ongoing war; native level, health and local progress survive save/load")
	return true

func check_coastal_save_upgrade(scene: Node, fresh_campaign: String) -> bool:
	assert(scene.port.load_island(fresh_campaign))
	var legacy: Dictionary = save_integers(JSON.parse_string(fresh_campaign))
	var corrections: Array = [[30,17],[30,18],[33,23],[34,23],[34,22]]
	legacy.world.navigation.walkable = legacy.world.navigation.walkable.filter(func(point): return not corrections.has([int(point.x),int(point.y)]))
	var holding := "preview.faction.pirates.prototype.holding"
	var quay := "preview.faction.pirates.prototype.producer"
	legacy.world.navigation.destinations[holding] = {"x":35,"y":14}
	legacy.world.navigation.building_obstacles.erase(quay)
	var shrine := "preview.faction.cthulhu.prototype.producer"
	var shrine_holding := "preview.faction.cthulhu.prototype.holding"
	legacy.world.navigation.destinations[shrine_holding] = {"x":28,"y":23}
	legacy.world.navigation.building_obstacles.erase(shrine)
	assert(scene.port.load_island(JSON.stringify(legacy)))
	scene.refresh_snapshot()
	assert(not scene.building_sprites.has(quay)) # no inland waterfront sprite
	assert(not scene.building_sprites.has(shrine)) # no new shrine over old palms
	var migrated: Dictionary = save_integers(JSON.parse_string(scene.port.save_island()))
	assert(migrated.world.navigation.destinations[holding] == {"x":35,"y":14})
	assert(migrated.world.navigation.destinations[shrine_holding] == {"x":28,"y":23})
	assert(migrated.world.navigation.walkable.size() == legacy.world.navigation.walkable.size() + corrections.size())
	assert(migrated.world.positions == legacy.world.positions)
	var preserved: String = scene.port.save_island()
	legacy.world.navigation.walkable.pop_back() # unrelated map change is not this upgrade
	assert(not scene.port.load_island(JSON.stringify(legacy)))
	assert(scene.port.save_island() == preserved)
	assert(scene.port.load_island(fresh_campaign))
	scene.refresh_snapshot()
	assert(scene.building_sprites.has(quay))
	assert(scene.building_sprites.has(shrine))
	print("PASS: coastal quay uses its own art/footprint; exact beach-mask upgrade preserves legacy inland holdings and rejects unrelated maps")
	return true
