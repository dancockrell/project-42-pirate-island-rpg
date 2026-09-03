class_name ReceptionTerraceSetpiece
extends Node3D
## Root composition only. Named children own the individual setpiece concerns.

const KIT_VERSION := "reception-terrace-standard-godot-kit-v2"


func _ready() -> void:
	set_meta("production_state", "procedural-standard-godot-setpiece")
	set_meta("kit_version", KIT_VERSION)
	set_meta("visual_authority", "docs/VISUAL_AUTHORITY_AND_3D_ENTRY_GATE.md")
	set_meta("art_replacement_rule", "Replace one named component scene at a time. Imported visuals never own collision, navigation, anchors, encounter IDs or prose IDs.")
	set_meta("setpiece_description", "A broken magical Bronze Age elven reception terrace. A processional approach runs through wet jungle toward a fractured gate. The sea and Handsome Jack flotsam remain visible downhill.")
