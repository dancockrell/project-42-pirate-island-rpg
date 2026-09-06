extends Node

## Autoload `ContentPackRegistry`. Presentation packs are the only shape the
## adult presentation layer takes: a pack replaces an authored scene's beats by
## scene ID and carries its own assets, and it never carries simulation data.
## C8 owns the manifest schema and `tools/src/validate.mjs` is its only
## validator; this registry is the runtime reader, not a second schema. It
## refuses a manifest it cannot use and says why, and it checks nothing the
## validator already proved about a pack that shipped through the gate.
##
## Scanning finds two things under a root: a `*.pck` (mounted with
## `ProjectSettings.load_resource_pack`, whose manifest is then read from
## `res://packs/<pack id>/pack.json`, because the validator holds a pack's ID
## equal to its directory name) and a plain directory carrying a `pack.json`,
## so the repository's fixture pack works without an export step.
##
## The simulation never reads presentation. Nothing here reaches the bridge.

## The one level ordering table. `fade_to_black` is where the base tree stops;
## `explicit` is the separate artifact built outside this repository. Rank is
## the index, so "highest level offered" is a maximum over this order.
const PRESENTATION_LEVELS := ["fade_to_black", "explicit"]
const BASE_TREE_PRESENTATION_LEVEL := "fade_to_black"

const DEFAULT_SCAN_ROOTS := ["user://packs", "res://packs"]
const MANIFEST_NAME := "pack.json"
const PACK_KIND := "presentation_override"

## Read-only index over the generated bundle, for a scene's base level. Left
## null until something asks; assigned directly by a caller that already has one.
var catalog: ContentCatalog = null

var _loaded_packs: Array[Dictionary] = []
var _overrides_by_scene_id: Dictionary = {}
var _refusals: Array[Dictionary] = []


func _ready() -> void:
	scan_default_roots()


## Scans `user://packs/` then `res://packs/`. A root that is not there is not a
## refusal: a game with no packs installed is the ordinary case.
func scan_default_roots() -> Dictionary:
	var loaded: Array[String] = []
	var refused: Array[Dictionary] = []
	for root in DEFAULT_SCAN_ROOTS:
		var result := scan(str(root))
		for pack_id in result.get("loaded", []):
			loaded.append(str(pack_id))
		for refusal in result.get("refused", []):
			refused.append(refusal)
	return {"loaded": loaded, "refused": refused}


## Scans one root for packs and merges what it finds, in name order, after
## everything already loaded. Later wins. `root` may be a `res://`/`user://`
## path or an absolute one, so a test can point this at the repository's
## `packs/` directory without a copy of the fixture living under `game/`.
func scan(root: String) -> Dictionary:
	var loaded: Array[String] = []
	var refused: Array[Dictionary] = []
	var directory := DirAccess.open(root)
	if directory == null:
		return {"loaded": loaded, "refused": refused}
	# Files and directories are kept apart by where they came from, never by
	# the shape of the name: a pack directory is `pack.<slug>`, whose last dot
	# is part of its ID rather than an extension.
	var resource_packs: Array[String] = []
	for file_name in directory.get_files():
		if file_name.get_extension().to_lower() == "pck":
			resource_packs.append(file_name)
	var pack_directories: Array[String] = []
	for directory_name in directory.get_directories():
		pack_directories.append(directory_name)
	resource_packs.sort()
	pack_directories.sort()

	for file_name in resource_packs:
		var pack_path := root.path_join(file_name)
		if not ProjectSettings.load_resource_pack(pack_path):
			refused.append(_refuse(pack_path, "Godot refused to mount the resource pack"))
			continue
		# The validator holds a pack's ID equal to its directory name, and E9
		# builds each pack from that directory, so a mounted .pck lands its
		# manifest at exactly this path.
		var mounted_manifest := "res://packs".path_join(file_name.get_basename()).path_join(MANIFEST_NAME)
		if not FileAccess.file_exists(mounted_manifest):
			refused.append(_refuse(pack_path, "the mounted resource pack carries no %s at %s" % [MANIFEST_NAME, mounted_manifest]))
			continue
		_admit(mounted_manifest, loaded, refused)

	for directory_name in pack_directories:
		var manifest_path := root.path_join(directory_name).path_join(MANIFEST_NAME)
		# A directory under a scan root that carries no manifest is not a pack
		# and is not a refusal; it is simply not ours.
		if FileAccess.file_exists(manifest_path):
			_admit(manifest_path, loaded, refused)

	_refusals.append_array(refused)
	return {"loaded": loaded, "refused": refused}


## The highest presentation level any loaded pack offers for this scene, else
## the scene's authored base level from the content bundle.
func presentation_level_for(scene_id: String) -> String:
	var best := base_presentation_level_for(scene_id)
	for pack in _loaded_packs:
		if not (pack.get("overrides", {}) as Dictionary).has(scene_id):
			continue
		var level := str(pack.get("targetLevel", ""))
		if level_rank(level) > level_rank(best):
			best = level
	return best


## The scene's authored level, which is `fade_to_black` for everything in this
## repository. A scene the bundle does not carry reads as the base tree's level
## rather than as an error: presentation is never the simulation's business.
func base_presentation_level_for(scene_id: String) -> String:
	var loaded_catalog := _catalog()
	if loaded_catalog == null or not loaded_catalog.has(scene_id):
		return BASE_TREE_PRESENTATION_LEVEL
	var level := str(loaded_catalog.get_record(scene_id).get("presentationLevel", ""))
	return level if level_rank(level) >= 0 else BASE_TREE_PRESENTATION_LEVEL


## The winning override for a scene: its beats and its assets resolved to paths
## inside the pack that won. Empty when no loaded pack overrides the scene.
func override_for(scene_id: String) -> Dictionary:
	if not _overrides_by_scene_id.has(scene_id):
		return {}
	return (_overrides_by_scene_id[scene_id] as Dictionary).duplicate(true)


func loaded_packs() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for pack in _loaded_packs:
		result.append(pack.duplicate(true))
	return result


## Every manifest this registry refused, each with the reason it was refused.
func refusals() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for refusal in _refusals:
		result.append(refusal.duplicate(true))
	return result


func level_rank(level: String) -> int:
	return PRESENTATION_LEVELS.find(level)


func _catalog() -> ContentCatalog:
	if catalog == null:
		var next := ContentCatalog.new()
		if next.load_default() != OK:
			return null
		catalog = next
	return catalog


## Reads one manifest and either merges it or records why it was refused.
func _admit(manifest_path: String, loaded: Array[String], refused: Array[Dictionary]) -> void:
	var pack := _read_manifest(manifest_path)
	if not pack.get("ok", false):
		refused.append(_refuse(manifest_path, str(pack.get("reason", "unreadable"))))
		return
	_merge(pack)
	loaded.append(str(pack.get("id", "")))


func _refuse(path: String, reason: String) -> Dictionary:
	return {"path": path, "reason": reason}


## Reads and resolves one manifest. Every asset an override names must be
## declared in the manifest's `assets` array and must be a file that is really
## there, so a pack whose art did not ship is refused whole rather than loaded
## into a scene that will play with nothing to show.
func _read_manifest(manifest_path: String) -> Dictionary:
	var file := FileAccess.open(manifest_path, FileAccess.READ)
	if file == null:
		return {"ok": false, "reason": "cannot be opened: %s" % error_string(FileAccess.get_open_error())}
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	if not parsed is Dictionary:
		return {"ok": false, "reason": "is not a JSON object"}
	var manifest: Dictionary = parsed
	var pack_id := str(manifest.get("id", ""))
	if not pack_id.begins_with("pack."):
		return {"ok": false, "reason": "id %s must be pack.<slug>" % pack_id}
	if str(manifest.get("kind", "")) != PACK_KIND:
		return {"ok": false, "reason": "kind %s must be %s" % [str(manifest.get("kind", "")), PACK_KIND]}
	var target_level := str(manifest.get("targetLevel", ""))
	if level_rank(target_level) < 0:
		return {"ok": false, "reason": "targetLevel %s is not one of %s" % [target_level, str(PRESENTATION_LEVELS)]}
	var pack_root := manifest_path.get_base_dir()
	var asset_paths: Dictionary = {}
	if not manifest.get("assets", null) is Array:
		return {"ok": false, "reason": "assets must be an array"}
	for asset in manifest.assets:
		if not asset is Dictionary:
			return {"ok": false, "reason": "every assets entry must be an object"}
		var asset_id := str(asset.get("id", ""))
		var relative := str(asset.get("path", ""))
		if asset_id.is_empty() or relative.is_empty():
			return {"ok": false, "reason": "every assets entry needs an id and a path"}
		var resolved := pack_root.path_join(relative)
		if not FileAccess.file_exists(resolved):
			return {"ok": false, "reason": "asset %s declares path %s, which is not a file inside the pack" % [asset_id, relative]}
		asset_paths[asset_id] = resolved
	if not manifest.get("overrides", null) is Dictionary:
		return {"ok": false, "reason": "overrides must be an object keyed by scene ID"}
	var overrides: Dictionary = {}
	for scene_id in manifest.overrides:
		if not manifest.overrides[scene_id] is Dictionary:
			return {"ok": false, "reason": "override %s must be an object" % str(scene_id)}
		var authored: Dictionary = manifest.overrides[scene_id]
		if not authored.get("assetIds", null) is Array or not authored.get("beats", null) is Array:
			return {"ok": false, "reason": "override %s must carry a beats array and an assetIds array" % str(scene_id)}
		var resolved_assets: Array[String] = []
		for asset_id in authored.assetIds:
			if not asset_paths.has(str(asset_id)):
				return {"ok": false, "reason": "override %s names asset %s, which the pack does not declare" % [str(scene_id), str(asset_id)]}
			resolved_assets.append(str(asset_paths[str(asset_id)]))
		var beats: Array = authored.beats
		if beats.is_empty():
			return {"ok": false, "reason": "override %s carries no beats" % str(scene_id)}
		overrides[str(scene_id)] = {
			"sceneId": str(scene_id),
			"packId": pack_id,
			"targetLevel": target_level,
			"beats": beats.duplicate(true),
			"assetPaths": resolved_assets,
		}
	return {
		"ok": true,
		"id": pack_id,
		"displayName": str(manifest.get("displayName", pack_id)),
		"targetLevel": target_level,
		"manifestPath": manifest_path,
		"packRoot": pack_root,
		"overrides": overrides,
	}


## Merge order is load order: a pack scanned later wins every scene it shares
## with one scanned earlier, and the winning pack's ID is what `override_for`
## hands back.
func _merge(pack: Dictionary) -> void:
	var overrides: Dictionary = pack.get("overrides", {})
	for scene_id in overrides:
		_overrides_by_scene_id[scene_id] = overrides[scene_id]
	_loaded_packs.append(pack)
