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
	cue_label.text = "DUMMY ACTION CUE  •  POSE: %s  •  MOTION: %s  •  CAMERA: %s / %s / %.2fX  •  VFX: %s / %s / %.0fX%.0f%%  •  HIT STOP: %d MS  •  SAFE FRAME: %s" % [pose, motion, camera, camera_mode, camera_zoom, vfx, vfx_anchor, vfx_width, vfx_height, hit_stop, ", ".join(PackedStringArray(frame_subjects))]
	cue_label.tooltip_text = "VFX purpose: %s\nPersistence: %s\nReduced flash: %s\nAsset: %s\nAudio cue: %s" % [vfx_purpose, "persistent" if vfx_persistence == -1 else "%d ms" % vfx_persistence, str(vfx_record.get("reducedFlashMode", "missing")), str(vfx_record.get("assetStatus", "missing")), str(cue.get("audio", "missing_audio"))]
	actor_panel.pivot_offset = actor_panel.size * 0.5
	enemy_panel.pivot_offset = enemy_panel.size * 0.5
	actor_panel.scale = Vector2.ONE * clampf(camera_zoom, 0.92, 1.12)
	actor_panel.modulate = Color("bffdf3") if vfx_id != "presentation.vfx.none" else Color.WHITE
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
