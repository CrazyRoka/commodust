# Filters

The SID provides a programmable multi-mode audio filter that can process any
combination of the three voices plus an external audio input. The filter is
the most analog portion of the SID and is the most challenging part to emulate
accurately.

## Filter Register Map

| Address | Name | R/W | Description |
|---|---|---|---|
| $D415 | CUTLO | W | Filter cutoff frequency (low 3 bits) |
| $D416 | CUTHI | W | Filter cutoff frequency (high byte) |
| $D417 | RESON | W | Resonance + voice routing |
| $D418 | SIGVOL | W | Volume + filter mode select + voice 3 off |

All filter registers are **write-only**.

## Filter Cutoff Frequency (11-bit)

| Address | Bits | Description |
|---|---|---|
| $D415 | 0–2 | Low 3 bits of cutoff frequency |
| $D415 | 3–7 | Unused (no effect) |
| $D416 | 0–7 | High 8 bits of cutoff frequency |

The 11-bit cutoff value combines as:
`CUTOFF_VALUE = (CUTLO AND $07) + (CUTHI × 8)`

### Cutoff Frequency Calculation

```
CUTOFF_FREQ = (CUTOFF_VALUE × 5.8) + 30 Hz
```

The range of 0–2047 maps to cutoff frequencies from approximately 30 Hz to
12,000 Hz.

**Important:** On the original 6581 SID, the cutoff frequency response is
exponential and highly non-linear. The formula above is an approximation.
The filter does not produce a perfectly linear relationship between register
value and cutoff frequency. Later SID revisions (8580) have a more linear
cutoff response, leading to audible differences in filtered sound between
chip revisions.

## Resonance and Voice Routing ($D417)

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Filter Voice 1 (1 = route through filter) |
| 1 | $02 | Filter Voice 2 |
| 2 | $04 | Filter Voice 3 |
| 3 | $08 | Filter external input (pin 5 of A/V port) |
| 4 | $10 | Resonance bit 0 |
| 5 | $20 | Resonance bit 1 |
| 6 | $40 | Resonance bit 2 |
| 7 | $80 | Resonance bit 3 |

### Voice Routing

Bits 0–3 select which voices are routed through the filter. A voice with its
bit set to 1 is processed through the filter; a voice with its bit cleared
passes directly to the audio output unfiltered.

### Resonance (4-bit)

Bits 4–7 select resonance from 0 (no resonance) to 15 (maximum resonance).
Resonance peaks the volume of frequencies nearest the cutoff frequency,
creating a sharper filtering effect.

`RESONANCE_VALUE = (RESON SHR 4)` gives a value from 0 to 15.

## Volume and Filter Mode ($D418)

**Default after Kernal init:** $0F (volume 15, no filters, voice 3 audible —
though the Kernal actually sets this to $00 or leaves it uninitialized;
software typically sets it explicitly).

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Volume bit 0 |
| 1 | $02 | Volume bit 1 |
| 2 | $04 | Volume bit 2 |
| 3 | $08 | Volume bit 3 |
| 4 | $10 | Low-pass filter (1 = enabled) |
| 5 | $20 | Band-pass filter (1 = enabled) |
| 6 | $40 | High-pass filter (1 = enabled) |
| 7 | $80 | Disconnect Voice 3 audio output (1 = off) |

### Volume (4-bit)

Bits 0–3 set the master output volume from 0 (silence) to 15 (maximum). This
applies to all voices and the external input. Some non-zero volume must be
set before any sound is audible.

### Filter Mode Selection

Bits 4–6 enable individual filter types. Multiple filter types can be enabled
simultaneously:

| Combination | Effect |
|---|---|
| Bit 4 only | Low-pass filter |
| Bit 5 only | Band-pass filter |
| Bit 6 only | High-pass filter |
| Bits 4+6 | Notch (band-reject) filter |
| Bits 4+5+6 | All three simultaneously (unusual) |

For any filter to have an effect, at least one voice must be routed through
it via $D417, and at least one filter mode bit must be set.

### Voice 3 Disconnect (Bit 7)

When bit 7 is set to 1, Voice 3's audio output is disconnected from the
mixer. This allows Voice 3 to be used as a modulation source (via $D41B
oscillator readback and $D41C envelope readback) without producing audible
output. The oscillator and envelope generator continue to run regardless of
this bit.

## Filter Characteristics

### Attenuation Slopes

| Filter Type | Slope |
|---|---|
| Low-pass | −12 dB per octave above cutoff |
| High-pass | −12 dB per octave below cutoff |
| Band-pass | −6 dB per octave on each side of cutoff |
| Notch (LP+HP) | Attenuates frequencies near cutoff |

### 6581 vs 8580 Differences

The original 6581 SID has a filter with significant analog non-linearities:
- The cutoff frequency is not linearly proportional to the register value.
- The resonance implementation distorts the waveform at high resonance
  settings.
- The external input has a DC offset that affects filtering.
- Different 6581 chips have measurably different filter characteristics due
  to manufacturing variations.

The later 8580 SID has a more linear and cleaner filter response, but sounds
noticeably different. Many C64 music compositions were tuned for the 6581's
particular filter behavior and sound "wrong" on an 8580.

## Emulation Notes

### Filter Implementation Challenge

The filter is widely considered the hardest SID component to emulate
accurately. A simple digital filter (biquad, etc.) does not capture the
analog non-linearities of the 6581. Several approaches exist:

1. **Simple biquad:** Fast but inaccurate. Acceptable for basic sound effects.
2. **Parameter-scaled biquad:** Adjusts coefficients based on measured 6581
   response curves. Better but still imperfect.
3. **Circuit-level simulation:** Models the actual analog transistor/capacitor
   network. Most accurate but computationally expensive.

### Conceptual Considerations for Emulation

In a Rust emulator, the filter state (previous input/output samples, internal
state variables) should be owned by the SID struct and updated per audio
sample. The filter should be applied after mixing the individual voice
outputs but before the final volume scaling.

For save-state purposes, the filter's internal delay lines and state must be
serialized, as they affect the output even after a register change.

### Register Write Transparency

All filter registers are write-only. The emulator must maintain shadow copies
for debugging and save-state support. Writing to CUTLO bits 3–7 has no effect
and these bits should read back as undefined (typically $FF on real hardware).