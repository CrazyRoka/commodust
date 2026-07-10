# Screen Memory and BASIC Text ($0400–$9FFF)

This ~36 KB region holds the default video matrix, sprite pointers,
and the BASIC program text with its variable storage.

## $0400–$07E7: Default Video Matrix (1000 bytes)

25 rows × 40 columns of screen codes. Each byte holds a screen code
(not ASCII/PETASCII) that determines which character shape is
displayed. The first byte ($0400) corresponds to the top-left corner
of the screen.

**Screen code layout:** See [Screen Codes](../appendices/screen_codes.md).

The video matrix can be relocated by changing bits 4–7 of $D018 and
bits 0–1 of $DD00 (VIC-II bank). When relocated, the OS pointer at
$0288 (HIBASE) must also be updated so PRINT statements target the
correct address.

### Color RAM Correspondence

Each byte in the video matrix has a corresponding nybble in Color
RAM at $D800–$DBFF. The color nybble at $D800 + offset controls the
foreground color of the character at screen position `offset`.

**Newer Kernal behavior:** When the screen is cleared, Color RAM is
initialized to the current background color value (from $D021),
making POKE'd characters invisible. To work around this, set the
background color to the desired foreground color, clear the screen,
then restore the background.

## $07E8–$07FF: Sprite Shape Data Pointers (8 bytes)

The last 8 bytes of the video matrix (regardless of where it is
relocated) serve as pointers to sprite shape data.

| Address | Sprite | Default Value |
|---|---|---|
| $07F8 | 0 | Set by Kernal to 13 (sprite data at $0340) |
| $07F9 | 1 | — |
| $07FA | 2 | — |
| $07FB | 3 | — |
| $07FC | 4 | — |
| $07FD | 5 | — |
| $07FE | 6 | — |
| $07FF | 7 | — |

Each pointer holds a block number (0–255). The block number × 64
gives the offset within the VIC-II's 16K bank where the 63-byte
sprite shape data begins. For example, a pointer value of 13 means
sprite data starts at offset 13 × 64 = 832 ($0340) within the
current VIC-II bank.

**Sprite data format:** 63 bytes of pixel data (24 pixels wide × 21
rows = 504 bits, rounded up to 63 bytes). The 64th byte is unused
(padding to reach 64-byte alignment). Each row is 3 bytes (24 bits).

## $0800–$9FFF: BASIC Program Text and Variables

### BASIC Program Text Layout

BASIC programs start at $0801 (the address pointed to by TXTTAB at
$002B). Each program line consists of:

```
┌──────────────────────────────────────────────────────┐
│ Byte 0-1: Pointer to next line (low, high)            │
│           Two zeros ($00 $00) = end of program        │
│ Byte 2-3: Line number (low, high)                     │
│ Byte 4+:  Tokenized BASIC code                        │
│ Last byte: $00 (line terminator)                      │
└──────────────────────────────────────────────────────┘
```

**Tokens:** BASIC keywords are stored as single bytes ≥ $80. For
example, PRINT is token $99 (153). Text outside quotes is
tokenized; text inside quotes is stored as-is.

### Memory Layout (low to high)

```
$0800  ┌──────────────────────────┐
       │  BASIC Program Text       │
       │  (linked lines of tokens) │
       ├──────────────────────────┤
       │  Non-array Variables      │
       │  (7 bytes each)           │
       ├──────────────────────────┤
       │  Array Variables          │
       │  (variable size)          │
       ├──────────────────────────┤
       │  Free RAM                 │
       │  (reported by FRE(0))     │
       ├──────────────────────────┤
       │  String Text Storage      │
       │  (grows downward)         │
$9FFF  └──────────────────────────┘
```

### Variable Descriptor Format (7 bytes)

| Bytes | Content |
|---|---|
| 0–1 | Variable name (2 ASCII chars, high bits encode type) |
| 2–6 | Value (format depends on type) |

**Name byte high bits:**

| Byte 0 bit 7 | Byte 1 bit 7 | Type |
|---|---|---|
| 0 | 0 | Floating point |
| 1 | 0 | String |
| 0 | 1 | Integer |
| 1 | 1 | Function (FN) |

**Value formats:**

| Type | Bytes 2–6 |
|---|---|
| Float | 5-byte floating point (exponent + 4-byte mantissa) |
| Integer | 2-byte value (high, low) + 3 unused bytes |
| String | 1 byte length + 2 byte pointer to text + 2 unused bytes |
| Function | 2 byte pointer to DEF text + 2 byte pointer to variable + 1 unused |

### Array Storage Format

| Bytes | Content |
|---|---|
| 0–1 | Array name (same format as non-array) |
| 2–3 | Offset to next array (low, high) |
| 4 | Number of dimensions |
| 5+ | Dimension sizes (2 bytes each, each = size + 1) |
| ... | Element values (packed: float=5, integer=2, string=3 bytes each) |

### NEW Command Behavior

NEW does not erase program text. It writes two zeros as the first
line's link pointer, marking the program as empty, and resets the
variable pointers. A program can be recovered from NEW by restoring
the original link pointer and finding the true end of program text.

## $1000–$1FFF: Character ROM Image (VIC-II Bank 0)

When the VIC-II uses bank 0 (default) or bank 2, it sees the
Character Generator ROM at offset $1000–$1FFF within the bank.
This means:

- The 6510 sees **RAM** at these addresses.
- The VIC-II sees **Character ROM** at these addresses.

This 4K region contains two 2K character sets:
- $1000–$17FF: Uppercase + graphics characters
- $1800–$1FFF: Lowercase + uppercase characters

**Implication for emulator:** The VIC-II memory access function
must overlay the Character ROM at these offsets when the bank is 0
or 2, even though the CPU sees RAM there. This is a separate
address space consideration from the CPU's banking logic.

## $8000–$9FFF: Cartridge / BASIC Extension Area

- $8000: Autostart cartridge cold-start vector.
- $8002: Autostart cartridge warm-start vector (RESTORE key).
- $8004–$8008: Autostart signature bytes ($C3, $C2, $CD, $38, $30
  = inverse "CBM80"). If these bytes are present at power-on, the
  Kernal jumps through the cold-start vector.
- $9000–$9FFF: Character ROM image for VIC-II bank 2.

## Emulation Notes

### Screen Memory Relocation

When software relocates the screen, it must update:
1. $D018 bits 4–7 (VIC-II video matrix pointer)
2. $DD00 bits 0–1 (VIC-II bank, if changing banks)
3. $0288 (OS screen memory page pointer)

If any of these is changed without updating the others, display
corruption or invisible text results. An emulator should not enforce
these relationships — software bugs that leave them inconsistent
should produce the same visual artifacts as real hardware.

### VIC-II vs CPU Address Space

The most common emulator bug in this memory region is forgetting
that the VIC-II and CPU see different things at $1000–$1FFF (and
$9000–$9FFF). The CPU always sees RAM. The VIC-II sees Character
ROM in banks 0 and 2. The emulator's VIC-II fetch function must
have its own address-decoding path that accounts for this overlay.

### BASIC Variable Storage Emulation

If the emulator runs real BASIC ROM, variable storage is handled
entirely by the ROM code. The emulator only needs to provide correct
RAM. If the emulator replaces BASIC with native code (unlikely for
a C64 emulator), it would need to model the variable format
described above.