# Color RAM ($D800–$DBFF)

Color RAM is a dedicated 1024 × 4-bit static RAM chip (2114) that
stores the foreground color for each character position on the text
screen. It is physically separate from the main 64K DRAM and is
always present at $D800–$DBFF regardless of banking configuration.

## Hardware Characteristics

| Property | Value |
|---|---|
| Address range | $D800–$DBFF (1024 bytes) |
| Physical size | 1024 × 4 bits (1K × 4 SRAM) |
| Data width | 4 bits (lower nybble only) |
| Upper 4 bits | Not connected; read as random or constant values |
| Banking | Always visible at $D800–$DBFF when I/O is mapped |
| Mirroring | No mirrors; fully decoded in the I/O range |

## Color Values

| Value | Color | Value | Color |
|---|---|---|---|
| 0 | Black | 8 | Orange |
| 1 | White | 9 | Brown |
| 2 | Red | 10 | Light Red |
| 3 | Cyan | 11 | Dark Gray |
| 4 | Purple | 12 | Medium Gray |
| 5 | Green | 13 | Light Green |
| 6 | Blue | 14 | Light Blue |
| 7 | Yellow | 15 | Light Gray |

## Read/Write Behavior

### Writing

Only bits 0–3 are stored. Bits 4–7 are ignored. Writing $FF stores
$0F. Writing $30 stores $00.

### Reading

Bits 0–3 return the stored color value. Bits 4–7 return undefined
data:
- On most C64s, the upper nybble reads as random values (noise).
- On some units, the upper nybble reads as a constant (often $F or $0).

**Rule:** Always mask Color RAM reads with `AND $0F` to extract
the valid color value.

## Usage by Display Mode

### Standard Text Mode

Each byte of Color RAM controls the foreground color of the
corresponding character on the screen. Byte 0 ($D800) controls the
top-left character; byte 999 ($DBE7) controls the bottom-right.

### Multicolor Text Mode

Only the lower 3 bits (values 0–7) select the foreground color for
the `11` bit-pair. Bit 3 acts as a mode selector:
- Bit 3 = 0: Standard hi-res character (8 colors)
- Bit 3 = 1: Multicolor character (8 color selections, but only
  colors 0–7 for the `11` bit-pair)

### Extended Background Color Mode

Color RAM still controls foreground color as in standard text mode.
Background colors are determined by the character's screen code
range (selecting from $D021–$D024).

### Standard Bitmap Mode

Color RAM is **not used**. Foreground and background colors for each
8×8 cell come from the video matrix (screen memory), using the upper
and lower nybbles of each byte.

### Multicolor Bitmap Mode

Color RAM provides the color for the `11` bit-pair for each 8×8 cell.

## Kernal Initialization Behavior

### Original Kernal (early C64s)

When the screen is cleared, Color RAM is filled with $01 (white).

### Later Kernal (most C64s)

When the screen is cleared, Color RAM is filled with the current
value of Background Color Register 0 ($D021). This means POKE'd
characters are invisible (same color as background) until Color RAM
is explicitly set.

**Workaround:** To initialize Color RAM to a specific color:

1. Save the current background color: `C = PEEK($D021)`
2. Set background to desired foreground: `POKE $D021, HUE`
3. Clear screen: `PRINT CHR$(147)`
4. Restore background: `POKE $D021, C`

## Emulation Notes

### 4-Bit Storage Model

The emulator should model Color RAM as a 1024-byte array where only
the lower 4 bits of each byte are significant. On write, mask with
`$0F`. On read, the upper 4 bits should return either:
- Random values (most accurate to real hardware), or
- $F0 (simpler, matches some units).

The random behavior is more accurate and can expose bugs in
software that fails to mask reads.

### Fixed Location

Unlike the video matrix (which can be relocated via $D018), Color
RAM is always at $D800–$DBFF. It cannot be relocated. The VIC-II
always reads color data from this fixed address.

### Banking Independence

Color RAM is part of the I/O block ($D000–$DFFF). It is visible
when CHAREN=1 and HIRAM=1 (or when the I/O space is mapped by the
PLA). When Character ROM is mapped instead of I/O (CHAREN=0),
Color RAM is not accessible by the CPU, but the VIC-II can still
read it for display purposes.

This means the emulator must maintain Color RAM as a separate
data structure that is:
1. Readable/writable by the CPU when I/O is mapped at $D000–$DFFF.
2. Always readable by the VIC-II for display rendering, regardless
   of CPU banking state.

### Save State

Color RAM must be included in save-state data. Only the lower 4
bits of each byte need be saved (512 bytes if packed, or 1024 bytes
if stored unpacked for simplicity).