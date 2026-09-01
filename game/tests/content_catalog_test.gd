extends SceneTree

func _initialize() -> void:
	var catalog := ContentCatalog.new()
	assert(catalog.load_default() == OK)
	assert(catalog.has("presentation.camera.registry"))
	assert(catalog.has_registry_entry("presentation.camera.impact_punch_in"))
	assert(catalog.has("presentation.vfx.registry"))
	assert(catalog.has_registry_entry("presentation.vfx.bronze_teal_impact_arc"))
	var camera := catalog.get_registry_entry("presentation.camera.impact_punch_in")
	assert(camera.mode == "punch")
	assert(camera.zoom == 1.12)
	assert(camera.safePaddingPercent >= 8)
	assert("impact_arc" in camera.focusSubjects)
	var copy := catalog.get_registry_entry("presentation.camera.impact_punch_in")
	copy.zoom = 99.0
	assert(catalog.get_registry_entry("presentation.camera.impact_punch_in").zoom == 1.12)
	var vfx := catalog.get_registry_entry("presentation.vfx.bronze_teal_impact_arc")
	assert(vfx.purpose.length() > 10)
	assert(vfx.safeFrameOverflowAllowed == false)
	assert(vfx.envelopeWidthPercent > 0)
	var vfx_copy := catalog.get_registry_entry("presentation.vfx.bronze_teal_impact_arc")
	vfx_copy.persistenceMs = 9999
	assert(catalog.get_registry_entry("presentation.vfx.bronze_teal_impact_arc").persistenceMs != 9999)
	print("ContentCatalog registry tests passed.")
	quit(0)
