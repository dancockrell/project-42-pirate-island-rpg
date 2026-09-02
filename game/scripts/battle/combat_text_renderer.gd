class_name CombatTextRenderer
extends RefCounted

## Pure presentation renderer. It converts structured simulation events into
## player-facing BBCode without changing simulation state or discovering rules.

var actor_names: Dictionary = {}

func configure(names: Dictionary) -> void:
	actor_names = names.duplicate(true)

func render(event: Dictionary) -> Dictionary:
	var kind := str(event.get("kind", "missing"))
	var subjects: Array = event.get("subjects", [])
	var payload: Dictionary = event.get("payload", {})
	match kind:
		"battle_started": return replace("[color=#b78a4b]BATTLE STARTED[/color] The party waits in the card rail until the simulation names an active actor.")
		"command_accepted": return replace("[color=#4fc7b4]ACCEPTED[/color] %s begins %s." % [name_at(subjects, 0), skill_name(payload.get("skill_id", "unknown_skill"))])
		"enemy_intent_declared": return enemy_intent(subjects, payload)
		"damage_applied": return append(" %s hits %s for %d damage; %d Vitality remains." % [name_at(subjects, 0), name_at(subjects, subjects.size() - 1), payload.get("amount", 0), payload.get("remaining_vitality", 0)])
		"guard_changed":
			var delta := int(payload.get("delta", 0))
			if delta > 0: return append(" %s gains %d Guard and now has %d." % [name_at(subjects, 0), delta, payload.get("total", 0)])
			if delta < 0: return append(" %s loses %d Guard and now has %d." % [name_at(subjects, 0), -delta, payload.get("total", 0)])
			return append(" %s's Guard remains %d." % [name_at(subjects, 0), payload.get("total", 0)])
		"vitality_changed":
			var delta := int(payload.get("delta", 0))
			if delta > 0: return append(" %s restores %d Vitality and now has %d." % [name_at(subjects, 0), delta, payload.get("total", 0)])
			if delta < 0: return append(" %s loses %d Vitality and now has %d." % [name_at(subjects, 0), -delta, payload.get("total", 0)])
			return append(" %s is already at maximum Vitality." % name_at(subjects, 0))
		"status_removed": return append(" %s is cleared of %s." % [name_at(subjects, 0), readable_id(payload.get("status_id", "condition"))])
		"actor_moved": return append(" %s crosses from band %d to band %d." % [name_at(subjects, 0), payload.get("from_band", 0), payload.get("to_band", 0)])
		"interception_set": return append(" %s takes position in front of %s and will intercept the next hostile attack aimed at her." % [name_at(subjects, 0), name_at(subjects, 1)])
		"interception_triggered": return append(" %s receives the attack meant for %s; the interception is spent." % [name_at(subjects, 0), name_at(subjects, 1)])
		"reaction_window_opened": return append(" [color=#c24e45]LETHAL HIT DETECTED[/color] %s is in immediate danger." % name_at(subjects, 0))
		"reaction_triggered": return append(" %s breaks out of the card rail and uses %s to protect %s." % [name_at(subjects, 0), skill_name(payload.get("skill_id", "reaction")), name_at(subjects, 1)])
		"defeat_prevented": return append(" %s's defeat is prevented by %s." % [name_at(subjects, 0), skill_name(payload.get("prevented_by_skill_id", "reaction"))])
		"battlefield_effect_created": return append(" %s deploys %s." % [name_at(subjects, 0), skill_name(payload.get("source_skill_id", "battlefield_effect"))])
		"battlefield_effect_pulse": return append(" [color=#4fc7b4]FIELD EFFECT[/color] %s's support reaches every eligible party member; %d pulse(s) remain." % [name_at(subjects, 0), payload.get("pulses_remaining_after", 0)])
		"battlefield_effect_removed": return append(" The battlefield effect ends: %s." % readable_id(payload.get("reason", "removed")))
		"actor_revived": return append(" [color=#4fc7b4]COMBAT REVIVAL[/color] %s returns with %d Vitality, zero Guard and no negative conditions." % [name_at(subjects, 0), payload.get("vitality", 0)])
		"bonus_turn_granted": return append(" %s receives an immediate bonus turn; normal initiative will resume afterward." % name_at(subjects, 0))
		"actor_defeated": return append(" %s is defeated." % name_at(subjects, 0))
		"round_started": return append(" [color=#b78a4b]ROUND %d[/color] begins." % payload.get("round", 0))
		"battle_ended": return replace("[color=#4fc7b4]VICTORY[/color] The hostile can no longer fight." if payload.get("victory", false) else "[color=#c24e45]DEFEAT[/color] The encounter ends; midnight return is pending.")
		"command_rejected": return replace("[color=#c24e45]REJECTED[/color] %s." % readable_id(payload.get("reason", "unknown_reason")))
		"turn_started", "turn_ended", "actor_focused": return none()
		_: return append(" [Unrendered event: %s]" % kind)

func enemy_intent(subjects: Array, facts: Dictionary) -> Dictionary:
	var target_name := name_at(subjects, 1)
	var skill := skill_name(facts.get("skill_id", "enemy_action"))
	var sentence := " [color=#c24e45]INTENT[/color] %s selects %s with %s. The attack carries %d raw damage" % [name_at(subjects, 0), target_name, skill, facts.get("raw_damage", 0)]
	if int(facts.get("guard_absorbed", 0)) > 0:
		sentence += "; %d will strike Guard and %d will reach Vitality" % [facts.get("guard_absorbed", 0), facts.get("vitality_damage", 0)]
	else:
		sentence += ", all of it against Vitality"
	if facts.get("lethal", false): sentence += ". The projected hit is lethal"
	var protector_id := str(facts.get("interception_protector_id", ""))
	if not protector_id.is_empty():
		sentence += ", though %s is already standing in the attack line" % name_for(protector_id)
	elif facts.get("fatal_intercept_available", false):
		sentence += ", and Betty is ready to break formation before it lands"
	return {"mode": "append", "text": sentence + ".", "intent": "INTENT: %s → %s%s" % [skill.to_upper(), target_name.to_upper(), " • LETHAL" if facts.get("lethal", false) else ""]}

func name_at(subjects: Array, index: int) -> String:
	if index < 0 or index >= subjects.size(): return "Unknown combatant"
	return name_for(str(subjects[index]))

func name_for(actor_id: String) -> String:
	return str(actor_names.get(actor_id, readable_id(actor_id)))

func skill_name(value: Variant) -> String:
	return readable_id(value).to_upper()

func readable_id(value: Variant) -> String:
	var parts := str(value).split(".")
	return str(parts[-1]).replace("_", " ") if not parts.is_empty() else "unknown"

func replace(text: String) -> Dictionary: return {"mode": "replace", "text": text}
func append(text: String) -> Dictionary: return {"mode": "append", "text": text}
func none() -> Dictionary: return {"mode": "none", "text": ""}
