extends Node2D

const Port = preload("res://scripts/simulation/native_simulation_port.gd")
const Actor = preload("res://scripts/world/directional_sprite.gd")
const MICHAEL := "character.protagonist.captain"
var port = Port.new()
var actor = Actor.new()
var map_root := Node2D.new()
var status := Label.new()
var hud_panel := PanelContainer.new()
var pause_button := Button.new()
var save_button := Button.new()
var load_button := Button.new()
var inspection := Label.new()
var conversation_actions := HBoxContainer.new()
var talk_button := Button.new()
var recruit_button := Button.new()
var conversation_notice := ""
var conversation_status := Label.new()
var party_controls := VBoxContainer.new()
var party_heading := Label.new()
var party_buttons: Array[Button] = []
var dismiss_buttons: Array[Button] = []
var inspected_id := ""
var inspected_person: Dictionary = {}
var campaign_path := "user://pirate-island-save.json"
var snapshot: Dictionary
var cell_size := 32
var elapsed := 0.0
var paused := false
var ready_ok := false
var save_notice := ""
var troop_sprites := {}
var troop_textures := {}
var troop_art: Dictionary
var missing_appearance_count := 0
var building_sprites := {}
var fort_texture: Texture2D
var building_art: Dictionary
var hit_effects: Array[Line2D] = []
var last_strikes: Array = []

func clear_hit_effects() -> void:
	for effect in hit_effects:
		effect.queue_free()
	hit_effects.clear()
	last_strikes.clear()

func save_action() -> void:
	save_notice = "Saved" if save_campaign(campaign_path) else "Save failed — previous save retained"
	refresh_snapshot()

func load_action() -> void:
	save_notice = "Loaded" if load_campaign(campaign_path) else "Could not load — current game unchanged"
	refresh_snapshot()

func save_campaign(path: String = "user://pirate-island-save.json") -> bool:
	var payload: String = port.save_island()
	if payload.is_empty():
		return false
	var file := FileAccess.open(path + ".tmp", FileAccess.WRITE)
	if file == null:
		return false
	file.store_string(payload)
	file.flush()
	var written := file.get_error() == OK
	file.close()
	if not written:
		return false
	# Preserve last primary until a complete replacement exists.
	if FileAccess.file_exists(path):
		var previous := FileAccess.open(path, FileAccess.READ)
		if previous == null:
			return false
		var previous_valid: bool = previous.get_length() <= 8 * 1024 * 1024 and port.valid_island_save(previous.get_as_text())
		previous.close()
		if previous_valid:
			if FileAccess.file_exists(path + ".bak") and DirAccess.remove_absolute(path + ".bak") != OK:
				return false
			if DirAccess.rename_absolute(path, path + ".bak") != OK:
				return false
		elif DirAccess.remove_absolute(path) != OK:
			return false
	if DirAccess.rename_absolute(path + ".tmp", path) != OK:
		if FileAccess.file_exists(path + ".bak"):
			DirAccess.rename_absolute(path + ".bak", path)
		return false
	return true

func load_campaign(path: String = "user://pirate-island-save.json") -> bool:
	for candidate in [path, path + ".bak"]:
		var file := FileAccess.open(candidate, FileAccess.READ)
		if file == null:
			continue
		if file.get_length() > 8 * 1024 * 1024:
			file.close()
			continue
		var payload := file.get_as_text()
		file.close()
		if port.load_island(payload):
			clear_hit_effects()
			paused = bool(port.island_snapshot().paused)
			elapsed = 0
			refresh_snapshot()
			return true
	return false

func _ready() -> void:
	RenderingServer.set_default_clear_color(Color("#073c48"))
	add_child(map_root)
	var data: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/island/navigation.json"))
	cell_size = int(data.cellSize)
	var polygon := PackedVector2Array()
	for point in data.landPolygon:
		polygon.append(Vector2(point[0], point[1]))
	var cells: Array[Vector2i] = []
	for y in range(int(data.size[1]) / cell_size):
		for x in range(int(data.size[0]) / cell_size):
			var center := Vector2(x + 0.5, y + 0.5) * cell_size
			if not Geometry2D.is_point_in_polygon(center, polygon):
				continue
			var blocked := false
			for rect in data.blockedRects:
				if Rect2(rect[0],rect[1],rect[2],rect[3]).grow(10).has_point(center):
					blocked = true
			if not blocked:
				cells.append(Vector2i(x,y))
	port.create_island()
	assert(port.configure_island_land(cells, Vector2i(data.start[0],data.start[1])))
	assert(port.install_preview_factions())
	building_art = JSON.parse_string(FileAccess.get_file_as_string("res://assets/island/buildings.json"))
	var fort_image := Image.load_from_file(building_art["site_archetype.colonial.watch_fort"].texture)
	assert(fort_image != null and fort_image.detect_alpha() == Image.ALPHA_BIT)
	fort_texture = ImageTexture.create_from_image(fort_image)
	troop_art = JSON.parse_string(FileAccess.get_file_as_string("res://assets/island/troops/appearances.json"))
	for definition in troop_art.values():
		for appearance in definition.variants.values():
			var cutout := Image.load_from_file(appearance.texture)
			assert(cutout != null and cutout.detect_alpha() == Image.ALPHA_BIT)
			assert(float(appearance.scale) > 0 and float(appearance.scale) <= 1)
			assert(appearance.pivot.size() == 2)
			troop_textures[appearance.texture] = ImageTexture.create_from_image(cutout)
	var terrain := Sprite2D.new()
	terrain.texture = ImageTexture.create_from_image(Image.load_from_file("res://assets/island/terrain.png"))
	terrain.centered = false
	terrain.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	map_root.add_child(terrain)
	map_root.add_child(actor)
	assert(actor.configure("res://assets/sprites/michael/source.png", "res://assets/sprites/michael/frames.json"))
	actor.scale = Vector2.ONE * 0.065
	var hud := CanvasLayer.new()
	add_child(hud)
	hud_panel.position = Vector2(12,12)
	var background := StyleBoxFlat.new()
	background.bg_color = Color(0.06, 0.11, 0.13, 0.94)
	background.content_margin_left = 12
	background.content_margin_right = 12
	background.content_margin_top = 8
	background.content_margin_bottom = 8
	hud_panel.add_theme_stylebox_override("panel", background)
	hud.add_child(hud_panel)
	var stack := VBoxContainer.new()
	hud_panel.add_child(stack)
	status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status.mouse_filter = Control.MOUSE_FILTER_IGNORE
	status.add_theme_color_override("font_shadow_color", Color.BLACK)
	status.add_theme_constant_override("shadow_offset_x", 2)
	status.add_theme_constant_override("shadow_offset_y", 2)
	stack.add_child(status)
	var actions := HBoxContainer.new()
	stack.add_child(actions)
	for button in [pause_button, save_button, load_button]:
		button.custom_minimum_size = Vector2(92,36)
		button.mouse_filter = Control.MOUSE_FILTER_STOP
		actions.add_child(button)
	pause_button.tooltip_text = "Pause or resume the island (Space)"
	save_button.text = "Save"
	save_button.tooltip_text = "Save this campaign (F5)"
	load_button.text = "Load"
	load_button.tooltip_text = "Load your saved campaign (F9)"
	pause_button.pressed.connect(func(): set_paused(not paused))
	save_button.pressed.connect(save_action)
	load_button.pressed.connect(load_action)
	inspection.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	inspection.mouse_filter = Control.MOUSE_FILTER_IGNORE
	inspection.visible = false
	stack.add_child(inspection)
	stack.add_child(conversation_actions)
	for button in [talk_button, recruit_button]:
		button.custom_minimum_size = Vector2(120,36)
		button.mouse_filter = Control.MOUSE_FILTER_STOP
		conversation_actions.add_child(button)
	talk_button.text = "Talk"
	recruit_button.text = "Join faction"
	talk_button.pressed.connect(talk_action)
	recruit_button.pressed.connect(recruit_action)
	conversation_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	conversation_status.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stack.add_child(conversation_status)
	stack.add_child(party_controls)
	party_heading.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	party_controls.add_child(party_heading)
	var slots := GridContainer.new()
	slots.columns = 2
	party_controls.add_child(slots)
	for slot in range(4):
		var row := HBoxContainer.new()
		row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		slots.add_child(row)
		var assign := Button.new()
		assign.custom_minimum_size = Vector2(90,36)
		assign.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		assign.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		assign.mouse_filter = Control.MOUSE_FILTER_STOP
		assign.pressed.connect(assign_action.bind(slot))
		row.add_child(assign)
		party_buttons.append(assign)
		var dismiss := Button.new()
		dismiss.text = "×"
		dismiss.custom_minimum_size = Vector2(36,36)
		dismiss.mouse_filter = Control.MOUSE_FILTER_STOP
		dismiss.pressed.connect(dismiss_action.bind(slot))
		row.add_child(dismiss)
		dismiss_buttons.append(dismiss)
	get_viewport().size_changed.connect(_fit)
	_fit()
	refresh_snapshot()
	ready_ok = true

func _fit() -> void:
	var area := get_viewport_rect().size
	hud_panel.size.x = maxf(310, minf(area.x - 24, 720))
	var zoom := minf(area.x / 1536.0, area.y / 1024.0)
	map_root.scale = Vector2.ONE * zoom
	map_root.position = ((area - Vector2(1536,1024) * zoom) * 0.5).round()

func request_move(cell: Vector2i) -> bool:
	var accepted: bool = port.move_island_party(cell)
	if not accepted:
		save_notice = port.party_move_failure(cell)
		if save_notice.is_empty():
			save_notice = "The party cannot reach that spot. Choose clear land nearby."
	else:
		save_notice = ""
	refresh_snapshot()
	return accepted

func set_paused(value: bool) -> void:
	paused = value
	port.pause_island(value)
	elapsed = 0.0
	refresh_snapshot()

func advance_tick() -> void:
	if paused:
		return
	clear_hit_effects()
	var result: Dictionary = port.tick_island()
	refresh_snapshot()
	last_strikes = result.get("strikes", [])
	for strike in last_strikes:
		var effect := Line2D.new()
		# Provisional chest-height effects, not authored weapon-socket animation.
		var from := (Vector2(strike.origin) + Vector2.ONE * 0.5) * cell_size - Vector2(0, 12)
		var to := (Vector2(strike.destination) + Vector2.ONE * 0.5) * cell_size - Vector2(0, 12)
		effect.points = PackedVector2Array([from, to])
		effect.width = 2
		effect.default_color = Color("e7c680")
		if strike.definition == "actor_def.cthulhu.drowned_cultist":
			effect.default_color = Color("84bdac")
		elif strike.definition == "actor_def.pirates.deckhand":
			effect.default_color = Color("d8d6cb")
		effect.z_index = 100
		map_root.add_child(effect)
		hit_effects.append(effect)

func refresh_snapshot() -> void:
	snapshot = port.island_snapshot()
	refresh_inspection()
	var visible_buildings := {}
	for building in snapshot.buildings:
		# Other archetypes await their own art, never substitute a colonial fort.
		if building.archetype != "site_archetype.colonial.watch_fort":
			continue
		visible_buildings[building.id] = true
		if not building_sprites.has(building.id):
			var fort := Sprite2D.new()
			fort.texture = fort_texture
			fort.centered = false
			# Door apron is the native producer/rally anchor.
			var art: Dictionary = building_art[building.archetype]
			fort.offset = -Vector2(art.pivot[0], art.pivot[1])
			fort.scale = Vector2.ONE * (float(art.display_width) / float(art.source_width))
			fort.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			map_root.add_child(fort)
			building_sprites[building.id] = fort
		var fort: Sprite2D = building_sprites[building.id]
		fort.position = (Vector2(building.x, building.y) + Vector2.ONE * 0.5) * cell_size
		fort.z_index = int(building.y) - 1
		fort.modulate = Color.WHITE if building.operational else Color(0.55, 0.55, 0.55)
	for id in building_sprites.keys():
		if not visible_buildings.has(id):
			building_sprites[id].queue_free()
			building_sprites.erase(id)
	var present := {}
	missing_appearance_count = 0
	for entry in snapshot.actors:
		if entry.id == MICHAEL:
			continue
		var appearance := appearance_for(entry)
		if appearance.is_empty():
			missing_appearance_count += 1
			continue
		present[entry.id] = true
		if not troop_sprites.has(entry.id):
			var troop := Sprite2D.new()
			troop.centered = false
			troop.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			map_root.add_child(troop)
			var health := ProgressBar.new()
			health.name = "Health"
			health.position = Vector2(-155,-400)
			health.size = Vector2(310,26)
			health.show_percentage = false
			health.mouse_filter = Control.MOUSE_FILTER_IGNORE
			troop.add_child(health)
			troop_sprites[entry.id] = troop
		var troop: Sprite2D = troop_sprites[entry.id]
		troop.texture = troop_textures[appearance.texture]
		troop.offset = -Vector2(appearance.pivot[0], appearance.pivot[1])
		troop.scale = Vector2.ONE * float(appearance.scale)
		var health: ProgressBar = troop_sprites[entry.id].get_node("Health")
		health.max_value = maxi(int(entry.max_health),1)
		health.value = int(entry.health)
		health.visible = int(entry.health) < int(entry.max_health)
		troop_sprites[entry.id].position = (Vector2(entry.x,entry.y) + Vector2.ONE * 0.5) * cell_size
		troop_sprites[entry.id].z_index = int(entry.y)
	for id in troop_sprites.keys():
		if not present.has(id):
			troop_sprites[id].queue_free()
			troop_sprites.erase(id)
	var person: Dictionary = {}
	for entry in snapshot.actors:
		if entry.id == MICHAEL:
			person = entry
	if person.is_empty():
		actor.visible = false
		paused = true
		port.pause_island(true)
		pause_button.text = "Fallen"
		pause_button.disabled = true
		status.text = "Michael has fallen. Load a saved campaign or save this outcome.\n" + save_notice
		return
	actor.visible = true
	pause_button.disabled = false
	pause_button.text = "Resume" if paused else "Pause"
	var destination := (Vector2(person.x, person.y) + Vector2.ONE * 0.5) * cell_size
	actor.project_heading(destination - actor.position)
	actor.position = destination
	actor.z_index = int(person.y)
	status.text = "PIRATE ISLAND · Michael HP %s/%s · %s\nClick land to travel. Right-click to fire. Shift-click to inspect.\nDevelopment slice · standing sprites. %s" % [person.health, person.max_health, "PAUSED" if paused else "Exploring", save_notice]
	if missing_appearance_count > 0:
		status.text += "\nMissing character art: %d" % missing_appearance_count

func appearance_for(entry: Dictionary) -> Dictionary:
	var definition: Dictionary = troop_art.get(entry.definition, {})
	var variants: Dictionary = definition.get("variants", {})
	var variant: String = entry.get("sex", "unknown")
	# Older versions only produced these three male appearances. Preserve their
	# presentation without inventing missing personal identity in the native save.
	if variant == "unknown":
		variant = definition.get("legacy_unknown_variant", "")
	return variants.get(variant, {})

func nearest_actor(point: Vector2, include_michael: bool = true) -> String:
	var closest := ""
	var distance := 24.0
	for entry in snapshot.actors:
		if entry.id == MICHAEL and not include_michael:
			continue
		var chest := (Vector2(entry.x, entry.y) + Vector2.ONE * 0.5) * cell_size - Vector2(0, 12)
		var gap := chest.distance_to(point)
		if gap < distance:
			closest = entry.id
			distance = gap
	return closest

func inspect_actor(id: String) -> void:
	inspected_id = id
	inspected_person = {}
	conversation_notice = ""
	refresh_inspection()

func talk_action() -> void:
	var offer: String = port.talk_island_person(inspected_id)
	conversation_notice = "" if not offer.is_empty() else "Move Michael closer, with a clear path between you, to talk."
	refresh_snapshot()

func recruit_action() -> void:
	conversation_notice = "Joined your faction. Choose a companion slot below." if port.recruit_island_person(inspected_id) else "Could not join. Talk first and keep Michael close enough to speak."
	refresh_snapshot()

func assign_action(slot: int) -> void:
	conversation_notice = "Companion slot updated." if port.assign_island_companion(inspected_id, slot) else "Choose a living woman in Michael’s faction who is not already in another slot."
	refresh_snapshot()

func dismiss_action(slot: int) -> void:
	conversation_notice = "Party slot cleared; faction membership is unchanged." if port.dismiss_island_companion(slot) else "That slot could not be cleared."
	refresh_snapshot()

func refresh_party_controls(live: bool) -> void:
	var slots: Array = snapshot.get("party", ["", "", "", ""])
	var michael_alive: bool = snapshot.actors.any(func(entry): return entry.id == MICHAEL)
	var eligible: bool = michael_alive and live and inspected_person.get("sex", "") == "female" and inspected_person.get("faction", "") == "faction.michael" and inspected_person.get("loyal_to_michael", false)
	var living_count := 0
	for entry in snapshot.actors:
		if slots.has(entry.id):
			living_count += 1
	party_controls.visible = eligible or slots.any(func(id): return not str(id).is_empty())
	party_heading.text = "Companions · %d/4 living" % living_count
	if eligible:
		party_heading.text += " · Choose a slot for %s" % inspected_person.get("name", "")
	for slot in range(4):
		var id: String = slots[slot] if slot < slots.size() else ""
		var name_text := "Empty"
		if not id.is_empty():
			var remembered_names: Array = snapshot.get("party_names", [])
			name_text = str(remembered_names[slot]) + " · Fallen or absent" if slot < remembered_names.size() and not str(remembered_names[slot]).is_empty() else "Fallen or absent"
			for entry in snapshot.actors:
				if entry.id == id:
					name_text = entry.name
		var button := party_buttons[slot]
		button.text = "%d · %s" % [slot + 1, name_text]
		button.disabled = not eligible or slots.has(inspected_id)
		button.tooltip_text = "Assign %s to slot %d%s" % [inspected_person.get("name", "selected woman"), slot + 1, " (replaces %s in the party only)" % name_text if not id.is_empty() else ""]
		dismiss_buttons[slot].disabled = id.is_empty()
		dismiss_buttons[slot].tooltip_text = "Clear slot %d; keep faction membership" % (slot + 1)

func refresh_inspection() -> void:
	inspection.visible = not inspected_id.is_empty()
	var live := false
	for entry in snapshot.actors:
		if entry.id == inspected_id:
			inspected_person = entry.duplicate(true)
			live = true
			break
	refresh_party_controls(live)
	var offer: String = inspected_person.get("recruitment_offer", "")
	var michael_alive: bool = snapshot.actors.any(func(entry): return entry.id == MICHAEL)
	var potential: bool = michael_alive and live and inspected_person.get("sex", "") == "female" and inspected_person.get("faction", "") != "faction.michael" and not offer.is_empty()
	talk_button.visible = potential
	recruit_button.visible = potential and inspected_person.get("discussed", false)
	conversation_actions.visible = potential
	conversation_status.text = conversation_notice
	conversation_status.visible = not conversation_notice.is_empty()
	if not inspection.visible:
		return
	var name_text: String = inspected_person.get("name", "Unknown person")
	var faction_text: String = inspected_person.get("faction", "Unknown faction")
	match faction_text:
		"faction.michael": faction_text = "Michael’s faction"
		"faction.colonial_powers.prototype": faction_text = "Colonial powers"
		"faction.pirates.prototype": faction_text = "Pirates"
		"faction.cthulhu.prototype": faction_text = "Cthulhu"
	# A missing live actor can mean death or departure; do not invent which.
	var state_text := faction_text if live else "Dead or departed · Last seen: " + faction_text
	inspection.text = "%s · %s\n%s\nShift-click empty land to close." % [name_text, state_text, inspected_person.get("biography", "")]
	if live and inspected_person.get("discussed", false) and not offer.is_empty():
		inspection.text += "\n“%s”" % offer

func _process(delta: float) -> void:
	if not ready_ok or paused:
		return
	elapsed += minf(delta, 0.2)
	if elapsed >= 0.15:
		elapsed -= 0.15
		advance_tick()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.physical_keycode == KEY_F5:
			save_action()
		if event.physical_keycode == KEY_F9:
			load_action()
	if event is InputEventKey and event.pressed and not event.echo and event.physical_keycode == KEY_SPACE:
		set_paused(not paused)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var point: Vector2 = map_root.get_global_transform_with_canvas().affine_inverse() * event.position
		if event.shift_pressed:
			inspect_actor(nearest_actor(point))
		else:
			request_move(Vector2i(floori(point.x / cell_size), floori(point.y / cell_size)))
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_RIGHT:
		var point: Vector2 = map_root.get_global_transform_with_canvas().affine_inverse() * event.position
		var closest := nearest_actor(point, false)
		if not closest.is_empty():
			save_notice = "Carbine aimed" if port.aim_island_carbine(closest) else "No clear shot in range"
			refresh_snapshot()
