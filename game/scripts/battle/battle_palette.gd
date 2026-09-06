class_name BattlePalette
extends RefCounted

## The vocabulary `content/presentation/vfx.registry.json` writes its effect
## palettes in, and the one place a word becomes a colour.
##
## Card P11: this file used to be the battle screen's palette -- seven tones the
## prototype restated because the Theme did not exist yet. It declares none of
## them now. Every UI colour the battle draws comes from
## `game/themes/bronze_vellum.tres` through `ThemeTokens`, and what is left here
## is the part the Theme has no opinion about: the words a VFX record authors
## its effect in.
##
## Those words split in two. Five of them name something the grammar already
## owns -- an effect in the game's teal is the same teal the cards are ruled in,
## and it should move when the player turns high contrast on -- so they resolve
## to a palette token. The rest are *game* colours: a warm skin tone, the brown
## of kicked-up dust, the white heart of a flash. They are marked as such below
## and they are the only hex values in this file.
##
## The factory never reads a hex out of a record; a word with no entry in either
## table is a validation failure rather than a silent black.

const ThemeTokensScript := preload("res://scripts/ui/theme_tokens.gd")

## Effect words the grammar answers, and the token that answers them. `gold` is
## the bright bronze the command cue and the dissolve already used.
const EFFECT_TOKENS := {
	"teal": "teal",
	"bronze": "bronze",
	"gold": "bronze_bright",
	"cream": "cream",
	"red": "danger"
}

## Effect words that are a colour in the fiction rather than in the interface.
# game colour: the authored effect palette of content/presentation/vfx.registry.json
# -- the white heart of a flash, the near-black of a shadow, the amber of a
# lantern, kicked-up dust and a warm skin tone. None of these is a UI colour and
# none of them changes when the interface does.
const EFFECT_GAME_COLORS := {
	"white": Color("fff6e2"),
	"black": Color("0b0f0e"),
	"amber": Color("d9a441"),
	"dust_brown": Color("8a6b4a"),
	"warm_skin": Color("f1c9ad"),
	"transparent": Color(0, 0, 0, 0),
	"palette_unestablished_pending_ayla_identity_lock": Color("d8cbb0")
}

## Ayla's palette is an Open decision: every one of her nine effect records
## carries the literal palette word
## `palette_unestablished_pending_ayla_identity_lock`, and this pass does not
## get to choose her colours. Her effects therefore render in the neutral
## cream-and-bronze the rest of the screen already owns, so her kit is visible
## and legible without asserting an identity nobody has approved.
## needs decision: Ayla's identity lock replaces this with her own two colours.
const AYLA_PENDING_WORD := "palette_unestablished_pending_ayla_identity_lock"


## Whether a record's palette word can be resolved at all. The validator and the
## presentation suite ask this; nothing invents a colour at a call site.
static func has_word(word: String) -> bool:
	return EFFECT_TOKENS.has(word) or EFFECT_GAME_COLORS.has(word)


## Every word this vocabulary knows, for a suite that wants to walk them.
static func words() -> PackedStringArray:
	var all := PackedStringArray()
	for word in EFFECT_TOKENS:
		all.append(str(word))
	for word in EFFECT_GAME_COLORS:
		all.append(str(word))
	return all


## One word, in the grammar `theme` is written in.
static func word_color(theme: Theme, word: String) -> Color:
	if EFFECT_TOKENS.has(word):
		return ThemeTokensScript.color(theme, str(EFFECT_TOKENS[word]))
	if EFFECT_GAME_COLORS.has(word):
		return EFFECT_GAME_COLORS[word]
	push_error("The VFX palette word '%s' has no colour. Add it to BattlePalette rather than inventing one at the call site." % word)
	return ThemeTokensScript.color(theme, "muted")


## A record's palette, resolved in the order it was authored. The first word is
## the effect's core, the second its edge, the third (when present) its accent.
static func effect_colors(theme: Theme, palette: Array) -> Array:
	var resolved: Array[Color] = []
	for word in palette:
		resolved.append(word_color(theme, str(word)))
	if resolved.is_empty():
		resolved.append(ThemeTokensScript.color(theme, "muted"))
	return resolved
