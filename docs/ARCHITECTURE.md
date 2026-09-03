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

## Targeting and animation presentation

`TargetingSession` is a presentation-side collector driven by each skill record's `targetRule`. It controls prompts, ordered selection and obvious local filtering so the player cannot casually click nonsense, but it never makes a command authoritative. Rust validates the submitted IDs again against current battle state. Zero-selection actions resolve immediately. Automatic reactions remain visible in the command display but cannot enter manual targeting. Ordered multi-target skills retain click order in `target_ids`; Rescue Charge therefore always submits the threatened heroine first and the threatening hostile second.

`SkillAnimationDirector` reads the skill record's ordered `animation.beats`, waits on authored millisecond offsets and emits signals for actor poses, VFX, camera and audio consumers. Each skill keeps `animation.eventBindings` beside those beats, mapping authoritative event kinds to the named beat where their already-decided result becomes visible. The director never applies damage, healing, status or movement. It schedules returned events onto beats while monotonically clamping cue times so presentation can never reorder Rust's event sequence. Unbound events remain ordered and release at action end. The command is accepted before presentation starts; the next actor's `TurnStarted` and subsequent turn events remain outside the current action segment.

Every authored beat has exactly one `presentationCue`. A cue names the pose, stage motion, camera treatment, VFX plate, audio cue, hit stop, normalized shake strength and all subjects that must remain inside the safe frame. `PlaceholderActionPresenter` proves the runtime consumes these fields by changing the dummy actor treatment and displaying the complete cue. It is explicitly a development adapter; final sprite, camera, VFX and audio consumers subscribe to the same director signal and replace it without changing simulation or skill data.

Camera treatments are stable registry entries under `presentation.camera.*`, not free-form labels. Each record defines mode, zoom, focus subjects, lead subject, safe padding and transition timing. Skill cues reference `cameraId`; `ContentCatalog` resolves a defensive copy from the bundled registry. The camera registry may shape framing only. It cannot change targets, range, hit results or any other simulation value.

VFX treatments follow the same stable-reference rule under `presentation.vfx.*`. Each effect record states exactly what gameplay result it communicates, its palette, attachment anchor, render layer, blend mode, screen envelope, motion direction, lifetime, reduced-flash substitute, safe-frame rule and asset status. Skill cues contain `vfxId`, never an improvised effect label. `ContentCatalog` resolves the record for the Godot presentation consumer after Rust has already decided the result. A VFX record can explain or emphasize an event; it cannot create damage, healing, guard, targeting or state changes.

Generated animation sources enter through `presentation.motion.*` records. A motion record keeps the character and usage, Magnific creation provenance, model and encoding details, duration, frame rate, dimensions, loop and camera rules, required in-frame subjects, intended motion, rejection gates, runtime path and separate import and approval states. A remote render is a review candidate, never an implicit game asset. It becomes `imported` only after the file exists under `res://`; it becomes `approved` only after identity, loop seam, weapon continuity and battle-safe framing are reviewed. This keeps rapid generated animation useful without allowing a visually unstable render to become a hidden dependency.

## Encounter state machine

Every encounter moves through `AwaitingActor -> AwaitingCommand -> Resolving -> AwaitingCommand`, ending in `Victory` or `Defeat`. Initiative selects the active actor. A command naming any other actor is rejected before mutation. Resolution emits ordered events: acceptance, actor focus, mechanical changes, defeat if any, turn end, then the next turn and visible enemy intent. Godot may animate that event sequence; it may not skip ahead and calculate the result itself.

The executable Rust slice implements all seven of Betty's bond skills: `Guarded Strike`, `Condition Cleanse`, `Rescue Charge`, `Healing Impact`, `Fatal Intercept`, `Mobile Infirmary`, and `Combat Revival`, plus the razorbeak's `Rushing Bite`. A skill remains locked in the prototype UI until its targeting presentation and native event projection are connected even when its authoritative resolver is complete. A locked button is an honest production state, not an implied implementation. The command validator binds each implemented signature skill to its stable owner ID, so another actor cannot execute Betty's rules merely by submitting Betty's skill ID.

`Hold Position` (`skill.system.hold_position`) is the sole owner-neutral prototype action. It takes no target, grants two Guard and ends the actor's turn. The automatic prototype driver uses it for a heroine whose authored kit is not executable yet. It is scaffolding, not a substitute signature skill, and must disappear from that heroine's behavior as soon as her real command set is implemented. The driver submits ordinary commands through `SimulationPort`; it never advances initiative or changes Guard in presentation code.

The active hostile's warning and executed action originate from one immutable Rust `EnemyDecision`. For the prototype Razorbeak, candidates are living party members ordered by lowest current-Vitality percentage, then lowest Guard, then stable actor ID. The decision exposes a stable rationale code, raw damage, projected Guard absorption, projected Vitality damage, lethality, an existing interception protector and Fatal Intercept availability. Godot turns those facts into readable prose and submits the recommended command unchanged. Presentation never chooses a different target, and prose never becomes simulation input.

`CombatTextRenderer` is the sole event-to-combat-prose adapter. It is a pure Godot `RefCounted` object configured with the current stable-ID-to-display-name table. Every sentence names the event's actual subjects and formats primitive payload facts; it may not hard-code a heroine as the recipient of a rescue, interception, revival, movement, heal or bonus turn. It returns `replace`, `append` or `none` plus optional intent-header text. The battle screen applies that result while separately projecting state changes. Text can be rewritten or localized without changing a command, event, target, damage result or card state.

Events carry semantic display facts instead of requiring the renderer to parse instance identifiers. `status_removed`, for example, includes both the unique `status_id` used for state tracking and the explicit `status_kind` used for prose, icons and accessibility output. The string `status.vix.poisoned.prototype` is never split to guess that the condition was Poisoned.

`Condition Cleanse` names one living party member, removes no more than two negative statuses in the fixed urgency order Stunned, Burning, Poisoned, Bleeding, then restores eight Vitality without exceeding maximum Vitality. The fixed order prevents array insertion order from deciding a combat result.

`Rescue Charge` names an ordered pair: first a threatened party member other than Betty, then the hostile threatening her. Betty moves to the ally's band, deals ten Guard-absorbed impact damage to the named hostile, and intercepts the next hostile attack aimed at that ally. An interception redirects one attack to Betty and then clears itself. Requiring both targets keeps targeting, animation staging and AI evaluation deterministic.

`Healing Impact` names one living hostile and deals `18 + Betty's level` raw damage. Guard applies first. Healing equals half of actual Vitality removed, rounded down. The recipient is the living party member with the lowest current-Vitality percentage, compared through integer cross multiplication; ties resolve by stable actor ID. Healing occurs after damage and defeat events but before victory, and cannot exceed maximum Vitality.

`Mobile Infirmary` is a normal action with no manually selected targets. It creates one battlefield effect and immediately pulses it. A pulse restores up to ten Vitality and grants two Guard to every living party member in stable actor-ID order; defeated party members receive nothing. The same effect pulses at the start of Betty's next two turns, for three pulses total, then removes itself before she submits the third turn's command. Stun prevents Betty from submitting a new command but does not suspend an infirmary already deployed. Betty's defeat removes every battlefield effect sourced by her. Recasting Mobile Infirmary removes the previous instance with the reason `replaced` before creating the new three-pulse instance. Presentation may anchor supplies and aura effects to Betty, but their screen position never determines legal recipients.

Persistent battlefield mechanics live in the general `BattlefieldEffect` collection. Each instance records a stable instance ID, source actor, source skill and remaining pulses. Creation, each pulse and removal are explicit ordered events. A snapshot includes the active effects so save/load, reconnect, tests and UI reconstruction do not depend on replaying animation state.

`Combat Revival` is Betty's once-per-battle SSS normal action. It names exactly one defeated party member other than Betty. The target returns with 40 percent of maximum Vitality using ceiling rounding, never less than one, zero Guard and no remaining negative statuses. Status-removal events precede the revival event. The simulation then emits a bonus-turn grant and starts the revived heroine's turn immediately after Betty's turn ends. A stored initiative continuation resumes at the actor who would naturally have followed Betty; the bonus turn does not consume the revived heroine's ordinary initiative slot. The continuation records both turn-order index and round so a revival cast by the last natural actor advances the round only after the bonus turn. Battle snapshots include both a pending forced actor and its continuation.

Combat defeat is not an island death. Combat Revival never increments or clears a named person's persistent death counter, never invokes midnight return and never changes world time. Fatal Intercept resolves before an actor reaches defeated state, so an intercepted ally is not a legal revival target. Once revived, the heroine is living and can receive subsequent Mobile Infirmary pulses. Encounter construction must initialize Betty's `skill.betty.combat_revival` use counter to one when the SSS skill is unlocked; spending the action decrements that authoritative counter before the target changes state.

## Reaction ordering

Damage resolution opens a reaction window only when an unredirected hostile attack would defeat a party member. Ordinary Rescue Charge interception changes the target before this check, so a redirected hit aimed at Betty cannot also trigger Fatal Intercept. An eligible Betty must be alive, not Stunned and have one `skill.betty.fatal_intercept` use remaining. The reaction spends that use, cancels the incoming hit completely, preserves the ally's current Vitality and counters the triggering attacker for 24 raw damage through normal Guard. The counter runs with reactions disabled. This prohibits nested reaction loops. After the reaction finishes, normal turn-end and victory evaluation resume.

## Midnight return transaction

Midnight is one atomic simulation transaction. The clock advances to the next day, named people become alive again without losing their death counters, then each region emits its deterministic daily monster instances in visible flashes. The same world seed, new day, region, and slot always produce the same instance. Each spawned monster receives an individual physical variant, condition, purpose, level, and loot seed. Prototype spawn rules enforce a group size of one because ordinary wilderness encounters are meant to read as D&D-like individual power relationships rather than anonymous packs of one-hit enemies.

## Runtime content bundle

Authoring remains split into local records under `content/` so ownership and relationships stay readable. `npm run build:content` validates those records, sorts them by stable ID, and writes one deterministic `game/generated/content_bundle.json` for Godot. The bundle includes every source path and a SHA-256 hash of its canonical record array. Godot loads the bundle; it does not crawl authoring directories or invent defaults. The generated file is committed so a build always identifies the exact validated content it consumed.
