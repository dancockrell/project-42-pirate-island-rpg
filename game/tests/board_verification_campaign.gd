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
