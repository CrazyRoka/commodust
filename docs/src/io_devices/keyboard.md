# Keyboard

The C64 keyboard is an 8×8 matrix of key switches connected to CIA #1
Data Port A (columns, output) and Data Port B (rows, input). The keyboard
is scanned by the system IRQ handler approximately 60 times per second.

## Keyboard Matrix

The matrix is organized as 8 columns (Port A) × 8 rows (Port B):

| | Col 0 ($FE) | Col 1 ($FD) | Col 2 ($FB) | Col 3 ($F7) | Col 4 ($EF) | Col 5 ($DF) | Col 6 ($BF) | Col 7 ($7F) |
|---|---|---|---|---|---|---|---|---|
| **Row 0** ($FE) | DEL | RETURN | CRSR← | f7 | f1 | f3 | f5 | CRSR↓ |
| **Row 1** ($FD) | 3 | W | A | 4 | Z | S | E | L-SHIFT |
| **Row 2** ($FB) | 5 | R | D | 6 | C | F | T | X |
| **Row 3** ($F7) | 7 | Y | G | 8 | B | H | U | V |
| **Row 4** ($EF) | 9 | I | J | 0 | M | K | O | N |
| **Row 5** ($DF) | + | P | L | - | . | : | @ | , |
| **Row 6** ($BF) | £ | * | ; | HOME | R-SHIFT | = | ↑ | / |
| **Row 7** ($7F) | 1 | ← | CTRL | 2 | SPACE | C= | Q | RUN/STOP |

The value in parentheses is the byte written to Port A to select that
column (0 in the bit position for the selected column, 1 for all others).

## Scanning Procedure

1. **Configure ports:** Port A DDR = $FF (all outputs), Port B DDR = $00
   (all inputs). This is the default.
2. **Select a column:** Write a byte to Port A ($DC00) with a 0 in the bit
   corresponding to the desired column and 1s in all other bits.
3. **Read rows:** Read Port B ($DC01). A 0 in any bit position means the
   key at that row and the selected column is pressed. $FF means no key in
   that column is pressed.

### System IRQ Scan

The system IRQ handler (at $EA31) cycles through all 8 columns every
1/60 second and reads Port B for each. The results are decoded into
PETASCII characters and placed in the keyboard buffer at $0277–$0280.

## Special Keys

### SHIFT LOCK

SHIFT LOCK is not part of the matrix. It is a mechanical latch that holds
the left SHIFT key switch closed. The keyboard scanner cannot distinguish
between SHIFT LOCK and holding left SHIFT.

### RESTORE

RESTORE is not part of the matrix. It is wired directly to the 6510 NMI
line and triggers an NMI interrupt when pressed. It has no keycode and
cannot be detected via the keyboard matrix.

## Keyboard Decode Tables

Four decode tables translate matrix positions to PETASCII characters,
selected by the modifier keys:

| Modifier | Table Address | Table Name |
|---|---|---|
| None (unshifted) | $EB81 | Standard uppercase/graphics |
| SHIFT | $EBC2 | Shifted characters |
| Commodore logo | $EC03 | Logo characters |
| CTRL | $EC78 | Control characters |

The table selection is performed by the routine at $EB48, vectored through
$028F–$0290 (KEYLOG). The SHIFT/CTRL/Logo flag at $028D determines which
table is used. CTRL takes precedence over other modifiers.

### CTRL Key Combinations

CTRL combined with A–Z produces PETASCII codes 1–26. Other CTRL
combinations produce control codes 0–31. For example:
- CTRL-@ = CHR$(0)
- CTRL-A = CHR$(1)
- CTRL-; = CHR$(29) (cursor right)
- CTRL-N = CHR$(14) (switch to lowercase)
- CTRL-R = CHR$(18) (reverse video on)

## SHIFT/Logo Character Set Toggle

Pressing SHIFT and the Commodore logo key simultaneously toggles between
the uppercase/graphics character set and the lowercase/uppercase character
set. This changes the VIC-II Memory Control Register ($D018 bits 1–3)
between offset $1000 (Set 1) and $1800 (Set 2).

This toggle can be disabled by setting $0291 to $80, or by printing
CHR$(8). It can be re-enabled with CHR$(9).

## Keyboard Buffer

| Location | Name | Description |
|---|---|---|
| $0277–$0280 | KEYD | 10-byte FIFO keyboard buffer |
| $00C6 | NDX | Character count in buffer |
| $0289 | XMAX | Maximum buffer size (default 10) |

The IRQ handler deposits PETASCII characters into the buffer. BASIC's GET
and INPUT retrieve them. Characters beyond the maximum are ignored.

### Dynamic Keyboard Technique

Software can simulate keyboard input by POKEing PETASCII characters
(including cursor control and carriage returns) into the buffer and setting
$00C6 to the character count. When the program ends (END), the buffered
characters are processed as if typed. This enables programs to modify
themselves by simulating direct-mode commands.

## Key Repeat

| Location | Name | Default | Function |
|---|---|---|---|
| $028A | RPTFLG | $00 | Repeat flag: 0 = cursor/DEL/SPACE only; $80 = all keys; $40 = none |
| $028B | KOUNT | 6 | Repeat delay counter (counts down after initial delay) |
| $028C | DELAY | 16 | Initial repeat delay (counts down every jiffy) |

The initial delay before first repeat is approximately 1/3 second (22
jiffies). Subsequent repeats occur at approximately 15 per second (4
jiffies apart).

## Emulation Notes

### Matrix Modeling

The emulator should model the keyboard as an 8×8 boolean matrix. Each key
on the host keyboard maps to a row/column pair. When a key is pressed, set
the corresponding matrix cell. When a key is released, clear it.

### Scanning Emulation

When the system IRQ handler writes to CIA #1 Port A, the emulator should
return the appropriate Port B value based on the current matrix state. A
0 bit in Port B means a key in that row (for the selected column) is
pressed.

### Modifier Key Handling

The SHIFT, CTRL, and Commodore logo keys occupy specific matrix positions
and should be modeled as part of the matrix. The RESTORE key is not in the
matrix and should be handled as a separate NMI trigger.

### Keyboard Buffer

If the emulator runs real Kernal ROM, the keyboard buffer is managed by
the ROM code. The emulator only needs to provide correct CIA #1 register
behavior. If the emulator uses HLE (high-level emulation) of the keyboard,
it must manage the buffer, character count, and decode tables itself.

## Key Repeat Timing

| Phase | Counter | Initial Value | Tick Rate | Result |
|---|---|---|---|---|
| Initial delay | $028C (DELAY) | 16 | Every 1/60s | When 0, begin repeat |
| Repeat interval | $028B (KOUNT) | 6 | Every 1/60s | When 0, output char and reset to 4 |

**Timing:**
- Initial delay before first repeat: 16 jiffies ≈ 0.267 seconds
- After first repeat, KOUNT resets to 4 (not 6)
- Subsequent repeats: every 4 jiffies ≈ 15 per second
- Total time to first repeat: approximately 1/3 second (22 jiffies)

### Repeat Flag ($028A)

| Value | Behavior |
|---|---|
| $00 | Only cursor keys, INST/DEL, and SPACE repeat (default) |
| $80 | All keys repeat |
| $40 | No keys repeat |

The repeat flag is checked by the IRQ keyscan routine. When a key
is held down, the routine checks RPTFLG to determine if the key
should be repeated. Only keys in the "repeat" group (or all keys
if $80) are placed in the keyboard buffer during repeat.

### Conceptual Considerations for Emulation

In a Rust emulator, the keyboard can be a struct owning an 8×8 array of
bools. Host keyboard events are mapped to matrix coordinates. When the
CPU reads CIA #1 Port B, the result is computed by checking which keys in
the currently selected column (from Port A) are pressed.