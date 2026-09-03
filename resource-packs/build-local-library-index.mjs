import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { join, relative, resolve } from "node:path";

const root = resolve(import.meta.dirname);
const repo = resolve(root, "..");
const sourceRoot = resolve(root, "source/cc0/kenney");
const out = resolve(root, "library.local.json");

const packs = {
  "nature-kit-2.1": { title: "Nature Kit 2.1", source: "https://kenney.nl/assets/nature-kit", ledger: "asset.shared.kenney.nature-kit.2_1.source", vendorFolder: "nature-kit", styleTags: ["fantasy-neutral", "tabletop"], defaultType: "flora" },
  "building-kit": { title: "Building Kit", source: "https://kenney.nl/assets/building-kit", ledger: "asset.shared.kenney.building-kit.source", styleTags: ["fantasy-neutral", "modular", "tabletop"], defaultType: "structures" },
  "survival-kit": { title: "Survival Kit", source: "https://kenney.nl/assets/survival-kit", ledger: "asset.shared.kenney.survival-kit.source", styleTags: ["wilderness", "adventure", "tabletop"], defaultType: "props" },
  "modular-cave-kit": { title: "Modular Cave Kit", source: "https://kenney.nl/assets/modular-cave-kit", ledger: "asset.shared.kenney.modular-cave-kit.1_0.source", styleTags: ["cave", "modular", "fantasy-neutral"], defaultType: "geology" },
  "modular-dungeon-kit": { title: "Modular Dungeon Kit", source: "https://kenney.nl/assets/modular-dungeon-kit", ledger: "asset.shared.kenney.modular-dungeon-kit.1_0.source", styleTags: ["dungeon", "modular", "fantasy-neutral"], defaultType: "structures" },
  "furniture-kit": { title: "Furniture Kit", source: "https://kenney.nl/assets/furniture-kit", styleTags: ["interior", "neutral", "low-poly"], defaultType: "furnishings" },
  "mini-forest": { title: "Mini Forest", source: "https://kenney.nl/assets/mini-forest", styleTags: ["miniature", "forest", "tabletop"], defaultType: "flora" },
  "mini-dungeon": { title: "Mini Dungeon", source: "https://kenney.nl/assets/mini-dungeon", styleTags: ["miniature", "dungeon", "tabletop"], defaultType: "structures" },
  "modular-buildings": { title: "Modular Buildings", source: "https://kenney.nl/assets/modular-buildings", styleTags: ["modular", "settlement", "low-poly"], defaultType: "structures" },
  "pirate-kit": { title: "Pirate Kit", source: "https://kenney.nl/assets/pirate-kit", styleTags: ["maritime", "island", "fantasy-adventure"], defaultType: "maritime" },
  "cube-pets": { title: "Cube Pets", source: "https://kenney.nl/assets/cube-pets", styleTags: ["cute", "animated", "toy-like"], defaultType: "fauna" },
  "factory-kit": { title: "Factory Kit 3.0", source: "https://kenney.nl/assets/factory-kit", styleTags: ["modern", "industrial", "modular"], defaultType: "industrial" },
  "space-kit": { title: "Space Kit", source: "https://kenney.nl/assets/space-kit", styleTags: ["sci-fi", "space", "low-poly"], defaultType: "sci-fi" },
  "modular-space-kit": { title: "Modular Space Kit 1.0", source: "https://kenney.nl/assets/modular-space-kit", styleTags: ["sci-fi", "space-station", "modular"], defaultType: "sci-fi" },
  "city-kit-commercial": { title: "City Kit Commercial 2.1", source: "https://kenney.nl/assets/city-kit-commercial", styleTags: ["modern", "commercial-city", "low-poly"], defaultType: "structures" },
  "car-kit": { title: "Car Kit", source: "https://kenney.nl/assets/car-kit", styleTags: ["modern", "vehicles", "low-poly"], defaultType: "vehicles" },
  "watercraft-kit": { title: "Watercraft Kit", source: "https://kenney.nl/assets/watercraft-kit", styleTags: ["maritime", "transport", "low-poly"], defaultType: "maritime" },
  "train-kit": { title: "Train Kit", source: "https://kenney.nl/assets/train-kit", styleTags: ["modern", "transport", "low-poly"], defaultType: "vehicles" },
  "city-kit-roads": { title: "City Kit Roads", source: "https://kenney.nl/assets/city-kit-roads", styleTags: ["modern", "city", "roads", "modular"], defaultType: "routes" },
  "city-kit-suburban": { title: "City Kit Suburban 2.0", source: "https://kenney.nl/assets/city-kit-suburban", styleTags: ["modern", "suburban", "city"], defaultType: "structures" },
  "castle-kit": { title: "Castle Kit 2.0", source: "https://kenney.nl/assets/castle-kit", styleTags: ["medieval", "fortification", "fantasy-neutral", "tabletop"], defaultType: "structures" },
  "retro-fantasy-kit": { title: "Retro Fantasy Kit 2.0", source: "https://kenney.nl/assets/retro-fantasy-kit", styleTags: ["fantasy", "medieval", "settlement", "low-poly"], defaultType: "structures" },
  "prototype-kit": { title: "Prototype Kit", source: "https://kenney.nl/assets/prototype-kit", styleTags: ["prototype", "neutral", "animated", "low-poly"], defaultType: "props" },
  "mini-characters": { title: "Mini Characters", source: "https://kenney.nl/assets/mini-characters", styleTags: ["characters", "animated", "placeholder", "low-poly"], defaultType: "rig" },
  "blocky-characters": { title: "Blocky Characters 2.0", source: "https://kenney.nl/assets/blocky-characters", styleTags: ["characters", "animated", "placeholder", "low-poly"], defaultType: "rig" },
  "platformer-kit": { title: "Platformer Kit 4.1", source: "https://kenney.nl/assets/platformer-kit", styleTags: ["prototype", "characters", "platformer", "low-poly"], defaultType: "props" },
  "racing-kit": { title: "Racing Kit", source: "https://kenney.nl/assets/racing-kit", styleTags: ["modern", "transport", "routes", "low-poly"], defaultType: "vehicles" },
  "animated-characters-retro": { title: "Animated Characters Retro", source: "https://kenney.nl/assets/animated-characters-retro", styleTags: ["characters", "rig", "animation-reference", "low-poly"], defaultType: "rig" },
  "animated-characters-protagonists": { title: "Animated Characters Protagonists", source: "https://kenney.nl/assets/animated-characters-protagonists", styleTags: ["characters", "rig", "animation-reference", "low-poly"], defaultType: "rig" },
  "animated-characters-survivors": { title: "Animated Characters Survivors", source: "https://kenney.nl/assets/animated-characters-survivors", styleTags: ["characters", "rig", "animation-reference", "low-poly"], defaultType: "rig" },
  "food-kit": { title: "Food Kit 2.0", source: "https://kenney.nl/assets/food-kit", styleTags: ["food", "interior", "market", "low-poly"], defaultType: "props" },
  "tower-defense-kit": { title: "Tower Defense Kit 2.1", source: "https://kenney.nl/assets/tower-defense-kit", styleTags: ["medieval", "defense", "routes", "low-poly"], defaultType: "structures" },
  "blaster-kit": { title: "Blaster Kit 2.1", source: "https://kenney.nl/assets/blaster-kit", styleTags: ["sci-fi", "weapons", "props", "low-poly"], defaultType: "props" }
};

const sourcePackRecords = {};
for (const [packId, pack] of Object.entries(packs)) {
  const vendorFolder = resolve(repo, "work/art/vendor/kenney", pack.vendorFolder ?? packId);
  const archiveName = (await readdir(vendorFolder)).find(name => name.endsWith(".zip"));
  if (!archiveName) throw new Error(`Missing source archive for ${packId} in ${vendorFolder}`);
  const archive = await readFile(resolve(vendorFolder, archiveName));
  sourcePackRecords[packId] = {
    ...pack,
    authoringLineage: "source_cc0",
    license: "CC0-1.0",
    retrievedOn: "2026-09-03",
    archive: {
      localPath: relative(repo, resolve(vendorFolder, archiveName)).replaceAll("\\", "/"),
      filename: archiveName,
      bytes: archive.length,
      sha256: createHash("sha256").update(archive).digest("hex")
    }
  };
}

function classify(filename, pack) {
  const value = filename.toLowerCase();
  if (/(tree|plant|grass|flower|crop|mushroom|moss|bush|lily|vine|leaf|stump)/.test(value)) return ["flora", "vegetation"];
  if (/(rock|stone|cliff|ground|platform|cave|crystal)/.test(value)) return ["geology", "terrain"];
  if (/(bridge|path|road|stairs|fence|gate|dock|pier)/.test(value)) return ["routes", "infrastructure"];
  if (/(house|wall|roof|door|window|tower|building|barricade|fort|corridor|room)/.test(value)) return ["structures", "architecture"];
  if (/(bed|chair|table|shelf|cabinet|drawer|lamp|chest|barrel|pot|bench|sofa|bookcase)/.test(value)) return ["furnishings", "interior-prop"];
  if (/(ship|boat|canoe|anchor|mast|sail)/.test(value)) return ["maritime", "travel"];
  if (/(dog|cat|pet|horse|animal|bird)/.test(value)) return ["fauna", "creature"];
  return [pack.defaultType, `${pack.defaultType}-source`];
}

const assets = [];
const sourceExtensions = new Set([".glb", ".fbx"]);
for (const [packId, pack] of Object.entries(packs)) {
  const folder = resolve(sourceRoot, packId);
  for (const filename of (await readdir(folder)).filter(name => sourceExtensions.has(name.slice(name.lastIndexOf(".")).toLowerCase())).sort()) {
    const bytes = await readFile(resolve(folder, filename));
    const [physicalType, family] = classify(filename, pack);
    const sourceFormat = filename.slice(filename.lastIndexOf(".")).slice(1).toLowerCase();
    assets.push({
      id: `resource.source.cc0.kenney.${packId}.${filename.slice(0, -4).replaceAll(/[^a-zA-Z0-9]+/g, "-").replaceAll(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}.${sourceFormat}`,
      path: relative(repo, resolve(folder, filename)).replaceAll("\\", "/"),
      sourcePack: packId,
      sourceLedgerId: pack.ledger ?? null,
      sourceArchiveSha256: sourcePackRecords[packId].archive.sha256,
      authoringLineage: "source_cc0",
      license: "CC0-1.0",
      physicalType,
      family,
      sourceFormat,
      tags: ["geometry", sourceFormat, "kenney", "cc0", "low-poly", "tabletop-compatible", packId, physicalType, family, ...pack.styleTags],
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      reviewState: "unreviewed_local_source"
    });
  }
}
const sourceAssetsByPath = new Map(assets.map(asset => [asset.path, asset]));
const derivativeRoot = resolve(root, "geometry");
for (const relativePackPath of (await readdir(derivativeRoot, { recursive: true })).filter(path => path.endsWith("resource-pack.json")).sort()) {
  const packPath = join(derivativeRoot, relativePackPath);
  const pack = JSON.parse(await readFile(packPath, "utf8"));
  const packDirectory = resolve(packPath, "..");
  const [physicalType = "derivative", family = "derived-prop"] = pack.search?.physicalType ?? [];
  const derivativeTags = [...new Set(["geometry", "glb", "derivative", "local-authored", physicalType, family, ...(pack.search?.style ?? []), ...(pack.search?.setting ?? []), ...(pack.search?.technical ?? [])])];
  for (const member of pack.assets ?? []) {
    const derivativeFile = resolve(packDirectory, member.derivative);
    const bytes = await readFile(derivativeFile);
    const source = sourceAssetsByPath.get(member.source);
    assets.push({
      id: `${pack.id}.${member.id}`,
      path: relative(repo, derivativeFile).replaceAll("\\", "/"),
      sourcePath: member.source,
      sourcePack: source?.sourcePack ?? null,
      sourceArchiveSha256: source?.sourceArchiveSha256 ?? null,
      authoringLineage: pack.authoring?.lineage ?? "project_derivative",
      derivativeKind: pack.authoring?.derivativeKind ?? "unspecified",
      license: pack.authoring?.sourceLicense ?? "unknown",
      physicalType,
      family,
      tags: derivativeTags,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      reviewState: pack.status ?? "local_draft"
    });
  }
}
assets.sort((a, b) => a.id.localeCompare(b.id));
if (new Set(assets.map(asset => asset.id)).size !== assets.length) throw new Error("Duplicate resource IDs in local library index");
const by = key => Object.fromEntries([...new Set(assets.map(asset => asset[key]))].sort().map(value => [value, assets.filter(asset => asset[key] === value).length]));
const index = {
  format: "shared_resource_library",
  schemaVersion: 1,
  scope: "local shared resource packs; not organized by consumer project",
  sourcePacks: sourcePackRecords,
  counts: { assets: assets.length, byPhysicalType: by("physicalType"), byFamily: by("family"), byAuthoringLineage: by("authoringLineage") },
  assets
};
await mkdir(root, { recursive: true });
await writeFile(out, `${JSON.stringify(index, null, 2)}\n`, "utf8");
console.log(`Indexed ${assets.length} local source assets at ${relative(repo, out)}.`);
