class_name VellumTooltip
extends PanelContainer

## What a control says about itself when the player rests on it: a title, an
## optional line of prose, and the stable ID in the mono face.
##
## The ID is shown on purpose. Every id in this project is stable and
## namespaced, and a player or a bug report that can quote one is worth more
## than a tidy tooltip. It is drawn in the mono step so it reads as a machine
## fact rather than as a sentence.

var _title: Label
var _body: Label
var _identifier: Label


func _ready() -> void:
	if _title == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


func configure(title: String, body := "", identifier := "") -> void:
	if _title == null:
		_build()
	_title.text = title
	_body.text = body
	_body.visible = not body.is_empty()
	_identifier.text = identifier
	_identifier.visible = not identifier.is_empty()
	_refresh()


func _build() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := VBoxContainer.new()
	column.name = "Column"
	column.add_theme_constant_override("separation", 4)
	add_child(column)
	_title = Label.new()
	_title.name = "Title"
	column.add_child(_title)
	_body = Label.new()
	_body.name = "Body"
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_body.custom_minimum_size.x = 260
	column.add_child(_body)
	_identifier = Label.new()
	_identifier.name = "Identifier"
	column.add_child(_identifier)

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing or _title == null:
		return
	_refreshing = true
	var active := ThemeTokens.active(self)
	add_theme_stylebox_override("panel", ThemeTokens.style(active, "VellumTooltip", "panel"))
	_title.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "body"))
	_title.add_theme_color_override("font_color", ThemeTokens.color(active, "cream"))
	_body.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
	_body.add_theme_color_override("font_color", ThemeTokens.color(active, "muted"))
	_identifier.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "mono"))
	_identifier.add_theme_color_override("font_color", ThemeTokens.color(active, "rule"))
	var mono := ThemeTokens.mono_font(active)
	if mono != null:
		_identifier.add_theme_font_override("font", mono)
	_refreshing = false
