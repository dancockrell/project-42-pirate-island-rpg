class_name AtmosphereTarget
extends RefCounted

## What `Atmosphere` writes the island's weather into.
##
## The autoload decides what the sky *is* -- it reads the snapshot, reads
## `content/atmosphere/`, and interpolates -- and this is the only thing it
## knows how to talk to. Four methods, each taking one already-resolved group
## of instantaneous values; the tween lives in `Atmosphere`, so an implementer
## never owns a duration and two implementations cannot disagree about how long
## dusk takes.
##
## ## The contract is duck-typed, and deliberately so
##
## Lane P2's `WorldEnvironment` script cannot extend this class -- it already
## extends `WorldEnvironment` -- so the contract is a set of method names, not
## an inheritance edge. Anything that defines [method describes] is a target:
## `Atmosphere` prefers the first node in the running scene that does, and
## falls back to [EnvironmentAtmosphereTarget] otherwise. There is one fallback
## and it is not a second code path: it writes the same four groups through the
## same four names, straight onto Godot's own `Environment` and
## `DirectionalLight3D`, and the day P2's node is in the tree it simply stops
## being chosen.
##
## ## The groups
##
## * `sun` -- `elevation_degrees`, `azimuth_degrees`, `colour` (Color),
##   `energy`, `ambient_energy`, `ambient_colour` (Color),
##   `sky_horizon_colour` (Color), `sky_top_colour` (Color), `exposure`.
## * `fog` -- `density`, `colour` (Color), `glow_intensity`.
## * `wind` -- `strength`, `particle_kind` (`none`, `rain` or `mist`),
##   `particle_intensity`, `stilled` (bool; reduced motion).
## * `grade` -- `palette_name`, `saturation`, `tint_colour` (Color),
##   `tint_strength`, `contrast`, `heat_shift_name`,
##   `sky_hue_shift_degrees`, `horizon_desaturation`.
##
## [method set_night_lamps] is the one optional method: a target that has no
## place to put a lamp simply does not define it, and `Atmosphere` still
## resolves the lamps and reports them.

## The four required method names. `Atmosphere.is_target` reads this list, so
## the contract is stated once and checked from one place.
const REQUIRED_METHODS := ["set_sun", "set_fog", "set_wind", "set_grade"]

## The optional method name, for a target that can hold the night's lamps.
const NIGHT_LAMP_METHOD := "set_night_lamps"


## A short line naming this target, for a capture log or a failed assertion.
func describes() -> String:
	return "atmosphere target"


## Sun angle, colour, energy, the ambient it sits in and the sky behind it.
func set_sun(_sun: Dictionary) -> void:
	pass


## Depth fog and the glow that rides with it.
func set_fog(_fog: Dictionary) -> void:
	pass


## Wind strength for foliage, and the rain or mist the weather is carrying.
func set_wind(_wind: Dictionary) -> void:
	pass


## The named colour grade: corruption's desaturation and tint, and the slow
## sky shift hidden pressure is allowed to make.
func set_grade(_grade: Dictionary) -> void:
	pass


## Warm point lights at the buildings held on the island, at night. `lamps` is
## an array of `{building_id, position}` dictionaries and `lamp` is the
## authored light itself; an empty array means take them all down.
func set_night_lamps(_lamps: Array, _lamp: Dictionary) -> void:
	pass
