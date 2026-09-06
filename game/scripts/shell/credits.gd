class_name CreditsScreen
extends Control

## Attribution for everything this build admitted.
##
## Every line below is copied from a provenance record in the repository and
## names the record it came from. Nothing here is written for the screen: the
## ledgers own these facts, this screen carries them, and
## `game/tests/shell_flow_test.gd` reads the ledgers back off disk and holds the
## two equal, so a licence that changes in the ledger and not here fails the
## gate rather than shipping wrong.
##
## The ledgers cannot be read at runtime: `res://` is `game/`, and
## `content/art/` is not one of the domains the content bundle carries, so the
## records are not in the exported tree. Copying them into `game/` would make a
## second copy that drifts, and folding an `art` domain into the bundle is the
## content lane's call, not the shell's -- so the shell carries the text and the
## suite holds it to the source. When an art domain does reach the bundle, this
## constant is read from it and deleted from here.

signal closed

## Where each block came from, so a reader can check the claim.
const PLACEHOLDER_LEDGER := "content/art/placeholders.json"
const SHARED_LEDGER := "content/art/shared_asset_ledger.json"
const VENDOR_MANIFEST := "work/art/vendor/magnific/betty-3d/N2cYw4m6D9/manifest.json"

## Assets that are in the game tree today, with the provenance their records
## state. Both are stand-ins and both say so: the models are the owner's, and
## nothing in `game/assets/` is a finished asset yet.
const ADMITTED_ASSETS: Array[Dictionary] = [
	{
		"asset": "game/assets/standins/reception_terrace/reception_terrace_backdrop_v1.png",
		"record": "art.placeholder.reception_road.stage",
		"ledger": PLACEHOLDER_LEDGER,
		"field": "source",
		"text": "Built-in image generation, 2026-09-02. Generated from the approved Reception Terrace composition contract; no reference art was copied into the runtime asset."
	},
	{
		"asset": "game/assets/candidates/betty_3d/betty_candidate_v1.glb",
		"record": "art.vendor.magnific.betty_3d.candidate_01",
		"ledger": VENDOR_MANIFEST,
		"field": "source",
		"text": "Magnific, generated with Tripo 3D, downloaded 2026-09-02. https://www.magnific.com/app/3d-scene/N2cYw4m6D9/invite"
	}
]

## The shared source library. Every record is a whole source pack, every one is
## still a candidate, and no member of any of them has been admitted to the
## runtime tree -- so they are credited as what they are.
const SHARED_SOURCES: Array[Dictionary] = [
	{"record": "asset.shared.kenney.nature-kit.2_1.source", "title": "Nature Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/nature-kit"},
	{"record": "asset.shared.kenney.building-kit.source", "title": "Building Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/building-kit"},
	{"record": "asset.shared.kenney.survival-kit.source", "title": "Survival Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/survival-kit"},
	{"record": "asset.shared.kenney.modular-cave-kit.1_0.source", "title": "Modular Cave Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/modular-cave-kit"},
	{"record": "asset.shared.kenney.modular-dungeon-kit.1_0.source", "title": "Modular Dungeon Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/modular-dungeon-kit"}
]

## The ledger's own licence rule, quoted, because it is the reason the list
## above is as short as it is.
const SHARED_POLICY := "CC0-1.0 only. Other sources require project-only or access-controlled treatment unless their exact license is independently recorded as redistribution-permitted."

## What this build still owes and has no record for. It is on the credits page
## rather than in a comment because an attribution that is missing is exactly
## the thing a credits page exists to make impossible to forget.
const STILL_OWED := "The engine and every other third-party component this build links are not yet described by an admission record in this repository, so nothing is claimed for them here. When a record is admitted its attribution appears on this page."

var close_button: Button


func _init() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _ready() -> void:
	build()


func build() -> void:
	var ground := ColorRect.new()
	ground.name = "Ground"
	ground.color = ShellStyle.NIGHT
	ground.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(ground)

	var frame := MarginContainer.new()
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 160)
	frame.add_theme_constant_override("margin_right", 160)
	frame.add_theme_constant_override("margin_top", 84)
	frame.add_theme_constant_override("margin_bottom", 64)
	add_child(frame)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 10)
	frame.add_child(page)

	page.add_child(ShellStyle.label(ShellStyle.tracked("CREDITS"), 34, ShellStyle.BRONZE))
	page.add_child(ShellStyle.label("Every line here is copied from a provenance record in this repository and names it.", 15, ShellStyle.MUTED))
	page.add_child(ShellStyle.rule(ShellStyle.BRONZE, 1200.0, 2.0))
	page.add_child(spacer(10))

	var scroll := ScrollContainer.new()
	scroll.name = "CreditsScroll"
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	page.add_child(scroll)
	var body := VBoxContainer.new()
	body.name = "Body"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 8)
	scroll.add_child(body)

	body.add_child(heading("ASSETS IN THE BUILD"))
	for entry in ADMITTED_ASSETS:
		body.add_child(ShellStyle.label(str(entry["asset"]), 17, ShellStyle.CREAM))
		body.add_child(ShellStyle.label("    %s" % str(entry["text"]), 15, ShellStyle.MUTED))
		body.add_child(ShellStyle.label("    RECORD %s  •  %s" % [str(entry["record"]), str(entry["ledger"])], 13, ShellStyle.HAIRLINE.lightened(0.45)))
		body.add_child(spacer(6))

	body.add_child(heading("SHARED SOURCE LIBRARY  •  ADMITTED AS CANDIDATES, NOT YET IN THE BUILD"))
	for entry in SHARED_SOURCES:
		body.add_child(ShellStyle.label("%s — %s  •  %s  •  %s" % [str(entry["title"]), str(entry["creator"]), str(entry["license"]), str(entry["url"])], 16, ShellStyle.CREAM))
	body.add_child(spacer(4))
	body.add_child(ShellStyle.label("    %s" % SHARED_POLICY, 14, ShellStyle.MUTED))
	body.add_child(ShellStyle.label("    LEDGER %s" % SHARED_LEDGER, 13, ShellStyle.HAIRLINE.lightened(0.45)))
	body.add_child(spacer(8))

	body.add_child(heading("STILL OWED"))
	body.add_child(ShellStyle.label(STILL_OWED, 15, ShellStyle.DANGER))

	page.add_child(spacer(10))
	close_button = Button.new()
	close_button.name = "CloseCredits"
	close_button.text = "BACK"
	close_button.custom_minimum_size = Vector2(240, 48)
	close_button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	close_button.add_theme_font_size_override("font_size", 18)
	close_button.add_theme_color_override("font_color", ShellStyle.CREAM)
	close_button.add_theme_color_override("font_focus_color", ShellStyle.TEAL)
	close_button.add_theme_color_override("font_hover_color", ShellStyle.TEAL)
	close_button.add_theme_stylebox_override("normal", ShellStyle.menu_box(ShellStyle.BRONZE, Color(0, 0, 0, 0)))
	close_button.add_theme_stylebox_override("hover", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.09)))
	close_button.add_theme_stylebox_override("focus", ShellStyle.menu_box(ShellStyle.TEAL, Color(ShellStyle.TEAL, 0.13)))
	close_button.pressed.connect(close)
	page.add_child(close_button)
	close_button.grab_focus()


func heading(text: String) -> Label:
	var made := ShellStyle.label(text, 15, ShellStyle.BRONZE)
	return made


func spacer(height: int) -> Control:
	var made := Control.new()
	made.custom_minimum_size = Vector2(0, height)
	made.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return made


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		get_viewport().set_input_as_handled()
		close()


func close() -> void:
	closed.emit()
	queue_free()
