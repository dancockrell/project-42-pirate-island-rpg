class_name ContentCatalog
extends RefCounted

## Read-only index over the validated generated content bundle. Runtime systems
## request stable IDs from this catalog; they never crawl authoring folders.

const BUNDLE_PATH := "res://generated/content_bundle.json"
const EXPECTED_FORMAT := "project42.content_bundle"
const SUPPORTED_VERSION := 1

var records_by_id: Dictionary = {}
var source_path_by_id: Dictionary = {}
var registry_entries_by_id: Dictionary = {}
var content_hash := ""

func load_default() -> Error:
	if not FileAccess.file_exists(BUNDLE_PATH):
		push_error("Missing generated content bundle. Run npm run build:content.")
		return ERR_FILE_NOT_FOUND
	var file := FileAccess.open(BUNDLE_PATH, FileAccess.READ)
	if file == null:
		return FileAccess.get_open_error()
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	if not parsed is Dictionary:
		push_error("Content bundle root must be a Dictionary.")
		return ERR_PARSE_ERROR
	return load_bundle(parsed)

func load_bundle(bundle: Dictionary) -> Error:
	if bundle.get("format", "") != EXPECTED_FORMAT or int(bundle.get("version", 0)) != SUPPORTED_VERSION:
		push_error("Unsupported Project 42 content bundle format or version.")
		return ERR_INVALID_DATA
	var records: Array = bundle.get("records", [])
	if records.size() != int(bundle.get("recordCount", -1)):
		push_error("Content bundle recordCount does not match its records array.")
		return ERR_INVALID_DATA
	var next_records: Dictionary = {}
	var next_sources: Dictionary = {}
	var next_registry_entries: Dictionary = {}
	for entry in records:
		if not entry is Dictionary or not entry.get("value", null) is Dictionary:
			return ERR_INVALID_DATA
		var id: String = str(entry.get("id", ""))
		if id.is_empty() or next_records.has(id) or next_registry_entries.has(id) or entry.value.get("id", "") != id:
			push_error("Content bundle contains an empty, duplicate or mismatched stable ID: %s" % id)
			return ERR_INVALID_DATA
		next_records[id] = entry.value
		next_sources[id] = str(entry.get("sourcePath", ""))
		var registry_defaults: Dictionary = entry.value.get("defaults", {})
		for registry_entry in entry.value.get("entries", []):
			if not registry_entry is Dictionary:
				return ERR_INVALID_DATA
			var registry_entry_id := str(registry_entry.get("id", ""))
			if registry_entry_id.is_empty() or next_registry_entries.has(registry_entry_id) or next_records.has(registry_entry_id):
				push_error("Content bundle contains an empty or duplicate registry entry ID: %s" % registry_entry_id)
				return ERR_INVALID_DATA
			var resolved_registry_entry := registry_defaults.duplicate(true)
			resolved_registry_entry.merge(registry_entry, true)
			next_registry_entries[registry_entry_id] = resolved_registry_entry
	records_by_id = next_records
	source_path_by_id = next_sources
	registry_entries_by_id = next_registry_entries
	content_hash = str(bundle.get("contentHash", ""))
	return OK

func has(id: String) -> bool:
	return records_by_id.has(id)

func get_record(id: String) -> Dictionary:
	if not records_by_id.has(id):
		push_error("Unknown Project 42 content ID: %s" % id)
		return {}
	return records_by_id[id].duplicate(true)

func has_registry_entry(id: String) -> bool:
	return registry_entries_by_id.has(id)

func get_registry_entry(id: String) -> Dictionary:
	if not registry_entries_by_id.has(id):
		push_error("Unknown Project 42 registry entry ID: %s" % id)
		return {}
	return registry_entries_by_id[id].duplicate(true)

func get_source_path(id: String) -> String:
	return str(source_path_by_id.get(id, ""))

func ids_with_prefix(prefix: String) -> Array[String]:
	var result: Array[String] = []
	for id in records_by_id:
		if str(id).begins_with(prefix):
			result.append(str(id))
	result.sort()
	return result
