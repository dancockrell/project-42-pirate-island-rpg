extends RefCounted

## The one campaign both of this lane's proofs run: the suite that asserts the
## board and the capture script that photographs it. One owner, so a picture and
## an assertion can never be of two different islands.
##
## It plays the authored opening exactly as the prototype suite does -- salvage
## the wreck, climb to the estate, drop to the river, take the safe road -- then
## hands two cells to the factions the fiction already gives them.
##
## ## The force on the road, and where it comes from
##
## It comes from the bridge's own verbs. S18 gave `raise_force` and
## `dispatch_force` a `#[func]` each and the port a line each, so the column
## below is raised and sent through the same calls the player uses to direct
## Captain Michael's faction (brief section 5.9) -- the save round trip this
## file used to do instead is gone, as its own note said it must be the day a
## verb landed.
##
## The column therefore stands at the head of the road it was sent down rather
## than part of the way along it: a dispatch plans the whole route and marches
## none of it, and the hours that would march it belong to the strategic clock,
## which no verb here turns. `progress` is 0.0 and `next_cell_id` is the cell
## the first hop reaches, which is what the board draws.

## ## P13: and what is standing on it
##
## The same rule, for the same reason: the suite that asserts a building on the
## board and the capture that photographs one must be of one island. The three
## instances below are that island's development, and `develop_island` is the one
## way either proof reaches it.
##
## **It goes through a save round trip, and P13's suite says at length why.** In
## short: S5 scores a goal from stockpiles and relationships, a campaign begun
## through the bridge has neither, so no faction ever chooses `Goal::Develop`;
## and `advance_construction` has no caller on a clock, so a building S17 did
## place could never finish and `produce_machine` refuses a building that is not
## `Operational`. No number of hours fixes either. The day a verb seeds a
## stockpile, or spends a construction hour, this file's splice is replaced by it
## exactly as the force above replaced its own round trip when S18 landed.

## Where the bridge's save spells what is standing, and what this file puts
## there. Written as JSON text rather than as a Dictionary because Godot's
## `JSON.stringify` writes every integer with a decimal point and the bridge's
## save parser refuses `"tier": 2.0` -- the port's integer coercion is on the way
## in to `configure`, and a save document is not configuration.
const EMPTY_DEVELOPMENT := '"buildings":{},"machines":{}'
const DEVELOPMENT_ENDS_BEFORE := ',"bond_ranks"'

## Captain Michael's yard on the terrace, standing and working; a pirate watch
## post on the landing they hold, with three hours of work still on it; and the
## dog the yard turned out, whole, standing at the yard's own cell. Real records
## out of `content/buildings/` and `content/machines/`, real cells off the
## authored island, and concept keys only.
const YARD_ID := "building_instance.michael_yard"
const POST_ID := "building_instance.pirate_post"
const DOG_ID := "machine_instance.yard_dog"
const YARD_CELL := "world.cell.reception_terrace"
const POST_CELL := "world.cell.river_landing"
const YARD_JSON := '"building_instance.michael_yard":{"id":"building_instance.michael_yard","def_id":"building.machine_shop","cell_id":"world.cell.reception_terrace","faction_id":"faction.michael","tier":2,"hp":200,"state":"operational","construction_hours_remaining":0}'
const POST_JSON := '"building_instance.pirate_post":{"id":"building_instance.pirate_post","def_id":"building.coast_watch_post","cell_id":"world.cell.river_landing","faction_id":"faction.pirates","tier":1,"hp":60,"state":"under_construction","construction_hours_remaining":3}'
const DOG_JSON := '"machine_instance.yard_dog":{"id":"machine_instance.yard_dog","def_id":"machine.mechanical_dog","faction_id":"faction.michael","cell_id":"world.cell.reception_terrace","built_by_building_instance_id":"building_instance.michael_yard","fuel_remaining":3,"water_remaining":0,"damage":0}'
const DEVELOPMENT := '"buildings":{' + YARD_JSON + ',' + POST_JSON + '},"machines":{' + DOG_JSON + '}'
## The same island with the post pulled down and the dog gone: what a board must
## show when a snapshot stops carrying an instance.
const HALF_CLEARED := '"buildings":{' + YARD_JSON + '},"machines":{}'

const OPENING_SALVAGE := "anchor.black_beach.salvage_point"
const OPENING_TRAVEL := [
	"world.portal.black_beach_to_damaged_estate",
	"world.portal.damaged_estate_to_river_landing",
	"world.portal.river_landing_to_reception_terrace_safe_road"
]
## The elves hold their own tomb (the four tomb cells' authored
## `dungeonContext.ownerConceptKey`) and the pirates hold the river landing,
## which is what makes the safe road out of it contested. Concept keys only.
const HELD_CELLS := [
	["world.cell.river_landing", "faction.pirates"],
	["world.cell.tomb_reception", "faction.elves"]
]

## The force this file raises. A pirate column on the safe road it now holds one
## end of. Concept keys and content's own actor ID; nothing invented here.
const MARCHING_FORCE_ID := "force.pirates.landing_column"
const MARCHING_FROM := "world.cell.river_landing"
const MARCHING_TO := "world.cell.reception_terrace"
const MARCHING_COMPOSITION := {"enemy.pirate.deckhand": 6}
const MARCHING_ASSIGNMENT := "assignment.board_verification.march"


## Plays the opening, or the first `steps` legs of it. Stopping one leg short is
## how the captures are taken with live roads on the board: arriving at the
## terrace arms its authored encounter, and a pending encounter makes every road
## illegal, so a board photographed there would truthfully show no route at all.
static func play_opening(session: Node, steps: int = OPENING_TRAVEL.size()) -> Dictionary:
	var snapshot: Dictionary = session.use_anchor(OPENING_SALVAGE)
	for index in mini(steps, OPENING_TRAVEL.size()):
		snapshot = session.travel(str(OPENING_TRAVEL[index]))
	return snapshot


## Hands the authored holders their cells. Returns the last snapshot.
static func take_control(session: Node) -> Dictionary:
	var snapshot: Dictionary = session.latest_snapshot
	for pair in HELD_CELLS:
		snapshot = session.set_control(str(pair[0]), str(pair[1]))
	return snapshot


## Raises one pirate column and sends it down the safe road, through the bridge
## verbs and nothing else. Returns the snapshot the dispatch gives, or an empty
## dictionary when either order was refused.
static func add_marching_force(session: Node) -> Dictionary:
	var raised: Dictionary = session.raise_force(
		MARCHING_FORCE_ID, "faction.pirates", MARCHING_FROM, MARCHING_COMPOSITION, MARCHING_ASSIGNMENT
	)
	if not bool(raised.get("configured", false)):
		return {}
	var dispatched: Dictionary = session.dispatch_force(MARCHING_FORCE_ID, MARCHING_TO)
	if not bool(dispatched.get("configured", false)):
		return {}
	return dispatched


## Develops the island: the campaign's own save is taken from the bridge, the
## whole of its `buildings` and `machines` maps is replaced by `development`, and
## it is loaded back **through the bridge**, which parses it, validates it and
## projects it. Returns the snapshot the bridge gives, or an empty dictionary if
## it refused.
static func develop_island(session: Node, development: String = DEVELOPMENT) -> Dictionary:
	var raw: String = session.expedition.save_json()
	var replaced := ""
	if raw.contains(EMPTY_DEVELOPMENT):
		replaced = raw.replace(EMPTY_DEVELOPMENT, development)
	else:
		var start := raw.find('"buildings":{')
		var ends := raw.find(DEVELOPMENT_ENDS_BEFORE)
		if start < 0 or ends <= start:
			push_error("The bridge's save no longer spells its buildings and machines where this file can develop them.")
			return {}
		replaced = raw.substr(0, start) + development + raw.substr(ends)
	var loaded: Dictionary = session.expedition.load_json(replaced)
	if not bool(loaded.get("configured", false)):
		return {}
	session.latest_snapshot = loaded
	return loaded
