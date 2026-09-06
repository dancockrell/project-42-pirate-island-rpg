#!/usr/bin/env bash
# Portable twin of tools/verify-godot.ps1: the Godot headless gate on Linux and
# macOS and in CI. The two scripts run the same sequence, fail on the same
# conditions and print the same messages; only the way Godot is located and the
# way its per-user directories are redirected differ per platform.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace="$(dirname -- "$script_dir")"

# Godot: $GODOT wins, else the first of godot4, godot on PATH. Never guess, and
# never pass silently when no engine is present.
godot_executable="${GODOT:-}"
if [[ -z "$godot_executable" ]]; then
    for candidate in godot4 godot; do
        if command -v "$candidate" >/dev/null 2>&1; then
            godot_executable="$(command -v "$candidate")"
            break
        fi
    done
fi
if [[ -z "$godot_executable" ]]; then
    echo "Godot executable not found: set GODOT to a Godot 4 binary, or put godot4 or godot on PATH." >&2
    exit 1
fi
if [[ ! -x "$godot_executable" ]]; then
    echo "Godot executable not found: $godot_executable" >&2
    exit 1
fi

game_path="$workspace/game"
if [[ ! -f "$game_path/project.godot" ]]; then
    echo "Godot project not found: $game_path/project.godot" >&2
    exit 1
fi

# Redirect Godot's per-user directories to task-specific temporary folders, as
# the PowerShell gate does with APPDATA and LOCALAPPDATA. On Linux and macOS the
# equivalent knobs are the XDG variables. Exported for this process only, so
# there is nothing to restore afterwards.
temp_root="${TMPDIR:-/tmp}"
task_profile="$temp_root/project42-godot-profile"
task_local="$temp_root/project42-godot-local"
mkdir -p "$task_profile" "$task_local"
export XDG_CONFIG_HOME="$task_profile"
export XDG_DATA_HOME="$task_profile"
export XDG_STATE_HOME="$task_profile"
export XDG_CACHE_HOME="$task_local"

# A suite that calls quit(1) on a failed check and then reaches an
# unconditional quit(0) exits 0: SceneTree.quit() sets the exit code and
# returns, and the last call wins. Fifteen of the seventeen suites had that
# shape on the day this was written, and world_cell_test.gd passed a missing
# scene anchor straight through a green gate. So the exit code is not the
# only verdict: any ERROR line -- push_error, a script error, a backtrace --
# in a step's output fails that step. Legitimate runs print none; that was
# checked against the gate logs before this rule was added.
error_pattern='^(ERROR|SCRIPT ERROR): |GDScript backtrace'

# A suite that trips a bare assert() halts in the debugger and never reaches
# its quit(), so without a ceiling the gate waits on it forever (the P5
# integration lost a ten-minute run to placeholder_action_presenter_test.gd
# that way). Each step gets a ceiling. The longest legitimate step on the
# pinned engine under CI's software renderer is well under a minute; the
# ceiling is generous so a slow runner is never mistaken for a hang. GNU
# timeout is present on Linux and in CI; where it is absent (a bare macOS)
# the step runs unbounded and says so once, rather than silently.
step_ceiling_seconds="${GODOT_STEP_TIMEOUT_SECONDS:-240}"
step_timeout=()
if command -v timeout >/dev/null 2>&1; then
    step_timeout=(timeout --signal=TERM --kill-after=10 "$step_ceiling_seconds")
elif command -v gtimeout >/dev/null 2>&1; then
    step_timeout=(gtimeout --signal=TERM --kill-after=10 "$step_ceiling_seconds")
else
    echo "No timeout binary on PATH; gate steps run without a ceiling." >&2
fi

run_godot() {
    local description="$1"
    shift
    local status=0
    local step_log
    step_log="$(mktemp "${TMPDIR:-/tmp}/project42-godot-step.XXXXXX")"
    # 2>&1 so push_error (stderr) is judged alongside stdout; tee so the gate
    # log the CI job archives still carries every line.
    "${step_timeout[@]}" "$godot_executable" --headless --path "$game_path" "$@" 2>&1 | tee "$step_log" || status=${PIPESTATUS[0]}
    if [[ $status -eq 124 ]]; then
        rm -f "$step_log"
        echo "$description hung past the ${step_ceiling_seconds}s ceiling and was stopped; a suite that halts on a bare assert() never reaches its quit()" >&2
        exit 124
    fi
    if [[ $status -ne 0 ]]; then
        rm -f "$step_log"
        echo "$description failed with exit code $status" >&2
        exit "$status"
    fi
    if grep -Eq -- "$error_pattern" "$step_log"; then
        rm -f "$step_log"
        echo "$description exited 0 but printed an ERROR line; a suite's exit code is not the only verdict" >&2
        exit 1
    fi
    rm -f "$step_log"
}

# 1. Editor import pass: registers extensions and imports resources.
echo "==> Godot import and extension registration"
import_status=0
"$godot_executable" --headless --editor --path "$game_path" --quit || import_status=$?
if [[ $import_status -ne 0 ]]; then
    echo "Godot import and extension registration failed with exit code $import_status" >&2
    exit "$import_status"
fi

# 2. Headless run of the configured main scene.
echo "==> Godot headless main scene"
run_godot "Godot headless verification" --quit-after 3

# 3. Review-scene smoke runs, enumerated rather than listed so a new review
#    scene is covered the day it lands.
shopt -s nullglob
review_scenes=("$game_path"/scenes/review/*_review.tscn)
shopt -u nullglob
if [[ ${#review_scenes[@]} -eq 0 ]]; then
    echo "No review scenes found under $game_path/scenes/review/" >&2
    exit 1
fi
for scene in "${review_scenes[@]}"; do
    scene_name="$(basename -- "$scene")"
    echo "==> Review scene $scene_name"
    run_godot "Godot review scene $scene_name" --scene "res://scenes/review/$scene_name" --quit-after 2
done

# 4. Every headless GDScript suite, enumerated by glob so the list cannot drift
#    away from what is on disk.
shopt -s nullglob
suites=("$game_path"/tests/*_test.gd)
shopt -u nullglob
if [[ ${#suites[@]} -eq 0 ]]; then
    echo "No GDScript suites found under $game_path/tests/" >&2
    exit 1
fi
for suite in "${suites[@]}"; do
    suite_name="$(basename -- "$suite")"
    echo "==> Suite $suite_name"
    run_godot "Godot $suite_name" --script "res://tests/$suite_name"
done

echo "Godot headless verification passed. ${#review_scenes[@]} review scenes, ${#suites[@]} suites."
