extends RefCounted

## The one campaign both of this lane's proofs run: the suite that asserts the
## board and the capture script that photographs it. One owner, so a picture and
## an assertion can never be of two different islands.
##
## It plays the authored opening exactly as the prototype suite does -- salvage
## the wreck, climb to the estate, drop to the river, take the safe road -- then
## hands two cells to the factions the fiction already gives them.
##
## ## The one stand-in here, and why
##
## S7's forces are real state and `expedition_state_dictionary` projects them,
## but **no bridge verb raises or marches a force yet**: `raise_force`,
## `dispatch_force` and `strategic_tick` exist in Rust and none of them is a
## `#[func]`. So through the live bridge the island's `forces` array is always
## empty, and the board's route distance would have nothing to draw.
##
## Rather than fake a snapshot, this puts a real marching force into the real
## campaign the only honest way available: it takes the bridge's own
## `save_json`, adds a force to it, and hands it back through `load_json` --
## the same parse, the same `validate` and the same save-version gate every
## slot meets. The force that comes back is `ExpeditionState::forces` as Rust
## built it, not a dictionary this file invented.
##
## **Delete this method the day a bridge verb raises a force**, and let the
## proofs drive that verb instead.

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

## The force this file adds through the save. A pirate column marching the safe
## road it now holds one end of, most of the way there.
const MARCHING_FORCE_ID := "force.pirates.landing_column"
const MARCHING_ROUTE := "world.portal.river_landing_to_reception_terrace_safe_road"
const MARCHING_PROGRESS_MINUTES := 27


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


## Puts one marching force into the live campaign through a save round trip, as
## the class comment explains. Returns the snapshot the reloaded campaign gives,
## or an empty dictionary when the round trip was refused.
static func add_marching_force(session: Node) -> Dictionary:
	var document: String = session.expedition.save_json()
	if document.is_empty():
		return {}
	var parsed: Variant = JSON.parse_string(document)
	if not parsed is Dictionary:
		return {}
	var save: Dictionary = parsed
	var forces: Dictionary = save.get("forces", {})
	forces[MARCHING_FORCE_ID] = {
		"id": MARCHING_FORCE_ID,
		"faction_id": "faction.pirates",
		"origin_cell_id": "world.cell.river_landing",
		"position_cell_id": "world.cell.river_landing",
		"route": [MARCHING_ROUTE],
		"destination_cell_id": "world.cell.reception_terrace",
		"progress_minutes": MARCHING_PROGRESS_MINUTES,
		"readiness": 100,
		"supply": 24,
		"strength": 6,
		"composition": {"enemy.pirate.deckhand": 6}
	}
	save["forces"] = forces
	# Godot writes every integer with a decimal point; serde refuses the whole
	# document rather than truncating. This is the same coercion the port makes
	# on the way in, reused rather than written a second time.
	return session.expedition.load_json(JSON.stringify(NativeExpeditionPort.as_rust_integers(save)))
