extends SceneTree

func _initialize() -> void:
	var actors := [
		{"id": "character.heroine.betty", "faction": "party", "vitality": 100},
		{"id": "character.heroine.ayla", "faction": "party", "vitality": 0},
		{"id": "character.heroine.vix", "faction": "party", "vitality": 70},
		{"id": "enemy.raptor.razorbeak", "faction": "hostile", "vitality": 70}
	]
	var session := TargetingSession.new()
	assert(session.has_legal_targets(skill("one_living_hostile"), actors))
	assert(session.has_legal_targets(skill("ordered_pair_threatened_ally_then_hostile"), actors))
	assert(session.has_legal_targets(skill("one_defeated_party_member_other_than_betty"), actors))
	assert(not session.has_legal_targets(skill("automatic_reaction_to_other_party_member_lethal_hit"), actors))
	# A7: Ayla's Reach Counter, the second automatic reaction, on its own rule.
	assert(not session.has_legal_targets(skill("automatic_reaction_to_hostile_attack_in_shared_band"), actors))
	var two_actor_encounter := [actors[0], actors[3]]
	assert(not session.has_legal_targets(skill("ordered_pair_threatened_ally_then_hostile"), two_actor_encounter))
	assert(not session.has_legal_targets(skill("one_defeated_party_member_other_than_betty"), two_actor_encounter))

	var result := session.begin(skill("one_living_hostile"), actors)
	assert(result.status == "selecting")
	assert(session.select("character.heroine.vix").status == "illegal")
	result = session.select("enemy.raptor.razorbeak")
	assert(result.status == "ready")
	assert(result.target_ids == ["enemy.raptor.razorbeak"])

	result = session.begin(skill("ordered_pair_threatened_ally_then_hostile"), actors)
	assert(session.select("character.heroine.betty").status == "illegal")
	assert(session.select("character.heroine.vix").status == "selecting")
	assert(session.select("character.heroine.vix").reason == "duplicate_target")
	result = session.select("enemy.raptor.razorbeak")
	assert(result.target_ids == ["character.heroine.vix", "enemy.raptor.razorbeak"])

	result = session.begin(skill("one_defeated_party_member_other_than_betty"), actors)
	assert(session.select("character.heroine.betty").status == "illegal")
	result = session.select("character.heroine.ayla")
	assert(result.status == "ready")

	result = session.begin(skill("all_living_party_members"), actors)
	assert(result.status == "ready")
	assert(result.target_ids.is_empty())

	result = session.begin(skill("automatic_reaction_to_other_party_member_lethal_hit"), actors)
	assert(result.status == "automatic")
	assert(not session.active)

	result = session.begin(skill("automatic_reaction_to_hostile_attack_in_shared_band"), actors)
	assert(result.status == "automatic")
	assert(not session.active)

	print("TargetingSession tests passed.")
	quit(0)

func skill(target_rule: String) -> Dictionary:
	return {"id": "skill.test", "targetRule": target_rule}
