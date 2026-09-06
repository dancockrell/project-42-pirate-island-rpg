extends SceneTree

## Brief section 14 on the real bridge: while the game is paused, the strategic
## clock does not move, and it moves again the moment the last surface closes.
##
## The clock this suite reads is the live native snapshot's -- `campaign_day`
## and `time_segment` -- not a Godot-side counter, because a Godot-side counter
## could agree with a simulation that had advanced anyway. The bridge-
## availability guard is copied from native_simulation_port_test.gd and
## expedition_prototype_test.gd: the skip line CI's fallback-marker list already
## watches for, so a run that never reached the bridge cannot pass as a proof.

## How many refused advances the paused clock must survive. More than one,
## because a guard that only caught the first call would still let the day turn.
const REFUSED_ADVANCE_ATTEMPTS := 5

## A second surface holding the game at the same time as the settings panel.
## The brief pauses for reading surfaces as well, and the two must not release
## each other.
const READING_REASON := "reading_surface"

var failures := 0


func _init() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Game pause native test skipped: bridge is not registered in this running Godot process.")
		quit(0)
		return
	var game_pause: Node = root.get_node_or_null("GamePause")
	check(game_pause != null, "GamePause must be registered as an autoload")
	var session: Node = root.get_node_or_null("CampaignSession")
	check(session != null, "CampaignSession must be registered as an autoload")
	if game_pause == null or session == null:
		finish()
		return

	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the validated content bundle must load")
	session.reset_for_test()
	var started: Dictionary = session.begin_if_needed(catalog)
	check(bool(started.get("configured", false)), "the campaign must start through the native bridge")
	if not bool(started.get("configured", false)):
		finish()
		return

	check(not game_pause.is_paused(), "a fresh game must not begin paused")
	check(not paused, "an unpaused game must leave the scene tree running")

	# Opening the settings surface is what pauses. The panel takes the reason
	# itself, so this is the player's route to pause and not a test shortcut.
	var scene := load("res://scenes/ui/settings_panel.tscn") as PackedScene
	check(scene != null, "the settings panel scene must load")
	if scene == null:
		finish()
		return
	var panel := scene.instantiate() as SettingsPanel
	root.add_child(panel)
	await process_frame
	check(game_pause.is_paused(), "opening settings must pause the game")
	check(paused, "a paused game must pause the scene tree")
	check(game_pause.pause_reasons() == [game_pause.SETTINGS_REASON], "the settings panel must hold exactly its own reason")

	# The clock, before any refused advance.
	var before: Dictionary = session.snapshot()
	var day_before := int(before.get("campaign_day", 0))
	var segment_before := str(before.get("time_segment", ""))
	check(day_before > 0, "the native snapshot must carry a campaign day")

	for attempt in REFUSED_ADVANCE_ATTEMPTS:
		var refused: Dictionary = session.resolve_midnight()
		check(not bool(refused.get("configured", true)), "advance %d must be refused while paused" % attempt)
		check(str(refused.get("error", "")) == "game_paused", "a refused advance must say why: attempt %d" % attempt)
		check(not bool((refused.get("metadata", {}) as Dictionary).get("authoritative", true)), "a refusal must not present itself as an authoritative snapshot")
	var during: Dictionary = session.snapshot()
	check(int(during.get("campaign_day", 0)) == day_before, "the strategic clock must not turn while the game is paused")
	check(str(during.get("time_segment", "")) == segment_before, "the time segment must not move while the game is paused")

	# Two surfaces, one pause. A reading surface opens on top of settings, and
	# closing settings must not resume the island underneath it.
	check(game_pause.pause(READING_REASON), "a second surface must be able to add its own reason")
	check(game_pause.pause_reasons() == [READING_REASON, game_pause.SETTINGS_REASON], "both reasons must be readable while both surfaces are open")
	check(not game_pause.pause(READING_REASON), "the same reason twice must not stack")
	panel.close()
	await process_frame
	check(game_pause.is_paused(), "closing settings must leave the game paused while a reading surface is open")
	check(paused, "the scene tree must stay paused while any reason stands")
	check(game_pause.pause_reasons() == [READING_REASON], "the settings reason must be the only one released")
	var still_refused: Dictionary = session.resolve_midnight()
	check(str(still_refused.get("error", "")) == "game_paused", "one released reason must not let the clock turn")
	check(int(session.snapshot().get("campaign_day", 0)) == day_before, "the day must still stand after the first surface closes")

	# The last surface closes and the island runs again.
	check(game_pause.resume(READING_REASON), "releasing the last reason must resume the game")
	check(not game_pause.is_paused(), "no reason left means no pause")
	check(not paused, "resuming must let the scene tree run")
	check(not game_pause.resume(READING_REASON), "releasing a reason that is not held must change nothing")
	var advanced: Dictionary = session.resolve_midnight()
	check(bool(advanced.get("configured", false)), "midnight must resolve through the bridge once nothing holds the game")
	check(int(advanced.get("campaign_day", 0)) == day_before + 1, "a resumed game must turn the day")

	await test_settings_persist()
	finish()


## The three accessibility values survive the settings file. Nothing else reads
## them yet -- the B9 card asks for the surface and the persistence and stops
## there -- so this is the whole of their contract today.
func test_settings_persist() -> void:
	var scene := load("res://scenes/ui/settings_panel.tscn") as PackedScene
	if scene == null:
		return
	var panel := scene.instantiate() as SettingsPanel
	root.add_child(panel)
	await process_frame
	panel.set_text_scale(1.45)
	panel.set_high_contrast(true)
	panel.set_reduced_motion(true)
	check(panel.save_settings() == OK, "settings must be writable to the user settings file")
	panel.close()
	await process_frame

	var reopened := scene.instantiate() as SettingsPanel
	root.add_child(reopened)
	await process_frame
	check(is_equal_approx(reopened.text_scale, 1.45), "a reopened panel must read back the saved text scale")
	check(reopened.high_contrast, "a reopened panel must read back the saved contrast choice")
	check(reopened.reduced_motion, "a reopened panel must read back the saved motion choice")
	# Out-of-range text scale is clamped, never refused: an unreadable game
	# cannot reach its own settings page to fix itself.
	reopened.set_text_scale(9.0)
	check(is_equal_approx(reopened.text_scale, SettingsPanel.MAXIMUM_TEXT_SCALE), "an impossible text scale must clamp to the readable maximum")
	reopened.set_text_scale(SettingsPanel.DEFAULT_TEXT_SCALE)
	reopened.set_high_contrast(false)
	reopened.set_reduced_motion(false)
	check(reopened.save_settings() == OK, "the restored defaults must be writable")
	reopened.close()
	await process_frame


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Game pause tests passed.")
	quit(0)
