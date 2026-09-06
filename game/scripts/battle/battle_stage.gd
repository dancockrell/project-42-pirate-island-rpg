class_name BattleStage
extends Control

## The shared plane the fight happens on, and the one thing that knows where
## anything is.
##
## Everything positional lives here so that no effect, no camera beat and no
## number has to guess: the plate and its floor line, the five bands' places on
## that floor, the actors standing on them, their contact shadows, and the
## resolution of a VFX record's `socket` to a point. The screen above it owns
## the HUD; the simulation owns the fight. This owns the geography.
##
## Nothing here reads a skill id. A cue names a camera record and a VFX record;
## this plays those records.

const PaperStageScript = preload("res://scripts/battle/paper_stage.gd")
const PaperDollScript = preload("res://scripts/battle/paper_doll.gd")
const VfxFactoryScript = preload("res://scripts/battle/vfx_factory.gd")
const CameraRigScript = preload("res://scripts/battle/battle_camera_rig.gd")
const DamageNumberScript = preload("res://scripts/battle/damage_number.gd")
const PaletteScript = preload("res://scripts/battle/battle_palette.gd")
## The one door onto the palette (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

## The five bands A5 named, placed across the plate's floor. These fractions are
## the stage's only opinion about where a band is; the names are the bridge's.
const BAND_X := {
	"party_rear": 0.13,
	"party_front": 0.30,
	"contested": 0.50,
	"enemy_front": 0.70,
	"enemy_rear": 0.87
}
## An actor with no stage body of its own -- everyone but Betty and the hostile,
## until the models arrive -- resolves its sockets to its band's floor point
## raised by this much, which is roughly chest height on the rigs that do exist.
const BODYLESS_CHEST_HEIGHT := 190.0
## The contact shadow: an ellipse under each rig, on the plate's floor line.
## Without it the rigs float, which is exactly what the baseline capture showed.
## It is painted in the grammar's darkest ground rather than a stated near-black,
## so a stage in high contrast casts a shadow the same contrast as everything
## else on it.
const SHADOW_HEIGHT_RATIO := 0.20
const SHADOW_ALPHA := 0.88
const SHADOW_TOKEN := "night"
## How far an actor's body slides when its band changes, and how long it takes.
const BAND_MOVE_SECONDS := 0.34
## Grave watch: the site rule the tomb carries. When the battle is under it,
## every living hostile takes a visible pulse at the round boundary, because the
## rule is what regenerates their Guard and the player must be able to see it.
const GRAVE_WATCH_PULSE_SECONDS := 0.55
const GRAVE_WATCH_RING_SIZE := Vector2(260.0, 96.0)
## The sequence prefix the bridge writes into `command_id` when a site rule --
## not an actor -- changed a Guard pool at the round boundary. Reading that is
## how this screen knows a rule fired without inventing a snapshot key.
const SITE_RULE_COMMAND_PREFIX := "site_rule.round."

var plate: PaperStage
var shadow_layer: Control
var actor_layer: Control
var effect_layer: Control
var number_layer: Control
var camera: BattleCameraRig
var factory := VfxFactoryScript.new()
var reduced_motion := false
var reduced_flash := false

## Actor id -> the PanelContainer that carries its rig, for the two actors that
## have one. Everyone else is resolved from their band.
var bodies: Dictionary = {}
var dolls: Dictionary = {}
var shadows: Dictionary = {}
var bands: Dictionary = {}
## Persistent effects (`persistenceMs == -1`), keyed by record id so a second
## placement replaces the first rather than stacking two ward lines.
var persistent_effects: Dictionary = {}


func build(motion_reduced: bool, flash_reduced: bool) -> void:
	name = "BattleStage"
	reduced_motion = motion_reduced
	reduced_flash = flash_reduced
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	plate = PaperStageScript.new()
	plate.name = "PaperStage"
	plate.mouse_filter = Control.MOUSE_FILTER_IGNORE
	plate.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(plate)
	shadow_layer = make_layer("ShadowLayer")
	actor_layer = make_layer("ActorLayer")
	effect_layer = make_layer("EffectLayer")
	number_layer = make_layer("NumberLayer")
	camera = CameraRigScript.new()
	camera.name = "BattleCameraRig"
	add_child(camera)
	camera.configure(self, reduced_motion)
	factory.theme = stage_theme()
	plate.repaint(factory.theme)


## The grammar this stage is drawn in. The stage is a child of the battle
## screen, which wears the Theme, so this is the Theme in force where it stands.
func stage_theme() -> Theme:
	return ThemeTokensScript.active(self)


## The screen above rebuilt the grammar. Every effect the factory makes from
## here takes the new palette; the plate, the shadows and the ring take it now.
func repaint(rebuilt: Theme) -> void:
	factory.theme = rebuilt
	if plate != null:
		plate.repaint(rebuilt)
	var shadow_colour := ThemeTokensScript.color(rebuilt, SHADOW_TOKEN)
	for actor_id in shadows:
		var shadow := shadows[actor_id] as TextureRect
		if shadow != null:
			shadow.texture = contact_shadow_texture(shadow_colour, SHADOW_ALPHA)
	for actor_id in dolls:
		var doll := dolls[actor_id] as PaperDoll
		if doll != null:
			doll.repaint(rebuilt)


func make_layer(layer_name: String) -> Control:
	var layer := Control.new()
	layer.name = layer_name
	layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(layer)
	return layer


## Put an actor's body on the stage. The panel keeps being the interaction owner
## -- targeting still clicks it -- and the stage owns where it stands.
func add_body(actor_id: String, panel: PanelContainer, doll: PaperDoll, band: String) -> void:
	bodies[actor_id] = panel
	dolls[actor_id] = doll
	bands[actor_id] = band
	actor_layer.add_child(panel)
	var shadow := TextureRect.new()
	shadow.name = "ContactShadow_" + actor_id.replace(".", "_")
	shadow.mouse_filter = Control.MOUSE_FILTER_IGNORE
	shadow.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	shadow.stretch_mode = TextureRect.STRETCH_SCALE
	shadow.texture = contact_shadow_texture(ThemeTokensScript.color(stage_theme(), SHADOW_TOKEN), SHADOW_ALPHA)
	shadow_layer.add_child(shadow)
	shadows[actor_id] = shadow
	call_deferred("ground_actor", actor_id, false)


## Stand an actor on the plate's floor line at its band. This is the fix for the
## baseline capture's worst fault: the rigs were positioned from their panels'
## own geometry and hung about a hundred pixels above the painted stone.
func ground_actor(actor_id: String, animated: bool) -> void:
	var panel := bodies.get(actor_id) as Control
	var doll := dolls.get(actor_id) as PaperDoll
	if panel == null or doll == null or plate == null:
		return
	var ground := doll.ground_offset()
	var destination := Vector2(band_x(str(bands.get(actor_id, "contested"))), plate.floor_y()) - ground - doll.position
	if animated and not reduced_motion:
		var tween := panel.create_tween()
		tween.set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_IN_OUT)
		tween.tween_property(panel, "position", destination, BAND_MOVE_SECONDS)
		tween.finished.connect(func() -> void: place_shadow(actor_id), CONNECT_ONE_SHOT)
	else:
		panel.position = destination
	place_shadow(actor_id)
	# The plate's own light at the actor's feet is what bleeds into the rig.
	doll.set_plate_bleed(plate.plate_tint(Vector2(
		clampf(band_x(str(bands.get(actor_id, "contested"))) / maxf(size.x, 1.0), 0.05, 0.95),
		clampf((plate.floor_y() - 120.0) / maxf(size.y, 1.0), 0.05, 0.95)
	)))


func place_shadow(actor_id: String) -> void:
	var shadow := shadows.get(actor_id) as Control
	var doll := dolls.get(actor_id) as PaperDoll
	var panel := bodies.get(actor_id) as Control
	if shadow == null or doll == null or panel == null:
		return
	var width := doll.ground_width() * 2.0
	var height := width * SHADOW_HEIGHT_RATIO
	var centre := panel.position + doll.position + doll.ground_offset()
	shadow.size = Vector2(width, height)
	shadow.position = centre - Vector2(width * .5, height * .42)


## An actor changed band. Its body slides, its shadow follows, and the rail card
## above moves separately because the rail is the roster and this is the floor.
func move_to_band(actor_id: String, band: String) -> void:
	bands[actor_id] = band
	if bodies.has(actor_id):
		ground_actor(actor_id, true)


func band_x(band: String) -> float:
	return size.x * float(BAND_X.get(band, 0.5))


## The boundary between a band and the next one enemyward -- where a ward line
## is drawn. `band_order` is the screen's own reading order, which is also the
## floor's left-to-right order.
func band_boundary_x(band: String, band_order: Array) -> float:
	var index := band_order.find(band)
	if index < 0:
		return size.x * .5
	var next: String = str(band_order[mini(index + 1, band_order.size() - 1)])
	return (band_x(band) + band_x(next)) * .5


## The forward direction for an actor: hostiles face the party, the party faces
## the hostiles. One rule, read from the faction the bridge sent.
func facing(actor_id: String, hostile: bool) -> Vector2:
	return Vector2.LEFT if hostile else Vector2.RIGHT


## Resolve one of the fourteen sockets `vfx.registry.json` may name. Everything
## the factory needs to place an effect comes through here, so a record's socket
## is the only thing that decides where its effect appears.
func socket_point(socket: String, actor_id: String, target_id: String) -> Vector2:
	match socket:
		"none": return Vector2(size.x * .5, plate.floor_y())
		"actor_body": return body_point(actor_id, "torso")
		"actor_main_hand": return body_point(actor_id, "mainHand")
		"actor_weapon_head": return body_point(actor_id, "weaponHead")
		"actor_feet": return feet_point(actor_id)
		"actor_overhead": return body_point(actor_id, "statusOverhead")
		"target_body": return body_point(target_id, "torso")
		"target_impact": return body_point(target_id, "biteImpact")
		"target_feet": return feet_point(target_id)
		"target_overhead": return body_point(target_id, "statusOverhead")
		"contact_midpoint": return (body_point(actor_id, "mainHand") + body_point(target_id, "torso")) * .5
		"ground_contact":
			var contact := (body_point(actor_id, "mainHand") + body_point(target_id, "torso")) * .5
			return Vector2(contact.x, plate.floor_y())
		"band_boundary": return Vector2(size.x * .5, plate.floor_y() - BODYLESS_CHEST_HEIGHT)
		"party_rail": return Vector2(size.x * .18, size.y * .90)
	push_error("The battle stage cannot resolve the VFX socket '%s'. The registry's closed vocabulary and this stage must stay equal." % socket)
	return Vector2(size.x * .5, plate.floor_y())


## A named point on an actor. An actor with a rig uses the rig's own registered
## attachment anchor; an actor that is only a rail card so far resolves to its
## band's floor, at chest height, which is honest about where it stands.
func body_point(actor_id: String, anchor_name: String) -> Vector2:
	var panel := bodies.get(actor_id) as Control
	var doll := dolls.get(actor_id) as PaperDoll
	if panel == null or doll == null:
		return Vector2(band_x(str(bands.get(actor_id, "contested"))), plate.floor_y() - BODYLESS_CHEST_HEIGHT)
	if anchor_name == "torso":
		# The middle of the figure, between its feet and the point its status
		# marks hang from -- not the registry's `rootPivot`, which sits down at
		# the hips and put every body-anchored effect around an actor's knees.
		return (feet_point(actor_id) + body_point(actor_id, "statusOverhead")) * .5
	return panel.position + doll.position + doll.anchor_point(anchor_name)


func feet_point(actor_id: String) -> Vector2:
	var panel := bodies.get(actor_id) as Control
	var doll := dolls.get(actor_id) as PaperDoll
	if panel == null or doll == null:
		return Vector2(band_x(str(bands.get(actor_id, "contested"))), plate.floor_y())
	return panel.position + doll.position + doll.ground_offset()


## Play one authored presentation cue: its camera record, its VFX record, its
## hit-stop and its shake. The cue is the whole input.
func play_cue(cue: Dictionary, actor_id: String, target_id: String, hostile_actor: bool) -> Node2D:
	var camera_record: Dictionary = cue.get("cameraRecord", {})
	if camera != null and not camera_record.is_empty():
		camera.push(camera_record, lead_point(camera_record, actor_id, target_id))
	var effect := spawn_effect(cue.get("vfxRecord", {}), actor_id, target_id, hostile_actor)
	if camera != null:
		camera.shake(float(cue.get("shake", 0.0)))
	return effect


## The stage point a camera record's `lead` names. Anything the registry can
## lead on is either an actor, the contact between them, or the whole plane.
func lead_point(camera_record: Dictionary, actor_id: String, target_id: String) -> Vector2:
	match str(camera_record.get("lead", "")):
		"betty", "ayla", "active_actor", "hands", "ally": return body_point(actor_id, "torso")
		"target", "patient": return body_point(target_id, "torso")
		"contact_point": return (body_point(actor_id, "mainHand") + body_point(target_id, "torso")) * .5
		"midpoint": return (body_point(actor_id, "torso") + body_point(target_id, "torso")) * .5
		_: return Vector2(size.x * .5, size.y * .5)


## Build a record's effect and put it on the stage. A record with `emitter:
## none` yields nothing, which is what `presentation.vfx.none` means. A record
## with `persistenceMs == -1` stays until it is released by name.
func spawn_effect(record: Dictionary, actor_id: String, target_id: String, hostile_actor: bool) -> Node2D:
	if record.is_empty() or effect_layer == null:
		return null
	var socket := str(record.get("socket", "none"))
	var origin := socket_point(socket, actor_id, target_id)
	var endpoint := body_point(target_id, "torso")
	if socket == "band_boundary":
		endpoint = Vector2(origin.x, plate.floor_y())
		origin = Vector2(origin.x, plate.floor_y() - BODYLESS_CHEST_HEIGHT * 2.0)
	var effect := factory.build(record, {
		"origin": origin,
		"endpoint": endpoint,
		"forward": facing(actor_id, hostile_actor),
		"reduced_flash": reduced_flash
	})
	if effect == null:
		return null
	var record_id := str(record.get("id", ""))
	var lifetime := int(record.get("persistenceMs", 0))
	if lifetime == VfxFactoryScript.PERSISTENT:
		release_effect(record_id)
		persistent_effects[record_id] = effect
	add_effect(effect)
	animate_effect(effect, lifetime)
	return effect


## Put an effect on the layer under the name its record gives it. Two instances
## of the same record can legitimately overlap -- two hits landing inside one
## action -- and Godot resolves a name collision by discarding the name
## entirely, so the second instance takes a numbered form of the same name
## rather than an anonymous one. The name stays derivable from the record.
func add_effect(effect: Node2D) -> void:
	var base := str(effect.name)
	if effect_layer.has_node(NodePath(base)):
		var serial := 2
		while effect_layer.has_node(NodePath("%s_%d" % [base, serial])):
			serial += 1
		effect.name = "%s_%d" % [base, serial]
	effect_layer.add_child(effect)


## Drive the record's own envelope: a quad's `progress` uniform runs 0 to 1 over
## the record's persistence, then the node frees itself. A persistent record
## settles instead and waits to be released.
func animate_effect(effect: Node2D, lifetime_ms: int) -> void:
	var quad := effect.get_node_or_null("Quad") as ColorRect
	var seconds := maxf(0.08, float(lifetime_ms) / 1000.0) if lifetime_ms > 0 else 0.9
	if quad != null and quad.material is ShaderMaterial:
		var material := quad.material as ShaderMaterial
		var tween := effect.create_tween()
		tween.set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_OUT)
		tween.tween_method(func(value: float) -> void: material.set_shader_parameter("progress", value), 0.0, 1.0, seconds)
		if lifetime_ms == VfxFactoryScript.PERSISTENT:
			tween.tween_property(effect, "modulate:a", VfxFactoryScript.PERSISTENT_SETTLED_ALPHA, 0.2)
	if lifetime_ms == VfxFactoryScript.PERSISTENT:
		return
	var life := effect.create_tween()
	life.tween_interval(seconds)
	life.tween_callback(effect.queue_free)


## Release a persistent effect by its record id -- the ward line when it breaks,
## the deployed infirmary when its last pulse is spent.
func release_effect(record_id: String) -> void:
	var existing := persistent_effects.get(record_id) as Node
	if existing != null and is_instance_valid(existing):
		existing.queue_free()
	persistent_effects.erase(record_id)


func release_all_effects() -> void:
	for record_id in persistent_effects.keys():
		release_effect(str(record_id))


## The ward line: a real drawn line between two bands, shimmering, built from
## `presentation.vfx.ayla.ward_line_drawn` like every other effect. The band the
## bridge sent decides where it stands; nothing about it is hand-placed.
func place_ward_line(record: Dictionary, band: String, band_order: Array) -> Node2D:
	if record.is_empty():
		return null
	var boundary := band_boundary_x(band, band_order)
	var effect := factory.build(record, {
		"origin": Vector2(boundary, plate.floor_y() - BODYLESS_CHEST_HEIGHT * 2.2),
		"endpoint": Vector2(boundary, plate.floor_y()),
		"forward": Vector2.DOWN,
		"reduced_flash": reduced_flash
	})
	if effect == null:
		return null
	release_effect(str(record.get("id", "")))
	persistent_effects[str(record.get("id", ""))] = effect
	add_effect(effect)
	animate_effect(effect, VfxFactoryScript.PERSISTENT)
	return effect


## The pool of shade a figure stands in: a radial gradient texture, not a
## shader and not a `_draw` call. Both of those were tried and neither renders
## on this project's declared compatibility path -- a flat magenta shader on
## the figure and a full-strength black ellipse under it both came back from
## the capture invisible -- and a texture drawn by a TextureRect renders
## anywhere the engine runs at all.
static func contact_shadow_texture(color: Color, strength: float) -> GradientTexture2D:
	var ramp := Gradient.new()
	ramp.offsets = PackedFloat32Array([0.0, 0.38, 1.0])
	ramp.colors = PackedColorArray([
		# not a colour: the caller's own colour at three opacities.
		Color(color.r, color.g, color.b, strength),
		Color(color.r, color.g, color.b, strength * 0.50),
		Color(color.r, color.g, color.b, 0.0)
	])
	var texture := GradientTexture2D.new()
	texture.gradient = ramp
	texture.fill = GradientTexture2D.FILL_RADIAL
	texture.fill_from = Vector2(0.5, 0.5)
	texture.fill_to = Vector2(1.0, 0.5)
	texture.width = 128
	texture.height = 128
	return texture


## Grave watch, made visible. The bridge does not put site rules in the snapshot,
## so this screen does not invent a key for them: it reads the `command_id` the
## bridge already writes on the Guard the rule hands out at the round boundary
## (`site_rule.round.<n>`), and pulses the hostiles that received it.
static func is_site_rule_command(command_id: String) -> bool:
	return command_id.begins_with(SITE_RULE_COMMAND_PREFIX)


func pulse_site_rule(actor_id: String) -> void:
	if effect_layer == null:
		return
	var ring := ColorRect.new()
	ring.name = "SiteRulePulse_" + actor_id.replace(".", "_")
	ring.mouse_filter = Control.MOUSE_FILTER_IGNORE
	ring.size = GRAVE_WATCH_RING_SIZE
	var at := feet_point(actor_id)
	ring.position = at - GRAVE_WATCH_RING_SIZE * .5
	var material := ShaderMaterial.new()
	material.shader = factory.shader_for("ring")
	material.set_shader_parameter("core_color", PaletteScript.word_color(stage_theme(), "bronze"))
	material.set_shader_parameter("edge_color", PaletteScript.word_color(stage_theme(), "teal"))
	material.set_shader_parameter("progress", 0.0)
	material.set_shader_parameter("outline_only", 1.0 if reduced_flash else 0.0)
	material.set_shader_parameter("inward", 0.0)
	ring.material = material
	effect_layer.add_child(ring)
	var tween := ring.create_tween()
	tween.tween_method(func(value: float) -> void: material.set_shader_parameter("progress", value), 0.0, 1.0, GRAVE_WATCH_PULSE_SECONDS)
	tween.tween_callback(ring.queue_free)


## A number with weight, thrown away from the blow.
func show_number(variant: String, amount: int, actor_id: String, from_actor_id: String) -> DamageNumber:
	if number_layer == null:
		return null
	var number := DamageNumberScript.new()
	number.name = "Number_" + variant
	number.configure(stage_theme(), variant, amount)
	number.position = body_point(actor_id, "torso") + Vector2(0, -60)
	number_layer.add_child(number)
	var drift := (body_point(actor_id, "torso") - body_point(from_actor_id, "torso")).normalized()
	if drift == Vector2.ZERO:
		drift = Vector2.RIGHT
	number.play(drift, reduced_motion)
	return number


## Death: the rig burns away where it stands.
func dissolve_actor(actor_id: String) -> void:
	var doll := dolls.get(actor_id) as PaperDoll
	if doll != null:
		doll.dissolve(reduced_motion)
	var shadow := shadows.get(actor_id) as Control
	if shadow != null:
		var tween := shadow.create_tween()
		tween.tween_property(shadow, "modulate:a", 0.0, PaperDollScript.DISSOLVE_SECONDS)


## The frame returns to the wide after an action. The camera record of the last
## beat owns how long that takes.
func settle_camera(camera_record: Dictionary) -> void:
	if camera != null:
		camera.settle(camera_record)
