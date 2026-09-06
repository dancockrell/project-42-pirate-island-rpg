extends Node

## The one owner of "is the game paused".
##
## Brief section 14: the game pauses normally, and while it is paused local
## movement, combat, strategic simulation, construction, convoys, weather and
## Cthulhu's pressure all stop. Brief section 17 adds the constraint that makes
## the implementation simple: normal pausing must not change simulation
## outcomes.
##
## `godot-rust/src/strategy/tick.rs` already settles where pause lives. There
## is no `paused` field in `ExpeditionState`, no paused branch in the tick, and
## no clock that reads the host machine: a paused game is a game whose bridge is
## not calling. So pause is a Godot-side fact, and this autoload is the single
## place that states it. Nothing in Rust is asked whether the game is paused,
## and nothing in Rust is told.
##
## Pause is a set of reasons rather than a flag, because the surfaces that pause
## overlap: the settings panel and a reading surface can both be open, and the
## game stays paused until both close. `pause("settings")` twice is still one
## reason; `resume` for a reason that was never given changes nothing.

## Emitted whenever the paused verdict actually changes. A surface that must
## react to pause listens here rather than polling `is_paused()` per frame.
signal pause_changed(paused: bool)

var _reasons: Dictionary = {}


func _ready() -> void:
	# The pause owner cannot be paused by its own verdict, or it could never
	# hear the call that resumes.
	process_mode = Node.PROCESS_MODE_ALWAYS


## Holds the game for `reason`. Returns true when this call is what paused it.
func pause(reason: String) -> bool:
	if reason.is_empty():
		push_error("GamePause.pause requires a named reason.")
		return false
	if _reasons.has(reason):
		return false
	var was_paused := is_paused()
	_reasons[reason] = true
	return apply(was_paused)


## Releases `reason`. The game resumes only once every reason has been
## released. Returns true when this call is what resumed it.
func resume(reason: String) -> bool:
	if not _reasons.has(reason):
		return false
	var was_paused := is_paused()
	_reasons.erase(reason)
	return apply(was_paused)


func is_paused() -> bool:
	return not _reasons.is_empty()


## Every reason currently holding the game, in a stable order so a status line
## or a test can read them without depending on insertion order.
func pause_reasons() -> Array[String]:
	var reasons: Array[String] = []
	for reason in _reasons.keys():
		reasons.append(str(reason))
	reasons.sort()
	return reasons


## Releases every reason at once. This is how a scene change or a test leaves a
## known state; ordinary play releases the reason it took.
func clear() -> void:
	if _reasons.is_empty():
		return
	_reasons.clear()
	apply(true)


func apply(was_paused: bool) -> bool:
	var now_paused := is_paused()
	var tree := get_tree()
	if tree != null:
		tree.paused = now_paused
	if now_paused == was_paused:
		return false
	pause_changed.emit(now_paused)
	return true
