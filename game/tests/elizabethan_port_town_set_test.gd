extends SceneTree

const REVIEW_SCENE := preload("res://scenes/review/elizabethan_port_town_set_review.tscn")
const TOWN_SCRIPT := preload("res://scripts/world/elizabethan_port_town_set.gd")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var review := REVIEW_SCENE.instantiate()
	root.add_child(review)
	await process_frame
	var town: Node = review.get_node_or_null("ElizabethanPortTownSet")
	check(town != null, "review must instantiate the canonical Elizabethan port-town set")
	check(town.get_meta("pack_id", "") == TOWN_SCRIPT.PACK_ID, "town must identify its resource pack")
	check(town.get_meta("production_state", "") == "procedural_proof_blockout", "town must not claim final-art authority")
	for required_section in ["Environment", "Routes", "Districts", "PopulationSlots", "GameplayAnchors"]:
		var section: Node = town.get_node_or_null(required_section)
		check(section != null, "town is missing required section: %s" % required_section)
		check(section.get_child_count() > 0, "required section must contain visible or semantic proof: %s" % required_section)
	var districts: Node = town.get_node("Districts")
	check(districts.get_child_count() == TOWN_SCRIPT.RECIPE_IDS.size(), "every recipe must have exactly one district owner")
	var found_recipe_ids: Array[String] = []
	for district in districts.get_children():
		var recipe_id := str(district.get_meta("recipe_id", ""))
		check(recipe_id in TOWN_SCRIPT.RECIPE_IDS, "district has unknown recipe id: %s" % recipe_id)
		check(recipe_id not in found_recipe_ids, "recipe has duplicate district owner: %s" % recipe_id)
		check(district.get_child_count() > 0, "district must visibly block its function: %s" % recipe_id)
		found_recipe_ids.append(recipe_id)
	for recipe_id in TOWN_SCRIPT.RECIPE_IDS:
		check(recipe_id in found_recipe_ids, "recipe is missing from scene: %s" % recipe_id)
	check(town.get_node("PopulationSlots").get_child_count() >= 10, "town must communicate population roles without fake named characters")
	check(town.get_node("GameplayAnchors").get_child_count() >= 8, "town must expose its important semantic positions")
	review.queue_free()
	quit(0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	push_error(message)
	quit(1)
