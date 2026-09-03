extends SceneTree

## Exercises the actual GDScript screen against the native Rust expedition
## bridge. The test proves that the route buttons are a projection of legal
## native commands and that selected travel updates the rendered location.

var failures := 0


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Expedition prototype native test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	var scene := load("res://scenes/world/expedition_prototype.tscn") as PackedScene
	check(scene != null, "expedition prototype scene must load")
	if scene == null:
		finish()
		return
	var expedition := scene.instantiate() as ExpeditionPrototype
	root.add_child(expedition)
	await process_frame
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.black_beach", "fresh expedition must begin at Black Beach")
	check(expedition.get_legal_route_count() == 1, "Black Beach must expose exactly one legal route")
	expedition.request_travel("world.portal.black_beach_to_damaged_estate")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.damaged_estate", "travel must use the native portal result")
	expedition.request_travel("world.portal.damaged_estate_to_river_landing")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.river_landing", "estate departure must arrive at River Landing")
	check(expedition.get_legal_route_count() == 3, "River Landing must project its three legal native routes")
	expedition.request_travel("world.portal.river_landing_to_reception_terrace_safe_road")
	check(expedition.get_authoritative_snapshot().get("active_location_id") == "world.cell.reception_terrace", "safe road must arrive at Reception Terrace")
	var pending_encounter: Dictionary = expedition.get_authoritative_snapshot().get("pending_encounter", {})
	check(pending_encounter.get("encounter_id") == "encounter.prototype.returning_names", "Reception Terrace must arm its authored Razorbeak encounter")
	check(pending_encounter.get("battle_id") == "battle.prototype.returning_names", "Encounter handoff must retain the authored battle ID")
	check(expedition.get_legal_route_count() == 1, "Pending encounter must replace route controls with one truthful blocked-state message")
	expedition.queue_free()
	finish()


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Expedition prototype tests passed.")
	quit(0)
