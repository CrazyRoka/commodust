# System Architecture Overview

## Bus Topology

The Commodore 64 is built around a shared address/data bus connecting
the 6510 CPU to 64 KB of DRAM, multiple ROMs, and several custom
integrated circuits. The VIC-II acts as the bus master, generating
the system clock (ø2), DRAM RAS/CAS signals, and controlling bus
arbitration via the BA (Bus Available) and AEC (Address Enable Control)
lines.

```
         ┌──────────┐
         │  6510    │
         │  CPU     │
         └────┬─────┘
              │ Address/Data/Control Bus
              │
    ┌─────────┼─────────────────┐
    │         │                 │
    ▼         ▼                 ▼
┌────────┐ ┌──────────┐  ┌───────────┐
│ 64K    │ │ VIC-II   │  │ I/O Logic │
│ DRAM   │ │ (video)  │  │ PLA       │
└────────┘ └──────────┘  └───────────┘
    │           │              │
    │           ▼              ▼
    │     ┌──────────┐  ┌───────────┐
    │     │ Character│  │ SID       │
    │     │ ROM      │  │ (audio)   │
    │     └──────────┘  └───────────┘
    │                       │
    │              ┌────────┴────────┐
    │              │                 │
    │              ▼                 ▼
    │        ┌──────────┐     ┌───────────┐
    │        │ CIA #1   │     │ CIA #2    │
    │        │(kbd/joy) │     │(serial/   │
    │        │          │     │ RS232/    │
    │        │          │     │ VIC bank) │
    │        └──────────┘     └───────────┘
    │
    ▼
┌──────────────────────────────┐
│ ROM: BASIC, Kernal, Char     │
│ (selected by PLA logic)      │
└──────────────────────────────┘
```

## Address Space Partitioning

The 6510 sees a 64 KB address space ($0000–$FFFF). At any given moment,
each region of that space is mapped to one of: physical RAM, ROM, or
memory-mapped I/O. The mapping is determined by a combination of:

1. **6510 I/O Port** (locations $00 and $01) — software-controlled
   LORAM, HIRAM, and CHAREN signals.
2. **Expansion port signals** — hardware-controlled GAME and EXROM lines
   from cartridges.
3. **PLA (Programmable Logic Array)** — combines the above signals to
   select which device responds to a given address range.

## Open Bus Behavior

The PLA does not fully decode all address ranges. Unconnected address ranges exhibit specific open-bus behavior:

- **$DE00–$DFFF (I/O Expansion):** A read from these unconnected addresses does not return random data. On many C64s, the data bus retains the last byte read by the VIC-II during the first phase of the clock cycle. This allows software to measure VIC-II timing by reading the $DE00 area.
- **$D800–$DBFF (Color RAM Upper Nybble):** The Color RAM is only 4 bits wide. The upper 4 bits are open. Similar to the $DE00 area, they often read back the upper bits of the last byte the VIC-II fetched.
- **RAM under I/O Port ($00–$01):** The 6510 I/O port overlays physical RAM at $00–$01. When the CPU writes to $00 or $01, the data bus drivers for the internal I/O port go into tri-state, but the R/W line is set low. If the VIC-II reads the desired value from RAM $00 or $01 during the first clock phase, the CPU's write cycle in the second phase will successfully write that value to the underlying physical RAM.

### Default Power-On Memory Configuration

With no cartridge inserted and default I/O port settings ($01 = %00110111,
i.e., LORAM=1, HIRAM=1, CHAREN=1):

| Address Range | Size | Content |
|---|---|---|
| $0000–$3FFF | 16 KB | RAM (system variables, screen, BASIC text) |
| $4000–$7FFF | 16 KB | RAM (BASIC program/variables continued) |
| $8000–$9FFF | 8 KB | RAM (or cartridge ROM if present) |
| $A000–$BFFF | 8 KB | BASIC ROM |
| $C000–$CFFF | 4 KB | RAM (unused by system, free for ML) |
| $D000–$DFFF | 4 KB | I/O: VIC-II, SID, CIAs, Color RAM |
| $E000–$FFFF | 8 KB | Kernal ROM |

## VIC-II vs. 6510 View

The VIC-II chip has its own address bus to RAM and sees memory
**differently** from the 6510:

- The VIC-II can only address **16 KB** at a time, selected by bits 0–1
  of CIA #2 Data Port A ($DD00). This selects one of four 16 KB "banks."
- Within the selected bank, the VIC-II sees the **Character ROM** at
  offsets $1000–$1FFF in banks 0 and 2, even though the 6510 cannot
  read it there (the 6510 sees RAM at those addresses).
- The VIC-II always reads from **physical RAM**, never from ROM. This
  means data written by the 6510 to RAM "underneath" the BASIC or
  Kernal ROM is visible to the VIC-II.
- Writes by the 6510 always go to RAM, even when ROM is mapped at the
  same address.

## Bus Sharing and Timing

The VIC-II and 6510 share the data bus. The VIC-II fetches most of its
data (screen codes, character dot data, sprite data) during the
"off" half of the 6510's cycle. However, certain operations require
the VIC-II to use the bus during the 6510's active half:

- **Bad lines**: Every 8th scan line (when the raster line is a multiple
  of 8 and within the visible display), the VIC-II must fetch 40 bytes
  of screen codes and 40 bytes of color data. This requires **stealing**
  40 bus cycles from the 6510, during which the CPU is stalled.
- **Sprite DMA**: Each enabled sprite requires additional bus cycles
  for data fetch. More sprites = more CPU stall cycles.
- **Screen blanking**: When Bit 4 of $D011 is cleared (screen blanked),
  the VIC-II does not perform character/sprite fetches, and the 6510
  runs at full speed (~7% faster than with display enabled).

This bus-sharing behavior is critical for cycle-accurate emulation.