class_name PlaceholderActionPresenter
extends Node

## Development-only visual proof that authored production cues are consumed.
## Final sprites, camera, VFX and audio systems will replace this adapter.

signal cue_presented(cue: Dictionary)

var actor_panel: PanelContainer
var enemy_panel: PanelContainer
var cue_label: Label
var actor_home_position := Vector2.ZERO
var enemy_home_position := Vector2.ZERO
var actor_home_scale := Vector2.ONE
var enemy_home_scale := Vector2.ONE
var actor_tween: Tween
var enemy_tween: Tween

func configure(active_actor: PanelContainer, hostile_actor: PanelContainer, display: Label) -> void:
	actor_panel = active_actor
	enemy_panel = hostile_actor
	cue_label = display
	actor_home_position = actor_panel.position
	enemy_home_position = enemy_panel.position
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
		actor_panel.position = actor_home_position
		actor_panel.scale = actor_home_scale
		actor_panel.modulate = Color.WHITE
	if enemy_panel != null:
		enemy_panel.position = enemy_home_position
		enemy_panel.scale = enemy_home_scale
		enemy_panel.rotation = 0.0
		enemy_panel.modulate = Color.WHITE
		reset_paper_pose(enemy_panel)
	if actor_panel != null:
		reset_paper_pose(actor_panel)

func animate_actor_plane(motion: String, camera_zoom: float, vfx_id: String) -> void:
	if actor_tween != null:
		actor_tween.kill()
	var target_scale := actor_home_scale * clampf(camera_zoom, 0.92, 1.12)
	var target_position := actor_home_position
	var start_scale := actor_panel.scale
	var start_position := actor_panel.position
	var duration := .12
	if motion == "card_to_battle_plane":
		# The rail card is still the roster state. This compact expansion makes
		# the selected woman read as entering the shared plane, without moving a
		# fake duplicate or putting a title card under her feet.
		start_scale = actor_home_scale * .82
		start_position = actor_home_position + Vector2(-38, 26)
		target_scale = actor_home_scale
		duration = .16
	elif motion == "contact_lunge":
		target_position = actor_home_position + Vector2(18, -7)
		duration = .08
	elif motion == "recoil_to_guard":
		target_position = actor_home_position + Vector2(8, -2)
		duration = .10
	elif motion == "battle_plane_to_card":
		target_position = actor_home_position
		target_scale = actor_home_scale * .91
		duration = .18
	actor_panel.position = start_position
	actor_panel.scale = start_scale
	actor_panel.modulate = Color("bffdf3") if vfx_id != "presentation.vfx.none" else Color.WHITE
	actor_tween = actor_panel.create_tween().set_parallel(true)
	actor_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	actor_tween.tween_property(actor_panel, "position", target_position, duration)
	actor_tween.tween_property(actor_panel, "scale", target_scale, duration)

func animate_enemy_reaction(motion: String, shake: float) -> void:
	if enemy_tween != null:
		enemy_tween.kill()
	var target_position := enemy_home_position
	var target_rotation := 0.0
	var target_modulate := Color.WHITE
	var duration := .12
	if motion == "contact_lunge" and shake > 0.0:
		# A single raptor recoils. This preserves the game’s individual-creature
		# power dynamic instead of making enemies look like disposable mob packs.
		target_position = enemy_home_position + Vector2(18, -8)
		target_rotation = deg_to_rad(2.5 * shake)
		target_modulate = Color("ffd6bf")
		duration = .08
	enemy_tween = enemy_panel.create_tween().set_parallel(true)
	enemy_tween.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	enemy_tween.tween_property(enemy_panel, "position", target_position, duration)
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
