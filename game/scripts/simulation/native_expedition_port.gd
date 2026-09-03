class_name NativeExpeditionPort
extends ExpeditionPort

## The only Godot-facing owner of Project42ExpeditionBridge. The content
## catalog supplies the authored route, while Rust owns state mutation and
## legal-command calculation.

const BRIDGE_CLASS := "Project42ExpeditionBridge"
const INITIAL_PARTY := ["character.protagonist.captain", "character.heroine.betty"]
const INITIAL_LOCATION_ID := "world.cell.black_beach"

var bridge: Object


func _init() -> void:
	if not ClassDB.class_exists(BRIDGE_CLASS) or not ClassDB.can_instantiate(BRIDGE_CLASS):
		push_error("%s is unavailable. Build the Rust GDExtension before selecting NativeExpeditionPort." % BRIDGE_CLASS)
		return
	bridge = ClassDB.instantiate(BRIDGE_CLASS)


func is_available() -> bool:
	return bridge != null


static func bridge_is_registered() -> bool:
	return ClassDB.class_exists(BRIDGE_CLASS) and ClassDB.can_instantiate(BRIDGE_CLASS)


func configure_from_catalog(catalog: ContentCatalog, seed: int) -> Dictionary:
	if not is_available():
		return unavailable_state()
	var region := catalog.get_record("world.region.black_beach")
	var portals: Array[Dictionary] = []
	var encounter_triggers: Array[Dictionary] = []
	for cell_id in region.get("worldCellIds", []):
		var cell := catalog.get_record(str(cell_id))
		for portal in cell.get("portals", []):
			portals.append({
				"id": str(portal.get("id", "")),
				"from_location_id": str(cell.get("id", "")),
				"target_location_id": str(portal.get("targetCellId", "")),
				"travel_mode": str(portal.get("travelMode", ""))
			})
		for entry in cell.get("battleEntries", []):
			if str(entry.get("status", "")) != "vertical_slice_encounter":
				continue
			var encounter := catalog.get_record(str(entry.get("encounterId", "")))
			encounter_triggers.append({
				"location_id": str(cell.get("id", "")),
				"encounter_id": str(encounter.get("id", "")),
				"battle_id": str(encounter.get("battleId", ""))
			})
	var configuration := {
		"seed": seed,
		"party_ids": INITIAL_PARTY,
		"active_location_id": INITIAL_LOCATION_ID,
		"portals": portals,
		"encounter_triggers": encounter_triggers
	}
	return bridge.configure(JSON.stringify(configuration))


func snapshot() -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.snapshot()


func travel(portal_id: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.travel(portal_id)


func unavailable_state() -> Dictionary:
	return {"configured": false, "error": "native_expedition_bridge_unavailable", "metadata": {"source": "godot_adapter", "authoritative": false}}
