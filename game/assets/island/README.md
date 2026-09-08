# Island development scene

Run res://scenes/world/island.tscn in the existing game project.
The scene installs four preview factions at startup: colonials, pirates,
Cthulhu and the fox people. They spend real resources on authored production.
The original three have six-unit preview caps; the fox people have eight lighter
spear skirmishers. Units rally toward a contested clearing, then select hostile
holdings. The four factions are mutually hostile; Michael begins neutral.
These are provisional scenario budgets, not final asymmetric faction balance.
Shift-click a person to inspect their name, faction and short biography;
Shift-click empty ground closes the panel without issuing movement. Selection
tracks the actor ID rather than the displayed name, which need not be unique.
personas.json supplies provisional fictional name/history pools for four
troop definitions, including male/female pirates and adult female fox skirmishers.
Rust embeds it: rebuild the extension
after changing it. NamedPerson is stored on the existing produced actor and
moves with that actor into its casualty record; names and history are not
rerolled on save/load. Older actors without identity data remain unknown.
No female actor is inferred from a male sprite. The pirate woman uses the
reviewed revision-01 standing source; female colonial and cultist production
remain disabled while their artwork needs correction. Biography text is not
yet a procedural quest system.

## First recruitment encounter

The pirate female pool authors an offer to leave over withheld shares. This is
one deliberately simple encounter condition, not universal automatic consent or
a completed romance progression. Inspect her, bring living Michael within two
clear navigation cells, and use Talk. Join faction appears after that discussion;
acceptance rechecks access and life state. Health is not a recruitment threshold.
The same actor joins Michael, preserving identity, provenance, current health and
appearance. Source production queues and their costs remain unchanged; only her
own population use transfers. Repeated acceptance must not duplicate her.
Immigrant accommodation provisionally raises Michael's capacity to cover actual
members; housing and wages are not modeled by this initial encounter.

Joining the faction leaves all four companion slots unchanged. Choose an explicit
slot to add/replace a living recruit; clearing or replacing a slot retains the
woman's faction membership and stops her old party order. Land clicks submit
Michael and living active companions through existing pathfinding toward nearby
distinct destinations, never teleporting. Invalid group requests retain previous
orders. Dead slots retain the person's identity and do not issue movement.
Membership, permanent attachment, discussion and slots persist through saves.
Midnight undead conversion now retains that slot as inactive under Cthulhu;
explicit reacquisition restores allegiance, not bodily life.

## Pirate waterfront

New campaigns place the pirate tide quay at cell [34,22], entrance [1104,720]
on the southeast cove's western edge. The existing shared tide-quay-01 cutout
is copied unchanged as tide_quay.png: SHA256
f9bc0ea135cc5a03959c7478a51103b3e6901830cec482e830088536b14be2d7.
buildings.json owns its 150-pixel display width, [360,805] stair pivot,
provisional collision offsets and exact coastal entrance. The same manifest
feeds native placement/collision and faction-specific Godot presentation.
The landing projects southeast into the cove; roof overlap with background
palms is not a claim that their ground footprints overlap.

Five narrow centerline corrections repair the overly conservative land mask:
[30,17] and [30,18] reconnect the otherwise isolated southeast region, then
[33,23], [34,23] and [34,22] approach the dock from the south. These are not
permission to walk throughout the inlet. [33,22] remains excluded because of a
palm near its center. The originally considered northern beach approach was
rejected because its supposedly connected starting cell was itself isolated.
The southern approach stays open in the provisional footprint. Grounding,
canopy occlusion, foot-radius clearance and the coarse
collision approximation still need actual rendered review; this is development
integration, not final art acceptance or a general procedural shoreline solver.

Old saves matching the exact previous mask receive only those five land
cells. Their actors and holdings do not move. A legacy inland pirate holding
does not receive the coastal sprite or an invented quay footprint; future map
or settlement migration is a separate decision. Unrelated map changes remain
rejected. Raw source, alpha correction and extraction evidence remain shared.

## Character presentation boundary

troops/appearances.json owns existing texture, pivot and scale by unit definition
and person sex, never current faction. Thus an allegiance change and sprite
recreation preserve costume. Explicit legacy-unknown mappings retain the three
old male appearances without inventing missing identity fields. Unsupported
definitions/sex variants have no arbitrary colonial fallback and the HUD reports
missing art. The pirate female texture is the exact shared revision cutout,
SHA256 52326f77901ee1d4643359d32fea47e25915bfd9146fd3e667b8e66b0c5afc19.
Her 0.03025 scale is approximately 36 image pixels high; [302,1080] is a
provisional support-point estimate between the boot contacts. It needs a common
ground-plane render comparison, not a claim of precisely calibrated stature.
Fine dithering, lack of directional walking/attacking poses, and actual native
visual review remain open. This development standing integration is not final
shipped-art or animation acceptance. Source, prompt, alpha correction and review
remain in shared troops-01/pirate-female-revision-01; no raw source was removed.
Right-click a nearby unit to queue one steam-carbine shot. A hit provokes that
faction into retaliation. Range and line of sight are native checks; reload and
pause delay firing. A successful movement order cancels a queued shot, while an
invalid destination preserves it. No automatic pursuit or Echo skills yet.
Provisional Michael profile: 30 health, 4 damage, 4-cell range, 5-tick reload.
His health appears in the HUD. Death pauses with a fallen message, leaves a
native casualty record, and can be saved/reloaded. Dead Michael cannot move or
fire. Michael's own defeat remains terminal for this run; ordinary casualties
follow the midnight rule below.
On-island skirmishes now use distinct provisional health, damage, range and
cooldown profiles. Attacks resolve simultaneously and cannot shoot through
nonwalkable cells. Autonomous units hold whenever a living hostile is in range
and has line of sight, including during reload. All hold decisions use the same
pre-movement snapshot. Ranged troops therefore engage before melee troops stop;
the strategic route remains queued and resumes when the firing target is lost.
Movement and attacks share target legality. This is not yet pursuit, retreat,
formation spacing or final tactical AI. Player-directed factions without an
autonomous policy are not forced to hold by this behavior.
Each actual strike now emits immutable hit positions and attacker definition
through the native bridge. The scene draws a brief two-pixel effect from attacker
to target: warm shot, pale melee strike, or muted teal cultist effect. These last
one simulation tick and freeze with pause. No random cosmetic attacks are emitted.
Effects survive lethal target removal because they use event positions, not
lookups of surviving sprites. Successful campaign loading clears old effects.
Chest-height offsets are provisional; authored weapon sockets and animated
attack/recoil poses are still missing. Native render review remains outstanding.
Dead units leave saved casualty records and release their
population slots. Bodily restoration machinery, diplomacy changes, combat
animation and final faction balance remain unfinished.
Producers now have provisional 80-point health persisted in saves. Autonomous
units without an in-range troop target can strike a hostile producer's entrance
when in range and line of sight.
Units hold their position while a legal siege target exists, including reload;
once it falls they resume their retained strategic route. Movement and attacks
share one native siege-target rule rather than separate distance checks.
A destroyed building loses its queued resources, releases queued population and
removes its obstacle/policy entry. Losing the last
producer invokes persistent faction elimination. Rallied troops now choose
reachable hostile holdings, weighing route length, defenders and damage.
Finite-supply campaign tests reach and destroy a holding without teleportation;
unlimited replacement income can still sustain a stalemate. Building attack
orders for Michael, final durability and destruction art remain unfinished.
The colonial watch fort now has a provisional transparent sprite projected from
its native building position. Operating state dims inactive forts, and removed
buildings disappear on the next snapshot. Other producer archetypes remain
without art: they must not masquerade as colonial forts. The fort's five-cell
ground obstruction now participates in native movement and shot visibility;
the entrance remains open. Elimination removes its obstruction, while inactive
buildings continue blocking. No construction/damage animation yet.
The coarse ground footprint still needs rendered overlay review against the art.

buildings.json owns the sprite path, pivot, scale and blocked cell offsets.
Rust embeds this contract, so changes require rebuilding the native extension.
New saves preserve per-building obstacles. Historical saves without that field
reconstruct them through the same building contract used by new scenarios.
If an old actor position is now inside a wall, loading is rejected without
teleporting or overwriting the current campaign. Such a save still requires an
explicit recovery policy before release; no silent unit movement is performed.

watch_fort.png is the exact shared watch-fort-01/extracted/cell_00_00.png.
SHA256: 34c8f250350f73f4cb6ae78c82a7bf9aca4266040a221d60e40e9f2527494c56.
Source-size silhouette and alpha were inspected; full game-scale visual review
remains pending. Ground pivot (775,825) in the trimmed 1125x928 image marks
the entrance apron; displayed width is provisionally 150 map pixels.

troops/ contains exact copies of the shared troops-01/extracted male idle PNGs.
The existing Cattle Trail sprite_grid.py extractor produced six transparent
cutouts without rescaling, filtering components, or gutter warnings. All six
remain shared; only consumed appearances ship here. Source prompts,
original images, extraction parameters, bounds and hashes remain in the shared
library. Runtime uses binary-alpha PNGs directly, without a chroma shader.
The three male cutouts were inspected at source size: weapons and silhouettes
are intact. Rendered game-scale edge review is still required.
Three male standing appearances, the pirate woman and the fox skirmisher are
selected from explicit NPC identity. Directional poses and animation remain
unfinished. Do not infer recruitment eligibility from artwork.
Click land to travel; Space pauses; F5 saves and F9 loads.
No paid service is required to run it.

Save files use Godot's user data directory, pirate-island-save.json, plus a
last-good .bak. Rust serializes the complete current FactionWorld: tick, pause,
factions/resources/buildings/production queues, actors/provenance, navigation,
positions, travel orders and ID counters. Version 1 loads validate into a new
world before replacing live state. The loaded land must match this map.
Writing first flushes a .tmp file; corrupt primaries do not replace good backups.
F9 tries the backup if the primary is corrupt. This is manual persistence, not
autosave or a shipped migration guarantee; future mechanics must join this state.

Terrain source authority:
shared-game-environment-library/procedural/sprites/candidates/pirate-island/island-terrain-01/GENERATION.md.
Runtime terrain.png is an exact source copy, SHA256
5b5af8b60df58e402ec69f9d6788ee788c60238ebc68fd38d60654e7583cf976.

navigation.json is the authored physical navigation mask for this image.
Godot reads the asset contract once and submits its walkable cells to Rust
at initialization. Rust owns routes, movement, positions, travel orders and pause.
The API rejects map changes after ticks/orders begin. This is not room-node travel.

The island is now the configured game entry scene: launching the project starts
this working integration preview, not the historical battle fixture or a finished
game. Michael begins alone. Four companions
must be recruited rather than silently granted. Current pose frames are standing
art; movement is tick-stepped, not an approved walk animation. Foliage occlusion,
pixel-density matching, terrain-mask visual review, complete autonomous factions
and final island scale remain unfinished.

The drowned shrine now uses its own dark sea-stone/tentacled-entrance sprite,
shared candidate drowned-shrine-01 (SHA256 804377de550667198f2b995d9c729d9991d4460738bc0596760fe31f2001675b).
It is displayed at 110px width with source pivot [690,800], the lower stair
approach. The entrance is cell [27,16], on clear inland ground above the track, not the old
simulation position inside the southern palm grove. No new walkable land was
added. The three blocked foundation cells are separate from the image bounds.
This is developmental placement; western palm-crown overlap and final foreground
sorting still need a rendered review. Older saves retain their former holding
positions and do not receive a falsely placed shrine sprite. The shared
placement_entrance contract now handles both shrine and quay.

Live holding development uses content/island/holding_development.json. Fully
staffed, undamaged holdings reserve costs atomically and stop unit production
while improving the same building. Costs multiply by the current level; levels
stop at five. Colonial forts gain 40 maximum/current HP and two population slots
per level in 100 ticks; pirate quays gain 16 HP and three slots in 80 ticks;
drowned shrines gain 24 HP and two slots in 120 ticks. All numbers are provisional
scenario tuning, not final faction balance. Damage taken during work is retained
on completion. Destroyed holdings lose their work; elimination never rebuilds it.
Pause and saves retain remaining work and reserved costs. Older saves default to
level one and keep their saved policy (no retroactive paid upgrade).
Nearby developing buildings show a small progress bar. Level-specific building
art, repair, unlocked vendor prerequisites, loot scaling and construction of new
sites remain unfinished; the progress bar is not approved construction art.

After recruitment, adult pirate women retain Talk and Approach. Their current
speech acknowledges joining Michael instead of repeating the recruitment offer.
Three authored companion responses are paired with the three existing histories
using the same seed selection; the selected response is saved on the person.
Older saves without this field use a neutral acknowledgement, not an invented
past. Dismissing a party slot does not remove conversation or faction loyalty.
This is post-join dialogue, not a completed romance quest or relationship system.

Active companions defend a stationary Michael: idle members take ordinary path
steps toward visible hostile units within four Manhattan cells of him, then use
their existing weapon profile. Every step of that route must stay inside this
provisional radius. Already-in-range attacks use the unchanged combat resolver.
Explicit travel, Michael's movement, and Approach conversations take priority.
No neutral provocation, building assault, distant pursuit or off-party auto-chase
is granted by this behavior. Multiple defenders reserve distinct next cells.
This supplies basic companion assistance, not final tactical stances or hero powers.

## Midnight and remembered companions

The live campaign now has a saved clock. Initial tuning is1440simulation ticks
per day; this is provisional, not a promised real-time day duration. Normal pause
freezes it. The interface reads Day/HH:MM from the native world.

At midnight, eligible ordinary casualties from earlier ticks return as the same
actors under Cthulhu, provided that faction survives with an operational holding.
They rise at their death location or nearby unoccupied reachable ground; when no
legal space exists they remain casualties for a later midnight. Michael's death
still ends the playable run. An eliminated faction is never rebuilt by this rule.

Names, history, production provenance and Michael loyalty survive. Returned units
receive undead state; the current visual is a restrained cold tint on their
existing standing sprite, not finished undead artwork. Cthulhu accommodation
grows by the actual returned population; this is supernatural conversion rather
than a free ordinary production queue. No new duplicate identity is created.

A returned companion keeps her remembered slot but is inactive while with
Cthulhu. Party travel and defense cannot command her. Michael must approach and
regain her explicitly through Talk/Join. She then accompanies him again if her
slot was retained, but remains undead: recruitment does not invent bodily
restoration. Faction/allied restoration machinery, rituals, madness recruitment,
undead-specific powers and final resurrection presentation remain unfinished.

## Fox settlement

The river market occupies northeast clearing cell[33,9]. Its140px building sprite
uses source pivot[630,1020]; five foundation cells block travel, while the lower
gate remains accessible. Its original checkerboard background was corrected and
the purple silk preserved during chroma extraction. Shared source:
`pirate-island/river-market-01`; PNG SHA256
`883e48fa10f6702bfd8258d95f68da20d0d2cc4c398da931386898b750ff1c36`.
Static terrain composition was inspected, not a final native-render approval.

The new spear skirmisher is explicitly a soldier, not a relabeled river porter.
She costs2rice/1silk and takes2ticks; starting capacity8, health8, damage2,
range2 and cooldown2. All four initial faction profiles now live in
`content/island/faction_roster.json`. Market development uses the existing
building system: rice4/silk4 times current level,100ticks, health+20,
population+2, maximum level5. Values remain provisional.

Fox women have seeded adult identities and brief dialogue through the same
Talk/Join/party system as pirates. The standing sprite is36px body height,
with a visible jade wrap, fox tail and spear; lower-body facing and spear/shin
separation remain art limitations. Fox male-wooing, special retreat tactics,
full worker/trade economy and hero powers are not implemented by this increment.
Older campaigns preserve their original faction population; start a new game
to include the fox settlement. The elven grove is a shared source candidate,
not a deployed fifth AI faction: its guardian art remains unavailable.

Checks:
Godot headless tests/island_scene_test.gd passes actual scene load, native position
projection, terrain rejection, travel and pause. The separate native bridge test
and Rust suite are tracked in the current task claims. Headless success does not prove rendered visual quality.
