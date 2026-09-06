class_name AtmosphereReviewBoard
extends RefCounted

## A review-only stand-in for the thing that will answer "where on the board is
## this building?" -- lane P4's board.
##
## `Atmosphere` resolves the night's lamps from the snapshot's own building
## list and asks its `board_anchor_provider` where each one stands. Until P4's
## board exists there is no honest answer, so in the game there are no lamps:
## the autoload reports them unplaced and spawns nothing.
##
## A committed capture still has to *show* the night the card describes, so
## this stands the lamps on the review scene's own `EntryAnchors` markers, in
## marker order. It decides nothing: the markers are the scene's, the buildings
## are the snapshot's, and no simulation reads a position from here. It is
## wired only by `tools/capture_review_scene.gd`, and the day P4's board can
## answer, that line points at the board and this file goes.

## The node the review scenes hang their marked positions under.
const ANCHOR_GROUP := "EntryAnchors"

var _anchors: Array[Vector3] = []
var _assigned: Dictionary = {}


func _init(scene: Node) -> void:
	for node in _descendants(scene):
		if node.name != ANCHOR_GROUP:
			continue
		for marker in node.get_children():
			var marker_3d := marker as Marker3D
			if marker_3d != null:
				_anchors.append(marker_3d.global_position)
		break


## True when the review scene carried markers to stand a lamp on. A scene with
## none gets no provider at all rather than a provider that answers zero.
func has_anchors() -> bool:
	return not _anchors.is_empty()


## Where this building's lamp stands, for review. Stable: the same building
## gets the same marker every time, so two captures of one snapshot are the
## same picture.
func building_anchor(building_id: String) -> Vector3:
	if _anchors.is_empty():
		return Vector3.ZERO
	if not _assigned.has(building_id):
		# Spread rather than crowd: two lamps on adjacent markers read as one
		# light, and the point of the capture is that held ground is lit.
		var stride := maxi(1, _anchors.size() / 2)
		_assigned[building_id] = _anchors[(_assigned.size() * stride) % _anchors.size()]
	return _assigned[building_id]


func _descendants(root: Node) -> Array[Node]:
	var found: Array[Node] = [root]
	for child in root.get_children():
		found.append_array(_descendants(child))
	return found
