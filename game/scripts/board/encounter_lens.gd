class_name EncounterLens
extends RefCounted

## What the board does when the party enters an encounter.
##
## **O1 is Open.** The brief has not decided what an encounter looks like when
## it opens off the board -- whether the board pushes in to the room, whether a
## separate battle view takes the screen, or something else -- and this lane
## does not decide it. So this class is a stand-in and says so: it loads the
## battle screen the project already has, `res://scenes/battle/battle_prototype.tscn`,
## and nothing more. It holds no camera move, no transition and no framing
## opinion, because those are exactly the choices O1 owns.
##
## What it does own is the *seam*: the board asks this class for the lens, and
## the day O1 closes, one class changes rather than the board.

## The battle screen the project already ships. When O1 closes this constant is
## replaced by whatever it names; the board does not learn a second path.
const BATTLE_SCENE_PATH := "res://scenes/battle/battle_prototype.tscn"

## The reason this class exists in the state it is in, in one readable string,
## so a suite and a reader are told the same thing.
const OPEN_DECISION := "O1: the encounter lens is Open; the board opens the existing battle screen unchanged"


## The scene an encounter opens. Null when the battle screen is missing, which
## is a project error rather than something to substitute for.
static func lens_scene() -> PackedScene:
	if not ResourceLoader.exists(BATTLE_SCENE_PATH):
		push_error("The encounter lens stand-in cannot find %s." % BATTLE_SCENE_PATH)
		return null
	return load(BATTLE_SCENE_PATH) as PackedScene
