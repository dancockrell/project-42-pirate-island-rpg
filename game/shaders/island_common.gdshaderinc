// Shared helpers for the island shader library.
//
// Everything here is procedural: no shader in this repository samples an
// image file it did not generate. The noise below is the source of every
// ripple, fibre and patina pattern in the library, so the look has one owner
// and a change to it moves the whole set together.
//
// Nothing here uses a Forward+-only built-in, so this file is safe to include
// from a shader that must also compile on gl_compatibility.

// A cheap deterministic hash. The constants are the usual ones; they are
// arbitrary but fixed, so two runs of the same frame produce the same noise.
float island_hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

// Value noise with a smoothstep interpolant: soft, banded, and cheap enough
// to call several times per fragment on a software rasteriser.
float island_value_noise(vec2 p) {
	vec2 cell = floor(p);
	vec2 within = fract(p);
	vec2 blend = within * within * (3.0 - 2.0 * within);
	float a = island_hash(cell);
	float b = island_hash(cell + vec2(1.0, 0.0));
	float c = island_hash(cell + vec2(0.0, 1.0));
	float d = island_hash(cell + vec2(1.0, 1.0));
	return mix(mix(a, b, blend.x), mix(c, d, blend.x), blend.y);
}

// Three octaves. Four is not visibly better at gameplay distance and costs a
// third more on the software renderer the local capture loop uses.
float island_fbm(vec2 p) {
	float total = 0.0;
	float amplitude = 0.5;
	vec2 sample_point = p;
	for (int octave = 0; octave < 3; octave++) {
		total += island_value_noise(sample_point) * amplitude;
		sample_point *= 2.03;
		amplitude *= 0.5;
	}
	return total;
}

// The rim term the whole library shares: a view-facing fresnel that lifts a
// silhouette off the background. This is the single reason a blockout cube
// reads as an intended object rather than an untextured box.
float island_rim(vec3 normal, vec3 view, float sharpness) {
	return pow(1.0 - clamp(dot(normalize(normal), normalize(view)), 0.0, 1.0), sharpness);
}

// A top-down light wrap. Upward-facing surfaces pick up sky colour even where
// no light reaches them, which is what makes an overcast island read as an
// island rather than a set of flat facets.
float island_sky_wrap(vec3 world_normal) {
	return clamp(world_normal.y * 0.5 + 0.5, 0.0, 1.0);
}
