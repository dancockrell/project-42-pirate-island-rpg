extends SceneTree

var failures := 0

func _init() -> void:
	check(ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS), "native bridge class must be registered")
	var port := NativeSimulationPort.new()
	check(port.is_available(), "native port must own a bridge")
	var before := port.create_debug_battle()
	check(before.metadata.source == "rust_gdextension", "snapshot must name Rust authority")
	check(before.phase == "awaiting_actor", "prototype must start awaiting an actor")
	check(before.actors.size() == 5, "native fixture must expose a party-scale encounter: Betty, Ayla, Vix, the Razorbeak and Captain Michael, whom B5 stands in the slice")
	check(before.has("recovery_openings") and before.recovery_openings.is_empty(), "native snapshot must expose an initially empty recovery-opening collection")
	check(actor(before, "character.heroine.ayla").vitality == 0, "Ayla must begin defeated so Combat Revival is testable")
	check(actor(before, "character.heroine.vix").vitality == 10, "Vix must begin wounded so healing and lethal reactions are testable")
	var start := port.start()
	check(start.map(func(event): return event.kind) == ["battle_started", "turn_started"], "start event order differs")
	var rejected := port.submit({
		"protocol_version": 9,
		"command_id": "command.test.bad_version",
		"battle_id": "battle.prototype.returning_names",
		"actor_id": "character.heroine.betty",
		"kind": "use_skill",
		"skill_id": "skill.betty.guarded_strike",
		"target_ids": ["enemy.raptor.razorbeak.prototype"]
	})
	check(rejected.size() == 1, "unsupported protocol must emit exactly one rejection")
	check(rejected[0].kind == "command_rejected", "unsupported protocol must reject")
	check(rejected[0].payload.reason == "unsupported_protocol_version", "rejection reason differs")
	check(actor(port.bridge.snapshot(), "enemy.raptor.razorbeak.prototype").vitality == 70, "rejected protocol must not mutate battle")
	var events := port.submit({
		"command_id": "command.test.guarded_strike",
		"battle_id": "battle.prototype.returning_names",
		"actor_id": "character.heroine.betty",
		"kind": "use_skill",
		"skill_id": "skill.betty.guarded_strike",
		"target_ids": ["enemy.raptor.razorbeak.prototype"]
	})
	var kinds := events.map(func(event): return event.kind)
	check(kinds.slice(0, 4) == ["command_accepted", "actor_focused", "damage_applied", "guard_changed"], "accepted event order differs")
	check(events[2].payload.remaining_vitality == 58, "bridge must project authoritative remaining vitality")
	var recommendation := port.recommended_enemy_command("command.test.recommended")
	check(recommendation.available, "native bridge must recommend a command for the active hostile")
	check(recommendation.target_ids == ["character.heroine.vix"], "Razorbeak must select the most wounded living party member")
	check(recommendation.rationale == "finish_most_wounded", "enemy rationale must be stable and machine-readable")
	check(recommendation.guard_break_amount == 0, "native recommendation must expose zero guard break for Rushing Bite")
	check(recommendation.raw_damage == 16 and recommendation.vitality_damage == 16, "enemy projection must expose exact damage facts")
	check(recommendation.lethal and recommendation.fatal_intercept_available, "enemy projection must expose lethal and reaction facts")
	test_condition_cleanse()
	test_rescue_and_interception()
	test_fatal_intercept()
	test_mobile_infirmary()
	test_combat_revival()
	if failures > 0:
		quit(1)
		return
	print("NativeSimulationPort tests passed.")
	quit(0)

func fresh_port() -> NativeSimulationPort:
	var port := NativeSimulationPort.new()
	port.create_debug_battle()
	port.start()
	return port

func command(port: NativeSimulationPort, id: String, actor_id: String, skill_id: String, targets: Array[String]) -> Array[Dictionary]:
	return port.submit({
		"command_id": id,
		"battle_id": "battle.prototype.returning_names",
		"actor_id": actor_id,
		"kind": "use_skill",
		"skill_id": skill_id,
		"target_ids": targets
	})

func actor(snapshot: Dictionary, actor_id: String) -> Dictionary:
	for candidate in snapshot.get("actors", []):
		if candidate.get("id", "") == actor_id:
			return candidate
	return {}

func test_condition_cleanse() -> void:
	var port := fresh_port()
	var events := command(port, "command.test.cleanse", "character.heroine.betty", "skill.betty.condition_cleanse", ["character.heroine.vix"])
	check(events.any(func(event): return event.kind == "status_removed"), "Condition Cleanse must remove Vix's authored poison")
	check(events.any(func(event): return event.kind == "status_removed" and event.payload.status_kind == "poisoned"), "status removal must expose the removed condition kind")
	check(actor(port.bridge.snapshot(), "character.heroine.vix").vitality == 18, "Condition Cleanse must heal Vix by eight")

func test_rescue_and_interception() -> void:
	var port := fresh_port()
	var rescue := command(port, "command.test.rescue", "character.heroine.betty", "skill.betty.rescue_charge", ["character.heroine.vix", "enemy.raptor.razorbeak.prototype"])
	check(rescue.any(func(event): return event.kind == "interception_set"), "Rescue Charge must establish an interception")
	check(actor(port.bridge.snapshot(), "character.heroine.betty").band == 1, "Rescue Charge must move Betty to Vix's band")
	var bite := command(port, "command.test.intercept", "enemy.raptor.razorbeak.prototype", "skill.enemy.razorbeak.rushing_bite", ["character.heroine.vix"])
	check(bite.any(func(event): return event.kind == "interception_triggered"), "Rushing Bite aimed at Vix must be redirected to Betty")
	check(actor(port.bridge.snapshot(), "character.heroine.vix").vitality == 10, "interception must preserve Vix's vitality")
	check(actor(port.bridge.snapshot(), "character.heroine.betty").vitality == 84, "intercepted bite must damage Betty")

func test_mobile_infirmary() -> void:
	var port := fresh_port()
	var events := command(port, "command.test.infirmary", "character.heroine.betty", "skill.betty.mobile_infirmary", [])
	check(events.any(func(event): return event.kind == "battlefield_effect_created"), "Mobile Infirmary must create a persistent effect")
	check(events.any(func(event): return event.kind == "battlefield_effect_pulse"), "Mobile Infirmary must pulse immediately")
	check(actor(port.bridge.snapshot(), "character.heroine.vix").vitality == 20, "first infirmary pulse must heal living Vix")
	check(actor(port.bridge.snapshot(), "character.heroine.ayla").vitality == 0, "infirmary must not revive defeated Ayla")

func test_fatal_intercept() -> void:
	var port := fresh_port()
	command(port, "command.test.open_reaction", "character.heroine.betty", "skill.betty.guarded_strike", ["enemy.raptor.razorbeak.prototype"])
	var events := command(port, "command.test.lethal_bite", "enemy.raptor.razorbeak.prototype", "skill.enemy.razorbeak.rushing_bite", ["character.heroine.vix"])
	check(events.any(func(event): return event.kind == "reaction_triggered" and event.payload.skill_id == "skill.betty.fatal_intercept"), "lethal bite must trigger Betty's automatic Fatal Intercept")
	check(events.any(func(event): return event.kind == "defeat_prevented"), "Fatal Intercept must cancel the lethal hit")
	check(actor(port.bridge.snapshot(), "character.heroine.vix").vitality == 10, "Fatal Intercept must preserve Vix's vitality")

func test_combat_revival() -> void:
	var port := fresh_port()
	var events := command(port, "command.test.revival", "character.heroine.betty", "skill.betty.combat_revival", ["character.heroine.ayla"])
	var kinds := events.map(func(event): return event.kind)
	check(kinds.find("status_removed") < kinds.find("actor_revived"), "revival must remove statuses before restoring Ayla")
	check(kinds.find("actor_revived") < kinds.find("bonus_turn_granted"), "revival must restore Ayla before granting her bonus turn")
	check(events[-1].kind == "turn_started" and events[-1].subjects[0] == "character.heroine.ayla", "revived Ayla must immediately become active")
	check(actor(port.bridge.snapshot(), "character.heroine.ayla").vitality == 32, "Ayla must revive at forty percent vitality")
	var hold := command(port, "command.test.ayla_hold", "character.heroine.ayla", "skill.system.hold_position", [])
	check(hold.any(func(event): return event.kind == "guard_changed"), "prototype support actor must be able to Hold Position")
	check(hold.any(func(event): return event.kind == "turn_started" and event.subjects[0] == "enemy.raptor.razorbeak.prototype"), "bonus turn must resume at Betty's natural successor")

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
