class_name AtmosphereReviewSnapshots
extends RefCounted

## Four authored snapshots, in exactly the shape the expedition bridge projects,
## for the two things that must be able to name a sky: `Atmosphere`'s suite and
## the four committed captures.
##
## They are fixtures, not content: no simulation reads them, they place nothing
## on the board, and `content/atmosphere/` still owns every number the sky is
## made of. What they own is the *situation* -- a clear dawn, a corrupted
## overcast dusk, a held midnight, a storm over a consumed cell -- so the suite
## and the capture argue about the same four islands.
##
## Every key here exists on the real snapshot. `weather`, `corruption`,
## `heat_band`, `hour_of_day`, `active_region_id` and `is_night` are the six
## `godot_bridge::expedition_state_dictionary` grew for this card; the rest were
## already there.

const CELL := "world.cell.reception_terrace"
const REGION := "world.region.black_beach"

## The four names, in the order a day runs them, so a caller that wants all of
## them does not retype the list.
const NAMES := ["dawn", "dusk", "night", "storm"]


## One authored snapshot by name, or an empty dictionary for a name that is not
## one of [constant NAMES].
static func of(name: String) -> Dictionary:
	match name:
		"dawn":
			# First light, clean ground, nothing stirring. The reference sky:
			# everything else is read against this one.
			return _snapshot("dawn", 1, "clear", "", "dormant", false, [])
		"dusk":
			# The light going, the sky gone flat, and the terrace itself
			# beginning to turn -- one touched cell, and pressure that has
			# started to lean.
			return _snapshot("dusk", 12, "overcast", "touched", "stirring", true, [])
		"night":
			# Held ground after dark: the moon is nearly out and the warmth is
			# entirely the lamps at the two buildings the island is holding.
			return _snapshot(
				"midnight",
				18,
				"clear",
				"",
				"rising",
				true,
				["building.coast_watch_post", "building.machine_shop"]
			)
		"storm":
			# Daylight that is no help. Full storm over a cell corruption has
			# finished with, and pressure one band from the confrontation.
			return _snapshot("day", 7, "storm", "consumed", "imminent", false, [])
	return {}


static func _snapshot(
	segment: String,
	hour: int,
	weather: String,
	corruption_band: String,
	heat_band: String,
	night: bool,
	buildings: Array
) -> Dictionary:
	var corruption: Dictionary = {}
	if not corruption_band.is_empty():
		corruption[CELL] = corruption_band
	return {
		"configured": true,
		"campaign_day": 12,
		"time_segment": segment,
		"hour_of_day": hour,
		"active_location_id": CELL,
		"active_region_id": REGION,
		"weather": {REGION: weather},
		"corruption": corruption,
		"heat_band": heat_band,
		"is_night": night,
		"buildings": buildings,
		"metadata": {"source": "atmosphere_review_snapshots", "authoritative": false}
	}
