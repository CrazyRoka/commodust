# Zero Page ($0000–$00FF)

Zero page holds the most performance-critical memory in the 6510
architecture. Instructions targeting zero-page addresses are shorter
(1 byte for the address instead of 2) and execute one cycle faster
than their absolute-mode equivalents. Because of this, the C64's
Operating System and BASIC interpreter make heavy use of every byte.

## Power-On Initialization

At cold start, the Kernal routine RAMTAS ($FD50) fills all 256 bytes
of zero page with $00, then initializes specific locations from ROM
constants. The CHRGET routine is copied from ROM at $E3A2 to $0073.

## Memory Map

### $0000–$0001: 6510 On-Chip I/O Port

See [6510 On-Chip I/O Port](./io_port.md) for full details.

### $0002: Unused

### $0003–$0004: ADRAY1 — Float-to-Integer Vector

Points to the BASIC routine that converts a floating-point number in
FAC1 to a signed 16-bit integer. Default target: $B1AA.

BASIC does not appear to call through this vector internally, but it
is provided for user programs (especially USR functions) that need
to convert floating-point parameters.

### $0005–$0006: ADRAY2 — Integer-to-Float Vector

Points to the BASIC routine that converts a 16-bit signed integer to
a floating-point number in FAC1. Default target: $B391.

### $0007: CHARAC — Search Character

Used by BASIC's text-scanning routines to detect significant
characters (quotes, commas, colons, end-of-line markers). The ASCII
value of the character being searched for is stored here.

### $0008: ENDCHR — Statement Terminator / Quote Search

Work byte during tokenization. Usually holds 0 or 34 (the quote
character).

### $0009: TRMPOS — Cursor Column Before TAB/SPC

Stores the cursor column position before a TAB or SPC function is
executed, used to calculate the final cursor position. Since logical
lines can be up to 80 characters, this value ranges from 0 to 79.

### $000A: VERCK — LOAD/VERIFY Flag

Set to 0 for LOAD, 1 for VERIFY. Passed to the Kernal LOAD routine,
which stores it at $0093.

### $000B: COUNT — Text Buffer Index / Array Subscript Count

Used as an index into the BASIC input buffer during tokenization.
Also used to count array subscripts during DIM operations.

### $000C: DIMFLG — Array Flags

Used by array routines to track whether a variable is an array,
whether it has been DIMensioned, and whether default dimensions
should apply.

### $000D: VALTYP — Data Type Flag

| Value | Meaning |
|---|---|
| $FF (255) | String data |
| $00 (0) | Numeric data |

Set every time a variable is located or created.

### $000E: INTFLG — Numeric Subtype Flag

| Value | Meaning |
|---|---|
| $80 (128) | Integer |
| $00 (0) | Floating point |

### $000F: GARBFL — Garbage Collection / LIST / Tokenization Flag

Triple-use flag: (1) LIST uses it to detect quoted strings. (2)
Garbage collection uses it to indicate a prior collection attempt.
(3) Tokenization uses it as a work byte.

### $0010: SUBFLG — Subscript / FN Flag

Set by PTRGET when an opening parenthesis is found, indicating an
array variable or user-defined function call.

### $0011: INPFLG — Input Source Flag

| Value | Meaning |
|---|---|
| $98 (152) | READ |
| $40 (64) | GET |
| $00 (0) | INPUT |

### $0012: TANSGN — Sign Flag for SIN/TAN / Comparison Result

For SIN/TAN: stores the sign of the result. For comparisons: stores
1 (A>B), 2 (A=B), or 4 (A<B), or combinations for compound
operators (>=, <=).

### $0013: CHANNL — Current I/O Channel Number

Holds the logical file number of the currently active I/O channel.
Value 0 means default (keyboard input / screen output). Nonzero
values indicate CMD redirection is active.

### $0014–$0015: LINNUM — Integer Line Number / Address Pointer

Target line number for GOTO, LIST, ON, GOSUB. Also used as an
address pointer for PEEK, POKE, WAIT, and SYS.

### $0016: TEMPPT — Temporary String Stack Pointer

Points to the next available slot in the temporary string descriptor
stack at $19–$21. Points to $19 (empty), $1C (one entry), $1F (two
entries), or $22 (full). If full when a new entry is needed, the
FORMULA TOO COMPLEX error is issued.

### $0017–$0018: LASTPT — Pointer to Last String Descriptor

Points to the last slot used in the temporary string descriptor
stack. $17 should be 3 less than $16; $18 is always 0.

### $0019–$0021: TEMPST — Temporary String Descriptor Stack

Holds up to three 3-byte descriptors for temporary strings (strings
not yet assigned to a variable). Each descriptor contains: length
(1 byte), start address displacement (2 bytes).

### $0022–$0025: INDEX — Miscellaneous Temp Pointers

Work area for temporary pointers and calculation results used by
many BASIC routines.

### $0026–$002A: RESHO — Floating Point Multiplication Work Area

Used by BASIC multiplication and division, and by array size
calculation routines.

### $002B–$002C: TXTTAB — Pointer to Start of BASIC Program Text

Default: $0801. Can be changed to relocate BASIC text. Also used as
the start address for SAVE.

### $002D–$002E: VARTAB — Pointer to Start of Variable Storage

Marks the end of BASIC program text and the beginning of non-array
variable storage. Variables are stored as 7-byte descriptors:
2 bytes name + 5 bytes value. Also used as the end address for SAVE.

### $002F–$0030: ARYTAB — Pointer to Start of Array Storage

Marks the end of non-array variables and the start of array variable
storage.

### $0031–$0032: STREND — Pointer to End of Arrays / Start of Free RAM

Marks the end of array storage and the beginning of free RAM. String
text builds downward from the top of memory toward this point.

### $0033–$0034: FRETOP — Pointer to Bottom of String Text Storage

Marks the current end of string text storage (strings grow downward
from top of memory). Garbage collection readjusts this upward.

### $0035–$0036: FRESPC — Temporary String Pointer

Temporary pointer to the most recently added string.

### $0037–$0038: MEMSIZ — Pointer to Highest BASIC Address

Set at power-on to $9FFF (highest byte before BASIC ROM). Lowered
by 512 bytes when RS-232 device is opened (to allocate I/O buffers).

### $0039–$003A: CURLIN — Current BASIC Line Number

Line number currently being executed. A value of $FF in $3A (line
number ≥ 65280) indicates immediate/direct mode.

### $003B–$003C: OLDLIN — Previous BASIC Line Number

Saved by STOP/END/BREAK, restored by CONT.

### $003D–$003E: OLDTXT — Pointer to Current Statement Address

Address (not line number) of the BASIC statement being executed.
Saved/restored by STOP/CONT.

### $003F–$0040: DATLIN — Current DATA Line Number

Line number of the DATA statement currently being READ. Used so
errors point to the DATA line, not the READ line.

### $0041–$0042: DATPTR — Pointer to Current DATA Item

Address within BASIC program text where the next DATA item will be
read from. RESTORE resets this to the address pointed to by TXTTAB
($2B).

### $0043–$0044: INPPTR — Pointer to Input Source

Pointer to the source of data for GET, READ, or INPUT.

### $0045–$0046: VARNAM — Current Variable Name

The name of the variable currently being searched for or created,
in the same 2-byte format used in variable descriptors.

### $0047–$0048: VARPNT — Pointer to Current Variable Value

Points to the byte immediately after the 2-byte variable name in
the variable descriptor.

### $0049–$004A: FORPNT — FOR Loop Index Variable Pointer

Address of the FOR/NEXT loop variable. Initially stored here, then
pushed onto the stack.

### $004B–$004C: OPPTR — Math Operator Table Displacement

Displacement of the current math operator in the operator table.

### $004D: OPMASK — Comparison Operation Mask

| Value | Meaning |
|---|---|
| 1 | Less-than comparison |
| 2 | Equals comparison |
| 4 | Greater-than comparison |

### $004E–$004F: DEFPNT — FN Descriptor Pointer

During DEF FN: points to the descriptor being created. During FN
call: points to the descriptor where results are saved.

### $0050–$0052: DSCPNT — Temporary String Descriptor Pointer

First two bytes: temporary pointer to current string descriptor.
Third byte: string length.

### $0053: FOUR6 — Garbage Collection Constant

Indicates whether a 3-byte or 7-byte string descriptor is being
collected.

### $0054–$0056: JMPER — Jump to Function Instruction

Contains a JMP instruction ($4C) followed by the address of the
required function from the FUNDSP table at $A052.

### $0057–$0060: BASIC Numeric Work Area

General-purpose work area used by many BASIC routines.

### $0061–$0066: FAC1 — Floating Point Accumulator #1

Central to all BASIC math operations. 5-byte floating-point format:

| Location | Name | Description |
|---|---|---|
| $0061 | FACEXP | Exponent (128 = 2^0, 129 = 2^1, etc.; 0 = value is zero) |
| $0062–$0065 | FACHO | 4-byte mantissa (bit 7 of $62 = sign) |
| $0066 | FACSGN | Sign (0 = positive, $FF = negative) |

**Internal format:** The mantissa represents a number between 1.0
and 1.9999... The exponent has a bias of 128. The value is:
`mantissa × 2^(exponent - 128)`.

During float-to-integer conversion, $0062–$0063 hold the signed
16-bit integer result (high byte, low byte).

### $0067: SGNFLG — Series Evaluation Term Count

Indicates the number of separate evaluations needed to resolve a
complex expression.

### $0068: BITS — FAC1 Overflow Digit

Holds the least significant figures during integer-to-float
conversion, used for rounding.

### $0069–$006E: FAC2 — Floating Point Accumulator #2

Second accumulator, same 5-byte format as FAC1:

| Location | Name | Description |
|---|---|---|
| $0069 | ARGEXP | Exponent |
| $006A–$006D | ARGHO | Mantissa |
| $006E | ARGSGN | Sign |

### $006F: ARISGN — Sign Comparison Result

0 = FAC1 and FAC2 have like signs. $FF = unlike signs.

### $0070: FACOV — FAC1 Low-Order Mantissa (Rounding)

Holds overflow digits for extended accuracy during intermediate
calculations.

### $0071–$0072: FBUFPT — Series Evaluation Pointer

Points to a temporary table of values for formula evaluation. Also
used as TI$ work area and array size determination workspace.

### $0073–$008A: CHRGET — Get Next BASIC Text Character

A 24-byte machine language subroutine copied from ROM ($E3A2) to
zero page at cold start. It is placed in RAM so it can self-modify
its internal pointer.

**Entry points:**

| Address | Label | Function |
|---|---|---|
| $0073 | CHRGET | Increment TXTPTR ($7A), fetch next character |
| $0079 | CHRGOT | Fetch current character without incrementing |
| $007A–$007B | TXTPTR | 2-byte pointer to current text position (self-modified operand) |

**On exit:**
- Accumulator (.A) holds the character read.
- Carry flag: Clear if character was ASCII digit 0–9; Set otherwise.
- Zero flag: Set if character was 0 (end of line) or ":" ($3A).

The CHRGET routine is critical to BASIC performance. Because it
resides in RAM, it is a common insertion point for "wedges" — code
that intercepts BASIC to add new commands.

### $008B–$008F: RNDX — RND Seed Value

5-byte floating-point seed for the RND function. Initialized at
power-on from ROM constants: $80, $4F, $C7, $52, $58.

### $0090: STATUS — Kernal I/O Status Word (ST)

Updated by all Kernal I/O routines. Equivalent to the BASIC
reserved variable ST.

**Cassette status bits:**

| Bit | Value | Meaning |
|---|---|---|
| 2 | 4 | Short block |
| 3 | 8 | Long block |
| 4 | 16 | Unrecoverable error / mismatch |
| 5 | 32 | Checksum error |
| 6 | 64 | End of file |
| 7 | 128 | End of tape |

**Serial device status bits:**

| Bit | Value | Meaning |
|---|---|---|
| 0 | 1 | Time out (write) |
| 1 | 2 | Time out (read) |
| 6 | 64 | EOI (End or Identify) |
| 7 | 128 | Device not present |

### $0091: STKEY — STOP Key Flag

Updated every 1/60 second by the IRQ keyboard scan. Holds the value
of the last keyboard matrix row (which contains STOP). Value $7F
(127) means STOP is pressed.

### $0092: SVXT — Tape Read Timing Constant

Adjustable timing constant for tape reads.

### $0093: VERCK — Load/Verify Flag (Kernal Copy)

0 = LOAD, 1 = VERIFY. Copied from $000A by the Kernal LOAD routine.

### $0094: C3PO — Serial Bus Output Buffered Flag

Indicates a character is waiting in the serial output buffer.

### $0095: BSOUR — Buffered Serial Character

The character waiting for serial output. $FF means no character is
waiting.

### $0096: SYNO — Cassette Block Sync Number

### $0097: XSAV — Temporary .X Register Save

### $0098: LDTND — Number of Open Files

Index to the end of the file/device/secondary-address tables at
$259–$276. Maximum is 10. Decremented by CLOSE; reset to 0 by
CLALL.

### $0099: DFLTN — Default Input Device

Default: 0 (keyboard). Changed by CHKIN.

### $009A: DFLTO — Default Output Device

Default: 3 (screen). Changed by CHKOUT.

### $009B: PRTY — Tape Character Parity

### $009C: DPSW — Tape Byte Received Flag

### $009D: MSGFLG — Kernal Message Control Flag

| Value | Meaning |
|---|---|
| $C0 (192) | Both error and control messages |
| $80 (128) | Control messages only |
| $40 (64) | Error messages only |
| $00 (0) | All messages suppressed (RUN mode) |

### $009E: PTR1 — Tape Pass 1 Error Log Index

### $009F: PTR2 — Tape Pass 2 Error Log Index

### $00A0–$00A2: Software Jiffy Clock

3-byte counter incremented 60 (NTSC) or 50 (PAL) times per second.
Wraps to 0 after 24 hours (1,946,000 jiffies approx).

| Location | Update Frequency |
|---|---|
| $A0 (160) | Every 65536 jiffies (~18.2 minutes) |
| $A1 (161) | Every 256 jiffies (~4.2 seconds) |
| $A2 (162) | Every jiffy (~16.67 ms NTSC) |

Accessed by BASIC reserved variables TI and TI$.

### $00A3–$00A4: Temporary Tape/Serial Storage

### $00A5: CNTDN — Cassette Sync Countdown

### $00A6: BUFPNT — Tape Buffer Character Count

### $00A7: INBIT — RS-232 Input Bits / Tape Temp

### $00A8: BITCI — RS-232 Input Bit Count / Tape Temp

### $00A9: RINONE — RS-232 Start Bit Flag

$90 = no start bit received; $00 = start bit received.

### $00AA: RIDATA — RS-232 Input Byte Buffer / Tape Temp

### $00AB: RIPRTY — RS-232 Input Parity / Tape Leader Counter

### $00AC–$00AD: SAL — Load/Save Start Address Pointer / Scroll Work

### $00AE–$00AF: EAL — Load/Save End Address Pointer

### $00B0–$00B1: CMPO — Tape Timing

### $00B2–$00B3: TAPE1 — Pointer to Tape Buffer

Default: $033C (828).

### $00B4: BITTS — RS-232 Output Bit Count / Tape Temp

### $00B5: NXTBIT — RS-232 Next Bit / Tape EOT Flag

### $00B6: RODATA — RS-232 Output Byte Buffer

### $00B7: FNLEN — Current Filename Length

### $00B8: LA — Current Logical File Number

File numbers > 127 cause a linefeed to follow each carriage return
on output.

### $00B9: SA — Current Secondary Address

Range 0–31 for serial devices, 0–127 for others.

### $00BA: FA — Current Device Number

| Number | Device |
|---|---|
| 0 | Keyboard |
| 1 | Datassette |
| 2 | RS-232 / User Port |
| 3 | Screen |
| 4–5 | Printer |
| 8–11 | Disk drive |

### $00BB–$00BC: FNADR — Pointer to Current Filename

### $00BD: ROPRTY — RS-232 Output Parity / Tape Temp

### $00BE: FSBLK — Cassette Block Count

### $00BF: MYCH — Tape Input Byte Buffer

### $00C0: CAS1 — Tape Motor Interlock

Maintained by the IRQ routine. If 0 and a cassette button is pressed, the motor is turned on (by clearing Bit 5 of `$01` to 0). Software must write a nonzero value here after a button press to bypass the automatic IRQ keyscan override and gain direct manual control of the motor.

### $00C1–$00C2: STAL — I/O Start Address

### $00C3–$00C4: MEMUSS — Tape Load Temporary Address

### $00C5: LSTX — Last Key Pressed (Matrix Coordinate)

Holds the debounced matrix coordinate of the last key pressed (64 = no key pressed). Used by the OS to prevent unwanted key repetition unless key repeating is explicitly authorized.

### $00C6: NDX — Keyboard Buffer Character Count

Number of characters waiting in the keyboard buffer at $277.
Maximum is determined by $0289 (default 10).

### $00C7: RVS — Reverse Video Flag

$12 = reverse on; $00 = normal. Reset to 0 at every carriage return.

### $00C8: INDX — End of Logical Line Pointer

Column number of the last nonblank character on the current logical
line (0–79).

### $00C9–$00CA: LXSP — Cursor X,Y at Start of Input

### $00CB: SFDX — Current Key Pressed (Matrix Coordinate)

Holds the raw matrix coordinate of the key currently pressed. This coordinate maps to specific keyboard matrix scan lines. The matrix values are decoded via the following coordinate indexes:

| Coordinate | Key | Coordinate | Key | Coordinate | Key |
|---|---|---|---|---|---|
| **0** | INST/DEL | **22** | T | **44** | . |
| **1** | RETURN | **23** | X | **45** | : |
| **2** | CRSR RIGHT | **24** | 7 | **46** | @ |
| **3** | f7 | **25** | Y | **47** | , |
| **4** | f1 | **26** | G | **48** | £ |
| **5** | f3 | **27** | 8 | **49** | * |
| **6** | f5 | **28** | B | **50** | ; |
| **7** | CRSR DOWN | **29** | H | **51** | CLR/HOME |
| **8** | 3 | **30** | U | **52** | Unused |
| **9** | W | **31** | V | **53** | Unused |
| **10** | A | **32** | 9 | **54** | ↑ |
| **11** | 4 | **33** | I | **55** | / |
| **12** | Z | **34** | J | **56** | 1 |
| **13** | S | **35** | 0 | **57** | ← |
| **14** | E | **36** | M | **58** | Unused |
| **15** | Unused | **37** | K | **59** | 2 |
| **16** | 5 | **38** | O | **60** | SPACE BAR |
| **17** | R | **39** | N | **61** | Unused |
| **18** | D | **40** | + | **62** | Q |
| **19** | 6 | **41** | P | **63** | RUN/STOP |
| **20** | C | **42** | L | **64** | No Key |
| **21** | F | **43** | - | | |

### $00CC: BLNSW — Cursor Blink Enable

0 = flash cursor; nonzero = disable blink.

### $00CD: BLNCT — Cursor Blink Countdown

Starts at 20, decremented every jiffy. Cursor blinks when it
reaches 0 (approximately 3 blinks/second).

### $00CE: GDBLN — Character Under Cursor

Holds the screen code of the character at the cursor position so it
can be restored when the cursor moves.

### $00CF: BLNON — Cursor Blink State

0 = cursor currently reversed; 1 = cursor restored to normal.

### $00D0: CRSW — Input Source Flag

3 = input from screen; 0 = input from keyboard.

### $00D1–$00D2: PNT — Pointer to Current Screen Line

Address in screen RAM of the first column of the cursor's current
logical line.

### $00D3: PNTR — Cursor Column Position

Column within the current logical line (0–79).

### $00D4: CITSW — Quote Mode Flag

Nonzero = editor is in quote mode. Toggled by each quote character
typed. In quote mode, cursor control characters appear as printed
equivalents instead of executing.

### $00D5: LNMX — Maximum Physical Line Length

### $00D6: TBLX — Current Cursor Physical Line (0–24)

### $00D7: Last Printed Character (Temporary)

### $00D8: INSRT — Insert Mode Flag

Value > 0 = number of insert spaces pending. Behaves like quote
mode, except DELETE also leaves a printed equivalent.

### $00D9–$00F2: LDTB1 — Screen Line Link Table

25 entries, one per screen row. Bits 0–3: page offset for screen
line address calculation. Bit 7: set if this line is the first or
only physical line of a logical line; clear if it is the second half
of a logical line.

### $00F3–$00F4: USER — Pointer to Current Color RAM Location

Synchronized with $D1 (screen line pointer). Holds the address of
the first byte of color RAM for the corresponding screen line.

### $00F5–$00F6: KEYTAB — Keyboard Decode Table Vector

Points to the currently active keyboard translation table:

| Address | Table |
|---|---|
| $EB81 | Default (unshifted) |
| $EBC2 | SHIFTed |
| $EC03 | Commodore logo |
| $EC78 | CTRL |

### $00F7–$00F8: RIBUF — RS-232 Input Buffer Pointer

### $00F9–$00FA: ROBUF — RS-232 Output Buffer Pointer

### $00FB–$00FE: FREKZP — Free Zero-Page Bytes

Four bytes specifically reserved for user machine-language programs.
BASIC will never alter these.

### $00FF: BASZPT — Float-to-ASCII Temporary

## Emulation Notes

### CHRGET Self-Modification

The CHRGET routine is self-modifying code: it writes to its own
operand bytes ($007A–$007B) to update TXTPTR. An emulator must
allow writes to the zero-page range $0073–$008A to modify the
emulated RAM at those locations, even though the code is "running"
there. This is straightforward if zero page is backed by the same
RAM array as the rest of memory.

### Kernal Dependency on Zero Page

The Kernal IRQ handler, keyboard scanner, screen editor, and I/O
routines all depend on zero-page state. An emulator that runs real
Kernal ROM code must ensure zero-page locations are correctly
initialized and maintained. If the emulator replaces Kernal
functions with native code (a "high-level" emulator), it can skip
most zero-page state but must still handle $0000–$0001 (banking)
and $0091 (STOP key) for compatibility.