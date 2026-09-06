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
	# B15: C9's six faction records, forwarded verbatim. They are authored in
	# the Rust field names already -- `concept_key`, `resource_priorities`,
	# `relationship_tendencies` -- so unlike cells and portals there is no
	# renaming here and no chance of the two halves drifting apart; the extra
	# authoring keys (`displayName`, `metadata`) have no Rust field and are
	# ignored on the way in. Without this the bridge ran the strategic hours on
	# an empty registry while the Rust harness ran them on the loaded one, and
	# Godot's island and the harness's island were two different islands.
	var factions: Array[Dictionary] = []
	for faction_id in catalog.ids_with_prefix("faction."):
		factions.append(as_rust_integers(catalog.get_record(faction_id)) as Dictionary)
	var configuration := {
		"seed": seed,
		"party_ids": INITIAL_PARTY,
		"active_location_id": INITIAL_LOCATION_ID,
		"cells": cells,
		"portals": portals,
		"encounter_triggers": encounter_triggers,
		"factions": factions
	}
	var configured: Dictionary = bridge.configure(JSON.stringify(configuration))
	# A refused configuration used to be silent here: every later call answered
	# `expedition_not_configured` and the suite that noticed was several steps
	# downstream of the record that caused it. The bridge already names the
	# reason; say it once, where the payload was built.
	if not bool(configured.get("configured", false)):
		push_error("Native expedition bridge refused the catalog configuration: %s (%d cells, %d portals, %d factions)" % [
			str(configured.get("error", "")), cells.size(), portals.size(), factions.size()
		])
	return configured


## Every number in a faction record, back to an integer.
##
## `FactionDefinition` has no floating-point field: every number in it is an
## `i16` weight, and the authored records write them as integers. Godot's JSON
## round trip is what loses that -- a weight comes back out of `JSON.stringify`
## with a decimal point, and serde refuses the whole record rather than
## silently truncating it, which refused the whole configuration and left the
## bridge unconfigured. The forwarding above stays verbatim in every other
## sense: no field is renamed, dropped or defaulted, and a fractional weight
## would still be wrong -- it would simply be wrong in Rust, where the schema
## lives, instead of here.
static func as_rust_integers(value: Variant) -> Variant:
	match typeof(value):
		TYPE_FLOAT:
			return int(value)
		TYPE_DICTIONARY:
			var mapped := {}
			var source: Dictionary = value
			for key in source:
				mapped[key] = as_rust_integers(source[key])
			return mapped
		TYPE_ARRAY:
			var list := []
			for item in (value as Array):
				list.append(as_rust_integers(item))
			return list
	return value


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


## S2's control model through the one bridge. An empty faction_id releases the
## cell rather than forcing it unheld. The returned snapshot carries the
## ControlChanged events the change produced.
func set_control(cell_id: String, faction_id: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.set_control(cell_id, faction_id)


## The effective controller of a cell, "" for unheld. Godot never reads the
## campaign's raw ownership map; this is the only answer.
func controller_of(cell_id: String) -> String:
	if not is_available():
		return ""
	return bridge.controller_of(cell_id)


## One road's live risk: the authored base plus the contested modifier while
## its endpoints are held by different parties. An integer, or the bridge's
## error dictionary when no portal carries this ID.
func effective_risk(portal_id: String) -> Variant:
	if not is_available():
		return unavailable_state()
	return bridge.effective_risk(portal_id)


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


## B8. The bridge's canonical save JSON, forwarded verbatim. Empty means the
## bridge holds no campaign to save; the port adds no placeholder document.
func save_json() -> String:
	if not is_available():
		return ""
	return str(bridge.save_json())


## B8. A save document back into the bridge, forwarded verbatim. The bridge
## parses, validates and refuses it -- this side neither inspects nor repairs
## it, so the only gate a slot meets is the one the Rust save-migration tests
## hold. Refused, the returned dictionary is the same `configured: false` shape
## every other refusal on this boundary uses, and the bridge still holds
## whatever campaign it held before.
func load_json(json: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.load_json(json)
