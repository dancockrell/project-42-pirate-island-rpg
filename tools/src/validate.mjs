import { readFile, readdir, stat } from "node:fs/promises";
import { resolve, relative } from "node:path";
import process from "node:process";
import { resolveDerivedRecords } from "./derived-records.mjs";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];
const ids = new Map();
const references = [];
// Every rule here has real behaviour in `game/scripts/battle/targeting_session.gd`.
// A7 added `automatic_reaction_to_hostile_attack_in_shared_band` for Ayla's
// Reach Counter: a reaction the player never targets, like Fatal Intercept's,
// but on a different trigger -- Fatal Intercept's names a lethal hit and Reach
// Counter's does not, so reusing that rule would have been a false statement
// about when the skill fires.
const supportedTargetRules = new Set(["self", "one_hostile", "one_living_hostile", "one_living_party_member", "ordered_pair_threatened_ally_then_hostile", "automatic_reaction_to_other_party_member_lethal_hit", "automatic_reaction_to_hostile_attack_in_shared_band", "all_living_party_members", "one_defeated_party_member_other_than_betty"]);
const bindableBattleEvents = new Set(["actor_focused", "actor_moved", "damage_applied", "guard_changed", "vitality_changed", "status_removed", "interception_set", "interception_triggered", "reaction_window_opened", "reaction_triggered", "defeat_prevented", "actor_revived", "bonus_turn_granted", "battlefield_effect_created", "battlefield_effect_pulse", "battlefield_effect_removed", "recovery_opening_created", "recovery_opening_consumed", "recovery_opening_expired", "actor_defeated", "turn_ended", "battle_ended", "target_inspected", "status_applied", "ward_line_placed", "ward_line_triggered", "activation_denied", "site_rule_overridden"]);
let skillCount = 0;
let presentationCueCount = 0;

function fail(file, message) { failures.push(`${relative(repo, file)}: ${message}`); }
function requireString(record, key, file) {
  if (typeof record[key] !== "string" || record[key].trim() === "") fail(file, `${key} must be a non-empty string`);
}
function registerId(id, file) {
  if (typeof id !== "string") return fail(file, "id must be a string");
  if (ids.has(id)) fail(file, `duplicate stable ID ${id}; first declared in ${relative(repo, ids.get(id))}`);
  else ids.set(id, file);
}
function reference(id, file, field) {
  if (typeof id !== "string" || id.trim() === "") fail(file, `${field} must contain a stable ID`);
  else references.push({ id, file, field });
}
async function readJsonDirectory(name) {
  const directory = resolve(repo, `content/${name}`);
  const records = [];
  for (const filename of (await readdir(directory)).filter(value => value.endsWith(".json"))) {
    const file = resolve(directory, filename);
    const value = JSON.parse(await readFile(file, "utf8"));
    registerId(value.id, file);
    records.push({ file, value });
  }
  return records;
}

const characterDir = resolve(repo, "content/characters");
for (const name of (await readdir(characterDir)).filter(name => name.endsWith(".json"))) {
  const file = resolve(characterDir, name);
  const value = JSON.parse(await readFile(file, "utf8"));
  registerId(value.id, file);
  requireString(value, "displayName", file);
  requireString(value, "implementationOwner", file);
  requireString(value, "presentationOwner", file);
  if (value.presentationOwner !== "godot-gdscript") fail(file, "presentationOwner must be godot-gdscript");
  if (!Array.isArray(value.tags) || value.tags.length < 3) fail(file, "tags must contain at least three searchable values");
  if (!Array.isArray(value.skillIds)) fail(file, "skillIds must be an array");
  else if (value.kind === "heroine" && value.skillIds.length !== 7) fail(file, "a heroine must declare exactly seven D-through-SSS skills");
  else value.skillIds.forEach((id, index) => reference(id, file, `skillIds[${index}]`));
  if (!value.art?.accessibilityDescription || value.art.accessibilityDescription.length < 20) fail(file, "art accessibilityDescription is missing or too short");
  if (value.art?.readyIdleMotionId) reference(value.art.readyIdleMotionId, file, "art.readyIdleMotionId");
  if (value.art?.status === "placeholder" && value.metadata?.releaseLegal !== false) fail(file, "placeholder character must set metadata.releaseLegal=false");
}

// The seven authored bond-rank letters, in order, low to high. One list, two
// readers: a skill's own `bondRank` and (A10) a relationship scene's
// `rule.raisesBondRankTo`. `battle::rank_index` is its twin in Rust; the ladder
// is letters and there is no numeric spelling of it anywhere.
const bondRanks = ["D", "C", "B", "A", "S", "SS", "SSS"];

for (const { file, value } of await readJsonDirectory("skills")) {
  skillCount += 1;
  requireString(value, "displayName", file);
  reference(value.ownerId, file, "ownerId");
  if (!bondRanks.includes(value.bondRank)) fail(file, `bondRank must be one of ${bondRanks.join(", ")}`);
  if (!value.rules || typeof value.rules !== "object") fail(file, "rules must be an object");
  if (!Array.isArray(value.animation?.beats) || value.animation.beats.length < 4) fail(file, "animation must contain at least four explicit beats");
  if (!value.animation?.framing?.includes("safe frame")) fail(file, "animation framing must state its safe-frame requirement");
  if (!supportedTargetRules.has(value.targetRule)) fail(file, `targetRule ${value.targetRule} has no registered Godot targeting-session behavior`);
  if (Array.isArray(value.animation?.beats)) {
    let previousAt = -1;
    const beatNames = new Set();
    for (const [index, beat] of value.animation.beats.entries()) {
      if (!Number.isInteger(beat?.atMs) || beat.atMs < 0) fail(file, `animation.beats[${index}].atMs must be a non-negative integer`);
      if (beat?.atMs < previousAt) fail(file, "animation beats must be ordered by atMs");
      if (typeof beat?.name !== "string" || beat.name.length === 0) fail(file, `animation.beats[${index}] requires a direct action name`);
      if (beatNames.has(beat?.name)) fail(file, `animation beat name ${beat?.name} is duplicated`);
      beatNames.add(beat?.name);
      previousAt = beat?.atMs ?? previousAt;
    }
    if (!Number.isInteger(value.animation?.durationMs) || value.animation.durationMs < previousAt) fail(file, "animation durationMs must include the final authored beat");
    const bindings = value.animation?.eventBindings;
    if (!bindings || typeof bindings !== "object" || Array.isArray(bindings) || Object.keys(bindings).length < 3) fail(file, "animation eventBindings must bind at least three authoritative event kinds");
    else for (const [eventKind, beatName] of Object.entries(bindings)) {
      if (!bindableBattleEvents.has(eventKind)) fail(file, `animation eventBindings contains unknown event kind ${eventKind}`);
      if (!beatNames.has(beatName)) fail(file, `animation event ${eventKind} targets missing beat ${beatName}`);
    }
    const cues = value.animation?.presentationCues;
    if (Array.isArray(cues)) presentationCueCount += cues.length;
    if (!Array.isArray(cues) || cues.length !== beatNames.size) fail(file, "animation presentationCues must define exactly one cue per beat");
    else {
      const cueBeats = new Set();
      for (const [index, cue] of cues.entries()) {
        for (const field of ["beat", "pose", "motion", "cameraId", "vfxId", "audioCueId", "audio"]) {
          if (typeof cue?.[field] !== "string" || cue[field].length === 0) fail(file, `animation.presentationCues[${index}].${field} must be a direct non-empty instruction`);
        }
        if (!beatNames.has(cue?.beat)) fail(file, `presentation cue targets missing beat ${cue?.beat}`);
        if (cueBeats.has(cue?.beat)) fail(file, `presentation cue for beat ${cue?.beat} is duplicated`);
        cueBeats.add(cue?.beat);
        if (!Number.isInteger(cue?.hitStopMs) || cue.hitStopMs < 0 || cue.hitStopMs > 200) fail(file, `presentation cue ${cue?.beat} hitStopMs must be an integer from 0 through 200`);
        if (typeof cue?.shake !== "number" || cue.shake < 0 || cue.shake > 1) fail(file, `presentation cue ${cue?.beat} shake must be from 0 through 1`);
        if (!Array.isArray(cue?.frameSubjects) || cue.frameSubjects.length < 2) fail(file, `presentation cue ${cue?.beat} must name at least two safe-frame subjects`);
        reference(cue?.cameraId, file, `animation.presentationCues[${index}].cameraId`);
        reference(cue?.vfxId, file, `animation.presentationCues[${index}].vfxId`);
        // P9: audio follows the VFX stable-reference rule -- a cue names an
        // `audio.cue.*` record, never an improvised sound label. The free-text
        // `audio` field that predates it is the same statement in prose, so the
        // two are held equal here rather than left to drift into two answers.
        reference(cue?.audioCueId, file, `animation.presentationCues[${index}].audioCueId`);
        if (typeof cue?.audio === "string" && cue?.audioCueId !== `audio.cue.${cue.audio}`) fail(file, `presentation cue ${cue?.beat} audioCueId must be audio.cue.${cue?.audio}, the stable record for the sound its audio label names`);
      }
    }
  }
  if (value.id === "skill.betty.condition_cleanse" && JSON.stringify(value.rules?.removalPriority) !== JSON.stringify(["stunned", "burning", "poisoned", "bleeding"])) {
    fail(file, "Condition Cleanse must preserve its deterministic status-removal priority");
  }
  if (value.id === "skill.betty.rescue_charge" && value.targetRule !== "ordered_pair_threatened_ally_then_hostile") {
    fail(file, "Rescue Charge must name the rescued ally first and threatening hostile second");
  }
  if (value.id === "skill.betty.healing_impact") {
    if (value.rules?.healingPercentOfActualVitalityDamage !== 50 || value.rules?.healingRounding !== "floor") fail(file, "Healing Impact must heal 50 percent of actual Vitality damage using floor rounding");
    if (value.rules?.recipientTieBreak !== "stable_actor_id_ascending" || value.rules?.resolveHealingBeforeVictory !== true) fail(file, "Healing Impact selection and event order must remain deterministic");
  }
  if (value.id === "skill.betty.fatal_intercept") {
    if (value.rules?.incomingDamageResult !== "cancel_entire_hit" || value.rules?.counterRawDamage !== 24) fail(file, "Fatal Intercept must cancel the lethal hit and counter for 24 raw damage");
    if (value.rules?.nestedReactionsAllowed !== false || value.rules?.rescueChargeInterceptionResolvesFirst !== true) fail(file, "Fatal Intercept must prohibit nested reactions and resolve after ordinary interception");
  }
  if (value.id === "skill.betty.mobile_infirmary") {
    if (value.activation !== "normal_action" || value.targetRule !== "all_living_party_members") fail(file, "Mobile Infirmary must be a zero-selection normal action affecting all living party members");
    if (value.rules?.pulseCountTotal !== 3 || value.rules?.pulseTiming !== "immediate_then_start_of_next_two_betty_turns") fail(file, "Mobile Infirmary must pulse immediately and at the start of Betty's next two turns");
    if (value.rules?.healingPerPulse !== 10 || value.rules?.guardPerPulse !== 2) fail(file, "Mobile Infirmary must restore 10 Vitality and grant 2 Guard per pulse");
    if (value.rules?.defeatedTargetsExcluded !== true || value.rules?.sourceDefeatRemovesEffect !== true) fail(file, "Mobile Infirmary must exclude defeated allies and end when Betty is defeated");
    if (value.rules?.sameSkillRecast !== "replace_existing_effect" || value.rules?.sourceStunDoesNotSuspendDeployedEffect !== true) fail(file, "Mobile Infirmary replacement and source-stun behavior must remain explicit");
  }
  if (value.id === "skill.betty.combat_revival") {
    if (value.activation !== "normal_action" || value.targetRule !== "one_defeated_party_member_other_than_betty") fail(file, "Combat Revival must target exactly one defeated party member other than Betty");
    if (value.rules?.restoreVitalityPercentOfMaximum !== 40 || value.rules?.restoreRounding !== "ceiling" || value.rules?.minimumRestoredVitality !== 1) fail(file, "Combat Revival must restore 40 percent maximum Vitality with ceiling rounding and a minimum of one");
    if (value.rules?.guardAfterRevival !== 0 || value.rules?.removeAllNegativeStatuses !== true) fail(file, "Combat Revival must reset Guard and clear all negative statuses");
    if (value.rules?.grantImmediateBonusTurn !== true || value.rules?.resumeInitiative !== "natural_successor_after_betty") fail(file, "Combat Revival must grant a bonus turn before resuming after Betty");
    if (value.rules?.oncePerBattle !== true || value.rules?.cannotTargetSelf !== true) fail(file, "Combat Revival must remain once per battle and unable to target Betty");
    if (value.rules?.countsAsWorldDeath !== false || value.rules?.changesIslandDeathCounter !== false) fail(file, "Combat defeat and revival must not alter persistent island death memory");
    if (value.rules?.fatalInterceptResolvesBeforeDefeat !== true || value.rules?.mobileInfirmaryCanAffectAfterRevival !== true) fail(file, "Combat Revival interactions with Fatal Intercept and Mobile Infirmary must remain explicit");
  }
}

// C1: loot records exist so `victory.lootTableId` and every habitat
// `drop_table_id` resolve against a registered stable ID instead of naming a
// record nobody wrote. Each yield line is a non-negative integer count of one
// resource the simulation already moves.
const lootResources = ["rations", "medicine", "coin"];
for (const { file, value } of await readJsonDirectory("loot")) {
  requireString(value, "displayName", file);
  if (!value.id?.startsWith("loot.")) fail(file, "loot record id must use the loot. prefix");
  if (!value.yield || typeof value.yield !== "object" || Array.isArray(value.yield)) fail(file, "loot record requires a yield object");
  else for (const resource of lootResources) {
    if (!Number.isInteger(value.yield[resource]) || value.yield[resource] < 0) fail(file, `yield.${resource} must be a non-negative integer`);
  }
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "loot metadata.releaseLegal must be boolean");
}

// C9: the six authored faction records, mirroring `FactionDefinition::validate`
// in `godot-rust/src/strategy/faction.rs` on the content side. Rust refuses a
// record whose ID is not exactly `faction.<concept_key>`; this refuses the same
// record before it ever reaches Rust, and adds the check Rust has no field to
// make -- that no proper faction name has been invented. Brief section 4: "Do
// not invent proper faction names without explicit approval", so `displayName`
// must keep saying it is undecided. When a human approves names, this rule is
// what they replace.
const factionConceptKeys = ["fox_people", "colonial_powers", "pirates", "elves", "cthulhu", "michael"];
const factionMapFields = ["resource_priorities", "building_priorities", "movement_preferences", "relationship_tendencies", "board_position_behavior", "weather_preferences", "terrain_influence"];
const factionStringFields = ["doctrine", "recruitment_or_population_rules", "recovery_rules", "elimination_rules", "corruption_interactions", "dungeon_grammar", "loot_grammar"];
const factionArrayFields = ["victory_conditions", "building_kit", "actor_kit"];
const authoredConceptKeys = new Set();
for (const { file, value } of await readJsonDirectory("factions")) {
  if (!factionConceptKeys.includes(value.concept_key)) fail(file, `concept_key ${value.concept_key} is not one of the six the brief accepts: ${factionConceptKeys.join(", ")}`);
  else if (authoredConceptKeys.has(value.concept_key)) fail(file, `a second record claims concept_key ${value.concept_key}; one concept, one record`);
  else authoredConceptKeys.add(value.concept_key);
  if (value.id !== `faction.${value.concept_key}`) fail(file, `id ${value.id} must be exactly faction.${value.concept_key}`);
  requireString(value, "displayName", file);
  // Deliberately inverted: the validator enforces that a name has NOT been
  // invented. A record that reads as named fails until the decision lands.
  if (typeof value.displayName === "string" && !value.displayName.includes("needs decision")) fail(file, "displayName must stay a placeholder containing \"needs decision\" until proper faction names are approved (brief section 4)");
  for (const field of factionStringFields) {
    if (typeof value[field] !== "string") fail(file, `${field} must be a string note, empty when the decision is still Open`);
  }
  for (const field of factionMapFields) {
    if (!value[field] || typeof value[field] !== "object" || Array.isArray(value[field])) fail(file, `${field} must be an object keyed by authored strings`);
  }
  for (const field of factionArrayFields) {
    if (!Array.isArray(value[field])) fail(file, `${field} must be an array`);
    else if (value[field].some(entry => typeof entry !== "string")) fail(file, `${field} must contain only strings`);
  }
  // Brief section 20 leaves the exact resource list Open, so a record may only
  // key `resource_priorities` by an obvious placeholder. An invented category
  // here would be this repository deciding the economy.
  for (const [key, weight] of Object.entries(value.resource_priorities ?? {})) {
    if (!key.startsWith("resource.open.")) fail(file, `resource_priorities key ${key} invents a resource category; brief section 20 leaves the resource list Open, so keys stay under resource.open.`);
    if (!Number.isInteger(weight)) fail(file, `resource_priorities.${key} must be an integer weight`);
  }
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "faction metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "faction metadata.notes must list what stays Provisional or Open");
}
for (const key of factionConceptKeys) {
  if (!authoredConceptKeys.has(key)) failures.push(`content/factions/: no record authors concept_key ${key}; the brief accepts exactly six factions`);
}

// C10: building records. S3 shipped `BuildingDefinition` and refuses at load
// every record this block refuses here; the two refusals are deliberately the
// same set, stated once in `godot-rust/src/strategy/building.rs` and mirrored
// once here, so a bad record dies at `node tools/src/validate.mjs` instead of
// only in `cargo test`. Read that file, not this one, for why each rule exists.
//
// The vocabularies below are the serde spellings of S3's enums, verbatim:
// `SocketKind`, `ProductionOutput`, `CaptureRules` and `RuinState` all carry
// `#[serde(rename_all = "snake_case")]`, so a record writes `entrance`,
// `machine`, `capturable`, `clears_completely`. `human_role` and
// `leaves_rubble` carry data, so they are authored as single-key objects.
// `every_authored_building_record_loads_and_validates` is what keeps these
// lists honest: it puts every file in this directory through the real Rust
// `validate`, so a spelling that drifts fails there.
const TIER_CAP = 3; // mirrors building.rs's TIER_CAP -- brief section 20, still needs decision
// Which of brief section 19's four socket fields each of section 8's socket
// kinds is authored in. This is `SocketKind::field()`, mirrored.
const socketFieldForKind = {
  entrance: "entrance_sockets",
  road: "road_sockets",
  worker: "actor_sockets",
  vendor: "actor_sockets",
  defender: "actor_sockets",
  spawn: "actor_sockets",
  delivery: "delivery_sockets",
  storage: "delivery_sockets"
};
const socketFields = ["entrance_sockets", "road_sockets", "actor_sockets", "delivery_sockets"];
// Mirrors `ProductionOutput` in godot-rust/src/strategy/building.rs. S13 made
// `machine` a struct variant carrying a `family`, so the bare string is refused
// here exactly as serde refuses it; the eight families are `MachineFamily::ALL`
// in strategy/production.rs (brief section 5.4), and the Rust equality test over
// content/buildings/ is what keeps this list honest.
const simpleProductionOutputs = new Set(["capacity", "service"]);
const machineFamilies = new Set(["mechanical_dog", "mechanical_cavalry", "mechanical_bear", "mechanical_elephant", "walker", "steam_wagon", "rocket", "airship"]);
let buildingCount = 0;
for (const { file, value } of await readJsonDirectory("buildings")) {
  buildingCount += 1;
  if (typeof value.id !== "string" || !value.id.startsWith("building.")) fail(file, `id ${value.id} must be building.<slug>`);
  requireString(value, "function", file);
  for (const field of ["dungeon_relationship", "loot_relationship"]) {
    if (typeof value[field] !== "string") fail(file, `${field} must be a string note, empty when the decision is still Open`);
  }

  // Concept keys, never proper names -- the same closed list C9 authors
  // against, because `ConceptKey` is closed in Rust for the same reason.
  if (!Array.isArray(value.faction_compatibility) || value.faction_compatibility.length === 0) fail(file, "faction_compatibility must name at least one faction concept key");
  else for (const key of value.faction_compatibility) {
    if (!factionConceptKeys.includes(key)) fail(file, `faction_compatibility ${key} is not one of the six concept keys the brief accepts: ${factionConceptKeys.join(", ")}`);
  }
  const buildsForMichael = Array.isArray(value.faction_compatibility) && value.faction_compatibility.includes("michael");

  // The envelope. Brief section 8: nothing essential outside it, so it is
  // counted, and every count is an integer share of a cell's capacity rather
  // than a measurement (brief section 20 leaves real dimensions Open).
  for (const field of ["footprint_cells", "clearance_cells"]) {
    if (!Number.isInteger(value[field]) || value[field] < 0) fail(file, `${field} must be a non-negative integer count of envelope cells`);
  }
  if (Number.isInteger(value.footprint_cells) && value.footprint_cells < 1) fail(file, "footprint_cells must be at least 1; a building with no footprint occupies no ground and cannot hold a socket");
  if (!Number.isInteger(value.maximum_slope) || value.maximum_slope < 0 || value.maximum_slope > 255) fail(file, "maximum_slope must be an integer from 0 through 255 (a u8 downstream)");
  // Deliberately inverted, as C9 does for displayName: the validator enforces
  // that a height band has NOT been invented. Brief section 20, "exact
  // standard building dimensions".
  requireString(value, "height_class", file);
  if (typeof value.height_class === "string" && !value.height_class.includes("needs decision")) fail(file, "height_class must stay a placeholder containing \"needs decision\" until standard building dimensions are decided (brief section 20)");

  const socketIds = new Set();
  for (const field of socketFields) {
    if (!Array.isArray(value[field])) { fail(file, `${field} must be an array of sockets, empty when the building declares none`); continue; }
    for (const [index, socket] of value[field].entries()) {
      const where = `${field}[${index}]`;
      if (typeof socket?.id !== "string" || socket.id.trim() === "") fail(file, `${where}.id must be a non-empty string`);
      else if (socketIds.has(socket.id)) fail(file, `${where}.id ${socket.id} is used twice in this record`);
      else socketIds.add(socket.id);
      const expectedField = socketFieldForKind[socket?.kind];
      if (!expectedField) fail(file, `${where}.kind ${socket?.kind} is not a SocketKind: ${Object.keys(socketFieldForKind).join(", ")}`);
      else if (expectedField !== field) fail(file, `${where} is a ${socket.kind} socket, which is authored in ${expectedField}, not ${field}`);
      if (!Number.isInteger(socket?.offset_cells) || socket.offset_cells < 0) fail(file, `${where}.offset_cells must be a non-negative integer`);
      else if (Number.isInteger(value.footprint_cells) && socket.offset_cells >= value.footprint_cells) fail(file, `${where}.offset_cells ${socket.offset_cells} sits outside the declared footprint of ${value.footprint_cells}; brief section 8 puts nothing essential outside the envelope`);
    }
  }

  // Ascending from 1, no gaps, no more than the cap allows -- fewer is fine.
  if (!Array.isArray(value.tier_states) || value.tier_states.length === 0) fail(file, "tier_states must author at least one tier");
  else {
    if (value.tier_states.length > TIER_CAP) fail(file, `tier_states authors ${value.tier_states.length} tiers; TIER_CAP is ${TIER_CAP} (brief section 20, still needs decision)`);
    for (const [index, tier] of value.tier_states.entries()) {
      if (tier?.tier !== index + 1) fail(file, `tier_states[${index}].tier must be ${index + 1}; tiers ascend from 1 with no gaps and no repeats`);
      for (const field of ["construction_hours", "hit_points"]) {
        if (!Number.isInteger(tier?.[field]) || tier[field] < 0) fail(file, `tier_states[${index}].${field} must be a non-negative integer`);
      }
      if (!Array.isArray(tier?.construction_requirements) || tier.construction_requirements.some(entry => typeof entry !== "string")) fail(file, `tier_states[${index}].construction_requirements must be an array of authored flag strings`);
      if (typeof tier?.notes !== "string") fail(file, `tier_states[${index}].notes must be a string`);
    }
  }
  const topTier = Array.isArray(value.tier_states) ? value.tier_states.length : 0;

  for (const field of ["services", "recruitment_support", "allowed_terrain", "construction_requirements"]) {
    if (!Array.isArray(value[field]) || value[field].some(entry => typeof entry !== "string")) fail(file, `${field} must be an array of authored strings`);
  }
  const supportsRecruitment = Array.isArray(value.recruitment_support) && value.recruitment_support.length > 0;

  // Brief section 20 leaves the exact resource list Open, so construction cost
  // may only be keyed by an obvious placeholder -- the rule C9 set for
  // `resource_priorities`, applied to the other side of the economy.
  if (!value.construction_cost || typeof value.construction_cost !== "object" || Array.isArray(value.construction_cost)) fail(file, "construction_cost must be an object keyed by authored resource strings");
  else for (const [key, amount] of Object.entries(value.construction_cost)) {
    if (!key.startsWith("resource.open.")) fail(file, `construction_cost key ${key} invents a resource category; brief section 20 leaves the resource list Open, so keys stay under resource.open.`);
    if (!Number.isInteger(amount) || amount < 0) fail(file, `construction_cost.${key} must be a non-negative integer`);
  }

  if (!Array.isArray(value.production)) fail(file, "production must be an array, empty when the building makes nothing");
  else {
    const ruleIds = new Set();
    for (const [index, rule] of value.production.entries()) {
      const where = `production[${index}]`;
      if (typeof rule?.id !== "string" || rule.id.trim() === "") fail(file, `${where}.id must be a non-empty string`);
      else if (ruleIds.has(rule.id)) fail(file, `${where}.id ${rule.id} is used twice in this record`);
      else ruleIds.add(rule.id);
      for (const field of ["amount", "interval_hours", "minimum_tier"]) {
        if (!Number.isInteger(rule?.[field]) || rule[field] < 0) fail(file, `${where}.${field} must be a non-negative integer`);
      }
      if (Number.isInteger(rule?.minimum_tier) && topTier > 0 && rule.minimum_tier > topTier) fail(file, `${where}.minimum_tier ${rule.minimum_tier} is above this record's top tier ${topTier}`);
      if (typeof rule?.output_key !== "string") fail(file, `${where}.output_key must be a string`);

      if (rule?.cost !== undefined) {
        if (rule.cost === null || typeof rule.cost !== "object" || Array.isArray(rule.cost)) fail(file, `${where}.cost must be an object of resource key to count`);
        else for (const [key, count] of Object.entries(rule.cost)) {
          if (!key.startsWith("resource.open.")) fail(file, `${where}.cost key ${key} invents a resource category; brief section 20 leaves the resource list Open, so keys stay under resource.open.`);
          if (!Number.isInteger(count) || count < 0) fail(file, `${where}.cost.${key} must be a non-negative integer`);
        }
      }

      const output = rule?.output;
      const isSingleKeyObject = output !== null && typeof output === "object" && !Array.isArray(output) && Object.keys(output).length === 1;
      const isHumanRole = isSingleKeyObject && Object.keys(output)[0] === "human_role";
      const isMachine = isSingleKeyObject && Object.keys(output)[0] === "machine";
      // C14: what `output_key` may name depends on what the rule makes. A
      // machine rule names the `machine.<...>` record it builds -- S13's
      // `produce_machine` looks the rule's `output_key` up in
      // `MachineDefinitions` and refuses a rule whose record it cannot find or
      // whose family disagrees -- so for that one output the key is a stable ID
      // reference, resolved at the bottom of this file against the records
      // content/machines/ registers. Every other output_key is still an open
      // resource string, and inventing a resource category is still refused.
      if (typeof rule?.output_key === "string") {
        if (isMachine) {
          if (!rule.output_key.startsWith("machine.")) fail(file, `${where}.output_key ${rule.output_key} must name the machine.<...> record this rule builds; ExpeditionState::produce_machine resolves it in MachineDefinitions (C14, S13)`);
          else reference(rule.output_key, file, `${where}.output_key`);
        } else if (rule.output_key !== "" && !rule.output_key.startsWith("resource.open.")) {
          fail(file, `${where}.output_key ${rule.output_key} invents a resource category; brief section 20 leaves the resource list Open, so keys stay under resource.open.`);
        }
      }
      if (typeof output === "string") {
        if (output === "machine") fail(file, `${where}.output "machine" names no family; since S13 a machine rule is { "machine": { "family": "<one of ${[...machineFamilies].join(", ")}>" } }`);
        else if (!simpleProductionOutputs.has(output)) fail(file, `${where}.output ${output} is not a ProductionOutput: ${[...simpleProductionOutputs].join(", ")}, { "machine": { "family": "..." } }, or { "human_role": { "role": "..." } }`);
      } else if (isMachine) {
        const family = output.machine?.family;
        if (typeof family !== "string" || !machineFamilies.has(family)) fail(file, `${where}.output.machine.family ${JSON.stringify(family)} is not one of brief section 5.4's families: ${[...machineFamilies].join(", ")}`);
      } else if (isHumanRole) {
        if (typeof output.human_role?.role !== "string" || output.human_role.role.trim() === "") fail(file, `${where}.output.human_role.role must name the role, as a non-empty string`);
        // Brief section 5.6, and section 20's rejected list. Michael's faction
        // grows by recruitment, migration, relationships, rescue and factional
        // change -- never because a timer completed. For the other five a
        // human role may be *supported*, never *produced*: a non-zero interval
        // is a manufacturing timer whoever owns it, and a record that claims
        // the support without describing it is refused rather than assumed.
        if (buildsForMichael) fail(file, `${where} produces a human role on a record compatible with concept key michael; Michael's buildings make machines, capacity and services, never people (brief section 5.6, brief section 20)`);
        else {
          if (rule.interval_hours !== 0) fail(file, `${where} produces a human role every ${rule.interval_hours} hours; a human-role rule must have interval_hours 0, because it is recruitment support and never a manufacturing timer (brief section 20)`);
          if (!supportsRecruitment) fail(file, `${where} produces a human role on a record whose recruitment_support is empty; the only reading under which the rule is legal is that this building helps recruitment happen, so the record must say what the support is`);
        }
      } else {
        fail(file, `${where}.output must be one of ${[...simpleProductionOutputs].join(", ")}, { "machine": { "family": "..." } }, or { "human_role": { "role": "..." } }`);
      }
    }
  }

  // Brief section 20, "capture versus destruction rules by building type", is
  // Open. S3 made `NeedsDecision` the default for both fields and refuses it at
  // load, so the decision lands on each authored record. This mirrors that
  // refusal, and also refuses the field being absent -- an omitted field
  // deserializes to the default, which is the same refusal one step later.
  if (value.capture_rules === undefined) fail(file, "capture_rules is missing; brief section 20 leaves capture versus destruction Open per building type, so every record chooses capturable or destroy_only explicitly");
  else if (value.capture_rules === "needs_decision") fail(file, "capture_rules must not be needs_decision; the decision belongs on this record (brief section 20)");
  else if (!new Set(["capturable", "destroy_only"]).has(value.capture_rules)) fail(file, `capture_rules ${JSON.stringify(value.capture_rules)} must be capturable or destroy_only`);
  if (value.ruin_state === undefined) fail(file, "ruin_state is missing; brief section 20 leaves capture versus destruction Open per building type, so every record chooses clears_completely or leaves_rubble explicitly");
  else if (value.ruin_state === "needs_decision") fail(file, "ruin_state must not be needs_decision; the decision belongs on this record (brief section 20)");
  else if (value.ruin_state === "clears_completely") { /* decided */ }
  else if (value.ruin_state !== null && typeof value.ruin_state === "object" && !Array.isArray(value.ruin_state) && Object.keys(value.ruin_state).length === 1 && Object.keys(value.ruin_state)[0] === "leaves_rubble") {
    const rubble = value.ruin_state.leaves_rubble?.footprint_cells;
    if (!Number.isInteger(rubble) || rubble < 0) fail(file, "ruin_state.leaves_rubble.footprint_cells must be a non-negative integer");
    else if (Number.isInteger(value.footprint_cells) && rubble > value.footprint_cells) fail(file, `ruin_state.leaves_rubble.footprint_cells ${rubble} is larger than the standing footprint ${value.footprint_cells}; a wreck does not grow`);
  } else {
    fail(file, `ruin_state ${JSON.stringify(value.ruin_state)} must be "clears_completely" or { "leaves_rubble": { "footprint_cells": <integer> } }`);
  }

  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "building metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "building metadata.notes must list what stays Provisional or Open");
}

// C14: machine records. S13 shipped `MachineDefinition` with brief section 18's
// "Animal automata and vehicles need" list field for field and said a C card
// must fill `content/machines/`; this block is the content half of that
// contract, and `every_authored_machine_record_loads` in
// `godot-rust/src/strategy/production.rs` is what holds the directory equal to
// `MachineDefinitions`. C10's building block above is the shape this copies.
//
// Two records, not eight: one machine per family would be a catalogue invented
// here, and the brief's section 5.4 families are candidates, not a roster. The
// two that exist are the two the rest of the tree already needs -- the dog a
// tier-two machine shop makes, and the hauler that shows what a road-bound
// machine's record looks like.
//
// The families below are `MachineFamily::ALL`'s serde spellings, the same eight
// the buildings block already mirrors for a machine rule's family. The route
// vocabulary is the authored `travelMode` set from
// `content/world/*.world_cell.json`, which the portals block below checks
// against this same const, so the two readers of that vocabulary cannot drift.
const travelModes = new Set(["on_foot", "safe_road", "jungle_edge"]);
// Brief section 20 leaves the island's geometry and the resource list Open, so
// every one of these is checked the inverted way C9 checks `displayName` and
// C10 checks `height_class`: the validator requires that the decision has NOT
// been made. A footprint written as 3 here would read as a number somebody
// chose, and nobody has.
const openMachineDimensions = ["operational_footprint_cells", "navigation_width_cells", "turning_clearance_cells", "maximum_slope", "wreck_footprint_cells", "salvage_value"];
const openResourcePlaceholder = "resource.open.needs_decision";
let machineCount = 0;
for (const { file, value } of await readJsonDirectory("machines")) {
  machineCount += 1;
  const stem = file.split("/").pop().replace(/\.json$/, "");
  if (typeof value.id !== "string" || !value.id.startsWith("machine.")) fail(file, `id ${value.id} must be machine.<slug>`);
  else if (value.id !== `machine.${stem}`) fail(file, `id ${value.id} must be named after its file, machine.${stem}; the Rust equality test reads the directory by filename`);
  requireString(value, "function", file);

  // The tie between the authored asset and the closed family list. A ninth
  // family is refused here and by `MachineFamily`'s serde in Rust, and a rule
  // whose family disagrees with the record it names does not produce at all --
  // ProductionError::FamilyMismatch.
  if (!machineFamilies.has(value.family)) fail(file, `family ${JSON.stringify(value.family)} is not one of brief section 5.4's eight families: ${[...machineFamilies].join(", ")}`);

  // Every dimension stays Open, and `open_dimensions` says so decision by
  // decision rather than in one blanket note.
  for (const field of ["operational_footprint_cells", "navigation_width_cells", "turning_clearance_cells", "wreck_footprint_cells"]) {
    if (!Number.isInteger(value[field])) fail(file, `${field} must be an integer count of cells`);
    else if (value[field] !== 0) fail(file, `${field} is ${value[field]}, which reads as a decided dimension; brief section 20 leaves exact dimensions Open, so it stays 0 and open_dimensions.${field} names the decision`);
  }
  if (!Number.isInteger(value.maximum_slope) || value.maximum_slope < 0 || value.maximum_slope > 255) fail(file, "maximum_slope must be an integer from 0 through 255 (a u8 downstream)");
  else if (value.maximum_slope !== 0) fail(file, `maximum_slope is ${value.maximum_slope}, which reads as a decided limit; the world graph carries no slope to compare one against, so it stays 0 and open_dimensions.maximum_slope names the decision`);
  if (!value.salvage_value || typeof value.salvage_value !== "object" || Array.isArray(value.salvage_value)) fail(file, "salvage_value must be an object keyed by authored resource strings");
  else if (JSON.stringify(value.salvage_value) !== JSON.stringify({ [openResourcePlaceholder]: 0 })) fail(file, `salvage_value must stay the single placeholder { "${openResourcePlaceholder}": 0 }; brief section 20 leaves the resource list Open, so a wreck is worth nothing this file may name`);
  if (!value.open_dimensions || typeof value.open_dimensions !== "object" || Array.isArray(value.open_dimensions)) fail(file, `open_dimensions must be an object naming the decision behind each of ${openMachineDimensions.join(", ")}`);
  else {
    for (const field of openMachineDimensions) {
      const note = value.open_dimensions[field];
      if (typeof note !== "string" || !note.includes("needs decision")) fail(file, `open_dimensions.${field} must be a placeholder containing "needs decision"; brief section 20 leaves exact dimensions Open and every one of them says so by name`);
    }
    for (const field of Object.keys(value.open_dimensions)) {
      if (!openMachineDimensions.includes(field)) fail(file, `open_dimensions.${field} is not one of the Open dimensions: ${openMachineDimensions.join(", ")}`);
    }
  }

  // Brief section 18's "valid route types", against content's own vocabulary.
  if (!Array.isArray(value.valid_route_types) || value.valid_route_types.length === 0) fail(file, `valid_route_types must name at least one authored travel mode: ${[...travelModes].join(", ")}`);
  else {
    const seen = new Set();
    for (const [index, mode] of value.valid_route_types.entries()) {
      if (!travelModes.has(mode)) fail(file, `valid_route_types[${index}] ${JSON.stringify(mode)} is not an authored travelMode; the vocabulary is content/world/*.world_cell.json's portals: ${[...travelModes].join(", ")}`);
      else if (seen.has(mode)) fail(file, `valid_route_types[${index}] ${mode} is listed twice`);
      else seen.add(mode);
    }
  }

  // "Bridge requirements": what reinforces a crossing is nobody's decision yet
  // (brief section 5.11 puts reinforced bridges in the terrain signature and
  // stops there), so a flag stays an obvious placeholder. An empty list is a
  // real answer: a small automaton asks nothing of a bridge.
  if (!Array.isArray(value.bridge_requirements)) fail(file, "bridge_requirements must be an array, empty when the machine asks nothing of a crossing");
  else for (const [index, flag] of value.bridge_requirements.entries()) {
    if (typeof flag !== "string" || !flag.startsWith("requirement.open.")) fail(file, `bridge_requirements[${index}] ${JSON.stringify(flag)} must be a requirement.open.<...> placeholder; what a crossing must satisfy is not decided`);
  }

  // Crew, fuel and water. The counts are real -- section 5.10 is about the crew
  // number being non-zero at all -- but the keys fuel and water are counted in
  // stay under resource.open., because section 20 leaves the resource list Open.
  for (const field of ["crew_or_handler_requirement", "fuel_requirement", "water_requirement"]) {
    if (!Number.isInteger(value[field]) || value[field] < 0) fail(file, `${field} must be a non-negative integer`);
  }
  for (const field of ["fuel_resource_key", "water_resource_key"]) {
    if (typeof value[field] !== "string" || !value[field].startsWith("resource.open.")) fail(file, `${field} ${JSON.stringify(value[field])} invents a resource category; brief section 20 leaves the resource list Open, so keys stay under resource.open.`);
  }

  // "Repair sockets" and "local and offscreen representations": section 18's
  // accepted list requires machines to retain future-ready sockets and
  // metadata, so the sockets are authored now; the representations are prose
  // until there is a materialiser to consume them, and they say so.
  if (!Array.isArray(value.repair_sockets) || value.repair_sockets.length === 0) fail(file, "repair_sockets must name at least one socket a repair attaches to (brief section 18)");
  else {
    const socketIds = new Set();
    for (const [index, socket] of value.repair_sockets.entries()) {
      if (typeof socket !== "string" || !socket.startsWith("socket.")) fail(file, `repair_sockets[${index}] ${JSON.stringify(socket)} must be a socket.<...> string`);
      else if (socketIds.has(socket)) fail(file, `repair_sockets[${index}] ${socket} is listed twice`);
      else socketIds.add(socket);
    }
  }
  requireString(value, "local_and_offscreen_representations", file);
  if (typeof value.local_and_offscreen_representations === "string" && !value.local_and_offscreen_representations.includes("needs decision")) fail(file, "local_and_offscreen_representations must stay placeholder prose containing \"needs decision\" until a materialiser and an aggregate-force weight consume it");

  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "machine metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "machine metadata.notes must list what stays Provisional or Open");
}

// C6: relationship scene records. S12 left `RecruitmentState.milestone_rules` a
// data table with nothing in it; a scene record is the row. The record owns the
// rule, `MilestoneRule::from_authored` in
// `godot-rust/src/strategy/recruitment.rs` is the one translation, and
// `every_authored_scene_rule_loads` holds the two equal. This block refuses the
// content before Rust ever sees it, and makes the checks Rust has no field to
// make: which women exist, and what the base tree is allowed to depict.
//
// The stage and inclination vocabularies below are the serde spellings of
// `RecruitmentStage` and `Inclination` -- the variant names verbatim, since
// neither enum carries a rename attribute. `RecruitmentStage::as_str`'s
// lowercase words are the bridge's vocabulary, not serde's, and are not what a
// record writes. Rust's test is what keeps these two lists honest.
const recruitmentStages = ["Unaware", "Aware", "Interested", "Contact", "Committed", "Joined", "Integrated"];
const inclinations = ["AwarenessOfMichael", "AttractionToMichael", "RomanticInterest", "TrustInMichael", "IdeologicalAlignment", "DissatisfactionWithOrigin", "Ambition", "PerceivedSafety", "PerceivedOpportunity", "FearOfRetaliation"];
// Brief section 20 leaves the other two women Open. Only these two exist, and a
// record naming a third is authoring a woman nobody has designed.
const establishedWomenIds = ["character.heroine.betty", "character.heroine.ayla"];
// The base game is fade-to-black, full stop. C8's presentation pack is a
// separate artifact that overrides this field; nothing in this repository may
// carry another level, so this is an equality and not a set membership.
const baseTreePresentationLevel = "fade_to_black";
const sceneBeatKinds = new Set(["conversation", "action", "choice", "fade"]);
let relationshipSceneCount = 0;
let sceneRulesRestingOnTrust = 0;
const grantedMilestoneIds = new Set();
for (const { file, value } of await readJsonDirectory("relationships")) {
  relationshipSceneCount += 1;
  requireString(value, "displayName", file);
  if (!establishedWomenIds.includes(value.womanId)) {
    fail(file, `womanId ${value.womanId} is not one of the two women who exist (${establishedWomenIds.join(", ")}); the other two are Open, brief section 20`);
  } else if (!ids.has(value.womanId)) {
    fail(file, `womanId references missing stable ID ${value.womanId}`);
  }
  const womanKey = typeof value.womanId === "string" ? value.womanId.split(".").pop() : null;
  if (typeof value.id !== "string" || !value.id.startsWith(`scene.${womanKey}.`)) fail(file, `id ${value.id} must be scene.${womanKey}.<slug>`);
  registerId(value.grantsMilestoneId, file);
  if (typeof value.grantsMilestoneId !== "string" || !value.grantsMilestoneId.startsWith(`milestone.${womanKey}.`)) fail(file, `grantsMilestoneId ${value.grantsMilestoneId} must be milestone.${womanKey}.<slug>`);
  else if (grantedMilestoneIds.has(value.grantsMilestoneId)) fail(file, `a second scene grants ${value.grantsMilestoneId}; one milestone, one scene`);
  else grantedMilestoneIds.add(value.grantsMilestoneId);
  if (value.presentationLevel !== baseTreePresentationLevel) fail(file, `presentationLevel must be ${baseTreePresentationLevel}; the base tree carries no other level and the adult pack is a separate artifact (C8)`);
  if (!new Set(["placeholder", "final"]).has(value.proseStatus)) fail(file, "proseStatus must be placeholder or final, so stand-in prose says that it is");
  const rule = value.rule;
  if (!rule || typeof rule !== "object" || Array.isArray(rule)) fail(file, "rule must be an object mirroring MilestoneRule");
  else {
    if (!recruitmentStages.includes(rule.advancesTo)) fail(file, `rule.advancesTo ${rule.advancesTo} is not a RecruitmentStage: ${recruitmentStages.join(", ")}`);
    if (!recruitmentStages.includes(rule.requiresStageAtLeast)) fail(file, `rule.requiresStageAtLeast ${rule.requiresStageAtLeast} is not a RecruitmentStage: ${recruitmentStages.join(", ")}`);
    if (recruitmentStages.indexOf(rule.advancesTo) <= recruitmentStages.indexOf(rule.requiresStageAtLeast)) fail(file, "rule.advancesTo must be further along the ladder than rule.requiresStageAtLeast, or the scene can never move her");
    for (const field of ["requiresAtLeast", "requiresAtMost"]) {
      const thresholds = rule[field];
      if (!thresholds || typeof thresholds !== "object" || Array.isArray(thresholds)) fail(file, `rule.${field} must be an object keyed by Inclination names`);
      else for (const [inclination, threshold] of Object.entries(thresholds)) {
        if (!inclinations.includes(inclination)) fail(file, `rule.${field} key ${inclination} is not an Inclination: ${inclinations.join(", ")}`);
        if (!Number.isInteger(threshold) || threshold < 0 || threshold > 100) fail(file, `rule.${field}.${inclination} must be an integer disposition from 0 through 100`);
      }
    }
    if (Number.isInteger(rule.requiresAtLeast?.TrustInMichael)) sceneRulesRestingOnTrust += 1;
    for (const field of ["blockedByConditions", "clearsConditions"]) {
      if (!Array.isArray(rule[field])) fail(file, `rule.${field} must be an array of authored condition IDs`);
      else if (rule[field].some(condition => typeof condition !== "string" || !condition.startsWith("condition."))) fail(file, `rule.${field} must contain condition.<...> stable IDs`);
    }
    // A10: optional, and when present it is one of the seven authored letters
    // and nothing else -- no number, no eighth rung, no lowercase. The scene
    // that carries it is what opens a woman's higher-ranked commands, so a
    // misspelling here would silently leave a skill locked for the campaign.
    if (rule.raisesBondRankTo !== undefined && !bondRanks.includes(rule.raisesBondRankTo)) fail(file, `rule.raisesBondRankTo ${rule.raisesBondRankTo} must be one of the seven authored bond ranks: ${bondRanks.join(", ")}`);
  }
  if (!Array.isArray(value.beats) || value.beats.length === 0) fail(file, "beats must contain at least one authored beat");
  else {
    const beatIds = new Set();
    for (const [index, beat] of value.beats.entries()) {
      if (typeof beat?.id !== "string" || !beat.id.startsWith("beat.")) fail(file, `beats[${index}].id must be a beat.<...> stable ID`);
      else if (beatIds.has(beat.id)) fail(file, `beats[${index}].id ${beat.id} is duplicated`);
      else beatIds.add(beat.id);
      if (!sceneBeatKinds.has(beat?.kind)) fail(file, `beats[${index}].kind ${beat?.kind} must be one of ${[...sceneBeatKinds].join(", ")}`);
      // A fade is where the base game stops, so nothing may be authored after
      // it. C8's pack replaces the fade beat; it does not append to the scene.
      if (beat?.kind === "fade" && index !== value.beats.length - 1) fail(file, `beats[${index}] is a fade beat with beats after it; a fade closes the scene`);
      if (typeof beat?.text !== "string" || beat.text.trim().length < 20) fail(file, `beats[${index}].text must carry the authored line`);
    }
  }
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "relationship metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "relationship metadata.notes must record what stays placeholder or Open");
}
// Attraction creates openings, not allegiance (brief section 5.6, and the test
// S12 already carries). At least one authored scene must put a floor under
// TrustInMichael, or nothing in the content tree is holding that line.
if (relationshipSceneCount > 0 && sceneRulesRestingOnTrust === 0) failures.push("content/relationships/: no authored scene puts a floor under TrustInMichael, so nothing in the content tree holds the line that attraction creates openings rather than allegiance (brief section 5.6)");

// C15: the creature registry's ID is the one that survives. `Habitats`,
// `WorldClock`'s spawn rules and every habitat roster spell these creatures
// `enemy.raptor.razorbeak`; the seven authored records used to spell the same
// creatures `enemy.raptor.razorbeak.prototype`, so a roster entry could never
// resolve against content. Content owns the ID and it is the registry's
// spelling, which is why the suffix is refused here rather than tolerated.
//
// `derivesFrom` is resolved before any of the checks below run, so a variant
// record is validated as the complete creature Godot will receive from
// `game/generated/content_bundle.json` -- the same resolution, from the same
// module, that the bundle builder applies.
const openEnemyFields = ["level", "stats.vitality", "stats.guard", "stats.initiative", "skillIds"];
const enemyFieldValue = (record, path) => path.split(".").reduce((value, key) => (value === undefined || value === null ? undefined : value[key]), record);
const authoredEnemies = await readJsonDirectory("enemies");
for (const { file, value } of authoredEnemies) {
  if (typeof value.id !== "string" || !value.id.startsWith("enemy.")) fail(file, `id ${JSON.stringify(value.id)} must use the enemy. prefix`);
  else if (value.id.endsWith(".prototype")) fail(file, `id ${value.id} carries a .prototype suffix the creature registry does not use; Habitats and WorldClock spell this creature ${value.id.slice(0, -".prototype".length)}, and a roster entry cannot resolve against a suffix content invented`);
}
const { records: resolvedEnemies, problems: enemyDerivationProblems } = resolveDerivedRecords(authoredEnemies);
for (const problem of enemyDerivationProblems) fail(problem.file, problem.message);
let enemyCount = 0;
for (const { file, value } of resolvedEnemies) {
  enemyCount += 1;
  requireString(value, "displayName", file);

  // The placeholder convention, inverted the way C14 checks a machine's Open
  // dimensions: a number nobody has decided stays 0 and `open` names the
  // decision by field, and -- the half that bites -- a field left at 0 or a
  // skill list left empty MUST be named there, so a hollow creature record
  // cannot pass as a finished one. `enemy.boar.thunderback` is the record this
  // exists for: the habitat fixture puts it on the river jungle's roster and
  // carries no stat, level or skill for it anywhere.
  for (const path of ["level", "stats.vitality", "stats.guard", "stats.initiative"]) {
    const stat = enemyFieldValue(value, path);
    if (!Number.isInteger(stat) || stat < 0) fail(file, `${path} must be a non-negative integer`);
  }
  if (!Array.isArray(value.skillIds)) fail(file, "skillIds must be an array");
  if (value.open !== undefined) {
    if (typeof value.open !== "object" || value.open === null || Array.isArray(value.open)) fail(file, `open must be an object naming the decision behind each undecided field: ${openEnemyFields.join(", ")}`);
    else for (const [field, note] of Object.entries(value.open)) {
      if (!openEnemyFields.includes(field)) fail(file, `open.${field} is not one of the fields a creature record may leave undecided: ${openEnemyFields.join(", ")}`);
      else if (typeof note !== "string" || !note.includes("needs decision")) fail(file, `open.${field} must be a placeholder containing "needs decision" that says what is undecided and why no source answers it`);
      else if (field === "skillIds") {
        if (!Array.isArray(value.skillIds) || value.skillIds.length !== 0) fail(file, "open.skillIds says this creature's skills are undecided, so skillIds must be the empty list rather than a guess");
      } else if (enemyFieldValue(value, field) !== 0) fail(file, `open.${field} says this number is undecided, so ${field} must stay 0 rather than read as a decided value`);
    }
  }
  for (const field of openEnemyFields) {
    const empty = field === "skillIds" ? Array.isArray(value.skillIds) && value.skillIds.length === 0 : enemyFieldValue(value, field) === 0;
    if (empty && typeof value.open?.[field] !== "string") fail(file, `${field} is empty, which makes this a placeholder record; open.${field} must name the decision it is waiting on (brief section 20's convention, as C14 applies it to machine dimensions)`);
  }
  if (value.worldPresence?.penAllowed !== false) fail(file, "island monsters may not be designed as pen exhibits");
  if (value.worldPresence?.packSize !== 1) fail(file, "prototype enemies must be tuned as individual threats");
  for (const [index, id] of (value.skillIds ?? []).entries()) {
    reference(id, file, `skillIds[${index}]`);
    if (typeof value.text?.intentTells?.[id] !== "string" || value.text.intentTells[id].length < 40) fail(file, `${id} must have an explicit textual intent tell`);
    const action = value.actions?.[id];
    if (!action || typeof action !== "object") {
      fail(file, `actions must specify ${id}`);
      continue;
    }
    requireString(action, "displayName", file);
    requireString(action, "rule", file);
    if (!action.framing?.includes("safe frame")) fail(file, `${id} framing must state its safe-frame requirement`);
    if (id === "skill.enemy.razorbeak.guard_breaking_kick") {
      const opening = action.recoveryOpening;
      if (opening?.createdAfterResolution !== true || opening?.bonusRawDamageOnNextPartyHit !== 6) fail(file, "Guard-Breaking Kick must create the six-damage recovery opening after resolution");
      if (opening?.consumedBy !== "next_party_damage_action_against_razorbeak" || opening?.expires !== "start_of_razorbeaks_next_turn") fail(file, "Guard-Breaking Kick recovery opening requires explicit consume and expiry rules");
      if (opening?.stacks !== false) fail(file, "Guard-Breaking Kick recovery opening must not stack");
      for (const field of ["createdPose", "activeLoop", "consumedResponse", "expiredRecovery"]) {
        if (typeof opening?.[field] !== "string" || opening[field].length < 60) fail(file, `Guard-Breaking Kick recoveryOpening.${field} must be an explicit animation instruction`);
      }
      if (typeof value.text?.recoveryOpening !== "string" || value.text.recoveryOpening.length < 60) fail(file, "Guard-Breaking Kick must have a concrete recovery-opening description");
    }
    if (!Array.isArray(action.animationBeats) || action.animationBeats.length < 4) {
      fail(file, `${id} must declare at least four animation beats`);
      continue;
    }
    let previousAt = -1;
    for (const [beatIndex, beat] of action.animationBeats.entries()) {
      if (!Number.isInteger(beat?.atMs) || beat.atMs < previousAt) fail(file, `${id} animationBeats[${beatIndex}].atMs must be an ordered integer`);
      if (typeof beat?.pose !== "string" || beat.pose.length < 20) fail(file, `${id} animationBeats[${beatIndex}].pose must be an explicit action instruction`);
      previousAt = beat?.atMs ?? previousAt;
    }
  }
}

// C15: `content/habitats/`. B7 recorded why this directory did not exist -- a
// habitat is mostly its roster, and two rostered creatures had no content
// record while the seven that did carried a suffix the registry never used --
// and left `habitat.` in `intentionallyExternalPrefixes` as the honest
// stopgap. Both halves are closed above, so the twin is authored and
// `habitat.` is an ordinary registered ID like any other.
//
// The arrangement is `content/loot/`'s exactly: content owns the numbers,
// `Habitats::black_beach_vertical_slice` in `godot-rust/src/habitat.rs`
// carries them so the simulation never reads the disk mid-day, and
// `habitat::tests::fixture_matches_the_authored_habitats` holds the two equal
// field for field. The field names and the `rank` and `family` spellings below
// are `HabitatRecord`'s, `EncounterRank`'s and `CreatureFamily`'s verbatim --
// the Rust enums carry no serde rename, so C6's treatment of `RecruitmentStage`
// applies: the variant names are what a record writes, and the Rust test is
// what keeps these lists honest.
const encounterRanks = new Set(["Ordinary", "Elevated", "Apex"]);
const creatureFamilies = new Set(["Beast", "Undead", "Spectral", "Eldritch"]);
const habitatRegionIds = new Map();
const habitatTerritory = new Map();
let habitatCount = 0;
for (const { file, value } of await readJsonDirectory("habitats")) {
  habitatCount += 1;
  if (typeof value.id !== "string" || !value.id.startsWith("habitat.")) fail(file, `id ${JSON.stringify(value.id)} must use the habitat. prefix`);
  requireString(value, "display_name", file);

  // Every habitat owns a distinct region, because `WorldClock::resolve_midnight`
  // builds each spawn's instance ID from region and slot: two habitats sharing
  // a region would collide into one `HabitatState`.
  if (typeof value.region_id !== "string" || !value.region_id.startsWith("world.region.")) fail(file, `region_id ${JSON.stringify(value.region_id)} must be a world.region.<...> ID`);
  else if (habitatRegionIds.has(value.region_id)) fail(file, `region_id ${value.region_id} is already held by ${habitatRegionIds.get(value.region_id)}; Midnight Return derives instance IDs from region and slot, so two habitats sharing one region would collide into a single spawn record`);
  else habitatRegionIds.set(value.region_id, value.id);

  // The roster is the habitat. Every entry names a creature content declares,
  // so a habitat can no longer roster something nobody authored.
  if (!Array.isArray(value.roster) || value.roster.length === 0) fail(file, "roster must list every creature that can hold this habitat, baseline first");
  else {
    const rostered = new Set();
    for (const [index, entry] of value.roster.entries()) {
      if (typeof entry?.definition_id !== "string" || !entry.definition_id.startsWith("enemy.")) fail(file, `roster[${index}].definition_id ${JSON.stringify(entry?.definition_id)} must be an enemy.<...> creature ID`);
      else {
        reference(entry.definition_id, file, `roster[${index}].definition_id`);
        if (rostered.has(entry.definition_id)) fail(file, `roster[${index}] rosters ${entry.definition_id} twice`);
        else rostered.add(entry.definition_id);
      }
      if (!creatureFamilies.has(entry?.family)) fail(file, `roster[${index}].family ${JSON.stringify(entry?.family)} is not one of ${[...creatureFamilies].join(", ")}`);
      if (!Number.isInteger(entry?.available_from_day) || entry.available_from_day < 1) fail(file, `roster[${index}].available_from_day must be an integer campaign day of 1 or more`);
      if (typeof entry?.night_only !== "boolean") fail(file, `roster[${index}].night_only must be boolean`);
    }
    // A habitat whose whole roster is night-only has no daylight holder, and
    // `leader_definition_id` would advertise a creature that cannot be there.
    if (value.roster.every(entry => entry?.night_only === true)) fail(file, "roster leaves this habitat with no daylight holder at all; the first entry is the baseline answer to what normally lives here");
  }

  if (!encounterRanks.has(value.rank)) fail(file, `rank ${JSON.stringify(value.rank)} is not one of ${[...encounterRanks].join(", ")}`);
  for (const field of ["behavior_tags", "intent_suite", "territory_location_ids"]) {
    if (!Array.isArray(value[field]) || value[field].length === 0) fail(file, `${field} must be a non-empty array`);
  }
  for (const [index, tag] of (Array.isArray(value.behavior_tags) ? value.behavior_tags : []).entries()) {
    if (typeof tag !== "string" || tag.trim() === "") fail(file, `behavior_tags[${index}] must be a non-empty string`);
  }
  // The intent suite is what this habitat advertises its holder will open with.
  // Its IDs are enemy skills, which stay external for the reason the prefix
  // list below gives.
  for (const [index, id] of (Array.isArray(value.intent_suite) ? value.intent_suite : []).entries()) {
    if (typeof id !== "string" || !id.startsWith("skill.enemy.")) fail(file, `intent_suite[${index}] ${JSON.stringify(id)} must be a skill.enemy.<...> ID`);
    else reference(id, file, `intent_suite[${index}]`);
  }
  // Territory is authored world cells, and no two habitats may claim the same
  // one: `Habitats::habitat_for_location` returns the first match, so an
  // overlap would silently pick a winner.
  for (const [index, id] of (Array.isArray(value.territory_location_ids) ? value.territory_location_ids : []).entries()) {
    if (typeof id !== "string" || !id.startsWith("world.cell.")) fail(file, `territory_location_ids[${index}] ${JSON.stringify(id)} must be a world.cell.<...> ID`);
    else {
      reference(id, file, `territory_location_ids[${index}]`);
      if (habitatTerritory.has(id)) fail(file, `territory_location_ids[${index}] ${id} is already territory of ${habitatTerritory.get(id)}; a location has one holder, because habitat_for_location answers with the first habitat that claims it`);
      else habitatTerritory.set(id, value.id);
    }
  }

  if (typeof value.drop_table_id !== "string" || !value.drop_table_id.startsWith("loot.")) fail(file, `drop_table_id ${JSON.stringify(value.drop_table_id)} must be a loot.<...> table ID`);
  else reference(value.drop_table_id, file, "drop_table_id");
  if (typeof value.return_eligible !== "boolean") fail(file, "return_eligible must be boolean");
  // base_level is a u8 and daily_pressure an i8 downstream.
  if (!Number.isInteger(value.base_level) || value.base_level < 1 || value.base_level > 255) fail(file, "base_level must be an integer from 1 through 255");
  if (!Number.isInteger(value.daily_pressure) || value.daily_pressure < -128 || value.daily_pressure > 127) fail(file, "daily_pressure must be an integer from -128 through 127");

  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "habitat metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "habitat metadata.notes must say what this habitat's schedule means and where its field names come from");
}

for (const { file, value } of await readJsonDirectory("encounters")) {
  for (const [index, id] of (value.partyActorIds ?? []).entries()) reference(id, file, `partyActorIds[${index}]`);
  for (const [index, id] of (value.hostileActorIds ?? []).entries()) reference(id, file, `hostileActorIds[${index}]`);
  // A2: an encounter names real places. These two fields carried a third
  // location namespace that matched no world cell, which is exactly the drift
  // that resolving them against the registered stable IDs now makes impossible.
  reference(value.locationId, file, "locationId");
  reference(value.defeat?.returnLocationId, file, "defeat.returnLocationId");
  // C1: the same drift, one field over. `victory.lootTableId` named
  // `loot.razorbeak.prototype` while no loot record existed anywhere, and
  // nothing checked it, so the dangling ID shipped silently.
  reference(value.victory?.lootTableId, file, "victory.lootTableId");
  if (value.presentation?.inactivePartyMode !== "card_rail" || value.presentation?.activeActorMode !== "full_body_battle_plane") fail(file, "encounter must preserve the card-to-active combat contract");
}

// C5: `content/dungeons/`. The design bible's section 3.13 names the twelve
// spaces of the Tomb of Returning Names; this record lists exactly those
// twelve, says which world cell realises each one today, and gives each space
// two site-rule sets -- the owning faction's and the corrupted variant's. The
// selection between them is S9's `DungeonContext.owning_faction_id`, read by
// `select_site_rules` in `godot-rust/src/strategy/dungeon_content.rs`; this
// block refuses a record that reader could not honour.
//
// A7: a site rule is no longer opaque. C5 registered twenty-six
// `site_rule.<...>` IDs from the dungeon record itself and said plainly that
// what they *do* was A7's to decide; this block is that decision's content
// half. `content/site_rules/<slug>.json` is now the owner of every one of
// those IDs -- it registers them, and the dungeon record and the tomb cells
// reference them like any other stable ID.
//
// The effect vocabulary is closed and is `SiteRuleEffect` in
// `godot-rust/src/strategy/site_rule.rs`. Exactly two shapes are legal, and a
// record carrying neither is refused here in the same words the Rust loader
// refuses it in. There is no catch-all: a vocabulary that accepts anything is
// not a vocabulary, and a rule whose mechanics are Open says so in its own
// `effect` rather than defaulting to a mechanic nobody chose.
const siteRuleIdPrefix = "site_rule.";
const authoredSiteRuleIds = new Set();
let siteRuleCount = 0;
for (const { file, value } of await readJsonDirectory("site_rules")) {
  siteRuleCount += 1;
  if (typeof value.id !== "string" || !value.id.startsWith(siteRuleIdPrefix)) fail(file, `id ${value.id} must use the site_rule. prefix`);
  else authoredSiteRuleIds.add(value.id);
  requireString(value, "displayName", file);
  const effect = value.effect;
  if (!effect || typeof effect !== "object" || Array.isArray(effect)) {
    fail(file, "effect must be an object naming exactly one of the closed vocabulary's two shapes");
  } else {
    const keys = Object.keys(effect);
    if (keys.length !== 1 || !["guard_regen_per_round", "needs_decision"].includes(keys[0])) {
      fail(file, `effect must be exactly one of {"guard_regen_per_round": <positive integer>} or {"needs_decision": true}; found ${JSON.stringify(keys)}`);
    } else if (keys[0] === "guard_regen_per_round") {
      if (!Number.isInteger(effect.guard_regen_per_round) || effect.guard_regen_per_round <= 0) fail(file, "effect.guard_regen_per_round must be a positive integer; a rule that regenerates nothing should have said needs_decision");
    } else if (effect.needs_decision !== true) {
      fail(file, "effect.needs_decision must be true; false would claim the decision is made and then name no mechanic");
    }
  }
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "site rule metadata.notes must say what the rule does or that the decision is Open");
}

const dungeonSpaceIdPrefix = "dungeon.";
const selectedSiteRuleIds = new Set();
const dungeonSpacesByDungeon = new Map();
const dungeonRecordsById = new Map();
let dungeonSpaceCount = 0;
for (const { file, value } of await readJsonDirectory("dungeons")) {
  if (value.kind !== "dungeon") fail(file, "kind must be dungeon");
  if (!value.id?.startsWith(dungeonSpaceIdPrefix)) fail(file, `id ${value.id} must use the dungeon. prefix`);
  requireString(value, "displayName", file);
  // Concept keys, never proper names: brief section 4 leaves faction names
  // Open, and `factionConceptKeys` above is the one list of the six.
  for (const field of ["defaultOwnerConceptKey", "corruptedVariantConceptKey"]) {
    if (!factionConceptKeys.includes(value[field])) fail(file, `${field} ${value[field]} is not one of the six concept keys: ${factionConceptKeys.join(", ")}`);
  }
  if (value.defaultOwnerConceptKey === value.corruptedVariantConceptKey) fail(file, "defaultOwnerConceptKey and corruptedVariantConceptKey must differ, or an owner change can never change the rules");
  const spaces = new Map();
  if (!Array.isArray(value.spaces) || value.spaces.length === 0) fail(file, "spaces must list the dungeon's authored spaces");
  else for (const [index, space] of value.spaces.entries()) {
    dungeonSpaceCount += 1;
    const where = `spaces[${index}]`;
    registerId(space?.id, file);
    if (!space?.id?.startsWith(`${value.id}.space.`)) fail(file, `${where}.id ${space?.id} must be ${value.id}.space.<space>`);
    if (typeof space?.space !== "string" || !/^[a-z][a-z0-9_]*$/.test(space.space)) fail(file, `${where}.space must be a lower_snake_case slug`);
    else if (space.id !== `${value.id}.space.${space.space}`) fail(file, `${where}.id must end in its own space slug ${space.space}`);
    else if (spaces.has(space.space)) fail(file, `${where}.space ${space.space} is authored twice`);
    else spaces.set(space.space, space);
    for (const field of ["displayName", "purpose", "persistentProof", "note"]) requireString(space ?? {}, field, file);
    // Either a cell realises this space today, or the record says plainly
    // that none does. An absent key is neither, and would read as "unknown".
    if (space?.worldCellId === undefined) fail(file, `${where}.worldCellId must be a world cell ID or null`);
    else if (space.worldCellId !== null) reference(space.worldCellId, file, `${where}.worldCellId`);
    for (const field of ["siteRuleIds", "corruptedSiteRuleIds"]) {
      const rules = space?.[field];
      if (!Array.isArray(rules) || rules.length === 0) fail(file, `${where}.${field} must name at least one site rule`);
      else for (const [ruleIndex, ruleId] of rules.entries()) {
        if (typeof ruleId !== "string" || !ruleId.startsWith(siteRuleIdPrefix)) fail(file, `${where}.${field}[${ruleIndex}] must be a site_rule.<...> stable ID`);
        else {
          reference(ruleId, file, `${where}.${field}[${ruleIndex}]`);
          selectedSiteRuleIds.add(ruleId);
        }
      }
    }
    // The card's whole point: a different owner is a different dungeon. A
    // space whose corrupted set equals its default set would make the owner
    // change invisible, which is brief section 12's "palette swap".
    if (JSON.stringify(space?.siteRuleIds) === JSON.stringify(space?.corruptedSiteRuleIds)) fail(file, `${where}.corruptedSiteRuleIds must differ from siteRuleIds; a corrupted variant that selects the same rules is a palette swap (brief section 12)`);
  }
  dungeonSpacesByDungeon.set(value.id, spaces);
  dungeonRecordsById.set(value.id, value);
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "dungeon metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "dungeon metadata.notes must record which spaces are unrealised and what stays Open");
}

// Both directions, the way `content/skills/` and `battle::skill_rank` are held
// equal: an authored rule no dungeon selects is a noodle to nowhere, and a
// selected rule with no record is a mechanic nobody wrote. The Rust twin is
// `every_site_rule_the_tomb_selects_has_a_record_and_the_other_way_round`.
for (const ruleId of authoredSiteRuleIds) {
  if (!selectedSiteRuleIds.has(ruleId)) fail(resolve(repo, `content/site_rules`), `${ruleId} has a record that no dungeon space selects`);
}

const worldRecords = await readJsonDirectory("world");
for (const { file, value } of worldRecords) {
  if (value.kind === "world_region") {
    requireString(value, "displayName", file);
    requireString(value, "description", file);
    if (!Array.isArray(value.worldCellIds) || value.worldCellIds.length === 0) fail(file, "world region requires at least one worldCellId");
    else value.worldCellIds.forEach((id, index) => reference(id, file, `worldCellIds[${index}]`));
  } else if (value.kind === "world_cell") {
    requireString(value, "displayName", file);
    reference(value.regionId, file, "regionId");
    if (!value.visualShell || typeof value.visualShell !== "object") fail(file, "world cell requires a visualShell record");
    else {
      requireString(value.visualShell, "runtimePath", file);
      requireString(value.visualShell, "status", file);
      if (!value.visualShell.runtimePath.startsWith("res://assets/world/")) fail(file, "world cell visual shell must use an assets/world runtime path");
      if (!Array.isArray(value.visualShell.requiredFeatures) || value.visualShell.requiredFeatures.length < 4) fail(file, "world cell visual shell requires four or more literal features");
      if (!Array.isArray(value.visualShell.prohibitedFeatures) || value.visualShell.prohibitedFeatures.length < 3) fail(file, "world cell visual shell requires three or more literal prohibitions");
    }
    for (const field of ["collisionScene", "navigationScene", "cameraRailId"]) requireString(value, field, file);
    if (!value.collisionScene?.startsWith("res://scenes/world/") || !value.navigationScene?.startsWith("res://scenes/world/")) fail(file, "world cell collision and navigation must be separate world scenes");
    if (!Array.isArray(value.entryAnchors) || value.entryAnchors.length < 2) fail(file, "world cell requires two or more entry anchors");
    else for (const [index, anchor] of value.entryAnchors.entries()) {
      requireString(anchor, "id", file);
      requireString(anchor, "role", file);
      if (!Array.isArray(anchor.positionMetres) || anchor.positionMetres.length !== 3 || anchor.positionMetres.some(component => typeof component !== "number")) fail(file, `entryAnchors[${index}].positionMetres must be three numeric metres`);
      if (!Number.isInteger(anchor.facingDegrees) || anchor.facingDegrees < 0 || anchor.facingDegrees >= 360) fail(file, `entryAnchors[${index}].facingDegrees must be 0 through 359`);
    }
    const entryAnchorIds = new Set((value.entryAnchors ?? []).map(anchor => anchor?.id));
    if (!Array.isArray(value.interactionAnchors) || value.interactionAnchors.length < 2) fail(file, "world cell requires two or more interaction anchors");
    else for (const [index, anchor] of value.interactionAnchors.entries()) {
      requireString(anchor, "id", file);
      requireString(anchor, "role", file);
      requireString(anchor, "actionId", file);
    }
    // C13: a cell may declare the anchors the simulation can act on here --
    // the content twin of CellDefinition.anchors in godot-rust/src/geography.rs,
    // which Geography::from_authored registers. An anchor's own id and any
    // discovery it grants are stable IDs, so a portal's requiredDiscoveryId can
    // resolve to something content actually declares rather than to a string
    // only the Rust fixture knows about. A discovery is therefore never
    // external: it is granted here or it does not exist.
    const anchorKinds = new Set(["salvage", "loot_cache", "infirmary", "workshop", "map_table", "inspect"]);
    if (value.anchors !== undefined && !Array.isArray(value.anchors)) fail(file, "anchors must be an array when present");
    for (const [index, anchor] of (Array.isArray(value.anchors) ? value.anchors : []).entries()) {
      registerId(anchor.id, file);
      if (!anchor.id?.startsWith("anchor.")) fail(file, `anchors[${index}].id must use the anchor. prefix`);
      if (!anchorKinds.has(anchor.kind)) fail(file, `anchors[${index}].kind is unsupported`);
      if (anchor.oncePerDay !== undefined && typeof anchor.oncePerDay !== "boolean") fail(file, `anchors[${index}].oncePerDay must be boolean`);
      const numeric = anchor.kind === "salvage" ? ["rations", "coin"] : anchor.kind === "loot_cache" ? ["rations", "medicine", "coin"] : [];
      for (const field of numeric) if (!Number.isInteger(anchor[field]) || anchor[field] < 0) fail(file, `anchors[${index}].${field} must be a non-negative integer for kind ${anchor.kind}`);
      if (anchor.requiresDiscoveryId !== undefined) reference(anchor.requiresDiscoveryId, file, `anchors[${index}].requiresDiscoveryId`);
      if (anchor.grantsDiscoveryId !== undefined) {
        registerId(anchor.grantsDiscoveryId, file);
        if (!anchor.grantsDiscoveryId.startsWith("discovery.")) fail(file, `anchors[${index}].grantsDiscoveryId must use the discovery. prefix`);
      }
    }
    if (!Array.isArray(value.portals) || value.portals.length === 0) fail(file, "world cell requires one or more explicit portals");
    else for (const [index, portal] of value.portals.entries()) {
      for (const field of ["id", "fromAnchorId", "targetAnchorId", "travelMode", "returnRule"]) requireString(portal, field, file);
      reference(portal.targetCellId, file, `portals[${index}].targetCellId`);
      if (!entryAnchorIds.has(portal.fromAnchorId)) fail(file, `portals[${index}].fromAnchorId must name an entry anchor in this cell`);
      if (!travelModes.has(portal.travelMode)) fail(file, `portals[${index}].travelMode is unsupported; the authored vocabulary is ${[...travelModes].join(", ")}`);
      if (!new Set(["always", "allowed_while_no_pending_encounter"]).has(portal.returnRule)) fail(file, `portals[${index}].returnRule is unsupported`);
      // C2: travel costs live with the portal that charges them. They are
      // optional so a connection can be authored before it is priced, but a
      // present field is type-checked: `PortalDefinition` in
      // godot-rust/src/geography.rs takes u32, u32 and u8, so a negative or
      // fractional cost is not representable downstream.
      for (const field of ["timeCostMinutes", "supplyCost", "riskLevel"]) {
        if (portal[field] === undefined) continue;
        if (!Number.isInteger(portal[field]) || portal[field] < 0) fail(file, `portals[${index}].${field} must be a non-negative integer`);
      }
      if (portal.requiredDiscoveryId !== undefined) reference(portal.requiredDiscoveryId, file, `portals[${index}].requiredDiscoveryId`);
    }
    // B7: a battle entry's `status` says what, if anything, actually fights
    // here, and it is now a closed set. `future_spawn_socket` is a reserved
    // place with nothing in it; `vertical_slice_encounter` is content's own
    // one-time fight; `habitat_holder` is the socket where whatever the named
    // habitat currently holds answers, through `begin_encounter`'s existing
    // habitat path rather than a second spawn rule. The last two are what make
    // a cell encounter-eligible in godot-rust/src/geography.rs, so a status
    // typo used to be a silently dead room.
    const battleEntryStatuses = new Set(["future_spawn_socket", "vertical_slice_encounter", "habitat_holder"]);
    if (!Array.isArray(value.battleEntries) || value.battleEntries.length === 0) fail(file, "world cell requires one or more battle entries");
    else for (const [index, entry] of value.battleEntries.entries()) {
      requireString(entry, "id", file);
      reference(entry.encounterId, file, `battleEntries[${index}].encounterId`);
      for (const field of ["returnAnchorId", "safeRetreatAnchorId"]) requireString(entry, field, file);
      if (!entryAnchorIds.has(entry.returnAnchorId)) fail(file, `battleEntries[${index}].returnAnchorId must name an entry anchor in this cell`);
      if (!entryAnchorIds.has(entry.safeRetreatAnchorId)) fail(file, `battleEntries[${index}].safeRetreatAnchorId must name an entry anchor in this cell`);
      if (!battleEntryStatuses.has(entry.status)) fail(file, `battleEntries[${index}].status must be one of ${[...battleEntryStatuses].join(", ")}`);
      if (entry.habitatId !== undefined) {
        reference(entry.habitatId, file, `battleEntries[${index}].habitatId`);
        if (typeof entry.habitatId === "string" && !entry.habitatId.startsWith("habitat.")) fail(file, `battleEntries[${index}].habitatId must use the habitat. prefix`);
        if (entry.status === "future_spawn_socket") fail(file, `battleEntries[${index}] is bound to ${entry.habitatId} and is therefore not a future socket; use status habitat_holder or vertical_slice_encounter`);
      } else if (entry.status === "habitat_holder") {
        fail(file, `battleEntries[${index}].status is habitat_holder but no habitatId names the habitat that holds it`);
      }
    }
    if (!Array.isArray(value.explorationActions) || value.explorationActions.length < 2) fail(file, "world cell requires two or more real exploration actions");
    else for (const [index, action] of value.explorationActions.entries()) {
      requireString(action, "id", file);
      requireString(action, "type", file);
      if (typeof action.once !== "boolean") fail(file, `explorationActions[${index}].once must be boolean`);
      if (!new Set(["inspect", "travel", "recover", "route_choice", "locked_departure"]).has(action.type)) fail(file, `explorationActions[${index}].type is unsupported`);
      if (action.type === "travel") {
        requireString(action, "portalId", file);
        if (!(value.portals ?? []).some(portal => portal.id === action.portalId)) fail(file, `explorationActions[${index}].portalId must name a portal in this cell`);
      }
      if (action.type === "route_choice") {
        if (!Array.isArray(action.choices) || action.choices.length < 2) fail(file, `explorationActions[${index}].choices must offer two or more portals`);
        else for (const portalId of action.choices) if (!(value.portals ?? []).some(portal => portal.id === portalId)) fail(file, `explorationActions[${index}].choices references a portal outside this cell`);
      }
    }
    if (!Array.isArray(value.readableDescriptions) || value.readableDescriptions.length < value.admission?.descriptionCountMinimum) fail(file, "world cell must include every required readable description");
    else for (const [index, description] of value.readableDescriptions.entries()) {
      requireString(description, "id", file);
      // A readable description's id is the observation the simulation records
      // when the party inspects here (`CellDefinition.observation_ids` in
      // godot-rust/src/geography.rs), and observations gate anchors and
      // portals. They were never registered as stable IDs, so nothing in
      // content could legally reference one; C13's authored gates made that
      // visible. Registered here, with the prefix the Rust side assumes.
      registerId(description.id, file);
      if (!description.id?.startsWith("observation.")) fail(file, `readableDescriptions[${index}].id must use the observation. prefix`);
      if (typeof description.text !== "string" || description.text.length < 90) fail(file, `readableDescriptions[${index}].text must be at least 90 characters`);
    }
    // C5: a cell may declare which dungeon space it realises. The dungeon
    // record above is the owner of the space table and of both rule sets; this
    // block is the mirror, and every field of it is checked against the record
    // rather than believed. `siteRuleIds` here is the space's default-owner
    // list and nothing else -- A7 reads a cell's site rules from this one
    // field, so a second list, or a drifted copy of this one, would be two
    // answers to one question.
    const dungeonContext = value.dungeonContext;
    if (dungeonContext !== undefined) {
      if (!dungeonContext || typeof dungeonContext !== "object" || Array.isArray(dungeonContext)) fail(file, "dungeonContext must be an object");
      else {
        reference(dungeonContext.dungeonId, file, "dungeonContext.dungeonId");
        const authoredSpaces = dungeonSpacesByDungeon.get(dungeonContext.dungeonId);
        const space = authoredSpaces?.get(dungeonContext.space);
        if (authoredSpaces && !space) fail(file, `dungeonContext.space ${dungeonContext.space} is not a space of ${dungeonContext.dungeonId}`);
        for (const field of ["ownerConceptKey", "corruptedVariantConceptKey"]) {
          if (!factionConceptKeys.includes(dungeonContext[field])) fail(file, `dungeonContext.${field} ${dungeonContext[field]} is not one of the six concept keys: ${factionConceptKeys.join(", ")}`);
        }
        const dungeonRecord = dungeonRecordsById.get(dungeonContext.dungeonId);
        if (dungeonRecord) {
          if (dungeonContext.ownerConceptKey !== dungeonRecord.defaultOwnerConceptKey) fail(file, `dungeonContext.ownerConceptKey must be ${dungeonRecord.defaultOwnerConceptKey}, the dungeon record's default owner`);
          if (dungeonContext.corruptedVariantConceptKey !== dungeonRecord.corruptedVariantConceptKey) fail(file, `dungeonContext.corruptedVariantConceptKey must be ${dungeonRecord.corruptedVariantConceptKey}, the dungeon record's corrupted variant`);
        }
        if (space) {
          if (space.worldCellId !== value.id) fail(file, `dungeonContext.space ${dungeonContext.space} is realised by ${space.worldCellId}, not by this cell`);
          if (JSON.stringify(dungeonContext.siteRuleIds) !== JSON.stringify(space.siteRuleIds)) fail(file, `dungeonContext.siteRuleIds must equal the ${dungeonContext.space} space's siteRuleIds in ${dungeonContext.dungeonId}`);
        } else if (!Array.isArray(dungeonContext.siteRuleIds) || dungeonContext.siteRuleIds.length === 0) {
          fail(file, "dungeonContext.siteRuleIds must name at least one site rule");
        }
        for (const [index, ruleId] of (dungeonContext.siteRuleIds ?? []).entries()) reference(ruleId, file, `dungeonContext.siteRuleIds[${index}]`);
      }
    }
    if (value.admission?.collisionSeparatedFromVisualShell !== true || value.admission?.navigationSeparatedFromVisualShell !== true) fail(file, "world cell must separate visual shell, collision and navigation");
  } else {
    if (value.trigger !== "midnight_flash") fail(file, "spawn rule trigger must be midnight_flash");
    if (value.grouping?.designRule !== "individual_threat") fail(file, "daily spawns must use individual_threat tuning");
    for (const [index, id] of (value.definitionIds ?? []).entries()) reference(id, file, `definitionIds[${index}]`);
  }
}

const worldCellsById = new Map(worldRecords.filter(({ value }) => value.kind === "world_cell").map(({ file, value }) => [value.id, { file, value }]));
for (const { file, value } of worldCellsById.values()) {
  for (const [index, portal] of (value.portals ?? []).entries()) {
    const target = worldCellsById.get(portal.targetCellId);
    if (!target) continue;
    if (!(target.value.entryAnchors ?? []).some(anchor => anchor.id === portal.targetAnchorId)) {
      fail(file, `portals[${index}].targetAnchorId must exist in target cell ${portal.targetCellId}`);
    }
  }
}

/// The standard room footprints, by ID, as `board.registry.json` declares them.
/// Filled by the `board_footprint_registry` branch below and read by the world
/// cells' `board` blocks after this loop, so there is one table and one reader.
const boardFootprints = new Map();
for (const { file, value } of await readJsonDirectory("presentation")) {
  if (!Array.isArray(value.entries) || value.entries.length === 0) fail(file, "presentation registry must contain entries");
  if (value.kind === "camera_registry") {
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.camera.")) fail(file, `entries[${index}].id must use presentation.camera prefix`);
      if (!new Set(["static", "track", "punch", "reset", "insert", "pan", "snap", "push"]).has(entry.mode)) fail(file, `${entry.id} has unsupported camera mode ${entry.mode}`);
      if (typeof entry.zoom !== "number" || entry.zoom < 0.8 || entry.zoom > 1.25) fail(file, `${entry.id} zoom must be from 0.8 through 1.25`);
      if (!Array.isArray(entry.focusSubjects) || entry.focusSubjects.length < 2) fail(file, `${entry.id} must name at least two focus subjects`);
      requireString(entry, "lead", file);
      if (!Number.isInteger(entry.safePaddingPercent) || entry.safePaddingPercent < 8 || entry.safePaddingPercent > 20) fail(file, `${entry.id} safePaddingPercent must be an integer from 8 through 20`);
      for (const field of ["transitionInMs", "transitionOutMs"]) if (!Number.isInteger(entry[field]) || entry[field] < 0 || entry[field] > 500) fail(file, `${entry.id} ${field} must be an integer from 0 through 500`);
    }
  } else if (value.kind === "vfx_registry") {
    const defaults = value.defaults ?? {};
    // P6's three closed vocabularies, hoisted so the failure message can print
    // them. `game/scripts/battle/vfx_factory.gd` implements exactly these.
    const vfxSockets = new Set([
      "none", "actor_body", "actor_main_hand", "actor_weapon_head", "actor_feet", "actor_overhead",
      "target_body", "target_impact", "target_feet", "target_overhead",
      "contact_midpoint", "ground_contact", "band_boundary", "party_rail"
    ]);
    const vfxEmitters = new Set(["none", "burst", "ring", "crescent", "beam", "trail", "motes", "panel"]);
    const vfxDirections = new Set(["none", "outward", "inward", "upward", "downward", "forward", "backward", "along_line"]);
    const vfxPaletteWords = new Set([
      "teal", "bronze", "gold", "cream", "white", "black", "red", "amber",
      "dust_brown", "warm_skin", "transparent",
      // Ayla's identity is an Open decision; her nine records carry this word
      // and the runtime renders them in a neutral marked `needs decision`.
      "palette_unestablished_pending_ayla_identity_lock"
    ]);
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.vfx.")) fail(file, `entries[${index}].id must use presentation.vfx prefix`);
      for (const field of ["purpose", "anchor", "motion"]) requireString(entry, field, file);
      // P6: `anchor` and `motion` stay the authored art words -- the brief a
      // painter reads -- and these three closed vocabularies are what the
      // runtime resolves them to, so a record owns its own effect and the
      // factory holds no table of its own. `socket` names the live rig socket
      // the effect is parented to, `emitter` the primitive that is built and
      // `direction` the way that primitive travels. Several anchors share one
      // socket (an ampoule and a mace core are both carried in the main hand);
      // that is a mapping, not a second answer to the same question.
      if (!vfxSockets.has(entry.socket)) fail(file, `${entry.id} socket must be one of ${[...vfxSockets].join(", ")}`);
      if (!vfxEmitters.has(entry.emitter)) fail(file, `${entry.id} emitter must be one of ${[...vfxEmitters].join(", ")}`);
      if (!vfxDirections.has(entry.direction)) fail(file, `${entry.id} direction must be one of ${[...vfxDirections].join(", ")}`);
      if ((entry.emitter === "none") !== (entry.socket === "none")) fail(file, `${entry.id} must build an emitter exactly when it hangs on a socket`);
      if (entry.emitter === "none" && entry.direction !== "none") fail(file, `${entry.id} emits nothing and must not claim a direction`);
      if (entry.emitter !== "beam" && entry.direction === "along_line") fail(file, `${entry.id} direction along_line belongs to a beam, the only emitter with two endpoints`);
      if (!Array.isArray(entry.palette) || entry.palette.length === 0 || entry.palette.some(color => typeof color !== "string" || color.length === 0)) fail(file, `${entry.id} palette must contain at least one named color`);
      // Every palette word must be one the runtime can resolve to a colour.
      // BattlePalette.EFFECT_COLORS is the other half of this pair; a word
      // added to one without the other is a black effect at runtime, so the
      // list is written here too and the two are kept equal by hand-off in
      // `game/tests/battle_presentation_test.gd`, which asserts every authored
      // word resolves.
      for (const word of entry.palette ?? []) if (!vfxPaletteWords.has(word)) fail(file, `${entry.id} palette word ${word} has no colour in BattlePalette.EFFECT_COLORS`);
      const layer = entry.layer ?? defaults.layer;
      const blend = entry.blend ?? defaults.blend;
      const reducedFlashMode = entry.reducedFlashMode ?? defaults.reducedFlashMode;
      const assetStatus = entry.assetStatus ?? defaults.assetStatus;
      const safeFrameOverflowAllowed = entry.safeFrameOverflowAllowed ?? defaults.safeFrameOverflowAllowed;
      if (!Number.isInteger(layer) || layer < 0 || layer > 100) fail(file, `${entry.id} layer must be an integer from 0 through 100`);
      if (!new Set(["mix", "add", "screen", "multiply"]).has(blend)) fail(file, `${entry.id} has unsupported blend ${blend}`);
      for (const field of ["envelopeWidthPercent", "envelopeHeightPercent"]) if (typeof entry[field] !== "number" || entry[field] < 0 || entry[field] > 100) fail(file, `${entry.id} ${field} must be from 0 through 100`);
      if (!Number.isInteger(entry.persistenceMs) || entry.persistenceMs < -1 || entry.persistenceMs > 5000) fail(file, `${entry.id} persistenceMs must be -1 or an integer through 5000`);
      if (typeof reducedFlashMode !== "string" || reducedFlashMode.length === 0) fail(file, `${entry.id} reducedFlashMode must be explicit or inherited`);
      if (!new Set(["placeholder", "approved", "not_required"]).has(assetStatus)) fail(file, `${entry.id} has unsupported assetStatus ${assetStatus}`);
      if (safeFrameOverflowAllowed !== false) fail(file, `${entry.id} must prohibit safe-frame overflow`);
    }
  } else if (value.kind === "motion_asset_registry") {
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.motion.")) fail(file, `entries[${index}].id must use presentation.motion prefix`);
      reference(entry.characterId, file, `entries[${index}].characterId`);
      for (const field of ["usage", "sourceProvider", "sourceCreationUrl", "sourceModel", "codec", "loopMode", "cameraMode", "importState", "approvalState"]) requireString(entry, field, file);
      if (!entry.sourceCreationUrl.startsWith("https://www.magnific.com/app/creation/")) fail(file, `${entry.id} must retain its Magnific creation provenance`);
      if (!Number.isInteger(entry.durationMs) || entry.durationMs < 250 || entry.durationMs > 10000) fail(file, `${entry.id} durationMs must be an integer from 250 through 10000`);
      if (!Number.isInteger(entry.frameRate) || entry.frameRate < 12 || entry.frameRate > 60) fail(file, `${entry.id} frameRate must be an integer from 12 through 60`);
      for (const field of ["width", "height"]) if (!Number.isInteger(entry[field]) || entry[field] < 256 || entry[field] > 4096) fail(file, `${entry.id} ${field} must be an integer from 256 through 4096`);
      if (!Number.isInteger(entry.safePaddingPercent) || entry.safePaddingPercent < 8 || entry.safePaddingPercent > 20) fail(file, `${entry.id} safePaddingPercent must be an integer from 8 through 20`);
      if (!Array.isArray(entry.requiredSubjects) || entry.requiredSubjects.length < 4) fail(file, `${entry.id} must name all required in-frame subjects`);
      if (!Array.isArray(entry.motionContract) || entry.motionContract.length < 3) fail(file, `${entry.id} must define the intended visible motion`);
      if (!Array.isArray(entry.hardRejects) || entry.hardRejects.length < 5) fail(file, `${entry.id} must define production rejection gates`);
      if (entry.importState === "source_downloaded_needs_runtime_transcode") {
        if (typeof entry.sourceLocalPath !== "string" || !entry.sourceLocalPath.startsWith("work/art/")) fail(file, `${entry.id} downloaded source requires a work/art sourceLocalPath`);
        if (typeof entry.sourceSha256 !== "string" || !/^[a-f0-9]{64}$/.test(entry.sourceSha256)) fail(file, `${entry.id} downloaded source requires a lowercase SHA-256`);
      }
      if (entry.importState === "imported" && (typeof entry.runtimePath !== "string" || !entry.runtimePath.startsWith("res://"))) fail(file, `${entry.id} imported assets require a res:// runtimePath`);
      if (entry.approvalState === "approved" && entry.importState !== "imported") fail(file, `${entry.id} cannot be approved before import`);
      if (entry.metadata?.releaseLegal !== false && entry.approvalState !== "approved") fail(file, `${entry.id} unapproved motion must set metadata.releaseLegal=false`);
    }
  } else if (value.kind === "paper_doll_registry") {
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.paper_")) fail(file, `entries[${index}].id must use presentation.paper_ prefix`);
      for (const field of ["placeholderAssetId", "subjectId", "displayName", "dollKind", "facing", "futureRuntimeAssetId", "futureFormat", "poseVariable"]) requireString(entry, field, file);
      if (!Array.isArray(entry.nativeCanvas) || entry.nativeCanvas.length !== 2 || entry.nativeCanvas.some(value => !Number.isInteger(value) || value < 256)) fail(file, `${entry.id} nativeCanvas must contain two integer dimensions of at least 256`);
      if (!Number.isInteger(entry.safePaddingPercent) || entry.safePaddingPercent < 8 || entry.safePaddingPercent > 20) fail(file, `${entry.id} safePaddingPercent must be from 8 through 20`);
      for (const field of ["groundAnchor", "rootPivot"]) if (!Array.isArray(entry[field]) || entry[field].length !== 2 || entry[field].some(value => typeof value !== "number" || value < 0 || value > 1)) fail(file, `${entry.id} ${field} must be a normalized coordinate pair`);
      if (!entry.attachmentAnchors || Object.keys(entry.attachmentAnchors).length < 3) fail(file, `${entry.id} requires at least three named attachment anchors`);
      else for (const [name, point] of Object.entries(entry.attachmentAnchors)) if (!Array.isArray(point) || point.length !== 2 || point.some(value => typeof value !== "number" || value < 0 || value > 1)) fail(file, `${entry.id} attachment anchor ${name} must be normalized`);
      if (!Array.isArray(entry.layerOrder) || entry.layerOrder.length < 5) fail(file, `${entry.id} requires an explicit layer order`);
      if (!Array.isArray(entry.identityInvariants) || entry.identityInvariants.length < 5) fail(file, `${entry.id} requires at least five identity invariants`);
      if (!Array.isArray(entry.replacementTests) || entry.replacementTests.length < 4) fail(file, `${entry.id} requires at least four replacement tests`);
    }
  } else if (value.kind === "board_footprint_registry") {
    // B11/P4: the one table of standard room footprints. Brief section 18
    // requires standard sizes so procedural placement cannot collide, and the
    // exact numbers are Open item O3 -- so every entry must carry the mark, and
    // a world cell names an entry here instead of writing metres of its own.
    // When O3 closes, six numbers change in one file and no room is re-authored.
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.board.footprint.")) fail(file, `entries[${index}].id must use the presentation.board.footprint prefix`);
      requireString(entry, "usage", file);
      for (const field of ["widthMetres", "depthMetres"]) {
        if (!Number.isInteger(entry[field]) || entry[field] < 8 || entry[field] > 200) fail(file, `${entry.id} ${field} must be a whole number of metres from 8 through 200`);
      }
      if (entry.needsDecision !== "O3") fail(file, `${entry.id} must be marked needsDecision O3: the exact standard dimensions are an open decision and a number here that claims otherwise is an invented one`);
      boardFootprints.set(entry.id, entry);
    }
  } else {
    fail(file, `unsupported presentation registry kind ${value.kind}`);
  }
}

// B11/P4: every world cell's `board` block, held against that table and against
// the room the cell already declares. Field names are dr-companion's
// (`footprint`, `spawnPoints`, `tethers`) so a single owner later is a rename
// rather than a rewrite. Four things are checked and none of them is believed:
// the footprint is a table entry rather than loose metres; the room's own entry
// anchors and every spawn point lie inside that footprint, so a cell cannot
// declare a box its authored doors are outside of; there is exactly one tether
// per portal, its kind is this file's classification of the portal's
// travelMode, and its anchor sits on the footprint edge facing the target room.
const TETHER_KIND_BY_TRAVEL_MODE = new Map([["on_foot", "walk"], ["safe_road", "road"], ["jungle_edge", "trail"]]);
const SPAWN_ROLES = new Map([["player", "humanoid-root"], ["occupant", "humanoid-root"], ["hostile", "creature-root"], ["item", "item-root"]]);
const BOARD_SPAWN_POINT_MINIMUM = 7;
for (const { file, value } of worldCellsById.values()) {
  const board = value.board;
  if (!board || typeof board !== "object" || Array.isArray(board)) {
    fail(file, "world cell requires a board block: B11's footprint, spawn points and tethers");
    continue;
  }
  const size = boardFootprints.get(board.footprint?.sizeId);
  reference(board.footprint?.sizeId, file, "board.footprint.sizeId");
  if (board.footprint?.sizeId !== undefined && !size) fail(file, `board.footprint.sizeId ${board.footprint.sizeId} is not an entry in content/presentation/board.registry.json; a room may not write its own metres while O3 is open`);
  if (!Array.isArray(board.islandPositionMetres) || board.islandPositionMetres.length !== 2 || board.islandPositionMetres.some(component => typeof component !== "number")) fail(file, "board.islandPositionMetres must be the room's two-component place on the island, in metres");
  if (typeof board.elevationMetres !== "number") fail(file, "board.elevationMetres must be the room's height above sea level, in metres");
  const halfWidth = size ? size.widthMetres / 2 : Infinity;
  const halfDepth = size ? size.depthMetres / 2 : Infinity;
  const inside = (position, margin = 0) => Math.abs(position[0]) <= halfWidth + margin && Math.abs(position[2]) <= halfDepth + margin;
  for (const anchor of value.entryAnchors ?? []) {
    if (Array.isArray(anchor.positionMetres) && !inside(anchor.positionMetres)) fail(file, `entry anchor ${anchor.id} stands outside the ${board.footprint?.sizeId} footprint this cell declares`);
  }
  if (!Array.isArray(board.spawnPoints) || board.spawnPoints.length < BOARD_SPAWN_POINT_MINIMUM) fail(file, `board.spawnPoints must offer at least ${BOARD_SPAWN_POINT_MINIMUM} sockets; S7's forces materialise into these`);
  for (const [index, spawn] of (Array.isArray(board.spawnPoints) ? board.spawnPoints : []).entries()) {
    registerId(spawn.id, file);
    if (!spawn.id?.startsWith("spawn.")) fail(file, `board.spawnPoints[${index}].id must use the spawn. prefix`);
    if (!SPAWN_ROLES.has(spawn.role)) fail(file, `board.spawnPoints[${index}].role must be one of ${[...SPAWN_ROLES.keys()].join(", ")}`);
    if (SPAWN_ROLES.has(spawn.role) && spawn.rigSocket !== SPAWN_ROLES.get(spawn.role)) fail(file, `board.spawnPoints[${index}] is a ${spawn.role} socket and must carry rigSocket ${SPAWN_ROLES.get(spawn.role)}`);
    if (!Array.isArray(spawn.positionMetres) || spawn.positionMetres.length !== 3 || spawn.positionMetres.some(component => typeof component !== "number")) fail(file, `board.spawnPoints[${index}].positionMetres must be three numeric metres`);
    else if (!inside(spawn.positionMetres)) fail(file, `board.spawnPoints[${index}] ${spawn.id} stands outside the ${board.footprint?.sizeId} footprint`);
  }
  for (const role of SPAWN_ROLES.keys()) {
    if (!(board.spawnPoints ?? []).some(spawn => spawn?.role === role)) fail(file, `board.spawnPoints must include at least one ${role} socket`);
  }
  const tetherPortalIds = (Array.isArray(board.tethers) ? board.tethers : []).map(tether => tether?.portalId);
  for (const [index, portal] of (value.portals ?? []).entries()) {
    if (!tetherPortalIds.includes(portal.id)) fail(file, `board.tethers is missing the tether for portals[${index}] ${portal.id}; B11 requires one tether per portal`);
  }
  for (const [index, tether] of (Array.isArray(board.tethers) ? board.tethers : []).entries()) {
    const portal = (value.portals ?? []).find(candidate => candidate.id === tether?.portalId);
    if (!portal) {
      fail(file, `board.tethers[${index}].portalId must name a portal of this cell`);
      continue;
    }
    const expectedKind = TETHER_KIND_BY_TRAVEL_MODE.get(portal.travelMode);
    if (tether.kind !== expectedKind) fail(file, `board.tethers[${index}].kind must be ${expectedKind}, this file's classification of travelMode ${portal.travelMode}`);
    if (!Array.isArray(tether.anchorMetres) || tether.anchorMetres.length !== 3 || tether.anchorMetres.some(component => typeof component !== "number")) {
      fail(file, `board.tethers[${index}].anchorMetres must be three numeric metres`);
      continue;
    }
    if (!size) continue;
    const [x, , z] = tether.anchorMetres;
    const onEdge = Math.abs(Math.abs(x) - halfWidth) < 0.05 || Math.abs(Math.abs(z) - halfDepth) < 0.05;
    if (!onEdge || !inside(tether.anchorMetres, 0.05)) fail(file, `board.tethers[${index}] anchor must sit on the ${board.footprint.sizeId} footprint's edge, not inside it and not beyond it`);
    const target = worldCellsById.get(portal.targetCellId)?.value?.board?.islandPositionMetres;
    const here = board.islandPositionMetres;
    if (Array.isArray(target) && Array.isArray(here)) {
      if ((target[0] - here[0]) * x + (target[1] - here[1]) * z <= 0) fail(file, `board.tethers[${index}] anchor faces away from ${portal.targetCellId}; a tether leaves the room toward the room it reaches`);
    }
  }
}

// Brief section 18: standard footprint sizes exist so procedural placement
// cannot collide. Two rooms whose declared boxes overlap on the island are that
// collision, authored -- and the board would draw one room inside another.
const boardPlacements = [...worldCellsById.values()]
  .map(({ file, value }) => ({ file, id: value.id, island: value.board?.islandPositionMetres, size: boardFootprints.get(value.board?.footprint?.sizeId) }))
  .filter(placement => Array.isArray(placement.island) && placement.size);
for (const [index, left] of boardPlacements.entries()) {
  for (const right of boardPlacements.slice(index + 1)) {
    const apart = Math.abs(left.island[0] - right.island[0]) >= (left.size.widthMetres + right.size.widthMetres) / 2
      || Math.abs(left.island[1] - right.island[1]) >= (left.size.depthMetres + right.size.depthMetres) / 2;
    if (!apart) fail(left.file, `board footprint overlaps ${right.id}; standard footprints exist so two rooms cannot occupy the same island ground`);
  }
}

const placeholderFile = resolve(repo, "content/art/placeholders.json");
const placeholderManifest = JSON.parse(await readFile(placeholderFile, "utf8"));
for (const asset of placeholderManifest.assets ?? []) {
  registerId(asset.id, placeholderFile);
  if (asset.placeholder !== true) fail(placeholderFile, `${asset.id} belongs in placeholder manifest but placeholder is not true`);
  for (const key of ["consumer", "visibleMark", "intendedFinal"]) requireString(asset, key, placeholderFile);
  if (!asset.visibleMark.includes("DUMMY")) fail(placeholderFile, `${asset.id} visibleMark must visibly say DUMMY`);
  if (!Array.isArray(asset.requiredTags) || asset.requiredTags.length < 3) fail(placeholderFile, `${asset.id} requires at least three art tags`);
  if (!Array.isArray(asset.replacementGate) || asset.replacementGate.length < 2) fail(placeholderFile, `${asset.id} needs explicit replacement gates`);
}

const reelPlanFile = resolve(repo, "content/art/video_reel_plan.json");
const reelPlan = JSON.parse(await readFile(reelPlanFile, "utf8"));
registerId(reelPlan.id, reelPlanFile);
if (reelPlan.productionSkill !== "magnific-2d-frame-factory") fail(reelPlanFile, "productionSkill must name the Magnific frame-factory contract");
const forbiddenPromptRationale = /\b(because|so that|we need|our goal|the reason|gameplay purpose|use this to|intended to provide)\b/i;
for (const [index, reel] of (reelPlan.reels ?? []).entries()) {
  registerId(reel.id, reelPlanFile);
  for (const field of ["type", "sourceAnchor", "aspectRatio", "prompt", "status"]) requireString(reel, field, reelPlanFile);
  if (!reel.id?.startsWith("art.reel.")) fail(reelPlanFile, `reels[${index}].id must use art.reel prefix`);
  if (forbiddenPromptRationale.test(reel.prompt ?? "")) fail(reelPlanFile, `${reel.id} prompt contains parser-confusing rationale`);
  if ((reel.prompt ?? "").length < 300) fail(reelPlanFile, `${reel.id} prompt is too short to be a production contract`);
  if (!Number.isInteger(reel.durationSeconds) || reel.durationSeconds < 5 || reel.durationSeconds > 15) fail(reelPlanFile, `${reel.id} durationSeconds must be an integer from 5 through 15`);
  if (![6, 12, 24].includes(reel.extractionFps)) fail(reelPlanFile, `${reel.id} extractionFps must be 6, 12 or 24`);
  if (!Number.isInteger(reel.maxFrames) || reel.maxFrames < 12 || reel.maxFrames > 240) fail(reelPlanFile, `${reel.id} maxFrames must be an integer from 12 through 240`);
  if (reel.aspectRatio !== "16:9") fail(reelPlanFile, `${reel.id} must use 16:9`);
}

const creaturePlanFile = resolve(repo, "content/art/creature_plan.json");
const creaturePlan = JSON.parse(await readFile(creaturePlanFile, "utf8"));
registerId(creaturePlan.id, creaturePlanFile);
if (creaturePlan.productionMethod !== "magnific_still_image_generation") fail(creaturePlanFile, "productionMethod must name the Magnific still-image generation contract");
let creaturePlateCount = 0;
for (const [index, plate] of (creaturePlan.plates ?? []).entries()) {
  creaturePlateCount += 1;
  registerId(plate.id, creaturePlanFile);
  for (const field of ["aspectRatio", "prompt", "status"]) requireString(plate, field, creaturePlanFile);
  if (!plate.id?.startsWith("art.plate.")) fail(creaturePlanFile, `plates[${index}].id must use art.plate prefix`);
  reference(plate.subjectId, creaturePlanFile, `plates[${index}].subjectId`);
  if (forbiddenPromptRationale.test(plate.prompt ?? "")) fail(creaturePlanFile, `${plate.id} prompt contains parser-confusing rationale`);
  if ((plate.prompt ?? "").length < 300) fail(creaturePlanFile, `${plate.id} prompt is too short to be a production contract`);
}

const sharedAssetLedgerFile = resolve(repo, "content/art/shared_asset_ledger.json");
const sharedAssetLedger = JSON.parse(await readFile(sharedAssetLedgerFile, "utf8"));
registerId(sharedAssetLedger.id, sharedAssetLedgerFile);
if (sharedAssetLedger.schemaVersion !== 1) fail(sharedAssetLedgerFile, "schemaVersion must be 1");
const sharedConsumers = new Set(["project42", "dr-companion", "professional-client"]);
if (!Array.isArray(sharedAssetLedger.allowedConsumers) || sharedAssetLedger.allowedConsumers.length !== sharedConsumers.size || sharedAssetLedger.allowedConsumers.some(value => !sharedConsumers.has(value))) fail(sharedAssetLedgerFile, "allowedConsumers must declare the three supported consuming project classes");
if (typeof sharedAssetLedger.sharedRawLicensePolicy !== "string" || !sharedAssetLedger.sharedRawLicensePolicy.includes("CC0-1.0")) fail(sharedAssetLedgerFile, "sharedRawLicensePolicy must explicitly reserve raw sharing for CC0-1.0");
const admittedAssetStatuses = new Set(["candidate", "quarantine", "approved_shared", "approved_project_only", "rejected"]);
const sharedAssetCategories = new Set(["vegetation", "terrain", "architecture", "prop", "material", "vfx", "rig", "character", "reference"]);
const ownershipClasses = new Set(["cc0_raw", "paid_source", "account_generated", "project_authored"]);
const reviewDecisions = new Set(["pending", "approved", "rejected"]);
for (const [index, asset] of (sharedAssetLedger.assetRecords ?? []).entries()) {
  const field = `assetRecords[${index}]`;
  registerId(asset?.id, sharedAssetLedgerFile);
  if (!admittedAssetStatuses.has(asset?.status)) fail(sharedAssetLedgerFile, `${field}.status is unsupported`);
  if (!sharedAssetCategories.has(asset?.category)) fail(sharedAssetLedgerFile, `${field}.category is unsupported`);
  if (!ownershipClasses.has(asset?.ownership)) fail(sharedAssetLedgerFile, `${field}.ownership is unsupported`);
  const source = asset?.source;
  for (const key of ["creator", "canonicalUrl", "retrievedOn", "originalFilename", "sha256", "licenseSpdx", "licenseEvidenceUrl", "licenseNotes", "sourceAccess"]) requireString(source ?? {}, key, sharedAssetLedgerFile);
  if (!source?.canonicalUrl?.startsWith("https://")) fail(sharedAssetLedgerFile, `${field}.source.canonicalUrl must be an HTTPS URL`);
  if (!(source?.licenseEvidenceUrl?.startsWith("https://") || source?.licenseEvidenceUrl?.startsWith("docs/"))) fail(sharedAssetLedgerFile, `${field}.source.licenseEvidenceUrl must be an HTTPS URL or committed documentation path`);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(source?.retrievedOn ?? "")) fail(sharedAssetLedgerFile, `${field}.source.retrievedOn must use YYYY-MM-DD`);
  if (!/^[a-f0-9]{64}$/.test(source?.sha256 ?? "")) fail(sharedAssetLedgerFile, `${field}.source.sha256 must be a lowercase SHA-256`);
  if (typeof source?.redistributionAllowed !== "boolean") fail(sharedAssetLedgerFile, `${field}.source.redistributionAllowed must be boolean`);
  if (!new Set(["repository_raw", "project_local_vendor_cache", "account_controlled"]).has(source?.sourceAccess)) fail(sharedAssetLedgerFile, `${field}.source.sourceAccess is unsupported`);
  if (!Array.isArray(asset?.consumers) || asset.consumers.length === 0 || asset.consumers.some(consumer => !sharedConsumers.has(consumer))) fail(sharedAssetLedgerFile, `${field}.consumers must name one or more supported consumers`);
  if (!Array.isArray(asset?.tags) || asset.tags.length < 3) fail(sharedAssetLedgerFile, `${field}.tags needs at least three searchable tags`);
  const review = asset?.review;
  for (const key of ["targetCamera", "visualDecision", "technicalDecision"]) requireString(review ?? {}, key, sharedAssetLedgerFile);
  if (!reviewDecisions.has(review?.visualDecision) || !reviewDecisions.has(review?.technicalDecision)) fail(sharedAssetLedgerFile, `${field}.review decisions are unsupported`);
  if (!Array.isArray(review?.notes) || review.notes.length === 0) fail(sharedAssetLedgerFile, `${field}.review.notes must record an actual review`);
  const imported = asset?.import;
  for (const key of ["sourceFormat", "intendedRuntimeFormat", "pivotAndGroundContact", "materialStrategy", "collisionOrSelection", "lodStrategy"]) requireString(imported ?? {}, key, sharedAssetLedgerFile);
  if (asset?.status === "approved_shared") {
    if (asset.ownership !== "cc0_raw" || source?.licenseSpdx !== "CC0-1.0" || source?.redistributionAllowed !== true) fail(sharedAssetLedgerFile, `${field} approved_shared assets must be CC0-1.0 and redistribution-permitted`);
    if (review?.visualDecision !== "approved" || review?.technicalDecision !== "approved") fail(sharedAssetLedgerFile, `${field} approved_shared assets require approved visual and technical review`);
  }
  if ((asset?.ownership === "paid_source" || asset?.ownership === "account_generated") && source?.sourceAccess === "repository_raw") fail(sharedAssetLedgerFile, `${field} paid or account-generated raw sources may not be stored as repository_raw`);
  if (asset?.ownership === "paid_source" && asset?.status === "approved_shared") fail(sharedAssetLedgerFile, `${field} paid sources may not be marked approved_shared without a separately modeled license exception`);
}

const sharedSourceCollectionsFile = resolve(repo, "content/art/shared_source_collections.json");
const sharedSourceCollections = JSON.parse(await readFile(sharedSourceCollectionsFile, "utf8"));
if (sharedSourceCollections.format !== "shared_source_collection_catalog" || sharedSourceCollections.schemaVersion !== 1) fail(sharedSourceCollectionsFile, "must declare shared_source_collection_catalog format version 1");
if (!Array.isArray(sharedSourceCollections.sourceCollections) || sharedSourceCollections.sourceCollectionCount !== sharedSourceCollections.sourceCollections.length) fail(sharedSourceCollectionsFile, "sourceCollectionCount must match sourceCollections");
const sourceCollectionKeys = new Set();
for (const [index, collection] of (sharedSourceCollections.sourceCollections ?? []).entries()) {
  const field = `sourceCollections[${index}]`;
  registerId(collection?.id, sharedSourceCollectionsFile);
  for (const key of ["collectionKey", "title", "creator", "canonicalUrl", "authoringLineage", "licenseSpdx", "retrievedOn", "defaultPhysicalType", "state"]) requireString(collection ?? {}, key, sharedSourceCollectionsFile);
  if (!collection?.id?.startsWith("source.collection.")) fail(sharedSourceCollectionsFile, `${field}.id must use source.collection prefix`);
  if (sourceCollectionKeys.has(collection?.collectionKey)) fail(sharedSourceCollectionsFile, `${field}.collectionKey is duplicated`);
  sourceCollectionKeys.add(collection?.collectionKey);
  if (!collection?.canonicalUrl?.startsWith("https://")) fail(sharedSourceCollectionsFile, `${field}.canonicalUrl must be HTTPS`);
  if (collection?.authoringLineage !== "source_cc0" || collection?.licenseSpdx !== "CC0-1.0") fail(sharedSourceCollectionsFile, `${field} must explicitly remain CC0 source material`);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(collection?.retrievedOn ?? "")) fail(sharedSourceCollectionsFile, `${field}.retrievedOn must use YYYY-MM-DD`);
  const archive = collection?.archive;
  for (const key of ["localPath", "filename", "sha256"]) requireString(archive ?? {}, key, sharedSourceCollectionsFile);
  if (!Number.isInteger(archive?.bytes) || archive.bytes <= 0 || !/^[a-f0-9]{64}$/.test(archive?.sha256 ?? "")) fail(sharedSourceCollectionsFile, `${field}.archive must have positive bytes and lowercase SHA-256`);
  if (!Array.isArray(collection?.styleTags) || collection.styleTags.length < 2) fail(sharedSourceCollectionsFile, `${field}.styleTags needs at least two tags`);
}

// C8: presentation-override packs. A pack is the adult presentation layer's
// only shape: it replaces a scene's beats, it never adds a scene, and it never
// carries simulation data. B10's `ContentPackRegistry` merges packs by scene ID
// at runtime and E9 builds each one into its own `.pck`, so a pack is a
// separate artifact -- validated here, and deliberately NOT bundled by
// `tools/src/build-content-bundle.mjs`.
//
// This block runs after `content/relationships/`, because "every override
// targets an existing scene" is resolved against the stable IDs that directory
// registered.
//
// Asset resolution is closed inside the pack on purpose: an override's
// `assetIds` may only name IDs the pack itself declares in its top-level
// `assets` array, and each declared asset's `path` must be a pack-relative
// path to a file that is actually there. A pack that could reach a base-game
// asset ID would break the day the base game moved one, and B10 merges packs
// without the base tree's ID map in hand. Pack asset IDs are therefore never
// registered as repository stable IDs either -- a base-game record must not be
// able to reference something that may not be installed.
const packTargetLevels = new Set(["fade_to_black", "explicit"]);
// A pack is presentation. These are the record kinds it may not carry, by key
// and by ID namespace: rules, skills, characters, factions and buildings are
// the simulation, and the simulation never reads presentation.
const forbiddenPackKeys = ["rules", "skill", "skills", "character", "characters", "faction", "factions", "building", "buildings"];
const forbiddenPackIdPrefixes = ["skill.", "character.", "faction.", "building.", "building_instance."];
const packsDirectory = resolve(repo, "packs");
let packCount = 0;
let packOverrideCount = 0;
const packDirectoryNames = await readdir(packsDirectory, { withFileTypes: true })
  .then(entries => entries.filter(entry => entry.isDirectory()).map(entry => entry.name).sort())
  .catch(() => []);
for (const directoryName of packDirectoryNames) {
  const file = resolve(packsDirectory, directoryName, "pack.json");
  let value;
  try { value = JSON.parse(await readFile(file, "utf8")); }
  catch (error) { fail(file, `every directory under packs/ must carry a readable pack.json: ${error.message}`); continue; }
  packCount += 1;

  registerId(value.id, file);
  if (typeof value.id !== "string" || !value.id.startsWith("pack.")) fail(file, `id ${value.id} must be pack.<slug>`);
  else if (value.id !== directoryName) fail(file, `id ${value.id} must equal its directory name ${directoryName}; B10 loads packs by directory`);
  if (value.kind !== "presentation_override") fail(file, `kind ${value.kind} must be presentation_override; a pack is presentation and nothing else`);
  if (!packTargetLevels.has(value.targetLevel)) fail(file, `targetLevel ${value.targetLevel} must be one of ${[...packTargetLevels].join(", ")}`);
  requireString(value, "displayName", file);

  for (const key of forbiddenPackKeys) {
    if (value[key] !== undefined) fail(file, `a presentation pack may not carry a ${key} key; rules, skill, character, faction and building records are the simulation, and a pack overrides presentation only (C8, B10)`);
  }

  const declaredAssetIds = new Set();
  if (!Array.isArray(value.assets)) fail(file, "assets must be an array declaring every asset this pack carries");
  else for (const [index, asset] of value.assets.entries()) {
    const where = `assets[${index}]`;
    if (typeof asset?.id !== "string" || !asset.id.startsWith("asset.")) fail(file, `${where}.id must be an asset.<...> ID local to this pack`);
    else if (declaredAssetIds.has(asset.id)) fail(file, `${where}.id ${asset.id} is declared twice`);
    else if (ids.has(asset.id)) fail(file, `${where}.id ${asset.id} collides with a base-game stable ID; a pack's assets live in their own namespace because a pack may not be installed`);
    else declaredAssetIds.add(asset.id);
    if (typeof asset?.path !== "string" || asset.path.trim() === "" || asset.path.startsWith("/") || asset.path.split("/").includes("..")) fail(file, `${where}.path must be a relative path inside the pack, with no leading slash and no ..`);
    else {
      const assetPath = resolve(packsDirectory, directoryName, asset.path);
      const exists = await stat(assetPath).then(entry => entry.isFile()).catch(() => false);
      if (!exists) fail(file, `${where}.path ${asset.path} does not resolve to a file inside the pack`);
    }
  }

  if (!value.overrides || typeof value.overrides !== "object" || Array.isArray(value.overrides)) fail(file, "overrides must be an object keyed by the scene IDs this pack replaces");
  else for (const [sceneId, override] of Object.entries(value.overrides)) {
    packOverrideCount += 1;
    if (!sceneId.startsWith("scene.")) fail(file, `overrides key ${sceneId} must be a scene.<...> ID; a pack overrides scenes and adds nothing`);
    else if (!ids.has(sceneId)) fail(file, `overrides targets ${sceneId}, which no record in content/relationships/ declares`);
    if (!override || typeof override !== "object" || Array.isArray(override)) { fail(file, `overrides.${sceneId} must be an object`); continue; }
    for (const key of forbiddenPackKeys) {
      if (override[key] !== undefined) fail(file, `overrides.${sceneId} may not carry a ${key} key; a pack overrides presentation only (C8)`);
    }
    // The same beat shape a scene carries, checked the same way, because the
    // pack's beats replace the scene's and B10 hands them to the same player.
    if (!Array.isArray(override.beats) || override.beats.length === 0) fail(file, `overrides.${sceneId}.beats must contain at least one beat`);
    else {
      const beatIds = new Set();
      for (const [index, beat] of override.beats.entries()) {
        const where = `overrides.${sceneId}.beats[${index}]`;
        if (typeof beat?.id !== "string" || !beat.id.startsWith("beat.")) fail(file, `${where}.id must be a beat.<...> ID`);
        else if (beatIds.has(beat.id)) fail(file, `${where}.id ${beat.id} is duplicated`);
        else beatIds.add(beat.id);
        if (!sceneBeatKinds.has(beat?.kind)) fail(file, `${where}.kind ${beat?.kind} must be one of ${[...sceneBeatKinds].join(", ")}`);
        if (beat?.kind === "fade" && index !== override.beats.length - 1) fail(file, `${where} is a fade beat with beats after it; a fade closes the scene`);
        if (typeof beat?.text !== "string" || beat.text.trim().length < 20) fail(file, `${where}.text must carry the authored line`);
      }
    }
    if (!Array.isArray(override.assetIds) || override.assetIds.length === 0) fail(file, `overrides.${sceneId}.assetIds must name at least one asset this pack declares`);
    else for (const [index, assetId] of override.assetIds.entries()) {
      if (forbiddenPackIdPrefixes.some(prefix => typeof assetId === "string" && assetId.startsWith(prefix))) fail(file, `overrides.${sceneId}.assetIds[${index}] ${assetId} names a simulation record, not an asset`);
      else if (!declaredAssetIds.has(assetId)) fail(file, `overrides.${sceneId}.assetIds[${index}] ${assetId} does not resolve inside this pack; an override may only name an asset the pack's own assets array declares`);
    }
  }

  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "pack metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "pack metadata.notes must record what the pack is and what it deliberately does not carry");
}

// P9: the soundscape's records. Four kinds and one voice shape between them: a
// `bed` is a seamless layer, a `cue` is a one-shot, an `ambience` record mixes
// beds for one region/segment/weather triple, and a `music` record mixes beds
// for one of the four states the surface asks for by name. Every record is a
// procedural placeholder until a real asset is admitted, and the asset block is
// where its licence, source and author go on the day it is -- the repository
// admits no audio file of uncertain provenance.
//
// `emittedBattleEvents` is the full list the bridge writes as `kind`; the
// bindable subset above is the part a skill beat may be tied to. The subset
// check below is what stops the two lists becoming two answers to one question.
const emittedBattleEvents = new Set([
  "activation_denied", "actor_defeated", "actor_focused", "actor_moved", "actor_revived",
  "battle_ended", "battle_retreated", "battle_started", "battlefield_effect_created",
  "battlefield_effect_pulse", "battlefield_effect_removed", "bonus_turn_granted",
  "command_accepted", "command_rejected", "damage_applied", "defeat_prevented",
  "enemy_intent_declared", "guard_changed", "interception_set", "interception_triggered",
  "reaction_triggered", "reaction_window_opened", "recovery_opening_consumed",
  "recovery_opening_created", "recovery_opening_expired", "round_started",
  "site_rule_overridden", "status_applied", "status_removed", "target_inspected",
  "turn_ended", "turn_started", "vitality_changed", "ward_line_placed", "ward_line_triggered",
]);
const audioFile = resolve(repo, "content/audio");
for (const eventKind of bindableBattleEvents) {
  if (!emittedBattleEvents.has(eventKind)) fail(audioFile, `bindable battle event ${eventKind} is not in the list of kinds the bridge emits`);
}

const audioBuses = new Set(["Master", "Music", "Ambience", "Effects", "Voice"]);
const audioWaveforms = new Set(["sine", "triangle", "square", "saw", "noise"]);
const audioRecordKinds = new Set(["audio_bed", "audio_cue", "audio_ambience", "audio_music"]);
const audioCrossfades = new Set(["ambience_slow", "ambience_weather_break", "music_settle", "music_cut"]);
const audioMusicStates = ["calm", "notable", "battle", "urgent"];
const audioTimeSegments = ["dawn", "day", "dusk", "midnight"];
// S8's `WeatherCondition`, in the order that enum declares: ordinary weather
// first, the corrupted end last. A sixth condition here would be an invention.
const audioWeatherConditions = ["clear", "overcast", "rain", "storm", "unnatural"];
let audioPlaceholderCount = 0;
const audioBedIds = new Set();
const audioCueIds = new Set();
const audioAmbienceTriples = new Set();
const audioMusicStatesSeen = new Set();

function validateVoice(voice, file, label) {
  if (!voice || typeof voice !== "object" || Array.isArray(voice)) return fail(file, `${label} voice must be an object the synth can render`);
  if (!audioWaveforms.has(voice.waveform)) fail(file, `${label} voice.waveform must be one of ${[...audioWaveforms].join(", ")}`);
  for (const key of ["pitchHz", "pitchGlideHz", "lowpassHz", "tremoloHz"]) {
    if (typeof voice[key] !== "number" || !Number.isFinite(voice[key])) fail(file, `${label} voice.${key} must be a finite number`);
  }
  if (!(voice.pitchHz > 0) || voice.pitchHz > 20000) fail(file, `${label} voice.pitchHz must be an audible frequency`);
  if (!(voice.lowpassHz > 0)) fail(file, `${label} voice.lowpassHz must be positive`);
  for (const key of ["noiseMix", "sustainLevel", "tremoloDepth"]) {
    if (typeof voice[key] !== "number" || voice[key] < 0 || voice[key] > 1) fail(file, `${label} voice.${key} must be from 0 through 1`);
  }
  for (const key of ["durationMs", "attackMs", "decayMs", "releaseMs", "seed"]) {
    if (!Number.isInteger(voice[key]) || voice[key] < 0) fail(file, `${label} voice.${key} must be a non-negative integer`);
  }
  // Determinism: the synth never draws an unseeded number, so a voice with any
  // noise in it has to say which stream it draws from.
  if (voice.noiseMix > 0 && voice.seed <= 0) fail(file, `${label} voice.seed must be a positive integer: noise is rendered from the record's own seed and never from an unseeded draw`);
  if (typeof voice.loop !== "boolean") fail(file, `${label} voice.loop must be boolean`);
  if (voice.durationMs < 40 || voice.durationMs > 8000) fail(file, `${label} voice.durationMs must be from 40 through 8000`);
  if (voice.attackMs + voice.decayMs + voice.releaseMs > voice.durationMs) fail(file, `${label} voice envelope is longer than the sound it shapes`);
}

function validateAudioAsset(record, file) {
  const asset = record.asset;
  if (!asset || typeof asset !== "object") return fail(file, `${record.id} must carry an asset block naming its provenance`);
  if (asset.status !== "procedural_placeholder" && asset.status !== "admitted_asset") return fail(file, `${record.id} asset.status must be procedural_placeholder until a real asset is admitted, then admitted_asset`);
  requireString(asset, "generator", file);
  for (const key of ["runtimePath", "licence", "source", "author"]) {
    if (typeof asset[key] !== "string") fail(file, `${record.id} asset.${key} must be a string, empty while the sound is a placeholder`);
  }
  if (asset.status === "procedural_placeholder") {
    audioPlaceholderCount += 1;
    for (const key of ["runtimePath", "licence", "source", "author"]) {
      if (asset[key] !== "") fail(file, `${record.id} asset.${key} claims a real asset while asset.status still says procedural_placeholder`);
    }
  } else {
    for (const key of ["runtimePath", "licence", "source", "author"]) {
      if (asset[key] === "") fail(file, `${record.id} asset.${key} must be recorded before an admitted audio file may ship`);
    }
  }
  if (!Array.isArray(asset.replacementGate) || asset.replacementGate.length < 2) fail(file, `${record.id} needs at least two explicit replacement gates`);
}

function validateAudioLayers(record, file) {
  if (!Array.isArray(record.layers) || record.layers.length === 0) return fail(file, `${record.id} must mix at least one bed`);
  const seen = new Set();
  for (const [index, layer] of record.layers.entries()) {
    reference(layer?.bedId, file, `${record.id}.layers[${index}].bedId`);
    if (seen.has(layer?.bedId)) fail(file, `${record.id} mixes ${layer?.bedId} twice`);
    seen.add(layer?.bedId);
    if (typeof layer?.gainDb !== "number" || layer.gainDb > 0 || layer.gainDb < -60) fail(file, `${record.id}.layers[${index}].gainDb must be a number from -60 through 0`);
  }
  if (!audioCrossfades.has(record.crossfade)) fail(file, `${record.id}.crossfade must name one of the durations Soundscape declares: ${[...audioCrossfades].join(", ")}`);
}

for (const { file, value } of await readJsonDirectory("audio")) {
  if (!audioRecordKinds.has(value.kind)) fail(file, `${value.id} kind must be one of ${[...audioRecordKinds].join(", ")}`);
  requireString(value, "displayName", file);
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "maturity", file);
  if (value.kind === "audio_bed" || value.kind === "audio_cue") {
    validateVoice(value.voice, file, value.id);
    validateAudioAsset(value, file);
    requireString(value, "purpose", file);
  }
  if (value.kind === "audio_bed") {
    if (value.voice?.loop !== true) fail(file, `${value.id} is a bed and must loop`);
    audioBedIds.add(value.id);
  }
  if (value.kind === "audio_cue") {
    if (value.voice?.loop !== false) fail(file, `${value.id} is a one-shot cue and must not loop`);
    if (value.bus !== "Effects") fail(file, `${value.id} must play on the Effects bus`);
    audioCueIds.add(value.id);
  }
  if (value.kind === "audio_ambience") {
    validateAudioLayers(value, file);
    reference(value.regionId, file, `${value.id}.regionId`);
    if (!audioTimeSegments.includes(value.timeSegment)) fail(file, `${value.id}.timeSegment must be one of ${audioTimeSegments.join(", ")}`);
    if (!audioWeatherConditions.includes(value.weather)) fail(file, `${value.id}.weather must be one of ${audioWeatherConditions.join(", ")}`);
    if (value.bus !== "Ambience") fail(file, `${value.id} must play on the Ambience bus`);
    audioAmbienceTriples.add(`${value.regionId}|${value.timeSegment}|${value.weather}`);
  }
  if (value.kind === "audio_music") {
    validateAudioLayers(value, file);
    if (!audioMusicStates.includes(value.state)) fail(file, `${value.id}.state must be one of ${audioMusicStates.join(", ")}`);
    if (value.bus !== "Music") fail(file, `${value.id} must play on the Music bus`);
    if (value.id !== `audio.music.${value.state}`) fail(file, `${value.id} must be named for the state it plays`);
    audioMusicStatesSeen.add(value.state);
  }
  if (value.bus !== undefined && !audioBuses.has(value.bus)) fail(file, `${value.id}.bus must be one of ${[...audioBuses].join(", ")}`);
}

for (const bedId of audioBedIds) {
  // A bed nothing mixes is a noodle to nowhere; the synth would render it and
  // no ambience or music record would ever play it.
  if (!references.some(item => item.id === bedId && item.field.includes(".layers["))) fail(audioFile, `${bedId} is mixed by no ambience or music record`);
}
for (const kind of emittedBattleEvents) {
  if (!audioCueIds.has(`audio.cue.event.${kind}`)) fail(audioFile, `battle event ${kind} has no cue record: content/audio/cue.event.${kind}.json is missing`);
}
for (const state of audioMusicStates) {
  if (!audioMusicStatesSeen.has(state)) fail(audioFile, `music state ${state} has no record`);
}
// Every authored region, every segment, every weather condition: the ambience
// chooser must never be handed a triple it cannot answer, and the region list
// is the one `content/world/*.region.json` declares -- there is no second list.
for (const { value } of worldRecords.filter(({ value }) => value.kind === "world_region")) {
  for (const segment of audioTimeSegments) {
    for (const weather of audioWeatherConditions) {
      if (!audioAmbienceTriples.has(`${value.id}|${segment}|${weather}`)) fail(audioFile, `no ambience record for ${value.id} at ${segment} in ${weather} weather`);
    }
  }
}
const audioCueCount = audioCueIds.size;
const audioAmbienceCount = audioAmbienceTriples.size;

// P3: `content/atmosphere/` -- the five tables the `Atmosphere` autoload reads
// to decide what the island's sky is. Every number the sky is made of lives
// here; the *names* those numbers are keyed by belong to Rust, because they are
// the bands and segments the simulation actually has.
//
// The three ladders below are written once here and once in Rust
// (`strategy::dungeon::CorruptionBand::ALL`, `HeatBand::ALL`,
// `strategy::clocks::WeatherCondition::ALL`), exactly as `bondRanks` above is
// written once here and once in `battle::rank_index` -- and, as there, a test
// holds them equal rather than a comment: `strategy::dungeon`'s
// `authored_atmosphere` module reads these very files and fails the build if a
// key here is not a band there. So this block checks what Rust cannot see --
// ranges, monotonicity, colour syntax and the presence of every field the
// autoload reads -- and does not re-litigate the names.
const atmosphereSegments = ["dawn", "day", "dusk", "midnight"];
const atmosphereWeather = ["clear", "overcast", "rain", "storm", "unnatural"];
const atmosphereCorruptionBands = ["untouched", "touched", "spreading", "consumed"];
const atmosphereHeatBands = ["dormant", "stirring", "rising", "imminent"];
const atmosphereParticleKinds = new Set(["none", "rain", "mist"]);
const atmosphereDurations = ["sun", "fog", "wind", "grade", "heat"];
let atmosphereRowCount = 0;

function atmosphereColour(record, value, file, where) {
  if (typeof value !== "string" || !/^#[0-9a-f]{6}$/.test(value)) fail(file, `${where} must be a #rrggbb colour the Godot Color constructor accepts`);
}
function atmosphereNumber(value, file, where, { min = -Infinity, max = Infinity } = {}) {
  if (typeof value !== "number" || !Number.isFinite(value)) fail(file, `${where} must be a finite number`);
  else if (value < min || value > max) fail(file, `${where} must be between ${min} and ${max}`);
}
function atmosphereRows(value, field, expectedKeys, file) {
  const rows = value[field];
  if (!rows || typeof rows !== "object" || Array.isArray(rows)) { fail(file, `${field} must be an object keyed by name`); return null; }
  const actual = Object.keys(rows).sort();
  const expected = [...expectedKeys].sort();
  if (actual.join(",") !== expected.join(",")) fail(file, `${field} must carry exactly ${expected.join(", ")}; found ${actual.join(", ") || "nothing"}`);
  return rows;
}
function atmosphereMetadata(value, file) {
  requireString(value.metadata ?? {}, "implementationOwner", file);
  requireString(value.metadata ?? {}, "presentationOwner", file);
  if (typeof value.metadata?.releaseLegal !== "boolean") fail(file, "metadata.releaseLegal must be boolean");
  if (!Array.isArray(value.metadata?.notes) || value.metadata.notes.length === 0) fail(file, "metadata.notes must say what this table is and what it deliberately does not decide");
  requireString(value, "displayName", file);
  requireString(value, "description", file);
}

const atmosphereRecordsById = new Map();
for (const { file, value } of await readJsonDirectory("atmosphere")) {
  atmosphereRecordsById.set(value.id, { file, value });
  if (typeof value.id !== "string" || !value.id.startsWith("atmosphere.")) fail(file, "id must be an atmosphere.<...> stable ID");
  atmosphereMetadata(value, file);

  if (value.id === "atmosphere.segment_table") {
    atmosphereNumber(value.hourFraction?.hoursPerSegment, file, "hourFraction.hoursPerSegment", { min: 1, max: 24 });
    const rows = atmosphereRows(value, "segments", atmosphereSegments, file);
    for (const [name, row] of Object.entries(rows ?? {})) {
      atmosphereRowCount += 1;
      atmosphereNumber(row?.sunElevationDegrees, file, `segments.${name}.sunElevationDegrees`, { min: -90, max: 90 });
      atmosphereNumber(row?.sunAzimuthDegrees, file, `segments.${name}.sunAzimuthDegrees`, { min: 0, max: 360 });
      // The kelvin is the only spelling of the sun's colour: atmosphere.gd
      // converts it once. An authored hex beside it would be a second answer.
      atmosphereNumber(row?.sunColourTemperatureKelvin, file, `segments.${name}.sunColourTemperatureKelvin`, { min: 1000, max: 20000 });
      if (row?.sunColour !== undefined) fail(file, `segments.${name} may not author a sunColour beside its kelvin; the temperature is the owner`);
      atmosphereNumber(row?.sunEnergy, file, `segments.${name}.sunEnergy`, { min: 0, max: 16 });
      atmosphereNumber(row?.ambientEnergy, file, `segments.${name}.ambientEnergy`, { min: 0, max: 16 });
      atmosphereNumber(row?.exposure, file, `segments.${name}.exposure`, { min: 0.1, max: 4 });
      for (const key of ["ambientColour", "skyHorizonColour", "skyTopColour"]) atmosphereColour(value, row?.[key], file, `segments.${name}.${key}`);
    }
  } else if (value.id === "atmosphere.weather_table") {
    const rows = atmosphereRows(value, "conditions", atmosphereWeather, file);
    for (const [name, row] of Object.entries(rows ?? {})) {
      atmosphereRowCount += 1;
      atmosphereNumber(row?.fogDensity, file, `conditions.${name}.fogDensity`, { min: 0, max: 1 });
      atmosphereNumber(row?.fogSkyAffect, file, `conditions.${name}.fogSkyAffect`, { min: 0, max: 1 });
      atmosphereNumber(row?.exposureScale, file, `conditions.${name}.exposureScale`, { min: 0.25, max: 1.5 });
      atmosphereColour(value, row?.fogColour, file, `conditions.${name}.fogColour`);
      atmosphereNumber(row?.sunEnergyScale, file, `conditions.${name}.sunEnergyScale`, { min: 0, max: 2 });
      atmosphereNumber(row?.ambientEnergyScale, file, `conditions.${name}.ambientEnergyScale`, { min: 0, max: 2 });
      atmosphereNumber(row?.glowIntensity, file, `conditions.${name}.glowIntensity`, { min: 0, max: 4 });
      atmosphereNumber(row?.windStrength, file, `conditions.${name}.windStrength`, { min: 0, max: 2 });
      atmosphereNumber(row?.particleIntensity, file, `conditions.${name}.particleIntensity`, { min: 0, max: 1 });
      if (!atmosphereParticleKinds.has(row?.particleKind)) fail(file, `conditions.${name}.particleKind must be one of ${[...atmosphereParticleKinds].join(", ")}`);
      // A kind with nothing to draw, or an intensity with nothing to draw it
      // with, is a row that says two different things about the same sky.
      if (row?.particleKind === "none" && row?.particleIntensity !== 0) fail(file, `conditions.${name} draws no particles but declares an intensity`);
      if (row?.particleKind !== "none" && !(row?.particleIntensity > 0)) fail(file, `conditions.${name} names a particle kind but never emits it`);
    }
    // The axis S8 authored, held: clear is the lightest sky and the corrupted
    // end is the heaviest, so the player reads the weather as evidence.
    const density = atmosphereWeather.map(name => rows?.[name]?.fogDensity ?? 0);
    for (let index = 1; index < density.length; index += 1) {
      if (!(density[index] > density[index - 1])) fail(file, `conditions.${atmosphereWeather[index]}.fogDensity must be thicker than ${atmosphereWeather[index - 1]}'s; the weather axis runs one way`);
    }
  } else if (value.id === "atmosphere.corruption_palette") {
    const rows = atmosphereRows(value, "bands", atmosphereCorruptionBands, file);
    const paletteNames = new Set();
    for (const [name, row] of Object.entries(rows ?? {})) {
      atmosphereRowCount += 1;
      requireString(row ?? {}, "paletteName", file);
      if (paletteNames.has(row?.paletteName)) fail(file, `bands.${name}.paletteName ${row?.paletteName} is used by another band; a grade's name is how a capture and a test say which one they mean`);
      else paletteNames.add(row?.paletteName);
      atmosphereNumber(row?.saturation, file, `bands.${name}.saturation`, { min: 0, max: 1 });
      atmosphereNumber(row?.tintStrength, file, `bands.${name}.tintStrength`, { min: 0, max: 1 });
      atmosphereNumber(row?.contrast, file, `bands.${name}.contrast`, { min: 0.5, max: 2 });
      atmosphereColour(value, row?.tintColour, file, `bands.${name}.tintColour`);
    }
    // Corruption only ever takes colour out. A band that read livelier than a
    // cleaner one would make the grade decorative instead of evidence.
    for (let index = 1; index < atmosphereCorruptionBands.length; index += 1) {
      const here = rows?.[atmosphereCorruptionBands[index]];
      const before = rows?.[atmosphereCorruptionBands[index - 1]];
      if (!(here?.saturation < before?.saturation)) fail(file, `bands.${atmosphereCorruptionBands[index]}.saturation must be lower than ${atmosphereCorruptionBands[index - 1]}'s`);
      if (!(here?.tintStrength > before?.tintStrength)) fail(file, `bands.${atmosphereCorruptionBands[index]}.tintStrength must be stronger than ${atmosphereCorruptionBands[index - 1]}'s`);
    }
  } else if (value.id === "atmosphere.heat_shift") {
    const rows = atmosphereRows(value, "bands", atmosphereHeatBands, file);
    const shiftNames = new Set();
    for (const [name, row] of Object.entries(rows ?? {})) {
      atmosphereRowCount += 1;
      requireString(row ?? {}, "shiftName", file);
      if (shiftNames.has(row?.shiftName)) fail(file, `bands.${name}.shiftName ${row?.shiftName} is used by another band`);
      else shiftNames.add(row?.shiftName);
      atmosphereNumber(row?.skyHueShiftDegrees, file, `bands.${name}.skyHueShiftDegrees`, { min: -60, max: 60 });
      atmosphereNumber(row?.ambientEnergyScale, file, `bands.${name}.ambientEnergyScale`, { min: 0.5, max: 1 });
      atmosphereNumber(row?.horizonDesaturation, file, `bands.${name}.horizonDesaturation`, { min: 0, max: 1 });
      // Brief section 13 keeps hidden pressure hidden. A band may carry a name
      // and a shift; it may never carry the number it was banded from.
      for (const forbidden of ["heat", "cthulhuHeat", "pressure", "score", "percent"]) {
        if (row?.[forbidden] !== undefined) fail(file, `bands.${name} may not carry a ${forbidden}; only the band's name crosses the bridge`);
      }
    }
    if (rows?.dormant?.skyHueShiftDegrees !== 0) fail(file, "bands.dormant.skyHueShiftDegrees must be 0; a world under no pressure is the sky everything else is read against");
  } else if (value.id === "atmosphere.transitions") {
    const durations = value.durationsSeconds;
    if (!durations || typeof durations !== "object" || Array.isArray(durations)) fail(file, "durationsSeconds must be an object");
    else {
      const actual = Object.keys(durations).sort();
      if (actual.join(",") !== [...atmosphereDurations].sort().join(",")) fail(file, `durationsSeconds must name exactly ${atmosphereDurations.join(", ")}`);
      for (const key of atmosphereDurations) atmosphereNumber(durations[key], file, `durationsSeconds.${key}`, { min: 0.05, max: 120 });
      atmosphereRowCount += atmosphereDurations.length;
      // The sky shift hidden pressure makes is the slowest thing on the
      // island: faster than the weather and it would read as weather.
      if (!(durations.heat > durations.fog)) fail(file, "durationsSeconds.heat must be slower than the weather's; pressure is felt before it is read");
    }
    atmosphereNumber(value.reducedMotion?.durationScale, file, "reducedMotion.durationScale", { min: 0.01, max: 1 });
    if (value.reducedMotion?.particlesStilled !== true) fail(file, "reducedMotion.particlesStilled must be true; B9's reduced motion stills the weather rather than removing it");
    atmosphereNumber(value.nightPointLight?.colourTemperatureKelvin, file, "nightPointLight.colourTemperatureKelvin", { min: 1000, max: 4000 });
    for (const [key, bounds] of [["energy", { min: 0, max: 32 }], ["rangeMetres", { min: 0.5, max: 200 }], ["heightMetres", { min: 0, max: 50 }], ["attenuation", { min: 0.1, max: 8 }]]) {
      atmosphereNumber(value.nightPointLight?.[key], file, `nightPointLight.${key}`, bounds);
    }
  } else {
    fail(file, `${value.id} is not one of the five tables the Atmosphere autoload reads; a sixth table would be a number with no reader`);
  }
}
for (const id of ["atmosphere.segment_table", "atmosphere.weather_table", "atmosphere.corruption_palette", "atmosphere.heat_shift", "atmosphere.transitions"]) {
  if (!atmosphereRecordsById.has(id)) failures.push(`content/atmosphere/: ${id} is missing; the Atmosphere autoload refuses to apply a sky without it`);
}

// An enemy *skill* is the one reference that still resolves outside this ID
// map. `content/skills/` authors the party's skills; an enemy's skill is
// declared inline by the creature record that uses it and by the habitat that
// advertises it, and battle.rs owns what it does, so `skill.enemy.` has no
// registered record to point at.
//
// `habitat.` used to sit beside it. B7 put it there because a habitat record
// could not be authored honestly yet -- the roster named creatures content did
// not declare, and the records that existed carried a `.prototype` suffix the
// registry never used. C15 closed both, so `content/habitats/` now registers
// every `habitat.*` ID and a world cell's `battleEntries[].habitatId` resolves
// against a record like any other reference.
const intentionallyExternalPrefixes = ["skill.enemy."];
for (const item of references) {
  if (!ids.has(item.id) && !intentionallyExternalPrefixes.some(prefix => item.id.startsWith(prefix))) {
    fail(item.file, `${item.field} references missing stable ID ${item.id}`);
  }
}

if (failures.length) {
  console.error(`Project 42 validation failed with ${failures.length} issue(s):`);
  for (const message of failures) console.error(`- ${message}`);
  process.exit(1);
}
console.log(`Project 42 content valid: ${ids.size} stable IDs checked; ${skillCount} skills, ${enemyCount} creature records, ${habitatCount} habitats, ${buildingCount} building records, ${machineCount} machine records, ${atmosphereRecordsById.size} atmosphere tables carrying ${atmosphereRowCount} rows, ${dungeonSpaceCount} authored dungeon spaces, ${siteRuleCount} site rules, ${relationshipSceneCount} relationship scenes, ${packCount} presentation packs carrying ${packOverrideCount} scene overrides, ${presentationCueCount} presentation cues, ${reelPlan.reels.length} video reels, ${creaturePlateCount} creature still-image plates, ${(sharedAssetLedger.assetRecords ?? []).length} shared asset records and ${(sharedSourceCollections.sourceCollections ?? []).length} source collections validated; ${placeholderManifest.assets.length} placeholders explicitly tracked; ${audioCueCount} audio cues and ${audioAmbienceCount} ambience triples resolve, ${audioPlaceholderCount} audio records still procedural placeholders.`);
