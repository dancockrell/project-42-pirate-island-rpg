extends SceneTree

## P11: the Theme is the one owner of the palette, and the screens adopt it.
##
## Two proofs, and they are the two halves of the same claim.
##
## The first walks every script under `res://scripts/` -- except `world/` and
## `board/`, which lane P10 owns this round -- and refuses a colour literal.
## Rule 17 says the palette has one owner after P5; a hex written beside a
## `draw_rect` is a second owner, and it is a second owner that no accessibility
## setting can ever reach. A literal survives only where it is *marked*: a game
## colour (a faction's tint on the rail, a character rig's hair, an authored
## effect palette word) or a value that is not a colour at all (a modulate
## identity, an absent-entry fallback, an accumulator). The marker carries a
## reason, because a marker without one is an opt-out.
##
## The second is what the first is for. The battle and the title are asked to
## re-theme while they stand, by calling `InformationSurface.apply_settings` the
## way the settings panel does, and their colours and their type must both move
## -- which is only possible because neither of them declares either any more.

const ThemeTokensScript := preload("res://scripts/ui/theme_tokens.gd")

## The directories the scan covers, and the two it does not. `world/` and
## `board/` are card P10's this round: it hosts the isometric board in the
## expedition screen and retires the 2D one, and `BoardPalette` adopts the Theme
## there. When P10 lands, the two exclusions below are deleted and this suite
## covers the whole of `scripts/`.
const SCRIPT_ROOT := "res://scripts"
const NOT_OURS_YET: PackedStringArray = ["res://scripts/world"]

## The two markers, each of which must be followed by a reason.
const GAME_COLOUR_MARKER := "# game colour:"
const NOT_A_COLOUR_MARKER := "# not a colour:"

## The two palette tokens and the two type-scale steps card P11 added to the
## Theme when the shell adopted it. They are asserted here so a later edit to
## the resource cannot quietly drop what the shell reads.
const ADDED_TOKENS: PackedStringArray = ["night", "hairline"]
const ADDED_STEPS: PackedStringArray = ["wordmark", "subtitle"]

## The text scale the accessibility capture is taken at, and the one this suite
## asserts against, so the number in the picture and the number in the proof are
## the same number.
const CAPTURE_TEXT_SCALE := 1.3

var failures := 0
var surface: Node


func _init() -> void:
	await process_frame
	surface = root.get_node_or_null("/root/InformationSurface")
	check(surface != null, "InformationSurface must be registered as an autoload")
	if surface == null:
		finish()
		return
	test_the_theme_carries_what_the_screens_read()
	test_no_unmarked_colour_literal_outside_the_lanes_p10_owns()
	await test_the_title_re_themes_where_it_stands()
	await test_the_battle_re_themes_where_it_stands()
	surface.apply_settings(1.0, false, false)
	finish()


# ---------------------------------------------------------------------------
# One owner.
# ---------------------------------------------------------------------------


func test_the_theme_carries_what_the_screens_read() -> void:
	var grammar := ThemeTokensScript.base_theme()
	check(grammar != null, "themes/bronze_vellum.tres must load as a Theme")
	if grammar == null:
		return
	for token in ThemeTokensScript.TOKENS:
		check(grammar.has_color(token, ThemeTokensScript.PALETTE),
			"the Theme must carry the palette token '%s' that ThemeTokens names" % token)
		check(grammar.has_color(token, ThemeTokensScript.HIGH_CONTRAST_PALETTE),
			"every palette token needs a high-contrast twin; '%s' has none" % token)
	for step in ThemeTokensScript.STEPS:
		check(grammar.has_font_size(step, ThemeTokensScript.TYPE_SCALE),
			"the Theme must carry the type-scale step '%s' that ThemeTokens names" % step)
	for token in ADDED_TOKENS:
		check(token in ThemeTokensScript.TOKENS, "'%s' must stay a named palette token" % token)
	for step in ADDED_STEPS:
		check(step in ThemeTokensScript.STEPS, "'%s' must stay a named type-scale step" % step)


## Every `.gd` the presentation draws with, refused a colour it declares itself.
func test_no_unmarked_colour_literal_outside_the_lanes_p10_owns() -> void:
	var paths := PackedStringArray()
	collect_scripts(SCRIPT_ROOT, paths)
	check(paths.size() >= 30, "the scan must actually find the scripts; it found %d" % paths.size())
	var scanned := 0
	var offences := PackedStringArray()
	for path in paths:
		scanned += 1
		offences.append_array(unmarked_literals(path))
	check(offences.is_empty(),
		"a colour must come from the Theme, or be marked as a game colour with a reason. Unmarked: %s" % ", ".join(offences))
	print("Palette scan: %d scripts read, %d unmarked colour literals." % [scanned, offences.size()])


func collect_scripts(directory: String, into: PackedStringArray) -> void:
	for skipped in NOT_OURS_YET:
		if directory == skipped:
			return
	for child in DirAccess.get_directories_at(directory):
		collect_scripts("%s/%s" % [directory, child], into)
	for file in DirAccess.get_files_at(directory):
		if file.ends_with(".gd"):
			into.append("%s/%s" % [directory, file])


## Every colour literal in `path` that no marker above it accounts for.
##
## A marker covers the line it is on and every line below it until a blank one,
## so a rig's whole palette is declared and marked once rather than line by
## line, and a marker cannot silently reach across a gap into unrelated code.
func unmarked_literals(path: String) -> PackedStringArray:
	var found := PackedStringArray()
	var handle := FileAccess.open(path, FileAccess.READ)
	if handle == null:
		return found
	var lines := handle.get_as_text().split("\n")
	handle.close()
	var hex := RegEx.create_from_string('Color\\(\\s*["0-9\\-]')
	var named := RegEx.create_from_string('Color\\.[A-Z][A-Z0-9_]*')
	for index in lines.size():
		var line := lines[index]
		if hex.search(line) == null and named.search(line) == null:
			continue
		if is_marked(lines, index):
			continue
		found.append("%s:%d" % [path.get_file(), index + 1])
	return found


func is_marked(lines: PackedStringArray, index: int) -> bool:
	var walker := index
	while walker >= 0:
		var line := lines[walker]
		if walker != index and line.strip_edges().is_empty():
			return false
		if carries_marker(line):
			return true
		walker -= 1
	return false


## A marker with nothing after the colon is not a marker: the reason is the
## whole point of allowing one.
func carries_marker(line: String) -> bool:
	for marker in [GAME_COLOUR_MARKER, NOT_A_COLOUR_MARKER]:
		var at := line.find(marker)
		if at >= 0 and not line.substr(at + marker.length()).strip_edges().is_empty():
			return true
	return false


# ---------------------------------------------------------------------------
# Adopted: the screens wear it, and keep wearing it.
# ---------------------------------------------------------------------------


func test_the_title_re_themes_where_it_stands() -> void:
	surface.apply_settings(1.0, false, false)
	var title := TitleScreen.new()
	root.add_child(title)
	await process_frame
	check(title.theme != null, "the title must wear a Theme rather than its own constants")
	var before := title.theme
	var body_before := ThemeTokensScript.font_size(title.theme, "body")
	var cream_before := ThemeTokensScript.color(title.theme, "cream")
	var ridge_before: Color = title.atmosphere.get("_far_ridge_colour")

	surface.apply_settings(CAPTURE_TEXT_SCALE, true, false)
	await process_frame

	check(title.theme != before, "the title must take the rebuilt Theme, not keep the one it opened with")
	check(ThemeTokensScript.font_size(title.theme, "body") > body_before,
		"1.3x text scale must reach the title's type: it was %d and stayed %d" % [body_before, ThemeTokensScript.font_size(title.theme, "body")])
	check(ThemeTokensScript.color(title.theme, "cream") != cream_before,
		"high contrast must reach the title's palette")
	check(title.atmosphere.get("_far_ridge_colour") != ridge_before,
		"high contrast must reach the weather behind the title, not only the lettering")
	# The menu rows carry no colour and no size of their own, which is the whole
	# reason the two assertions above can be true at all.
	var new_game := title.menu_buttons[TitleScreen.NEW_GAME] as Button
	check(new_game.theme_type_variation == &"MenuRow",
		"a title menu row must be the Theme's MenuRow variation")
	check(not new_game.has_theme_stylebox_override("normal"),
		"a title menu row must not override the box the Theme gives it")
	check(not new_game.has_theme_color_override("font_color"),
		"a title menu row must not override the colour the Theme gives it")
	title.queue_free()
	await process_frame


func test_the_battle_re_themes_where_it_stands() -> void:
	if not ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS):
		push_error("palette_owner_test requires the native bridge: %s is not registered." % NativeSimulationPort.BRIDGE_CLASS)
		failures += 1
		return
	surface.apply_settings(1.0, false, false)
	var battle := BattlePrototype.new()
	root.add_child(battle)
	await process_frame
	check(battle.theme != null, "the battle must wear a Theme rather than its own constants")
	var before := battle.theme
	var caption_before := ThemeTokensScript.font_size(battle.theme, "caption")
	var ground := battle.get_node_or_null("BackgroundBase") as ColorRect
	check(ground != null, "the battle must own the ground it paints")
	var ground_before := ground.color if ground != null else Color.BLACK
	var dock_before := (battle.command_dock as PanelContainer).get_theme_stylebox("panel") as StyleBoxFlat

	surface.apply_settings(CAPTURE_TEXT_SCALE, true, false)
	await process_frame

	check(battle.theme != before, "the battle must take the rebuilt Theme, not keep the one it opened with")
	check(ThemeTokensScript.font_size(battle.theme, "caption") > caption_before,
		"1.3x text scale must reach the battle's type")
	check(ground != null and ground.color != ground_before,
		"high contrast must reach the ground the battle stands on")
	var dock_after := (battle.command_dock as PanelContainer).get_theme_stylebox("panel") as StyleBoxFlat
	check(dock_before != null and dock_after != null and dock_after.bg_color != dock_before.bg_color,
		"high contrast must reach the command dock, which builds its own box")
	check(battle.stage.factory.theme == battle.theme,
		"an effect played after the settings change must be played in the palette the screen just moved to")
	# The band rail's headings are Theme variations, so the rail moved with it.
	check(battle.intent_label.theme_type_variation != &"",
		"the battle's HUD lines must name a Theme variation rather than a size and a colour")
	battle.queue_free()
	await process_frame


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
	print("Palette owner tests passed.")
	quit(0)
