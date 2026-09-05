extends SceneTree

## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0

const CELL_SCENE := preload("res://scenes/world/black_beach/reception_terrace.tscn")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var cell: WorldCell = CELL_SCENE.instantiate()
	root.add_child(cell)
	await process_frame
	var setpiece := cell.visual_shell.get_node_or_null("ReceptionTerraceSetpiece") as ReceptionTerraceSetpiece
	check(setpiece != null, "Reception Terrace must contain its standard-Godot visual setpiece")
	check(setpiece.get_meta("production_state", "") == "procedural-standard-godot-setpiece", "setpiece must declare that it is a replaceable Godot primitive kit")
	for required_section in ["Lighting", "ProcessionalTerrace", "ElvenGate", "JungleMass", "ShipwreckFlotsam", "SeaAndSky"]:
		check(setpiece.get_node_or_null(required_section) != null, "setpiece is missing required visual section: %s" % required_section)
		check(setpiece.get_node(required_section).get_child_count() > 0, "setpiece section must visibly block its intended mass: %s" % required_section)
	check(setpiece.get_node_or_null("Character") == null, "environment setpiece must not smuggle in a character proxy")
	check(cell.terrain_and_collision.get_child_count() > 0, "visual setpiece must not replace separate collision geometry")
	check(cell.navigation.get_child_count() > 0, "visual setpiece must not replace separate navigation geometry")
	cell.queue_free()
	quit(1 if failures > 0 else 0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
