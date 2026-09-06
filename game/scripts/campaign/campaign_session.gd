extends Node

## Runtime owner for one native campaign bridge. This node never calculates
## routes, encounters, outcomes, or resources; it only keeps the same
## NativeExpeditionPort alive while Godot swaps presentation scenes.

const INITIAL_SEED := 42

var expedition: NativeExpeditionPort
var catalog: ContentCatalog
var latest_snapshot: Dictionary = {}


func begin_if_needed(next_catalog: ContentCatalog) -> Dictionary:
	if expedition != null and bool(latest_snapshot.get("configured", false)):
		return latest_snapshot.duplicate(true)
	catalog = next_catalog
	if not NativeExpeditionPort.bridge_is_registered():
		latest_snapshot = {
			"configured": false,
			"error": "native_expedition_bridge_unavailable",
			"metadata": {"source": "campaign_session", "authoritative": false}
		}
		return latest_snapshot.duplicate(true)
	expedition = NativeExpeditionPort.new()
	latest_snapshot = expedition.configure_from_catalog(catalog, INITIAL_SEED)
	return latest_snapshot.duplicate(true)


func travel(portal_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.travel(portal_id)
	return latest_snapshot.duplicate(true)


func use_anchor(anchor_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.use_anchor(anchor_id)
	return latest_snapshot.duplicate(true)


func set_control(cell_id: String, faction_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.set_control(cell_id, faction_id)
	return latest_snapshot.duplicate(true)


func controller_of(cell_id: String) -> String:
	if expedition == null:
		return ""
	return expedition.controller_of(cell_id)


func effective_risk(portal_id: String) -> Variant:
	if expedition == null:
		return unavailable_state()
	return expedition.effective_risk(portal_id)


func inspect(observation_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.inspect(observation_id)
	return latest_snapshot.duplicate(true)


func resolve_midnight() -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.resolve_midnight()
	return latest_snapshot.duplicate(true)


func snapshot() -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.snapshot()
	return latest_snapshot.duplicate(true)


func pending_encounter() -> Dictionary:
	return latest_snapshot.get("pending_encounter", {}).duplicate(true)


func has_pending_encounter() -> bool:
	return not pending_encounter().is_empty()


## Campaign encounters are created by the exact native expedition bridge that
## armed them. The scene layer cannot substitute a battle ID or create a
## disconnected debug fight while a campaign handoff is active.
func begin_pending_battle() -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return battle_unavailable_snapshot("campaign_session_uninitialized")
	return expedition.bridge.begin_pending_battle()


func start_pending_battle() -> Array[Dictionary]:
	if expedition == null or expedition.bridge == null:
		return [battle_unavailable_event("campaign_session_uninitialized")]
	return typed_event_array(expedition.bridge.start_pending_battle())


func submit_pending_battle(command: Dictionary) -> Array[Dictionary]:
	if expedition == null or expedition.bridge == null:
		return [battle_unavailable_event("campaign_session_uninitialized")]
	var normalized := {
		"protocol_version": int(command.get("protocol_version", 1)),
		"command_id": str(command.get("command_id", "")),
		"battle_id": str(command.get("battle_id", "")),
		"actor_id": str(command.get("actor_id", "")),
		"kind": str(command.get("kind", "")),
		"skill_id": str(command.get("skill_id", "")),
		"target_ids": Array(command.get("target_ids", []), TYPE_STRING, "", null)
	}
	var events := typed_event_array(expedition.bridge.submit_pending_command(normalized))
	latest_snapshot = expedition.snapshot()
	return events


func recommended_pending_enemy_command(command_id: String) -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return {"available": false, "reason": "campaign_session_uninitialized"}
	return expedition.bridge.recommended_pending_enemy_command(command_id)


func pending_battle_snapshot() -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return battle_unavailable_snapshot("campaign_session_uninitialized")
	return expedition.bridge.pending_battle_snapshot()


func reset_for_test() -> void:
	expedition = null
	catalog = null
	latest_snapshot = {}


func unavailable_state() -> Dictionary:
	return {
		"configured": false,
		"error": "campaign_session_uninitialized",
		"metadata": {"source": "campaign_session", "authoritative": false}
	}


func typed_event_array(value: Variant) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if value is Array:
		for event in value:
			if event is Dictionary:
				result.append(event)
	return result


func battle_unavailable_snapshot(reason: String) -> Dictionary:
	return {"battle_id": "", "phase": "error", "actors": [], "error": {"kind": reason}}


func battle_unavailable_event(reason: String) -> Dictionary:
	return {"event_id": "event.campaign.unavailable", "sequence": 0, "kind": "command_rejected", "subjects": [], "payload": {"reason": reason}}
