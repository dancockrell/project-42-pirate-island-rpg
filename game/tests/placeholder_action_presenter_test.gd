extends SceneTree

const PlaceholderActionPresenterScript = preload("res://scripts/battle/placeholder_action_presenter.gd")

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
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
		"cameraId": "presentation.camera.impact_punch_in",
		"cameraRecord": {"mode": "punch", "zoom": 1.12},
		"vfxId": "presentation.vfx.bronze_teal_impact_arc",
		"vfxRecord": {"purpose": "Shows the mace contact", "anchor": "weapon_contact", "envelopeWidthPercent": 22, "envelopeHeightPercent": 28, "persistenceMs": 180, "reducedFlashMode": "replace_flash_with_outline", "assetStatus": "placeholder"},
		"audio": "mace_armor_impact",
		"hitStopMs": 80,
		"shake": 0.35,
		"frameSubjects": ["betty", "mace", "target", "impact_arc"]
	})
	# Wait on the presenter's own tweens, not a wall-clock timer: the tree steps
	# timers before tweens, and the first frame's delta carries engine start-up
	# (0.13 s with the current autoloads), so a 0.12 s timer fired before the
	# 0.10 s tweens had taken one step and the assertions below halted the
	# suite without the presenter being wrong.
	await presenter.actor_tween.finished
	if presenter.enemy_tween != null and presenter.enemy_tween.is_running():
		await presenter.enemy_tween.finished
	assert(label.text == "HORIZONTAL MACE HIT  •  BRONZE TEAL IMPACT ARC")
	assert("Safe frame: betty, mace, target, impact_arc" in label.tooltip_text)
	assert("VFX purpose: Shows the mace contact" in label.tooltip_text)
	assert("Audio cue: mace_armor_impact" in label.tooltip_text)
	assert(actor.scale.x > 1.0)
	assert(enemy.rotation != 0.0)
	presenter.reset()
	assert(actor.scale == Vector2.ONE)
	assert(enemy.rotation == 0.0)
	print("PlaceholderActionPresenter tests passed.")
	quit(0)
