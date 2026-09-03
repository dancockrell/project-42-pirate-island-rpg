extends SceneTree

const CELL_SCENE := preload("res://scenes/world/black_beach/reception_terrace.tscn")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "content catalog must load before a world cell can configure")
	var cell: WorldCell = CELL_SCENE.instantiate()
	root.add_child(cell)
	await process_frame
	check(cell.configure(catalog) == OK, "Reception Terrace must match its world-cell content record")
	check(cell.entry_anchor("entry.reception_terrace.shipwreck_trail") != null, "Shipwreck Trail entry anchor must exist")
	check(cell.entry_anchor("entry.reception_terrace.estate_gate") != null, "Estate Gate entry anchor must exist")
	check(cell.battle_entry("battle_entry.reception_terrace.razorbeak") != null, "Razorbeak battle entry must exist")
	check(cell.terrain_and_collision.get_child_count() > 0, "world-cell collision must be separate authored geometry")
	check(cell.navigation.get_child_count() > 0, "world-cell navigation must be a separate scene")
	cell.queue_free()
	quit(0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	push_error(message)
	quit(1)
