extends SceneTree

## B8. Proves save slots and continue against the live native expedition
## bridge: a campaign is written to a slot, the session is reset, the campaign
## is continued, and the legal actions on the far side are the same list. It
## also proves the two things that are easy to get quietly wrong -- that
## "newest" is the save's own campaign day and not the file's timestamp, and
## that a slot Rust refuses is refused with its reason and leaves the running
## campaign exactly where it stood.
##
## Live bridge only: MockSimulationPort mirrors the battle boundary and has no
## campaign in it, so a mock here would prove nothing about the save format.

const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")

var failures := 0
var campaign_session: Node
var catalog: ContentCatalog


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Save slots test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	campaign_session = root.get_node_or_null("CampaignSession")
	if campaign_session == null:
		campaign_session = CampaignSessionScript.new()
		campaign_session.name = "CampaignSession"
		root.add_child(campaign_session)
	catalog = ContentCatalog.new()
	check(catalog.load_default() == OK, "catalog must load before a campaign can be saved")
	clear_slots()
	test_save_reset_and_continue()
	test_newest_is_the_saves_own_day_not_the_file_time()
	test_a_slot_of_nonsense_is_refused_and_the_campaign_stands()
	test_a_slot_from_a_future_build_is_refused_and_the_campaign_stands()
	test_a_slot_name_cannot_escape_the_save_directory()
	clear_slots()
	finish()


## The card's own done-when: save, quit the session, continue, same legal
## actions.
func test_save_reset_and_continue() -> void:
	clear_slots()
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var climbed := campaign_at_the_river_landing()
	var expected_commands := legal_commands(climbed)
	check(not expected_commands.is_empty(), "a campaign at the river landing must have legal actions to compare")

	var saved: Dictionary = campaign_session.save_to_slot("continue_proof")
	check(bool(saved.get("configured", false)), "saving a live campaign must be accepted: %s" % str(saved.get("error", "")))
	check(str(saved.get("slot", "")) == "continue_proof", "a saved slot must name itself")
	check(FileAccess.file_exists("user://saves/continue_proof.json"), "a saved slot must exist under user://saves")

	var listed := campaign_session.list_slots()
	check(listed.size() == 1, "one save must list as one slot, not %d" % listed.size())
	var entry: Dictionary = listed[0] if listed.size() > 0 else {}
	check(bool(entry.get("readable", false)), "a slot just written by the bridge must read back")
	check(int(entry.get("campaign_day", 0)) == int(climbed.get("campaign_day", 0)), "a slot must report the campaign day its own save carries")
	check(int(entry.get("save_version", 0)) == int(climbed.get("save_version", 0)), "a slot must report the save version its own save carries")

	# Quit the session the way the game does between screens: nothing of the
	# campaign survives in Godot, so what comes back can only have come from
	# the slot.
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var fresh := campaign_session.snapshot()
	check(str(fresh.get("active_location_id", "")) == "world.cell.black_beach", "a reset session must begin a new campaign on the beach")
	check(legal_commands(fresh) != expected_commands, "a new campaign must not already offer the saved campaign's actions, or this proves nothing")

	var continued: Dictionary = campaign_session.continue_newest()
	check(bool(continued.get("configured", false)), "continuing the only slot must be accepted: %s" % str(continued.get("error", "")))
	check(str(continued.get("active_location_id", "")) == "world.cell.river_landing", "continuing must restore the saved location")
	check(legal_commands(continued) == expected_commands, "continuing must restore exactly the saved campaign's legal actions")
	check(legal_commands(campaign_session.snapshot()) == expected_commands, "the bridge itself must hold the continued campaign, not just the returned dictionary")


## Two slots: the later campaign written to disk first and named last. File
## time and alphabetical order both point at the wrong slot, so only the save's
## own day and hour can pick the right one.
func test_newest_is_the_saves_own_day_not_the_file_time() -> void:
	clear_slots()
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var climbed := campaign_at_the_river_landing()
	var first_day := int(climbed.get("campaign_day", 0))
	var turned: Dictionary = campaign_session.resolve_midnight()
	check(bool(turned.get("configured", false)), "midnight must resolve so the two slots stand on different days: %s" % str(turned.get("error", "")))
	var later_day := int(turned.get("campaign_day", 0))
	check(later_day > first_day, "midnight must turn the campaign day")
	var later_saved: Dictionary = campaign_session.save_to_slot("zulu")
	check(bool(later_saved.get("configured", false)), "the later campaign must save")

	# Written second, so it is the newest file on disk; named first, so it also
	# wins any name-ordered tie. It is still the older campaign.
	var restarted: Dictionary = campaign_session.new_game()
	check(bool(restarted.get("configured", false)), "new_game must configure a fresh campaign: %s" % str(restarted.get("error", "")))
	check(str(restarted.get("active_location_id", "")) == "world.cell.black_beach", "a new game must begin on the beach")
	check(int(restarted.get("campaign_day", 0)) == first_day, "a new game must begin on the first campaign day")
	var earlier_saved: Dictionary = campaign_session.save_to_slot("alpha")
	check(bool(earlier_saved.get("configured", false)), "the fresh campaign must save")

	check(campaign_session.list_slots().size() == 2, "both slots must list")
	var continued: Dictionary = campaign_session.continue_newest()
	check(bool(continued.get("configured", false)), "continuing must be accepted: %s" % str(continued.get("error", "")))
	check(int(continued.get("campaign_day", 0)) == later_day, "continue must pick the save standing on the later campaign day, not the newer file")
	check(str(continued.get("active_location_id", "")) == "world.cell.river_landing", "continue must restore the later campaign's location")


func test_a_slot_of_nonsense_is_refused_and_the_campaign_stands() -> void:
	clear_slots()
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var standing := campaign_at_the_river_landing()
	var standing_commands := legal_commands(standing)
	write_slot("broken", "{ this was never a save")

	var listed := campaign_session.list_slots()
	check(listed.size() == 1, "an unreadable slot must still be listed rather than hidden")
	check(not bool((listed[0] as Dictionary).get("readable", true)), "a slot that does not parse must be listed unreadable")
	check(str((listed[0] as Dictionary).get("error", "")) == "malformed_json", "an unreadable slot must name why")

	var refused: Dictionary = campaign_session.continue_newest()
	check(not bool(refused.get("configured", true)), "a slot of nonsense must be refused, never loaded")
	check(str(refused.get("error", "")) == "malformed_json", "the refusal must carry the bridge's own reason, not a generic failure: %s" % str(refused.get("error", "")))
	check(legal_commands(campaign_session.snapshot()) == standing_commands, "a refused load must leave the running campaign untouched")
	check(str(campaign_session.snapshot().get("active_location_id", "")) == "world.cell.river_landing", "a refused load must not quietly restart the campaign")


func test_a_slot_from_a_future_build_is_refused_and_the_campaign_stands() -> void:
	clear_slots()
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var standing := campaign_at_the_river_landing()
	var standing_commands := legal_commands(standing)

	# A save whose every other field is exactly what this build writes, so the
	# only thing that can refuse it is the version gate itself.
	var document: String = campaign_session.expedition.save_json()
	check(document.begins_with("{\"save_version\":"), "the canonical save must open with its version")
	var from_the_future := document.replace("{\"save_version\":%d," % int(standing.get("save_version", 1)), "{\"save_version\":9999,")
	check(from_the_future != document, "the fixture must actually have changed the save version")
	write_slot("future", from_the_future)

	var listed := campaign_session.list_slots()
	check(listed.size() == 1, "a version-bumped slot is still a readable file and must list")
	check(int((listed[0] as Dictionary).get("save_version", 0)) == 9999, "a slot must report the save version it actually carries")

	var refused: Dictionary = campaign_session.continue_newest()
	check(not bool(refused.get("configured", true)), "a save from a future build must be refused")
	check(str(refused.get("error", "")) == "future_save_version", "the refusal must be the bridge's version code: %s" % str(refused.get("error", "")))
	check(legal_commands(campaign_session.snapshot()) == standing_commands, "a refused version must leave the running campaign untouched")


func test_a_slot_name_cannot_escape_the_save_directory() -> void:
	clear_slots()
	campaign_session.reset_for_test()
	campaign_session.begin_if_needed(catalog)
	var escaped: Dictionary = campaign_session.save_to_slot("../escape")
	check(not bool(escaped.get("configured", true)), "a slot name with a path in it must be refused")
	check(str(escaped.get("error", "")) == "invalid_slot_name", "a refused slot name must say so")
	check(campaign_session.list_slots().is_empty(), "a refused save must write nothing")
	var empty: Dictionary = campaign_session.continue_newest()
	check(not bool(empty.get("configured", true)), "continuing with no slots on disk must be refused")
	check(str(empty.get("error", "")) == "no_save_slot", "an empty save directory must say so")


## The vertical slice's opening, played through the live bridge: the party
## lands with no rations, salvages the wreck, then climbs to the river landing.
## Every leg is checked so a refused road fails here by name.
func campaign_at_the_river_landing() -> Dictionary:
	var salvage: Dictionary = campaign_session.use_anchor("anchor.black_beach.salvage_point")
	check(bool(salvage.get("configured", false)), "the wreck must be salvageable: %s" % str(salvage.get("error", "")))
	var estate: Dictionary = campaign_session.travel("world.portal.black_beach_to_damaged_estate")
	check(str(estate.get("active_location_id", "")) == "world.cell.damaged_estate", "the estate climb must arrive: %s" % str(estate.get("error", "")))
	var river: Dictionary = campaign_session.travel("world.portal.damaged_estate_to_river_landing")
	check(str(river.get("active_location_id", "")) == "world.cell.river_landing", "the river gate must arrive: %s" % str(river.get("error", "")))
	return river


func legal_commands(snapshot: Dictionary) -> Array:
	var commands: Array = []
	for command in snapshot.get("legal_commands", []):
		commands.append(str(command))
	return commands


func write_slot(slot: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(CampaignSessionScript.SAVE_DIRECTORY)
	var file := FileAccess.open("%s/%s.json" % [CampaignSessionScript.SAVE_DIRECTORY, slot], FileAccess.WRITE)
	if file == null:
		check(false, "the test fixture must be able to write %s" % slot)
		return
	file.store_string(text)
	file.close()


func clear_slots() -> void:
	if not DirAccess.dir_exists_absolute(CampaignSessionScript.SAVE_DIRECTORY):
		return
	for file_name in DirAccess.get_files_at(CampaignSessionScript.SAVE_DIRECTORY):
		DirAccess.remove_absolute("%s/%s" % [CampaignSessionScript.SAVE_DIRECTORY, str(file_name)])


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	campaign_session.reset_for_test()
	if failures > 0:
		quit(1)
		return
	print("Save slot tests passed.")
	quit(0)
