# Native simulation bridge contract

## Purpose and ownership

`Project42SimulationBridge` is the only Godot-visible Rust class. It owns one `Battle` instance for the vertical slice. `NativeSimulationPort` owns the bridge object on the Godot side. Battle scenes talk only to `SimulationPort`. This prevents engine nodes, animation code, content files, and UI widgets from acquiring their own partial simulation state.

## Pinned compatibility decision

The bridge will use the official `godot` crate v0.5 with the `api-4-2` feature. Godot 4.7 can load an extension built against the older compatible 4.2 API, while pinning the minimum API avoids accidentally depending on a 4.7-only engine method. Do not add `api-custom`; that would make reproducibility depend on one developer's editor binary and LLVM setup.

The dependency is not yet present in `Cargo.toml`. Cargo cannot reach crates.io on this Windows host because its TLS backend fails before authentication. The attempted dependency was removed after the failed verification. Add it only when `cargo check --features godot-ext` can complete and update `Cargo.lock` from the official registry.

## Exact Godot-visible class

Class name: `Project42SimulationBridge`

Base class: `RefCounted`

Methods:

| Method | Input | Output | Mutation |
| --- | --- | --- | --- |
| `create_debug_battle` | none | snapshot `Dictionary` | replaces the owned battle with the prototype encounter |
| `start_battle` | none | typed `Array[Dictionary]` | moves `AwaitingActor` to `AwaitingCommand` |
| `submit_command` | command `Dictionary` | typed `Array[Dictionary]` | resolves one legal command or emits one rejection |
| `snapshot` | none | snapshot `Dictionary` | none |

Every command contains `protocol_version`, `command_id`, `battle_id`, `actor_id`, `kind`, `skill_id`, and `target_ids`. Protocol version 1 accepts only `kind=use_skill`. Missing IDs and unsupported versions are rejected before calling `Battle::submit`.

Every event contains `event_id`, `command_id` when applicable, monotonically increasing `sequence`, `kind`, `subjects`, and `payload`. Event payloads contain primitive values and stable IDs. They never contain Godot node paths, localized prose, or asset paths.

## Build outputs

The eventual Rust library target is `cdylib`. Windows development output goes to `game/bin/windows/project42_sim.windows.template_debug.x86_64.dll`. The `.gdextension` file goes to `game/bin/project42_sim.gdextension` and maps Windows debug and release libraries separately. Neither file may be invented or checked in before a native build succeeds.

## Platform scope

The native library build and `tools/verify-godot.ps1` are Windows-only steps: they produce a `.windows.template_debug.x86_64.dll` and drive a locally installed Windows Godot binary. A Linux or CI environment without that toolchain should skip both rather than treat their absence as a failure; `cargo test` and `node tools/src/validate.mjs` remain fully runnable everywhere and are the checks to rely on outside Windows.

## Acceptance gates

The bridge milestone is complete only when all of the following are true:

1. `cargo test` passes the engine-independent suite.
2. `cargo check --features godot-ext` passes.
3. A native debug library builds.
4. Godot loads the `.gdextension` without an editor error.
5. `ClassDB.can_instantiate("Project42SimulationBridge")` returns true.
6. A command submitted through `NativeSimulationPort` produces the same ordered event kinds and final snapshot as the corresponding direct Rust test.
7. An unsupported protocol version returns `command_rejected` without mutating the battle.
8. A release export cannot select `MockSimulationPort`.
