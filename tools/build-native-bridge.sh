#!/usr/bin/env bash
# Portable twin of tools/build-native-bridge.ps1: builds the Rust crate with the
# godot-ext feature and installs the GDExtension library under game/bin/<platform>/.
# Same sequence and same failure behaviour as the PowerShell gate; the library
# name and the platform directory are the only things that differ per platform.
#
# E11. The installed name carries the architecture this build actually produced,
# read from `uname -m`, not a hard-coded x86_64. cargo builds for the host, so on
# an Apple Silicon runner the file used to be called ...x86_64.dylib while being
# arm64 -- and Godot, which resolves <platform>.<config>.<arch of the host it is
# running on>, then found no library at all and exported a build with no
# simulation in it. The token written here is the same one Godot asks for, and
# game/bin/project42_sim.gdextension declares a row per architecture to match.
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

cargo_arguments=(build --manifest-path "$manifest" --features godot-ext)
if [[ "$configuration" == "release" ]]; then
    cargo_arguments+=(--release)
fi

build_status=0
cargo "${cargo_arguments[@]}" || build_status=$?
if [[ $build_status -ne 0 ]]; then
    echo "Rust GDExtension build failed with exit code $build_status" >&2
    exit "$build_status"
fi

source_path="$workspace/godot-rust/target/$configuration/$library_file"
destination_directory="$workspace/game/bin/$platform"
destination_name="project42_sim.$platform.expedition_v5_$configuration.$architecture.$library_extension"
destination="$destination_directory/$destination_name"

if [[ ! -f "$source_path" ]]; then
    echo "Expected native library was not produced: $source_path" >&2
    exit 1
fi
mkdir -p "$destination_directory"
cp -f "$source_path" "$destination"
echo "Built native bridge: $destination"
