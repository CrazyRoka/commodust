# ADSR Envelopes

The SID's envelope generator produces a four-phase volume contour for each
voice: Attack, Decay, Sustain, and Release. The envelope is controlled by
two write-only registers per voice.

## Register Layout

| Offset | Register | Nybble | Function |
|---|---|---|---|
| +5 | ATDCY | High (bits 4–7) | Attack rate (0–15) |
| +5 | ATDCY | Low (bits 0–3) | Decay rate (0–15) |
| +6 | SUREL | High (bits 4–7) | Sustain level (0–15) |
| +6 | SUREL | Low (bits 0–3) | Release rate (0–15) |

Register value formula: `ATDCY = (ATTACK × 16) + DECAY`
`SUREL = (SUSTAIN × 16) + RELEASE`

## Envelope Phases

```
Volume
  ^
  |     /\  ← Attack phase (rise to peak)
  |    /  \ ← Decay phase (fall to sustain level)
  |   /    ──────── ← Sustain phase (held until gate cleared)
  |  /             \ ← Release phase (fall to zero)
  | /               \
  |/_________________\________→ Time
  |   Gate=1                    Gate=0
```

### Phase Transitions

1. **Attack:** Begins when gate bit is set to 1. Volume rises from 0 to peak
   (maximum amplitude) over the attack duration.
2. **Decay:** Begins when attack completes. Volume falls from peak to the
   sustain level over the decay duration.
3. **Sustain:** Volume holds at the sustain level as long as the gate bit
   remains 1.
4. **Release:** Begins when gate bit is cleared to 0 (while waveform bit
   remains set). Volume falls from the current sustain level to near-zero
   over the release duration.

**Residual volume note:** The release cycle may not reach exactly zero.
Software that requires absolute silence should explicitly turn off the
waveform, set frequency to 0, or set volume to 0 after the release phase.

## Attack Rate Table

| Value | Duration |
|---|---|
| 0 | 2 ms |
| 1 | 8 ms |
| 2 | 16 ms |
| 3 | 24 ms |
| 4 | 38 ms |
| 5 | 56 ms |
| 6 | 68 ms |
| 7 | 80 ms |
| 8 | 100 ms |
| 9 | 250 ms |
| 10 | 500 ms |
| 11 | 800 ms |
| 12 | 1 s |
| 13 | 3 s |
| 14 | 5 s |
| 15 | 8 s |

## Decay Rate Table

| Value | Duration |
|---|---|
| 0 | 6 ms |
| 1 | 24 ms |
| 2 | 48 ms |
| 3 | 72 ms |
| 4 | 114 ms |
| 5 | 168 ms |
| 6 | 204 ms |
| 7 | 240 ms |
| 8 | 300 ms |
| 9 | 750 ms |
| 10 | 1.5 s |
| 11 | 2.4 s |
| 12 | 3 s |
| 13 | 9 s |
| 14 | 15 s |
| 15 | 24 s |

## Release Rate Table

| Value | Duration |
|---|---|
| 0 | 6 ms |
| 1 | 24 ms |
| 2 | 48 ms |
| 3 | 72 ms |
| 4 | 114 ms |
| 5 | 168 ms |
| 6 | 204 ms |
| 7 | 240 ms |
| 8 | 300 ms |
| 9 | 750 ms |
| 10 | 1.5 s |
| 11 | 2.4 s |
| 12 | 3 s |
| 13 | 9 s |
| 14 | 15 s |
| 15 | 24 s |

Decay and release share the same timing table.

## Sustain Level

| Value | Volume |
|---|---|
| 0 | Silence (no sustain) |
| 1–14 | Proportional intermediate level |
| 15 | Peak volume (equal to attack peak) |

The sustain level is a fraction of the peak amplitude, not an absolute value.
Sustain level 15 means the note stays at full volume; sustain level 0 means
the note is silent during the sustain phase.

## Envelope Readback (Voice 3 Only)

The output of Voice 3's envelope generator can be read from $D41C (ENV3).
This returns the current 8-bit envelope amplitude value. This is commonly
used for:
- Modulating other voices' frequency, pulse width, or filter cutoff
- Creating delayed envelope effects
- LFO generation when combined with a slow-attacking oscillator

Voice 3's audio output can be disconnected (bit 7 of $D418 = 1) while still
allowing envelope readback, enabling silent modulation sources.

## Emulation Notes

### Clock-Driven Envelope

The envelope generator advances based on the SID's internal clock, not CPU
write timing. The emulator must tick the envelope generator at the SID clock
rate (same as the CPU clock: ~1.02273 MHz NTSC, ~0.98525 MHz PAL).

### Non-Linear Timing

The ADSR rate tables are exponential, not linear. Each step does not represent
an equal time increment. The emulator must use the exact duration table above
rather than interpolating linearly between values.

### Gate Transition Latency

When the gate bit is set or cleared, the envelope phase transition does not
occur instantaneously — it happens on the next envelope generator clock tick.
For cycle-accurate emulation, this means the envelope may continue in its
current phase for up to one clock cycle after the gate bit write.

### Attack/Release Re-triggering

If the gate bit is set again during the release phase, the attack phase
restarts from the current volume level, not from zero. This is the standard
behavior for re-triggering a note without a gap. However, if the gate is
cleared and re-set very rapidly, some SID revisions exhibit anomalous
behavior where the envelope jumps to zero before re-attacking.

### Conceptual Considerations for Emulation

The envelope generator should be modeled as a state machine with four states
(Attack, Decay, Sustain, Release) and a continuous amplitude counter. The
state transitions are driven by:
- Gate bit transitions (set → Attack; clear → Release)
- Amplitude thresholds (peak reached → Decay; sustain level reached → Sustain;
  zero reached → idle)

In a Rust emulator, this suggests an enum-based state machine where each
variant carries the phase-specific rate constant and the direction (ascending
or descending). The envelope amplitude can be stored as a u16 for internal
precision and downshifted to u8 for output, matching the SID's internal
resolution.