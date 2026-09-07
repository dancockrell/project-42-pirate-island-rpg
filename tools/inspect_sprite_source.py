"""Read-only 2D sprite-source admission checks. Does not create or modify art."""
import argparse
import hashlib
import json
import math
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
            pivot = frame.get("footPivot")
            if not isinstance(pivot, list) or len(pivot) != 2 or any(type(n) is not int for n in pivot) or not (0 <= pivot[0] <= w and 0 <= pivot[1] <= h):
                errors.append(f"{label}: footPivot must be within the local frame canvas")
            duration = frame.get("durationSeconds")
            if type(duration) not in (int, float) or not math.isfinite(duration) or not 0 < duration <= 60:
                errors.append(f"{label}: durationSeconds must be finite and in (0,60]")
            if type(frame.get("loop")) is not bool:
                errors.append(f"{label}: loop must be an explicit boolean")
            # Read only: cropping here calculates frame coverage, never writes pixels.
            if not rgba.getchannel("A").crop((x, y, x+w, y+h)).getbbox():
                errors.append(f"{label}: frame contains no visible pixels")
            frames_checked += 1
    return {"source": path.name, "sha256": digest, "width": width, "height": height,
        "transparentPixels": histogram[0], "partialAlphaPixels": sum(histogram[1:255]),
        "framesChecked": frames_checked, "errors": errors, "warnings": warnings,
        "structuralAdmission": not errors, "artAdmission": "not-assessed",
        "scope": "Read-only source/metadata checks; no automatic grid inference, frame extraction, animation continuity or visual approval"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--manifest", type=Path)
    args = parser.parse_args()
    result = inspect_source(args.source, json.loads(args.manifest.read_text(encoding="utf-8")) if args.manifest else None)
    print(json.dumps(result, indent=2, allow_nan=False))
    raise SystemExit(0 if result["structuralAdmission"] else 1)
