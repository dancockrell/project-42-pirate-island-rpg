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
	var cells: Array[Dictionary] = []
	for cell_id in region.get("worldCellIds", []):
		var cell := catalog.get_record(str(cell_id))
		# The cell itself, with the anchors it declares (C13). Before this the
		# bridge learned only the portals, so no anchor existed in Godot: the
		# wreck could not be salvaged and the estate's rooms did not exist,
		# and with real road costs the party was stranded on the sand.
		# Anchors are forwarded in their authored shape; Rust owns the one
		# translation of `kind`.
		var observation_ids: Array[String] = []
		for description in cell.get("readableDescriptions", []):
			observation_ids.append(str(description.get("id", "")))
		var anchors: Array[Dictionary] = []
		for anchor in cell.get("anchors", []):
			var forwarded_anchor := {
				"id": str(anchor.get("id", "")),
				"kind": str(anchor.get("kind", "")),
				"rations": int(anchor.get("rations", 0)),
				"medicine": int(anchor.get("medicine", 0)),
				"coin": int(anchor.get("coin", 0)),
				"once_per_day": bool(anchor.get("oncePerDay", false))
			}
			for pair in [["requiresDiscoveryId", "requires_discovery_id"], ["grantsDiscoveryId", "grants_discovery_id"]]:
				var value := str(anchor.get(pair[0], ""))
				if not value.is_empty():
					forwarded_anchor[pair[1]] = value
			anchors.append(forwarded_anchor)
		var encounter_eligible := false
		for entry in cell.get("battleEntries", []):
			if str(entry.get("status", "")) == "vertical_slice_encounter":
				encounter_eligible = true
		cells.append({
			"id": str(cell.get("id", "")),
			"region_id": str(cell.get("regionId", "")),
			"display_name": str(cell.get("displayName", "")),
			"observation_ids": observation_ids,
			"anchors": anchors,
			"encounter_eligible": encounter_eligible
		})
		for portal in cell.get("portals", []):
			# Every field PortalDefinition reads, not only the endpoints. Until
			# C13 this forwarded id, endpoints and travelMode alone, so through
			# the real bridge every authored road was free and no gate existed:
			# C2's costs and A4's tidal-cut gate lived in content and never
			# reached the game. serde(default) on the Rust side means an absent
			# cost is free, so a portal authored before it is priced still loads.
			var forwarded := {
				"id": str(portal.get("id", "")),
				"from_location_id": str(cell.get("id", "")),
				"target_location_id": str(portal.get("targetCellId", "")),
				"travel_mode": str(portal.get("travelMode", "")),
				"time_cost_minutes": int(portal.get("timeCostMinutes", 0)),
				"supply_cost": int(portal.get("supplyCost", 0)),
				"risk_level": int(portal.get("riskLevel", 0))
			}
			var required_discovery_id := str(portal.get("requiredDiscoveryId", ""))
			if not required_discovery_id.is_empty():
				forwarded["required_discovery_id"] = required_discovery_id
			portals.append(forwarded)
		for entry in cell.get("battleEntries", []):
			if str(entry.get("status", "")) != "vertical_slice_encounter":
				continue
			var encounter := catalog.get_record(str(entry.get("encounterId", "")))
			var trigger := {
				"location_id": str(cell.get("id", "")),
				"encounter_id": str(encounter.get("id", "")),
				"battle_id": str(encounter.get("battleId", ""))
			}
			var estate_consequence: Dictionary = encounter.get("estateConsequence", {})
			var estate_upgrade_id := str(estate_consequence.get("estateUpgradeId", ""))
			if not estate_upgrade_id.is_empty():
				trigger["estate_upgrade_id"] = estate_upgrade_id
			encounter_triggers.append(trigger)
	var configuration := {
		"seed": seed,
		"party_ids": INITIAL_PARTY,
		"active_location_id": INITIAL_LOCATION_ID,
		"cells": cells,
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


func use_anchor(anchor_id: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.use_anchor(anchor_id)


func inspect(observation_id: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.inspect(observation_id)


func resolve_midnight() -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.resolve_midnight()


func unavailable_state() -> Dictionary:
	return {"configured": false, "error": "native_expedition_bridge_unavailable", "metadata": {"source": "godot_adapter", "authoritative": false}}
