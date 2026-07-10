# Display Generation and Border Unit

The VIC-II generates the screen border using two flip-flops: the **Main Border Flip-Flop** and the **Vertical Border Flip-Flop**.

## Display Window Dimensions

The display window is defined by the RSEL and CSEL bits, which select between two hardwired comparison values.

| RSEL | Height | First Line | Last Line |
|---|---|---|---|
| 0 | 24 rows / 192 px | 55 ($37) | 246 ($F6) |
| 1 | 25 rows / 200 px | 51 ($33) | 250 ($FA) |

| CSEL | Width | First X | Last X |
|---|---|---|---|
| 0 | 38 cols / 304 px | 31 ($1F) | 334 ($14E) |
| 1 | 40 cols / 320 px | 24 ($18) | 343 ($157) |

*Note: If RSEL=0, the upper/lower borders extend by 4 pixels. If CSEL=0, the left border extends by 7 pixels and the right by 9 pixels. The resolution of the display window itself does not change.*

## Flip-Flop Logic

### Main Border Flip-Flop
Controls the border display. If set, the border color ($D020) is displayed, overriding all graphics and sprites.

### Vertical Border Flip-Flop
Controls the upper/lower border. If set, it prevents the Main Flip-Flop from being reset and disables the graphics data sequencer (forcing background color output, preventing sprite-data collisions in the border).

### Transition Rules

The flip-flops are controlled by comparing the raster X/Y coordinates with the hardwired values. The comparisons only match exact values, not intervals.

1. **Right Edge:** If X reaches the right comparison value, the Main Flip-Flop is set.
2. **Bottom Edge (Cycle 63):** If Y reaches the bottom comparison value in cycle 63, the Vertical Flip-Flop is set.
3. **Top Edge (Cycle 63):** If Y reaches the top comparison value in cycle 63 AND `DEN` is set, the Vertical Flip-Flop is reset.
4. **Left/Bottom Corner:** If X reaches the left comparison value AND Y reaches the bottom, the Vertical Flip-Flop is set.
5. **Left/Top Corner:** If X reaches the left AND Y reaches the top AND `DEN` is set, the Vertical Flip-Flop is reset.
6. **Left Edge:** If X reaches the left comparison value AND the Vertical Flip-Flop is NOT set, the Main Flip-Flop is reset.

*Note: The Y coordinate is checked once or twice per line: in cycle 63, and when X reaches the left comparison value.*

## Display Enable (DEN)

The `DEN` bit (bit 4 of $D011) affects two areas:
1. It is required to reset the Vertical Border Flip-Flop (Rules 3 and 5). If cleared, the upper/lower border cannot be turned off.
2. It is required for Bad Lines. A Bad Line can only occur if `DEN` was set at some point in raster line $30.

## Display Pipeline Delay

It is critical to note that the internal memory accesses (`c-accesses` and `g-accesses`) and the actual video output signal are decoupled by a data buffer. The border unit's comparisons and the resulting video signal changes occur approximately **8 pixels (1 cycle)** *after* the corresponding memory access or internal counter event for that screen position. 

When emulating the border unit, the X/Y coordinate comparisons must be evaluated against the raster counter at the precise cycle, but the resulting pixel output modification is delayed. This decoupling is what allows techniques like DMA Delay to shift the screen without corrupting the currently displayed line.