extends SceneTree

## P14: the battle at every text scale.
##
## P11 left the battle wearing the Theme's palette and the Theme's type, and
## found two things it did not own: a roster card's status line was wider than
## the copy column the card gave it, at every text scale, and at 1.3x the
## command names lost their tails inside their plates -- "GUARDED STRI".
##
## Both were the same fault. The widths were typed once, at 1.0x, and the type
## scale moved without them. So this suite instances the real battle at 1.0x and
## again at 1.3x with high contrast, exactly as the settings panel would, and
## asks of the screen in front of it:
##
##   - does every line of copy fit the box it is drawn in, or is something
##     being clipped;
##   - does every command name fit its plate whole;
##   - do the boxes themselves grow when the type does;
##   - and does the bottom of the screen still fit on the screen -- the rail,
##     the gutter, the dock, all inside the canvas, none of them overlapping.
##
## The live bridge is required rather than optional, for the same reason
## `palette_owner_test` requires it: this is the real screen with the real
## simulation behind it, not a fixture that would agree with whatever it was
## handed.

const ThemeTokensScript := preload("res://scripts/ui/theme_tokens.gd")
const BattleMetricsScript := preload("res://scripts/battle/battle_metrics.gd")

## The two settings the captures are taken at, so the numbers in the picture and
## the numbers in the proof are the same numbers.
const DEFAULT_TEXT_SCALE := 1.0
const CAPTURE_TEXT_SCALE := 1.3

## A pixel of slack, because a width is measured in floats and a box is laid out
## in floats. Anything wider than this is a visible overflow.
const SLACK := 1.0

var failures := 0
var surface: Node
## What each scale measured, so the last test can ask whether the boxes grew.
var measured: Dictionary = {}


func _init() -> void:
	await process_frame
	surface = root.get_node_or_null("/root/InformationSurface")
	check(surface != null, "InformationSurface must be registered as an autoload")
	if surface == null:
		finish()
		return
	if not ClassDB.can_instantiate(NativeSimulationPort.BRIDGE_CLASS):
		push_error("battle_layout_test requires the native bridge: %s is not registered." % NativeSimulationPort.BRIDGE_CLASS)
		failures += 1
		finish()
		return
	await test_the_battle_fits_at(DEFAULT_TEXT_SCALE, false)
	await test_the_battle_fits_at(CAPTURE_TEXT_SCALE, true)
	test_every_box_grew_with_the_type()
	surface.apply_settings(DEFAULT_TEXT_SCALE, false, false)
	await process_frame
	finish()


func test_the_battle_fits_at(text_scale: float, high_contrast: bool) -> void:
	surface.apply_settings(text_scale, high_contrast, false)
	var battle := BattlePrototype.new()
	root.add_child(battle)
	# Two frames: one for the screen to build and project its first snapshot,
	# one for the containers inside the dock to flow at the size they were
	# given. Nothing here waits on a clock.
	await process_frame
	await process_frame
	var at := "at %.1fx text" % text_scale
	# What the Theme measured, and what the canvas then allowed. The first is
	# what must grow with the type scale; the second is the first fitted to a
	# screen of a fixed width, and on a narrow screen a card is deliberately
	# squeezed towards its floor rather than pushed off the edge.
	measured[text_scale] = {
		"the copy column the type asks for": BattleMetricsScript.card_copy_width(battle.theme, battle.roster_names()),
		"the narrowest copy column the type allows": BattleMetricsScript.card_copy_floor(battle.theme, battle.roster_names()),
		"the command plate": Vector2(battle.layout.plate).x,
		"the card": float(battle.layout.card_height),
	}
	check(not battle.card_buttons.is_empty(), "the rail must hold a card for every actor %s" % at)
	# The names the rail was measured against must be the names it is setting --
	# the simulation's hostiles included, since "RAZORBEAK" is longer than any
	# word in a heroine's name and would otherwise break inside itself.
	var roster: Array = battle.roster_names()
	for actor in battle.snapshot_actors:
		var shown := str(actor.get("display_name", "")).to_upper()
		check(roster.has(shown), "the copy column must be measured against '%s', which the rail is setting %s" % [shown, at])
	check(not battle.command_buttons.is_empty(), "the dock must hold Betty's commands %s" % at)

	assert_the_copy_column_is_wide_enough(battle, at)
	assert_no_card_copy_overflows(battle, at)
	assert_no_command_name_is_trimmed(battle, at)
	assert_the_bottom_of_the_screen_fits(battle, at)

	battle.queue_free()
	await process_frame


## The floor. A column narrower than the widest unbreakable word in the lines it
## holds has nowhere to put them: wrapping cannot help, and what is left is the
## clipping this card exists to remove. This is the assertion a restored fixed
## column width fails first.
func assert_the_copy_column_is_wide_enough(battle: BattlePrototype, at: String) -> void:
	var floor_width := BattleMetricsScript.card_copy_floor(battle.theme, battle.roster_names())
	check(float(battle.layout.copy_width) + SLACK >= floor_width,
		"a roster card's copy column (%.0f) must be at least as wide as the widest word it holds (%.0f) %s"
			% [float(battle.layout.copy_width), floor_width, at])
	var plate_room := Vector2(battle.layout.plate).x - BattleMetricsScript.PLATE_PADDING
	for skill_id in battle.command_buttons:
		var diamond := battle.command_buttons[skill_id] as PaperSkillDiamond
		var needed := BattleMetricsScript.text_width(battle.theme, "caption", diamond.label_text())
		check(needed <= plate_room + SLACK,
			"the command plate (%.0f) must be measured from the longest command name; '%s' needs %.0f %s"
				% [plate_room, diamond.label_text(), needed, at])


## Nothing inside a card is lost. Both ways a line can be lost are asked: past
## the side of the column, and off the bottom of it. The second is asked of the
## Label itself -- `get_line_count` is what the text came to, and
## `get_visible_line_count` is what the box shows -- so the assertion is about
## what is on the screen rather than about what was intended.
func assert_no_card_copy_overflows(battle: BattlePrototype, at: String) -> void:
	for id in battle.card_buttons:
		var holder := battle.card_buttons[id] as Control
		var copy := holder.get_node_or_null("Copy") as Control
		check(copy != null, "card %s must own a copy column %s" % [id, at])
		if copy == null:
			continue
		check(copy.position.x + copy.size.x <= holder.size.x + SLACK,
			"card %s: the copy column must end inside the card %s" % [id, at])
		var well_top := holder.size.y - BattleMetricsScript.well_height(battle.theme) - BattleMetricsScript.WELL_GAP
		check(copy.position.y + copy.size.y <= well_top + SLACK,
			"card %s: the copy column must stop above the Composure well %s" % [id, at])
		for node in copy.get_children():
			var label := node as Label
			if label == null or not label.visible:
				continue
			check(label.position.x + label.size.x <= copy.size.x + SLACK
					and label.position.y + label.size.y <= copy.size.y + SLACK,
				"card %s: '%s' must be laid out inside the copy column %s" % [id, label.name, at])
			check(label.get_visible_line_count() >= label.get_line_count(),
				"card %s: '%s' has %d lines and shows %d -- the card must grow, not clip %s"
					% [id, label.name, label.get_line_count(), label.get_visible_line_count(), at])
			check(label.get_line_count() <= 1 or label.autowrap_mode != TextServer.AUTOWRAP_OFF,
				"card %s: '%s' must wrap by a stated rule rather than run past its column %s" % [id, label.name, at])
		var name_label := battle.name_labels.get(id) as Label
		check(name_label != null and name_label.get_line_count() <= BattleMetricsScript.NAME_MAXIMUM_LINES,
			"card %s: a name may take at most %d lines %s" % [id, BattleMetricsScript.NAME_MAXIMUM_LINES, at])
		check(name_label != null and BattleMetricsScript.widest_word(battle.theme, "body", name_label.text) <= name_label.size.x + SLACK,
			"card %s: '%s' must not be broken inside a word %s" % [id, "" if name_label == null else name_label.text, at])


## No command loses its tail. This is "GUARDED STRI" as an assertion.
func assert_no_command_name_is_trimmed(battle: BattlePrototype, at: String) -> void:
	for skill_id in battle.command_buttons:
		var diamond := battle.command_buttons[skill_id] as PaperSkillDiamond
		check(diamond.label_fits(),
			"the command name '%s' must fit its plate whole %s" % [diamond.label_text(), at])
		check(diamond.size.x + SLACK >= diamond.custom_minimum_size.x
				and diamond.size.y + SLACK >= diamond.custom_minimum_size.y,
			"the command slot for '%s' must be given the size it asked for %s" % [diamond.label_text(), at])
	for skill_id in battle.captain_command_buttons:
		var button := battle.captain_command_buttons[skill_id] as Button
		var needed := BattleMetricsScript.text_width(battle.theme, "body", button.text)
		check(needed <= button.custom_minimum_size.x + SLACK,
			"Michael's command '%s' needs %.0f and its button asks for %.0f %s"
				% [button.text, needed, button.custom_minimum_size.x, at])


## The rail, the gutter and the dock, all three inside the canvas and none of
## them over another. A dock pushed off the right edge and a rail running under
## it are the two ways this screen has failed before.
func assert_the_bottom_of_the_screen_fits(battle: BattlePrototype, at: String) -> void:
	var canvas := battle.design_size()
	var rail := Rect2(battle.band_rail.position, battle.band_rail.size)
	var dock := Rect2(battle.command_dock.position, battle.command_dock.size)
	for named in [["the roster rail", rail], ["the command dock", dock]]:
		var rect: Rect2 = named[1]
		check(rect.position.x >= -SLACK and rect.position.y >= -SLACK
				and rect.end.x <= canvas.x + SLACK and rect.end.y <= canvas.y + SLACK,
			"%s (%s) must stand inside the %.0fx%.0f canvas %s" % [named[0], rect, canvas.x, canvas.y, at])
	check(not rail.intersects(dock),
		"the roster rail (%s) and the command dock (%s) must not stand on each other %s" % [rail, dock, at])
	check(dock.size.x + SLACK >= battle.command_dock.get_combined_minimum_size().x
			and dock.size.y + SLACK >= battle.command_dock.get_combined_minimum_size().y,
		"the command dock must be at least as large as the commands inside it %s" % at)
	for id in battle.card_buttons:
		var holder := battle.card_buttons[id] as Control
		var card := Rect2(rail.position + holder.position, holder.size)
		check(rail.encloses(card.grow(-SLACK)),
			"card %s (%s) must stand inside the rail (%s) %s" % [id, card, rail, at])


## The point of all of it: a larger type scale makes the boxes larger. A width
## that was typed once does not move, so this is the assertion that a restored
## constant cannot pass however the rest of the screen is arranged.
func test_every_box_grew_with_the_type() -> void:
	if not measured.has(DEFAULT_TEXT_SCALE) or not measured.has(CAPTURE_TEXT_SCALE):
		check(false, "both text scales must have been measured before they can be compared")
		return
	var small: Dictionary = measured[DEFAULT_TEXT_SCALE]
	var large: Dictionary = measured[CAPTURE_TEXT_SCALE]
	for field in small:
		check(float(large[field]) > float(small[field]),
			"%s must grow with the text scale: %.0f at %.1fx and %.0f at %.1fx"
				% [field, float(small[field]), DEFAULT_TEXT_SCALE, float(large[field]), CAPTURE_TEXT_SCALE])


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
	print("Battle layout tests passed.")
	quit(0)
