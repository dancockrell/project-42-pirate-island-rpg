class_name PaperStage
extends Control

func _draw() -> void:
	var s := size
	draw_rect(Rect2(Vector2.ZERO, s), Color("102522"))
	# Late-afternoon sky, deep jungle ridges, distant ruins and a left-hand
	# processional wall recreate the approved world grammar in controllable paper
	# layers. The road stays open through the middle for readable combat bands.
	draw_colored_polygon(PackedVector2Array([Vector2(0,.08*s.y),Vector2(.22*s.x,.02*s.y),Vector2(.43*s.x,.13*s.y),Vector2(.66*s.x,.03*s.y),Vector2(s.x,.12*s.y),Vector2(s.x,.58*s.y),Vector2(0,.58*s.y)]),Color("274640"))
	draw_colored_polygon(PackedVector2Array([Vector2(0,.31*s.y),Vector2(.19*s.x,.14*s.y),Vector2(.36*s.x,.32*s.y),Vector2(.57*s.x,.11*s.y),Vector2(.75*s.x,.27*s.y),Vector2(s.x,.12*s.y),Vector2(s.x,.68*s.y),Vector2(0,.68*s.y)]),Color("355c4c"))
	draw_colored_polygon(PackedVector2Array([Vector2(0,.53*s.y),Vector2(.12*s.x,.28*s.y),Vector2(.28*s.x,.46*s.y),Vector2(.43*s.x,.26*s.y),Vector2(.64*s.x,.47*s.y),Vector2(.83*s.x,.25*s.y),Vector2(s.x,.43*s.y),Vector2(s.x,.73*s.y),Vector2(0,.73*s.y)]),Color("1d4035"))
	# A ruined elven receiving wall is pushed left, rather than blocking the
	# center. Its cyan inlay marks Bronze Age magic without implying a colony.
	draw_colored_polygon(PackedVector2Array([Vector2(.06*s.x,.16*s.y),Vector2(.25*s.x,.12*s.y),Vector2(.29*s.x,.65*s.y),Vector2(.03*s.x,.69*s.y)]),Color("5e5a42"))
	draw_rect(Rect2(.09*s.x,.21*s.y,.047*s.x,.42*s.y),Color("9b8352"))
	draw_rect(Rect2(.205*s.x,.18*s.y,.042*s.x,.45*s.y),Color("9b8352"))
	draw_line(Vector2(.12*s.x,.26*s.y),Vector2(.23*s.x,.21*s.y),Color("c7a75f"),9)
	draw_line(Vector2(.16*s.x,.30*s.y),Vector2(.16*s.x,.57*s.y),Color("4fc7b4",.56),3)
	draw_line(Vector2(.20*s.x,.27*s.y),Vector2(.20*s.x,.55*s.y),Color("4fc7b4",.46),3)
	# Distant stepped ruins establish a real archaeological horizon.
	for silhouette in [Rect2(.59*s.x,.39*s.y,.045*s.x,.18*s.y),Rect2(.66*s.x,.34*s.y,.036*s.x,.23*s.y),Rect2(.72*s.x,.42*s.y,.051*s.x,.15*s.y)]:
		draw_rect(silhouette,Color("49604a"))
		draw_line(Vector2(silhouette.position.x, silhouette.position.y),Vector2(silhouette.end.x,silhouette.position.y),Color("b69a5b",.55),3)
	# Stone road and terrace cracks establish the shared contact plane.
	draw_colored_polygon(PackedVector2Array([Vector2(.21*s.x,s.y),Vector2(.45*s.x,.57*s.y),Vector2(.62*s.x,.57*s.y),Vector2(.89*s.x,s.y)]),Color("504637"))
	draw_colored_polygon(PackedVector2Array([Vector2(.32*s.x,s.y),Vector2(.49*s.x,.58*s.y),Vector2(.56*s.x,.58*s.y),Vector2(.73*s.x,s.y)]),Color("766043"))
	for y in [.68,.75,.82,.90]:
		draw_line(Vector2(.25*s.x,y*s.y),Vector2(.83*s.x,y*s.y),Color("b8965a",.28),2)
	# Cut-paper foliage closes the edges without covering actor boxes.
	for x in [0.01,0.055,0.11,0.84,0.90,0.97]:
		draw_circle(Vector2(x*s.x,.70*s.y),.14*s.y,Color("15372e"))
		draw_circle(Vector2((x+.018)*s.x,.48*s.y),.10*s.y,Color("256147"))
	draw_line(Vector2(.18*s.x,.77*s.y),Vector2(.83*s.x,.77*s.y),Color("d3a75d",.38),2)
