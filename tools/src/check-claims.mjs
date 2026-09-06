// Makes the two ledgers a gate rather than a discipline.
//
// The first ledger is .agents/claims/*.json — who holds what, and what a
// finished claim proved. The second is the lane tables and cards in
// docs/SHIP_PLAN.md. Both close by hand today, and both have been wrong:
// rows without cards five times, and completion commits naming SHAs that no
// longer exist on the trunk. This script reads both and exits non-zero with
// one line per failure. Node built-ins and git only; git is invoked through
// execFileSync with an argument vector, never a shell string.

import { readFile, readdir } from "node:fs/promises";
import { resolve, relative, basename } from "node:path";
import { execFileSync } from "node:child_process";
import process from "node:process";

const repo = resolve(import.meta.dirname, "../..");
const failures = [];

function fail(file, message) { failures.push(`${relative(repo, file)}: ${message}`); }

// A commit-ish that git resolves to a real commit object in this clone.
const commitCache = new Map();
function commitExists(sha) {
  if (commitCache.has(sha)) return commitCache.get(sha);
  let ok = false;
  try {
    execFileSync("git", ["cat-file", "-e", `${sha}^{commit}`], { cwd: repo, stdio: "ignore" });
    ok = true;
  } catch { ok = false; }
  commitCache.set(sha, ok);
  return ok;
}

// Reachable from HEAD. A lane-branch SHA that was squashed on merge is not,
// and that is exactly the record this check is here to catch.
function isAncestorOfHead(sha) {
  try {
    execFileSync("git", ["merge-base", "--is-ancestor", sha, "HEAD"], { cwd: repo, stdio: "ignore" });
    return true;
  } catch { return false; }
}

// UTC ISO 8601, as .agents/README.md's schema requires: a Z-terminated
// timestamp that Date agrees round-trips to the same instant.
const isoUtc = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/;
function checkTimestamp(record, key, file) {
  const value = record[key];
  if (typeof value !== "string" || !isoUtc.test(value) || Number.isNaN(Date.parse(value))) {
    fail(file, `${key} must be an ISO-8601 UTC timestamp ending in Z, got ${JSON.stringify(value)}`);
  }
}

const allowedStatuses = new Set(["active", "blocked", "completed"]);
const checkOutcomes = ["passed", "failed", "not_run"];

const claimsDir = resolve(repo, ".agents/claims");
const claimFiles = (await readdir(claimsDir)).filter(name => name.endsWith(".json")).sort();
let claimsChecked = 0;

for (const name of claimFiles) {
  const file = resolve(claimsDir, name);
  const stem = basename(name, ".json");
  let claim;
  try {
    claim = JSON.parse(await readFile(file, "utf8"));
  } catch (error) {
    fail(file, `is not valid JSON: ${error.message}`);
    continue;
  }
  claimsChecked += 1;

  if (claim.schema_version === undefined || claim.schema_version === null) fail(file, "schema_version is missing");
  if (claim.task_id !== stem) fail(file, `task_id ${JSON.stringify(claim.task_id)} must equal the file stem ${stem}`);
  if (!allowedStatuses.has(claim.status)) fail(file, `status ${JSON.stringify(claim.status)} must be active, blocked or completed`);
  checkTimestamp(claim, "claimed_at", file);
  checkTimestamp(claim, "updated_at", file);

  if (!Array.isArray(claim.paths) || claim.paths.length === 0 || !claim.paths.every(value => typeof value === "string" && value.trim() !== "")) {
    fail(file, "paths must be a non-empty array of non-empty strings");
  }

  const completion = claim.completion ?? {};
  if (claim.status === "completed") {
    const sha = completion.commit;
    if (typeof sha !== "string" || sha.trim() === "") {
      fail(file, "a completed claim must record completion.commit as a non-null string");
    } else if (!commitExists(sha)) {
      fail(file, `completion.commit ${sha} is not a commit in this repository`);
    } else if (!isAncestorOfHead(sha)) {
      fail(file, `completion.commit ${sha} is not an ancestor of HEAD`);
    }
    const checks = completion.checks;
    if (!Array.isArray(checks) || checks.length === 0) {
      fail(file, "a completed claim must record a non-empty completion.checks array");
    } else {
      for (const [index, entry] of checks.entries()) {
        if (typeof entry !== "string" || !checkOutcomes.some(outcome => entry.trimEnd().endsWith(outcome))) {
          fail(file, `completion.checks[${index}] must end in passed, failed or not_run: ${JSON.stringify(entry)}`);
        }
      }
    }
  } else if (allowedStatuses.has(claim.status)) {
    if (completion.commit !== null && completion.commit !== undefined) {
      fail(file, `a ${claim.status} claim must leave completion.commit null, got ${JSON.stringify(completion.commit)}`);
    }
  }
}

// ---------------------------------------------------------------------------
// The ship-plan ledger.

const planFile = resolve(repo, "docs/SHIP_PLAN.md");
const planLines = (await readFile(planFile, "utf8")).split("\n");

// Cards are `### <ID> · <title>`; the Status: line is the first line under the
// heading that starts with `Status:`.
const cards = new Map();
for (const [index, line] of planLines.entries()) {
  const match = /^###\s+([A-Z]+[0-9]+)\s+·/.exec(line);
  if (!match) continue;
  let status = null;
  for (let cursor = index + 1; cursor < planLines.length && !/^###\s/.test(planLines[cursor]); cursor += 1) {
    const found = /^Status:\s*(.*)$/.exec(planLines[cursor]);
    if (found) { status = found[1]; break; }
  }
  cards.set(match[1], { line: index + 1, status });
}

// Lane ledger tables are identified by their header row, which is exactly
// `| ID | Task | Depends on | Status |`. The milestone table and the
// lane-ownership table have different headers and are skipped by construction
// rather than by a special case.
function splitRow(line) {
  return line.replace(/^\|/, "").replace(/\|\s*$/, "").split("|").map(cell => cell.trim());
}

let inLedger = false;
let rowsChecked = 0;
for (const [index, line] of planLines.entries()) {
  if (!line.trimStart().startsWith("|")) { inLedger = false; continue; }
  const cells = splitRow(line);
  if (cells.length === 4 && cells[0] === "ID" && cells[3] === "Status") { inLedger = true; continue; }
  if (!inLedger) continue;
  if (cells.every(cell => /^-{1,}$/.test(cell))) continue;
  if (cells.length !== 4) continue;

  const [id, , , status] = cells;
  if (!/^[A-Z]+[0-9]+$/.test(id)) continue;
  rowsChecked += 1;
  const where = `docs/SHIP_PLAN.md:${index + 1}`;

  const shipped = /^shipped\s+([0-9a-f]{7,40})\b/.exec(status);
  if (shipped) {
    const sha = shipped[1];
    const card = cards.get(id);
    if (!card) {
      failures.push(`${where}: row ${id} is shipped but has no "### ${id} ·" card`);
    } else if (card.status === null) {
      failures.push(`${where}: the "### ${id} ·" card (line ${card.line}) has no Status: line`);
    } else {
      if (!/\bshipped\b/.test(card.status)) {
        failures.push(`${where}: row ${id} says shipped but its card's Status: line (line ${card.line}) does not: ${card.status}`);
      }
      const cardShas = card.status.match(/\b[0-9a-f]{7,40}\b/g) ?? [];
      const agrees = cardShas.some(candidate => candidate.startsWith(sha) || sha.startsWith(candidate));
      if (!agrees) {
        failures.push(`${where}: row ${id} is shipped ${sha} but its card's Status: line (line ${card.line}) names ${cardShas.length ? cardShas.join(", ") : "no sha"}`);
      }
    }
    if (!commitExists(sha)) {
      failures.push(`${where}: row ${id} is shipped ${sha}, which is not a commit in this repository`);
    }
    continue;
  }

  // The row-without-card rule. `shipped` rows are handled above; a superseded
  // row and a `done (this pass)` row describe work that never had a card.
  if (/^shipped\b/.test(status) || /^superseded\b/.test(status) || status === "done (this pass)") continue;
  if (!cards.has(id)) {
    failures.push(`${where}: row ${id} has status ${JSON.stringify(status)} and no "### ${id} ·" card`);
  }
}

if (failures.length) {
  console.error(`Project 42 ledger check failed with ${failures.length} issue(s):`);
  for (const message of failures) console.error(`- ${message}`);
  process.exit(1);
}
console.log(`Project 42 ledgers consistent: ${claimsChecked} claims checked, ${rowsChecked} ship-plan rows checked, ${cards.size} cards read.`);
