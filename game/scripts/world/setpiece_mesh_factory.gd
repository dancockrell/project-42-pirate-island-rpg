class_name SetpieceMeshFactory
extends RefCounted
## Thin adapter around Godot's built-in mesh and material classes.

static func material(color: Color, metallic: float = 0.0, roughness: float = 0.9, emission: Color = Color.BLACK, emission_energy: float = 0.0) -> StandardMaterial3D:
	var result := StandardMaterial3D.new()
	result.albedo_color = color
	result.metallic = metallic
	result.roughness = roughness
	result.emission_enabled = emission_energy > 0.0
	result.emission = emission
	result.emission_energy_multiplier = emission_energy
	return result


static func box(parent: Node3D, node_name: String, size: Vector3, location: Vector3, surface: Material, rotation_value: Vector3 = Vector3.ZERO) -> MeshInstance3D:
	var mesh := BoxMesh.new()
	mesh.size = size
	mesh.material = surface
	return _mesh(parent, node_name, mesh, location, rotation_value)


static func cylinder(parent: Node3D, node_name: String, top_radius: float, bottom_radius: float, height: float, location: Vector3, surface: Material, radial_segments: int = 8, rotation_value: Vector3 = Vector3.ZERO) -> MeshInstance3D:
	var mesh := CylinderMesh.new()
	mesh.top_radius = top_radius
	mesh.bottom_radius = bottom_radius
	mesh.height = height
	mesh.radial_segments = radial_segments
	mesh.material = surface
	return _mesh(parent, node_name, mesh, location, rotation_value)


static func sphere(parent: Node3D, node_name: String, scale_value: Vector3, location: Vector3, surface: Material) -> MeshInstance3D:
	var mesh := SphereMesh.new()
	mesh.radial_segments = 16
	mesh.rings = 8
	mesh.material = surface
	var instance := _mesh(parent, node_name, mesh, location, Vector3.ZERO)
	instance.scale = scale_value
	return instance


static func _mesh(parent: Node3D, node_name: String, mesh: Mesh, location: Vector3, rotation_value: Vector3) -> MeshInstance3D:
	var instance := MeshInstance3D.new()
	instance.name = node_name
	instance.mesh = mesh
	instance.position = location
	instance.rotation_degrees = rotation_value
	instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON
	parent.add_child(instance)
	return instance
