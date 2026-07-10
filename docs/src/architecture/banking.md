# Memory Banking and Configuration

## The 6510 On-Chip I/O Port

The 6510 CPU used in the C64 has an integrated 8-bit I/O port, accessed
via two zero-page registers:

- **$00** — Data Direction Register (DDR)
- **$01** — Data Port Register

### Register $00: DDR (Default = $EF / %11101111)

| Bit | Direction | Default | Function |
|---|---|---|---|
| 0 | Output | 1 | DIR for LORAM signal |
| 1 | Output | 1 | DIR for HIRAM signal |
| 2 | Output | 1 | DIR for CHAREN signal |
| 3 | Output | 1 | DIR for Cassette Data Output |
| 4 | Input | 0 | DIR for Cassette Switch Sense |
| 5 | Output | 1 | DIR for Cassette Motor Control |
| 6 | — | 1 | Not connected (reads as 1) |
| 7 | — | 1 | Not connected (reads as 1) |

### Register $01: Data Port (Default = $37 / %00110111)

| Bit | Signal | Default | Function |
|---|---|---|---|
| 0 | LORAM | 1 | 1 = BASIC ROM at $A000; 0 = RAM at $A000 |
| 1 | HIRAM | 1 | 1 = Kernal ROM at $E000; 0 = RAM at $E000 |
| 2 | CHAREN | 1 | 1 = I/O at $D000; 0 = Character ROM at $D000 |
| 3 | — | 0 | Cassette Data Output line |
| 4 | — | 1 | Cassette Switch Sense (0 = button pressed) |
| 5 | — | 0 | Cassette Motor Control (1 = motor on) |
| 6 | — | 0 | Not connected |
| 7 | — | 0 | Not connected |

## Banking Truth Table

The effective memory configuration depends on LORAM, HIRAM, CHAREN,
and the hardware GAME and EXROM lines from the expansion port.

### No Cartridge (GAME=1, EXROM=1)

| LORAM | HIRAM | CHAREN | $A000–$BFFF | $D000–$DFFF | $E000–$FFFF |
|---|---|---|---|---|---|
| 1 | 1 | 1 | BASIC ROM | I/O (VIC/SID/CIA/Color) | Kernal ROM |
| 1 | 1 | 0 | BASIC ROM | Character ROM | Kernal ROM |
| 1 | 0 | 1 | BASIC ROM | I/O | RAM |
| 1 | 0 | 0 | BASIC ROM | Character ROM | RAM |
| 0 | 1 | 1 | RAM | I/O | Kernal ROM |
| 0 | 1 | 0 | RAM | Character ROM | Kernal ROM |
| 0 | 0 | 1 | RAM | I/O | RAM |
| 0 | 0 | 0 | RAM | Character ROM | RAM |

**Key rule:** CHAREN only has effect when HIRAM=1 (i.e., when the
Kernal ROM is visible). When HIRAM=0, the $D000 region is always I/O
regardless of CHAREN. This is because the CHAREN signal is gated
through the Kernal ROM select logic.

### Cartridge Configurations

| Cartridge Type | GAME | EXROM | Effect |
|---|---|---|---|
| 8K ROM at $8000 | 1 | 0 | ROM at $8000–$9FFF |
| 8K ROM at $A000 | 0 | 1 | ROM at $A000–$BFFF (replaces BASIC) |
| 16K ROM at $8000 | 0 | 0 | ROM at $8000–$BFFF |
| Ultimax mode | 0 | 1 | ROM at $E000–$FFFF (replaces Kernal); only 6K RAM used |

## Write-Through Behavior

Regardless of banking, **all writes by the 6510 go to physical RAM**.
When ROM is mapped at an address, a write to that address stores data
in the RAM "underneath" the ROM. This is a fundamental property of the
C64's address decoding logic and is essential for:

- Copying ROM to RAM (e.g., `FOR I=A000 TO BFFF: POKE I, PEEK(I): NEXT`
  copies BASIC ROM into RAM).
- VIC-II display: the VIC-II always reads RAM, so data written to
  RAM-under-ROM is visible to the video chip even when the 6510 sees ROM.

## Conceptual Considerations for Emulation

The banking system creates a complex web of state that determines what
data appears at any given address for any given bus master (CPU vs.
VIC-II). In a Rust emulator, this suggests:

- A **central memory bus abstraction** that evaluates all banking signals
  (LORAM, HIRAM, CHAREN, GAME, EXROM, VIC bank) for each access.
- Separate read and write paths, since writes always target RAM but
  reads may return ROM or I/O data.
- A **VIC-II bus master** path that bypasses CPU banking entirely and
  reads directly from physical RAM (with Character ROM overlaid in
  banks 0 and 2).
- The PLA logic can be modeled as a pure function of
  `(address, loram, hiram, charen, game, exrom) → device_select`.