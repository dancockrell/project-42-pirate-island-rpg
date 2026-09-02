import { spawnSync } from "node:child_process";

const rules = [
  ["GitHub token", new RegExp(`gh[opsu]_[A-Za-z0-9]{${20},}`)],
  ["OpenAI API key", new RegExp(`s${"k"}-[A-Za-z0-9_-]{${20},}`)],
  ["assigned OpenAI API key", new RegExp(`OPENAI_API_KEY\\s*=\\s*[^\\s$%]+`, "i")],
  ["private key block", new RegExp(`BEGIN (?:RSA|OPENSSH|EC) PRIVATE KEY`)],
  ["AWS access key", new RegExp(`AKIA[0-9A-Z]{${16}}`)],
];

function findSecrets(lines) {
  const findings = [];
  for (const [index, line] of lines.entries()) {
    for (const [label, pattern] of rules) {
      if (pattern.test(line)) findings.push(`${label} on added diff line ${index + 1}`);
    }
  }
  return findings;
}

if (process.argv.includes("--self-test")) {
  const fakeGitHubToken = `gh${"p"}_${"A".repeat(24)}`;
  const fakeOpenAiKey = `s${"k"}-${"B".repeat(24)}`;
  const fakeAwsKey = `AKIA${"C".repeat(16)}`;
  const findings = findSecrets([fakeGitHubToken, fakeOpenAiKey, fakeAwsKey, "ordinary project text"]);
  if (findings.length !== 3) {
    console.error(`Secret scanner self-test failed: expected 3 findings, received ${findings.length}.`);
    process.exit(1);
  }
  console.log("Secret scanner self-test passed.");
  process.exit(0);
}

const diff = spawnSync(
  "git",
  ["diff", "--cached", "--no-ext-diff", "--unified=0", "--diff-filter=ACMR"],
  { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
);

if (diff.error) {
  console.error(`Secret scan could not start Git: ${diff.error.message}`);
  process.exit(1);
}
if (diff.status !== 0) {
  process.stderr.write(diff.stderr);
  process.exit(diff.status ?? 1);
}

const addedLines = diff.stdout
  .split(/\r?\n/)
  .filter((line) => line.startsWith("+") && !line.startsWith("+++"))
  .map((line) => line.slice(1));

const findings = findSecrets(addedLines);

if (findings.length > 0) {
  console.error("COMMIT BLOCKED: likely secret material was found in staged additions.");
  for (const finding of findings) console.error(`- ${finding}`);
  console.error("Remove the secret from the staged content before committing.");
  process.exit(1);
}

console.log(`Staged-secret scan passed (${addedLines.length} added lines checked).`);
