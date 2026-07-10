# VIC-II Register Reference

The VIC-II chip occupies 47 registers at $D000–$D02E. Only the lower
6 address bits are decoded, so every 64-byte block within $D000–$D3FF
is a mirror of the base 64 bytes.

## Register Summary

| Addr | Name | Default | Description |
|---|---|---|---|
| $D000 | SP0X | $00 | Sprite 0 horizontal position (low 8 bits) |
| $D001 | SP0Y | $00 | Sprite 0 vertical position |
| $D002 | SP1X | $00 | Sprite 1 horizontal position (low 8 bits) |
| $D003 | SP1Y | $00 | Sprite 1 vertical position |
| $D004 | SP2X | $00 | Sprite 2 horizontal position (low 8 bits) |
| $D005 | SP2Y | $00 | Sprite 2 vertical position |
| $D006 | SP3X | $00 | Sprite 3 horizontal position (low 8 bits) |
| $D007 | SP3Y | $00 | Sprite 3 vertical position |
| $D008 | SP4X | $00 | Sprite 4 horizontal position (low 8 bits) |
| $D009 | SP4Y | $00 | Sprite 4 vertical position |
| $D00A | SP5X | $00 | Sprite 5 horizontal position (low 8 bits) |
| $D00B | SP5Y | $00 | Sprite 5 vertical position |
| $D00C | SP6X | $00 | Sprite 6 horizontal position (low 8 bits) |
| $D00D | SP6Y | $00 | Sprite 6 vertical position |
| $D00E | SP7X | $00 | Sprite 7 horizontal position (low 8 bits) |
| $D00F | SP7Y | $00 | Sprite 7 vertical position |
| $D010 | MSIGX | $00 | MSB of horizontal position for sprites 0–7 |
| $D011 | SCROLY | $9B | Screen control register 1 (raster MSB, modes, scroll) |
| $D012 | RASTER | — | Raster line (read) / raster compare (write) |
| $D013 | LPENX | — | Light pen X position |
| $D014 | LPENY | — | Light pen Y position |
| $D015 | SPENA | $00 | Sprite enable register |
| $D016 | SCROLX | $08 | Screen control register 2 (modes, scroll, reset) |
| $D017 | YXPAND | $00 | Sprite vertical expansion |
| $D018 | VMCSB | $15 | Memory control register (screen/char base) |
| $D019 | VICIRCL | $0F | Interrupt flag register (read; write 1 to clear) |
| $D01A | IRQMSK | $00 | Interrupt mask register |
| $D01B | SPBGPR | $00 | Sprite-to-foreground priority |
| $D01C | SPMC | $00 | Sprite multicolor mode enable |
| $D01D | XXPAND | $00 | Sprite horizontal expansion |
| $D01E | SPSPCL | — | Sprite-to-sprite collision (read; cleared on read) |
| $D01F | SPBGCL | — | Sprite-to-foreground collision (read; cleared on read) |
| $D020 | EXTCOL | $0E | Border color |
| $D021 | BGCOL0 | $06 | Background color 0 |
| $D022 | BGCOL1 | $01 | Background color 1 |
| $D023 | BGCOL2 | $02 | Background color 2 |
| $D024 | BGCOL3 | $03 | Background color 3 |
| $D025 | SPMC0 | $04 | Sprite multicolor 0 |
| $D026 | SPMC1 | $00 | Sprite multicolor 1 |
| $D027 | SP0COL | $01 | Sprite 0 color |
| $D028 | SP1COL | $02 | Sprite 1 color |
| $D029 | SP2COL | $03 | Sprite 2 color |
| $D02A | SP3COL | $04 | Sprite 3 color |
| $D02B | SP4COL | $05 | Sprite 4 color |
| $D02C | SP5COL | $06 | Sprite 5 color |
| $D02D | SP6COL | $07 | Sprite 6 color |
| $D02E | SP7COL | $0C | Sprite 7 color |

**Note:** All color registers ($D020–$D02E) have only the lower 4 bits
connected. The upper 4 bits read back as random/undefined values and
must be masked with `AND $0F`.

### Kernal Default Table Quirk

The Kernal initializes VIC-II registers from a table at `$ECB9`. This
table is 47 bytes long (one byte per VIC-II register `$D000`–`$D02E`).

**Known issue:** The table appears to run one byte short. Sprite Color
Registers 0–6 are correctly initialized to 1–7, but Sprite Color
Register 7 (`$D02E`) receives the value 76 (`$4C`), which is actually
the first byte of the *next* data structure in ROM — the ASCII text
"LOAD" at `$ECE7` (used for the SHIFT+RUN keyboard buffer injection).

The value `$4C` corresponds to... no valid C64 color. However, since
only the lower 4 bits of color registers are connected, the effective
color is `$4C AND $0F = $0C` (12 = medium gray), which matches the
documented default. This is either a fortunate coincidence or deliberate
design.

**Emulation note:** If the emulator initializes VIC-II registers
directly (rather than executing the Kernal table copy), simply set
`$D02E` to `$0C`. If emulating the Kernal table copy exactly, the
table at `$ECB9` should be 47 bytes, and the 48th register (`$D02E`)
should receive the byte at `$ECE7` (which is `$4C`).

## Register $D011 — Screen Control Register 1

**Default: $9B (%10011011)**

| Bit | Mask | Default | Function |
|---|---|---|---|
| 0–2 | $07 | 3 | Vertical fine scroll offset (0–7) |
| 3 | $08 | 1 | Rows: 1 = 25 rows, 0 = 24 rows |
| 4 | $10 | 1 | Display: 1 = visible, 0 = blanked (border color fills screen) |
| 5 | $20 | 0 | Bitmap mode: 1 = enabled |
| 6 | $40 | 0 | Extended background color mode: 1 = enabled |
| 7 | $80 | 1 | Raster compare bit 8 (MSB of $D012) |

### Bit 4 — Screen Blanking

When bit 4 = 0, the VIC-II does not perform character/sprite DMA,
eliminating bad-line and sprite bus stealing. The 6510 runs
approximately **7% faster**. The entire screen shows the border color.
This is used during tape I/O for timing-critical operations.

### Bit 7 — Raster MSB

Bit 7 of $D011 is bit 8 of the 9-bit raster compare register. The low
8 bits are in $D012. This split arrangement is a common source of bugs
when setting up raster interrupts — if bit 7 is not set correctly, the
interrupt fires on the wrong line (off by 256).

### Machine-language note

Bit 7's position at bit 7 allows testing with the Negative (N) flag:

```text
LOOP    LDA $D011      ; load raster MSB
        BPL LOOP       ; branch if bit 7 = 0 (raster < 256)
```

This idiom waits for the raster to enter the 256–311 range (vertical
blanking interval on NTSC), which is the safe time to modify screen
data without visible artifacts.

## Register $D012 — Raster Line

This register is dual-function:

- **Read:** Returns the current raster line being scanned (bits 0–7).
  Combine with bit 7 of $D011 for the full 9-bit value.
- **Write:** Sets the raster compare value for raster interrupts.

**NTSC:** 262 total lines (0–261), visible 50–249.
**PAL:** 312 total lines (0–311), visible 50–249.

## Register $D016 — Screen Control Register 2

**Default: $08 (%00001000)**

| Bit | Mask | Default | Function |
|---|---|---|---|
| 0–2 | $07 | 0 | Horizontal fine scroll offset (0–7) |
| 3 | $08 | 1 | Columns: 1 = 40 columns, 0 = 38 columns |
| 4 | $10 | 0 | Multicolor mode: 1 = enabled |
| 5 | $20 | 0 | VIC reset: 0 = normal, 1 = video completely off |
| 6–7 | $C0 | 0 | Unused |

### Bit 3 — 38/40 Column Select

Setting to 0 covers the first and last columns with the border, useful
for hiding the edge effects of horizontal fine scrolling.

### Bit 4 — Multicolor Enable

In text mode: enables multicolor character mode (4-color, half horizontal
resolution). In bitmap mode: enables multicolor bitmap mode. See
[Text Modes](text_modes.md) and [Bitmap Modes](bitmap_modes.md).

### Bit 5 — VIC Reset

Setting to 1 completely stops the VIC-II. On older C64s, the screen
goes black. Should always be 0 for normal operation.

## Register $D018 — Memory Control Register

**Default: $15 (%00010101)**

| Bit | Mask | Default | Function |
|---|---|---|---|
| 0 | $01 | 1 | Unused |
| 1–3 | $0E | 4 (→$1000) | Character dot-data base address (offset in KB from VIC bank start) |
| 4–7 | $F0 | 1 (→$0400) | Video matrix base address (offset in KB from VIC bank start) |

### Character Data Address (bits 1–3)

These bits select the 2K offset within the VIC bank for character
dot-data. Only even values 0–14 are valid (step 2):

| Bits 1–3 | Offset | Address (Bank 0) | Content |
|---|---|---|---|
| 000 (0) | $0000 | $0000 | RAM data |
| 010 (2) | $0800 | $0800 | RAM data |
| 100 (4) | $1000 | $1000 | Character ROM (uppercase/graphics) |
| 110 (6) | $1800 | $1800 | Character ROM (lowercase/uppercase) |

Default value 4 ($1000) points to the Character ROM image that the
VIC-II sees in bank 0. To switch to lowercase, set bits 1–3 to 6:
`POKE 53272, PEEK(53272) OR 2`.

In **bitmap mode**, bit 3 selects the 8K bitmap location:
- Bit 3 = 0: bitmap at offset $0000 (first 8K of VIC bank)
- Bit 3 = 1: bitmap at offset $2000 (second 8K of VIC bank)

### Video Matrix Address (bits 4–7)

These bits select the 1K offset within the VIC bank for the screen
matrix (and sprite pointers). Values 0–15 map to offsets $0000–$F000
in 1K steps.

Default value 1 (bits 4–7 = 0001) places the screen matrix at offset
$0400, which is the default screen memory at $0400 in bank 0.

## Register $D019 — Interrupt Flag Register

**Default after clear: $0F (all latched interrupts cleared)**

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Raster compare IRQ flag |
| 1 | $02 | Sprite-to-foreground collision IRQ flag |
| 2 | $04 | Sprite-to-sprite collision IRQ flag |
| 3 | $08 | Light pen IRQ flag |
| 4–6 | $70 | Unused |
| 7 | $80 | Any IRQ: OR of bits 0–3 |

When any interrupt condition is met and the corresponding bit is set,
bit 7 is also set. The interrupt service routine reads this register
to determine the source.

**Clearing:** Writing a 1 to any bit **clears** that bit's latch.
This is an inverted logic from typical "write 0 to clear" patterns.
Writing $0F clears all VIC interrupt flags.

## Register $D01A — Interrupt Mask Register

**Default: $00 (all disabled)**

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Enable raster compare IRQ |
| 1 | $02 | Enable sprite-to-foreground collision IRQ |
| 2 | $04 | Enable sprite-to-sprite collision IRQ |
| 3 | $08 | Enable light pen IRQ |
| 4–7 | $F0 | Unused |

A VIC-II IRQ is only generated when both the flag bit in $D019 is set
**and** the corresponding mask bit in $D01A is set to 1.

## Light Pen Registers ($D013–$D014)

| Address | Name | Description |
|---|---|---|
| $D013 | LPENX | Light pen X position (in 2-pixel units, 0–140) |
| $D014 | LPENY | Light pen Y position (raster line, 0–255) |

### Latching Behavior

When the light pen trigger fires (fire button on Control Port 1
closes), the VIC-II latches the current raster position into these
registers. The values are updated once per frame. Subsequent trigger
closures during the same frame do not update the registers.

### X-Position Resolution

The horizontal position has half-resolution: the value in LPENX
must be multiplied by 2 to approximate the actual pixel column.
The range is approximately 0–140, corresponding to pixel columns
0–280.

### Y-Position Accuracy

The vertical position corresponds exactly to the current raster
line being scanned when the trigger fired.

### Accuracy Considerations

Light pen readings may be inaccurate due to:
- Sensor latency in the light pen hardware
- Timing delays in the trigger circuit
- User hand movement

Software should average multiple readings for stable positioning.

## Collision Registers

### $D01E — Sprite-to-Sprite Collision

| Bit | Mask | Function |
|---|---|---|
| 0–7 | $FF | Bit N = 1 if sprite N collided with another sprite |

### $D01F — Sprite-to-Foreground Collision

| Bit | Mask | Function |
|---|---|---|
| 0–7 | $FF | Bit N = 1 if sprite N collided with foreground graphics |

**Critical behavior:** Reading either register **clears** it. If
multiple tests are needed, copy the value to a RAM variable first.

**Collision rules:**
- Only **non-zero** data bits cause collisions. A sprite pixel with
  value 0 (transparent) cannot collide.
- In multicolor mode, the `01` bit-pair is considered background and
  **cannot** cause a collision.
- Collisions can occur in areas **covered by the screen border**.
- A sprite that is invisible (color = background color) but has non-zero
  data bits **can** still collide.