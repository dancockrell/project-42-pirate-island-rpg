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
	port.tick_island()
	refresh_snapshot()

func refresh_snapshot() -> void:
	snapshot = port.island_snapshot()
	var person: Dictionary = snapshot.actors[0]
	var destination := (Vector2(person.x, person.y) + Vector2.ONE * 0.5) * cell_size
	actor.project_heading(destination - actor.position)
	actor.position = destination
	status.text = "PIRATE ISLAND   |   Michael   |   Click land to travel   |   Space: pause\n%s  •  Development scene / standing sprite; animation pending" % ("PAUSED" if paused else "Exploring")

func _process(delta: float) -> void:
	if not ready_ok or paused:
		return
	elapsed += minf(delta, 0.2)
	if elapsed >= 0.15:
		elapsed -= 0.15
		advance_tick()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.physical_keycode == KEY_SPACE:
		set_paused(not paused)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var point := map_root.to_local(get_global_mouse_position())
		request_move(Vector2i(floori(point.x / cell_size), floori(point.y / cell_size)))
