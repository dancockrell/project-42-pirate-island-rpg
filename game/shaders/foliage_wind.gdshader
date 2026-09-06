shader_type spatial;
render_mode depth_draw_opaque, cull_disabled, diffuse_burley, specular_schlick_ggx;

// Jungle mass. The clay grammar plus a vertex sway driven by the global
// `wind` parameter, so every plant on the island leans the same way at the
// same moment without any of them reading a clock of its own.
//
// Sway is proportional to height above the instance origin, which is what
// makes a palm crown move and its trunk base stay put. The phase is offset by
// world position, so a stand of trees does not move as one board.
//
// `motion_scale` is B9's reduced-motion switch: at zero every vertex sits
// still and nothing else about the plant changes.
//
// Backfaces are lit (cull_disabled plus a translucency term), because a
// single-sided leaf card that goes black from behind is the tell that gives
// away cheap foliage.
//
// No Forward+-only built-in appears below; this compiles unchanged on
// gl_compatibility.

#include "res://shaders/island_common.gdshaderinc"

global uniform float wind;
global uniform float wind_heading;
global uniform float motion_scale;

uniform vec4 albedo : source_color = vec4(0.094, 0.235, 0.184, 1.0);
uniform vec4 tip_color : source_color = vec4(0.176, 0.408, 0.259, 1.0);
uniform float roughness : hint_range(0.0, 1.0) = 0.95;

// Sway. Amplitude in metres at one metre of height and one unit of wind.
uniform float sway_amplitude : hint_range(0.0, 1.0) = 0.055;
uniform float sway_frequency : hint_range(0.0, 6.0) = 1.15;
// A second, faster, smaller motion: the flutter on top of the lean.
uniform float flutter_amplitude : hint_range(0.0, 0.4) = 0.018;
uniform float flutter_frequency : hint_range(0.0, 12.0) = 4.3;
// Below this height nothing moves, so trunks stay planted.
uniform float anchor_height : hint_range(0.0, 4.0) = 0.35;

uniform vec4 rim_color : source_color = vec4(0.333, 0.788, 0.675, 1.0);
uniform float rim_strength : hint_range(0.0, 3.0) = 0.45;
uniform float rim_sharpness : hint_range(1.0, 8.0) = 2.6;
uniform vec4 sky_wrap_color : source_color = vec4(0.55, 0.74, 0.78, 1.0);
uniform float sky_wrap_strength : hint_range(0.0, 1.0) = 0.3;
// How much light passes through a leaf mass from behind.
uniform float translucency : hint_range(0.0, 1.0) = 0.35;

varying vec3 world_position;
varying vec3 world_normal;
varying float height_factor;

void vertex() {
	vec3 origin = MODEL_MATRIX[3].xyz;
	vec3 world_vertex = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float height = max(0.0, world_vertex.y - origin.y - anchor_height);
	height_factor = clamp(height * 0.35, 0.0, 1.0);

	float heading = radians(wind_heading);
	vec2 direction = vec2(sin(heading), cos(heading));
	// Phase from world position: neighbours are out of step with each other.
	float phase = dot(origin.xz, vec2(0.37, 0.71)) + island_hash(floor(origin.xz * 4.0)) * 6.2831;
	float strength = wind * motion_scale;

	float lean = sin(TIME * sway_frequency + phase) * sway_amplitude * height * strength;
	float flutter = sin(TIME * flutter_frequency + phase * 1.7 + world_vertex.y * 2.3) * flutter_amplitude * height_factor * strength;

	vec3 displaced = world_vertex;
	displaced.xz += direction * (lean + flutter);
	displaced.y -= abs(lean) * 0.25;
	VERTEX = (inverse(MODEL_MATRIX) * vec4(displaced, 1.0)).xyz;

	world_position = displaced;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	float wrap = island_sky_wrap(world_normal);
	float variation = island_fbm(world_position.xz * 0.6 + world_position.y * 0.2);
	vec3 base = mix(albedo.rgb, tip_color.rgb, clamp(height_factor + variation * 0.4, 0.0, 1.0));
	base = mix(base, base * sky_wrap_color.rgb * 1.35, sky_wrap_strength * wrap);

	ALBEDO = base;
	METALLIC = 0.0;
	ROUGHNESS = roughness;
	// Backfaces read as lit-through rather than black.
	BACKLIGHT = vec3(translucency) * tip_color.rgb;

	float rim = island_rim(NORMAL, VIEW, rim_sharpness);
	EMISSION = rim_color.rgb * rim * rim_strength;
}
