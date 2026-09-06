class_name BoardPalette
extends RefCounted

## Every colour the isometric board draws with, in one place.
##
## The five constants at the top are the ones the two prototypes already carry
## (`expedition_prototype.gd`, `expedition_route_board.gd`, `battle_prototype.gd`)
## and they are repeated here rather than reinvented: P5's `bronze_vellum.tres`
## becomes the one owner of every value in this file, and when it lands this
## class reads the Theme instead of declaring hex.
##
## The faction tints below are new. They exist because the world distance has to
## say who holds a cell and there was no colour for that anywhere in the tree.
## They are keyed by the brief's six concept keys and by nothing else -- no
## faction proper name appears in this file or may appear in it -- and they are
## deliberately low-chroma washes over the blockout clay rather than team
## colours: a held cell should read as *tinted stone*, not as a painted counter.

const DEEP := Color("0b1514")
const TEAL := Color("55c9ac")
const CREAM := Color("eadfca")
const BRONZE := Color("b78a4b")
const DANGER := Color("c24e45")
const MUTED := Color("52625b")
const PANEL := Color("132321")

## The unheld blockout clay: what a cell nobody holds is made of.
const CLAY := Color("6d6a5f")
## The sea the island stands in, and the sky it stands under.
const SEA := Color("0d2a30")

## The hue a road of no danger and a road of full danger are drawn in, and the
## risk at which the scale tops out. **needs decision** on the last one: S2's
## `CONTESTED_RISK_MODIFIER` is 2 and the authored slice runs 0 through 5, so
## five is where the scale ends today; the eventual risk ceiling is a design
## decision nobody has taken.
const SAFE_HUE := 0.44
const DANGER_HUE := 0.02
const RISK_AT_FULL_DANGER := 5

## Concept key -> the wash a cell held by that faction takes. Concept keys only.
const FACTION_TINT := {
	"michael": Color("b78a4b"),
	"pirates": Color("8e5a4b"),
	"colonial_powers": Color("6f7f9c"),
	"fox_people": Color("b08749"),
	"elves": Color("55c9ac"),
	"cthulhu": Color("6a5b7d")
}

## **needs decision.** How tall each role's socket post stands, in metres. The
## four roles are told apart by height as well as by colour, so the room reads
## without relying on colour alone. A framing choice; P5 owns the final scale.
const ROLE_POST_HEIGHT := {
	"player": 3.4,
	"occupant": 2.2,
	"hostile": 1.2,
	"item": 0.0
}

## The four spawn-socket roles B11 declares, each with the colour the room
## distance marks it in.
const ROLE_TINT = {
	"player": Color("55c9ac"),
	"occupant": Color("eadfca"),
	"hostile": Color("c24e45"),
	"item": Color("b78a4b")
}


## The colour a faction's own pieces are drawn in -- a miniature, not a wash, so
## it is the tint at full strength and legible against the clay it stands on.
static func faction_colour(faction_id: String) -> Color:
	var concept_key := faction_id.trim_prefix("faction.")
	if faction_id.is_empty() or not FACTION_TINT.has(concept_key):
		return MUTED
	return FACTION_TINT[concept_key].lightened(0.18)


## The wash for a cell held by `faction_id` (`faction.<concept_key>`), or the
## unheld clay for "". An unknown faction ID is unheld rather than a guess: a
## proper name must never be able to enter through a colour lookup.
static func controller_tint(faction_id: String) -> Color:
	var concept_key := faction_id.trim_prefix("faction.")
	if faction_id.is_empty() or not FACTION_TINT.has(concept_key):
		return CLAY
	return CLAY.lerp(FACTION_TINT[concept_key], 0.52)


## A road's colour by the live risk the snapshot gives it. Risk is the only
## input: nothing here re-derives danger, and a contested road is already a
## higher number by the time it reaches this function.
static func risk_tint(risk_level: int) -> Color:
	# Around the hue wheel, not straight through it. A straight RGB blend from
	# the safe teal to the danger red passes through mud at exactly the risks the
	# authored slice uses most, which made a contested road look like a quiet
	# one; running the hue from teal down through green, yellow and orange to red
	# keeps every step of danger a colour the player can name.
	var danger := pow(clampf(float(risk_level) / float(RISK_AT_FULL_DANGER), 0.0, 1.0), 0.62)
	return Color.from_hsv(lerpf(SAFE_HUE, DANGER_HUE, danger), 0.58, 0.80)
