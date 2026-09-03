# Instructions for AI agents working in this repository

## Rule 0 — Never fork. Solve the problem.

**In no case is an AI ever to fork — not in code, not in commits.** A problem is
always something to be solved, never something to be dodged. You are trusted to
use your own judgement, so use it.

When you hit something that does not work, you have exactly three legitimate
moves:

1. **Build on the existing solution.** Fix it, extend it, correct it in place.
2. **Replace it.** Delete the old implementation in the same change that
   introduces the new one, and move every caller across.
3. **Delete it and eliminate the feature.** Sometimes this is the best option.
   A feature that does not work and is not worth fixing should leave the
   codebase, not linger.

Forking is any move that leaves two answers to one question standing side by
side, and it is forbidden. Concretely, do not:

- add `thing_v2`, `thing_new`, `thing_ex`, or a "convenience" wrapper that
  duplicates an existing entry point instead of changing it;
- copy a file or a function to avoid editing the original;
- leave a parallel code path, flag or shim in place "for compatibility" when
  nothing outside this repository depends on it;
- leave branches, PRs or working copies to diverge and rot instead of merging
  or closing them;
- work around a broken component by routing past it and leaving it broken.

If replacing something means changing many call sites, change them. If deleting
a feature means telling the user it is gone, tell them. Churn in one honest
change is cheaper than a second half-solution nobody can safely remove later.

The one thing that is never acceptable is silently shipping both.

---

## The rest of the contract

Read these in order; they are the authority for what this project is and how it
is built:

1. `docs/ARCHITECTURE.md` — simulation, command/event, targeting, and event-order
   contract.
2. `docs/VERTICAL_SLICE_BUILD_CONTRACT.md` — the exact first playable loop and
   its proof gates.
3. `docs/GAME_BUILD_PLAN.md` — product direction, build order, and the fixed
   player promises.
4. `docs/GODOT_LANGUAGE_AND_SETPIECE_BOUNDARIES.md` — language and scene
   boundaries.
5. The JSON records under `content/`, plus their schemas and
   `tools/src/validate.mjs` — the executable authored-data contract.

When prose and executable schema disagree, do not silently pick one. Preserve
the stable ID, document the conflict in the change, and make the smallest
compatible correction.

A feature is not done because a type, class, JSON record or UI button exists. It
is done when the authoritative state can save, reload, and produce the same
legal next commands and ordered results — proven by a deterministic test.
