# Text Modes

The VIC-II supports three text (character) display modes. All use the video
matrix (1K of screen codes) and character dot-data (2K of shape data) to
generate the display.

## Mode Selection

| Mode | $D011 Bit 6 (ECM) | $D016 Bit 4 (MCM) | Description |
|---|---|---|---|
| Standard text | 0 | 0 | 256 characters, 16 fg colors + 1 bg |
| Multicolor text | 0 | 1 | 256 chars, 4 colors per char (half horizontal res) |
| Extended bg color | 1 | 0 | 64 characters, 4 bg colors |
| Invalid | 1 | 1 | Undefined behavior on real hardware |

**Default:** Standard text mode ($D011 = $9B → bit 6 = 0; $D016 = $08 →
bit 4 = 0).

## Standard Text Mode

### Display Structure

- 40 columns × 25 rows of characters
- Each character position: 8×8 pixels
- Total resolution: 320×200 pixels
- Each video matrix byte holds a screen code (0–255)
- Each Color RAM nybble holds a foreground color (0–15)

### Color Assignment

| Pixel State | Color Source |
|---|---|
| Bit = 1 (foreground) | Color RAM nybble for that character position |
| Bit = 0 (background) | Background Color Register 0 ($D021) |

### Character Data Format

Each character is 8 bytes, one per row. Bit 7 is the leftmost pixel; bit 0
is the rightmost. A set bit (1) indicates foreground color; a clear bit (0)
indicates background color.

### Video Matrix

The video matrix is 1000 bytes (40 × 25), located at the 1K boundary
selected by $D018 bits 4–7. Each byte is a screen code, not a PETASCII
value. Screen codes 0–255 map to character shapes in the character data
area.

The last 8 bytes of the 1K video matrix region ($07F8–$07FF in the default
configuration) are sprite shape data pointers.

## Multicolor Text Mode

Enabled when $D016 bit 4 = 1 (and $D011 bit 6 = 0).

### Display Structure

- 40 columns × 25 rows
- Each character position: 4×8 pixels (half horizontal resolution)
- Each pixel is 2× normal width (double-width dots)
- Character dot-data bits are grouped in pairs

### Color Assignment per Bit-Pair

| Bit Pair | Color Source |
|---|---|
| 00 | Background Color Register 0 ($D021) |
| 01 | Background Color Register 1 ($D022) |
| 10 | Background Color Register 2 ($D023) |
| 11 | Color RAM nybble (lower 3 bits only) |

### Per-Character Mode Selection

Bit 3 of the Color RAM nybble acts as a mode selector for each character
individually:

| Color RAM Bit 3 | Character Display Mode |
|---|---|
| 0 | Standard hi-res character (8 pixels wide, 8 colors for fg) |
| 1 | Multicolor character (4 pixels wide, 4 colors) |

When bit 3 = 0, the character is displayed in standard mode despite the
global multicolor bit being set. Only characters with Color RAM bit 3 = 1
are displayed in multicolor mode.

When bit 3 = 1, only colors 0–7 are available for the `11` bit-pair (the
4th bit is consumed by the mode flag). The color value is the lower 3 bits
of the Color RAM nybble.

### Custom Character Sets

Standard character ROM data is designed for hi-res mode. Multicolor mode
requires custom character sets designed with bit-pair encoding. Each byte
of character data encodes 4 double-width pixels rather than 8 single-width
pixels.

## Extended Background Color Mode

Enabled when $D011 bit 6 = 1 (and $D016 bit 4 = 0).

### Display Structure

- 40 columns × 25 rows
- Only 64 character shapes available (screen codes 0–63)
- Each character can have one of 4 different background colors

### Background Color Selection

The background color for each character is determined by its screen code
range:

| Screen Code Range | Background Color Register | Default Color |
|---|---|---|
| 0–63 | BGCOL0 ($D021) | Blue ($06) |
| 64–127 | BGCOL1 ($D022) | White ($01) |
| 128–191 | BGCOL2 ($D023) | Red ($02) |
| 192–255 | BGCOL3 ($D024) | Cyan ($03) |

Characters with screen codes 64–255 display the same shape as the
corresponding character in the 0–63 range (screen code AND $3F), but with a
different background color.

### Foreground Color

Foreground color is still determined by Color RAM, same as standard text
mode. All 16 colors are available for foreground.

### Limitations

- Only 64 unique character shapes (the alphabet, numbers, and some
  punctuation). Graphics characters (screen codes 64+) are unavailable as
  unique shapes.
- Characters with codes ≥ 64 must be typed using shifted or reversed
  character equivalents, which is inconvenient from BASIC.

### Use Case: Windows

Extended background color mode is useful for creating colored windows on
the screen. Different regions can have different background colors, and the
background can be changed instantly by modifying the color register.

## Screen Codes vs. PETASCII

Text mode uses **screen codes**, not PETASCII values. Screen codes map
directly to character shape indices. See [Screen Codes](../appendices/screen_codes.md)
for the full mapping.

Key differences:
- Screen code 0 = `@` (not NUL)
- Screen code 1 = `A`
- Screen codes 32–63 = shifted/graphics characters
- Screen codes 64–95 = same shapes as 0–31 but reversed
- Screen codes 96–127 = same shapes as 32–63 but reversed
- Screen codes 128–255 = duplicates of 0–127

## Emulation Notes

### Character Fetch Pipeline

For each visible scan line that is a "bad line" (every 8th line within the
visible display), the VIC-II fetches:
1. 40 bytes of screen codes from the video matrix (40 bus cycles)
2. 40 bytes of color data from Color RAM (40 bus cycles, but color is only
   4 bits wide, so the VIC-II reads a full byte and masks)

The character dot-data is fetched on the fly during each character row's
display, using the screen code as an index into the character data area.

### Multicolor Bit-Pair Processing

In multicolor mode, the emulator must process character data bytes in
2-bit pairs rather than individual bits. For each pair, the color source
is selected from one of four registers/RAM locations. This halves the
effective horizontal resolution.

### Extended Color Background Lookup

The emulator must check the screen code range for each character to select
the appropriate background color register. This is a simple range check
(top 2 bits of the screen code select the register).

### Conceptual Considerations for Emulation

The text mode renderer can be implemented as a function that, for each
character cell (x, y), reads the screen code from the video matrix, reads
the color nybble from Color RAM, fetches 8 bytes of character data, and
renders 8 pixels (or 4 in multicolor). The rendering must respect the
current mode bits and apply the correct color mapping.

For cycle-accurate emulation, the character data fetch must be interleaved
with CPU execution, as bad lines steal cycles from the CPU.