class_name SkillAnimationDirector
extends Node

## Presentation timing only. It consumes authored animation beats and emits
## signals for sprites, VFX, camera and audio. It never applies combat state.

signal action_started(skill_id: String, duration_ms: int)
signal beat_started(skill_id: String, beat_index: int, beat: Dictionary)
signal event_cued(skill_id: String, beat_name: String, event: Dictionary)
signal action_finished(skill_id: String)

var playing := false
var playback_serial := 0
var playback_time_scale := 1.0

func play(skill_record: Dictionary, action_events: Array[Dictionary] = []) -> void:
	playback_serial += 1
	var serial := playback_serial
	var skill_id := str(skill_record.get("id", ""))
	var animation: Dictionary = skill_record.get("animation", {})
	var duration_ms := int(animation.get("durationMs", 0))
	var beats: Array = animation.get("beats", [])
	var scheduled_events := schedule_events(animation, action_events)
	var next_event_index := 0
	playing = true
	action_started.emit(skill_id, duration_ms)
	var elapsed_ms := 0
	for index in beats.size():
		if serial != playback_serial:
			return
		var beat: Dictionary = beats[index]
		var beat_ms := int(beat.get("atMs", elapsed_ms))
		var wait_ms := maxi(0, beat_ms - elapsed_ms)
		if wait_ms > 0 and playback_time_scale > 0.0:
			await get_tree().create_timer(float(wait_ms) / 1000.0 * playback_time_scale).timeout
		elapsed_ms = beat_ms
		beat_started.emit(skill_id, index, beat.duplicate(true))
		while next_event_index < scheduled_events.size() and int(scheduled_events[next_event_index].cue_ms) <= beat_ms:
			var scheduled: Dictionary = scheduled_events[next_event_index]
			event_cued.emit(skill_id, str(scheduled.beat_name), scheduled.event.duplicate(true))
			next_event_index += 1
	if serial != playback_serial:
		return
	var tail_ms := maxi(0, duration_ms - elapsed_ms)
	if tail_ms > 0 and playback_time_scale > 0.0:
		await get_tree().create_timer(float(tail_ms) / 1000.0 * playback_time_scale).timeout
	if serial == playback_serial:
		while next_event_index < scheduled_events.size():
			var scheduled: Dictionary = scheduled_events[next_event_index]
			event_cued.emit(skill_id, str(scheduled.beat_name), scheduled.event.duplicate(true))
			next_event_index += 1
		playing = false
		action_finished.emit(skill_id)

func schedule_events(animation: Dictionary, action_events: Array[Dictionary]) -> Array[Dictionary]:
	var beat_times: Dictionary = {}
	for beat in animation.get("beats", []):
		beat_times[str(beat.get("name", ""))] = int(beat.get("atMs", 0))
	var bindings: Dictionary = animation.get("eventBindings", {})
	var duration_ms := int(animation.get("durationMs", 0))
	var previous_cue_ms := 0
	var scheduled: Array[Dictionary] = []
	for event in action_events:
		var event_kind := str(event.get("kind", ""))
		var beat_name := str(bindings.get(event_kind, "action_end"))
		var requested_ms := int(beat_times.get(beat_name, duration_ms))
		var cue_ms := maxi(previous_cue_ms, requested_ms)
		scheduled.append({"cue_ms": cue_ms, "beat_name": beat_name, "event": event.duplicate(true)})
		previous_cue_ms = cue_ms
	return scheduled

func cancel() -> void:
	playback_serial += 1
	playing = false
