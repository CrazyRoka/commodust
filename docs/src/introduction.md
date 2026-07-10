# Introduction

This documentation serves as a hardware-level reference for building a
Commodore 64 emulator from scratch. Every component is documented with
exact register layouts, bit-masks, power-on defaults, memory maps,
timing constraints, and known hardware quirks.

## Document Conventions

- **Hexadecimal** values are prefixed with `$` (e.g., `$D020`).
- **Binary** values are prefixed with `%` (e.g., `%10011011`).
- **Decimal** values have no prefix.
- Register bit numbering is 0 (LSB) through 7 (MSB) unless otherwise stated.
- "Default" or "power-on" values refer to the state of a register
  immediately after the Kernal initialization routines complete at
  cold-start RESET.

## The Commodore 64 at a Glance

| Feature | Specification |
|---|---|
| CPU | MOS 6510 (6502-compatible) |
| CPU Clock (NTSC) | ~1.02273 MHz |
| CPU Clock (PAL) | ~0.98525 MHz |
| RAM | 64 KB (65536 bytes) |
| BASIC ROM | 8 KB ($A000–$BFFF) |
| Kernal ROM | 8 KB ($E000–$FFFF) |
| Character ROM | 4 KB ($D000–$DFFF, when banked) |
| Video Chip | VIC-II (6567/6569) |
| Sound Chip | SID (6581) |
| I/O Chips | Two 6526 CIA chips |
| Video Output | 320×200 hi-res / 160×200 multicolor |
| Sprites | 8 independent, 24×21 pixels each |
| Audio | 3-voice synthesizer + programmable filter |
| Display Standard | NTSC (262 scan lines) or PAL (312 scan lines) |

## Emulator Development Philosophy

When emulating the C64, the following principles should guide implementation:

1. **Cycle accuracy over instruction accuracy.** The C64's custom chips
   operate in lockstep with the CPU's bus cycles. A raster interrupt
   that fires on the wrong cycle will produce visible artifacts.

2. **Banking is global state.** The 6510 I/O port, expansion port
   signals (GAME/EXROM), and VIC-II bank select bits on CIA #2 all
   interact to determine what appears at any given address. The memory
   subsystem must evaluate all of these for every bus access.

3. **Write-through to underlying RAM.** Even when ROM or I/O is mapped
   at a given address, writes always go to the physical RAM beneath.
   This is critical for VIC-II display, which reads RAM regardless of
   the 6510's banking state.

4. **Undocumented behavior is real behavior.** Illegal opcodes,
   partial register decodes, and open bus reads must be emulated
   because commercial software relies on them.