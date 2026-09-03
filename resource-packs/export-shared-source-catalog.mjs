import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname);
const repo = resolve(root, "..");
const library = JSON.parse(await readFile(resolve(root, "library.local.json"), "utf8"));
const sourceCollections = Object.entries(library.sourcePacks).map(([collectionKey, pack]) => ({
  id: `source.collection.cc0.kenney.${collectionKey}`,
  collectionKey,
  title: pack.title,
  creator: "Kenney",
  canonicalUrl: pack.source,
  authoringLineage: pack.authoringLineage,
  licenseSpdx: pack.license,
  retrievedOn: pack.retrievedOn,
  archive: pack.archive,
  styleTags: pack.styleTags,
  defaultPhysicalType: pack.defaultType,
  ledgerRecord: pack.ledger ?? null,
  state: "local_source_available"
})).sort((a, b) => a.id.localeCompare(b.id));

const output = {
  format: "shared_source_collection_catalog",
  schemaVersion: 1,
  purpose: "Compact, committed provenance catalog for local reusable source collections. It intentionally does not ship the raw collection payload.",
  sourceCollectionCount: sourceCollections.length,
  sourceCollections
};
await writeFile(resolve(repo, "content/art/shared_source_collections.json"), `${JSON.stringify(output, null, 2)}\n`, "utf8");
console.log(`Exported ${sourceCollections.length} source-collection records.`);
