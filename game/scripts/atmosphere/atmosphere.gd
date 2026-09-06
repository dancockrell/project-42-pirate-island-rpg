extends Node

## Autoload `Atmosphere`. The island's weather, read from the simulation and
## from nothing else.
##
## Brief section 13 and card P3: the sky is evidence. What time it is, what the
## weather is doing over this region, how far corruption has taken this cell
## and which of four rooms hidden pressure has the world in -- all four are
## facts the simulation already owns, and all four arrive here on the snapshot.
## Nothing in this file reads a wall clock, calls `randi()`, or decides a
## number of its own: [method resolve] is a pure function of a snapshot and
## `content/atmosphere/`, and [method apply] is that function plus a tween.
##
## ## Where the numbers live
##
## `content/atmosphere/` owns every one of them -- the segment table, the
## weather table, the corruption palette, the heat shift and the named
## transition durations. Rust owns the *names* those tables are keyed by, and
## `strategy::dungeon`'s `authored_atmosphere` tests hold the two equal, so a
## band that exists in the simulation and not in the palette is a red build
## rather than an ungraded island.
##
## ## Where the values go
##
## Through [AtmosphereTarget], and only through it. Lane P2's `WorldEnvironment`
## script implements the four methods and is preferred automatically;
## [EnvironmentAtmosphereTarget] drives Godot's own `Environment` and
## `DirectionalLight3D` for every scene that has no P2 node yet.
##
## ## What it does not do
##
## It decides no game result, writes nothing back to the bridge, and shows no
## number. `cthulhu_heat` never reaches Godot at all -- only its band's name --
## so there is nothing here that could be turned into a pressure bar.

## The authored records this autoload reads. Named once; a missing one is a
## refusal with the ID in it, never a default quietly substituted.
const SEGMENT_TABLE_ID := "atmosphere.segment_table"
const WEATHER_TABLE_ID := "atmosphere.weather_table"
const CORRUPTION_PALETTE_ID := "atmosphere.corruption_palette"
const HEAT_SHIFT_ID := "atmosphere.heat_shift"
const TRANSITIONS_ID := "atmosphere.transitions"

## The four time segments in the order the day runs them, so "the next segment"
## is the next entry and midnight's next is dawn. `ExpeditionState::advance_time`
## owns this order; this is its presentation-side reading of the same ring.
const SEGMENT_ORDER := ["dawn", "day", "dusk", "midnight"]

## The accessibility settings this autoload honours live where B9's settings
## panel put them. Read, never written, and never copied into a second file.
const SETTINGS_PATH := SettingsPanel.SETTINGS_PATH
const SETTINGS_SECTION := SettingsPanel.SECTION

## Emitted after every applied snapshot, carrying the resolved values. A
## capture, a suite or an audio lane (P9) reads the sky here rather than
## recomputing it.
signal atmosphere_applied(resolved: Dictionary)

## Read-only index over the generated bundle. Left null until something asks,
## exactly as `ContentPackRegistry` leaves its own; a caller that already has a
## catalog assigns it rather than making a second one.
var catalog: ContentCatalog = null

## Where the sky is written. Left null to let [method apply] find one: the
## first node in the running scene that implements the contract, else a fresh
## [EnvironmentAtmosphereTarget] bound to that scene.
var target: Object = null

## The node that answers "where on the board is this building?" -- lane P4's
## board when it exists. Any object with a
## `building_anchor(building_id: String) -> Vector3` method will do. Until one
## is attached the night lamps are still resolved and reported, and simply
## nothing is spawned; see `content/atmosphere/transitions.json`.
var board_anchor_provider: Object = null

## The last resolved values, so a target that arrives late can be caught up.
var last_resolved: Dictionary = {}

var _tables: Dictionary = {}
var _refusal := ""
var _current: Dictionary = {}
var _tweens: Dictionary = {}


func _ready() -> void:
	# The sky keeps travelling while a reading surface holds the game, because
	# B9's pause is about the simulation and a frozen tween mid-dusk would read
	# as a bug. Nothing here advances the simulation, so this is safe by
	# construction rather than by convention.
	process_mode = Node.PROCESS_MODE_ALWAYS


## Loads the five authored tables. Returns OK, or the error, with
## [method refusal] carrying the reason. Called automatically on first use.
func load_tables() -> Error:
	if not _tables.is_empty():
		return OK
	if catalog == null:
		catalog = ContentCatalog.new()
		var loaded := catalog.load_default()
		if loaded != OK:
			catalog = null
			_refusal = "atmosphere_content_bundle_unavailable"
			return loaded
	var wanted := {
		"segments": SEGMENT_TABLE_ID,
		"weather": WEATHER_TABLE_ID,
		"corruption": CORRUPTION_PALETTE_ID,
		"heat": HEAT_SHIFT_ID,
		"transitions": TRANSITIONS_ID,
	}
	var next: Dictionary = {}
	for key in wanted:
		var id: String = wanted[key]
		if not catalog.has(id):
			_refusal = "atmosphere_record_missing:%s" % id
			return ERR_FILE_NOT_FOUND
		next[key] = catalog.get_record(id)
	_tables = next
	_refusal = ""
	return OK


## Why the last call refused, or "" when it did not.
func refusal() -> String:
	return _refusal


## The whole of the atmosphere, as named values, from one snapshot. Pure: the
## same snapshot gives the same sky on every machine, and calling this a
## hundred times changes nothing.
func resolve(snapshot: Dictionary) -> Dictionary:
	if load_tables() != OK:
		return {}
	var segment := str(snapshot.get("time_segment", "dawn"))
	if not SEGMENT_ORDER.has(segment):
		segment = "dawn"
	var hour := int(snapshot.get("hour_of_day", 0))
	var weather_kind := _weather_kind(snapshot)
	var corruption_band := _corruption_band(snapshot)
	var heat_band := str(snapshot.get("heat_band", "dormant"))
	var heat_bands: Dictionary = _tables.heat.get("bands", {})
	if not heat_bands.has(heat_band):
		heat_band = "dormant"

	var sun := _sun_for(segment, hour)
	var weather: Dictionary = _tables.weather.get("conditions", {}).get(weather_kind, {})
	var grade_row: Dictionary = _tables.corruption.get("bands", {}).get(corruption_band, {})
	var heat_row: Dictionary = heat_bands.get(heat_band, {})

	sun["energy"] = float(sun["energy"]) * float(weather.get("sunEnergyScale", 1.0))
	sun["exposure"] = float(sun["exposure"]) * float(weather.get("exposureScale", 1.0))
	sun["ambient_energy"] = (
		float(sun["ambient_energy"])
		* float(weather.get("ambientEnergyScale", 1.0))
		* float(heat_row.get("ambientEnergyScale", 1.0))
	)

	var reduced := reduced_motion()
	var night := bool(snapshot.get("is_night", segment == "dusk" or segment == "midnight"))
	var resolved := {
		"segment": segment,
		"hour_of_day": hour,
		"weather_kind": weather_kind,
		"corruption_band": corruption_band,
		"heat_band": heat_band,
		"reduced_motion": reduced,
		"is_night": night,
		"sun": sun,
		"fog":
		{
			"density": float(weather.get("fogDensity", 0.0)),
			"colour": _colour(weather.get("fogColour", "#ffffff")),
			"glow_intensity": float(weather.get("glowIntensity", 0.3)),
			"sky_affect": float(weather.get("fogSkyAffect", 0.5)),
		},
		"wind":
		{
			"strength": float(weather.get("windStrength", 0.0)),
			"particle_kind": str(weather.get("particleKind", "none")),
			"particle_intensity": float(weather.get("particleIntensity", 0.0)),
			"stilled": reduced,
		},
		"grade":
		{
			"palette_name": str(grade_row.get("paletteName", "clean_light")),
			"saturation": float(grade_row.get("saturation", 1.0)),
			"tint_colour": _colour(grade_row.get("tintColour", "#ffffff")),
			"tint_strength": float(grade_row.get("tintStrength", 0.0)),
			"contrast": float(grade_row.get("contrast", 1.0)),
			"heat_shift_name": str(heat_row.get("shiftName", "settled")),
			"sky_hue_shift_degrees": float(heat_row.get("skyHueShiftDegrees", 0.0)),
			"horizon_desaturation": float(heat_row.get("horizonDesaturation", 0.0)),
		},
		"night_lamps": _night_lamps(snapshot, night),
		"night_lamp": _night_lamp(),
		"durations": durations(),
	}
	return resolved


## Resolves `snapshot` and travels to it: every group tweened over its own
## named duration, from wherever the sky currently stands. Returns the resolved
## values, or an empty dictionary when the content or the scene refused.
func apply(snapshot: Dictionary) -> Dictionary:
	var resolved := resolve(snapshot)
	if resolved.is_empty():
		return {}
	if not _target_is_live():
		target = _find_target()
	if target == null:
		_refusal = "atmosphere_no_target_in_scene"
		return {}
	last_resolved = resolved
	var durations_by_group: Dictionary = resolved.durations
	# The grade carries two travellers at two speeds: corruption's tint moves
	# with the grade, and the sky shift hidden pressure makes moves at the heat
	# duration, which `content/atmosphere/transitions.json` makes the slowest
	# on purpose. Whichever is slower governs, so the shift is never dragged
	# along faster than it was authored to move.
	var grade_duration: float = maxf(
		float(durations_by_group.get("grade", 0.0)), float(durations_by_group.get("heat", 0.0))
	)
	_travel("sun", resolved.sun, float(durations_by_group.get("sun", 0.0)))
	_travel("fog", resolved.fog, float(durations_by_group.get("fog", 0.0)))
	_travel("wind", resolved.wind, float(durations_by_group.get("wind", 0.0)))
	_travel("grade", resolved.grade, grade_duration)
	if target.has_method(AtmosphereTarget.NIGHT_LAMP_METHOD):
		target.call(AtmosphereTarget.NIGHT_LAMP_METHOD, resolved.night_lamps, resolved.night_lamp)
	atmosphere_applied.emit(resolved)
	return resolved


## Resolves and writes `snapshot` with no travel at all: every value arrives
## this frame. What a capture and a suite want, and what the first snapshot of
## a scene wants -- there is nowhere to tween from when the sky has never been
## set.
func apply_immediately(snapshot: Dictionary) -> Dictionary:
	_current.clear()
	_cancel_tweens()
	var resolved := resolve(snapshot)
	if resolved.is_empty():
		return {}
	if not _target_is_live():
		target = _find_target()
	if target == null:
		_refusal = "atmosphere_no_target_in_scene"
		return {}
	last_resolved = resolved
	for group in ["sun", "fog", "wind", "grade"]:
		_current[group] = resolved[group].duplicate(true)
		_write(group)
	if target.has_method(AtmosphereTarget.NIGHT_LAMP_METHOD):
		target.call(AtmosphereTarget.NIGHT_LAMP_METHOD, resolved.night_lamps, resolved.night_lamp)
	atmosphere_applied.emit(resolved)
	return resolved


## The named transition durations, already scaled by reduced motion. Reduced
## motion shortens; it never removes, because a value that snapped would be a
## different bug from the one B9 is preventing.
func durations() -> Dictionary:
	if load_tables() != OK:
		return {}
	var authored: Dictionary = _tables.transitions.get("durationsSeconds", {})
	var scale := 1.0
	if reduced_motion():
		scale = float(_tables.transitions.get("reducedMotion", {}).get("durationScale", 1.0))
	var scaled: Dictionary = {}
	for key in authored:
		scaled[key] = float(authored[key]) * scale
	return scaled


## B9's reduced-motion setting, read from the file its panel writes. False when
## no settings file exists yet, which is the default the panel itself carries.
func reduced_motion() -> bool:
	var config := ConfigFile.new()
	if config.load(SETTINGS_PATH) != OK:
		return false
	return bool(config.get_value(SETTINGS_SECTION, "reduced_motion", false))


## True when `node` answers every method [AtmosphereTarget] requires. This is
## the whole of the contract check: lane P2's `WorldEnvironment` script cannot
## inherit the class, so it is recognised by what it can do.
func is_target(node: Object) -> bool:
	if node == null:
		return false
	for method in AtmosphereTarget.REQUIRED_METHODS:
		if not node.has_method(method):
			return false
	return true


## Whether the target the autoload is holding can still be written to. A scene
## swap frees the nodes a target was bound to, and writing into the freed
## scene's `Environment` is how a capture silently renders the sky before last:
## a target that is no longer live is dropped and found again.
func _target_is_live() -> bool:
	if target == null:
		return false
	if target.has_method("is_bound"):
		return bool(target.call("is_bound"))
	var node := target as Node
	return node != null and is_instance_valid(node) and node.is_inside_tree()


func _find_target() -> Object:
	var tree := get_tree()
	if tree == null:
		return null
	# A `--script` main loop has no current scene: a capture or a suite adds
	# the review scene straight to the root, so the first Node3D under the
	# root is the scene.
	var scene: Node = tree.current_scene
	if scene == null:
		for child in tree.root.get_children():
			if child != self and child is Node3D:
				scene = child
				break
	if scene == null:
		return null
	for node in _descendants(scene):
		if node == self:
			continue
		if is_target(node):
			return node
	var fallback := EnvironmentAtmosphereTarget.new()
	if not fallback.bind_to(scene):
		return null
	return fallback


func _descendants(root: Node) -> Array[Node]:
	var found: Array[Node] = [root]
	for child in root.get_children():
		found.append_array(_descendants(child))
	return found


## Tweens one group from where it stands to `to` over `duration`, and writes
## every step through the target. The tween is the only thing in this file that
## takes time, and it takes it in frames rather than by reading a clock.
func _travel(group: String, to: Dictionary, duration: float) -> void:
	if not _current.has(group) or duration <= 0.0:
		_current[group] = to.duplicate(true)
		_write(group)
		return
	var from: Dictionary = _current[group].duplicate(true)
	if _tweens.has(group) and is_instance_valid(_tweens[group]):
		_tweens[group].kill()
	var tween := create_tween()
	tween.tween_method(
		func(weight: float) -> void:
			_current[group] = _blend(from, to, weight)
			_write(group),
		0.0,
		1.0,
		duration
	)
	_tweens[group] = tween


func _cancel_tweens() -> void:
	for group in _tweens.keys():
		if is_instance_valid(_tweens[group]):
			_tweens[group].kill()
	_tweens.clear()


func _write(group: String) -> void:
	if target == null:
		return
	match group:
		"sun":
			target.set_sun(_current[group])
		"fog":
			target.set_fog(_current[group])
		"wind":
			target.set_wind(_current[group])
		"grade":
			target.set_grade(_current[group])


## Interpolates two resolved groups. Floats and Colors travel; names, kinds and
## flags take the destination's value at the halfway mark, because "half of
## drowned_violet" is not a colour grade and a particle kind cannot be a
## fraction.
func _blend(from: Dictionary, to: Dictionary, weight: float) -> Dictionary:
	var blended: Dictionary = {}
	for key in to:
		var target_value: Variant = to[key]
		var source_value: Variant = from.get(key, target_value)
		if target_value is float and source_value is float:
			if key == "azimuth_degrees":
				blended[key] = rad_to_deg(
					lerp_angle(deg_to_rad(source_value), deg_to_rad(target_value), weight)
				)
			else:
				blended[key] = lerpf(source_value, target_value, weight)
		elif target_value is Color and source_value is Color:
			blended[key] = source_value.lerp(target_value, weight)
		else:
			blended[key] = target_value if weight >= 0.5 else source_value
	return blended


## The sun for one segment, refined by the hour inside it. The segment supplies
## the row; the hour supplies how far this segment has travelled toward the
## next one, so the sun crawls across the sky instead of snapping four times a
## day. Both come from the snapshot -- `time_segment` and `hour_of_day` -- and
## nothing here reads a host clock.
func _sun_for(segment: String, hour: int) -> Dictionary:
	var segments: Dictionary = _tables.segments.get("segments", {})
	var hours_per_segment := maxi(
		1, int(_tables.segments.get("hourFraction", {}).get("hoursPerSegment", 6))
	)
	var index := SEGMENT_ORDER.find(segment)
	var next_segment: String = SEGMENT_ORDER[(index + 1) % SEGMENT_ORDER.size()]
	var here: Dictionary = segments.get(segment, {})
	var there: Dictionary = segments.get(next_segment, {})
	var fraction := clampf(
		float(posmod(hour, hours_per_segment)) / float(hours_per_segment), 0.0, 1.0
	)
	var kelvin := lerpf(
		float(here.get("sunColourTemperatureKelvin", 6000.0)),
		float(there.get("sunColourTemperatureKelvin", 6000.0)),
		fraction
	)
	return {
		"elevation_degrees":
		lerpf(
			float(here.get("sunElevationDegrees", 45.0)),
			float(there.get("sunElevationDegrees", 45.0)),
			fraction
		),
		"azimuth_degrees":
		rad_to_deg(
			lerp_angle(
				deg_to_rad(float(here.get("sunAzimuthDegrees", 180.0))),
				deg_to_rad(float(there.get("sunAzimuthDegrees", 180.0))),
				fraction
			)
		),
		"colour_temperature_kelvin": kelvin,
		"colour": kelvin_to_colour(kelvin),
		"energy": lerpf(float(here.get("sunEnergy", 1.0)), float(there.get("sunEnergy", 1.0)), fraction),
		"ambient_energy":
		lerpf(
			float(here.get("ambientEnergy", 0.5)), float(there.get("ambientEnergy", 0.5)), fraction
		),
		"ambient_colour":
		_colour(here.get("ambientColour", "#ffffff")).lerp(
			_colour(there.get("ambientColour", "#ffffff")), fraction
		),
		"sky_horizon_colour":
		_colour(here.get("skyHorizonColour", "#000000")).lerp(
			_colour(there.get("skyHorizonColour", "#000000")), fraction
		),
		"sky_top_colour":
		_colour(here.get("skyTopColour", "#000000")).lerp(
			_colour(there.get("skyTopColour", "#000000")), fraction
		),
		"exposure": lerpf(float(here.get("exposure", 1.0)), float(there.get("exposure", 1.0)), fraction),
	}


## Black-body colour for a temperature in kelvin, as one function rather than a
## second authored hex beside every kelvin in the segment table. The
## approximation is Tanner Helland's, clamped to the range the table uses; it
## is deterministic and reads correctly warm at 2000 K and correctly cold at
## 9000 K, which is the whole of what the sky needs from it.
func kelvin_to_colour(kelvin: float) -> Color:
	var temperature := clampf(kelvin, 1000.0, 40000.0) / 100.0
	var red := 255.0
	var green := 255.0
	var blue := 255.0
	if temperature <= 66.0:
		green = 99.4708025861 * log(temperature) - 161.1195681661
		if temperature <= 19.0:
			blue = 0.0
		else:
			blue = 138.5177312231 * log(temperature - 10.0) - 305.0447927307
	else:
		red = 329.698727446 * pow(temperature - 60.0, -0.1332047592)
		green = 288.1221695283 * pow(temperature - 60.0, -0.0755148492)
	return Color(
		clampf(red, 0.0, 255.0) / 255.0,
		clampf(green, 0.0, 255.0) / 255.0,
		clampf(blue, 0.0, 255.0) / 255.0
	)


## The weather over the region the party is standing in. The snapshot names the
## region (`active_region_id`), so Godot never has to guess which of several
## skies applies; an unknown region reads clear rather than inventing a storm.
func _weather_kind(snapshot: Dictionary) -> String:
	var conditions: Dictionary = _tables.weather.get("conditions", {})
	var region := str(snapshot.get("active_region_id", ""))
	var weather: Dictionary = snapshot.get("weather", {})
	var kind := str(weather.get(region, "clear"))
	return kind if conditions.has(kind) else "clear"


## The corruption band at the cell the party is standing in. `corruption` maps
## a cell to S9's band name and leaves a clean cell out entirely, so an absent
## cell is untouched.
func _corruption_band(snapshot: Dictionary) -> String:
	var bands: Dictionary = _tables.corruption.get("bands", {})
	var cell := str(snapshot.get("active_location_id", ""))
	var corruption: Dictionary = snapshot.get("corruption", {})
	var band := str(corruption.get(cell, "untouched"))
	return band if bands.has(band) else "untouched"


## The lamps the night wants: one per building the snapshot says the island is
## holding, placed by the board when a board is attached. Resolved even with no
## board, so a suite can assert what the night would light.
func _night_lamps(snapshot: Dictionary, night: bool) -> Array:
	if not night:
		return []
	var lamps: Array = []
	for building_id in snapshot.get("buildings", []):
		var id := str(building_id)
		if id.is_empty():
			continue
		var position := Vector3.ZERO
		var placed := false
		if board_anchor_provider != null and board_anchor_provider.has_method("building_anchor"):
			var anchor: Variant = board_anchor_provider.call("building_anchor", id)
			if anchor is Vector3:
				position = anchor
				placed = true
		lamps.append({"building_id": id, "position": position, "placed": placed})
	return lamps


func _night_lamp() -> Dictionary:
	var authored: Dictionary = _tables.transitions.get("nightPointLight", {})
	return {
		"colour": kelvin_to_colour(float(authored.get("colourTemperatureKelvin", 2100.0))),
		"energy": float(authored.get("energy", 2.0)),
		"range_metres": float(authored.get("rangeMetres", 12.0)),
		"height_metres": float(authored.get("heightMetres", 3.0)),
		"attenuation": float(authored.get("attenuation", 1.5)),
	}


func _colour(value: Variant) -> Color:
	var text := str(value)
	# not a colour: white when a record authors no colour at all.
	return Color(text) if text.begins_with("#") else Color.WHITE
