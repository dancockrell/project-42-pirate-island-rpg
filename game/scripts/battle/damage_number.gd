class_name DamageNumber
extends Node2D

## A number with weight. It scales in past its final size, drifts, and fades.
##
## The variants are the ones the bridge's own event payloads distinguish, not
## an invented critical-hit table: this simulation has no crit. What it has is
## damage that reached Vitality (`damage_applied`), damage a Guard pool absorbed
## (`guard_changed` with a negative delta), healing (`vitality_changed` with a
## positive delta) and the punish for striking into a recovery opening
## (`recovery_opening_consumed`, carrying `bonus_raw_damage`) -- which is this
## game's heavy hit and reads as one.

const PaletteScript = preload("res://scripts/battle/battle_palette.gd")

const DAMAGE := "damage"
const GUARD := "guard"
const HEAL := "heal"
const PUNISH := "punish"

## Presentation numbers, named so the weight can be tuned in one place.
const RISE_PX := 74.0
const DRIFT_PX := 26.0
const POP_SCALE := 1.34
const POP_SECONDS := 0.09
const SETTLE_SECONDS := 0.10
const HOLD_SECONDS := 0.30
const FADE_SECONDS := 0.34
## Reduced motion keeps the number and drops the travel: it appears, holds a
## little longer so it can still be read, and fades in place.
const REDUCED_HOLD_SECONDS := 0.70

var label: Label
var variant := DAMAGE


static func font_size_for(variant_name: String) -> int:
	match variant_name:
		PUNISH: return 52
		GUARD: return 26
		HEAL: return 34
		_: return 40


static func color_for(variant_name: String) -> Color:
	match variant_name:
		PUNISH: return PaletteScript.GOLD
		GUARD: return PaletteScript.BRONZE
		HEAL: return PaletteScript.TEAL
		_: return PaletteScript.DANGER


static func text_for(variant_name: String, amount: int) -> String:
	match variant_name:
		GUARD: return "GUARD %d" % amount
		HEAL: return "+%d" % amount
		PUNISH: return "%d!" % amount
		_: return str(amount)


func configure(next_variant: String, amount: int) -> void:
	variant = next_variant
	label = Label.new()
	label.name = "Value"
	label.text = text_for(variant, absi(amount))
	label.add_theme_font_size_override("font_size", font_size_for(variant))
	label.add_theme_color_override("font_color", color_for(variant))
	# A dark outline is what keeps a number legible over a bright painted plate
	# as well as over the dark stone; the baseline screen had none and lost its
	# top-right label into the sky.
	label.add_theme_color_override("font_outline_color", PaletteScript.VOID)
	label.add_theme_constant_override("outline_size", 10)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.size = Vector2(200, 56)
	label.position = Vector2(-100, -28)
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(label)
	scale = Vector2.ZERO


## Plays the whole life of the number and frees itself at the end. `drift` is
## the direction it travels -- away from the blow, which the stage knows and
## this node does not.
func play(drift: Vector2, reduced_motion: bool) -> void:
	if reduced_motion:
		scale = Vector2.ONE
		await get_tree().create_timer(REDUCED_HOLD_SECONDS).timeout
		var fade := create_tween()
		fade.tween_property(self, "modulate:a", 0.0, FADE_SECONDS)
		await fade.finished
		queue_free()
		return
	var travel := drift.normalized() * DRIFT_PX + Vector2.UP * RISE_PX
	var destination := position + travel
	var pop := create_tween()
	pop.set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	pop.tween_property(self, "scale", Vector2.ONE * POP_SCALE, POP_SECONDS)
	pop.tween_property(self, "scale", Vector2.ONE, SETTLE_SECONDS)
	var rise := create_tween()
	rise.set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_OUT)
	rise.tween_property(self, "position", destination, POP_SECONDS + SETTLE_SECONDS + HOLD_SECONDS + FADE_SECONDS)
	await get_tree().create_timer(POP_SECONDS + SETTLE_SECONDS + HOLD_SECONDS).timeout
	if not is_inside_tree():
		return
	var fade_out := create_tween()
	fade_out.tween_property(self, "modulate:a", 0.0, FADE_SECONDS)
	await fade_out.finished
	queue_free()
