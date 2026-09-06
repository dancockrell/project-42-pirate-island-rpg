class_name ShellChooser
extends Control

## The shell's one list-and-pick surface.
##
## The title's Load screen and the pause menu's Save screen ask the same
## question -- which slot -- so they ask it with the same surface rather than
## growing two save screens that would drift apart. The caller supplies the
## rows and what each row means; this control knows nothing about saves.
##
## A row whose payload is empty is drawn but cannot be chosen: that is how a
## broken slot appears on the load screen, listed and marked, without needing a
## second kind of row.

signal chosen(payload: String)
signal closed

## The one door onto the palette and the type scale (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

const ROW_HEIGHT := 46

var rows_box: VBoxContainer
var row_buttons: Array[Button] = []
var close_button: Button


func _init() -> void:
	# The chooser opens over a paused game and must keep answering.
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func configure(heading: String, note: String, lines: Array[String], payloads: Array[String]) -> void:
	build(heading, note)
	for index in lines.size():
		var payload := payloads[index] if index < payloads.size() else ""
		add_row(lines[index], payload)
	if lines.is_empty():
		rows_box.add_child(ShellStyle.label("No slot has been recorded yet.", ShellStyle.MUTED))
	if is_inside_tree():
		grab_first_focus()


## `configure` is called before the chooser is added to the tree, and a control
## outside the tree cannot take focus, so the first row claims it here.
func _ready() -> void:
	# `configure` runs before the chooser is added, so the Theme it built with
	# was the resource as authored. Now that it is in the tree it takes the one
	# in force and subscribes to the next rebuild.
	_on_theme_rebuilt(ThemeTokensScript.adopt(self))
	grab_first_focus()


func build(heading: String, note: String) -> void:
	ThemeTokensScript.adopt(self)
	var scrim := ColorRect.new()
	scrim.name = "Scrim"
	ShellStyle.paint(scrim, theme, "night", 0.90)
	scrim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(scrim)

	var centre := CenterContainer.new()
	centre.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(centre)

	var card := PanelContainer.new()
	card.name = "ChooserCard"
	card.custom_minimum_size = Vector2(880, 0)
	card.add_theme_stylebox_override("panel", ShellStyle.card_box(theme))
	centre.add_child(card)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 12)
	card.add_child(page)
	page.add_child(ShellStyle.label(heading, ShellStyle.TITLE))
	page.add_child(ShellStyle.label(note, ShellStyle.MUTED))
	page.add_child(ShellStyle.rule(theme, "hairline", 840.0))

	rows_box = VBoxContainer.new()
	rows_box.name = "Rows"
	rows_box.add_theme_constant_override("separation", 4)
	page.add_child(rows_box)

	page.add_child(ShellStyle.rule(theme, "hairline", 840.0))
	close_button = make_row_button("CLOSE", "muted")
	close_button.name = "CloseChooser"
	close_button.pressed.connect(close)
	page.add_child(close_button)


func add_row(text: String, payload: String) -> void:
	var button := make_row_button(text, "hairline" if not payload.is_empty() else "danger")
	button.name = "Row%d" % row_buttons.size()
	if payload.is_empty():
		button.disabled = true
	else:
		button.pressed.connect(func() -> void: choose(payload))
	rows_box.add_child(button)
	row_buttons.append(button)


## `bar_token` is a palette token: a row that can be chosen carries the quiet
## hairline bar, a row for a slot that will not load carries the danger bar. The
## Theme decides what either of those looks like.
func make_row_button(text: String, bar_token: String) -> Button:
	var button := Button.new()
	button.text = text
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.custom_minimum_size = Vector2(840, ROW_HEIGHT)
	return ShellStyle.bar_button(button, theme, bar_token)


## The grammar changed under the chooser: the scrim, the card and every row are
## built in code, so all three are put back here.
func _on_theme_rebuilt(rebuilt: Theme) -> void:
	theme = rebuilt
	ShellStyle.repaint_marked(self, rebuilt)
	ShellStyle.restyle_bar_buttons(self, rebuilt)
	for node in find_children("ChooserCard", "PanelContainer", true, false):
		(node as PanelContainer).add_theme_stylebox_override("panel", ShellStyle.card_box(rebuilt))


func grab_first_focus() -> void:
	for button in row_buttons:
		if not button.disabled:
			button.grab_focus()
			return
	if close_button != null:
		close_button.grab_focus()


func choose(payload: String) -> void:
	chosen.emit(payload)
	close()


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		get_viewport().set_input_as_handled()
		close()


func close() -> void:
	closed.emit()
	queue_free()
