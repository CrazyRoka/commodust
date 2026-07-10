# Pages 2 and 3 ($0200–$03FF)

Pages 2 and 3 contain Operating System and BASIC working storage:
I/O file tables, the keyboard buffer, RS-232 pseudo-registers, the
cassette buffer, and RAM vectors for Kernal and BASIC routines.

## Memory Map

### $0200–$0258: BUF — BASIC Input Buffer (89 bytes)

Stores characters typed in immediate mode, and data received via
INPUT and GET. The screen editor allows a maximum of 80 characters
per logical line, plus a 0 terminator. The extra 8 bytes (88–96,
relative to buffer start) are normally unused and available to the
programmer.

**Emulation note:** INPUT and GET are illegal in immediate mode
because they share this buffer with the immediate-mode statement
itself.

### $0259–$0262: LAT — Logical File Number Table (10 bytes)

Holds the logical file numbers of all currently open I/O files.
Indexed by $0098 (LDTND). Each OPEN adds an entry; each CLOSE removes
one and shifts higher entries down.

### $0263–$026C: FAT — Device Number Table (10 bytes)

Holds the device number corresponding to each open file. The entry
at position N corresponds to the file at position N in the LAT.

### $026D–$0276: SAT — Secondary Address Table (10 bytes)

Holds the secondary address for each open file. Position N
corresponds to file N in the LAT.

### $0277–$0280: KEYD — Keyboard Buffer (10 bytes)

FIFO queue holding PETASCII values of keys pressed but not yet
consumed by the application. The system IRQ handler deposits
characters here; BASIC's GET and INPUT retrieve them.

- Maximum size controlled by $0289 (default 10).
- Character count tracked at $00C6 (NDX).
- Characters beyond the maximum are ignored.
- The "dynamic keyboard" technique involves POKEing characters and
  the count here to simulate keyboard input from a program.

### $0281–$0282: MEMSTR — OS Start of Memory Pointer

Set to $0800 at power-on by RAMTAS. BASIC uses this to set TXTTAB
($2B). Can be read/set via the Kernal MEMBOT routine ($FE34).

### $0283–$0284: MEMSIZ — OS End of Memory Pointer

Set to $A000 at power-on (first address after contiguous RAM).
Lowered by 512 bytes when RS-232 device is opened (for I/O buffers).
Can be read/set via the Kernal MEMTOP routine ($FE25).

### $0285: TIMOUT — IEEE Time-Out Flag

Used only with the external IEEE interface card. See Kernal SETTMO
routine ($FE21).

### $0286: COLOR — Current Foreground Color

The color value placed into Color RAM whenever a character is
PRINTed. Default: 14 (light blue). Can be changed by CTRL+number
keys, Commodore+number keys, CHR$ codes, or direct POKE.

### $0287: GDCOL — Color of Character Under Cursor

Saves the original color code of the character at the cursor
position so it can be restored when the cursor moves.

### $0288: HIBASE — Screen Memory Page

Page number of the start of screen memory. Default: 4 (meaning
screen starts at $0400 = 4 × 256). The OS PRINT routines use this
to determine where to write characters. Must be updated when
relocating the screen.

### $0289: XMAX — Maximum Keyboard Buffer Size

Default: 10. Can be increased up to 15 (at the risk of overwriting
the memory pointers at $0281–$0284).

### $028A: RPTFLG — Key Repeat Flag

| Value | Behavior |
|---|---|
| $00 (0) | Only cursor keys, INST/DEL, and space repeat (default) |
| $80 (128) | All keys repeat |
| $40 (64) | No keys repeat |

### $028B: KOUNT — Key Repeat Delay Counter

Starts at 6. After the initial repeat delay ($028C reaches 0), this
counts down. When it reaches 0, the key repeats and it resets to 4
(approximately 15 repeats/second).

### $028C: DELAY — Initial Key Repeat Delay

Starts at 16, counted down every 1/60 second while a key is held.
When it reaches 0, $028B takes over. Total time before first repeat:
approximately 1/3 second.

### $028D: SHFLAG — SHIFT/CTRL/Logo Flag

| Bit | Value | Key |
|---|---|---|
| 0 | 1 | SHIFT pressed |
| 1 | 2 | Commodore logo pressed |
| 2 | 4 | CTRL pressed |

Values are OR'd together if multiple modifier keys are held. Used
to select the appropriate keyboard decode table.

### $028E: LSTSHF — Previous SHIFT/CTRL/Logo State

Used for debouncing the SHIFT+logo character-set toggle.

### $028F–$0290: KEYLOG — Vector to Keyboard Table Setup Routine

Points to the routine that reads $028D and sets $F5–$F6 (KEYTAB) to
point to the appropriate decode table. Default: $EB48. Can be
redirected to add custom key preprocessing.

### $0291: MODE — Character Set Toggle Enable Flag

| Value | Behavior |
|---|---|
| $00 (0) | SHIFT+logo toggles character set (default) |
| $80 (128) | Character set toggle disabled |

Also controlled by CHR$(8) (disable) and CHR$(9) (enable).

### $0292: AUTODN — Screen Scroll Enable

| Value | Behavior |
|---|---|
| $00 (0) | Scroll enabled (default) |
| Nonzero | Scroll disabled |

Set to nonzero when characters are in the keyboard buffer (to
prevent scroll during dynamic keyboard operations).

### $0293: M51CTR — RS-232 Control Register (Mock 6551)

Emulates the control register of a 6551 ACIA. It determines word length, stop bits, and baud rate.

* **Bit 7 (Stop Bits):**
  * `0`: 1 Stop Bit
  * `1`: 2 Stop Bits (forced to 1 stop bit under standard software execution)
* **Bits 6–5 (Word Length):**
  * `00`: 8 Data Bits
  * `01`: 7 Data Bits
  * `10`: 6 Data Bits
  * `11`: 5 Data Bits
* **Bit 4:** Unused
* **Bits 3–0 (Baud Rate Selection):**
  * `0000`: Nonstandard / User-defined (Not implemented in ROM)
  * `0001`: 50 Baud
  * `0010`: 75 Baud
  * `0011`: 110 Baud
  * `0100`: 134.5 Baud
  * `0101`: 150 Baud
  * `0110`: 300 Baud
  * `0111`: 600 Baud
  * `1000`: 1200 Baud
  * `1001`: 1800 Baud
  * `1010`: 2400 Baud
  * `1011–1111`: 3600 to 19200 Baud (Unimplemented due to CPU software timing limits)

### $0294: M51CDR — RS-232 Command Register (Mock 6551)

Emulates the command register of a 6551 ACIA. It manages parity, duplex mode, and handshaking.

* **Bits 7–5 (Parity Selection):**
  * `xx0`: No Parity Generated or Checked
  * `001`: Odd Parity Transmitted and Checked
  * `011`: Even Parity Transmitted and Checked
  * `101`: Mark Parity Transmitted (No Parity Checked)
  * `111`: Space Parity Transmitted (No Parity Checked)
* **Bit 4 (Duplex Mode):**
  * `0`: Full Duplex
  * `1`: Half Duplex
* **Bits 3–1:** Unused
* **Bit 0 (Handshake Protocol):**
  * `0`: 3-Line Handshake
  * `1`: X-Line / Full Handshake

### $0295–$0296: M51AJB — Nonstandard Baud Rate Timing

Intended for user-defined baud rates. Formula:
`value = (CLOCK / BAUD / 2) - 100`. Not implemented in the current
Kernal.

### $0297: RSSTAT — RS-232 Status Register

| Bit | Value | Meaning |
|---|---|---|
| 7 | 128 | Break detected |
| 6 | 64 | DSR signal missing |
| 5 | — | Unused |
| 4 | 16 | CTS signal missing |
| 3 | 8 | Receiver buffer empty |
| 2 | 4 | Receiver buffer overrun |
| 1 | 2 | Framing error |
| 0 | 1 | Parity error |

Reading this register (via ST or READST) clears it to 0.

### $0298: BITNUM — RS-232 Bits Remaining

Number of zero-padding bits to add to the current character to
reach the specified word length.

### $0299–$029A: BAUDOF — RS-232 Bit Time Prescaler

Value loaded into CIA #2 Timer A/B for RS-232 baud rate generation.

### $029B: RIDBE — RS-232 Receive Buffer End Index

### $029C: RIDBS — RS-232 Receive Buffer Start Index

### $029D: RODBS — RS-232 Transmit Buffer Start Index

### $029E: RODBE — RS-232 Transmit Buffer End Index

These four indices manage the two 256-byte wraparound FIFO buffers
for RS-232 receive and transmit.

### $029F–$02A0: IRCLTMP — IRQ Vector Save (Cassette I/O)

Saves the normal IRQ vector while cassette I/O routines redirect
the IRQ for their own timing.

### $02A1: ENABL — RS-232 NMI Interrupt Flags

| Bit | Value | Meaning |
|---|---|---|
| 0 | 1 | System is transmitting |
| 1 | 2 | System is receiving |
| 4 | 16 | Waiting for receiver edge |

### $02A2–$02A5: Cassette I/O Save Areas

Temporary storage for CIA #1 register values during cassette read
operations.

### $02A6: PAL/NTSC Flag

| Value | System |
|---|---|
| 0 | NTSC (262 scan lines, ~60 Hz) |
| 1 | PAL (312 scan lines, ~50 Hz) |

Set at power-on by the CINT routine, which sets a raster interrupt
for line 311 and checks if it fires. If it does, the system is PAL.

### $02A7–$02FF: Unused (89 bytes)

Free for machine language programs or graphics data. In the default
VIC-II bank (bank 0), locations $02A7–$02FF can hold sprite data
block 11 (partial) without conflicting with BASIC or system
variables.

## BASIC Indirect Vector Table ($0300–$030B)

These vectors allow BASIC routines to be redirected or extended.

| Address | Label | Default Target | Function |
|---|---|---|---|
| $0300–$0301 | IERROR | $E38B | Error message handler |
| $0302–$0303 | IMAIN | $A483 | Main BASIC loop |
| $0304–$0305 | ICRNCH | $A57C | Tokenize (crunch) ASCII to tokens |
| $0306–$0307 | IQPLOP | $A71A | List tokens as ASCII text |
| $0308–$0309 | IGONE | $A7E4 | Execute next token |
| $030A–$030B | IEVAL | $AE86 | Evaluate arithmetic expression |

Each vector points one byte past a JMP instruction in ROM that
indirectly jumps through the corresponding RAM vector. This design
allows user code to intercept any of these routines by changing
the RAM vector.

### Register Storage for SYS ($030C–$030F)

| Address | Label | Register |
|---|---|---|
| $030C | SAREG | .A (Accumulator) |
| $030D | SXREG | .X |
| $030E | SYREG | .Y |
| $030F | SPREG | .P (Status) |

Before SYS, each register is loaded from its storage location. After
the ML routine returns (RTS), the new register values are stored
back. This does not apply to USR.

### USR Function ($0310–$0312)

| Address | Label | Content |
|---|---|---|
| $0310 | USRPOK | $4C (JMP instruction) |
| $0311–$0312 | USRADD | Target address (low, high) |

Default target is the error handler. Must be set before calling USR.
The parameter is passed in FAC1 ($61–$66) as a floating-point number.
The return value must be placed in FAC1.

## Kernal Indirect Vectors ($0314–$0333)

These 16 vectors (2 bytes each) point to Kernal I/O routines. They
are initialized by RESTOR from a ROM table and can be changed to
support custom I/O devices.

| Address | Label | Default | Function |
|---|---|---|---|
| $0314–$0315 | CINV | $EA31 | IRQ interrupt handler |
| $0316–$0317 | CBINV | $FE66 | BRK instruction handler |
| $0318–$0319 | NMINV | $FE47 | NMI interrupt handler |
| $031A–$031B | IOPEN | $F34A | OPEN routine |
| $031C–$031D | ICLOSE | $F291 | CLOSE routine |
| $031E–$031F | ICHKIN | $F20E | CHKIN routine |
| $0320–$0321 | ICKOUT | $F250 | CHKOUT routine |
| $0322–$0323 | ICLRCH | $F333 | CLRCHN routine |
| $0324–$0325 | IBASIN | $F157 | CHRIN routine |
| $0326–$0327 | IBSOUT | $F1CA | CHROUT routine |
| $0328–$0329 | ISTOP | $F6ED | STOP key check |
| $032A–$032B | IGETIN | $F13E | GETIN routine |
| $032C–$032D | ICLALL | $F32F | CLALL routine |
| $032E–$032F | USRCMD | $FE66 | User command (unused, points to BRK) |
| $0330–$0331 | ILOAD | $F49E | LOAD routine |
| $0332–$0333 | ISAVE | $F5DD | SAVE routine |

### $0334–$033B: Unused (8 bytes)

Free for user vectors or data.

## Cassette I/O Buffer ($033C–$03FB)

192-byte buffer for tape data. The first byte of any block
identifies its type:

| Value | Block Type |
|---|---|
| 1 | Relocatable program header |
| 3 | Non-relocatable program header |
| 4 | Data file header |
| 2 | Data storage block |
| 5 | End-of-tape marker |

**Header block format:** 1 byte type + 2 bytes start address + 2
bytes end address + 187 bytes filename (padded with spaces).

**Data block format:** 1 byte type + 191 bytes data.

Program file data is transferred directly to/from RAM after the
header is read from this buffer. Data file data is buffered here.

When not in use for tape I/O, this area is available for machine
language routines or graphics data (e.g., sprite data blocks 13
and 14 at $0340–$03BF).

### $03FC–$03FF: Unused (4 bytes)

## Emulation Notes

### RS-232 Buffer Management

The RS-232 system allocates two 256-byte FIFO buffers at the top
of BASIC memory when device 2 is opened. The buffer indices at
$029B–$029E are wraparound pointers into these buffers. An emulator
implementing RS-232 at the Kernal level must model these buffers
and the NMI-driven bit-bang send/receive routines.

### Keyboard Buffer and Dynamic Keyboard

The keyboard buffer at $0277 with its count at $00C6 is a critical
interface for both real software and emulators. Programs that use
the "dynamic keyboard" technique (POKEing characters into the
buffer to simulate typing) rely on exact buffer behavior. An
emulator must:

- Accept characters from the host keyboard into this buffer.
- Honor the $0289 maximum.
- Update $00C6 correctly.
- Allow software to read and clear the buffer.

### PAL/NTSC Detection

The $02A6 flag is set by testing whether a raster interrupt at line
311 fires. An emulator should set this flag based on its configured
video standard, as it affects the CIA #1 Timer A prescaler value
(17045 for PAL, 16664 for NTSC).

### Dynamic Keyboard Technique

Software can simulate keyboard input by POKEing PETASCII characters
into the keyboard buffer and setting the count. This enables
programs to modify themselves during execution.

**Procedure:**
1. POKE desired PETASCII characters (including cursor controls and
   carriage returns) into $0277–$0280.
2. POKE the character count into $00C6 (NDX).
3. Position cursor on screen where input should begin.
4. Execute END (or similar) to return to direct mode.
5. The keyboard buffer is processed as if the keys were typed.

**Example applications:**
- Adding, deleting, or modifying program lines during execution
- Merging programs from tape/disk by simulating LOAD and RUN
- Entering numbered DATA statements automatically
- Creating self-modifying programs

**Important:** The screen must show the lines to be entered, properly
positioned, before the buffer is processed. The carriage returns in
the buffer execute these lines as if typed in immediate mode.