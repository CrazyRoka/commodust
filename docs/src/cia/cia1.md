# CIA #1 — Keyboard and Joystick

**Base address: $DC00 ($DC00–$DC0F)**
**Interrupt line: 6510 IRQ**

## Port A ($DC00) — Keyboard Columns / Joystick 2

**Default DDR: $FF (all outputs)**

| Bit | Keyboard Function | Joystick 2 | Other |
|---|---|---|---|
| 0 | Select column 0 | Up | — |
| 1 | Select column 1 | Down | — |
| 2 | Select column 2 | Left | Paddle 1 fire |
| 3 | Select column 3 | Right | Paddle 2 fire |
| 4 | Select column 4 | Fire button | — |
| 5 | Select column 5 | — | — |
| 6 | Select column 6 | — | Paddle port select |
| 7 | Select column 7 | — | Paddle port select |

### Paddle Port Selection

| Bit 7 | Bit 6 | Paddles Read |
|---|---|---|
| 1 | 0 | Port 1 (paddles 1 & 2) |
| 0 | 1 | Port 2 (paddles 3 & 4) |

## Port B ($DC01) — Keyboard Rows / Joystick 1

**Default DDR: $00 (all inputs)**

| Bit | Keyboard Function | Joystick 1 | Other |
|---|---|---|---|
| 0 | Row 0 (DEL, RETURN, CRSR←, f7, f1, f3, f5, CRSR↓) | Up | — |
| 1 | Row 1 (3, W, A, 4, Z, S, E, L-SHIFT) | Down | — |
| 2 | Row 2 (5, R, D, 6, C, F, T, X) | Left | Paddle 1 fire |
| 3 | Row 3 (7, Y, G, 8, B, H, U, V) | Right | Paddle 2 fire |
| 4 | Row 4 (9, I, J, 0, M, K, O, N) | Fire button | — |
| 5 | Row 5 (+, P, L, -, ., :, @, ,) | — | — |
| 6 | Row 6 (£, *, ;, HOME, R-SHIFT, =, ↑, /) | — | Timer A output |
| 7 | Row 7 (1, ←, CTRL, 2, SPACE, C=, Q, RUN/STOP) | — | Timer B output |

## Keyboard Matrix Scanning

The keyboard is an 8×8 matrix. To read a specific key:

1. Set Port A DDR to all outputs ($FF) and Port B DDR to all inputs ($00).
   (This is the default.)
2. Write a byte to Port A with a 0 in the bit corresponding to the
   desired column, and 1s in all other bits.
3. Read Port B. A 0 in any bit position means the key at that row and
   the selected column is pressed. A value of $FF means no key in that
   column is pressed.

**Important:** The system IRQ handler scans the keyboard every 1/60
second by cycling through all 8 columns on Port A and reading Port B.
This means software that needs to read Port A for other purposes
(joystick 2, paddles) will conflict with the keyscan routine. The
conflict can be resolved by temporarily disabling the IRQ
(`POKE 56333, 127`) during such reads.

## Joystick Reading

### Joystick 1 (Controller Port 1, read via Port B)

```
J1 = NOT PEEK($DC01) AND $1F
```

| Value | Direction |
|---|---|
| 0 | None |
| 1 | Up |
| 2 | Down |
| 4 | Left |
| 5 | Up-Left |
| 6 | Down-Left |
| 8 | Right |
| 9 | Up-Right |
| 10 | Down-Right |

Fire button: `(PEEK($DC01) AND $10) = 0` when pressed.

### Joystick 2 (Controller Port 2, read via Port A)

```
J2 = NOT PEEK($DC00) AND $1F
```

Same value table as above. Fire button: `(PEEK($DC00) AND $10) = 0`.

## Joystick/Keyboard Conflict

Joystick 1 shares lines with the keyboard matrix. Moving joystick 1
in certain directions registers as keypresses:

| Joystick 1 Direction | Key Registered |
|---|---|
| Up | 1 (row 0) |
| Down | 2 (row 1) |
| Left | CTRL (row 6) |
| Right | 2 (row 7) |
| Fire | SPACE (row 4) |

This is why many games use **joystick 2** (Port 2) instead — it
conflicts with the keyboard column selection rather than the row
readout, causing fewer phantom keypresses.

## System Timer

CIA #1 Timer A is used by the system to generate the 1/60 second IRQ
that drives:

- Keyboard scanning
- Software jiffy clock ($A0–$A2)
- Cursor blink
- Cassette motor interlock
- STOP key checking

**Latch value (NTSC):** $4118 (16664 decimal) → 1022730/16664 ≈ 61.4 Hz
**Latch value (PAL):** $4295 (17045 decimal) → 985250/17045 ≈ 57.8 Hz

The actual exact values are tuned for 60 Hz (NTSC) or 50 Hz (PAL).