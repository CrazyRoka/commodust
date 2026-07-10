# Internal Registers and Counters

The VIC-II contains several internal registers that track the state of the display and memory fetches. These are not directly accessible by the CPU but must be modeled by the emulator to achieve cycle-accurate behavior. 

## Text/Bitmap Counters

These counters manage the c-accesses (video matrix) and g-accesses (character/bitmap data).

| Register | Width | Function |
|---|---|---|
| VC | 10-bit | Video Counter. Generates addresses for the video matrix. |
| VCBASE | 10-bit | Video Counter Base. Holds the start address for the current text line. |
| RC | 3-bit | Row Counter. Tracks the 8 pixel lines of a text row (0-7). |
| VMLI | 6-bit | Video Matrix Line Index. Points to the current position in the internal 40x12-bit video matrix/color line buffer. |

### Update Rules

The behavior of these counters is strictly tied to the Bad Line Condition and the cycle number within the current raster line:

1. **Reset:** Once outside the range of raster lines $30–$F7, VCBASE is reset to 0. (Presumably in line 0).
2. **Cycle 14:** In the first phase of cycle 14, VC is loaded from VCBASE (`VCBASE -> VC`) and VMLI is cleared. If a Bad Line Condition exists in this phase, RC is also reset to 0.
3. **Bad Lines (C-accesses):** If a Bad Line Condition exists in cycles 12-54, BA is set low and the c-accesses are started. One c-access occurs per cycle in the range 15-54. The read data is stored in the video matrix/color line buffer at the position specified by VMLI.
4. **Display State:** After every g-access in display state, VC and VMLI are incremented.
5. **Cycle 58:** In the first phase of cycle 58, the VIC checks if RC = 7. If so, the video logic enters idle state and VCBASE is loaded from VC (`VC -> VCBASE`). If the video logic remains in display state (meaning a Bad Line Condition still exists), RC is incremented.

## Sprite Counters

Each of the 8 sprites has its own set of counters and an expansion flip-flop.

| Register | Width | Function |
|---|---|---|
| MC | 6-bit | MOB Data Counter. Generates addresses for sprite data (s-accesses). |
| MCBASE | 6-bit | MOB Data Counter Base. Holds the base address for the current sprite line. |
| Expansion FF | 1-bit | Flip-flop controlling Y expansion. |

### Update Rules

1. **Cycle 55:** If the sprite Y coordinate matches the raster line, DMA is enabled, MCBASE is cleared, and the expansion flip-flop is reset. (If MxYE is set, the flip-flop is inverted).
2. **Cycle 58:** MC is loaded from MCBASE (`MCBASE -> MC`). If Y matches and DMA is on, sprite display is turned on.
3. **S-accesses:** 3 s-accesses occur per line for the sprite if DMA is on. MC is incremented after each access.
4. **Cycle 15:** If the expansion flip-flop is set, MCBASE is incremented by 2.
5. **Cycle 16:** If the expansion flip-flop is set, MCBASE is incremented by 1. If MCBASE reaches 63, DMA and display are turned off.

## Refresh Counter

| Register | Width | Function |
|---|---|---|
| REF | 8-bit | DRAM Refresh Counter. |

### Update Rules
- Reset to $FF in raster line 0.
- Decremented by 1 after each refresh access (5 per line).

## Conceptual Considerations for Emulation

In a Rust emulator, the VIC-II state struct should own all of these counters. The `tick()` method—called every cycle—must evaluate the phase of the clock (first or second) and the current cycle number to determine which update rules apply. State transitions (like idle to display state) should be modeled as explicit enum variants to strictly gate the c-access and g-access logic.