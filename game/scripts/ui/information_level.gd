class_name InformationLevel
extends RefCounted

## Brief section 14's three levels, and the one place their names and their
## marks are stated.
##
## The autoload that classifies events (`InformationSurface`) and the components
## that draw them (`VellumNotice`, `VellumJournalEntry`) all read this, so
## "Notable" is one word in one file rather than three copies that drift.
##
## * **Ambient** -- most changes simply appear in the world. Silent: it marks
##   the journal and does nothing else on screen.
## * **Notable** -- a companion, messenger, journal or map update identifies a
##   development without interrupting the player.
## * **Urgent** -- the game interrupts, and only for an immediate and
##   understandable consequence involving the party, a major relationship, a
##   critical player-faction location, or a final-stage threat.

enum Level { AMBIENT, NOTABLE, URGENT }

## The word the player sees.
static func name_of(level: int) -> String:
	match level:
		Level.NOTABLE: return "NOTABLE"
		Level.URGENT: return "URGENT"
		_: return "AMBIENT"


## The palette token a level is marked in. Ambient is muted because it is not
## asking for anything; Urgent is the soft danger, never a flashing red.
static func token_of(level: int) -> String:
	match level:
		Level.NOTABLE: return "bronze"
		Level.URGENT: return "danger_soft"
		_: return "muted"


## Whether a level puts anything on screen at all.
static func is_silent(level: int) -> bool:
	return level == Level.AMBIENT


## Whether a level is allowed to take the player's attention. Exactly one is.
static func interrupts(level: int) -> bool:
	return level == Level.URGENT
