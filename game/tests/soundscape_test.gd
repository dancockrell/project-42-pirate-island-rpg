extends SceneTree
## The script rather than the `PlaceholderSynth` class name: a `--script` run
## compiles an autoload before the SceneTree's global class cache exists, so
## naming the class here is a parse error there and nowhere else (measured on
## paper_razorbeak_rig_test.gd in CI). Same reason content_registry records.
const PlaceholderSynthScript := preload("res://scripts/audio/placeholder_synth.gd")

## P9: the soundscape's contract, asserted against the authored records and the
## live autoload. Nothing here is mocked -- the suite loads the same generated
## bundle the game does and renders the same placeholders.

const BATTLE_EVENT_KINDS := [
	"activation_denied", "actor_defeated", "actor_focused", "actor_moved", "actor_revived",
	"battle_ended", "battle_retreated", "battle_started", "battlefield_effect_created",
	"battlefield_effect_pulse", "battlefield_effect_removed", "bonus_turn_granted",
	"command_accepted", "command_rejected", "damage_applied", "defeat_prevented",
	"enemy_intent_declared", "guard_changed", "interception_set", "interception_triggered",
	"reaction_triggered", "reaction_window_opened", "recovery_opening_consumed",
	"recovery_opening_created", "recovery_opening_expired", "round_started",
	"site_rule_overridden", "status_applied", "status_removed", "target_inspected",
	"turn_ended", "turn_started", "vitality_changed", "ward_line_placed", "ward_line_triggered",
]
const TIME_SEGMENTS := ["dawn", "day", "dusk", "midnight"]
const WEATHER_CONDITIONS := ["clear", "overcast", "rain", "storm", "unnatural"]
const MUSIC_STATES := ["calm", "notable", "battle", "urgent"]
const BUS_NAMES := ["Master", "Music", "Ambience", "Effects", "Voice"]

var failures := 0


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	print("FAILED: %s" % message)


func _init() -> void:
	# Autoloads are added to the root after a `--script` main loop is
	# instantiated, so `_init` runs before Soundscape exists. One frame is the
	# whole difference -- the same wait game_pause_test.gd takes.
	await process_frame
	var soundscape: Node = root.get_node_or_null("Soundscape")
	check(soundscape != null, "the Soundscape autoload is registered")
	if soundscape == null:
		quit(1)
		return
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the content bundle loads")

	check_buses()
	check_cues(soundscape, catalog)
	check_ambience(soundscape, catalog)
	check_synth(soundscape, catalog)
	check_bus_volumes(soundscape)
	check_music(soundscape)

	if failures == 0:
		print("Soundscape tests passed.")
	quit(1 if failures > 0 else 0)


## Every bus the layout declares is live, and each of the four carriers sends to
## Master rather than straight to the output.
func check_buses() -> void:
	for bus_name in BUS_NAMES:
		check(AudioServer.get_bus_index(bus_name) >= 0, "bus %s exists in the layout" % bus_name)
	for bus_name in ["Music", "Ambience", "Effects", "Voice"]:
		var index := AudioServer.get_bus_index(bus_name)
		if index >= 0:
			check(AudioServer.get_bus_send(index) == "Master", "bus %s sends to Master" % bus_name)


## Every battle event kind the bridge emits resolves to a cue, and so does every
## presentation cue of every skill.
func check_cues(soundscape: Node, catalog: ContentCatalog) -> void:
	for kind in BATTLE_EVENT_KINDS:
		var event := {"kind": kind, "command_id": "", "subjects": [], "payload": {}}
		var cue_id: String = soundscape.cue_id_for_event(event)
		check(cue_id == "audio.cue.event.%s" % kind, "battle event %s resolves to its own cue, got '%s'" % [kind, cue_id])
		check(soundscape.on_battle_event(event) == cue_id, "battle event %s plays the cue it resolved" % kind)

	var skill_ids := catalog.ids_with_prefix("skill.")
	check(skill_ids.size() >= 14, "the skill records are present, found %d" % skill_ids.size())
	for skill_id in skill_ids:
		var animation: Dictionary = catalog.get_record(skill_id).get("animation", {})
		var cues: Array = animation.get("presentationCues", [])
		check(not cues.is_empty(), "%s has presentation cues" % skill_id)
		for cue: Dictionary in cues:
			var audio_cue_id := str(cue.get("audioCueId", ""))
			check(catalog.has(audio_cue_id), "%s beat %s names a cue record that exists: '%s'" % [skill_id, cue.get("beat", ""), audio_cue_id])
			check(audio_cue_id == "audio.cue.%s" % str(cue.get("audio", "")), "%s beat %s audioCueId matches its audio label" % [skill_id, cue.get("beat", "")])
		# A bound event kind must reach that skill's own beat, not the fallback.
		for kind: String in animation.get("eventBindings", {}):
			var bound_beat := str(animation.eventBindings[kind])
			var expected := ""
			for cue: Dictionary in cues:
				if str(cue.get("beat", "")) == bound_beat:
					expected = str(cue.get("audioCueId", ""))
			var event := {
				"kind": kind, "command_id": "command.test", "subjects": [],
				"payload": {"skill_id": skill_id},
			}
			var resolved: String = soundscape.cue_id_for_event(event)
			check(resolved == expected, "%s binds %s to beat %s and its cue %s, got '%s'" % [skill_id, kind, bound_beat, expected, resolved])

	# The skill is remembered from the accepted command, so a later event in the
	# same command finds the beat without carrying the skill ID itself.
	soundscape.skill_by_command.clear()
	soundscape.on_battle_event({
		"kind": "command_accepted", "command_id": "command.remember", "subjects": [],
		"payload": {"skill_id": "skill.betty.guarded_strike"},
	})
	var remembered: String = soundscape.cue_id_for_event({
		"kind": "damage_applied", "command_id": "command.remember", "subjects": [], "payload": {},
	})
	check(remembered == "audio.cue.mace_armor_impact", "an accepted command is remembered so damage_applied lands on Guarded Strike's impact, got '%s'" % remembered)

	check(soundscape.cue_id_for_event({"kind": "not_an_event", "payload": {}}) == "", "an unknown event kind resolves to no cue rather than a guess")


## Every region the world declares, at every segment, in every weather.
func check_ambience(soundscape: Node, catalog: ContentCatalog) -> void:
	var region_ids: Array[String] = []
	for id in catalog.ids_with_prefix("world.region."):
		region_ids.append(id)
	check(not region_ids.is_empty(), "the world declares at least one region")
	for region_id in region_ids:
		for segment in TIME_SEGMENTS:
			for weather in WEATHER_CONDITIONS:
				var record_id: String = soundscape.ambience_id_for(region_id, segment, weather)
				check(catalog.has(record_id), "%s at %s in %s weather has an ambience record, got '%s'" % [region_id, segment, weather, record_id])

	# The snapshot chooses: the cell's own region, the snapshot's segment, and
	# the region's weather when the bridge projects it.
	var snapshot := {"active_location_id": "world.cell.black_beach", "time_segment": "dusk"}
	check(soundscape.apply(snapshot) == "audio.ambience.black_beach.dusk.clear", "a snapshot with no weather key falls back to clear")
	snapshot["weather"] = {"world.region.black_beach": {"condition": "storm"}}
	var stormy: String = soundscape.apply(snapshot)
	check(stormy == "audio.ambience.black_beach.dusk.storm", "the region's weather chooses the ambience, got '%s'" % stormy)
	var storm_record := catalog.get_record(stormy)
	for layer: Dictionary in storm_record.get("layers", []):
		var bed_id := str(layer.get("bedId", ""))
		check(is_equal_approx(float(soundscape.layer_target_db[bed_id]), float(layer.get("gainDb", 0.0))), "%s is asked for its authored gain" % bed_id)
	check(soundscape.apply({"active_location_id": "world.cell.nowhere"}) == stormy, "an unknown cell leaves the ambience alone")


## The synth renders every record, and renders it the same way twice.
func check_synth(soundscape: Node, catalog: ContentCatalog) -> void:
	soundscape.render_all()
	check(soundscape.render_failures.is_empty(), "every bed and cue rendered: %s" % str(soundscape.render_failures))
	var renderable: Array[String] = []
	renderable.append_array(catalog.ids_with_prefix("audio.bed."))
	renderable.append_array(catalog.ids_with_prefix("audio.cue."))
	check(renderable.size() >= 110, "the audio records are present, found %d" % renderable.size())
	for id in renderable:
		var record := catalog.get_record(id)
		check(str(record.get("asset", {}).get("status", "")) == "procedural_placeholder", "%s is still marked a procedural placeholder" % id)
		var stream: AudioStreamWAV = soundscape.streams_by_id.get(id, null)
		check(stream != null, "%s rendered to a stream" % id)
		if stream == null:
			continue
		check(stream.data.size() > 0, "%s rendered audible data" % id)
		var looping: bool = bool(record.get("voice", {}).get("loop", false))
		check((stream.loop_mode == AudioStreamWAV.LOOP_FORWARD) == looping, "%s loop mode matches its record" % id)
	var again := PlaceholderSynthScript.render(catalog.get_record("audio.cue.mace_armor_impact").voice)
	check(again.data == soundscape.streams_by_id["audio.cue.mace_armor_impact"].data, "the synth is deterministic: the same record renders the same samples")


func check_bus_volumes(soundscape: Node) -> void:
	var config := ConfigFile.new()
	config.set_value("audio", "master", 0.5)
	config.set_value("audio", "music", 0.25)
	config.set_value("audio", "ambience", 1.0)
	config.set_value("audio", "effects", 0.0)
	config.set_value("audio", "voice", 0.75)
	check(config.save(soundscape.SETTINGS_PATH) == OK, "the settings file is writable")
	soundscape.load_bus_volumes()
	soundscape.apply_bus_volumes()
	check(is_equal_approx(soundscape.bus_volume("master"), 0.5), "master volume is read from the config file")
	check(is_equal_approx(soundscape.bus_volume("Music"), 0.25), "a bus is found by its capitalised name too")
	check(AudioServer.is_bus_mute(AudioServer.get_bus_index("Effects")), "a bus set to zero is muted rather than left at minus infinity")
	check(soundscape.set_bus_volume("voice", 0.4), "a slider can set a bus")
	check(not soundscape.set_bus_volume("not_a_bus", 0.4), "a name that is not a bus is refused")
	check(soundscape.save_bus_volumes() == OK, "the volumes are written back")
	var reread := ConfigFile.new()
	check(reread.load(soundscape.SETTINGS_PATH) == OK, "the settings file reloads")
	check(is_equal_approx(float(reread.get_value("audio", "voice", 0.0)), 0.4), "the written value survives the round trip")


func check_music(soundscape: Node) -> void:
	for state in MUSIC_STATES:
		check(soundscape.set_music_state(state), "the music machine accepts %s" % state)
		check(soundscape.music_state == state, "the music machine is in %s" % state)
	check(not soundscape.set_music_state("panic"), "a name that is not a state is refused")
	check(soundscape.music_state == "urgent", "a refused name leaves the machine where it was")
	check(soundscape.set_music_state("calm"), "the machine returns to calm")
	check(soundscape.previous_music_state == "urgent", "the machine remembers the state it came from")
