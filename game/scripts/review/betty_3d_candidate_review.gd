extends Node3D
## Review-only staging for the downloaded Betty GLB.
##
## This scene is deliberately isolated from the combat scene. The asset was
## structurally inspected before import and contains no Skin or Animation data;
## this script may fit its static silhouette to the battle camera, but it may
## never imply that the candidate is a usable animated combat actor.

const TARGET_FIGURE_HEIGHT_METRES := 3.85
const CAMERA_TARGET_HEIGHT_METRES := 2.03
const CAMERA_DISTANCE_METRES := 8.5

@onready var candidate_root: Node3D = $CandidateRoot
@onready var candidate: Node3D = $CandidateRoot/BettyCandidate
@onready var battle_camera: Camera3D = $BattleCamera


func _ready() -> void:
	call_deferred("_fit_static_candidate_to_battle_crop")


func _fit_static_candidate_to_battle_crop() -> void:
	var local_bounds := _mesh_bounds_relative_to_candidate()
	if local_bounds.size.y <= 0.001:
		push_error("Betty 3D candidate review could not find usable mesh bounds.")
		return

	var uniform_scale := TARGET_FIGURE_HEIGHT_METRES / local_bounds.size.y
	candidate_root.scale = Vector3.ONE * uniform_scale
	candidate_root.position.y = -local_bounds.position.y * uniform_scale
	battle_camera.position = Vector3(0.0, CAMERA_TARGET_HEIGHT_METRES, CAMERA_DISTANCE_METRES)
	battle_camera.look_at(Vector3(0.0, CAMERA_TARGET_HEIGHT_METRES, 0.0), Vector3.UP)


func _mesh_bounds_relative_to_candidate() -> AABB:
	var meshes: Array[MeshInstance3D] = []
	_collect_meshes(candidate, meshes)
	var result := AABB()
	var initialized := false
	for mesh_instance in meshes:
		if mesh_instance.mesh == null:
			continue
		var mesh_bounds := mesh_instance.get_aabb()
		var relative_transform := candidate.global_transform.affine_inverse() * mesh_instance.global_transform
		var transformed_bounds := _transform_aabb(mesh_bounds, relative_transform)
		if initialized:
			result = result.merge(transformed_bounds)
		else:
			result = transformed_bounds
			initialized = true
	return result


func _collect_meshes(node: Node, output: Array[MeshInstance3D]) -> void:
	if node is MeshInstance3D:
		output.append(node)
	for child in node.get_children():
		_collect_meshes(child, output)


func _transform_aabb(source: AABB, transform_to_target: Transform3D) -> AABB:
	var transformed := AABB(transform_to_target * source.position, Vector3.ZERO)
	for corner in [
		Vector3(source.position.x + source.size.x, source.position.y, source.position.z),
		Vector3(source.position.x, source.position.y + source.size.y, source.position.z),
		Vector3(source.position.x, source.position.y, source.position.z + source.size.z),
		Vector3(source.position.x + source.size.x, source.position.y + source.size.y, source.position.z),
		Vector3(source.position.x + source.size.x, source.position.y, source.position.z + source.size.z),
		Vector3(source.position.x, source.position.y + source.size.y, source.position.z + source.size.z),
		Vector3(source.position.x + source.size.x, source.position.y + source.size.y, source.position.z + source.size.z)
	]:
		transformed = transformed.expand(transform_to_target * corner)
	return transformed
