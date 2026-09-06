extends SceneTree

## B10. ContentPackRegistry read against C8's fixture pack, which lives at the
## repository's `packs/pack.presentation.fixture/` and is NOT copied under
## `game/`: the suite points the registry at that directory by absolute path,
## exactly as the registry's `scan(root)` exists to allow.
##
## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
##
## No live bridge is read here. Presentation is not the simulation.

const PackRegistryScript = preload("res://scripts/content/pack_registry.gd")

const OVERRIDDEN_SCENE := "scene.betty.the_lamp_left_burning"
const SECOND_OVERRIDDEN_SCENE := "scene.ayla.what_the_tomb_is_for"
const UNOVERRIDDEN_SCENE := "scene.betty.the_second_watch"
const FIXTURE_PACK_ID := "pack.presentation.fixture"

## Two packs written by the suite at runtime, each under its own user:// root so
## neither can collide with the autoload's `user://packs/` scan. They are
## written here rather than committed because neither belongs in the repository:
## one declares an asset that is not there, and the other carries targetLevel
## `explicit`, which is the separate artifact built outside this repository and
## which `packs/` may therefore never hold. Nothing explicit is authored in
## either: `targetLevel` is a string, and the beats are placeholder markers.
const BROKEN_ROOT := "user://b10_missing_asset_pack_fixture"
const BROKEN_PACK := "pack.b10_missing_asset"
const HIGHER_ROOT := "user://b10_higher_level_pack_fixture"
const HIGHER_PACK := "pack.b10_higher_level"

var failures := 0


func _init() -> void:
	call_deferred("run")


func run() -> void:
	var repository_packs := ProjectSettings.globalize_path("res://").path_join("../packs").simplify_path()
	check(DirAccess.dir_exists_absolute(repository_packs.path_join(FIXTURE_PACK_ID)), "the repository's fixture pack must be where the suite scans: %s" % repository_packs)

	# The ordering table itself: one table, two levels, and the base tree's
	# level is the floor.
	check(PackRegistryScript.PRESENTATION_LEVELS == ["fade_to_black", "explicit"], "the level table is fade_to_black then explicit and nothing else")

	# 1. No pack scanned: every scene reads its authored base level.
	var bare := PackRegistryScript.new()
	check(bare.loaded_packs().is_empty(), "a registry that scanned nothing has loaded no pack")
	check(bare.override_for(OVERRIDDEN_SCENE).is_empty(), "with no pack scanned there is no override to hand back")
	check(bare.presentation_level_for(OVERRIDDEN_SCENE) == "fade_to_black", "with no pack scanned the scene falls back to its authored base level")
	check(bare.base_presentation_level_for(OVERRIDDEN_SCENE) == "fade_to_black", "the base level comes from the scene's own record in the content bundle")

	# 2. The fixture pack scanned from the repository directory.
	var registry := PackRegistryScript.new()
	var result := registry.scan(repository_packs)
	check(result.get("refused", []).is_empty(), "the fixture pack must load without a refusal: %s" % str(result.get("refused", [])))
	var loaded_ids: Array = result.get("loaded", [])
	check(loaded_ids.size() == 1 and str(loaded_ids[0]) == FIXTURE_PACK_ID, "scanning the repository packs directory loads exactly the fixture pack: %s" % str(loaded_ids))
	check(registry.loaded_packs().size() == 1, "one pack scanned, one pack loaded")

	check(registry.presentation_level_for(OVERRIDDEN_SCENE) == "fade_to_black", "an overridden scene reads the fixture pack's targetLevel")
	check(registry.presentation_level_for(SECOND_OVERRIDDEN_SCENE) == "fade_to_black", "the pack's second override is merged too")

	# 3. A scene the pack does not override still falls back to base.
	check(registry.override_for(UNOVERRIDDEN_SCENE).is_empty(), "the fixture overrides two scenes and no others")
	check(registry.presentation_level_for(UNOVERRIDDEN_SCENE) == "fade_to_black", "a scene the pack does not override reads its authored base level")

	# The override the registry hands the scene player: the pack's beats, the
	# winning pack recorded, and both asset paths resolved to files inside it.
	var resolved_assets: Array[String] = []
	for scene_id in [OVERRIDDEN_SCENE, SECOND_OVERRIDDEN_SCENE]:
		var override_entry := registry.override_for(scene_id)
		check(str(override_entry.get("packId", "")) == FIXTURE_PACK_ID, "%s records the pack that won it" % scene_id)
		check(str(override_entry.get("targetLevel", "")) == "fade_to_black", "%s carries the pack's level" % scene_id)
		var beats: Array = override_entry.get("beats", [])
		check(beats.size() == 2, "%s carries the pack's two beats" % scene_id)
		check(str(beats.back().get("kind", "")) == "fade", "%s still closes on a fade" % scene_id)
		for path in override_entry.get("assetPaths", []):
			check(FileAccess.file_exists(str(path)), "%s resolves to a file inside the pack: %s" % [scene_id, str(path)])
			resolved_assets.append(str(path))
	check(resolved_assets.size() == 2, "the fixture's two overrides resolve two asset paths, one each")

	# A copy, not the registry's own dictionary.
	var mutated := registry.override_for(OVERRIDDEN_SCENE)
	mutated["packId"] = "pack.not_this_one"
	check(str(registry.override_for(OVERRIDDEN_SCENE).get("packId", "")) == FIXTURE_PACK_ID, "override_for hands back a copy")

	# 4. A pack offering a higher level, scanned after the fixture: the level
	#    switches, the later pack wins the scene it shares, and the fixture keeps
	#    the scene the higher pack does not touch.
	write_pack(HIGHER_ROOT, HIGHER_PACK, "explicit", true)
	var higher := registry.scan(HIGHER_ROOT)
	check(higher.get("refused", []).is_empty(), "the higher-level pack must load: %s" % str(higher.get("refused", [])))
	check(registry.loaded_packs().size() == 2, "two packs scanned, two packs loaded")
	check(registry.presentation_level_for(OVERRIDDEN_SCENE) == "explicit", "the highest level any loaded pack offers for the scene is what the scene player reads")
	check(str(registry.override_for(OVERRIDDEN_SCENE).get("packId", "")) == HIGHER_PACK, "the pack scanned later wins the scene, and the registry records which one won")
	check(str(registry.override_for(SECOND_OVERRIDDEN_SCENE).get("packId", "")) == FIXTURE_PACK_ID, "a scene the later pack does not touch stays with the pack that did")
	check(registry.presentation_level_for(SECOND_OVERRIDDEN_SCENE) == "fade_to_black", "and that scene keeps the fixture's level")
	check(registry.presentation_level_for(UNOVERRIDDEN_SCENE) == "fade_to_black", "a scene no pack overrides still reads its authored base level")
	remove_pack(HIGHER_ROOT, HIGHER_PACK, true)

	# 5. A manifest declaring an asset that is not there is refused, with a
	#    reason, and nothing of it is merged.
	write_pack(BROKEN_ROOT, BROKEN_PACK, "explicit", false)
	var refusing := PackRegistryScript.new()
	var broken := refusing.scan(BROKEN_ROOT)
	check(broken.get("loaded", []).is_empty(), "a pack with a missing asset does not load")
	check(refusing.loaded_packs().is_empty(), "a refused pack is not among the loaded packs")
	check(refusing.override_for(OVERRIDDEN_SCENE).is_empty(), "a refused pack merges none of its overrides")
	check(refusing.presentation_level_for(OVERRIDDEN_SCENE) == "fade_to_black", "a refused pack lifts no scene's level")
	var refusals := refusing.refusals()
	check(refusals.size() == 1, "the refusal is recorded")
	if refusals.size() == 1:
		var reason := str(refusals[0].get("reason", ""))
		check(reason.contains("assets/plate.txt"), "the refusal names the asset path that is not there: %s" % reason)
	remove_pack(BROKEN_ROOT, BROKEN_PACK, false)

	bare.free()
	registry.free()
	refusing.free()
	if failures == 0:
		print("ContentPackRegistry pack tests passed.")
	quit(1 if failures > 0 else 0)


## Writes one pack in C8's manifest shape. `with_asset` decides whether the
## plate the manifest declares is actually written, which is the whole
## difference between a pack that loads and a pack that is refused.
func write_pack(root: String, pack_id: String, target_level: String, with_asset: bool) -> void:
	var pack_directory := root.path_join(pack_id)
	DirAccess.make_dir_recursive_absolute(pack_directory.path_join("assets"))
	var manifest := {
		"id": pack_id,
		"kind": "presentation_override",
		"targetLevel": target_level,
		"displayName": "A pack the suite wrote",
		"assets": [{"id": "asset.b10.plate", "path": "assets/plate.txt"}],
		"overrides": {
			OVERRIDDEN_SCENE: {
				"beats": [
					{"id": "beat.b10.suite", "kind": "conversation", "text": "PLACEHOLDER PROSE written by the suite. It says nothing the base beat did not say."},
					{"id": "beat.b10.suite_close", "kind": "fade", "text": "PLACEHOLDER PROSE. The fade closes the scene, as it does everywhere in the base tree."},
				],
				"assetIds": ["asset.b10.plate"],
			},
		},
	}
	if with_asset:
		write_text(pack_directory.path_join("assets/plate.txt"), "DUMMY placeholder plate written by pack_registry_test.gd.")
	write_text(pack_directory.path_join("pack.json"), JSON.stringify(manifest))


func write_text(path: String, text: String) -> void:
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		failures += 1
		push_error("the suite could not write %s" % path)
		return
	file.store_string(text)
	file.close()


func remove_pack(root: String, pack_id: String, with_asset: bool) -> void:
	var pack_directory := root.path_join(pack_id)
	if with_asset:
		DirAccess.remove_absolute(pack_directory.path_join("assets/plate.txt"))
	DirAccess.remove_absolute(pack_directory.path_join("assets"))
	DirAccess.remove_absolute(pack_directory.path_join("pack.json"))
	DirAccess.remove_absolute(pack_directory)
	DirAccess.remove_absolute(root)


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
