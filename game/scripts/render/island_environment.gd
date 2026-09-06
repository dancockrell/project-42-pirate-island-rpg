class_name IslandEnvironment
extends WorldEnvironment
## The island's world environment as a scene, not only a resource.
##
## `res://render/world_environment.tres` holds the values that are the look:
## tonemapping, ambient source, glow, screen-space effects, the colour
## correction ramp. Those belong in the resource because they do not change
## while the game runs.
##
## The values that *do* change belong here, as exported properties, because
## P3 drives them from the simulation snapshot (sun angle and colour by time
## segment, fog density by weather, wind into the foliage shader). A value P3
## has to move is not hard-coded in the resource; it is applied from here into
## a per-instance duplicate of it, so two scenes never share one mutated
## Environment.

## Sun. `sun_pitch_degrees` is the elevation above the horizon and
## `sun_yaw_degrees` its compass heading; P3 replaces both per time segment.
@export_range(-90.0, 90.0, 0.1) var sun_pitch_degrees := 38.0:
	set(value):
		sun_pitch_degrees = value
		_apply_sun()

@export_range(-360.0, 360.0, 0.1) var sun_yaw_degrees := -47.0:
	set(value):
		sun_yaw_degrees = value
		_apply_sun()

# game colour: the island's daylight. It is world lighting, not interface, and
# `content/atmosphere/` overrides it per segment through the Atmosphere target;
# this is the value the scene opens on before a snapshot arrives.
@export var sun_color := Color("ffd8a2"):
	set(value):
		sun_color = value
		_apply_sun()

@export_range(0.0, 8.0, 0.01) var sun_energy := 1.85:
	set(value):
		sun_energy = value
		_apply_sun()

## Bounce light from the wet ground and the sea, opposite the sun. Never casts
## a shadow: it is a fill, and a second shadow map would read as an error.
# game colour: bounce off the wet ground and the sea, as above.
@export var fill_color := Color("6fd0c0"):
	set(value):
		fill_color = value
		_apply_fill()

@export_range(0.0, 4.0, 0.01) var fill_energy := 0.42:
	set(value):
		fill_energy = value
		_apply_fill()

## Weather. Fog density is the whole weather dial at this level; P3 tweens it.
@export_range(0.0, 0.2, 0.0001) var fog_density := 0.0075:
	set(value):
		fog_density = value
		_apply_environment_overrides()

# game colour: the weather's own colour, as above.
@export var fog_color := Color("2f6f74"):
	set(value):
		fog_color = value
		_apply_environment_overrides()

## Overall exposure, so a storm can darken the plate without touching any
## other value.
@export_range(0.1, 4.0, 0.01) var exposure := 1.0:
	set(value):
		exposure = value
		_apply_environment_overrides()

## Wind, published as the global shader parameters the foliage and water
## shaders read. Strength is metres-per-second-ish and heading is a compass
## bearing in degrees. P3 owns both; nothing here reads a clock to change them.
@export_range(0.0, 4.0, 0.01) var wind_strength := 0.55:
	set(value):
		wind_strength = value
		_apply_wind()

@export_range(-360.0, 360.0, 0.1) var wind_heading_degrees := 118.0:
	set(value):
		wind_heading_degrees = value
		_apply_wind()

## Global motion scale. B9's reduced-motion setting drives this to zero, which
## stills every shader that sways, ripples or drifts without changing a colour.
@export_range(0.0, 1.0, 0.01) var motion_scale := 1.0:
	set(value):
		motion_scale = value
		_apply_wind()

## The names the shader library reads. Declared in `[shader_globals]` in
## `game/project.godot`.
const WIND_PARAMETER := "wind"
const WIND_HEADING_PARAMETER := "wind_heading"
const MOTION_SCALE_PARAMETER := "motion_scale"

const SUN_NODE_NAME := "KeySun"
const FILL_NODE_NAME := "SeaFill"


func _ready() -> void:
	set_meta("render_contract", "P3 drives sun, fog, exposure and wind through the exported properties on this node; the resource holds only what does not move.")
	# A per-instance copy, so driving one scene's weather never edits the
	# shared resource on disk or any other scene using it.
	if environment != null:
		environment = environment.duplicate(true)
	_apply_sun()
	_apply_fill()
	_apply_environment_overrides()
	_apply_wind()


func sun() -> DirectionalLight3D:
	return get_node_or_null(SUN_NODE_NAME) as DirectionalLight3D


func fill() -> DirectionalLight3D:
	return get_node_or_null(FILL_NODE_NAME) as DirectionalLight3D


func _apply_sun() -> void:
	var light := sun()
	if light == null:
		return
	light.rotation_degrees = Vector3(-sun_pitch_degrees, sun_yaw_degrees, 0.0)
	light.light_color = sun_color
	light.light_energy = sun_energy


func _apply_fill() -> void:
	var light := fill()
	if light == null:
		return
	light.light_color = fill_color
	light.light_energy = fill_energy


func _apply_environment_overrides() -> void:
	if environment == null:
		return
	environment.fog_density = fog_density
	environment.fog_light_color = fog_color
	environment.tonemap_exposure = exposure


func _apply_wind() -> void:
	RenderingServer.global_shader_parameter_set(WIND_PARAMETER, wind_strength)
	RenderingServer.global_shader_parameter_set(WIND_HEADING_PARAMETER, wind_heading_degrees)
	RenderingServer.global_shader_parameter_set(MOTION_SCALE_PARAMETER, motion_scale)
