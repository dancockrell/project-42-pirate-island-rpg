class_name PaperStage
extends Control

## Reception Terrace is a tracked provisional environment plate. The active
## party and enemies remain separate articulated paper rigs. The stage owns no
## combat state; its sole job is to establish the correct shared floor, depth
## and ancient-elven geography at the player camera.

## The one door onto the palette (card P11). The plate declares no colour: the
## wash, the sky scrim and the vignette are all the grammar's darkest ground at
## three opacities, and the painting itself supplies every other colour on it.
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")
const RECEPTION_TERRACE_BACKDROP := preload("res://assets/standins/reception_terrace/reception_terrace_backdrop_v1.png")

## The one owner of the shared standing floor. Every actor's feet, every ground
## effect and every contact shadow is placed from this fraction of the stage's
## height. Before this pass the rigs were sized and positioned independently of
## it and floated roughly a hundred pixels above the painted stone.
const FLOOR_Y_FRACTION := 0.700
## How far the plate is pushed back so the rigs in front of it read. A painted
## background at full contrast competes with the actors standing on it; these
## are the numbers that settle that competition and they live here rather than
## being spread across the screen.
const DEPTH_WASH_ALPHA := 0.20
const VIGNETTE_ALPHA := 0.46
## The horizon band above the terrace is the brightest thing in the plate, and
## the enemy intent line disappeared into it. A soft gradient darkens the top
## sixth so text over the sky stays legible without dimming the painting.
const SKY_SCRIM_HEIGHT_FRACTION := 0.16
const SKY_SCRIM_ALPHA := 0.42

var plate_image: Image
# not a colour: four placeholders until `repaint()` runs.
var _wash := Color.BLACK
var _scrim := Color.BLACK
var _vignette := Color.BLACK
var _fallback_tint := Color.BLACK


func _ready() -> void:
	repaint(ThemeTokensScript.active(self))


## Take the plate's three darks from the Theme. The stage calls this again when
## the grammar is rebuilt, so a high-contrast battle sits on a harder plate.
func repaint(theme: Theme) -> void:
	var night := ThemeTokensScript.color(theme, "night")
	_wash = night.lightened(0.04)
	_scrim = night
	_vignette = night.darkened(0.40)
	_fallback_tint = ThemeTokensScript.color(theme, "bronze")
	queue_redraw()


func _draw() -> void:
	var bounds := Rect2(Vector2.ZERO, size)
	# Use the entire 16:9 environment plate. The generated asset deliberately
	# contains no cast, monsters, UI or text, so it can never contradict live
	# simulation data and can be replaced by a production-painted layer stack.
	draw_texture_rect(RECEPTION_TERRACE_BACKDROP, bounds, false)
	# A restrained theatre wash lets the living UI and paper rigs read without
	# turning the painted background into a dim generic rectangle.
	draw_rect(bounds, Color(_wash, DEPTH_WASH_ALPHA))
	draw_sky_scrim()
	draw_vignette()
	# The floor is not drawn any more. It was a bronze line and two coloured
	# arcs -- layout guides for placing the rigs -- and they were visible in the
	# running game under Betty's feet. The rigs stand on FLOOR_Y_FRACTION now
	# and their contact shadows are what the player sees instead.


## A soft top-down scrim so the sky cannot swallow the labels drawn over it.
func draw_sky_scrim() -> void:
	var band := size.y * SKY_SCRIM_HEIGHT_FRACTION
	var steps := 14
	for step in steps:
		var fraction := float(step) / float(steps)
		var alpha := SKY_SCRIM_ALPHA * (1.0 - fraction) * (1.0 - fraction)
		draw_rect(Rect2(0.0, band * fraction, size.x, band / float(steps) + 1.0), Color(_scrim, alpha))


## A drawn vignette rather than a post-process, because this screen must render
## the same on the compatibility path as on Forward+. Three edge gradients, no
## shader, no environment resource.
func draw_vignette() -> void:
	var depth := size.x * 0.22
	var steps := 16
	for step in steps:
		var fraction := float(step) / float(steps)
		var alpha := VIGNETTE_ALPHA * fraction * fraction * 3.0 / float(steps)
		var inset := depth * (1.0 - fraction)
		draw_rect(Rect2(0.0, 0.0, inset, size.y), Color(_vignette, alpha))
		draw_rect(Rect2(size.x - inset, 0.0, inset, size.y), Color(_vignette, alpha))
		draw_rect(Rect2(0.0, size.y - inset * .5, size.x, inset * .5), Color(_vignette, alpha))


## The plate's own colour at a normalised point on it. This is what bleeds into
## the paper rigs: the light in the painting, sampled from the painting, rather
## than a hex value invented beside it.
func plate_tint(at: Vector2) -> Color:
	if plate_image == null:
		plate_image = RECEPTION_TERRACE_BACKDROP.get_image()
	if plate_image == null or plate_image.get_width() == 0 or plate_image.get_height() == 0:
		return _fallback_tint
	var width := plate_image.get_width()
	var height := plate_image.get_height()
	# A coarse average over a small window, so one dark leaf cannot decide the
	# light a whole figure is lit by.
	# not a colour: an accumulator for the average below.
	var total := Color(0, 0, 0, 0)
	var samples := 0
	for x_step in 5:
		for y_step in 5:
			var x := clampi(int((at.x + (float(x_step) - 2.0) * 0.02) * float(width)), 0, width - 1)
			var y := clampi(int((at.y + (float(y_step) - 2.0) * 0.02) * float(height)), 0, height - 1)
			var pixel := plate_image.get_pixel(x, y)
			total += pixel
			samples += 1
	# not a colour: the painting's own average, read out of the painting.
	return Color(total.r / float(samples), total.g / float(samples), total.b / float(samples), 1.0)


## The shared standing floor, in stage coordinates.
func floor_y() -> float:
	return size.y * FLOOR_Y_FRACTION
