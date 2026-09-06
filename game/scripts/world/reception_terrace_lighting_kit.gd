class_name ReceptionTerraceLightingKit
extends Node3D
## Lighting is isolated so look development does not modify geometry or game
## semantics.
##
## The kit no longer builds its own Environment and its own two lights. It
## instances the render foundation's `res://render/world_environment.tscn` and
## sets the hour this setpiece is lit at, so the island has one light rig and
## a change to it reaches every scene. Everything named below is an exported
## property on that scene, which is the same seam P3 drives from the
## simulation snapshot.

const ISLAND_ENVIRONMENT := preload("res://render/world_environment.tscn")

## The storm break: a low, warm sun coming in under the cloud from the seaward
## side, which is most of why this terrace reads as a place something has just
## happened at.
const STORM_BREAK_PITCH_DEGREES := 34.0
const STORM_BREAK_YAW_DEGREES := -36.0
const STORM_BREAK_COLOR := Color("ffd28d")
const STORM_BREAK_ENERGY := 2.05

## Wet jungle after rain: dense fog in the sea's own colour, and a cool fill
## bouncing off the water downhill.
const JUNGLE_FOG_DENSITY := 0.0125
const JUNGLE_FOG_COLOR := Color("46807e")
const JUNGLE_FILL_COLOR := Color("74d6c5")
const JUNGLE_FILL_ENERGY := 0.55


func _ready() -> void:
	set_meta("replacement_scope", "world environment, storm-break key, jungle fill and fog")
	set_meta("render_owner", "res://render/world_environment.tscn; this kit only chooses the hour and the weather.")
	var world := ISLAND_ENVIRONMENT.instantiate() as IslandEnvironment
	world.name = "SetpieceEnvironment"
	world.sun_pitch_degrees = STORM_BREAK_PITCH_DEGREES
	world.sun_yaw_degrees = STORM_BREAK_YAW_DEGREES
	world.sun_color = STORM_BREAK_COLOR
	world.sun_energy = STORM_BREAK_ENERGY
	world.fill_color = JUNGLE_FILL_COLOR
	world.fill_energy = JUNGLE_FILL_ENERGY
	world.fog_color = JUNGLE_FOG_COLOR
	world.fog_density = JUNGLE_FOG_DENSITY
	add_child(world)
