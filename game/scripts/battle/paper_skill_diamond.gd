class_name PaperSkillDiamond
extends Control

## One real command slot in the compact D-to-SSS skill grid. The visual is a
## cut-paper bronze diamond; the transparent Button is the input owner. A
## locked diamond always identifies its authored rank and skill in a tooltip.

## The one door onto the palette and the type scale (card P11). A command slot
## is interface: it declares no colour of its own.
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

signal command_pressed(skill_id: String)

var skill_id := ""
var rank := "D"
var display_name := ""
var story_locked := true
var command_button: Button

func configure(next_skill_id: String, next_rank: String, next_display_name: String, available: bool) -> void:
	skill_id = next_skill_id
	rank = next_rank
	display_name = next_display_name
	story_locked = not available
	custom_minimum_size = Vector2(118, 92)
	tooltip_text = "%s rank — %s\nStable skill ID: %s%s" % [rank, display_name, skill_id, "\nLocked until its authored bond milestone." if story_locked else ""]
	if command_button != null:
		set_command_enabled(available)
	queue_redraw()

func _ready() -> void:
	command_button = Button.new()
	command_button.flat = true
	command_button.tooltip_text = tooltip_text
	command_button.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	command_button.pressed.connect(func() -> void: command_pressed.emit(skill_id))
	add_child(command_button)
	set_command_enabled(not story_locked)

func set_command_enabled(enabled: bool) -> void:
	story_locked = not enabled
	if command_button != null:
		command_button.disabled = not enabled
		command_button.tooltip_text = "%s rank — %s\nStable skill ID: %s%s" % [rank, display_name, skill_id, "\nLocked until its authored bond milestone." if not enabled else ""]
	queue_redraw()

func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		queue_redraw()


func _draw() -> void:
	var theme := ThemeTokensScript.active(self)
	var center := Vector2(size.x * .5, size.y * .5)
	var r: float = float(min(size.x, size.y)) * .30
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
	var font := ThemeDB.fallback_font
	var rank_font_size := ThemeTokensScript.font_size(theme, "caption" if rank.length() > 1 else "body")
	draw_string(font,center + Vector2(-17,5),rank,HORIZONTAL_ALIGNMENT_CENTER,34,rank_font_size,ThemeTokensScript.color(theme, "focus" if not story_locked else "muted"))
	# The command's name is a label the player reads at a glance in a lit room,
	# not a caption. It sits on its own dark plate and it is not grey-on-grey:
	# a locked command is still legible, it is just visibly not available.
	var label_color := ThemeTokensScript.color(theme, "cream" if not story_locked else "muted")
	var plate := Rect2(2, size.y - 22, size.x - 4, 18)
	draw_rect(plate, Color(ThemeTokensScript.color(theme, "night"), .62), true)
	draw_string(font,Vector2(2,size.y - 8),display_name.to_upper(),HORIZONTAL_ALIGNMENT_CENTER,size.x - 4,ThemeTokensScript.font_size(theme, "caption"),label_color)
