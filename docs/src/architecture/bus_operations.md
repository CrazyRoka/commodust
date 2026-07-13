# Bus Operations

This page consolidates all bus-related behavior into a single
reference: bus masters, signals, access types, address decoding
paths, arbitration, refresh, and open-bus quirks.

## Bus Topology

The C64 is built around a shared address/data bus connecting the
6510 CPU to 64 KB of DRAM, multiple ROMs, and several custom
integrated circuits. The VIC-II acts as the bus master, generating
the system clock, DRAM RAS/CAS signals, and controlling bus
arbitration via the BA and AEC lines.

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

## Bus Masters

The system has two bus masters that share the address and data bus
to DRAM:

### 6510 CPU

| Property | Value |
|---|---|
| Address bus | 16-bit (A0–A15), full 64 KB |
| Data bus | 8-bit (D0–D7) |
| Bus access phase | ø2 high (second clock phase) |
| Stall control | RDY pin (connected to VIC-II BA) |
| Bus release | AEC pin (tri-states address bus when low) |

The CPU accesses the bus only when ø2 is high. It can be stalled by
pulling RDY low, but it ignores RDY during write cycles (it can
complete up to 3 consecutive writes before stopping).

### VIC-II

| Property | Value |
|---|---|
| Address bus | 14-bit (A0–A13), 16 KB bank at a time |
| Data bus | 12-bit (D0–D7 to RAM/CPU; D8–D11 to Color RAM only) |
| Bus access phase | ø2 low (first clock phase) — normal operation |
| Bus access phase | ø2 high — during Bad Lines and sprite DMA (stolen cycles) |
| Role | Bus master; generates system clock and DRAM timing |

The VIC-II generates ø0 (system clock) from øIN (pixel clock).
ø0 is fed into the 6510, which derives ø2 from it. The VIC-II
normally performs its memory fetches during ø2 low (the CPU's
"off" half), but steals additional cycles during ø2 high when
needed.

**Address bus multiplexing:** A0–A5 and A8–A13 are multiplexed
(for DRAM RAS/CAS). A6–A11 are on separate lines.

> **TODO:** Document the exact DRAM RAS/CAS multiplexing sequence
> and how VIC-II address bits map to DRAM row/column addresses.

## Clock Signals

| Signal | Source | Frequency (NTSC) | Frequency (PAL) | Description |
|---|---|---|---|---|
| øIN | Crystal | 8.18 MHz | 7.88 MHz | Pixel clock input to VIC-II |
| ø0 | VIC-II (øIN ÷ 8) | 1.023 MHz | 0.985 MHz | System clock, fed to 6510 |
| ø2 | 6510 (derived from ø0) | 1.023 MHz | 0.985 MHz | Phase 2 clock; CPU accesses bus when high |

| Clock Phase | Bus Owner (normal) | Bus Owner (bad line / sprite DMA) |
|---|---|---|
| ø2 low (first phase) | VIC-II | VIC-II |
| ø2 high (second phase) | 6510 CPU | VIC-II (CPU stalled via BA/AEC) |

## Bus Arbitration Signals

### BA (Bus Available)

- Connected to 6510 RDY pin.
- VIC-II pulls BA low to stop the CPU.
- BA goes low **3 cycles** before the VIC-II needs the bus.
- The 3-cycle delay ensures the 6510 can finish any sequence of
  consecutive writes (maximum 3 consecutive write cycles), since
  the 6510 ignores RDY during write cycles.
- During the 3-cycle BA-low-but-AEC-high window, the CPU can still
  perform writes but stops on reads.

### AEC (Address Enable Control)

- Tri-states the 6510's address bus drivers.
- AEC normally follows ø2: low during ø2 low, high during ø2 high.
- When the VIC-II steals cycles, AEC **remains low** during ø2 high,
  disabling the 6510's address bus so the VIC-II can drive the bus.

### Take-Over Sequence

```
       _ _ _ _ _ _ _ _ _ _ _ _ _    _ _ _ _ _ _ _ _ _ _ _ _ _
 ø2   _ _ _ _ _ _ _ _ _ _ _ _ _ ..._ _ _ _ _ _ _ _ _ _ _ _ _
      ______________                       __________________
 BA                 ____________...________
       _ _ _ _ _ _ _ _ _ _                  _ _ _ _ _ _ _ _ _
 AEC  _ _ _ _ _ _ _ _ _ _ ______..._________ _ _ _ _ _ _ _ _ _

 Chip VPVPVPVPVPVPVPVpVpVpVVVVVV...VVVVVVVVVPVPVPVPVPVPVPVP

           1       |  2  |       3        |       4
         Normal    |Take-| VIC has taken  |  VIC releases
      bus activity |over | over the bus   |    the bus
```

Legend: `V` = VIC-II access, `P` = 6510 access, `p` = 6510 write
access only (stops on read).

**Phase 1 — Normal:** Both CPU and VIC-II alternate bus access
normally.

**Phase 2 — Take-over (3 cycles):** BA is pulled low. CPU can
still write but cannot read. AEC is still following ø2.

**Phase 3 — VIC-II has the bus:** AEC stays low during ø2 high.
CPU address bus is tri-stated. VIC-II performs its reads during
what would normally be CPU time.

**Phase 4 — Release:** BA and AEC return to normal. CPU resumes.

## Bus Access Types

The VIC-II performs 6 different types of memory accesses during a
raster line:

| Type | Name | Width | Description |
|---|---|---|---|
| c-access | Video matrix | 12 bits | 8 bits from RAM (screen code), 4 bits from Color RAM (color nybble) via D8–D11 |
| g-access | Graphics | 8 bits | Character generator or bitmap data from RAM |
| p-access | Sprite pointer | 8 bits | Sprite data pointer from end of video matrix |
| s-access | Sprite data | 8 bits | Sprite shape data from RAM |
| r-access | DRAM refresh | 8 bits | 5 refresh reads per line; address derived from 8-bit REF counter |
| i-access | Idle | 8 bits | Read from $3FFF (or $39FF if ECM set); result discarded |

### c-Access Detail

During a c-access, the VIC-II reads **two memory sources
simultaneously** on a single bus cycle:
- D0–D7: screen code byte from RAM (pointed to by video matrix
  address)
- D8–D11: color nybble from Color RAM (fixed 1K×4 chip, always
  accessible by VIC-II)

The 12-bit result combines the screen code (bits 0–7) and color
nybble (bits 8–11).

### r-Access (DRAM Refresh) Detail

| Property | Value |
|---|---|
| Counter | 8-bit REF, reset to $FF at raster line 0 |
| Accesses per line | 5 |
| Address format | Bits 13–6 = `111111`, bits 5–0 = REF[7:0] |
| Phase | ø2 low (first phase); does not interfere with CPU |

Line 0 accesses: $3FFF, $3FFE, $3FFD, $3FFC, $3FFB.
Line 1 accesses: $3FFA down to $3FF6, etc.

### i-Access (Idle) Detail

When the VIC-II has no useful data to fetch, it performs an idle
read to $3FFF (or $39FF if ECM is set). The result is discarded.
This keeps the bus active and maintains timing.

## Raster Line Access Sequence

The sequence of VIC-II memory accesses within a raster line is
hard-wired, independent of graphics mode, and identical for every
raster line (with exceptions for sprite DMA).

### VIC-II Revision Timing

| Parameter | 6567R56A (NTSC) | 6567R8 (NTSC) | 6569 (PAL) |
|---|---|---|---|
| Total lines per frame | 262 | 263 | 312 |
| Visible lines | 234 | 235 | 284 |
| Cycles per line | 64 | 65 | 63 |
| Visible pixels per line | 411 | 418 | 403 |
| First VBlank line | 13 | 13 | 300 |
| Last VBlank line | 40 | 40 | 15 |

### PAL (6569) Bad Line, No Sprites — Timing Diagram

The following diagram shows the access pattern for a Bad Line with
no sprites active on a PAL system (63 cycles per line):

```
   VIC i _ i _ i _ i _ i _ i r r r r r c g c g c g ... c g i i _ i _ i _ i _
  6510  x x x x x x x x x x x x X X X                                x x x x x x x x x x
```

- Cycles before ~15: i-accesses (idle) and r-accesses (5 refresh)
- Cycles 15–54: alternating c-access and g-access (40 each, one
  pair per cycle)
- Cycles after ~54: i-accesses (idle)

The `X` marks in the 6510 row indicate cycles where the CPU is
stalled (BA low, AEC low).

> **TODO:** The numbers 3, 4, 5, 6, 7 appearing between i-accesses
> in the original timing diagram are not explained in the source
> documentation. Their meaning needs clarification.

> **TODO:** Provide NTSC (6567R8, 65 cycles) and 6567R56A
> (64 cycles) timing diagrams. Only the PAL diagram is available
> in the current documentation.

> **TODO:** Document the exact cycle numbers for p-access (sprite
> pointer) and s-access (sprite data) slots. The docs state these
> are "statically assigned slots" but do not list the cycle numbers.

## CPU Bus Path: PLA and Banking

### PLA (Programmable Logic Array)

The PLA is the central address decoding component for CPU bus
accesses. It combines multiple signals to select which device
responds to a given address:

**Inputs:**
- 16-bit CPU address (A0–A15)
- LORAM (bit 0 of $01)
- HIRAM (bit 1 of $01)
- CHAREN (bit 2 of $01)
- GAME (expansion port)
- EXROM (expansion port)

**Output:** device select (RAM, BASIC ROM, Kernal ROM, Character
ROM, I/O block, cartridge ROM)

The PLA logic can be modeled as a pure function:

```
device_select = f(address, loram, hiram, charen, game, exrom)
```

> **TODO:** Document the complete PLA equations or full truth table
> for all address ranges (not just $A000–$FFFF). The current
> documentation only provides the truth table for $A000–$BFFF,
> $D000–$DFFF, and $E000–$FFFF with no cartridge.

### Banking Signals

The three software-controlled banking signals come from the 6510
on-chip I/O port at $01:

| Signal | Source | Default | Controls |
|---|---|---|---|
| LORAM | $01 bit 0 | 1 | $A000–$BFFF: 1 = BASIC ROM, 0 = RAM |
| HIRAM | $01 bit 1 | 1 | $E000–$FFFF: 1 = Kernal ROM, 0 = RAM |
| CHAREN | $01 bit 2 | 1 | $D000–$DFFF: 1 = I/O, 0 = Character ROM |

**Key rule:** CHAREN only has effect when HIRAM = 1. When HIRAM = 0,
the $D000 region is always I/O regardless of CHAREN, because the
CHAREN signal is gated through the Kernal ROM select logic.

The two hardware-controlled signals come from the expansion port:

| Signal | Source | Default (no cart) |
|---|---|---|
| GAME | Cartridge | 1 (pulled high) |
| EXROM | Cartridge | 1 (pulled high) |

### Banking Truth Table (No Cartridge: GAME=1, EXROM=1)

| LORAM | HIRAM | CHAREN | $A000–$BFFF | $D000–$DFFF | $E000–$FFFF |
|---|---|---|---|---|---|
| 1 | 1 | 1 | BASIC ROM | I/O | Kernal ROM |
| 1 | 1 | 0 | BASIC ROM | Character ROM | Kernal ROM |
| 1 | 0 | 1 | BASIC ROM | I/O | RAM |
| 1 | 0 | 0 | BASIC ROM | Character ROM | RAM |
| 0 | 1 | 1 | RAM | I/O | Kernal ROM |
| 0 | 1 | 0 | RAM | Character ROM | Kernal ROM |
| 0 | 0 | 1 | RAM | I/O | RAM |
| 0 | 0 | 0 | RAM | Character ROM | RAM |

### Cartridge Configurations

| Cartridge Type | GAME | EXROM | Effect |
|---|---|---|---|
| 8K ROM at $8000 | 1 | 0 | ROM at $8000–$9FFF |
| 8K ROM at $A000 | 0 | 1 | ROM at $A000–$BFFF (replaces BASIC) |
| 16K ROM at $8000 | 0 | 0 | ROM at $8000–$BFFF |
| Ultimax mode | 0 | 1 | ROM at $E000–$FFFF; only 6K RAM used |

> **TODO:** Document the full banking truth table for all cartridge
> configurations (all GAME/EXROM combinations), not just the
> no-cartridge case.

### Write-Through to RAM

Regardless of banking, **all writes by the 6510 go to physical
RAM**. When ROM or I/O is mapped at an address, a write still
stores data in the RAM "underneath." This is fundamental to the
C64's address decoding logic and is essential for:

1. **Copying ROM to RAM:** Software can read ROM addresses and
   write to the same addresses to copy ROM into RAM.
2. **VIC-II display:** The VIC-II always reads from physical RAM,
   so data written to RAM-under-ROM is visible to the video chip
   even when the 6510 sees ROM at those addresses.

### I/O Block Internal Decoding

When the I/O block is mapped at $D000–$DFFF, individual devices
are selected by partial address decoding:

| Range | Device | Decoded Bits | Mirror Period |
|---|---|---|---|
| $D000–$D02E | VIC-II (47 registers) | A0–A5 (6 bits) | Every 64 bytes within $D000–$D3FF |
| $D040–$D3FF | VIC-II mirrors | — | — |
| $D400–$D41C | SID (29 registers) | A0–A4 (5 bits) | Every 32 bytes within $D400–$D7FF |
| $D41D–$D41F | SID unconnected | — | Read as $FF |
| $D420–$D7FF | SID mirrors | — | — |
| $D800–$DBFF | Color RAM (1K×4) | Full decode | No mirrors |
| $DC00–$DC0F | CIA #1 (16 registers) | A0–A3 (4 bits) | Every 16 bytes within $DC00–$DCFF |
| $DD00–$DD0F | CIA #2 (16 registers) | A0–A3 (4 bits) | Every 16 bytes within $DD00–$DDFF |
| $DE00–$DEFF | I/O expansion slot 1 | — | — |
| $DF00–$DFFF | I/O expansion slot 2 | — | — |

Unconnected addresses within a device's range always read as $FF
and ignore writes.

> **TODO:** Document how the PLA further decodes the I/O block into
> individual device selects (VIC-II vs SID vs CIA vs Color RAM vs
> expansion). The exact decode logic is not provided.

## VIC-II Bus Path

The VIC-II has a **completely separate** address decoding path
from the CPU. It does not go through the PLA banking logic.

### VIC-II Bank Selection

The VIC-II can address only 16 KB at a time, selected by bits 0–1
of CIA #2 Data Port A ($DD00):

| $DD00 Bits 1:0 | Bank | Address Range (CPU view) | Character ROM Visible? |
|---|---|---|---|
| 00 | 3 | $C000–$FFFF | No |
| 01 | 2 | $8000–$BFFF | Yes (at $9000–$9FFF) |
| 10 | 1 | $4000–$7FFF | No |
| 11 | 0 | $0000–$3FFF (default) | Yes (at $1000–$1FFF) |

Note the inverted logic: bit pattern 11 = bank 0, 00 = bank 3.

### Character ROM Overlay

The VIC-II sees the Character Generator ROM at offset $1000–$1FFF
within banks 0 and 2. This overlay is **hardwired** and cannot be
disabled by software.

| VIC-II Bank | VIC-II Address $1000–$1FFF | CPU Address Same Range |
|---|---|---|
| 0 ($0000–$3FFF) | Character ROM | RAM (CPU sees RAM, not ROM) |
| 2 ($8000–$BFFF) | Character ROM | RAM (CPU sees RAM, not ROM) |
| 1 ($4000–$7FFF) | RAM | RAM |
| 3 ($C000–$FFFF) | RAM | RAM |

This is independent of the CPU's CHAREN banking signal. The VIC-II
fetch function must check:

```
IF (VIC_bank == 0 OR VIC_bank == 2) AND (offset >= $1000 AND offset <= $1FFF):
    RETURN Character_ROM[offset - $1000]
ELSE:
    RETURN RAM[VIC_bank_base + offset]
```

### Color RAM Access

Color RAM is a dedicated 1K×4 static RAM chip, physically separate
from the 64K DRAM. It has special bus characteristics:

- **VIC-II access:** Always accessible at a fixed internal address,
  regardless of CPU banking state. The VIC-II reads Color RAM via
  D8–D11 during c-accesses.
- **CPU access:** Visible at $D800–$DBFF when I/O is mapped at
  $D000–$DFFF. CPU reads/writes use D0–D7 (only lower 4 bits
  connected).
- **Banking independence:** When Character ROM is mapped instead of
  I/O (CHAREN=0), Color RAM is not accessible by the CPU, but the
  VIC-II can still read it for display.

### VIC-II vs. CPU Memory Access Summary

| Feature | 6510 CPU | VIC-II |
|---|---|---|
| Addressable range | Full 64K ($0000–$FFFF) | 16K bank (selected by $DD00) |
| Character ROM visibility | Via CHAREN (bit 2 of $01) | Hardwired in banks 0, 2 |
| RAM under ROM | Always writes to RAM | Always reads RAM |
| Banking control | LORAM, HIRAM, CHAREN ($01) | VIC bank ($DD00 bits 0–1) |
| I/O registers | Visible when I/O banked | Not visible (VIC-II has own registers) |
| Color RAM | Visible at $D800–$DBFF (when I/O mapped) | Always accessible at fixed internal address |
| Data bus width | 8-bit (D0–D7) | 12-bit (D0–D11) |

## Bad Lines and Bus Stealing

### Bad Line Condition

A Bad Line Condition exists at the negative edge of ø0 at the
beginning of a cycle if **all** of the following are true:

1. `RASTER` is between $30 and $F7 (inclusive).
2. The lower 3 bits of `RASTER` equal `YSCROLL` (bits 0–2 of $D011).
3. The `DEN` bit (bit 4 of $D011) was set at any point during
   raster line $30.

The Bad Line Condition can be triggered or suppressed dynamically
by modifying `YSCROLL` mid-frame.

### Bad Line Bus Behavior

On a Bad Line, the VIC-II must fetch 40 bytes of video matrix data
(c-accesses) and 40 bytes of color data. This requires stealing
bus cycles from the CPU:

| Event | Cycle |
|---|---|
| BA goes low (start of take-over) | Cycle 12 |
| AEC stays low (VIC-II takes bus) | Cycle 15 |
| c-accesses (40 total, one per cycle) | Cycles 15–54 |
| g-accesses (interleaved with c-accesses) | Cycles 15–54 |

During cycles 15–54, the CPU is completely stalled. Each cycle has
a c-access (first phase) and a g-access (second phase).

The VIC-II fetches:
- 40 screen codes from the video matrix (via D0–D7 from RAM)
- 40 color nybbles from Color RAM (via D8–D11)
- 40 character/bitmap data bytes (via D0–D7 from RAM)

Bad lines occur every 8th scan line during the visible display,
for a total of 25 bad lines per frame.

## Sprite DMA Bus Usage

Each enabled sprite requires additional bus cycles for data fetch.
The VIC-II steals these cycles in statically assigned slots.

| Sprites Enabled | Cycles Stolen per Line |
|---|---|
| 0 | 0 |
| 1 | 3 |
| 2 | 6 |
| ... | ... |
| 8 | 24 (maximum) |

Each sprite requires 3 s-accesses (2 for data, 1 for pointer) per
scan line, plus an additional access if the sprite is being
displayed.

> **TODO:** Document the exact cycle numbers for each sprite's
> p-access and s-access slots within the raster line.

## Performance Impact

When the display is enabled (bit 4 of $D011 set), the VIC-II
steals cycles from the CPU for Bad Lines and sprite DMA. With
display on and all 8 sprites enabled, the 6510 loses a significant
portion of its bus cycles.

When the screen is blanked (bit 4 of $D011 = 0):
- No character/sprite fetches occur
- No bad lines or sprite DMA
- The 6510 runs at full speed — approximately **7% faster** than
  with display enabled

This is used during tape I/O for timing-critical operations.

## Open Bus Behavior

The PLA does not fully decode all address ranges. Unconnected
address ranges exhibit specific open-bus behavior:

### $DE00–$DFFF (I/O Expansion)

A read from these unconnected addresses does not return random
data. On many C64s, the data bus retains the last byte read by
the VIC-II during the first phase of the clock cycle. This allows
software to measure VIC-II timing by reading the $DE00 area.

### $D800–$DBFF (Color RAM Upper Nybble)

Color RAM is only 4 bits wide. The upper 4 bits are open. Similar
to the $DE00 area, they often read back the upper bits of the last
byte the VIC-II fetched.

### $00–$01 (RAM Under I/O Port)

The 6510 I/O port overlays physical RAM at $00–$01. When the CPU
writes to $00 or $01, the data bus drivers for the internal I/O
port go into tri-state, but the R/W line is set low. If the VIC-II
reads the desired value from RAM $00 or $01 during the first clock
phase, the CPU's write cycle in the second phase will successfully
write that value to the underlying physical RAM.

## Default Power-On Memory Configuration

With no cartridge inserted and default I/O port settings
($01 = $37, LORAM=1, HIRAM=1, CHAREN=1):

| Address Range | Size | Content |
|---|---|---|
| $0000–$3FFF | 16 KB | RAM (system variables, screen, BASIC text) |
| $4000–$7FFF | 16 KB | RAM (BASIC program/variables continued) |
| $8000–$9FFF | 8 KB | RAM (or cartridge ROM if present) |
| $A000–$BFFF | 8 KB | BASIC ROM |
| $C000–$CFFF | 4 KB | RAM (unused by system, free for ML) |
| $D000–$DFFF | 4 KB | I/O: VIC-II, SID, CIAs, Color RAM |
| $E000–$FFFF | 8 KB | Kernal ROM |

## Emulation Notes

### Dual Address Decoding Paths

The emulator must implement two completely separate address
decoding paths:

1. **CPU path:** Evaluates LORAM, HIRAM, CHAREN, GAME, EXROM for
   each access. Routes to RAM, ROM, or I/O.
2. **VIC-II path:** Evaluates only the VIC-II bank ($DD00 bits 0–1)
   and the Character ROM overlay. Always reads from RAM (or
   Character ROM at fixed offsets).

The memory bus should expose two methods:
- `cpu_read(addr)` / `cpu_write(addr, val)` — goes through
  PLA/banking logic
- `vic_read(addr)` — bypasses CPU banking, applies only VIC-II
  bank and Character ROM overlay

This separation prevents the common bug where the VIC-II
accidentally reads ROM or I/O data instead of the underlying RAM.
The VIC-II and CPU should share the same underlying 64K RAM array,
but with different decoding logic on top.

### Separate Read and Write Paths

Writes always target RAM (write-through behavior). Reads may return
ROM or I/O data depending on banking. The emulator must not route
writes through the same decoding as reads.

### Cycle-Driven Main Loop

The emulator cannot simply step the CPU by one instruction; it
must step the system by one clock cycle:

- **CPU stepping:** The CPU struct requires a `step()` method that
  executes exactly one cycle, pausing if `RDY` is low.
- **VIC-II stepping:** The VIC-II struct requires a `step()` method
  that checks the current cycle number and performs the appropriate
  access (c, g, r, i, p, s).
- **Bus arbitration:** A central `Bus` struct should manage
  ownership. If the VIC-II performs an i-access, it should not
  block CPU writes, but a c-access must assert `RDY = false` and
  `AEC = false` at the correct cycles to freeze the CPU.

### Bus Interface Model

The CPU should be modeled with a bus interface that routes each
read/write through the PLA/banking logic. This ensures that dummy
reads and side effects are correctly routed. A single
`bus_read(addr)` and `bus_write(addr, val)` interface, shared by
the CPU, VIC-II, and cartridge DMA, keeps the memory model
consistent.

### BA/AEC Timing

The 3-cycle BA-low-before-AEC-low delay must be modeled precisely.
During this window, the CPU can still write but cannot read. If
the emulator does not model this window, software that performs
timing-sensitive writes during bad lines may break.

### VIC-II Internal Architecture

The VIC-II is not a monolithic state machine. The memory interface
(generating addresses and fetching data) is separate from the
display sequencer (converting fetched data into pixels). A 40×12
bit video matrix/color line buffer decouples these two units.

Suggested Rust design:
1. **Memory Fetch Frontend:** Cycle-by-cycle bus arbitration and
   address generation. Writes into internal `video_matrix_line`
   and `sprite_data_buffers`. Owns bus access state.
2. **Display Backend:** Renderer that reads from internal buffers
   and border/color registers to produce pixels. Operates with a
   fixed delay relative to the fetch frontend.
3. **State Isolation:** Frontend and backend should not share
   mutable state directly beyond the buffers.

### Color RAM Model

Color RAM must be maintained as a separate data structure that is:
1. Readable/writable by the CPU when I/O is mapped at $D000–$DFFF.
2. Always readable by the VIC-II for display rendering, regardless
   of CPU banking state.

Model as a 1024-byte array where only the lower 4 bits of each
byte are significant. On write, mask with `$0F`. On read, the
upper 4 bits should return either random values (most accurate)
or `$F0` (simpler).