# Raster Line Timing and Memory Access

The sequence of VIC-II memory accesses within a raster line is hard-wired, independent of the graphics mode, and identical for every raster line (with specific exceptions for sprite DMA). 

## VIC-II Revision Specifications

The exact line length, visible area, and timing differ between VIC-II chip revisions. Cycle-accurate emulation must account for these differences.

| Parameter | 6567R56A (NTSC) | 6567R8 (NTSC) | 6569 (PAL) |
|---|---|---|---|
| Video System | NTSC-M | NTSC-M | PAL-B |
| Total Lines per Frame | 262 | 263 | 312 |
| Visible Lines | 234 | 235 | 284 |
| Cycles per Line | 64 | 65 | 63 |
| Visible Pixels per Line | 411 | 418 | 403 |
| First VBlank Line | 13 | 13 | 300 |
| Last VBlank Line | 40 | 40 | 15 |
| First X Coordinate of a Line | 412 ($19C) | 412 ($19C) | 404 ($194) |
| First Visible X Coordinate | 488 ($1E8) | 489 ($1E9) | 480 ($1E0) |
| Last Visible X Coordinate | 388 ($184) | 396 ($18C) | 380 ($17C) |

**Note on X Coordinates:** The "First X Coordinate" refers to the sprite X coordinate at cycle 0 (the start of the raster line, defined by the raster IRQ). The X coordinates run up to $1FF (or $1F7 on PAL) within a line before wrapping to 0. The visible area is offset from the cycle-0 reference point.

## Access Types

The VIC performs 6 different types of memory accesses:

1. **c-access:** Video matrix access (12 bits wide; 8 bits from RAM, 4 bits from Color RAM).
2. **g-access:** Graphics access (8 bits wide; character generator or bitmap).
3. **p-access:** Sprite data pointer access (8 bits wide).
4. **s-access:** Sprite data access (8 bits wide).
5. **r-access:** DRAM refresh (5 read accesses per line).
6. **i-access:** Idle access. A read access to video address $3FFF (or $39FF if ECM is set), of which the result is discarded.

## Bus Take-Over Sequence (BA and AEC)

When the VIC needs extra cycles (for c-accesses on Bad Lines or s-accesses for sprites), it initiates a bus take-over from the 6510:

1. **BA goes Low:** The VIC pulls the BA line low 3 cycles before it needs the bus. This is connected to the 6510's RDY pin.
2. **Write Completion:** The 6510 ignores RDY during write cycles. The 3-cycle delay ensures the 6510 can finish any sequence of consecutive writes (max 3).
3. **AEC stays Low:** After the 3 cycles, AEC remains low during the second clock phase (ø2 high). This tri-states the 6510's address bus.
4. **VIC Access:** The VIC performs its reads during the second clock phase.

```text
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
*Note: "V" = VIC access, "P" = 6510 access, "p" = 6510 access (writes only, stops on read).*

## Raster Line Timing Diagrams

The exact cycle numbers for accesses differ between PAL (6569, 63 cycles) and NTSC (6567R8, 65 cycles) models.

### 6569 (PAL) - Bad Line, No Sprites
```text
Cycl-# 6                   1 1 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2 2 2 3 3 3 3 3 3 3 3 3 3 4 4 4 4 4 4 4 4 4 4 5 5 5 5 5 5 5 5 5 5 6 6 6 6
       3 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 1

   VIC i 3 i 4 i 5 i 6 i 7 i r r r r rcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcgcg i i 0 i 1 i 2 i 3
  6510  x x x x x x x x x x x x X X X                                                                                 x x x x x x x x x x

Graph.                      |===========01020304050607080910111213141516171819202122232425262728293031323334353637383940=========

X coo. \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\
       1111111111111111111111111110000000000000000000000000000000000000000000000000000000000000000111111111111111111111111111111111111111
       89999aaaabbbbccccddddeeeeff0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff000011112222333344445555666677778888999
       c048c048c048c048c048c048c04048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048c048
```

## Conceptual Considerations for Emulation

Modeling this timing requires a cycle-driven main loop. The emulator cannot simply step the CPU by one instruction; it must step the system by one clock cycle. 
- **CPU Stepping:** The CPU struct requires a `step()` method that executes exactly one cycle, pausing if `RDY` is low.
- **VIC Stepping:** The VIC struct requires a `step()` method that checks the current cycle number and performs the appropriate access (`c`, `g`, `r`, `i`, `p`, `s`).
- **Bus Arbitration:** A central `Bus` struct should manage ownership. If the VIC performs an `i-access`, it should not block CPU writes, but a `c-access` must assert `RDY = false` and `AEC = false` at the correct cycles to freeze the CPU.