extends Node3D
## Camera-only review behavior. The production set remains independent of how
## an art director chooses to inspect it.

@onready var review_camera: Camera3D = $ReviewCamera


func _ready() -> void:
	review_camera.look_at(Vector3(0.0, 4.0, -4.0), Vector3.UP)
