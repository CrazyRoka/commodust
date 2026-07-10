# Miscellaneous Registers

The SID's last four registers ($D419–$D41C) are the only **readable** SID
registers. They provide paddle position inputs, oscillator 3 readback, and
envelope 3 readback.

## Register Map

| Address | Name | R/W | Description |
|---|---|---|---|
| $D419 | POTX | R | Paddle 1 (or 3) position |
| $D41A | POTY | R | Paddle 2 (or 4) position |
| $D41B | RANDOM | R | Oscillator 3 output / random number |
| $D41C | ENV3 | R | Envelope generator 3 output |

## $D419 — POTX: Paddle X Position

Returns an 8-bit value (0–255) representing the resistance of the paddle
potentiometer connected to joystick Controller Port 1 or 2 (selected via
CIA #1 Port A bits 6–7).

- Value 0: Minimum resistance (knob fully counter-clockwise)
- Value 255: Maximum resistance (knob fully clockwise)

The SID contains an analog-to-digital converter that measures the RC charge
time of the paddle potentiometer. The conversion takes approximately 512
cycles. During this time, the previous value is held. Software should wait
at least 512 cycles between changing the paddle port select and reading the
registers.

## $D41A — POTY: Paddle Y Position

Identical to POTX but reads the second paddle on the selected port. Together,
POTX and POTY provide positions for two paddles on the currently selected
controller port.

### Paddle Port Selection

The SID's internal A/D converters read only one pair of paddle inputs (`POTX` and `POTY`) at any given moment. Multiplexing between Controller Port 1 and Controller Port 2 is controlled by writing a bit-pattern to Bits 6 and 7 of CIA #1 Data Port A (`$DC00`):

| CIA #1 Port A Bit 7 | Bit 6 | Bit-Pair Value | Selected Controller Port | Registers Read |
|---|---|---|---|---|
| **1** | **0** | `$40` (64) | Controller Port 1 (Paddles 1 & 2) | `POTX` = Paddle 1, `POTY` = Paddle 2 |
| **0** | **1** | `$80` (128) | Controller Port 2 (Paddles 3 & 4) | `POTX` = Paddle 3, `POTY` = Paddle 4 |

**Conflict note:** Because CIA #1 Port A is also the output register for scanning keyboard columns, the system IRQ handler (`$EA31`) overwrites the paddle selection bits 60 times per second. To avoid values fluctuating, software must temporarily disable the IRQ (`$DC0D`), configure the multiplexer bit-pair at `$DC00`, wait approximately 512 clock cycles for the charge/discharge conversion cycle to stabilize, read the conversion values from `$D419` and `$D41A`, and then restore the IRQ state.

### Paddle Fire Buttons

Paddle fire buttons are **not** read through the SID. They are read via
CIA #1 data ports:

| Button | Port | Bit | Test |
|---|---|---|---|
| Paddle 1 fire | CIA #1 Port B ($DC01) | 2 | 0 = pressed |
| Paddle 2 fire | CIA #1 Port B ($DC01) | 3 | 0 = pressed |
| Paddle 3 fire | CIA #1 Port A ($DC00) | 2 | 0 = pressed |
| Paddle 4 fire | CIA #1 Port A ($DC00) | 3 | 0 = pressed |

## $D41B — RANDOM: Oscillator 3 Output

Returns the upper 8 bits of Oscillator 3's waveform output. The values
returned depend on the waveform selected for Voice 3:

| Waveform | Output Pattern |
|---|---|
| Sawtooth | Ramps 0→255, then resets to 0 |
| Triangle | Ramps 0→255→0 (triangle wave) |
| Pulse | Either 0 or 255 (depending on pulse width) |
| Noise | Pseudo-random values 0–255 |

The rate of change is determined by Voice 3's frequency register. This
register is commonly used for:
- **Random number generation:** Select noise waveform; read $D41B for each
  random byte.
- **Frequency modulation:** Add $D41B to another voice's frequency registers
  for vibrato or pitch-bend effects.
- **Pulse width modulation:** Combine $D41B with pulse width registers for
  timbre effects.
- **Filter modulation:** Add $D41B to the filter cutoff registers for
  sweeping filter effects.

**Important:** Oscillator 3 output is available regardless of the gate bit
state. The oscillator runs continuously. However, if the test bit (bit 3 of
$D412) is set, the oscillator is halted and $D41B reads 0.

To use Voice 3 for modulation without audible output, set bit 7 of $D418
(disconnect Voice 3 audio). The oscillator and envelope continue to run and
remain readable.

## $D41C — ENV3: Envelope Generator 3 Output

Returns the current 8-bit amplitude output of Voice 3's envelope generator.
This value follows the ADSR envelope curve in real-time.

Unlike the oscillator readback ($D41B), the envelope output **does** depend
on the gate bit. The envelope generator must be gated (bit 0 of $D412 set)
to produce non-zero output. The envelope follows the standard attack→decay→
sustain→release cycle.

Uses include:
- Adding $D41C to a frequency register for envelope-driven pitch modulation
- Adding $D41C to the filter cutoff for envelope-driven filter sweeps
- Adding $D41C to a pulse width register for envelope-driven timbre changes

## Mirroring

The SID uses only 5 address bits (A0–A4), so only 32 addresses are decoded.
The 1K region $D400–$D7FF contains 32 mirrors of the 32-byte register set.
Unconnected addresses $D41D–$D41F always read as $FF and ignore writes.

## Emulation Notes

### Paddle Input Emulation

The paddle inputs require modeling the SID's internal analog-to-digital
conversion. Each paddle potentiometer charges a capacitor through a variable
resistance. The SID counts clock cycles until the capacitor reaches a
threshold. The emulator should:
1. Track the paddle position (0–255) from the host input.
2. Update POTX/POTY registers at the correct interval (~512 cycles).
3. Model the port selection logic via CIA #1 Port A bits 6–7.

### Oscillator 3 Readback

$D41B should return the upper 8 bits of the current oscillator 3 phase
accumulator, shaped by the selected waveform. For noise, the emulator must
maintain the SID's internal LFSR (linear feedback shift register) state,
as the noise pattern is deterministic for a given seed.

### Envelope 3 Readback

$D41C should return the current envelope generator 3 amplitude (0–255). This
requires the envelope generator to be modeled as a continuously running state
machine, not just computed on demand. The readback value changes every cycle
as the envelope progresses through its phases.

### Write-Only vs Read-Only

The emulator must enforce that $D400–$D418 are write-only (reads return $FF
or undefined) and $D419–$D41C are read-only (writes have no effect). This
matters for software that accidentally reads from write-only registers
expecting to get back the last value written.