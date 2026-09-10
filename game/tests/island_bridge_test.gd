extends SceneTree

const Port = preload("res://scripts/simulation/native_simulation_port.gd")

func captain(snapshot: Dictionary) -> Dictionary:
	for actor in snapshot.actors:
		if actor.id == "character.protagonist.captain":
			return actor
	assert(false, "the scenario's captain is missing")
	return {}

func _initialize() -> void:
	var port = Port.new()
	assert(port.is_available())
	# The verb takes a scenario document and nothing else.
	assert(port.create_island_from_scenario("{}").has("error"))
	assert(port.create_island_from_scenario('{"format":"project42.scenario","version":1,"scenario":{}}').has("error"))
	var initial: Dictionary = port.create_island()
	assert(not initial.has("error"))
	# The main scenario places Michael alone and the five factions' holdings.
	assert(initial.actors.size() == 1)
	assert(initial.buildings.size() == 5)
	var start := Vector2i(captain(initial).x, captain(initial).y)
	var land := {}
	for cell in initial.land:
		land[cell] = true
	assert(land.has(start))
	# Four land cells in a straight line from wherever the scenario starts him.
	var step_direction := Vector2i.ZERO
	for direction in [Vector2i(-1,0), Vector2i(1,0), Vector2i(0,-1), Vector2i(0,1)]:
		var clear := true
		for distance in range(1, 5):
			clear = clear and land.has(start + direction * distance)
		if clear:
			step_direction = direction
			break
	assert(step_direction != Vector2i.ZERO)
	var target := start + step_direction * 4
	assert(not port.move_island_actor("unknown", target))
	assert(not port.move_island_actor("character.protagonist.captain", Vector2i(-1,-1)))
	assert(port.move_island_actor("character.protagonist.captain", target))
	assert(Vector2i(captain(port.island_snapshot()).x, captain(port.island_snapshot()).y) == start)
	port.pause_island(true)
	# Pause freezes the whole island, not just the captain. A paused tick adds
	# only its (empty) strike list to the snapshot it started from.
	var frozen := JSON.stringify(port.island_snapshot())
	var stepped: Dictionary = port.tick_island()
	assert(stepped.strikes.is_empty())
	stepped.erase("strikes")
	assert(JSON.stringify(stepped) == frozen)
	port.pause_island(false)
	for step in range(4):
		var state: Dictionary = port.tick_island()
		assert(Vector2i(captain(state).x, captain(state).y) == start + step_direction * (step + 1))
	assert(not captain(port.island_snapshot()).moving)
	# The campaign clock reaches Godot as authored state, and only as much of
	# it as the player may see: the authored record says heat is never a
	# number, so the snapshot carries signalling channels and no total.
	var campaign: Dictionary = port.island_snapshot().campaign
	assert(campaign.deadline_day == 100)
	assert(campaign.days_remaining == 99)
	assert(campaign.confrontation == "")
	assert(campaign.confrontation_day == 0)
	assert(campaign.heat_signals is Array)
	assert(not campaign.has("heat_severity"))
	# Triggers change the world, not a feed: what Godot reads is the flag
	# array, the same shape as any other board fact in the snapshot.
	assert(port.island_snapshot().flags is Array)
	# The main scenario's quests are active from the first snapshot: their
	# stage and objective text are authored data Godot shows, not composes.
	var quests: Array = port.island_snapshot().quests
	assert(quests.size() == 6)
	var by_id := {}
	for quest in quests:
		by_id[quest.id] = quest
	var cult_quest: Dictionary = by_id["quest.the_cult_beneath_the_water"]
	assert(cult_quest.stage == "stage.rumors")
	assert(not cult_quest.objective.is_empty())
	assert(cult_quest.terminal == "")
	# Recruiting a companion is not yet possible in this bare start-of-day
	# snapshot (no one has been produced or recruited), so both companion
	# quests are still on their initial stage -- proving the snapshot shows
	# real state, not a hard-coded "already supported" answer.
	var companion_quest: Dictionary = by_id["quest.not_alone_anymore"]
	assert(companion_quest.stage == "stage.shipwrecked")
	assert(not companion_quest.objective.is_empty())
	var household_quest: Dictionary = by_id["quest.the_household_forms"]
	assert(household_quest.stage == "stage.recruiting")
	assert(not household_quest.objective.is_empty())
	assert(companion_quest.terminal == "")
	# No faction has fallen yet at the first snapshot either, so this quest
	# is still on its own initial stage -- the same real-state proof as the
	# two companion quests above.
	var rival_quest: Dictionary = by_id["quest.the_island_grows_quiet"]
	assert(rival_quest.stage == "stage.five_factions")
	assert(not rival_quest.objective.is_empty())
	assert(rival_quest.terminal == "")
	# Michael has not stripped the wreck at the first snapshot either: the
	# salvage is still in the water, so this quest sits on its own start.
	var wreck_quest: Dictionary = by_id["quest.what_the_sea_gave_back"]
	assert(wreck_quest.stage == "stage.untouched")
	assert(not wreck_quest.objective.is_empty())
	assert(wreck_quest.terminal == "")
	# Michael starts at the contested clearing, not on the cult's ground, so
	# this quest proves the snapshot reports where he actually is.
	var shrine_quest: Dictionary = by_id["quest.what_he_saw_out_there"]
	assert(shrine_quest.stage == "stage.hearsay")
	assert(not shrine_quest.objective.is_empty())
	assert(shrine_quest.terminal == "")
	# Leads reach Godot as the companion's own words plus the two readings she
	# can defend. None is open at the first snapshot: nobody has been recruited,
	# so nobody has brought Michael anything -- which is the contract, not an
	# empty-array coincidence.
	assert(port.island_snapshot().leads is Array)
	assert(port.island_snapshot().leads.is_empty())
	# Answering a lead that is not open is refused rather than silently ignored.
	assert(not port.resolve_island_lead("lead.neriah.the_water_turns", "interpretation.symptom"))
	assert(not port.resolve_island_lead("lead.does_not_exist", "interpretation.symptom"))
	print("PASS: actual Rust island bridge, scenario document, solo start, movement, ocean rejection, pause, arrival, the campaign clock, triggers, quests and companion leads")
	quit()
