class_name ElizabethanPortTownSet
extends Node3D
## Production blockout for a complete early-modern Atlantic port town.
## Geometry is deliberately simple. Names, metre scale, district boundaries,
## routes, landmarks, anchors, and population slots are the replacement contract.

const PACK_ID := "resource.geometry.settlement.elizabethan-atlantic-port-town.v1"
const RECIPE_IDS := [
	"arrival_gate_and_watch",
	"market_green",
	"merchant_high_street",
	"craft_lane",
	"coaching_inn_courtyard",
	"martial_training_yard",
	"wilderness_lodge",
	"town_hall_and_exchange",
	"temple_close_and_memorial",
	"quay_and_customs",
	"shipwright_yard",
	"riverside_warehouse",
	"residential_close",
	"hidden_alley_loop",
]

var plaster: StandardMaterial3D
var oak: StandardMaterial3D
var roof: StandardMaterial3D
var slate: StandardMaterial3D
var stone: StandardMaterial3D
var cobble: StandardMaterial3D
var sand: StandardMaterial3D
var water: StandardMaterial3D
var green: StandardMaterial3D
var sea_green: StandardMaterial3D
var brass: StandardMaterial3D
var oxblood: StandardMaterial3D
var canvas_red: StandardMaterial3D
var canvas_gold: StandardMaterial3D
var canvas_blue: StandardMaterial3D
var dark: StandardMaterial3D


func _ready() -> void:
	set_meta("pack_id", PACK_ID)
	set_meta("production_state", "procedural_proof_blockout")
	set_meta("unit_scale", "1 Godot unit = 1 metre")
	set_meta("visual_authority", "resource-packs/geometry/settlement/elizabethan-atlantic-port-town-v1/README.md")
	set_meta("replacement_rule", "Replace named visual sections only; preserve routes, entrances, anchors, population slots and semantic IDs.")
	set_meta("art_budget_rule", "CC0 and Godot geometry first; inexpensive scene-wide paint passes second; expensive generation only for approved named characters.")
	_make_materials()
	_build_environment()
	_build_routes()
	_build_districts()
	_build_population_slots()
	_build_gameplay_anchors()


func _make_materials() -> void:
	plaster = SetpieceMeshFactory.material(Color("d8c49c"), 0.0, 0.94)
	oak = SetpieceMeshFactory.material(Color("3b2418"), 0.0, 0.86)
	roof = SetpieceMeshFactory.material(Color("71382d"), 0.02, 0.82)
	slate = SetpieceMeshFactory.material(Color("33464d"), 0.03, 0.78)
	stone = SetpieceMeshFactory.material(Color("69675d"), 0.0, 0.97)
	cobble = SetpieceMeshFactory.material(Color("4f5350"), 0.0, 0.93)
	sand = SetpieceMeshFactory.material(Color("a18b62"), 0.0, 1.0)
	water = SetpieceMeshFactory.material(Color("1b6871"), 0.22, 0.24, Color("174f58"), 0.12)
	green = SetpieceMeshFactory.material(Color("315937"), 0.0, 1.0)
	sea_green = SetpieceMeshFactory.material(Color("267d73"), 0.03, 0.7)
	brass = SetpieceMeshFactory.material(Color("ad7c32"), 0.72, 0.34)
	oxblood = SetpieceMeshFactory.material(Color("7e2f32"), 0.05, 0.72)
	canvas_red = SetpieceMeshFactory.material(Color("a63d3f"), 0.0, 0.88)
	canvas_gold = SetpieceMeshFactory.material(Color("d3a642"), 0.0, 0.9)
	canvas_blue = SetpieceMeshFactory.material(Color("366a91"), 0.0, 0.86)
	dark = SetpieceMeshFactory.material(Color("192326"), 0.05, 0.9)


func _build_environment() -> void:
	var section := _section("Environment", "lighting, sea plane, ground mass and town edge")
	SetpieceMeshFactory.box(section, "TownGround", Vector3(134.0, 0.5, 96.0), Vector3(0.0, -0.3, -8.0), green)
	SetpieceMeshFactory.box(section, "HarborWater", Vector3(134.0, 0.18, 45.0), Vector3(0.0, -0.48, 61.0), water)
	SetpieceMeshFactory.box(section, "ShoreBand", Vector3(132.0, 0.24, 8.0), Vector3(0.0, -0.25, 36.0), sand)
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color("8db4bd")
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color("b7cbd0")
	environment.ambient_light_energy = 0.75
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_exposure = 1.12
	var world := WorldEnvironment.new()
	world.name = "TownEnvironment"
	world.environment = environment
	section.add_child(world)
	var sun := DirectionalLight3D.new()
	sun.name = "SoftCoastalSun"
	sun.rotation_degrees = Vector3(-52.0, -34.0, 0.0)
	sun.light_color = Color("ffe6b5")
	sun.light_energy = 1.3
	sun.shadow_enabled = true
	section.add_child(sun)


func _build_routes() -> void:
	var section := _section("Routes", "primary road, market square, lanes, quay and discovered shortcut loop")
	_road(section, "NorthGateRoad", Vector3(0.0, 0.0, -31.0), Vector3(7.0, 0.18, 42.0))
	_road(section, "MerchantHighStreet", Vector3(0.0, 0.01, -2.0), Vector3(74.0, 0.2, 7.0))
	_road(section, "MarketSquare", Vector3(0.0, 0.02, 9.0), Vector3(28.0, 0.22, 22.0))
	_road(section, "QuayRoad", Vector3(0.0, 0.01, 30.0), Vector3(112.0, 0.22, 6.0))
	_road(section, "WestServiceLane", Vector3(-39.0, 0.0, 12.0), Vector3(4.5, 0.18, 40.0))
	_road(section, "EastServiceLane", Vector3(39.0, 0.0, 12.0), Vector3(4.5, 0.18, 40.0))
	_road(section, "HiddenAlley", Vector3(28.0, 0.04, -13.0), Vector3(2.2, 0.12, 27.0), dark)


func _build_districts() -> void:
	_build_arrival_gate(Vector3(0.0, 0.0, -49.0))
	_build_market_green(Vector3(0.0, 0.0, 9.0))
	_build_high_street(Vector3(-22.0, 0.0, -7.0))
	_build_craft_lane(Vector3(-43.0, 0.0, 7.0))
	_build_inn(Vector3(22.0, 0.0, -7.0))
	_build_training_yard(Vector3(-43.0, 0.0, -27.0))
	_build_wilderness_lodge(Vector3(43.0, 0.0, -29.0))
	_build_town_hall(Vector3(-19.0, 0.0, 18.0))
	_build_temple_close(Vector3(42.0, 0.0, 8.0))
	_build_quay_customs(Vector3(-18.0, 0.0, 32.0))
	_build_shipwright(Vector3(-48.0, 0.0, 47.0))
	_build_warehouse(Vector3(18.0, 0.0, 32.0))
	_build_residential(Vector3(40.0, 0.0, -9.0))
	_build_hidden_alley(Vector3(28.0, 0.0, -13.0))


func _build_arrival_gate(origin: Vector3) -> void:
	var d := _district("ArrivalGateAndWatch", "arrival_gate_and_watch", origin)
	for side in [-1.0, 1.0]:
		_house(d, "Gatehouse%s" % ("Left" if side < 0 else "Right"), Vector3(side * 5.1, 0.0, 0.0), Vector3(6.0, 7.0, 5.0), slate, sea_green)
		SetpieceMeshFactory.cylinder(d, "GateTurret%s" % side, 1.45, 1.7, 7.8, Vector3(side * 8.0, 3.9, 0.0), stone, 10)
	SetpieceMeshFactory.box(d, "GateLintel", Vector3(7.0, 1.0, 2.0), Vector3(0.0, 6.4, 0.0), oak)
	SetpieceMeshFactory.cylinder(d, "WatchBell", 0.45, 0.65, 0.8, Vector3(0.0, 8.0, 0.0), brass, 12)
	_sign(d, "TownMapAndNotices", Vector3(4.5, 1.4, 5.2), sea_green)


func _build_market_green(origin: Vector3) -> void:
	var d := _district("MarketGreen", "market_green", origin)
	SetpieceMeshFactory.box(d, "PublicGreen", Vector3(18.0, 0.16, 13.0), Vector3(0.0, 0.06, 0.0), green)
	SetpieceMeshFactory.cylinder(d, "WellBase", 2.0, 2.0, 0.75, Vector3(0.0, 0.45, 0.0), stone, 12)
	SetpieceMeshFactory.cylinder(d, "WellCurb", 1.45, 1.45, 1.0, Vector3(0.0, 0.9, 0.0), dark, 12)
	for index in range(6):
		var x := -10.5 + float(index % 3) * 10.5
		var z := -7.5 if index < 3 else 7.5
		_stall(d, "MarketStall%02d" % index, Vector3(x, 0.0, z), [canvas_red, canvas_gold, canvas_blue][index % 3])
	SetpieceMeshFactory.box(d, "SpeakerStep", Vector3(4.0, 0.6, 2.5), Vector3(7.0, 0.3, 0.0), stone)


func _build_high_street(origin: Vector3) -> void:
	var d := _district("MerchantHighStreet", "merchant_high_street", origin)
	for index in range(4):
		var x := -12.0 + float(index) * 8.0
		_house(d, "Shop%02d" % index, Vector3(x, 0.0, -5.0), Vector3(7.2, 5.5 + float(index % 2) * 1.2, 7.0), roof if index % 2 == 0 else slate, [oxblood, sea_green, canvas_blue][index % 3])
		_sign(d, "TradeSign%02d" % index, Vector3(x + 2.3, 2.5, -0.9), [brass, sea_green, oxblood][index % 3])


func _build_craft_lane(origin: Vector3) -> void:
	var d := _district("CraftLane", "craft_lane", origin)
	_house(d, "Smithy", Vector3(-5.0, 0.0, 0.0), Vector3(9.0, 5.0, 8.0), roof, oxblood)
	_house(d, "Herbalist", Vector3(5.5, 0.0, 1.0), Vector3(7.0, 4.8, 7.0), slate, sea_green)
	SetpieceMeshFactory.box(d, "SmithyApron", Vector3(9.0, 0.16, 5.0), Vector3(-5.0, 0.05, 6.0), cobble)
	SetpieceMeshFactory.cylinder(d, "Chimney", 0.65, 0.9, 4.5, Vector3(-7.4, 6.2, 0.0), stone, 8)
	_prop_cluster(d, "DeliveryCluster", Vector3(4.8, 0.0, 6.0))


func _build_inn(origin: Vector3) -> void:
	var d := _district("CoachingInnCourtyard", "coaching_inn_courtyard", origin)
	_house(d, "InnMainWing", Vector3(0.0, 0.0, -6.0), Vector3(18.0, 8.0, 7.0), roof, oxblood)
	_house(d, "StableWing", Vector3(8.0, 0.0, 3.0), Vector3(6.0, 4.0, 10.0), slate, oak)
	SetpieceMeshFactory.box(d, "LanternCourt", Vector3(14.0, 0.14, 11.0), Vector3(-1.0, 0.05, 3.0), cobble)
	for table_index in range(3):
		SetpieceMeshFactory.cylinder(d, "CourtyardTable%02d" % table_index, 0.8, 0.8, 0.12, Vector3(-5.0 + table_index * 4.0, 0.9, 2.5), oak, 12)
	_sign(d, "InnSign", Vector3(-6.0, 3.3, -1.8), brass)


func _build_training_yard(origin: Vector3) -> void:
	var d := _district("MartialTrainingYard", "martial_training_yard", origin)
	SetpieceMeshFactory.box(d, "SandDrillCourt", Vector3(22.0, 0.18, 17.0), Vector3(0.0, 0.06, 0.0), sand)
	_house(d, "OpenDrillHall", Vector3(-8.5, 0.0, -7.5), Vector3(10.0, 4.2, 5.0), slate, oxblood)
	for index in range(4):
		_training_dummy(d, "TrainingDummy%02d" % index, Vector3(-6.0 + index * 4.0, 0.0, 2.0))
	SetpieceMeshFactory.box(d, "FencingStrip", Vector3(13.0, 0.08, 2.0), Vector3(2.5, 0.16, -4.5), oxblood)
	SetpieceMeshFactory.box(d, "SpectatorRail", Vector3(13.0, 1.0, 0.15), Vector3(1.0, 0.6, 7.7), oak)


func _build_wilderness_lodge(origin: Vector3) -> void:
	var d := _district("WildernessLodge", "wilderness_lodge", origin)
	_house(d, "ScoutLodge", Vector3(0.0, 0.0, -3.0), Vector3(13.0, 5.5, 8.0), slate, sea_green)
	_sign(d, "TrailBoard", Vector3(-7.5, 1.5, 1.0), green)
	for index in range(3):
		_training_dummy(d, "ArcheryTarget%02d" % index, Vector3(-6.0 + index * 5.0, 0.0, 7.0))
	for index in range(7):
		_tree(d, "EdgeTree%02d" % index, Vector3(-10.0 + index * 3.2, 0.0, -8.0 - float(index % 2)))
	SetpieceMeshFactory.box(d, "AnimalPen", Vector3(7.0, 0.14, 5.0), Vector3(8.0, 0.07, 4.0), sand)


func _build_town_hall(origin: Vector3) -> void:
	var d := _district("TownHallAndExchange", "town_hall_and_exchange", origin)
	_house(d, "CouncilHall", Vector3(0.0, 0.0, 0.0), Vector3(16.0, 8.0, 10.0), slate, sea_green)
	for step in range(3):
		SetpieceMeshFactory.box(d, "FormalStep%02d" % step, Vector3(8.0 - step * 0.7, 0.25, 1.0), Vector3(0.0, 0.13 + step * 0.22, 5.4 + step * 0.65), stone)
	SetpieceMeshFactory.cylinder(d, "CouncilCupola", 1.4, 1.8, 3.0, Vector3(0.0, 9.4, 0.0), sea_green, 8)
	SetpieceMeshFactory.cylinder(d, "CupolaFinial", 0.08, 0.35, 1.4, Vector3(0.0, 11.6, 0.0), brass, 8)


func _build_temple_close(origin: Vector3) -> void:
	var d := _district("TempleCloseAndMemorial", "temple_close_and_memorial", origin)
	_house(d, "Chapel", Vector3(-3.0, 0.0, -3.0), Vector3(13.0, 7.0, 10.0), slate, plaster)
	SetpieceMeshFactory.cylinder(d, "BellTower", 2.0, 2.4, 10.0, Vector3(-8.0, 5.0, -3.0), stone, 8)
	SetpieceMeshFactory.box(d, "MemorialGarden", Vector3(15.0, 0.15, 8.0), Vector3(2.0, 0.05, 7.0), green)
	for index in range(6):
		SetpieceMeshFactory.box(d, "MemorialStone%02d" % index, Vector3(0.7, 1.2, 0.28), Vector3(-3.0 + index * 2.1, 0.65, 6.0 + float(index % 2) * 2.0), stone)


func _build_quay_customs(origin: Vector3) -> void:
	var d := _district("QuayAndCustoms", "quay_and_customs", origin)
	_house(d, "CustomsHouse", Vector3(0.0, 0.0, -3.0), Vector3(14.0, 6.0, 8.0), roof, sea_green)
	SetpieceMeshFactory.box(d, "WorkingQuay", Vector3(31.0, 0.5, 7.0), Vector3(0.0, -0.1, 8.0), oak)
	for index in range(6):
		SetpieceMeshFactory.cylinder(d, "Bollard%02d" % index, 0.22, 0.3, 0.8, Vector3(-13.0 + index * 5.2, 0.42, 11.0), dark, 8)
	_crane(d, "CargoCrane", Vector3(-10.0, 0.0, 5.0))
	_prop_cluster(d, "InspectedCargo", Vector3(7.0, 0.0, 5.0))


func _build_shipwright(origin: Vector3) -> void:
	var d := _district("ShipwrightYard", "shipwright_yard", origin)
	SetpieceMeshFactory.box(d, "RepairSlip", Vector3(19.0, 0.3, 20.0), Vector3(0.0, -0.1, 5.0), sand, Vector3(7.0, 0.0, 0.0))
	_house(d, "ShipShed", Vector3(-8.0, 0.0, -5.0), Vector3(11.0, 5.0, 8.0), slate, oxblood)
	for rib in range(7):
		SetpieceMeshFactory.box(d, "HullRib%02d" % rib, Vector3(0.28, 3.4, 7.0 - abs(3 - rib) * 0.55), Vector3(-6.0 + rib * 2.0, 1.8, 5.0), oak, Vector3(0.0, 0.0, -12.0 + rib * 4.0))
	SetpieceMeshFactory.box(d, "Keel", Vector3(16.0, 0.45, 0.5), Vector3(0.0, 0.5, 5.0), oak)
	SetpieceMeshFactory.cylinder(d, "BoilerPrototype", 1.2, 1.2, 3.8, Vector3(8.0, 1.2, -4.0), brass, 12, Vector3(0.0, 0.0, 90.0))


func _build_warehouse(origin: Vector3) -> void:
	var d := _district("RiversideWarehouse", "riverside_warehouse", origin)
	_house(d, "BondedWarehouse", Vector3(0.0, 0.0, 0.0), Vector3(19.0, 8.0, 11.0), roof, oxblood)
	_crane(d, "RiverHoist", Vector3(-9.0, 0.0, 6.0))
	_prop_cluster(d, "CargoCourt", Vector3(6.5, 0.0, 7.0))
	SetpieceMeshFactory.box(d, "SecureCage", Vector3(5.0, 2.4, 4.0), Vector3(-4.5, 1.2, 7.0), dark)


func _build_residential(origin: Vector3) -> void:
	var d := _district("ResidentialClose", "residential_close", origin)
	for index in range(4):
		var x := -9.0 + float(index % 2) * 12.0
		var z := -6.0 + float(index / 2) * 12.0
		_house(d, "House%02d" % index, Vector3(x, 0.0, z), Vector3(9.0, 5.5 + float(index % 3), 8.0), roof if index % 2 == 0 else slate, [canvas_blue, oxblood, sea_green][index % 3])
	for line in range(3):
		SetpieceMeshFactory.box(d, "LaundryLine%02d" % line, Vector3(8.0, 0.05, 0.05), Vector3(-4.0, 2.7 + line * 0.35, -1.5 + line * 1.4), dark, Vector3(0.0, 18.0 - line * 18.0, 0.0))
		for cloth in range(3):
			SetpieceMeshFactory.box(d, "Laundry%02d_%02d" % [line, cloth], Vector3(1.3, 1.0, 0.05), Vector3(-6.2 + cloth * 2.3, 2.2 + line * 0.35, -2.2 + line * 1.4), [canvas_red, canvas_gold, canvas_blue][cloth])


func _build_hidden_alley(origin: Vector3) -> void:
	var d := _district("HiddenAlleyLoop", "hidden_alley_loop", origin)
	SetpieceMeshFactory.box(d, "DrainChannel", Vector3(1.0, 0.1, 23.0), Vector3(0.0, 0.03, 0.0), dark)
	SetpieceMeshFactory.box(d, "CoveredBridge", Vector3(7.0, 3.0, 2.8), Vector3(0.0, 6.0, 0.0), oak)
	SetpieceMeshFactory.box(d, "ConcealedCellarDoor", Vector3(1.6, 2.0, 0.18), Vector3(-2.4, 1.0, 8.0), sea_green, Vector3(0.0, 90.0, 0.0))
	_prop_cluster(d, "ContrabandCache", Vector3(2.0, 0.0, -7.0))


func _build_population_slots() -> void:
	var section := _section("PopulationSlots", "role-based character placement; no named character appearance authority")
	var slots := [
		["watch_captain", Vector3(2, 0, -44), "guard"], ["town_crier", Vector3(6, 0, 9), "civic"],
		["market_vendor", Vector3(-9, 0, 13), "merchant"], ["innkeeper", Vector3(17, 0, -2), "hospitality"],
		["martial_instructor", Vector3(-43, 0, -27), "trainer"], ["scoutmaster", Vector3(43, 0, -29), "trainer"],
		["harbormaster", Vector3(-19, 0, 35), "maritime"], ["shipwright", Vector3(-49, 0, 43), "craft"],
		["memorial_keeper", Vector3(43, 0, 14), "faith"], ["smuggler_contact", Vector3(30, 0, -18), "illicit"],
	]
	for record in slots:
		var marker := Marker3D.new()
		marker.name = "%sSlot" % str(record[0]).to_pascal_case()
		marker.position = record[1]
		marker.set_meta("role_id", record[0])
		marker.set_meta("role_family", record[2])
		marker.set_meta("rig_contract", "humanoid_1p75m; ground-contact feet; forward -Z; root/pelvis/head/hand_l/hand_r/weapon/prop sockets")
		marker.set_meta("appearance_authority", "none; resolve an approved character or population archetype before runtime")
		section.add_child(marker)


func _build_gameplay_anchors() -> void:
	var section := _section("GameplayAnchors", "semantic positions only; gameplay systems retain authority")
	var anchors := [
		["TownArrival", Vector3(0, 0, -43), "arrival_spawn"], ["MarketQuestBoard", Vector3(8, 0, 11), "quest_board"],
		["MartialTrainer", Vector3(-43, 0, -31), "trainer"], ["WildernessTrailExit", Vector3(49, 0, -36), "outer_route"],
		["HarborTravel", Vector3(-18, 0, 42), "boat_travel"], ["MidnightMemorial", Vector3(44, 0, 16), "night_respawn_flash"],
		["HiddenRouteEntry", Vector3(28, 0, -23), "secret_discovery"], ["ShipUpgrade", Vector3(-40, 0, 43), "steampunk_upgrade"],
	]
	for record in anchors:
		var marker := Marker3D.new()
		marker.name = record[0]
		marker.position = record[1]
		marker.set_meta("anchor_id", record[2])
		marker.set_meta("visual_only", true)
		section.add_child(marker)


func _section(section_name: String, scope: String) -> Node3D:
	var section := Node3D.new()
	section.name = section_name
	section.set_meta("replacement_scope", scope)
	add_child(section)
	return section


func _district(section_name: String, recipe_id: String, origin: Vector3) -> Node3D:
	var districts := get_node_or_null("Districts") as Node3D
	if districts == null:
		districts = _section("Districts", "fourteen recipe-driven functional town sets")
	var district := Node3D.new()
	district.name = section_name
	district.position = origin
	district.set_meta("recipe_id", recipe_id)
	district.set_meta("replacement_scope", "visual geometry for %s; preserve district origin, entrances, routes and anchors" % recipe_id)
	districts.add_child(district)
	return district


func _road(parent: Node3D, road_name: String, location: Vector3, size: Vector3, surface: Material = null) -> void:
	SetpieceMeshFactory.box(parent, road_name, size, location, cobble if surface == null else surface)


func _house(parent: Node3D, house_name: String, location: Vector3, size: Vector3, roof_surface: Material, accent: Material) -> Node3D:
	var house := Node3D.new()
	house.name = house_name
	house.position = location
	house.set_meta("replacement_scope", "one complete building shell; doors and footprint are binding")
	parent.add_child(house)
	SetpieceMeshFactory.box(house, "PlasterMass", size, Vector3(0.0, size.y * 0.5, 0.0), plaster)
	for floor_index in range(1, int(size.y / 2.6) + 1):
		SetpieceMeshFactory.box(house, "FloorBeam%02d" % floor_index, Vector3(size.x + 0.25, 0.22, size.z + 0.25), Vector3(0.0, floor_index * 2.6, 0.0), oak)
	for side in [-1.0, 1.0]:
		SetpieceMeshFactory.box(house, "CornerPost%sA" % side, Vector3(0.28, size.y, 0.28), Vector3(side * size.x * 0.48, size.y * 0.5, -size.z * 0.47), oak)
		SetpieceMeshFactory.box(house, "CornerPost%sB" % side, Vector3(0.28, size.y, 0.28), Vector3(side * size.x * 0.48, size.y * 0.5, size.z * 0.47), oak)
	SetpieceMeshFactory.box(house, "RoofLeft", Vector3(size.x * 0.62, 0.35, size.z + 0.8), Vector3(-size.x * 0.24, size.y + 1.0, 0.0), roof_surface, Vector3(0.0, 0.0, -24.0))
	SetpieceMeshFactory.box(house, "RoofRight", Vector3(size.x * 0.62, 0.35, size.z + 0.8), Vector3(size.x * 0.24, size.y + 1.0, 0.0), roof_surface, Vector3(0.0, 0.0, 24.0))
	SetpieceMeshFactory.box(house, "FrontDoor", Vector3(1.25, 2.1, 0.16), Vector3(0.0, 1.05, size.z * 0.51), accent)
	for window_index in [-1.0, 1.0]:
		SetpieceMeshFactory.box(house, "FrontWindow%s" % window_index, Vector3(1.1, 1.35, 0.12), Vector3(window_index * size.x * 0.28, min(size.y - 1.2, 3.3), size.z * 0.515), accent)
	return house


func _stall(parent: Node3D, stall_name: String, location: Vector3, canopy: Material) -> void:
	var stall := Node3D.new()
	stall.name = stall_name
	stall.position = location
	parent.add_child(stall)
	SetpieceMeshFactory.box(stall, "Counter", Vector3(3.5, 0.25, 1.6), Vector3(0.0, 1.0, 0.0), oak)
	for side in [-1.0, 1.0]:
		SetpieceMeshFactory.box(stall, "Post%s" % side, Vector3(0.13, 2.8, 0.13), Vector3(side * 1.55, 1.4, 0.0), oak)
	SetpieceMeshFactory.box(stall, "Canopy", Vector3(3.9, 0.16, 2.4), Vector3(0.0, 2.85, 0.0), canopy, Vector3(0.0, 0.0, 4.0))


func _sign(parent: Node3D, sign_name: String, location: Vector3, surface: Material) -> void:
	SetpieceMeshFactory.box(parent, "%sPost" % sign_name, Vector3(0.15, 2.8, 0.15), location - Vector3(0.0, 1.4, 0.0), oak)
	SetpieceMeshFactory.box(parent, sign_name, Vector3(1.7, 1.0, 0.18), location, surface)


func _training_dummy(parent: Node3D, dummy_name: String, location: Vector3) -> void:
	var dummy := Node3D.new()
	dummy.name = dummy_name
	dummy.position = location
	dummy.set_meta("standin_for", "training target or archery butt; simple geometry is semantically correct")
	parent.add_child(dummy)
	SetpieceMeshFactory.cylinder(dummy, "Post", 0.12, 0.16, 2.2, Vector3(0.0, 1.1, 0.0), oak, 8)
	SetpieceMeshFactory.box(dummy, "Target", Vector3(1.3, 1.4, 0.28), Vector3(0.0, 1.8, 0.0), canvas_red)


func _tree(parent: Node3D, tree_name: String, location: Vector3) -> void:
	var tree := Node3D.new()
	tree.name = tree_name
	tree.position = location
	parent.add_child(tree)
	SetpieceMeshFactory.cylinder(tree, "Trunk", 0.25, 0.4, 4.3, Vector3(0.0, 2.15, 0.0), oak, 7)
	SetpieceMeshFactory.sphere(tree, "CanopyA", Vector3(2.0, 1.5, 1.7), Vector3(0.0, 4.9, 0.0), green)
	SetpieceMeshFactory.sphere(tree, "CanopyB", Vector3(1.4, 1.1, 1.3), Vector3(1.2, 4.3, 0.4), green)


func _prop_cluster(parent: Node3D, cluster_name: String, location: Vector3) -> void:
	var cluster := Node3D.new()
	cluster.name = cluster_name
	cluster.position = location
	cluster.set_meta("replacement_scope", "cargo activity cluster: crates, barrels, rope and one clear working aisle")
	parent.add_child(cluster)
	SetpieceMeshFactory.box(cluster, "CrateA", Vector3(1.1, 1.1, 1.1), Vector3(0.0, 0.55, 0.0), oak)
	SetpieceMeshFactory.box(cluster, "CrateB", Vector3(0.8, 0.8, 0.8), Vector3(0.9, 0.4, 0.3), oak, Vector3(0.0, 14.0, 0.0))
	SetpieceMeshFactory.cylinder(cluster, "Barrel", 0.48, 0.56, 1.1, Vector3(-1.0, 0.55, 0.2), oxblood, 10)


func _crane(parent: Node3D, crane_name: String, location: Vector3) -> void:
	var crane := Node3D.new()
	crane.name = crane_name
	crane.position = location
	parent.add_child(crane)
	SetpieceMeshFactory.box(crane, "Mast", Vector3(0.45, 7.0, 0.45), Vector3(0.0, 3.5, 0.0), oak)
	SetpieceMeshFactory.box(crane, "Boom", Vector3(6.5, 0.35, 0.35), Vector3(2.6, 6.3, 0.0), oak, Vector3(0.0, 0.0, 12.0))
	SetpieceMeshFactory.cylinder(crane, "Hoist", 0.5, 0.5, 0.6, Vector3(5.1, 5.0, 0.0), brass, 10, Vector3(90.0, 0.0, 0.0))
