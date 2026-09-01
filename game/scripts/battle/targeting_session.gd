class_name TargetingSession
extends RefCounted

## Presentation-side target collector. It determines which IDs the player may
## point at from authored target rules, but the simulation remains the final
## authority and may reject any submitted command.

var skill_id := ""
var target_rule := ""
var selected_ids: Array[String] = []
var actors_by_id: Dictionary = {}
var active := false

func begin(skill_record: Dictionary, actors: Array) -> Dictionary:
	cancel()
	skill_id = str(skill_record.get("id", ""))
	target_rule = str(skill_record.get("targetRule", ""))
	for actor in actors:
		if actor is Dictionary:
			actors_by_id[str(actor.get("id", ""))] = actor.duplicate(true)
	if target_rule in ["all_living_party_members"]:
		return {"status": "ready", "target_ids": []}
	if target_rule == "automatic_reaction_to_other_party_member_lethal_hit":
		return {"status": "automatic", "target_ids": []}
	active = true
	return {"status": "selecting", "prompt": prompt()}

func select(actor_id: String) -> Dictionary:
	if not active:
		return {"status": "inactive", "reason": "no_targeting_session"}
	if not can_select(actor_id):
		return {"status": "illegal", "reason": illegal_reason(actor_id), "prompt": prompt()}
	selected_ids.append(actor_id)
	if selected_ids.size() >= required_count():
		active = false
		return {"status": "ready", "target_ids": selected_ids.duplicate()}
	return {"status": "selecting", "prompt": prompt()}

func cancel() -> void:
	skill_id = ""
	target_rule = ""
	selected_ids.clear()
	actors_by_id.clear()
	active = false

func required_count() -> int:
	return 2 if target_rule == "ordered_pair_threatened_ally_then_hostile" else 1

func can_select(actor_id: String) -> bool:
	if not actors_by_id.has(actor_id) or selected_ids.has(actor_id):
		return false
	var actor: Dictionary = actors_by_id[actor_id]
	var faction := str(actor.get("faction", ""))
	var living := int(actor.get("vitality", 0)) > 0
	match target_rule:
		"one_hostile", "one_living_hostile": return faction == "hostile" and living
		"one_living_party_member": return faction == "party" and living
		"one_defeated_party_member_other_than_betty": return faction == "party" and not living and actor_id != "character.heroine.betty"
		"ordered_pair_threatened_ally_then_hostile":
			if selected_ids.is_empty():
				return faction == "party" and living and actor_id != "character.heroine.betty"
			return faction == "hostile" and living
	return false

func prompt() -> String:
	match target_rule:
		"one_hostile", "one_living_hostile": return "SELECT ONE LIVING HOSTILE"
		"one_living_party_member": return "SELECT ONE LIVING PARTY MEMBER"
		"one_defeated_party_member_other_than_betty": return "SELECT ONE DEFEATED HEROINE"
		"ordered_pair_threatened_ally_then_hostile":
			return "SELECT THE THREATENED HEROINE" if selected_ids.is_empty() else "SELECT THE HOSTILE THREATENING HER"
	return "TARGET RULE NOT CONNECTED: %s" % target_rule

func illegal_reason(actor_id: String) -> String:
	if not actors_by_id.has(actor_id):
		return "unknown_actor"
	if selected_ids.has(actor_id):
		return "duplicate_target"
	return "actor_does_not_satisfy_%s" % target_rule
