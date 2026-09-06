extends Node
## Autoload `RenderProfile`. Reports which rendering method is actually live in
## this process and which rendering features that method cannot do, so nothing
## downstream has to guess and no card claims a look its renderer never drew.
##
## The feature table is not invented. Every entry is the engine's own refusal
## message, taken from the Godot 4.7.2 binary this project pins, e.g.
##   "Screen-space indirect lighting (SSIL) is only available when using the
##    Forward+ renderer."
##   "Screen-space ambient occlusion (SSAO) is only available when using the
##    Forward+ or Compatibility renderers."
## SSAO is therefore available on Compatibility in 4.7 and SSIL is not; the two
## are not interchangeable and are listed separately below.

## Rendering method names as `RenderingServer.get_current_rendering_method()`
## returns them and as `game/project.godot`'s `[rendering]` block writes them.
const FORWARD_PLUS := "forward_plus"
const MOBILE := "mobile"
const COMPATIBILITY := "gl_compatibility"

## Feature keys. One per rendering feature this project's environment, camera
## or shader library can ask for.
const FEATURE_SSAO := "ssao"
const FEATURE_SSIL := "ssil"
const FEATURE_SDFGI := "sdfgi"
const FEATURE_SSR := "ssr"
const FEATURE_VOLUMETRIC_FOG := "volumetric_fog"
const FEATURE_SUBSURFACE_SCATTERING := "subsurface_scattering"
const FEATURE_TAA := "taa"
const FEATURE_SCREEN_SPACE_AA := "screen_space_aa"
const FEATURE_AUTO_EXPOSURE := "auto_exposure"
const FEATURE_DEPTH_OF_FIELD := "depth_of_field"
const FEATURE_DECALS := "decals"
const FEATURE_VOXEL_GI := "voxel_gi"
const FEATURE_LIGHT_PROJECTORS := "light_projectors"
const FEATURE_NORMAL_ROUGHNESS_TEXTURE := "normal_roughness_texture"
const FEATURE_PARTICLE_TRAILS := "particle_trails"
const FEATURE_PARTICLE_SUB_EMITTERS := "particle_sub_emitters"
const FEATURE_GLOW := "glow"
const FEATURE_DEPTH_FOG := "depth_fog"
const FEATURE_MSAA_3D := "msaa_3d"
const FEATURE_TONEMAP_AGX := "tonemap_agx"
const FEATURE_COLOR_CORRECTION := "color_correction"

## Every feature key, in a stable order, so a report reads the same twice.
const FEATURES: Array[String] = [
	FEATURE_SSAO,
	FEATURE_SSIL,
	FEATURE_SDFGI,
	FEATURE_SSR,
	FEATURE_VOLUMETRIC_FOG,
	FEATURE_SUBSURFACE_SCATTERING,
	FEATURE_TAA,
	FEATURE_SCREEN_SPACE_AA,
	FEATURE_AUTO_EXPOSURE,
	FEATURE_DEPTH_OF_FIELD,
	FEATURE_DECALS,
	FEATURE_VOXEL_GI,
	FEATURE_LIGHT_PROJECTORS,
	FEATURE_NORMAL_ROUGHNESS_TEXTURE,
	FEATURE_PARTICLE_TRAILS,
	FEATURE_PARTICLE_SUB_EMITTERS,
	FEATURE_GLOW,
	FEATURE_DEPTH_FOG,
	FEATURE_MSAA_3D,
	FEATURE_TONEMAP_AGX,
	FEATURE_COLOR_CORRECTION,
]

## Which methods each feature works on. Read straight off the engine's own
## availability messages; a feature absent from a method's list is a feature
## that method silently ignores.
const SUPPORT: Dictionary = {
	FEATURE_SSAO: [FORWARD_PLUS, COMPATIBILITY],
	FEATURE_SSIL: [FORWARD_PLUS],
	FEATURE_SDFGI: [FORWARD_PLUS],
	FEATURE_SSR: [FORWARD_PLUS],
	FEATURE_VOLUMETRIC_FOG: [FORWARD_PLUS],
	FEATURE_SUBSURFACE_SCATTERING: [FORWARD_PLUS],
	FEATURE_TAA: [FORWARD_PLUS],
	FEATURE_SCREEN_SPACE_AA: [FORWARD_PLUS, MOBILE],
	FEATURE_AUTO_EXPOSURE: [FORWARD_PLUS],
	FEATURE_DEPTH_OF_FIELD: [FORWARD_PLUS, MOBILE],
	FEATURE_DECALS: [FORWARD_PLUS, MOBILE],
	FEATURE_VOXEL_GI: [FORWARD_PLUS, MOBILE],
	FEATURE_LIGHT_PROJECTORS: [FORWARD_PLUS, MOBILE],
	FEATURE_NORMAL_ROUGHNESS_TEXTURE: [FORWARD_PLUS],
	FEATURE_PARTICLE_TRAILS: [FORWARD_PLUS, MOBILE],
	FEATURE_PARTICLE_SUB_EMITTERS: [FORWARD_PLUS, MOBILE],
	FEATURE_GLOW: [FORWARD_PLUS, MOBILE, COMPATIBILITY],
	FEATURE_DEPTH_FOG: [FORWARD_PLUS, MOBILE, COMPATIBILITY],
	FEATURE_MSAA_3D: [FORWARD_PLUS, MOBILE, COMPATIBILITY],
	FEATURE_TONEMAP_AGX: [FORWARD_PLUS, MOBILE, COMPATIBILITY],
	FEATURE_COLOR_CORRECTION: [FORWARD_PLUS, MOBILE, COMPATIBILITY],
}

## The project's declared desktop method and its declared fallback. These are
## the two `[rendering]` keys, named here so a suite can hold the settings file
## and this table to each other rather than trusting either alone.
const DESKTOP_SETTING := "rendering/renderer/rendering_method"
const MOBILE_SETTING := "rendering/renderer/rendering_method.mobile"


## The rendering method this process is actually running. Not the setting: the
## engine falls back on its own when a device cannot do what was asked for.
func method() -> String:
	return RenderingServer.get_current_rendering_method()


## The rendering driver behind that method ("vulkan", "opengl3", "d3d12", ...).
func driver() -> String:
	return RenderingServer.get_current_rendering_driver_name()


## True when no window server is present. Under `--headless` Godot still
## reports the configured method and still parses shaders, but it draws
## nothing, so a capture taken here would be worthless and callers should say
## so rather than measure a black frame.
func is_headless() -> bool:
	return DisplayServer.get_name() == "headless"


func is_forward_plus() -> bool:
	return method() == FORWARD_PLUS


func is_compatibility() -> bool:
	return method() == COMPATIBILITY


## Whether a named feature works on the live method. An unknown key is not
## quietly true: it is refused loudly, because a silent yes is how a card ends
## up claiming an effect nothing drew.
func supports(feature: String) -> bool:
	if not SUPPORT.has(feature):
		push_error("RenderProfile was asked about an unknown rendering feature: %s" % feature)
		return false
	var methods: Array = SUPPORT[feature]
	return methods.has(method())


## Features the live method cannot do, in the stable order of FEATURES.
func unavailable_features() -> PackedStringArray:
	var missing := PackedStringArray()
	for feature in FEATURES:
		var methods: Array = SUPPORT[feature]
		if not methods.has(method()):
			missing.append(feature)
	return missing


## Features the live method can do, in the same stable order.
func available_features() -> PackedStringArray:
	var present := PackedStringArray()
	for feature in FEATURES:
		var methods: Array = SUPPORT[feature]
		if methods.has(method()):
			present.append(feature)
	return present


## One dictionary a capture caption, a diagnostic panel or a suite can read.
func report() -> Dictionary:
	return {
		"method": method(),
		"driver": driver(),
		"headless": is_headless(),
		"declared_desktop_method": String(ProjectSettings.get_setting(DESKTOP_SETTING, "")),
		"declared_mobile_method": String(ProjectSettings.get_setting(MOBILE_SETTING, "")),
		"available": available_features(),
		"unavailable": unavailable_features(),
	}


## A single line for a log or a capture caption.
func summary_line() -> String:
	var missing := unavailable_features()
	if missing.is_empty():
		return "Render: %s via %s; every declared feature available." % [method(), driver()]
	return "Render: %s via %s; unavailable here: %s." % [method(), driver(), ", ".join(missing)]
