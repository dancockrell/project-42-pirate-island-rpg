class_name VellumPanel
extends PanelContainer

## A sheet of the grammar: soft paper fill, one hairline bronze rule, no gloss.
##
## Two variants, both owned by `themes/bronze_vellum.tres`: `panel` is a raised
## sheet for anything the player reads, `well` is the sunken ground a rail or a
## list sits in. Nothing else about a panel is configurable, because a screen
## made of six slightly different panels is the debug-panel look this card
## exists to leave behind.

const VARIANT_PANEL := "panel"
const VARIANT_WELL := "well"

## The heading printed above the content, in the type scale's `title` step.
## Empty for a panel that carries no heading.
@export var heading := "":
	set(value):
		heading = value
		_refresh()

## `panel` or `well`.
@export var variant := VARIANT_PANEL:
	set(value):
		variant = value
		_refresh()

var _heading_label: Label
var _rule: Panel
var _body: VBoxContainer


func _init() -> void:
	_body = VBoxContainer.new()
	_body.name = "Body"
	_body.add_theme_constant_override("separation", 10)


func _ready() -> void:
	if _heading_label == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


## Where a caller puts what the panel is about. Always a VBoxContainer, so a
## panel's contents stack in the same rhythm everywhere.
func body() -> VBoxContainer:
	if _heading_label == null:
		_build()
	return _body


func _build() -> void:
	var column := VBoxContainer.new()
	column.name = "Column"
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	_heading_label = Label.new()
	_heading_label.name = "Heading"
	column.add_child(_heading_label)
	_rule = Panel.new()
	_rule.name = "Rule"
	_rule.custom_minimum_size = Vector2(0, 1)
	_rule.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(_rule)
	column.add_child(_body)

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing or _heading_label == null:
		return
	_refreshing = true
	var active := ThemeTokens.active(self)
	add_theme_stylebox_override("panel", ThemeTokens.style(active, "VellumPanel", variant))
	_heading_label.text = heading.to_upper()
	_heading_label.visible = not heading.is_empty()
	_rule.visible = not heading.is_empty()
	_heading_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "title"))
	_heading_label.add_theme_color_override("font_color", ThemeTokens.color(active, "bronze"))
	var hairline := StyleBoxFlat.new()
	hairline.bg_color = ThemeTokens.color(active, "rule_faint")
	_rule.add_theme_stylebox_override("panel", hairline)
	_refreshing = false
