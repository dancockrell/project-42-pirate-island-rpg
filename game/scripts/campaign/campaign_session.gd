extends Node

## Runtime owner for one native campaign bridge. This node never calculates
## routes, encounters, outcomes, or resources; it only keeps the same
## NativeExpeditionPort alive while Godot swaps presentation scenes.

const INITIAL_SEED := 42

var expedition: NativeExpeditionPort
var catalog: ContentCatalog
var latest_snapshot: Dictionary = {}


func begin_if_needed(next_catalog: ContentCatalog) -> Dictionary:
	if expedition != null and bool(latest_snapshot.get("configured", false)):
		return latest_snapshot.duplicate(true)
	catalog = next_catalog
	if not NativeExpeditionPort.bridge_is_registered():
		latest_snapshot = {
			"configured": false,
			"error": "native_expedition_bridge_unavailable",
			"metadata": {"source": "campaign_session", "authoritative": false}
		}
		return latest_snapshot.duplicate(true)
	expedition = NativeExpeditionPort.new()
	latest_snapshot = expedition.configure_from_catalog(catalog, INITIAL_SEED)
	return latest_snapshot.duplicate(true)


func travel(portal_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.travel(portal_id)
	return latest_snapshot.duplicate(true)


func snapshot() -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.snapshot()
	return latest_snapshot.duplicate(true)


func pending_encounter() -> Dictionary:
	return latest_snapshot.get("pending_encounter", {}).duplicate(true)


func reset_for_test() -> void:
	expedition = null
	catalog = null
	latest_snapshot = {}


func unavailable_state() -> Dictionary:
	return {
		"configured": false,
		"error": "campaign_session_uninitialized",
		"metadata": {"source": "campaign_session", "authoritative": false}
	}
