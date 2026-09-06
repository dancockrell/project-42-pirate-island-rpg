shader_type spatial;
render_mode depth_draw_opaque, cull_disabled, diffuse_lambert_wrap, specular_disabled;

// Vellum: sailcloth, charts, awnings, the paper rigs the battle screen uses
// until models arrive, and the physical stock the interface's bronze-and-
// vellum grammar is drawn on. Warm, fibrous, and lit through from behind.
//
// Specular is off and diffuse is wrapped, because vellum has no highlight and
// scatters light across its surface: that combination is what stops a
// sail from reading as a painted board.
//
// No Forward+-only built-in appears below. Real subsurface scattering is
// Forward+ only, so the translucency here is a BACKLIGHT term instead, which
// both methods draw identically.

#include "res://shaders/island_common.gdshaderinc"

uniform vec4 stock_color : source_color = vec4(0.918, 0.874, 0.792, 1.0);
uniform vec4 shade_color : source_color = vec4(0.694, 0.639, 0.522, 1.0);
uniform vec4 ink_color : source_color = vec4(0.204, 0.153, 0.098, 1.0);
// How far the darkest fibre valleys bleed toward ink.
uniform float ink_bleed : hint_range(0.0, 1.0) = 0.12;

// The fibre grain: fine, directional, low contrast.
uniform float fibre_scale : hint_range(1.0, 60.0) = 18.0;
uniform float fibre_stretch : hint_range(1.0, 12.0) = 5.0;
uniform float fibre_strength : hint_range(0.0, 1.0) = 0.18;

// Foxing: the blotchy age staining round the edges of old stock.
uniform float foxing_scale : hint_range(0.1, 8.0) = 1.3;
uniform float foxing_amount : hint_range(0.0, 1.0) = 0.22;

// How much light comes through from behind.
uniform float translucency : hint_range(0.0, 1.0) = 0.55;
uniform float roughness : hint_range(0.0, 1.0) = 0.95;

uniform vec4 rim_color : source_color = vec4(0.918, 0.874, 0.792, 1.0);
uniform float rim_strength : hint_range(0.0, 3.0) = 0.3;
uniform float rim_sharpness : hint_range(1.0, 8.0) = 4.0;

varying vec3 world_position;

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	// Stretched noise reads as a fibre direction without a texture.
	vec2 fibre_uv = vec2(UV.x * fibre_scale * fibre_stretch, UV.y * fibre_scale);
	float fibre = island_value_noise(fibre_uv) - 0.5;
	float foxing = clamp((island_fbm(world_position.xz * foxing_scale + UV * 2.0) - 0.52) * 3.0, 0.0, 1.0) * foxing_amount;

	vec3 base = mix(stock_color.rgb, shade_color.rgb, foxing);
	base *= 1.0 + fibre * fibre_strength;
	base = mix(base, ink_color.rgb, clamp(-fibre * 2.0, 0.0, 1.0) * ink_bleed);

	ALBEDO = base;
	METALLIC = 0.0;
	ROUGHNESS = roughness;
	BACKLIGHT = vec3(translucency) * stock_color.rgb;

	float rim = island_rim(NORMAL, VIEW, rim_sharpness);
	EMISSION = rim_color.rgb * rim * rim_strength;
}
