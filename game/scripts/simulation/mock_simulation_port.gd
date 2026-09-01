class_name MockSimulationPort
extends SimulationPort

## Development-only adapter. It exists to exercise presentation before the
## Rust GDExtension bridge is attached. It must not ship in a release export.

var sequence := 0

func create_debug_battle() -> Dictionary:
	return {
		"battle_id": "battle.prototype.returning_names",
		"description": "The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush.",
		"metadata": {"source": "mock", "release_legal": false}
	}

func submit(command: Dictionary) -> Array[Dictionary]:
	sequence += 1
	var command_id: String = command.get("command_id", "command.missing")
	var actor_id: String = command.get("actor_id", "actor.missing")
	var label: String = command.get("payload", {}).get("display_command", "UNKNOWN")
	return [
		{
			"event_id": "event.mock.%d.accepted" % sequence,
			"command_id": command_id,
			"sequence": sequence,
			"kind": "command_accepted",
			"subjects": [actor_id],
			"payload": {"actor_name": actor_id.get_slice(".", 2).capitalize(), "command_label": label}
		},
		{
			"event_id": "event.mock.%d.damage" % sequence,
			"command_id": command_id,
			"sequence": sequence + 1,
			"kind": "damage_applied",
			"subjects": ["enemy.raptor.razorbeak.prototype"],
			"payload": {"amount": 17, "placeholder_result": true}
		}
	]

