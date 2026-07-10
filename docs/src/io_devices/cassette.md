

# Cassette (Datassette) I/O

The C64's cassette interface uses the 6510 on-chip I/O port ($01)
for data and motor control, plus CIA #1 timers for bit timing. Tape
I/O routines redirect the IRQ vector to drive read/write operations.

## Hardware Signals

All cassette signals are on the 6510 I/O port ($01):

| Bit | Signal | Direction | Function |
|---|---|---|---|
| 3 | Cassette Data Output | Output | Serial data to cassette |
| 4 | Cassette Switch Sense | Input | 0 = button pressed, 1 = not pressed |
| 5 | Cassette Motor Control | Output | 1 = motor on, 0 = motor off |

## Motor Interlock

The cassette motor is not freely controllable. The system IRQ handler
(location $EA31) maintains an interlock at $C0:

1. If no button is pressed: IRQ sets bit 5 of $01 to 0 (motor off),
   sets $C0 to 0.
2. If a button is pressed and $C0 = 0: IRQ sets bit 5 to 0 (motor on).
   Software cannot override this.
3. To gain software control: write a nonzero value to $C0 *after* a
   button has been pressed. Then bit 5 of $01 can be manipulated
   freely.

## Tape Data Format

### Block Structure

Each tape block is 192 bytes and is written twice consecutively
to minimize data loss from transmission errors.

| Byte | Content |
|---|---|
| 0 | Block type identifier |
| 1–2 | Start address (low, high) — header blocks only |
| 3–4 | End address (low, high) — header blocks only |
| 5–191 | Filename (padded to 187 bytes) or data (191 bytes) |

### Block Type Identifiers

| Value | Type |
|---|---|
| 1 | Relocatable program header (SA=0 on SAVE) |
| 3 | Non-relocatable program header (SA=1 on SAVE) |
| 4 | Data file header |
| 2 | Data storage block |
| 5 | End-of-tape marker |

### Header Block Detail

Header blocks contain:
- 1 byte: type identifier
- 2 bytes: start RAM address
- 2 bytes: end RAM address
- 187 bytes: filename (padded with spaces)

### Data Block Detail

Data blocks contain:
- 1 byte: type identifier ($02)
- 191 bytes: data

## Tape I/O and System State

During tape I/O:

1. The screen is blanked (bit 4 of $D011 set to 0) to prevent
   VIC-II bus stealing from disrupting timing.
2. The IRQ vector ($0314–$0315) is redirected to the tape read/write
   routine.
3. CIA #1 Timer B is used for bit timing.
4. The normal keyboard scan, clock update, and cursor blink are
   suspended.
5. A tape error log is maintained at $0100–$013E (62 bytes) to
   track which bytes were incorrectly received on the first pass,
   for correction on the second pass.

## Cassette Buffer

The cassette I/O buffer is at `$033C–$03FB` (192 bytes). The pointer to this buffer is at `$00B2–$00B3` (`TAPE1`), defaulting to `$033C`.

* **Program files:** Only the header uses the buffer; program data is transferred directly to/from RAM.
* **Data files:** All data is buffered through this buffer.

## Tape Input Error Log (Page 1)

Because each physical tape block is written to tape twice consecutively to mitigate transmission errors, the Kernal maintains a dedicated tape error log in Page 1 from `$0100–$013E` (62 bytes). 

During the first read pass of a block, if a parity or checksum error is detected in a byte, its offset is logged as an index in this Page 1 table. During the second read pass, the Kernal reads the duplicate block, references this error table, and corrects the bad bytes in-place before copying the verified data to its final destination in RAM.

## Software Motor Control

* **Motor Interlock Flag (`$C0`):** This zero-page location prevents accidental software control of the motor if no physical buttons are pressed on the Datassette.
* **Keyscan Scan Override:** The standard keyscan routine continuously checks the Datassette switch sense line (Bit 4 of `$01`). If no keys are pressed, it forces Bit 5 of `$01` to 0 (motor off) and clears the interlock (`$C0 = 0`).
* **Direct Control:** To override this automatic behavior and control the motor programmatically, software must first ensure a physical button is depressed on the recorder, write a non-zero value to `$C0` to disable the automatic IRQ keyscan override, and then toggle Bit 5 of `$01` directly to switch the tape motor on (`1`) or off (`0`).

When not in use for tape I/O, this area is available for ML routines
or graphics data (sprite blocks 13 and 14 at $0340–$03BF).

## Timing Constants

CIA #1 Timer A is adjusted for tape bit timing during I/O. The
adjustable timing constant at $0092 (SVXT) allows for slight speed
variations between tapes. Location $00B0 (CMPO) determines the value
of this constant.

## LOAD/SAVE Behavior

| Secondary Address | LOAD Behavior | SAVE Behavior |
|---|---|---|
| 0 (or even) | Relocating: loads to start-of-BASIC pointer | Relocatable header |
| 1 (or odd) | Non-relocating: loads to address in file header | Non-relocatable header |
| 2 (added to 1) | — | End-of-tape marker written after file |

## Emulation Notes

### Timing-Critical Operation

Tape I/O is extremely timing-sensitive. The read/write routines are
driven by CIA #1 Timer B interrupts and depend on precise cycle
timing. If the emulator runs real Kernal ROM, tape I/O will work
as long as CIA timers and IRQ routing are correct.

### Screen Blanking

The emulator must honor the screen blanking during tape I/O. When
bit 4 of $D011 is cleared, no bad lines or sprite DMA should occur,
and the CPU should run at full speed.

### Motor and Switch Sense

The emulator should model the cassette switch sense (bit 4 of $01)
and motor control (bit 5 of $01) with the interlock behavior
described above. If no virtual cassette is attached, bit 4 should
read as 1 (no button pressed).

### Error Log

The tape error log at $0100–$013E is used by the Kernal's tape
routines. The emulator does not need to implement this separately
if running real Kernal ROM — it is handled by the ROM code using
normal RAM.