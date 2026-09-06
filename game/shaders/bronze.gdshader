shader_type spatial;
render_mode depth_draw_opaque, cull_back, diffuse_burley, specular_schlick_ggx;

// Bronze, in the two states this island has it in: struck and polished on the
// raised faces, and gone to verdigris everywhere the rain sits. It is the
// metal of the elven gate fittings, the Handsome Jack's boiler and every
// bronze-and-vellum edge the interface will later be drawn in.
//
// Patina is driven by the same upward-facing rule as wet_stone, so a fitting
// and the slab it is bolted to agree about which way the weather came from.
//
// No Forward+-only built-in appears below.

#include "res://shaders/island_common.gdshaderinc"

// The palette's bronze, and the green it becomes.
uniform vec4 metal_color : source_color = vec4(0.718, 0.541, 0.294, 1.0);
uniform vec4 patina_color : source_color = vec4(0.259, 0.518, 0.443, 1.0);
uniform vec4 tarnish_color : source_color = vec4(0.204, 0.145, 0.075, 1.0);

uniform float patina_amount : hint_range(0.0, 1.0) = 0.45;
uniform float patina_bias : hint_range(-1.0, 1.0) = 0.15;
uniform float patina_scale : hint_range(0.05, 8.0) = 1.6;

uniform float polish : hint_range(0.0, 1.0) = 0.72;
uniform float metal_roughness : hint_range(0.0, 1.0) = 0.28;
uniform float patina_roughness : hint_range(0.0, 1.0) = 0.85;

// Bronze catches a warm edge even in shadow; that is most of what says metal
// at gameplay distance where no reflection is legible.
uniform vec4 rim_color : source_color = vec4(0.918, 0.678, 0.353, 1.0);
uniform float rim_strength : hint_range(0.0, 3.0) = 0.85;
uniform float rim_sharpness : hint_range(1.0, 8.0) = 2.4;

uniform vec4 emission_color : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform float emission_energy : hint_range(0.0, 8.0) = 0.0;

varying vec3 world_position;
varying vec3 world_normal;

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	float upward = clamp(world_normal.y * 0.5 + 0.5 + patina_bias, 0.0, 1.0);
	float blotch = island_fbm(world_position.xz * patina_scale + world_position.y * patina_scale * 0.7);
	float patina = clamp((blotch * 0.85 + upward * 0.55 - 0.45) * 2.4, 0.0, 1.0) * patina_amount;

	// Tarnish in the cavities: the same field, inverted and weaker.
	float tarnish = clamp((0.55 - blotch) * 1.6, 0.0, 1.0) * (1.0 - patina) * 0.5;

	vec3 base = mix(metal_color.rgb, tarnish_color.rgb, tarnish);
	base = mix(base, patina_color.rgb, patina);

	ALBEDO = base;
	// Verdigris is not metal; it is a crust sitting on it.
	METALLIC = mix(polish, 0.0, patina);
	ROUGHNESS = mix(metal_roughness, patina_roughness, patina);
	SPECULAR = mix(0.55, 0.2, patina);

	float rim = island_rim(NORMAL, VIEW, rim_sharpness);
	EMISSION = emission_color.rgb * emission_energy + rim_color.rgb * rim * rim_strength * (1.0 - patina * 0.6);
}
