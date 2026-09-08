# Shared asset platform

The existing [shared art repository](https://github.com/dancockrell/shared-game-environment-library) owns reusable sprite sheets, environment art, props, portraits and effects. Pirate Island keeps the artwork it actually consumes under `game/assets/`.

Follow [visual authority](VISUAL_AUTHORITY.md). Preserve full-resolution artwork and colors, compatible perspective, stable ground pivots, animation timing and real transparency. Character deliverables are coherent animated sheets, not isolated poses presented as finished units.

Record source, license or generation provenance, source hash, consumer and actual visual review with each admitted asset. External sources require redistribution rights for the intended use. Do not generate paid assets without explicit approval.

Delete rejected and superseded assets from the working tree and remove their references. Git history supplies recovery. Use the existing repositories and established history, without forks or duplicate libraries.
