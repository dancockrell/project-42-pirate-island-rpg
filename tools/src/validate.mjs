import { readFile, readdir } from "node:fs/promises";
import { resolve, relative } from "node:path";
import process from "node:process";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];
const ids = new Map();
const references = [];
const supportedTargetRules = new Set(["one_hostile", "one_living_hostile", "one_living_party_member", "ordered_pair_threatened_ally_then_hostile", "automatic_reaction_to_other_party_member_lethal_hit", "all_living_party_members", "one_defeated_party_member_other_than_betty"]);
const bindableBattleEvents = new Set(["actor_focused", "actor_moved", "damage_applied", "guard_changed", "vitality_changed", "status_removed", "interception_set", "interception_triggered", "reaction_window_opened", "reaction_triggered", "defeat_prevented", "actor_revived", "bonus_turn_granted", "battlefield_effect_created", "battlefield_effect_pulse", "battlefield_effect_removed", "actor_defeated", "turn_ended", "battle_ended"]);
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
  if (!Array.isArray(value.skillIds) || value.skillIds.length !== 7) fail(file, "a heroine must declare exactly seven D-through-SSS skills");
  else value.skillIds.forEach((id, index) => reference(id, file, `skillIds[${index}]`));
  if (!value.art?.accessibilityDescription || value.art.accessibilityDescription.length < 20) fail(file, "art accessibilityDescription is missing or too short");
  if (value.art?.readyIdleMotionId) reference(value.art.readyIdleMotionId, file, "art.readyIdleMotionId");
  if (value.art?.status === "placeholder" && value.metadata?.releaseLegal !== false) fail(file, "placeholder character must set metadata.releaseLegal=false");
}

for (const { file, value } of await readJsonDirectory("skills")) {
  skillCount += 1;
  requireString(value, "displayName", file);
  reference(value.ownerId, file, "ownerId");
  if (!new Set(["D", "C", "B", "A", "S", "SS", "SSS"]).has(value.bondRank)) fail(file, "bondRank must be D, C, B, A, S, SS or SSS");
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
        for (const field of ["beat", "pose", "motion", "cameraId", "vfxId", "audio"]) {
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

for (const { file, value } of await readJsonDirectory("enemies")) {
  requireString(value, "displayName", file);
  if (value.worldPresence?.penAllowed !== false) fail(file, "island monsters may not be designed as pen exhibits");
  if (value.worldPresence?.packSize !== 1) fail(file, "prototype enemies must be tuned as individual threats");
  for (const [index, id] of (value.skillIds ?? []).entries()) reference(id, file, `skillIds[${index}]`);
}

for (const { file, value } of await readJsonDirectory("encounters")) {
  for (const [index, id] of (value.partyActorIds ?? []).entries()) reference(id, file, `partyActorIds[${index}]`);
  for (const [index, id] of (value.hostileActorIds ?? []).entries()) reference(id, file, `hostileActorIds[${index}]`);
  if (value.presentation?.inactivePartyMode !== "card_rail" || value.presentation?.activeActorMode !== "full_body_battle_plane") fail(file, "encounter must preserve the card-to-active combat contract");
}

for (const { file, value } of await readJsonDirectory("world")) {
  if (value.trigger !== "midnight_flash") fail(file, "spawn rule trigger must be midnight_flash");
  if (value.grouping?.designRule !== "individual_threat") fail(file, "daily spawns must use individual_threat tuning");
  for (const [index, id] of (value.definitionIds ?? []).entries()) reference(id, file, `definitionIds[${index}]`);
}

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
    for (const [index, entry] of (value.entries ?? []).entries()) {
      registerId(entry.id, file);
      if (!entry.id?.startsWith("presentation.vfx.")) fail(file, `entries[${index}].id must use presentation.vfx prefix`);
      for (const field of ["purpose", "anchor", "motion"]) requireString(entry, field, file);
      if (!Array.isArray(entry.palette) || entry.palette.length === 0 || entry.palette.some(color => typeof color !== "string" || color.length === 0)) fail(file, `${entry.id} palette must contain at least one named color`);
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
  } else {
    fail(file, `unsupported presentation registry kind ${value.kind}`);
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

const stillPlanFile = resolve(repo, "content/art/still_image_plan.json");
const stillPlan = JSON.parse(await readFile(stillPlanFile, "utf8"));
registerId(stillPlan.id, stillPlanFile);
if (stillPlan.productionSkill !== "magnific-2d-identity-sheet") fail(stillPlanFile, "productionSkill must name the Magnific identity-sheet contract");
for (const [index, plate] of (stillPlan.plates ?? []).entries()) {
  registerId(plate.id, stillPlanFile);
  for (const field of ["type", "prompt", "status"]) requireString(plate, field, stillPlanFile);
  if (!plate.id?.startsWith("art.plate.")) fail(stillPlanFile, `plates[${index}].id must use art.plate prefix`);
  if (forbiddenPromptRationale.test(plate.prompt ?? "")) fail(stillPlanFile, `${plate.id} prompt contains parser-confusing rationale`);
  if ((plate.prompt ?? "").length < 300) fail(stillPlanFile, `${plate.id} prompt is too short to be a production contract`);
  if (!Array.isArray(plate.sourceAnchors)) fail(stillPlanFile, `${plate.id} sourceAnchors must be an array`);
  if (!Array.isArray(plate.consumers) || plate.consumers.length === 0) fail(stillPlanFile, `${plate.id} must name at least one placeholder consumer`);
  else plate.consumers.forEach((id, consumerIndex) => reference(id, stillPlanFile, `plates[${index}].consumers[${consumerIndex}]`));
}

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
console.log(`Project 42 content valid: ${ids.size} stable IDs checked; ${skillCount} skills, ${presentationCueCount} presentation cues, ${reelPlan.reels.length} video reels and ${stillPlan.plates.length} still-image plates validated; ${placeholderManifest.assets.length} placeholders explicitly tracked.`);
