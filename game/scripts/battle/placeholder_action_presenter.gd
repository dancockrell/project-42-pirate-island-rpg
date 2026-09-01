class_name PlaceholderActionPresenter
extends Node

## Development-only visual proof that authored production cues are consumed.
## Final sprites, camera, VFX and audio systems will replace this adapter.

signal cue_presented(cue: Dictionary)

var actor_panel: PanelContainer
var enemy_panel: PanelContainer
var cue_label: Label

func configure(active_actor: PanelContainer, hostile_actor: PanelContainer, display: Label) -> void:
	actor_panel = active_actor
	enemy_panel = hostile_actor
	cue_label = display

func present(cue: Dictionary) -> void:
	if actor_panel == null or enemy_panel == null or cue_label == null:
		return
	var pose := str(cue.get("pose", "missing_pose")).replace("_", " ").to_upper()
	var motion := str(cue.get("motion", "missing_motion")).replace("_", " ").to_upper()
	var vfx := str(cue.get("vfx", "none")).replace("_", " ").to_upper()
	var camera_record: Dictionary = cue.get("cameraRecord", {})
	var camera_id := str(cue.get("cameraId", "missing_camera"))
	var camera := camera_id.trim_prefix("presentation.camera.").replace("_", " ").to_upper()
	var camera_mode := str(camera_record.get("mode", "missing_mode")).to_upper()
	var camera_zoom := float(camera_record.get("zoom", 1.0))
	var hit_stop := int(cue.get("hitStopMs", 0))
	var shake := float(cue.get("shake", 0.0))
	var frame_subjects: Array = cue.get("frameSubjects", [])
	cue_label.text = "DUMMY ACTION CUE  •  POSE: %s  •  MOTION: %s  •  CAMERA: %s / %s / %.2fX  •  VFX: %s  •  HIT STOP: %d MS  •  SAFE FRAME: %s" % [pose, motion, camera, camera_mode, camera_zoom, vfx, hit_stop, ", ".join(PackedStringArray(frame_subjects))]
	cue_label.tooltip_text = "Audio cue: %s" % str(cue.get("audio", "missing_audio"))
	actor_panel.pivot_offset = actor_panel.size * 0.5
	enemy_panel.pivot_offset = enemy_panel.size * 0.5
	actor_panel.scale = Vector2.ONE * clampf(camera_zoom, 0.92, 1.12)
	actor_panel.modulate = Color("bffdf3") if vfx != "NONE" else Color.WHITE
	enemy_panel.rotation = deg_to_rad(2.5 * shake)
	enemy_panel.modulate = Color("ffd6bf") if shake > 0.0 else Color.WHITE
	cue_presented.emit(cue.duplicate(true))

func reset() -> void:
	if actor_panel != null:
		actor_panel.scale = Vector2.ONE
		actor_panel.modulate = Color.WHITE
	if enemy_panel != null:
		enemy_panel.rotation = 0.0
		enemy_panel.modulate = Color.WHITE
