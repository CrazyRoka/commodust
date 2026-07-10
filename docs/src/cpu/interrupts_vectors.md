# Interrupts and Vectors

The 6510 supports three interrupt types and three hardware vectors
located at the top of the address space. The Kernal ROM also
maintains a set of RAM-based indirect vectors that allow software to
redirect interrupt handling.

## Hardware Vectors (ROM, $FFFA–$FFFF)

These six bytes are hardwired in the Kernal ROM and cannot be
changed by software.

| Address | Vector | Target | Trigger |
|---|---|---|---|
| $FFFA–$FFFB | NMI | $FE43 | NMI line asserted (RESTORE key or CIA #2) |
| $FFFC–$FFFD | RESET | $FCE2 | Power-on or RESET line |
| $FFFE–$FFFF | IRQ/BRK | $FF48 | IRQ line asserted (CIA #1 or VIC-II) or BRK opcode |

## Interrupt Types

### IRQ (Maskable Interrupt Request)

- Triggered by: CIA #1 interrupt line, VIC-II interrupt line.
- Maskable: Disabled when I flag = 1 (set by SEI, cleared by CLI).
- Vector: RAM vector at $0314–$0315 (CINV), default target $EA31.
- Processing: CPU pushes PC(hi), PC(lo), P (with B=0) onto stack,
  sets I flag, reads target from $FFFE–$FFFF.

### NMI (Non-Maskable Interrupt)

- Triggered by: RESTORE key, CIA #2 interrupt line.
- Not maskable by the I flag. Can only be disabled at the CIA #2
  level via the ICR mask.
- Vector: RAM vector at $0318–$0319 (NMINV), default target $FE47.
- Processing: CPU pushes PC(hi), PC(lo), P (with B=0) onto stack,
  sets I flag, reads target from $FFFA–$FFFB.

### BRK (Software Interrupt)

- Triggered by: executing the BRK opcode ($00).
- Uses the same hardware vector as IRQ ($FFFE–$FFFF).
- The CPU sets the B bit in the pushed P register to distinguish
  BRK from IRQ.
- The ROM handler at $FF48 tests the B bit: if set, it routes
  through the BRK RAM vector at $0316–$0317 (CBINV).

## IRQ Processing Flow

```
                    ┌─────────────┐
                    │ IRQ occurs  │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Push PC(hi) │
                    │ Push PC(lo) │
                    │ Push P      │
                    │ (B=0, bit5=1)│
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Set I flag  │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Read $FFFE  │
                    │ → $FF48     │
                    └──────┬──────┘
                           │
              ┌────────────▼────────────┐
              │ $FF48: Test B bit in    │
              │ pushed P                │
              └────────────┬────────────┘
                     │           │
              B=0 (IRQ)    B=1 (BRK)
                     │           │
         ┌───────────▼──┐   ┌───▼──────────────┐
         │ JMP ($0314)  │   │ JMP ($0316)      │
         │ → $EA31      │   │ → $FE66 (BRK     │
         │ (standard    │   │    warm start)   │
         │  IRQ handler)│   │                  │
         └──────────────┘   └──────────────────┘
```

## RAM Interrupt Vectors

These vectors are in RAM and can be redirected by software. They
are initialized by the Kernal RESTOR routine from a ROM table.

| Address | Label | Default Target | Function |
|---|---|---|---|
| $0314–$0315 | CINV | $EA31 | IRQ handler |
| $0316–$0317 | CBINV | $FE66 | BRK handler |
| $0318–$0319 | NMINV | $FE47 | NMI handler |

### Standard IRQ Handler ($EA31)

Executed every 1/60 second (driven by CIA #1 Timer A). Performs:

1. Update software jiffy clock ($A0–$A2).
2. Blink cursor (decrement $CD, toggle cursor when it reaches 0).
3. Maintain tape motor interlock ($C0).
4. Scan keyboard (call SCNKEY).
5. Check STOP key (update $91).

### NMI Handler ($FE47)

1. Check if CIA #2 caused the NMI (RS-232 interrupt).
   - If yes, handle RS-232 send/receive.
2. If not CIA #2, assume RESTORE key:
   - Check for autostart cartridge; if present, jump to warm start
     at $8002.
   - Check STOP key; if pressed simultaneously with RESTORE, execute
     BRK warm start ($FE66): calls RESTOR, IOINIT, part of CINT,
     then enters BASIC through warm start vector at $A002.
   - If STOP not pressed, silently return (NMI ignored).

### BRK Handler ($FE66)

Calls RESTOR, IOINIT, and part of CINT, then enters BASIC through
the warm start vector at $A002. This is the same routine triggered
by STOP+RESTORE.

## Interrupt Sources

### CIA #1 (IRQ)

Five sources, all OR'd into a single IRQ line:

| Source | ICR Bit |
|---|---|
| Timer A underflow | 0 |
| Timer B underflow | 1 |
| TOD alarm match | 2 |
| Serial shift register complete | 3 |
| FLAG line (cassette read) | 4 |

### CIA #2 (NMI)

Same five sources, but connected to NMI:

| Source | ICR Bit |
|---|---|
| Timer A underflow | 0 |
| Timer B underflow | 1 |
| TOD alarm match | 2 |
| Serial shift register complete | 3 |
| FLAG line (User Port pin B) | 4 |

### VIC-II (IRQ)

Four sources:

| Source | $D019 Bit | $D01A Mask Bit |
|---|---|---|
| Raster compare match | 0 | 0 |
| Sprite-to-foreground collision | 1 | 1 |
| Sprite-to-sprite collision | 2 | 2 |
| Light pen trigger | 3 | 3 |

## Interrupt Latency

The 6510 completes the current instruction before servicing an
interrupt. Worst-case latency is 7 cycles (the longest 6502
instruction). The interrupt response itself (pushing PC and P,
reading the vector) takes 7 cycles. Total worst-case latency from
interrupt signal to first instruction of the handler: **14 cycles**.

## SEI/CLI Safety

When modifying RAM interrupt vectors (e.g., installing a custom IRQ
handler), software must:

1. Execute SEI to disable IRQs.
2. Change the vector.
3. Execute CLI to re-enable IRQs.

Failure to do this risks an interrupt occurring while the vector is
half-changed (only the low byte written), which would jump to a
garbage address and crash the system.

The Kernal provides the VECTOR routine ($FD1A / jump table $FF8D)
to safely read or set all 16 I/O vectors at once. This routine
should be used in preference to direct POKEing.

## Emulation Notes

### IRQ vs NMI Priorities

NMI has higher priority than IRQ. If both are pending, NMI is
serviced first. In practice, an emulator should check NMI before
IRQ at each cycle boundary.

### BRK Detection

The BRK opcode ($00) is a 1-byte instruction, but the CPU reads it
as a 2-byte instruction (the byte after $00 is ignored). The
emulator must increment PC by 2 before pushing it, so that RTS/RTI
from a BRK handler skips the padding byte.

### Interrupt Timing Within Instructions

For cycle-accurate emulation, interrupts should be checked at the
end of each instruction (or more precisely, at the last cycle of
each instruction). The 6510 samples IRQ during the last cycle of
each instruction. NMI is edge-triggered and sampled similarly.

### Kernal Dependency

If the emulator runs the real Kernal ROM, it must ensure that:
- CIA #1 Timer A is initialized by IOINIT to generate IRQ at ~60 Hz.
- The IRQ vector at $0314 points to $EA31.
- The NMI vector at $0318 points to $FE47.
- The RESTORE key is wired to the NMI line (not a keyboard matrix
  key).

If the emulator uses high-level (HLE) interrupt handling, it must
still correctly model the CIA and VIC-II interrupt sources, ICR
mask/flag behavior, and the $D019/$D01A VIC-II interrupt registers.