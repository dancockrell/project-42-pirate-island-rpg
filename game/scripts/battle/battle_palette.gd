class_name BattlePalette
extends RefCounted

## The one place the battle screen names a colour.
##
## Rule 17 of this presentation pass: until `game/themes/bronze_vellum.tres`
## (P5) exists, the prototypes' own constants stay the owner and no new hex
## value is invented anywhere else. Every colour the battle screen and the VFX
## factory use is written here once. When the Theme lands it becomes the owner
## of DEEP, PANEL, CREAM, TEAL, BRONZE, DANGER and MUTED, and this file's job
## shrinks to the effect palette words below -- which the Theme has no opinion
## about, because they are the vocabulary `content/presentation/vfx.registry.json`
## authors its effects in.

## The six the two prototypes already carried, unchanged in value.
const DEEP := Color("101817")
const PANEL := Color("182321")
const CREAM := Color("eadfca")
const TEAL := Color("4fc7b4")
const BRONZE := Color("b78a4b")
const DANGER := Color("c24e45")
## The muted grey-green the expedition surface already uses for inert text.
const MUTED := Color("9eb0a7")
## The battle screen's backdrop behind the plate, as the prototype wrote it.
const VOID := Color("07100f")
## Warm gold, the value the prototype already used for the action cue and the
## bronze highlight ring on the skill diamonds.
const GOLD := Color("e4b75e")

## Ayla's palette is an Open decision: every one of her nine effect records
## carries the literal palette word
## `palette_unestablished_pending_ayla_identity_lock`, and this pass does not
## get to choose her colours. Her effects therefore render in the neutral
## cream-and-bronze the rest of the screen already owns, so her kit is visible
## and legible without asserting an identity nobody has approved.
## needs decision: Ayla's identity lock replaces this with her own two colours.
const AYLA_PENDING_IDENTITY_LOCK := Color("d8cbb0")

## The vocabulary `vfx.registry.json` writes its palettes in. A record names
## colours by word; the factory never reads a hex value out of a record, and a
## word with no entry here is a validation failure rather than a silent black.
const EFFECT_COLORS := {
	"teal": TEAL,
	"bronze": BRONZE,
	"gold": GOLD,
	"cream": CREAM,
	"white": Color("fff6e2"),
	"black": Color("0b0f0e"),
	"red": DANGER,
	"amber": Color("d9a441"),
	"dust_brown": Color("8a6b4a"),
	"warm_skin": Color("f1c9ad"),
	"transparent": Color(0, 0, 0, 0),
	"palette_unestablished_pending_ayla_identity_lock": AYLA_PENDING_IDENTITY_LOCK
}

## A record's palette, resolved in the order it was authored. The first word is
## the effect's core, the second its edge, the third (when present) its accent.
static func effect_colors(palette: Array) -> Array:
	var resolved: Array[Color] = []
	for word in palette:
		var key := str(word)
		if not EFFECT_COLORS.has(key):
			push_error("The VFX palette word '%s' has no colour. Add it to BattlePalette.EFFECT_COLORS rather than inventing one at the call site." % key)
			resolved.append(MUTED)
			continue
		resolved.append(EFFECT_COLORS[key])
	if resolved.is_empty():
		resolved.append(MUTED)
	return resolved
