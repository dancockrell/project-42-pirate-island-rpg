class_name EnvironmentAtmosphereTarget
extends AtmosphereTarget

## The fallback [AtmosphereTarget]: Godot's own `Environment`,
## `DirectionalLight3D` and a procedural weather emitter, driven directly.
##
## It exists because the atmosphere must be visible before lane P2's render
## foundation lands, and it stays afterwards as the answer for any scene that
## has an environment but no P2 node -- a review scene, a capture, a suite.
## It is not a parallel implementation of P2's work: it writes the same four
## groups through the same four method names, and `Atmosphere` chooses P2's
## node over it the moment one is in the tree.
##
## Everything it builds is procedural and marked as such. No mesh, texture or
## material file is loaded; the rain and mist are `GPUParticles3D` over a
## generated `QuadMesh`, and each carries `production_state =
## "procedural-placeholder"` so a model or VFX author can find it by name.

## Metadata every node this target creates carries, so nothing it builds can be
## mistaken for authored art.
const PRODUCTION_STATE := "procedural-placeholder"

## The name of the emitter node this target parents its weather to.
const EMITTER_NAME := "AtmosphereWeather"

## The name of the node the night lamps hang under.
const NIGHT_LAMPS_NAME := "AtmosphereNightLamps"

## How many particles a full-intensity downpour draws. A ceiling, not a
## measurement: intensity scales the emitter's `amount_ratio` beneath it.
const PARTICLE_CEILING := 2400

var environment: Environment
var sun: DirectionalLight3D
## Every other `DirectionalLight3D` in the scene. A blockout scene lights
## itself with a key and one or more fills; the key is the sun, and the fills
## are the ambient arriving from a direction. Driving the key alone leaves a
## scene's authored teal fill fighting a warm dawn, which is exactly how a
## sunrise ends up reading as noon -- so the fills take the ambient's colour
## and the ambient's energy, and the segment table stays the one owner of both.
var fills: Array[DirectionalLight3D] = []
## Where the emitter and the lamps are parented. Usually the scene root.
var host: Node3D

var _emitter: GPUParticles3D
var _lamps_root: Node3D
var _lamp_nodes: Dictionary = {}
var _sky_material: ProceduralSkyMaterial
## The sky as `set_sun` left it, before the grade touched it. Kept so a grade
## applied twice does not compound: corruption and pressure always shift the
## authored sky, never the already-shifted one.
var _sky_base: Dictionary = {}


func describes() -> String:
	var where := "no host" if host == null else str(host.name)
	var key := "no key light" if sun == null else str(sun.name)
	var sky := "no environment" if environment == null else "environment"
	return "EnvironmentAtmosphereTarget on %s (%s, %s)" % [where, key, sky]


## Finds an `Environment` and a key `DirectionalLight3D` anywhere under `root`
## and binds to them. Returns false when the scene has no environment to drive,
## which is a refusal and not a silent no-op: a caller that gets false knows
## the sky it asked for was not applied.
##
## The key light is the brightest `DirectionalLight3D` in the tree. A scene
## lights itself with a key and one or more fills (the terrace does), and the
## key is the one the sun is; driving a fill would rotate the wrong light.
func bind_to(root: Node) -> bool:
	if root == null:
		return false
	var world_environment := _find(root, "WorldEnvironment") as WorldEnvironment
	if world_environment == null or world_environment.environment == null:
		return false
	environment = world_environment.environment
	var lights := _directional_lights(root)
	sun = null
	fills.clear()
	for light in lights:
		if sun == null or light.light_energy > sun.light_energy:
			if sun != null:
				fills.append(sun)
			sun = light
		else:
			fills.append(light)
	host = root as Node3D
	if host == null:
		host = _find(root, "Node3D") as Node3D
	return sun != null


func set_sun(sun_values: Dictionary) -> void:
	if sun != null:
		var elevation := float(sun_values.get("elevation_degrees", 45.0))
		var azimuth := float(sun_values.get("azimuth_degrees", 180.0))
		sun.rotation_degrees = Vector3(-elevation, azimuth, 0.0)
		sun.light_color = sun_values.get("colour", Color.WHITE)
		sun.light_energy = maxf(0.0, float(sun_values.get("energy", 1.0)))
	if environment == null:
		return
	var ambient_colour: Color = sun_values.get("ambient_colour", Color.WHITE)
	var ambient_energy := maxf(0.0, float(sun_values.get("ambient_energy", 0.5)))
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = ambient_colour
	environment.ambient_light_energy = ambient_energy
	for fill in fills:
		if not is_instance_valid(fill):
			continue
		fill.light_color = ambient_colour
		fill.light_energy = ambient_energy
	environment.tonemap_exposure = float(sun_values.get("exposure", 1.0))
	# A flat background colour cannot say what time it is: the segment table
	# authors a horizon and a zenith, and the difference between them is most of
	# what makes a dawn read as a dawn. A procedural sky is the one shape that
	# carries both on either renderer, so the sky is a gradient rather than a
	# fill and the ground under it takes the ambient's colour.
	_ensure_sky()
	_sky_base = {
		"top": sun_values.get("sky_top_colour", Color.BLACK),
		"horizon": sun_values.get("sky_horizon_colour", Color.BLACK),
		"ground": sun_values.get("ambient_colour", Color.BLACK),
	}
	_paint_sky(Color.WHITE, 0.0, 0.0, 0.0)


func set_fog(fog: Dictionary) -> void:
	if environment == null:
		return
	environment.fog_enabled = true
	environment.fog_density = maxf(0.0, float(fog.get("density", 0.0)))
	environment.fog_light_color = fog.get("colour", Color.WHITE)
	# How much of the sky the weather swallows. Left at Godot's default of 1.0
	# the depth fog paints over the whole sky and every hour of the day looks
	# the same; the weather table authors this per condition for that reason.
	environment.fog_sky_affect = clampf(float(fog.get("sky_affect", 0.5)), 0.0, 1.0)
	environment.glow_enabled = true
	environment.glow_intensity = maxf(0.0, float(fog.get("glow_intensity", 0.3)))


func set_wind(wind: Dictionary) -> void:
	var kind := str(wind.get("particle_kind", "none"))
	var intensity := clampf(float(wind.get("particle_intensity", 0.0)), 0.0, 1.0)
	var stilled := bool(wind.get("stilled", false))
	if host == null:
		return
	if kind == "none" or intensity <= 0.0:
		if _emitter != null:
			_emitter.emitting = false
		return
	_ensure_emitter()
	_emitter.amount_ratio = intensity
	_emitter.emitting = true
	# Reduced motion (B9): the weather is still there and still says what the
	# sky is doing -- it simply stops moving. `speed_scale = 0` freezes the
	# particles where they are rather than deleting the evidence.
	_emitter.speed_scale = 0.0 if stilled else 1.0
	var material := _emitter.process_material as ParticleProcessMaterial
	if material == null:
		return
	var strength := maxf(0.0, float(wind.get("strength", 0.0)))
	if kind == "rain":
		material.gravity = Vector3(strength * 6.0, -22.0, 0.0)
		material.initial_velocity_min = 9.0
		material.initial_velocity_max = 13.0
		_emitter.draw_pass_1 = _rain_mesh()
	else:
		material.gravity = Vector3(strength * 1.6, -0.25, strength * 0.6)
		material.initial_velocity_min = 0.2
		material.initial_velocity_max = 0.9
		_emitter.draw_pass_1 = _mist_mesh()


func set_grade(grade: Dictionary) -> void:
	if environment == null:
		return
	# Corruption desaturates and tints; hidden pressure turns the whole plate a
	# few degrees around the wheel. Both land in the one colour-correction
	# stage the Environment has, so they compose instead of fighting.
	environment.adjustment_enabled = true
	environment.adjustment_saturation = maxf(0.0, float(grade.get("saturation", 1.0)))
	environment.adjustment_contrast = maxf(0.0, float(grade.get("contrast", 1.0)))
	var tint: Color = grade.get("tint_colour", Color.WHITE)
	var strength := clampf(float(grade.get("tint_strength", 0.0)), 0.0, 1.0)
	var shift := float(grade.get("sky_hue_shift_degrees", 0.0))
	var desaturation := clampf(float(grade.get("horizon_desaturation", 0.0)), 0.0, 1.0)
	_paint_sky(tint, strength, shift, desaturation)
	environment.fog_light_color = _shift_hue(
		environment.fog_light_color.lerp(Color(tint.r, tint.g, tint.b, 1.0), strength * 0.6),
		shift
	)


func set_night_lamps(lamps: Array, lamp: Dictionary) -> void:
	if host == null:
		return
	if lamps.is_empty():
		if _lamps_root != null:
			_lamps_root.queue_free()
			_lamps_root = null
			_lamp_nodes.clear()
		return
	if _lamps_root == null or not is_instance_valid(_lamps_root):
		_lamps_root = Node3D.new()
		_lamps_root.name = NIGHT_LAMPS_NAME
		_lamps_root.set_meta("production_state", PRODUCTION_STATE)
		host.add_child(_lamps_root)
		_lamp_nodes.clear()
	var wanted: Dictionary = {}
	for entry in lamps:
		if not entry is Dictionary:
			continue
		var building_id := str(entry.get("building_id", ""))
		if building_id.is_empty():
			continue
		wanted[building_id] = true
		var light: OmniLight3D = _lamp_nodes.get(building_id, null)
		if light == null or not is_instance_valid(light):
			light = OmniLight3D.new()
			light.name = "lamp.%s" % building_id
			light.set_meta("production_state", PRODUCTION_STATE)
			_lamps_root.add_child(light)
			_lamp_nodes[building_id] = light
		var position: Vector3 = entry.get("position", Vector3.ZERO)
		light.position = position + Vector3(0.0, float(lamp.get("height_metres", 3.0)), 0.0)
		light.light_color = lamp.get("colour", Color.WHITE)
		light.light_energy = float(lamp.get("energy", 2.0))
		light.omni_range = float(lamp.get("range_metres", 12.0))
		light.omni_attenuation = float(lamp.get("attenuation", 1.5))
	for building_id in _lamp_nodes.keys():
		if wanted.has(building_id):
			continue
		var stale: OmniLight3D = _lamp_nodes[building_id]
		if is_instance_valid(stale):
			stale.queue_free()
		_lamp_nodes.erase(building_id)


## Whether this target is still bound to living nodes. A scene swap frees them
## and leaves the target holding an `Environment` nothing renders.
func is_bound() -> bool:
	if environment == null or host == null or not is_instance_valid(host):
		return false
	return host.is_inside_tree() and (sun == null or is_instance_valid(sun))


func _ensure_sky() -> void:
	if _sky_material != null and environment.sky != null:
		environment.background_mode = Environment.BG_SKY
		return
	_sky_material = ProceduralSkyMaterial.new()
	_sky_material.sun_angle_max = 12.0
	_sky_material.sun_curve = 0.2
	var sky := Sky.new()
	sky.sky_material = _sky_material
	environment.sky = sky
	environment.background_mode = Environment.BG_SKY


## Writes the authored sky through the grade. Always from [member _sky_base],
## so calling it for the sun and again for the grade tints once, not twice.
func _paint_sky(tint: Color, strength: float, shift: float, desaturation: float) -> void:
	if _sky_material == null or _sky_base.is_empty():
		return
	var top := _graded(_sky_base.top, tint, strength, shift, desaturation * 0.5)
	var horizon := _graded(_sky_base.horizon, tint, strength, shift, desaturation)
	var ground := _graded(_sky_base.ground, tint, strength, shift, desaturation)
	_sky_material.sky_top_color = top
	_sky_material.sky_horizon_color = horizon
	_sky_material.ground_horizon_color = horizon.darkened(0.35)
	_sky_material.ground_bottom_color = ground.darkened(0.5)


func _graded(
	colour: Color, tint: Color, strength: float, shift: float, desaturation: float
) -> Color:
	var graded := colour.lerp(Color(tint.r, tint.g, tint.b, colour.a), strength)
	graded = _shift_hue(graded, shift)
	return graded.lerp(Color(graded.v, graded.v, graded.v, graded.a), desaturation)


func _ensure_emitter() -> void:
	if _emitter != null and is_instance_valid(_emitter):
		return
	_emitter = GPUParticles3D.new()
	_emitter.name = EMITTER_NAME
	_emitter.set_meta("production_state", PRODUCTION_STATE)
	_emitter.set_meta(
		"replacement_contract",
		"Procedural weather stand-in. Replace with the authored rain and mist VFX; keep the node name and the amount_ratio/speed_scale contract Atmosphere writes."
	)
	_emitter.amount = PARTICLE_CEILING
	_emitter.lifetime = 2.4
	_emitter.local_coords = false
	# A capture is two frames old; without a preprocess the first frame of a
	# storm is an empty sky with the rain still at the top of it.
	_emitter.preprocess = 2.0
	_emitter.position = Vector3(0.0, 16.0, 0.0)
	var material := ParticleProcessMaterial.new()
	material.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
	material.emission_box_extents = Vector3(34.0, 8.0, 34.0)
	material.scale_min = 0.5
	material.scale_max = 1.0
	_emitter.process_material = material
	_emitter.draw_pass_1 = _rain_mesh()
	host.add_child(_emitter)


func _rain_mesh() -> Mesh:
	var mesh := QuadMesh.new()
	mesh.size = Vector2(0.03, 0.9)
	mesh.material = _particle_material(Color(0.86, 0.92, 0.96, 0.8))
	return mesh


func _mist_mesh() -> Mesh:
	var mesh := QuadMesh.new()
	mesh.size = Vector2(2.6, 2.6)
	mesh.material = _particle_material(Color(0.66, 0.6, 0.78, 0.16))
	return mesh


func _particle_material(colour: Color) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	material.billboard_mode = BaseMaterial3D.BILLBOARD_ENABLED
	material.vertex_color_use_as_albedo = false
	material.albedo_color = colour
	return material


## Hue rotation in degrees, in the one place that needs it. Godot's Color has
## `h`, so this is a read, an add and a write rather than a matrix.
func _shift_hue(colour: Color, degrees: float) -> Color:
	if is_zero_approx(degrees):
		return colour
	var shifted := Color.from_hsv(
		fposmod(colour.h + degrees / 360.0, 1.0), colour.s, colour.v, colour.a
	)
	return shifted


func _directional_lights(root: Node) -> Array[DirectionalLight3D]:
	var lights: Array[DirectionalLight3D] = []
	for node in _all(root):
		var light := node as DirectionalLight3D
		if light != null:
			lights.append(light)
	return lights


func _find(root: Node, type_name: String) -> Node:
	for node in _all(root):
		if node.is_class(type_name):
			return node
	return null


func _all(root: Node) -> Array[Node]:
	var found: Array[Node] = [root]
	for child in root.get_children():
		found.append_array(_all(child))
	return found
