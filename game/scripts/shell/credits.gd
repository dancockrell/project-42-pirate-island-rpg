class_name CreditsScreen
extends Control

## The one door onto the palette and the type scale (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

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

## Every third-party component this build links, runs on, or is packaged by,
## from the ledger that admits them. E12 gave the engine and the bindings the
## record P8 said they did not have. A component's `notice` names the licence
## body below that it is credited under; an empty `notice` means the ledger
## carries `noticeText: null` for it, because no licence file for that component
## can be read in the environment this ledger was written from -- so this page
## says its notice is pending rather than writing one for it.
const THIRD_PARTY_LEDGER := "content/art/third_party_ledger.json"

const THIRD_PARTY: Array[Dictionary] = [
	{"record": "third_party.crate.godot", "name": "godot", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": ""},
	{"record": "third_party.crate.godot_core", "name": "godot-core", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": ""},
	{"record": "third_party.crate.godot_cell", "name": "godot-cell", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": ""},
	{"record": "third_party.crate.godot_ffi", "name": "godot-ffi", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": ""},
	{"record": "third_party.crate.godot_macros", "name": "godot-macros", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": ""},
	{"record": "third_party.crate.glam", "name": "glam", "version": "0.32.1", "license": "MIT OR Apache-2.0", "url": "https://github.com/bitshifter/glam-rs", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.libc", "name": "libc", "version": "0.2.189", "license": "MIT OR Apache-2.0", "url": "https://github.com/rust-lang/libc", "role": "linked_into_the_gdextension", "notice": "mit_rust_project_developers"},
	{"record": "third_party.crate.proc_macro2", "name": "proc-macro2", "version": "1.0.107", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/proc-macro2", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.quote", "name": "quote", "version": "1.0.47", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/quote", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.unicode_ident", "name": "unicode-ident", "version": "1.0.24", "license": "(MIT OR Apache-2.0) AND Unicode-3.0", "url": "https://github.com/dtolnay/unicode-ident", "role": "linked_into_the_gdextension", "notice": "mit_and_unicode_3_0"},
	{"record": "third_party.crate.venial", "name": "venial", "version": "0.6.1", "license": "MIT", "url": "https://github.com/PoignardAzur/venial", "role": "linked_into_the_gdextension", "notice": "mit_olivier_faure"},
	{"record": "third_party.crate.serde", "name": "serde", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_core", "name": "serde_core", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_derive", "name": "serde_derive", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.syn", "name": "syn", "version": "3.0.4", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/syn", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_json", "name": "serde_json", "version": "1.0.151", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/json", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.itoa", "name": "itoa", "version": "1.0.18", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/itoa", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.memchr", "name": "memchr", "version": "2.8.3", "license": "Unlicense OR MIT", "url": "https://github.com/BurntSushi/memchr", "role": "linked_into_the_gdextension", "notice": "mit_andrew_gallant"},
	{"record": "third_party.crate.zmij", "name": "zmij", "version": "1.0.23", "license": "MIT", "url": "https://github.com/dtolnay/zmij", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.engine.godot", "name": "Godot Engine", "version": "4.7.2-stable", "license": "MIT", "url": "https://github.com/godotengine/godot", "role": "runs_the_game", "notice": ""},
	{"record": "third_party.engine.godot_export_templates", "name": "Godot Engine export templates", "version": "4.7.2-stable", "license": "MIT", "url": "https://github.com/godotengine/godot", "role": "packaged_into_every_exported_build", "notice": ""},
	{"record": "third_party.ci.mesa", "name": "Mesa (mesa-vulkan-drivers, lavapipe)", "version": "", "license": "MIT", "url": "https://gitlab.freedesktop.org/mesa/mesa", "role": "continuous_integration_only", "notice": ""}
]


const NOTICE_BODIES := {
	"mit_permission_only": """Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.""",
	"mit_rust_project_developers": """Copyright (c) The Rust Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.""",
	"mit_and_unicode_3_0": """Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2023 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.""",
	"mit_olivier_faure": """MIT License

Copyright (c) 2022 Olivier FAURE

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.""",
	"mit_andrew_gallant": """The MIT License (MIT)

Copyright (c) 2015 Andrew Gallant

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.""",
}

## What this build still owes. It is on the credits page rather than in a
## comment because an attribution that is missing is exactly the thing a
## credits page exists to make impossible to forget. P8 owed every third-party
## component a record; what is owed now is narrower and named.
## How many components the ledger carries with no notice text. Held equal to
## the ledger's own `noticePendingCount` by the shell suite, so a notice that
## arrives and is not put on this page fails the gate.
const NOTICES_PENDING := 8

const STILL_OWED := "%d of the components above are credited with the licence their own metadata declares and no notice text, because no licence file for them exists in the tree this ledger was written from: the five godot-rust crates publish none, and the pinned Godot download is the executable alone. Their notices are pending, not waived."

var close_button: Button


func _init() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _ready() -> void:
	ThemeTokensScript.adopt(self)
	build()


func build() -> void:
	var ground := ColorRect.new()
	ground.name = "Ground"
	ShellStyle.paint(ground, theme, "night")
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

	page.add_child(ShellStyle.label(ShellStyle.tracked("CREDITS"), ShellStyle.DISPLAY))
	page.add_child(ShellStyle.label("Every line here is copied from a provenance record in this repository and names it.", ShellStyle.MUTED))
	page.add_child(ShellStyle.rule(theme, "bronze", 1200.0, 2.0))
	page.add_child(spacer(10))

	var scroll := ScrollContainer.new()
	scroll.name = "CreditsScroll"
	# The notice bodies wrap, and a wrapping label needs a settled width: with
	# horizontal scrolling on, the container would grow sideways to fit the
	# longest line instead and the page would scroll both ways.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	page.add_child(scroll)
	var body := VBoxContainer.new()
	body.name = "Body"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 8)
	scroll.add_child(body)

	body.add_child(heading("ASSETS IN THE BUILD"))
	for entry in ADMITTED_ASSETS:
		body.add_child(ShellStyle.label(str(entry["asset"]), ShellStyle.SUBTITLE))
		body.add_child(ShellStyle.label("    %s" % str(entry["text"]), ShellStyle.MUTED))
		body.add_child(ShellStyle.label("    RECORD %s  •  %s" % [str(entry["record"]), str(entry["ledger"])], ShellStyle.FAINT))
		body.add_child(spacer(6))

	body.add_child(heading("SHARED SOURCE LIBRARY  •  ADMITTED AS CANDIDATES, NOT YET IN THE BUILD"))
	for entry in SHARED_SOURCES:
		body.add_child(ShellStyle.label("%s — %s  •  %s  •  %s" % [str(entry["title"]), str(entry["creator"]), str(entry["license"]), str(entry["url"])], ShellStyle.BODY))
	body.add_child(spacer(4))
	body.add_child(ShellStyle.label("    %s" % SHARED_POLICY, ShellStyle.MUTED))
	body.add_child(ShellStyle.label("    LEDGER %s" % SHARED_LEDGER, ShellStyle.FAINT))
	body.add_child(spacer(8))

	body.add_child(heading("THIRD-PARTY COMPONENTS  •  THE ENGINE, THE BINDINGS, AND EVERY CRATE THE EXTENSION LINKS"))
	# One line each, and the line says whether a notice exists. A component with
	# no notice is drawn in the colour the page already uses for what is owed,
	# because that is what it is.
	for component in THIRD_PARTY:
		var version := str(component["version"])
		var titled: String = str(component["name"]) if version == "" else "%s %s" % [str(component["name"]), version]
		var notice := str(component["notice"])
		var standing := str(ROLES.get(str(component["role"]), component["role"]))
		var status := "NOTICE PENDING" if notice == "" else "notice below: %s" % notice
		var tone: StringName = ShellStyle.DANGER if notice == "" else ShellStyle.BODY
		body.add_child(ShellStyle.label("%s  •  %s  •  %s  •  %s  •  %s" % [titled, str(component["license"]), str(component["url"]), standing, status], tone))
	body.add_child(spacer(4))
	body.add_child(ShellStyle.label("    LEDGER %s" % THIRD_PARTY_LEDGER, ShellStyle.FAINT))
	body.add_child(spacer(8))

	body.add_child(heading("LICENCE NOTICES  •  COPIED FROM EACH PROJECT'S OWN LICENCE FILE"))
	for key in NOTICE_BODIES:
		body.add_child(ShellStyle.label(str(key), ShellStyle.BRONZE))
		body.add_child(ShellStyle.label(credited_under(str(key)), ShellStyle.FAINT))
		var notice_label := ShellStyle.label(str(NOTICE_BODIES[key]), ShellStyle.MUTED)
		notice_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		notice_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		body.add_child(notice_label)
		body.add_child(spacer(6))
	body.add_child(spacer(4))

	body.add_child(heading("STILL OWED"))
	body.add_child(ShellStyle.label(STILL_OWED % NOTICES_PENDING, ShellStyle.DANGER))

	page.add_child(spacer(10))
	close_button = Button.new()
	close_button.name = "CloseCredits"
	close_button.text = "BACK"
	close_button.custom_minimum_size = Vector2(240, 48)
	close_button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	close_button.theme_type_variation = ShellStyle.MENU_ROW
	close_button.pressed.connect(close)
	page.add_child(close_button)
	close_button.grab_focus()


## What each `role` means in words, so the page never prints a machine token.
const ROLES := {
	"linked_into_the_gdextension": "linked into the simulation extension",
	"runs_the_game": "the engine this build runs on",
	"packaged_into_every_exported_build": "packaged into every exported build",
	"continuous_integration_only": "used only to render this project's CI captures",
}


## The components credited under one licence body, named so a reader can see
## which notice covers what without counting lines.
func credited_under(key: String) -> String:
	var names: Array[String] = []
	for component in THIRD_PARTY:
		if str(component["notice"]) == key:
			names.append(str(component["name"]))
	return "    covers %s" % ", ".join(names)


## The grammar changed under the page. Every Label and the Back row are Theme
## items and have already moved; the ground and the rule are flat fills.
func _on_theme_rebuilt(rebuilt: Theme) -> void:
	theme = rebuilt
	ShellStyle.repaint_marked(self, rebuilt)


func heading(text: String) -> Label:
	return ShellStyle.label(text, ShellStyle.BRONZE)


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
