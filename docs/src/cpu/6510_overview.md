# 6510 Overview

The Commodore 64's central processor is the MOS 6510, a variant of
the MOS 6502. The 6510 is instruction-set compatible with the 6502
but adds an integrated 8-bit I/O port mapped to zero-page locations
$00 and $01. This I/O port is used for memory banking control and
cassette interface signals.

## Architecture Summary

| Feature | Specification |
|---|---|
| Architecture | 8-bit, MOS 6502-compatible |
| Clock (NTSC) | ~1.02273 MHz (1/14 of 14.31818 MHz colorburst) |
| Clock (PAL) | ~0.98525 MHz (1/18 of 17.73447 MHz colorburst) |
| Address bus | 16-bit (64 KB addressable) |
| Data bus | 8-bit |
| Registers | A (Accumulator), X, Y, SP (Stack Pointer), PC, P (Status) |
| Stack | Hardware stack at $0100–$01FF, descending |
| Interrupts | IRQ (maskable), NMI (non-maskable), BRK (software) |
| Reset vector | $FFFC–$FFFD (points to $FCE2 in Kernal) |
| NMI vector | $FFFA–$FFFB (points to $FE43 in Kernal) |
| IRQ/BRK vector | $FFFE–$FFFF (points to $FF48 in Kernal) |
| On-chip I/O | 8-bit port at $00 (DDR) / $01 (Data) |

## Programmer's Model

### Registers

```
┌─────────────────────────────────────────────┐
│              6510 Register Model             │
├──────────┬──────┬───────────────────────────┤
│Register  │ Width│ Function                   │
├──────────┼──────┼───────────────────────────┤
│ A        │ 8-bit│ Accumulator (math target)  │
│ X        │ 8-bit│ X index register           │
│ Y        │ 8-bit│ Y index register           │
│ SP       │ 8-bit│ Stack pointer (hardwired   │
│          │     │ to page $01)               │
│ PC       │16-bit│ Program counter            │
│ P        │ 8-bit│ Processor status flags     │
└──────────┴──────┴───────────────────────────┘
```

### Status Register (P) Bit Layout

| Bit | Mask | Label | Name | Function |
|---|---|---|---|---|
| 0 | $01 | C | Carry | Set if arithmetic carry/borrow occurred |
| 1 | $02 | Z | Zero | Set if result is zero |
| 2 | $04 | I | Interrupt Disable | Set = IRQ disabled (SEI), cleared = enabled (CLI) |
| 3 | $08 | D | Decimal Mode | Set = BCD arithmetic for ADC/SBC |
| 4 | $10 | B | Break | Set in pushed P only when BRK executes (not a real flag) |
| 5 | $20 | — | Unused | Always reads as 1 in pushed P |
| 6 | $40 | V | Overflow | Set if signed overflow occurred |
| 7 | $80 | N | Negative | Set if bit 7 of result is 1 |

**Note for emulator developers:** The B (Break) bit and bit 5 are not
physical flags in the P register. They exist only in the value pushed
to the stack during IRQ/NMI/BRK. When an IRQ or NMI pushes P, B=0 and
bit 5=1. When BRK pushes P, B=1 and bit 5=1. Reading P directly (via
PHP) always shows B=1 and bit 5=1 in the pushed value. These two bits
are not readable from the live P register.

## Instruction Set

The 6510 executes 151 official opcodes (out of 256 possible byte
values) plus numerous undocumented "illegal" opcodes. Instructions
are 1–3 bytes long and execute in 2–7 cycles depending on addressing
mode and whether a page boundary is crossed.

### Addressing Modes

| Mode | Syntax Example | Bytes | Cycles (base) |
|---|---|---|---|
| Implied | `INX` | 1 | 2 |
| Accumulator | `ASL A` | 1 | 2 |
| Immediate | `LDA #$44` | 2 | 2 |
| Zero Page | `LDA $44` | 2 | 3 |
| Zero Page,X | `LDA $44,X` | 2 | 4 |
| Zero Page,Y | `LDX $44,Y` | 2 | 4 |
| Absolute | `LDA $4400` | 3 | 4 |
| Absolute,X | `LDA $4400,X` | 3 | 4 (+1 if page cross) |
| Absolute,Y | `LDA $4400,Y` | 3 | 4 (+1 if page cross) |
| Indirect | `JMP ($4400)` | 3 | 5 |
| (Indirect,X) | `LDA ($44,X)` | 2 | 6 |
| (Indirect),Y | `LDA ($44),Y` | 2 | 5 (+1 if page cross) |
| Relative | `BNE $44` | 2 | 2 (+1 if branch taken, +1 if page cross) |

### Page Boundary Cross Penalty

When an indexed addressing mode crosses a 256-byte page boundary
(i.e., when the base address + index results in a different high byte
than the base address), an extra cycle is added. This is critical for
cycle-accurate emulation:

```
Base address high byte != (Base address + index) high byte
→ add 1 cycle
```

### Branch Timing

| Condition | Cycles |
|---|---|
| Branch not taken | 2 |
| Branch taken, same page | 3 |
| Branch taken, page cross | 4 |

## Known Hardware Quirks

### Indirect JMP Page Wrap Bug

The `JMP (addr)` instruction has a famous bug: if `addr` is at the
last byte of a page (e.g., `$xxFF`), the high byte of the target is
fetched from `$xx00` instead of `$(xx+1)00`. The low byte comes from
`$xxFF` and the high byte from `$xx00`, wrapping within the same page.

```
Example: JMP ($10FF)
  Low byte of target  ← read from $10FF  (correct)
  High byte of target ← read from $1000  (BUG: should be $1100)
```

This bug must be emulated. Commercial software and even the Kernal
ROM avoid placing indirect jump pointers at page boundaries.

### Dummy Reads

Several instructions perform "dummy" read cycles before the actual
write cycle. For example, `INC $xx` performs a read, then a write,
then another write with the incremented value. These dummy reads
matter because they can trigger side effects in memory-mapped I/O
registers (e.g., clearing VIC-II interrupt flags or CIA interrupt
registers).

### RMW (Read-Modify-Write) Instruction Dummy Cycles

Instructions that read-modify-write (ASL, LSR, ROL, ROR, INC, DEC)
on absolute or zero-page addresses perform a dummy write cycle
between the read and the final write. Emulators must account for
this when modeling memory-mapped I/O side effects.

### Decimal Mode (BCD)

When the D flag is set, ADC and SBC operate in Binary Coded Decimal
mode. The 6510's BCD implementation has specific behavior for the
N, V, and Z flags that differs from later 6502 variants. In BCD
mode, the N and V flags are set based on intermediate binary results,
not the final BCD result. The Z flag is set based on the binary
result of the addition.

### Illegal Opcodes

There are approximately 85 undocumented opcode values. Some are
useful and were used in commercial software:

- **LAX** ($A7, $B7, $AF, $BF, $A3, $B3): Load A and X simultaneously
- **SAX** ($87, $97, $8F, $83): Store A AND X to memory
- **DCP** ($C7, $D7, $CF, $DF, $C3, $D3): DEC memory, then CMP
- **ISC** / **ISB** ($E7, $F7, $EF, $FF, $E3, $F3): INC memory, then SBC
- **SLO** ($07, $17, $0F, $1F, $03, $13): ASL memory, then ORA
- **RLA** ($27, $37, $2F, $3F, $23, $33): ROL memory, then AND
- **SRE** ($47, $57, $4F, $5F, $43, $53): LSR memory, then EOR
- **RRA** ($67, $77, $6F, $7F, $63, $73): ROR memory, then ADC
- **ANC** ($0B, $2B): AND immediate, then copy result to carry
- **ALR** / **ASR** ($4B): AND immediate, then LSR
- **ARR** ($6B): AND immediate, then ROR (with complex flag behavior)
- **SBX** / **AXS** ($CB): (A AND X) − immediate → X
- **NOP** variants ($1A, $3A, $5A, $7A, $DA, $FA): Do nothing, 2 cycles
- **JAM** / **KIL** ($02, $12, $22, $32, $42, $52, $62, $72, $92,
  $B2, $D2, $F2): Halt the CPU (no recovery except reset)

An emulator targeting compatibility with commercial C64 software
**must** implement at least the commonly used illegal opcodes (LAX,
SAX, DCP, ISC, SLO, RLA, SRE, RRA, and the stable NOPs).

## Power-On / Reset Behavior

On RESET (cold start):

1. The 6510 reads the RESET vector from $FFFC–$FFFD.
2. On the C64, this points to $FCE2 (the Kernal power-on routine).
3. The Interrupt disable flag (I) is set.
4. The Decimal mode flag (D) is cleared.
5. The Stack Pointer is set to a high value (typically $FD after
   Kernal initialization pushes/pops).
6. Registers A, X, Y are in an indeterminate state.

The Kernal RESET routine then:
- Tests for an autostart cartridge at $8004–$8008 (looking for the
  bytes $C3, $C2, $CD, $38, $30 — inverse "CBM80").
- If found, jumps through the cartridge cold-start vector at $8000.
- If not found, calls IOINIT, RAMTAS, RESTOR, and CINT.
- Clears the I flag and enters BASIC through the cold-start vector
  at $A000.

## Conceptual Considerations for Emulation

### Cycle Accounting

The 6510 should be emulated as a cycle-driven state machine, not an
instruction-driven one. Each instruction advances a number of cycles,
and the emulator must interleave CPU execution with VIC-II and CIA
chip updates at the cycle granularity. This is because:

- Bad-line stalls must pause the CPU at the exact cycle.
- CIA timers tick every cycle and can fire interrupts mid-instruction.
- VIC-II raster interrupts fire at a specific cycle within a scan line.

### Bus Interface Model

Conceptually, the CPU should be modeled with a bus interface that
routes each read/write through the PLA/banking logic. This ensures
that dummy reads and side effects are correctly routed. A single
`bus_read(addr)` and `bus_write(addr, val)` interface, shared by
the CPU, VIC-II, and cartridge DMA, keeps the memory model
consistent.

### Illegal Opcode Handling

Rather than throwing an error, the emulator should implement illegal
opcodes as their documented behavioral equivalents. Many are simple
combinations of two legal operations. The JAM/KIL opcodes should
halt execution and require a hardware reset.

### Register Save for BRK/SYS

The Kernal uses locations $30C–$30F to pass register values to and
from `SYS` calls. The emulator does not need to model this at the
CPU level — it is handled by Kernal software. However, the CPU must
correctly push P (with B=1 for BRK, B=0 for IRQ/NMI) during
interrupt processing.