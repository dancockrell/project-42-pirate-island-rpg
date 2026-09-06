class_name NativeExpeditionPort
extends ExpeditionPort

## The only Godot-facing owner of Project42ExpeditionBridge. The content
## catalog supplies the authored route, while Rust owns state mutation and
## legal-command calculation.

const BRIDGE_CLASS := "Project42ExpeditionBridge"
const INITIAL_PARTY := ["character.protagonist.captain", "character.heroine.betty"]
const INITIAL_LOCATION_ID := "world.cell.black_beach"

var bridge: Object
## The catalog the last `configure_from_catalog` read. Kept so the authored
## board blocks can be forwarded from one place; see `board_for`.
var catalog: ContentCatalog


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
	self.catalog = catalog
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
		# B7: the cell's battle entries, forwarded rather than judged here.
		# This used to send a single `encounter_eligible` bool that GDScript
		# derived from `status == "vertical_slice_encounter"`, which meant the
		# eligibility rule lived on this side and the tomb's service passage --
		# whose fight is the terrace precinct's habitat holder, not a one-time
		# authored encounter -- had no way to say so. The rule now has one
		# owner, `AuthoredCell::encounter_eligible` in Rust, and this forwards
		# the two authored fields it reads.
		var battle_entries: Array[Dictionary] = []
		for entry in cell.get("battleEntries", []):
			var forwarded_entry := {
				"id": str(entry.get("id", "")),
				"status": str(entry.get("status", ""))
			}
			var habitat_id := str(entry.get("habitatId", ""))
			if not habitat_id.is_empty():
				forwarded_entry["habitat_id"] = habitat_id
			battle_entries.append(forwarded_entry)
		# A7: C5's dungeonContext, forwarded rather than judged here. Two fields
		# of it reach the simulation -- the site rules in force in this cell and
		# the dungeon the cell belongs to -- and `AuthoredCell` reads both. The
		# rest of the block is `strategy/dungeon_content.rs`'s business. Without
		# this the engine fought under no site rules while the Rust harness
		# fought under the tomb's, which is the same drift B15 and B16 closed
		# for factions and buildings.
		var forwarded_cell := {
			"id": str(cell.get("id", "")),
			"region_id": str(cell.get("regionId", "")),
			"display_name": str(cell.get("displayName", "")),
			"observation_ids": observation_ids,
			"anchors": anchors,
			"battle_entries": battle_entries
		}
		var dungeon_context: Dictionary = cell.get("dungeonContext", {})
		if not dungeon_context.is_empty():
			var forwarded_context := {"site_rule_ids": dungeon_context.get("siteRuleIds", [])}
			var dungeon_id := str(dungeon_context.get("dungeonId", ""))
			if not dungeon_id.is_empty():
				forwarded_context["dungeon_id"] = dungeon_id
			forwarded_cell["dungeon_context"] = forwarded_context
		cells.append(forwarded_cell)
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
	# B16: C10's building records, forwarded the same way and for the same
	# reason. They are authored in BuildingDefinition's own field names, and
	# every number in one is a whole count -- footprint cells, construction
	# hours, hit points -- so the same integer coercion the factions need
	# applies here unchanged. Without this the engine ran S10's hourly
	# elimination sweep against an empty registry while the Rust harness ran it
	# against the loaded one, and the two islands disagreed about what a
	# building makes and how much of a cell it takes.
	var buildings: Array[Dictionary] = []
	for building_id in catalog.ids_with_prefix("building."):
		buildings.append(as_rust_integers(catalog.get_record(building_id)) as Dictionary)
	# B19: C14's machine records, forwarded the same way and for the same
	# reason. They are authored in MachineDefinition's own field names, and
	# every number in one is a whole count -- footprint and clearance cells, a
	# crew requirement, fuel and water, a salvage value -- so the same integer
	# coercion the factions and buildings need applies here unchanged. Without
	# this the engine ran S16's production hour against an empty registry: a
	# machine rule that came due journaled a skip naming a record it could not
	# find, while the Rust harness made the dog out of the same yard.
	var machines: Array[Dictionary] = []
	for machine_id in catalog.ids_with_prefix("machine."):
		machines.append(as_rust_integers(catalog.get_record(machine_id)) as Dictionary)
	# A7: the authored site-rule records, forwarded verbatim. They are authored
	# in `AuthoredSiteRule`'s own field names -- `id`, `displayName`, `effect` --
	# and the one number in an effect is a whole count, so they take the same
	# integer coercion the factions and buildings need. A cell's siteRuleIds
	# resolve against this registry; without it `Battle` stands under nothing.
	var site_rules: Array[Dictionary] = []
	for site_rule_id in catalog.ids_with_prefix("site_rule."):
		site_rules.append(as_rust_integers(catalog.get_record(site_rule_id)) as Dictionary)
	var configuration := {
		"seed": seed,
		"party_ids": INITIAL_PARTY,
		"active_location_id": INITIAL_LOCATION_ID,
		"cells": cells,
		"portals": portals,
		"encounter_triggers": encounter_triggers,
		"factions": factions,
		"buildings": buildings,
		"machines": machines,
		"site_rules": site_rules
	}
	var configured: Dictionary = bridge.configure(JSON.stringify(configuration))
	# A refused configuration used to be silent here: every later call answered
	# `expedition_not_configured` and the suite that noticed was several steps
	# downstream of the record that caused it. The bridge already names the
	# reason; say it once, where the payload was built.
	if not bool(configured.get("configured", false)):
		push_error("Native expedition bridge refused the catalog configuration: %s (%d cells, %d portals, %d factions, %d buildings, %d machines)" % [
			str(configured.get("error", "")), cells.size(), portals.size(), factions.size(), buildings.size(), machines.size()
		])
	return configured


## Every number in a forwarded record, back to an integer.
##
## No forwarded record type -- `FactionDefinition`, `BuildingDefinition`,
## `MachineDefinition`, `AuthoredSiteRule` -- has a floating-point field: every
## number in one is an `i16` weight or a whole count -- footprint cells,
## construction hours, hit points, a crew requirement, fuel and water -- and the
## authored records write them as integers. Godot's JSON
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


## S18. Raises one offscreen force, forwarded verbatim. `composition` is
## {actor id: count}; the bridge floors each value, the same coercion every
## other number crossing here gets. The returned snapshot carries the
## `force_raised` journal line in its `events` array; a refusal is the bridge's
## own error dictionary naming the force module's error.
func raise_force(force_id: String, faction_id: String, cell_id: String, composition: Dictionary, assignment: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.raise_force(force_id, faction_id, cell_id, composition, assignment)


## S18. Sends a raised force at a cell, planning the whole road now, so an order
## that cannot arrive is refused here rather than accepted and never completed.
## The snapshot's `events` carries the `force_departed` line.
func dispatch_force(force_id: String, destination_cell_id: String) -> Dictionary:
	if not is_available():
		return unavailable_state()
	return bridge.dispatch_force(force_id, destination_cell_id)


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


## B11's `board` block for one cell, forwarded from the catalog in the wire
## shape the isometric board reads: snake_case keys, and the footprint resolved
## from `presentation.board.registry` to actual metres exactly once, here.
##
## This is the only place the authored board block is read. The board scene does
## not reach into the catalog for it, because the footprint's metres live behind
## a size ID while O3 is open and two readers would mean two resolutions of it.
## Nothing about a board block crosses into Rust: the simulation owns where the
## party is, and this owns what that place looks like.
func board_for(cell_id: String) -> Dictionary:
	if catalog == null or not catalog.has(cell_id):
		return {}
	var cell := catalog.get_record(cell_id)
	var board: Dictionary = cell.get("board", {})
	if board.is_empty():
		return {}
	var footprint: Dictionary = board.get("footprint", {})
	var size_id := str(footprint.get("sizeId", ""))
	if not catalog.has_registry_entry(size_id):
		push_error("World cell %s names board footprint %s, which content/presentation/board.registry.json does not declare." % [cell_id, size_id])
		return {}
	var size := catalog.get_registry_entry(size_id)
	var spawn_points: Array[Dictionary] = []
	for spawn in board.get("spawnPoints", []):
		spawn_points.append({
			"id": str(spawn.get("id", "")),
			"role": str(spawn.get("role", "")),
			"rig_socket": str(spawn.get("rigSocket", "")),
			"position_metres": as_vector3(spawn.get("positionMetres", []))
		})
	var tethers: Array[Dictionary] = []
	for tether in board.get("tethers", []):
		tethers.append({
			"portal_id": str(tether.get("portalId", "")),
			"kind": str(tether.get("kind", "")),
			"anchor_metres": as_vector3(tether.get("anchorMetres", []))
		})
	var island: Array = board.get("islandPositionMetres", [0, 0])
	return {
		"cell_id": cell_id,
		"display_name": str(cell.get("displayName", cell_id)),
		"footprint": {
			"size_id": size_id,
			"width_metres": float(size.get("widthMetres", 0)),
			"depth_metres": float(size.get("depthMetres", 0)),
			"needs_decision": str(size.get("needsDecision", ""))
		},
		"island_position_metres": Vector2(float(island[0]), float(island[1])),
		"elevation_metres": float(board.get("elevationMetres", 0.0)),
		"spawn_points": spawn_points,
		"tethers": tethers
	}


## Three authored metres as a Vector3. Authored positions are always three
## numbers; anything else is a content error the validator already refuses, so
## this returns the origin rather than inventing a repair.
static func as_vector3(metres: Variant) -> Vector3:
	if not (metres is Array) or (metres as Array).size() != 3:
		return Vector3.ZERO
	var values: Array = metres
	return Vector3(float(values[0]), float(values[1]), float(values[2]))
