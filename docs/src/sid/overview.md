# SID Overview

The SID (Sound Interface Device, MOS 6581) is a custom 3-voice
synthesizer chip mapped at $D400–$D41C (29 registers). It provides:

- 3 independent oscillators (voices)
- 4 waveforms per voice (triangle, sawtooth, pulse, noise)
- 16-bit frequency resolution
- ADSR envelope generation per voice
- Oscillator sync and ring modulation
- Programmable multi-mode filter (low-pass, band-pass, high-pass)
- 3-channel volume control
- Oscillator 3 and envelope 3 readback for modulation
- 2 analog paddle inputs

## Register Map

| Address | Register | R/W | Description |
|---|---|---|---|
| $D400 | FRELO1 | W | Voice 1 frequency low byte |
| $D401 | FREHI1 | W | Voice 1 frequency high byte |
| $D402 | PWLO1 | W | Voice 1 pulse width low byte |
| $D403 | PWHI1 | W | Voice 1 pulse width high nybble |
| $D404 | VCREG1 | W | Voice 1 control register |
| $D405 | ATDCY1 | W | Voice 1 attack/decay |
| $D406 | SUREL1 | W | Voice 1 sustain/release |
| $D407–$D40D | — | W | Voice 2 registers (identical layout) |
| $D40E–$D414 | — | W | Voice 3 registers (identical layout) |
| $D415 | CUTLO | W | Filter cutoff low 3 bits |
| $D416 | CUTHI | W | Filter cutoff high byte |
| $D417 | RESON | W | Filter resonance + voice routing |
| $D418 | SIGVOL | W | Volume + filter select + voice 3 off |
| $D419 | POTX | R | Paddle 1 (or 3) position |
| $D41A | POTY | R | Paddle 2 (or 4) position |
| $D41B | RANDOM | R | Oscillator 3 output / random number |
| $D41C | ENV3 | R | Envelope generator 3 output |

**Critical:** Registers $D400–$D418 are **write-only**. Reading them
returns undefined values (typically $FF). Only $D419–$D41C are
readable.

## Frequency Calculation

The relationship between the 16-bit frequency register value and the
output frequency is:

```
FREQUENCY = (REGISTER_VALUE × CLOCK) / 16777216
```

Where CLOCK = 1,022,730 Hz (NTSC) or 985,250 Hz (PAL).

For NTSC: `FREQUENCY = REGISTER_VALUE × 0.060959458 Hz`

The 16-bit range provides over 8 octaves of pitch resolution, from
near-DC to approximately 4 kHz.

## Sound Generation Sequence

To produce a sound on a SID voice:

1. Set the **volume** ($D418 bits 0–3) to a nonzero value.
2. Set the **frequency** ($D400–$D401 for voice 1) to the desired pitch.
3. Set the **pulse width** ($D402–$D403) if using the pulse waveform.
4. Set the **ADSR envelope** ($D405 attack/decay, $D406 sustain/release).
5. **Gate** the oscillator by writing the waveform bit AND bit 0 (gate)
   to the control register ($D404). This starts the attack-decay-sustain
   cycle.
6. To end the note, write the same waveform bit with bit 0 cleared
   (gate = 0). This starts the release cycle.

**Important:** When clearing the gate bit, the waveform selection bit(s)
must remain set. If the waveform bits are also cleared, the sound stops
immediately without a release phase.

## Mirroring

The SID uses only 5 address bits (A0–A4), so only 32 addresses are
decoded. The 1K region $D400–$D7FF contains 32 mirrors of the 32-byte
register set. Always use the base address $D400 for clarity.

## Emulation Notes

- All voice registers are write-only. An emulator should maintain
  internal shadow state for each register to support debugging and
  save states.
- The envelope generator runs independently of the gate bit timing —
  it advances based on the SID's internal clock, not CPU writes.
- Oscillator 3 output ($D41B) and envelope 3 output ($D41C) can be read
  even when voice 3's audio output is disconnected (bit 7 of $D418 = 1).
  This is commonly used for modulation and random number generation.
- The filter section is the most challenging part of SID emulation due
  to analog nonlinearities. The cutoff frequency response is exponential,
  not linear.