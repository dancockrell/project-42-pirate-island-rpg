class_name BattleCameraRig
extends Node

## The battle screen's camera, driven from `content/presentation/camera.registry.json`.
##
## The screen is a Control tree over a painted plate, so the "camera" is a
## transform on the stage: `zoom` scales it about the beat's lead point and
## `transitionInMs` / `transitionOutMs` are the push-in and the settle. Nothing
## here reads a skill; it reads the camera record the cue names, which is the
## record's own business.
##
## Three beats, in the order a fight uses them:
##   push(record, lead)  the transitionIn -- the frame moves onto the action
##   hold()              the frame stays where the record put it
##   settle()            the transitionOut -- the frame returns to the wide
##
## Plus the two things a hit needs: a named short freeze on damage, and a shake
## whose amplitude is a named constant and which is off entirely under reduced
## motion.

signal beat_started(camera_id: String, mode: String, zoom: float)
signal settled()

## The frame the fight lives in when no beat owns it. `battle_wide_reset` in the
## registry is authored at 0.92, so the rest frame is 1.0 and a reset reads as
## pulling back.
const REST_ZOOM := 1.0
## A hit-stop is a freeze, not a slow-motion: the frame stops dead for this long
## and then the action continues from exactly where it was. Eighty milliseconds
## is what the authored cues already ask for (`hitStopMs: 80` on Betty's impact
## beat); this is the ceiling a cue may request, so a bad record cannot stall
## the fight.
const MAXIMUM_HIT_STOP_MS := 140
## The shake's whole amplitude, in stage pixels at rest zoom. A cue's `shake` is
## a fraction of it, and a fight never moves the frame further than this.
const SHAKE_AMPLITUDE_PX := 7.0
## Three decaying swings. Deterministic: a sine on alternating axes, no RNG.
const SHAKE_SWINGS := 3
const SHAKE_DURATION := 0.16
## A record's transition is in milliseconds and may be zero (`battle_wide_reset`
## has `transitionOutMs: 0`); a zero-length tween is illegal, so this is the
## floor every transition is clamped to.
const MINIMUM_TRANSITION := 0.02

var stage: Control
var reduced_motion := false
var zoom := REST_ZOOM
var frozen := false
var camera_tween: Tween
var shake_tween: Tween
var shake_offset := Vector2.ZERO
var lead_point := Vector2.ZERO
var current_camera_id := ""


func configure(target_stage: Control, motion_reduced: bool) -> void:
	stage = target_stage
	reduced_motion = motion_reduced
	if stage != null:
		stage.pivot_offset = stage.size * .5
		lead_point = stage.pivot_offset
	apply_transform()


## The push-in. `lead` is the stage point the record's `lead` subject resolves
## to; the frame scales about it, so a punch-in on a contact point puts the
## contact in the middle of the screen rather than the stage's geometric centre.
func push(record: Dictionary, lead: Vector2) -> void:
	if stage == null:
		return
	current_camera_id = str(record.get("id", ""))
	var mode := str(record.get("mode", "static"))
	var target_zoom := clampf(float(record.get("zoom", REST_ZOOM)), 0.8, 1.25)
	beat_started.emit(current_camera_id, mode, target_zoom)
	lead_point = lead
	stage.pivot_offset = lead
	if reduced_motion:
		# Reduced motion keeps the framing decision -- a punch-in still puts the
		# contact in the middle -- and drops the travel: the frame cuts.
		zoom = target_zoom
		apply_transform()
		return
	var duration := transition_seconds(record, "transitionInMs")
	# A `punch` or `snap` arrives hard and a `push` or `pan` arrives soft. The
	# record's own mode chooses the curve; the screen does not choose per skill.
	var trans := Tween.TRANS_QUAD
	var ease := Tween.EASE_OUT
	match mode:
		"punch", "snap":
			trans = Tween.TRANS_EXPO
		"push", "pan":
			trans = Tween.TRANS_SINE
			ease = Tween.EASE_IN_OUT
		"reset":
			trans = Tween.TRANS_CUBIC
	start_camera_tween(target_zoom, duration, trans, ease)


## The hold. A beat that has arrived stays until the next one asks for the
## frame; this exists so the sequence reads push, hold, settle rather than a
## continuous drift, and so a caller can say plainly that it is holding.
func hold() -> void:
	if camera_tween != null:
		camera_tween.kill()
		camera_tween = null


## The settle: back to the rest frame over the record's own transitionOut.
func settle(record: Dictionary = {}) -> void:
	if stage == null:
		return
	current_camera_id = ""
	if reduced_motion:
		zoom = REST_ZOOM
		shake_offset = Vector2.ZERO
		apply_transform()
		settled.emit()
		return
	var duration := transition_seconds(record, "transitionOutMs")
	start_camera_tween(REST_ZOOM, duration, Tween.TRANS_SINE, Tween.EASE_IN_OUT)
	if camera_tween != null:
		camera_tween.finished.connect(func() -> void: settled.emit(), CONNECT_ONE_SHOT)
	else:
		settled.emit()


## The named short freeze on damage. The frame stops for the cue's own
## `hitStopMs`, clamped, and the caller's beat resumes afterwards. It is a
## freeze of the presentation only: no simulation event is delayed or reordered
## by it, because the events are already resolved before any of this plays.
func hit_stop(milliseconds: int, tree: SceneTree) -> void:
	var held := clampi(milliseconds, 0, MAXIMUM_HIT_STOP_MS)
	if held <= 0 or tree == null or reduced_motion:
		return
	frozen = true
	hold()
	await tree.create_timer(float(held) / 1000.0).timeout
	frozen = false


## A restrained shake. `strength` is the cue's own `shake`, 0..1; the amplitude
## is SHAKE_AMPLITUDE_PX and nothing may exceed it. Off entirely under reduced
## motion, which is the accessibility contract, not a preference.
func shake(strength: float) -> void:
	if stage == null or reduced_motion or strength <= 0.0:
		return
	if shake_tween != null:
		shake_tween.kill()
	var amplitude := SHAKE_AMPLITUDE_PX * clampf(strength, 0.0, 1.0)
	shake_tween = stage.create_tween()
	for swing in SHAKE_SWINGS:
		# Alternating axes, decaying by swing. Deterministic by construction:
		# the sequence is the same every time a given strength is asked for.
		var decay := amplitude * (1.0 - float(swing) / float(SHAKE_SWINGS))
		var offset := Vector2(decay, -decay * .5) if swing % 2 == 0 else Vector2(-decay * .7, decay * .4)
		shake_tween.tween_method(set_shake_offset, shake_offset, offset, SHAKE_DURATION / float(SHAKE_SWINGS + 1))
	shake_tween.tween_method(set_shake_offset, shake_offset, Vector2.ZERO, SHAKE_DURATION / float(SHAKE_SWINGS + 1))


func set_shake_offset(offset: Vector2) -> void:
	shake_offset = offset
	apply_transform()


func set_zoom(value: float) -> void:
	zoom = value
	apply_transform()


func apply_transform() -> void:
	if stage == null:
		return
	stage.scale = Vector2.ONE * zoom
	# The lead point stays put while the stage scales about it, so a push-in
	# never lets the plate's edge into the safe frame.
	stage.position = shake_offset


func transition_seconds(record: Dictionary, key: String) -> float:
	return maxf(MINIMUM_TRANSITION, float(record.get(key, 100)) / 1000.0)


func start_camera_tween(target_zoom: float, duration: float, trans: Tween.TransitionType, ease: Tween.EaseType) -> void:
	if camera_tween != null:
		camera_tween.kill()
	camera_tween = stage.create_tween()
	camera_tween.set_trans(trans).set_ease(ease)
	camera_tween.tween_method(set_zoom, zoom, target_zoom, duration)
