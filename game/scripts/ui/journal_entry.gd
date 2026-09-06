class_name VellumJournalEntry
extends PanelContainer

## One line of S11's strategic journal, as the player reads it: when it
## happened, at what level it was shown, and what happened, in prose.
##
## The stamp is drawn in the mono step because it is a coordinate, not a
## sentence. The level mark is a word and a colour and never a flashing icon --
## brief section 14's avoid list is what this component is shaped around.

var day := 0
var hour := 0
var level := 0
var prose := ""

var _stamp: Label
var _mark: Label
var _prose: Label


func _ready() -> void:
	if _stamp == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


## `entry` is one journal record: `day`, `hour`, `level` (an
## `InformationLevel.Level`) and `prose`.
func configure(entry: Dictionary) -> void:
	day = int(entry.get("day", 0))
	hour = int(entry.get("hour", 0))
	level = int(entry.get("level", 0))
	prose = str(entry.get("prose", ""))
	if _stamp == null:
		_build()
	_refresh()


func _build() -> void:
	var row := HBoxContainer.new()
	row.name = "Row"
	row.add_theme_constant_override("separation", 14)
	add_child(row)
	_stamp = Label.new()
	_stamp.name = "Stamp"
	_stamp.custom_minimum_size.x = 108
	row.add_child(_stamp)
	_mark = Label.new()
	_mark.name = "Mark"
	_mark.custom_minimum_size.x = 78
	row.add_child(_mark)
	_prose = Label.new()
	_prose.name = "Prose"
	_prose.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_prose.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	row.add_child(_prose)

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing or _stamp == null:
		return
	_refreshing = true
	var active := ThemeTokens.active(self)
	add_theme_stylebox_override("panel", ThemeTokens.style(active, "VellumJournal", "row"))
	_stamp.text = "DAY %02d · %02d:00" % [day, hour]
	_stamp.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "mono"))
	_stamp.add_theme_color_override("font_color", ThemeTokens.color(active, "rule"))
	var mono := ThemeTokens.mono_font(active)
	if mono != null:
		_stamp.add_theme_font_override("font", mono)
	_mark.text = InformationLevel.name_of(level)
	_mark.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
	_mark.add_theme_color_override("font_color", ThemeTokens.color(active, InformationLevel.token_of(level)))
	_prose.text = prose
	_prose.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "body"))
	_prose.add_theme_color_override("font_color", ThemeTokens.color(active, "cream"))
	_refreshing = false
