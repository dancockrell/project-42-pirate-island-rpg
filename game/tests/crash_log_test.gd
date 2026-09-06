extends SceneTree

## E7. Two proofs, and the second is the point of the card.
##
## 1. A failure recorded through the CrashLog autoload lands on disk as JSON
##    carrying the reason and the exact snapshot the session handed it, and a
##    second failure in the same in-world hour writes a second file instead of
##    overwriting the first.
## 2. No script under res://scripts/ mentions the engine's HTTP classes. That
##    is the whole no-telemetry guarantee: the crash file cannot be uploaded by
##    code that does not exist. The scan reads every .gd file itself rather
##    than trusting a review.
##
## Needs no native bridge and nothing beyond the generated content bundle, so
## it runs identically with or without the Rust GDExtension and prints none of
## the markers the CI gate greps for as evidence of a fallback path.
##
## Field-by-field comparisons rather than whole-dictionary ones: JSON writes
## every number as a float, so a parsed campaign day is 7.0 where the source
## dictionary held 7.

## quit() sets the exit code and returns; a trailing quit(0) would erase a
## failure. Count them and decide once at the end.
var failures := 0

const CrashLogScript = preload("res://scripts/diagnostics/crash_log.gd")
const SCRIPT_ROOT := "res://scripts"
const FORBIDDEN_NETWORK_CLASSES := ["HTTPRequest", "HTTPClient"]
const TEST_DAY := 7
const TEST_SEGMENT := "dusk"


func _initialize() -> void:
	var crash_log: Node = root.get_node_or_null("CrashLog")
	check(crash_log != null, "CrashLog must be registered as an autoload in project.godot")
	if crash_log == null:
		crash_log = CrashLogScript.new()
		crash_log.name = "CrashLog"
		root.add_child(crash_log)
	purge_test_logs()

	var snapshot := {
		"configured": true,
		"campaign_day": TEST_DAY,
		"time_segment": TEST_SEGMENT,
		"active_location_id": "world.cell.river_landing",
		"party_ids": ["actor.betty", "actor.ayla"]
	}
	crash_log.remember_snapshot(snapshot)

	var first_path: String = crash_log.record_failure("expedition_travel_refused")
	check(
		first_path == test_path(0),
		"the first failure of an in-world hour must be counter 0, got: %s" % first_path
	)
	var first := read_json(first_path)
	check(str(first.get("reason", "")) == "expedition_travel_refused", "the reason must round-trip")
	var recorded: Dictionary = first.get("snapshot", {})
	check(bool(recorded.get("configured", false)), "the remembered snapshot must round-trip")
	check(int(recorded.get("campaign_day", -1)) == TEST_DAY, "the snapshot's campaign day must round-trip")
	check(str(recorded.get("time_segment", "")) == TEST_SEGMENT, "the snapshot's time segment must round-trip")
	check(
		str(recorded.get("active_location_id", "")) == "world.cell.river_landing",
		"the snapshot's location must round-trip"
	)
	var party: Array = recorded.get("party_ids", [])
	check(party == ["actor.betty", "actor.ayla"], "the snapshot's party must round-trip")
	check(str(first.get("bundle_hash", "")).begins_with("sha256:"), "the content bundle hash must be recorded")
	var engine_version: Dictionary = first.get("engine_version", {})
	check(not str(engine_version.get("string", "")).is_empty(), "the engine version must be recorded")
	var written_at: Dictionary = first.get("written_at", {})
	check(
		int(written_at.get("campaign_day", -1)) == TEST_DAY and str(written_at.get("time_segment", "")) == TEST_SEGMENT,
		"written_at must be in-world time only -- this feature reads no wall clock anywhere"
	)

	# The counter is what lets the file name stay clock-free: a second failure
	# inside the same in-world hour must not overwrite the first.
	var second_path: String = crash_log.record_failure("battle_command_rejected")
	check(
		second_path == test_path(1),
		"a second failure in the same in-world hour must increment the counter, got: %s" % second_path
	)
	check(FileAccess.file_exists(first_path), "the first crash file must survive the second failure")
	check(str(read_json(second_path).get("reason", "")) == "battle_command_rejected", "the second reason must round-trip")
	check(
		str(read_json(first_path).get("reason", "")) == "expedition_travel_refused",
		"the first file must still hold the first failure"
	)

	# Closing a healthy game writes nothing. The notification hooks exist only
	# to finish a failure that was already under way when the process ended.
	check(str(crash_log.pending_failure_reason).is_empty(), "a completed record leaves no failure in flight")
	crash_log.notification(Node.NOTIFICATION_WM_CLOSE_REQUEST)
	check(not FileAccess.file_exists(test_path(2)), "a clean quit must write no crash file")
	crash_log.pending_failure_reason = "interrupted_midnight_resolution"
	crash_log.notification(Node.NOTIFICATION_WM_CLOSE_REQUEST)
	check(FileAccess.file_exists(test_path(2)), "a failure in flight when the window closes must be flushed")
	check(
		str(read_json(test_path(2)).get("reason", "")) == "interrupted_midnight_resolution",
		"the flushed reason must round-trip"
	)
	check(str(crash_log.pending_failure_reason).is_empty(), "a flushed failure is no longer in flight")

	check_no_network_classes()
	purge_test_logs()
	if failures == 0:
		print("Crash log tests passed.")
	quit(1 if failures > 0 else 0)


## The no-telemetry proof. Every .gd file under res://scripts/, read here and
## searched for the two engine classes that could send a crash file anywhere.
func check_no_network_classes() -> void:
	var scripts := gdscript_paths(SCRIPT_ROOT)
	check(scripts.size() > 10, "the scan must actually find the game's scripts, found %d" % scripts.size())
	for path in scripts:
		var file := FileAccess.open(path, FileAccess.READ)
		if file == null:
			check(false, "the no-telemetry scan could not read %s" % path)
			continue
		var text := file.get_as_text()
		file.close()
		for forbidden in FORBIDDEN_NETWORK_CLASSES:
			check(
				not text.contains(forbidden),
				"%s mentions %s: nothing in this game may send a crash log anywhere" % [path, forbidden]
			)


func gdscript_paths(directory: String) -> Array[String]:
	var result: Array[String] = []
	var dir := DirAccess.open(directory)
	if dir == null:
		check(false, "the no-telemetry scan could not open %s" % directory)
		return result
	for file_name in dir.get_files():
		if file_name.ends_with(".gd"):
			result.append("%s/%s" % [directory, file_name])
	for sub_directory in dir.get_directories():
		result.append_array(gdscript_paths("%s/%s" % [directory, sub_directory]))
	result.sort()
	return result


func test_path(counter: int) -> String:
	return "user://logs/crash-%d-%s-%d.json" % [TEST_DAY, TEST_SEGMENT, counter]


## This suite owns exactly the files it names: day 7 at dusk. Purged before the
## assertions so a leftover file from an earlier run cannot shift the counter,
## and again at the end so the suite leaves the user directory as it found it.
func purge_test_logs() -> void:
	var counter := 0
	while FileAccess.file_exists(test_path(counter)):
		DirAccess.remove_absolute(test_path(counter))
		counter += 1


func read_json(path: String) -> Dictionary:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		check(false, "crash file missing or unreadable: %s" % path)
		return {}
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	file.close()
	if not parsed is Dictionary:
		check(false, "crash file must be a JSON object: %s" % path)
		return {}
	return parsed


func check(condition: bool, message: String) -> void:
	if condition:
		return
	failures += 1
	push_error(message)
