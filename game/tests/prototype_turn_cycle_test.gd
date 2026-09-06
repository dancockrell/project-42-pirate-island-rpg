extends SceneTree

var failures := 0

func _init() -> void:
	call_deferred("run")

func run() -> void:
	var prototype := BattlePrototype.new()
	root.add_child(prototype)
	await process_frame
	check(prototype.active_actor_id == "character.heroine.betty", "Betty must own the opening player turn")
	check(prototype.drawn_card_ids().size() == 5, "every actor the simulation reports must be drawn: Betty, Ayla, Vix, the Razorbeak and Captain Michael")
	for actor in prototype.snapshot_actors:
		check(prototype.card_band_name(str(actor.id)) == str(actor.band_name), "%s's card must carry the band name the simulation sent, not one the screen derived" % actor.id)
	check(not prototype.is_command_enabled("skill.captain.reposition"), "Michael's command grid must be disabled on Betty's turn")
	check(prototype.card_band_name("character.protagonist.captain") == "party_rear", "Michael must stand in the party's rear band at the start of the encounter")
	await prototype.submit_skill("skill.betty.guarded_strike", ["enemy.raptor.razorbeak"])
	# B5: the Captain stands in the slice at initiative 10, between the Razorbeak
	# and the second round, so the automatic turns now stop at his. He is
	# commanded like Betty, not held automatically like Vix.
	check(prototype.active_actor_id == "character.protagonist.captain", "the automatic turns must stop at Captain Michael, who is commanded rather than held")
	check(prototype.is_command_enabled("skill.captain.reposition"), "Michael's command grid must be enabled on his own turn")
	check(prototype.is_command_enabled("skill.captain.weapon_attack"), "Michael's Weapon Attack must be offered while a living hostile stands on the board")
	await prototype.submit_skill("skill.captain.reposition", [])
	check(prototype.card_band_name("character.protagonist.captain") == "party_front", "a Reposition submitted through the port must move Michael's card to the other party band")
	check(prototype.active_actor_id == "character.heroine.betty", "automatic enemy and support turns must return control to Betty")
	check(prototype.is_command_enabled("skill.betty.guarded_strike"), "Betty's legal commands must re-enable after the full initiative cycle")
	check(not prototype.is_command_enabled("skill.captain.reposition"), "Michael's command grid must fold away again once the turn is Betty's")
	check("VIX" in prototype.intent_label.text and "LETHAL" in prototype.intent_label.text, "visible enemy intent must name the authoritative target and lethal projection")
	prototype.queue_free()
	if failures > 0:
		quit(1)
		return
	print("BattlePrototype turn-cycle test passed.")
	quit(0)

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
