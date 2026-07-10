# User Port

The User Port is an 8-bit parallel I/O port accessible via a 24-pin edge
connector on the back of the C64. It provides direct access to CIA #2
Port B, CIA #2 serial ports, and additional signals.

## Pinout

| Pin | Label | Signal | Description |
|---|---|---|---|
| 1 | GND | Ground | Signal ground |
| 2 | +5V | +5 Volts | 100 mA maximum |
| 3 | RESET | Reset | Grounding this pin causes a cold start |
| 4 | CNT1 | CIA #1 CNT | Counter/timer input (CIA #1) |
| 5 | SP1 | CIA #1 SP | Serial port data (CIA #1) |
| 6 | CNT2 | CIA #2 CNT | Counter/timer input (CIA #2) |
| 7 | SP2 | CIA #2 SP | Serial port data (CIA #2) |
| 8 | PC2 | CIA #2 PC | Handshaking line (pulses low on Port B read/write) |
| 9 | ATN | Serial Bus ATN | Attention line (inverted) |
| 10 | 9VAC | +9V AC | 50 mA maximum |
| 11 | 9VAC | -9V AC | 50 mA maximum (opposite phase) |
| 12 | GND | Ground | Signal ground |
| A | GND | Ground | Signal ground |
| B | FLAG2 | CIA #2 FLAG | FLAG line (triggers NMI when pulled low) |
| C | PB0 | Port B bit 0 | RS-232 SIN / User I/O |
| D | PB1 | Port B bit 1 | RS-232 RTS / User I/O |
| E | PB2 | Port B bit 2 | RS-232 DTR / User I/O |
| F | PB3 | Port B bit 3 | RS-232 RI / User I/O |
| H | PB4 | Port B bit 4 | RS-232 DCD / User I/O |
| J | PB5 | Port B bit 5 | User I/O |
| K | PB6 | Port B bit 6 | RS-232 CTS / Timer A output / User I/O |
| L | PB7 | Port B bit 7 | RS-232 DSR / Timer B output / User I/O |
| M | PA2 | Port A bit 2 | RS-232 SOUT / User I/O |
| N | GND | Ground | Signal ground |

## CIA #2 Port B Access

Pins C through L correspond to CIA #2 Port B bits 0–7. The data direction
is controlled by CIA #2 DDRB ($DD03). Default DDRB = $00 (all inputs).
When the RS-232 device is opened, bits 1–2 are set to output.

## FLAG Line (Pin B)

The FLAG line on CIA #2 is connected to pin B of the User Port. When this
line is pulled low, it triggers bit 4 of the CIA #2 ICR, which can generate
an NMI interrupt if enabled. This provides an external interrupt source
for handshaking or event counting.

Note: On CIA #1, the FLAG line is connected to the cassette read signal
and is not available on the User Port.

## PC2 Handshaking Line (Pin 8)

The PC line pulses low for one cycle following a read or write of CIA #2
Port B. This signal can be used by external devices to synchronize data
transfers.

## Serial Ports (Pins 4–7)

Both CIA chips have on-chip serial shift registers:
- CIA #1 serial port: pins 4 (CNT1) and 5 (SP1)
- CIA #2 serial port: pins 6 (CNT2) and 7 (SP2)

The serial port can send or receive one byte at a time, MSB first, under
control of the CNT line and Timer A (for baud rate generation). The
serial port is not used by the C64 system software but is available for
user applications.

## Timer/CNT Lines

The CNT lines (pins 4 and 6) serve dual purposes:
1. Serial port clock for the respective CIA's serial shift register
2. External input for the CIA's timers (when configured to count CNT
   pulses instead of system clock cycles)

## RESET Line (Pin 3)

Grounding the RESET pin causes a hardware reset, equivalent to pressing
the reset button or power-cycling. This triggers the 6510 RESET vector at
$FFFC–$FFFD.

## Emulation Notes

### User Port Modeling

The emulator can model the User Port as a simple 8-bit bidirectional port
(CIA #2 Port B) plus the additional signals (FLAG, CNT, SP, PC). For most
emulation purposes, only Port B and FLAG need to be functional. The serial
ports and CNT lines can be left unconnected unless the emulated software
specifically uses them.

### FLAG/NMI Handling

If the emulator supports User Port devices, the FLAG line should be
connectable to an external trigger. When FLAG is pulled low, set bit 4 of
CIA #2's ICR. If the mask is enabled, generate an NMI.

### Conceptual Considerations for Emulation

In a Rust emulator, the User Port can be an optional trait object that
receives Port B writes and provides Port B reads. This allows plug-in
emulation of User Port devices (parallel printers, MIDI interfaces, etc.)
without changing the core emulator code.