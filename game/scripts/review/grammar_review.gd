extends Control

## The style guide: every component of the bronze-and-vellum grammar, in every
## state, on one page (card P5).
##
## It is a review scene, not a screen: nothing here reads the simulation, and
## every value is a fixture stated in this file. The point is to be *looked at*
## -- `docs/verification/captures/` carries what it renders -- so that the
## grammar can be judged as a whole rather than one control at a time.
##
## The right-hand column wears a second Theme built with 130% text and high
## contrast, so the accessibility settings are visible beside the defaults
## instead of being taken on trust.

const REVIEW_TEXT_SCALE := 1.1

## The sheet is laid out on the canvas the project declares
## (`display/window/size/viewport`), and then scaled as a whole to whatever the
## capture or the window actually is. A style guide has to be looked at
## complete, and a page that reflowed differently at every capture size would be
## a picture of the capture rather than of the grammar.
const SHEET_SIZE := Vector2(1920.0, 1080.0)

## The three actors on the demonstration rail. Ids are the slice's real ones;
## the numbers are a fixture and decide nothing.
const RAIL_FIXTURE := [
	{
		"actor_id": "character.protagonist.captain", "display_name": "Captain Michael",
		"band_name": "party_front", "vitality": 22, "vitality_maximum": 24,
		"guard": 5, "guard_maximum": 8, "composure": 10, "composure_maximum": 10,
		"shaken": false, "portrait_kind": "captain",
	},
	{
		"actor_id": "character.heroine.betty", "display_name": "Betty",
		"band_name": "party_front", "vitality": 18, "vitality_maximum": 24,
		"guard": 2, "guard_maximum": 8, "composure": 7, "composure_maximum": 10,
		"shaken": false, "portrait_kind": "heroine",
	},
	{
		"actor_id": "character.heroine.vix", "display_name": "Vix",
		"band_name": "party_rear", "vitality": 6, "vitality_maximum": 24,
		"guard": 0, "guard_maximum": 8, "composure": 2, "composure_maximum": 10,
		"shaken": true, "portrait_kind": "heroine",
	},
	{
		"actor_id": "character.heroine.ayla", "display_name": "Ayla",
		"band_name": "party_rear", "vitality": 0, "vitality_maximum": 24,
		"guard": 0, "guard_maximum": 8, "composure": 0, "composure_maximum": 10,
		"shaken": false, "portrait_kind": "heroine",
	},
	{
		"actor_id": "character.companion.grisha", "display_name": "Grisha",
		"band_name": "contested", "vitality": 24, "vitality_maximum": 24,
		"guard": 8, "guard_maximum": 8, "composure": 10, "composure_maximum": 10,
		"shaken": false, "portrait_kind": "captain",
	},
]

const COMMAND_FIXTURE := [
	{"command_id": "skill.betty.guarded_strike", "rank": "D", "display_name": "Guarded Strike",
		"state": VellumCommandDiamond.State.UNLOCKED},
	{"command_id": "skill.betty.rally", "rank": "C", "display_name": "Rally",
		"state": VellumCommandDiamond.State.UNLOCKED},
	{"command_id": "skill.betty.mobile_infirmary", "rank": "A", "display_name": "Mobile Infirmary",
		"state": VellumCommandDiamond.State.REFUSED, "reason": "No wounded ally is in reach."},
	{"command_id": "skill.betty.combat_revival", "rank": "SSS", "display_name": "Combat Revival",
		"state": VellumCommandDiamond.State.LOCKED, "reason": "Locked until its authored bond milestone."},
]

const JOURNAL_FIXTURE := [
	{"day": 12, "hour": 3, "level": InformationLevel.Level.AMBIENT,
		"prose": "Hour 3 passed on the island."},
	{"day": 12, "hour": 9, "level": InformationLevel.Level.NOTABLE,
		"prose": "A force of faction.pirates reached world.cell.black_beach.landing."},
	{"day": 12, "hour": 11, "level": InformationLevel.Level.URGENT,
		"prose": "A force of faction.pirates reached world.cell.black_beach.reception_terrace."},
]

## A directive that has been explained but not yet confirmed, in the shape
## `Project42ExpeditionBridge.strategic_surface` projects. Every word of the
## explanation is `ExpeditionState::explain_directive`'s.
const DIRECTIVE_FIXTURE := {
	"id": "directive.hold_the_landing",
	"faction_id": "faction.michael",
	"intent": "protect",
	"priority": "standing",
	"target_node_id": "world.cell.black_beach.landing",
	"explanation": {
		"goal": "hold what it already has and keep it working",
		"why_target": "world.cell.black_beach.landing is two roads from the nearest holding and is held by nobody",
		"resources": "no limit was stated, and the faction has 34 in stores",
		"blockers": ["the road route.black_beach.shore_path is contested, which makes it more dangerous than it was built to be"],
		"withdrawal_conditions": "if the shore path closes behind it",
		"party_could_help": true,
	},
}

var _surface: Node
var _sheet: Control


func _ready() -> void:
	_surface = get_node_or_null("/root/InformationSurface")
	theme = ThemeTokens.build()
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var ground := ColorRect.new()
	ground.name = "Ground"
	ground.color = ThemeTokens.color(theme, "deep")
	ground.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(ground)

	_sheet = Control.new()
	_sheet.name = "Sheet"
	_sheet.size = SHEET_SIZE
	_sheet.pivot_offset = Vector2.ZERO
	add_child(_sheet)
	get_viewport().size_changed.connect(_fit_sheet)
	_fit_sheet()
	call_deferred("_fit_sheet")

	var frame := MarginContainer.new()
	frame.name = "Frame"
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 46)
	frame.add_theme_constant_override("margin_right", 46)
	frame.add_theme_constant_override("margin_top", 26)
	frame.add_theme_constant_override("margin_bottom", 20)
	_sheet.add_child(frame)

	var page := VBoxContainer.new()
	page.name = "Page"
	page.add_theme_constant_override("separation", 10)
	frame.add_child(page)

	page.add_child(_masthead())
	page.add_child(_upper_row())
	page.add_child(_party_rail())
	page.add_child(_information_row())


## Scale the whole sheet to fit whatever it is being rendered into, and centre
## it. One transform, so nothing inside ever reflows.
func _fit_sheet() -> void:
	if _sheet == null:
		return
	# The viewport rather than this control's own size: a Control added to the
	# window has not been laid out yet when `_ready` runs, and a factor of zero
	# would make the sheet vanish.
	var into := get_viewport_rect().size
	var factor: float = minf(into.x / SHEET_SIZE.x, into.y / SHEET_SIZE.y)
	if factor <= 0.0:
		return
	_sheet.scale = Vector2(factor, factor)
	_sheet.position = (into - SHEET_SIZE * factor) * 0.5


func _masthead() -> Control:
	var column := VBoxContainer.new()
	column.name = "Masthead"
	column.add_theme_constant_override("separation", 4)
	var title := _label("BRONZE AND VELLUM", "DisplayLabel")
	column.add_child(title)
	column.add_child(_label(
		"The one grammar: palette, type scale and surfaces, owned by themes/bronze_vellum.tres. The directive panel wears 110% text, high contrast and reduced motion.",
		"MutedLabel"))
	column.add_child(_hairline())
	return column


func _upper_row() -> Control:
	var row := HBoxContainer.new()
	row.name = "UpperRow"
	row.add_theme_constant_override("separation", 20)

	var palette := VellumPanel.new()
	palette.name = "PalettePanel"
	palette.heading = "Palette"
	palette.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	palette.body().add_child(_swatches())
	row.add_child(palette)

	var type_panel := VellumPanel.new()
	type_panel.name = "TypePanel"
	type_panel.heading = "Type scale"
	type_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for specimen in [
		["DisplayLabel", "Display · 34"],
		["TitleLabel", "Title · 22"],
		["BodyLabel", "Body · 15 — the island holds while you read."],
		["CaptionLabel", "CAPTION · 12"],
		["MonoLabel", "world.cell.black_beach.landing"],
	]:
		type_panel.body().add_child(_label(str(specimen[1]), str(specimen[0])))
	row.add_child(type_panel)

	var commands := VellumPanel.new()
	commands.name = "CommandPanel"
	commands.heading = "Command grid"
	commands.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var grid := VellumCommandGrid.new()
	grid.name = "Commands"
	grid.columns = 4
	commands.body().add_child(grid)
	grid.set_commands(COMMAND_FIXTURE)
	var tooltip := VellumTooltip.new()
	tooltip.name = "Tooltip"
	tooltip.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	commands.body().add_child(tooltip)
	tooltip.configure("A rank · Mobile Infirmary", "No wounded ally is in reach.",
		"skill.betty.mobile_infirmary")
	row.add_child(commands)
	return row


func _party_rail() -> Control:
	var panel := VellumPanel.new()
	panel.name = "RailPanel"
	panel.heading = "The party rail"
	var rail := VellumCardRail.new()
	rail.name = "Rail"
	rail.centred = true
	panel.body().add_child(rail)
	rail.set_entries(RAIL_FIXTURE)
	rail.select("character.heroine.betty")
	panel.body().add_child(_label(
		"Ordinary · selected · Shaken · defeated · whole. Unlocked · unlocked · refused · locked. Portraits are procedural placeholders, marked by the hairline cross; the illustrated package replaces them.",
		"CaptionLabel"))
	return panel


func _information_row() -> Control:
	var row := HBoxContainer.new()
	row.name = "InformationRow"
	row.add_theme_constant_override("separation", 20)
	row.size_flags_vertical = Control.SIZE_EXPAND_FILL

	var levels := VellumPanel.new()
	levels.name = "LevelsPanel"
	levels.heading = "Three levels"
	levels.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	levels.body().add_child(_label(
		"AMBIENT — silent. It marks the journal and shows nothing at all.", "CaptionLabel"))
	levels.body().add_child(_notice_preview(InformationLevel.Level.NOTABLE, 84,
		"A force of faction.pirates reached world.cell.black_beach.landing.",
		"", "Betty"))
	levels.body().add_child(_notice_preview(InformationLevel.Level.URGENT, 130,
		"They are at the terrace.",
		"A force of faction.pirates reached the cell the party is standing in. The island is held until you answer.",
		"Urgent · the party"))
	row.add_child(levels)

	var journal := VellumPanel.new()
	journal.name = "JournalPanel"
	journal.heading = "Journal, readable while paused"
	journal.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for entry in JOURNAL_FIXTURE:
		var line := VellumJournalEntry.new()
		journal.body().add_child(line)
		line.configure(entry as Dictionary)
	row.add_child(journal)

	var confirm := VellumPanel.new()
	confirm.name = "DirectivePanel"
	confirm.heading = "Before you confirm"
	confirm.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	# The right-hand column wears the accessibility settings, so 130% text and
	# the high-contrast token set are visible beside the defaults.
	confirm.theme = ThemeTokens.build(REVIEW_TEXT_SCALE, true, true)
	var identifier := _label(str(DIRECTIVE_FIXTURE["id"]), "MonoLabel")
	confirm.body().add_child(identifier)
	for line in _explanation_lines():
		var paragraph := _label(line, "BodyLabel")
		paragraph.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		confirm.body().add_child(paragraph)
	var confirm_button := Button.new()
	confirm_button.name = "Confirm"
	confirm_button.text = "CONFIRM THIS DIRECTIVE"
	confirm.body().add_child(confirm_button)
	row.add_child(confirm)
	return row


## The explanation, through the surface when the autoload is present so the
## review shows the real labelling rather than a copy of it.
func _explanation_lines() -> PackedStringArray:
	if _surface != null:
		return _surface.directive_explanation_lines(DIRECTIVE_FIXTURE)
	var lines: PackedStringArray = []
	var explanation := DIRECTIVE_FIXTURE["explanation"] as Dictionary
	lines.append("What it becomes: %s" % explanation["goal"])
	return lines


func _notice_preview(level: int, height: int, headline: String, body: String, source: String) -> Control:
	var stage := Control.new()
	stage.name = "NoticeStage%d" % level
	stage.custom_minimum_size = Vector2(0, height)
	stage.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var notice := VellumNotice.present(level, headline, body, source)
	notice.preview = true
	stage.add_child(notice)
	return stage


func _swatches() -> Control:
	var grid := GridContainer.new()
	grid.name = "Swatches"
	grid.columns = 4
	grid.add_theme_constant_override("h_separation", 8)
	grid.add_theme_constant_override("v_separation", 4)
	for token in ThemeTokens.TOKENS:
		var cell := HBoxContainer.new()
		cell.add_theme_constant_override("separation", 8)
		var chip := ColorRect.new()
		chip.color = ThemeTokens.color(theme, token)
		chip.custom_minimum_size = Vector2(22, 22)
		cell.add_child(chip)
		cell.add_child(_label(token, "CaptionLabel"))
		grid.add_child(cell)
	return grid


func _hairline() -> Control:
	var rule := ColorRect.new()
	rule.name = "Hairline"
	rule.color = ThemeTokens.color(theme, "rule_faint")
	rule.custom_minimum_size = Vector2(0, 1)
	return rule


func _label(text: String, variation: String) -> Label:
	var label := Label.new()
	label.text = text
	label.theme_type_variation = variation
	return label
