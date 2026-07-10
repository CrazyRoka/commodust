# Joystick

The C64 has two joystick controller ports. Joysticks are read through
CIA #1 data ports, sharing lines with the keyboard matrix.

## Controller Port Assignments

| Port | Read From | CIA Register |
|---|---|---|
| Controller Port 1 | CIA #1 Port B | $DC01 |
| Controller Port 2 | CIA #1 Port A | $DC00 |

**Note the swapped assignment:** Port 1 is read from Port B, and Port 2
from Port A. This is counter-intuitive but matches the hardware wiring.

## Joystick Switch Layout

Each joystick has 5 switches: up, down, left, right, and fire. The switches
are active-low (0 = pressed, 1 = not pressed).

### Port 1 (read via $DC01, Port B)

| Bit | Direction | Mask |
|---|---|---|
| 0 | Up | $01 |
| 1 | Down | $02 |
| 2 | Left | $04 |
| 3 | Right | $08 |
| 4 | Fire | $10 |

### Port 2 (read via $DC00, Port A)

| Bit | Direction | Mask |
|---|---|---|
| 0 | Up | $01 |
| 1 | Down | $02 |
| 2 | Left | $04 |
| 3 | Right | $08 |
| 4 | Fire | $10 |

## Reading the Joystick

### Joystick 1 (Port 1)

```
J1 = NOT PEEK($DC01) AND $1F
```

### Joystick 2 (Port 2)

```
J2 = NOT PEEK($DC00) AND $1F
```

The `NOT` inverts the active-low logic. The `AND $1F` masks out bits 5–7
which are used for other functions.

## Direction Values

| Value | Direction |
|---|---|
| 0 | No direction pressed |
| 1 | Up |
| 2 | Down |
| 4 | Left |
| 5 | Up-Left |
| 6 | Down-Left |
| 8 | Right |
| 9 | Up-Right |
| 10 | Down-Right |
| 16 | Fire (can be OR'd with direction) |

Values 3 and 7 are impossible (up+down simultaneously).

## Keyboard Conflict

### Joystick 1 and Keyboard

Joystick 1 shares lines with the keyboard matrix (both use Port B). Moving
joystick 1 registers as keypresses:

| Joystick 1 Direction | Key Registered |
|---|---|
| Up | 1 (row 0) |
| Down | 2 (row 1) |
| Left | CTRL (row 6) |
| Right | 2 (row 7) |
| Fire | SPACE (row 4) |

This conflict occurs because the joystick grounds the same lines that the
keyboard matrix uses. The system keyscan routine cannot distinguish between
a joystick direction and a keypress.

### Joystick 2 and Keyboard

Joystick 2 shares lines with Port A (keyboard columns). Moving joystick 2
interferes with the column selection, causing fewer phantom keypresses but
potentially disrupting the keyscan. This is why most games use Port 2.

### Workaround

To read joystick 1 without keyboard interference:
1. Disable the keyscan IRQ: `POKE 56333, 127`
2. Select all columns: `POKE 56320, 255`
3. Read Port B: `J1 = NOT PEEK(56321) AND $1F`
4. Re-enable keyscan: `POKE 56333, 129`

Alternatively, clear the keyboard buffer after reading: `POKE 198, 0`.

## Emulation Notes

### Active-Low Logic

Joystick directions are active-low. The emulator must invert the host
joystick/keyboard state before returning it from the CIA port read. When
no direction is pressed, the port reads $FF (all bits 1) for the joystick
bits. When up is pressed on Port 1, bit 0 of $DC01 reads 0.

### Keyboard Interaction

The emulator must model the shared lines between joystick and keyboard.
When a joystick direction is pressed on Port 1, it should ground the
corresponding bit on CIA #1 Port B, which may also be read as a keypress
by the keyboard scanning routine.

### Conceptual Considerations for Emulation

In a Rust emulator, the joystick state can be a simple struct with five
bools (up, down, left, right, fire) per port. When the CPU reads CIA #1
Port A or B, the joystick state is OR'd into the result (with appropriate
inversion). The keyboard matrix state is also OR'd, allowing both devices
to affect the same port bits.