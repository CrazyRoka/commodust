# Voice Registers

The SID's three voices are controlled by identical 7-byte register blocks.
Voice 1 occupies $D400–$D406, Voice 2 occupies $D407–$D40D, and Voice 3
occupies $D40E–$D414. All voice registers are **write-only**; reading them
returns undefined values (typically $FF).

## Register Layout Per Voice

| Offset | Voice 1 | Voice 2 | Voice 3 | Name | Description |
|---|---|---|---|---|---|
| +0 | $D400 | $D407 | $D40E | FRELO | Frequency low byte |
| +1 | $D401 | $D408 | $D40F | FREHI | Frequency high byte |
| +2 | $D402 | $D409 | $D410 | PWLO | Pulse width low byte |
| +3 | $D403 | $D40A | $D411 | PWHI | Pulse width high nybble |
| +4 | $D404 | $D40B | $D412 | VCREG | Control register |
| +5 | $D405 | $D40C | $D413 | ATDCY | Attack/Decay |
| +6 | $D406 | $D40D | $D414 | SUREL | Sustain/Release |

## Frequency Registers (16-bit)

| Offset | Name | Bits | Description |
|---|---|---|---|
| +0 | FRELO | 0–7 | Low byte of 16-bit frequency |
| +1 | FREHI | 0–7 | High byte of 16-bit frequency |

The 16-bit value combines as: `FREQUENCY_VALUE = FRELO + (FREHI × 256)`.

### Frequency-to-Pitch Conversion

```
OUTPUT_FREQ = (REGISTER_VALUE × CLOCK) / 16777216
```

| System | CLOCK | Resolution |
|---|---|---|
| NTSC | 1,022,730 Hz | REGISTER_VALUE × 0.060959458 Hz |
| PAL | 985,250 Hz | REGISTER_VALUE × 0.058728929 Hz |

The 16-bit range provides over 8 octaves, from near-DC to approximately 4 kHz.

**Default at power-on:** Undefined (SID registers are not initialized by the
Kernal). Software must set frequency before gating the oscillator.

## Pulse Width Registers (12-bit)

| Offset | Name | Bits | Description |
|---|---|---|---|
| +2 | PWLO | 0–7 | Low 8 bits of pulse width |
| +3 | PWHI | 0–3 | High 4 bits of pulse width (bits 8–11) |

Bits 4–7 of PWHI are unused and have no effect.

The 12-bit value combines as: `PULSE_WIDTH = PWLO + (PWHI AND $0F) × 256`.

### Duty Cycle Calculation

```
DUTY_CYCLE = (PULSE_WIDTH / 40.95) %
```

The range 0–4095 covers duty cycles from 0% to 100% in 4096 steps. Changing
the pulse width dramatically alters the harmonic content of the pulse waveform
output.

**Default:** Undefined. Must be set before using the pulse waveform.

## Control Register (VCREG)

**Default:** Undefined (not initialized by Kernal).

| Bit | Mask | Name | Function |
|---|---|---|---|
| 0 | $01 | Gate | 1 = Start attack/decay/sustain; 0 = Start release |
| 1 | $02 | Sync | 1 = Synchronize oscillator with oscillator above it |
| 2 | $04 | Ring Mod | 1 = Ring modulate with oscillator above it |
| 3 | $08 | Test | 1 = Disable oscillator (reset to zero) |
| 4 | $10 | Triangle | 1 = Select triangle waveform |
| 5 | $20 | Sawtooth | 1 = Select sawtooth waveform |
| 6 | $40 | Pulse | 1 = Select pulse waveform |
| 7 | $80 | Noise | 1 = Select noise waveform |

### Bit 0 — Gate Bit

Setting bit 0 to 1 while a waveform bit is set starts the attack/decay/sustain
cycle. Setting bit 0 back to 0 (while **keeping the same waveform bit set**)
starts the release cycle.

**Critical:** If the waveform bits are also cleared when clearing the gate bit,
the sound stops immediately without a release phase. The waveform selection
bit(s) must remain set for the release cycle to execute.

### Bit 1 — Oscillator Synchronization

When set, the oscillator is synchronized with the oscillator one number higher
(Voice 1 syncs to Voice 3, Voice 2 syncs to Voice 1, Voice 3 syncs to Voice 2).
The sync source oscillator must be set to a non-zero frequency, but its other
parameters (waveform, volume, gate) do not affect the sync result. Sync creates
complex harmonic structures.

### Bit 2 — Ring Modulation

When set, the triangle waveform output of the current voice is replaced with a
ring-modulated combination of the current oscillator and the oscillator one
number higher. This produces non-harmonic overtone structures useful for bell
or gong effects.

### Bit 3 — Test Bit

When set to 1, the oscillator is disabled and its output is forced to zero.
This is useful for software-controlled waveform generation and speech
synthesis, as it allows direct phase reset.

### Bits 4–7 — Waveform Selection

At least one waveform bit must be set to produce sound. Setting multiple
waveform bits simultaneously results in a logical AND of the waveforms.

**Warning:** Combining the noise waveform with another waveform is not
recommended and produces unpredictable results on real hardware.

## Voice-to-Voice Sync/Ring Modulation Pairing

| Voice | Sync source | Ring mod source |
|---|---|---|
| Voice 1 ($D404) | Oscillator 3 | Oscillator 3 |
| Voice 2 ($D40B) | Oscillator 1 | Oscillator 1 |
| Voice 3 ($D412) | Oscillator 2 | Oscillator 2 |

Note the "oscillator above" pairing: Voice 1 references Oscillator 3 (wrapping
around), Voice 2 references Oscillator 1, and Voice 3 references Oscillator 2.

## Emulation Notes

### Write-Only Shadow State

All voice registers are write-only. The emulator must maintain internal shadow
copies of every register for:
- Save-state serialization
- Debugging interfaces
- Determining current oscillator/envelope state

### Oscillator Independence

Each oscillator runs continuously at its programmed frequency, independent of
the gate bit. The gate bit only controls the envelope generator's phase. The
oscillator waveform output is always available for readback via $D41B
(Oscillator 3) regardless of the gate state.

### Register Write Ordering

Software commonly writes frequency, pulse width, and ADSR registers before
setting the gate bit. The emulator must process each write immediately, as
some software relies on intermediate states (e.g., setting test bit, then
clearing it to reset oscillator phase before gating).