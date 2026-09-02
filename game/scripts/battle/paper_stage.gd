class_name PaperStage
extends Control

func _draw() -> void:
	var s := size
	draw_rect(Rect2(Vector2.ZERO, s), Color("18312d"))
	draw_colored_polygon(PackedVector2Array([Vector2(0,.34*s.y),Vector2(.18*s.x,.14*s.y),Vector2(.34*s.x,.31*s.y),Vector2(.53*s.x,.10*s.y),Vector2(.72*s.x,.29*s.y),Vector2(s.x,.12*s.y),Vector2(s.x,.68*s.y),Vector2(0,.68*s.y)]),Color("29493d"))
	# Magical Bronze Age elven gate: ancient, monumental, partially swallowed.
	draw_rect(Rect2(.45*s.x,.17*s.y,.18*s.x,.42*s.y),Color("816b45"))
	draw_rect(Rect2(.49*s.x,.25*s.y,.10*s.x,.34*s.y),Color("142823"))
	draw_circle(Vector2(.54*s.x,.24*s.y),.07*s.x,Color("816b45"),false,14)
	draw_colored_polygon(PackedVector2Array([Vector2(.39*s.x,s.y),Vector2(.49*s.x,.56*s.y),Vector2(.59*s.x,.56*s.y),Vector2(.72*s.x,s.y)]),Color("725c3b"))
	for x in [0.04,0.14,0.77,0.88,0.96]:
		draw_circle(Vector2(x*s.x,.49*s.y),.15*s.y,Color("21513f"))
		draw_circle(Vector2((x+.03)*s.x,.36*s.y),.10*s.y,Color("32705a"))
	draw_string(ThemeDB.fallback_font,Vector2(18,28),"DUMMY PAPER STAGE • RECEPTION ROAD • art.environment.reception_road.battle.afternoon",HORIZONTAL_ALIGNMENT_LEFT,-1,12,Color("e4c487"))
