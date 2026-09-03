extends SceneTree

## Runtime contract for the first connected expedition route. This is not a
## map mock-up: it confirms that Godot receives the same stable cells, portals
## and actionable choice that the content validator admits.

const EXPECTED_CELLS := [
	"world.cell.black_beach",
	"world.cell.damaged_estate",
	"world.cell.river_landing",
	"world.cell.reception_terrace",
	"world.cell.processional_ramp"
]


func _init() -> void:
	var catalog := ContentCatalog.new()
	assert(catalog.load_default() == OK)
	var region := catalog.get_record("world.region.black_beach")
	assert(region.get("worldCellIds", []) == EXPECTED_CELLS)
	for cell_id in EXPECTED_CELLS:
		var cell := catalog.get_record(cell_id)
		assert(cell.get("kind") == "world_cell")
		assert((cell.get("entryAnchors", []) as Array).size() >= 2)
		assert((cell.get("interactionAnchors", []) as Array).size() >= 2)
		assert((cell.get("explorationActions", []) as Array).size() >= 2)
		assert((cell.get("portals", []) as Array).size() >= 1)

	var river_landing := catalog.get_record("world.cell.river_landing")
	var route_choice: Array = (river_landing.get("explorationActions", []) as Array).filter(func(action: Dictionary) -> bool: return action.get("type") == "route_choice")
	assert(route_choice.size() == 1)
	assert(route_choice[0].get("choices", []).size() == 2)
	assert(route_choice[0].get("choices", []).has("world.portal.river_landing_to_reception_terrace_safe_road"))
	assert(route_choice[0].get("choices", []).has("world.portal.river_landing_to_reception_terrace_jungle_edge"))
	print("Black Beach route content contract passed.")
	quit(0)
