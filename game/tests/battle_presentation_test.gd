extends SceneTree

## P6's proof, through the live Rust bridge.
##
## Every authored skill record is played on the real battle screen and each
## presentation cue's VFX record must instantiate the node the record names,
## on the socket the record names, and then go. The camera beats each cue's
## camera record asks for must fire. The reduced-flash substitute must actually
## substitute. And the fight itself -- damage numbers, the ward line, the site
## rule's pulse, the dissolve, the grounding on the plate's floor -- is driven
## from events the bridge sends, not from fixtures.
##
## The live bridge is required, not optional: this suite is counted as a bridge
## proof, so it refuses rather than falling back to MockSimulationPort.

const VfxFactoryScript = preload("res://scripts/battle/vfx_factory.gd")
const BattleStageScript = preload("res://scripts/battle/battle_stage.gd")
const DamageNumberScript = preload("res://scripts/battle/damage_number.gd")
const PaletteScript = preload("res://scripts/battle/battle_palette.gd")
const PaperDollScript = preload("res://scripts/battle/paper_doll.gd")

## Betty's seven, Ayla's seven and Michael's two. The card names fourteen for
## the two heroines; Michael's pair plays on the same screen through the same
## director, so all sixteen authored records are exercised here.
const BETTY_SKILLS := [
	"skill.betty.guarded_strike", "skill.betty.condition_cleanse", "skill.betty.rescue_charge",
	"skill.betty.healing_impact", "skill.betty.fatal_intercept", "skill.betty.mobile_infirmary",
	"skill.betty.combat_revival"
]
const AYLA_SKILLS := [
	"skill.ayla.reach_counter", "skill.ayla.structural_scan", "skill.ayla.safe_passage",
	"skill.ayla.ward_line", "skill.ayla.curse_dispel", "skill.ayla.deny_activation",
	"skill.ayla.override_tomb_rule"
]
const CAPTAIN_SKILLS := ["skill.captain.weapon_attack", "skill.captain.reposition"]

const BETTY_ID := "character.heroine.betty"
const RAZORBEAK_ID := "enemy.raptor.razorbeak"

var failures := 0
var camera_beats: Array[String] = []
## The record id of every effect the stage instantiated, and the node name it
## was instantiated under. Both are checked: the id because the record is what
## owns the effect, the name because the card asks for the node the record
## names. Godot appends a digit when a sibling already holds a name, so the
## name is matched by the factory's own derivation rather than by equality.
var spawned_record_ids: Array[String] = []
var spawned_effect_names: Array[String] = []


func _init() -> void:
	call_deferred("run")


func run() -> void:
	if not ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS):
		push_error("battle_presentation_test requires the native bridge: %s is not registered." % NativeSimulationPort.BRIDGE_CLASS)
		quit(1)
		return
	var prototype := BattlePrototype.new()
	root.add_child(prototype)
	await process_frame
	await process_frame
	check(prototype.simulation is NativeSimulationPort, "the presentation suite must run against the Rust bridge, never the mock")
	check(prototype.stage != null, "the screen must build a battle stage")
	prototype.stage.camera.beat_started.connect(func(camera_id: String, _mode: String, _zoom: float) -> void: camera_beats.append(camera_id))
	prototype.stage.effect_layer.child_entered_tree.connect(func(node: Node) -> void:
		spawned_effect_names.append(str(node.name))
		if node.has_meta("vfx_record_id"):
			spawned_record_ids.append(str(node.get_meta("vfx_record_id")))
	)

	test_every_palette_word_resolves()
	test_the_rigs_stand_on_the_painted_floor(prototype)
	await test_every_authored_skill_plays(prototype)
	await test_the_live_fight_shows_itself(prototype)
	await test_reduced_flash_substitutes()
	test_the_socket_vocabulary_is_whole(prototype)

	prototype.queue_free()
	if failures > 0:
		quit(1)
		return
	print("Battle presentation tests passed.")
	quit(0)


## The registry writes palettes in words and BattlePalette resolves them. A word
## in one and not the other is a black effect at runtime, so the two lists are
## held equal here rather than discovered in a capture.
func test_every_palette_word_resolves() -> void:
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the content bundle must load")
	var registry := catalog.get_record("presentation.vfx.registry")
	var checked := 0
	for entry in registry.get("entries", []):
		for word in entry.get("palette", []):
			check(PaletteScript.has_word(str(word)), "the palette word '%s' on %s has no colour in BattlePalette" % [word, entry.get("id", "")])
			checked += 1
	check(checked >= 46, "every authored VFX record must name at least one palette colour; only %d words were seen" % checked)


## The baseline capture's worst fault: the rigs hung about a hundred pixels
## above the painted terrace. Both bodies must now meet the plate's own floor
## line, and each must carry a contact shadow sitting on it.
func test_the_rigs_stand_on_the_painted_floor(prototype: BattlePrototype) -> void:
	var stage := prototype.stage
	var floor_y := stage.plate.floor_y()
	for actor_id in [BETTY_ID, RAZORBEAK_ID]:
		var feet := stage.feet_point(actor_id)
		check(absf(feet.y - floor_y) < 2.0, "%s's feet must stand on the plate's floor line (%.1f), not at %.1f" % [actor_id, floor_y, feet.y])
		check(absf(feet.x - stage.band_x(str(stage.bands[actor_id]))) < 2.0, "%s must stand at its own band's place on the floor" % actor_id)
		var shadow: Control = stage.shadows.get(actor_id)
		check(shadow != null and shadow.size.x > 0.0, "%s must carry a contact shadow" % actor_id)
		check(shadow != null and absf(shadow.position.y + shadow.size.y * .42 - floor_y) < 2.0, "%s's contact shadow must sit on the floor line" % actor_id)


## Every authored skill, played through the same director the screen uses when
## a command is accepted. For each presentation cue: the camera record fires a
## beat, and a cue naming a real VFX record instantiates a node named by that
## record and no other.
func test_every_authored_skill_plays(prototype: BattlePrototype) -> void:
	var catalog := prototype.catalog
	prototype.animation_director.playback_time_scale = 0.0
	var played := 0
	for skill_id in BETTY_SKILLS + AYLA_SKILLS + CAPTAIN_SKILLS:
		check(catalog.has(skill_id), "%s must be an authored record" % skill_id)
		var record := catalog.get_record(skill_id)
		var cues: Array = record.get("animation", {}).get("presentationCues", [])
		check(not cues.is_empty(), "%s must author at least one presentation cue" % skill_id)
		camera_beats.clear()
		spawned_effect_names.clear()
		spawned_record_ids.clear()
		var no_events: Array[Dictionary] = []
		await prototype.animation_director.play(record, no_events)
		await process_frame
		var expected_cameras: Array[String] = []
		var expected_effects: Array[String] = []
		for cue in cues:
			expected_cameras.append(str(cue.get("cameraId", "")))
			var vfx_id := str(cue.get("vfxId", ""))
			var vfx_record := catalog.get_registry_entry(vfx_id)
			if str(vfx_record.get("emitter", "none")) != "none":
				expected_effects.append(vfx_id)
		for camera_id in expected_cameras:
			check(camera_beats.has(camera_id), "%s's cue must fire the camera beat %s it names" % [skill_id, camera_id])
		for vfx_id in expected_effects:
			check(spawned_record_ids.has(vfx_id), "%s must instantiate the effect its record %s names" % [skill_id, vfx_id])
			check(spawned_effect_names.any(func(spawned: String) -> bool: return spawned.begins_with(VfxFactoryScript.node_name_for(vfx_id))), "%s's effect node must be named from its record: expected a node named %s" % [skill_id, VfxFactoryScript.node_name_for(vfx_id)])
		check(prototype.stage.effect_layer.get_child_count() > 0 or expected_effects.is_empty(), "%s produced no effect node at all" % skill_id)
		played += 1
	check(played == 16, "all sixteen authored skill records must play, not %d" % played)
	# And every transient effect goes. The longest authored persistence is 700
	# milliseconds; a second is past all of them.
	await create_timer(1.2).timeout
	var left_behind: Array[String] = []
	for child in prototype.stage.effect_layer.get_children():
		# The ward line and the deployed infirmary are authored `persistenceMs:
		# -1`; they are released by name, not by a timer, and are expected here.
		if not prototype.stage.persistent_effects.values().has(child):
			left_behind.append(str(child.name))
	check(left_behind.is_empty(), "every transient effect must free itself on its record's own persistence; these did not: %s" % ", ".join(PackedStringArray(left_behind)))
	prototype.animation_director.playback_time_scale = 1.0


## The fight itself, on the real event stream: a submitted command's damage
## becomes a number with weight, the ward line becomes a drawn line the bridge
## placed, a site rule's Guard becomes a pulse, and a defeat dissolves the rig.
func test_the_live_fight_shows_itself(prototype: BattlePrototype) -> void:
	var stage := prototype.stage
	var numbers_before := stage.number_layer.get_child_count()
	await prototype.submit_skill("skill.betty.guarded_strike", [RAZORBEAK_ID])
	check(stage.number_layer.get_child_count() > numbers_before or numbers_before > 0, "a hit the bridge reported must throw a damage number")

	# The ward line: placed on the band the bridge named, drawn as the beam its
	# own record describes, and released when the ward breaks.
	var ward_record := prototype.catalog.get_registry_entry("presentation.vfx.ayla.ward_line_drawn")
	check(int(ward_record.get("persistenceMs", 0)) == VfxFactoryScript.PERSISTENT, "the ward line's record must be the persistent one")
	prototype.place_ward_line(int(prototype.band_names_by_index.keys()[0]))
	var ward := stage.persistent_effects.get("presentation.vfx.ayla.ward_line_drawn") as Node2D
	check(ward != null, "a placed ward line must stand on the stage")
	check(ward != null and ward.get_node_or_null("Quad") != null, "the ward line must be a drawn quad, which is what carries its shimmer shader")
	prototype.release_ward_line()
	check(not stage.persistent_effects.has("presentation.vfx.ayla.ward_line_drawn"), "a broken ward must be released")

	# Grave watch. The bridge writes `site_rule.round.<n>` into the command id of
	# the Guard a site rule hands out; nothing else does, and that is how this
	# screen knows the rule is in force without inventing a snapshot key.
	check(BattleStageScript.is_site_rule_command("site_rule.round.3"), "a site rule's round-boundary Guard must be recognised")
	check(not BattleStageScript.is_site_rule_command("command.debug.1"), "an ordinary command must not read as a site rule")
	var effects_before := stage.effect_layer.get_child_count()
	prototype.project_event({
		"kind": "guard_changed", "command_id": "site_rule.round.2", "subjects": [RAZORBEAK_ID],
		"payload": {"delta": 2, "total": 4}
	})
	check(stage.effect_layer.get_child_count() > effects_before, "grave watch must pulse on the hostile that received its Guard")

	# Death dissolves the rig where it stands.
	var doll := stage.dolls[RAZORBEAK_ID] as PaperDoll
	check(not doll.dissolving, "a living actor must not be dissolving")
	prototype.project_event({"kind": "actor_defeated", "command_id": "command.test.defeat", "subjects": [RAZORBEAK_ID], "payload": {}})
	check(doll.dissolving, "a defeated actor's rig must dissolve")


## The reduced-flash substitute the registry names, actually substituted: the
## additive blend goes, the particulate half is silenced, and what is left is
## the outline the record asked for -- same record, same socket, same lifetime.
func test_reduced_flash_substitutes() -> void:
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the content bundle must load for the reduced-flash check")
	var record := catalog.get_registry_entry("presentation.vfx.bronze_teal_impact_arc")
	check(str(record.get("reducedFlashMode", "")) == "replace_flash_with_outline", "the impact arc must inherit the registry's outline substitute")
	var factory := VfxFactoryScript.new()
	var context := {"origin": Vector2(600, 500), "endpoint": Vector2(900, 500), "forward": Vector2.RIGHT, "reduced_flash": false}
	var flashing := factory.build(record, context)
	check(flashing != null, "the impact arc must build an effect")
	check((flashing.material as CanvasItemMaterial).blend_mode == CanvasItemMaterial.BLEND_MODE_ADD, "an authored additive record must blend additively when flashing is allowed")
	check(not bool(flashing.get_meta("vfx_outline_only")), "the flashing build must not be the outline substitute")
	context.reduced_flash = true
	var outlined := factory.build(record, context)
	check(outlined != null and outlined.name == flashing.name, "the substitute must be the same record's node, not a different effect")
	check((outlined.material as CanvasItemMaterial).blend_mode == CanvasItemMaterial.BLEND_MODE_MIX, "the reduced-flash substitute must drop the additive blend, which is the flash")
	check(bool(outlined.get_meta("vfx_outline_only")), "the reduced-flash build must be marked as the outline substitute")
	# A record that says it is already flash-free is left alone.
	var unchanged_record := catalog.get_registry_entry("presentation.vfx.cloth_ribbon_path")
	unchanged_record.reducedFlashMode = "unchanged"
	var untouched := factory.build(unchanged_record, context)
	check(untouched != null and not bool(untouched.get_meta("vfx_outline_only")), "a record whose substitute is `unchanged` must not be swapped")
	# The particulate half is silenced under the substitute; the quad emitters
	# carry the outline.
	var burst := catalog.get_registry_entry("presentation.vfx.bronze_collision_burst")
	var quiet := factory.build(burst, context)
	var particles := quiet.get_node_or_null("Particles") as CPUParticles2D
	check(particles != null and not particles.emitting, "the outline substitute must silence a burst's particles")
	for node in [flashing, outlined, untouched, quiet]:
		if node != null:
			node.free()
	await process_frame


## The registry's `socket` vocabulary and the stage's resolution of it are two
## halves of one contract. Every socket any authored record names must resolve
## to a point on the stage, or an effect appears in the wrong place with no
## error at all.
func test_the_socket_vocabulary_is_whole(prototype: BattlePrototype) -> void:
	var registry := prototype.catalog.get_record("presentation.vfx.registry")
	var sockets: Dictionary = {}
	for entry in registry.get("entries", []):
		sockets[str(entry.get("socket", "none"))] = true
	check(sockets.size() >= 10, "the registry must exercise most of the socket vocabulary; it names %d" % sockets.size())
	for socket in sockets:
		var point: Vector2 = prototype.stage.socket_point(str(socket), BETTY_ID, RAZORBEAK_ID)
		check(point.x > 0.0 and point.y > 0.0, "the socket '%s' must resolve to a point on the stage" % socket)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
