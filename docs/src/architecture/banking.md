# Memory Banking and Configuration

> See [Bus Operations](./bus_operations.md) for how banking signals
> feed into the bus arbitration and address decoding paths.

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

## Detailed Bank Switching Behavior

Bank Switching is a technique which configures the memory accessible by the CPU at any given time. As the MOS 6510 CPU can only access 2^16 = 65536 memory locations ($0000-$FFFF), banks of memory can be switched (which exposes or hides them from the CPU) in order to provide a greater overall capacity. In its original state the C64 contains 64 kBytes of RAM, 20 kBytes of ROM, and 512 bytes of Color RAM. Through the expansion port, extensions such as additional ROM and/or RAM can be added. Bank switching allows the CPU to access all of this memory (albeit non-contiguously) to execute a program.

### Caution
Caution should be taken when bank switching away from the default configuration. Because the standard interrupt handling routines in ROM access the I/O hardware (for example in order to scan the keyboard), normal interrupt operations should be disabled when the I/O hardware is banked out (not accessible by the CPU). Likewise care should be taken to avoid the CPU trying to execute code expected from ROM, when that ROM is banked out.

### Control Bits
In order to achieve bank switching, the CPU needs access to a latch. For the C64, the Programmable Logic Array (PLA) acts as a latch which enables memory configurations by setting or clearing latch bits. The PLA is connected to the CPU control lines and the expansion port which, together, dictate which areas of memory are banked.

#### CPU Control Lines
The three least significant bits in the port register (address 1) correspond to the three control lines in the port used for bank switching. To use these control lines, they must be configured as outputs, i.e. the same three least significant bits in the port's directional data register (at address 0) must be set to 1. This is the default upon power-up, but a programmer may want to make certain of this before bank switching. The control lines, in general, perform the function given in their descriptions. However, a combination of control lines are occasionally used to set a particular memory configuration.

- **LORAM (bit 0, weight 1)** is a control line which banks the 8 kByte BASIC ROM in or out of the CPU address space. Normally, this line is logically high (set to 1) for BASIC operation. If this line is logically low (cleared to 0), the BASIC ROM will typically disappear from the memory map and be replaced by 8 kBytes of RAM from $A000-$BFFF. Some exceptions to this rule exist; see the table below for a full overview.
- **HIRAM (bit 1, weight 2)** is a control line which banks the 8 kByte KERNAL ROM in or out of the CPU address space. Normally, this line is logically high (set to 1) for KERNAL ROM operation. If this line is logically low (cleared to 0), the KERNAL ROM will typically disappear from the memory map and be replaced by 8 kBytes of RAM from $E000-$FFFF. Some exceptions to this rule exist; see the table below for a full overview.
- **CHAREN (bit 2, weight 4)** is a control line which banks the 4 kByte character generator ROM in or out of the CPU address space. From the CPU point of view, the character generator ROM occupies the same address space as the I/O devices ($D000-$DFFF). When the CHAREN line is set to 1 (as is normal), the I/O devices appear in the CPU address space, and the character generator ROM is not accessible. When the CHAREN bit is cleared to 0, the character generator ROM appears in the CPU address space, and the I/O devices are not accessible. The CPU only needs to access the character generator ROM when downloading the character set from ROM to RAM. CHAREN can be overridden by other control lines in certain memory configurations. CHAREN will have no effect on any memory configuration without I/O devices. RAM will appear from $D000-$DFFF instead.

#### Expansion Port
The bank switching technique in the C64 also handles external ROM in the form of a cartridge plugged into the expansion port and is linked to the PLA by 2 lines: GAME and EXROM. These lines have been designed to allow a cartridge to map one or two 8 kByte banks of ROM into the system easily by wiring the ROM's enable pin to ROMH/ROML lines of the port. Extra logic on the cartridge can be used to implement more complex behaviour such as Freezers though.

- **GAME (pin 8)** is pulled logically high (set to 1) through internal pull-up resistor RP4. When a cartridge is attached and GAME is pulled logically low (cleared to 0), the interaction with the CPU Control Lines can enable cartridge ROM to be banked in between $8000-$BFFF and/or $E000-$EFFF. GAME can also have the effect of completely banking out all memory between $1000-$7FFF and $A000-$CFFF (Ultimax mode).
- **EXROM (pin 9)** is pulled logically high (set to 1) through internal pull-up resistor RP4. When a cartridge is attached and EXROM is pulled logically low (cleared to 0), the interaction with the CPU Control Lines can enable cartridge ROM to be banked in between $8000-$BFFF and/or $E000-$EFFF.

### Bank Switching Zones

Because there are 5 latch bits (LORAM, HIRAM, CHAREN, GAME, EXROM) that control the memory configurations, there are 2^5 = 32 possible memory arrangements or modes. However, in reality, there is duplication and so only 14 different modes actually exist. Each mode is a combination of various-sized memory banks (although are multiples of 4 kBytes or 16 pages). Additionally, the banks are pegged to specific addresses and so their location when switched in or out of memory is fixed. Consequently, the 65536 addressable memory locations are essentially divided into 7 uneven Bank Switching Zones where memory banks can be manipulated.

| Hex Address | Dec Address | Page | Size | Contents |
|---|---|---|---|---|
| $0000-$0FFF | 0-4095 | Page 0-15 | 4 kBytes | RAM (which the system requires and must appear in each mode) |
| $1000-$7FFF | 4096-32767 | Page 16-127 | 28 kBytes | RAM or is unmapped |
| $8000-$9FFF | 32768-40959 | Page 128-159 | 8 kBytes | RAM or cartridge ROM |
| $A000-$BFFF | 40960-49151 | Page 160-191 | 8 kBytes | RAM, BASIC interpretor ROM, cartridge ROM or is unmapped |
| $C000-$CFFF | 49152-53247 | Page 192-207 | 4 kBytes | RAM or is unmapped |
| $D000-$DFFF | 53248-57343 | Page 208-223 | 4 kBytes | RAM, Character generator ROM, or I/O registers and Color RAM |
| $E000-$FFFF | 57344-65535 | Page 224-255 | 8 kBytes | RAM, KERNAL ROM or cartridge ROM |

### Mode Table

This table displays the 32 modes of bank switching.

| Mode | _EXROM | _GAME | CHAREN | HIRAM | LORAM | $0000-$0FFF | $1000-$7FFF | $8000-$9FFF | $A000-$BFFF | $C000-$CFFF | $D000-$DFFF | $E000-$FFFF |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 31 | 1 | 1 | 1 | 1 | 1 | RAM | RAM | RAM | BASIC ROM | RAM | I/O | KERNAL ROM |
| 30 | 1 | 1 | 1 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | I/O | KERNAL ROM |
| 29 | 1 | 1 | 1 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | I/O | RAM |
| 28 | 1 | 1 | 1 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 27 | 1 | 1 | 0 | 1 | 1 | RAM | RAM | RAM | BASIC ROM | RAM | CHAR ROM | KERNAL ROM |
| 26 | 1 | 1 | 0 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | KERNAL ROM |
| 25 | 1 | 1 | 0 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | RAM |
| 24 | 1 | 1 | 0 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 23 | 1 | 0 | 1 | 1 | 1 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 22 | 1 | 0 | 1 | 1 | 0 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 21 | 1 | 0 | 1 | 0 | 1 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 20 | 1 | 0 | 1 | 0 | 0 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 19 | 1 | 0 | 0 | 1 | 1 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 18 | 1 | 0 | 0 | 1 | 0 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 17 | 1 | 0 | 0 | 0 | 1 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 16 | 1 | 0 | 0 | 0 | 0 | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 15 | 0 | 1 | 1 | 1 | 1 | RAM | RAM | CART ROM LO | BASIC ROM | RAM | I/O | KERNAL ROM |
| 14 | 0 | 1 | 1 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | I/O | KERNAL ROM |
| 13 | 0 | 1 | 1 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | I/O | RAM |
| 12 | 0 | 1 | 1 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 11 | 0 | 1 | 0 | 1 | 1 | RAM | RAM | CART ROM LO | BASIC ROM | RAM | CHAR ROM | KERNAL ROM |
| 10 | 0 | 1 | 0 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | KERNAL ROM |
| 9 | 0 | 1 | 0 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | RAM |
| 8 | 0 | 1 | 0 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 7 | 0 | 0 | 1 | 1 | 1 | RAM | RAM | CART ROM LO | CART ROM HI | RAM | I/O | KERNAL ROM |
| 6 | 0 | 0 | 1 | 1 | 0 | RAM | RAM | RAM | CART ROM HI | RAM | I/O | KERNAL ROM |
| 5 | 0 | 0 | 1 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | I/O | RAM |
| 4 | 0 | 0 | 1 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 3 | 0 | 0 | 0 | 1 | 1 | RAM | RAM | CART ROM LO | CART ROM HI | RAM | CHAR ROM | KERNAL ROM |
| 2 | 0 | 0 | 0 | 1 | 0 | RAM | RAM | RAM | CART ROM HI | RAM | CHAR ROM | KERNAL ROM |
| 1 | 0 | 0 | 0 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 0 | 0 | 0 | 0 | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |

#### Mode Table Notes
- The default is mode 31 (no cartridge) as all latch bits are logically high (set to 1).
- Modes 16-23 are Ultimax compatibility modes.
- "-" means "open" or "unmapped": No (internal) chip of the C64 is enabled, and the data bus can be written to by the cartridge, just like in the "CART ROM" states.
- "I/O" includes Color RAM as well ($D800-$DBFF)
- $DE00-$DFFF is special and reserved for I/O with chips on the expansion port (by wiring the IOx lines of the port to the enable pins of the chips)
- A cartridge may configure the system to one of the modes 0-23 as one or both of the expansion port latch bits will be logically low (cleared to 0) by grounding the EXROM/GAME lines.
- For cartridges that statically ground EXROM/GAME, manipulation during run-time is limited to the CPU control lines.
- This means that bank switching is confined within a group of modes (within modes 0-7, 8-15, 16-23 or 24-31).
- For example, from mode 31 the system can bank switch to mode 27 but not to mode 12.
- For example, from mode 2 the system can bank switch to mode 7 but not to mode 19.
- For cartridges that include own logic, more complex behaviour can be implemented.
- Modes 16-23 and additional cartridge logic allow arbitrary chips (be it ROMs, RAMs, or even other chips such as additional CIAs) to be visible to the CPU at $1000-$CFFF/$DE00-$FFFF.

### Optimised Mode Table

Although the results are the same as the Mode Table, this table is condensed for ease of reference.

| Modes | EXROM | GAME | CHAREN | HIRAM | LORAM | $0000-$0FFF | $1000-$7FFF | $8000-$9FFF | $A000-$BFFF | $C000-$CFFF | $D000-$DFFF | $E000-$FFFF |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 31 | 1 | 1 | 1 | 1 | 1 | RAM | RAM | RAM | BASIC ROM | RAM | I/O | KERNAL ROM |
| 30, 14 | X | 1 | 1 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | I/O | KERNAL ROM |
| 29, 13 | X | 1 | 1 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | I/O | RAM |
| 28, 24 | 1 | 1 | X | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 27 | 1 | 1 | 0 | 1 | 1 | RAM | RAM | RAM | BASIC ROM | RAM | CHAR ROM | KERNAL ROM |
| 26, 10 | X | 1 | 0 | 1 | 0 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | KERNAL ROM |
| 25, 9 | X | 1 | 0 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | CHAR ROM | RAM |
| 23, 22, 21, 20, 19, 18, 17, 16 | 1 | 0 | X | X | X | RAM | - | CART ROM LO | - | - | I/O | CART ROM HI |
| 15 | 0 | 1 | 1 | 1 | 1 | RAM | RAM | CART ROM LO | BASIC ROM | RAM | I/O | KERNAL |
| 12, 8, 4, 0 | 0 | X | X | 0 | 0 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |
| 11 | 0 | 1 | 0 | 1 | 1 | RAM | RAM | CART ROM LO | BASIC ROM | RAM | CHAR ROM | KERNAL ROM |
| 7 | 0 | 0 | 1 | 1 | 1 | RAM | RAM | CART ROM LO | CART ROM HI | RAM | I/O | KERNAL ROM |
| 6 | 0 | 0 | 1 | 1 | 0 | RAM | RAM | RAM | CART ROM HI | RAM | I/O | KERNAL ROM |
| 5 | 0 | 0 | 1 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | I/O | RAM |
| 3 | 0 | 0 | 0 | 1 | 1 | RAM | RAM | CART ROM LO | CART ROM HI | RAM | CHAR ROM | KERNAL ROM |
| 2 | 0 | 0 | 0 | 1 | 0 | RAM | RAM | RAM | CART ROM HI | RAM | CHAR ROM | KERNAL ROM |
| 1 | 0 | 0 | 0 | 0 | 1 | RAM | RAM | RAM | RAM | RAM | RAM | RAM |

#### Optimised Mode Table Notes
- Although there are 17 lines in the table, there remain 14 different modes.
- The "X" state of the latch bit is irrelevant, as the mode will be chosen regardless of being logically high (set to 1) or low (cleared to 0).
- All Mode Table Notes remain valid.