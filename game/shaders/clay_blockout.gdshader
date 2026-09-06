shader_type spatial;
render_mode depth_draw_opaque, cull_back, diffuse_burley, specular_schlick_ggx;

// The placeholder grammar. Every procedural blockout in the game wears this:
// a matte base, a rim light that separates a silhouette from the plate, and a
// top-down sky wrap so upward faces read as lit even in shadow. It is a
// placeholder look, but it is a *deliberate* one, and it is the thing an
// arriving model replaces rather than a thing the model has to fight.
//
// Engine lighting is not overridden. metallic and roughness go straight into
// the standard PBR terms, so a bronze fitting and a wet slab differ here the
// same way they would differ under StandardMaterial3D, and a model dropped
// beside a blockout sits in the same light.
//
// This shader never writes ALPHA, and that is deliberate rather than an
// omission. A spatial shader that assigns ALPHA at all is put in the
// transparent pass, where it stops writing depth and is sorted per object
// instead of per fragment -- which, on a blockout made of hundreds of boxes,
// means a distant sea plane painting over the trees in front of it. Every
// opaque material in this library leaves ALPHA alone; only the water writes
// it, because only the water is really transparent.
//
// Compiles unchanged on Forward+ and gl_compatibility: no Forward+-only
// built-in appears below.

#include "res://shaders/island_common.gdshaderinc"

uniform vec4 albedo : source_color = vec4(0.38, 0.44, 0.40, 1.0);
uniform float metallic : hint_range(0.0, 1.0) = 0.0;
uniform float roughness : hint_range(0.0, 1.0) = 0.9;

// Emission, kept here rather than in a second material type so a luminous
// elven seam is the same shader as the slab it is set into.
uniform vec4 emission_color : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform float emission_energy : hint_range(0.0, 8.0) = 0.0;

// The rim. Teal by default because the island's fill light is the sea.
uniform vec4 rim_color : source_color = vec4(0.333, 0.788, 0.675, 1.0);
uniform float rim_strength : hint_range(0.0, 3.0) = 0.55;
uniform float rim_sharpness : hint_range(1.0, 8.0) = 3.2;

// The sky wrap. Cool above, warm below, which is what a bright overcast sky
// over wet ground actually does.
uniform vec4 sky_wrap_color : source_color = vec4(0.55, 0.74, 0.78, 1.0);
uniform vec4 ground_bounce_color : source_color = vec4(0.24, 0.19, 0.13, 1.0);
uniform float sky_wrap_strength : hint_range(0.0, 1.0) = 0.15;

// Surface break-up. Large, low-contrast, and in world space, so neighbouring
// boxes do not repeat the same patch and the blockout stops looking tiled.
uniform float grain_scale : hint_range(0.01, 4.0) = 0.22;
uniform float grain_strength : hint_range(0.0, 0.6) = 0.18;

varying vec3 world_position;
varying vec3 world_normal;

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	float wrap = island_sky_wrap(world_normal);
	vec3 ambient_tint = mix(ground_bounce_color.rgb, sky_wrap_color.rgb, wrap);
	vec3 base = mix(albedo.rgb, albedo.rgb * ambient_tint * 1.35, sky_wrap_strength);

	// Triplanar-ish grain: the dominant axis picks the plane, which is enough
	// break-up for box and cylinder blockouts and costs one noise call.
	vec3 axis = abs(world_normal);
	vec2 grain_uv = world_position.xz;
	if (axis.x > axis.y && axis.x > axis.z) {
		grain_uv = world_position.zy;
	} else if (axis.z > axis.y) {
		grain_uv = world_position.xy;
	}
	float grain = island_fbm(grain_uv * grain_scale) - 0.5;
	base *= 1.0 + grain * grain_strength * 2.0;

	ALBEDO = base;
	METALLIC = metallic;
	ROUGHNESS = clamp(roughness + grain * 0.12, 0.02, 1.0);

	float rim = island_rim(NORMAL, VIEW, rim_sharpness);
	EMISSION = emission_color.rgb * emission_energy + rim_color.rgb * rim * rim_strength;
}
