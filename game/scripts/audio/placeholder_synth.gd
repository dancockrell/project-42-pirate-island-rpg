class_name PlaceholderSynth
extends RefCounted

## Renders an audio record's `voice` block into an [AudioStreamWAV].
##
## Every sound in this game is a placeholder. No audio file has been admitted to
## the repository, so rather than ship silence -- which would let a missing cue
## pass unnoticed -- each record describes a sound in parameters (waveform,
## pitch and glide, noise mix, envelope, low-pass, tremolo, duration, loop) and
## this synth renders it once at load. Each result is honestly a stand-in: a
## bronze hit is a filtered triangle and noise burst, not a mace on armour. The
## record's `asset` block carries the status, the generator's name and the empty
## licence, source and author fields that must be filled before a real file is
## admitted in its place.
##
## Determinism, per the repository's rule: nothing here draws an unseeded
## number. Noise comes from a small linear congruential generator seeded from
## the record's own authored `voice.seed`, so the same record renders the same
## samples on every machine and every run. The validator refuses a voice that
## mixes noise without a positive seed.
##
## The card that ordered this work named `AudioStreamGenerator`. A generator
## fills a playback buffer frame by frame while the sound plays, which would put
## the synth on the audio thread and re-draw the same samples for every replay.
## Baking the same parameters into one `AudioStreamWAV` at load renders each
## record exactly once, plays it from as many players as the scene wants, and is
## what makes the result byte-identical run to run -- so that is what this does.

## Mono, 16-bit, 22.05 kHz. High enough for a placeholder to be legible,
## low enough that rendering every record at load costs a fraction of a second.
const MIX_RATE := 22050
const CHANNELS := 1

## The wrap crossfade a looping bed gets, so the end of the buffer meets its own
## beginning without a click. Beds are the only looping records.
const LOOP_WRAP_MS := 200

## The LCG the noise comes from. Numerical Recipes' constants; the point is that
## it is small, stated here, and identical on every platform.
const NOISE_MULTIPLIER := 1664525
const NOISE_INCREMENT := 1013904223
const NOISE_MODULUS := 4294967296


## Renders one record's voice. Returns null if the voice is unrenderable, which
## the validator already refuses at authoring time.
static func render(voice: Dictionary) -> AudioStreamWAV:
	var duration_ms := int(voice.get("durationMs", 0))
	if duration_ms <= 0:
		return null
	var looping: bool = bool(voice.get("loop", false))
	var wrap_samples := 0
	if looping:
		wrap_samples = int(round(float(LOOP_WRAP_MS) * MIX_RATE / 1000.0))
	var sample_count := int(round(float(duration_ms) * MIX_RATE / 1000.0))
	if sample_count <= 0:
		return null

	var waveform := str(voice.get("waveform", "sine"))
	var pitch_hz := float(voice.get("pitchHz", 440.0))
	var glide_hz := float(voice.get("pitchGlideHz", 0.0))
	var noise_mix := clampf(float(voice.get("noiseMix", 0.0)), 0.0, 1.0)
	var lowpass_hz := maxf(float(voice.get("lowpassHz", 8000.0)), 1.0)
	var tremolo_hz := float(voice.get("tremoloHz", 0.0))
	var tremolo_depth := clampf(float(voice.get("tremoloDepth", 0.0)), 0.0, 1.0)
	var state := int(voice.get("seed", 1)) % NOISE_MODULUS

	# One-pole low-pass, the coefficient stated from the cutoff so a record's
	# lowpassHz means the same thing at any mix rate.
	var rc := 1.0 / (TAU * lowpass_hz)
	var dt := 1.0 / float(MIX_RATE)
	var alpha := dt / (rc + dt)
	var filtered := 0.0
	var phase := 0.0

	var total := sample_count + wrap_samples
	var raw := PackedFloat32Array()
	raw.resize(total)
	for index in total:
		var progress := float(index) / float(sample_count)
		var frequency := maxf(pitch_hz + glide_hz * minf(progress, 1.0), 1.0)
		phase = fmod(phase + frequency * dt, 1.0)
		var tone := _wave(waveform, phase)
		state = (NOISE_MULTIPLIER * state + NOISE_INCREMENT) % NOISE_MODULUS
		var noise := float(state) / float(NOISE_MODULUS) * 2.0 - 1.0
		var mixed: float = lerpf(tone, noise, noise_mix)
		filtered += alpha * (mixed - filtered)
		var amplitude := _envelope(voice, index, sample_count, looping)
		if tremolo_hz > 0.0 and tremolo_depth > 0.0:
			var seconds := float(index) * dt
			amplitude *= 1.0 - tremolo_depth * (0.5 - 0.5 * cos(TAU * tremolo_hz * seconds))
		raw[index] = filtered * amplitude

	if looping and wrap_samples > 0:
		# The tail is folded back over the head with an equal-power pair, so the
		# loop point is inaudible instead of a click every four seconds.
		for index in wrap_samples:
			var mix := float(index) / float(wrap_samples)
			var head := raw[index]
			var tail := raw[sample_count + index]
			raw[index] = head * sqrt(mix) + tail * sqrt(1.0 - mix)

	var data := PackedByteArray()
	data.resize(sample_count * 2)
	for index in sample_count:
		var value := int(round(clampf(raw[index], -1.0, 1.0) * 32767.0))
		data.encode_s16(index * 2, value)

	var stream := AudioStreamWAV.new()
	stream.format = AudioStreamWAV.FORMAT_16_BITS
	stream.mix_rate = MIX_RATE
	stream.stereo = CHANNELS > 1
	stream.data = data
	if looping:
		stream.loop_mode = AudioStreamWAV.LOOP_FORWARD
		stream.loop_begin = 0
		stream.loop_end = sample_count
	return stream


static func _wave(waveform: String, phase: float) -> float:
	match waveform:
		"sine":
			return sin(TAU * phase)
		"triangle":
			return 4.0 * absf(phase - 0.5) - 1.0
		"square":
			return 1.0 if phase < 0.5 else -1.0
		"saw":
			return 2.0 * phase - 1.0
		"noise":
			return 0.0
	return 0.0


## Attack, decay, sustain, release across the record's own duration. A looping
## bed holds at full level throughout: its edges are joined by the wrap
## crossfade instead, because an envelope inside a loop would breathe once per
## wrap and give the island a heartbeat nobody authored.
static func _envelope(voice: Dictionary, index: int, sample_count: int, looping: bool) -> float:
	if looping:
		return 1.0
	var attack := _samples(voice.get("attackMs", 0))
	var decay := _samples(voice.get("decayMs", 0))
	var release := _samples(voice.get("releaseMs", 0))
	var sustain := clampf(float(voice.get("sustainLevel", 0.0)), 0.0, 1.0)
	var release_start: int = maxi(sample_count - release, attack + decay)
	if index < attack:
		return float(index) / float(maxi(attack, 1))
	if index < attack + decay:
		return lerpf(1.0, sustain, float(index - attack) / float(maxi(decay, 1)))
	if index < release_start:
		return sustain
	var remaining := float(sample_count - index)
	return sustain * clampf(remaining / float(maxi(release, 1)), 0.0, 1.0)


static func _samples(milliseconds: Variant) -> int:
	return int(round(float(milliseconds) * MIX_RATE / 1000.0))
