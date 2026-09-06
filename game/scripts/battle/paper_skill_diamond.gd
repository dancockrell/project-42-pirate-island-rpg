class_name PaperSkillDiamond
extends Control

## One real command slot in the compact D-to-SSS skill grid. The visual is a
## cut-paper bronze diamond; the transparent Button is the input owner. A
## locked diamond always identifies its authored rank and skill in a tooltip.

## The one door onto the palette and the type scale (card P11). A command slot
## is interface: it declares no colour of its own.
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")
## The one owner of the screen's sizes (card P14). The plate under a command's
## name is as wide as the widest command name at the Theme's caption step, so
## "GUARDED STRIKE" is a whole word at 1.0x and at 1.3x alike.
const BattleMetricsScript = preload("res://scripts/battle/battle_metrics.gd")

signal command_pressed(skill_id: String)

var skill_id := ""
var rank := "D"
var display_name := ""
var story_locked := true
var command_button: Button
## The size the dock gives every one of its commands, so a row of them is a row
## and not a staircase. Zero until the dock measures; the diamond then measures
## its own name, which is what a diamond built outside a dock -- a suite, a
## review scene -- needs.
var plate_size := Vector2.ZERO


func configure(next_skill_id: String, next_rank: String, next_display_name: String, available: bool) -> void:
	skill_id = next_skill_id
	rank = next_rank
	display_name = next_display_name
	story_locked = not available
	refresh_metrics()
	tooltip_text = "%s rank — %s\nStable skill ID: %s%s" % [rank, display_name, skill_id, "\nLocked until its authored bond milestone." if story_locked else ""]
	if command_button != null:
		set_command_enabled(available)
	queue_redraw()


## Take the size the dock measured for every command, and wear it.
func set_plate_size(next_plate_size: Vector2) -> void:
	plate_size = next_plate_size
	refresh_metrics()


## The slot is as large as the Theme's type needs it to be. Nothing here is a
## stated pixel count: the width comes from the name at the caption step and the
## height from that step's line box under the drawn diamond.
func refresh_metrics() -> void:
	var measured: Vector2 = plate_size
	if measured == Vector2.ZERO:
		measured = BattleMetricsScript.diamond_size(ThemeTokensScript.active(self), [display_name])
	custom_minimum_size = measured
	queue_redraw()


func _ready() -> void:
	command_button = Button.new()
	command_button.flat = true
	command_button.tooltip_text = tooltip_text
	command_button.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	command_button.pressed.connect(func() -> void: command_pressed.emit(skill_id))
	add_child(command_button)
	set_command_enabled(not story_locked)
	refresh_metrics()


func set_command_enabled(enabled: bool) -> void:
	story_locked = not enabled
	if command_button != null:
		command_button.disabled = not enabled
		command_button.tooltip_text = "%s rank — %s\nStable skill ID: %s%s" % [rank, display_name, skill_id, "\nLocked until its authored bond milestone." if not enabled else ""]
	queue_redraw()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		refresh_metrics()


## The rectangle the command's name is drawn inside, and the width it may use.
## The suite asks for both: a name wider than this rectangle is a name the
## player cannot read, which is the fault card P14 exists to remove.
func plate_rect() -> Rect2:
	var height := BattleMetricsScript.plate_height(ThemeTokensScript.active(self))
	return Rect2(0.0, size.y - height, size.x, height)


func label_text() -> String:
	return display_name.to_upper()


## True when the whole name fits the plate it is drawn on. False is the failure
## the card names: "GUARDED STRI" is not a command.
func label_fits() -> bool:
	var room := plate_rect().size.x - BattleMetricsScript.PLATE_PADDING
	return BattleMetricsScript.text_width(ThemeTokensScript.active(self), "caption", label_text()) <= room


func _draw() -> void:
	var theme := ThemeTokensScript.active(self)
	var plate := plate_rect()
	var art_height := size.y - plate.size.y
	var center := Vector2(size.x * .5, art_height * .5)
	var r: float = minf(size.x, art_height) * .40
	var diamond := PackedVector2Array([center + Vector2(0,-r),center + Vector2(r,0),center + Vector2(0,r),center + Vector2(-r,0)])
	# An available command is lit in teal on a raised panel; a locked one keeps
	# its shape in the sunken panel and the faint rule, so it is visibly not
	# available rather than illegible.
	var fill := ThemeTokensScript.color(theme, "panel_active" if not story_locked else "panel_sunken")
	var border := ThemeTokensScript.color(theme, "teal" if not story_locked else "rule_faint")
	draw_colored_polygon(diamond, fill)
	draw_polyline(PackedVector2Array([diamond[0],diamond[1],diamond[2],diamond[3],diamond[0]]),border,3,true)
	if not story_locked:
		draw_circle(center, r*.25, Color(ThemeTokensScript.color(theme, "teal"), .35))
		draw_arc(center,r*.27,0,TAU,16,ThemeTokensScript.color(theme, "bronze_bright"),2,true)
	var font := BattleMetricsScript.face(theme)
	# The rank glyph is centred across the whole slot rather than nudged by a
	# stated offset, so it stays under the diamond's point at every text scale.
	var rank_step := "caption" if rank.length() > 1 else "body"
	draw_string(font, Vector2(0.0, center.y + BattleMetricsScript.line_height(theme, rank_step) * .3),
		rank, HORIZONTAL_ALIGNMENT_CENTER, size.x, ThemeTokensScript.font_size(theme, rank_step),
		ThemeTokensScript.color(theme, "focus" if not story_locked else "muted"))
	# The command's name is a label the player reads at a glance in a lit room,
	# not a caption. It sits on its own dark plate and it is not grey-on-grey:
	# a locked command is still legible, it is just visibly not available.
	#
	# Card P14: the plate is as wide as the widest command name at this text
	# scale, so the name fits at 1.0x and at 1.3x alike. If a longer name ever
	# arrives, the line is trimmed with an ellipsis by the one rule stated here
	# -- the player is told the name is cut -- rather than losing its tail to
	# the edge of the plate with nothing to say that anything is missing.
	var label_color := ThemeTokensScript.color(theme, "cream" if not story_locked else "muted")
	draw_rect(plate, Color(ThemeTokensScript.color(theme, "night"), .62), true)
	var line := TextLine.new()
	line.width = plate.size.x - BattleMetricsScript.PLATE_PADDING
	line.alignment = HORIZONTAL_ALIGNMENT_CENTER
	line.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	line.add_string(label_text(), font, ThemeTokensScript.font_size(theme, "caption"))
	line.draw(get_canvas_item(),
		plate.position + Vector2(BattleMetricsScript.PLATE_PADDING * .5, BattleMetricsScript.LINE_PADDING),
		label_color)
