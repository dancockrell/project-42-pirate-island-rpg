extends SceneTree

## P5: the calm information surface, and the grammar it draws in.
##
## Card B13's Done-when is the first test below: a hundred strategic events out
## of an ordinary day produce at most one Urgent. The fixture is deliberately an
## ordinary day -- forces on the roads, yards working, the fighting nowhere near
## the party -- because a hundred events in which the island storms the player's
## door *should* interrupt, and a fixture rigged so that no event could ever be
## urgent would prove nothing. The single Urgent one is the enemy arriving in
## the cell the party is standing in, which is exactly the brief's first
## permission.
##
## The rest holds the grammar: the Theme is the one owner of the palette (every
## StyleBox in `bronze_vellum.tres` is painted in a palette token, asserted
## colour by colour), the accessibility settings change it live, and every
## component instantiates under it and draws without an error line.

## The Done-when's stream length.
const EVENT_COUNT := 100

## The party's cell in the fixture, and the one cell an arrival may interrupt
## for.
const PARTY_CELL := "world.cell.black_beach.reception_terrace"
const PLAYER_FACTION := "faction.michael"
const CORE_CELL := "world.cell.black_beach.estate"

## Cells the ordinary day's traffic passes through. None of them is the party's
## cell or the player's core, which is what makes the day ordinary.
const ORDINARY_CELLS := [
	"world.cell.black_beach", "world.cell.black_beach.landing",
	"world.cell.black_beach.shore", "world.cell.black_beach.wreck",
	"world.cell.black_beach.tomb_mouth",
]

const OTHER_FACTIONS = ["faction.pirates", "faction.colonial_powers", "faction.fox_people", "faction.elves"]

## Every kind `journal_entry_dictionary` can project, in the order
## `StrategicEvent` declares them.
const EVENT_KINDS = [
	"hour_passed", "force_departed", "force_moved", "force_arrived",
	"force_halted", "recovery_link_lost", "faction_eliminated", "machine_produced",
]

var failures := 0
var surface: Node
var stage: Control


func _init() -> void:
	await process_frame
	surface = root.get_node_or_null("/root/InformationSurface")
	check(surface != null, "InformationSurface must be registered as an autoload")
	if surface == null:
		finish()
		return
	stage = Control.new()
	stage.name = "GrammarStage"
	stage.size = Vector2(1280, 720)
	root.add_child(stage)

	test_theme_owns_the_palette()
	test_hundred_events_produce_at_most_one_urgent()
	test_the_level_rules_by_example()
	test_the_explanation_comes_before_the_confirmation()
	test_the_journal_holds_the_island_while_it_is_read()
	await test_settings_change_the_theme_live()
	await test_every_component_instantiates_under_the_theme()
	await test_the_bridge_carries_directives_and_the_journal()
	finish()


# ---------------------------------------------------------------------------
# The Theme is the one owner of the palette
# ---------------------------------------------------------------------------

## Every colour in the resource is a palette token.
##
## This is what "one owner" has to mean in a file a person can hand-edit: a
## StyleBox painted in a hex nobody named would be a second palette, and it
## would be invisible until it drifted. The walk below finds it the day it is
## added, and `ThemeTokens._swap_palette` -- which recolours by value -- is
## correct exactly while this passes.
func test_theme_owns_the_palette() -> void:
	var theme := ThemeTokens.base_theme()
	check(theme != null, "themes/bronze_vellum.tres must load as a Theme")
	if theme == null:
		return
	var known := {}
	for token in ThemeTokens.TOKENS:
		check(theme.has_color(token, ThemeTokens.PALETTE), "the palette must carry the token '%s'" % token)
		check(theme.has_color(token, ThemeTokens.HIGH_CONTRAST_PALETTE),
			"the high-contrast palette must answer for the token '%s'" % token)
		known[theme.get_color(token, ThemeTokens.PALETTE)] = token
	for step in ThemeTokens.STEPS:
		check(theme.has_font_size(step, ThemeTokens.TYPE_SCALE), "the type scale must carry the step '%s'" % step)

	for theme_type in theme.get_stylebox_type_list():
		for style_name in theme.get_stylebox_list(theme_type):
			var box := theme.get_stylebox(style_name, theme_type) as StyleBoxFlat
			if box == null:
				continue
			check(known.has(box.bg_color),
				"%s/%s is filled with a colour no palette token names" % [theme_type, style_name])
			check(known.has(box.border_color),
				"%s/%s is ruled in a colour no palette token names" % [theme_type, style_name])
	for theme_type in theme.get_color_type_list():
		if theme_type == ThemeTokens.PALETTE or theme_type == ThemeTokens.HIGH_CONTRAST_PALETTE:
			continue
		for color_name in theme.get_color_list(theme_type):
			check(known.has(theme.get_color(color_name, theme_type)),
				"%s/%s is a colour no palette token names" % [theme_type, color_name])


# ---------------------------------------------------------------------------
# Card B13's Done-when
# ---------------------------------------------------------------------------

func test_hundred_events_produce_at_most_one_urgent() -> void:
	surface.clear_journal()
	surface.set_context({
		"player_faction_id": PLAYER_FACTION,
		"party_character_ids": ["character.protagonist.captain", "character.heroine.betty"],
		"party_cell_id": PARTY_CELL,
		"critical_core_cell_ids": [CORE_CELL],
		"major_relationship_character_ids": ["character.heroine.betty"],
	})
	var events := ordinary_day()
	check(events.size() == EVENT_COUNT, "the fixture must be exactly %d events" % EVENT_COUNT)
	var counts := {
		InformationLevel.Level.AMBIENT: 0,
		InformationLevel.Level.NOTABLE: 0,
		InformationLevel.Level.URGENT: 0,
	}
	for event in events:
		counts[surface.present(event)] += 1
	check(counts[InformationLevel.Level.URGENT] <= 1,
		"a hundred strategic events must interrupt at most once, not %d times" % counts[InformationLevel.Level.URGENT])
	check(counts[InformationLevel.Level.URGENT] == 1,
		"the one event that reaches the party's own cell must be the one interruption")
	check(counts[InformationLevel.Level.AMBIENT] > EVENT_COUNT / 2,
		"most of an ordinary day must simply appear in the world, not %d of %d"
			% [counts[InformationLevel.Level.AMBIENT], EVENT_COUNT])
	check(counts[InformationLevel.Level.NOTABLE] < counts[InformationLevel.Level.AMBIENT],
		"a companion cannot have more to say than the island has to show")
	# Every one of them is in the journal, Ambient included: that is what
	# "Ambient only marks the journal" means.
	check(surface.journal().size() == EVENT_COUNT,
		"every event must mark the journal, at every level")
	for kind in ["hour_passed", "force_moved", "force_departed"]:
		check(surface.classify({"kind": kind}) == InformationLevel.Level.AMBIENT,
			"'%s' is the island breathing and must never reach a notice" % kind)


## A hundred events from an ordinary day, in the shape
## `Project42ExpeditionBridge.strategic_surface` projects them, using every kind
## `StrategicEvent` declares.
##
## The multiplicities are what an actual day looks like rather than an equal
## share each: twenty-four hours turn, forces spend most of their time on the
## roads, a handful of columns set out and arrive, a couple of yards turn
## something out, one faction loses a way back and one is finally taken off the
## board. Cycling the eight kinds evenly would have put a dozen eliminations in
## a single day, which is not an ordinary day and would have made the count
## below a measure of the fixture instead of a measure of the rules.
##
## Deterministic: the index chooses the cell and the faction, so the stream is
## the same on every machine and in every run.
func ordinary_day() -> Array:
	var events: Array = []
	for index in 24:
		events.append(event("hour_passed", "", "", index))
	for index in 40:
		events.append(event("force_moved", other_faction(index), ordinary_cell(index), index))
	for index in 10:
		events.append(event("force_departed", other_faction(index), ordinary_cell(index), index))
	for index in 10:
		events.append(event("force_arrived", other_faction(index), ordinary_cell(index), index))
	for index in 4:
		events.append(event("force_halted", other_faction(index), ordinary_cell(index), index))
	for index in 8:
		# Two of the eight are the player's own yard; the rest are yards the
		# player cannot see into.
		var faction := PLAYER_FACTION if index < 2 else other_faction(index)
		events.append(event("machine_produced", faction, "", index))
	for index in 3:
		events.append(event("recovery_link_lost", other_faction(index), "", index))
	events.append(event("faction_eliminated", other_faction(1), "", 23))
	# One thing happens today that the player must be told about: a hostile
	# force walks into the cell the party is standing in.
	events[64] = event("force_arrived", "faction.pirates", PARTY_CELL, 11)
	return events


func event(kind: String, faction_id: String, cell_id: String, hour: int) -> Dictionary:
	return {
		"day": 12, "hour": hour, "kind": kind,
		"faction_id": faction_id, "cell_id": cell_id, "route_id": "", "force_id": "",
		"character_ids": [], "prose": "%s at %s" % [kind, cell_id],
	}


func ordinary_cell(index: int) -> String:
	return str(ORDINARY_CELLS[index % ORDINARY_CELLS.size()])


func other_faction(index: int) -> String:
	return str(OTHER_FACTIONS[index % OTHER_FACTIONS.size()])


# ---------------------------------------------------------------------------
# The rules themselves, one example each
# ---------------------------------------------------------------------------

func test_the_level_rules_by_example() -> void:
	# Urgent, permission 1: the party, by the cell it stands in.
	check(surface.urgent_reason({"kind": "force_arrived", "cell_id": PARTY_CELL,
		"faction_id": "faction.pirates"}) == "the party",
		"a force arriving where the party stands must interrupt for the party")
	# Urgent, permission 1: the party, by name.
	check(surface.urgent_reason({"kind": "force_moved",
		"character_ids": ["character.protagonist.captain"]}) == "the party",
		"an event naming a party member must interrupt for the party")
	# Urgent, permission 2: a major relationship.
	check(surface.urgent_reason({"kind": "force_moved",
		"character_ids": ["character.heroine.betty"]}) != "",
		"an event naming a major relationship must interrupt")
	# Urgent, permission 3: a critical player-faction location, reached by
	# somebody else.
	check(surface.urgent_reason({"kind": "force_arrived", "cell_id": CORE_CELL,
		"faction_id": "faction.pirates"}) == "a critical location",
		"a hostile force reaching the player's core must interrupt")
	check(surface.urgent_reason({"kind": "force_arrived", "cell_id": CORE_CELL,
		"faction_id": PLAYER_FACTION}) == "",
		"the player's own force coming home to its own core is not an emergency")
	# Urgent, permission 4: a final-stage threat.
	check(surface.urgent_reason({"kind": "faction_eliminated", "faction_id": PLAYER_FACTION})
		== "a final-stage threat",
		"the player's faction leaving the board must interrupt")
	check(surface.classify({"kind": "faction_eliminated", "faction_id": "faction.elves"})
		== InformationLevel.Level.NOTABLE,
		"another faction leaving the board is news, not an interruption")

	# Notable: a map update.
	check(surface.classify({"kind": "force_arrived", "cell_id": "world.cell.black_beach.landing",
		"faction_id": "faction.pirates"}) == InformationLevel.Level.NOTABLE,
		"a force arriving somewhere else is a map update")
	# Notable only for the player's own yard.
	check(surface.classify({"kind": "machine_produced", "faction_id": PLAYER_FACTION})
		== InformationLevel.Level.NOTABLE, "the player's own yard turning something out is news")
	check(surface.classify({"kind": "machine_produced", "faction_id": "faction.pirates"})
		== InformationLevel.Level.AMBIENT,
		"another faction's production is not something the player can see")
	# Ambient: an unknown kind is never an emergency.
	check(surface.classify({"kind": "something.this.build.has.never.heard.of"})
		== InformationLevel.Level.AMBIENT, "an unrecognised development must not interrupt")
	# A surface that has been told nothing interrupts for nothing.
	var untold := preload("res://scripts/surface/information_surface.gd").new()
	check(untold.urgent_reason({"kind": "faction_eliminated", "faction_id": ""}) == "",
		"a surface with no context must never interrupt")
	untold.free()


# ---------------------------------------------------------------------------
# S6's explanation
# ---------------------------------------------------------------------------

func test_the_explanation_comes_before_the_confirmation() -> void:
	var explained := {
		"id": "directive.hold_the_landing",
		"explanation": {
			"goal": "hold what it already has and keep it working",
			"why_target": "the landing is two roads away and is held by nobody",
			"resources": "no limit was stated, and the faction has 34 in stores",
			"blockers": ["the shore path is contested"],
			"withdrawal_conditions": "if the shore path closes behind it",
			"party_could_help": true,
		},
	}
	var lines: PackedStringArray = surface.directive_explanation_lines(explained)
	check(lines.size() == 6, "every explained field must reach the player, not %d of them" % lines.size())
	check(lines[0].begins_with("What it becomes:"), "the explanation must open with what the directive becomes")
	check("In the way: the shore path is contested" in lines,
		"brief section 5.9's obvious risks must be shown before confirmation")
	check(surface.can_confirm_directive(explained), "an explained directive may be confirmed")
	check(not surface.can_confirm_directive({"id": "directive.unexplained"}),
		"a directive nobody has explained cannot be confirmed")
	for line in lines:
		check(not line.contains("score") and not line.contains("weight"),
			"no utility arithmetic may reach the player: '%s'" % line)


# ---------------------------------------------------------------------------
# The journal, readable while paused
# ---------------------------------------------------------------------------

func test_the_journal_holds_the_island_while_it_is_read() -> void:
	var game_pause: Node = root.get_node_or_null("/root/GamePause")
	check(game_pause != null, "GamePause must be registered as an autoload")
	if game_pause == null:
		return
	check(not game_pause.is_paused(), "the journal test must begin with nothing holding the island")
	surface.open_journal()
	check(surface.journal_is_open(), "the journal must know it is open")
	check(game_pause.is_paused(), "reading the journal must hold the island")
	check(game_pause.pause_reasons().has(surface.PAUSE_REASON), "the journal must own its own reason")
	surface.open_journal()
	check(game_pause.pause_reasons().size() == 1, "opening a journal twice must not stack a second reason")
	surface.close_journal()
	check(not game_pause.is_paused(), "closing the journal must let the island run")
	check(surface.recent_journal(3).size() == 3, "the journal must be readable a few lines at a time")


# ---------------------------------------------------------------------------
# B9's settings, through the Theme
# ---------------------------------------------------------------------------

func test_settings_change_the_theme_live() -> void:
	var scene := load("res://scenes/ui/settings_panel.tscn") as PackedScene
	check(scene != null, "the settings panel scene must load")
	if scene == null:
		return
	var panel := scene.instantiate() as SettingsPanel
	stage.add_child(panel)
	await process_frame

	var before: Theme = panel.theme
	check(before != null, "the settings panel must wear the grammar")
	var body_before := ThemeTokens.font_size(before, "body")
	var cream_before := ThemeTokens.color(before, "cream")

	panel.set_text_scale(1.5)
	await process_frame
	var scaled: Theme = panel.theme
	check(ThemeTokens.font_size(scaled, "body") > body_before,
		"a larger text scale must grow the type scale the Theme carries")
	check(scaled.default_font_size > before.default_font_size,
		"a larger text scale must grow the default font every control reads")
	check(surface.current_theme() == scaled, "the surface must be wearing the Theme the panel just built")

	panel.set_high_contrast(true)
	await process_frame
	var contrasted: Theme = panel.theme
	check(ThemeTokens.color(contrasted, "cream") != cream_before,
		"high contrast must swap the token set, not merely brighten one label")
	check(ThemeTokens.color(contrasted, "deep") == Color.BLACK,
		"the high-contrast ground must be black")
	var card_box := ThemeTokens.style(contrasted, "VellumCard", "panel") as StyleBoxFlat
	check(card_box.bg_color == ThemeTokens.color(contrasted, "panel"),
		"a StyleBox must follow the palette into high contrast")

	check(not ThemeTokens.reduced_motion(contrasted), "reduced motion must be off until it is chosen")
	panel.set_reduced_motion(true)
	await process_frame
	check(ThemeTokens.reduced_motion(panel.theme), "reduced motion must reach the components through the Theme")

	# The base resource is never mutated by any of that: it is the owner, and
	# `build` works on a duplicate.
	check(ThemeTokens.font_size(ThemeTokens.base_theme(), "body") == body_before,
		"building a Theme must never edit the resource it was built from")

	panel.set_text_scale(SettingsPanel.DEFAULT_TEXT_SCALE)
	panel.set_high_contrast(false)
	panel.set_reduced_motion(false)
	panel.close()
	await process_frame


# ---------------------------------------------------------------------------
# The components
# ---------------------------------------------------------------------------

func test_every_component_instantiates_under_the_theme() -> void:
	var host := Control.new()
	host.name = "Components"
	host.theme = ThemeTokens.build()
	host.size = Vector2(1200, 640)
	stage.add_child(host)
	await process_frame

	var panel := VellumPanel.new()
	panel.heading = "Party"
	host.add_child(panel)
	check(panel.body() != null, "a panel must offer somewhere to put its content")

	var rail := VellumCardRail.new()
	panel.body().add_child(rail)
	rail.set_entries([
		{"actor_id": "character.heroine.betty", "display_name": "Betty", "band_name": "party_front",
			"vitality": 18, "vitality_maximum": 24, "guard": 2, "guard_maximum": 8,
			"composure": 7, "composure_maximum": 10, "shaken": false},
		{"actor_id": "character.heroine.vix", "display_name": "Vix", "band_name": "party_rear",
			"vitality": 6, "vitality_maximum": 24, "guard": 0, "guard_maximum": 8,
			"composure": 2, "composure_maximum": 10, "shaken": true},
	])
	await process_frame
	check(rail.cards().size() == 2, "the rail must draw one card per actor")
	check(rail.selected_actor_id == "character.heroine.betty", "the rail must select its first card")
	rail.select("character.heroine.vix")
	check(rail.cards()[1].selected, "selecting an actor must select their card")
	check(rail.cards()[1].shaken, "a Shaken actor's card must carry the mark the simulation gave them")

	var grid := VellumCommandGrid.new()
	grid.columns = 3
	panel.body().add_child(grid)
	grid.set_commands([
		{"command_id": "skill.betty.guarded_strike", "rank": "D", "display_name": "Guarded Strike",
			"state": VellumCommandDiamond.State.UNLOCKED},
		{"command_id": "skill.betty.mobile_infirmary", "rank": "A", "display_name": "Mobile Infirmary",
			"state": VellumCommandDiamond.State.REFUSED, "reason": "No wounded ally is in reach."},
		{"command_id": "skill.betty.combat_revival", "rank": "SSS", "display_name": "Combat Revival",
			"state": VellumCommandDiamond.State.LOCKED, "reason": "Locked until its authored bond milestone."},
	])
	await process_frame
	check(grid.slots().size() == 3, "the grid must draw every command, locked ones included")
	var locked := grid.slots()[2]
	check(locked.tooltip_lines().contains("SSS") and locked.tooltip_lines().contains("Combat Revival"),
		"a locked slot must still identify its rank and its skill")
	check(locked.tooltip_lines().contains("skill.betty.combat_revival"),
		"a slot must name its stable id")

	var tooltip := VellumTooltip.new()
	host.add_child(tooltip)
	tooltip.configure("A rank · Mobile Infirmary", "No wounded ally is in reach.", "skill.betty.mobile_infirmary")

	var entry := VellumJournalEntry.new()
	panel.body().add_child(entry)
	entry.configure({"day": 12, "hour": 9, "level": InformationLevel.Level.NOTABLE,
		"prose": "A force of faction.pirates reached world.cell.black_beach.landing."})

	# Ambient builds nothing at all. That is the whole of "most changes simply
	# appear in the world".
	check(VellumNotice.present(InformationLevel.Level.AMBIENT, "nothing to see") == null,
		"an Ambient event must never build a notice")

	var notable := VellumNotice.present(InformationLevel.Level.NOTABLE,
		"A force of faction.pirates reached the landing.", "", "Betty")
	notable.preview = true
	host.add_child(notable)
	await process_frame
	check(notable.get_node("Notice").mouse_filter == Control.MOUSE_FILTER_IGNORE,
		"a Notable line must not take a click aimed at the world underneath it")
	check(notable.get_node("Notice/Column/Acknowledge").focus_mode == Control.FOCUS_NONE,
		"a Notable line must never take the keyboard")

	var game_pause: Node = root.get_node_or_null("/root/GamePause")
	var urgent := VellumNotice.present(InformationLevel.Level.URGENT,
		"They are at the terrace.", "The island is held until you answer.", "Urgent · the party")
	host.add_child(urgent)
	await process_frame
	check(game_pause.is_paused(), "an Urgent notice must hold the island")
	check(game_pause.pause_reasons().has(VellumNotice.PAUSE_REASON), "an Urgent notice must own its own reason")
	urgent.acknowledge()
	await process_frame
	check(not game_pause.is_paused(), "acknowledging an Urgent notice must let the island run again")

	host.queue_free()
	await process_frame


# ---------------------------------------------------------------------------
# The bridge hunk
# ---------------------------------------------------------------------------

## `Project42ExpeditionBridge.strategic_surface` is the only way the standing
## directives and S11's journal reach a screen. Guarded exactly as the other
## native suites are: without the GDExtension there is nothing to prove here,
## and CI fails the whole gate if that skip line is ever printed.
func test_the_bridge_carries_directives_and_the_journal() -> void:
	if not NativeExpeditionPort.bridge_is_registered():
		print("Information surface bridge test skipped: bridge is not registered in this running Godot process.")
		return
	var session: Node = root.get_node_or_null("/root/CampaignSession")
	check(session != null, "CampaignSession must be registered as an autoload")
	if session == null:
		return
	var catalog := ContentCatalog.new()
	check(catalog.load_default() == OK, "the validated content bundle must load")
	session.reset_for_test()
	var started: Dictionary = session.begin_if_needed(catalog)
	check(bool(started.get("configured", false)), "the campaign must start through the native bridge")
	if not bool(started.get("configured", false)):
		return
	var port: Object = session.expedition.bridge

	var fresh: Dictionary = port.strategic_surface(64)
	check(bool(fresh.get("configured", false)), "a configured campaign must answer the surface")
	check((fresh.get("journal", []) as Array).is_empty(),
		"a campaign that has run no hour has no journal to read")
	check((fresh.get("directives", []) as Array).is_empty(),
		"a fresh campaign stands under no directive")

	var advanced: Dictionary = session.resolve_midnight()
	check(bool(advanced.get("configured", false)), "midnight must resolve through the bridge")
	var after: Dictionary = port.strategic_surface(64)
	var journal: Array = after.get("journal", [])
	check(not journal.is_empty(), "a day that ran must leave lines in the journal")
	check(journal.size() <= 64, "the journal must honour the limit it was asked for")
	var line: Dictionary = journal[journal.size() - 1]
	for key in ["day", "hour", "kind", "faction_id", "cell_id", "route_id", "force_id",
			"character_ids", "prose"]:
		check(line.has(key), "a journal line must carry '%s'" % key)
	check(not str(line["prose"]).is_empty(), "the bridge writes the prose, not the engine")
	check(str(line["kind"]) in EVENT_KINDS, "'%s' is not a kind this surface knows" % line["kind"])
	# The classification runs on what the bridge actually said, not on a
	# hand-written twin of it.
	check(surface.classify(line) == InformationLevel.Level.AMBIENT,
		"the hours of a quiet day must classify as Ambient")
	check(int(str(after.get("journal_total_recorded", "0"))) >= journal.size(),
		"the surface must report every entry ever recorded, not only the window")
	check(port.strategic_surface(0).get("journal", []).is_empty(),
		"a limit of zero must return no lines")
	check(not (port.strategic_surface(100000).get("journal", []) as Array).is_empty(),
		"a limit past the window must clamp rather than fail")


# ---------------------------------------------------------------------------

func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)


func finish() -> void:
	if failures > 0:
		quit(1)
		return
	print("Information surface tests passed.")
	quit(0)
