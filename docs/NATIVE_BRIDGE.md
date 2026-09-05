# Native simulation bridge contract

## Purpose and ownership

Two Rust classes are visible to Godot. `Project42SimulationBridge` owns the standalone debug battle used by isolated presentation tests. `Project42ExpeditionBridge` owns the persistent expedition state and, when that state declares an encounter, the battle created for that exact handoff. `NativeSimulationPort` owns the former; `CampaignSession` owns the latter. Battle scenes still speak only to `SimulationPort`. This prevents engine nodes, animation code, content files, and UI widgets from acquiring partial simulation state.

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

## Campaign encounter handoff

Class name: `Project42ExpeditionBridge`

The campaign bridge accepts a validated route and encounter configuration, owns legal travel plus arrival state, then permits these battle calls only while its `pending_encounter` record exists:

| Method | Input | Output | Mutation |
| --- | --- | --- | --- |
| `begin_pending_battle` | none | snapshot `Dictionary` | creates the battle named by `pending_encounter.battle_id` |
| `start_pending_battle` | none | typed `Array[Dictionary]` | starts that retained battle |
| `submit_pending_command` | command `Dictionary` | typed `Array[Dictionary]` | resolves a command against that retained battle |
| `recommended_pending_enemy_command` | command ID `String` | command plus decision facts | none |
| `pending_battle_snapshot` | none | snapshot `Dictionary` | none |

The current authored trigger names `battle.prototype.returning_names`. Godot never passes a battle ID into `begin_pending_battle`; native code compares the pending stable ID before creating the supported vertical-slice battle. On a victory `battle_ended`, native code clears the pending encounter and applies its declared household result, so returning to the world restores legal travel only after the threat has been defeated. A defeat leaves the encounter pending for the later authored recovery/retry flow.

## Build outputs

The Rust library emits both `rlib` for engine-independent tests and `cdylib` for Godot. `tools/build-native-bridge.ps1` builds the selected configuration and copies the verified Windows library to `game/bin/windows/`. `game/bin/project42_sim.gdextension` maps debug and release libraries separately. The active pair is named `expedition_v5`: it accepts the JSON expedition configuration, creates a pending encounter only from the content entry explicitly marked `vertical_slice_encounter`, then owns the battle created through that pending handoff. Resolved encounter IDs are durable expedition state; a resolved authored encounter will not re-arm after a later arrival. An encounter may declare one stable estate upgrade; native state applies it only on victory, and the Godot snapshot exposes the resulting estate-upgrade IDs for authored location responses. The versioned filenames prevent a running Godot process from holding an obsolete DLL while a new native contract is being verified. Godot import is part of `tools/verify-godot.ps1`, because a compiled DLL that the editor has not registered is not a usable bridge.

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
13. A Reception Terrace encounter creates its named battle through `CampaignSession`, and the battle screen does not construct an unrelated debug battle. **Covered by `campaign_encounter_port_test.gd`.**
14. A resolved authored encounter persists through save/load and is excluded from future encounter arming. **Covered by `resolved_authored_encounter_stays_resolved_after_return_and_save_reload`.**
15. The Razorbeak victory applies its declared `estate_upgrade.river_gate_alarm`, and the damaged estate projects its authored household response. **Covered by Rust expedition state and `expedition_prototype_test.gd`.**
