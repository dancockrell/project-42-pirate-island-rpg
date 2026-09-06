# Placeholder art replacement report

Open placeholders: 13

## art.placeholder.captain.card_portrait

- Consumer: `game/scripts/battle/battle_prototype.gd#PartyRailCaptain`
- Visible mark: **DUMMY CAPTAIN CARD**
- Intended final: Complete Captain rail portrait: lean adult inventor-captain of the Handsome Jack, with a clear 1850s American frontier read, weathered slouch hat, worn dark sea coat, compact boiler pack, steam carbine, experimental sidearm, cutlass and brass echo compass.
- Required tags: adult, male-protagonist, 1850s-frontier, handsome-jack, steampunk-inventor, steam-carbine, card-portrait, echo-compass, no-crop
- Replacement gate:
  - approved Captain identity sheet
  - readable at rail scale
  - costume and compass match active actor when authored

## art.placeholder.captain.active_actor

- Consumer: `future-game/scenes/battle/protagonist_active_actor`
- Visible mark: **DUMMY CAPTAIN FULL-BODY ACTOR**
- Intended final: Complete Captain battle actor: lean 1850s American frontier inventor silhouette with slouch hat, sea coat, compact boiler pack, Handsome Jack steam carbine, experimental sidearm, cutlass, echo compass, stable root pivot and generous VFX safe margin.
- Required tags: adult, male-protagonist, full-body, 1850s-frontier, handsome-jack, steampunk-inventor, steam-carbine, echo-weapon, no-crop
- Replacement gate:
  - approved Captain identity rig
  - complete boots, head and weapon
  - battle safe-frame test

## art.placeholder.betty.active_actor

- Consumer: `game/scenes/battle/battle_prototype.tscn#ActiveActor`
- Visible mark: **DUMMY FULL-BODY ACTOR**
- Intended final: Complete full-body premium adult anime-fantasy Betty 3D battle actor. She reads cute and sexy before she reads realistic: slender adult build, small chest, bright green eyes, freckles, auburn curls, fitted dark tobacco leather bodice over warm ivory lace, dark leather short field shorts, tall brown boots, one restrained green-and-navy tartan hip sash, brown physician satchel and a bronze-and-glass boarding mace. It has a complete head-to-boot mesh, named skeleton, separate mace at a named hand socket, literal ready/step/Guarded Strike clips, an anchored root pivot and a fifteen-percent VFX safe margin.
- Required tags: adult, full-body, cute-sexy-adult-anime, slender-small-bust, signature-weapon-visible, 3d-rigged-glb, named-skeleton, hand-weapon-socket, no-crop
- Replacement gate:
  - approved 3D identity plate
  - rigged GLB with inspectable skeleton
  - separate complete mace at hand socket
  - ready-step-contact-recovery clip set
  - 1920x1080 battle-scale test
  - colorblind silhouette review

## art.placeholder.betty.card_portrait

- Consumer: `game/scripts/battle/battle_prototype.gd#make_party_card`
- Visible mark: **DUMMY PORTRAIT**
- Intended final: Betty card portrait with direct green-eyed eye contact, freckles, auburn curls, a teasing adult smile and dark tobacco leather, ivory lace and tartan identity accents. It must stay cute and sexy at card scale while preserving clear injury and readiness overlays.
- Required tags: adult, portrait, cute-sexy-adult-anime, flirtatious-confidence, card-safe-crop
- Replacement gate:
  - matches active actor face
  - readable at 280x150
  - expression set approved

## art.placeholder.razorbeak.active_actor

- Consumer: `game/scenes/battle/battle_prototype.tscn#EnemyActor`
- Visible mark: **DUMMY ENEMY ACTOR**
- Intended final: One complete individually scarred razorbeak raptor with full tail, feet, threat tell and VFX envelope.
- Required tags: full-body, individual-threat, wild-dinosaur, no-crop
- Replacement gate:
  - complete tail
  - level-seven threat read
  - three attack tells
  - no enclosure imagery

## art.placeholder.reception_road.stage

- Consumer: `game/scripts/battle/battle_prototype.gd#PaperStage`
- Visible mark: **DUMMY PAPER STAGE**
- Intended final: Layered 1920x1080 Reception Road battle environment dominated by wild jungle and magical Bronze Age elven funerary masonry.
- Required tags: wild-jungle, magical-bronze-age, battle-stage, layered-depth, no-colonial-city
- Replacement gate:
  - party and hostile standing zones remain readable
  - foreground occluders are separate
  - battle and exploration geography agree
  - no enclosure imagery

## art.placeholder.undead.tomb_wight.active_actor

- Consumer: `future-game/scenes/battle/undead.tomb_wight_active_actor`
- Visible mark: **DUMMY TOMB WIGHT ACTOR**
- Intended final: Complete Tomb Wight battle actor per art.plate.undead.tomb_wight's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.undead.drowned_bearer.active_actor

- Consumer: `future-game/scenes/battle/undead.drowned_bearer_active_actor`
- Visible mark: **DUMMY DROWNED BEARER ACTOR**
- Intended final: Complete Drowned Bearer battle actor per art.plate.undead.drowned_bearer's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.spectral.lament.active_actor

- Consumer: `future-game/scenes/battle/spectral.lament_active_actor`
- Visible mark: **DUMMY LAMENT ACTOR**
- Intended final: Complete Lament battle actor per art.plate.spectral.lament's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.eldritch.tide_spawn.active_actor

- Consumer: `future-game/scenes/battle/eldritch.tide_spawn_active_actor`
- Visible mark: **DUMMY TIDE SPAWN ACTOR**
- Intended final: Complete Tide Spawn battle actor per art.plate.eldritch.tide_spawn's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.hunter.bounty_tracker.active_actor

- Consumer: `future-game/scenes/battle/hunter.bounty_tracker_active_actor`
- Visible mark: **DUMMY BOUNTY TRACKER ACTOR**
- Intended final: Complete Bounty Tracker battle actor per art.plate.hunter.bounty_tracker's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.hunter.revenant.active_actor

- Consumer: `future-game/scenes/battle/hunter.revenant_active_actor`
- Visible mark: **DUMMY REVENANT ACTOR**
- Intended final: Complete Revenant battle actor per art.plate.hunter.revenant's production plate.
- Required tags: full-body, individual-threat, no-crop
- Replacement gate:
  - approved identity plate
  - readable at battle scale
  - attack tell matches its declared skill

## art.placeholder.boar.thunderback.active_actor

- Consumer: `future-game/scenes/battle/boar.thunderback_active_actor`
- Visible mark: **DUMMY THUNDERBACK ACTOR**
- Intended final: Complete Thunderback battle actor: one individual heavy island boar, the elevated holder of the river landing's jungle edge, built to cross open ground in a single committed charge, with a full snout-to-tail silhouette, all four feet and a readable charge lane.
- Required tags: full-body, individual-threat, charger, no-crop
- Replacement gate:
  - complete snout, tail and all four feet
  - individual-threat read rather than a herd animal
  - charge tell readable at battle scale

