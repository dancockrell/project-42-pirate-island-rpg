class_name PaperDoll
extends Control

## Deliberately artificial development actor. Geometry and anchors are the
## production contract; painted art will replace the drawing, not its API.

var spec: Dictionary = {}
var accent := Color("4fc7b4")
var paper := Color("e8d7b7")
var ink := Color("2a211a")
var show_anchors := true

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
	if spec.get("dollKind", "") == "wild_raptor":
		draw_raptor(s)
	else:
		draw_betty(s)
	draw_string(ThemeDB.fallback_font, Vector2(18, s.y - 16), "PAPER ACTOR • %s" % spec.get("id", "missing"), HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color("caa66a"))
	if show_anchors:
		for key in spec.get("attachmentAnchors", {}):
			var point: Array = spec.attachmentAnchors[key]
			var p := Vector2(float(point[0]) * s.x, float(point[1]) * s.y)
			draw_circle(p, 4.0, Color("ffd166"))
			draw_string(ThemeDB.fallback_font, p + Vector2(7, -5), str(key), HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color("ffe6a7"))

func draw_betty(s: Vector2) -> void:
	var shadow := Vector2(s.x * 0.52, s.y * 0.88)
	draw_paper_ellipse(shadow, Vector2(s.x * 0.20, s.y * 0.035), Color(0, 0, 0, 0.35))
	# Rear coat and legs are separate pieces with visible brass paper fasteners.
	draw_colored_polygon(PackedVector2Array([Vector2(.40*s.x,.48*s.y),Vector2(.62*s.x,.48*s.y),Vector2(.70*s.x,.78*s.y),Vector2(.52*s.x,.68*s.y),Vector2(.33*s.x,.80*s.y)]), Color("1f625d"))
	draw_limb(Vector2(.46*s.x,.55*s.y), Vector2(.43*s.x,.76*s.y), Vector2(.39*s.x,.88*s.y), 24, Color("d8c29e"))
	draw_limb(Vector2(.56*s.x,.55*s.y), Vector2(.61*s.x,.75*s.y), Vector2(.66*s.x,.88*s.y), 24, Color("d8c29e"))
	draw_colored_polygon(PackedVector2Array([Vector2(.41*s.x,.31*s.y),Vector2(.58*s.x,.31*s.y),Vector2(.63*s.x,.57*s.y),Vector2(.39*s.x,.57*s.y)]), Color("24766f"))
	draw_limb(Vector2(.42*s.x,.36*s.y), Vector2(.31*s.x,.48*s.y), Vector2(.27*s.x,.61*s.y), 18, paper)
	draw_limb(Vector2(.58*s.x,.36*s.y), Vector2(.70*s.x,.46*s.y), Vector2(.73*s.x,.59*s.y), 18, paper)
	draw_circle(Vector2(.50*s.x,.20*s.y), s.x*.075, paper)
	draw_circle(Vector2(.48*s.x,.17*s.y), s.x*.083, Color("a94f32"))
	draw_circle(Vector2(.50*s.x,.20*s.y), s.x*.061, paper)
	# Oversized boarding mace gives the paper silhouette its final equipment envelope.
	draw_line(Vector2(.72*s.x,.58*s.y), Vector2(.83*s.x,.23*s.y), Color("7c5730"), 12)
	draw_circle(Vector2(.84*s.x,.21*s.y), s.x*.055, Color("b78948"))
	draw_circle(Vector2(.84*s.x,.21*s.y), s.x*.035, Color("69d7ca"))
	draw_line(Vector2(.40*s.x,.31*s.y), Vector2(.60*s.x,.31*s.y), ink, 3)
	draw_pins([Vector2(.42*s.x,.36*s.y),Vector2(.58*s.x,.36*s.y),Vector2(.46*s.x,.55*s.y),Vector2(.56*s.x,.55*s.y)])

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
