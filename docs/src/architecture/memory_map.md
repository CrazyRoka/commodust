# Memory Map

## Full 64K Address Space (Default Configuration)

| Address Range | Size | Label | Default Content |
|---|---|---|---|
| $0000–$00FF | 256 B | Page 0 | Zero page: 6510 I/O port, BASIC/Kernal variables, CHRGET routine |
| $0100–$01FF | 256 B | Page 1 | 6510 hardware stack |
| $0200–$0258 | 89 B | — | BASIC input buffer |
| $0259–$0262 | 10 B | LAT | Logical file number table |
| $0263–$026C | 10 B | FAT | Device number table |
| $026D–$0276 | 10 B | SAT | Secondary address table |
| $0277–$0280 | 10 B | KEYD | Keyboard buffer |
| $0281–$0284 | 4 B | — | O.S. memory bottom/top pointers |
| $0285–$0290 | 12 B | — | O.S. variables (color, cursor, keyboard) |
| $0291–$0297 | 7 B | — | Mode flags, scroll flag |
| $0293–$0297 | 5 B | — | RS-232 pseudo-6551 registers |
| $0298–$02A6 | 15 B | — | RS-232 variables, PAL/NTSC flag |
| $02A7–$02FF | 89 B | — | Unused (free for ML/graphics) |
| $0300–$030B | 12 B | — | BASIC indirect vector table |
| $030C–$030F | 4 B | — | Register storage for SYS |
| $0310–$0312 | 3 B | — | USR vector |
| $0314–$0315 | 2 B | CINV | IRQ vector |
| $0316–$0317 | 2 B | CBINV | BRK vector |
| $0318–$0319 | 2 B | NMINV | NMI vector |
| $031A–$0333 | 26 B | — | Kernal indirect vectors |
| $0334–$033B | 8 B | — | Unused (free for user vectors) |
| $033C–$03FB | 192 B | TBUFFER | Cassette I/O buffer |
| $03FC–$03FF | 4 B | — | Unused |
| $0400–$07E7 | 1000 B | — | Default screen matrix (video memory) |
| $07E8–$07FF | 24 B | — | Sprite shape data pointers (8 sprites × 3 bytes) |
| $0800–$9FFF | ~36 KB | — | BASIC program text, variables, arrays, string storage |
| $A000–$BFFF | 8 KB | — | BASIC ROM (or RAM if LORAM=0) |
| $C000–$CFFF | 4 KB | — | Free RAM (unused by system) |
| $D000–$D02E | 47 B | — | VIC-II registers |
| $D02F–$D03F | 17 B | — | VIC-II: not connected (read as $FF) |
| $D040–$D3FF | 960 B | — | VIC-II register images (mirrors every 64 bytes) |
| $D400–$D41C | 29 B | — | SID registers |
| $D41D–$D41F | 3 B | — | SID: not connected (read as $FF) |
| $D420–$D7FF | ~1 KB | — | SID register images (mirrors every 32 bytes) |
| $D800–$DBFF | 1 KB | — | Color RAM (only low 4 bits connected per byte) |
| $DC00–$DC0F | 16 B | — | CIA #1 registers |
| $DC10–$DCFF | 240 B | — | CIA #1 register images (mirrors every 16 bytes) |
| $DD00–$DD0F | 16 B | — | CIA #2 registers |
| $DD10–$DDFF | 240 B | — | CIA #2 register images (mirrors every 16 bytes) |
| $DE00–$DEFF | 256 B | — | I/O expansion slot 1 |
| $DF00–$DFFF | 256 B | — | I/O expansion slot 2 |
| $E000–$FFFF | 8 KB | — | Kernal ROM (or RAM if HIRAM=0) |
| $FFFA–$FFFB | 2 B | — | NMI vector (hardware, points to $FE43) |
| $FFFC–$FFFD | 2 B | — | RESET vector (hardware, points to $FCE2) |
| $FFFE–$FFFF | 2 B | — | IRQ/BRK vector (hardware, points to $FF48) |

## Mirror/Shadow Behavior

Several I/O devices do not fully decode their address lines, causing
**register mirroring**:

| Device | Actual Registers | Decoded Bits | Mirror Period |
|---|---|---|---|
| VIC-II | 47 registers ($D000–$D02E) | A0–A5 (6 bits) | Every 64 bytes within $D000–$D3FF |
| SID | 29 registers ($D400–$D41C) | A0–A4 (5 bits) | Every 32 bytes within $D400–$D7FF |
| CIA #1 | 16 registers ($DC00–$DC0F) | A0–A3 (4 bits) | Every 16 bytes within $DC00–$DCFF |
| CIA #2 | 16 registers ($DD00–$DD0F) | A0–A3 (4 bits) | Every 16 bytes within $DD00–$DDFF |

Unconnected addresses within a device's range always read as $FF and
ignore writes.

## Color RAM Quirk

Color RAM ($D800–$DBFF) is a dedicated 1K×4 static RAM. Only the
**lower 4 bits** of each byte are connected. Writing to the upper 4
bits has no effect, and reading returns random (or on some units,
constant) values in the upper nybble. Always mask with `AND $0F` when
reading Color RAM.

## Detailed Memory Map Overview

The following provides a short overview of the C64 memory map (pages and memory addresses) as seen by its CPU. The address space may look different from the view of other chips such as the VIC. This overview shows the status after power on of the C64 in the standard memory configuration ($37/55 in memory address $01, no cartridge). The memory management is implemented mostly by the C64 PLA.

### Configurations
The default configuration is for KERNAL ROM, I/O, BASIC ROM and the remaining RAM banks to be visible to the CPU. All configurations depend upon the state of latch bits set in the Programmable Logic Unit (PLA). The 7 distinct RAM banks are the smallest zones which can be bank switched.

### ROM vs RAM
If ROM is visible to the CPU during a write procedure, the ROM will be read but, any data is written to the underlying RAM. This principle is particularly significant to understanding how the I/O registers are addressed.

### Cartridge ROM
If cartridge ROM is present it can be located in up to three addressable locations. However, only two 8 kByte banks can be seen by the CPU at any time.

### BASIC RAM
The BASIC program storage space crosses the boundaries of RAM zones, sitting between $0800-$9FFF (38911 BASIC bytes).

### RAM Table

| Hex Address | Dec Address | Page | Contents |
|---|---|---|---|
| $0000-$00FF | 0-255 | Page 0 | Zeropage addressing |
| $0100-$01FF | 256-511 | Page 1 | Enhanced Zeropage contains the stack |
| $0200-$02FF | 512-767 | Page 2 | Operating System and BASIC pointers |
| $0300-$03FF | 768-1023 | Page 3 | Operating System and BASIC pointers |
| $0400-$07FF | 1024-2047 | Page 4-7 | Screen Memory |
| $0800-$9FFF | 2048-40959 | Page 8-159 | Free BASIC program storage area (38911 bytes) |
| $A000-$BFFF | 40960-49151 | Page 160-191 | Free machine language program storage area (when switched-out with ROM) |
| $C000-$CFFF | 49152-53247 | Page 192-207 | Free machine language program storage area |
| $D000-$D3FF | 53248-54271 | Page 208-211 | |
| $D400-$D7FF | 54272-54527 | Page 212-215 | |
| $D800-$DBFF | 55296-56319 | Page 216-219 | |
| $DC00-$DCFF | 56320-56575 | Page 220 | |
| $DD00-$DDFF | 56576-56831 | Page 221 | |
| $DE00-$DFFF | 56832-57343 | Page 222-223 | Reserved for interface extensions |
| $E000-$FFFF | 57344-65535 | Page 224-255 | Free machine language program storage area (when switched-out with ROM) |

### ROM Table

Cartridge ROM only becomes resident if attached to the expansion port on power-up. It is included for completeness as a record of the addresses it occupies as a ROM bank.

| Hex Address | Dec Address | Page | Contents |
|---|---|---|---|
| $8000-$9FFF | 32768-40959 | Page 128-159 | Cartridge ROM (low) |
| $A000-$BFFF | 40960-49151 | Page 160-191 | BASIC interpretor ROM or cartridge ROM (high) |
| $D000-$DFFF | 53248-57343 | Page 208-223 | Character generator ROM |
| $E000-$FFFF | 57344-65535 | Page 224-255 | KERNAL ROM or cartridge ROM (high) |

### I/O Table

| Hex Address | Dec Address | Page | Contents |
|---|---|---|---|
| $0000-$0001 | 0-1 | - | CPU I/O port - see Zeropage |
| $D000-$D3FF | 53248-54271 | Page 208-211 | VIC-II registers |
| $D400-$D7FF | 54272-55295 | Page 212-215 | SID registers |
| $D800-$DBFF | 55296-56319 | Page 216-219 | Color Memory |
| $DC00-$DCFF | 56320-56575 | Page 220 | CIA 1 |
| $DD00-$DDFF | 56576-56831 | Page 221 | CIA 2 |
| $DE00-$DEFF | 56832-57087 | Page 222 | I/O 1 |
| $DF00-$DFFF | 57088-57343 | Page 223 | I/O 2 |