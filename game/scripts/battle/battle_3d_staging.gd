class_name Battle3DStaging
extends Node3D
## The future 3D battle plane. It owns spatial anchors and camera placement,
## never combat rules, target selection, UI state or an implied hit result.
##
## Candidate 01 may appear here only for static silhouette and camera review.
## Animated actor admission is separately guarded by the 3D asset contract.

const PARTY_FOREGROUND := Vector3(-2.65, 0.0, 0.0)
const CONTESTED_ACTION := Vector3(0.0, 0.0, 0.0)
const ENEMY_FOREGROUND := Vector3(2.65, 0.0, 0.1)
const CAMERA_FOCUS := Vector3(0.0, 1.65, 0.0)
const CAMERA_POSITION := Vector3(0.0, 1.85, 9.25)
const CAMERA_FOV_DEGREES := 31.0

@onready var party_foreground_anchor: Node3D = $PartyAnchors/BettyAnchor
@onready var contested_action_anchor: Node3D = $EffectAnchors/ContestedAction
@onready var enemy_foreground_anchor: Node3D = $EnemyAnchors/RazorbeakAnchor
@onready var battle_camera: Camera3D = $BattleCamera


func _ready() -> void:
	party_foreground_anchor.position = PARTY_FOREGROUND
	contested_action_anchor.position = CONTESTED_ACTION
	enemy_foreground_anchor.position = ENEMY_FOREGROUND
	battle_camera.position = CAMERA_POSITION
	battle_camera.fov = CAMERA_FOV_DEGREES
	battle_camera.look_at(CAMERA_FOCUS, Vector3.UP)


func actor_anchor(actor_id: String) -> Node3D:
	match actor_id:
		"character.heroine.betty":
			return party_foreground_anchor
		"enemy.raptor.razorbeak.prototype":
			return enemy_foreground_anchor
		_:
			push_error("No 3D staging anchor registered for actor: %s" % actor_id)
			return contested_action_anchor


func action_anchor() -> Node3D:
	return contested_action_anchor
