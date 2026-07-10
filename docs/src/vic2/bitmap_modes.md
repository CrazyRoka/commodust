# Bitmap Modes

The VIC-II supports two bitmap display modes that provide pixel-level
graphics control. In bitmap mode, the screen is divided into 320×200
(standard) or 160×200 (multicolor) individually addressable pixels.

## Mode Selection

| Mode | $D011 Bit 5 (BMM) | $D016 Bit 4 (MCM) | Resolution | Colors per 8×8 cell |
|---|---|---|---|---|
| Standard bitmap | 1 | 0 | 320×200 | 2 |
| Multicolor bitmap | 1 | 1 | 160×200 | 4 |
| Invalid (text+bitmap) | — | — | — | Undefined |

**Enable bitmap mode:** Set bit 5 of $D011 (`POKE 53265, PEEK(53265) OR 32`).

## Standard Bitmap Mode

### Resolution

320×200 pixels. Each bit in the bitmap controls one pixel: 1 = foreground
color, 0 = background color.

### Memory Layout

The bitmap is 8K (8192 bytes, 8000 used). Its location within the VIC-II
bank is determined by $D018 bit 3:

| $D018 Bit 3 | Bitmap Offset | Bank 0 Address |
|---|---|---|
| 0 | $0000 | $0000 |
| 1 | $2000 | $2000 |

### Bitmap Data Organization

The bitmap is organized as if it were 1000 programmable characters (40
columns × 25 rows), each 8×8 pixels. The data layout is **not sequential**
in the typical left-to-right, top-to-bottom order.

```
Byte Address = Bitmap_Base + 320 × Row_Group + 8 × Column + Row_Within_Group
```

Where:
- `Row_Group` = Y coordinate divided by 8 (integer division), range 0–24
- `Row_Within_Group` = Y coordinate AND 7, range 0–7
- `Column` = X coordinate divided by 8, range 0–39

This means the first 8 bytes control the first column of 8 pixels vertically,
the next 8 bytes control the second column, etc. After 320 bytes (40
columns), the pattern repeats for the next row group.

```
Address  Data           Screen Position
$2000    XXXXXXXX  →    Row 0, Column 0 (top-left 8 pixels, first row)
$2001    XXXXXXXX  →    Row 1, Column 0 (second row of first column)
...
$2007    XXXXXXXX  →    Row 7, Column 0
$2008    XXXXXXXX  →    Row 0, Column 1
$2009    XXXXXXXX  →    Row 1, Column 1
...
$201F    XXXXXXXX  →    Row 7, Column 1
$2020    XXXXXXXX  →    Row 0, Column 2
...
```

### Color Information

Color in standard bitmap mode comes from the video matrix (the same 1K
region used for screen codes in text mode). Each byte of the video matrix
provides two colors for one 8×8 cell:

| Nybble | Color Source |
|---|---|
| Upper 4 bits (bits 4–7) | Foreground color (bit = 1 in bitmap) |
| Lower 4 bits (bits 0–3) | Background color (bit = 0 in bitmap) |

**Color RAM is not used** in standard bitmap mode. Each 8×8 cell can have
exactly 2 colors (foreground and background), independently of all other
cells.

### Pixel Addressing Formula

Given pixel coordinates (X, Y) where X = 0–319 and Y = 0–199:

```
BYTE_ADDRESS = BITMAP_BASE + (Y AND 248) × 40 + (Y AND 7) + (X AND 504)
BIT_POSITION = 7 - (X AND 7)   (bit 7 = leftmost pixel)
```

Where `BITMAP_BASE` is the bitmap start address within the VIC-II bank.

## Multicolor Bitmap Mode

Enabled when $D011 bit 5 = 1 AND $D016 bit 4 = 1.

### Resolution

160×200 pixels. Each pixel is 2× normal width. Pairs of bits in the bitmap
control one pixel.

### Color Information per Bit-Pair

| Bit Pair | Color Source |
|---|---|
| 00 | Background Color Register 0 ($D021) |
| 01 | Upper 4 bits of video matrix byte for that 8×8 cell |
| 10 | Lower 4 bits of video matrix byte for that 8×8 cell |
| 11 | Color RAM nybble for that 8×8 cell |

### Advantages Over Standard Bitmap

Each 8×8 cell can have up to **4 different colors** (one from $D021 shared
globally, two from the video matrix byte unique per cell, and one from
Color RAM unique per cell). This is more colors per cell than standard
bitmap mode (which allows only 2).

### Pixel Addressing Formula

Given pixel coordinates (X, Y) where X = 0–159 and Y = 0–199:

```
BYTE_ADDRESS = BITMAP_BASE + (Y AND 248) × 40 + (Y AND 7) + (X × 2 AND 504)
BIT_PAIR_POSITION = 6 - (X AND 3) × 2   (bit-pair starts at this bit position)
```

### Setting a Multicolor Pixel

To set pixel (X, Y) to color CO (0–3):

1. Calculate BYTE_ADDRESS and BIT_PAIR_POSITION
2. Clear the existing bit-pair: `DATA = DATA AND NOT (3 × BIT_MASK)`
3. Set the new bit-pair: `DATA = DATA OR (CO × BIT_MASK)`

Where BIT_MASK = `1 << BIT_PAIR_POSITION` (with 2-bit pairs, the mask is
3 bits wide shifted to the correct position).

## Bitmap Mode and Character ROM

In VIC-II banks 0 and 2, the Character ROM overlay at $1000–$1FFF can
interfere with bitmap data. If the bitmap is placed at offset $0000 (bit 3
of $D018 = 0), it occupies $0000–$1FFF, which overlaps the Character ROM
overlay region. The VIC-II will read Character ROM data instead of RAM for
the $1000–$1FFF portion, corrupting the bitmap.

**Solution:** In banks 0 and 2, always place the bitmap at offset $2000
(bit 3 of $D018 = 1) to avoid the Character ROM overlay.

## Emulation Notes

### Bitmap Data Fetch

The VIC-II fetches bitmap data using the same bad-line mechanism as text
mode. On each bad line (every 8th scan line), the VIC-II fetches 40 bytes
of video matrix data (for color information) and 40 bytes of bitmap data.
During the display of each character row, it fetches the 8 bytes of bitmap
data for that row.

### Non-Sequential Memory Layout

The non-sequential bitmap layout must be handled carefully in the
emulator's rendering function. A common bug is to assume sequential layout
(first byte = top-left, second byte = next pixel right). The actual layout
is character-oriented: 8 bytes per character column, 40 columns per row
group, 25 row groups.

### Color Source Multiplexing

In standard bitmap mode, the color for each 8×8 cell comes from the video
matrix, not Color RAM. The emulator must read from the video matrix address
(pointed to by $D018 bits 4–7) rather than Color RAM ($D800).

In multicolor bitmap mode, three color sources are mixed: $D021, video
matrix nybbles, and Color RAM nybbles. The emulator must correctly route
each bit-pair to its color source.

### Conceptual Considerations for Emulation

The bitmap renderer can share much of the text mode renderer's
infrastructure. Both modes use 40×25 cells of 8×8 pixels. The difference
is:
- Text mode: cell shape comes from character data (indexed by screen code)
- Bitmap mode: cell shape comes directly from bitmap memory

The color mapping also differs. A unified rendering pipeline that checks
the mode bits and dispatches to the appropriate color/data source is
cleaner than separate renderers for each mode.