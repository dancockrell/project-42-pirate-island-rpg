shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back, diffuse_burley, specular_schlick_ggx;

// The island is wet. This is the clay grammar with the two things rain does
// to stone: upward faces hold a film of water, so they darken and go glossy,
// and the low places hold more of it than the high ones.
//
// The terrace, the steps, the quay and the cobbles wear this. It is the
// difference between "grey box" and "a place it has just stopped raining in".
//
// No Forward+-only built-in appears below.

#include "res://shaders/island_common.gdshaderinc"

uniform vec4 dry_color : source_color = vec4(0.380, 0.439, 0.373, 1.0);
uniform vec4 wet_color : source_color = vec4(0.114, 0.161, 0.153, 1.0);
uniform float dry_roughness : hint_range(0.0, 1.0) = 0.92;
uniform float wet_roughness : hint_range(0.0, 1.0) = 0.12;

// How wet the surface is overall. P3 drives this from weather; at zero the
// material is a plain matte stone and nothing below has any effect.
uniform float wetness : hint_range(0.0, 1.0) = 0.55;
// Only faces pointing this far up hold water.
uniform float pooling_bias : hint_range(0.0, 1.0) = 0.45;
// Patchiness of the film, in world-space metres.
uniform float puddle_scale : hint_range(0.05, 4.0) = 0.55;
uniform float puddle_contrast : hint_range(0.0, 3.0) = 1.4;

// Weathering: a paler crust on exposed edges, a darker deposit in cavities.
uniform vec4 lichen_color : source_color = vec4(0.286, 0.373, 0.259, 1.0);
uniform float lichen_amount : hint_range(0.0, 1.0) = 0.22;

uniform vec4 rim_color : source_color = vec4(0.333, 0.788, 0.675, 1.0);
uniform float rim_strength : hint_range(0.0, 3.0) = 0.5;
uniform float rim_sharpness : hint_range(1.0, 8.0) = 3.4;
uniform vec4 sky_wrap_color : source_color = vec4(0.55, 0.74, 0.78, 1.0);
uniform float sky_wrap_strength : hint_range(0.0, 1.0) = 0.24;

varying vec3 world_position;
varying vec3 world_normal;

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	float upward = clamp((world_normal.y - pooling_bias) / max(1.0 - pooling_bias, 0.0001), 0.0, 1.0);
	float patch = clamp((island_fbm(world_position.xz * puddle_scale) - 0.5) * puddle_contrast + 0.5, 0.0, 1.0);
	float film = clamp(upward * patch * wetness, 0.0, 1.0);

	float lichen = clamp((island_fbm(world_position.xz * 1.9 + world_position.y * 0.7) - 0.45) * 2.2, 0.0, 1.0) * lichen_amount * (1.0 - film);
	vec3 base = mix(dry_color.rgb, lichen_color.rgb, lichen);
	base = mix(base, wet_color.rgb, film);

	float wrap = island_sky_wrap(world_normal);
	base = mix(base, base * sky_wrap_color.rgb * 2.0, sky_wrap_strength * wrap * (1.0 - film));

	ALBEDO = base;
	METALLIC = 0.0;
	ROUGHNESS = mix(dry_roughness, wet_roughness, film);
	// A water film has a dielectric's specular; dry stone has almost none.
	SPECULAR = mix(0.15, 0.6, film);

	float rim = island_rim(NORMAL, VIEW, rim_sharpness);
	EMISSION = rim_color.rgb * rim * rim_strength;
}
