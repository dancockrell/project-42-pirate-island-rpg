class_name ReceptionTerraceLightingKit
extends Node3D
## Lighting is isolated so look development does not modify geometry or game
## semantics.

func _ready() -> void:
	set_meta("replacement_scope", "world environment, storm-break key, jungle fill and fog")
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color("091b20")
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color("42646a")
	environment.ambient_light_energy = 0.62
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_exposure = 1.08
	environment.glow_enabled = true
	environment.glow_intensity = 0.32
	environment.fog_enabled = true
	environment.fog_light_color = Color("6e9283")
	environment.fog_light_energy = 0.42
	environment.fog_density = 0.012
	var world_environment := WorldEnvironment.new()
	world_environment.name = "SetpieceEnvironment"
	world_environment.environment = environment
	add_child(world_environment)
	var warm_sun := DirectionalLight3D.new()
	warm_sun.name = "StormBreakSun"
	warm_sun.rotation_degrees = Vector3(-48.0, -36.0, 0.0)
	warm_sun.light_color = Color("ffd28d")
	warm_sun.light_energy = 1.55
	warm_sun.shadow_enabled = true
	add_child(warm_sun)
	var cool_fill := DirectionalLight3D.new()
	cool_fill.name = "JungleFill"
	cool_fill.rotation_degrees = Vector3(-32.0, 141.0, 0.0)
	cool_fill.light_color = Color("74d6c5")
	cool_fill.light_energy = 0.55
	add_child(cool_fill)
