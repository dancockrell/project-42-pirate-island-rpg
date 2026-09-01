import { readFile, readdir } from "node:fs/promises";
import { resolve, relative } from "node:path";
import process from "node:process";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];
const ids = new Map();
const references = [];

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
  if (value.art?.status === "placeholder" && value.metadata?.releaseLegal !== false) fail(file, "placeholder character must set metadata.releaseLegal=false");
}

for (const { file, value } of await readJsonDirectory("skills")) {
  requireString(value, "displayName", file);
  reference(value.ownerId, file, "ownerId");
  if (!new Set(["D", "C", "B", "A", "S", "SS", "SSS"]).has(value.bondRank)) fail(file, "bondRank must be D, C, B, A, S, SS or SSS");
  if (!value.rules || typeof value.rules !== "object") fail(file, "rules must be an object");
  if (!Array.isArray(value.animation?.beats) || value.animation.beats.length < 4) fail(file, "animation must contain at least four explicit beats");
  if (!value.animation?.framing?.includes("safe frame")) fail(file, "animation framing must state its safe-frame requirement");
  if (value.id === "skill.betty.condition_cleanse" && JSON.stringify(value.rules?.removalPriority) !== JSON.stringify(["stunned", "burning", "poisoned", "bleeding"])) {
    fail(file, "Condition Cleanse must preserve its deterministic status-removal priority");
  }
  if (value.id === "skill.betty.rescue_charge" && value.targetRule !== "ordered_pair_threatened_ally_then_hostile") {
    fail(file, "Rescue Charge must name the rescued ally first and threatening hostile second");
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
console.log(`Project 42 content valid: ${ids.size} stable IDs checked; ${placeholderManifest.assets.length} placeholders explicitly tracked.`);
