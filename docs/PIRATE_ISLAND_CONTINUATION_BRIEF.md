# Pirate Island

## Claude Continuation Brief: Autonomous RTS World and Faction Design

**Status:** Current design authority, with provisional material explicitly labeled  
**Date:** 5 September 2026  
**Implementation status:** Design only. Do not claim that these systems are implemented unless a game repository is located and verified.  

---

# Instructions for Claude

Use this document as the current authority for continuing Pirate Island design.

Preserve explicit accepted decisions. When expanding the design:

- Mark new recommendations as **Provisional** until the user approves them.
- Do not silently combine rejected ideas with the current direction.
- Do not invent proper names for the factions.
- Use **Captain Michael** as the protagonist's current name. Earlier references to Captain Jack are superseded.
- Treat the game as an autonomous RTS experienced from inside by five controllable RPG characters.
- Do not turn the island into a three-lane map.
- Do not turn Captain Michael into an overhead RTS commander who selects and micromanages individual units.
- Do not reduce Captain Michael's faction to a seduction or recruitment gimmick.
- Do not reduce its steampunk identity to Captain Michael's personal equipment or mount.
- Do not manufacture human population from player-faction buildings.
- Preserve the four women's agency as investigators, leaders, companions, and quest drivers.
- Preserve normal pause and a calm, friendly player-facing interface.
- Preserve genuine strategic loss and multiple organic victory paths.
- Keep design claims separate from verified implementation.

When a detail is not established, identify the open question rather than presenting an invention as settled fact.

---

# 1. Core game promise

## Accepted

Pirate Island is a character-scale RPG taking place inside a living autonomous RTS simulation.

The player controls five heroes:

- Captain Michael.
- Four women.

The four women are active investigators, quest drivers, leaders, and characters with their own theories, requests, priorities, relationships, and personal quests. They are not passive followers or prizes waiting for Captain Michael to advance the story.

Behind the scenes, the island's factions behave like RTS factions. They:

- Gather and consume resources.
- Select strategic goals.
- Construct and upgrade buildings.
- Spawn or recruit workers, defenders, vendors, specialists, monsters, and military forces according to faction rules.
- Assign those actors to jobs and destinations.
- Move along the island network.
- Protect supply.
- Trade.
- Raid.
- Defend territory.
- Attack rivals.
- Form independent relationships.
- Grow, shrink, recover, or collapse.
- Pursue survival or victory according to their position on the board.
- Permanently eliminate competing factions when possible.

The simulation exists to make the island dynamic rather than static. It is not intended to menace the player with constant timers, alerts, or chores.

The player experiences strategic change through character-scale consequences:

- A safe road becomes contested.
- A bridge changes which settlements can be supplied.
- A market gains or loses vendors.
- A settlement upgrades and fields better defenders.
- A defeated faction abandons territory.
- Another faction occupies or transforms that territory.
- A dungeon changes when its owner, building tier, magical state, or local conditions change.
- An allied faction becomes more or less capable of supporting the party.
- Refugees, patrols, convoys, traders, monsters, and rumors move through the world.
- Weather and terrain become more corrupt as the Cthulhu faction grows.
- Companion observations turn simulation changes into investigations and stories.

The strategic world creates circumstances. Characters turn those circumstances into stories.

---

# 2. Player and simulation ownership

## Accepted

The player influences the board at two scales.

## The five-person party

The party is the player's precise instrument. It can:

- Investigate.
- Enter dungeons.
- Negotiate.
- Rescue individuals.
- Recruit important women.
- Sabotage buildings.
- Steal or recover critical objects.
- Defeat specific dangerous enemies.
- Discover hidden information.
- Interfere with rituals.
- Open or close routes.
- Protect a convoy or settlement at a decisive moment.
- Create opportunities that no autonomous faction could create by itself.

## Captain Michael's faction

The faction is the player's island-scale instrument. It can:

- Build and operate settlements.
- Hold territory.
- Maintain routes.
- Produce machinery.
- Move supplies.
- Field automata and recruited female forces.
- Develop resources.
- Protect populations.
- Support allies.
- Pressure or attack enemies.
- Exploit opportunities created by the party.
- Become strong enough to change which factions can survive or win.

The faction should play mostly automatically but be easy to direct.

The player establishes strategic intent. The faction AI handles routine execution.

The party changes what is possible. The faction makes that change durable.

---

# 3. The island is a complex graph

## Accepted

The island is not a three-lane tower-defense map.

It is a complex network containing:

- Branching routes.
- Loops.
- Alternate approaches.
- Roads.
- Trails.
- Rivers.
- Bridges.
- Ports.
- Coastal routes.
- Sea routes.
- Caves.
- Tunnels.
- Interior connections.
- Magical connections.
- Weather-sensitive routes.
- Faction-created routes.
- Chokepoints that emerge from geography rather than prescribed lanes.

Temporary fronts can form around bridges, ports, resources, strongholds, settlements, and ritual sites. Those fronts can move, split, or disappear.

## One node equals one playable room

A strategic graph node is one playable room or immediately playable local scene.

A town, fortress, dungeon, or settlement may consist of many connected nodes, such as:

- Gate room.
- Market room.
- Dock room.
- Residential room.
- Courtyard room.
- Workshop room.
- Keep entrance.
- Interior hall.
- Cellar.
- Tunnel.
- Shrine.
- Dungeon chamber.

Each room requires:

- A gameplay function.
- Defined entrances and exits.
- Camera composition.
- Dimensions.
- Walkable circulation.
- Building or interaction slots.
- Landmarks.
- Spawn points.
- Encounter space.
- Material language.
- Explicit elements to avoid.

Do not replace this room-level contract with vague district-scale prompts.

---

# 4. Factions

## Accepted faction concepts

The island contains five established autonomous factions plus the faction Captain Michael creates.

The established factions are:

- Asian fox people.
- Colonial powers.
- Pirates.
- Elves.
- Cthulhu faction.

The sixth faction is:

- Captain Michael's advanced 1870s steampunk faction.

Do not invent proper faction names without explicit approval.

Every faction needs a complete RTS design covering:

- Survival requirements.
- Economy.
- Building priorities.
- Population or recruitment source.
- Unit and actor production.
- Mobility.
- Preferred terrain.
- Strategic utility weights.
- Relationships.
- Recovery behavior.
- Expansion behavior.
- Terrain effects.
- Weather effects.
- Dungeon grammar.
- Loot grammar.
- Escalation.
- Victory behavior.
- Elimination consequences.

---

# 5. Captain Michael's faction

## 5.1 Founding situation

### Accepted

Captain Michael begins as a shipwreck survivor.

He does not arrive with:

- A nation.
- An inherited settlement.
- An army.
- A civilian population.
- A workforce supplied by an existing government.
- A functioning industrial base.

He begins with survival, knowledge, salvage, character-scale abilities, and the potential to construct a new power.

His faction must be created during play.

The personal base question is therefore resolved: the first base becomes the first strategic core of Captain Michael's new faction.

The founding method, exact starting location, first construction sequence, and how the four principal women join remain open.

## 5.2 Faction fantasy

### Accepted

Captain Michael's faction is a spectacular alternate-1870s steampunk frontier power.

It has substantially more technology than any other faction.

Its identity combines:

- Steam power.
- Mechanical automation.
- Frontier practicality.
- Industrial production.
- Mobile logistics.
- Experimental engineering.
- Armed independence.
- Captain Michael's experience with Argentines and in Texas.
- Female recruits drawn from the island's existing factions and communities.
- Strong but easy player direction.

Captain Michael's ideas are revolutionary, useful, and dangerous.

The faction is intended to become influential and strong. It is not a weak novelty settlement that survives only through constant player babysitting.

## 5.3 Historical and technological boundary

### Accepted

The visual and conceptual reference point is the 1870s made dramatically more technologically capable.

The faction's engineers understand the world through systems known or imaginable in that period:

- Steam engines.
- Boilers.
- Pressure regulation.
- Pumps.
- Mechanical governors.
- Gears, cams, escapements, and clockwork sequencing.
- Belt, chain, cable, and rod transmission.
- Pneumatics and hydraulics.
- Telegraphy.
- Batteries and galvanic effects.
- Electrical ignition and signaling.
- Gas production, storage, and illumination.
- Lighter-than-air lift.
- Rockets.
- Railroad engineering.
- Maritime engineering.
- Artillery.
- Precision machining.
- Standardized mechanical components.
- Animal locomotion.

Steam remains the principal industrial and motive language.

Electricity and gas are known but are usually used in service of other systems rather than replacing the faction's steam and mechanical identity.

Electricity can support:

- Telegraphy.
- Signals.
- Ignition.
- Detonation.
- Measurement.
- Alarms.
- Limited lighting.
- Experimental controls.

Gas can support:

- Lighting.
- Chemical work.
- Heating.
- Lighter-than-air lift.
- Specialized fuel experiments.

Do not turn the faction into the twentieth century.

Avoid:

- Modern cars with brass decorations.
- Diesel machinery.
- Modern tanks.
- Modern robots.
- Clean science-fiction mechs.
- Decorative gears with no purpose.
- Generic Victorian-London fashion as the entire visual identity.
- Modern electronics.
- Sleek retrofuturism.

Favor:

- Riveted iron and steel.
- Cast housings.
- Structural timber.
- Brass valves and gauges.
- Leather seals, harnesses, and padding.
- Canvas.
- Boiler tanks.
- Exposed connecting rods.
- Repair plates.
- Smoke, dust, oil, salt, and hard use.
- Bold silhouettes readable from the fixed isometric camera.
- Machinery that looks heroic because it performs understandable work.

## 5.4 Animal-form machinery

### Accepted direction

Captain Michael's engineers model mobile machines after animals that people of the 1870s understand.

This is faction design, not a requirement that Captain Michael personally receives a particular mount.

Animal forms provide known answers for:

- Gait.
- Weight distribution.
- Terrain use.
- Tactical role.
- Formation.
- Handling.
- Cultural understanding.

Clockwork mechanisms provide sequencing and control. Steam, stored mechanical tension, compressed air, or pressure systems provide power.

The faction's principal candidate machine families are:

### Mechanical dogs

Potential functions:

- Patrol.
- Warning.
- Tracking.
- Pursuit.
- Messaging.
- Escort.
- Confined-route reconnaissance.
- Light carrying.
- Harassment.

### Mechanical cavalry

Potential functions:

- Scouting.
- Rapid response.
- Communication.
- Pursuit.
- Escort.
- Flanking.
- Control of roads and broad trails.

This is a faction-level troop family, not a mandatory personal mount system.

### Mechanical bears

Potential functions:

- Compact heavy labor.
- Close defense.
- Breaching.
- Protecting engineers.
- Moving damaged equipment.
- Fighting large monsters.
- Anchoring narrow routes.

### Mechanical elephants

Potential functions:

- Heavy transport.
- Mobile crane work.
- Artillery carriage.
- Siege.
- Command and observation.
- Bridge construction.
- Walker recovery.
- Mobile repair and supply.

### Walkers

Walkers are used when an articulated industrial or battlefield machine solves a problem better than an animal-form automaton or wagon.

Potential functions:

- Construction.
- Excavation.
- Logging.
- Fortification.
- Siege.
- Heavy weapons.
- Large-monster combat.
- Operations across selected broken terrain.

Do not add walkers merely because steampunk factions are expected to have mechs. Each walker needs a functional reason.

### Steam wagons

Steam wagons are more likely than recognizable cars.

They provide:

- Freight.
- Troop transport.
- Artillery towing.
- Mobile workshops.
- Medical evacuation.
- Protected convoy movement.
- Fuel and water transport.
- Settlement construction support.
- Dungeon-salvage recovery.

### Rockets

Rockets can support:

- Signaling.
- Illumination.
- Line carrying.
- Communication.
- Fire.
- Area denial.
- Defensive barrage.
- Siege bombardment.
- Airship operations.

They remain unguided, smoky, weather-sensitive, dangerous to store, and capable of accidents.

### Steam airships

Airships can support:

- Reconnaissance.
- Communication.
- Specialist transport.
- Valuable light cargo.
- Emergency supply.
- Evacuation.
- Observation.
- Access to selected isolated areas.
- Limited strategic military support.

Lifting gas provides lift. Steam powers propulsion, pumps, winches, and support machinery.

Airships require moorings, fuel, water, lift gas, crews, weather knowledge, and repair facilities. Magical weather prevents them from making geography irrelevant.

## 5.5 Faction pillars

### Accepted

Captain Michael's faction has several mutually supporting sources of power.

It must not be reduced to any one of them.

### Industrial power

It manufactures the island's most advanced machinery, weapons, transport, construction equipment, communications, and defensive systems.

### Automated military strength

It can produce machines and automata without receiving a new human recruit for every additional unit.

### Captain Michael's pull

Captain Michael has an unusual ability to attract women. This gives the faction access to romantic, diplomatic, political, intelligence, and recruitment opportunities that would otherwise be unavailable.

This is a healthy and fun part of the faction, not the entire faction kit.

### Female leadership

Women who join can become companions, officers, engineers, diplomats, commanders, navigators, spies, magical specialists, governors, and recruiters.

### Player direction

The faction receives strategic intent from the player and converts it into autonomous action.

### Logistics and territory

The faction must still secure resources, construct buildings, maintain routes, protect infrastructure, and survive attacks.

Its conceptual strength derives from:

> industrial capacity + machine force + recruited talent + leadership + logistics + relationships + player direction

Captain Michael's attraction principally affects:

> recruitment opportunities + diplomatic access + personal loyalty + political instability inside rival factions

It does not replace strategy, construction, fuel, supply, territory, combat, magic, or companion investigation.

## 5.6 Population and recruitment

### Accepted current direction

Captain Michael begins without a native faction population.

The people who join his faction are women drawn from the island's existing factions, communities, and independent population.

Male characters may trade, cooperate, negotiate, provide temporary services, fight as allies, or remain external partners. They do not become ordinary recruits within Captain Michael's faction under the current direction.

The exact boundary between military membership, civilian residence, alliance, contracting, and citizenship remains open and should be defined later without weakening the female-faction identity.

### Buildings do not manufacture women

Player-faction buildings can create:

- Automata.
- Vehicles.
- Weapons.
- Equipment.
- Industrial resources.
- Housing capacity.
- Training capacity.
- Medical capacity.
- Command capacity.
- Recruitment reach.
- Social stability.
- Integration capacity.

They cannot create:

- Soldiers.
- Engineers.
- Officers.
- Navigators.
- Scouts.
- Vendors.
- Doctors.
- Companions.
- Settlers.

A building may support one of those roles, but a real woman must fill it.

### Why women may join

Captain Michael's pull creates opportunities, but women retain distinct motives.

A woman may join because of:

- Attraction.
- Love.
- Sexual interest.
- Admiration.
- Trust.
- A relationship with one of the companions.
- Belief in Captain Michael's political project.
- Access to technology.
- Ambition.
- Rescue.
- Safety.
- Revenge.
- Disillusionment with her original faction.
- Desire for authority.
- Desire to protect others.
- Interest in the society existing recruits are building.
- The example of another woman who already joined.

Attraction creates openings. It does not erase character identity or automatically resolve allegiance.

### Recruitment scales

#### Principal women

Major women receive complete relationship, courtship, conflict, and recruitment arcs. Some may become controllable companions, lovers, major officers, or political leaders.

#### Named recruits

Important specialists and officers receive individual identity and consequential recruitment without necessarily becoming principal companions.

#### Group recruitment

A woman may bring a squad, crew, technical team, household, refugee group, or faction splinter.

#### Reputation recruitment

As the faction matures, women may arrive because of Captain Michael's reputation, the companions, the technology, the authority women receive, the settlements, or women who have already joined.

### Women may pursue Captain Michael

Courtship is not one-directional.

Women may:

- Seek him out.
- Flirt.
- Challenge him.
- Test him.
- Offer strategic gifts.
- Arrange meetings.
- Ask companions about him.
- Join the faction before pursuing romance.
- Attempt to recruit him to their cause.
- Make the first move.
- Demand proof that he is worth following.

This keeps the tone reciprocal, lively, and fun.

### Attraction without immediate defection

A woman attracted to Michael may remain in her faction and still:

- Share information.
- Delay an enemy action.
- Protect someone.
- Offer trade.
- Advocate for alliance.
- Warn him.
- Arrange a meeting.
- Become a long-term contact.
- Join later.
- Never formally join.

Michael's pull therefore contributes to diplomacy and intelligence as well as population.

### Recruitment must not dominate every scene

Faction encounters can primarily concern territory, trade, weather, magic, machines, dungeons, convoys, investigations, or war while attraction remains a secondary layer.

The romantic premise should permeate the game without replacing every character's immediate purpose.

## 5.7 Party and leadership

### Accepted

The four controllable women are the faction's founding leadership, not passive followers.

They should eventually own meaningful portfolios according to their characters, such as:

- Intelligence.
- Investigation.
- Recruitment.
- Diplomacy.
- Industry.
- Logistics.
- Military operations.
- Magic.
- Corruption containment.
- Medicine.
- Exploration.
- Relations with a specific faction.

Do not assign the final four portfolios until the women are defined.

Captain Michael's charisma begins the faction's social expansion. The companions help turn that charisma into a functioning institution.

## 5.8 Mixed-origin identity

### Derived

Women who join retain meaningful parts of their former identity:

- Skills.
- Language.
- Culture.
- Clothing elements.
- Weapons.
- Magic.
- Relationships.
- Grievances.
- Terrain knowledge.
- Professional knowledge.
- Loyalty conflicts.

Captain Michael's faction adds:

- Steampunk equipment.
- Mechanical support.
- Shared logistics.
- Common communication.
- New authority.
- A developing common visual identity.

The faction should visibly look like a new society created from the changing history of the island, not like a preexisting national army that arrived fully formed.

## 5.9 Automatic operation and player direction

### Accepted

The faction plays mostly automatically but is easy to direct.

The player controls:

- Standing policy.
- Strategic priorities.
- Desired relationships.
- Development targets.
- Protected locations.
- Expansion targets.
- Avoided locations.
- Acceptable risk.
- Major construction.
- Major attacks.
- Withdrawal.
- Reserve commitment.
- Recruitment of important named women.
- Major political promises.
- Unique technology projects.
- Final victory strategy.

The faction AI controls:

- Worker and machine assignments.
- Routine production.
- Convoy schedules.
- Patrol composition.
- Normal routing.
- Routine maintenance.
- Repair priorities within policy.
- Local defensive responses.
- Routine training.
- Housing and equipping new arrivals.
- Execution of approved construction.

The companions may autonomously:

- Identify promising recruits.
- Maintain contacts.
- Recruit ordinary volunteers.
- Organize approved extractions.
- Integrate new arrivals.
- Recommend named women to Michael.
- Manage portfolios delegated to them.

### Strategic directive vocabulary

A small intention-based vocabulary is preferred over unit micromanagement:

- Protect.
- Supply.
- Develop.
- Expand.
- Pressure.
- Attack.
- Support.
- Investigate.
- Avoid.
- Withdraw.

Before confirmation, the faction should explain how it understands a major directive in ordinary language, including obvious risks and competing commitments.

Directions should persist until completed, cancelled, superseded, made impossible, or returned for reconsideration.

## 5.10 Human scarcity and military doctrine

### Derived

Because women cannot be manufactured and important recruits carry relationships and history, the faction should value human life.

Its AI should often prefer:

- Machines taking initial exposure.
- Automata scouting dangerous routes.
- Steam transport evacuating the wounded.
- Mechanical heavy units shielding crews.
- Recovery of stranded personnel.
- Retreat from wasteful attrition.
- Rescue.
- Negotiated surrender.
- Fortification.
- Remote fire or machines before committing scarce women.
- Protection of specialists and officers.

This does not make the faction passive. It makes advanced machinery a force multiplier for a smaller, unusually valuable human population.

## 5.11 Terrain signature

### Derived

Captain Michael's developed territory may show:

- Graded roads.
- Reinforced bridges.
- Fuel and water stations.
- Telegraph or signal systems.
- Crane structures.
- Machine yards.
- Workshops.
- Fortified depots.
- Mechanical tracks.
- Standardized building plots.
- Airship moorings.
- Rocket positions.
- Salvage yards.
- Smoke and industrial activity.

Its growth can also produce conflict through:

- Deforestation.
- Fuel consumption.
- Water consumption.
- Fire risk.
- Noise.
- Industrial waste.
- Rapid territorial expansion.
- Increased visibility.
- Conflict with elven ecological goals.

---

# 6. Provisional doctrines for the established factions

The following designs are **Provisional**. They are concrete enough to test and revise but are not yet user-approved at the same level as Captain Michael's faction direction.

## 6.1 Asian fox people

### Proposed strategic identity

A distributed network faction built around communities, intelligence, spiritual relationships, mobility, reciprocity, concealment, and controlled misdirection.

Do not use a generic pan-Asian cultural collage. A coherent cultural source and art direction must be selected later. Strategic behavior should derive primarily from the fox people's fantasy nature and relationship to the island rather than stereotypes about real people.

### Proposed strengths

- Intelligence.
- Hidden routes.
- Flexible movement.
- Social networks.
- Ambush.
- Diplomacy.
- Distributed recovery.
- Spirits, wards, concealment, and localized weather effects.

### Proposed weaknesses

- Prolonged frontal siege.
- Heavy territorial occupation.
- Defense of several exposed sacred locations simultaneously.
- Network damage caused by lost trust or false information.

### Proposed building functions

- Household.
- Food production.
- Craft workshop.
- Market.
- Shrine.
- Archive or school.
- Scout post.
- Hidden store.
- Spirit garden.
- Healing site.
- Concealed route entrance.
- Defensive ward.
- Diplomatic gathering place.

### Proposed victory pressure

Create a resilient network of communities and spiritual anchors that cannot be conquered through simple territorial occupation.

## 6.2 Colonial powers

### Proposed strategic identity

A centralized administrative and military occupation faction built around ports, roads, forts, standardized production, taxation, artillery, and external supply.

### Proposed strengths

- Disciplined forces.
- Fortification.
- Artillery.
- Standardization.
- Ports.
- Major-road logistics.
- Administrative control.
- External reinforcement.

### Proposed weaknesses

- Rigid command.
- Expensive occupation.
- Long supply chains.
- Isolated garrisons.
- Dependence on ports and formal routes.
- Resistance by local populations.

### Proposed building functions

- Barracks.
- Fort.
- Customs house.
- Administrative office.
- Warehouse.
- Port facility.
- Artillery position.
- Prison.
- Hospital.
- Survey station.
- Telegraph station.
- Extraction facility.
- Formal market.
- Naval support.

### Proposed victory pressure

Convert the island into governable, taxable, militarily controlled territory connected to external authority.

## 6.3 Pirates

### Proposed strategic identity

A maritime, opportunistic, partially decentralized faction built around raiding, capture, salvage, smuggling, reputation, tribute, and control of movement.

### Proposed strengths

- Sea movement.
- Coastal attack.
- Rapid exploitation.
- Capture and reuse of equipment.
- Smuggling.
- Raiding supply.
- Flexible alliances.
- Movable wealth.

### Proposed weaknesses

- Long formal sieges.
- Deep interior occupation.
- Stable large-scale production.
- Cohesion after major leadership or wealth loss.
- Repair of specialized captured equipment.

### Proposed building functions

- Dock.
- Shipyard.
- Hidden cove.
- Storehouse.
- Tavern or meeting house.
- Black market.
- Salvage yard.
- Lookout.
- Coastal battery.
- Prison or ransom site.
- Smuggling tunnel.
- Raider barracks.
- Repair workshop.
- Treasure vault.

### Proposed victory pressure

Control enough ports, routes, movable wealth, and tribute that commerce occurs with pirate permission.

## 6.4 Elves

### Proposed strategic identity

An old territorial power whose economy, military, magic, and infrastructure are integrated into the island's living systems.

### Proposed strengths

- Terrain shaping.
- Defensive depth.
- High-quality specialists.
- Ranged combat.
- Magical support.
- Ecological recovery.
- Weather stabilization.
- Corruption resistance.

### Proposed weaknesses

- Slow population replacement.
- Strong attachment to specific irreplaceable places.
- Fire and industrial damage.
- Simultaneous attacks against several ancient anchors.
- Reduced effectiveness when drawn away from prepared living terrain.

### Proposed building functions

- Living residence.
- Grove.
- Healing site.
- Ranger post.
- Magical workshop.
- Archive.
- Shrine.
- Root or water nexus.
- Defensive growth.
- Creature habitat.
- Path-shaping structure.
- Weather-stabilizing site.
- Ritual hall.
- Magical stronghold.

### Proposed victory pressure

Restore or impose a stable ecological and magical order across enough of the island to prevent destructive domination and Cthulhu corruption.

## 6.5 Cthulhu faction

### Accepted broad identity

The Cthulhu faction has an unusual hidden victory function, escalating corruption, weather and terrain influence, weighted assistance events, and a summoning endgame.

### Proposed operational stages

#### Hidden

- Recruitment and infiltration.
- Dreams and secrets.
- Isolated corruption.
- Hidden ritual anchors.
- Avoidance of direct confrontation.

#### Emerging

- Connected corrupted regions.
- Monster production.
- Weather destabilization.
- Attacks on weakened settlements.
- Protection of ritual infrastructure.

#### Dominant

- Open territorial expansion.
- Destruction of containment.
- Isolation of enemies.
- Summoning preparation.
- Final ritual defense.

### Proposed economy

- Corruption.
- Fear.
- Death.
- Captives.
- Secrets.
- Magical sites.
- Ritual materials.
- Compromised people.
- Storms.
- Abandoned settlements.

### Proposed building functions

- Hidden meeting place.
- Corrupted household.
- Shrine.
- Ritual chamber.
- Captive site.
- Monster nest.
- Transformation site.
- Dream or influence anchor.
- Weather focus.
- Coastal or submerged gate.
- Summoning component.

### Accepted victory pressure

Corrupt the island sufficiently to complete the summoning of the major threat.

---

# 7. Faction relationships

## Accepted

Faction relationships are independent and pairwise rather than a single good-versus-evil alignment.

A relationship may track:

- Trust.
- Fear.
- Hatred.
- Grievance.
- Dependence.
- Trade value.
- Territorial conflict.
- Ideological incompatibility.
- Recent aid.
- Recent aggression.
- Treaty state.
- Known betrayal.
- Perceived strength.
- Perceived opportunity.

Temporary cooperation does not automatically become permanent alliance.

## Provisional relationship pressures

### Captain Michael and the fox people

Possible cooperation:

- Information for engineering support.
- Protection of communities.
- Opposition to Cthulhu.
- Resistance to colonial domination.

Possible conflict:

- Industry threatening spiritual sites.
- Mechanical surveillance threatening hidden routes.
- Rapid expansion becoming difficult to contain.

### Captain Michael and the colonial powers

Possible cooperation:

- Trade.
- Roads and ports.
- Military coordination.
- Anti-Cthulhu operations.

Possible conflict:

- His independent power challenges their authority.
- His technology threatens their advantage.
- Women in their institutions may defect.
- They may attempt to regulate, seize, license, or nationalize his workshops.

### Captain Michael and the pirates

Possible cooperation:

- Salvage.
- Smuggling.
- Parts.
- Sea transport.
- Informal trade.

Possible conflict:

- Machinery and convoys are valuable targets.
- Michael may impose route security.
- Pirate captains resist centralized authority.

### Captain Michael and the elves

Possible cooperation:

- Machinery used against corruption.
- Elven magic stabilizing weather or dangerous technology.
- Shared opposition to Cthulhu.

Possible conflict:

- Fuel consumption.
- Deforestation.
- Smoke.
- Roads through living terrain.
- Industrial expansion.

### Captain Michael and Cthulhu

Cthulhu can exploit:

- Dangerous experimentation.
- Concentrated infrastructure.
- Ambition.
- Corrupted power sources.
- Secrets carried by defectors.
- Jealousy and divided loyalty.
- The temptation to use forbidden knowledge to accelerate industrial growth.

---

# 8. Buildings, spawning, and standardized space

## Accepted

Buildings use standardized bounding boxes to support procedural placement and avoid collision.

Multi-box buildings are allowed.

Nothing essential may extend outside the declared envelope, including:

- Collision geometry.
- Walls.
- Roof masses.
- Stairs.
- Open doors.
- Navigation-critical projections.
- Interactive furniture.
- Required combat clearance.
- Worker positions.
- Vendor positions.
- Spawn points.
- Delivery points.

Each building should declare:

- Footprint cells.
- Clearance cells.
- Height class.
- Allowed terrain.
- Maximum slope.
- Entrance sockets.
- Road sockets.
- Worker sockets.
- Vendor sockets.
- Defender sockets.
- Spawn sockets.
- Delivery and storage sockets.
- Camera-occlusion profile.
- Upgrade compatibility.
- Ruined-state footprint.
- Capture compatibility.

## Faction production distinction

Other factions may use buildings to generate population or actor availability according to their economy.

Captain Michael's buildings generate machines, equipment, capacity, and services. Women join through recruitment, migration, relationships, rescue, or factional change rather than appearing because a building timer completed.

---

# 9. Strategic AI

## Accepted

Faction AI chooses goals and actor destinations through strategic utility.

Relevant considerations include:

- Survival.
- Threat.
- Hatred.
- Opportunity.
- Strategic value.
- Supply.
- Distance.
- Route danger.
- Relationship.
- Territorial pressure.
- Board position.
- Victory progress.
- Recovery needs.
- Player directives for Captain Michael's faction.
- Bounded personality variation.

Whether a faction merely survives or actively tries to win depends on its board position.

Useful strategic states include:

- Desperate.
- Recovering.
- Contesting.
- Advantaged.
- Closing.

These categories are simulation state, not necessarily player-facing labels.

## Captain Michael's faction

Player directives are high-weight strategic intentions, not magical commands.

The AI should be able to explain a major decision in ordinary language:

- What it is trying to accomplish.
- Why the target matters.
- Which resources are committed.
- What blocks progress.
- What would cause withdrawal.
- Whether the party's intervention would materially help.

Do not expose raw utility arithmetic as the normal interface.

---

# 10. Full local simulation and offscreen simulation

## Derived implementation direction

The active room and nearby relevant rooms should use full local simulation.

Distant activity should use aggregated strategic forces and events.

A distant force record may retain:

- Faction.
- Roles.
- Composition.
- Strength.
- Readiness.
- Supply.
- Origin.
- Route.
- Destination.
- Assignment.
- Progress.
- Player-detectable evidence.

When a force enters the active area, it materializes through valid routes and spawn sockets. It must not teleport into the room merely because the simulation decided reinforcements exist.

The transition between aggregate and local simulation must preserve:

- Composition.
- Damage.
- Supply.
- Leadership.
- Equipment.
- Relevant recruited-character identity.
- Travel history where player-facing consequences depend on it.

---

# 11. Dynamic difficulty

## Accepted

Enemy and friendly strength should derive from actual faction strength.

A stronger faction can field:

- Better-equipped forces.
- More experienced actors.
- More reliable reinforcement.
- Higher-tier buildings.
- More dangerous dungeons.
- Better defended supply.

A stronger allied faction can provide correspondingly better support.

Avoid arbitrary global scaling disconnected from the board.

The game becomes harder because events caused a faction to become stronger, not because the player has won a predetermined number of encounters.

Examples:

- The player destroys one faction's rival, freeing it to seize two ports.
- A faction protects its supply and upgrades a fort.
- Captain Michael expands faster than his routes can support.
- Cthulhu captures ritual sites and receives favorable weather.
- A player-created alliance gives one faction room to recover.

The cause should be inspectable in the world.

---

# 12. Dungeons and loot

## Accepted

Dungeon generation depends on more than a numerical level.

At minimum, generation should consider:

- Campaign seed.
- Node.
- Entrance building.
- Owning faction.
- Building archetype.
- Building tier.
- Upgrade history.
- Corruption state.
- Weather.
- World epoch.
- Encounter budget.
- Hazard budget.
- Reward budget.

A level-five dungeon controlled by one faction must not be the same dungeon with a palette swap as a level-two dungeon controlled by another faction.

Building tier increases the value and danger of attacking the location because a developed building contains or supports more:

- Equipment.
- Stored resources.
- Specialists.
- Defenses.
- Production.
- Strategic information.
- Faction-specific rewards.

Repeated farming must not generate infinite high-tier loot. Rewards must correspond to actual stored value, production, reinforcements, abandonment, and recovery.

---

# 13. Weather, magic, terrain, and Cthulhu

## Accepted

Weather is connected to magic.

Factions can change terrain and local conditions.

Cthulhu growth makes weather and terrain increasingly:

- Chaotic.
- Necromantic.
- Corrupt.
- Hostile to ordinary life.
- Favorable to Cthulhu movement, production, and ritual progress.

Cthulhu can receive weighted, state-gated scripted assistance events when strategically strong.

Those events must arise from visible board conditions and must not simply rescue Cthulhu whenever it is losing.

The confrontation can begin through three routes:

- Deliberate discovery.
- Day 100.
- Maximum hidden pressure.

World time and irreversible hidden pressure are separate systems.

Exact corruption reversibility, summoning-interruption rules, and early elimination rules remain open.

---

# 14. Pause and player-facing interface

## Accepted

The game pauses normally.

When paused:

- Local movement stops.
- Combat stops.
- Strategic simulation stops.
- Construction stops.
- Convoys stop.
- Weather progression stops.
- Cthulhu pressure stops advancing.
- No faction can capture territory, finish a ritual, or eliminate another faction.

The game does not progress while closed.

Full-screen management, reading, and accessibility interfaces should pause by default. The player should not be punished for reading slowly or studying the board.

The interface must be friendly rather than menacing.

Avoid:

- Constant flashing territory alerts.
- An alarm for every faction action.
- Red countdowns during ordinary play.
- A quest log filled automatically by simulation noise.
- Warnings about events the player cannot reasonably affect.
- Requiring frequent returns to the base for routine maintenance.

Use three levels of information:

## Ambient

Most changes simply appear in the world.

## Notable

A companion, messenger, journal, map update, or rumor identifies a potentially useful development without interrupting the player.

## Urgent

The game interrupts only for an immediate and understandable consequence involving the party, a major relationship, a critical player-faction location, or a final-stage threat.

Strategic planning should be available while paused.

---

# 15. Organic paths, victory, and loss

## Accepted

The dynamic island creates an organic player path.

The player is not expected to see all content or resolve every crisis in a prescribed order.

The simulation produces:

- Opportunities.
- Problems.
- Changed routes.
- Recoveries.
- Defections.
- New alliances.
- Faction collapses.
- Changed dungeons.
- Unexpected front lines.
- Mixed strategic outcomes.

There are many ways to win or lose.

The campaign can end in real defeat even after approximately one hundred hours of play. This is not a real-time countdown. It means a long investment does not make the player narratively immune to failure.

Day 100 remains an in-world escalation threshold and should not be confused with one hundred hours of real play.

## Provisional victory families

- Captain Michael establishes a durable independent power.
- A coalition stabilizes the island.
- The player supports an acceptable allied-faction victory.
- Cthulhu is defeated.
- Cthulhu is contained.
- A viable population escapes an unsalvageable island.
- Captain Michael's political and technological project transforms the final settlement.
- Key women from several factions create a new governing alliance around the player faction.

## Provisional loss families

- Captain Michael's faction loses every viable recovery path.
- Cthulhu completes an irreversible victory.
- Another faction closes the board in a way that makes the player's objectives impossible.
- The five-person party is permanently defeated.
- The player faction loses its identity through absorption, fracture, or political collapse.
- The island becomes unsustainable.
- Captain Michael wins a local war but creates an uncontested path for the true strategic victor.

## Mixed endings

The ending should record what actually survived rather than reducing the campaign to one score.

Possible combinations include:

- Cthulhu defeated but the player faction destroyed.
- Captain Michael's faction secure but a companion lost.
- The island preserved through an uncomfortable alliance.
- Most territory lost but a viable population evacuated.
- Technological success accompanied by political hatred.
- Captain Michael's ideas surviving even if he does not.

Loss must remain plainly identifiable as loss. Do not rename every outcome as success.

---

# 16. Faction elimination

## Accepted

Faction elimination is persistent.

A faction should be eliminated only when it has no viable recovery chain, including no useful combination of:

- Operational core building.
- Remaining population.
- Worker production or recruitment.
- Resource reserve.
- Controlled settlement.
- Mobile recovery force.
- Allied refuge.
- Valid construction site.
- Authored recovery event.

After elimination:

- Strategic scheduling stops.
- Production queues end.
- Buildings become ruined, abandoned, neutral, or capturable according to rules.
- Survivors may flee, surrender, become refugees, defect, or disappear according to content.
- Territory becomes available to competitors.
- Quests resolve, fail, or transform.
- Other factions immediately reevaluate the board.
- The eliminated faction does not automatically respawn.

Any rare story exception must be explicit.

---

# 17. Persistence and determinism

## Derived implementation direction

A save must preserve:

- Campaign seed.
- Island graph.
- Room revisions.
- Ownership.
- Influence.
- Buildings and tiers.
- Construction queues.
- Spawn and recruitment state.
- Faction resources.
- Relationships.
- Goals.
- Known intelligence.
- Strategic forces.
- Weather.
- Corruption.
- World time.
- Hidden pressure.
- Faction elimination.
- Dungeon generation signatures.
- Companion observations and theories.
- Recruitment histories.
- Romantic and political relationship state.
- Quest consequences.
- Strategic event history.

Irreversible events must remain irreversible after save and reload.

The same saved state, player actions, and simulation ticks should reproduce the same strategic outcomes unless nondeterminism is deliberately authored and recorded.

Normal pausing must not change simulation outcomes.

---

# 18. Production and visual constraints

## Accepted

- Fixed-view isometric presentation.
- Buildings can begin as cubes or multi-cube blockouts.
- Standard footprint sizes prevent procedural collisions.
- Nothing essential extends outside its declared placement box.
- Character production is likely more expensive than blockout architecture.
- Animation is deferred.
- Characters and machines should retain future-ready pivots, sockets, rigs, and metadata.
- Visual changes must be reviewed at actual gameplay distance.
- A readable blockout is not proof of finished production art.

## Character and unit asset implications

Captain Michael's faction needs a system that can retain a recruited woman's origin while adding shared player-faction equipment.

Assets should support:

- Stable character IDs.
- Origin faction.
- Current faction.
- Role.
- Equipment sockets.
- Hand sockets.
- Portrait reference.
- Navigation radius.
- Interaction anchor.
- Future rig.
- Shared and origin-specific clothing elements.
- Player-faction insignia or equipment overlays.
- Relationship and recruitment state.

Animal automata and vehicles need:

- Stable IDs.
- Operational footprint.
- Navigation width.
- Turning clearance.
- Maximum slope.
- Valid route types.
- Bridge requirements.
- Crew or handler requirements.
- Fuel and water requirements.
- Repair sockets.
- Local and offscreen representations.
- Wreck footprint.
- Salvage value.

---

# 19. Suggested authored data contracts

These are **Derived** implementation shapes, not locked programming-language definitions.

## Faction definition

```text
FactionDefinition
  id
  concept_key
  doctrine
  resource_priorities
  building_priorities
  recruitment_or_population_rules
  movement_preferences
  relationship_tendencies
  board_position_behavior
  victory_conditions
  recovery_rules
  elimination_rules
  weather_preferences
  terrain_influence
  corruption_interactions
  building_kit
  actor_kit
  dungeon_grammar
  loot_grammar
```

## Strategic directive

```text
StrategicDirective
  id
  issuing_character_id
  intent
  target_node_id
  target_region_id
  target_faction_id
  priority
  acceptable_risk
  resource_limit
  reserve_policy
  completion_condition
  withdrawal_condition
  state
  explanation
  blocking_reasons
```

## Recruitable woman

```text
RecruitmentState
  character_id
  origin_faction_id
  current_faction_id
  role
  awareness_of_michael
  attraction_to_michael
  romantic_interest
  trust_in_michael
  trust_in_companions
  ideological_alignment
  dissatisfaction_with_origin
  ambition
  personal_obligations
  perceived_safety
  perceived_opportunity
  fear_of_retaliation
  recruitment_stage
  conditions
  followers
  retained_relationships
  integration_state
  loyalty_state
```

Do not expose this complete structure as a numerical romance interface.

## Building definition

```text
BuildingDefinition
  id
  faction_compatibility
  function
  footprint_cells
  clearance_cells
  height_class
  entrance_sockets
  road_sockets
  actor_sockets
  delivery_sockets
  allowed_terrain
  maximum_slope
  construction_cost
  construction_requirements
  tier_states
  production
  services
  recruitment_support
  capture_rules
  ruin_state
  dungeon_relationship
  loot_relationship
```

## Dungeon context

```text
DungeonContext
  campaign_seed
  node_id
  entrance_building_id
  owning_faction_id
  building_archetype_id
  building_tier
  upgrade_signature
  corruption_band
  weather_state
  world_epoch
  generated_seed
  grammar_id
  encounter_budget
  hazard_budget
  reward_budget
  generated_room_ids
  revision
```

---

# 20. Approval ledger

## Accepted

- The game is an RTS beneath an RPG.
- The player controls Captain Michael and four women.
- The four women are active investigators, quest drivers, and leaders.
- The island is a complex network, not three lanes.
- One map node is one playable room.
- Factions autonomously build, spawn or recruit, route, fight, trade, recover, expand, and pursue victory.
- Faction relationships are independent.
- Factions can grow, shrink, replace one another, and be permanently eliminated.
- Building tier affects enemy strength, dungeon context, and loot.
- Faction identity affects dungeon generation.
- Enemy and allied strength derive from real board strength.
- Weather is tied to magic.
- Cthulhu growth corrupts terrain and makes weather more chaotic and favorable to itself.
- Cthulhu can receive weighted, state-gated assistance events.
- Deliberate discovery, Day 100, or maximum hidden pressure can trigger confrontation.
- World time and hidden pressure are separate.
- Normal pause stops local and strategic simulation.
- The UI should be calm and friendly rather than a constant warning system.
- The dynamic island should create organic player paths.
- There are many ways to win, lose, and reach mixed endings.
- A long campaign can end in genuine defeat.
- Captain Michael is a shipwreck survivor who creates a sixth faction.
- Captain Michael has history working with Argentines and in Texas.
- His faction is a powerful alternate-1870s steampunk faction.
- It possesses substantially more technology than the other factions.
- Steam is the primary industrial and motive language.
- Gas and electricity mainly support larger systems.
- Steam wagons are more likely than recognizable modern cars.
- Mobile machines often imitate animals familiar to 1870s people.
- Mechanical dogs, cavalry, bears, elephants, walkers, rockets, and airships are core candidate families.
- Captain Michael's faction plays mostly automatically but is easy to direct.
- It is the player's major board-scale instrument.
- Its people are recruited women rather than population manufactured by buildings.
- Captain Michael has an exceptional pull with women.
- That pull is a healthy, fun, important part of the faction rather than its whole identity.
- Women may pursue Michael and retain their own motives and agency.
- Recruitment can affect romance, diplomacy, intelligence, politics, and faction strength.
- Animation is deferred, but assets remain future-ready.
- Buildings use standard collision-safe envelopes.
- No proper faction names are to be invented yet.

## Rejected or superseded

- Three-lane world design.
- Ordinary static faction camps.
- Treating the game as standard tower defense.
- Overhead unit micromanagement as the main player interface.
- Arbitrary global enemy scaling.
- Automatic faction resurrection.
- Buildings with uncontrolled procedural overhang.
- Passive companions.
- Invented faction proper names.
- Captain Jack as the protagonist's current name.
- Making a personal mount the center of Captain Michael's faction identity.
- Making recruitment or seduction the faction's entire kit.
- Treating attraction as automatic mind control or an instant convert-enemy button.
- Having player-faction buildings generate human soldiers or workers from timers.
- Treating an attractive blockout as finished production art.

## Provisional

- Detailed strategic doctrines of the fox people, colonial powers, pirates, and elves.
- The staged operating model for Cthulhu.
- Exact animal-automata battlefield roles.
- Exact resource categories.
- Pairwise relationship pressures.
- Victory families and loss families.
- Active-room versus aggregated offscreen simulation.
- Utility scoring implementation.
- Strategic board-position categories.
- Snapshot plus event-journal persistence.
- Technology and recruitment data schemas.

## Open

- Exact historical year or tight year range.
- Exact alternate-history divergence.
- Exact nature of Captain Michael's Argentine and Texas history.
- Identities and backgrounds of the four women.
- How each woman joins the party.
- Which leadership portfolio each woman eventually owns.
- Exact founding location and first-base sequence.
- Political form of Captain Michael's faction.
- Exact civilian-membership boundary for men outside the female recruitment system.
- Exact resource list.
- Exact building-tier cap.
- Exact standard building dimensions.
- Capture versus destruction rules by building type.
- Exact ordinary-faction victory conditions.
- Exact allied tactical-command vocabulary.
- Exact time progression during conversations, travel, combat, rest, and explicit waiting.
- Which corruption effects are reversible.
- Cthulhu early-elimination rules.
- Cthulhu summoning-interruption rules.
- What happens if all ordinary factions collapse.
- How much of the island graph is authored versus generated.
- Final character asset strategy.
- Final machine asset strategy.
- Final animation plan.

---

# 21. Recommended next design work for Claude

Continue from this document rather than restarting the concept.

The highest-value next work is:

1. Refine the four ordinary faction doctrines into distinct but interacting RTS economies.
2. Test each doctrine against the island graph rather than designing isolated unit rosters.
3. Define the identities and strategic portfolios of the four principal women.
4. Design Captain Michael's faction-founding sequence from shipwreck to first strategic core.
5. Define a recruitment system that is frequent, fun, romantic, and strategically meaningful without dominating every encounter.
6. Define how recruited women retain origin-faction abilities while adopting Captain Michael's technology.
7. Define a first vertical slice involving two ordinary factions, Captain Michael's early machinery, one recruitable woman, one contested route, one building upgrade, one faction-specific dungeon, and a save/reload proof.
8. Maintain the accepted/provisional/open distinction in every continuation.

Do not implement the game unless explicitly requested and unless the actual Pirate Island repository has been located or created for that purpose.

---

# 22. Compact authority statement

Pirate Island is an autonomous RTS war experienced through Captain Michael and four active female companions. The island is a complex graph of playable rooms whose settlements, routes, buildings, dungeons, terrain, weather, populations, and ownership change as six factions act. Five established factions compete for survival and victory while Cthulhu corrupts the island and progresses toward a summoning. Captain Michael begins as a shipwreck survivor and creates a sixth faction: a powerful alternate-1870s steampunk force of steam wagons, animal-form automata, walkers, rockets, airships, advanced industry, recruited women, and player-directed strategy. His unusual pull with women creates recurring romantic and political recruitment opportunities, but it is one part of a broader industrial, military, diplomatic, and territorial faction. The faction plays mostly automatically and is easy to direct. The player may pause normally, pursue an organic path, and reach multiple victories, mixed outcomes, or genuine defeat.
