# Raster Interrupts

The VIC-II can generate an IRQ when the raster beam reaches a specific scan
line. This enables mid-screen changes to display registers, split-screen
effects, scrolling zones, and cycle-exact timing.

## Raster Interrupt Registers

### $D012 — Raster Line Register

**Dual function:**
- **Read:** Returns the current raster line being scanned (bits 0–7 of the
  9-bit value).
- **Write:** Sets the raster compare value for raster interrupts (bits 0–7).

### $D011 Bit 7 — Raster MSB

Bit 7 of $D011 is bit 8 of the 9-bit raster value. This applies to both
reading the current raster line and writing the compare value.

Full 9-bit raster value: `RASTER = $D012 + (($D011 AND $80) × 2)`

**Default compare value after Kernal init:** The Kernal initializes the
raster compare to line 311 (for PAL detection). After initialization, the
default compare value depends on the Kernal version and PAL/NTSC detection
result.

**Default $D011:** $9B (%10011011) → bit 7 = 1 → raster MSB = 1.

### Raster Line Ranges

| System | Total Lines | Visible Lines | Vertical Blank |
|---|---|---|---|
| NTSC | 262 (0–261) | 50–249 | 250–261, 0–49 |
| PAL | 312 (0–311) | 50–249 | 250–311, 0–49 |

## Interrupt Flag Register ($D019)

**Default after clear:** $0F (all latched interrupts cleared).

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Raster compare IRQ flag |
| 1 | $02 | Sprite-to-foreground collision IRQ flag |
| 2 | $04 | Sprite-to-sprite collision IRQ flag |
| 3 | $08 | Light pen IRQ flag |
| 4–6 | $70 | Unused |
| 7 | $80 | Any IRQ (OR of bits 0–3) |

When any interrupt condition is met, the corresponding bit is set and bit 7
is set if any of bits 0–3 are set.

**Clearing:** Writing a 1 to any bit **clears** that bit's latch. This is
inverted logic — writing $01 clears the raster interrupt flag, not $00.
Writing $0F clears all VIC-II interrupt flags.

## Interrupt Mask Register ($D01A)

**Default:** $00 (all disabled).

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Enable raster compare IRQ |
| 1 | $02 | Enable sprite-to-foreground collision IRQ |
| 2 | $04 | Enable sprite-to-sprite collision IRQ |
| 3 | $08 | Enable light pen IRQ |
| 4–7 | $F0 | Unused |

A VIC-II IRQ is only generated when **both** the flag bit in $D019 is set
**and** the corresponding mask bit in $D01A is set to 1.

## Setting Up a Raster Interrupt

The sequence for installing a raster interrupt:

1. **Disable interrupts:** Set I flag (SEI) to prevent IRQ during setup.
2. **Enable raster interrupt mask:** Set bit 0 of $D01A to 1.
3. **Set raster compare line:** Write low byte to $D012, set/clear bit 7
   of $D011 for bit 8.
4. **Set IRQ vector:** Write the handler address to $0314–$0315 (CINV).
5. **Clear existing raster flag:** Write $01 to $D019 to clear any pending
   raster interrupt.
6. **Re-enable interrupts:** Clear I flag (CLI).

### Multiple Split Points

A single raster interrupt can only trigger at one line. To create multiple
split points (e.g., three display sections with different modes), the
interrupt handler must:
1. Determine which section triggered the interrupt (by reading $D012)
2. Apply the appropriate register changes
3. Re-program $D012 and $D011 bit 7 for the next split point
4. Clear the raster interrupt flag ($D019 = $01)
5. Chain to the system IRQ handler if it's time for the 1/60s keyscan

### Preserving the System IRQ

The standard system IRQ handler (at $EA31) runs every ~1/60 second and
handles keyboard scanning, clock updates, and cursor blink. If a raster
interrupt replaces the IRQ vector, the system IRQ must still be called
approximately once per frame.

Common approach: The raster interrupt handler checks if it's the "main"
interrupt (at the bottom of the screen, line ~250–260) and chains to the
system handler. Other interrupts during the frame are handled locally
without chaining.

## Raster Interrupt Timing

### When the Interrupt Fires

The raster interrupt fires at a specific **cycle** within the scan line,
not just at the start of the line. On NTSC systems, the raster compare
match occurs approximately at cycle 0 of the target line (the exact cycle
varies slightly by VIC-II revision).

### Cycle Accuracy

For cycle-accurate emulation, the raster interrupt should be checked every
cycle, not just once per scan line. The interrupt fires when the 9-bit
raster counter matches the 9-bit compare value. The raster counter
increments at the start of each scan line (cycle 0).

### Latency

The 6510 completes the current instruction before servicing the interrupt.
Worst-case latency is 7 cycles (longest instruction) + 7 cycles (interrupt
response) = 14 cycles from interrupt signal to first instruction of the
handler. During these 14 cycles, the raster beam has moved approximately
14 pixels to the right.

## Waiting for the Vertical Blank

A common technique (without using interrupts) is to poll bit 7 of $D011:

```
LOOP    LDA $D011      ; load raster MSB
        BPL LOOP       ; branch if bit 7 = 0 (raster < 256)
```

This waits until the raster enters lines 256–261 (NTSC) or 256–311 (PAL),
which is the vertical blanking interval — the safe time to modify screen
data without visual artifacts.

The BASIC equivalent is `WAIT 53265, 128`, but BASIC is usually too slow
to act before the beam leaves the blanking interval.

## Emulation Notes

### Raster Counter Implementation

The emulator must maintain a 9-bit raster counter that increments every
scan line and wraps at the total line count (262 NTSC, 312 PAL). The
counter must be readable via $D012 and $D011 bit 7 at any time.

### Interrupt Generation

When the raster counter matches the compare value, set bit 0 of $D019. If
bit 0 of $D01A is also set, assert the VIC-II IRQ line. The IRQ line
should be asserted until the interrupt flag is cleared (by writing $01 to
$D019).

### Cycle-Level Check

For cycle-accurate emulation, the raster compare should be checked at the
specific cycle within the scan line when the match occurs (approximately
cycle 0). This ensures that software using cycle-counting to determine the
raster position gets correct results.

### Multiple Interrupts per Frame

The emulator must support re-entrant raster interrupt programming, where
the handler changes the compare value for the next interrupt. Each time
the compare value is changed, the emulator should check if the new value
has already been passed on the current line (in which case the interrupt
fires on the next frame).

### PAL/NTSC Detection

During system initialization (`CINT` / `$FF5B`), the Kernal executes a physical raster scan line test to auto-detect whether the host machine is PAL or NTSC. 

1. The Kernal writes the value 311 (`$137`) into the Raster Compare registers (setting `$D012` to `$37` and Bit 7 of `$D011` to `1`).
2. It clears the interrupt status flags in `$D019` and waits for a brief duration.
3. Because NTSC systems generate only 262 scan lines per frame (0 to 261), they never reach line 311.
4. If a raster compare interrupt flag (Bit 0 of `$D019`) registers as set, the system is verified as PAL (312 lines). Otherwise, it defaults to NTSC.
5. The result of this test is stored at zero-page/working storage location `$02A6` (0 = NTSC, 1 = PAL).
6. This flag directly determines which CIA #1 Timer A latch value is selected to configure the system jiffy clock at exactly 60 Hz (NTSC) or 50 Hz (PAL):
   * **NTSC:** Latch = `$4118` (16664 cycles)
   * **PAL:** Latch = `$4295` (17045 cycles)
