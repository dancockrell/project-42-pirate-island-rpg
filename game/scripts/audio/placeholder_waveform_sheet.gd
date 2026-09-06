extends SceneTree
## The script rather than the `PlaceholderSynth` class name: a `--script` run
## compiles an autoload before the SceneTree's global class cache exists, so
## naming the class here is a parse error there and nowhere else (measured on
## paper_razorbeak_rig_test.gd in CI). Same reason content_registry records.
const PlaceholderSynthScript := preload("res://scripts/audio/placeholder_synth.gd")
## The one owner of the palette (card P11). The sheet is a picture of this
## game's sounds and it is drawn in this game's grammar, read straight out of
## `themes/bronze_vellum.tres` -- a `--script` run has no Control to walk up
## from, so the resource as authored is what it asks for.
const ThemeTokensScript := preload("res://scripts/ui/theme_tokens.gd")

## Draws every procedural placeholder this lane generates as a waveform sheet,
## so the sounds can be *looked at*. A placeholder that renders to silence, or
## clips flat against the rails, or has no envelope at all is obvious in a
## picture and inaudible in a headless gate; this is how that gets caught
## without a pair of speakers.
##
## Run it headless, offscreen, from the `game` directory:
##
##     godot --headless --path . --script res://scripts/audio/placeholder_waveform_sheet.gd -- /abs/out.png
##
## It renders the same streams `Soundscape` renders, from the same records, and
## writes one PNG. It draws nothing and decides nothing.

const SHEET_WIDTH := 1280
const SHEET_HEIGHT := 720
const COLUMNS := 8
## The three tokens the sheet is drawn in: the deep ground, bronze for an
## ambience bed and teal for a cue.
const GROUND_TOKEN := "deep"
const BED_TOKEN := "bronze"
const CUE_TOKEN := "teal"


func _init() -> void:
	await process_frame
	var arguments := OS.get_cmdline_user_args()
	var output_path: String = arguments[0] if arguments.size() > 0 else "user://placeholder_waveforms.png"
	var grammar := ThemeTokensScript.base_theme()
	var ground := ThemeTokensScript.color(grammar, GROUND_TOKEN)
	var bed := ThemeTokensScript.color(grammar, BED_TOKEN)
	var cue := ThemeTokensScript.color(grammar, CUE_TOKEN)
	var catalog := ContentCatalog.new()
	if catalog.load_default() != OK:
		print("Waveform sheet: the content bundle did not load.")
		quit(1)
		return

	var ids: Array[String] = []
	ids.append_array(catalog.ids_with_prefix("audio.bed."))
	ids.append_array(catalog.ids_with_prefix("audio.cue."))
	var image := Image.create(SHEET_WIDTH, SHEET_HEIGHT, false, Image.FORMAT_RGBA8)
	image.fill(ground)
	var rows: int = int(ceil(float(ids.size()) / float(COLUMNS)))
	var cell_width: int = SHEET_WIDTH / COLUMNS
	var cell_height: int = SHEET_HEIGHT / maxi(rows, 1)
	var drawn := 0
	for index in ids.size():
		var stream := PlaceholderSynthScript.render(catalog.get_record(ids[index]).get("voice", {}))
		if stream == null:
			continue
		drawn += 1
		var column: int = index % COLUMNS
		var row: int = index / COLUMNS
		var left: int = column * cell_width
		var middle: int = row * cell_height + cell_height / 2
		var samples: int = stream.data.size() / 2
		var colour := cue if ids[index].begins_with("audio.cue.") else bed
		for x in cell_width - 2:
			var sample_index: int = int(float(x) / float(cell_width - 2) * float(samples - 1))
			var value: float = float(stream.data.decode_s16(sample_index * 2)) / 32768.0
			var height: int = int(absf(value) * float(cell_height - 4) * 0.5)
			for y in range(middle - height, middle + height + 1):
				if y >= 0 and y < SHEET_HEIGHT:
					image.set_pixel(left + x + 1, y, colour)
	if image.save_png(output_path) != OK:
		print("Waveform sheet: could not write %s" % output_path)
		quit(1)
		return
	print("Waveform sheet: %d of %d placeholders drawn to %s" % [drawn, ids.size(), output_path])
	quit(0)
