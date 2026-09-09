#!/usr/bin/env bash
# Portable twin of tools/build-native-bridge.ps1: builds the Rust GDExtension
# with the godot-ext feature and copies the shared library to the exact path
# game/bin/project42_sim.gdextension declares for this platform. The two scripts
# perform the same sequence and name the same artefacts; only the library
# extension and destination folder differ per platform.
#
# Usage: tools/build-native-bridge.sh [debug|release]
set -euo pipefail

configuration="${1:-debug}"
case "$configuration" in
    debug|release) ;;
    *)
        echo "Configuration must be debug or release, got: $configuration" >&2
        exit 1
        ;;
esac

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
workspace="$(dirname -- "$script_dir")"
manifest="$workspace/godot-rust/Cargo.toml"

arguments=(build --manifest-path "$manifest" --features godot-ext)
if [[ "$configuration" == "release" ]]; then
    arguments+=(--release)
fi
cargo "${arguments[@]}"

case "$(uname -s)" in
    Linux)
        platform="linux"
        library="libproject42_sim.so"
        suffix="so"
        ;;
    Darwin)
        platform="macos"
        library="libproject42_sim.dylib"
        suffix="dylib"
        ;;
    *)
        echo "Unsupported platform for this script: $(uname -s). Use tools/build-native-bridge.ps1 on Windows." >&2
        exit 1
        ;;
esac

source="$workspace/godot-rust/target/$configuration/$library"
if [[ ! -f "$source" ]]; then
    echo "Expected native library was not produced: $source" >&2
    exit 1
fi

destination_directory="$workspace/game/bin/$platform"
destination="$destination_directory/project42_sim.$platform.template_$configuration.x86_64.$suffix"
mkdir -p "$destination_directory"
cp -f "$source" "$destination"
echo "Built native bridge: $destination"
