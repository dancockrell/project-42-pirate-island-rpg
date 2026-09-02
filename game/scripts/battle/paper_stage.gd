class_name PaperStage
extends Control

func _draw() -> void:
	var s := size
	draw_rect(Rect2(Vector2.ZERO, s), Color("102522"))
	# Paper sky and three distant jungle silhouettes establish depth before actors enter.
	draw_colored_polygon(PackedVector2Array([Vector2(0,.10*s.y),Vector2(.25*s.x,.03*s.y),Vector2(.48*s.x,.14*s.y),Vector2(.72*s.x,.04*s.y),Vector2(s.x,.15*s.y),Vector2(s.x,.63*s.y),Vector2(0,.63*s.y)]),Color("2e5148"))
	draw_colored_polygon(PackedVector2Array([Vector2(0,.37*s.y),Vector2(.18*s.x,.16*s.y),Vector2(.34*s.x,.34*s.y),Vector2(.53*s.x,.12*s.y),Vector2(.72*s.x,.31*s.y),Vector2(s.x,.13*s.y),Vector2(s.x,.70*s.y),Vector2(0,.70*s.y)]),Color("29493d"))
	draw_colored_polygon(PackedVector2Array([Vector2(0,.52*s.y),Vector2(.15*s.x,.30*s.y),Vector2(.28*s.x,.49*s.y),Vector2(.44*s.x,.32*s.y),Vector2(.65*s.x,.50*s.y),Vector2(.82*s.x,.29*s.y),Vector2(s.x,.46*s.y),Vector2(s.x,.76*s.y),Vector2(0,.76*s.y)]),Color("1d3d35"))
	# Magical Bronze Age elven gate: ancient, monumental, partially swallowed.
	draw_rect(Rect2(.45*s.x,.17*s.y,.18*s.x,.42*s.y),Color("816b45"))
	draw_rect(Rect2(.465*s.x,.19*s.y,.035*s.x,.40*s.y),Color("aa8b50"))
	draw_rect(Rect2(.58*s.x,.19*s.y,.035*s.x,.40*s.y),Color("aa8b50"))
	draw_rect(Rect2(.49*s.x,.25*s.y,.10*s.x,.34*s.y),Color("142823"))
	draw_circle(Vector2(.54*s.x,.24*s.y),.07*s.x,Color("b79859"),false,14)
	draw_circle(Vector2(.54*s.x,.24*s.y),.045*s.x,Color("4fc7b4",.28),false,4)
	draw_colored_polygon(PackedVector2Array([Vector2(.39*s.x,s.y),Vector2(.49*s.x,.56*s.y),Vector2(.59*s.x,.56*s.y),Vector2(.72*s.x,s.y)]),Color("725c3b"))
	draw_colored_polygon(PackedVector2Array([Vector2(.45*s.x,s.y),Vector2(.50*s.x,.57*s.y),Vector2(.54*s.x,.57*s.y),Vector2(.59*s.x,s.y)]),Color("a67a46"))
	for x in [0.04,0.14,0.77,0.88,0.96]:
		draw_circle(Vector2(x*s.x,.49*s.y),.15*s.y,Color("21513f"))
		draw_circle(Vector2((x+.03)*s.x,.36*s.y),.10*s.y,Color("32705a"))
	# Foreground leaves and floor bands make actor depth and collision ground explicit.
	for x in [0.02,0.09,0.17,0.84,0.92,0.99]:
		draw_circle(Vector2(x*s.x,.78*s.y),.13*s.y,Color("16352e"))
	draw_line(Vector2(.08*s.x,.84*s.y),Vector2(.92*s.x,.84*s.y),Color("d3a75d",.32),2)
	draw_string(ThemeDB.fallback_font,Vector2(18,28),"RECEPTION ROAD",HORIZONTAL_ALIGNMENT_LEFT,-1,16,Color("e4c487"))
	draw_string(ThemeDB.fallback_font,Vector2(18,46),"ELVEN FUNERARY WAY • LATE AFTERNOON",HORIZONTAL_ALIGNMENT_LEFT,-1,10,Color("b9c0a8"))
