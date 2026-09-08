class_name NativeSimulationPort
extends SimulationPort

## The sole Godot-facing owner of the future Rust GDExtension instance.
## Battle scenes depend on SimulationPort and never call the extension directly.

const BRIDGE_CLASS := "Project42SimulationBridge"
var bridge: Object

func _init() -> void:
	if not ClassDB.can_instantiate(BRIDGE_CLASS):
		push_error("%s is unavailable. Build the Rust GDExtension before selecting NativeSimulationPort." % BRIDGE_CLASS)
		return
	bridge = ClassDB.instantiate(BRIDGE_CLASS)

func is_available() -> bool:
	return bridge != null

func create_island() -> Dictionary:
	return bridge.create_island() if is_available() else error_snapshot("native_bridge_unavailable")

func install_preview_factions() -> bool:
	return is_available() and bridge.install_preview_factions()

func save_island() -> String:
	return bridge.save_island() if is_available() else ""

func load_island(payload: String) -> bool:
	return is_available() and bridge.load_island(payload)

func valid_island_save(payload: String) -> bool:
	return is_available() and bridge.valid_island_save(payload)

func configure_island_land(cells: Array[Vector2i], start: Vector2i) -> bool:
	return is_available() and bridge.configure_island_land(cells, start)

func island_snapshot() -> Dictionary:
	return bridge.island_snapshot() if is_available() else error_snapshot("native_bridge_unavailable")

func move_island_actor(actor_id: String, target: Vector2i) -> bool:
	return is_available() and bridge.move_island_actor(actor_id, target)

func move_island_party(target: Vector2i) -> bool:
	return is_available() and bridge.move_island_party(target)

func party_move_failure(target: Vector2i) -> String:
	return bridge.party_move_failure(target) if is_available() else "The island is unavailable."

func salvage_island_foothold() -> String:
	return bridge.salvage_island_foothold() if is_available() else "The island is unavailable."

func build_island_foothold(target: Vector2i) -> String:
	return bridge.build_island_foothold(target) if is_available() else "The island is unavailable."

func restore_island_foothold_person(actor_id: String) -> String:
	return bridge.restore_island_foothold_person(actor_id) if is_available() else "The island is unavailable."

func repair_island_foothold() -> String:
	return bridge.repair_island_foothold() if is_available() else "The island is unavailable."

func talk_island_person(actor_id: String) -> String:
	return bridge.talk_island_person(actor_id) if is_available() else ""

func approach_island_person(actor_id: String) -> bool:
	return is_available() and bridge.approach_island_person(actor_id)

func can_talk_island_person(actor_id: String) -> bool:
	return is_available() and bridge.can_talk_island_person(actor_id)

func recruit_island_person(actor_id: String) -> bool:
	return is_available() and bridge.recruit_island_person(actor_id)

func assign_island_companion(actor_id: String, slot: int) -> bool:
	return is_available() and bridge.assign_island_companion(actor_id, slot)

func dismiss_island_companion(slot: int) -> bool:
	return is_available() and bridge.dismiss_island_companion(slot)

func tick_island() -> Dictionary:
	return bridge.tick_island() if is_available() else error_snapshot("native_bridge_unavailable")

func aim_island_carbine(target: String) -> bool:
	return is_available() and bridge.aim_island_carbine(target)

func pause_island(paused: bool) -> void:
	if is_available():
		bridge.pause_island(paused)

func create_debug_battle() -> Dictionary:
	if not is_available():
		return error_snapshot("native_bridge_unavailable")
	return bridge.create_debug_battle()

func start() -> Array[Dictionary]:
	if not is_available():
		return [error_event("native_bridge_unavailable")]
	return typed_event_array(bridge.start_battle())

func submit(command: Dictionary) -> Array[Dictionary]:
	if not is_available():
		return [error_event("native_bridge_unavailable")]
	var normalized := {
		"protocol_version": int(command.get("protocol_version", 1)),
		"command_id": str(command.get("command_id", "")),
		"battle_id": str(command.get("battle_id", "")),
		"actor_id": str(command.get("actor_id", "")),
		"kind": str(command.get("kind", "")),
		"skill_id": str(command.get("skill_id", "")),
		"target_ids": Array(command.get("target_ids", []), TYPE_STRING, "", null)
	}
	return typed_event_array(bridge.submit_command(normalized))

func recommended_enemy_command(command_id: String) -> Dictionary:
	if not is_available():
		return {"available": false, "reason": "native_bridge_unavailable"}
	return bridge.recommended_enemy_command(command_id)

func typed_event_array(value: Variant) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if value is Array:
		for event in value:
			if event is Dictionary:
				result.append(event)
	return result

func error_snapshot(reason: String) -> Dictionary:
	return {"battle_id": "", "phase": "error", "actors": [], "error": {"kind": reason}, "metadata": {"source": "native_adapter", "release_legal": false}}

func error_event(reason: String) -> Dictionary:
	return {"event_id": "event.native.adapter_error", "sequence": 0, "kind": "command_rejected", "subjects": [], "payload": {"reason": reason}}
