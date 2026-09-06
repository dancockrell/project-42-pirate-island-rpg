shader_type spatial;
render_mode depth_draw_opaque, cull_back, unshaded;

// Deliberately ugly, and that is the whole specification.
//
// AGENTS.md forbids placeholders that can be mistaken for finished work. This
// material is what a mesh wears when nobody has decided what it should look
// like yet: a hard checker in the palette's danger colour against the panel
// dark, with a bright band on every upward face so it cannot be missed in a
// capture taken at gameplay distance.
//
// It is unshaded on purpose. Lighting would flatter it, and a flattered
// placeholder gets left in.
//
// No Forward+-only built-in appears below.

uniform vec4 checker_color : source_color = vec4(0.761, 0.306, 0.271, 1.0);
uniform vec4 ground_color : source_color = vec4(0.075, 0.137, 0.129, 1.0);
// The label colour: what a caption drawn over this mesh would be written in.
uniform vec4 label_color : source_color = vec4(0.918, 0.874, 0.792, 1.0);

// Checker size in world metres, so two placeholders of different sizes show
// the same squares and the scale of the scene stays readable.
uniform float checker_metres : hint_range(0.05, 8.0) = 0.5;
// The band across upward faces.
uniform float band_metres : hint_range(0.05, 8.0) = 2.0;
uniform float band_width : hint_range(0.0, 0.5) = 0.12;

varying vec3 world_position;
varying vec3 world_normal;

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 cells = floor(world_position / max(checker_metres, 0.0001));
	float parity = mod(cells.x + cells.y + cells.z, 2.0);
	vec3 base = mix(ground_color.rgb, checker_color.rgb, parity);

	// Diagonal hazard band on anything facing up.
	float diagonal = fract((world_position.x + world_position.z) / max(band_metres, 0.0001));
	float band = step(diagonal, band_width) * clamp(world_normal.y, 0.0, 1.0);
	base = mix(base, label_color.rgb, band);

	ALBEDO = base;
	// Unshaded already, but the emission keeps it visible where fog and
	// exposure would otherwise let it blend into the plate.
	EMISSION = base * 0.35;
}
