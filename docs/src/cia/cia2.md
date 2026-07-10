# CIA #2 — Serial Bus, RS-232, VIC Bank

**Base address: $DD00 ($DD00–$DD0F)**
**Interrupt line: 6510 NMI**

## Port A ($DD00) — Serial Bus, RS-232, VIC Bank

**Default DDR: $3F (%00111111)** — bits 0–5 output, bits 6–7 input.

| Bit | Function | Direction |
|---|---|---|
| 0 | VIC bank select bit 0 | Output |
| 1 | VIC bank select bit 1 | Output |
| 2 | RS-232 data output (Sout) / User Port pin M | Output |
| 3 | Serial bus ATN signal output | Output |
| 4 | Serial bus clock output | Output |
| 5 | Serial bus data output | Output |
| 6 | Serial bus clock input | Input |
| 7 | Serial bus data input | Input |

### VIC-II Bank Selection (bits 0–1)

| Bits 1:0 | Bank | VIC Address Range |
|---|---|---|
| 00 | 3 | $C000–$FFFF |
| 01 | 2 | $8000–$BFFF |
| 10 | 1 | $4000–$7FFF |
| 11 | 0 | $0000–$3FFF (default) |

**Note the inverted mapping:** 11 = bank 0, 00 = bank 3.

Changing the VIC bank requires coordination with the VIC Memory Control
Register ($D018) and the OS screen pointer ($0288) to ensure the screen
remains visible.

## Port B ($DD01) — RS-232 / User Port

**Default DDR: $00 (all inputs)**. Bits 1–2 are set to output when
RS-232 device 2 is opened.

| Bit | RS-232 Function | User Port Pin |
|---|---|---|
| 0 | Received Data (SIN) | C |
| 1 | Request to Send (RTS) | D |
| 2 | Data Terminal Ready (DTR) | E |
| 3 | Ring Indicator (RI) | F |
| 4 | Carrier Detect (DCD) | H |
| 5 | — | J |
| 6 | Clear to Send (CTS) | K |
| 7 | Data Set Ready (DSR) | L |

Timer A can output to bit 6, Timer B to bit 7 (same as CIA #1).

## NMI vs. IRQ

CIA #2's interrupt line is connected to the 6510's **NMI** (Non-Maskable
Interrupt) line, not the IRQ line. This has important implications:

1. NMI **cannot be disabled** by the 6510's interrupt disable flag (SEI).
   It can only be disabled at the CIA level via the ICR mask.
2. The RESTORE key is also wired to the NMI line. The NMI handler must
   distinguish between CIA #2 interrupts and RESTORE key presses.
3. RS-232 I/O uses CIA #2 timers to drive NMI-based send/receive
   routines.

## Serial Bus Protocol

The Commodore serial bus is a daisy-chainable bus using 3 signal lines
plus ATN:

| Line | CIA #2 Port A Bit | Direction | Function |
|---|---|---|---|
| DATA | Bit 5 (out) / Bit 7 (in) | Bidirectional | Data signal |
| CLOCK | Bit 4 (out) / Bit 6 (in) | Bidirectional | Clock signal |
| ATN | Bit 3 (out) | Output | Attention signal |

The serial bus protocol is used to communicate with the 1541 disk drive
and serial printers. Data is transferred one bit at a time, making it
approximately 8× slower than a parallel bus.

## RS-232 Implementation

When device 2 (RS-232) is opened, the Kernal:

1. Allocates two 256-byte FIFO buffers at the top of BASIC memory
   (512 bytes total), lowering the top-of-memory pointer.
2. Sets up CIA #2 Timer A and Timer B to generate NMI interrupts at
   the baud rate for transmit and receive.
3. Uses the NMI handler to bit-bang serial data through Port B.

The RS-232 "registers" at $0293–$0297 are software-emulated
pseudo-registers that mimic a 6551 UART chip.