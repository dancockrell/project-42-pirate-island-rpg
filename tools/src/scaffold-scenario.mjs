// node tools/src/scaffold-scenario.mjs <scenario-id> [--out <directory>]
//
// Writes a scenario pack skeleton that already validates: reserved arrays empty,
// the main scenario's rule, buildings, personas and navigation documents
// referenced by relative path, and two factions with their seeds and tuning.
// Every value is taken from the committed main scenario pack, so the skeleton
// invents no numbers. Contract: docs/SCENARIO_PACKS.md.
import { mkdir, writeFile } from "node:fs/promises";
import { relative, resolve } from "node:path";
import process from "node:process";
import { loadScenarioPack } from "./scenario-pack.mjs";

const repo = resolve(import.meta.dirname, "../..");
const argv = process.argv.slice(2);
const outIndex = argv.indexOf("--out");
const positional = argv.filter((argument, index) => outIndex === -1 || (index !== outIndex && index !== outIndex + 1));
const scenarioId = positional[0];
if (!scenarioId || !/^scenario\.[a-z0-9_]+$/.test(scenarioId)) {
  console.error("usage: node tools/src/scaffold-scenario.mjs <scenario-id> [--out <directory>]");
  console.error("       <scenario-id> must be a stable ID of the form scenario.<name>");
  process.exit(2);
}
const name = scenarioId.slice("scenario.".length);
const packDirectory = outIndex === -1 ? resolve(repo, "content/scenarios", name) : resolve(argv[outIndex + 1] ?? "", name);

const template = await loadScenarioPack(resolve(repo, "content/scenarios/pirate_island"));
const broken = template.documents.find(document => document.error);
if (broken) {
  console.error(`the main scenario pack is not readable: ${broken.field} ${broken.error}`);
  process.exit(1);
}
const documentFile = field => template.documents.find(document => document.field === field).file;
const documentValue = field => template.documents.find(document => document.field === field).value;
const packRelative = file => relative(packDirectory, file).split("\\").join("/");

const factions = [];
const tuningFiles = new Map();
for (const [index, faction] of template.manifest.factions.slice(0, 2).entries()) {
  const factionName = faction.id.replace(/^faction\./, "").replace(/\.prototype$/, "");
  tuningFiles.set(`factions/${factionName}.tuning.json`, documentValue(`factions[${index}].tuning`));
  factions.push({
    id: faction.id,
    seed: faction.seed,
    tuning: `factions/${factionName}.tuning.json`,
    production: packRelative(documentFile(`factions[${index}].production`))
  });
}

const rules = {};
for (const key of Object.keys(template.manifest.rules)) rules[key] = packRelative(documentFile(`rules.${key}`));

const manifest = {
  schemaVersion: 1,
  id: scenarioId,
  title: name.replace(/_/g, " ").replace(/\b[a-z]/g, letter => letter.toUpperCase()),
  summary: `Scaffolded scenario pack ${scenarioId}; edit every field before playing it.`,
  geography: {
    terrainTexture: template.manifest.geography.terrainTexture,
    navigation: packRelative(documentFile("geography.navigation"))
  },
  start: structuredClone(template.manifest.start),
  factions,
  rules,
  buildings: packRelative(documentFile("buildings")),
  personas: packRelative(documentFile("personas")),
  resources: [],
  items: [],
  triggers: [],
  quests: []
};

await mkdir(resolve(packDirectory, "factions"), { recursive: true });
for (const [path, value] of tuningFiles) await writeFile(resolve(packDirectory, path), `${JSON.stringify(value, null, 2)}\n`, "utf8");
await writeFile(resolve(packDirectory, "scenario.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`Scaffolded ${scenarioId} into ${packDirectory} with ${factions.length} factions and every reserved array empty.`);
console.log(`Validate it with: node tools/src/validate.mjs --scenario-root ${resolve(packDirectory, "..")}`);
