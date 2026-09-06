extends Node

## The calm information surface (card B13, brief section 14).
##
## One question, answered in one place: **what does this event deserve?**
## Ambient, Notable or Urgent, by the brief's own rules and nothing else. Every
## screen asks here rather than deciding for itself, because "the interface must
## be friendly rather than menacing" is a property of the whole game and cannot
## be held by six screens that each guess.
##
## What this autoload owns:
##
## * the classification (`classify`), by explicit rule;
## * the presentation journal (`journal`) -- every event, at every level,
##   including the Ambient ones that show nothing, because "it only marks the
##   journal" has to mean something is marked;
## * the reading of that journal while the game is paused (`open_journal`),
##   which is brief section 14's "strategic planning should be available while
##   paused";
## * S6's explanation, shown before a directive is confirmed
##   (`directive_explanation_lines`, `can_confirm_directive`);
## * the live Theme (`current_theme`), rebuilt from B9's three settings so that
##   changing a setting changes every screen at once.
##
## What it does not own: prose. The words in a journal line are written by the
## bridge, beside the fact they describe (`Project42ExpeditionBridge.strategic_surface`),
## so the engine never rewords what the simulation said.

## Emitted after `apply_settings` rebuilds the Theme. Screens connect and assign
## the new Theme; that is the whole of "the settings change the Theme live".
signal theme_rebuilt(theme: Theme)

## Emitted for every event that reaches `present`, at whatever level it was
## given -- Ambient included, so a journal view can update without a second
## channel. The notice, if any, is built by the caller from `VellumNotice.present`.
signal event_presented(level: int, event: Dictionary)

## How many presented events are kept. The same window S11's `StrategicJournal`
## keeps whole, for the same reason and with the same provisional status: it is
## how far back a player can read, and nothing derives it.
const JOURNAL_LINES := 256

## The reason the journal gives `GamePause` while it is open. The surface that
## gives a reason owns its name, as `SettingsPanel.PAUSE_REASON` does.
const PAUSE_REASON := "journal"

## Event kinds that are a development the player could use -- a map update, a
## faction's fortunes, a yard turning something out. Brief section 14's Notable
## level: "a companion, messenger, journal, map update, or rumor identifies a
## potentially useful development without interrupting the player."
##
## `hour_passed` and `force_moved` are deliberately absent. An hour passing and
## a force crossing one road are the island breathing; a surface that reported
## them would be brief section 14's "quest log filled automatically by
## simulation noise", which is on the avoid list rather than the feature list.
const NOTABLE_KINDS: PackedStringArray = [
	"force_arrived", "force_halted", "faction_eliminated", "recovery_link_lost", "control_changed",
]

## Notable only when it is the player's own yard: another faction's production
## is not something the player can see, and reporting it would be an alarm for
## every faction action -- the second entry on the brief's avoid list.
const OWN_FACTION_NOTABLE_KINDS: PackedStringArray = ["machine_produced"]

## Kinds at which a force reaching a place is the event. Used by the two Urgent
## rules that are about a place.
const ARRIVAL_KINDS: PackedStringArray = ["force_arrived", "force_halted", "control_changed"]

## Who the player is, and what they have at stake. Empty by default: a surface
## that has been told nothing interrupts for nothing, which is the right failure.
var player_faction_id := ""
var party_character_ids: PackedStringArray = []
var party_cell_id := ""
var critical_core_cell_ids: PackedStringArray = []
var major_relationship_character_ids: PackedStringArray = []

var _journal: Array[Dictionary] = []
var _theme: Theme = null
var _game_pause: Node = null
var _journal_open := false


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	_theme = ThemeTokens.build()
	_load_persisted_settings()


# ---------------------------------------------------------------------------
# What the player has at stake
# ---------------------------------------------------------------------------

## Tell the surface who the player is. Keys: `player_faction_id`,
## `party_character_ids`, `party_cell_id`, `critical_core_cell_ids`,
## `major_relationship_character_ids`. A key that is absent is left as it was,
## so a screen can update the party's cell without restating the rest.
func set_context(context: Dictionary) -> void:
	if context.has("player_faction_id"):
		player_faction_id = str(context["player_faction_id"])
	if context.has("party_character_ids"):
		party_character_ids = _strings(context["party_character_ids"])
	if context.has("party_cell_id"):
		party_cell_id = str(context["party_cell_id"])
	if context.has("critical_core_cell_ids"):
		critical_core_cell_ids = _strings(context["critical_core_cell_ids"])
	if context.has("major_relationship_character_ids"):
		major_relationship_character_ids = _strings(context["major_relationship_character_ids"])


# ---------------------------------------------------------------------------
# The classification
# ---------------------------------------------------------------------------

## What this event deserves.
##
## Urgent has exactly four permissions, and they are the brief's four: the
## party, a major relationship, a critical player-faction location, and a
## final-stage threat. Nothing else may interrupt, however dramatic it reads.
## An event whose kind this surface does not recognise is Ambient -- an unknown
## development is not an emergency.
func classify(event: Dictionary) -> int:
	if not urgent_reason(event).is_empty():
		return InformationLevel.Level.URGENT
	var kind := str(event.get("kind", ""))
	if NOTABLE_KINDS.has(kind):
		return InformationLevel.Level.NOTABLE
	if OWN_FACTION_NOTABLE_KINDS.has(kind) and str(event.get("faction_id", "")) == player_faction_id:
		return InformationLevel.Level.NOTABLE
	return InformationLevel.Level.AMBIENT


## Why this event may interrupt, in the words of the rule that permits it, or
## empty when nothing does. Stated as a reason rather than a boolean so an
## Urgent notice can say which of the four it is, and so a test can name the
## rule it is exercising.
func urgent_reason(event: Dictionary) -> String:
	var kind := str(event.get("kind", ""))
	var cell_id := str(event.get("cell_id", ""))
	var faction_id := str(event.get("faction_id", ""))
	var characters := _strings(event.get("character_ids", []))

	# 1. The party. Either the event names someone in it, or something has
	#    arrived where it is standing.
	for character_id in characters:
		if party_character_ids.has(character_id):
			return "the party"
	if ARRIVAL_KINDS.has(kind) and not party_cell_id.is_empty() and cell_id == party_cell_id:
		return "the party"

	# 2. A major relationship.
	for character_id in characters:
		if major_relationship_character_ids.has(character_id):
			return "a major relationship"

	# 3. A critical player-faction location, reached by somebody else. The
	#    player's own force coming home to its own core is not an emergency.
	if ARRIVAL_KINDS.has(kind) and critical_core_cell_ids.has(cell_id) and faction_id != player_faction_id:
		return "a critical location"

	# 4. A final-stage threat. Brief section 16's recovery chain is the only
	#    final stage this simulation models today: the player's own faction
	#    losing a way back, or being taken off the board, is the end of the
	#    campaign arriving. A threat model with more stages than that would add
	#    its own kinds here rather than widen these.
	if faction_id == player_faction_id and not player_faction_id.is_empty():
		if kind == "faction_eliminated" or kind == "recovery_link_lost":
			return "a final-stage threat"
	return ""


## Classify an event, mark the journal, and tell the screens. Returns the level,
## so a caller can hand it straight to `VellumNotice.present` -- which builds
## nothing at all for an Ambient one.
func present(event: Dictionary) -> int:
	var level := classify(event)
	var line := {
		"day": int(event.get("day", 0)),
		"hour": int(event.get("hour", 0)),
		"level": level,
		"kind": str(event.get("kind", "")),
		"prose": str(event.get("prose", event.get("kind", ""))),
	}
	_journal.append(line)
	while _journal.size() > JOURNAL_LINES:
		_journal.remove_at(0)
	event_presented.emit(level, event)
	return level


# ---------------------------------------------------------------------------
# The journal, and reading it while paused
# ---------------------------------------------------------------------------

## Everything presented, oldest first, in `VellumJournalEntry`'s shape.
func journal() -> Array[Dictionary]:
	return _journal.duplicate()


## The most recent `count` lines, newest last.
func recent_journal(count: int) -> Array[Dictionary]:
	if count <= 0 or _journal.is_empty():
		return []
	return _journal.slice(maxi(0, _journal.size() - count))


func clear_journal() -> void:
	_journal.clear()


## Open the journal for reading. The island is held while it is open, because
## the brief refuses to punish a player for studying the board.
func open_journal() -> void:
	if _journal_open:
		return
	_journal_open = true
	_pause_service().pause(PAUSE_REASON)


func close_journal() -> void:
	if not _journal_open:
		return
	_journal_open = false
	_pause_service().resume(PAUSE_REASON)


func journal_is_open() -> bool:
	return _journal_open


# ---------------------------------------------------------------------------
# S6's explanation, before the player confirms
# ---------------------------------------------------------------------------

## The explanation the player must be shown before a directive is confirmed,
## as labelled lines.
##
## Brief section 5.9: "Before confirmation, the faction should explain how it
## understands a major directive in ordinary language, including obvious risks
## and competing commitments." Every word here comes from
## `ExpeditionState::explain_directive`; this function labels and orders it and
## writes nothing of its own.
func directive_explanation_lines(directive: Dictionary) -> PackedStringArray:
	var lines: PackedStringArray = []
	var explanation := directive.get("explanation", {}) as Dictionary
	if explanation.is_empty():
		return lines
	_append_line(lines, "What it becomes", str(explanation.get("goal", "")))
	_append_line(lines, "Why there", str(explanation.get("why_target", "")))
	_append_line(lines, "What it spends", str(explanation.get("resources", "")))
	for blocker in _strings(explanation.get("blockers", [])):
		_append_line(lines, "In the way", blocker)
	_append_line(lines, "It breaks off when", str(explanation.get("withdrawal_conditions", "")))
	if bool(explanation.get("party_could_help", false)):
		_append_line(lines, "The party", "could reach this and matter there.")
	return lines


## Whether the confirm control may be offered at all. A directive whose
## explanation has not been computed cannot be confirmed: the player agrees to
## the explanation, so an unexplained directive is a directive nobody agreed to.
func can_confirm_directive(directive: Dictionary) -> bool:
	return not directive_explanation_lines(directive).is_empty()


# ---------------------------------------------------------------------------
# B9's settings, applied through the Theme
# ---------------------------------------------------------------------------

## The Theme every screen should be wearing.
func current_theme() -> Theme:
	if _theme == null:
		_theme = ThemeTokens.build()
	return _theme


## Rebuild the Theme from B9's three settings and tell every screen. The
## settings panel calls this as the player moves the slider, which is why the
## change is visible without reopening anything.
func apply_settings(text_scale: float, high_contrast: bool, reduced_motion: bool) -> Theme:
	_theme = ThemeTokens.build(text_scale, high_contrast, reduced_motion)
	theme_rebuilt.emit(_theme)
	return _theme


## Read the three settings from the file `SettingsPanel` owns, so the Theme is
## already right on the first frame of a session rather than only after the
## player opens the settings page. The path, the section and the defaults are
## the panel's constants: there is one settings file and one set of key names.
func _load_persisted_settings() -> void:
	var config := ConfigFile.new()
	if config.load(SettingsPanel.SETTINGS_PATH) != OK:
		return
	var section := SettingsPanel.SECTION
	apply_settings(
		float(config.get_value(section, "text_scale", SettingsPanel.DEFAULT_TEXT_SCALE)),
		bool(config.get_value(section, "high_contrast", false)),
		bool(config.get_value(section, "reduced_motion", false)))


# ---------------------------------------------------------------------------

func _pause_service() -> Node:
	if _game_pause == null:
		_game_pause = get_node_or_null("/root/GamePause")
	if _game_pause == null:
		# Only reachable in a review scene or a suite that runs without the
		# autoload; the journal still opens, it simply holds nothing.
		return _NullPause.new()
	return _game_pause


func _append_line(lines: PackedStringArray, label: String, value: String) -> void:
	if value.strip_edges().is_empty():
		return
	lines.append("%s: %s" % [label, value])


func _strings(value: Variant) -> PackedStringArray:
	var result: PackedStringArray = []
	if value is PackedStringArray:
		return value
	if value is Array:
		for entry in value as Array:
			result.append(str(entry))
	return result


class _NullPause extends Node:
	func pause(_reason: String) -> bool:
		return false

	func resume(_reason: String) -> bool:
		return false
