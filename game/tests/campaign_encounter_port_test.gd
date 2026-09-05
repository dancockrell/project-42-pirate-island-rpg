extends SceneTree

## Proves a Terrace encounter is consumed through CampaignSession rather than
## a scene-local debug bridge. It intentionally stops at the created battle
## snapshot; battle rules remain covered by the native Rust suite.

var failures := 0
var campaign_session: Node
const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Campaign encounter port test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	campaign_session = root.get_node_or_null("CampaignSession")
	if campaign_session == null:
		campaign_session = CampaignSessionScript.new()
		campaign_session.name = "CampaignSession"
		root.add_child(campaign_session)
	check(campaign_session != null, "campaign encounter tests must have a persistent session owner")
	if campaign_session == null:
		finish()
		return
	campaign_session.reset_for_test()
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "catalog must load before campaign setup")
	campaign_session.begin_if_needed(catalog)
	campaign_session.travel("world.portal.black_beach_to_damaged_estate")
	campaign_session.travel("world.portal.damaged_estate_to_river_landing")
	campaign_session.travel("world.portal.river_landing_to_reception_terrace_safe_road")
	check(campaign_session.has_pending_encounter(), "Terrace travel must arm the campaign encounter")
	var port := CampaignEncounterSimulationPort.new(campaign_session)
	check(port.is_available(), "campaign encounter adapter must accept the armed session")
	var snapshot := port.create_debug_battle()
	check(snapshot.get("battle_id") == "battle.prototype.returning_names", "campaign adapter must create the battle declared by the encounter")
	var started := port.start()
	check(not started.is_empty() and started[0].get("kind") == "battle_started", "campaign encounter must start through the native retained bridge")
	check(campaign_session.has_pending_encounter(), "battle start must not erase the pending encounter before an outcome")
	campaign_session.reset_for_test()
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
	print("Campaign encounter port tests passed.")
	quit(0)
