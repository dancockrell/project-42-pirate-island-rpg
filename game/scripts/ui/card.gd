class_name VellumCard
extends PanelContainer

## One person, as the grammar draws them: portrait slot, name, band, vitality
## and guard, Composure, and the Shaken mark.
##
## Data in, nothing out. `configure` takes the actor dictionary a snapshot
## already carries and this control makes no call of its own -- no bridge, no
## autoload, no content lookup -- so the same card draws a live actor, a review
## fixture and a test row identically. It is the reusable form of the battle
## prototype's `PaperCard` (card D11); the prototype adopts it in P6.
##
## The portrait is a **procedurally drawn placeholder**, marked as one on the
## face of the card. Models and illustrated portraits are the owner's; when one
## arrives it is assigned to `portrait_texture` and the placeholder stops being
## drawn. Nothing else about the card changes that day.

signal pressed(actor_id: String)

## The keys `configure` reads. Anything else in the dictionary is ignored, so a
## snapshot that grows does not break a card.
const FIELDS := ["actor_id", "display_name", "band_name", "vitality", "vitality_maximum",
	"guard", "guard_maximum", "composure", "composure_maximum", "shaken", "portrait_kind"]

const CARD_SIZE := Vector2(256, 158)
const PORTRAIT_SIZE := Vector2(50, 62)

var actor_id := ""
var display_name := ""
var band_name := ""
var vitality := 0
var vitality_maximum := 0
var guard := 0
var guard_maximum := 0
var composure := 0
var composure_maximum := 0
var shaken := false
var portrait_kind := ""
var selected := false

## Set when the owner's portrait art exists. While it is null the placeholder
## below is drawn instead, and the card says so.
var portrait_texture: Texture2D = null

var _portrait: Control
var _name_label: Label
var _band_label: Label
var _vitality_row: VellumPipRow
var _guard_row: VellumPipRow
var _composure_row: VellumPipRow
var _shaken_label: Label
var _button: Button


func _ready() -> void:
	custom_minimum_size = CARD_SIZE
	if _name_label == null:
		_build()
	_refresh()


func _notification(what: int) -> void:
	if what == NOTIFICATION_THEME_CHANGED:
		_refresh()


## The one way data reaches a card.
func configure(actor: Dictionary) -> void:
	actor_id = str(actor.get("actor_id", ""))
	display_name = str(actor.get("display_name", ""))
	band_name = str(actor.get("band_name", ""))
	vitality = int(actor.get("vitality", 0))
	vitality_maximum = maxi(int(actor.get("vitality_maximum", 0)), vitality)
	guard = int(actor.get("guard", 0))
	guard_maximum = maxi(int(actor.get("guard_maximum", 0)), guard)
	composure = int(actor.get("composure", 0))
	composure_maximum = maxi(int(actor.get("composure_maximum", 0)), composure)
	shaken = bool(actor.get("shaken", false))
	portrait_kind = str(actor.get("portrait_kind", ""))
	if _name_label == null:
		_build()
	_refresh()


func set_selected(value: bool) -> void:
	selected = value
	_refresh()


func _build() -> void:
	mouse_filter = Control.MOUSE_FILTER_PASS
	var column := VBoxContainer.new()
	column.name = "Column"
	column.add_theme_constant_override("separation", 6)
	add_child(column)

	var head := HBoxContainer.new()
	head.name = "Head"
	head.add_theme_constant_override("separation", 10)
	column.add_child(head)

	_portrait = Control.new()
	_portrait.name = "PortraitSlot"
	_portrait.custom_minimum_size = PORTRAIT_SIZE
	_portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_portrait.draw.connect(_draw_portrait)
	head.add_child(_portrait)

	var titles := VBoxContainer.new()
	titles.name = "Titles"
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.add_theme_constant_override("separation", 2)
	head.add_child(titles)
	_name_label = Label.new()
	_name_label.name = "Name"
	_name_label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	titles.add_child(_name_label)
	_band_label = Label.new()
	_band_label.name = "Band"
	titles.add_child(_band_label)
	_shaken_label = Label.new()
	_shaken_label.name = "Shaken"
	_shaken_label.text = "SHAKEN"
	titles.add_child(_shaken_label)

	_vitality_row = _add_meter(column, "Vitality", "VITALITY", VellumPipRow.Shape.BAR)
	_guard_row = _add_meter(column, "Guard", "GUARD", VellumPipRow.Shape.BAR)
	_composure_row = _add_meter(column, "Composure", "COMPOSURE", VellumPipRow.Shape.DOT)

	_button = Button.new()
	_button.name = "Select"
	_button.flat = true
	_button.focus_mode = Control.FOCUS_ALL
	_button.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_button.pressed.connect(func() -> void: pressed.emit(actor_id))
	add_child(_button)


func _add_meter(column: VBoxContainer, node_name: String, caption: String, shape: VellumPipRow.Shape) -> VellumPipRow:
	var row := HBoxContainer.new()
	row.name = node_name
	row.add_theme_constant_override("separation", 8)
	column.add_child(row)
	var label := Label.new()
	label.name = "Caption"
	label.text = caption
	label.custom_minimum_size.x = 78
	row.add_child(label)
	var pips := VellumPipRow.new()
	pips.name = "Pips"
	pips.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pips.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(pips)
	return pips

## Applying a Theme override inside `_refresh` makes Godot send
## NOTIFICATION_THEME_CHANGED straight back, which would call `_refresh` again
## for ever. One flag, and a restyle is one pass.
var _refreshing := false



func _refresh() -> void:
	if _refreshing or _name_label == null:
		return
	_refreshing = true
	var active := ThemeTokens.active(self)
	var style_name := "panel"
	if shaken:
		style_name = "alarmed"
	elif selected:
		style_name = "selected"
	add_theme_stylebox_override("panel", ThemeTokens.style(active, "VellumCard", style_name))

	_name_label.text = display_name
	_name_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "title"))
	_name_label.add_theme_color_override("font_color", ThemeTokens.color(active, "cream"))
	_band_label.text = band_name
	_band_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
	_band_label.add_theme_color_override("font_color", ThemeTokens.color(active, "muted"))
	_shaken_label.visible = shaken
	_shaken_label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
	_shaken_label.add_theme_color_override("font_color", ThemeTokens.color(active, "danger_soft"))

	_vitality_row.configure(vitality_maximum, vitality, "teal", VellumPipRow.Shape.BAR)
	_guard_row.configure(guard_maximum, guard, "bronze", VellumPipRow.Shape.BAR)
	_composure_row.configure(composure_maximum, composure, "danger" if shaken else "teal", VellumPipRow.Shape.DOT)
	for caption in [_vitality_row, _guard_row, _composure_row]:
		var label := (caption as Control).get_parent().get_node("Caption") as Label
		label.add_theme_font_size_override("font_size", ThemeTokens.font_size(active, "caption"))
		label.add_theme_color_override("font_color", ThemeTokens.color(active, "muted"))
	_portrait.queue_redraw()


## The placeholder. Two arcs and a block: enough to hold the space a portrait
## will take and to read as "no art yet", and deliberately not a drawing of a
## person -- the people are the owner's. Marked on the card by the hairline
## cross, which no illustrated portrait will carry.
	_refreshing = false
func _draw_portrait() -> void:
	var active := ThemeTokens.active(self)
	var rect := Rect2(Vector2.ZERO, _portrait.size)
	if portrait_texture != null:
		_portrait.draw_texture_rect(portrait_texture, rect, false)
		return
	_portrait.draw_rect(rect, ThemeTokens.color(active, "panel_sunken"))
	_portrait.draw_rect(rect, ThemeTokens.color(active, "rule_faint"), false, 1.0)
	var accent := ThemeTokens.color(active, "danger" if shaken else "teal_deep")
	var head_centre := Vector2(rect.size.x * 0.5, rect.size.y * 0.36)
	_portrait.draw_arc(head_centre, rect.size.x * 0.19, 0.0, TAU, 24, accent, 1.5, true)
	var shoulders := Rect2(rect.size.x * 0.18, rect.size.y * 0.62, rect.size.x * 0.64, rect.size.y * 0.3)
	_portrait.draw_rect(shoulders, accent, false, 1.5)
	var faint := ThemeTokens.color(active, "rule_faint")
	_portrait.draw_line(rect.position, rect.end, faint, 1.0)
	_portrait.draw_line(Vector2(rect.end.x, rect.position.y), Vector2(rect.position.x, rect.end.y), faint, 1.0)
