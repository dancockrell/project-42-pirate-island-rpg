extends Node2D

const Actor = preload("res://scripts/world/directional_sprite.gd")

func _ready() -> void:
	RenderingServer.set_default_clear_color(Color("#253a3b"))
	var title := Label.new()
	title.text = "MICHAEL / PIXEL SPRITE REVIEW — standing views, not walk animation"
	title.position = Vector2(30, 25)
	add_child(title)
	for i in range(4):
		var actor = Actor.new()
		add_child(actor)
		assert(actor.configure("res://assets/sprites/michael/source.png", "res://assets/sprites/michael/frames.json"))
		actor.set_facing(["se", "sw", "ne", "nw"][i])
		actor.scale = Vector2.ONE * 0.18
		actor.position = Vector2(110 + i * 170, 225)
		var label := Label.new()
		label.text = ["SE", "SW", "NE", "NW"][i]
		label.position = actor.position + Vector2(-10, 15)
		add_child(label)
