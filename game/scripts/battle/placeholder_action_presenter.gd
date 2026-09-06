class_name PlaceholderActionPresenter
extends Node

## The one door onto the palette (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

## The rigs' reaction to one authored presentation cue: the pose the doll takes,
## the lunge or recoil of the panel it stands in, and the cue's technical
## contract on a tooltip for the art pass.
##
## It is one half of consuming a cue and it stays that half. `battle_stage.gd`
## owns the other -- the camera beat, the effect the VFX record builds, the
## hit-stop and the shake -- because those are properties of the stage rather
## than of a body. Both are handed the same cue and neither reads a skill id.

signal cue_presented(cue: Dictionary)

var actor_panel: PanelContainer
var enemy_panel: PanelContainer
var cue_label: Label
## How much weight a lunge puts into the body. The camera's zoom is not reused
## here: `battle_camera_rig.gd` owns the frame, and scaling the actor by the
## camera record's zoom as well was the same beat played twice.
const LUNGE_SCALE := 1.06
const ENTRY_SCALE := 0.86

## The presenter never owns where an actor stands -- `battle_stage.gd` does,
## and it moves actors between bands underneath this. So a cue's displacement
## is kept as an offset from wherever the stage has put the body, applied as a
## delta, and returned to zero on reset.
var actor_offset := Vector2.ZERO
var enemy_offset := Vector2.ZERO
var actor_home_scale := Vector2.ONE
var enemy_home_scale := Vector2.ONE
var actor_tween: Tween
var enemy_tween: Tween

func configure(active_actor: PanelContainer, hostile_actor: PanelContainer, display: Label) -> void:
	actor_panel = active_actor
	enemy_panel = hostile_actor
	cue_label = display
	actor_home_scale = actor_panel.scale
	enemy_home_scale = enemy_panel.scale

func present(cue: Dictionary) -> void:
	if actor_panel == null or enemy_panel == null or cue_label == null:
		return
	var pose := str(cue.get("pose", "missing_pose")).replace("_", " ").to_upper()
	var raw_motion := str(cue.get("motion", "missing_motion"))
	var motion := raw_motion.replace("_", " ").to_upper()
	var vfx_record: Dictionary = cue.get("vfxRecord", {})
	var vfx_id := str(cue.get("vfxId", "presentation.vfx.none"))
	var vfx := vfx_id.trim_prefix("presentation.vfx.").replace("_", " ").to_upper()
	var vfx_purpose := str(vfx_record.get("purpose", "missing purpose"))
	var vfx_anchor := str(vfx_record.get("anchor", "missing_anchor")).replace("_", " ").to_upper()
	var vfx_width := float(vfx_record.get("envelopeWidthPercent", 0.0))
	var vfx_height := float(vfx_record.get("envelopeHeightPercent", 0.0))
	var vfx_persistence := int(vfx_record.get("persistenceMs", 0))
	var camera_record: Dictionary = cue.get("cameraRecord", {})
	var camera_id := str(cue.get("cameraId", "missing_camera"))
	var camera := camera_id.trim_prefix("presentation.camera.").replace("_", " ").to_upper()
	var camera_mode := str(camera_record.get("mode", "missing_mode")).to_upper()
	var camera_zoom := float(camera_record.get("zoom", 1.0))
	var hit_stop := int(cue.get("hitStopMs", 0))
	var shake := float(cue.get("shake", 0.0))
	var frame_subjects: Array = cue.get("frameSubjects", [])
	# The game announces the action in player language. Every technical detail
	# remains on the control tooltip for the art and animation pass.
	cue_label.text = "%s  •  %s" % [pose, vfx]
	cue_label.tooltip_text = "Production cue\nPose: %s\nMotion: %s\nCamera: %s / %s / %.2fx\nVFX: %s at %s (%.0fx%.0f%%)\nVFX purpose: %s\nPersistence: %s\nHit stop: %d ms\nSafe frame: %s\nReduced flash: %s\nAsset: %s\nAudio cue: %s" % [pose, motion, camera, camera_mode, camera_zoom, vfx, vfx_anchor, vfx_width, vfx_height, vfx_purpose, "persistent" if vfx_persistence == -1 else "%d ms" % vfx_persistence, hit_stop, ", ".join(PackedStringArray(frame_subjects)), str(vfx_record.get("reducedFlashMode", "missing")), str(vfx_record.get("assetStatus", "missing")), str(cue.get("audio", "missing_audio"))]
	actor_panel.pivot_offset = actor_panel.size * 0.5
	enemy_panel.pivot_offset = enemy_panel.size * 0.5
	animate_actor_plane(raw_motion, camera_zoom, vfx_id)
	animate_enemy_reaction(raw_motion, shake)
	set_paper_pose(actor_panel, cue)
	cue_presented.emit(cue.duplicate(true))

func reset() -> void:
	if actor_tween != null:
		actor_tween.kill()
	if enemy_tween != null:
		enemy_tween.kill()
	if actor_panel != null:
		set_actor_offset(Vector2.ZERO)
		actor_panel.scale = actor_home_scale
		# not a colour: no tint.
		actor_panel.modulate = Color.WHITE
	if enemy_panel != null:
		set_enemy_offset(Vector2.ZERO)
		enemy_panel.scale = enemy_home_scale
		enemy_panel.rotation = 0.0
		# not a colour: no tint.
		enemy_panel.modulate = Color.WHITE
		reset_paper_pose(enemy_panel)
	if actor_panel != null:
		reset_paper_pose(actor_panel)

func set_actor_offset(offset: Vector2) -> void:
	if actor_panel == null:
		return
	actor_panel.position += offset - actor_offset
	actor_offset = offset


func set_enemy_offset(offset: Vector2) -> void:
	if enemy_panel == null:
		return
	enemy_panel.position += offset - enemy_offset
	enemy_offset = offset


func animate_actor_plane(motion: String, camera_zoom: float, vfx_id: String) -> void:
	if actor_tween != null:
		actor_tween.kill()
	var target_scale := actor_home_scale
	var target_offset := Vector2.ZERO
	var start_scale := actor_panel.scale
	var duration := .12
	if motion == "card_to_battle_plane":
		# The rail card is still the roster state. This compact expansion makes
		# the selected woman read as entering the shared plane, without moving a
		# fake duplicate or putting a title card under her feet.
		start_scale = actor_home_scale * ENTRY_SCALE
		duration = .16
	elif motion == "contact_lunge":
		target_offset = Vector2(18, -7)
		target_scale = actor_home_scale * LUNGE_SCALE
		duration = .08
	elif motion == "recoil_to_guard":
		target_offset = Vector2(8, -2)
		duration = .10
	elif motion == "battle_plane_to_card":
		target_scale = actor_home_scale * .96
		duration = .18
	actor_panel.scale = start_scale
	# The acting figure is lifted into the effect's own light for the beat. The
	# tint is the grammar's teal at a whisper rather than a pale green stated
	# here, so an actor lit under high contrast is lit in that palette.
	var lift := ThemeTokensScript.color(ThemeTokensScript.active(actor_panel), "teal").lightened(0.82)
	# not a colour: no tint, for a beat with no effect on it.
	actor_panel.modulate = lift if vfx_id != "presentation.vfx.none" else Color.WHITE
	actor_tween = actor_panel.create_tween().set_parallel(true)
	actor_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	actor_tween.tween_method(set_actor_offset, actor_offset, target_offset, duration)
	actor_tween.tween_property(actor_panel, "scale", target_scale, duration)

func animate_enemy_reaction(motion: String, shake: float) -> void:
	if enemy_tween != null:
		enemy_tween.kill()
	var target_offset := Vector2.ZERO
	var target_rotation := 0.0
	# not a colour: no tint unless the recoil below asks for one.
	var target_modulate := Color.WHITE
	var duration := .12
	if motion == "contact_lunge" and shake > 0.0:
		# A single raptor recoils. This preserves the game’s individual-creature
		# power dynamic instead of making enemies look like disposable mob packs.
		target_offset = Vector2(18, -8)
		target_rotation = deg_to_rad(2.5 * shake)
		target_modulate = ThemeTokensScript.color(ThemeTokensScript.active(enemy_panel), "danger_soft").lightened(0.62)
		duration = .08
	enemy_tween = enemy_panel.create_tween().set_parallel(true)
	enemy_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	enemy_tween.tween_method(set_enemy_offset, enemy_offset, target_offset, duration)
	enemy_tween.tween_property(enemy_panel, "rotation", target_rotation, duration)
	enemy_tween.tween_property(enemy_panel, "modulate", target_modulate, duration)
	set_paper_reaction(enemy_panel, motion, shake)

func set_paper_pose(panel: PanelContainer, cue: Dictionary) -> void:
	if panel == null or panel.get_child_count() == 0:
		return
	var stack := panel.get_child(0)
	if stack.get_child_count() == 0:
		return
	var doll := stack.get_child(0)
	if doll.has_method("set_presentation_cue"):
		doll.set_presentation_cue(cue)

func reset_paper_pose(panel: PanelContainer) -> void:
	if panel == null or panel.get_child_count() == 0:
		return
	var stack := panel.get_child(0)
	if stack.get_child_count() == 0:
		return
	var doll := stack.get_child(0)
	if doll.has_method("reset_presentation"):
		doll.reset_presentation()

func set_paper_reaction(panel: PanelContainer, motion: String, shake: float) -> void:
	if panel == null or panel.get_child_count() == 0:
		return
	var stack := panel.get_child(0)
	if stack.get_child_count() == 0:
		return
	var doll := stack.get_child(0)
	if doll.has_method("set_reaction"):
		doll.set_reaction(motion, shake)
