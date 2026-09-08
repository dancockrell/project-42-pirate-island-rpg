# Pirate Island asset shelf

Open [index.html](index.html) in a browser to search the current island art, preview transparency, open full-resolution PNGs, and follow the actual game metadata. It works directly from disk without installing or running a server.

## Existing source of truth

- Michael: [sprite sheet](sprites/michael/source.png) and [frame data](sprites/michael/frames.json).
- Troops: [appearance manifest](island/troops/appearances.json) maps unit identity and sex to PNG, pixel pivot and display scale.
- Buildings: [building manifest](island/buildings.json) maps sites to PNGs, placement anchors and independent collision footprints.
- Terrain: [background](island/terrain.png) and [navigation mask](island/navigation.json).

The shelf is generated from those files, not another manually maintained manifest. Regenerate it from the repository root with `python tools/inspect_sprite_source.py --catalog`. Use `--island` for a read-only audit. It checks paths, exact source hashes where recorded, transparency, dimensions, frame bounds, pivot bounds and unreferenced PNGs in the current island/sprite directories.

The PNGs retain their native pixels and colors. Browser previews and game display scales do not change the source images.

## Actual animation coverage

Michael currently has four standing directions. The nine troop appearances each have one standing pose. These are usable static assets, not finished movement or combat sheets. Walking, attacking, hit reactions and death still require coherent new animation frames and in-game review. Do not manufacture apparent coverage by repeating poses.

Older battle-fixture assets outside the current island directories are not included in this shelf. The shelf is not a claim that the entire shared repository has been cleaned.
