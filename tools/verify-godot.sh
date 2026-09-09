#!/usr/bin/env bash
# Portable twin of tools/verify-godot.ps1: the bounded Godot verification of the
# current sprite-based island on Linux and macOS and in CI. The two scripts run
# the same suites, require the same completion lines, fail on the same
# conditions and print the same closing message; only the way Godot is located
# and the way its per-user directories are isolated differ per platform.
#
# Usage: GODOT=/path/to/godot tools/verify-godot.sh [timeout-seconds]
#   Godot: $GODOT wins, else .local-tools/godot-4.7.2/Godot_v4.7.2-stable_linux.x86_64,
#   else the first of godot4, godot on PATH.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace="$(dirname -- "$script_dir")"
timeout_seconds="${1:-30}"
if ! [[ "$timeout_seconds" =~ ^[0-9]+$ ]] || (( timeout_seconds < 1 || timeout_seconds > 60 )); then
    echo "Timeout must be a whole number of seconds from 1 to 60, got: $timeout_seconds" >&2
    exit 1
fi

godot_executable="${GODOT:-}"
if [[ -z "$godot_executable" ]]; then
    candidate="$workspace/.local-tools/godot-4.7.2/Godot_v4.7.2-stable_linux.x86_64"
    if [[ -x "$candidate" ]]; then
        godot_executable="$candidate"
    else
        for name in godot4 godot; do
            if command -v "$name" >/dev/null 2>&1; then
                godot_executable="$(command -v "$name")"
                break
            fi
        done
    fi
fi
if [[ -z "$godot_executable" || ! -x "$godot_executable" ]]; then
    echo "Godot executable not found: set GODOT to a Godot 4 binary, place it under .local-tools/godot-4.7.2/, or put godot4 or godot on PATH." >&2
    exit 1
fi

game_path="$workspace/game"
if [[ ! -f "$game_path/project.godot" ]]; then
    echo "Godot project not found: $game_path/project.godot" >&2
    exit 1
fi

# 1. Editor import pass. A fresh checkout has no .godot/ cache, and without it
#    a script that preloads another by path fails to resolve ("Could not
#    resolve script"), which is exactly what the first CI run of this gate hit.
#    The pass registers the extension and imports resources, then quits.
echo "==> Godot import and extension registration"
import_status=0
(
    cd "$workspace"
    timeout --kill-after=5 120 "$godot_executable" --headless --editor --path "$game_path" --quit
) >/dev/null 2>&1 || import_status=$?
if [[ $import_status -ne 0 ]]; then
    echo "Godot import and extension registration failed with exit code $import_status" >&2
    exit "$import_status"
fi

# 2. Verify the current sprite-based island. The suite list and each completion
# line are the ones verify-godot.ps1 requires; a suite that exits 0 without
# printing its line has not finished, and any ERROR line fails it regardless
# of the exit code.
checks=(
    "res://tests/directional_sprite_test.gd|PASS: four facings, alpha-source identity, frame regions, nearest sampling, idle retention, no simulation movement"
    "res://tests/island_scene_test.gd|PASS: island scene suite complete"
    "res://tests/island_bridge_test.gd|PASS: actual Rust island bridge, scenario document, solo start, movement, ocean rejection, pause, arrival and the campaign clock"
)
error_pattern='^(SCRIPT ERROR:|ERROR:|USER ERROR:)'

for check in "${checks[@]}"; do
    script="${check%%|*}"
    completion="${check#*|}"
    # Unique profiles avoid concurrent tests sharing or overwriting user saves.
    profile="$(mktemp -d "${TMPDIR:-/tmp}/project42-test.XXXXXXXX")"
    output_log="$profile/godot-output.log"
    status=0
    (
        export XDG_CONFIG_HOME="$profile" XDG_DATA_HOME="$profile" XDG_STATE_HOME="$profile" XDG_CACHE_HOME="$profile"
        cd "$workspace"
        timeout --kill-after=5 "$timeout_seconds" "$godot_executable" \
            --headless --path "$game_path" --script "$script" --quit-after 120
    ) >"$output_log" 2>&1 || status=$?
    cat "$output_log"
    if [[ $status -eq 124 || $status -eq 137 ]]; then
        echo "Godot test timed out after $timeout_seconds seconds: $script" >&2
        echo "Isolated test profile: $profile"
        exit 1
    fi
    if [[ $status -ne 0 ]] || grep -Eq -- "$error_pattern" "$output_log"; then
        echo "Godot test failed: $script (exit $status)" >&2
        echo "Isolated test profile: $profile"
        exit 1
    fi
    if ! grep -Fxq -- "$completion" "$output_log"; then
        echo "Godot exited without completing test: $script" >&2
        echo "Isolated test profile: $profile"
        exit 1
    fi
    # Keep isolated diagnostics for inspection; never prune shared user data.
    echo "Isolated test profile: $profile"
done
echo "Current 2D Godot verification passed (not rendered visual acceptance)."
