class_name BattleMetrics
extends RefCounted

## Where every size on the battle screen comes from.
##
## Card P14: P11 left the battle wearing the Theme's colours and the Theme's
## type, but still standing in a layout measured in pixels typed once at 1.0x --
## a 196-wide card with a 114-wide copy column, a 118-wide command plate, a
## 766-wide dock at a fixed corner. A roster card's status line was wider than
## its copy column at every scale, and at 1.3x the command labels lost their
## last four letters inside their plates.
##
## So the screen stopped stating widths. Every width here is *measured*: the
## face and the size come from `ThemeTokens`, the string comes from the thing
## that will actually be drawn, and the box is as wide as the string needs. A
## text scale that makes the type larger therefore makes the boxes larger, with
## nothing to keep in step by hand. What is still stated is the placeholder
## illustration's own geometry -- the portrait gutter, the diamond's drawn
## height -- which is art rather than type, does not move with the type scale,
## and is marked as the blockout dimension it is.
##
## There is one owner of these numbers and it is this file. `paper_card.gd`,
## `paper_skill_diamond.gd` and `battle_prototype.gd` all ask here; none of them
## measures for itself, so a card and the column it stands in cannot disagree.

const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

# ---------------------------------------------------------------------------
# The strings the screen actually draws. They are here because a box measured
# against one string and filled with another is the bug this card exists to fix.

## A roster card's status line. The words are the simulation's facts; the only
## things this file decides are how wide a box has to be to hold them and where
## the line is allowed to break. The space before each separator is a
## non-breaking one, so a bullet stays with the fact it follows instead of
## beginning the next line on its own when the column wraps.
const STATUS_FORMAT := "VIT %d/%d\u00a0\u2022  GRD %d\u00a0\u2022  %s"
## The widest line the format above can produce in this game: three digits of
## vitality on both sides of the slash, three of guard, and the longer of the
## two states. The copy column is measured against this rather than against the
## actor in front of the player, so five cards in a row share one column width
## and a card does not change width when its actor takes a hit.
const STATUS_WIDEST := "VIT 100/100\u00a0\u2022  GRD 100\u00a0\u2022  DEFEATED"
## The two states `STATUS_FORMAT` ends in.
const STATE_READY := "READY"
const STATE_DEFEATED := "DEFEATED"
## The mark a Shaken actor's card carries beside its name.
const SHAKEN_MARK := "SHAKEN"
## The name the copy column falls back to being measured against when nobody has
## said which names the roster is carrying -- a card built in a review scene or
## a suite. In the battle the real names are handed in, hostiles included, so
## "RAZORBEAK" is measured rather than assumed to be shorter than a heroine's.
const FALLBACK_NAME := "MICHAEL CORRIGAN"
## How many lines that name may take beside the Shaken mark before the rest of
## it is trimmed with an ellipsis.
const NAME_MAXIMUM_LINES := 2

## How many lines the status line is allowed to take at its natural width. Two:
## one line would make a card wider than a fifth of the screen at 1.3x, and
## three would push the copy past the portrait it stands beside. When the screen
## is too narrow to give a card its natural width the label wraps to as many
## lines as it needs and the card grows taller -- it never clips.
const STATUS_NATURAL_LINES := 2

# ---------------------------------------------------------------------------
# The placeholder illustration's own geometry. These are blockout dimensions in
# the sense of rule 16: they describe cut-paper art that the illustrated
# portrait package replaces by name, not type, so they do not scale with the
# type scale.

## **needs decision.** How wide the portrait column on the left of a roster card
## is, in pixels at the screen's design height. `paper_card.gd` draws the
## cut-paper figure inside it; the copy column begins where it ends. It goes
## when `content/art/placeholders.json` names a real portrait package.
const PORTRAIT_GUTTER := 74.0
## **needs decision.** How tall that drawn figure stands, in the same pixels.
const PORTRAIT_HEIGHT := 101.0
## **needs decision.** The height of the bronze diamond a command is drawn as,
## above its name plate. The diamond is a shape, not a word, so it keeps its
## size when the type grows; only its plate and its width follow the type.
const DIAMOND_ART_HEIGHT := 62.0

# ---------------------------------------------------------------------------
# The screen's own spacing. Stated once, in one place, and used by every caller.

## The gap between a card's frame and the copy inside it.
const CARD_INSET := 12.0
## The gap between the top of a card and the first line of copy.
const CARD_COPY_TOP := 14.0
## The gap between two roster columns, and between two rows of cards.
const RAIL_GUTTER := 16.0
const RAIL_ROW_GAP := 8.0
## The band heading above a roster column, measured from the type it is set in.
const RAIL_HEADING_GAP := 4.0
## The margin the bottom HUD keeps from the edges of the screen, and the gutter
## between the roster rail and the command dock.
const SCREEN_MARGIN := 34.0
const BOTTOM_MARGIN := 24.0
const RAIL_DOCK_GUTTER := 24.0
## The dock's own padding, and the gap between two commands inside it.
const DOCK_PADDING := 16.0
const DOCK_ROW_GAP := 6.0
## The padding a command name keeps inside its plate, left and right together.
const PLATE_PADDING := 12.0
## The padding around a Label used as a mark rather than a line of prose.
const MARK_PADDING := 10.0
## The gap a line of type keeps above and below itself inside a plate.
const LINE_PADDING := 4.0
## The gap between the card's copy and the Composure well beneath it.
const WELL_GAP := 6.0

# ---------------------------------------------------------------------------
# Measurement.

## The face the screen's type is actually set in. The Theme names a monospace
## family for stable IDs and nothing else, so a Label and a `draw_string` both
## land on the engine's default face; measuring in that face is measuring what
## will be drawn.
static func face(theme: Theme) -> Font:
	if theme != null and theme.default_font != null:
		return theme.default_font
	return ThemeDB.fallback_font


static func line_height(theme: Theme, step: String) -> float:
	return face(theme).get_height(ThemeTokensScript.font_size(theme, step))


## The height of one line of a Label set in this step. A Label adds the Theme's
## own `line_spacing` to the face's height, and a box measured without it is a
## box one gap short per line, so the constant is read rather than guessed.
static func label_line_height(theme: Theme, step: String) -> float:
	return line_height(theme, step) + line_spacing(theme)


static func line_spacing(theme: Theme) -> float:
	if theme != null and theme.has_constant("line_spacing", "Label"):
		return float(theme.get_constant("line_spacing", "Label"))
	var fallback := ThemeDB.get_default_theme()
	if fallback != null and fallback.has_constant("line_spacing", "Label"):
		return float(fallback.get_constant("line_spacing", "Label"))
	return 0.0


static func text_width(theme: Theme, step: String, text: String) -> float:
	var size := ThemeTokensScript.font_size(theme, step)
	return face(theme).get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x


## The width of the widest single word in `text`. A label given at least this
## much can always wrap; a label given less than this is the one case where
## text has nowhere to go, so it is the floor every box here is clamped to.
static func widest_word(theme: Theme, step: String, text: String) -> float:
	var widest := 0.0
	for word in text.split(" ", false):
		widest = maxf(widest, text_width(theme, step, word))
	return widest


## How many lines `text` takes when it is wrapped into `width`.
static func wrapped_lines(theme: Theme, step: String, text: String, width: float) -> int:
	var size := ThemeTokensScript.font_size(theme, step)
	var height := face(theme).get_multiline_string_size(
		text, HORIZONTAL_ALIGNMENT_LEFT, width, size).y
	return maxi(1, int(round(height / maxf(1.0, face(theme).get_height(size)))))


## The narrowest width at which `text` wraps into `lines` lines or fewer. This
## is the whole of "the copy column derives from the type scale": the answer is
## a measurement of the Theme's own font at the Theme's own size, so it grows
## with the scale and there is no second number to keep in step.
static func width_for_lines(theme: Theme, step: String, text: String, lines: int) -> float:
	var high := ceilf(text_width(theme, step, text))
	if lines <= 1:
		return high
	var low := ceilf(widest_word(theme, step, text))
	if wrapped_lines(theme, step, text, low) <= lines:
		return low
	# Eight halvings resolve a 1000-pixel span to about four pixels, which is
	# below the width of a space in any step of this scale.
	for _step in range(8):
		var middle := floorf((low + high) * 0.5)
		if middle <= low:
			break
		if wrapped_lines(theme, step, text, middle) <= lines:
			high = middle
		else:
			low = middle
	return high

# ---------------------------------------------------------------------------
# The roster card.

## The card's copy column at its natural width: wide enough for the status line
## in two lines, and for the name beside its Shaken mark.
static func card_copy_width(theme: Theme, names := []) -> float:
	var status := width_for_lines(theme, "caption", STATUS_WIDEST, STATUS_NATURAL_LINES)
	return maxf(status, name_floor(theme, names))


## The narrowest copy column the card may be squeezed to before the screen gives
## up width elsewhere: enough for the longest unbreakable word in either line,
## so the copy always has somewhere to wrap to and never has to clip.
static func card_copy_floor(theme: Theme, names := []) -> float:
	return maxf(widest_word(theme, "caption", STATUS_WIDEST), name_word_floor(theme, names))


## The narrowest name column that never breaks a name inside a word: the widest
## single word in any name the roster carries. Below this there is nowhere for a
## name to go, so it is the floor; above it a name wraps, and above
## `name_floor` the Shaken mark stands beside it as well.
static func name_word_floor(theme: Theme, names: Array) -> float:
	var widest := 0.0
	for entry in roster_names(names):
		widest = maxf(widest, widest_word(theme, "body", str(entry)))
	return widest


## The column at which the Shaken mark can stand beside the name rather than
## under it.
static func name_floor(theme: Theme, names: Array) -> float:
	return name_word_floor(theme, names) + shaken_mark_size(theme).x + RAIL_ROW_GAP


## Where the Shaken mark goes. Beside the name while the column can hold both
## whole; on its own line under the name when it cannot -- one rule, and the
## card is measured to be as tall as the answer, so neither the name nor the
## mark is ever the thing that gives way.
static func mark_beside_name(theme: Theme, copy_width: float, names := []) -> bool:
	return copy_width >= name_floor(theme, names)


static func roster_names(names: Array) -> Array:
	return names if not names.is_empty() else [FALLBACK_NAME]


## The longest of them, which is the one the row's height is measured on.
static func longest_name(theme: Theme, names: Array) -> String:
	var longest := ""
	var width := -1.0
	for entry in roster_names(names):
		var measured := text_width(theme, "body", str(entry))
		if measured > width:
			width = measured
			longest = str(entry)
	return longest


static func card_width(theme: Theme, copy_width: float) -> float:
	return PORTRAIT_GUTTER + copy_width + CARD_INSET


## The card's height, for the copy column it was actually given. The name takes
## up to two lines, the status takes as many as its width leaves it, and the
## Composure well sits under both; a card is never shorter than the drawn
## portrait beside the copy.
static func card_height(theme: Theme, copy_width: float, names := []) -> float:
	var copy_height := CARD_COPY_TOP + name_height(theme, copy_width, names) + status_height(theme, copy_width)
	return maxf(copy_height, PORTRAIT_HEIGHT) + WELL_GAP + well_height(theme) + CARD_INSET


## The name row: the longest name in the game, wrapped into what the Shaken
## mark leaves it, and never taller than two lines or shorter than the mark.
static func name_width(theme: Theme, copy_width: float, names := []) -> float:
	if not mark_beside_name(theme, copy_width, names):
		return maxf(1.0, copy_width)
	return maxf(1.0, copy_width - shaken_mark_size(theme).x - RAIL_ROW_GAP)


static func name_height(theme: Theme, copy_width: float, names := []) -> float:
	var lines := mini(NAME_MAXIMUM_LINES,
		wrapped_lines(theme, "body", longest_name(theme, names), name_width(theme, copy_width, names)))
	var height := label_line_height(theme, "body") * float(lines)
	if mark_beside_name(theme, copy_width, names):
		return maxf(height, shaken_mark_size(theme).y)
	return height + shaken_mark_size(theme).y


static func status_height(theme: Theme, copy_width: float) -> float:
	return label_line_height(theme, "caption") * float(
		wrapped_lines(theme, "caption", STATUS_WIDEST, copy_width))


## The Composure well: ten pips in a dark trough, as tall as one caption line
## with a little air, so it grows with the type it sits under rather than
## staying a 24-pixel bar while everything above it grows.
static func well_height(theme: Theme) -> float:
	return label_line_height(theme, "caption") + LINE_PADDING * 2.0


## The Shaken mark's plate, measured from the word it holds.
static func shaken_mark_size(theme: Theme) -> Vector2:
	return Vector2(
		text_width(theme, "caption", SHAKEN_MARK) + MARK_PADDING * 2.0,
		label_line_height(theme, "caption") + LINE_PADDING * 2.0)


static func rail_heading_height(theme: Theme) -> float:
	return label_line_height(theme, "mono") + RAIL_HEADING_GAP

# ---------------------------------------------------------------------------
# The command dock.

## One command plate, measured from the longest name it will ever hold. Every
## diamond in the dock is given the same width -- a row of commands that were
## each as wide as their own word would read as a ransom note -- so the widest
## name decides, and no name is ever trimmed inside its plate.
static func diamond_size(theme: Theme, names: Array) -> Vector2:
	var widest := 0.0
	for entry in names:
		widest = maxf(widest, text_width(theme, "caption", str(entry).to_upper()))
	return Vector2(
		ceilf(widest + PLATE_PADDING * 2.0),
		DIAMOND_ART_HEIGHT + plate_height(theme))


## The plate a command's name sits on, at the foot of its diamond.
static func plate_height(theme: Theme) -> float:
	return label_line_height(theme, "caption") + LINE_PADDING * 2.0


## How many commands fit across a dock of this width, and therefore how many
## rows the seven take. The dock flows: it is never given a row count to obey.
static func commands_per_row(dock_width: float, plate_width: float) -> int:
	var inner := dock_width - DOCK_PADDING * 2.0
	return maxi(1, int(floorf((inner + DOCK_ROW_GAP) / (plate_width + DOCK_ROW_GAP))))


static func dock_height(theme: Theme, rows: int, plate: Vector2) -> float:
	return (DOCK_PADDING * 2.0
		+ label_line_height(theme, "body") + DOCK_ROW_GAP
		+ float(rows) * (plate.y + DOCK_ROW_GAP) - DOCK_ROW_GAP)


static func dock_natural_width(plate: Vector2, commands: int, per_row: int) -> float:
	var across := mini(commands, maxi(1, per_row))
	# A pixel of air, so a row that comes out exactly as wide as the dock is not
	# wrapped by a rounding error into a row of one fewer.
	return DOCK_PADDING * 2.0 + float(across) * plate.x + float(across - 1) * DOCK_ROW_GAP + 2.0


# ---------------------------------------------------------------------------
# The bottom of the screen, where the two of them meet.

## The one place the roster rail and the command dock are fitted against each
## other. Both want their natural width; when the screen cannot give both, the
## rail's cards are squeezed towards their floor (their copy wraps to another
## line and the cards grow taller) and the dock's commands flow onto another
## row -- because a command name may not be trimmed and a status line may not be
## clipped, but either of them may take more room downwards.
##
## Returns `card_width`, `copy_width`, `card_height`, `rail_width`,
## `dock_width`, `plate` and `dock_rows`.
static func bottom_hud(theme: Theme, viewport_width: float, bands: int, command_names: Array, roster: Array = []) -> Dictionary:
	var plate := diamond_size(theme, command_names)
	var commands := command_names.size()
	var available := viewport_width - SCREEN_MARGIN * 2.0 - RAIL_DOCK_GUTTER

	var natural_copy := card_copy_width(theme, roster)
	var natural_card := card_width(theme, natural_copy)
	var natural_rail := float(bands) * (natural_card + RAIL_GUTTER) - RAIL_GUTTER
	# The dock asks for the widest arrangement that is no taller than two rows,
	# which is the shape P6 gave it and the shape seven commands read best in.
	var natural_per_row := int(ceilf(float(commands) / 2.0))
	var natural_dock := dock_natural_width(plate, commands, natural_per_row)

	var floor_copy := card_copy_floor(theme, roster)
	var floor_rail := float(bands) * (card_width(theme, floor_copy) + RAIL_GUTTER) - RAIL_GUTTER
	var floor_dock := dock_natural_width(plate, commands, 1)

	# The dock is served first, and never more than it asks for. Its plates
	# cannot be narrowed -- a trimmed command name is the fault this card
	# exists to remove -- and a command surface that reflows from three across
	# to one between two text scales is a different screen rather than the same
	# screen at another size. The rail then takes what is left: its cards are
	# squeezed towards their floor, their copy wraps onto another line, and the
	# cards grow taller, none of which loses a word.
	var dock_width := minf(natural_dock, maxf(floor_dock, available - floor_rail))
	var rail_width := maxf(floor_rail, available - dock_width)
	if natural_rail + natural_dock <= available:
		rail_width = natural_rail
		dock_width = natural_dock
	var copy_width := clampf(
		rail_width / float(bands) - RAIL_GUTTER - PORTRAIT_GUTTER - CARD_INSET,
		floor_copy, natural_copy)
	var per_row := mini(commands, commands_per_row(dock_width, plate.x))
	var rows := int(ceilf(float(commands) / float(maxi(1, per_row))))
	return {
		"copy_width": copy_width,
		"card_width": card_width(theme, copy_width),
		"card_height": card_height(theme, copy_width, roster),
		"rail_width": rail_width,
		"dock_width": dock_width,
		"plate": plate,
		"dock_rows": rows,
		"dock_height": dock_height(theme, rows, plate),
	}
