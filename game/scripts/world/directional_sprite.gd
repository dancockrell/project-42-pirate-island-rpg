extends Sprite2D

## Presentation only: consumes authoritative heading; never moves a simulation actor.
## Current source is review-only standing art, not a walking animation.
var definitions: Array = []
var facing := "se"
var source_texture: Texture2D

func configure(image_path: String, manifest_path: String) -> bool:
	var data = JSON.parse_string(FileAccess.get_file_as_string(manifest_path))
	if not data is Dictionary or data.get("schemaVersion") != 1:
		return false
	var image := Image.load_from_file(image_path)
	if image == null or image.is_empty():
		return false
	if FileAccess.get_sha256(image_path) != data.get("sourceSha256"):
		return false
	var candidates = data.get("frames", [])
	if not candidates is Array or candidates.is_empty():
		return false
	var directions := {}
	for entry in candidates:
		if not entry is Dictionary:
			return false
		var rect = entry.get("rect", [])
		var pivot = entry.get("footPivot", [])
		var direction = entry.get("direction", "")
		if rect.size() != 4 or pivot.size() != 2 or direction not in ["se", "sw", "ne", "nw"]:
			return false
		if directions.has(direction) or entry.get("action") != "idle":
			return false
		if rect[0] < 0 or rect[1] < 0 or rect[2] <= 0 or rect[3] <= 0:
			return false
		if rect[0] + rect[2] > image.get_width() or rect[1] + rect[3] > image.get_height():
			return false
		if pivot[0] < 0 or pivot[1] < 0 or pivot[0] > rect[2] or pivot[1] > rect[3]:
			return false
		directions[direction] = true
	if directions.size() != 4:
		return false
	definitions = candidates
	source_texture = ImageTexture.create_from_image(image)
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	centered = false
	set_facing(facing)
	return true

func set_facing(direction: String) -> bool:
	for entry in definitions:
		if entry.direction != direction:
			continue
		var rect: Array = entry.rect
		var atlas := AtlasTexture.new()
		atlas.atlas = source_texture
		atlas.region = Rect2(rect[0], rect[1], rect[2], rect[3])
		atlas.filter_clip = true
		texture = atlas
		offset = -Vector2(entry.footPivot[0], entry.footPivot[1])
		facing = direction
		return true
	return false

func project_heading(heading: Vector2) -> void:
	if heading.is_zero_approx():
		return
	set_facing(("s" if heading.y >= 0 else "n") + ("e" if heading.x >= 0 else "w"))
