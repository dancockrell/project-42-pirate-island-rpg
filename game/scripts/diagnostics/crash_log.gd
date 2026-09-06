extends Node

## The whole of Project 42's failure reporting: a JSON file on the player's own
## disk. Nothing here opens a socket, spawns a process or asks the player to
## send anything anywhere; the file is the feature, and the suite proves the
## absence by scanning res://scripts/ for the engine's HTTP classes by name.
##
## Two entry points. `remember_snapshot` is fed by CampaignSession every time it
## refreshes its expedition snapshot, so the newest in-world state is already in
## hand when something goes wrong. `record_failure` writes the file.
##
## Determinism (the continuation brief's rule): no wall clock is read, here or
## in the file. `written_at` is in-world time -- the campaign day and time
## segment of the remembered snapshot -- and so is the file name, which is why
## the file name also carries a counter: two failures inside one in-world hour
## must not overwrite each other.

const LOG_DIRECTORY := "user://logs"
const FILE_PREFIX := "crash-"
const FILE_SUFFIX := ".json"
const UNKNOWN_SEGMENT := "unknown"

## The last snapshot the campaign session handed over. Duplicated on the way in
## so a later mutation of the session's dictionary cannot rewrite history.
var last_snapshot: Dictionary = {}

## Set the moment `record_failure` starts and cleared only once the file is on
## disk. If the process dies or the window closes between those two points the
## failure is still in flight, and the notification hook below finishes it. A
## clean quit leaves this empty and therefore writes nothing at all.
var pending_failure_reason := ""

## Read once from the generated bundle through ContentCatalog -- the one owner
## of what that file says -- and kept, because a crash path should parse as
## little as it can get away with.
var _bundle_hash := ""
var _bundle_hash_read := false


func remember_snapshot(snapshot: Dictionary) -> void:
	last_snapshot = snapshot.duplicate(true)


## Writes one crash file and returns the path it wrote, or "" if the directory
## or the file could not be opened. The caller gets the path so a failure
## report can name the file the player would attach by hand.
func record_failure(reason: String) -> String:
	pending_failure_reason = reason
	var day := int(last_snapshot.get("campaign_day", 0))
	var segment := str(last_snapshot.get("time_segment", UNKNOWN_SEGMENT))
	if segment.is_empty():
		segment = UNKNOWN_SEGMENT
	if not DirAccess.dir_exists_absolute(LOG_DIRECTORY) and DirAccess.make_dir_recursive_absolute(LOG_DIRECTORY) != OK:
		push_error("CrashLog could not create %s; the failure was not recorded." % LOG_DIRECTORY)
		return ""
	var path := next_free_path(day, segment)
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		push_error("CrashLog could not write %s (error %d)." % [path, FileAccess.get_open_error()])
		return ""
	file.store_string(JSON.stringify(failure_report(reason, day, segment), "\t"))
	file.close()
	pending_failure_reason = ""
	return path


## Everything the file carries. Split out so the shape is readable in one
## place and so the suite can compare it field by field.
func failure_report(reason: String, day: int, segment: String) -> Dictionary:
	return {
		"reason": reason,
		"snapshot": last_snapshot.duplicate(true),
		"bundle_hash": bundle_hash(),
		"engine_version": Engine.get_version_info(),
		"written_at": {"campaign_day": day, "time_segment": segment}
	}


## `crash-<day>-<segment>-<n>.json`, with the lowest `n` this in-world hour has
## not used. The directory is the only record of what has already been written,
## because there is no clock to disambiguate two failures inside one hour.
func next_free_path(day: int, segment: String) -> String:
	var stem := "%s%d-%s-" % [FILE_PREFIX, day, segment]
	var counter := 0
	while FileAccess.file_exists("%s/%s%d%s" % [LOG_DIRECTORY, stem, counter, FILE_SUFFIX]):
		counter += 1
	return "%s/%s%d%s" % [LOG_DIRECTORY, stem, counter, FILE_SUFFIX]


func bundle_hash() -> String:
	if _bundle_hash_read:
		return _bundle_hash
	_bundle_hash_read = true
	var catalog := ContentCatalog.new()
	if catalog.load_default() == OK:
		_bundle_hash = catalog.content_hash
	return _bundle_hash


## The only two notifications this node answers, and both do the same thing:
## finish a write that was already under way. Neither of them starts one, so
## quitting a healthy game writes no file.
func _notification(what: int) -> void:
	if what == NOTIFICATION_CRASH or what == NOTIFICATION_WM_CLOSE_REQUEST:
		flush_pending_failure()


func flush_pending_failure() -> String:
	if pending_failure_reason.is_empty():
		return ""
	return record_failure(pending_failure_reason)
