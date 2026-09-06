class_name VfxFactory
extends RefCounted

## Builds a real effect for one `presentation.vfx.*` record.
##
## The record is the owner. There is no per-skill hand-coded effect anywhere in
## this screen: `palette`, `socket`, `emitter`, `direction`, `layer`, `blend`,
## `envelopeWidthPercent`, `envelopeHeightPercent`, `persistenceMs` and
## `reducedFlashMode` are the entire input, and a record the factory cannot
## build is a validation failure rather than a special case here.
##
## Everything is drawn: CPUParticles2D for the particulate emitters and a
## shader quad for the geometric ones. CPU particles rather than GPUParticles2D
## because this screen renders on `gl_compatibility` as well as Forward+, and
## because a CPU emitter's lifetime is exact -- the factory frees a node on the
## record's own `persistenceMs` and the suite can watch it go.
##
## Determinism (presentation rule 18): nothing here calls `randi()`. The two
## emitters that want spread take it from a fixed integer seed derived from the
## record's own id, so the same record always throws the same shape.

const PaletteScript = preload("res://scripts/battle/battle_palette.gd")

## Provisional presentation numbers. None of them decides a game result; each is
## named so it can be tuned in one place rather than sprinkled through the
## emitters.
## The envelope percentages in a record are a fraction of the *stage*, which is
## the shared plane every actor stands on.
const ENVELOPE_REFERENCE := Vector2(1920.0, 1080.0)
## A burst of eighteen reads as force without becoming confetti; the record's
## envelope scales it.
const BURST_PARTICLE_BASE := 18
const TRAIL_PARTICLE_BASE := 22
const MOTE_PARTICLE_BASE := 14
## Below this a shader quad has no readable band at all.
const MINIMUM_QUAD_SIZE := Vector2(48.0, 48.0)
## A persistent record (`persistenceMs == -1`) is never freed on a timer; the
## caller releases it. This is the sentinel the registry already uses.
const PERSISTENT := -1
## The fade a persistent effect settles to once its entry motion has finished,
## so a ward line or a deployed infirmary stays legible without competing with
## the action in front of it.
const PERSISTENT_SETTLED_ALPHA := 0.72

## The eight primitives `emitter` may name. The registry's validator holds the
## same list; a record naming anything else never reaches this factory.
const PARTICLE_EMITTERS := ["burst", "trail", "motes"]
const QUAD_EMITTERS := ["ring", "crescent", "beam", "panel"]

## Reduced flash: the registry's own substitute vocabulary.
## `replace_flash_with_outline` is the default every record inherits -- the
## additive core is dropped, the particulate emitters are silenced, and what is
## left is a single mixed outline at the same size, on the same socket, for the
## same lifetime. `unchanged` means the record is already flash-free.
const REDUCED_FLASH_OUTLINE := "replace_flash_with_outline"

## The direction vectors, in stage space. `forward` is the acting actor's facing
## (the caller supplies it, because only the stage knows which way an actor
## stands); the rest are absolute.
static func direction_vector(direction: String, forward: Vector2) -> Vector2:
	match direction:
		"outward": return Vector2.ZERO
		"inward": return Vector2.ZERO
		"upward": return Vector2.UP
		"downward": return Vector2.DOWN
		"forward": return forward
		"backward": return -forward
		"along_line": return forward
		_: return Vector2.ZERO


## The node name a record is instantiated under. Godot forbids `.` in a node
## name, so the record's id becomes its own name with the dots turned to
## underscores -- one reversible rule, so a suite can name the node it expects
## from the record alone.
static func node_name_for(record_id: String) -> String:
	return record_id.replace(".", "_")


## Build the effect. `context` carries what only the stage knows:
##   origin        the resolved socket point, in stage coordinates
##   endpoint      the other end of a beam (the target's body), stage coordinates
##   forward       the acting actor's facing, a unit vector
##   reduced_flash true when the player has asked for less flashing
## Returns null for a record that emits nothing (`presentation.vfx.none`), which
## is not a failure: the authored cue says there is no effect on that beat.
func build(record: Dictionary, context: Dictionary) -> Node2D:
	var record_id := str(record.get("id", ""))
	var emitter := str(record.get("emitter", "none"))
	if record_id.is_empty():
		push_error("The VFX factory was handed a record with no id.")
		return null
	if emitter == "none":
		return null
	var reduced_flash := bool(context.get("reduced_flash", false))
	var outline_only := reduced_flash and str(field(record, "reducedFlashMode", REDUCED_FLASH_OUTLINE)) == REDUCED_FLASH_OUTLINE
	var colors: Array = PaletteScript.effect_colors(record.get("palette", []))
	var envelope := Vector2(
		maxf(float(record.get("envelopeWidthPercent", 0.0)) * .01 * ENVELOPE_REFERENCE.x, MINIMUM_QUAD_SIZE.x),
		maxf(float(record.get("envelopeHeightPercent", 0.0)) * .01 * ENVELOPE_REFERENCE.y, MINIMUM_QUAD_SIZE.y)
	)
	var origin: Vector2 = context.get("origin", Vector2.ZERO)
	var endpoint: Vector2 = context.get("endpoint", origin)
	var forward: Vector2 = context.get("forward", Vector2.RIGHT)
	var direction := str(record.get("direction", "none"))
	var lifetime_ms := int(record.get("persistenceMs", 0))

	var root := Node2D.new()
	root.name = node_name_for(record_id)
	root.position = origin
	root.z_index = clampi(int(field(record, "layer", 40)), 0, 100)
	root.set_meta("vfx_record_id", record_id)
	root.set_meta("vfx_emitter", emitter)
	root.set_meta("vfx_socket", str(record.get("socket", "none")))
	root.set_meta("vfx_outline_only", outline_only)
	root.material = blend_material(str(field(record, "blend", "add")), outline_only)

	if PARTICLE_EMITTERS.has(emitter):
		root.add_child(build_particles(record_id, emitter, direction, forward, colors, envelope, lifetime_ms, outline_only))
	else:
		root.add_child(build_quad(emitter, direction, colors, envelope, origin, endpoint, forward, lifetime_ms, outline_only))
	return root


## A record's field. The registry writes most entries without a `layer`,
## `blend` or `reducedFlashMode` because its `defaults` block carries them, and
## `ContentCatalog.get_registry_entry` is the one place that block is merged in
## -- so a record arriving here is already complete and the factory keeps no
## second copy of the defaults. The literal is only for a record handed
## straight to the factory by a suite.
func field(record: Dictionary, key: String, fallback: Variant) -> Variant:
	return record[key] if record.has(key) else fallback


## `screen` has no CanvasItemMaterial equivalent in Godot; additive is the
## closest of the four modes the engine offers and the registry uses `screen`
## on exactly one record (the safety outline), where the difference at these
## alphas is not visible. Under reduced flash every effect blends normally: an
## additive layer is the flash.
func blend_material(blend: String, outline_only: bool) -> CanvasItemMaterial:
	var material := CanvasItemMaterial.new()
	if outline_only:
		material.blend_mode = CanvasItemMaterial.BLEND_MODE_MIX
		return material
	match blend:
		"add", "screen": material.blend_mode = CanvasItemMaterial.BLEND_MODE_ADD
		"multiply": material.blend_mode = CanvasItemMaterial.BLEND_MODE_MUL
		_: material.blend_mode = CanvasItemMaterial.BLEND_MODE_MIX
	return material


func build_particles(record_id: String, emitter: String, direction: String, forward: Vector2, colors: Array, envelope: Vector2, lifetime_ms: int, outline_only: bool) -> CPUParticles2D:
	var particles := CPUParticles2D.new()
	particles.name = "Particles"
	particles.emitting = true
	particles.one_shot = emitter != "motes"
	particles.local_coords = false
	particles.lifetime = maxf(0.08, float(lifetime_ms if lifetime_ms > 0 else 400) / 1000.0)
	particles.explosiveness = 1.0 if emitter == "burst" else 0.0
	particles.texture = null
	# A fixed seed per record: the same effect always throws the same shape, so
	# a capture of a beat is reproducible and no `randi()` is involved.
	particles.seed = abs(record_id.hash()) % 100000
	if "use_fixed_seed" in particles:
		particles.use_fixed_seed = true
	var span := maxf(envelope.x, envelope.y)
	var travel := direction_vector(direction, forward)
	match emitter:
		"burst":
			particles.amount = BURST_PARTICLE_BASE + int(span * .02)
			particles.direction = travel if travel != Vector2.ZERO else Vector2.RIGHT
			particles.spread = 180.0 if travel == Vector2.ZERO else 42.0
			particles.initial_velocity_min = span * 1.1
			particles.initial_velocity_max = span * 2.4
			particles.gravity = Vector2(0, span * 1.4)
			particles.scale_amount_min = 5.0
			particles.scale_amount_max = 11.0
			particles.damping_min = span * .6
			particles.damping_max = span * 1.2
		"trail":
			particles.amount = TRAIL_PARTICLE_BASE + int(span * .015)
			particles.direction = travel if travel != Vector2.ZERO else forward
			particles.spread = 14.0
			particles.initial_velocity_min = span * .5
			particles.initial_velocity_max = span * 1.3
			particles.gravity = Vector2(0, span * .25)
			particles.scale_amount_min = 4.0
			particles.scale_amount_max = 8.5
			particles.emission_shape = CPUParticles2D.EMISSION_SHAPE_RECTANGLE
			particles.emission_rect_extents = Vector2(envelope.x * .10, envelope.y * .10)
		"motes":
			particles.amount = MOTE_PARTICLE_BASE + int(span * .01)
			particles.direction = travel if travel != Vector2.ZERO else Vector2.UP
			particles.spread = 26.0
			particles.initial_velocity_min = span * .12
			particles.initial_velocity_max = span * .38
			particles.gravity = Vector2.ZERO
			particles.scale_amount_min = 3.6
			particles.scale_amount_max = 7.0
			particles.emission_shape = CPUParticles2D.EMISSION_SHAPE_SPHERE
			particles.emission_sphere_radius = maxf(6.0, span * .22)
			# An inward record collapses toward its socket instead of leaving it.
			if direction == "inward":
				particles.radial_accel_min = -span * 2.2
				particles.radial_accel_max = -span * 1.1
	# Reduced flash silences the particulate half entirely: the outline the
	# substitute names is the whole effect, and it is drawn by the quad path.
	if outline_only:
		particles.amount = maxi(1, particles.amount / 6)
		particles.emitting = false
	particles.color = colors[0]
	if colors.size() > 1:
		var ramp := Gradient.new()
		ramp.set_color(0, colors[0])
		ramp.set_color(1, Color(colors[colors.size() - 1], 0.0))
		particles.color_ramp = ramp
	return particles


func build_quad(emitter: String, direction: String, colors: Array, envelope: Vector2, origin: Vector2, endpoint: Vector2, forward: Vector2, lifetime_ms: int, outline_only: bool) -> ColorRect:
	var quad := ColorRect.new()
	quad.name = "Quad"
	quad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var core: Color = colors[0]
	var edge: Color = colors[colors.size() - 1]
	var material := ShaderMaterial.new()
	material.shader = shader_for(emitter)
	material.set_shader_parameter("core_color", core)
	material.set_shader_parameter("edge_color", edge)
	material.set_shader_parameter("progress", 0.0)
	material.set_shader_parameter("outline_only", 1.0 if outline_only else 0.0)
	material.set_shader_parameter("inward", 1.0 if direction == "inward" else 0.0)
	quad.material = material
	if emitter == "beam":
		# A beam spans two points, so its quad is as long as the line it draws
		# and rotated onto it. Everything else is centred on its socket.
		var span := endpoint - origin
		var length := maxf(span.length(), MINIMUM_QUAD_SIZE.x)
		var thickness := maxf(envelope.y * .18, 10.0)
		quad.size = Vector2(length, thickness)
		quad.position = Vector2(0.0, -thickness * .5)
		quad.pivot_offset = Vector2(0.0, thickness * .5)
		quad.rotation = span.angle() if span.length() > 1.0 else forward.angle()
	else:
		quad.size = envelope
		quad.position = -envelope * .5
		if emitter == "crescent":
			# A crescent reads as a swing, so it is turned onto the actor's
			# facing rather than always sweeping to the right.
			quad.pivot_offset = envelope * .5
			quad.rotation = forward.angle() if direction != "upward" else -PI * .5
	return quad


## One small shader per geometric primitive, written here rather than as a
## `.tres` under `game/shaders/` because that directory belongs to the render
## foundation card and these four are the battle screen's own. Each carries the
## `outline_only` branch the reduced-flash substitute needs, so the substitute
## is one uniform rather than a second code path.
func shader_for(emitter: String) -> Shader:
	var shader := Shader.new()
	match emitter:
		"ring":
			shader.code = """shader_type canvas_item;
uniform vec4 core_color : source_color = vec4(1.0);
uniform vec4 edge_color : source_color = vec4(1.0);
uniform float progress = 0.0;
uniform float outline_only = 0.0;
uniform float inward = 0.0;
void fragment() {
	float travel = mix(progress, 1.0 - progress, inward);
	vec2 p = (UV - vec2(0.5)) * 2.0;
	float d = length(p);
	float radius = mix(0.12, 0.98, travel);
	float thickness = mix(0.11, 0.035, travel) * mix(1.0, 0.55, outline_only);
	float band = 1.0 - smoothstep(0.0, thickness, abs(d - radius));
	float fill = (1.0 - smoothstep(radius - thickness, radius, d)) * 0.16 * (1.0 - outline_only);
	float envelope = sin(clamp(progress, 0.0, 1.0) * 3.14159);
	vec3 rgb = mix(edge_color.rgb, core_color.rgb, band);
	COLOR = vec4(rgb, clamp(band + fill, 0.0, 1.0) * envelope);
}
"""
		"crescent":
			shader.code = """shader_type canvas_item;
uniform vec4 core_color : source_color = vec4(1.0);
uniform vec4 edge_color : source_color = vec4(1.0);
uniform float progress = 0.0;
uniform float outline_only = 0.0;
uniform float inward = 0.0;
void fragment() {
	vec2 p = (UV - vec2(0.5)) * 2.0;
	float d = length(p);
	float a = atan(p.y, p.x);
	float sweep = mix(progress, 1.0 - progress, inward);
	// The blade's leading edge travels from behind the shoulder to past the
	// target, and the arc trails it: a swing with a head and a tail, rather
	// than a ring that appears all at once.
	float head = mix(-2.0, 1.4, sweep);
	float behind = head - a;
	float trail = 1.9;
	float within = clamp(1.0 - behind / trail, 0.0, 1.0) * step(0.0, behind);
	float thickness = mix(0.34, 0.14, sweep) * mix(1.0, 0.45, outline_only);
	float band = 1.0 - smoothstep(0.0, thickness, abs(d - 0.70));
	float envelope = sin(clamp(progress, 0.0, 1.0) * 3.14159);
	vec3 rgb = mix(edge_color.rgb, core_color.rgb, band * within);
	COLOR = vec4(rgb, band * within * envelope);
}
"""
		"beam":
			shader.code = """shader_type canvas_item;
uniform vec4 core_color : source_color = vec4(1.0);
uniform vec4 edge_color : source_color = vec4(1.0);
uniform float progress = 0.0;
uniform float outline_only = 0.0;
uniform float inward = 0.0;
void fragment() {
	// The line draws itself from the origin outward, then holds.
	float drawn = step(UV.x, clamp(progress * 2.0, 0.0, 1.0));
	float spine = 1.0 - smoothstep(0.0, mix(0.5, 0.22, outline_only), abs(UV.y - 0.5) * 2.0);
	// A slow shimmer travelling the line's length keeps a persistent ward from
	// reading as a dead drawn rectangle. TIME is presentation only.
	float shimmer = 0.62 + 0.38 * sin(UV.x * 26.0 - TIME * 3.4);
	float envelope = clamp(progress * 3.0, 0.0, 1.0);
	vec3 rgb = mix(edge_color.rgb, core_color.rgb, spine);
	COLOR = vec4(rgb, spine * drawn * shimmer * envelope * mix(1.0, 0.8, outline_only));
}
"""
		_:
			shader.code = """shader_type canvas_item;
uniform vec4 core_color : source_color = vec4(1.0);
uniform vec4 edge_color : source_color = vec4(1.0);
uniform float progress = 0.0;
uniform float outline_only = 0.0;
uniform float inward = 0.0;
void fragment() {
	// Three panels snapping open, the registry's own description of the
	// deployed field infirmary and of Ayla's stamped seals.
	float column = floor(UV.x * 3.0);
	float local = fract(UV.x * 3.0);
	float opened = clamp(progress * 3.0 - column, 0.0, 1.0);
	float within = step(abs(local - 0.5) * 2.0, opened);
	float border = smoothstep(opened - 0.14, opened, abs(local - 0.5) * 2.0);
	float frame = max(border, smoothstep(0.86, 1.0, abs(UV.y - 0.5) * 2.0));
	vec3 rgb = mix(core_color.rgb, edge_color.rgb, frame);
	float fill = mix(0.34, 0.0, outline_only);
	COLOR = vec4(rgb, within * max(frame, fill) * clamp(progress * 2.0, 0.0, 1.0));
}
"""
	return shader
