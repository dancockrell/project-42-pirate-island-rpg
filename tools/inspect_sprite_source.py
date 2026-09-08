"""Read-only 2D sprite-source admission checks. Does not create or modify art."""
import argparse
import hashlib
import json
import math
import html
from pathlib import Path

from PIL import Image


def inspect_source(path, manifest=None):
    path = Path(path)
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError("Source exceeds 64 MiB inspection budget")
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    with Image.open(path) as source:
        width, height = source.size
        if width * height > 16_000_000:
            raise ValueError("Source exceeds 16 million pixel inspection budget")
        if source.format != "PNG" or getattr(source, "n_frames", 1) != 1:
            raise ValueError("Atlas must be a single PNG image")
        rgba = source.convert("RGBA")
    histogram = rgba.getchannel("A").histogram()
    errors = []
    warnings = []
    if not histogram[0]:
        errors.append("No fully transparent pixels: not admitted as a cutout sprite atlas; a visible checkerboard is not alpha")
    if sum(histogram[1:255]):
        warnings.append("Partial alpha present: review edge smoothing against the approved pixel grid")
    if not sum(histogram[1:]):
        errors.append("Image is fully transparent")
    frames_checked = 0
    unique_drawings = set()
    inspected_pixels = 0
    if manifest is None:
        errors.append("Missing authored frame/action/direction/pivot metadata")
    elif not isinstance(manifest, dict):
        errors.append("Manifest must be an object")
    else:
        if manifest.get("sourceSha256") != digest:
            errors.append("Manifest does not identify these exact source bytes")
        if type(manifest.get("schemaVersion")) is not int or manifest["schemaVersion"] != 1:
            errors.append("Unsupported manifest schemaVersion")
        frames = manifest.get("frames")
        if not isinstance(frames, list) or not 1 <= len(frames) <= 4096:
            errors.append("Manifest requires 1..4096 frames")
            frames = []
        identities = set()
        for frame in frames:
            if not isinstance(frame, dict):
                errors.append("Frame must be an object")
                continue
            label = frame.get("id")
            if not isinstance(label, str) or not label or label in identities:
                errors.append("Frame IDs must be nonempty unique strings")
            else:
                identities.add(label)
            for field in ("action", "direction"):
                if not isinstance(frame.get(field), str) or not frame[field].strip():
                    errors.append(f"{label}: missing {field}")
            rect = frame.get("rect")
            if not isinstance(rect, list) or len(rect) != 4 or any(type(n) is not int for n in rect):
                errors.append(f"{label}: rect must be four integer pixel coordinates")
                continue
            x, y, w, h = rect
            if x < 0 or y < 0 or w <= 0 or h <= 0 or x+w > width or y+h > height:
                errors.append(f"{label}: rectangle outside source")
                continue
            inspected_pixels += w * h
            if inspected_pixels > 16_000_000:
                errors.append("Frame inspection exceeds 16 million pixel budget")
                break
            pivot = frame.get("footPivot")
            if not isinstance(pivot, list) or len(pivot) != 2 or any(type(n) is not int for n in pivot) or not (0 <= pivot[0] <= w and 0 <= pivot[1] <= h):
                errors.append(f"{label}: footPivot must be within the local frame canvas")
            duration = frame.get("durationSeconds")
            if type(duration) not in (int, float) or not math.isfinite(duration) or not 0 < duration <= 60:
                errors.append(f"{label}: durationSeconds must be finite and in (0,60]")
            if type(frame.get("loop")) is not bool:
                errors.append(f"{label}: loop must be an explicit boolean")
            # Read only: cropping here calculates frame coverage, never writes pixels.
            frame_image = rgba.crop((x, y, x+w, y+h))
            bounds = frame_image.getchannel("A").getbbox()
            if not bounds:
                errors.append(f"{label}: frame contains no visible pixels")
            else:
                trimmed = frame_image.crop(bounds)
                # Compare visible drawings independently of canvas padding and
                # horizontal mirrors. Reused holds are legal, but not new art.
                keys = []
                for candidate in (trimmed, trimmed.transpose(Image.Transpose.FLIP_LEFT_RIGHT)):
                    pixels = bytearray(candidate.tobytes())
                    for i in range(0, len(pixels), 4):
                        if pixels[i+3] == 0:
                            pixels[i:i+3] = b"\x00\x00\x00"
                    keys.append(hashlib.sha256(str(candidate.size).encode() + pixels).hexdigest())
                unique_drawings.add(min(keys))
            frames_checked += 1
    return {"source": path.name, "sha256": digest, "width": width, "height": height,
        "transparentPixels": histogram[0], "partialAlphaPixels": sum(histogram[1:255]),
        "framesChecked": frames_checked, "distinctDrawingsIgnoringHorizontalMirrors": len(unique_drawings),
        "coverageAdmission": "not-assessed: unique pixels do not prove distinct useful actions, facing, state or recolor-free coverage",
        "errors": errors, "warnings": warnings,
        "structuralAdmission": not errors, "artAdmission": "not-assessed",
        "scope": "Read-only source/metadata checks; no automatic grid inference, frame extraction, animation continuity or visual approval"}


def inspect_island(root):
    """Read the existing runtime manifests, not a second hand-maintained catalog."""
    root = Path(root).resolve()
    assets = root / "game/assets"
    entries = []
    errors = []

    def add(label, relative, category, pivot=None, expected=None, detail="", source=None):
        path = (assets / relative).resolve()
        if not path.is_relative_to(assets) or not path.is_file():
            errors.append(f"{label}: missing or unsafe asset path {relative}")
            return
        if path.stat().st_size > 64 * 1024 * 1024:
            errors.append(f"{label}: image exceeds inspection budget")
            return
        with Image.open(path) as im:
            if im.width * im.height > 16_000_000:
                errors.append(f"{label}: image exceeds pixel budget")
                return
            width, height = im.size
            alpha = im.convert("RGBA").getchannel("A")
            bounds = alpha.getbbox()
            transparent = alpha.histogram()[0]
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if expected and expected != digest:
            errors.append(f"{label}: source hash mismatch")
        if not bounds or (category != "Terrain" and not transparent):
            errors.append(f"{label}: missing visible content or cutout transparency")
        if pivot is not None and (len(pivot) != 2 or not all(type(n) in (int, float) and math.isfinite(n) for n in pivot) or not (0 <= pivot[0] <= width and 0 <= pivot[1] <= height)):
            errors.append(f"{label}: pivot outside image")
        entries.append(dict(name=label, path=relative, category=category, width=width,
                            height=height, pivot=pivot, bounds=bounds, sha256=digest,
                            detail=detail, source=source))

    frames = json.loads((assets / "sprites/michael/frames.json").read_text(encoding="utf-8"))
    review = inspect_source(assets / "sprites/michael/source.png", frames)
    errors.extend(review["errors"])
    add("Captain Michael", "sprites/michael/source.png", "Characters", expected=frames["sourceSha256"],
        detail=f"{len(frames['frames'])} standing directions. Walk, attack, hit and death animation missing.",
        source="sprites/michael/frames.json")
    appearances = json.loads((assets / "island/troops/appearances.json").read_text(encoding="utf-8"))
    for definition, group in appearances.items():
        for variant, entry in group["variants"].items():
            relative = entry["texture"].removeprefix("res://assets/")
            add(definition.removeprefix("actor_def.").replace("_", " ") + " · " + variant,
                relative, "Characters", entry["pivot"], entry.get("sha256"),
                "One standing pose. Directional action sheets missing.", "island/troops/appearances.json")
            if type(entry.get("scale")) not in (int, float) or not math.isfinite(entry["scale"]) or entry["scale"] <= 0:
                errors.append(f"{definition}/{variant}: invalid display scale")
    buildings = json.loads((assets / "island/buildings.json").read_text(encoding="utf-8"))
    for definition, entry in buildings.items():
        add(definition.removeprefix("site_archetype.").replace("_", " "),
            entry["texture"].removeprefix("res://assets/"), "Buildings", entry["pivot"],
            entry.get("sha256"), "Static building cutout; collision is separate.", "island/buildings.json")
    add("Island terrain", "island/terrain.png", "Terrain", detail="Current island background; navigation is separate.", source="island/navigation.json")
    add("Salvage", "island/salvage_bale.png", "Props", [252, 304], detail="Static pickup cutout.")
    used = {entry["path"] for entry in entries}
    extras = sorted(str(p.relative_to(assets)).replace("\\", "/") for folder in [assets / "island", assets / "sprites"] for p in folder.rglob("*.png") if str(p.relative_to(assets)).replace("\\", "/") not in used)
    return {"entries": entries, "errors": errors, "unreferencedPngs": extras}


def write_catalog(report, output):
    """Generate only a browsing document. Original image bytes are never changed."""
    if report["errors"]:
        raise ValueError("Cannot publish catalog with invalid asset metadata")
    cards = []
    for e in report["entries"]:
        esc = html.escape
        metadata = f"{e['width']} × {e['height']} · " + (f"pivot {e['pivot']}" if e['pivot'] else "see metadata")
        source_link = f'<a href="{esc(e["source"], quote=True)}">Metadata</a>' if e["source"] else ""
        cards.append(f'''<article data-category="{esc(e['category'])}"><a class="preview" href="{esc(e['path'], quote=True)}"><img loading="lazy" src="{esc(e['path'], quote=True)}" alt="{esc(e['name'], quote=True)}"></a><section><small>{esc(e['category'])}</small><h2>{esc(e['name'])}</h2><p>{esc(metadata)}</p><p>{esc(e['detail'])}</p><nav><a href="{esc(e['path'], quote=True)}" download>PNG</a>{source_link}</nav><details><summary>SHA-256</summary><code>{e['sha256']}</code></details></section></article>''')
    page = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Pirate Island — asset shelf</title><style>
*{box-sizing:border-box}body{margin:0;background:#161b1b;color:#eee8da;font:16px system-ui,sans-serif}header{max-width:1300px;margin:auto;padding:36px 24px 20px}h1{font-size:34px;margin:0 0 12px}header p{color:#b8c1ba;max-width:850px;line-height:1.6}input,select{font:inherit;padding:12px;background:#252e2b;color:inherit;border:1px solid #53625b;border-radius:6px;margin:4px 8px 4px 0}main{max-width:1300px;margin:auto;padding:0 24px 40px;display:grid;grid-template-columns:repeat(auto-fit,minmax(270px,1fr));gap:20px}article{border:1px solid #3a4941;border-radius:9px;overflow:hidden;background:#212a25}.preview{height:330px;display:flex;align-items:center;justify-content:center;background-color:#899387;background-image:conic-gradient(#788274 25%,transparent 0 50%,#788274 0 75%,transparent 0);background-size:24px 24px}.preview img{max-width:100%;max-height:100%;object-fit:contain;image-rendering:pixelated}section{padding:18px}h2{font-size:19px;margin:8px 0}small{color:#a9c4b0}p{line-height:1.5;font-size:14px}nav{display:flex;gap:20px;margin:18px 0}a{color:#d3e6ce}details{font-size:12px;color:#b8c1ba}code{overflow-wrap:anywhere}article[hidden]{display:none}
</style><header><h1>Pirate Island · asset shelf</h1><p>The current game files, at full source quality. Open an image for native resolution or download its PNG. This shelf does not fabricate missing animation frames.</p><input id="search" type="search" placeholder="Find a character or building" aria-label="Search assets"><select id="category" aria-label="Asset category"><option>All</option><option>Characters</option><option>Buildings</option><option>Terrain</option><option>Props</option></select><p id="count"></p></header><main>''' + "".join(cards) + '''</main><script>
const search=document.querySelector('#search'), category=document.querySelector('#category'), cards=[...document.querySelectorAll('article')];function filter(){let n=0;for(const c of cards){c.hidden=!(c.textContent.toLowerCase().includes(search.value.toLowerCase())&&(category.value==='All'||c.dataset.category===category.value));if(!c.hidden)n++}document.querySelector('#count').textContent=n+' assets'}search.addEventListener('input',filter);category.addEventListener('change',filter);filter();</script></html>'''
    Path(output).write_text(page, encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, nargs="?")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--island", action="store_true", help="Audit the existing island manifests")
    parser.add_argument("--catalog", action="store_true", help="Regenerate game/assets/index.html from the island audit")
    args = parser.parse_args()
    if args.island or args.catalog:
        root = Path(__file__).resolve().parent.parent
        result = inspect_island(root)
        if args.catalog:
            write_catalog(result, root / "game/assets/index.html")
        print(json.dumps(result, indent=2, allow_nan=False))
        raise SystemExit(1 if result["errors"] else 0)
    if args.source is None:
        parser.error("Provide a source PNG, --island or --catalog")
    result = inspect_source(args.source, json.loads(args.manifest.read_text(encoding="utf-8")) if args.manifest else None)
    print(json.dumps(result, indent=2, allow_nan=False))
    raise SystemExit(0 if result["structuralAdmission"] else 1)
