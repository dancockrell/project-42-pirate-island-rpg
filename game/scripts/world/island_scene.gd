extends Node2D

const Port = preload("res://scripts/simulation/native_simulation_port.gd")
const Actor = preload("res://scripts/world/directional_sprite.gd")
const MICHAEL := "character.protagonist.captain"
var port = Port.new()
var actor = Actor.new()
var map_root := Node2D.new()
var status := Label.new()
var snapshot: Dictionary
var cell_size := 32
var elapsed := 0.0
var paused := false
var ready_ok := false
var save_notice := ""
var troop_sprites := {}
var troop_textures: Array[Texture2D] = []
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
	for appearance in ["colonial", "pirate", "cultist"]:
		var cutout := Image.load_from_file("res://assets/island/troops/%s.png" % appearance)
		assert(cutout != null and cutout.detect_alpha() == Image.ALPHA_BIT)
		troop_textures.append(ImageTexture.create_from_image(cutout))
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
	status.position = Vector2(20,18)
	status.add_theme_color_override("font_shadow_color", Color.BLACK)
	status.add_theme_constant_override("shadow_offset_x", 2)
	status.add_theme_constant_override("shadow_offset_y", 2)
	hud.add_child(status)
	get_viewport().size_changed.connect(_fit)
	_fit()
	refresh_snapshot()
	ready_ok = true

func _fit() -> void:
	var area := get_viewport_rect().size
	var zoom := minf(area.x / 1536.0, area.y / 1024.0)
	map_root.scale = Vector2.ONE * zoom
	map_root.position = ((area - Vector2(1536,1024) * zoom) * 0.5).round()

func request_move(cell: Vector2i) -> bool:
	return port.move_island_actor(MICHAEL, cell)

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
	for entry in snapshot.actors:
		if entry.id == MICHAEL:
			continue
		present[entry.id] = true
		if not troop_sprites.has(entry.id):
			var troop := Sprite2D.new()
			var column := 0
			if entry.faction == "faction.pirates.prototype":
				column = 1
			elif entry.faction == "faction.cthulhu.prototype":
				column = 2
			troop.texture = troop_textures[column]
			troop.centered = false
			# Original authored ground pivots minus source trim origins.
			troop.offset = -[Vector2(116,366), Vector2(143,363), Vector2(127,374)][column]
			troop.scale = Vector2.ONE * 0.105
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
	assert(not person.is_empty())
	var destination := (Vector2(person.x, person.y) + Vector2.ONE * 0.5) * cell_size
	actor.project_heading(destination - actor.position)
	actor.position = destination
	actor.z_index = int(person.y)
	status.text = "PIRATE ISLAND   |   Michael   |   Click land to travel   |   Space: pause   |   F5: save   |   F9: load\n%s  •  Development scene / standing sprite; animation pending  %s" % ["PAUSED" if paused else "Exploring", save_notice]

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
			save_notice = "Saved" if save_campaign() else "Save failed — previous save retained"
			refresh_snapshot()
		if event.physical_keycode == KEY_F9:
			save_notice = "Loaded" if load_campaign() else "Could not load — current game unchanged"
			refresh_snapshot()
	if event is InputEventKey and event.pressed and not event.echo and event.physical_keycode == KEY_SPACE:
		set_paused(not paused)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var point := map_root.to_local(get_global_mouse_position())
		request_move(Vector2i(floori(point.x / cell_size), floori(point.y / cell_size)))
