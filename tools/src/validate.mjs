import { readFile, readdir } from "node:fs/promises";
import { resolve, relative } from "node:path";
import process from "node:process";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];
const ids = new Map();

function fail(file, message) { failures.push(`${relative(repo, file)}: ${message}`); }
function requireString(record, key, file) {
  if (typeof record[key] !== "string" || record[key].trim() === "") fail(file, `${key} must be a non-empty string`);
}
function registerId(id, file) {
  if (typeof id !== "string") return fail(file, "id must be a string");
  if (ids.has(id)) fail(file, `duplicate stable ID ${id}; first declared in ${relative(repo, ids.get(id))}`);
  else ids.set(id, file);
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
  if (!Array.isArray(value.skillIds) || value.skillIds.length === 0) fail(file, "skillIds must not be empty");
  if (!value.art?.accessibilityDescription || value.art.accessibilityDescription.length < 20) fail(file, "art accessibilityDescription is missing or too short");
  if (value.art?.status === "placeholder" && value.metadata?.releaseLegal !== false) fail(file, "placeholder character must set metadata.releaseLegal=false");
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

if (failures.length) {
  console.error(`Project 42 validation failed with ${failures.length} issue(s):`);
  for (const message of failures) console.error(`- ${message}`);
  process.exit(1);
}
console.log(`Project 42 content valid: ${ids.size} stable IDs checked; ${placeholderManifest.assets.length} placeholders explicitly tracked.`);

