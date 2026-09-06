class_name VellumNotice
extends Control

## How the three levels of brief section 14 reach the screen.
##
## * **Ambient** never builds a notice at all. `VellumNotice.present()` returns
##   `null` for it, and the caller's only record of it is the journal line the
##   surface already wrote. That is what "most changes simply appear in the
##   world" has to mean in code: silence, not a quiet toast.
## * **Notable** is a companion line -- a strip along the bottom of the screen,
##   `MOUSE_FILTER_IGNORE` and `FOCUS_NONE` throughout, that never pauses,
##   never takes the keyboard, and lets go by itself after
##   `NOTABLE_SECONDS`. Nothing about it is clickable, so it cannot steal a
##   click aimed at the world underneath.
## * **Urgent** is a modal: the screen dims, the game pauses through `GamePause`
##   (which is what the brief means by "the game interrupts"), and the player's
##   acknowledgement is what releases it.
##
## Reduced motion is honoured by substitution, not by removal: the fade in and
## out become an instant show and hide, and the Notable strip still lives
## exactly as long. Nothing is ever conveyed by motion alone.

signal acknowledged

## How long a Notable strip stays before it lets go of itself.
const NOTABLE_SECONDS := 6.0
## The fade, when motion is not reduced.
const FADE_SECONDS := 0.28
## The reason an Urgent notice gives `GamePause`. The surface that gives a
## reason owns its name, as `SettingsPanel.PAUSE_REASON` does.
const PAUSE_REASON := "urgent_notice"

## How much room the two shapes take. An Urgent modal is a fixed block in the
## middle of the screen; a Notable strip is a band along the bottom, inset from
## both edges so it reads as a line somebody spoke rather than as a bar.
const MODAL_SIZE := Vector2(560.0, 148.0)
const STRIP_HEIGHT := 74.0
const STRIP_INSET := 40.0

var level: int = InformationLevel.Level.NOTABLE
var headline := ""
var body := ""
## Who is telling the player. A companion's name, for a Notable line; empty
## when the island itself is the messenger.
var source_name := ""

## Draw the notice without any of its behaviour: no pause, no dismissal timer,
## no fade. The style guide (`scenes/review/grammar_review.tscn`) shows a
## Notable strip and an Urgent modal side by side this way, which is the only
## way to look at both at once. Set it before the notice enters the tree.
var preview := false

var _panel: PanelContainer
var _dim: ColorRect
var _headline_label: Label
var _body_label: Label
var _source_label: Label
var _acknowledge: Button
var _life: SceneTreeTimer
var _game_pause: Node


## Build a notice for `level`, or `null` when the level is silent. The only way
## a notice should be made, because "Ambient shows nothing" is a rule about the
## whole interface and not a decision each caller re-takes.
static func present(level_value: int, headline_text: String, body_text := "", source := "") -> VellumNotice:
	if InformationLevel.is_silent(level_value):
		return null
	var notice := VellumNotice.new()
	notice.level = level_value
	notice.headline = headline_text
	notice.body = body_text
	notice.source_name = source
	return notice


func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	if _panel == null:
		_build()
	_refresh()
	if preview:
		modulate.a = 1.0
		return
	if InformationLevel.interrupts(level):
		_begin_urgent()
	else:
		_begin_notable()


func _exit_tree() -> void:
	_release_pause()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


## The player has seen it. Releases the pause an Urgent notice took and frees
## the notice; a Notable strip calls this on itself when its time is up.
func acknowledge() -> void:
	if is_queued_for_deletion():
		return
	_release_pause()
	acknowledged.emit()
	if ThemeTokens.reduced_motion(ThemeTokens.active(self)):
		queue_free()
		return
	var fade := create_tween()
	fade.tween_property(self, "modulate:a", 0.0, FADE_SECONDS)
	fade.tween_callback(queue_free)


func _build() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_dim = ColorRect.new()
	_dim.name = "Dim"
	_dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_dim)

	_panel = PanelContainer.new()
	_panel.name = "Notice"
	add_child(_panel)

	var column := VBoxContainer.new()
	column.name = "Column"
	column.add_theme_constant_override("separation", 8)
	_panel.add_child(column)

	_source_label = Label.new()
	_source_label.name = "Source"
	column.add_child(_source_label)
	_headline_label = Label.new()
	_headline_label.name = "Headline"
	column.add_child(_headline_label)
	_body_label = Label.new()
	_body_label.name = "Body"
	_body_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_body_label)
	_acknowledge = Button.new()
	_acknowledge.name = "Acknowledge"
	_acknowledge.text = "UNDERSTOOD"
	_acknowledge.pressed.connect(acknowledge)
	column.add_child(_acknowledge)

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing or _panel == null:
		return
	_refreshing = true
	var active := ThemeTokens.active(self)
	var urgent := InformationLevel.interrupts(level)
	_panel.add_theme_stylebox_override("panel",
		ThemeTokens.style(active, "VellumNotice", "urgent" if urgent else "notable"))
	_dim.color = Color(ThemeTokens.color(active, "deep"), 0.72 if urgent else 0.0)
	_dim.visible = urgent

	if urgent:
		_panel.set_anchors_and_offsets_preset(Control.PRESET_CENTER, Control.PRESET_MODE_KEEP_SIZE)
		_panel.custom_minimum_size = Vector2(0, 0)
		_panel.anchor_left = 0.5
		_panel.anchor_right = 0.5
		_panel.anchor_top = 0.5
		_panel.anchor_bottom = 0.5
		_panel.offset_left = -MODAL_SIZE.x * 0.5
		_panel.offset_right = MODAL_SIZE.x * 0.5
		_panel.offset_top = -MODAL_SIZE.y * 0.5
		_panel.offset_bottom = MODAL_SIZE.y * 0.5
	else:
		_panel.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE, Control.PRESET_MODE_KEEP_SIZE)
		_panel.offset_left = STRIP_INSET
		_panel.offset_right = -STRIP_INSET
		_panel.offset_top = -STRIP_HEIGHT
		_panel.offset_bottom = 0.0

	_source_label.text = source_name.to_upper()
	_source_label.visible = not source_name.is_empty()
	_source_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
	_source_label.add_theme_color_override("font_color",
		ThemeTokens.color(active, InformationLevel.token_of(level)))
	_headline_label.text = headline
	_headline_label.add_theme_font_size_override("font_size",
		ThemeTokens.font_size(active, "title" if urgent else "body"))
	_headline_label.add_theme_color_override("font_color", ThemeTokens.color(active, "cream"))
	_body_label.text = body
	_body_label.visible = not body.is_empty()
	_body_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "body"))
	_body_label.add_theme_color_override("font_color", ThemeTokens.color(active, "muted"))
	# Only an Urgent notice is answerable. A Notable line carries no control at
	# all, so there is nothing on it for a click or the keyboard to land on.
	_acknowledge.visible = urgent
	_set_focus_blocked(not urgent)


## A Notable strip takes neither the mouse nor the keyboard, all the way down.
	_refreshing = false
func _set_focus_blocked(blocked: bool) -> void:
	var pending: Array[Node] = [self]
	while not pending.is_empty():
		var node: Node = pending.pop_back()
		if node is Control:
			var control := node as Control
			if blocked:
				control.mouse_filter = Control.MOUSE_FILTER_IGNORE
				control.focus_mode = Control.FOCUS_NONE
			elif control != self and control != _dim:
				control.mouse_filter = Control.MOUSE_FILTER_STOP
		pending.append_array(node.get_children())


func _begin_notable() -> void:
	_fade_in()
	_life = get_tree().create_timer(NOTABLE_SECONDS, true, false, true)
	_life.timeout.connect(acknowledge)


func _begin_urgent() -> void:
	_fade_in()
	_game_pause = get_node_or_null("/root/GamePause")
	if _game_pause != null:
		_game_pause.pause(PAUSE_REASON)
	_acknowledge.grab_focus()


func _fade_in() -> void:
	if ThemeTokens.reduced_motion(ThemeTokens.active(self)):
		modulate.a = 1.0
		return
	modulate.a = 0.0
	create_tween().tween_property(self, "modulate:a", 1.0, FADE_SECONDS)


func _release_pause() -> void:
	if _game_pause == null:
		return
	_game_pause.resume(PAUSE_REASON)
	_game_pause = null
