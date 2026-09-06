class_name VellumCommandGrid
extends GridContainer

## The command grid: `VellumCommandDiamond`s in reading order, in as many
## columns as the caller asks for.
##
## The grid never filters. Every command the actor has is drawn, locked ones
## included, because a grid that hid what it could not offer would change shape
## under the player's hand -- the thing brief section 14 calls menacing.

signal command_pressed(command_id: String)

const DEFAULT_COLUMNS := 4

var _slots: Array[VellumCommandDiamond] = []


func _ready() -> void:
	if columns < 1:
		columns = DEFAULT_COLUMNS
	add_theme_constant_override("h_separation", 10)
	add_theme_constant_override("v_separation", 10)


## One dictionary per command: `command_id`, `rank`, `display_name`, `state`
## (a `VellumCommandDiamond.State`), and an optional `reason`.
func set_commands(commands: Array) -> void:
	for slot in _slots:
		slot.queue_free()
	_slots.clear()
	for entry in commands:
		var command := entry as Dictionary
		var slot := VellumCommandDiamond.new()
		slot.name = "Slot%d" % _slots.size()
		add_child(slot)
		slot.configure(
			str(command.get("command_id", "")),
			str(command.get("rank", "")),
			str(command.get("display_name", "")),
			command.get("state", VellumCommandDiamond.State.UNLOCKED) as VellumCommandDiamond.State,
			str(command.get("reason", "")))
		slot.command_pressed.connect(func(id: String) -> void: command_pressed.emit(id))
		_slots.append(slot)


func slots() -> Array[VellumCommandDiamond]:
	return _slots.duplicate()
