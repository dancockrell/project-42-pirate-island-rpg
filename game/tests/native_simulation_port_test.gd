extends SceneTree

var failures := 0

func _init() -> void:
	check(ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS), "native bridge class must be registered")
	var port := NativeSimulationPort.new()
	check(port.is_available(), "native port must own a bridge")
	var before := port.create_debug_battle()
	check(before.metadata.source == "rust_gdextension", "snapshot must name Rust authority")
	check(before.phase == "awaiting_actor", "prototype must start awaiting an actor")
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
	check(port.bridge.snapshot().actors[1].vitality == 70, "rejected protocol must not mutate battle")
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
	if failures > 0:
		quit(1)
		return
	print("NativeSimulationPort tests passed.")
	quit(0)

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
