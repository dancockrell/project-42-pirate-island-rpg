extends SceneTree

func _initialize() -> void:
	var director := SkillAnimationDirector.new()
	director.playback_time_scale = 0.0
	root.add_child(director)
	var cue_kinds: Array[String] = []
	var beat_names: Array[String] = []
	var presentation_beats: Array[String] = []
	director.event_cued.connect(func(_skill_id: String, beat_name: String, event: Dictionary) -> void:
		cue_kinds.append(str(event.kind))
		beat_names.append(beat_name)
	)
	director.presentation_cue_started.connect(func(_skill_id: String, cue: Dictionary) -> void:
		presentation_beats.append(str(cue.beat))
	)
	var record := {
		"id": "skill.test.guarded_strike",
		"animation": {
			"durationMs": 300,
			"beats": [
				{"atMs": 0, "name": "select"},
				{"atMs": 150, "name": "impact"},
				{"atMs": 300, "name": "recover"}
			],
			"eventBindings": {
				"actor_focused": "select",
				"damage_applied": "impact",
				"turn_ended": "recover"
			},
			"presentationCues": [
				{"beat": "select"},
				{"beat": "impact"},
				{"beat": "recover"}
			]
		}
	}
	var events: Array[Dictionary] = [
		{"kind": "actor_focused", "subjects": ["character.heroine.betty"], "payload": {}},
		{"kind": "damage_applied", "subjects": ["character.heroine.betty", "enemy.test"], "payload": {"amount": 12}},
		{"kind": "turn_ended", "subjects": ["character.heroine.betty"], "payload": {}},
		{"kind": "unbound_fixture_event", "subjects": [], "payload": {}}
	]
	await director.play(record, events)
	assert(cue_kinds == ["actor_focused", "damage_applied", "turn_ended", "unbound_fixture_event"])
	assert(beat_names == ["select", "impact", "recover", "action_end"])
	assert(presentation_beats == ["select", "impact", "recover"])
	assert(not director.playing)

	var reversed := director.schedule_events(record.animation, [
		{"kind": "damage_applied"},
		{"kind": "actor_focused"}
	])
	assert(reversed[0].cue_ms == 150)
	assert(reversed[1].cue_ms == 150)
	assert(reversed[0].event.kind == "damage_applied")
	assert(reversed[1].event.kind == "actor_focused")

	print("SkillAnimationDirector tests passed.")
	quit(0)
