# Scene captures — how to look at this game

Every presentation card ends in a picture, because a change to how the game
looks cannot be reviewed from a diff. This directory is where those pictures
are kept, and this file is how to make your own.

## The one command

```bash
GODOT=/path/to/Godot_v4.7.2-stable_linux.x86_64 \
CAPTURE_WIDTH=1280 CAPTURE_HEIGHT=720 \
  xvfb-run -a -s "-screen 0 1280x720x24" bash tools/capture-scenes.sh
```

On Windows, `tools/capture-scenes.ps1` does the same thing with the same
defaults; it is the twin of the shell script, not a second implementation.

It renders every `.tscn` under `game/scenes/review`, `game/scenes/world`,
`game/scenes/battle` and `game/scenes/shell` — recursively, by glob, so a scene
is captured the day it lands and nobody has to remember to add it to a list — and
writes each one to `work/captures/<scene>.png` with a `<scene>.thumb.png`
beside it. `work/captures/` is git-ignored: it is build output, rebuilt in about
twelve seconds.

Defaults are 1920×1080 with 1280×720 thumbnails, which is what CI does.
Locally, set `CAPTURE_WIDTH`/`CAPTURE_HEIGHT` to 1280×720 as above: the owner's
machine runs many threads at once and a full-size software render is not free.
The other knobs are `CAPTURE_OUTPUT_DIR`, `CAPTURE_THUMBNAIL_WIDTH`,
`CAPTURE_THUMBNAIL_HEIGHT` and `CAPTURE_FRAMES` (settle frames, default 6).

### One scene while you iterate

```bash
xvfb-run -a -s "-screen 0 1280x720x24" \
  "$GODOT" --rendering-driver opengl3 --audio-driver Dummy --path game \
  --script res://tools/capture_review_scene.gd -- \
  res://scenes/review/reception_terrace_setpiece_review.tscn \
  --output /absolute/path/out.png --size 1280x720 --frames 6
```

`game/tools/capture_review_scene.gd` takes a *list* of scenes and renders all of
them in one engine process. Never loop a shell over it one scene at a time: an
engine start costs about ten seconds and a great deal of memory, and other
people are working on the same machine. Run Godot `--headless` or under
`xvfb-run`, never in a window, one process at a time, with `nice -n 10`, a
`timeout`, and `ulimit -v 4000000` in the same shell.

## What the gate refuses

* **Any `ERROR:` or `SCRIPT ERROR:` line**, exactly as `tools/verify-godot.sh`
  refuses one. An exit code is not the only verdict.
* **A uniform frame.** The capture is downsampled and the standard deviation of
  its luminance measured on the 0..255 scale; below 0.65 the capture is refused
  and the run exits 1. A black render, an environment clear colour with nothing
  in front of it, and a camera inside geometry all measure at or near zero. For
  scale: the review scenes measure 20–45, and the thinnest picture in the
  project — a wireframe collision proxy on flat grey — measures 1.1.

Two things the tool does *for* a scene, both of them capture framing and never
authored composition:

* A 3D scene with no camera of its own gets one from the tool, fitted to the
  merged bounds of everything the scene can put on screen, from a fixed
  three-quarter angle so two runs frame it identically. An authored camera
  always wins.
* A scene that draws nothing at all is rendered a second time with the engine's
  collision and navigation debug drawing on, so a proxy-only scene such as
  `scenes/world/black_beach/reception_terrace_collision.tscn` shows its proxy
  instead of an empty frame. A scene that did draw is never captured with debug
  drawing on, so no wireframe lands over art under review.

A scene with genuinely nothing renderable in it — today only
`scenes/world/black_beach/reception_terrace_navigation.tscn`, an empty
`NavigationRegion3D` whose own metadata says
`authored_navigation_shell_pending_baked_walk_mesh` — is reported as skipped and
named in the run's summary. It is not a pass and not a failure: there is no
picture of it to take. The day it gains geometry it is captured like everything
else.

## Where CI puts them

The `captures` job in `.github/workflows/verify.yml` runs on every push. It
installs `xvfb`, `mesa-vulkan-drivers` and `vulkan-tools`, downloads the same
pinned Godot 4.7.2 the `godot-suites` job does (one pin, at the top of the
workflow, read by both), builds the debug GDExtension the same way, and runs
`tools/capture-scenes.sh` under Xvfb at 1920×1080.

It uploads two artifacts per run:

* **`captures-<sha>`** — every PNG and thumbnail. `if-no-files-found: error`, so
  an empty artifact fails the job rather than being uploaded quietly.
* **`captures-log-<sha>`** — the run's full output, including the driver it
  chose and the luminance measurement for every scene.

To look: open the run on GitHub → *Summary* → *Artifacts* → `captures-<sha>`.

Because `mesa-vulkan-drivers` is installed there, CI reports a Vulkan device
(lavapipe) and the script selects `--rendering-driver vulkan`, so CI captures
show the Forward+ path. A container without lavapipe falls back to `opengl3`,
which is the method `game/project.godot` declares. The chosen driver is printed
at the top of every run and belongs in any judgement of the images: the two do
not light a scene identically.

## The committed ledger

Files here are named `<card>-<short sha>-<scene>.png`, where the SHA is the
commit the capture was taken from. They are the evidence a card was looked at,
not a texture source and not an asset: nothing in `game/` loads them.

They are 512×288 so that every file stays well under the 400 KB the ledger caps
each image at — a photographic backdrop such as the battle prototype's is a
large PNG at any generous size. Full-resolution renders are in the CI artifact
for the same commit. Reproduce a ledger set with:

```bash
"$GODOT" --rendering-driver opengl3 --audio-driver Dummy --path game \
  --script res://tools/capture_review_scene.gd -- \
  <every scene> --out-dir /absolute/dir --thumbnails \
  --size 1280x720 --thumbnail-size 512x288
```
