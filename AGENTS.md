# How to work in this repo

This is an early prototype of a game. The goal is a playable game, not a paper
trail. Build features. Keep the tests passing. That's it.

## The one rule that stays

**Don't fork.** When existing code is wrong or nearly-what-you-need, you have
three moves: build on it, replace it (and move every caller), or delete it and
the feature with it. There is no fourth. No `v2`, no sibling module, no second
manifest, no parallel branch carrying the same work. Two things that answer the
same question will drift, and then both are wrong.

If you catch yourself explaining why your copy is acceptable, you're forking.

## The rest, briefly

- **Look at the actual code before designing.** Not a summary, not a memory.
  Most mistakes in this repo's history came from building on something the
  author assumed was there.
- **Run the tests before you say it works.** `cargo test` in `godot-rust/`.
  A command you didn't run is not a pass, and "should work" is not a result.
- **Don't fake completion.** No placeholders that read as finished, no claiming
  something is pushed when it isn't.
- **Don't invent content.** If you need a fact about the game — a character, a
  place, a number — read it out of `content/` or ask. Making it up produces
  lore that contradicts the game.

## What is deliberately gone

The work-ledger protocol (`.agents/claims/*.json`), the break-it-to-prove-the-
test-fails ritual, and the requirement to run the whole build chain and write
an essay for every change. On a prototype they cost more than they caught and
crowded out the actual work.

Judgement replaces them: run the narrow test while iterating, run the suite
before pushing something real, run the Godot gate when you touch the bridge.
