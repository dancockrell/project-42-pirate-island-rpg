# Character and Haremlit Authoring Contract

> **Current scope:** follow [GAME_BUILD_PLAN.md](GAME_BUILD_PLAN.md). Michael starts alone and may recruit adult women from the island's shared NPC population. Up to four recruited women may occupy his active companion slots. Notables and generated ordinary women use the same population and membership rules; there is no separate protected heroine caste. Use the approved 2D sprite presentation, not the historical card-battle or rig-ready format.

## Recruitment, faction membership, and party contract

**Authority:** the following consolidates the user's NPC-pool decisions. These are required game rules, not a claim that the current native island implements them. The current slice has production, movement, combat, casualties and faction elimination, but no recruitment or companion roster.

- Michael begins as the sole shipwreck survivor in his faction. Do not seed four allies merely to satisfy a five-character screenshot.
- Every adult female person is potentially recruitable under the right conditions, irrespective of current faction, ordinary/hero status, madness, or suitable undead form. Conditions may involve relationships, quests, artifacts or restoration; potential eligibility is not instant success.
- Male and non-female units cannot be recruited by Michael's distinctive power. Animals and machines do not become romance candidates merely because an asset has a sex label. Mechanical production is Michael's separate faction strength.
- Faction membership is not an active companion slot. More than four women may join Michael's faction; only four accompany him as the main party. Removing a woman from the active party does not revoke faction membership or loyalty.
- Ordinary women are named generated workers or fighters produced by buildings. Notables use the same person record with authored identity and deeper quests. Heroes may be much stronger than ordinary women and each other; do not equalize their strength to make four interchangeable slots.
- Both men and women in other factions can rebel under low morale. Women's willingness to join Michael is a separate relationship decision, not a synonym for low morale or low health. Do not implement recruitment by shooting a woman until she agrees.
- Successful recruitment transfers the existing person, equipment, health and abilities. It weakens the source faction rather than creating a duplicate. Diplomatic effects depend on the circumstances: departure may offend the source faction, be negotiated, or resolve a problem for it.
- A woman who has joined Michael does not voluntarily defect afterward. Death and Cthulhu's return are separate from defection. Preserve her attachment to Michael across that transition so recovery can be easier without becoming automatic.
- Death preserves identity. Unrecovered eligible dead return on Cthulhu's side at midnight; faction or allied resurrection machinery/rituals can avert or reverse that outcome. Do not erase a dead companion's history, clone a replacement, or reset romance progress.

### Single simulation owner

Extend the existing `FactionWorld` and `ProducedActor` in `godot-rust/src/world.rs`. Godot renders the native snapshot and submits commands; it must not maintain a second authoritative party, romance score or faction ownership table.

Keep these concepts distinct in that owner:

| Record | Meaning and persistence |
| --- | --- |
| Person identity | Stable instance ID, displayed name, adult age, person/species kind, sex, original production provenance; survives transfer and death. Definition ID alone is not personal identity. |
| Narrative provenance | Authored notable profile or generated backstory seed and resolved quest selections. Loading must not reroll names, appearance, conditions or history. |
| Current allegiance | Existing actor faction ID. Original faction in production provenance remains historical and must not be rewritten on recruitment. |
| Recruitment relationship | Michael-specific relationship state, resolved conditions and permanent prior-membership marker. Separate from faction morale and diplomacy. |
| Life state | Existing live actor or casualty record, later extended with undead/restoration state. Exactly one incarnation of each identity can be active. |
| Active party | Ordered roster of at most four female member IDs, separate from total faction population. No duplicated slots; Michael is not one of the four slots. |

Unknown identity fields in older saves remain unknown, not silently inferred from a portrait or name. Existing male preview art must not render a newly female actor unchanged. Admit matching female sprite variants before producing those identities in the visible slice.

### Command behavior

1. **Inspect/talk:** select an actual known NPC. Show her name, current faction and current conversation choices. Offer recruitment only through a resolved authored or generated encounter; show a concise reason when she declines or a requirement is unmet. No global remote recruit button.
2. **Join faction:** native validation checks living/restorable state as appropriate, adult female person eligibility, Michael's living state, actual encounter access, and fulfilled recruitment conditions. On success atomically transfer ownership and population accounting, remove stale source-faction orders, clear illegal queued attacks, record permanent attachment, and apply the specified diplomatic consequence once. Retrying must not duplicate population or rewards.
3. **Join party:** after joining the faction, the player may explicitly fill an empty slot. A full roster offers a deliberate replacement choice; never silently eject someone or reject faction membership because all four party slots are occupied.
4. **Travel:** the party command proposes reachable nearby destinations for living active members using the existing navigation and travel queue. Do not teleport followers. If a member cannot reach the destination, retain her valid position and tell the player who is separated. Pause freezes everyone normally.
5. **Leave active party:** remove the slot assignment, not the person's faction, relationship or equipment. A safe fallback duty/location must be resolved through the same orders, not by deleting the actor.
6. **Death/recovery:** retain a dead member's slot until the player chooses replacement, so failure is visible rather than silently repaired. A dead slot issues no movement or combat command. Returning her to life or recovering her from Cthulhu restores the same identity and relationship; roster eligibility is revalidated against current allegiance and life state.

Exact relationship thresholds, negotiation costs, morale rates, resurrection costs and the calendar's real-time duration remain tunable/unresolved. Do not smuggle provisional values into the user-approved rules. An initial authored conversation can demonstrate recruitment without inventing the complete procedural romance system.

### Required verification before calling the recruitment slice playable

- Start with Michael alone; building production creates distinct named adult NPCs with admitted matching sprites.
- Resolve one real NPC encounter and recruit that same visible person; the source loses her, Michael gains her, and no copy remains.
- Recruit five women into the faction, select four as companions, then deliberately replace one while preserving both women's faction membership and histories.
- Reject male/non-person/ineligible-state recruitment without mutation; distinguish temporarily unmet conditions from permanent category exclusion.
- Check pursuit orders, queued attacks, population reservations and diplomacy after transfer. Recheck target ownership when a shot resolves, not only when it is queued.
- Walk the selected party around blocked terrain, pause mid-route, save/load, and reproduce positions, order, equipment and membership.
- Kill a companion, retain her identity and slot, then prove midnight transfer and faction restoration once those systems exist. Until then, label that branch unimplemented rather than reporting the whole recruitment lifecycle complete.
- Visually review party selection, actual map-scale male/female sprites and moving formations. Structural tests or source sheets alone are not visual acceptance.

## Purpose

Project 42 is a haremlit adventure. A heroine is never only a combat class, a
portrait, or a reward for clearing content. Every core woman must be someone
the player can understand in private, in public, on an expedition, at the
household, and in the changing shape of the group. The Captain receives the
same treatment. His history with the *Handsome Jack*, his invention work, and
his ability to learn from the women around him must change what the player can
see and do.

## Required Record for Michael and Authored Notables

Before an authored major character enters production, the character source
must point to a profile that supplies the following. Generated ordinary women
instead use reviewed name/backstory/quest templates with stable seeds; they do
not require a full bespoke seven-part narrative before they can be units.

1. **History.** A concrete life before the first meeting: home, former work,
   important competence, a loss or unfinished obligation, and a reason she is
   on the island.
2. **Present pressure.** A specific current problem with people, place and
   stakes. It must create quests, conversation scenes, exploration choices or
   household decisions.
3. **Competence.** A field in which she can teach Michael something real. Her
   knowledge must affect an invention, investigation, base upgrade, route,
   battle option or social solution.
4. **Household life.** A room, routine, project, preference and one meaningful
   interaction with at least one other heroine. The household must still feel
   alive when Michael is away.
5. **Romance progression.** A clear route from meeting to committed household
   member, with attraction, trust, flirtation, intimacy and a positive shared
   future. The route cannot use romantic rivals, cheating, betrayal or a late
   rug-pull as its main source of drama.
6. **Group relationships.** At least two specific relationships with other
   women: friendship, teasing, mentorship, shared work, attraction, courtship,
   sex, love, or a mix appropriate to the pair. These relationships are part
   of the harem fantasy, not side material that vanishes when Michael enters a
   scene.
7. **Adventure expression.** A memorable character concept, signature weapon,
   four powers, an ultimate, one party passive and one strategic passive for
   a hero; readable actions, exploration contribution and a visible reason to
   bring her along. Ordinary units retain their smaller unit kit. Historical
   seven-rank skill prototypes are not a universal NPC requirement.

## Michael Corrigan

Michael's title-card name is **Michael**. His full name is **Captain Michael
Corrigan**. “Captain Corrigan” is used in formal, military, contractual and
hostile contexts.

He was the captain of the *Handsome Jack*, an obsolete tea cutter equipped
with a later steam refit. The engine made the cutter unusually fast in good
conditions, although it was too heavy and poorly balanced for the hull. A
typhoon punished that weakness and tore the ship apart. Michael survived.

His visual language is fantastic 1870s American frontier filtered through a wrecked
steampunk ship: slouch hat, worn sea coat, gun belt, compact boiler pack,
brass tools, Echo compass, *Handsome Jack* steam carbine, experimental sidearm
and cutlass. Every piece should look repairable and related to the same person.

Michael is a genius inventor, not a man who arrives knowing every answer. Each
heroine can teach him a method she actually owns. Their joint projects turn
those methods into practical expedition hardware, base upgrades and Echo
applications. This gives romance scenes, conversations and group bonds a
visible downstream effect without turning a heroine into a consumable upgrade.

## Content Gate

An authored notable is production-ready only after her identity, gameplay kit,
quest/relationship content and required sprite actions have been reviewed
together. A generated ordinary NPC instead needs validated templates, stable
identity, legal unit behavior and admitted sprite variants. Neither gate
requires 3D rigs or historical title-card behavior. Do not confuse narrative
depth with recruiting eligibility or use this gate to create a special,
unrecruitable category of women.
