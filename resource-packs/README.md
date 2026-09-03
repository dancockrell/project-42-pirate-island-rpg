# Shared resource packs

This directory is the repository-backed shared resource library used by both
Project 42 and DR Companion. It is intentionally independent of any consuming
game. The source geometry, curated derivatives, licenses, and searchable index
are tracked here so the library does not disappear with a disposable worktree.

```text
resource-packs/
  source/cc0/<creator>/<collection>/     # recovered, redistribution-safe source geometry
  geometry/<physical-type>/<pack>/       # locally authored or curated derivatives
  library.local.json                     # generated searchable index of every tracked asset
```

The index is the discovery mechanism. Search assets by `physicalType`, `family`,
style tags, setting tags, authoring lineage, license, source pack, technical
format, or review state. Never arrange reusable assets by a particular game.

Source collections are not automatically runtime-approved. A derivative pack
must state exactly what was changed and must preserve the source relationship.
The original downloadable archives and temporary extraction trees remain in the
ignored vendor cache because the tracked source geometry and license evidence
are the canonical reusable payload. The Git ledger and source-collection
catalog provide a second, compact provenance view for auditing and discovery.
