class_name SimulationPort
extends RefCounted

## Interface implemented by mock and native Rust adapters.

func create_debug_battle() -> Dictionary:
	push_error("SimulationPort.create_debug_battle is abstract")
	return {}

func start() -> Array[Dictionary]:
	push_error("SimulationPort.start is abstract")
	return []

func submit(_command: Dictionary) -> Array[Dictionary]:
	push_error("SimulationPort.submit is abstract")
	return []

func recommended_enemy_command(_command_id: String) -> Dictionary:
	return {"available": false, "reason": "not_implemented"}
