# Keyboard Keycodes

The C64 keyboard is scanned as an 8×8 matrix. Each key has a matrix
coordinate (row, column) that is converted to a keycode (0–63) by the
Kernal's SCNKEY routine. Keycode 64 means no key pressed.

## Keycode Assignment

The keycode is calculated as: `KEYCODE = ROW × 8 + COLUMN`

| Keycode | Key | Keycode | Key | Keycode | Key |
|---|---|---|---|---|---|
| 0 | DEL | 22 | U | 44 | = |
| 1 | RETURN | 23 | V | 45 | ↑ |
| 2 | CRSR RIGHT | 24 | 7 | 46 | / |
| 3 | f7 | 25 | Y | 47 | , |
| 4 | f1 | 26 | G | 48 | £ |
| 5 | f3 | 27 | 8 | 49 | * |
| 6 | f5 | 28 | B | 50 | ; |
| 7 | CRSR DOWN | 29 | H | 51 | HOME |
| 8 | 3 | 30 | U | 52 | (R-SHIFT) |
| 9 | W | 31 | V | 53 | (unused) |
| 10 | A | 32 | 9 | 54 | ↑ |
| 11 | 4 | 33 | I | 55 | / |
| 12 | Z | 34 | J | 56 | 1 |
| 13 | S | 35 | 0 | 57 | ← |
| 14 | E | 36 | M | 58 | (CTRL) |
| 15 | (L-SHIFT) | 37 | K | 59 | 2 |
| 16 | 5 | 38 | O | 60 | SPACE |
| 17 | R | 39 | N | 61 | (C=) |
| 18 | D | 40 | + | 62 | Q |
| 19 | 6 | 41 | P | 63 | RUN/STOP |
| 20 | C | 42 | L | 64 | (no key) |
| 21 | F | 43 | - | | |

## Keycodes Not in Matrix

| Key | Detection Method |
|---|---|
| RESTORE | Wired directly to 6510 NMI line |
| SHIFT LOCK | Mechanical latch on L-SHIFT key |

## Shift/Control/Logo Flag ($028D)

The modifier key state is tracked at $028D (SHFLAG):

| Bit | Value | Key |
|---|---|---|
| 0 | 1 | SHIFT (left or right) pressed |
| 1 | 2 | Commodore logo pressed |
| 2 | 4 | CTRL pressed |

Multiple modifiers OR together (e.g., SHIFT+CTRL = 5).

## Keyboard Decode Table Selection

| Modifier | Table Address | Characters Produced |
|---|---|---|
| None | $EB81 | Uppercase letters, graphics symbols |
| SHIFT | $EBC2 | Shifted graphics, shifted letters |
| C= logo | $EC03 | Graphics characters in upper area |
| CTRL | $EC78 | Control characters (PETASCII 0–31) |

CTRL takes precedence over SHIFT and C= when selecting the decode table.

## Matrix Layout (Row × Column)

```
        Col0   Col1   Col2   Col3   Col4   Col5   Col6   Col7
        $FE    $FD    $FB    $F7    $EF    $DF    $BF    $7F
Row0 $FE  DEL   RET    CRSR←  f7     f1     f3     f5     CRSR↓
Row1 $FD  3     W      A      4      Z      S      E      L-SHIFT
Row2 $FB  5     R      D      6      C      F      T      X
Row3 $F7  7     Y      G      8      B      H      U      V
Row4 $EF  9     I      J      0      M      K      O      N
Row5 $DF  +     P      L      -      .      :      @      ,
Row6 $BF  £     *      ;      HOME   R-SHIFT =      ↑      /
Row7 $7F  1     ←      CTRL   2      SPACE  C=     Q      RUN/STOP
```

## Emulation Notes

When emulating the keyboard, map host keyboard events to (row, column)
pairs. The keycode is `row × 8 + column`. The Kernal SCNKEY routine
converts this to PETASCII using the decode tables. The emulator only needs
to provide the raw matrix state; the ROM handles all conversion.