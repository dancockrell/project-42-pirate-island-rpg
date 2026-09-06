extends Node

## Autoload `Soundscape`. One owner for everything the player hears.
##
## Three jobs, each driven from somewhere that already exists:
##
## 1. **Ambience** follows the expedition snapshot the way P3's light does.
##    `apply(snapshot)` reads the active cell's region, the `time_segment` and
##    that region's weather, and crossfades to the `audio.ambience.*` record for
##    that triple. Nothing is decided here: the snapshot is the only input.
## 2. **Cues** follow the battle events the bridge emits. `on_battle_event(event)`
##    takes one record from `NativeSimulationPort` exactly as it arrives and
##    plays the sound the content names for it -- the active skill's own cue when
##    the skill binds that event kind to a beat, and the event's own cue when it
##    does not.
## 3. **Music** is a four-state machine -- calm, notable, battle, urgent -- moved
##    only by explicit calls. It never reads the simulation and never decides
##    anything; B13's three levels and a fight are what the callers translate.
##
## ## The API other lanes call
##
## P6 (battle presentation) and the surface call exactly these, and nothing in
## `battle/`, `ui/` or `render/` is touched by this lane:
##
## - `Soundscape.apply(snapshot: Dictionary) -> String`
##       Chooses and crossfades the ambience for a snapshot. Returns the
##       `audio.ambience.*` ID it settled on, or `""` when the snapshot names no
##       region this content covers.
## - `Soundscape.on_battle_event(event: Dictionary) -> String`
##       One bridge event record: `kind`, `command_id`, `subjects`, `payload`.
##       Plays its cue and returns the `audio.cue.*` ID played, or `""` for an
##       event kind no record covers. Call it once per record, in order.
## - `Soundscape.set_music_state(state: String) -> bool`
##       `calm`, `notable`, `battle` or `urgent`. Returns false and changes
##       nothing when the name is not a state.
## - `Soundscape.set_bus_volume(bus: String, linear: float) -> bool` and
##   `Soundscape.bus_volume(bus: String) -> float`
##       The five buses by name. Linear 0..1, as a slider gives it.
## - `Soundscape.save_bus_volumes() -> Error`
##       Writes the five values back to `user://settings.cfg`.
##
## ## The settings keys P5 should put a slider on
##
## Volumes live in `user://settings.cfg` under section `audio`, one key per bus,
## lowercase: `audio/master`, `audio/music`, `audio/ambience`, `audio/effects`,
## `audio/voice`, each a linear float from 0.0 through 1.0. This autoload reads
## them at `_ready` and writes them in `save_bus_volumes`; it owns the keys and
## the bus layout. The settings panel is P5's this round, so it is not touched
## here: when P5 adds the five sliders it calls `set_bus_volume` on change and
## `save_bus_volumes` on close, and keeps no second copy of the values.
##
## ## What is a placeholder here
##
## Every sound is generated in-repo by `placeholder_synth.gd` from its record's
## own parameters. No audio file has been admitted; each record says so in its
## `asset` block and carries the empty licence, source and author fields that
## must be filled before one is. The validator counts them in its summary line.

const CATALOG_AMBIENCE_PREFIX := "audio.ambience."
const CATALOG_CUE_PREFIX := "audio.cue."
const CATALOG_BED_PREFIX := "audio.bed."
const CATALOG_MUSIC_PREFIX := "audio.music."

const SETTINGS_PATH := "user://settings.cfg"
const SETTINGS_SECTION := "audio"
## The bus layout's five buses, in the order `default_bus_layout.tres` declares
## them. The lowercase word is the settings key; the capitalised word is the bus.
const BUSES := {
	"master": "Master",
	"music": "Music",
	"ambience": "Ambience",
	"effects": "Effects",
	"voice": "Voice",
}
const DEFAULT_BUS_VOLUME := 0.8

## The music states, in the order they escalate. A name outside this list is
## refused rather than guessed at.
const MUSIC_STATES := ["calm", "notable", "battle", "urgent"]
const DEFAULT_MUSIC_STATE := "calm"

## Named crossfade durations, in milliseconds. A record names one of these; the
## number lives here so a change is made once and heard everywhere.
const CROSSFADE_MS := {
	"ambience_slow": 2400,
	"ambience_weather_break": 900,
	"music_settle": 1800,
	"music_cut": 450,
}

## The gain a layer sits at while it is not part of the current mix. Below the
## audible floor, so a silent bed costs nothing but is always ready to fade in.
const SILENT_DB := -60.0

## How many one-shot players the cue pool keeps. Enough that a busy exchange of
## blows never cuts its own earlier cue short; small enough to be free.
const CUE_VOICES := 12

## Weather the ambience chooser assumes when the snapshot does not carry any.
## S8 draws weather per region and `ExpeditionState::weather` holds it, but the
## bridge does not project it yet -- P3's card owns that hunk of
## `godot_bridge.rs`, and this lane touches no Rust. The day the key arrives as
## `snapshot.weather[region_id].condition`, this reads it with no change here.
const DEFAULT_WEATHER := "clear"
const DEFAULT_TIME_SEGMENT := "day"

var catalog: ContentCatalog
var bus_volumes: Dictionary = {}
var streams_by_id: Dictionary = {}
var bed_players: Dictionary = {}
var cue_players: Array[AudioStreamPlayer] = []
var cue_cursor := 0
var active_ambience_id := ""
var music_state := DEFAULT_MUSIC_STATE
var previous_music_state := ""
var music_applied := false
## `command_id` -> `skill_id`, remembered from `command_accepted` so a later
## `damage_applied` in the same command can find the skill whose beat it lands
## on. Cleared when a battle starts or ends.
var skill_by_command: Dictionary = {}
## Every layer's requested gain, in decibels, whether or not a tween has
## finished moving the player there yet. A test reads the intent; the ear hears
## the tween.
var layer_target_db: Dictionary = {}
var render_failures: Array[String] = []


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	catalog = ContentCatalog.new()
	if catalog.load_default() != OK:
		catalog = null
		return
	load_bus_volumes()
	apply_bus_volumes()
	render_all()
	build_players()
	set_music_state(DEFAULT_MUSIC_STATE)


# ---------------------------------------------------------------- the records

func ids_with_prefix(prefix: String) -> Array[String]:
	var empty: Array[String] = []
	if catalog == null:
		return empty
	return catalog.ids_with_prefix(prefix)


## Renders every bed and every cue once, here, at load. `render_failures` names
## any record the synth could not render; the suite asserts it stays empty.
func render_all() -> void:
	render_failures.clear()
	streams_by_id.clear()
	if catalog == null:
		return
	var renderable: Array[String] = []
	renderable.append_array(ids_with_prefix(CATALOG_BED_PREFIX))
	renderable.append_array(ids_with_prefix(CATALOG_CUE_PREFIX))
	for id in renderable:
		var record := catalog.get_record(id)
		var stream := PlaceholderSynth.render(record.get("voice", {}))
		if stream == null:
			render_failures.append(id)
			continue
		streams_by_id[id] = stream


## Which bus each bed belongs to is a fact about the records that mix it, not a
## second list: a bed named by a music record plays on Music, everything else on
## Ambience. The validator already refuses a bed nothing mixes.
func bed_bus_map() -> Dictionary:
	var buses: Dictionary = {}
	for id in ids_with_prefix(CATALOG_BED_PREFIX):
		buses[id] = "Ambience"
	for id in ids_with_prefix(CATALOG_MUSIC_PREFIX):
		for layer: Dictionary in catalog.get_record(id).get("layers", []):
			buses[str(layer.get("bedId", ""))] = "Music"
	return buses


func build_players() -> void:
	var buses := bed_bus_map()
	for id: String in streams_by_id:
		if not id.begins_with(CATALOG_BED_PREFIX):
			continue
		var player := AudioStreamPlayer.new()
		player.name = "Bed_%s" % id.replace(".", "_")
		player.stream = streams_by_id[id]
		player.volume_db = SILENT_DB
		player.bus = str(buses.get(id, "Ambience"))
		add_child(player)
		bed_players[id] = player
		layer_target_db[id] = SILENT_DB
	for index in CUE_VOICES:
		var player := AudioStreamPlayer.new()
		player.name = "Cue_%d" % index
		player.bus = "Effects"
		add_child(player)
		cue_players.append(player)


# --------------------------------------------------------------- the ambience

## The ambience record for one region, segment and weather condition. Returns
## `""` when no record covers the triple; the validator makes that unreachable
## for an authored region, and a caller that invents one gets silence, not a
## guess.
func ambience_id_for(region_id: String, time_segment: String, weather: String) -> String:
	if catalog == null or region_id.is_empty():
		return ""
	var key := "%s%s.%s.%s" % [CATALOG_AMBIENCE_PREFIX, region_id.trim_prefix("world.region."), time_segment, weather]
	return key if catalog.has(key) else ""


## The region an expedition snapshot is standing in, read through the cell
## record's own `regionId`. There is no second map from cell to region.
func region_of_snapshot(snapshot: Dictionary) -> String:
	if catalog == null:
		return ""
	var cell_id := str(snapshot.get("active_location_id", ""))
	if cell_id.is_empty() or not catalog.has(cell_id):
		return ""
	return str(catalog.get_record(cell_id).get("regionId", ""))


func weather_of_snapshot(snapshot: Dictionary, region_id: String) -> String:
	var weather: Variant = snapshot.get("weather", {})
	if weather is Dictionary and weather.has(region_id):
		var region_weather: Variant = weather[region_id]
		if region_weather is Dictionary:
			return str(region_weather.get("condition", DEFAULT_WEATHER))
	return DEFAULT_WEATHER


## Chooses and crossfades the ambience for a snapshot. Returns the record ID.
func apply(snapshot: Dictionary) -> String:
	var region_id := region_of_snapshot(snapshot)
	var segment := str(snapshot.get("time_segment", DEFAULT_TIME_SEGMENT))
	var weather := weather_of_snapshot(snapshot, region_id)
	var record_id := ambience_id_for(region_id, segment, weather)
	if record_id.is_empty() or record_id == active_ambience_id:
		return active_ambience_id if record_id.is_empty() else record_id
	var record := catalog.get_record(record_id)
	mix_layers(record, "Ambience")
	active_ambience_id = record_id
	return record_id


## Brings the record's beds to their authored gains and everything else on the
## same bus down to silence, over the record's named crossfade.
func mix_layers(record: Dictionary, bus: String) -> void:
	var milliseconds: int = CROSSFADE_MS.get(str(record.get("crossfade", "")), CROSSFADE_MS["ambience_slow"])
	var wanted: Dictionary = {}
	for layer: Dictionary in record.get("layers", []):
		wanted[str(layer.get("bedId", ""))] = float(layer.get("gainDb", SILENT_DB))
	for bed_id: String in bed_players:
		var player: AudioStreamPlayer = bed_players[bed_id]
		if player.bus != bus:
			continue
		var target: float = wanted.get(bed_id, SILENT_DB)
		if wanted.has(bed_id) and not player.playing:
			player.play()
		fade_layer(bed_id, target, milliseconds)


func fade_layer(bed_id: String, target_db: float, milliseconds: int) -> void:
	layer_target_db[bed_id] = target_db
	var player: AudioStreamPlayer = bed_players.get(bed_id, null)
	if player == null:
		return
	if not is_inside_tree() or milliseconds <= 0:
		player.volume_db = target_db
		return
	var tween := create_tween()
	tween.tween_property(player, "volume_db", target_db, float(milliseconds) / 1000.0)


# ------------------------------------------------------------------- the cues

## The cue a battle event resolves to, without playing it. The skill's own beat
## wins when the skill binds this event kind; the event's own cue answers when
## it does not. Returns `""` for a kind no record covers.
func cue_id_for_event(event: Dictionary) -> String:
	if catalog == null:
		return ""
	var kind := str(event.get("kind", ""))
	if kind.is_empty():
		return ""
	var skill_id := skill_of_event(event)
	if not skill_id.is_empty() and catalog.has(skill_id):
		var animation: Dictionary = catalog.get_record(skill_id).get("animation", {})
		var bindings: Dictionary = animation.get("eventBindings", {})
		if bindings.has(kind):
			var beat := str(bindings[kind])
			for cue: Dictionary in animation.get("presentationCues", []):
				if str(cue.get("beat", "")) == beat:
					return str(cue.get("audioCueId", ""))
	var event_cue := "%sevent.%s" % [CATALOG_CUE_PREFIX, kind]
	return event_cue if catalog.has(event_cue) else ""


## Which skill an event belongs to. Events that carry `skill_id` say so; the
## rest are matched to the command that was accepted for them.
func skill_of_event(event: Dictionary) -> String:
	var payload: Dictionary = event.get("payload", {})
	var command_id := str(event.get("command_id", ""))
	var skill_id := str(payload.get("skill_id", ""))
	if not skill_id.is_empty():
		if not command_id.is_empty():
			skill_by_command[command_id] = skill_id
		return skill_id
	return str(skill_by_command.get(command_id, ""))


## Plays one bridge event record. Returns the cue ID played.
func on_battle_event(event: Dictionary) -> String:
	var kind := str(event.get("kind", ""))
	if kind == "battle_started" or kind == "battle_ended":
		skill_by_command.clear()
	var cue_id := cue_id_for_event(event)
	if cue_id.is_empty():
		return ""
	play_cue(cue_id)
	return cue_id


func play_cue(cue_id: String) -> bool:
	if not streams_by_id.has(cue_id) or cue_players.is_empty():
		return false
	var player := cue_players[cue_cursor]
	cue_cursor = (cue_cursor + 1) % cue_players.size()
	player.stream = streams_by_id[cue_id]
	player.volume_db = 0.0
	player.play()
	return true


# ------------------------------------------------------------------ the music

## Moves the music machine. The four states are named, never numbered, and a
## name outside the four changes nothing.
func set_music_state(state: String) -> bool:
	if not MUSIC_STATES.has(state):
		return false
	if state == music_state and music_applied:
		return true
	previous_music_state = music_state
	music_state = state
	music_applied = true
	if catalog == null:
		return true
	var record_id := "%s%s" % [CATALOG_MUSIC_PREFIX, state]
	if not catalog.has(record_id):
		return false
	mix_layers(catalog.get_record(record_id), "Music")
	return true


# ------------------------------------------------------------------- the buses

func load_bus_volumes() -> void:
	var config := ConfigFile.new()
	var loaded := config.load(SETTINGS_PATH) == OK
	for key: String in BUSES:
		var stored: Variant = config.get_value(SETTINGS_SECTION, key, DEFAULT_BUS_VOLUME) if loaded else DEFAULT_BUS_VOLUME
		bus_volumes[key] = clampf(float(stored), 0.0, 1.0)


func apply_bus_volumes() -> void:
	for key: String in BUSES:
		apply_bus_volume(key)


func apply_bus_volume(key: String) -> void:
	var index := AudioServer.get_bus_index(BUSES[key])
	if index < 0:
		return
	var linear: float = bus_volumes.get(key, DEFAULT_BUS_VOLUME)
	AudioServer.set_bus_volume_db(index, SILENT_DB if linear <= 0.0 else linear_to_db(linear))
	AudioServer.set_bus_mute(index, linear <= 0.0)


## Sets one bus from a linear 0..1 value, as a slider gives it. Returns false
## for a name that is not one of the five buses.
func set_bus_volume(bus: String, linear: float) -> bool:
	var key := bus.to_lower()
	if not BUSES.has(key):
		return false
	bus_volumes[key] = clampf(linear, 0.0, 1.0)
	apply_bus_volume(key)
	return true


func bus_volume(bus: String) -> float:
	return float(bus_volumes.get(bus.to_lower(), DEFAULT_BUS_VOLUME))


func save_bus_volumes() -> Error:
	var config := ConfigFile.new()
	config.load(SETTINGS_PATH)
	for key: String in BUSES:
		config.set_value(SETTINGS_SECTION, key, bus_volumes[key])
	return config.save(SETTINGS_PATH)
