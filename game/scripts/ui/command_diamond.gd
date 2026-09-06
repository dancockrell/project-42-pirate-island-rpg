class_name VellumCommandDiamond
extends Control

## One command in the grid: a bronze diamond carrying its authored rank letter
## and its name, in one of three states.
##
## `UNLOCKED` is a command the actor may give now. `LOCKED` is one the actor has
## not earned -- it is still drawn, still named, still explains itself, because
## a player who cannot see what a rank leads to cannot aim at it. `REFUSED` is
## one that is earned but illegal right now (no legal target, not this actor's
## turn, an anchor already spent today); it says why in its tooltip rather than
## disappearing, which is the difference between a calm interface and one that
## rearranges itself under the player's hand.
##
## This is D11's command grid slot as a reusable component, generalising the
## battle prototype's `PaperSkillDiamond`; the prototype adopts it in P7.

signal command_pressed(command_id: String)

enum State { UNLOCKED, LOCKED, REFUSED }

const SLOT_SIZE := Vector2(124, 96)

var command_id := ""
var rank := ""
var display_name := ""
var state: State = State.UNLOCKED
## Why this command is refused or locked, in the words the player is shown.
var reason := ""

var _button: Button


func _ready() -> void:
	custom_minimum_size = SLOT_SIZE
	if _button == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


func configure(next_command_id: String, next_rank: String, next_display_name: String, next_state: State, next_reason := "") -> void:
	command_id = next_command_id
	rank = next_rank
	display_name = next_display_name
	state = next_state
	reason = next_reason
	if _button == null:
		_build()
	_refresh()


func _build() -> void:
	_button = Button.new()
	_button.name = "Command"
	_button.flat = true
	_button.focus_mode = Control.FOCUS_ALL
	_button.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_button.pressed.connect(func() -> void: command_pressed.emit(command_id))
	add_child(_button)


func _refresh() -> void:
	if _button == null:
		return
	_button.disabled = state != State.UNLOCKED
	tooltip_text = tooltip_lines()
	_button.tooltip_text = tooltip_text
	queue_redraw()


## What the diamond says about itself: rank and name, the stable ID in the mono
## face, and the reason when there is one. A locked slot always identifies its
## rank and skill, exactly as the prototype's diamond did.
func tooltip_lines() -> String:
	var lines := "%s rank  ·  %s\n%s" % [rank, display_name, command_id]
	if not reason.is_empty():
		lines += "\n%s" % reason
	return lines


func _make_custom_tooltip(_for_text: String) -> Object:
	var tooltip := VellumTooltip.new()
	tooltip.theme = ThemeTokens.active(self)
	tooltip.configure("%s rank  ·  %s" % [rank, display_name], reason, command_id)
	return tooltip


func _draw() -> void:
	var active := ThemeTokens.active(self)
	var centre := size * 0.5 - Vector2(0, 8)
	var radius: float = minf(size.x, size.y) * 0.31
	var points := PackedVector2Array([
		centre + Vector2(0, -radius), centre + Vector2(radius, 0),
		centre + Vector2(0, radius), centre + Vector2(-radius, 0),
	])
	var fill_token := "panel_raised"
	var edge_token := "rule"
	var rank_token := "muted"
	var name_token := "muted"
	match state:
		State.UNLOCKED:
			fill_token = "teal_deep"
			edge_token = "teal"
			rank_token = "cream"
			name_token = "cream"
		State.LOCKED:
			fill_token = "panel_sunken"
			edge_token = "rule_faint"
		State.REFUSED:
			fill_token = "panel_sunken"
			edge_token = "rule"
			rank_token = "bronze"
	draw_colored_polygon(points, ThemeTokens.color(active, fill_token))
	var outline := PackedVector2Array(points)
	outline.append(points[0])
	draw_polyline(outline, ThemeTokens.color(active, edge_token), 2.0, true)
	if state == State.UNLOCKED:
		draw_arc(centre, radius * 0.33, 0.0, TAU, 20, ThemeTokens.color(active, "bronze_bright"), 1.5, true)
	elif state == State.REFUSED:
		# A refused command is struck through rather than dimmed away: the
		# player can see it exists, and the tooltip says why it is not offered.
		draw_line(centre + Vector2(-radius, radius) * 0.62, centre + Vector2(radius, -radius) * 0.62,
			ThemeTokens.color(active, "rule"), 1.5, true)

	var font := get_theme_default_font()
	var rank_size := ThemeTokens.font_size(active, "body")
	draw_string(font, centre + Vector2(-radius, rank_size * 0.36), rank, HORIZONTAL_ALIGNMENT_CENTER,
		radius * 2.0, rank_size, ThemeTokens.color(active, rank_token))
	var caption := ThemeTokens.font_size(active, "caption")
	draw_string(font, Vector2(4, size.y - 6), display_name, HORIZONTAL_ALIGNMENT_CENTER,
		size.x - 8, caption, ThemeTokens.color(active, name_token))
