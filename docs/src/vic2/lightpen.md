# Light Pen

On a negative edge on the LP input, the VIC latches the current X and Y position of the raster beam into registers `$D013` (LPX) and `$D014` (LPY).

## Latching Behavior

- **LPX:** Contains the upper 8 bits of the 9-bit X coordinate. (Resolution is 2 pixels).
- **LPY:** Contains the lower 8 bits of the 9-bit Y coordinate.
- Only one negative edge is recognized per frame. Subsequent edges are ignored until the next vertical blanking interval.

## Software Triggering

The LP pin shares a line with the keyboard matrix (specifically, CIA #1 Port B bit 4). Software can trigger a light pen interrupt by pulling this line low.

This allows software to determine the exact X coordinate of the raster beam, which is normally unreadable. The reference point is the end of the cycle in which LP is triggered. For example, triggering LP in cycle 20 yields `$1E` in LPX.

## Interrupts

A negative edge on LP sets bit 3 of the Interrupt Flag Register (`$D019`). If enabled in `$D01A`, an IRQ is generated.