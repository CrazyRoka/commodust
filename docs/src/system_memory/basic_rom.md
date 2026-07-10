# BASIC ROM ($A000–$BFFF)

The BASIC ROM is an 8K ROM containing the Commodore BASIC V2.0
interpreter. It is a modified version of PET BASIC 2.0, almost identical
to the VIC-20 BASIC interpreter. The ROM is enabled by default (LORAM=1)
and can be switched out to expose underlying RAM.

## ROM Organization

| Address Range | Size | Content |
|---|---|---|
| $A000–$A001 | 2 B | Cold start vector (points to $E394) |
| $A002–$A003 | 2 B | Warm start vector (points to $E37B) |
| $A004–$A00B | 8 B | ASCII text "CBMBASIC" (identifier) |
| $A00C–$A051 | 64 B | Statement dispatch vector table (STMDSP) |
| $A052–$A07F | 46 B | Function dispatch vector table (FUNDSP) |
| $A080–$A09D | 30 B | Operator dispatch vector table (OPTAB) |
| $A09E–$A19D | 256 B | Keyword list (RESLST) |
| $A19E–$A327 | 330 B | Error message text (ERRTAB) |
| $A328–$A364 | 61 B | Error message vector table |
| $A365–$A389 | 37 B | Miscellaneous messages |
| $A38A–$BFFF | ~5K | BASIC interpreter code |

## Cold Start Vector ($A000–$A001)

Points to the BASIC initialization routine at $E394 (in Kernal ROM). This
routine:
1. Sets up RAM vectors at $0300–$030B
2. Initializes BASIC zero-page locations
3. Copies CHRGET routine from ROM to zero page ($0073–$008A)
4. Sets the initial RND seed value
5. Prints the startup message
6. Enters the main BASIC loop

## Warm Start Vector ($A002–$A003)

Points to $E37B. Entered after STOP+RESTORE or BRK. Resets I/O channels,
restores the stack, and enters the main loop without reinitializing
variables.

## Statement Dispatch Table (STMDSP, $A00C–$A051)

Contains 2-byte vectors for each BASIC statement, in token number order
(tokens 128–162). Each vector points one byte before the actual routine
address (RTS-based dispatch).

| Token | Statement | Routine Address |
|---|---|---|
| $80 (128) | END | $A831 |
| $81 (129) | FOR | $A742 |
| $82 (130) | NEXT | $AD1E |
| $83 (131) | DATA | $A8F8 |
| $84 (132) | INPUT# | $ABA5 |
| $85 (133) | INPUT | $ABBF |
| $86 (134) | DIM | $B081 |
| $87 (135) | READ | $AC06 |
| $88 (136) | LET | $A9A5 |
| $89 (137) | GOTO | $A8A0 |
| $8A (138) | RUN | $A871 |
| $8B (139) | IF | $A928 |
| $8C (140) | RESTORE | $A81D |
| $8D (141) | GOSUB | $A883 |
| $8E (142) | RETURN | $A8D2 |
| $8F (143) | REM | $A93B |
| $90 (144) | STOP | $A82F |
| $91 (145) | ON | $A94B |
| $92 (146) | WAIT | $B82D |
| $93 (147) | LOAD | $E168 |
| $94 (148) | SAVE | $E156 |
| $95 (149) | VERIFY | $E165 |
| $96 (150) | DEF | $B3B3 |
| $97 (151) | POKE | $B824 |
| $98 (152) | PRINT# | $AA80 |
| $99 (153) | PRINT | $AAA0 |
| $9A (154) | CONT | $A857 |
| $9B (155) | LIST | $A69C |
| $9C (156) | CLR | $A65E |
| $9D (157) | CMD | $AA86 |
| $9E (158) | SYS | $E12A |
| $9F (159) | OPEN | $E1BE |
| $A0 (160) | CLOSE | $E1C7 |
| $A1 (161) | GET | $AB7B |
| $A2 (162) | NEW | $A642 |

## Function Dispatch Table (FUNDSP, $A052–$A07F)

Contains 2-byte vectors for BASIC functions (tokens 180–202):

| Token | Function | Routine Address |
|---|---|---|
| $B4 | SGN | $BC39 |
| $B5 | INT | $BCCC |
| $B6 | ABS | $BC58 |
| $B7 | USR | $0310 (RAM vector) |
| $B8 | FRE | $B37D |
| $B9 | POS | $B39E |
| $BA | SQR | $BF71 |
| $BB | RND | $E097 |
| $BC | LOG | $B9EA |
| $BD | EXP | $BFED |
| $BE | COS | $E264 |
| $BF | SIN | $E26B |
| $C0 | TAN | $E2B4 |
| $C1 | ATN | $E30E |
| $C2 | PEEK | $B80D |
| $C3 | LEN | $B77C |
| $C4 | STR$ | $B465 |
| $C5 | VAL | $B7AD |
| $C6 | ASC | $B78B |
| $C7 | CHR$ | $B6EC |
| $C8 | LEFT$ | $B700 |
| $C9 | RIGHT$ | $B72C |
| $CA | MID$ | $B737 |

## Keyword Table (RESLST, $A09E–$A19D)

All BASIC keywords stored as ASCII text in token number order. The last
letter of each keyword has bit 7 set (value + 128) to mark the end of the
word. Keywords include all statements, functions, operators, and the word
"GO" (token $CB, for compatibility with "GO TO").

## Error Messages (ERRTAB, $A19E–$A327)

ASCII text of 29 BASIC error messages, each terminated by a byte with bit
7 set. The messages are:

1. TOO MANY FILES
2. FILE OPEN
3. FILE NOT OPEN
4. FILE NOT FOUND
5. DEVICE NOT PRESENT
6. NOT INPUT FILE
7. NOT OUTPUT FILE
8. MISSING FILENAME
9. ILLEGAL DEVICE NUMBER
10. NEXT WITHOUT FOR
11. SYNTAX
12. RETURN WITHOUT GOSUB
13. OUT OF DATA
14. ILLEGAL QUANTITY
15. OVERFLOW
16. OUT OF MEMORY
17. UNDEF'D STATEMENT
18. BAD SUBSCRIPT
19. REDIM'D ARRAY
20. DIVISION BY ZERO
21. ILLEGAL DIRECT
22. TYPE MISMATCH
23. STRING TOO LONG
24. FILE DATA
25. FORMULA TOO COMPLEX
26. CAN'T CONTINUE
27. UNDEF'D FUNCTION
28. VERIFY
29. LOAD

(Message 30, BREAK, is stored separately.)

## Banking

The BASIC ROM is visible at $A000–$BFFF when LORAM=1 (bit 0 of $01 = 1).
When LORAM=0, RAM is visible at this address range instead.

**Write-through:** Even when BASIC ROM is visible, CPU writes to $A000–
$BFFF go to the underlying RAM. This allows software to copy ROM to RAM
by reading ROM addresses and writing to the same addresses.

## Emulation Notes

### ROM Image

The BASIC ROM is an 8K (8192-byte) image, typically loaded from a file
named `basic` or `basic.bin`. The emulator should load this image at
initialization and map it into the $A000–$BFFF address range when LORAM=1.

### Kernal Cross-Reference

The BASIC ROM is not self-contained — it contains JMP instructions to
Kernal ROM addresses (e.g., $E000, $E097, $E12A, etc.). This is because
the BASIC and Kernal ROMs were split across non-contiguous address ranges
($A000–$BFFF and $E000–$FFFF respectively). The emulator must have both
ROM images loaded for BASIC to function.

### CHRGET Copy

At cold start, the BASIC initialization routine copies 24 bytes from
$E3A2 (in Kernal ROM) to $0073–$008A (zero page). This is the CHRGET
routine, which self-modifies its operand to track the current text
position. The emulator must allow writes to zero page to modify this code.

### Variable Storage

BASIC variables are stored in RAM above the program text. The emulator
does not need to model variable storage specifically — it only needs to
provide correct RAM. The BASIC ROM code handles all variable management.