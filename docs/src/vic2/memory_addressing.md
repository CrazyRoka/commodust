# VIC-II Memory Addressing and Banking

The VIC-II has its own address bus to system RAM and sees memory differently
from the 6510 CPU. Understanding this dual-bus architecture is critical for
correct emulation, as it is the source of many common emulator bugs.

## VIC-II Address Space

The VIC-II can address only **16 KB** of memory at a time. The active 16K
bank is selected by bits 0–1 of CIA #2 Data Port A ($DD00).

### Bank Selection

| $DD00 Bits 1:0 | Bank | Address Range (CPU view) | Character ROM Visible? |
|---|---|---|---|
| 00 | 3 | $C000–$FFFF | No |
| 01 | 2 | $8000–$BFFF | Yes (at $9000–$9FFF) |
| 10 | 1 | $4000–$7FFF | No |
| 11 | 0 | $0000–$3FFF | Yes (at $1000–$1FFF) |

**Note the inverted logic:** bit pattern 11 = bank 0, 00 = bank 3. This is
because the VIC-II bank select bits are active-low.

**Default at power-on:** $DD00 = $3F (bits 0–1 = 11) → Bank 0 ($0000–$3FFF).

## Character ROM Overlay

The VIC-II sees the Character Generator ROM at offset $1000–$1FFF within
banks 0 and 2. This overlay is **hardwired** and cannot be disabled by
software.

| VIC-II Bank | VIC-II Address $1000–$1FFF | CPU Address $1000–$1FFF |
|---|---|---|
| 0 ($0000–$3FFF) | Character ROM | RAM (CPU sees RAM, not ROM) |
| 2 ($8000–$BFFF) | Character ROM | RAM (CPU sees RAM, not ROM) |
| 1 ($4000–$7FFF) | RAM | RAM |
| 3 ($C000–$FFFF) | RAM | RAM |

**Critical for emulation:** The VIC-II memory fetch function must have a
separate address decoding path from the CPU. When the VIC-II reads from
offset $1000–$1FFF within banks 0 or 2, it must return Character ROM data,
not RAM data. This is entirely independent of the CPU's CHAREN banking
signal.

## VIC-II Internal Memory Pointers

Within the selected 16K bank, the VIC-II uses $D018 (Memory Control
Register) to locate several data structures:

### Video Matrix (Screen Memory)

Selected by $D018 bits 4–7. The video matrix is 1K (1024 bytes: 1000 bytes
of screen codes + 24 bytes for 8 sprite pointers).

| $D018 Bits 4–7 | Offset in Bank | Bank 0 Address |
|---|---|---|
| 0000 (0) | $0000 | $0000 |
| 0001 (1) | $0400 | $0400 (default) |
| 0010 (2) | $0800 | $0800 |
| ... | ... | ... |
| 1111 (15) | $F000 | $F000 |

Offset = (bits 4–7) × 1024.

**Default:** $D018 = $15 → bits 4–7 = 0001 → offset $0400 → screen at $0400.

### Character Dot-Data

Selected by $D018 bits 1–3. The character data is 2K (2048 bytes = 256
characters × 8 bytes each).

| $D018 Bits 1–3 | Offset in Bank | Content (Bank 0/2) | Content (Bank 1/3) |
|---|---|---|---|
| 000 (0) | $0000 | RAM | RAM |
| 010 (2) | $0800 | RAM | RAM |
| 100 (4) | $1000 | Character ROM Set 1 | RAM |
| 110 (6) | $1800 | Character ROM Set 2 | RAM |

Only even values are valid (step of 2, since each set is 2K).

**Default:** $D018 = $15 → bits 1–3 = 100 → offset $1000 → Character ROM
Set 1 (uppercase/graphics).

### Bitmap Data (Bitmap Mode Only)

In bitmap mode ($D011 bit 5 = 1), the bitmap is 8K and its location is
determined by $D018 bit 3:

| $D018 Bit 3 | Bitmap Offset in Bank | Bank 0 Address |
|---|---|---|
| 0 | $0000 | $0000 |
| 1 | $2000 | $2000 |

The bitmap is always 8K (8000 bytes used, 8192 allocated). The color map
in bitmap mode is the 1K region pointed to by $D018 bits 4–7 (the same
pointer that selects the video matrix in text mode).

### Sprite Data

Sprite shape data is located via the sprite pointers at the end of the video
matrix (last 8 bytes). Each pointer is a single byte (0–255) that selects
one of 256 64-byte blocks within the current VIC-II bank.

Sprite data address = (VIC-II bank base) + (pointer value × 64).

Since the pointer is 8 bits and each block is 64 bytes, the 256 blocks
exactly cover the 16K bank. However, blocks that overlap with the Character
ROM overlay ($1000–$1FFF in banks 0/2) are inaccessible because the VIC-II
reads ROM data there, not RAM.

## VIC-II vs. CPU Memory Access Summary

| Feature | 6510 CPU | VIC-II |
|---|---|---|
| Addressable range | Full 64K ($0000–$FFFF) | 16K bank (selected by $DD00) |
| Character ROM visibility | Via CHAREN (bit 2 of $01) | Hardwired in banks 0, 2 |
| RAM under ROM | Always writes to RAM | Always reads RAM |
| Banking control | LORAM, HIRAM, CHAREN ($01) | VIC bank ($DD00 bits 0–1) |
| I/O registers | Visible when I/O banked | Not visible (VIC-II has own registers) |
| Color RAM | Visible at $D800–$DBFF (when I/O mapped) | Always accessible at fixed internal address |

## Write-Through to RAM

Regardless of banking, **all writes by the 6510 go to physical RAM**. When
ROM or I/O is mapped at a given address, a write still stores data in the
RAM "underneath." This is essential for VIC-II display because:

1. The CPU can write screen data, sprite data, or bitmap data to RAM
   addresses that are overlaid by ROM in the CPU's view.
2. The VIC-II reads from physical RAM and sees the written data.
3. This allows, for example, placing screen memory under the BASIC ROM
   ($A000–$BFFF) in VIC-II bank 2, where the CPU writes to RAM but reads
   ROM.

## Emulation Notes

### Dual Address Decoding

The emulator must implement two completely separate address decoding paths:

1. **CPU path:** Evaluates LORAM, HIRAM, CHAREN, GAME, EXROM for each
   access. Routes to RAM, ROM, or I/O.
2. **VIC-II path:** Evaluates only the VIC-II bank ($DD00 bits 0–1) and the
   Character ROM overlay. Always reads from RAM (or Character ROM at fixed
   offsets).

### Character ROM Overlay Implementation

The VIC-II fetch function should check:
```
IF (VIC_bank == 0 OR VIC_bank == 2) AND (offset >= $1000 AND offset <= $1FFF):
    RETURN Character_ROM[offset - $1000]
ELSE:
    RETURN RAM[VIC_bank_base + offset]
```

### Color RAM Access

The VIC-II always reads Color RAM from its fixed internal address
($D800–$DBFF equivalent), regardless of banking. The Color RAM is a separate
1K×4 chip not part of the 64K DRAM. The VIC-II accesses it directly; it does
not go through the PLA banking logic.

### Conceptual Considerations for Emulation

In a Rust emulator, the memory bus should expose two methods:
- `cpu_read(addr)` / `cpu_write(addr, val)` — goes through PLA/banking logic
- `vic_read(addr)` — bypasses CPU banking, applies only VIC-II bank and
  Character ROM overlay

This separation prevents the common bug where the VIC-II accidentally reads
ROM or I/O data instead of the underlying RAM. The VIC-II and CPU should
share the same underlying 64K RAM array, but with different decoding logic
on top.

### Sprite Pointer Calculation

Sprite pointers are always at the last 8 bytes of the video matrix,
regardless of where the video matrix is located. If the video matrix is at
offset $0400, sprite pointers are at $07F8–$07FF. The pointer value × 64
gives the offset within the VIC-II bank (not the CPU address) where sprite
shape data begins.