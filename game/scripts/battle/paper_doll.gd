class_name PaperDoll
extends Control

## Deliberately artificial development actor. Geometry and anchors are the
## production contract; painted art will replace the drawing, not its API.
##
## P6's lighting pass, in three parts, none of which knows a skill:
##
##   * a warm backlight halo drawn behind the figure, on the side the terrace's
##     key light comes from, so the rig sits in the plate's golden hour instead
##     of on top of it;
##   * a vertical shade and the plate's own sampled colour, applied per rig
##     piece, so the legs stand in the terrace's shade while the head keeps the
##     sky and every figure is tinted by the light it is standing in;
##   * a dissolve for death, as a shader on the rig root that every piece
##     renders through.
##
## The first was a CanvasGroup composite with an edge-detecting rim shader.
## That renders nothing at all on the compatibility path this project declares
## (`renderer/rendering_method="gl_compatibility"`), verified by putting a flat
## magenta shader on the group and capturing an unchanged figure, so the group
## is gone and the light is drawn instead of sampled.

const PaperBettyRigScript = preload("res://scripts/battle/paper_betty_rig.gd")
const PaperRazorbeakRigScript = preload("res://scripts/battle/paper_razorbeak_rig.gd")
const PaletteScript = preload("res://scripts/battle/battle_palette.gd")
## The one door onto the palette (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

## The terrace's key light comes from the sky behind and above the gate, so the
## rim falls on the upper right of every figure. One direction for every actor:
## two figures lit from two directions is the tell that they were pasted on.
const KEY_DIRECTION := Vector2(0.58, -0.82)
## How much of the plate's colour is allowed into the rig. Enough that a figure
## belongs to the golden hour it is standing in; not so much that Betty's teal
## corset turns brown. Named here because it is the number to tune when the
## plate is repainted.
const PLATE_BLEED_AMOUNT := 0.22
## The lower body sits in the terrace's own shade; the head and shoulders keep
## the sky. This is the vertical falloff across the figure.
const GROUND_SHADE_STRENGTH := 0.30
const RIM_STRENGTH := 0.90
## How far the backlight halo is pushed along the key direction, as a fraction
## of the figure's height, and how wide it spreads.
const RIM_OFFSET_RATIO := 0.10
const RIM_SPREAD_RATIO := 0.62
## How long a defeated actor takes to go. Long enough to be read as a death
## rather than a disappearance.
const DISSOLVE_SECONDS := 0.85

## Anchor dots are available in the inspector/tooltip contract, while the
## running game keeps the silhouette clean. Toggle this locally only while
## laying out a replacement sprite sheet.
var show_anchors := false

var spec: Dictionary = {}
# not a colour: the stage calls `configure` with the actor's accent before the
# first draw; this is what the rig wears if it is ever drawn without one.
var accent := Color.WHITE
var pose_name := "card_ready"
var betty_rig: PaperBettyRig
var razorbeak_rig
var lighting: ShaderMaterial
var backlight: TextureRect
# not a colour: replaced by `set_plate_bleed` with the painting's own light.
var plate_bleed := Color.WHITE
var dissolving := false


func set_presentation_cue(cue: Dictionary) -> void:
	pose_name = str(cue.get("pose", "card_ready"))
	if betty_rig != null:
		# Each authored beat moves the same articulated hierarchy. This is the
		# temporary animation implementation, not a static key-art swap.
		betty_rig.animate_to_pose(pose_name, str(cue.get("motion", "")))
	queue_redraw()


func reset_presentation() -> void:
	pose_name = "card_ready"
	if betty_rig != null:
		betty_rig.set_pose("ready_idle")
	if razorbeak_rig != null:
		razorbeak_rig.set_pose("ready_idle")
	queue_redraw()


func configure(next_spec: Dictionary, next_accent: Color) -> void:
	spec = next_spec.duplicate(true)
	accent = next_accent
	custom_minimum_size = Vector2(spec.get("nativeCanvas", [520, 560])[0], spec.get("nativeCanvas", [520, 560])[1])
	tooltip_text = metadata_tooltip()
	build_lighting_material()
	backlight = build_backlight()
	add_child(backlight)
	if spec.get("id", "") == "presentation.paper_doll.betty.active":
		betty_rig = PaperBettyRigScript.new()
		betty_rig.name = "BettyArticulatedPaperRig"
		betty_rig.material = lighting
		add_child(betty_rig)
		call_deferred("rebuild_betty_rig")
	elif spec.get("id", "") == "presentation.paper_doll.razorbeak.active":
		razorbeak_rig = PaperRazorbeakRigScript.new()
		razorbeak_rig.name = "RazorbeakArticulatedPaperRig"
		razorbeak_rig.material = lighting
		add_child(razorbeak_rig)
		call_deferred("rebuild_razorbeak_rig")
	queue_redraw()


## The plate's colour and the death dissolve, as one shader on the rig root
## that every piece renders through by `use_parent_material`. It reads the
## primitive's own colour rather than a texture, because a cut-paper rig draws
## lines and polygons and has no texture to sample.
func build_lighting_material() -> ShaderMaterial:
	var shader := Shader.new()
	shader.code = """shader_type canvas_item;
uniform vec4 bleed_color : source_color = vec4(0.72, 0.54, 0.29, 1.0);
uniform float bleed_amount = 0.22;
uniform float dissolve = 0.0;
uniform vec4 dissolve_color : source_color = vec4(0.94, 0.62, 0.32, 1.0);

void fragment() {
	vec3 rgb = COLOR.rgb;
	rgb = mix(rgb, rgb * bleed_color.rgb * 2.0, bleed_amount);
	float alpha = COLOR.a;
	if (dissolve > 0.001) {
		// A deterministic hash of the screen cell, not a random source: the
		// same pixel always burns away at the same moment, so a capture of a
		// death is reproducible.
		vec2 cell = floor(FRAGCOORD.xy / 7.0);
		float grain = fract(sin(dot(cell, vec2(12.9898, 78.233))) * 43758.5453);
		float burn = clamp((grain - dissolve) * 7.0, 0.0, 1.0);
		rgb = mix(dissolve_color.rgb, rgb, burn);
		alpha *= step(dissolve, grain);
	}
	COLOR = vec4(rgb, alpha);
}
"""
	lighting = ShaderMaterial.new()
	lighting.shader = shader
	lighting.set_shader_parameter("bleed_color", plate_bleed)
	lighting.set_shader_parameter("bleed_amount", PLATE_BLEED_AMOUNT)
	lighting.set_shader_parameter("dissolve", 0.0)
	lighting.set_shader_parameter("dissolve_color", PaletteScript.word_color(ThemeTokensScript.active(self), "gold"))
	return lighting


## The grammar changed. The only colour this rig takes from it is the gold the
## dissolve burns in; the accent is the actor's and the bleed is the painting's.
func repaint(rebuilt: Theme) -> void:
	if lighting != null:
		lighting.set_shader_parameter("dissolve_color", PaletteScript.word_color(rebuilt, "gold"))
	queue_redraw()


## The stage hands each doll the colour of the plate directly behind it, so the
## bleed is the painting's own light rather than a value invented here.
func set_plate_bleed(color: Color) -> void:
	plate_bleed = color
	if lighting != null:
		lighting.set_shader_parameter("bleed_color", color)
	place_backlight()


## Death. The rig burns away over DISSOLVE_SECONDS and stays gone; reduced
## motion skips the burn and cuts to the empty frame, which is the same
## information without the animation.
func dissolve(reduced_motion: bool) -> void:
	if dissolving or lighting == null:
		return
	dissolving = true
	if reduced_motion:
		lighting.set_shader_parameter("dissolve", 1.0)
		return
	var tween := create_tween()
	tween.set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN)
	tween.tween_method(set_dissolve, 0.0, 1.0, DISSOLVE_SECONDS)


func set_dissolve(value: float) -> void:
	if lighting != null:
		lighting.set_shader_parameter("dissolve", value)


func _notification(what: int) -> void:
	if what != NOTIFICATION_RESIZED:
		return
	if betty_rig != null:
		rebuild_betty_rig()
	if razorbeak_rig != null:
		rebuild_razorbeak_rig()


func rebuild_betty_rig() -> void:
	if betty_rig != null and size.x > 0.0 and size.y > 0.0:
		betty_rig.build(size)
		betty_rig.set_pose(pose_name)
		betty_rig.apply_lighting(GROUND_SHADE_STRENGTH)
		place_backlight()


func rebuild_razorbeak_rig() -> void:
	if razorbeak_rig != null and size.x > 0.0 and size.y > 0.0:
		razorbeak_rig.build(size)
		razorbeak_rig.apply_lighting(GROUND_SHADE_STRENGTH)
		place_backlight()


## Where this doll's feet are, in its own coordinates. The stage grounds every
## actor on the plate's floor line from this, which is why the rigs stopped
## floating: the number comes from the rig that is actually drawn rather than
## from the panel's height.
func ground_offset() -> Vector2:
	if betty_rig != null:
		return betty_rig.ground_point()
	if razorbeak_rig != null:
		return razorbeak_rig.ground_point()
	var anchor: Array = spec.get("groundAnchor", [0.5, 0.9])
	return Vector2(size.x * float(anchor[0]), size.y * float(anchor[1]))


## The doll's own width at the ground, used to size its contact shadow.
func ground_width() -> float:
	if betty_rig != null:
		return betty_rig.ground_width()
	if razorbeak_rig != null:
		return razorbeak_rig.ground_width()
	return size.x * .3


## A named attachment anchor in doll coordinates, from the registry's own
## normalised pair. The VFX factory resolves a record's socket through the
## stage, which asks this; no effect ever hard-codes a position.
func anchor_point(anchor_name: String) -> Vector2:
	var anchors: Dictionary = spec.get("attachmentAnchors", {})
	if not anchors.has(anchor_name):
		return Vector2(size.x * .5, size.y * .5)
	var point: Array = anchors[anchor_name]
	return Vector2(float(point[0]) * size.x, float(point[1]) * size.y)


func set_reaction(motion: String, shake: float) -> void:
	if razorbeak_rig != null:
		razorbeak_rig.animate_reaction(motion, shake)


func metadata_tooltip() -> String:
	return "PAPER DOLL — DEVELOPMENT ONLY\nStable presentation: %s\nSubject: %s\nFuture asset: %s\nCanvas: %s\nSafe padding: %s%%\nGround anchor: %s\nRoot pivot: %s\n\nIdentity invariants:\n• %s\n\nReplacement tests:\n• %s" % [spec.get("id", "missing"), spec.get("subjectId", "missing"), spec.get("futureRuntimeAssetId", "missing"), str(spec.get("nativeCanvas", [])), str(spec.get("safePaddingPercent", 0)), str(spec.get("groundAnchor", [])), str(spec.get("rootPivot", [])), "\n• ".join(PackedStringArray(spec.get("identityInvariants", []))), "\n• ".join(PackedStringArray(spec.get("replacementTests", [])))]


## The terrace's key light, behind the figure: a warm radial texture placed
## under the rig, on the side the plate's light comes from. A `_draw` halo and a
## silhouette-sampling shader were both tried first and neither renders on this
## project's declared compatibility path; a texture does.
func build_backlight() -> TextureRect:
	var ramp := Gradient.new()
	ramp.offsets = PackedFloat32Array([0.0, 0.45, 1.0])
	ramp.colors = PackedColorArray([
		# not a colour: the halo is white light at three opacities, tinted by the
		# plate's own bleed where it is placed.
		Color(1.0, 1.0, 1.0, RIM_STRENGTH * 0.30),
		Color(1.0, 1.0, 1.0, RIM_STRENGTH * 0.13),
		Color(1.0, 1.0, 1.0, 0.0)
	])
	var texture := GradientTexture2D.new()
	texture.gradient = ramp
	texture.fill = GradientTexture2D.FILL_RADIAL
	texture.fill_from = Vector2(0.5, 0.5)
	texture.fill_to = Vector2(1.0, 0.5)
	texture.width = 128
	texture.height = 128
	var halo := TextureRect.new()
	halo.name = "KeyLightHalo"
	halo.mouse_filter = Control.MOUSE_FILTER_IGNORE
	halo.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	halo.stretch_mode = TextureRect.STRETCH_SCALE
	halo.texture = texture
	# Additive, so the halo is light behind the figure rather than fog in front
	# of the plate.
	var material := CanvasItemMaterial.new()
	material.blend_mode = CanvasItemMaterial.BLEND_MODE_ADD
	halo.material = material
	return halo


## Place the halo from the rig's own footprint, and tint it with the plate's
## light at this actor's feet.
func place_backlight() -> void:
	if backlight == null:
		return
	var ground := ground_offset()
	if ground.y <= 0.0:
		return
	var height := ground.y * RIM_SPREAD_RATIO
	var extent := Vector2(height * 1.16, height * 1.72)
	var centre := Vector2(ground.x, ground.y - height * .78) + KEY_DIRECTION * (ground.y * RIM_OFFSET_RATIO)
	backlight.size = extent
	backlight.position = centre - extent * .5
	backlight.modulate = plate_bleed.lightened(.42)


func _draw() -> void:
	# The complete asset contract lives in the tooltip. Safe-frame geometry is
	# deliberately invisible in the running game; it appears only during an
	# explicit layout inspection so the battle plane never reads as boxed-in.
	#
	# Three drawing paths were removed here in this pass rather than left
	# beside the rigs: `draw_betty` had no caller at all once the articulated
	# rig landed, `draw_raptor` was unreachable behind `razorbeak_rig == null`,
	# and `draw_presentation_vfx` hand-drew three specific effect ids -- exactly
	# the per-skill hand-coded effect this card forbids. `vfx_factory.gd` builds
	# every effect from its record now, so the third had to go for the first
	# one to be true.
	if spec.is_empty() or not show_anchors:
		return
	var pad := float(spec.get("safePaddingPercent", 8)) / 100.0
	draw_rect(Rect2(size.x * pad, size.y * pad, size.x * (1.0 - pad * 2.0), size.y * (1.0 - pad * 2.0)), Color(accent, 0.55), false, 2.0)
	for key in spec.get("attachmentAnchors", {}):
		var point := anchor_point(str(key))
		var marker := ThemeTokensScript.color(ThemeTokensScript.active(self), "focus")
		draw_circle(point, 4.0, marker)
		draw_string(ThemeDB.fallback_font, point + Vector2(7, -5), str(key), HORIZONTAL_ALIGNMENT_LEFT, -1, 10, marker.lightened(0.25))
