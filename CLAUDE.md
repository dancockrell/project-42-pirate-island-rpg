# AI Working Instructions

## Non-negotiable: never fork the implementation

AI agents are not authorized to create or use a fork as a way to perform repository work. There must be one product, one implementation of each behavior, and one continuously reconciled history.

This prohibition includes:

- creating a repository fork, alternate worktree, new branch, or parallel commit lineage to avoid working with the current code;
- building a second component, service, store, listener, command path, asset catalog, or “V2” beside an existing one because integration is difficult;
- preserving two competing implementations “for safety” instead of choosing and completing one;
- evading a merge conflict, failing test, design inconsistency, or architectural problem by routing around it;
- abandoning incomplete work in a disconnected branch or noodle-to-nowhere code path.

Required behavior:

1. Work in the existing checkout and on its current shared branch. Normal small, coherent commits on that branch are required; an alternate history is not.
2. Fetch and inspect current shared work before changing overlapping code. Treat concurrent work as material to reconcile, not territory to avoid.
3. Solve the problem in the existing product. Build on the current solution, deliberately replace it, or delete it and remove the feature when deletion is the better product decision.
4. When implementations conflict, compare them in the real application and with focused tests. Keep the strongest ideas, merge compatible behavior, select one owner, and remove superseded paths.
5. Preserve user-authored data and public contracts unless the task explicitly changes them. Never force-push or discard another contributor’s work merely to make integration easier.
6. Commit and push verified checkpoints frequently so the shared branch remains the place where the product exists.
7. If a genuine external blocker remains, document the exact blocker and the best in-place next action. Do not create an alternate implementation to make the blocker disappear from view.

Difficulty is a reason to reason, test, and integrate more carefully. It is never permission to fork.
