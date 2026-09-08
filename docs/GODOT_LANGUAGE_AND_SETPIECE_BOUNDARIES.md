# Godot Language and Setpiece Boundaries

## Default implementation language

GDScript is Project 42's default language. Use it for Godot scenes, world
cells, interaction, camera, UI, presentation, VFX, animation adapters,
content loading, editor tooling and ordinary game behaviour. Prefer Godot
nodes, resources, sprites, particles, audio and
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

The island uses a fixed elevated view with sprite layers, terrain artwork and an authoritative navigation mask. Logical collision and navigation stay separate from sprite bounds. Decorative artwork cannot invent routes, encounters or simulation state.
