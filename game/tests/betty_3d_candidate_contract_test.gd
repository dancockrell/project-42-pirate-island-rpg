extends SceneTree

## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0
## Guards the review boundary: the downloaded GLB is a real source asset but
## cannot silently become an animated battle actor while it has no skeleton.

const VENDOR_MANIFEST := "res://../work/art/vendor/magnific/betty-3d/N2cYw4m6D9/manifest.json"
const ENGINE_CANDIDATE := "res://assets/candidates/betty_3d/betty_candidate_v1.glb"


func _init() -> void:
	var manifest_file := FileAccess.open(VENDOR_MANIFEST, FileAccess.READ)
	check(manifest_file != null, "vendor provenance manifest must exist")
	var manifest: Dictionary = JSON.parse_string(manifest_file.get_as_text())
	check(manifest.get("status") == "quarantined-static-blocking-candidate", "candidate must remain quarantined")
	check(manifest.get("structuralInspection", {}).get("skins") == 0, "static candidate must declare no skeleton")
	check(manifest.get("structuralInspection", {}).get("animations") == 0, "static candidate must declare no animation clips")
	check(FileAccess.file_exists(ENGINE_CANDIDATE), "Godot review copy must exist")
	quit(1 if failures > 0 else 0)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
