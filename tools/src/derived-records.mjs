// One owner for record derivation, imported by both readers of the content
// tree: `tools/src/validate.mjs` checks the resolved record and
// `tools/src/build-content-bundle.mjs` ships it, so Godot never sees a
// half-record and the merge rule cannot differ between the two.
//
// C15 introduced it for `content/enemies/razorbeak.crested.json`. The Rust
// habitat fixture defines the crested razorbeak as a razorbeak that holds the
// terrace precinct and overrides nothing else, so copying the razorbeak's
// stats, skills, actions, recovery-opening rules and art into a second file
// would have made two records answer one question -- and the copy would keep
// the old numbers the first time anyone tuned the original.

/// Merge `override` onto `base`. Plain objects merge key by key; every other
/// value, arrays included, is replaced outright, because a partially
/// overridden list is never what an author means.
export function mergeRecords(base, override) {
  const isPlainObject = value => value !== null && typeof value === "object" && !Array.isArray(value);
  const result = { ...base };
  for (const [key, value] of Object.entries(override)) {
    result[key] = isPlainObject(result[key]) && isPlainObject(value) ? mergeRecords(result[key], value) : value;
  }
  return result;
}

/// Resolve every `derivesFrom` in one domain. `records` is an array of
/// `{ file, value }`; the return is the same array with `value` replaced by
/// the resolved record, plus the problems found, so the caller decides how to
/// report them. A record derives from a record in the same domain, chains are
/// followed, and a missing base, a self-reference or a cycle is an error
/// rather than a silent partial record.
export function resolveDerivedRecords(records) {
  const byId = new Map(records.map(record => [record.value?.id, record]));
  const resolved = new Map();
  const problems = [];

  function resolve(record, seen) {
    if (resolved.has(record.value.id)) return resolved.get(record.value.id);
    const base = record.value.derivesFrom;
    if (base === undefined) {
      resolved.set(record.value.id, record.value);
      return record.value;
    }
    if (typeof base !== "string" || base.trim() === "") {
      problems.push({ file: record.file, message: "derivesFrom must be the stable ID of the record this one is a variant of" });
      resolved.set(record.value.id, record.value);
      return record.value;
    }
    if (seen.includes(base)) {
      problems.push({ file: record.file, message: `derivesFrom ${base} closes a derivation cycle: ${[...seen, base].join(" -> ")}` });
      resolved.set(record.value.id, record.value);
      return record.value;
    }
    const parent = byId.get(base);
    if (parent === undefined) {
      problems.push({ file: record.file, message: `derivesFrom ${base}, which no record in this directory declares` });
      resolved.set(record.value.id, record.value);
      return record.value;
    }
    const merged = mergeRecords(resolve(parent, [...seen, base]), record.value);
    resolved.set(record.value.id, merged);
    return merged;
  }

  const output = records.map(record => ({ file: record.file, value: resolve(record, [record.value?.id]) }));
  return { records: output, problems };
}
