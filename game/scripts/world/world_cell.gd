class_name WorldCell
extends Node3D
## A bounded 3D island location. Visual geometry is replaceable. Collision,
## navigation, entry, battle and camera semantics are stable authored nodes.
## This scene never derives gameplay meaning from an imported baked mesh.

@export var world_cell_id := ""

@onready var visual_shell: Node3D = $VisualShell
@onready var terrain_and_collision: Node3D = $TerrainAndCollision
@onready var navigation: Node3D = $Navigation
@onready var entry_anchors: Node3D = $EntryAnchors
@onready var interaction_anchors: Node3D = $InteractionAnchors
@onready var battle_entries: Node3D = $BattleEntries
@onready var camera_rails: Node3D = $CameraRails

var record: Dictionary = {}


func configure(catalog: ContentCatalog) -> Error:
	if world_cell_id.is_empty() or not catalog.has(world_cell_id):
		push_error("World cell has no content record: %s" % world_cell_id)
		return ERR_DOES_NOT_EXIST
	record = catalog.get_record(world_cell_id)
	if record.get("kind") != "world_cell":
		push_error("World cell record has wrong kind: %s" % world_cell_id)
		return ERR_INVALID_DATA
	if not _has_named_anchors(record.get("entryAnchors", []), entry_anchors):
		return ERR_INVALID_DATA
	if not _has_named_anchors(record.get("interactionAnchors", []), interaction_anchors):
		return ERR_INVALID_DATA
	if not _has_named_anchors(record.get("battleEntries", []), battle_entries):
		return ERR_INVALID_DATA
	return OK


func entry_anchor(anchor_id: String) -> Node3D:
	var anchor := entry_anchors.get_node_or_null(NodePath(anchor_id)) as Node3D
	if anchor == null:
		push_error("World cell %s is missing entry anchor %s" % [world_cell_id, anchor_id])
	return anchor


func battle_entry(entry_id: String) -> Node3D:
	var anchor := battle_entries.get_node_or_null(NodePath(entry_id)) as Node3D
	if anchor == null:
		push_error("World cell %s is missing battle entry %s" % [world_cell_id, entry_id])
	return anchor


func interaction_anchor(anchor_id: String) -> Node3D:
	var anchor := interaction_anchors.get_node_or_null(NodePath(anchor_id)) as Node3D
	if anchor == null:
		push_error("World cell %s is missing interaction anchor %s" % [world_cell_id, anchor_id])
	return anchor


func _has_named_anchors(definitions: Array, parent: Node3D) -> bool:
	for definition in definitions:
		var anchor_id := str(definition.get("id", ""))
		if anchor_id.is_empty() or parent.get_node_or_null(NodePath(anchor_id)) == null:
			push_error("World cell %s is missing authored anchor %s" % [world_cell_id, anchor_id])
			return false
	return true
