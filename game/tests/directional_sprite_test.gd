extends SceneTree

const Actor = preload("res://scripts/world/directional_sprite.gd")

func _initialize() -> void:
	var actor = Actor.new()
	assert(actor.configure("res://assets/sprites/michael/source.png", "res://assets/sprites/michael/frames.json"))
	assert(actor.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST)
	for pair in [[Vector2(1,1),"se"],[Vector2(-1,1),"sw"],[Vector2(1,-1),"ne"],[Vector2(-1,-1),"nw"]]:
		actor.project_heading(pair[0])
		assert(actor.facing == pair[1])
		assert(actor.texture is AtlasTexture)
		assert(actor.texture.region.size.x == 607)
	actor.project_heading(Vector2.ZERO)
	assert(actor.facing == "nw")
	assert(not actor.set_facing("invalid"))
	assert(actor.facing == "nw")
	assert(actor.position == Vector2.ZERO)
	actor.free()
	print("PASS: four facings, alpha-source identity, frame regions, nearest sampling, idle retention, no simulation movement")
	quit()
