extends SceneTree

## P3: the sky is the simulation's.
##
## Four authored snapshots -- a clear dawn, a corrupted overcast dusk, a held
## midnight, a storm over a consumed cell -- are resolved through the
## `Atmosphere` autoload and asserted against the values
## `content/atmosphere/` authored for them. Nothing here reads a clock, and the
## suite would pass identically at any hour of any day on any machine, which is
## the property the card is actually about.
##
## What it proves, in the card's order: a snapshot maps to the named values;
## a corrupted cell tints and a clean one does not; reduced motion stills the
## particles and shortens every transition; night lights the buildings the
## island is holding; and no number that the bridge refuses to project can be
## read back out of the sky.
##
## The bridge is not needed and not used: `Atmosphere.resolve` is a pure
## function of a snapshot dictionary, so this suite proves the mapping without
## an engine-side campaign. The live snapshot's own six keys are proved by
## `godot-rust` (`strategy::dungeon::authored_atmosphere`) and by
## `cargo check --features godot-ext`.

var failures := 0
var atmosphere: Node


func _init() -> void:
	# Autoloads are added to the root after a `--script` main loop is
	# instantiated, so `_init` runs before Atmosphere exists. One frame is the
	# whole difference.
	await process_frame
	atmosphere = root.get_node_or_null("Atmosphere")
	check(atmosphere != null, "Atmosphere must be registered as an autoload")
	if atmosphere == null:
		finish()
		return
	# The suite writes B9's settings file, so it must leave it as it found it.
	var restore := saved_reduced_motion()
	set_reduced_motion(false)

	check(atmosphere.load_tables() == OK, "the five authored atmosphere tables must load: %s" % atmosphere.refusal())

	test_dawn()
	test_dusk()
	test_night()
	test_storm()
	test_a_corrupted_cell_tints_and_a_clean_one_does_not()
	test_reduced_motion_stills_the_weather_and_shortens_the_travel()
	test_the_hour_moves_the_sun_inside_its_segment()
	test_nothing_hidden_crosses()

	set_reduced_motion(restore)
	finish()


## Dawn: warm, low, clear and ungraded. The reference sky.
func test_dawn() -> void:
	var sky: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("dawn"))
	check(not sky.is_empty(), "the dawn snapshot must resolve")
	if sky.is_empty():
		return
	check(sky.segment == "dawn", "dawn's segment must be dawn")
	check(sky.weather_kind == "clear", "dawn's authored weather is clear")
	check(sky.corruption_band == "untouched", "dawn stands on clean ground")
	check(sky.grade.palette_name == "clean_light", "clean ground takes the clean_light grade, not %s" % sky.grade.palette_name)
	check(is_equal_approx(sky.grade.saturation, 1.0), "an untouched cell must not desaturate")
	check(is_equal_approx(sky.grade.sky_hue_shift_degrees, 0.0), "a dormant world must not shift the sky")
	check(sky.grade.heat_shift_name == "settled", "a dormant world's shift is named settled")
	check(sky.sun.elevation_degrees < 30.0, "dawn's sun must be low, not %.1f degrees" % sky.sun.elevation_degrees)
	check(sky.sun.colour_temperature_kelvin < 4000.0, "dawn's light must be warm, not %.0f K" % sky.sun.colour_temperature_kelvin)
	check(sky.sun.colour.r > sky.sun.colour.b, "warm light is redder than it is blue")
	check(sky.wind.particle_kind == "none", "a clear dawn draws no particles")
	check(not sky.is_night, "dawn is not night")
	check(sky.night_lamps.is_empty(), "no lamps burn by day")


## Dusk: the light going, the sky flat, the ground beginning to turn.
func test_dusk() -> void:
	var sky: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("dusk"))
	check(sky.segment == "dusk", "dusk's segment must be dusk")
	check(sky.weather_kind == "overcast", "dusk's authored weather is overcast")
	check(sky.corruption_band == "touched", "dusk's terrace is touched")
	check(sky.grade.palette_name == "verdigris_creep", "a touched cell takes the verdigris_creep grade")
	check(sky.grade.saturation < 1.0, "a touched cell must desaturate")
	check(sky.grade.heat_shift_name == "listing", "a stirring world's shift is named listing")
	check(sky.grade.sky_hue_shift_degrees < 0.0, "a stirring world must shift the sky")
	check(sky.is_night, "dusk is night for the island's purposes, as habitat::is_night says")
	# Overcast is thicker than clear and still lighter than a storm: the axis
	# the weather table authored, read back through the resolver.
	var clear: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("dawn"))
	var storm: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("storm"))
	check(sky.fog.density > clear.fog.density, "overcast must be thicker than clear")
	check(sky.fog.density < storm.fog.density, "overcast must be thinner than a storm")
	check(sky.sun.energy < clear.sun.energy, "an overcast sun must be dimmer than a clear one")


## Night: the moon nearly out, and every warm thing a lamp at a held building.
func test_night() -> void:
	var snapshot := AtmosphereReviewSnapshots.of("night")
	var sky: Dictionary = atmosphere.resolve(snapshot)
	check(sky.segment == "midnight", "night's segment must be midnight")
	check(sky.is_night, "midnight is night")
	var dawn: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("dawn"))
	check(sky.sun.ambient_energy < dawn.sun.ambient_energy * 0.5, "night's ambient must fall well below dawn's")
	check(sky.sun.colour_temperature_kelvin > 6000.0, "the moon is cold light, not %.0f K" % sky.sun.colour_temperature_kelvin)
	check(sky.night_lamps.size() == snapshot.buildings.size(), "one lamp per held building: expected %d, resolved %d" % [snapshot.buildings.size(), sky.night_lamps.size()])
	var lit: Array = []
	for lamp in sky.night_lamps:
		lit.append(str(lamp.building_id))
		check(not bool(lamp.placed), "with no board attached a lamp must report itself unplaced rather than pretending to a position")
	check(lit == snapshot.buildings, "the lamps must be the snapshot's buildings, in the snapshot's order")
	check(sky.night_lamp.colour.r > sky.night_lamp.colour.b, "a lamp at a held building is warm")
	# The board hook, exercised: a provider that answers puts the lamp
	# somewhere, and that is the only thing that ever does.
	atmosphere.board_anchor_provider = StubBoard.new()
	var placed: Dictionary = atmosphere.resolve(snapshot)
	check(bool(placed.night_lamps[0].placed), "a board that answers must place the lamp")
	check(placed.night_lamps[0].position == StubBoard.ANCHOR, "the lamp must stand where the board put it")
	atmosphere.board_anchor_provider = null


## Storm: daylight that is no help, over a cell corruption has finished with.
func test_storm() -> void:
	var sky: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("storm"))
	check(sky.segment == "day", "the storm's segment is day")
	check(sky.weather_kind == "storm", "the storm's authored weather is storm")
	check(sky.wind.particle_kind == "rain", "a storm draws rain")
	check(is_equal_approx(sky.wind.particle_intensity, 1.0), "a storm draws rain at full intensity")
	check(sky.wind.strength >= 1.0, "a storm blows hardest of the five conditions")
	check(sky.corruption_band == "consumed", "the storm stands on a consumed cell")
	check(sky.grade.palette_name == "drowned_violet", "a consumed cell takes the drowned_violet grade")
	check(sky.grade.heat_shift_name == "capsized", "an imminent world's shift is named capsized")
	var dawn: Dictionary = atmosphere.resolve(AtmosphereReviewSnapshots.of("dawn"))
	check(sky.sun.energy < dawn.sun.energy, "a storm at noon must be darker than a clear dawn")


## The card's own clause: a corrupted cell tints, and the tint is the band's.
func test_a_corrupted_cell_tints_and_a_clean_one_does_not() -> void:
	var clean := AtmosphereReviewSnapshots.of("dawn")
	var corrupted := clean.duplicate(true)
	corrupted.corruption = {AtmosphereReviewSnapshots.CELL: "spreading"}
	var before: Dictionary = atmosphere.resolve(clean)
	var after: Dictionary = atmosphere.resolve(corrupted)
	check(is_equal_approx(before.grade.tint_strength, 0.0), "a clean cell must not tint at all")
	check(after.grade.tint_strength > 0.0, "a corrupted cell must tint")
	check(after.grade.saturation < before.grade.saturation, "a corrupted cell must desaturate")
	check(after.grade.palette_name == "ashen_bruise", "a spreading cell takes the ashen_bruise grade")
	check(after.grade.tint_colour != before.grade.tint_colour, "the tint must be the band's colour, not the clean one's")
	# Only the cell the party stands in grades the sky. A corrupted cell
	# somewhere else on the island is somewhere else.
	var elsewhere := clean.duplicate(true)
	elsewhere.corruption = {"world.cell.black_beach": "consumed"}
	var unaffected: Dictionary = atmosphere.resolve(elsewhere)
	check(unaffected.corruption_band == "untouched", "corruption in another cell must not grade this one")


## B9's reduced motion: the weather still says what it says, and stops moving.
func test_reduced_motion_stills_the_weather_and_shortens_the_travel() -> void:
	var storm := AtmosphereReviewSnapshots.of("storm")
	set_reduced_motion(false)
	var moving: Dictionary = atmosphere.resolve(storm)
	var moving_durations: Dictionary = atmosphere.durations()
	set_reduced_motion(true)
	var stilled: Dictionary = atmosphere.resolve(storm)
	var stilled_durations: Dictionary = atmosphere.durations()
	check(not bool(moving.wind.stilled), "with motion allowed the weather moves")
	check(bool(stilled.wind.stilled), "reduced motion must still the weather")
	check(bool(stilled.reduced_motion), "the resolved sky must say reduced motion was honoured")
	check(is_equal_approx(stilled.wind.particle_intensity, moving.wind.particle_intensity), "reduced motion stills the rain; it does not delete it")
	check(stilled.wind.particle_kind == moving.wind.particle_kind, "reduced motion must not change what the sky is doing")
	for key in moving_durations:
		check(stilled_durations[key] < moving_durations[key], "reduced motion must shorten the %s transition" % key)
		check(stilled_durations[key] > 0.0, "reduced motion must shorten a transition, never remove it")
	set_reduced_motion(false)


## The hour refines the segment rather than replacing it: the sun crawls.
func test_the_hour_moves_the_sun_inside_its_segment() -> void:
	var first := AtmosphereReviewSnapshots.of("dawn")
	first.hour_of_day = 0
	var late := AtmosphereReviewSnapshots.of("dawn")
	late.hour_of_day = 5
	var early_sky: Dictionary = atmosphere.resolve(first)
	var late_sky: Dictionary = atmosphere.resolve(late)
	check(early_sky.segment == late_sky.segment, "both hours are still dawn")
	check(late_sky.sun.elevation_degrees > early_sky.sun.elevation_degrees, "the sun must climb across the segment")
	check(late_sky.sun.colour_temperature_kelvin > early_sky.sun.colour_temperature_kelvin, "the light must cool toward day")


## Nothing hidden crosses. The sky is made of names and the numbers behind them
## are the content's, never the simulation's hidden state.
func test_nothing_hidden_crosses() -> void:
	for name in AtmosphereReviewSnapshots.NAMES:
		var snapshot := AtmosphereReviewSnapshots.of(name)
		check(not snapshot.has("cthulhu_heat"), "%s must not carry cthulhu_heat; only its band's name crosses" % name)
		var sky: Dictionary = atmosphere.resolve(snapshot)
		check(sky.heat_band == str(snapshot.heat_band), "%s must grade by the band the snapshot named" % name)
		check(not sky.grade.has("heat"), "the resolved grade must carry no pressure number")


func saved_reduced_motion() -> bool:
	var config := ConfigFile.new()
	if config.load(SettingsPanel.SETTINGS_PATH) != OK:
		return false
	return bool(config.get_value(SettingsPanel.SECTION, "reduced_motion", false))


func set_reduced_motion(value: bool) -> void:
	var config := ConfigFile.new()
	config.load(SettingsPanel.SETTINGS_PATH)
	config.set_value(SettingsPanel.SECTION, "reduced_motion", value)
	config.save(SettingsPanel.SETTINGS_PATH)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Atmosphere tests passed.")
	quit(0)


## A board that answers "where is this building?". The one thing that ever
## places a night lamp; lane P4's board will answer the same question.
class StubBoard:
	extends RefCounted
	const ANCHOR := Vector3(4.0, 0.0, -2.0)

	func building_anchor(_building_id: String) -> Vector3:
		return ANCHOR
