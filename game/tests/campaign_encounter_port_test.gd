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
	# Roads cost rations and the party lands with none (A3, reaching the engine
	# under C13), so the campaign opens the way the Rust slice does: salvage the
	# wreck, then climb. Each leg is checked rather than fire-and-forget, so a
	# refused road fails here by name instead of as a missing encounter later.
	var salvage: Dictionary = campaign_session.use_anchor("anchor.black_beach.salvage_point")
	check(bool(salvage.get("configured", false)), "the wreck must be salvageable through the campaign session")
	var estate: Dictionary = campaign_session.travel("world.portal.black_beach_to_damaged_estate")
	check(estate.get("active_location_id") == "world.cell.damaged_estate", "the estate climb must arrive")
	var river: Dictionary = campaign_session.travel("world.portal.damaged_estate_to_river_landing")
	check(river.get("active_location_id") == "world.cell.river_landing", "the river gate must arrive")
	var terrace: Dictionary = campaign_session.travel("world.portal.river_landing_to_reception_terrace_safe_road")
	check(terrace.get("active_location_id") == "world.cell.reception_terrace", "the safe road must arrive, paid for with salvaged rations (refused: %s)" % str(terrace.get("error", "")))
	check(campaign_session.has_pending_encounter(), "Terrace travel must arm the campaign encounter")
	var port := CampaignEncounterSimulationPort.new(campaign_session)
	check(port.is_available(), "campaign encounter adapter must accept the armed session")
	var snapshot := port.create_debug_battle()
	check(snapshot.get("battle_id") == "battle.prototype.returning_names", "campaign adapter must create the battle declared by the encounter")
	# B17: the battle the campaign arms carries the campaign's bond ranks, not the
	# review fixture's. This session is fresh, so Betty stands at the floor and her
	# rank C Condition Cleanse is not hers yet.
	check(campaign_betty(snapshot).get("bond_rank", "") == "D", "a fresh campaign's Betty must reach the fight at bond rank D, not at the review fixture's SSS")
	var started := port.start()
	check(not started.is_empty() and started[0].get("kind") == "battle_started", "campaign encounter must start through the native retained bridge")
	check(campaign_session.has_pending_encounter(), "battle start must not erase the pending encounter before an outcome")
	campaign_session.reset_for_test()
	finish()


func campaign_betty(snapshot: Dictionary) -> Dictionary:
	for candidate in snapshot.get("actors", []):
		if candidate.get("id", "") == "character.heroine.betty":
			return candidate
	return {}


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
