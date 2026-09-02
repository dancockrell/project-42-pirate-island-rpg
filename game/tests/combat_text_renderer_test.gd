extends SceneTree

var failures := 0
var renderer := CombatTextRenderer.new()

func _init() -> void:
	renderer.configure({
		"character.heroine.betty": "Betty",
		"character.heroine.isabella": "Isabella",
		"enemy.raptor.razorbeak.prototype": "Razorbeak"
	})
	check_text("interception_set", ["character.heroine.betty", "character.heroine.isabella"], {}, ["Betty", "Isabella"], ["Vix"])
	check_text("interception_triggered", ["character.heroine.betty", "character.heroine.isabella", "enemy.raptor.razorbeak.prototype"], {}, ["Betty", "Isabella"], ["Vix"])
	check_text("actor_revived", ["character.heroine.isabella"], {"vitality": 44}, ["Isabella", "44"], ["Ayla"])
	check_text("bonus_turn_granted", ["character.heroine.isabella"], {}, ["Isabella"], ["Ayla"])
	check_text("actor_moved", ["character.heroine.isabella"], {"from_band": 2, "to_band": 0}, ["Isabella", "band 2", "band 0"], ["Betty"])
	check_text("status_removed", ["character.heroine.isabella"], {"status_id": "status.isabella.poisoned.prototype", "status_kind": "poisoned"}, ["Isabella", "poisoned"], ["prototype"])
	var intent := renderer.render(event("enemy_intent_declared", ["enemy.raptor.razorbeak.prototype", "character.heroine.isabella"], {
		"skill_id": "skill.enemy.razorbeak.rushing_bite", "raw_damage": 16,
		"guard_absorbed": 4, "vitality_damage": 12, "lethal": true,
		"interception_protector_id": "character.heroine.betty", "fatal_intercept_available": false
	}))
	check(intent.intent == "INTENT: RUSHING BITE → ISABELLA • LETHAL", "intent header must use structured skill, target and lethality")
	check("Isabella" in intent.text and "Betty" in intent.text and "4 will strike Guard" in intent.text, "intent prose must describe actual subjects and projection facts")
	if failures > 0:
		quit(1)
		return
	print("CombatTextRenderer tests passed.")
	quit(0)

func check_text(kind: String, subjects: Array, payload: Dictionary, required: Array[String], forbidden: Array[String]) -> void:
	var text: String = renderer.render(event(kind, subjects, payload)).text
	for fragment in required:
		check(fragment in text, "%s text must include %s" % [kind, fragment])
	for fragment in forbidden:
		check(fragment not in text, "%s text must not hard-code %s" % [kind, fragment])

func event(kind: String, subjects: Array, payload: Dictionary) -> Dictionary:
	return {"kind": kind, "subjects": subjects, "payload": payload}

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
