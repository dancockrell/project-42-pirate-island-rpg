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

run_godot() {
    local description="$1"
    shift
    local status=0
    "$godot_executable" --headless --path "$game_path" "$@" || status=$?
    if [[ $status -ne 0 ]]; then
        echo "$description failed with exit code $status" >&2
        exit "$status"
    fi
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
