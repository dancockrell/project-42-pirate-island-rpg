extends Node

## The one owner of "which screen is on the glass".
##
## Before this autoload every screen changed the scene itself: the expedition
## called `change_scene_to_file` on the battle and the battle called it back.
## Two screens each knowing how to become the other is two answers to one
## question, and neither of them could fade, hold the campaign, or show that
## something was loading. So the two calls became one call each into here, and
## the transition -- veil down, threaded load, swap, veil up -- lives in one
## place.
##
## What this node deliberately does not do: it does not hold campaign state.
## `CampaignSession` is the one holder of the campaign and survives every scene
## swap because it is an autoload; this node never copies a snapshot, never
## reads one, and never decides a game result. It moves screens.

## The one door onto the palette and the type scale (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

const TITLE_SCENE := "res://scenes/shell/title.tscn"
const EXPEDITION_SCENE := "res://scenes/world/expedition_prototype.tscn"
const BATTLE_SCENE := "res://scenes/battle/battle_prototype.tscn"
const PAUSE_MENU_SCENE := "res://scenes/shell/pause_menu.tscn"

## Every screen this flow will move to. A path that is not on this list is
## refused rather than loaded, so a typed or constructed path can never become
## a scene swap.
const KNOWN_SCENES := [TITLE_SCENE, EXPEDITION_SCENE, BATTLE_SCENE]

## The screens Escape may pause. The title is not one of them: a menu over a
## menu is a trap, and the title's own controls already lead out.
const PAUSABLE_SCENES := [EXPEDITION_SCENE, BATTLE_SCENE]

## How long the veil takes to close and to open again. Short enough that the
## game feels responsive and long enough to read as a deliberate cut rather
## than a stutter. Reduced motion collapses both to zero.
const FADE_SECONDS := 0.22

## The veil paints the same deep green-black every screen opens on, so a
## transition reads as the island going dark rather than as a black frame. Card
## P11: it is `night` and `muted` out of the Theme now, not two hexes stated
## here, so a transition between two high-contrast screens is not the one frame
## that forgets.
const VEIL_TOKEN := "night"
## The variation the loading line is written in, which is the `muted` token.
const VEIL_TEXT_VARIATION := &"MutedLabel"

## What screen the flow last landed on, and the only thing outside this node
## that needs to know. A transition-started and a scene-changed signal were
## written here first and then deleted before shipping, because nothing
## subscribed to either: the shell's own suite watches this path and
## `is_transitioning()`, so a signal it missed by one frame would hang the gate
## rather than fail it. The signals come back the day a screen actually needs
## to hear about a transition.

var current_scene_path := ""

var _target_path := ""
var _surface: Node
var _veil_layer: CanvasLayer
var _veil: ColorRect
var _veil_label: Label
var _pause_menu: Node


func _ready() -> void:
	# The flow cannot be paused by the pause menu it opens, or it could never
	# lift the veil or answer the button that resumes.
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_process(false)
	build_veil()


# ---------------------------------------------------------------------------
# The four transitions. Every screen change in the game is one of these.
# ---------------------------------------------------------------------------


func go_to_title() -> bool:
	return transition_to(TITLE_SCENE)


func enter_expedition() -> bool:
	return transition_to(EXPEDITION_SCENE)


func enter_battle() -> bool:
	return transition_to(BATTLE_SCENE)


func return_to_expedition() -> bool:
	return transition_to(EXPEDITION_SCENE)


## True when this call started a transition. A second call while one is in
## flight is refused rather than queued: two overlapping scene swaps would free
## a screen out from under the one loading.
func transition_to(target_path: String) -> bool:
	if is_transitioning():
		return false
	if not KNOWN_SCENES.has(target_path):
		return false
	close_pause_menu()
	_target_path = target_path
	if ResourceLoader.load_threaded_request(target_path) != OK:
		_target_path = ""
		return false
	set_veil_alpha(1.0 if reduced_motion() else 0.0)
	if not reduced_motion():
		var tween := create_tween()
		tween.tween_method(set_veil_alpha, 0.0, 1.0, FADE_SECONDS)
	set_process(true)
	return true


func is_transitioning() -> bool:
	return not _target_path.is_empty()


## The loading veil is polled rather than awaited: `load_threaded_get_status`
## is the only way to know a threaded load has landed, and a screen that took
## three seconds to load must still be able to draw the veil while it waits.
func _process(_delta: float) -> void:
	if _target_path.is_empty():
		set_process(false)
		return
	var status := ResourceLoader.load_threaded_get_status(_target_path)
	if status == ResourceLoader.THREAD_LOAD_IN_PROGRESS:
		return
	set_process(false)
	var landed := _target_path
	_target_path = ""
	if status != ResourceLoader.THREAD_LOAD_LOADED:
		set_veil_alpha(0.0)
		return
	var packed := ResourceLoader.load_threaded_get(landed) as PackedScene
	if packed == null:
		set_veil_alpha(0.0)
		return
	finish_transition(landed, packed)


func finish_transition(landed: String, packed: PackedScene) -> void:
	# The veil must be fully down before the swap, or the player sees the old
	# screen vanish. With reduced motion it already is.
	if not reduced_motion():
		set_veil_alpha(1.0)
	get_tree().change_scene_to_packed(packed)
	# change_scene_to_packed is deferred to the end of the frame, so the new
	# scene is only the current scene one frame later.
	await get_tree().process_frame
	current_scene_path = landed
	if reduced_motion():
		set_veil_alpha(0.0)
	else:
		var tween := create_tween()
		tween.tween_method(set_veil_alpha, 1.0, 0.0, FADE_SECONDS)


# ---------------------------------------------------------------------------
# The pause menu. Escape opens it on the screens that have a game running
# behind them; the menu itself owns the GamePause reason.
# ---------------------------------------------------------------------------


func _unhandled_input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return
	if is_transitioning() or not PAUSABLE_SCENES.has(current_scene_path):
		return
	if open_pause_menu():
		get_viewport().set_input_as_handled()


## Opens the pause menu over the running screen. True when this call opened it;
## false when one is already open, which is what makes Escape idempotent rather
## than a way to stack menus.
func open_pause_menu() -> bool:
	if is_instance_valid(_pause_menu):
		return false
	var packed := load(PAUSE_MENU_SCENE) as PackedScene
	if packed == null:
		return false
	_pause_menu = packed.instantiate()
	get_tree().root.add_child(_pause_menu)
	return true


func close_pause_menu() -> void:
	if not is_instance_valid(_pause_menu):
		_pause_menu = null
		return
	_pause_menu.close()
	_pause_menu = null


func pause_menu() -> Node:
	return _pause_menu if is_instance_valid(_pause_menu) else null


# ---------------------------------------------------------------------------
# The veil.
# ---------------------------------------------------------------------------


## The grammar in force. The veil is drawn by an autoload rather than by a
## screen, so it asks `InformationSurface` directly instead of walking up a
## Control chain it does not have; before that autoload exists (a suite, a
## review scene) the resource as authored is the answer.
func veil_theme() -> Theme:
	if _surface == null:
		_surface = get_node_or_null("/root/InformationSurface")
	if _surface == null:
		return ThemeTokensScript.base_theme()
	return _surface.current_theme()


func build_veil() -> void:
	_veil_layer = CanvasLayer.new()
	_veil_layer.name = "TransitionVeil"
	# Above every screen and above the pause menu, which is a root child too.
	_veil_layer.layer = 128
	add_child(_veil_layer)
	_veil = ColorRect.new()
	_veil.name = "Veil"
	_veil.color = Color(ThemeTokensScript.color(veil_theme(), VEIL_TOKEN), 0.0)
	_veil.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_veil.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_veil_layer.add_child(_veil)
	_veil_label = Label.new()
	_veil_label.name = "VeilLabel"
	_veil_label.text = "MAKING LANDFALL"
	_veil_label.theme = veil_theme()
	_veil_label.theme_type_variation = VEIL_TEXT_VARIATION
	_veil_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_veil_label.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	_veil_label.offset_left = -72.0
	_veil_label.offset_top = -80.0
	_veil_label.offset_right = -72.0
	_veil_label.offset_bottom = -48.0
	# not a colour: the label's own tint is opacity only -- the Theme decides
	# what colour it is drawn in.
	_veil_label.modulate = Color(1, 1, 1, 0)
	_veil_layer.add_child(_veil_label)


func set_veil_alpha(alpha: float) -> void:
	if _veil == null:
		return
	var built := veil_theme()
	_veil.color = Color(ThemeTokensScript.color(built, VEIL_TOKEN), alpha)
	_veil.visible = alpha > 0.0
	if _veil_label != null:
		_veil_label.theme = built
		# not a colour: opacity only, as above.
		_veil_label.modulate = Color(1, 1, 1, alpha)
		_veil_label.visible = alpha > 0.0


func veil_alpha() -> float:
	return 0.0 if _veil == null else _veil.color.a


## Reduced motion is read from the file the settings panel writes, through the
## panel's own constants, so there is one name for the setting and one place it
## is stored. A missing file means the default, which is motion on.
static func reduced_motion() -> bool:
	var config := ConfigFile.new()
	if config.load(SettingsPanel.SETTINGS_PATH) != OK:
		return false
	return bool(config.get_value(SettingsPanel.SECTION, "reduced_motion", false))
