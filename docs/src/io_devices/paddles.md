# Game Paddles

The C64 supports up to 4 paddle controllers (2 per joystick port). Paddles
use variable resistors (potentiometers) read by the SID's analog-to-digital
converters.

## Paddle Connections

| Paddle | Port | Position Register | Fire Button |
|---|---|---|---|
| Paddle 1 | Controller Port 1 | POTX ($D419) | CIA #1 Port B bit 2 ($DC01) |
| Paddle 2 | Controller Port 1 | POTY ($D41A) | CIA #1 Port B bit 3 ($DC01) |
| Paddle 3 | Controller Port 2 | POTX ($D419) | CIA #1 Port A bit 2 ($DC00) |
| Paddle 4 | Controller Port 2 | POTY ($D41A) | CIA #1 Port A bit 3 ($DC00) |

## Port Selection

Only two paddle positions can be read at a time (POTX and POTY). The
active pair is selected via CIA #1 Port A bits 6–7:

| Port A Bit 7 | Port A Bit 6 | Paddles Read |
|---|---|---|
| 1 | 0 | Port 1 (paddles 1 & 2) |
| 0 | 1 | Port 2 (paddles 3 & 4) |

**Conflict with keyboard:** Port A is also used for keyboard column
selection. The keyscan IRQ writes to Port A 60 times per second, so
reliable paddle reading requires disabling the IRQ.

## Reading Paddle Position

1. Disable keyscan IRQ: `POKE 56333, 127`
2. Select paddle port: Write appropriate value to Port A
   - Port 1: `POKE 56320, (PEEK(56320) AND $3F) OR $80`
   - Port 2: `POKE 56320, (PEEK(56320) AND $3F) OR $40`
3. Wait ~512 cycles for A/D conversion
4. Read positions:
   - Paddle X (1 or 3): `X = PEEK(54297)`
   - Paddle Y (2 or 4): `Y = PEEK(54298)`
5. Re-enable keyscan: `POKE 56333, 129`

## Position Values

The SID's paddle registers return values 0–255 based on the potentiometer
resistance:
- 0: Minimum resistance (knob fully counter-clockwise)
- 255: Maximum resistance (knob fully clockwise)

## Fire Button Reading

| Button | Location | Bit | Test |
|---|---|---|---|
| Paddle 1 fire | $DC01 (Port B) | 2 | 0 = pressed |
| Paddle 2 fire | $DC01 (Port B) | 3 | 0 = pressed |
| Paddle 3 fire | $DC00 (Port A) | 2 | 0 = pressed |
| Paddle 4 fire | $DC00 (Port A) | 3 | 0 = pressed |

Fire buttons are active-low and share lines with joystick left/right
directions on the same port.

## Emulation Notes

### A/D Conversion Timing

The SID's paddle inputs use a simple RC charging circuit. The potentiometer
resistance controls the charge time, which is measured by counting clock
cycles. The conversion takes approximately 512 clock cycles. The emulator
should update POTX/POTY values at this interval, not instantaneously on
register read.

### Port Selection Modeling

The emulator must track the CIA #1 Port A bits 6–7 state and return the
appropriate paddle values from POTX/POTY. Since the keyscan routine
constantly changes Port A, the paddle values may be unstable if read
without disabling the IRQ.

### Conceptual Considerations for Emulation

In a Rust emulator, paddles can be modeled as a pair of u8 values per port
(4 values total). The currently selected port is determined by the
CIA #1 Port A output register bits 6–7. The POTX/POTY read returns the
value for the selected port. If no paddle is connected, the value should
read as 255 (maximum resistance / open circuit) or 0 (shorted),
depending on the emulated hardware state.