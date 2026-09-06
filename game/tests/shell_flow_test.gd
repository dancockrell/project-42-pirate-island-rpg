extends SceneTree

## P8. The whole front door walked through the live native bridge: title, new
## game, expedition, a pending encounter, the battle screen, back, the pause
## menu, a save, the title again, and continue -- with the same legal actions on
## the far side of the round trip as before it.
##
## Every screen change here goes through `SceneFlow`, which is the point: the
## two prototypes no longer change the scene themselves, so if the flow stopped
## owning transitions this suite would never reach the battle at all.
##
## Live bridge only, guarded exactly as `native_simulation_port_test.gd` and
## `game_pause_test.gd` guard: MockSimulationPort has no campaign in it, so a
## mock here would prove nothing about saving, continuing, or the strategic
## clock the pause menu has to stop.

## The two autoloads this suite reaches for constants through. An autoload's
## name resolves to the node, and a script constant is not readable through an
## instance, so the scripts themselves are preloaded -- the same idiom
## save_slots_test.gd uses for the session.
const SceneFlowScript = preload("res://scripts/shell/scene_flow.gd")
const CampaignSessionScript = preload("res://scripts/campaign/campaign_session.gd")

## How many frames a transition is given before the suite calls it stuck. A
## threaded load of a prototype screen lands in a handful of frames; this is a
## ceiling that fails loudly instead of a suite that hangs the gate.
const TRANSITION_FRAME_BUDGET := 600

## How many refused advances the paused clock must survive under the pause menu.
## More than one, because a guard that only caught the first call would still
## let the day turn. The number matches game_pause_test.gd's for the same reason.
const REFUSED_ADVANCE_ATTEMPTS := 5

var failures := 0
var flow: Node
var session: Node
var game_pause: Node


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Shell flow test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	# Autoloads are added to the root after a `--script` main loop is
	# instantiated, so `_init` runs before any of them exist. One frame is the
	# whole difference.
	await process_frame
	flow = root.get_node_or_null("SceneFlow")
	session = root.get_node_or_null("CampaignSession")
	game_pause = root.get_node_or_null("GamePause")
	check(flow != null, "SceneFlow must be registered as an autoload")
	check(session != null, "CampaignSession must be registered as an autoload")
	check(game_pause != null, "GamePause must be registered as an autoload")
	if flow == null or session == null or game_pause == null:
		finish()
		return
	check(ProjectSettings.get_setting("application/run/main_scene", "") == SceneFlowScript.TITLE_SCENE, "the project must open on the title, not on a prototype screen")

	clear_slots()
	session.reset_for_test()
	await walk_the_shell()
	clear_slots()
	finish()


func walk_the_shell() -> void:
	# ---- The title, with nothing saved -------------------------------------
	check(flow.go_to_title(), "the flow must accept the opening transition to the title")
	if not await settled(SceneFlowScript.TITLE_SCENE):
		return
	var title := current_scene as TitleScreen
	check(title != null, "the title scene must be the current scene")
	if title == null:
		return
	check(button_of(title, TitleScreen.CONTINUE).disabled, "Continue must be dark when no campaign has been saved")
	check(not button_of(title, TitleScreen.NEW_GAME).disabled, "New Game must always be offered")
	check(button_of(title, TitleScreen.NEW_GAME).has_focus(), "the title must hand focus to a control, so a keyboard or a gamepad can move without a mouse first")
	check_focus_chain(title)
	check_credits_match_the_ledgers()

	# ---- New game ----------------------------------------------------------
	button_of(title, TitleScreen.NEW_GAME).pressed.emit()
	if not await settled(SceneFlowScript.EXPEDITION_SCENE):
		return
	var expedition := current_scene as ExpeditionPrototype
	check(expedition != null, "New Game must open the expedition screen")
	if expedition == null:
		return
	check(str(session.snapshot().get("active_location_id", "")) == "world.cell.black_beach", "a new game must begin on the beach")

	# ---- Reach the authored encounter, the way expedition_prototype_test does
	expedition.request_anchor("anchor.black_beach.salvage_point")
	expedition.request_travel("world.portal.black_beach_to_damaged_estate")
	expedition.request_travel("world.portal.damaged_estate_to_river_landing")
	expedition.request_travel("world.portal.river_landing_to_reception_terrace_safe_road")
	check(session.has_pending_encounter(), "the terrace must arm its authored encounter before the battle can be entered")

	# ---- Into the battle, through the screen's own control ------------------
	expedition.enter_pending_battle()
	if not await settled(SceneFlowScript.BATTLE_SCENE):
		return
	var battle := current_scene as BattlePrototype
	check(battle != null, "engaging must open the battle screen")
	if battle == null:
		return
	check(battle.is_campaign_encounter, "the battle must be the campaign's encounter, not a disconnected debug fight")

	# ---- And back ----------------------------------------------------------
	battle.return_to_expedition()
	if not await settled(SceneFlowScript.EXPEDITION_SCENE):
		return
	check(current_scene is ExpeditionPrototype, "returning must open the expedition screen again")
	var standing: Dictionary = session.snapshot()
	var expected_commands := legal_commands(standing)
	check(not expected_commands.is_empty(), "a campaign standing on the terrace must have legal actions to compare")
	check(str(standing.get("active_location_id", "")) == "world.cell.reception_terrace", "the round trip must leave the party where it stood")

	# ---- The pause menu holds the island ------------------------------------
	check(flow.open_pause_menu(), "Escape's menu must open over the expedition")
	await process_frame
	check(not flow.open_pause_menu(), "a second Escape must not stack a second pause menu")
	var menu := flow.pause_menu() as PauseMenu
	check(menu != null, "the flow must hand back the menu it opened")
	if menu == null:
		return
	check(game_pause.is_paused(), "the pause menu must pause the game")
	check(game_pause.pause_reasons().has(PauseMenu.PAUSE_REASON), "the pause menu must hold its own named reason")
	check(paused, "a paused game must pause the scene tree")
	var day_before := int(standing.get("campaign_day", 0))
	var segment_before := str(standing.get("time_segment", ""))
	for attempt in REFUSED_ADVANCE_ATTEMPTS:
		var refused: Dictionary = session.resolve_midnight()
		check(str(refused.get("error", "")) == "game_paused", "advance %d must be refused while the pause menu is open" % attempt)
	var during: Dictionary = session.snapshot()
	check(int(during.get("campaign_day", 0)) == day_before, "the strategic clock must not turn behind the pause menu")
	check(str(during.get("time_segment", "")) == segment_before, "the time segment must not move behind the pause menu")

	# ---- Save, through the menu's own slot picker ---------------------------
	button_of(menu, PauseMenu.SAVE).pressed.emit()
	await process_frame
	var picker := chooser_of(menu)
	check(picker != null, "Save must open the slot picker")
	if picker == null:
		return
	check(picker.row_buttons.size() == PauseMenu.OFFERED_SLOTS.size(), "the picker must offer every slot the menu names")
	picker.row_buttons[0].pressed.emit()
	await process_frame
	var slot := PauseMenu.OFFERED_SLOTS[0]
	check(FileAccess.file_exists(CampaignSessionScript.slot_path(slot)), "choosing a slot must write it")
	var listed: Array = session.list_slots()
	check(listed.size() == 1, "one save must list as one slot, not %d" % listed.size())
	var entry: Dictionary = listed[0] if listed.size() > 0 else {}
	check(str(entry.get("active_location_id", "")) == "world.cell.reception_terrace", "a listed slot must carry the location its own save records, so a load screen can tell two campaigns apart")

	# ---- Return to the title, confirmed --------------------------------------
	button_of(menu, PauseMenu.TITLE).pressed.emit()
	await process_frame
	check(menu.pending_confirmation == PauseMenu.TITLE, "leaving a running campaign must ask first")
	var confirm := confirm_button_of(menu)
	check(confirm != null, "the confirmation must offer a control to confirm with")
	if confirm == null:
		return
	confirm.pressed.emit()
	if not await settled(SceneFlowScript.TITLE_SCENE):
		return
	check(not game_pause.is_paused(), "leaving the campaign must release the pause the menu held")
	check(not paused, "the scene tree must run again once nothing holds it")

	# ---- Continue, and the same legal actions --------------------------------
	title = current_scene as TitleScreen
	check(title != null, "the confirmed return must open the title")
	if title == null:
		return
	check(not button_of(title, TitleScreen.CONTINUE).disabled, "Continue must be live once a campaign has been saved")
	check("RECEPTION TERRACE" in title.continue_detail.text, "the title must say which campaign Continue would resume: %s" % title.continue_detail.text)
	button_of(title, TitleScreen.CONTINUE).pressed.emit()
	if not await settled(SceneFlowScript.EXPEDITION_SCENE):
		return
	check(current_scene is ExpeditionPrototype, "Continue must open the expedition screen")
	var continued: Dictionary = session.snapshot()
	check(str(continued.get("active_location_id", "")) == "world.cell.reception_terrace", "continuing must restore the saved location")
	check(legal_commands(continued) == expected_commands, "the round trip must leave exactly the legal actions it started with")


# ---------------------------------------------------------------------------
# The credits page and the ledgers it copies.
#
# `content/art/` is not one of the domains the content bundle carries, so the
# credits screen cannot read the ledgers at runtime and states the attribution
# itself. That makes two places holding one fact, which is only acceptable
# because this check holds them equal: the ledgers own the attribution, the
# screen carries it, and a licence changed in one and not the other fails here.
# The ledgers live outside `res://`, so they are read through the globalized
# project path; when the suite is running from an export that has no repository
# beside it, the check says so rather than passing quietly.
# ---------------------------------------------------------------------------


func check_credits_match_the_ledgers() -> void:
	var repository := ProjectSettings.globalize_path("res://").path_join("..")
	var shared: Dictionary = read_json(repository.path_join(CreditsScreen.SHARED_LEDGER))
	if shared.is_empty():
		print("Shell flow credits check skipped: %s is not readable from this run." % CreditsScreen.SHARED_LEDGER)
		return
	check(str(shared.get("sharedRawLicensePolicy", "")) == CreditsScreen.SHARED_POLICY, "the credits must quote the shared ledger's licence policy exactly")
	var by_id := {}
	for record in shared.get("assetRecords", []):
		by_id[str((record as Dictionary).get("id", ""))] = record
	check(by_id.size() == CreditsScreen.SHARED_SOURCES.size(), "the credits must list every shared source record: %d in the ledger, %d credited" % [by_id.size(), CreditsScreen.SHARED_SOURCES.size()])
	for credited in CreditsScreen.SHARED_SOURCES:
		var record: Dictionary = by_id.get(str(credited["record"]), {})
		check(not record.is_empty(), "credited record %s must exist in the shared ledger" % str(credited["record"]))
		if record.is_empty():
			continue
		var source: Dictionary = record.get("source", {})
		check(str(source.get("creator", "")) == str(credited["creator"]), "the credited creator must be the ledger's: %s" % str(credited["record"]))
		check(str(source.get("licenseSpdx", "")) == str(credited["license"]), "the credited licence must be the ledger's: %s" % str(credited["record"]))
		check(str(source.get("canonicalUrl", "")) == str(credited["url"]), "the credited source URL must be the ledger's: %s" % str(credited["record"]))
		check(str(record.get("status", "")) == "candidate", "a shared record that stopped being a candidate must be re-credited as what it became: %s" % str(credited["record"]))

	check_credits_match_the_third_party_ledger(repository)

	var placeholders: Dictionary = read_json(repository.path_join(CreditsScreen.PLACEHOLDER_LEDGER))
	var placeholder_sources := {}
	for asset in placeholders.get("assets", []):
		placeholder_sources[str((asset as Dictionary).get("id", ""))] = str((asset as Dictionary).get("source", ""))
	var manifest: Dictionary = read_json(repository.path_join(CreditsScreen.VENDOR_MANIFEST))
	for credited_asset in CreditsScreen.ADMITTED_ASSETS:
		check(FileAccess.file_exists("res://%s" % str(credited_asset["asset"]).trim_prefix("game/")), "a credited asset must actually be in the build: %s" % str(credited_asset["asset"]))
		var record_id := str(credited_asset["record"])
		if str(credited_asset["ledger"]) == CreditsScreen.PLACEHOLDER_LEDGER:
			check(placeholder_sources.get(record_id, "") == str(credited_asset["text"]), "the credited provenance must be the placeholder ledger's own words: %s" % record_id)
		elif str(credited_asset["ledger"]) == CreditsScreen.VENDOR_MANIFEST:
			check(str(manifest.get("assetId", "")) == record_id, "the vendor manifest must be the record the credits name")
			var vendor_source: Dictionary = manifest.get("source", {})
			for fragment in [str(vendor_source.get("vendor", "")), str(vendor_source.get("generator", "")), str(vendor_source.get("downloadedAt", "")), str(vendor_source.get("creationUrl", ""))]:
				check(fragment != "" and fragment in str(credited_asset["text"]), "the credited vendor line must carry the manifest's own '%s'" % fragment)


## E12. The engine, the bindings and every crate the extension links have an
## admission record now, and the credits page carries what those records say.
## The ledger owns the facts -- name, pinned version, SPDX id, canonical URL and
## the notice text copied out of the component's own licence file -- and this
## holds the page to it component for component and body for body. A component
## the ledger records with no notice must appear on the page as pending: the one
## thing the page may never do is invent a notice for something whose licence
## text nobody has read.
func check_credits_match_the_third_party_ledger(repository: String) -> void:
	var ledger: Dictionary = read_json(repository.path_join(CreditsScreen.THIRD_PARTY_LEDGER))
	if ledger.is_empty():
		print("Shell flow credits check skipped: %s is not readable from this run." % CreditsScreen.THIRD_PARTY_LEDGER)
		return
	var components: Array = ledger.get("components", [])
	check(components.size() == CreditsScreen.THIRD_PARTY.size(), "the credits must carry every third-party component: %d in the ledger, %d credited" % [components.size(), CreditsScreen.THIRD_PARTY.size()])
	var credited := {}
	for entry in CreditsScreen.THIRD_PARTY:
		credited[str(entry["record"])] = entry
	var pending := 0
	for record in components:
		var component: Dictionary = record
		var id := str(component.get("id", ""))
		var entry: Dictionary = credited.get(id, {})
		check(not entry.is_empty(), "the credits must carry the ledger's component %s" % id)
		if entry.is_empty():
			continue
		check(str(entry["name"]) == str(component.get("name", "")), "the credited name must be the ledger's: %s" % id)
		var version: Variant = component.get("version")
		check(str(entry["version"]) == ("" if version == null else str(version)), "the credited version must be the version this repository pins: %s" % id)
		check(str(entry["license"]) == str(component.get("licenseSpdx", "")), "the credited licence must be the ledger's: %s" % id)
		check(str(entry["url"]) == str(component.get("canonicalUrl", "")), "the credited URL must be the ledger's: %s" % id)
		check(str(entry["role"]) == str(component.get("distribution", "")), "the credited role must be the ledger's: %s" % id)
		check(CreditsScreen.ROLES.has(str(entry["role"])), "every credited role must have words on the page: %s" % str(entry["role"]))
		var notice_key := str(entry["notice"])
		if bool(component.get("needsReview", false)):
			pending += 1
			check(notice_key == "", "a component the ledger has read no notice for must be credited as pending, not under a licence body: %s" % id)
		else:
			check(notice_key != "", "a component the ledger carries a notice for must be credited under a licence body: %s" % id)
			check(notice_key == str(component.get("noticeTextKey", "")), "the credited licence body must be the ledger's noticeTextKey: %s" % id)
			check(CreditsScreen.NOTICE_BODIES.has(notice_key), "the credits must carry the licence body %s that %s is credited under" % [notice_key, id])
			if CreditsScreen.NOTICE_BODIES.has(notice_key):
				check(str(CreditsScreen.NOTICE_BODIES[notice_key]) == str(component.get("noticeText", "")), "the notice on the page must be the ledger's text, character for character: %s" % id)
	check(pending == int(ledger.get("noticePendingCount", -1)), "the ledger's own pending count must be the number of components with no notice: %d counted, %d declared" % [pending, int(ledger.get("noticePendingCount", -1))])
	check(CreditsScreen.NOTICES_PENDING == pending, "the credits must say how many notices are still owed: %d on the page, %d in the ledger" % [CreditsScreen.NOTICES_PENDING, pending])
	# The other direction: a record the page credits that the ledger no longer
	# carries. Both directions are named, so whichever side loses a component
	# the failure says which one it was rather than only that a count moved.
	var ledger_ids := {}
	for record in components:
		ledger_ids[str((record as Dictionary).get("id", ""))] = true
	for entry in CreditsScreen.THIRD_PARTY:
		check(ledger_ids.has(str(entry["record"])), "%s is credited on the page and has no record in the ledger" % str(entry["record"]))
	var bodies: Dictionary = ledger.get("noticeBodies", {})
	check(CreditsScreen.NOTICE_BODIES.size() == bodies.size(), "the credits must carry every licence body the ledger names: %d in the ledger, %d on the page" % [bodies.size(), CreditsScreen.NOTICE_BODIES.size()])
	for key in bodies:
		check(CreditsScreen.NOTICE_BODIES.has(str(key)), "the credits must carry the licence body %s" % str(key))


func read_json(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		return {}
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {}
	var text := file.get_as_text()
	file.close()
	var reader := JSON.new()
	if reader.parse(text) != OK:
		return {}
	var parsed: Variant = reader.data
	return parsed if parsed is Dictionary else {}


# ---------------------------------------------------------------------------
# Helpers.
# ---------------------------------------------------------------------------


## Waits for the flow to land on `path`. Returns false, having failed a named
## check, rather than hanging the gate when a transition never completes.
func settled(path: String) -> bool:
	for frame in TRANSITION_FRAME_BUDGET:
		if flow.current_scene_path == path and not flow.is_transitioning() and current_scene != null:
			await process_frame
			return true
		await process_frame
	check(false, "the flow never landed on %s" % path)
	return false


func check_focus_chain(title: TitleScreen) -> void:
	var first := button_of(title, TitleScreen.NEW_GAME)
	var last := button_of(title, TitleScreen.QUIT)
	check(first.get_node_or_null(first.focus_neighbor_bottom) == button_of(title, TitleScreen.CONTINUE), "the menu must walk downward from the first row")
	check(first.get_node_or_null(first.focus_neighbor_top) == first, "the first row must not wrap upward past the top of the menu")
	check(last.get_node_or_null(last.focus_neighbor_bottom) == last, "the last row must not wrap downward past the bottom of the menu")


func button_of(screen, action: String) -> Button:
	return screen.menu_buttons.get(action) as Button


func chooser_of(node: Node) -> ShellChooser:
	for child in node.get_children():
		if child is ShellChooser:
			return child
	return null


func confirm_button_of(menu: PauseMenu) -> Button:
	for node in menu.confirm_row.find_children("*", "Button", true, false):
		if (node as Button).text == "CONFIRM":
			return node as Button
	return null


func legal_commands(snapshot: Dictionary) -> Array:
	var commands: Array = []
	for command in snapshot.get("legal_commands", []):
		commands.append(str(command))
	return commands


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
	if failures > 0:
		quit(1)
		return
	print("Shell flow tests passed.")
	quit(0)
