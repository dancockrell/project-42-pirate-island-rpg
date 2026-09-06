extends SceneTree

## The render foundation's contract: the project declares Forward+ with
## gl_compatibility as the fallback, the environment resource carries the
## values the P2 card names, every shader in the library compiles on BOTH
## renderers, and the camera rig frames what it is pointed at on the fixed
## isometric heading.
##
## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0

const ENVIRONMENT_RESOURCE := "res://render/world_environment.tres"
const ENVIRONMENT_SCENE := "res://render/world_environment.tscn"
const CAMERA_RIG_SCENE := "res://render/camera_rig.tscn"
const SHADER_DIRECTORY := "res://shaders"

## The autoload's script, preloaded so its constants are readable at compile
## time; the live singleton itself is fetched from the tree by name, the way
## every other suite in this project reaches an autoload.
const PROFILE := preload("res://scripts/render/render_profile.gd")

## Every shader the library owes. Named literally, so deleting one is a
## failure rather than a shorter loop.
const REQUIRED_SHADERS: Array[String] = [
	"clay_blockout",
	"stylised_water",
	"foliage_wind",
	"wet_stone",
	"bronze",
	"vellum",
	"placeholder",
]

## The material resources the two setpieces reach through
## SetpieceMeshFactory.LIBRARY.
const REQUIRED_MATERIALS: Array[String] = [
	"clay",
	"wet_stone",
	"bronze",
	"vellum",
	"foliage",
	"sea",
	"river",
	"placeholder",
]

## The global shader parameters the moving shaders read.
const REQUIRED_SHADER_GLOBALS: Array[String] = ["wind", "wind_heading", "motion_scale"]


func _init() -> void:
	call_deferred("run")


func run() -> void:
	_check_project_settings()
	_check_environment_resource()
	_check_environment_scene()
	_check_shaders_compile_on_both_renderers()
	_check_material_library()
	await _check_camera_rig()
	_check_render_profile()
	if failures == 0:
		print("Render foundation tests passed.")
	quit(1 if failures > 0 else 0)


func _check_project_settings() -> void:
	check(
		ProjectSettings.get_setting(PROFILE.DESKTOP_SETTING, "") == PROFILE.FORWARD_PLUS,
		"the project must declare forward_plus as the desktop rendering method"
	)
	check(
		ProjectSettings.get_setting(PROFILE.MOBILE_SETTING, "") == PROFILE.COMPATIBILITY,
		"the project must declare gl_compatibility as the mobile fallback"
	)
	check(
		ProjectSettings.get_setting("rendering/environment/defaults/default_environment", "") == ENVIRONMENT_RESOURCE,
		"the default environment must be the island's own resource"
	)
	# MSAA rather than TAA or screen-space AA: the same edge on both renderers.
	check(int(ProjectSettings.get_setting("rendering/anti_aliasing/quality/msaa_3d", -1)) == 2, "MSAA 3D must be 4x")
	check(not bool(ProjectSettings.get_setting("rendering/anti_aliasing/quality/use_taa", true)), "TAA must be off: it is Forward+ only and it smears an isometric plate")
	check(int(ProjectSettings.get_setting("rendering/anti_aliasing/quality/screen_space_aa", -1)) == 0, "screen-space AA must be off: it does not exist on gl_compatibility")
	check(int(ProjectSettings.get_setting("rendering/textures/default_filters/anisotropic_filtering_level", -1)) == 3, "anisotropic filtering must be on for the grazing isometric angle")
	check(int(ProjectSettings.get_setting("rendering/lights_and_shadows/directional_shadow/size", 0)) == 4096, "the directional shadow map must be 4096")
	check(int(ProjectSettings.get_setting("rendering/lights_and_shadows/directional_shadow/soft_shadow_filter_quality", -1)) == 3, "directional shadows must use a soft filter")
	for parameter in REQUIRED_SHADER_GLOBALS:
		check(
			ProjectSettings.has_setting("shader_globals/%s" % parameter),
			"the project must declare the global shader parameter: %s" % parameter
		)


func _check_environment_resource() -> void:
	var environment := load(ENVIRONMENT_RESOURCE) as Environment
	check(environment != null, "the island environment resource must load")
	if environment == null:
		return

	check(environment.tonemap_mode == Environment.TONE_MAPPER_AGX, "tonemapping must be AgX")
	check(environment.background_mode == Environment.BG_SKY, "the background must be the sky, not a flat colour")
	check(environment.sky != null, "the environment must carry a sky")
	if environment.sky != null:
		check(environment.sky.sky_material is ProceduralSkyMaterial, "the sky must be procedural, not an imported panorama")
	check(environment.ambient_light_source == Environment.AMBIENT_SOURCE_SKY, "ambient light must come from the sky")
	check(environment.reflected_light_source == Environment.REFLECTION_SOURCE_SKY, "reflections must come from the sky")

	check(environment.ssao_enabled, "SSAO must be on: it is available on Forward+ and on gl_compatibility")
	check(environment.ssil_enabled, "SSIL must be on (Forward+ only; gl_compatibility ignores it)")
	check(not environment.sdfgi_enabled, "SDFGI must be off: it is Forward+ only and this board is lit by one sun")

	check(environment.glow_enabled, "glow must be on")
	check(environment.glow_hdr_threshold < 1.0, "the glow threshold must be low enough that a luminous seam blooms")
	check(environment.glow_blend_mode == Environment.GLOW_BLEND_MODE_SCREEN, "glow must blend by screen, the mode gl_compatibility forces, so both renderers draw one image")

	check(environment.fog_enabled, "depth fog must be on")
	check(environment.fog_mode == Environment.FOG_MODE_DEPTH, "the fog must be depth fog, not volumetric: volumetric fog is Forward+ only")
	# The island's fog is the sea's colour, not grey: blue-green, and not on
	# the neutral line where red, green and blue are all within a hair.
	var fog := environment.fog_light_color
	check(fog.b > fog.r and fog.g > fog.r, "the fog must be the sea's colour, not grey")
	check(absf(fog.g - fog.b) < 0.2, "the fog must read as sea-teal rather than pure blue")
	check(environment.fog_aerial_perspective > 0.0, "aerial perspective must put the sky behind the far fog")

	check(environment.adjustment_enabled, "colour correction must be on")
	var correction := environment.adjustment_color_correction as GradientTexture1D
	check(correction != null, "colour correction must use the in-repo gradient ramp")
	if correction != null and correction.gradient != null:
		var shadow := correction.gradient.sample(0.0)
		var highlight := correction.gradient.sample(1.0)
		check(shadow.b > shadow.r, "the correction ramp must push shadows toward teal")
		check(highlight.r > highlight.b, "the correction ramp must push highlights toward warm")


func _check_environment_scene() -> void:
	var packed := load(ENVIRONMENT_SCENE) as PackedScene
	check(packed != null, "the island environment scene must load")
	if packed == null:
		return
	var world := packed.instantiate() as IslandEnvironment
	check(world != null, "the environment scene's root must be an IslandEnvironment")
	if world == null:
		return
	root.add_child(world)
	check(world.environment != null, "the environment scene must carry the island environment")
	check(world.sun() != null, "the environment scene must carry a key sun")
	check(world.fill() != null, "the environment scene must carry a sea fill")
	if world.sun() != null:
		check(world.sun().shadow_enabled, "the key sun must cast shadows")
	if world.fill() != null:
		check(not world.fill().shadow_enabled, "the fill must not cast a second shadow")

	# The values P3 drives are properties on this node, not literals in the
	# resource: setting one has to actually reach the light and the fog.
	world.sun_pitch_degrees = 12.5
	world.fog_density = 0.031
	check(is_equal_approx(world.sun().rotation_degrees.x, -12.5), "sun_pitch_degrees must drive the key light")
	check(is_equal_approx(world.environment.fog_density, 0.031), "fog_density must drive the environment")

	# And driving one instance must not edit the resource on disk or any other
	# scene using it.
	var shared := load(ENVIRONMENT_RESOURCE) as Environment
	check(not is_equal_approx(shared.fog_density, 0.031), "driving one scene's weather must not mutate the shared environment resource")

	world.queue_free()


func _check_shaders_compile_on_both_renderers() -> void:
	var directory := DirAccess.open(SHADER_DIRECTORY)
	check(directory != null, "the shader directory must exist")
	if directory == null:
		return
	var found: Array[String] = []
	for file_name in directory.get_files():
		if file_name.ends_with(".gdshader"):
			found.append(file_name.get_basename())
	for required in REQUIRED_SHADERS:
		check(found.has(required), "the shader library is missing: %s.gdshader" % required)

	for base_name in found:
		var path := "%s/%s.gdshader" % [SHADER_DIRECTORY, base_name]
		var shader := load(path) as Shader
		check(shader != null, "shader must load: %s" % path)
		if shader == null:
			continue
		# A shader that fails to compile reports no parameters at all, on
		# every driver including the headless one, because the parse is done
		# by the driver-independent front end. That makes this a real compile
		# check in CI and not only on a machine with a GPU.
		check(shader.get_shader_uniform_list().size() > 0, "shader must compile and expose its uniforms: %s" % path)
		var material := ShaderMaterial.new()
		material.shader = shader
		check(material.shader == shader, "shader must be usable as a ShaderMaterial: %s" % path)

		# The live renderer only ever compiles one side of a
		# `#if CURRENT_RENDERER == ...` guard. Compile the other side too, by
		# renaming the macro and defining it ourselves, so a compatibility
		# path that has rotted is caught on a Forward+ run and the other way
		# round.
		for forced in [PROFILE.FORWARD_PLUS, PROFILE.COMPATIBILITY]:
			var twin := Shader.new()
			twin.code = _force_renderer(shader.code, forced)
			check(
				twin.get_shader_uniform_list().size() > 0,
				"shader must also compile forced onto %s: %s" % [forced, path]
			)


## Rewrite a shader's renderer guards so a chosen branch is taken, whichever
## renderer is actually live.
func _force_renderer(code: String, method: String) -> String:
	if not code.contains("CURRENT_RENDERER"):
		return code
	var macro := "RENDERER_FORWARD_PLUS" if method == PROFILE.FORWARD_PLUS else "RENDERER_COMPATIBILITY"
	var rewritten := code.replace("CURRENT_RENDERER", "FORCED_RENDERER")
	# The define has to sit after `shader_type`, which must be the first
	# statement in the file.
	var marker := ";"
	var shader_type_end := rewritten.find(marker, rewritten.find("shader_type"))
	if shader_type_end < 0:
		return rewritten
	return rewritten.insert(shader_type_end + 1, "\n#define FORCED_RENDERER %s\n" % macro)


func _check_material_library() -> void:
	for key in REQUIRED_MATERIALS:
		check(SetpieceMeshFactory.LIBRARY.has(key), "the factory must offer the library material: %s" % key)
		var material := SetpieceMeshFactory.library_material(key)
		check(material != null, "library material must load: %s" % key)
		if material != null:
			check(material.shader != null, "library material must carry a shader: %s" % key)

	# The factory hook: a setpiece asking for a colour gets the clay grammar,
	# not a flat StandardMaterial3D.
	var tinted := SetpieceMeshFactory.material(Color("55c9ac"), 0.25, 0.4, Color("55c9ac"), 1.2) as ShaderMaterial
	check(tinted != null, "SetpieceMeshFactory.material must return a shader material")
	if tinted != null:
		check(tinted.get_shader_parameter("albedo") == Color("55c9ac"), "the factory must pass the caller's colour through")
		check(is_equal_approx(float(tinted.get_shader_parameter("emission_energy")), 1.2), "the factory must pass the caller's emission through")

	# Duplicated, not shared: tuning one wall must not repaint every wall.
	var first := SetpieceMeshFactory.library_material(SetpieceMeshFactory.CLAY)
	var second := SetpieceMeshFactory.library_material(SetpieceMeshFactory.CLAY)
	first.set_shader_parameter("albedo", Color.RED)
	check(second.get_shader_parameter("albedo") != Color.RED, "library materials must be duplicated per caller")


func _check_camera_rig() -> void:
	var packed := load(CAMERA_RIG_SCENE) as PackedScene
	check(packed != null, "the camera rig scene must load")
	if packed == null:
		return
	var rig := packed.instantiate() as CameraDirector
	check(rig != null, "the camera rig's root must be a CameraDirector")
	if rig == null:
		return
	root.add_child(rig)
	await process_frame

	var view := rig.camera()
	check(view != null, "the rig must carry its isometric camera")
	if view == null:
		rig.queue_free()
		return

	# The fixed heading, named once and applied.
	check(is_equal_approx(view.rotation_degrees.y, CameraDirector.ISOMETRIC_YAW_DEGREES), "the rig must hold the named isometric yaw")
	check(is_equal_approx(view.rotation_degrees.x, -CameraDirector.ISOMETRIC_PITCH_DEGREES), "the rig must hold the named isometric pitch")
	check(is_equal_approx(CameraDirector.ISOMETRIC_YAW_DEGREES, 45.0), "the isometric yaw is 45 degrees")
	check(is_equal_approx(CameraDirector.ISOMETRIC_PITCH_DEGREES, 30.0), "the isometric pitch is 30 degrees")

	# Orthographic framing: a bigger box is framed bigger, and a wider
	# distance leaves more air around the same box.
	var box := AABB(Vector3(-10.0, 0.0, -10.0), Vector3(20.0, 6.0, 20.0))
	rig.set_projection_mode(CameraDirector.ProjectionMode.ORTHOGONAL)
	rig.frame_bounds(box, CameraDirector.Distance.ROOM)
	check(view.projection == Camera3D.PROJECTION_ORTHOGONAL, "orthographic mode must use an orthographic projection")
	var room_size := view.size
	check(room_size > 0.0, "orthographic framing must produce a positive size")
	rig.frame_bounds(box, CameraDirector.Distance.WORLD)
	check(view.size > room_size, "world distance must leave more air than room distance")
	rig.frame_bounds(box, CameraDirector.Distance.ROUTE)
	check(view.size > room_size and view.size < room_size * CameraDirector.MARGIN_WORLD / CameraDirector.MARGIN_ROOM + 0.001, "route distance must sit between room and world")

	# Doubling the subject doubles the framed size: the margins are ratios.
	rig.frame_bounds(box, CameraDirector.Distance.ROOM)
	var single := view.size
	rig.frame_bounds(AABB(box.position * 2.0, box.size * 2.0), CameraDirector.Distance.ROOM)
	check(is_equal_approx(view.size, single * 2.0), "framing must scale with the subject")

	# The camera looks at what it framed, from the fixed heading.
	rig.frame_bounds(box, CameraDirector.Distance.ROOM)
	var to_centre := (box.get_center() - view.global_position).normalized()
	check(to_centre.dot(-CameraDirector.isometric_offset_direction()) > 0.999, "the camera must sit on the isometric heading from what it frames")

	# Near-orthographic: still perspective, still the same heading, and much
	# further away than the orthographic camera.
	var orthographic_distance := view.global_position.distance_to(box.get_center())
	rig.set_projection_mode(CameraDirector.ProjectionMode.NEAR_ORTHOGONAL)
	check(view.projection == Camera3D.PROJECTION_PERSPECTIVE, "near-orthographic mode must use a perspective projection")
	check(is_equal_approx(view.fov, CameraDirector.NEAR_ORTHOGONAL_FOV_DEGREES), "near-orthographic mode must use the narrow named field of view")
	check(view.fov < 20.0, "a near-orthographic field of view must be narrow enough to read as parallel")
	check(view.global_position.distance_to(box.get_center()) > orthographic_distance, "near-orthographic mode must pull the camera back")
	check(is_equal_approx(view.rotation_degrees.y, CameraDirector.ISOMETRIC_YAW_DEGREES), "changing projection must not change the heading")

	# A degenerate box must not produce a zero-size camera drawing a uniform
	# frame, which is the failure a screenshot gate cannot see.
	rig.set_projection_mode(CameraDirector.ProjectionMode.ORTHOGONAL)
	rig.frame_bounds(AABB(Vector3.ZERO, Vector3.ZERO), CameraDirector.Distance.ROOM)
	check(view.size > 0.0, "an empty box must still frame something rather than collapse")

	rig.queue_free()


func _check_render_profile() -> void:
	var profile: Node = root.get_node_or_null("RenderProfile")
	check(profile != null, "RenderProfile must be registered as an autoload")
	if profile == null:
		return
	var live: String = profile.method()
	check(live != "", "RenderProfile must report a live rendering method")

	# The feature table is the point: the effects that do not exist on
	# gl_compatibility must be named as absent there and present on Forward+.
	for feature in [
		PROFILE.FEATURE_SSIL,
		PROFILE.FEATURE_SDFGI,
		PROFILE.FEATURE_VOLUMETRIC_FOG,
		PROFILE.FEATURE_SSR,
		PROFILE.FEATURE_SUBSURFACE_SCATTERING,
		PROFILE.FEATURE_TAA,
		PROFILE.FEATURE_AUTO_EXPOSURE,
	]:
		var methods: Array = PROFILE.SUPPORT[feature]
		check(methods.has(PROFILE.FORWARD_PLUS), "%s must be listed as a Forward+ feature" % feature)
		check(not methods.has(PROFILE.COMPATIBILITY), "%s must be listed as unavailable on gl_compatibility" % feature)

	# SSAO is the exception the engine's own message names, and the
	# environment relies on it being available in both captures.
	var ssao: Array = PROFILE.SUPPORT[PROFILE.FEATURE_SSAO]
	check(ssao.has(PROFILE.COMPATIBILITY) and ssao.has(PROFILE.FORWARD_PLUS), "SSAO must be listed as available on both methods")

	# Screen-space AA exists on Forward+ and Mobile but not Compatibility,
	# which is why the project chose MSAA instead.
	var screen_space_aa: Array = PROFILE.SUPPORT[PROFILE.FEATURE_SCREEN_SPACE_AA]
	check(not screen_space_aa.has(PROFILE.COMPATIBILITY), "screen-space AA must be listed as unavailable on gl_compatibility")

	var missing: PackedStringArray = profile.unavailable_features()
	if live == PROFILE.FORWARD_PLUS:
		check(missing.is_empty(), "Forward+ must report no unavailable features, got: %s" % ", ".join(missing))
	if live == PROFILE.COMPATIBILITY:
		check(missing.has(PROFILE.FEATURE_SSIL), "on gl_compatibility SSIL must be reported unavailable")
		check(not missing.has(PROFILE.FEATURE_SSAO), "on gl_compatibility SSAO must not be reported unavailable")

	# Every key in the ordered list has a support entry, and the other way
	# round: a feature nobody can ask about is a feature nobody checks.
	check(PROFILE.FEATURES.size() == PROFILE.SUPPORT.size(), "every feature key must carry a support entry")
	for feature in PROFILE.FEATURES:
		check(PROFILE.SUPPORT.has(feature), "feature has no support entry: %s" % feature)
	check(not String(profile.summary_line()).is_empty(), "RenderProfile must be able to describe itself in one line")


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
