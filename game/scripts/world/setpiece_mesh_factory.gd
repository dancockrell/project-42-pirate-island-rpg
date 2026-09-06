class_name SetpieceMeshFactory
extends RefCounted
## Thin adapter around Godot's built-in mesh classes, and the one place a
## procedural setpiece gets a material from.
##
## `material()` used to return a flat StandardMaterial3D. It now returns a
## clay-blockout ShaderMaterial from `res://render/materials/`, so every
## procedural box in the game wears the render foundation's grammar -- matte
## base, rim light, top-down sky wrap -- and a change to that grammar moves
## every setpiece at once instead of being copied into each one.
##
## The signature and the palette are unchanged on purpose: callers keep
## passing the colour, metalness, roughness and emission they already chose,
## and the look changes without any of them being rewritten.

## The material library. A key is what a caller asks for by name; the value is
## the resource `game/render/materials/` owns.
const CLAY := "clay"
const WET_STONE := "wet_stone"
const BRONZE := "bronze"
const VELLUM := "vellum"
const FOLIAGE := "foliage"
const SEA := "sea"
const RIVER := "river"
const PLACEHOLDER := "placeholder"

const LIBRARY := {
	CLAY: "res://render/materials/clay_blockout.tres",
	WET_STONE: "res://render/materials/wet_stone.tres",
	BRONZE: "res://render/materials/bronze.tres",
	VELLUM: "res://render/materials/vellum.tres",
	FOLIAGE: "res://render/materials/foliage.tres",
	SEA: "res://render/materials/stylised_sea.tres",
	RIVER: "res://render/materials/stylised_river.tres",
	PLACEHOLDER: "res://render/materials/placeholder.tres",
}


## A clay-blockout material tinted for one surface. The parameters are the
## ones the old StandardMaterial3D took, so no caller had to change.
static func material(color: Color, metallic: float = 0.0, roughness: float = 0.9, emission: Color = Color.BLACK, emission_energy: float = 0.0) -> Material:
	var result := library_material(CLAY)
	if result == null:
		return null
	result.set_shader_parameter("albedo", color)
	result.set_shader_parameter("metallic", metallic)
	result.set_shader_parameter("roughness", roughness)
	result.set_shader_parameter("emission_color", emission)
	result.set_shader_parameter("emission_energy", emission_energy)
	return result


## A fresh copy of a named library material, ready to be tuned per surface.
## Duplicated rather than shared, so setting a parameter on one wall does not
## repaint every other wall in the game.
static func library_material(key: String) -> ShaderMaterial:
	if not LIBRARY.has(key):
		push_error("SetpieceMeshFactory has no library material named: %s" % key)
		return null
	var source := load(LIBRARY[key]) as ShaderMaterial
	if source == null:
		push_error("SetpieceMeshFactory could not load library material: %s" % LIBRARY[key])
		return null
	return source.duplicate() as ShaderMaterial


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
