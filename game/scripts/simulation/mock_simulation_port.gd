class_name MockSimulationPort
extends SimulationPort

## Development-only projection fixture. It mirrors the Rust event vocabulary so
## the battle UI can be built before the GDExtension adapter is attached.

var sequence := 0
var round_number := 1
var started := false
var betty_vitality := 100
var betty_guard := 0
var razorbeak_vitality := 70
var razorbeak_guard := 3

func create_debug_battle() -> Dictionary:
	return {
		"battle_id": "battle.prototype.returning_names",
		"round": round_number,
		"phase": "awaiting_actor" if not started else "awaiting_command",
		"active_actor_id": "character.heroine.betty",
		"description": "The razorbeak keeps its wounded flank away from Betty. Its feet are coiled for a two-band rush.",
		"actors": [
			actor_snapshot("character.heroine.betty", "Betty", "party", betty_vitality, 100, betty_guard, 0),
			actor_snapshot("enemy.raptor.razorbeak.prototype", "Razorbeak", "hostile", razorbeak_vitality, 70, razorbeak_guard, 2)
		],
		"metadata": {"source": "mock", "release_legal": false, "authoritative_equivalent": "godot-rust/src/battle.rs"}
	}

func start() -> Array[Dictionary]:
	if started:
		return []
	started = true
	return [make_event("battle_started", [], {"round": round_number}), make_event("turn_started", ["character.heroine.betty"], {"round": round_number})]

func submit(command: Dictionary) -> Array[Dictionary]:
	if not started:
		return [make_event("command_rejected", [], {"reason": "battle_not_started"})]
	var command_id: String = command.get("command_id", "command.missing")
	var actor_id: String = command.get("actor_id", "actor.missing")
	var skill_id: String = command.get("skill_id", "skill.missing")
	if actor_id != "character.heroine.betty" or skill_id != "skill.betty.guarded_strike":
		return [make_event("command_rejected", [actor_id], {"reason": "prototype_supports_betty_guarded_strike_only", "command_id": command_id})]

	var player_damage := maxi(0, 15 - razorbeak_guard)
	razorbeak_guard = maxi(0, razorbeak_guard - 15)
	razorbeak_vitality = maxi(0, razorbeak_vitality - player_damage)
	betty_guard += 2
	var events: Array[Dictionary] = [
		make_event("command_accepted", [actor_id], {"command_id": command_id, "skill_id": skill_id}),
		make_event("actor_focused", [actor_id], {"card_state": "expanding", "active_state": "active"}),
		make_event("damage_applied", [actor_id, "enemy.raptor.razorbeak.prototype"], {"amount": player_damage, "remaining_vitality": razorbeak_vitality}),
		make_event("guard_changed", [actor_id], {"delta": 2, "total": betty_guard}),
		make_event("turn_ended", [actor_id], {"round": round_number})
	]
	if razorbeak_vitality <= 0:
		events.append(make_event("actor_defeated", ["enemy.raptor.razorbeak.prototype"], {}))
		events.append(make_event("battle_ended", [], {"victory": true}))
		return events

	events.append(make_event("turn_started", ["enemy.raptor.razorbeak.prototype"], {"round": round_number}))
	events.append(make_event("enemy_intent_declared", ["enemy.raptor.razorbeak.prototype", "character.heroine.betty"], {
		"skill_id": "skill.enemy.razorbeak.rushing_bite", "intent_name": "Rushing Bite", "target_id": "character.heroine.betty"
	}))
	var enemy_damage := maxi(0, 16 - betty_guard)
	betty_guard = maxi(0, betty_guard - 16)
	betty_vitality = maxi(0, betty_vitality - enemy_damage)
	events.append(make_event("actor_focused", ["enemy.raptor.razorbeak.prototype"], {"active_state": "active"}))
	events.append(make_event("damage_applied", ["enemy.raptor.razorbeak.prototype", "character.heroine.betty"], {"amount": enemy_damage, "remaining_vitality": betty_vitality}))
	events.append(make_event("turn_ended", ["enemy.raptor.razorbeak.prototype"], {"round": round_number}))
	if betty_vitality <= 0:
		events.append(make_event("actor_defeated", ["character.heroine.betty"], {}))
		events.append(make_event("battle_ended", [], {"victory": false, "advance_to_midnight": true}))
	else:
		round_number += 1
		events.append(make_event("round_started", [], {"round": round_number}))
		events.append(make_event("turn_started", ["character.heroine.betty"], {"round": round_number}))
	return events

func actor_snapshot(id: String, display_name: String, faction: String, vitality: int, maximum: int, guard: int, band: int) -> Dictionary:
	return {"id": id, "display_name": display_name, "faction": faction, "vitality": vitality, "max_vitality": maximum, "guard": guard, "band": band}

func make_event(kind: String, subjects: Array, payload: Dictionary) -> Dictionary:
	sequence += 1
	return {"event_id": "event.mock.%04d" % sequence, "sequence": sequence, "kind": kind, "subjects": subjects, "payload": payload}
