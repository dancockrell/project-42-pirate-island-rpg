class_name VellumPipRow
extends Control

## A count the player can read at a glance: pips, never a percentage.
##
## `paper_card.gd` already established the rule this component generalises --
## ten Composure pips, one per point, "a count, not a bar and not a percentage:
## the player must be able to see how many are left". Vitality and Guard are
## drawn the same way in the grammar, in the `bar` shape, because a long count
## reads better as a row of ticks than as a row of dots.

enum Shape { DOT, BAR }

## How many pips there are in total.
@export var total := 10:
	set(value):
		total = maxi(0, value)
		_resize()

## How many of them are filled.
@export var filled := 10:
	set(value):
		filled = value
		queue_redraw()

## The palette token a filled pip is drawn in.
@export var fill_token := "teal":
	set(value):
		fill_token = value
		queue_redraw()

## The palette token an empty pip's outline is drawn in.
@export var empty_token := "rule_faint":
	set(value):
		empty_token = value
		queue_redraw()

@export var shape: Shape = Shape.DOT:
	set(value):
		shape = value
		_resize()

const DOT_RADIUS := 3.5
const DOT_PITCH := 11.0
const BAR_HEIGHT := 10.0
const BAR_PITCH := 7.0
## The tightest a row may pack before it stops being countable. A count of
## twenty-four in a card two hundred pixels wide is why this exists: the pips
## close up rather than running off the edge of the card.
const MINIMUM_PITCH := 3.0


func configure(next_total: int, next_filled: int, next_fill_token: String, next_shape: Shape = Shape.DOT) -> void:
	total = next_total
	filled = next_filled
	fill_token = next_fill_token
	shape = next_shape


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED or what == NOTIFICATION_RESIZED:
		queue_redraw()


func _resize() -> void:
	var height := DOT_RADIUS * 2.0 if shape == Shape.DOT else BAR_HEIGHT
	custom_minimum_size = Vector2(maxi(total, 1) * MINIMUM_PITCH, height)
	queue_redraw()


## How far apart the pips actually sit: the comfortable pitch when there is
## room, tighter when the row is narrower than the count wants.
func pitch() -> float:
	var comfortable := DOT_PITCH if shape == Shape.DOT else BAR_PITCH
	if total <= 0:
		return comfortable
	return clampf(size.x / float(total), MINIMUM_PITCH, comfortable)


func _draw() -> void:
	var active := ThemeTokens.active(self)
	var lit := ThemeTokens.color(active, fill_token)
	var unlit := ThemeTokens.color(active, empty_token)
	var shown := clampi(filled, 0, total)
	var step := pitch()
	for index in total:
		var is_lit := index < shown
		if shape == Shape.DOT:
			var radius: float = minf(DOT_RADIUS, step * 0.42)
			var centre := Vector2(step * 0.5 + index * step, size.y * 0.5)
			if is_lit:
				draw_circle(centre, radius, lit)
			else:
				draw_arc(centre, radius, 0.0, TAU, 14, unlit, 1.0, true)
		else:
			var rect := Rect2(index * step, (size.y - BAR_HEIGHT) * 0.5,
				maxf(2.0, step - 2.0), BAR_HEIGHT)
			if is_lit:
				draw_rect(rect, lit)
			else:
				draw_rect(rect, unlit, false, 1.0)
