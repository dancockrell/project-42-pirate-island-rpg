extends SceneTree

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")

func _initialize() -> void:
	var actor := PanelContainer.new()
	actor.size = Vector2(480, 560)
	var enemy := PanelContainer.new()
	enemy.size = Vector2(480, 560)
	var label := Label.new()
	root.add_child(actor)
	root.add_child(enemy)
	root.add_child(label)
	var presenter := PlaceholderActionPresenterScript.new()
	root.add_child(presenter)
	presenter.configure(actor, enemy, label)
	presenter.present({
		"pose": "horizontal_mace_hit",
		"motion": "contact_lunge",
		"camera": "impact_punch_in",
		"vfx": "bronze_teal_impact_arc",
		"audio": "mace_armor_impact",
		"hitStopMs": 80,
		"shake": 0.35,
		"frameSubjects": ["betty", "mace", "target", "impact_arc"]
	})
	assert("HORIZONTAL MACE HIT" in label.text)
	assert("SAFE FRAME: betty, mace, target, impact_arc" in label.text)
	assert(label.tooltip_text == "Audio cue: mace_armor_impact")
	assert(actor.scale.x > 1.0)
	assert(enemy.rotation != 0.0)
	presenter.reset()
	assert(actor.scale == Vector2.ONE)
	assert(enemy.rotation == 0.0)
	print("PlaceholderActionPresenter tests passed.")
	quit(0)
