# PETASCII Table

PETASCII (PET Standard Code for Information Interchange) is Commodore's
variant of ASCII. It is used by BASIC for string data, keyboard input,
and screen output. It is **not** the same as standard ASCII.

## Control Characters (0–31, 128–159)

| Code | Char | Function |
|---|---|---|
| 0 | — | (unused) |
| 1 | — | (unused) |
| 2 | — | (unused) |
| 3 | — | Stop (RUN/STOP key) |
| 4 | — | (unused) |
| 5 | white | Set foreground color to white |
| 8 | — | Disable SHIFT/C= character set toggle |
| 9 | — | Enable SHIFT/C= character set toggle |
| 13 | CR | Carriage return (ENTER) |
| 14 | — | Switch to lowercase/uppercase character set |
| 17 | CRSR↓ | Cursor down |
| 18 | RVS ON | Reverse video on |
| 19 | HOME | Cursor home |
| 20 | DEL | Delete (INST/DEL key) |
| 28 | red | Set foreground color to red |
| 29 | CRSR→ | Cursor right |
| 30 | green | Set foreground color to green |
| 31 | blue | Set foreground color to blue |
| 129 | orange | Set foreground color to orange |
| 133 | f1 | Function key f1 |
| 134 | f3 | Function key f3 |
| 135 | f5 | Function key f5 |
| 136 | f7 | Function key f7 |
| 137 | f2 | Function key f2 |
| 138 | f4 | Function key f4 |
| 139 | f6 | Function key f6 |
| 140 | f8 | Function key f8 |
| 142 | — | Switch to uppercase/graphics character set |
| 144 | black | Set foreground color to black |
| 145 | CRSR↑ | Cursor up |
| 146 | RVS OFF | Reverse video off |
| 147 | CLR | Clear screen |
| 148 | INST | Insert |
| 149 | brown | Set foreground color to brown |
| 150 | lt red | Set foreground color to light red |
| 151 | dk gray | Set foreground color to dark gray |
| 152 | med gray | Set foreground color to medium gray |
| 153 | lt green | Set foreground color to light green |
| 154 | lt blue | Set foreground color to light blue |
| 155 | lt gray | Set foreground color to light gray |
| 156 | purple | Set foreground color to purple |
| 157 | CRSR← | Cursor left |
| 158 | yellow | Set foreground color to yellow |
| 159 | cyan | Set foreground color to cyan |

## Printable Characters (32–127, 160–255)

### Range $20–$3F (32–63): Punctuation and Digits

| Code | Char | Code | Char | Code | Char |
|---|---|---|---|---|---|
| 32 | space | 44 | , | 56 | 8 |
| 33 | ! | 45 | - | 57 | 9 |
| 34 | " | 46 | . | 58 | : |
| 35 | # | 47 | / | 59 | ; |
| 36 | $ | 48 | 0 | 60 | < |
| 37 | % | 49 | 1 | 61 | = |
| 38 | & | 50 | 2 | 62 | > |
| 39 | ' | 51 | 3 | 63 | ? |
| 40 | ( | 52 | 4 | | |
| 41 | ) | 53 | 5 | | |
| 42 | * | 54 | 6 | | |
| 43 | + | 55 | 7 | | |

### Range $40–$5F (64–95): Letters and Symbols (Uppercase Mode)

| Code | Char | Code | Char | Code | Char |
|---|---|---|---|---|---|
| 64 | @ | 76 | L | 88 | X |
| 65 | A | 77 | M | 89 | Y |
| 66 | B | 78 | N | 90 | Z |
| 67 | C | 79 | O | 91 | [ |
| 68 | D | 80 | P | 92 | £ |
| 69 | E | 81 | Q | 93 | ] |
| 70 | F | 82 | R | 94 | ↑ |
| 71 | G | 83 | S | 95 | ← |
| 72 | H | 84 | T | | |
| 73 | I | 85 | U | | |
| 74 | J | 86 | V | | |
| 75 | K | 87 | W | | |

### Range $60–$7F (96–127)

In uppercase/graphics mode, these are graphics characters. In
lowercase/uppercase mode, they are lowercase letters a–z plus symbols.

### Range $A0–$FF (160–255): Shifted/Reversed Characters

These are the shifted versions of $20–$5F and the reversed versions of
$60–$7F. They include shifted graphics characters and reversed text.

## PETASCII vs. ASCII Differences

| Feature | PETASCII | ASCII |
|---|---|---|
| Letters | $41–$5A (A–Z), $C1–$DA (a–z in lowercase mode) | $41–$5A (A–Z), $61–$7A (a–z) |
| @ symbol | $40 | $40 |
| Line ending | $0D (CR only) | $0D $0A (CR LF) |
| Graphics chars | $60–$7F, $A0–$BF | Not present |
| Case toggle | $0E (lowercase on), $8E (uppercase on) | Not applicable |

## Emulation Notes

PETASCII conversion is handled by the Kernal ROM's screen editor. The
emulator does not need to implement PETASCII conversion if it runs real
ROM. If HLE is used, the emulator must implement the CHRIN/CHROUT
routines that convert between PETASCII and screen codes.