# CIA (6526 Complex Interface Adapter) Overview

The C64 contains two 6526 CIA chips. Each provides:

- Two 8-bit bidirectional data ports (A and B)
- Two 16-bit programmable timers (A and B) with one-shot and continuous modes
- A Time-of-Day (TOD) clock with alarm (BCD format)
- A serial shift register
- A FLAG pin for external interrupt triggering
- 5 independent interrupt sources

## CIA Register Map (per chip)

| Offset | Register | Description |
|---|---|---|
| $00 | PRA | Data Port A |
| $01 | PRB | Data Port B |
| $02 | DDRA | Data Direction Register A |
| $03 | DDRB | Data Direction Register B |
| $04 | TIMALO | Timer A low byte |
| $05 | TIMAHI | Timer A high byte |
| $06 | TIMBLO | Timer B low byte |
| $07 | TIMBHI | Timer B high byte |
| $08 | TODTEN | TOD tenths of seconds (BCD) |
| $09 | TODSEC | TOD seconds (BCD) |
| $0A | TODMIN | TOD minutes (BCD) |
| $0B | TODHRS | TOD hours (BCD, bit 7 = AM/PM) |
| $0C | SDR | Serial Data Register |
| $0D | ICR | Interrupt Control Register |
| $0E | CRA | Control Register A |
| $0F | CRB | Control Register B |

Only 4 address bits (A0–A3) are decoded, so every 16-byte block within
a chip's 256-byte region is a mirror.

## CIA #1 vs. CIA #2

| Feature | CIA #1 ($DC00) | CIA #2 ($DD00) |
|---|---|---|
| Port A devices | Keyboard columns, Joystick 2, paddle select | Serial bus, RS-232 out, VIC bank |
| Port B devices | Keyboard rows, Joystick 1, paddle fire | RS-232 signals, User Port |
| Interrupt line | Connected to 6510 **IRQ** | Connected to 6510 **NMI** |
| FLAG pin | Cassette read line | User Port pin B |
| System use | Keyboard scan, 1/60s IRQ, tape timing | RS-232 timing, serial bus |

## Interrupt Control Register ($0D)

This register has dual read/write behavior:

### Read

| Bit | Source |
|---|---|
| 0 | Timer A reached zero |
| 1 | Timer B reached zero |
| 2 | TOD reached alarm time |
| 3 | Serial shift register completed |
| 4 | FLAG line pulled low |
| 5–6 | Unused |
| 7 | Any enabled interrupt occurred (OR of enabled bits 0–4) |

**Reading clears all bits.** Copy to RAM if multiple tests are needed.

### Write

| Bit 7 | Effect |
|---|---|
| 1 | Bits written with 1 are **set** (enabled) |
| 0 | Bits written with 1 are **cleared** (disabled) |

Bits written with 0 are unaffected.

Example: Writing $81 (bit 7=1, bit 0=1) enables Timer A interrupt.
Writing $7F (bit 7=0, bits 0–6=1) disables all interrupts.

## Timer Operation

Each timer counts down from a 16-bit latch value at the system clock
rate (1 cycle per count). When it reaches zero:

1. The corresponding ICR bit is set.
2. If the interrupt is enabled, an IRQ/NMI is generated.
3. The latch value is reloaded.
4. In one-shot mode, the timer stops. In continuous mode, it continues.

### Timer Input Sources

| Source | Timer A | Timer B |
|---|---|---|
| System clock (default) | ✓ | ✓ |
| CNT pin (User Port pin 4/C) | ✓ | ✓ |
| Timer A underflow | — | ✓ |
| Timer A underflow + CNT pulse | — | ✓ |

Timer B can count Timer A underflows, effectively chaining the two
timers into a 32-bit counter.

## Data Port Operation

Each data port bit can be independently configured as input or output
via the DDR. When a bit is set as output, the value written to the
data port register appears on the pin. When set as input, the pin's
external logic level can be read.

**Port B special feature:** Timer A can output to bit 6 of Port B, and
Timer B can output to bit 7 of Port B, overriding the normal data port
value. This is controlled by bits 1–2 of the respective control
register.

## Time of Day Clock

The TOD clock keeps time in BCD (Binary Coded Decimal) format:

- Tenths: 0–9 (4 bits)
- Seconds: 00–59 (7 bits, two BCD nybbles)
- Minutes: 00–59 (7 bits, two BCD nybbles)
- Hours: 01–12 AM/PM (5 bits + AM/PM flag in bit 7)

### Latching Behavior

- **Reading** the hours register **freezes** the registers (latch).
  They resume updating after the tenths register is read.
- **Writing** the hours register **stops** the clock. It resumes after
  the tenths register is written.

### Alarm

When bit 7 of Control Register B is set to 1, writes to TOD registers
set the **alarm** time instead of the clock time. When the clock
reaches the alarm time, ICR bit 2 is set.

### Clock Source

The TOD clock is driven by a 50 Hz or 60 Hz signal on the TOD pin,
selected by bit 7 of Control Register A. The C64 uses 60 Hz.

**Emulation note:** The C64's Kernal never starts the CIA #1 TOD clock
(it never writes to the tenths register), so the clock sits at zero
unless software explicitly starts it. This is a known bug that affects
`RND(0)`, which reads the TOD seconds and tenths registers.