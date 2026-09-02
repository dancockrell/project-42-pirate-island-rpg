class_name PaperDoll
extends Control

## Deliberately artificial development actor. Geometry and anchors are the
## production contract; painted art will replace the drawing, not its API.

var spec: Dictionary = {}
var accent := Color("4fc7b4")
var paper := Color("e8d7b7")
var ink := Color("2a211a")
## An approved visual identity plate is deliberately shipped for this first
## playable slice. It establishes real character blocking while the separate
## paper-part contract below remains the animation replacement target.
const BETTY_IDENTITY_REFERENCE := preload("res://assets/standins/betty_identity_reference.png")
## Anchor dots are available in the inspector/tooltip contract, while the
## running game keeps the silhouette clean. Toggle this locally only while
## laying out a replacement sprite sheet.
var show_anchors := false
var pose_name := "card_ready"
var vfx_id := "presentation.vfx.none"

func set_presentation_cue(cue: Dictionary) -> void:
	pose_name = str(cue.get("pose", "card_ready"))
	vfx_id = str(cue.get("vfxId", "presentation.vfx.none"))
	queue_redraw()

func reset_presentation() -> void:
	pose_name = "card_ready"
	vfx_id = "presentation.vfx.none"
	queue_redraw()

func configure(next_spec: Dictionary, next_accent: Color) -> void:
	spec = next_spec.duplicate(true)
	accent = next_accent
	custom_minimum_size = Vector2(spec.get("nativeCanvas", [520, 560])[0], spec.get("nativeCanvas", [520, 560])[1])
	tooltip_text = metadata_tooltip()
	queue_redraw()

func metadata_tooltip() -> String:
	return "PAPER DOLL — DEVELOPMENT ONLY\nStable presentation: %s\nSubject: %s\nFuture asset: %s\nCanvas: %s\nSafe padding: %s%%\nGround anchor: %s\nRoot pivot: %s\n\nIdentity invariants:\n• %s\n\nReplacement tests:\n• %s" % [spec.get("id", "missing"), spec.get("subjectId", "missing"), spec.get("futureRuntimeAssetId", "missing"), str(spec.get("nativeCanvas", [])), str(spec.get("safePaddingPercent", 0)), str(spec.get("groundAnchor", [])), str(spec.get("rootPivot", [])), "\n• ".join(PackedStringArray(spec.get("identityInvariants", []))), "\n• ".join(PackedStringArray(spec.get("replacementTests", [])))]

func _draw() -> void:
	if spec.is_empty():
		return
	var s := size
	var pad := float(spec.get("safePaddingPercent", 8)) / 100.0
	draw_rect(Rect2(s.x * pad, s.y * pad, s.x * (1.0 - pad * 2.0), s.y * (1.0 - pad * 2.0)), Color(accent, 0.18), false, 2.0)
	var body_offset := Vector2.ZERO
	var body_rotation := 0.0
	if pose_name == "forward_step":
		body_offset = Vector2(s.x * .07, -s.y * .01)
	elif pose_name == "horizontal_mace_hit":
		body_offset = Vector2(s.x * .13, -s.y * .035)
		body_rotation = deg_to_rad(-7.0)
	elif pose_name == "mace_cross_body":
		body_rotation = deg_to_rad(4.0)
	# Battle actors occupy a shared interaction field, not isolated poster slots.
	# At sixty percent of the safe canvas each fighter has room to step, swing,
	# be displaced, receive VFX and share the plane with later party members.
	# This scale is intentionally common to heroine and monster blockouts so
	# contact ranges can be tuned before final art replaces either silhouette.
	var interaction_scale := Vector2(.60, .60)
	var interaction_origin := Vector2(s.x * .20, s.y * .18)
	draw_set_transform(interaction_origin + body_offset, body_rotation, interaction_scale)
	if spec.get("dollKind", "") == "wild_raptor":
		draw_raptor(s)
	else:
		draw_betty(s)
	draw_set_transform(Vector2.ZERO, 0.0)
	draw_presentation_vfx(s)
	# The complete asset contract lives in the tooltip. The scene itself keeps a
	# quiet, player-readable cut-paper silhouette.
	if show_anchors:
		for key in spec.get("attachmentAnchors", {}):
			var point: Array = spec.attachmentAnchors[key]
			var p := Vector2(float(point[0]) * s.x, float(point[1]) * s.y)
			draw_circle(p, 4.0, Color("ffd166"))
			draw_string(ThemeDB.fallback_font, p + Vector2(7, -5), str(key), HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color("ffe6a7"))

func draw_presentation_vfx(s: Vector2) -> void:
	if vfx_id == "presentation.vfx.bronze_teal_impact_arc":
		var impact := Vector2(s.x * .84, s.y * .34)
		draw_arc(impact, s.x*.12, deg_to_rad(205), deg_to_rad(345), 16, Color("e4b75e"), 7, true)
		draw_arc(impact, s.x*.095, deg_to_rad(210), deg_to_rad(340), 16, Color("4fc7b4"), 3, true)
		for point in [Vector2(.76,.27),Vector2(.88,.24),Vector2(.93,.37),Vector2(.80,.45)]: draw_circle(Vector2(point.x*s.x,point.y*s.y),4,Color("fff1cc"))
	elif vfx_id == "presentation.vfx.teal_guard_ring":
		var center := Vector2(s.x*.50,s.y*.61)
		draw_arc(center,s.x*.18,0,TAU,36,Color("4fc7b4"),4,true)
		draw_arc(center,s.x*.145,0,TAU,36,Color("d9bd72"),2,true)
	elif vfx_id == "presentation.vfx.teal_card_release" or vfx_id == "presentation.vfx.card_recall_trail":
		for x in [.20,.28,.36,.64,.72,.80]: draw_line(Vector2(x*s.x,s.y*.80),Vector2((x+.04)*s.x,s.y*.34),Color("4fc7b4",.7),3)

func draw_betty(s: Vector2) -> void:
	# Use the approved Betty identity plate for the player-facing prototype. This
	# is not final runtime animation art: the metadata identifies it as a visual
	# stand-in and the paper-part layout below supplies the future rig contract.
	# The source crop contains her complete head, boots, satchel and mace, with
	# safe breathing room on every edge so it blocks the actual actor footprint.
	var source := Rect2(430, 0, 470, 720)
	var destination := Rect2(s.x * .19, s.y * .055, s.x * .62, s.y * .89)
	draw_texture_rect_region(BETTY_IDENTITY_REFERENCE, destination, source, Color.WHITE)
	if show_anchors:
		draw_betty_paper_rig(s)

func draw_betty_paper_rig(s: Vector2) -> void:
	var shadow := Vector2(s.x * 0.51, s.y * 0.89)
	draw_paper_ellipse(shadow, Vector2(s.x * 0.18, s.y * 0.026), Color(0, 0, 0, 0.35))
	# Betty is a close-to-final paper-rig demo, not a simplified mascot. Her
	# silhouette uses an adult seven-head fashion proportion: compact head,
	# narrow corseted torso, long legs and tall boots. Each visible overlap is a
	# separate replacement/animation piece with a stable joint underneath.
	# Rear coat tails frame the body without hiding the short field-shorts read.
	draw_colored_polygon(PackedVector2Array([Vector2(.39*s.x,.38*s.y),Vector2(.62*s.x,.39*s.y),Vector2(.69*s.x,.68*s.y),Vector2(.57*s.x,.65*s.y),Vector2(.52*s.x,.72*s.y),Vector2(.42*s.x,.65*s.y),Vector2(.33*s.x,.69*s.y)]), Color("174f4d"))
	# Long, light legs remain fully within the documented safe frame.
	draw_line(Vector2(.46*s.x,.60*s.y), Vector2(.43*s.x,.78*s.y), Color("f1c9ad"), 14)
	draw_line(Vector2(.55*s.x,.60*s.y), Vector2(.60*s.x,.78*s.y), Color("f1c9ad"), 14)
	draw_line(Vector2(.43*s.x,.76*s.y), Vector2(.40*s.x,.88*s.y), Color("3a2928"), 20)
	draw_line(Vector2(.60*s.x,.76*s.y), Vector2(.64*s.x,.88*s.y), Color("3a2928"), 20)
	draw_line(Vector2(.38*s.x,.89*s.y), Vector2(.45*s.x,.89*s.y), Color("1a1518"), 9)
	draw_line(Vector2(.63*s.x,.89*s.y), Vector2(.70*s.x,.89*s.y), Color("1a1518"), 9)
	# Short tailored field shorts plus a slim teal corset make the practical,
	# flirtatious medic costume readable before any final painting is installed.
	draw_colored_polygon(PackedVector2Array([Vector2(.41*s.x,.54*s.y),Vector2(.59*s.x,.54*s.y),Vector2(.61*s.x,.62*s.y),Vector2(.53*s.x,.64*s.y),Vector2(.48*s.x,.63*s.y),Vector2(.40*s.x,.61*s.y)]), Color("123d40"))
	draw_colored_polygon(PackedVector2Array([Vector2(.43*s.x,.38*s.y),Vector2(.57*s.x,.38*s.y),Vector2(.59*s.x,.54*s.y),Vector2(.51*s.x,.57*s.y),Vector2(.41*s.x,.54*s.y)]), Color("237e76"))
	draw_colored_polygon(PackedVector2Array([Vector2(.42*s.x,.32*s.y),Vector2(.58*s.x,.32*s.y),Vector2(.57*s.x,.42*s.y),Vector2(.53*s.x,.45*s.y),Vector2(.47*s.x,.45*s.y),Vector2(.42*s.x,.42*s.y)]), Color("f1e6cf"))
	draw_line(Vector2(.50*s.x,.33*s.y), Vector2(.50*s.x,.43*s.y), Color("b58d4b"), 2)
	draw_limb(Vector2(.42*s.x,.37*s.y), Vector2(.32*s.x,.45*s.y), Vector2(.29*s.x,.55*s.y), 12, Color("f1c9ad"))
	draw_limb(Vector2(.58*s.x,.37*s.y), Vector2(.66*s.x,.45*s.y), Vector2(.70*s.x,.55*s.y), 12, Color("f1c9ad"))
	# Layered auburn hair and a small composed anime face. Eyes are deliberately
	# compact at battle scale rather than the oversized childlike face used by a
	# generic placeholder.
	draw_circle(Vector2(.50*s.x,.20*s.y), s.x*.061, Color("8e3d2e"))
	draw_circle(Vector2(.47*s.x,.16*s.y), s.x*.024, Color("9f452e"))
	draw_circle(Vector2(.53*s.x,.15*s.y), s.x*.026, Color("a94f32"))
	draw_paper_ellipse(Vector2(.50*s.x,.205*s.y), Vector2(s.x*.043, s.y*.064), Color("f1c9ad"))
	draw_arc(Vector2(.50*s.x,.195*s.y),s.x*.055,deg_to_rad(205),deg_to_rad(340),14,Color("8e3d2e"),8,true)
	draw_line(Vector2(.474*s.x,.207*s.y), Vector2(.484*s.x,.207*s.y), Color("244a45"), 2)
	draw_line(Vector2(.516*s.x,.207*s.y), Vector2(.526*s.x,.207*s.y), Color("244a45"), 2)
	draw_circle(Vector2(.479*s.x,.208*s.y), 2.1, Color("52bda7"))
	draw_circle(Vector2(.521*s.x,.208*s.y), 2.1, Color("52bda7"))
	draw_line(Vector2(.487*s.x,.242*s.y), Vector2(.513*s.x,.242*s.y), Color("a94f50"), 1.7)
	for x in [.47,.48,.52,.53]: draw_circle(Vector2(x*s.x,.226*s.y),1.4,Color("c36f53"))
	# The bronze-and-glass boarding mace keeps a clear silhouette without
	# extending past the safe box. The shaft is an independent rig layer.
	draw_line(Vector2(.70*s.x,.55*s.y), Vector2(.80*s.x,.28*s.y), Color("76512e"), 8)
	draw_circle(Vector2(.81*s.x,.255*s.y), s.x*.041, Color("b78948"))
	draw_circle(Vector2(.81*s.x,.255*s.y), s.x*.025, Color("5bcabe"))
	draw_arc(Vector2(.81*s.x,.255*s.y),s.x*.045,0,TAU,16,Color("ddbb72"),2,true)
	draw_pins([Vector2(.42*s.x,.37*s.y),Vector2(.58*s.x,.37*s.y),Vector2(.45*s.x,.57*s.y),Vector2(.56*s.x,.57*s.y)])

func draw_raptor(s: Vector2) -> void:
	var hide := Color("a74636")
	draw_paper_ellipse(Vector2(.52*s.x,.86*s.y), Vector2(.27*s.x,.035*s.y), Color(0,0,0,.35))
	draw_colored_polygon(PackedVector2Array([Vector2(.55*s.x,.43*s.y),Vector2(.92*s.x,.39*s.y),Vector2(.72*s.x,.53*s.y)]), Color("78362f"))
	draw_paper_ellipse(Vector2(.53*s.x,.46*s.y), Vector2(.22*s.x,.12*s.y), hide)
	draw_limb(Vector2(.46*s.x,.52*s.y),Vector2(.40*s.x,.70*s.y),Vector2(.38*s.x,.85*s.y),20,Color("9d503c"))
	draw_limb(Vector2(.60*s.x,.52*s.y),Vector2(.65*s.x,.70*s.y),Vector2(.68*s.x,.86*s.y),20,Color("9d503c"))
	draw_line(Vector2(.48*s.x,.43*s.y),Vector2(.34*s.x,.31*s.y),hide,42)
	draw_colored_polygon(PackedVector2Array([Vector2(.17*s.x,.28*s.y),Vector2(.35*s.x,.25*s.y),Vector2(.39*s.x,.34*s.y),Vector2(.22*s.x,.38*s.y)]), hide)
	draw_colored_polygon(PackedVector2Array([Vector2(.17*s.x,.34*s.y),Vector2(.32*s.x,.33*s.y),Vector2(.22*s.x,.40*s.y)]), Color("d4b071"))
	draw_circle(Vector2(.26*s.x,.29*s.y),5,Color("ffd166"))
	draw_limb(Vector2(.47*s.x,.44*s.y),Vector2(.40*s.x,.52*s.y),Vector2(.35*s.x,.56*s.y),10,Color("8b4135"))
	draw_limb(Vector2(.55*s.x,.43*s.y),Vector2(.61*s.x,.51*s.y),Vector2(.66*s.x,.54*s.y),10,Color("8b4135"))
	draw_line(Vector2(.34*s.x,.85*s.y),Vector2(.43*s.x,.85*s.y),Color("d4b071"),7)
	draw_line(Vector2(.64*s.x,.86*s.y),Vector2(.73*s.x,.86*s.y),Color("d4b071"),7)
	draw_pins([Vector2(.46*s.x,.52*s.y),Vector2(.60*s.x,.52*s.y),Vector2(.47*s.x,.44*s.y),Vector2(.55*s.x,.43*s.y),Vector2(.34*s.x,.31*s.y)])

func draw_limb(a: Vector2, joint: Vector2, b: Vector2, width: float, color: Color) -> void:
	draw_line(a, joint, color, width, true)
	draw_line(joint, b, color.darkened(.08), width*.82, true)
	draw_circle(joint, 6, Color("c79a55"))
	draw_circle(joint, 2, ink)

func draw_pins(points: Array[Vector2]) -> void:
	for point in points:
		draw_circle(point, 7, Color("d0a85f"))
		draw_circle(point, 2, ink)

func draw_paper_ellipse(center: Vector2, radii: Vector2, color: Color) -> void:
	var points := PackedVector2Array()
	for index in range(32):
		var angle := TAU * float(index) / 32.0
		points.append(center + Vector2(cos(angle)*radii.x, sin(angle)*radii.y))
	draw_colored_polygon(points, color)
