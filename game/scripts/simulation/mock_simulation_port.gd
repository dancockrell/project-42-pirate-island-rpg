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
var captain_vitality := 96
var ayla_vitality := 0
var vix_vitality := 72
var grisha_vitality := 88
var infirmary_pulses_remaining := 0
var combat_revival_uses := 1

func recommended_enemy_command(command_id: String) -> Dictionary:
	return {
		"available": true,
		"protocol_version": 1,
		"command_id": command_id,
		"battle_id": "battle.prototype.returning_names",
		"actor_id": "enemy.raptor.razorbeak",
		"kind": "use_skill",
		"skill_id": "skill.enemy.razorbeak.rushing_bite",
		"target_ids": ["character.heroine.betty"],
		"rationale": "pressure_active_actor",
		"guard_break_amount": 0,
		"raw_damage": 16,
		"guard_absorbed": 0,
		"vitality_damage": 16,
		"lethal": false,
		"interception_protector_id": "",
		"fatal_intercept_available": false
	}

func create_debug_battle() -> Dictionary:
	return {
		"battle_id": "battle.prototype.returning_names",
		"round": round_number,
		"phase": "awaiting_actor" if not started else "awaiting_command",
		"active_actor_id": "character.heroine.betty",
		"description": "Elven Gate • Late Afternoon",
		"actors": [
			actor_snapshot("character.protagonist.captain", "Michael Corrigan", "party", captain_vitality, 96, 1, 0),
			# A10: the mock mirrors the native prototype fixture, where Betty
			# stands at the top of the ladder so the slice can demonstrate her
			# whole authored kit up to the rank SSS Combat Revival. A fresh
			# campaign's Betty stands at "D", which is this helper's default.
			actor_snapshot("character.heroine.betty", "Betty", "party", betty_vitality, 100, betty_guard, 0, 10, "SSS"),
			actor_snapshot("character.heroine.ayla", "Ayla", "party", ayla_vitality, 90, 0, 0),
			actor_snapshot("character.heroine.vix", "Vix", "party", vix_vitality, 90, 0, 1),
			actor_snapshot("character.heroine.grisha", "Grisha", "party", grisha_vitality, 110, 4, 0),
			actor_snapshot("enemy.raptor.razorbeak", "Razorbeak", "hostile", razorbeak_vitality, 70, razorbeak_guard, 3)
		],
		"effects": [],
		"recovery_openings": [],
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
	if actor_id != "character.heroine.betty" or skill_id not in ["skill.betty.guarded_strike", "skill.betty.condition_cleanse", "skill.betty.rescue_charge", "skill.betty.healing_impact", "skill.betty.mobile_infirmary", "skill.betty.combat_revival"]:
		return [make_event("command_rejected", [actor_id], {"reason": "prototype_skill_not_implemented", "command_id": command_id})]
	var opening: Array[Dictionary] = [
		make_event("command_accepted", [actor_id], {"command_id": command_id, "skill_id": skill_id}),
		make_event("actor_focused", [actor_id], {"card_state": "expanding", "active_state": "active"})
	]
	if skill_id == "skill.betty.rescue_charge":
		opening.append(make_event("actor_moved", [actor_id], {"from_band": 0, "to_band": 1}))
		var rescue_damage := mini(razorbeak_vitality, maxi(0, 10 - razorbeak_guard))
		razorbeak_guard = maxi(0, razorbeak_guard - 10)
		razorbeak_vitality -= rescue_damage
		opening.append(make_event("damage_applied", [actor_id, "enemy.raptor.razorbeak"], {"amount": rescue_damage, "remaining_vitality": razorbeak_vitality}))
		opening.append(make_event("interception_set", [actor_id, "character.heroine.vix"], {"protector_id": actor_id, "protected_id": "character.heroine.vix"}))
		opening.append(make_event("turn_ended", [actor_id], {"round": round_number}))
		opening.append_array(resolve_enemy_turn())
		return opening
	if skill_id == "skill.betty.mobile_infirmary":
		infirmary_pulses_remaining = 2
		opening.append(make_event("battlefield_effect_created", [actor_id], {"effect_id": "effect.mock.mobile_infirmary", "source_skill_id": skill_id, "total_pulses": 3}))
		for party_id in ["character.heroine.betty", "character.heroine.vix", "character.heroine.grisha"]:
			opening.append(make_event("vitality_changed", [party_id], {"delta": 10, "total": heal_mock_actor(party_id, 10)}))
			opening.append(make_event("guard_changed", [party_id], {"delta": 2, "total": 2}))
		opening.append(make_event("battlefield_effect_pulse", [actor_id], {"effect_id": "effect.mock.mobile_infirmary", "pulses_remaining_after": infirmary_pulses_remaining}))
		opening.append(make_event("turn_ended", [actor_id], {"round": round_number}))
		opening.append_array(resolve_enemy_turn())
		return opening
	if skill_id == "skill.betty.combat_revival":
		if ayla_vitality > 0 or combat_revival_uses == 0:
			return [make_event("command_rejected", [actor_id], {"reason": "no_legal_defeated_target_or_use_spent", "command_id": command_id})]
		combat_revival_uses -= 1
		ayla_vitality = 36
		opening.append(make_event("actor_revived", ["character.heroine.ayla"], {"vitality": ayla_vitality}))
		opening.append(make_event("bonus_turn_granted", ["character.heroine.ayla"], {"resume_after": actor_id}))
		opening.append(make_event("turn_ended", [actor_id], {"round": round_number}))
		opening.append(make_event("turn_started", ["character.heroine.ayla"], {"round": round_number, "bonus_turn": true}))
		return opening
	if skill_id == "skill.betty.condition_cleanse":
		var healed := mini(8, 100 - betty_vitality)
		betty_vitality += healed
		var cleanse_events: Array[Dictionary] = [
			make_event("command_accepted", [actor_id], {"command_id": command_id, "skill_id": skill_id}),
			make_event("actor_focused", [actor_id], {"card_state": "expanding", "active_state": "active"}),
			make_event("vitality_changed", [actor_id], {"delta": healed, "total": betty_vitality}),
			make_event("turn_ended", [actor_id], {"round": round_number})
		]
		cleanse_events.append_array(resolve_enemy_turn())
		return cleanse_events
	if skill_id == "skill.betty.healing_impact":
		var effective := maxi(0, 21 - razorbeak_guard)
		razorbeak_guard = maxi(0, razorbeak_guard - 21)
		var impact_damage := mini(razorbeak_vitality, effective)
		razorbeak_vitality -= impact_damage
		var healed := mini(int(impact_damage / 2), 100 - betty_vitality)
		betty_vitality += healed
		var impact_events: Array[Dictionary] = [
			make_event("command_accepted", [actor_id], {"command_id": command_id, "skill_id": skill_id}),
			make_event("actor_focused", [actor_id], {"card_state": "expanding", "active_state": "active"}),
			make_event("damage_applied", [actor_id, "enemy.raptor.razorbeak"], {"amount": impact_damage, "remaining_vitality": razorbeak_vitality}),
			make_event("vitality_changed", [actor_id], {"delta": healed, "total": betty_vitality}),
			make_event("turn_ended", [actor_id], {"round": round_number})
		]
		if razorbeak_vitality <= 0:
			impact_events.insert(4, make_event("actor_defeated", ["enemy.raptor.razorbeak"], {}))
			impact_events.append(make_event("battle_ended", [], {"victory": true}))
		else:
			impact_events.append_array(resolve_enemy_turn())
		return impact_events

	var player_damage := maxi(0, 15 - razorbeak_guard)
	razorbeak_guard = maxi(0, razorbeak_guard - 15)
	razorbeak_vitality = maxi(0, razorbeak_vitality - player_damage)
	betty_guard += 2
	var events: Array[Dictionary] = [
		make_event("command_accepted", [actor_id], {"command_id": command_id, "skill_id": skill_id}),
		make_event("actor_focused", [actor_id], {"card_state": "expanding", "active_state": "active"}),
		make_event("damage_applied", [actor_id, "enemy.raptor.razorbeak"], {"amount": player_damage, "remaining_vitality": razorbeak_vitality}),
		make_event("guard_changed", [actor_id], {"delta": 2, "total": betty_guard}),
		make_event("turn_ended", [actor_id], {"round": round_number})
	]
	if razorbeak_vitality <= 0:
		events.append(make_event("actor_defeated", ["enemy.raptor.razorbeak"], {}))
		events.append(make_event("battle_ended", [], {"victory": true}))
		return events

	events.append_array(resolve_enemy_turn())
	return events

func resolve_enemy_turn() -> Array[Dictionary]:
	var events: Array[Dictionary] = [make_event("turn_started", ["enemy.raptor.razorbeak"], {"round": round_number})]
	events.append(make_event("enemy_intent_declared", ["enemy.raptor.razorbeak", "character.heroine.betty"], {
		"skill_id": "skill.enemy.razorbeak.rushing_bite", "intent_name": "Rushing Bite", "target_id": "character.heroine.betty",
		"rationale": "pressure_active_actor", "guard_break_amount": 0, "raw_damage": 16, "guard_absorbed": mini(16, betty_guard),
		"vitality_damage": maxi(0, 16 - betty_guard), "lethal": maxi(0, 16 - betty_guard) >= betty_vitality,
		"interception_protector_id": "", "fatal_intercept_available": false
	}))
	var enemy_damage := maxi(0, 16 - betty_guard)
	betty_guard = maxi(0, betty_guard - 16)
	betty_vitality = maxi(0, betty_vitality - enemy_damage)
	events.append(make_event("actor_focused", ["enemy.raptor.razorbeak"], {"active_state": "active"}))
	events.append(make_event("damage_applied", ["enemy.raptor.razorbeak", "character.heroine.betty"], {"amount": enemy_damage, "remaining_vitality": betty_vitality}))
	events.append(make_event("turn_ended", ["enemy.raptor.razorbeak"], {"round": round_number}))
	if betty_vitality <= 0:
		events.append(make_event("actor_defeated", ["character.heroine.betty"], {}))
		events.append(make_event("battle_ended", [], {"victory": false, "advance_to_midnight": true}))
	else:
		round_number += 1
		events.append(make_event("round_started", [], {"round": round_number}))
		if infirmary_pulses_remaining > 0:
			infirmary_pulses_remaining -= 1
			for party_id in ["character.heroine.betty", "character.heroine.vix", "character.heroine.grisha"]:
				events.append(make_event("vitality_changed", [party_id], {"delta": 10, "total": heal_mock_actor(party_id, 10)}))
			events.append(make_event("battlefield_effect_pulse", ["character.heroine.betty"], {"effect_id": "effect.mock.mobile_infirmary", "pulses_remaining_after": infirmary_pulses_remaining}))
			if infirmary_pulses_remaining == 0:
				events.append(make_event("battlefield_effect_removed", [], {"effect_id": "effect.mock.mobile_infirmary", "reason": "completed"}))
		events.append(make_event("turn_started", ["character.heroine.betty"], {"round": round_number}))
	return events

func heal_mock_actor(actor_id: String, amount: int) -> int:
	match actor_id:
		"character.heroine.betty":
			betty_vitality = mini(100, betty_vitality + amount)
			return betty_vitality
		"character.heroine.vix":
			vix_vitality = mini(90, vix_vitality + amount)
			return vix_vitality
		"character.heroine.grisha":
			grisha_vitality = mini(110, grisha_vitality + amount)
			return grisha_vitality
	return 0

# The five bands, named exactly as Band::name() in godot-rust/src/battle.rs.
# The mock and the native bridge must agree on the actor dictionary, or a
# screen built against one breaks silently on the other.
const BAND_NAMES := ["party_rear", "party_front", "contested", "enemy_front", "enemy_rear"]

func band_name(band: int) -> String:
	if band < 0 or band >= BAND_NAMES.size():
		return "unknown"
	return BAND_NAMES[band]

## B5: `statuses` completes the actor dictionary. The bridge sends it on every
## actor and the battle screen reads it for the Shaken mark; the mock carried
## every other key and not this one, which is the same shape disagreement A5
## found with `band_name` and `composure`. The mock states the shape only -- it
## gains no behaviour the bridge lacks, and nothing here applies a status.
# A10: bond_rank is one of the seven authored letters, exactly as
# `actor_dictionary` in godot-rust/src/godot_bridge.rs hands it over. "D" is the
# floor every woman starts at, so the mock agrees with a fresh campaign rather
# than with the prototype battle fixture.
func actor_snapshot(id: String, display_name: String, faction: String, vitality: int, maximum: int, guard: int, band: int, composure: int = 10, bond_rank: String = "D") -> Dictionary:
	return {"id": id, "display_name": display_name, "faction": faction, "vitality": vitality, "max_vitality": maximum, "guard": guard, "band": band, "band_name": band_name(band), "composure": composure, "bond_rank": bond_rank, "statuses": []}

func make_event(kind: String, subjects: Array, payload: Dictionary) -> Dictionary:
	sequence += 1
	return {"event_id": "event.mock.%04d" % sequence, "sequence": sequence, "kind": kind, "subjects": subjects, "payload": payload}
