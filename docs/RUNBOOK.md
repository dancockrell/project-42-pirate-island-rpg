# Development runbook

## Godot editor

Open `game/project.godot` in Godot 4.7.2 or a later compatible Godot 4 maintenance release. The main scene is already configured. Press F6/F5 to show the battle prototype.

Expected screen: four vertical heroine cards on the left; one large active placeholder actor and one large razorbeak placeholder on the battle plane; descriptive observation text; four functional Betty command buttons. Clicking a heroine card changes the focused presentation actor. Clicking a command sends a stable-ID command through `SimulationPort` and projects returned events.

## Current deliberate mock

`MockSimulationPort` is development-only. It exists because the native Rust bridge is a separate boundary milestone. It must be removed from release exports. Rust tests already own the first real `Guarded Strike` rule; the next bridge task serializes Godot commands into the Rust library and maps `BattleEvent` values back into Dictionaries.

## Definition of done for replacing dummy art

Do not merely replace the colored rectangle with any image. Open `content/art/placeholders.json`, satisfy every listed gate, change the content reference to the final stable asset ID, set the placeholder record to deprecated or remove it through a reviewed migration, run `npm run validate`, and capture a 1920x1080 screenshot proving body, weapon and VFX safe margins.

