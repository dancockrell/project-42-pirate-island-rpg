class_name ReceptionTerraceComponent
extends Node3D
## One reusable construction script for several named scene components. The
## scene file, not a string convention, declares which environment concern is
## present at a location.

@export_enum("processional_terrace", "elven_gate", "jungle_mass", "shipwreck_flotsam", "sea_and_sky") var component_id := "processional_terrace"


## Wet stone in a named colour. The render foundation owns how rain reads on
## stone; this names which stone it is raining on. Terrace slabs hold more
## water than the gate does, because the terrace is what people walk on.
func _stone(dry: Color, wet: Color, wetness: float) -> ShaderMaterial:
	var surface := SetpieceMeshFactory.library_material(SetpieceMeshFactory.WET_STONE)
	surface.set_shader_parameter("dry_color", dry)
	surface.set_shader_parameter("wet_color", wet)
	surface.set_shader_parameter("wetness", wetness)
	return surface


## Leaf mass that moves with the global wind.
func _leaf(base: Color, tip: Color) -> ShaderMaterial:
	var surface := SetpieceMeshFactory.library_material(SetpieceMeshFactory.FOLIAGE)
	surface.set_shader_parameter("albedo", base)
	surface.set_shader_parameter("tip_color", tip)
	return surface


func _ready() -> void:
	match component_id:
		"processional_terrace":
			_build_processional_terrace()
		"elven_gate":
			_build_elven_gate()
		"jungle_mass":
			_build_jungle_mass()
		"shipwreck_flotsam":
			_build_shipwreck_flotsam()
		"sea_and_sky":
			_build_sea_and_sky()
		_:
			push_error("Unknown Reception Terrace component: %s" % component_id)


func _build_processional_terrace() -> void:
	set_meta("replacement_scope", "ground slabs, broken steps, edge fragments and luminous seam inlays")
	var dark_stone := _stone(Color("27332d"), Color("10181a"), 0.55)
	var light_stone := _stone(Color("61705f"), Color("2a3733"), 0.8)
	var teal_inlay := SetpieceMeshFactory.material(Color("1b8c79"), 0.25, 0.35, Color("1b8c79"), 1.45)
	SetpieceMeshFactory.box(self, "TerraceGround", Vector3(24.0, 0.55, 15.0), Vector3(0.0, -0.28, 0.0), dark_stone)
	for row in range(4):
		for column in range(7):
			var x := -8.7 + float(column) * 2.9
			var z := -4.2 + float(row) * 2.85
			var lift := 0.18 if (row + column) % 3 == 0 else 0.0
			SetpieceMeshFactory.box(self, "Slab_%02d_%02d" % [row, column], Vector3(2.58, 0.18, 2.5), Vector3(x, lift, z), light_stone, Vector3(0.0, float(((row * 17 + column * 11) % 5) - 2), 0.0))
	for step in range(4):
		SetpieceMeshFactory.box(self, "BrokenStep_%02d" % step, Vector3(11.0 - step * 0.75, 0.42, 0.9), Vector3(0.0, 0.18 + step * 0.34, -6.6 - step * 0.65), light_stone)
	for side in [-1.0, 1.0]:
		for index in range(4):
			SetpieceMeshFactory.box(self, "EdgeFragment_%s_%02d" % ["L" if side < 0.0 else "R", index], Vector3(0.72, 0.58 + index * 0.13, 2.1), Vector3(side * (10.0 + float(index % 2) * 0.55), 0.2, -4.6 + float(index) * 3.15), dark_stone, Vector3(0.0, 0.0, side * (7.0 + index * 3.0)))
	for seam in range(5):
		var location := Vector3(-6.5 + seam * 3.25, 0.01, 1.1)
		SetpieceMeshFactory.box(self, "SeamInlay_%02d" % seam, Vector3(0.12, 0.035, 2.0), location, teal_inlay)
		var light := OmniLight3D.new()
		light.name = "LuminousSeam_%02d" % seam
		light.position = location + Vector3(0.0, 0.15, 0.0)
		light.light_color = Color("1b8c79")
		light.light_energy = 0.62
		light.omni_range = 2.6
		add_child(light)


func _build_elven_gate() -> void:
	set_meta("replacement_scope", "monumental bronze-age gate, carved pillars, broken lintel and teal magical inlay")
	position = Vector3(-0.4, 0.0, -8.45)
	var gate_stone := _stone(Color("39483f"), Color("17211d"), 0.4)
	var carved_stone := _stone(Color("69755f"), Color("2e372c"), 0.5)
	var rune_surface := SetpieceMeshFactory.material(Color("1b8c79"), 0.2, 0.25, Color("1b8c79"), 1.2)
	for side in [-1.0, 1.0]:
		var pillar := Node3D.new()
		pillar.name = "GatePillar_%s" % ("Left" if side < 0.0 else "Right")
		pillar.position = Vector3(side * 4.35, 0.0, 0.0)
		add_child(pillar)
		SetpieceMeshFactory.box(pillar, "Base", Vector3(2.05, 1.05, 2.15), Vector3(0.0, 0.52, 0.0), gate_stone)
		SetpieceMeshFactory.cylinder(pillar, "CarvedColumn", 0.76, 0.76, 8.2, Vector3(0.0, 4.6, 0.0), carved_stone, 8)
		SetpieceMeshFactory.box(pillar, "Crown", Vector3(1.75, 0.74, 1.7), Vector3(0.0, 8.95, 0.0), gate_stone, Vector3(0.0, 0.0, side * 5.0))
		for rune in range(3):
			var location := Vector3(-side * 0.74, 2.6 + rune * 1.65, -0.74)
			SetpieceMeshFactory.box(pillar, "RunePlate_%02d" % rune, Vector3(0.32, 0.42, 0.08), location, rune_surface)
			var light := OmniLight3D.new()
			light.name = "Rune_%02d" % rune
			light.position = location
			light.light_color = Color("1b8c79")
			light.light_energy = 0.24
			light.omni_range = 1.25
			pillar.add_child(light)
	SetpieceMeshFactory.box(self, "FracturedLintel", Vector3(8.25, 1.16, 1.25), Vector3(-0.45, 8.7, 0.0), gate_stone, Vector3(0.0, 0.0, -4.0))
	SetpieceMeshFactory.box(self, "FallenLintelHalf", Vector3(4.3, 0.82, 1.1), Vector3(4.25, 0.48, 1.75), gate_stone, Vector3(7.0, 0.0, -24.0))
	SetpieceMeshFactory.box(self, "ReceptionDais", Vector3(7.9, 0.35, 2.0), Vector3(-0.45, 0.18, 1.75), carved_stone)


func _build_jungle_mass() -> void:
	set_meta("replacement_scope", "large foliage clumps, palm trunks, roots, vine curtains and cliffside silhouette")
	var bark := SetpieceMeshFactory.material(Color("4b3823"), 0.0, 0.95)
	var dark_leaf := _leaf(Color("183c2f"), Color("27664c"))
	var light_leaf := _leaf(Color("2d6842"), Color("4b8c5c"))
	var placements: Array[Vector3] = [Vector3(-12.6, 1.1, -8.0), Vector3(-14.2, 0.8, -2.8), Vector3(-13.1, 1.0, 3.7), Vector3(12.7, 1.0, -8.3), Vector3(14.1, 0.8, -2.2), Vector3(12.8, 1.0, 4.4), Vector3(-9.8, 1.2, 8.2), Vector3(10.3, 1.2, 8.4), Vector3(-7.7, 1.0, -10.7), Vector3(7.9, 1.0, -10.6)]
	for index in range(placements.size()):
		var clump := Node3D.new()
		clump.name = "FoliageClump_%02d" % index
		clump.position = placements[index]
		add_child(clump)
		var height := 4.2 + float(index % 3) * 1.1
		SetpieceMeshFactory.cylinder(clump, "PalmTrunk", 0.23, 0.34, height, Vector3(0.0, height * 0.5, 0.0), bark, 7)
		for crown in range(4):
			var offset := Vector3(float((crown % 2) * 2 - 1) * 0.8, height + 0.15 + float(crown % 2) * 0.35, float((crown / 2) * 2 - 1) * 0.8)
			SetpieceMeshFactory.sphere(clump, "LeafMass_%02d" % crown, Vector3(1.75, 0.92, 1.75), offset, dark_leaf if (index + crown) % 2 == 0 else light_leaf)
		SetpieceMeshFactory.sphere(clump, "Understory", Vector3(2.5, 1.1, 2.2), Vector3(0.0, 0.62, 0.0), light_leaf)
	for vine in range(8):
		SetpieceMeshFactory.cylinder(self, "VineCurtain_%02d" % vine, 0.045, 0.075, 2.2 + float(vine % 3) * 0.65, Vector3(-8.8 + vine * 2.45, 7.2, -8.5), dark_leaf, 6, Vector3(0.0, 0.0, float((vine % 3) - 1) * 4.0))


func _build_shipwreck_flotsam() -> void:
	set_meta("replacement_scope", "Handsome Jack hull ribs, tea crates, boiler casing, broken mast timber and foam-facing debris")
	position = Vector3(8.8, -0.1, 7.2)
	var wood := SetpieceMeshFactory.material(Color("3b2719"), 0.0, 0.88)
	var crate := SetpieceMeshFactory.material(Color("604126"), 0.0, 0.86)
	var brass := SetpieceMeshFactory.library_material(SetpieceMeshFactory.BRONZE)
	brass.set_shader_parameter("metal_color", Color("8e622a"))
	for plank in range(7):
		SetpieceMeshFactory.box(self, "HullPlank_%02d" % plank, Vector3(4.4 - float(plank % 3) * 0.38, 0.24, 0.52), Vector3(-1.7 + float(plank % 2) * 1.2, 0.38 + float(plank % 3) * 0.3, -1.1 + plank * 0.75), wood, Vector3(float((plank % 4) * 8 - 10), float(plank * 13), float((plank % 3) * 9 - 7)))
	for tea_crate in range(3):
		SetpieceMeshFactory.box(self, "TeaCrate_%02d" % tea_crate, Vector3(0.95, 0.85, 0.95), Vector3(2.0 + tea_crate * 0.72, 0.43 + float(tea_crate % 2) * 0.36, 1.2 - tea_crate * 0.48), crate, Vector3(0.0, tea_crate * 17.0, 0.0))
	SetpieceMeshFactory.cylinder(self, "BoilerCasing", 0.8, 0.8, 3.25, Vector3(-1.65, 0.9, 2.25), brass, 12, Vector3(0.0, 0.0, 90.0))
	SetpieceMeshFactory.cylinder(self, "LooseBarrel", 0.46, 0.55, 1.08, Vector3(3.6, 0.54, -0.55), wood, 10, Vector3(0.0, 0.0, 77.0))
	SetpieceMeshFactory.box(self, "BrokenMast", Vector3(0.34, 0.32, 8.8), Vector3(0.85, 0.68, 0.2), wood, Vector3(0.0, 22.0, 68.0))
	SetpieceMeshFactory.box(self, "SteamPipe", Vector3(2.6, 0.19, 0.19), Vector3(-2.7, 0.65, 1.2), brass, Vector3(0.0, 0.0, -18.0))


func _build_sea_and_sky() -> void:
	set_meta("replacement_scope", "downhill sea plane, coast silhouette, storm break and distant ruin mass")
	var sand := _stone(Color("8d7650"), Color("40351f"), 0.65)
	var sea := SetpieceMeshFactory.library_material(SetpieceMeshFactory.SEA)
	var islands := SetpieceMeshFactory.material(Color("173d36"), 0.0, 1.0)
	SetpieceMeshFactory.box(self, "DownhillSand", Vector3(16.0, 0.22, 13.0), Vector3(10.0, -0.45, 11.0), sand, Vector3(3.0, 0.0, -7.0))
	SetpieceMeshFactory.box(self, "SeaPlane", Vector3(28.0, 0.1, 18.0), Vector3(11.0, -0.7, 18.0), sea, Vector3(0.0, -5.0, 0.0))
	for island in range(4):
		SetpieceMeshFactory.sphere(self, "DistantIsland_%02d" % island, Vector3(3.7 + island, 2.1 + float(island % 2), 2.8), Vector3(-6.8 + island * 5.4, 0.85, 22.5 + float(island % 2) * 3.2), islands)
