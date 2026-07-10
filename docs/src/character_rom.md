# Character Generator ROM ($D000–$DFFF alt.)

The Character Generator ROM is a 4K ROM containing the dot-matrix
patterns for 512 character shapes (two sets of 256 characters each).
It is not directly addressable by the CPU in the default
configuration — the I/O block occupies $D000–$DFFF instead. It can
be made visible to the CPU by clearing CHAREN (bit 2 of $01), and
it is always visible to the VIC-II at specific offsets within banks
0 and 2.

## Accessing the ROM

### CPU Access (via CHAREN)

To read the Character ROM from the CPU:

1. Disable IRQ: `POKE 56333, 127` (prevents keyboard scan from
   crashing when I/O is unavailable).
2. Clear CHAREN: `POKE 1, PEEK(1) AND 251` (maps Character ROM at
   $D000–$DFFF instead of I/O).
3. Read data from $D000–$DFFF.
4. Restore CHAREN: `POKE 1, PEEK(1) OR 4`.
5. Re-enable IRQ: `POKE 56333, 129`.

### VIC-II Access (Automatic)

The VIC-II sees the Character ROM at offset $1000–$1FFF within
VIC-II banks 0 ($0000–$3FFF) and 2 ($8000–$BFFF). This overlay is
hardwired and cannot be disabled. In banks 1 and 3, the VIC-II sees
RAM at these offsets, and software must provide character data in
RAM.

## ROM Organization

The 4K ROM contains two 2K character sets:

| Offset | Set | Contents |
|---|---|---|
| $0000–$07FF | Set 1 | Uppercase + graphics (default) |
| $0800–$0FFF | Set 2 | Lowercase + uppercase |

Within each set, 256 characters are stored sequentially. Each
character occupies 8 bytes, representing 8 rows of 8 pixels each.

### Character Ordering

Characters are ordered by **screen code**, not PETASCII. Screen code 0
(@ symbol) is at offset $0000, screen code 1 (A) at $0008, screen
code 2 (B) at $0010, etc.

### Byte Format

Each byte represents one row of 8 pixels. Bit 7 is the leftmost
pixel; bit 0 is the rightmost. A set bit (1) indicates foreground
(foreground color); a clear bit (0) indicates background.

```
Example: Screen code 1 (letter "A"), Set 1
Offset   Byte    Binary     Visual
$0008    $1C     00011100       ███
$0009    $3E     00111110      █████
$000A    $66     01100110     ██  ██
$000B    $66     01100110     ██  ██
$000C    $7E     01111110     ██████
$000D    $66     01100110     ██  ██
$000E    $66     01100110     ██  ██
$000F    $00     00000000
```

## Relationship to $D018

The VIC-II Memory Control Register ($D018) bits 1–3 select which 2K
block within the current VIC-II bank is used for character dot-data.

| Bits 1–3 | Offset in Bank | Content (Bank 0/2) | Content (Bank 1/3) |
|---|---|---|---|
| 000 (0) | $0000 | RAM | RAM |
| 010 (2) | $0800 | RAM | RAM |
| 100 (4) | $1000 | Character ROM Set 1 | RAM |
| 110 (6) | $1800 | Character ROM Set 2 | RAM |

**Default:** $D018 = $15 → bits 1–3 = 100 → Character ROM Set 1
(uppercase/graphics) at offset $1000.

**Switch to lowercase:** Set bits 1–3 to 110 (offset $1800):
`POKE 53272, PEEK(53272) OR 2`

**Switch back:** `POKE 53272, PEEK(53272) AND 253`

## User-Defined Character Sets

Software can create custom character sets by:

1. Copying ROM character data to RAM (using the CHAREN technique
   above).
2. Modifying individual character definitions.
3. Pointing $D018 bits 1–3 to the RAM location of the custom set.

The custom set must be at a 2K boundary within the current VIC-II
bank, and must not overlap with the video matrix, bitmap, or sprite
data.

## Multicolor Text Mode and Character Data

In multicolor text mode (bit 4 of $D016 set), the character dot-data
is interpreted differently. Instead of each bit representing one
pixel, bits are grouped in pairs:

| Bit Pair | Color Source |
|---|---|
| 00 | Background Color Register 0 ($D021) |
| 01 | Background Color Register 1 ($D022) |
| 10 | Background Color Register 2 ($D023) |
| 11 | Color RAM nybble (lower 3 bits) |

This halves horizontal resolution (4 pixels per row instead of 8)
but allows 4 colors per character. Custom character sets designed
for multicolor mode use this bit-pair encoding.

## Emulation Notes

### VIC-II Character ROM Overlay

The emulator's VIC-II memory fetch function must implement a
separate address decoding path from the CPU. When the VIC-II reads
from offset $1000–$1FFF within banks 0 or 2, it should return data
from the Character ROM image, not from RAM. This is independent of
the CPU's CHAREN banking signal.

### ROM Image

The Character ROM is a 4K (4096-byte) image. The emulator should
load it from a ROM file (commonly named `chargen` or `charrom`).
The image contains both character sets back-to-back.

### CPU Access Model

When CHAREN=0 (and HIRAM=1 for the $D000 range), CPU reads from
$D000–$DFFF return Character ROM data. CPU writes to this range
always go to the underlying RAM (write-through behavior). This
allows software to copy ROM to RAM by reading ROM addresses and
writing to the same addresses.

### Kernal Default Table

The Kernal initializes VIC-II registers from a table at $ECB9. This
table sets $D018 to $15 (pointing to Character ROM Set 1) and other
defaults. The emulator need not model this table if it initializes
VIC-II registers directly, but it must produce the same result.