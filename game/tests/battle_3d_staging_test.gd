extends SceneTree

const STAGE_SCENE := preload("res://scenes/battle/battle_3d_staging.tscn")


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var stage: Battle3DStaging = STAGE_SCENE.instantiate()
	get_root().add_child(stage)
	await process_frame
	check(stage.party_foreground_anchor.position.is_equal_approx(Battle3DStaging.PARTY_FOREGROUND), "Betty must have a stable 3D party anchor")
	check(stage.enemy_foreground_anchor.position.is_equal_approx(Battle3DStaging.ENEMY_FOREGROUND), "Razorbeak must have a stable 3D enemy anchor")
	check(stage.action_anchor().position.is_equal_approx(Battle3DStaging.CONTESTED_ACTION), "skills must share one explicit contested action point")
	check(stage.actor_anchor("character.heroine.betty") == stage.party_foreground_anchor, "Betty must resolve to her 3D anchor")
	check(stage.actor_anchor("enemy.raptor.razorbeak.prototype") == stage.enemy_foreground_anchor, "Razorbeak must resolve to its 3D anchor")
	check(stage.battle_camera.fov == Battle3DStaging.CAMERA_FOV_DEGREES, "battle 3D camera FOV must match the production contract")
	stage.queue_free()
	quit(0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	push_error(message)
	quit(1)
