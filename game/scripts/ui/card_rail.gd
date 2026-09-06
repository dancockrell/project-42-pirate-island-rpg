class_name VellumCardRail
extends PanelContainer

## The party, along the bottom of a screen: a sunken well holding one
## `VellumCard` per actor, in the order the caller gives them.
##
## D11's rail as a component. It owns layout and selection and nothing else --
## it never decides who is in the party, never reorders, never hides a card.
## `set_entries` is the whole input; `selected_actor_id` and `card_selected` are
## the whole output.

signal card_selected(actor_id: String)

const SEPARATION := 14

## Whether the cards sit in the middle of the rail rather than against its left
## edge. A party that fills the rail wants neither; a short one looks composed
## centred and unfinished left-aligned.
var centred := false:
	set(value):
		centred = value
		if _row != null:
			_row.alignment = BoxContainer.ALIGNMENT_CENTER if centred else BoxContainer.ALIGNMENT_BEGIN

var selected_actor_id := ""

var _row: HBoxContainer
var _cards: Array[VellumCard] = []


func _ready() -> void:
	if _row == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


## Replace the rail's contents. One dictionary per actor, in `VellumCard`'s
## shape; an empty array empties the rail.
func set_entries(entries: Array) -> void:
	if _row == null:
		_build()
	for card in _cards:
		card.queue_free()
	_cards.clear()
	for entry in entries:
		var card := VellumCard.new()
		card.name = "Card%d" % _cards.size()
		_row.add_child(card)
		card.configure(entry as Dictionary)
		card.pressed.connect(_on_card_pressed)
		_cards.append(card)
	if not entries.is_empty() and selected_actor_id.is_empty():
		selected_actor_id = str((entries[0] as Dictionary).get("actor_id", ""))
	_apply_selection()


## The cards currently on the rail, in order. Read-only to callers; the rail
## rebuilds them from `set_entries`.
func cards() -> Array[VellumCard]:
	return _cards.duplicate()


func select(actor_id: String) -> void:
	selected_actor_id = actor_id
	_apply_selection()
	card_selected.emit(actor_id)


func _on_card_pressed(actor_id: String) -> void:
	select(actor_id)


func _apply_selection() -> void:
	for card in _cards:
		card.set_selected(card.actor_id == selected_actor_id)


func _build() -> void:
	_row = HBoxContainer.new()
	_row.name = "Cards"
	_row.add_theme_constant_override("separation", SEPARATION)
	_row.alignment = BoxContainer.ALIGNMENT_CENTER if centred else BoxContainer.ALIGNMENT_BEGIN
	add_child(_row)

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing:
		return
	_refreshing = true
	add_theme_stylebox_override("panel", ThemeTokens.style(ThemeTokens.active(self), "VellumPanel", "well"))
	_refreshing = false
