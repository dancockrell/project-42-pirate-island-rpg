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

## Encounter state machine

Every encounter moves through `AwaitingActor -> AwaitingCommand -> Resolving -> AwaitingCommand`, ending in `Victory` or `Defeat`. Initiative selects the active actor. A command naming any other actor is rejected before mutation. Resolution emits ordered events: acceptance, actor focus, mechanical changes, defeat if any, turn end, then the next turn and visible enemy intent. Godot may animate that event sequence; it may not skip ahead and calculate the result itself.

The executable Rust slice currently implements Betty's `Guarded Strike`, `Condition Cleanse`, and `Rescue Charge`, plus the razorbeak's `Rushing Bite`. Later Betty skills remain fully specified as content records but stay locked in the prototype UI until each receives a tested Rust resolver. A locked button is an honest production state, not an implied implementation.

`Condition Cleanse` names one living party member, removes no more than two negative statuses in the fixed urgency order Stunned, Burning, Poisoned, Bleeding, then restores eight Vitality without exceeding maximum Vitality. The fixed order prevents array insertion order from deciding a combat result.

`Rescue Charge` names an ordered pair: first a threatened party member other than Betty, then the hostile threatening her. Betty moves to the ally's band, deals ten Guard-absorbed impact damage to the named hostile, and intercepts the next hostile attack aimed at that ally. An interception redirects one attack to Betty and then clears itself. Requiring both targets keeps targeting, animation staging and AI evaluation deterministic.

## Midnight return transaction

Midnight is one atomic simulation transaction. The clock advances to the next day, named people become alive again without losing their death counters, then each region emits its deterministic daily monster instances in visible flashes. The same world seed, new day, region, and slot always produce the same instance. Each spawned monster receives an individual physical variant, condition, purpose, level, and loot seed. Prototype spawn rules enforce a group size of one because ordinary wilderness encounters are meant to read as D&D-like individual power relationships rather than anonymous packs of one-hit enemies.

## Runtime content bundle

Authoring remains split into local records under `content/` so ownership and relationships stay readable. `npm run build:content` validates those records, sorts them by stable ID, and writes one deterministic `game/generated/content_bundle.json` for Godot. The bundle includes every source path and a SHA-256 hash of its canonical record array. Godot loads the bundle; it does not crawl authoring directories or invent defaults. The generated file is committed so a build always identifies the exact validated content it consumed.
