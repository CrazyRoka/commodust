# VIC-II Overview

The VIC-II (Video Interface Controller) is the custom video chip
responsible for all graphics display on the C64. It operates
semi-independently from the 6510 CPU, with its own address bus to
system RAM.

## Key Specifications

| Parameter | NTSC (6567) | PAL (6569) |
|---|---|---|
| Total scan lines per frame | 262 | 312 |
| Visible scan lines | 200 (lines 50–249) | 200 (lines 50–249) |
| Frames per second | ~60 | ~50 |
| CPU clock | 1.02273 MHz | 0.98525 MHz |
| Cycles per line | 65 | 63 |
| Visible pixels per line | 320 | 320 |
| Total pixels per line (incl. blanking) | ~403 | ~403 |

## Memory Architecture

The VIC-II can address only **16 KB** of memory at a time. The active
16K bank is selected by bits 0–1 of CIA #2 Data Port A ($DD00):

| Bits 1:0 | Bank | Address Range |
|---|---|---|
| 00 | 3 | $C000–$FFFF |
| 01 | 2 | $8000–$BFFF |
| 10 | 1 | $4000–$7FFF |
| 11 | 0 | $0000–$3FFF (default) |

**Note the inverted logic:** bit pattern 11 = bank 0, 00 = bank 3.

Within the selected 16K bank, the VIC-II places:

- **Character ROM** at offset $1000–$1FFF (banks 0 and 2 only)
- **Video matrix** (screen codes + sprite pointers) at a 1K boundary
  selected by $D018 bits 4–7
- **Character dot-data** at a 2K boundary selected by $D018 bits 1–3
- **Bitmap data** at offset $0000 or $2000 (selected by $D018 bit 3
  in bitmap mode)
- **Sprite data** in any of 256 64-byte blocks within the bank

## Bus Arbitration (Bad Lines and Sprite DMA)

The VIC-II and 6510 share the data bus to system RAM. The VIC-II
performs its memory fetches during the portion of each clock cycle when
the 6510 is not using the bus. However, certain fetches require
additional bus cycles that must be stolen from the CPU:

### Bad Lines

A "bad line" occurs when:
1. The raster line is within the visible display area (lines 50–249).
2. The raster line number modulo 8 equals the vertical scroll value
   (bits 0–2 of $D011).
3. Bit 4 of $D011 is set (display enabled).

On a bad line, the VIC-II fetches 40 screen codes and 40 color nybbles
(80 bytes total), requiring **40 additional bus cycles** stolen from
the 6510. During these cycles, the CPU is completely stalled.

This means bad lines occur every 8th scan line during the visible
display, for a total of 25 bad lines per frame.

### Sprite DMA

Each enabled sprite requires 2 bytes per scan line for pointer fetch
plus additional cycles for data fetch:

| Sprites Enabled | Additional Cycles Stolen per Line |
|---|---|
| 0 | 0 |
| 1 | 3 |
| 2 | 6 |
| ... | ... |
| 8 | 24 (maximum) |

### Performance Impact

With display on and all 8 sprites enabled, the 6510 loses a significant
portion of its bus cycles to the VIC-II. With the screen blanked
(bit 4 of $D011 = 0), no DMA occurs and the CPU runs at full speed —
approximately **7% faster** than with display enabled.

## Display Modes Summary

| Mode | Bit 5 ($D011) | Bit 4 ($D016) | Resolution | Colors |
|---|---|---|---|---|
| Standard text | 0 | 0 | 40×25 chars | 16 fg + 1 bg |
| Multicolor text | 0 | 1 | 40×25 chars (4px wide) | 4 per char |
| Extended bg text | 1* | 0 | 40×25 chars (64 shapes) | 16 fg + 4 bg |
| Standard bitmap | 1 | 0 | 320×200 | 2 per 8×8 cell |
| Multicolor bitmap | 1 | 1 | 160×200 | 4 per 8×8 cell |

*Bit 6 of $D011, not bit 5, enables extended background color mode.

## Graphics Systems

The VIC-II provides three independent graphics systems that can
partially coexist:

1. **Text/Bitmap display** — The main screen, using the video matrix
   and character/bitmap data. All display modes use this system.

2. **Sprites** — 8 independent 24×21 pixel objects that float above or
   below the main display. Sprites have their own position, color,
   priority, and collision detection registers.

3. **Raster interrupts** — Programmable interrupts that fire at a
   specific scan line, enabling mid-screen mode changes, split-screen
   effects, and scrolling zones.

See the individual sections for detailed documentation of each system.