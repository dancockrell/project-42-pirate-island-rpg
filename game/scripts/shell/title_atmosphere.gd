class_name TitleAtmosphere
extends Control

## The slow weather behind the title.
##
## Everything here is drawn, not imported: a canvas shader for sky, low sun,
## fog banks and sea, and `_draw` for the headland silhouettes, the cutter on
## the horizon and the drifting embers. No mesh, texture or font enters the
## repository for this screen, which is what lets it exist before the art does.
## Card P2 owns the 3D environment; when `origin/lane/P2-render-foundation`
## publishes a scene the title can instance, this treatment is what it
## replaces -- the title screen instances the environment and keeps this only
## as the compatibility fallback.
##
## Determinism: the ember field is laid out by a RandomNumberGenerator seeded
## with the named constant below, never by `randi()`, so the same title screen
## draws the same field on every machine and in every capture.
## Reduced motion (card B9) stops the shader clock and freezes the embers where
## they stand rather than hiding them.

const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

const SEED := 42
const EMBER_COUNT := 44
const EMBER_RISE := 0.021
const HORIZON_Y := 0.60

const SHADER_CODE := """
shader_type canvas_item;

uniform float motion : hint_range(0.0, 1.0) = 1.0;
// Card P11: these four are set from the Theme's palette in `repaint()`, so the
// weather behind the title is painted out of the same tokens the type is and
// high contrast reaches the sky. The values written here are never used --
// `repaint()` runs before the first frame -- and are kept only so the shader
// compiles when read on its own.
uniform vec3 night_color : source_color = vec3(0.0);
uniform vec3 sky_color : source_color = vec3(0.0);
uniform vec3 horizon_color : source_color = vec3(0.0);
uniform vec3 sea_color : source_color = vec3(0.0);
uniform float horizon_y = 0.60;

float band(float x, float phase) {
	return sin(x * 6.2831853 + phase);
}

void fragment() {
	vec2 uv = UV;
	float t = TIME * motion;
	float y = uv.y;

	vec3 color = mix(night_color, sky_color, smoothstep(0.0, horizon_y, y));

	vec2 sun = vec2(0.70, horizon_y);
	float glow = exp(-length((uv - sun) * vec2(1.0, 2.1)) * 3.4);
	color += horizon_color * glow * 1.55;

	float fog = 0.0;
	fog += 0.5 + 0.5 * band(uv.x * 0.6 + t * 0.006, 0.0);
	fog += 0.5 + 0.5 * band(uv.x * 1.1 - t * 0.010, 2.1);
	fog += 0.5 + 0.5 * band(uv.x * 1.9 + t * 0.017, 4.3);
	fog /= 3.0;
	color += horizon_color * fog * exp(-abs(y - (horizon_y - 0.03)) * 22.0) * 0.55;

	float sea = smoothstep(horizon_y - 0.010, horizon_y + 0.055, y);
	vec3 water = mix(sea_color, night_color, smoothstep(horizon_y, 1.0, y));
	float glint = pow(max(0.0, band(uv.x * 2.0 + t * 0.05, y * 210.0)), 12.0);
	water += horizon_color * glint * 0.10 * exp(-(y - horizon_y) * 9.0);
	color = mix(color, water, sea);

	float vignette = length((uv - vec2(0.5, 0.5)) * vec2(1.1, 1.0));
	color *= 1.0 - smoothstep(0.40, 1.00, vignette) * 0.72;

	// The left of the frame is where the type stands, so it is deliberately the
	// quiet side: the weather is dimmed there rather than the type being given a
	// box to sit in.
	color *= 1.0 - (1.0 - smoothstep(0.04, 0.58, uv.x)) * 0.42;

	COLOR = vec4(color, 1.0);
}
"""

## The far ridge, the near headland and the shore, as fractions of the screen.
## They are a silhouette, so they carry no material and need no model: an island
## read at this distance is a shape against a sky. The far ridge closes at the
## waterline rather than at the bottom of the frame, so the band of sea the
## shader draws between the two headlands stays open and the cutter has water to
## stand on.
const FAR_RIDGE: Array[Vector2] = [
	Vector2(0.00, 0.556), Vector2(0.06, 0.498), Vector2(0.13, 0.532),
	Vector2(0.21, 0.442), Vector2(0.28, 0.500), Vector2(0.35, 0.470),
	Vector2(0.44, 0.548), Vector2(0.53, 0.520), Vector2(0.62, 0.572),
	Vector2(0.74, 0.545), Vector2(0.86, 0.578), Vector2(1.00, 0.560)
]
const FAR_RIDGE_WATERLINE := 0.600
const NEAR_RIDGE: Array[Vector2] = [
	Vector2(0.00, 0.735), Vector2(0.09, 0.702), Vector2(0.16, 0.744),
	Vector2(0.24, 0.716), Vector2(0.33, 0.753), Vector2(0.42, 0.722),
	Vector2(0.55, 0.762), Vector2(0.68, 0.729), Vector2(0.80, 0.768),
	Vector2(0.91, 0.738), Vector2(1.00, 0.756)
]
const SHORE: Array[Vector2] = [
	Vector2(0.00, 0.905), Vector2(0.14, 0.880), Vector2(0.29, 0.916),
	Vector2(0.46, 0.893), Vector2(0.63, 0.928), Vector2(0.81, 0.902),
	Vector2(1.00, 0.934)
]

## Where the cutter stands: on the open water between the two headlands, to the
## left of the low sun so it reads as a silhouette against the light.
const CUTTER_AT := Vector2(0.585, 0.652)

var motion_enabled := true

var _background: ColorRect
var _material: ShaderMaterial
var _embers: PackedVector2Array = PackedVector2Array()
var _ember_sizes: PackedFloat32Array = PackedFloat32Array()
var _ember_drift: PackedFloat32Array = PackedFloat32Array()
var _phase := 0.0
## The plate's colours, resolved from the Theme by `repaint()`. Nothing here is
## a hex: the island is drawn out of the same palette the menu is.
# not a colour: five placeholders until `repaint()` runs on the first frame.
var _far_ridge_colour := Color.BLACK
var _near_ridge_colour := Color.BLACK
var _shore_colour := Color.BLACK
var _ink_colour := Color.BLACK
var _ember_colour := Color.BLACK


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	build_background()
	build_embers()
	repaint(ThemeTokensScript.active(self))
	set_process(motion_enabled)


## Take the plate's whole palette from the Theme. The title calls this again
## whenever the grammar is rebuilt, which is how high contrast reaches the sky
## and the headlands and not only the lettering.
func repaint(theme: Theme) -> void:
	var night := ThemeTokensScript.color(theme, "night")
	var deep := ThemeTokensScript.color(theme, "deep")
	var bronze := ThemeTokensScript.color(theme, "bronze")
	if _material != null:
		_material.set_shader_parameter("night_color", night)
		_material.set_shader_parameter("sky_color", deep.lightened(0.06))
		_material.set_shader_parameter("horizon_color", bronze.darkened(0.30))
		_material.set_shader_parameter("sea_color", deep.darkened(0.10))
	if _background != null:
		_background.color = night
	# Three silhouettes, each a step further forward and a step darker, so the
	# island reads as depth rather than as one flat shape.
	_far_ridge_colour = deep
	_near_ridge_colour = night
	_shore_colour = night.darkened(0.50)
	_ink_colour = night.darkened(0.35)
	_ember_colour = bronze
	queue_redraw()


func set_motion_enabled(enabled: bool) -> void:
	motion_enabled = enabled
	if _material != null:
		_material.set_shader_parameter("motion", 1.0 if enabled else 0.0)
	set_process(enabled)


func build_background() -> void:
	var shader := Shader.new()
	shader.code = SHADER_CODE
	_material = ShaderMaterial.new()
	_material.shader = shader
	_material.set_shader_parameter("motion", 1.0 if motion_enabled else 0.0)
	_material.set_shader_parameter("horizon_y", HORIZON_Y)
	_background = ColorRect.new()
	_background.name = "Weather"
	_background.material = _material
	# Painted by `repaint()` from the Theme before the first frame.
	_background.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_background.mouse_filter = Control.MOUSE_FILTER_IGNORE
	# The ridges, the cutter and the embers are this control's own `_draw`, and
	# a child draws over its parent, so the sky is told to stay behind.
	_background.show_behind_parent = true
	add_child(_background)


func build_embers() -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = SEED
	for index in EMBER_COUNT:
		_embers.append(Vector2(rng.randf(), rng.randf_range(0.34, 1.0)))
		_ember_sizes.append(rng.randf_range(0.9, 2.6))
		_ember_drift.append(rng.randf_range(0.55, 1.0))


func _process(delta: float) -> void:
	_phase += delta
	queue_redraw()


func _draw() -> void:
	var box := size
	draw_ridge(FAR_RIDGE, box, _far_ridge_colour, FAR_RIDGE_WATERLINE)
	draw_cutter(box)
	draw_ridge(NEAR_RIDGE, box, _near_ridge_colour, 1.0)
	draw_ridge(SHORE, box, _shore_colour, 1.0)
	draw_embers(box)


## One silhouette, closed at `bottom` as a fraction of the frame's height.
func draw_ridge(profile: Array[Vector2], box: Vector2, color: Color, bottom: float) -> void:
	var points := PackedVector2Array()
	for point in profile:
		points.append(Vector2(point.x * box.x, point.y * box.y))
	points.append(Vector2(box.x, bottom * box.y))
	points.append(Vector2(0.0, bottom * box.y))
	draw_colored_polygon(points, color)


## The cutter Handsome Jack, hull-down on the horizon: the one piece of story
## in the plate, drawn as three lines and a hull so it costs no asset.
func draw_cutter(box: Vector2) -> void:
	var anchor := Vector2(CUTTER_AT.x * box.x, CUTTER_AT.y * box.y)
	var unit := box.y * 0.001
	var ink := _ink_colour
	var hull := PackedVector2Array([
		anchor + Vector2(-26.0 * unit, 0.0),
		anchor + Vector2(26.0 * unit, 0.0),
		anchor + Vector2(19.0 * unit, 7.0 * unit),
		anchor + Vector2(-19.0 * unit, 7.0 * unit)
	])
	draw_colored_polygon(hull, ink)
	draw_line(anchor + Vector2(-7.0 * unit, 0.0), anchor + Vector2(-7.0 * unit, -34.0 * unit), ink, 2.0 * unit)
	draw_line(anchor + Vector2(9.0 * unit, 0.0), anchor + Vector2(9.0 * unit, -26.0 * unit), ink, 2.0 * unit)
	draw_line(anchor + Vector2(-7.0 * unit, -34.0 * unit), anchor + Vector2(9.0 * unit, -26.0 * unit), ink, 1.5 * unit)


## Embers lifting off the shore. They rise, wrap, and are frozen where they
## stand when reduced motion is on.
func draw_embers(box: Vector2) -> void:
	for index in _embers.size():
		var seeded: Vector2 = _embers[index]
		var drift: float = _ember_drift[index]
		var rise := 0.0
		if motion_enabled:
			rise = fposmod(_phase * EMBER_RISE * drift, 1.0)
		var y := fposmod(seeded.y - rise, 1.0)
		var sway: float = (sin((_phase * 0.35 * drift) + seeded.x * 22.0) * 0.004) if motion_enabled else 0.0
		var at := Vector2((seeded.x + sway) * box.x, y * box.y)
		# Embers fade out where the sky is brightest so they never read as dust
		# on the lens, and fade in as they climb.
		var alpha: float = clampf((y - 0.30) * 1.4, 0.0, 1.0) * 0.42
		draw_circle(at, _ember_sizes[index] * (box.y / 1080.0) + 0.6, Color(_ember_colour, alpha))
