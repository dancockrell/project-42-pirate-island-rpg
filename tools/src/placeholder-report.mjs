import { readFile, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const repo = resolve(import.meta.dirname, "../..");
const manifest = JSON.parse(await readFile(resolve(repo, "content/art/placeholders.json"), "utf8"));
const lines = ["# Placeholder art replacement report", "", `Open placeholders: ${manifest.assets.length}`, ""];
for (const asset of manifest.assets) {
  lines.push(`## ${asset.id}`, "", `- Consumer: \`${asset.consumer}\``, `- Visible mark: **${asset.visibleMark}**`,
    `- Intended final: ${asset.intendedFinal}`, `- Required tags: ${asset.requiredTags.join(", ")}`,
    "- Replacement gate:", ...asset.replacementGate.map(item => `  - ${item}`), "");
}
await mkdir(resolve(repo, "reports"), { recursive: true });
await writeFile(resolve(repo, "reports/placeholder-art.md"), `${lines.join("\n")}\n`, "utf8");
console.log(`Wrote reports/placeholder-art.md with ${manifest.assets.length} replacement records.`);

