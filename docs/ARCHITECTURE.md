# Architecture contract

## Authority

The simulation is the only authority for vitality, guard, composure, initiative, legal targets, costs, hit results, damage, status application, defeat, loot, time advancement, respawn and death memory. Godot submits commands and projects returned events. A visual animation may anticipate an accepted command, but it may not change authoritative state.

## Stable identifiers

Identifiers are lowercase dotted strings. Their prefix names the domain: `character.heroine.betty`, `skill.betty.guarded_strike`, `enemy.raptor.razorbeak`, `location.tomb.returning_names`, `art.placeholder.betty.active_actor`. IDs survive file moves and scene refactors.

## Metadata locality

Every content record keeps its implementation facts beside the record that consumes them. Required metadata includes ownership, lifecycle, source status, tags, art state, accessibility description, dependencies and validation notes. Cross-domain relationships use stable IDs. Prose may explain a rule; only schemas and executable records define it.

## Placeholder-art contract

Every placeholder has `placeholder: true`, a visible `DUMMY` label, a unique asset ID, an intended final-art description, framing requirements, replacement acceptance tests and a named consumer. Placeholder assets may ship only in development builds. The TypeScript validator fails a release profile containing placeholders.

## Command/event boundary

Godot sends commands shaped as `{command_id, battle_id, actor_id, kind, target_ids, payload}`. Rust returns `{event_id, command_id, sequence, kind, subjects, payload}`. Presentation code switches on event `kind`; it never parses human-readable combat prose to discover results.

