class_name BoardPalette
extends RefCounted

## Every colour the isometric board draws with, in one place -- and, since P10,
## not one hex among them.
##
## P5 made `res://themes/bronze_vellum.tres` the one owner of the palette and
## `ThemeTokens` the one door onto it. This class used to restate seven of those
## values as constants "until P5 lands"; P5 has landed, so it reads them. What
## remains here is the board's own vocabulary -- the clay a room is made of, the
## sea it stands in, the wash a held cell takes -- and each of those is
## *derived* from tokens the Theme already carries rather than added to the
## Theme (that is P5's file and P11's round) or kept as a second hex (that is
## the thing this card removes).
##
## Two derivations are worth naming, because they are the only places where a
## colour on the board is not simply a token:
##
## * **The faction washes.** Six concept keys need six colours the Theme has no
##   token for. Rather than invent six hexes, each key names a fixed turn around
##   the hue wheel from the Theme's own `teal`, and takes that token's
##   saturation and value. So the six are the Theme's chroma at six hues: they
##   move when the Theme moves, they cannot drift from it, and a faction
##   *proper name* still cannot enter -- the table is keyed by the brief's six
##   concept keys and by nothing else.
## * **The risk scale.** A road's colour runs from the Theme's `teal` hue down
##   to its `danger` hue, so "safe" and "dangerous" on the board are the two
##   hues the interface uses for safe and dangerous everywhere else.
##
## Nothing in this file reads the simulation, and nothing in it decides a
## result.

## The Theme these colours are read out of. Null means the resource as authored;
## a screen that has built a Theme for B9's accessibility settings hands it here
## through `use_theme`, so high contrast reaches the board's clay and its roads
## as surely as it reaches a button.
static var _theme: Theme = null

## **needs decision.** How far each derived surface sits between the two tokens
## it is mixed from. Framing choices, all of them: they say how the island reads,
## not what anything is worth. They live here as named fractions rather than as
## mixed hexes so that a change to the Theme moves them.
const CLAY_TOWARD_RULE := 0.5
const CLAY_DARKEN := 0.24
const SEA_TOWARD_TEAL_DEEP := 0.18
## How far the water is taken down again afterwards. The board's ambient is a
## pale blue-grey and it lifts every flat surface it touches; without this the
## sea came back the same value as the land it surrounds, and an island whose
## coast you cannot find is not an island.
const SEA_DARKEN := 0.34
const ROCK_TOWARD_MUTED := 0.28
const LAND_TOWARD_RULE := 0.34
const LAND_DARKEN := 0.22

## **needs decision.** The risk at which the road scale tops out. S2's
## `CONTESTED_RISK_MODIFIER` is 2 and the authored slice runs 0 through 5, so
## five is where the scale ends today; the eventual risk ceiling is a design
## decision nobody has taken.
const RISK_AT_FULL_DANGER := 5

## The curve the risk scale runs on. Below one, so the first step away from
## safety is the largest one on screen: a road that has just become contested
## must not look almost safe.
const RISK_CURVE := 0.62

## Concept key -> how far round the hue wheel that faction's wash stands from
## the Theme's own teal, in turns. Concept keys only: no faction proper name
## appears in this file or may appear in it. The six turns are spread so that no
## two are neighbours at the size a miniature is drawn.
const FACTION_HUE_TURN := {
	"michael": -0.28,
	"pirates": -0.40,
	"colonial_powers": 0.15,
	"fox_people": -0.34,
	"elves": 0.0,
	"cthulhu": 0.27
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

## The four spawn-socket roles B11 declares, each with the palette token the
## room distance marks it in. Token names, not values.
const ROLE_TOKEN := {
	"player": "teal",
	"occupant": "cream",
	"hostile": "danger",
	"item": "bronze"
}

## How a cell stands to the party right now, which is P7's whole legality
## vocabulary on the board: the tile the party is on, a tile a legal road
## reaches from there, and a tile it does not.
const CELL_ACTIVE := "active"
const CELL_REACHABLE := "reachable"
const CELL_DISTANT := "distant"

## **needs decision.** How far a cell's clay is carried toward the party's teal
## when the party stands on it, toward the bronze of an open road when one
## reaches it, and into the deep when none does. All three are framing choices
## about how loudly the board states reachability.
const ACTIVE_TOWARD_TEAL := 0.42
const REACHABLE_TOWARD_BRONZE := 0.30
const DISTANT_TOWARD_DEEP := 0.30


## Draw against `theme` from now on. A screen calls this once, with the Theme it
## has assigned to itself, before the board is built.
static func use_theme(theme: Theme) -> void:
	_theme = theme


## Back to the resource as authored. A suite calls this so one suite's contrast
## setting cannot colour the next one's assertions.
static func use_authored_theme() -> void:
	_theme = null


## Whether motion is to be suppressed, asked of the same Theme the colours are
## read from. The board asks here rather than reading the settings file, so one
## answer reaches the ring, the interface and every other surface alike.
static func reduced_motion() -> bool:
	return ThemeTokens.reduced_motion(_theme)


## One palette token, through P5's one door.
static func token(token_name: String) -> Color:
	return ThemeTokens.color(_theme, token_name)


static func deep() -> Color:
	return token("deep")


static func panel() -> Color:
	return token("panel")


static func bronze() -> Color:
	return token("bronze")


static func teal() -> Color:
	return token("teal")


static func cream() -> Color:
	return token("cream")


static func muted() -> Color:
	return token("muted")


static func danger() -> Color:
	return token("danger")


## The unheld blockout clay: what a cell nobody holds is made of. A stone
## between the Theme's `muted` green-grey and its `rule` bronze -- warm, low
## chroma, and the ground every other colour on the board is judged against.
static func clay() -> Color:
	return token("muted").lerp(token("rule"), CLAY_TOWARD_RULE).darkened(CLAY_DARKEN)


## The sea the island stands in. The Theme's `deep` carried toward `teal_deep`,
## so the water is the same green-black the whole interface opens on rather than
## a blue nothing else in the game uses.
static func sea() -> Color:
	return token("deep").lerp(token("teal_deep"), SEA_TOWARD_TEAL_DEEP).darkened(SEA_DARKEN)


## The island's rock, under every room's ground. `deep` lifted toward `muted`:
## darker than the clay it carries, and colder.
static func rock() -> Color:
	return token("deep").lerp(token("muted"), ROCK_TOWARD_MUTED)


## The island's own ground between the rooms -- jungle over volcanic soil.
static func land() -> Color:
	return token("teal_deep").lerp(token("rule"), LAND_TOWARD_RULE).darkened(LAND_DARKEN)


## The structure of the board that is not a place and not a road the party can
## take: a road it knows of but cannot use today, and the kerb that marks where a
## room's declared footprint ends. `rule_faint` is the Theme's own faintest
## structural line and that is exactly what these are.
##
## This is the one colour on the board that the old constants got wrong twice
## over: `muted` in the Theme is a *text* colour, light enough to read a caption
## in, and an unusable road painted in it was the brightest thing on the island.
static func faint_structure() -> Color:
	return token("rule_faint")


## The colour of one spawn socket's role marker.
static func role_tint(role: String) -> Color:
	if not ROLE_TOKEN.has(role):
		return muted()
	return token(str(ROLE_TOKEN[role]))


## The colour a faction's own pieces are drawn in -- a miniature, not a wash, so
## it is the tint at full strength and legible against the clay it stands on.
static func faction_colour(faction_id: String) -> Color:
	var concept_key := faction_id.trim_prefix("faction.")
	if faction_id.is_empty() or not FACTION_HUE_TURN.has(concept_key):
		return muted()
	return faction_hue(concept_key).lightened(0.18)


## The wash for a cell held by `faction_id` (`faction.<concept_key>`), or the
## unheld clay for "". An unknown faction ID is unheld rather than a guess: a
## proper name must never be able to enter through a colour lookup.
static func controller_tint(faction_id: String) -> Color:
	var concept_key := faction_id.trim_prefix("faction.")
	if faction_id.is_empty() or not FACTION_HUE_TURN.has(concept_key):
		return clay()
	return clay().lerp(faction_hue(concept_key), 0.52)


## The Theme's teal, turned round the hue wheel by the amount one concept key
## names, at that token's own saturation and value.
static func faction_hue(concept_key: String) -> Color:
	var base := token("teal")
	var turn := float(FACTION_HUE_TURN.get(concept_key, 0.0))
	return Color.from_hsv(fposmod(base.h + turn, 1.0), base.s, base.v)


## What one cell's tile is drawn in: who holds it, and how it stands to the
## party. Both facts share one surface because a cell has one colour; a second
## overlay saying reachability again is how the 2D board came to have a legend
## and a row of dots answering one question twice, and P7 deleted those.
##
## Ownership is the base and reachability is the modulation, so a cell that
## changes hands changes colour whether the party can reach it or not.
static func cell_tint(faction_id: String, state: String) -> Color:
	var base := controller_tint(faction_id)
	match state:
		CELL_ACTIVE:
			return base.lerp(token("teal"), ACTIVE_TOWARD_TEAL)
		CELL_REACHABLE:
			return base.lerp(token("bronze"), REACHABLE_TOWARD_BRONZE)
		_:
			return base.lerp(token("deep"), DISTANT_TOWARD_DEEP)


## A road's colour by the live risk the snapshot gives it. Risk is the only
## input: nothing here re-derives danger, and a contested road is already a
## higher number by the time it reaches this function.
static func risk_tint(risk_level: int) -> Color:
	# Around the hue wheel, not straight through it. A straight RGB blend from
	# the safe teal to the danger red passes through mud at exactly the risks the
	# authored slice uses most, which made a contested road look like a quiet
	# one; running the hue down from teal through green, yellow and orange to red
	# keeps every step of danger a colour the player can name. The two ends are
	# the Theme's own `teal` and `danger` -- the hue descends between them, which
	# is the direction that stays out of the blues.
	var safe := token("teal")
	var risky := token("danger")
	var danger_fraction := pow(clampf(float(risk_level) / float(RISK_AT_FULL_DANGER), 0.0, 1.0), RISK_CURVE)
	return Color.from_hsv(
		lerpf(safe.h, risky.h, danger_fraction),
		lerpf(safe.s, risky.s, danger_fraction),
		lerpf(safe.v, risky.v, danger_fraction)
	)
