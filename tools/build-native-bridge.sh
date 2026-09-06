#!/usr/bin/env bash
# Portable twin of tools/build-native-bridge.ps1: builds the Rust crate with the
# godot-ext feature and installs the GDExtension library under game/bin/<platform>/.
# Same sequence and same failure behaviour as the PowerShell gate; the library
# name, the platform directory, and -- on macOS alone -- how many architectures
# are built, are what differ per platform.
#
# E11. The installed name carries the architecture this build actually produced,
# read from `uname -m`, not a hard-coded x86_64. cargo builds for the host, so on
# an Apple Silicon runner the file used to be called ...x86_64.dylib while being
# arm64 -- and Godot, which resolves <platform>.<config>.<arch of the host it is
# running on>, then found no library at all and exported a build with no
# simulation in it. The token written here is the same one Godot asks for, and
# game/bin/project42_sim.gdextension declares a row per architecture to match.
# macOS builds both of its architectures; see the build plan below for why.
set -euo pipefail

configuration="${1:-debug}"
case "$configuration" in
    debug|release) ;;
    *)
        echo "Unknown configuration: $configuration (expected 'debug' or 'release')" >&2
        exit 2
        ;;
esac

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace="$(dirname -- "$script_dir")"
manifest="$workspace/godot-rust/Cargo.toml"

# Godot's architecture tokens, not the kernel's spelling: Godot writes arm64
# where Linux says aarch64, and x86_64 where some hosts say amd64. Anything else
# is refused rather than guessed at -- a wrong token here produces a library
# Godot silently will not load.
case "$(uname -m)" in
    x86_64|amd64)   architecture="x86_64" ;;
    aarch64|arm64)  architecture="arm64" ;;
    *)
        echo "Unsupported architecture for the native bridge build: $(uname -m)" >&2
        exit 2
        ;;
esac

case "$(uname -s)" in
    Linux)   platform="linux"; library_file="libproject42_sim.so";    library_extension="so" ;;
    Darwin)  platform="macos"; library_file="libproject42_sim.dylib"; library_extension="dylib" ;;
    MINGW*|MSYS*|CYGWIN*)
             platform="windows"; library_file="project42_sim.dll";    library_extension="dll" ;;
    *)
        echo "Unsupported platform for the native bridge build: $(uname -s)" >&2
        exit 2
        ;;
esac

# What to build, as "<rust target triple or empty for the host>:<Godot
# architecture token>". Every platform but macOS builds for its host alone.
#
# macOS is the exception, and not by choice. The official export-template
# archive ships one macOS template, godot_macos_release.universal, so the
# preset's binary_format/architecture has to be "universal" -- and for a
# universal preset Godot's exporter copies the GDExtension library declared for
# EVERY architecture into the .app, not just the host's. With only the host's
# dylib on disk the export answered "Failed to open
# ...project42_sim.macos.expedition_v5_release.x86_64.dylib" at
# core/io/dir_access.cpp:426 and wrote an .app with no simulation in it
# (nightly run 34031119836). So macOS builds both architectures, one of them by
# cross-compiling, which is also what makes the resulting .app honestly
# universal rather than a universal engine wrapped around a single-architecture
# library.
declare -a build_plan
if [[ "$platform" == "macos" ]]; then
    build_plan=("aarch64-apple-darwin:arm64" "x86_64-apple-darwin:x86_64")
else
    build_plan=(":$architecture")
fi

destination_directory="$workspace/game/bin/$platform"
mkdir -p "$destination_directory"

for plan in "${build_plan[@]}"; do
    target="${plan%%:*}"
    target_architecture="${plan##*:}"

    cargo_arguments=(build --manifest-path "$manifest" --features godot-ext)
    if [[ "$configuration" == "release" ]]; then
        cargo_arguments+=(--release)
    fi
    if [[ -n "$target" ]]; then
        cargo_arguments+=(--target "$target")
        # rustup ships the host target only. Adding the second one is part of
        # producing a universal build, so the script does it rather than
        # failing with a linker error the caller has to decode. Without rustup
        # (a distribution toolchain, say) the build simply fails and says so.
        if command -v rustup >/dev/null 2>&1; then
            if ! rustup target list --installed | grep -qx "$target"; then
                echo "Adding the Rust target $target for the universal macOS build"
                rustup target add "$target"
            fi
        fi
    fi

    build_status=0
    cargo "${cargo_arguments[@]}" || build_status=$?
    if [[ $build_status -ne 0 ]]; then
        echo "Rust GDExtension build failed with exit code $build_status" >&2
        exit "$build_status"
    fi

    if [[ -n "$target" ]]; then
        source_path="$workspace/godot-rust/target/$target/$configuration/$library_file"
    else
        source_path="$workspace/godot-rust/target/$configuration/$library_file"
    fi
    destination="$destination_directory/project42_sim.$platform.expedition_v5_$configuration.$target_architecture.$library_extension"

    if [[ ! -f "$source_path" ]]; then
        echo "Expected native library was not produced: $source_path" >&2
        exit 1
    fi
    cp -f "$source_path" "$destination"
    echo "Built native bridge: $destination"
done
