# Native simulation bridge contract

## Purpose and ownership

`Project42SimulationBridge` is the only Godot-visible Rust class. It owns one `Battle` instance for the vertical slice. `NativeSimulationPort` owns the bridge object on the Godot side. Battle scenes talk only to `SimulationPort`. This prevents engine nodes, animation code, content files, and UI widgets from acquiring their own partial simulation state.

## Pinned compatibility decision

The bridge will use the official `godot` crate v0.5 with the `api-4-2` feature. Godot 4.7 can load an extension built against the older compatible 4.2 API, while pinning the minimum API avoids accidentally depending on a 4.7-only engine method. Do not add `api-custom`; that would make reproducibility depend on one developer's editor binary and LLVM setup.

The dependency is present as optional `godot` crate v0.5.5 with `api-4-2`, enabled by the `godot-ext` feature. `cargo check --features godot-ext`, native debug compilation, extension registration under Godot 4.7.2, command submission, rejection-without-mutation, and headless bridge tests all pass on the development host.

## Exact Godot-visible class

Class name: `Project42SimulationBridge`

Base class: `RefCounted`

Methods:

| Method | Input | Output | Mutation |
| --- | --- | --- | --- |
| `create_debug_battle` | none | snapshot `Dictionary` | replaces the owned battle with the prototype encounter |
| `start_battle` | none | typed `Array[Dictionary]` | moves `AwaitingActor` to `AwaitingCommand` |
| `submit_command` | command `Dictionary` | typed `Array[Dictionary]` | resolves one legal command or emits one rejection |
| `recommended_enemy_command` | command ID `String` | command plus structured decision facts `Dictionary` | none |
| `snapshot` | none | snapshot `Dictionary` | none |

Every command contains `protocol_version`, `command_id`, `battle_id`, `actor_id`, `kind`, `skill_id`, and `target_ids`. Protocol version 1 accepts only `kind=use_skill`. Missing IDs and unsupported versions are rejected before calling `Battle::submit`.

Every event contains `event_id`, `command_id` when applicable, monotonically increasing `sequence`, `kind`, `subjects`, and `payload`. Event payloads contain primitive values and stable IDs. They never contain Godot node paths, localized prose, or asset paths.

## Build outputs

The Rust library emits both `rlib` for engine-independent tests and `cdylib` for Godot. `tools/build-native-bridge.ps1` builds the selected configuration and copies the verified Windows library to `game/bin/windows/`; `tools/build-native-bridge.sh` is its twin for Linux and macOS and writes to `game/bin/linux/` or `game/bin/macos/`. `game/bin/project42_sim.gdextension` maps debug and release libraries separately for each platform. Godot import is part of `tools/verify-godot.ps1` and `tools/verify-godot.sh`, because a compiled library the editor has not registered is not a usable bridge.

The Windows libraries under `game/bin/windows/` are committed; the Linux and macOS ones are built locally and ignored. A committed library goes stale the moment the bridge's exported verbs change, and a contributor without Windows cannot rebuild it, so the `Windows native bridge` CI job builds both configurations on every pull request, reports whether the committed pair still matches the code, and uploads the result as the `windows-bridge-<sha>` artefact. Download it and commit it when it differs. CI never commits a binary itself: an artefact enters the repository through a person who has looked at it.

## Acceptance gates

The bridge milestone is complete only when all of the following are true:

1. `cargo test` passes the engine-independent suite. **Passing.**
2. `cargo check --features godot-ext` passes. **Passing.**
3. A native debug library builds. **Passing.**
4. Godot loads and registers the `.gdextension`. **Passing.**
5. `ClassDB.can_instantiate("Project42SimulationBridge")` returns true. **Passing.**
6. A guarded-strike command submitted through `NativeSimulationPort` emits the same ordered opening events and final hostile vitality as the direct Rust test. **Passing.**
7. An unsupported protocol version returns `command_rejected` without mutating the battle. **Passing.**
8. The native fixture contains Betty, wounded Vix, defeated Ayla and one level-seven razorbeak, so party healing, ordered rescue, interception, automatic reaction eligibility and revival are representable without mock state. **Passing.**
9. Godot-boundary tests prove Condition Cleanse, Rescue Charge plus the following enemy interception, Mobile Infirmary's living-party pulse, and Combat Revival's status removal, forty-percent restore and immediate bonus turn. **Passing.**
10. The prototype drives non-player turns through ordinary native commands: the razorbeak uses Rushing Bite, while party members without executable kits use the explicit temporary `Hold Position` action. A full Betty-to-enemy-to-support-to-Betty initiative cycle no longer freezes after the first player command. **Passing.**
11. The intent event and the submitted enemy command derive from one Rust `EnemyDecision`, including target, rationale, projected damage, lethality, interception and reaction facts. **Passing.**
12. A release runtime cannot select `MockSimulationPort`. **Enforced in `BattlePrototype._ready`.**
