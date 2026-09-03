# Shared Tabletop Asset Platform

Status: **production contract**

This repository supplies a shared asset platform for Project 42, DR Companion,
and later paid professional work. The goal is not indiscriminate reuse. The
goal is to build a small, high-quality physical vocabulary that makes all of
those games faster to construct and more coherent at play distance.

The platform's visual language is original, readable, slightly toy-like 3D:
deliberate geometry, clean silhouettes, compatible materials, practical pivots,
and clear selection/interaction space. It may support a cute geometric tabletop
scene, a painted 2D composition, or a more richly dressed Godot set, but it
must never force any consuming game to inherit another game's lore, characters,
race presentation, or historical identity.

## 1. What is shared, and what is not

| Shared physical vocabulary | Project-owned vocabulary |
| --- | --- |
| Trees, trunks, roots, ferns, vines, flowers, reeds, mushrooms, rocks, cliffs, cave chunks, sand, mud, water surfaces, basalt, ash, road pieces, stairs, arches, beams, doors, roofs, bridges, docks, carts, crates, barrels, tables, lanterns, ropes, neutral rubble, generic weapon silhouettes, neutral VFX building blocks and material families. | DragonRealms playable-race adapters, guild and shrine identities, named creatures, landmark sets, room-description recipes, live MUD event bindings, specific world iconography, and its visual-material grammar. |
| Common mesh, material, LOD, collision, pivot, scale, provenance, and Godot-import standards. | Project 42's cast, Handsome Jack, proprietary weapons, elven Bronze Age visual motifs, fox-folk maritime culture, island factions, named locations, and story props. |
| Generic rig/socket/animation contract for compatible purchased or internally owned figure bases. | The exact faces, proportions, costumes, race identity, action performance, and character-specific animations for either game's people. |

`Shared` means a consumer may use the asset after its ledger entry says it is
approved for that consumer. It does **not** mean a model can be resold,
redistributed, or copied into another product without checking its license.

## 2. Legal and provenance boundary

Every asset is registered in `content/art/shared_asset_ledger.json` before it
is used in a consumer project. The ledger is validated by `npm run validate` in
`tools/`.

### Source-pack intake is not runtime admission

A `reference` record may document a retrieved source pack before its contents
are individually chosen. It must still name the exact canonical page, the
retrieved archive's SHA-256, license evidence, and its ignored local cache.
That record proves where a cohort came from; it does **not** make every model in
the cohort usable in either game. A runtime candidate needs its own record and
camera-scale review, and an approved runtime asset needs the normal visual and
technical decisions.

### Raw shared source

Only assets whose original license explicitly permits commercial use,
modification, and redistribution may be placed in a shared raw-source folder.
For the first library that means **CC0-1.0 only**. Every entry needs the creator,
canonical source URL, retrieval date, original filename, SHA-256, exact license
identifier, and a record of review at the intended game scale.

### Paid and account-generated assets

Paid marketplace sources, subscription assets, and vendor-generated output may
be extremely valuable, but they are **not** raw shared source by default. Store
their purchase/license evidence, vendor URL, source version, original hash,
account/seat restrictions, and import recipe in the ledger. Keep source files
project-local or in an access-controlled vendor cache unless the exact license
allows redistribution to every intended consumer.

An admitted derivative can be shared only when the source license permits that
use. If the answer is unclear, set `redistributionAllowed` to `false` and admit
the asset only for the licensed project. Never solve uncertainty by stripping
metadata or calling an asset "free."

### Generated material and scene art

Magnific or other generated work records its creation URL/ID, tool/model,
complete prompt, source inputs, review result, relevant terms, and a clear
state. A broad scene image is art direction unless an explicit review admits a
separate engine-ready derivative. It is never silently treated as runtime 3D
geometry.

## 3. Repository layout

```text
content/art/shared_asset_ledger.json    # committed admission/provenance ledger
content/art/shared_source_collections.json # committed source-collection provenance catalog
docs/SHARED_ASSET_PLATFORM.md           # asset legal/style contract
docs/SHARED_ASSET_INTEGRATION_PROTOCOL.md # consumer selection and proof contract
resource-packs/                         # tracked reusable source + derivative library
work/art/vendor/                        # ignored paid/vendor source caches
work/art/generated/                     # ignored raw generation sources
<consumer project>/assets/              # only named, admitted consumer derivatives
```

The reviewed shared source geometry, license evidence, derivative packs, and
generated library index in `resource-packs/` are tracked so the library stays
available across worktrees. Do not commit the original download archives or
temporary extraction trees merely because they are convenient; their hashes,
canonical URLs, and provenance records remain in the catalog. Admit a coherent,
reviewed subset with deterministic identifiers into a consumer only when it
passes the integration protocol. If a future asset exceeds normal Git hosting
limits, use the repository's approved large-file workflow rather than quietly
leaving it in a sandbox.

## 4. Asset admission gates

An asset is not approved because it is attractive in a marketplace thumbnail.
It must pass every applicable gate below.

1. **Legal gate:** exact license and distribution boundary recorded; source
   creator and canonical URL retained; source hash known.
2. **Semantic gate:** the item is neutral enough for its declared reuse. A
   generic lantern can be shared; a DragonRealms guild crest or the Handsome
   Jack cannot.
3. **Style gate:** readable silhouette and restrained material language at the
   actual target camera. Reject photoreal noise, baked scenery, logos, text,
   unrelated branding, and styles that make a consuming game look accidental.
4. **Technical gate:** known scale, `y = 0` ground contact, sensible pivot,
   clean import, declared format, material slots, collision/selection intent,
   and a low-detail strategy where distance use needs one.
5. **Assembly gate:** the asset has tags, compatible footprint/connectors where
   relevant, and an explicit intended role. Terrain and structures cannot block
   a route or battle lane merely because the mesh is decorative.
6. **Consumer gate:** each consuming project is listed explicitly. A project
   gets a palette/material adapter and may reject an otherwise shared asset.

## 5. Required asset records

Every entry supplies:

```ts
type SharedAssetRecord = {
  id: string
  status: 'candidate' | 'quarantine' | 'approved_shared' |
          'approved_project_only' | 'rejected'
  category: 'vegetation' | 'terrain' | 'architecture' | 'prop' |
            'material' | 'vfx' | 'rig' | 'character' | 'reference'
  ownership: 'cc0_raw' | 'paid_source' | 'account_generated' |
             'project_authored'
  source: {
    creator: string
    canonicalUrl: string
    retrievedOn: string
    originalFilename: string
    sha256: string
    licenseSpdx: string
    licenseEvidenceUrl: string
    licenseNotes: string
    redistributionAllowed: boolean
    sourceAccess: 'repository_raw' | 'project_local_vendor_cache' |
                  'account_controlled'
  }
  consumers: Array<'project42' | 'dr-companion' | 'professional-client'>
  review: {
    targetCamera: string
    visualDecision: 'pending' | 'approved' | 'rejected'
    technicalDecision: 'pending' | 'approved' | 'rejected'
    notes: string[]
  }
  import: {
    sourceFormat: string
    intendedRuntimeFormat: string
    scaleMetres?: number
    pivotAndGroundContact: string
    materialStrategy: string
    collisionOrSelection: string
    lodStrategy: string
  }
  tags: string[]
}
```

Approved shared material has a stricter rule: it must be CC0-1.0 with explicit
redistribution permission, both legal and visual review approved, at least one
consumer, and a complete import contract. Other licensing statuses are valid
only for candidates, quarantine, rejection, or project-only admission.

`licenseSpdx` may use a `LicenseRef-…` value when a commercial license has no
SPDX identifier, but it must then give a durable `licenseEvidenceUrl` and plain
`licenseNotes` describing the product, seat/account, client, or redistribution
restriction. A paid or account-generated raw source is never stored as
`repository_raw` unless the recorded license specifically grants that right.

## 6. Breadth roadmap

This platform should eventually cover the broad pre-modern vocabulary needed by
both games: forest, jungle, swamp, coast, river, harbor, cave, tomb, ruin,
volcanic/basalt, mountain, desert, settlement, road, market, workshop, temple,
shipboard, black-powder, fantasy arms, armor, and magical effects. Breadth is
not permission to lower quality. Work by coherent families, beginning with
plants, terrain, and materials because they add value to every environment,
then generic construction/dressing, then weapons and specialist sets.

Characters have a different production path. Shared rig/socket standards are
valuable; generic premium bodies and head bases may be admitted when licensed
correctly. Finished heroes, player races, named NPCs, and distinctive creatures
remain project-specific. The eventual DragonRealms layer needs race adapters,
not a procedural-face-generator detour.

## 7. First intake sequence

1. Confirm the large-file and source-cache workflow.
2. Admit a small CC0 vegetation/rocks/materials cohort after visual review.
3. Normalize scale, pivots, materials, tags, and Godot import behavior.
4. Prove each item in one Project 42 and one DR Companion test scene.
5. Add construction/prop cohorts only after the first cohort improves a real
   scene at game camera distance.
6. Keep candidate, rejected, and vendor-only material out of runtime manifests.

This is how the library becomes a professional advantage: the more it grows,
the more dependable, coherent, and legally understandable it becomes.
