# Godot Language and Setpiece Boundaries

> **Rule 0 — never fork.** A problem is to be solved, never dodged. Fix the thing,
> replace it outright, or delete the feature — those are the only three moves.
> Never leave two answers to one question standing side by side, and never route
> a parallel path around something you did not want to touch. That is a noodle to
> nowhere, and it is the most serious thing you can do to this codebase.
> Full rule: [`CLAUDE.md`](../CLAUDE.md).

## Default implementation language

GDScript is Project 42's default language. Use it for Godot scenes, world
cells, interaction, camera, UI, presentation, VFX, animation adapters,
content loading, editor tooling and ordinary game behaviour. Prefer Godot
nodes, resources, built-in meshes, materials, lighting, particles, audio and
navigation before introducing a custom subsystem.

TypeScript remains appropriate for offline content validators, schema builders
and asset-manifest tooling. It is not an in-game presentation dependency.

## Rust boundary

Rust is allowed only behind a small, tested native boundary where a measured
simulation, deterministic replay, save calculation, heavy procedural operation
or browser-sensitive performance hotspot cannot meet its budget in idiomatic
GDScript. Rust exposes typed data and commands; it does not create scenes, own
a visual node, animate an actor, choose art, or hold UI state.

Do not add Rust pre-emptively. Profile first, state the measured bottleneck,
write a narrow interface, and retain a GDScript-facing contract test.

## C and C++

Project code must not be authored in C or C++. Godot and third-party runtime
dependencies may use native code internally; that is not a project-language
choice. New performance work uses Rust when it genuinely needs native code.

## Environment best practice

Each playable location remains a `WorldCell` with separate visual, collision,
navigation, encounter, interactive, audio and camera layers. An environment
setpiece is composed from small named scene components. Reception Terrace uses
`Lighting`, `ProcessionalTerrace`, `ElvenGate`, `JungleMass`,
`ShipwreckFlotsam`, and `SeaAndSky`.

The initial kit may use Godot built-in primitives, `StandardMaterial3D`,
lights and fog to establish composition cheaply. A curated external asset
replaces one named component, never a tangle of individual runtime nodes.
Visual art never supplies collision, navigation, encounter placement, stable
world IDs or prose identifiers.
