#!/usr/bin/env bash
# The visual loop. Renders every scene the project presents -- everything under
# game/scenes/review, game/scenes/world, game/scenes/battle and
# game/scenes/shell -- to work/captures/<scene>.png, with a 1280x720
# <scene>.thumb.png beside it.
#
# Portable twin of tools/capture-scenes.ps1: the two scripts render the same
# scenes, fail on the same conditions and print the same messages; only the way
# Godot is located and the way its per-user directories are redirected differ
# per platform.
#
# Two engine runs, and never more: one import pass, then ONE process that
# renders every scene. That is the owner's standing instruction about Godot --
# an engine start costs about ten seconds and a great deal of memory, and a
# gate that opened and closed one per scene would spend most of its life
# starting up while every other thread on the machine waited.
#
# The gate is the one tools/verify-godot.sh applies: a step that exits 0 but
# printed an ERROR line has failed. On top of that the capture tool refuses a
# frame whose luminance is uniform, so a black render is a red run rather than
# a 40 KB PNG of nothing.
#
# Environment: GODOT, CAPTURE_OUTPUT_DIR, CAPTURE_WIDTH, CAPTURE_HEIGHT,
# CAPTURE_THUMBNAIL_WIDTH, CAPTURE_THUMBNAIL_HEIGHT, CAPTURE_FRAMES.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace="$(dirname -- "$script_dir")"

# Godot: $GODOT wins, else the first of godot4, godot on PATH. Never guess, and
# never pass silently when no engine is present. Same rule as verify-godot.sh.
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

output_dir="${CAPTURE_OUTPUT_DIR:-$workspace/work/captures}"
capture_width="${CAPTURE_WIDTH:-1920}"
capture_height="${CAPTURE_HEIGHT:-1080}"
thumbnail_width="${CAPTURE_THUMBNAIL_WIDTH:-1280}"
thumbnail_height="${CAPTURE_THUMBNAIL_HEIGHT:-720}"
settle_frames="${CAPTURE_FRAMES:-6}"

# Same redirection as verify-godot.sh, for the same reason: Godot writes to the
# user's config, data and cache directories, and a gate should not.
temp_root="${TMPDIR:-/tmp}"
task_profile="$temp_root/project42-godot-profile"
task_local="$temp_root/project42-godot-local"
mkdir -p "$task_profile" "$task_local" "$output_dir"
export XDG_CONFIG_HOME="$task_profile"
export XDG_DATA_HOME="$task_profile"
export XDG_STATE_HOME="$task_profile"
export XDG_CACHE_HOME="$task_local"

# The driver: Forward+ under Vulkan when a device is actually present --
# lavapipe on a runner with mesa-vulkan-drivers installed counts -- and the
# OpenGL compatibility renderer otherwise, which is what game/project.godot
# declares as the project's method. Which one ran is printed, and it belongs in
# any judgement of the images: the two do not light a scene identically.
rendering_driver="opengl3"
driver_reason="vulkaninfo is not installed"
if command -v vulkaninfo >/dev/null 2>&1; then
    if vulkaninfo --summary >/dev/null 2>&1; then
        rendering_driver="vulkan"
        driver_reason="$(vulkaninfo --summary 2>/dev/null | grep -m1 -i 'deviceName' | sed 's/^[[:space:]]*//')"
        [[ -n "$driver_reason" ]] || driver_reason="a Vulkan device is present"
    else
        driver_reason="vulkaninfo reports no device"
    fi
fi
echo "==> Rendering driver: $rendering_driver ($driver_reason)"
echo "==> Output directory: $output_dir"

error_pattern='^(ERROR|SCRIPT ERROR): |GDScript backtrace'

# The editor import pass, exactly as verify-godot.sh runs it. Without it the
# global script class cache is empty on a fresh checkout and every scene whose
# script names a class_name fails to parse -- which the ERROR check below would
# then report as a broken scene rather than an unimported project.
echo "==> Godot import and extension registration"
import_status=0
"$godot_executable" --headless --editor --path "$game_path" --quit || import_status=$?
if [[ $import_status -ne 0 ]]; then
    echo "Godot import and extension registration failed with exit code $import_status" >&2
    exit "$import_status"
fi

# Enumerated by glob rather than listed, so a new scene is captured the day it
# lands. A directory that does not exist yet -- scenes/shell, until the shell
# card builds it -- is skipped rather than failing the run.
shopt -s nullglob globstar
scenes=()
for directory in review world battle shell; do
    [[ -d "$game_path/scenes/$directory" ]] || continue
    for scene in "$game_path/scenes/$directory"/**/*.tscn; do
        scenes+=("res://${scene#"$game_path"/}")
    done
done
shopt -u nullglob globstar
if [[ ${#scenes[@]} -eq 0 ]]; then
    echo "No scenes found under $game_path/scenes/{review,world,battle,shell}/" >&2
    exit 1
fi
echo "==> ${#scenes[@]} scenes, one engine process"

# --audio-driver Dummy because a capture is silent, and a run on a machine with
# no sound device otherwise prints ALSA failures that the ERROR check below
# would read as a broken scene.
status=0
run_log="$(mktemp "${TMPDIR:-/tmp}/project42-captures.XXXXXX")"
"$godot_executable" \
    --rendering-driver "$rendering_driver" \
    --audio-driver Dummy \
    --path "$game_path" \
    --script res://tools/capture_review_scene.gd \
    -- "${scenes[@]}" \
    --out-dir "$output_dir" \
    --thumbnails \
    --frames "$settle_frames" \
    --size "${capture_width}x${capture_height}" \
    --thumbnail-size "${thumbnail_width}x${thumbnail_height}" \
    2>&1 | tee "$run_log" || status=${PIPESTATUS[0]}
if [[ $status -ne 0 ]]; then
    rm -f "$run_log"
    echo "The capture run failed with exit code $status" >&2
    exit "$status"
fi
if grep -Eq -- "$error_pattern" "$run_log"; then
    rm -f "$run_log"
    echo "The capture run exited 0 but printed an ERROR line; an exit code is not the only verdict" >&2
    exit 1
fi
rm -f "$run_log"

# A green run that wrote nothing is the exact failure this gate exists to make
# impossible, so the files are counted on disk rather than taken on trust.
written=0
for png in "$output_dir"/*.png; do
    if [[ -s "$png" ]]; then
        written=$((written + 1))
    fi
done
if [[ $written -eq 0 ]]; then
    echo "The capture run reported success but left no PNG in $output_dir" >&2
    exit 1
fi

echo "Captures complete: $written files in $output_dir on the $rendering_driver driver."
