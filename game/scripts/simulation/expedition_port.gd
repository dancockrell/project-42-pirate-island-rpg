class_name ExpeditionPort
extends RefCounted

## Narrow presentation boundary for campaign travel. It does not own a second
## route state and it never derives legal exits from scene geometry.

func configure_from_catalog(_catalog: ContentCatalog, _seed: int) -> Dictionary:
	push_error("ExpeditionPort.configure_from_catalog is abstract")
	return {"configured": false, "error": "abstract_port"}


func snapshot() -> Dictionary:
	push_error("ExpeditionPort.snapshot is abstract")
	return {"configured": false, "error": "abstract_port"}


func travel(_portal_id: String) -> Dictionary:
	push_error("ExpeditionPort.travel is abstract")
	return {"configured": false, "error": "abstract_port"}


## B8. The campaign as its canonical save document, and back again. The port
## never reads the shape it carries: the simulation owns the save format, so a
## slot is bytes to Godot and a state only to Rust.
func save_json() -> String:
	push_error("ExpeditionPort.save_json is abstract")
	return ""


func load_json(_json: String) -> Dictionary:
	push_error("ExpeditionPort.load_json is abstract")
	return {"configured": false, "error": "abstract_port"}
