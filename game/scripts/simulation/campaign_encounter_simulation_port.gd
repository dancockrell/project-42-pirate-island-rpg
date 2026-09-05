class_name CampaignEncounterSimulationPort
extends SimulationPort

## Routes a battle scene to the one pending encounter retained by
## CampaignSession. It deliberately owns no combat state: the native expedition
## bridge owns both the encounter handoff and the battle it creates.

var campaign_session: Node


func _init(next_campaign_session: Node) -> void:
	campaign_session = next_campaign_session


func is_available() -> bool:
	return campaign_session != null and campaign_session.has_method("has_pending_encounter") and campaign_session.has_pending_encounter()


func create_debug_battle() -> Dictionary:
	if not is_available():
		return error_snapshot("no_pending_campaign_encounter")
	return campaign_session.begin_pending_battle()


func start() -> Array[Dictionary]:
	if campaign_session == null:
		return [error_event("campaign_session_unavailable")]
	return campaign_session.start_pending_battle()


func submit(command: Dictionary) -> Array[Dictionary]:
	if campaign_session == null:
		return [error_event("campaign_session_unavailable")]
	return campaign_session.submit_pending_battle(command)


func recommended_enemy_command(command_id: String) -> Dictionary:
	if campaign_session == null:
		return {"available": false, "reason": "campaign_session_unavailable"}
	return campaign_session.recommended_pending_enemy_command(command_id)


func error_snapshot(reason: String) -> Dictionary:
	return {"battle_id": "", "phase": "error", "actors": [], "error": {"kind": reason}}


func error_event(reason: String) -> Dictionary:
	return {"event_id": "event.campaign.adapter_error", "sequence": 0, "kind": "command_rejected", "subjects": [], "payload": {"reason": reason}}
