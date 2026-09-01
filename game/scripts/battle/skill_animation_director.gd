class_name SkillAnimationDirector
extends Node

## Presentation timing only. It consumes authored animation beats and emits
## signals for sprites, VFX, camera and audio. It never applies combat state.

signal action_started(skill_id: String, duration_ms: int)
signal beat_started(skill_id: String, beat_index: int, beat: Dictionary)
signal action_finished(skill_id: String)

var playing := false
var playback_serial := 0

func play(skill_record: Dictionary) -> void:
	playback_serial += 1
	var serial := playback_serial
	var skill_id := str(skill_record.get("id", ""))
	var animation: Dictionary = skill_record.get("animation", {})
	var duration_ms := int(animation.get("durationMs", 0))
	var beats: Array = animation.get("beats", [])
	playing = true
	action_started.emit(skill_id, duration_ms)
	var elapsed_ms := 0
	for index in beats.size():
		if serial != playback_serial:
			return
		var beat: Dictionary = beats[index]
		var beat_ms := int(beat.get("atMs", elapsed_ms))
		var wait_ms := maxi(0, beat_ms - elapsed_ms)
		if wait_ms > 0:
			await get_tree().create_timer(float(wait_ms) / 1000.0).timeout
		elapsed_ms = beat_ms
		beat_started.emit(skill_id, index, beat.duplicate(true))
	if serial != playback_serial:
		return
	var tail_ms := maxi(0, duration_ms - elapsed_ms)
	if tail_ms > 0:
		await get_tree().create_timer(float(tail_ms) / 1000.0).timeout
	if serial == playback_serial:
		playing = false
		action_finished.emit(skill_id)

func cancel() -> void:
	playback_serial += 1
	playing = false
