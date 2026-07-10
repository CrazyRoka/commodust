# Screen Codes

Screen codes are the byte values stored in the video matrix (screen memory)
to select which character shape is displayed. They are **not** the same as
PETASCII values.

## Screen Code Table

### Uppercase/Graphics Character Set (Set 1, $1000–$17FF)

| Code | Char | Code | Char | Code | Char | Code | Char |
|---|---|---|---|---|---|---|---|
| 0 | @ | 32 | space | 64 | -@ | 96 | -space |
| 1 | A | 33 | ! | 65 | -A | 97 | -! |
| 2 | B | 34 | " | 66 | -B | 98 | -" |
| 3 | C | 35 | # | 67 | -C | 99 | -# |
| 4 | D | 36 | $ | 68 | -D | 100 | -$ |
| 5 | E | 37 | % | 69 | -E | 101 | -% |
| 6 | F | 38 | & | 70 | -F | 102 | -& |
| 7 | G | 39 | ' | 71 | -G | 103 | -' |
| 8 | H | 40 | ( | 72 | -H | 104 | -( |
| 9 | I | 41 | ) | 73 | -I | 105 | -) |
| 10 | J | 42 | * | 74 | -J | 106 | -* |
| 11 | K | 43 | + | 75 | -K | 107 | -+ |
| 12 | L | 44 | , | 76 | -L | 108 | -, |
| 13 | M | 45 | - | 77 | -M | 109 | -- |
| 14 | N | 46 | . | 78 | -N | 110 | -. |
| 15 | O | 47 | / | 79 | -O | 111 | -/ |
| 16 | P | 48 | 0 | 80 | -P | 112 | -0 |
| 17 | Q | 49 | 1 | 81 | -Q | 113 | -1 |
| 18 | R | 50 | 2 | 82 | -R | 114 | -2 |
| 19 | S | 51 | 3 | 83 | -S | 115 | -3 |
| 20 | T | 52 | 4 | 84 | -T | 116 | -4 |
| 21 | U | 53 | 5 | 85 | -U | 117 | -5 |
| 22 | V | 54 | 6 | 86 | -V | 118 | -6 |
| 23 | W | 55 | 7 | 87 | -W | 119 | -7 |
| 24 | X | 56 | 8 | 88 | -X | 120 | -8 |
| 25 | Y | 57 | 9 | 89 | -Y | 121 | -9 |
| 26 | Z | 58 | : | 90 | -Z | 122 | -: |
| 27 | [ | 59 | ; | 91 | -[ | 123 | -; |
| 28 | £ | 60 | < | 92 | -£ | 124 | -< |
| 29 | ] | 61 | = | 93 | -] | 125 | -= |
| 30 | ↑ | 62 | > | 94 | -↑ | 126 | -> |
| 31 | ← | 63 | ? | 95 | -← | 127 | -? |

Codes 128–255 are duplicates of 0–127 (same character shapes).

The `-` prefix indicates reversed video (inverse) characters. Codes 64–95
show the same shapes as 0–31 but reversed. Codes 96–127 show the same
shapes as 32–63 but reversed.

## Screen Code Organization

| Code Range | Description |
|---|---|
| 0–31 | Standard characters (@, A–Z, [, £, ], ↑, ←) |
| 32–63 | Numbers, punctuation, graphics symbols |
| 64–95 | Reversed versions of 0–31 |
| 96–127 | Reversed versions of 32–63 |
| 128–255 | Duplicates of 0–127 |

## Screen Code vs. PETASCII Conversion

### PETASCII to Screen Code

| PETASCII Range | Screen Code | Operation |
|---|---|---|
| $00–$1F | — | Control characters (no screen code) |
| $20–$3F | $20–$3F | Same value (space through ?) |
| $40–$5F | $00–$1F | Subtract $40 (@ through ←) |
| $60–$7F | — | Not used in uppercase mode |
| $80–$9F | — | Control characters (no screen code) |
| $A0–$BF | $60–$7F | Subtract $40 (shifted space through ?) |
| $C0–$DF | $40–$5F | Subtract $80 (shifted @ through ←) |
| $E0–$FF | $60–$7F | Subtract $80 (reversed graphics) |

### Screen Code to PETASCII

| Screen Code | PETASCII (uppercase mode) | PETASCII (lowercase mode) |
|---|---|---|
| 0–31 | $40–$5F (chr$("@ABC...")) | $40–$5F (same) |
| 32–63 | $20–$3F (same) | $20–$3F (same) |
| 64–95 | $00–$1F (control chars, unlikely) | $60–$7F (lowercase a–z) |

## Extended Background Color Mode Restriction

In extended background color mode ($D011 bit 6 = 1), only screen codes 0–63
select unique character shapes. Codes 64–255 display the same shape as the
code AND $3F, but with a different background color register.

## Emulation Notes

The video matrix stores screen codes, not PETASCII. The screen editor
routines (in Kernal ROM) handle the conversion between PETASCII (what
BASIC uses) and screen codes (what the VIC-II displays). The emulator
does not need to perform this conversion if it runs real Kernal ROM —
the ROM handles it. If using HLE, the emulator must implement the
conversion table.