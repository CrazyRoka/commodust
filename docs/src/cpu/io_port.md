# 6510 On-Chip I/O Port

The MOS 6510 microprocessor in the C64 includes an 8-bit I/O port that
is memory-mapped to zero-page locations $00 and $01. This port is
**not present on a standard 6502** and is unique to the 6510/8502
family.

## Register $00 — Data Direction Register (DDR)

| Bit | Signal | Default | Description |
|---|---|---|---|
| 0 | DIR for LORAM | 1 (output) | Controls direction of bit 0 on port |
| 1 | DIR for HIRAM | 1 (output) | Controls direction of bit 1 on port |
| 2 | DIR for CHAREN | 1 (output) | Controls direction of bit 2 on port |
| 3 | DIR for Cassette Data Out | 1 (output) | Controls direction of bit 3 |
| 4 | DIR for Cassette Switch | 0 (input) | Controls direction of bit 4 |
| 5 | DIR for Cassette Motor | 1 (output) | Controls direction of bit 5 |
| 6 | — | 1 | Not connected (reads as 1 at power-on) |
| 7 | — | 1 | Not connected (reads as 1 at power-on) |

**Power-on default: $EF (%11101111)** — All bits initialize to 1
except bit 4 (cassette switch sense, which is an input). Bits 6–7
are not connected to any external signal but still read as 1 in the
register. Only bits 0–5 have meaningful function.

## Register $01 — Data Port

| Bit | Signal | Default | Description |
|---|---|---|---|
| 0 | LORAM | 1 | Selects BASIC ROM (1) or RAM (0) at $A000–$BFFF |
| 1 | HIRAM | 1 | Selects Kernal ROM (1) or RAM (0) at $E000–$FFFF |
| 2 | CHAREN | 1 | Selects I/O (1) or Character ROM (0) at $D000–$DFFF |
| 3 | Cassette Data Out | 0 | Serial data line to cassette |
| 4 | Cassette Switch Sense | 1 | Reads 0 if a cassette button is pressed |
| 5 | Cassette Motor | 0 | 1 = motor on, 0 = motor off |
| 6 | — | 0 | Not connected |
| 7 | — | 0 | Not connected |

**Power-on default: $37 (%00110111)**

## Cassette Motor Interlock

The cassette motor control (bit 5) is not freely controllable from
software at all times. The system IRQ handler (executing every 1/60
second) maintains an **interlock** at location $C0:

- If no cassette button is pressed, the IRQ routine sets bit 5 to 0
  (motor off) and clears the interlock at $C0 to 0.
- If a button is pressed and $C0 is 0, the IRQ routine sets bit 5 to 0
  (motor on) — software cannot override this.
- To gain software control of the motor, a program must write a
  **nonzero** value to $C0 **after** a cassette button has been pressed.

This interlock behavior must be emulated if cassette operations are
supported.

## Interaction with Expansion Port

Bits 0–2 (LORAM, HIRAM, CHAREN) are not the sole determinants of the
memory map. The expansion port exposes two hardware lines, **GAME** and
**EXROM**, which can override or modify the software-controlled banking.
See [Banking](../architecture/banking.md) for the complete truth table.

## Emulation Notes

In an emulator, the I/O port registers should be modeled as:

1. A 2-byte register pair that feeds into the memory banking logic.
2. Any write to $01 that changes bits 0–2 must immediately update the
   memory map for subsequent CPU accesses.
3. The cassette switch sense (bit 4 of $01) is an input; its value is
   determined by the virtual cassette device state, not by CPU writes.
4. The cassette motor control is overridden by the IRQ interlock logic
   described above.