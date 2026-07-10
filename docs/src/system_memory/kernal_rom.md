# Kernal ROM ($E000–$FFFF)

The Kernal ROM is an 8K ROM containing the C64's Operating System. It
handles all I/O, device communication, interrupt handling, system
initialization, and provides a jump table for stable API access.

## ROM Organization

| Address Range | Size | Content |
|---|---|---|
| $E000–$E042 | ~67 B | Continuation of EXP routine (from BASIC ROM) |
| $E043–$E08C | ~74 B | Polynomial evaluation (POLY1, POLY2) |
| $E08D–$E096 | 10 B | RND constants (RMULC, RADDC) |
| $E097–$E0F8 | ~102 B | RND function |
| $E0F9–$E129 | ~49 B | BASIC I/O call wrapper |
| $E12A–$E155 | ~44 B | SYS, SAVE, VERIFY, LOAD |
| $E156–$E1D3 | ~126 B | OPEN, CLOSE, parameter setup |
| $E264–$E33D | ~218 B | COS, SIN, TAN, ATN + constants |
| $E33E–$E3A1 | ~100 B | Warm start, error handler, cold start |
| $E3A2–$E3BA | 25 B | CHRGET source + RND seed |
| $E3BF–$E4FF | ~325 B | BASIC initialization + patches |
| $E4DA–$FF5A | ~6.7K | Kernal I/O routines |
| $FF5B–$FF7F | 37 B | PAL/NTSC detection patch |
| $FF80 | 1 B | Kernal version identifier byte |
| $FF81–$FFF5 | 117 B | Jump table (39 entries) |
| $FFFA–$FFFF | 6 B | Hardware vectors |

## Kernal Jump Table ($FF81–$FFF5)

The jump table provides stable entry points for Kernal routines. Each
entry is 3 bytes: JMP instruction ($4C) followed by the 2-byte target
address.

| Address | Name | Target | Function |
|---|---|---|---|
| $FF81 | CINT | $FF5B | Initialize screen editor and video chip |
| $FF84 | IOINIT | $FDA3 | Initialize CIA I/O devices |
| $FF87 | RAMTAS | $FD50 | Initialize RAM, tape buffer, screen |
| $FF8A | RESTOR | $FD15 | Restore default I/O vectors |
| $FF8D | VECTOR | $FD1A | Read/set I/O vector table |
| $FF90 | SETMSG | $FE18 | Set Kernal message control flag |
| $FF93 | SECOND | $EDB9 | Send secondary address after LISTEN |
| $FF96 | TKSA | $EDC7 | Send secondary address after TALK |
| $FF99 | MEMTOP | $FE25 | Read/set top of memory pointer |
| $FF9C | MEMBOT | $FE34 | Read/set bottom of memory pointer |
| $FF9F | SCNKEY | $EA87 | Scan the keyboard |
| $FFA2 | SETTMO | $FE21 | Set time-out flag for IEEE bus |
| $FFA5 | ACPTR | $EE13 | Input byte from serial bus |
| $FFA8 | CIOUT | $EDDD | Output byte to serial bus |
| $FFAB | UNTLK | $EDEF | Command serial bus device to UNTALK |
| $FFAE | UNLSN | $EDFE | Command serial bus device to UNLISTEN |
| $FFB1 | LISTEN | $ED0C | Command serial bus device to LISTEN |
| $FFB4 | TALK | $ED09 | Command serial bus device to TALK |
| $FFB7 | READST | $FE07 | Read I/O status word |
| $FFBA | SETLFS | $FE00 | Set logical file parameters |
| $FFBD | SETNAM | $FDF9 | Set filename parameters |
| $FFC0 | OPEN | $F34A | Open a logical file (via RAM vector $031A) |
| $FFC3 | CLOSE | $F291 | Close a logical file (via $031C) |
| $FFC6 | CHKIN | $F20E | Define an input channel (via $031E) |
| $FFC9 | CHKOUT | $F250 | Define an output channel (via $0320) |
| $FFCC | CLRCHN | $F333 | Restore default devices (via $0322) |
| $FFCF | CHRIN | $F157 | Input a character (via $0324) |
| $FFD2 | CHROUT | $F1CA | Output a character (via $0326) |
| $FFD5 | LOAD | $F49E | Load from a device (via $0330) |
| $FFD8 | SAVE | $F5DD | Save to a device (via $0332) |
| $FFDB | SETTIM | $F6E4 | Set the software clock |
| $FFDE | RDTIM | $F6DD | Read the software clock |
| $FFE1 | STOP | $F6ED | Check STOP key (via $0328) |
| $FFE4 | GETIN | $F13E | Get a character (via $032A) |
| $FFE7 | CLALL | $F32F | Close all files (via $032C) |
| $FFEA | UDTIM | $F69B | Update the software clock |
| $FFED | SCREEN | $E505 | Read screen rows and columns |
| $FFF0 | PLOT | $E50A | Read/set cursor position |
| $FFF3 | IOBASE | $E500 | Read base address of I/O devices |

The first 15 entries ($FFC0–$FFEA) are the same across all Commodore
machines from the original PET onward.

## Hardware Vectors ($FFFA–$FFFF)

| Address | Vector | Target | Trigger |
|---|---|---|---|
| $FFFA–$FFFB | NMI | $FE43 | NMI line asserted |
| $FFFC–$FFFD | RESET | $FCE2 | Power-on or RESET line |
| $FFFE–$FFFF | IRQ/BRK | $FF48 | IRQ line asserted or BRK opcode |

## Kernal Version Identifier ($FF80)

| Value | Kernal Version |
|---|---|
| $AA (170) | Original/early Kernal |
| $00 (0) | Later/revised Kernal |
| $64 (100) | PET 64 (educational model) |

## Power-On Reset Sequence ($FCE2)

1. Set Interrupt disable flag (SEI)
2. Set stack pointer
3. Clear Decimal mode flag (CLD)
4. Check for autostart cartridge at $8004–$8008 (looking for bytes $C3,
   $C2, $CD, $38, $30 — inverse "CBM80")
5. If cartridge found: jump through cold-start vector at $8000
6. If no cartridge: call IOINIT, RAMTAS, RESTOR, CINT
7. Clear Interrupt disable flag (CLI)
8. Enter BASIC through cold-start vector at $A000

### IOINIT ($FDA3)

Initializes CIA chips:
- Sets CIA #1 Timer A latch to $4118 (NTSC) or $4295 (PAL) for ~60/50 Hz IRQ
- Starts Timer A in continuous mode
- Enables Timer A interrupt
- Sets SID volume to 0 ($D418 = $00)

### RAMTAS ($FD50)

- Clears pages 0, 2, and 3 to zeros
- Sets tape buffer pointer to $033C
- Tests RAM from $0400 up to $A000
- Sets memory bottom pointer ($0281–$0282) to $0800
- Sets memory top pointer ($0283–$0284) to $A000
- Sets screen page pointer ($0288) to $04 (screen at $0400)

### RESTOR ($FD15)

Copies the default I/O vector table from ROM ($FD30) to RAM
($0314–$0333).

### CINT ($FF5B)

- Initializes VIC-II registers from table at $ECB9
- Sets keyboard as input device, screen as output device
- Initializes cursor blink variables
- Builds screen line link table
- Clears screen and Color RAM
- Performs PAL/NTSC detection via raster interrupt at line 311
- Sets CIA #1 Timer A prescaler based on PAL/NTSC result

## PAL/NTSC Detection

The CINT routine sets a raster interrupt for line 311 and checks if it
fires. NTSC systems have only 262 scan lines, so the interrupt at line 311
can only fire on PAL systems (312 lines).

| Result | $02A6 Value | Timer A Latch |
|---|---|---|
| NTSC | 0 | $4118 (16664) → ~61.4 Hz |
| PAL | 1 | $4295 (17045) → ~57.8 Hz |

## Kernal Patches

Later Kernal versions include two patch areas with code not in the
original:

### $E4AD–$E4FF (lower patch)
- Fix for CHKOUT preserving .A register
- Baud rate table for PAL systems ($E4EC)
- 35 unused bytes ($AA fill at $E4B7)

### $FF5B–$FF7F (upper patch)
- PAL/NTSC detection code
- Timer prescaler selection
- Kernal version byte at $FF80

### Color RAM Initialization Change

**Original Kernal:** Clears Color RAM to $01 (white) when screen is
cleared. POKE'd characters are visible immediately.

**Later Kernal:** Clears Color RAM to the current background color
($D021 value) when screen is cleared. POKE'd characters are invisible
(same color as background) until Color RAM is explicitly set.

## Banking

The Kernal ROM is visible at $E000–$FFFF when HIRAM=1 (bit 1 of $01 = 1).
When HIRAM=0, RAM is visible instead.

**Write-through:** CPU writes always go to RAM, even when Kernal ROM is
visible.

**CHAREN dependency:** The CHAREN signal (bit 2 of $01) only has effect
when HIRAM=1. When HIRAM=0, the $D000 region is always I/O regardless of
CHAREN.

## Emulation Notes

### ROM Image

The Kernal ROM is an 8K (8192-byte) image, typically loaded from a file
named `kernal` or `kernal.bin`. The last 6 bytes ($FFFA–$FFFF) contain
the hardware vectors and must be readable even when HIRAM=0 (the 6510
always reads vectors from ROM at these addresses — actually, when HIRAM=0,
the vectors come from RAM, which is why software that switches out the
Kernal must provide its own vectors).

### Correct: Vector Reads

When HIRAM=0, reads from $FFFA–$FFFF return RAM values. The 6510 reads
its interrupt vectors from the address bus, which goes through the PLA.
If Kernal ROM is switched out, the vectors come from RAM underneath. This
means software that switches out the Kernal must write valid vectors to
RAM at $FFFA–$FFFF.

### Interrupt Handling

If the emulator runs real Kernal ROM:
- The IRQ vector at $FFFE–$FFFF points to $FF48 (Kernal IRQ entry)
- $FF48 pushes registers, checks B flag, dispatches through $0314 (IRQ)
  or $0316 (BRK)
- The NMI vector at $FFFA–$FFFB points to $FE43 (Kernal NMI entry)
- $FE43 dispatches through $0318 (NMI vector)

The emulator must ensure these vectors are correctly mapped at all times.

### RND(0) Implementation Defects

The RND(0) function reads four bytes as a seed:
- CIA #1 Timer A high byte (range 0–66, not full 0–255)
- CIA #1 TOD tenths register (always 0 — clock never started)
- CIA #1 TOD seconds register (always 0 — clock never started)
- A fourth byte from the timer/registers

Three problems make RND(0) unsuitable for generating random sequences:

1. **TOD clock never started:** The Kernal never writes to the CIA #1
   TOD tenths register, so the clock remains stopped at zero. Both
   TOD registers always read 0.

2. **BCD format limitation:** Even if the TOD clock were running, it
   uses BCD format where nybble values only go 0–9, not 0–15. This
   limits the range of byte values produced.

3. **Timer A high byte range:** Timer A is latched at 17045 (PAL) or
   16664 (NTSC), so the high byte only ranges from 0 to 66 (PAL) or
   0 to 65 (NTSC). This is far less than the full 0–255 range.

**Workaround:** Use `RND(-TI)` to seed from the software jiffy clock,
or `RND(-RND(0))` for a slightly more random (but still imperfect) seed.


### Conceptual Considerations for Emulation

In a Rust emulator, the Kernal ROM should be loaded as a fixed byte array.
The memory bus must check HIRAM for every access to $E000–$FFFF and route
to ROM or RAM accordingly. The hardware vectors at $FFFA–$FFFF are part of
the Kernal ROM image and are automatically accessible when HIRAM=1. When
HIRAM=0, they come from RAM, which the CPU must have pre-populated with
valid vector addresses.