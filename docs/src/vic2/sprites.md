# Sprite Graphics

The VIC-II provides 8 independent hardware sprites. Each sprite is a 24×21
pixel graphics object that can be positioned anywhere on the screen,
independently of the text or bitmap display. Sprites have their own color,
priority, expansion, and collision detection.

## Sprite Capabilities

| Feature | Specification |
|---|---|
| Number of sprites | 8 (Sprite 0–7) |
| Sprite size | 24×21 pixels |
| Horizontal positions | 0–511 (9-bit, via MSB register) |
| Vertical positions | 0–255 |
| Colors per sprite | 1 (hi-res) or 3 (multicolor) |
| Expansion | 2× horizontal, 2× vertical (independent) |
| Priority | Per-sprite: in front of or behind foreground |
| Collision detection | Sprite-to-sprite, sprite-to-foreground |

## Sprite Position Registers

| Address | Register | Sprite | Function |
|---|---|---|---|
| $D000 | SP0X | 0 | Horizontal position (low 8 bits) |
| $D001 | SP0Y | 0 | Vertical position |
| $D002 | SP1X | 1 | Horizontal position (low 8 bits) |
| $D003 | SP1Y | 1 | Vertical position |
| $D004 | SP2X | 2 | Horizontal position (low 8 bits) |
| $D005 | SP2Y | 2 | Vertical position |
| $D006 | SP3X | 3 | Horizontal position (low 8 bits) |
| $D007 | SP3Y | 3 | Vertical position |
| $D008 | SP4X | 4 | Horizontal position (low 8 bits) |
| $D009 | SP4Y | 4 | Vertical position |
| $D00A | SP5X | 5 | Horizontal position (low 8 bits) |
| $D00B | SP5Y | 5 | Vertical position |
| $D00C | SP6X | 6 | Horizontal position (low 8 bits) |
| $D00D | SP6Y | 6 | Vertical position |
| $D00E | SP7X | 7 | Horizontal position (low 8 bits) |
| $D00F | SP7Y | 7 | Vertical position |

**Default:** All $00.

### Horizontal Position MSB ($D010)

| Bit | Sprite | Function |
|---|---|---|
| 0 | 0 | MSB of Sprite 0 horizontal position |
| 1 | 1 | MSB of Sprite 1 horizontal position |
| 2 | 2 | MSB of Sprite 2 horizontal position |
| 3 | 3 | MSB of Sprite 3 horizontal position |
| 4 | 4 | MSB of Sprite 4 horizontal position |
| 5 | 5 | MSB of Sprite 5 horizontal position |
| 6 | 6 | MSB of Sprite 6 horizontal position |
| 7 | 7 | MSB of Sprite 7 horizontal position |

Setting a bit to 1 adds 256 to the corresponding sprite's horizontal
position, extending the range to 0–511.

Full horizontal position: `X = SPxX + (MSIGX bit × 256)`.

### Visible Position Ranges

| Direction | Fully Off-Screen | Fully On-Screen |
|---|---|---|
| Left | X = 0 | X = 24 |
| Right | X = 344 | X = 321 (rightmost pixel exits) |
| Top | Y ≤ 29 | Y = 50 |
| Bottom | Y = 250 | Y = 230 (bottom pixel exits) |

The visible display area is horizontal positions 24–343 and vertical
positions 50–249.

## Sprite Enable Register ($D015)

**Default:** $00 (all sprites disabled).

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = sprite enabled, 0 = sprite disabled |

A sprite will only be displayed if:
1. Its enable bit is set in $D015
2. Its data pointer points to non-zero shape data
3. Its position is within the visible screen range
4. Its color is different from the background (for visible sprites)

## Sprite Shape Data

Each sprite is 24×21 pixels = 504 bits = 63 bytes. The 64th byte is unused
padding to align to 64-byte blocks.

### Data Format

Each row of the sprite is 3 bytes (24 bits). Bit 7 of the first byte is the
leftmost pixel; bit 0 of the third byte is the rightmost pixel.

```
Row 0:  Byte 0 (bits 7-0)  Byte 1 (bits 7-0)  Byte 2 (bits 7-0)
Row 1:  Byte 3             Byte 4             Byte 5
...
Row 20: Byte 60            Byte 61            Byte 62
        Byte 63 (unused)
```

### Sprite Data Pointers

The last 8 bytes of the video matrix serve as sprite data pointers.

| Address (default) | Sprite | Default Value |
|---|---|---|
| $07F8 | 0 | $00 (cleared by RAMTAS) |
| $07F9 | 1 | $00 |
| $07FA | 2 | $00 |
| $07FB | 3 | $00 |
| $07FC | 4 | $00 |
| $07FD | 5 | $00 |
| $07FE | 6 | $00 |
| $07FF | 7 | $00 |

Each pointer holds a block number (0–255). The sprite data starts at:
`SPRITE_DATA_ADDRESS = VIC_BANK_BASE + (POINTER_VALUE × 64)`

Multiple sprites can share the same data pointer, allowing 8 sprites with
the same shape to use only 64 bytes of shape data.

## Sprite Color Registers

| Address | Register | Sprite | Default |
|---|---|---|---|
| $D027 | SP0COL | 0 | $01 (white) |
| $D028 | SP1COL | 1 | $02 (red) |
| $D029 | SP2COL | 2 | $03 (cyan) |
| $D02A | SP3COL | 3 | $04 (purple) |
| $D02B | SP4COL | 4 | $05 (green) |
| $D02C | SP5COL | 5 | $06 (blue) |
| $D02D | SP6COL | 6 | $07 (yellow) |
| $D02E | SP7COL | 7 | $0C (medium gray) |

Only the lower 4 bits are connected (upper 4 bits read as random/undefined).

## Sprite Expansion Registers

### $D017 — Vertical Expansion (YXPAND)

**Default:** $00.

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = double height, 0 = normal height |

When enabled, each pixel row of the sprite is displayed as 2 scan lines,
doubling the sprite height from 21 to 42 pixels.

### $D01D — Horizontal Expansion (XXPAND)

**Default:** $00.

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = double width, 0 = normal width |

When enabled, each pixel column is displayed as 2 pixels, doubling the
sprite width from 24 to 48 pixels.

Horizontal and vertical expansion can be combined for a 4× area sprite
(48×42 pixels).

## Sprite Priority Register ($D01B)

**Default:** $00 (all sprites in front of foreground).

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 0 = sprite in front of foreground; 1 = foreground in front of sprite |

### Inter-Sprite Priority

Between sprites, there is a fixed priority: Sprite 0 has highest priority
(displays in front), Sprite 7 has lowest. When two overlapping sprites both
have priority over the foreground, the lower-numbered sprite is displayed
in front.

### Multicolor Background Consideration

In multicolor text or bitmap mode, the `01` bit-pair is considered
background. A sprite with foreground priority will appear in front of `01`
pixels, even if the sprite's priority bit is set to "behind foreground."

## Sprite Multicolor Mode ($D01C)

**Default:** $00 (all sprites hi-res).

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = multicolor mode, 0 = hi-res mode |

### Multicolor Sprite Color Assignment

| Bit Pair | Color Source |
|---|---|
| 00 | Background Color Register 0 ($D021) — transparent |
| 01 | Sprite Multicolor Register 0 ($D025) |
| 10 | Sprite Color Register (SPxCOL) |
| 11 | Sprite Multicolor Register 1 ($D026) |

**Default multicolor register values:**
- $D025 (SPMC0): $04 (purple)
- $D026 (SPMC1): $00 (black)

In multicolor mode, the sprite is 12 double-width pixels wide (same visual
size, half horizontal resolution). All multicolor sprites share the two
common multicolor registers; only the `10` bit-pair color is unique per
sprite.

## Collision Detection Registers

### $D01E — Sprite-to-Sprite Collision (SPSPCL)

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = sprite N collided with another sprite |

### $D01F — Sprite-to-Foreground Collision (SPBGCL)

| Bit | Sprite | Function |
|---|---|---|
| 0–7 | 0–7 | 1 = sprite N collided with foreground graphics |

### Collision Rules

1. **Only non-zero data bits cause collisions.** A sprite pixel with value 0
   (transparent) cannot collide. A foreground pixel that is the background
   color (bit = 0 in standard mode) cannot collide.
2. **The `01` bit-pair in multicolor mode is considered background** and
   cannot cause a collision.
3. **Collisions can occur under the screen border.** Sprites that overlap
   in the border area (covered by 38-column or 24-row mode) can still
   register collisions.
4. **An invisible sprite (color = background) with non-zero data can still
   cause collisions.** Invisibility does not prevent collision detection.
5. **Reading either collision register clears it.** If multiple tests are
   needed, copy the value to RAM first.

### Collision Interrupts

Collision events can trigger VIC-II interrupts:
- Bit 1 of $D019 / $D01A: Sprite-to-foreground collision
- Bit 2 of $D019 / $D01A: Sprite-to-sprite collision

## Emulation Notes

### Sprite DMA and Bus Stealing

Each enabled sprite requires additional bus cycles for data fetch. The
VIC-II steals these cycles from the CPU:

| Sprites Enabled | Cycles Stolen per Line |
|---|---|
| 0 | 0 |
| 1 | 3 |
| 2 | 6 |
| ... | ... |
| 8 | 24 (maximum) |

The sprite data is fetched during the horizontal blanking period and during
cycles stolen from the CPU. Each sprite requires 3 memory accesses per
scan line (2 for data, 1 for the pointer), plus an additional access if
the sprite is being displayed.

### Sprite Display Timing

A sprite is displayed on a scan line when:
1. The sprite is enabled ($D015)
2. The sprite's vertical position ≤ current raster line < vertical position
   + sprite height (21, or 42 if vertically expanded)
3. The sprite's horizontal position falls within the visible range for that
   scan line

The VIC-II checks sprite enable and vertical position at the start of each
scan line to determine which sprites need DMA. Horizontal position is
checked pixel-by-pixel during the display.

### Expansion Timing Quirk

When a sprite is horizontally expanded, the VIC-II fetches each data byte
and holds it for 2 pixels. The data fetch timing is not changed by
expansion — only the display duration of each pixel is doubled. This means
the expanded sprite's data is still fetched at the same rate as an
unexpanded sprite.

### Collision Detection Precision

Collision detection should be performed at the pixel level. For each pixel
position where a sprite has a non-zero data bit, check if:
- Another sprite also has a non-zero data bit at the same position
(sprite-to-sprite collision)
- The foreground graphics (text or bitmap) has a non-zero data bit at the
  same position (sprite-to-foreground collision)

The collision registers are set as soon as any collision is detected and
remain set until read.

### Collision and Sprite Data

A collision is registered only when a sprite pixel with a **non-zero
data bit** overlaps another non-zero pixel (sprite or foreground).

**Key behaviors:**

1. A sprite with all-zero shape data can never cause a collision,
   even if enabled and positioned on screen.

2. A sprite whose color matches the background (making it invisible)
   can still cause collisions, as long as its shape data contains
   non-zero bits.

3. In multicolor mode, the `01` bit-pair is considered background
   and cannot cause collisions — even though it displays a color.
   Only `10` and `11` bit-pairs (and `01` in hi-res mode, which is
   just bit 1) can cause collisions.

4. Foreground data bits that are 0 (background color in standard
   mode) cannot cause sprite-to-foreground collisions.

5. Collisions can occur in areas covered by the screen border
   (when display is reduced to 38 columns or 24 rows). The collision
   detection operates on the full 320×200 pixel area, not just the
   visible portion.

### Conceptual Considerations for Emulation

In a Rust emulator, each sprite can be modeled as a struct containing its
position, enable flag, color, expansion flags, priority, and multicolor
mode. The sprite rendering pass occurs per scan line, after the main
display has been rendered. For each active sprite on the current line:
1. Fetch the 3 bytes of sprite data for the current row
2. For each of the 24 (or 12 in multicolor) pixels, check if the pixel is
   non-zero
3. If non-zero, check for collisions with the framebuffer and other sprites
4. Write the pixel to the framebuffer if priority allows

The collision registers should be updated during the sprite rendering pass,
not after it, to ensure correct timing relative to interrupt generation.