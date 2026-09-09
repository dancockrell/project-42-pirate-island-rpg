// Scenario pack loading: one owner for finding packs and resolving the documents
// a manifest references. tools/src/validate.mjs proves the rules over what this
// returns; tools/src/build-content-bundle.mjs embeds what this returns.
// Contract: docs/SCENARIO_PACKS.md.
import { readFile, readdir } from "node:fs/promises";
import { isAbsolute, relative, resolve, sep } from "node:path";

export const scenarioDocumentFields = [
  ["geography.navigation", manifest => manifest.geography?.navigation],
  ["buildings", manifest => manifest.buildings],
  ["personas", manifest => manifest.personas]
];

/**
 * Manifest keys that name a document only when the pack carries one. A pack with
 * no item catalogue leaves `items` as the contract's empty array, so the key is
 * a declared path only when its value is a string.
 * Contract: docs/SCENARIO_PACKS.md, "Items".
 */
export const optionalScenarioDocumentFields = [["items", manifest => manifest.items]];

/** Every `<field, declared path>` pair a manifest carries, in a stable order. */
export function declaredPaths(manifest) {
  const declared = [];
  for (const [field, read] of scenarioDocumentFields) declared.push({ field, path: read(manifest) });
  for (const [field, read] of optionalScenarioDocumentFields) {
    if (typeof read(manifest) === "string") declared.push({ field, path: read(manifest) });
  }
  for (const [index, faction] of (manifest.factions ?? []).entries()) {
    declared.push({ field: `factions[${index}].tuning`, path: faction?.tuning });
    declared.push({ field: `factions[${index}].production`, path: faction?.production });
  }
  for (const [index, path] of (Array.isArray(manifest.resources) ? manifest.resources : []).entries()) {
    declared.push({ field: `resources[${index}]`, path });
  }
  for (const [index, path] of (Array.isArray(manifest.triggers) ? manifest.triggers : []).entries()) {
    declared.push({ field: `triggers[${index}]`, path });
  }
  for (const name of Object.keys(manifest.rules ?? {}).sort()) {
    declared.push({ field: `rules.${name}`, path: manifest.rules[name] });
  }
  return declared;
}

/**
 * Read one pack. Returns the manifest and, for every declared path, either the
 * parsed document or the reason it could not be read. Nothing is validated here.
 */
export async function loadScenarioPack(packDirectory) {
  const manifestFile = resolve(packDirectory, "scenario.json");
  let manifest;
  try {
    manifest = JSON.parse(await readFile(manifestFile, "utf8"));
  } catch (cause) {
    return { directory: packDirectory, manifestFile, manifest: {}, documents: [], manifestError: `scenario.json is not a readable JSON manifest (${cause.code ?? cause.message})` };
  }
  const documents = [];
  for (const { field, path } of declaredPaths(manifest)) {
    const entry = { field, path, file: null, value: null, error: null };
    if (typeof path !== "string" || path.trim() === "") entry.error = "must name a document path relative to the pack directory";
    else if (isAbsolute(path) || path.includes("://")) entry.error = `path ${path} must be relative to the pack directory`;
    else {
      entry.file = resolve(packDirectory, path);
      try {
        entry.value = JSON.parse(await readFile(entry.file, "utf8"));
      } catch (cause) {
        entry.error = `path ${path} does not resolve to a readable JSON document (${cause.code ?? cause.message})`;
      }
    }
    documents.push(entry);
  }
  return { directory: packDirectory, manifestFile, manifest, documents, manifestError: null };
}

/** Every pack directly under `root`, sorted, or an empty list when there is no root. */
export async function listScenarioPackDirectories(root) {
  let entries;
  try {
    entries = await readdir(root, { withFileTypes: true });
  } catch (cause) {
    if (cause.code === "ENOENT") return [];
    throw cause;
  }
  return entries.filter(entry => entry.isDirectory()).map(entry => resolve(root, entry.name)).sort();
}

/** Load every pack under every root, in root then directory order. */
export async function loadScenarioPacks(roots) {
  const packs = [];
  for (const root of roots) {
    for (const directory of await listScenarioPackDirectories(root)) packs.push(await loadScenarioPack(directory));
  }
  return packs;
}

/** The scenario roots a CLI run covers: content/scenarios plus any --scenario-root. */
export function scenarioRootsFromArgv(repo, argv) {
  const roots = [resolve(repo, "content/scenarios")];
  for (const [index, argument] of argv.entries()) {
    if (argument === "--scenario-root") {
      const value = argv[index + 1];
      if (typeof value !== "string") throw new Error("--scenario-root requires a directory");
      roots.push(resolve(value));
    }
  }
  return roots;
}

/** A pack label for messages: repository-relative where possible, absolute otherwise. */
export function packLabel(repo, packDirectory) {
  const relativePath = relative(repo, packDirectory);
  return relativePath.startsWith(`..${sep}`) || isAbsolute(relativePath) ? packDirectory : relativePath;
}

/**
 * The manifest with every declared path replaced by the document it names, so a
 * consumer reads `geography.navigation`, `rules.<name>`, `factions[n].tuning`
 * and `factions[n].production` as documents rather than as paths.
 */
export function resolveScenarioManifest(pack) {
  if (pack.manifestError) throw new Error(`${pack.manifestFile}: ${pack.manifestError}`);
  const resolved = structuredClone(pack.manifest);
  for (const document of pack.documents) {
    if (document.error) throw new Error(`${pack.manifestFile}: ${document.field} ${document.error}`);
    const segments = document.field.split(".");
    let target = resolved;
    for (const [index, segment] of segments.entries()) {
      const match = /^(\w+)\[(\d+)\]$/.exec(segment);
      const key = match ? match[1] : segment;
      const last = index === segments.length - 1;
      if (match) {
        if (last) target[key][Number(match[2])] = document.value;
        else target = target[key][Number(match[2])];
      } else if (last) target[key] = document.value;
      else target = target[key];
    }
  }
  return resolved;
}
