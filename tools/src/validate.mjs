import { readFile, readdir } from "node:fs/promises";
import { resolve, relative } from "node:path";
import process from "node:process";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];
const ids = new Map();
const references = [];
const supportedTargetRules = new Set(["one_hostile", "one_living_hostile", "one_living_party_member", "ordered_pair_threatened_ally_then_hostile", "automatic_reaction_to_other_party_member_lethal_hit", "all_living_party_members", "one_defeated_party_member_other_than_betty"]);
const bindableBattleEvents = new Set(["actor_focused", "actor_moved", "damage_applied", "guard_changed", "vitality_changed", "status_removed", "interception_set", "interception_triggered", "reaction_window_opened", "reaction_triggered", "defeat_prevented", "actor_revived", "bonus_turn_granted", "battlefield_effect_created", "battlefield_effect_pulse", "battlefield_effect_removed", "recovery_opening_created", "recovery_opening_consumed", "recovery_opening_expired", "actor_defeated", "turn_ended", "battle_ended"]);
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

for (const { file, value } of await readJsonDirectory("encounters")) {
  for (const [index, id] of (value.partyActorIds ?? []).entries()) reference(id, file, `partyActorIds[${index}]`);
  for (const [index, id] of (value.hostileActorIds ?? []).entries()) reference(id, file, `hostileActorIds[${index}]`);
  // A2: an encounter names real places. These two fields carried a third
  // location namespace that matched no world cell, which is exactly the drift
  // that resolving them against the registered stable IDs now makes impossible.
  reference(value.locationId, file, "locationId");
  reference(value.defeat?.returnLocationId, file, "defeat.returnLocationId");
  if (value.presentation?.inactivePartyMode !== "card_rail" || value.presentation?.activeActorMode !== "full_body_battle_plane") fail(file, "encounter must preserve the card-to-active combat contract");
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
    if (!Array.isArray(value.portals) || value.portals.length === 0) fail(file, "world cell requires one or more explicit portals");
    else for (const [index, portal] of value.portals.entries()) {
      for (const field of ["id", "fromAnchorId", "targetAnchorId", "travelMode", "returnRule"]) requireString(portal, field, file);
      reference(portal.targetCellId, file, `portals[${index}].targetCellId`);
      if (!entryAnchorIds.has(portal.fromAnchorId)) fail(file, `portals[${index}].fromAnchorId must name an entry anchor in this cell`);
      if (!new Set(["on_foot", "safe_road", "jungle_edge"]).has(portal.travelMode)) fail(file, `portals[${index}].travelMode is unsupported`);
      if (!new Set(["always", "allowed_while_no_pending_encounter"]).has(portal.returnRule)) fail(file, `portals[${index}].returnRule is unsupported`);
    }
    if (!Array.isArray(value.battleEntries) || value.battleEntries.length === 0) fail(file, "world cell requires one or more battle entries");
    else for (const [index, entry] of value.battleEntries.entries()) {
      requireString(entry, "id", file);
      reference(entry.encounterId, file, `battleEntries[${index}].encounterId`);
      for (const field of ["returnAnchorId", "safeRetreatAnchorId"]) requireString(entry, field, file);
      if (!entryAnchorIds.has(entry.returnAnchorId)) fail(file, `battleEntries[${index}].returnAnchorId must name an entry anchor in this cell`);
      if (!entryAnchorIds.has(entry.safeRetreatAnchorId)) fail(file, `battleEntries[${index}].safeRetreatAnchorId must name an entry anchor in this cell`);
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
      if (typeof description.text !== "string" || description.text.length < 90) fail(file, `readableDescriptions[${index}].text must be at least 90 characters`);
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
console.log(`Project 42 content valid: ${ids.size} stable IDs checked; ${skillCount} skills, ${presentationCueCount} presentation cues, ${reelPlan.reels.length} video reels, ${creaturePlateCount} creature still-image plates, ${(sharedAssetLedger.assetRecords ?? []).length} shared asset records and ${(sharedSourceCollections.sourceCollections ?? []).length} source collections validated; ${placeholderManifest.assets.length} placeholders explicitly tracked.`);
