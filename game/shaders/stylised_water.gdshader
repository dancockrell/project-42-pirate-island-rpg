shader_type spatial;
render_mode blend_mix, depth_draw_always, cull_back, diffuse_burley, specular_schlick_ggx;

// Sea and river. One shader, two setups: the sea is wide, slow and deep, the
// river is narrow, fast and shallow, and the difference is uniforms rather
// than a second shader.
//
// Two-tone depth colour, a normal that moves with the global wind, and shore
// foam measured against the depth buffer. The ripple normal comes from a
// seamless NoiseTexture2D generated in-repo by FastNoiseLite -- declared in
// the material resource, never a file on disk, so nothing here is an asset of
// uncertain provenance and nothing has to be baked.
//
// Renderer split. The depth-buffer foam and the two-tone depth ramp run on
// both methods. Screen-space refraction runs on Forward+ only: on
// gl_compatibility the whole colour buffer is resolved for every transparent
// draw that asks for it, which is the wrong price for a body of water that
// covers a third of the frame, so the compatibility path pays for the same
// read with a shallow-water lightening instead. Both paths are guarded with
// the engine's own renderer preprocessor macros.

global uniform float wind;
global uniform float wind_heading;
global uniform float motion_scale;

// The moving surface. Seamless procedural noise; see the material resource.
uniform sampler2D ripple_noise : repeat_enable, filter_linear_mipmap;

// The two tones. Shallow is what the water is over sand and stone; deep is
// what it is over nothing.
uniform vec4 shallow_color : source_color = vec4(0.176, 0.588, 0.596, 1.0);
uniform vec4 deep_color : source_color = vec4(0.031, 0.145, 0.180, 1.0);
// How many metres of water it takes to reach the deep tone.
uniform float depth_fade_distance : hint_range(0.05, 40.0) = 6.5;

// Foam. A bright broken band against everything the water meets, plus a
// slower wash further out, so a shoreline does not read as a drawn outline.
uniform vec4 foam_color : source_color = vec4(0.918, 0.874, 0.792, 1.0);
uniform float foam_width : hint_range(0.0, 6.0) = 0.85;
uniform float foam_outer_width : hint_range(0.0, 12.0) = 2.6;
uniform float foam_sharpness : hint_range(0.2, 6.0) = 1.6;
uniform float foam_noise_scale : hint_range(0.01, 2.0) = 0.22;

// Surface motion. Two ripple layers crossing at an angle: one is a repeating
// pattern, two is water.
uniform float ripple_scale : hint_range(0.005, 2.0) = 0.08;
uniform float ripple_strength : hint_range(0.0, 3.0) = 1.0;
uniform float ripple_speed : hint_range(0.0, 1.0) = 0.035;
// The river turns this up and stops letting the wind steer it.
uniform float flow_speed : hint_range(0.0, 1.0) = 0.0;
uniform float flow_heading_degrees : hint_range(-360.0, 360.0) = 0.0;
uniform bool follows_wind = true;

uniform float water_roughness : hint_range(0.0, 1.0) = 0.06;
uniform float water_metallic : hint_range(0.0, 1.0) = 0.15;
uniform vec4 sun_glint_color : source_color = vec4(1.0, 0.90, 0.72, 1.0);
uniform float sun_glint_strength : hint_range(0.0, 4.0) = 0.6;
uniform float surface_alpha : hint_range(0.0, 1.0) = 0.92;

uniform sampler2D depth_texture : hint_depth_texture, repeat_disable, filter_nearest;

#if CURRENT_RENDERER == RENDERER_FORWARD_PLUS
uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_linear_mipmap;
uniform float refraction_strength : hint_range(0.0, 0.2) = 0.035;
#endif

varying vec3 world_position;

vec2 drift_offset() {
	float heading = follows_wind ? wind_heading : flow_heading_degrees;
	float radians_heading = radians(heading);
	float speed = ((follows_wind ? wind * ripple_speed : 0.0) + flow_speed) * motion_scale;
	return vec2(sin(radians_heading), cos(radians_heading)) * speed * TIME;
}

// Two crossing layers of the same texture. The second is rotated and moves at
// a different rate, so the pair does not beat.
float ripple_height(vec2 uv, vec2 drift) {
	float a = texture(ripple_noise, uv - drift).r;
	float b = texture(ripple_noise, uv * 1.73 + vec2(drift.y, -drift.x) * 0.63 + vec2(0.37, 0.11)).r;
	return (a + b) * 0.5;
}

// Metres of water between this fragment and whatever is behind it, read off
// the depth buffer and reconstructed in view space.
float water_thickness(vec2 screen_uv, vec3 view_position, mat4 inverse_projection) {
	float raw = texture(depth_texture, screen_uv).r;
	vec3 ndc = vec3(screen_uv * 2.0 - 1.0, raw);
	vec4 behind = inverse_projection * vec4(ndc, 1.0);
	behind.xyz /= behind.w;
	return max(0.0, -behind.z + view_position.z);
}

void vertex() {
	world_position = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 drift = drift_offset();
	vec2 base_uv = world_position.xz * ripple_scale;

	float height = ripple_height(base_uv, drift);
	// Finite differences on the same field, so the bump and the highlight
	// agree with each other.
	float step_size = 0.02;
	float hx = ripple_height(base_uv + vec2(step_size, 0.0), drift);
	float hz = ripple_height(base_uv + vec2(0.0, step_size), drift);
	vec3 ripple_normal = normalize(vec3((height - hx) * ripple_strength * 12.0, 1.0, (height - hz) * ripple_strength * 12.0));
	NORMAL = normalize((VIEW_MATRIX * vec4(ripple_normal, 0.0)).xyz);

	float thickness = water_thickness(SCREEN_UV, VERTEX, INV_PROJECTION_MATRIX);
	float depth_mix = clamp(thickness / depth_fade_distance, 0.0, 1.0);
	vec3 body = mix(shallow_color.rgb, deep_color.rgb, depth_mix);

	float foam_noise = texture(ripple_noise, world_position.xz * foam_noise_scale - drift * 0.5).r;
	float near_band = 1.0 - smoothstep(0.0, max(foam_width, 0.0001), thickness);
	float outer_band = 1.0 - smoothstep(0.0, max(foam_outer_width, 0.0001), thickness);
	float foam = clamp(pow(near_band, foam_sharpness) * (0.55 + foam_noise * 0.9) + outer_band * outer_band * 0.22, 0.0, 1.0);

	vec3 surface = mix(body, foam_color.rgb, foam);

#if CURRENT_RENDERER == RENDERER_FORWARD_PLUS
	// Refraction: the scene behind, pushed by the ripple normal, faded out
	// under the foam so a bright band never smears.
	vec2 refracted_uv = SCREEN_UV + ripple_normal.xz * refraction_strength * (1.0 - depth_mix);
	vec3 behind = texture(screen_texture, clamp(refracted_uv, vec2(0.001), vec2(0.999))).rgb;
	surface = mix(mix(behind * shallow_color.rgb * 1.6, surface, clamp(depth_mix + 0.35, 0.0, 1.0)), foam_color.rgb, foam);
#else
	// gl_compatibility: no screen copy. Shallow water lightens toward the
	// shallow tone instead of showing what is under it, which reads as clear
	// water at gameplay distance and costs one mix.
	surface = mix(surface, shallow_color.rgb * 1.25, (1.0 - depth_mix) * 0.35 * (1.0 - foam));
#endif

	ALBEDO = surface;
	METALLIC = mix(water_metallic, 0.0, foam);
	ROUGHNESS = mix(water_roughness, 0.85, foam);
	EMISSION = sun_glint_color.rgb * pow(clamp(height, 0.0, 1.0), 6.0) * sun_glint_strength * (1.0 - foam);
	ALPHA = mix(surface_alpha, 1.0, max(foam, depth_mix));
}
