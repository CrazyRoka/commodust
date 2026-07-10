# Known Effects and Quirks

The VIC-II's internal design allows for several unconventional programming techniques that exploit bus timing and internal counters.

## Hyperscreen (Opening the Border)
By switching the `CSEL` or `RSEL` bits at precise cycles, the X/Y coordinate comparisons for the border flip-flops can be skipped. This prevents the border from turning on, allowing graphics or sprites to be displayed in the border area.
- **Vertical:** Switch from RSEL=1 to RSEL=0 between raster lines 248-250. The comparator switches to line 247, but that line has already passed.
- **Horizontal:** Switch from CSEL=1 to CSEL=0 exactly in cycle 56.

## FLD (Flexible Line Distance)
By modifying `YSCROLL` mid-frame, the Bad Line Condition can be delayed. This increases the distance between text lines, pushing the graphics down the screen without moving memory.

## FLI (Flexible Line Interpretation)
By forcing a Bad Line Condition on *every* raster line (via `YSCROLL`), the VIC-II reads new color information from the video matrix every 8 pixels. In multicolor bitmap mode, this allows each 4x8 pixel block to have a unique color.
- **Artifact:** Because the Bad Line is triggered in cycle 14, BA goes low in cycle 15. AEC doesn't stay low until cycle 18. For the first 3 c-accesses, AEC is still high, meaning the VIC-II's internal data bus drivers (D0-D7) are in tri-state. Because the chip is manufactured in NMOS technology, it reads `$FF` instead of video matrix data. The upper 4 bits (D8-D11, Color RAM) are always set to input, but the Color RAM chip select is inactive (as the 6510 is still technically the bus master), so no valid color data is read either. This results in a 24-pixel wide artifact on the left edge of the screen (3 bytes of `$FF` character pointers).

## Linecrunch
By suppressing a Bad Line Condition before cycle 14, the current text line is aborted. `VCBASE` increments by 40, effectively skipping a text line. This allows rapid vertical scrolling upward.

## DMA Delay
If a Bad Line Condition is triggered mid-line (cycles 15-53) while in idle state, the VIC-II starts c-accesses immediately. The 3-cycle BA/AEC delay causes the first 3 reads to return `$FF`. The subsequent reads are misaligned, causing the entire screen to shift right by a number of characters.

## Sprite Stretching
By toggling the `MxYE` (Y Expansion) bit in specific cycles (15 and 16), the expansion flip-flop logic can be confused. This causes `MCBASE` to increment incorrectly, forcing the VIC-II to read the same sprite line 3 times or display the sprite twice in succession.