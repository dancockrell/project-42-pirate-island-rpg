extends Node

## Runtime owner for one native campaign bridge. This node never calculates
## routes, encounters, outcomes, or resources; it only keeps the same
## NativeExpeditionPort alive while Godot swaps presentation scenes.

const INITIAL_SEED := 42

var expedition: NativeExpeditionPort
var catalog: ContentCatalog
var latest_snapshot: Dictionary = {}


func begin_if_needed(next_catalog: ContentCatalog) -> Dictionary:
	if expedition != null and bool(latest_snapshot.get("configured", false)):
		return latest_snapshot.duplicate(true)
	catalog = next_catalog
	if not NativeExpeditionPort.bridge_is_registered():
		latest_snapshot = {
			"configured": false,
			"error": "native_expedition_bridge_unavailable",
			"metadata": {"source": "campaign_session", "authoritative": false}
		}
		return latest_snapshot.duplicate(true)
	expedition = NativeExpeditionPort.new()
	latest_snapshot = expedition.configure_from_catalog(catalog, INITIAL_SEED)
	return latest_snapshot.duplicate(true)


func travel(portal_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.travel(portal_id)
	return latest_snapshot.duplicate(true)


func use_anchor(anchor_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.use_anchor(anchor_id)
	return latest_snapshot.duplicate(true)


func set_control(cell_id: String, faction_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.set_control(cell_id, faction_id)
	return latest_snapshot.duplicate(true)


func controller_of(cell_id: String) -> String:
	if expedition == null:
		return ""
	return expedition.controller_of(cell_id)


func effective_risk(portal_id: String) -> Variant:
	if expedition == null:
		return unavailable_state()
	return expedition.effective_risk(portal_id)


func inspect(observation_id: String) -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.inspect(observation_id)
	return latest_snapshot.duplicate(true)


func resolve_midnight() -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.resolve_midnight()
	return latest_snapshot.duplicate(true)


func snapshot() -> Dictionary:
	if expedition == null:
		return unavailable_state()
	latest_snapshot = expedition.snapshot()
	return latest_snapshot.duplicate(true)


func pending_encounter() -> Dictionary:
	return latest_snapshot.get("pending_encounter", {}).duplicate(true)


func has_pending_encounter() -> bool:
	return not pending_encounter().is_empty()


## Campaign encounters are created by the exact native expedition bridge that
## armed them. The scene layer cannot substitute a battle ID or create a
## disconnected debug fight while a campaign handoff is active.
func begin_pending_battle() -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return battle_unavailable_snapshot("campaign_session_uninitialized")
	return expedition.bridge.begin_pending_battle()


func start_pending_battle() -> Array[Dictionary]:
	if expedition == null or expedition.bridge == null:
		return [battle_unavailable_event("campaign_session_uninitialized")]
	return typed_event_array(expedition.bridge.start_pending_battle())


func submit_pending_battle(command: Dictionary) -> Array[Dictionary]:
	if expedition == null or expedition.bridge == null:
		return [battle_unavailable_event("campaign_session_uninitialized")]
	var normalized := {
		"protocol_version": int(command.get("protocol_version", 1)),
		"command_id": str(command.get("command_id", "")),
		"battle_id": str(command.get("battle_id", "")),
		"actor_id": str(command.get("actor_id", "")),
		"kind": str(command.get("kind", "")),
		"skill_id": str(command.get("skill_id", "")),
		"target_ids": Array(command.get("target_ids", []), TYPE_STRING, "", null)
	}
	var events := typed_event_array(expedition.bridge.submit_pending_command(normalized))
	latest_snapshot = expedition.snapshot()
	return events


func recommended_pending_enemy_command(command_id: String) -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return {"available": false, "reason": "campaign_session_uninitialized"}
	return expedition.bridge.recommended_pending_enemy_command(command_id)


func pending_battle_snapshot() -> Dictionary:
	if expedition == null or expedition.bridge == null:
		return battle_unavailable_snapshot("campaign_session_uninitialized")
	return expedition.bridge.pending_battle_snapshot()


func reset_for_test() -> void:
	discard_campaign()


## Drops whatever campaign this session is holding. The one owner of "this
## session now holds nothing": both the test reset above and B8's new_game go
## through here, so there is no second idea of what an empty session is.
func discard_campaign() -> void:
	expedition = null
	catalog = null
	latest_snapshot = {}


func unavailable_state() -> Dictionary:
	return session_error("campaign_session_uninitialized")


## A refusal from this node, in the one shape the whole expedition boundary
## uses: `configured: false` and a machine-readable reason. The native bridge
## answers a refused travel, configuration or load exactly like this, so a
## caller has one thing to check whether the refusal came from Rust or from
## here.
func session_error(reason: String) -> Dictionary:
	return {
		"configured": false,
		"error": reason,
		"metadata": {"source": "campaign_session", "authoritative": false}
	}


func typed_event_array(value: Variant) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if value is Array:
		for event in value:
			if event is Dictionary:
				result.append(event)
	return result


func battle_unavailable_snapshot(reason: String) -> Dictionary:
	return {"battle_id": "", "phase": "error", "actors": [], "error": {"kind": reason}}


func battle_unavailable_event(reason: String) -> Dictionary:
	return {"event_id": "event.campaign.unavailable", "sequence": 0, "kind": "command_rejected", "subjects": [], "payload": {"reason": reason}}


# ---------------------------------------------------------------------------
# B8: new game, save slots, continue.
#
# A slot is one file of the exact bytes the native bridge writes, and nothing
# on this side understands them: `ExpeditionState` owns the save format, so the
# only thing Godot decides is which slot to hand back and what to call the
# refusal when Rust will not take it.
# ---------------------------------------------------------------------------

const SAVE_DIRECTORY := "user://saves"
const SLOT_NAME_CHARACTERS := "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
const MAXIMUM_SLOT_NAME_LENGTH := 64


## Starts a fresh campaign from the catalog this session already has.
##
## `begin_if_needed` deliberately keeps a configured campaign alive across a
## scene swap, so "new game" is that same call with what was held dropped
## first. There is no second configuration path and no second seed: a new game
## is the same INITIAL_SEED opening the same island, which is what makes the
## first day reproducible.
func new_game() -> Dictionary:
	if catalog == null:
		return session_error("campaign_session_uninitialized")
	var held_catalog := catalog
	discard_campaign()
	return begin_if_needed(held_catalog)


## Writes the current campaign to `user://saves/<slot>.json`.
##
## The document is whatever `save_json` gives back, stored verbatim. On success
## the current snapshot comes back with the `slot` and `path` that were written
## added to it -- the same idiom `use_anchor` uses when it returns the snapshot
## with the outcome of the action beside it. A refusal is the session's error
## dictionary, and nothing is written.
func save_to_slot(slot: String) -> Dictionary:
	if expedition == null:
		return session_error("campaign_session_uninitialized")
	if not is_valid_slot_name(slot):
		return session_error("invalid_slot_name")
	var document := expedition.save_json()
	if document.is_empty():
		return session_error("expedition_not_configured")
	if not DirAccess.dir_exists_absolute(SAVE_DIRECTORY):
		DirAccess.make_dir_recursive_absolute(SAVE_DIRECTORY)
	if not DirAccess.dir_exists_absolute(SAVE_DIRECTORY):
		return session_error("save_directory_unwritable")
	var path := slot_path(slot)
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return session_error("save_slot_unwritable")
	file.store_string(document)
	file.close()
	var saved := snapshot()
	saved["slot"] = slot
	saved["path"] = path
	return saved


## Every slot on disk with what its own save says about it.
##
## Exactly three facts are read out of each document -- `save_version`,
## `campaign_day` and the strategic `hour_of_day` -- because those are the only
## ones `continue_newest` needs to choose between slots. Everything else about
## a save stays the simulation's business. A slot that cannot be read or does
## not parse is listed rather than hidden, with `readable` false and the reason
## in `error`, so a player is told a slot is broken instead of watching it
## vanish.
func list_slots() -> Array[Dictionary]:
	var entries: Array[Dictionary] = []
	if not DirAccess.dir_exists_absolute(SAVE_DIRECTORY):
		return entries
	var names := DirAccess.get_files_at(SAVE_DIRECTORY)
	names.sort()
	for entry_name in names:
		var file_name := str(entry_name)
		if not file_name.ends_with(".json"):
			continue
		entries.append(describe_slot(file_name.trim_suffix(".json")))
	return entries


## Loads the newest slot into this session.
##
## "Newest" is the save's own campaign day and strategic hour, never the file's
## modification time: a copied, restored or synchronised file keeps its
## campaign but not its timestamp, and a clock the player cannot see must not
## decide which campaign they resume.
##
## The chosen document goes to the bridge unexamined, so a corrupted or
## future-versioned slot is refused by the same validation the Rust save
## migration tests hold, with the bridge's own reason. A refusal leaves this
## session exactly as it was -- `latest_snapshot` is only replaced once the
## load has been accepted -- so continuing from a broken slot never quietly
## becomes a new game.
func continue_newest() -> Dictionary:
	if expedition == null:
		return session_error("campaign_session_uninitialized")
	var slots := list_slots()
	if slots.is_empty():
		return session_error("no_save_slot")
	var newest: Dictionary = slots[0]
	for entry in slots:
		if slot_is_newer(entry, newest):
			newest = entry
	var loaded: Dictionary = expedition.load_json(read_slot(str(newest.get("path", ""))))
	if not bool(loaded.get("configured", false)):
		return loaded
	latest_snapshot = loaded
	return latest_snapshot.duplicate(true)


## One slot, described from its own document.
func describe_slot(slot: String) -> Dictionary:
	var path := slot_path(slot)
	var entry := {
		"slot": slot,
		"path": path,
		"readable": false,
		"save_version": 0,
		"campaign_day": 0,
		"hour_of_day": 0,
		"error": ""
	}
	var text := read_slot(path)
	if text.is_empty():
		entry["error"] = "save_slot_unreadable"
		return entry
	var parsed: Variant = JSON.parse_string(text)
	if not (parsed is Dictionary):
		entry["error"] = "malformed_json"
		return entry
	var document: Dictionary = parsed
	if not document.has("campaign_day"):
		entry["error"] = "malformed_json"
		return entry
	var clock: Variant = document.get("strategic_clock", {})
	entry["readable"] = true
	entry["save_version"] = int(document.get("save_version", 0))
	entry["campaign_day"] = int(document.get("campaign_day", 0))
	if clock is Dictionary:
		entry["hour_of_day"] = int((clock as Dictionary).get("hour_of_day", 0))
	return entry


## Whether `candidate` is the later campaign of the two.
##
## Day first, then the strategic hour within the day. A slot nothing can read
## has no campaign time at all and therefore loses to every readable one, but
## it is still a candidate when it is all there is -- so the only slot on disk
## being corrupt produces the bridge's refusal rather than silence.
static func slot_is_newer(candidate: Dictionary, incumbent: Dictionary) -> bool:
	var candidate_readable := bool(candidate.get("readable", false))
	var incumbent_readable := bool(incumbent.get("readable", false))
	if candidate_readable != incumbent_readable:
		return candidate_readable
	var candidate_day := int(candidate.get("campaign_day", 0))
	var incumbent_day := int(incumbent.get("campaign_day", 0))
	if candidate_day != incumbent_day:
		return candidate_day > incumbent_day
	var candidate_hour := int(candidate.get("hour_of_day", 0))
	var incumbent_hour := int(incumbent.get("hour_of_day", 0))
	if candidate_hour != incumbent_hour:
		return candidate_hour > incumbent_hour
	# Two saves standing at the same instant are equally new, so the tie is
	# broken by name to keep the choice deterministic rather than dependent on
	# the order the directory happened to enumerate.
	return str(candidate.get("slot", "")) < str(incumbent.get("slot", ""))


## The one place a slot name becomes a path. Names are restricted to letters,
## digits, underscore and hyphen so a slot can never address a file outside the
## save directory.
static func slot_path(slot: String) -> String:
	return "%s/%s.json" % [SAVE_DIRECTORY, slot]


static func is_valid_slot_name(slot: String) -> bool:
	if slot.is_empty() or slot.length() > MAXIMUM_SLOT_NAME_LENGTH:
		return false
	for index in slot.length():
		if not SLOT_NAME_CHARACTERS.contains(slot[index]):
			return false
	return true


## A slot's bytes, or "" when there are none to read. An empty string is handed
## to the bridge unchanged rather than short-circuited here, so an unreadable
## slot is refused by the same parser that refuses a corrupt one.
static func read_slot(path: String) -> String:
	if not FileAccess.file_exists(path):
		return ""
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return ""
	var text := file.get_as_text()
	file.close()
	return text
