# Pinout and Signals

The VIC-II is a 40-pin DIP package that acts as the bus master for the C64 system. It generates the system clock and manages all DRAM timing.

## Clock Signals

| Pin | Name | Description |
|---|---|---|
| øIN | Pixel Clock Input | 8.18 MHz (NTSC) or 7.88 MHz (PAL) generated from the crystal. |
| ø0 | System Clock Output | øIN divided by 8. 1.023 MHz (NTSC) or 0.985 MHz (PAL). Fed into the 6510. |
| ø2 | Phase 2 Clock | Derived from ø0 by the 6510. The 6510 accesses the bus only when ø2 is high. The VIC-II normally accesses the bus when ø2 is low. |

## Bus Arbitration

| Pin | Name | Description |
|---|---|---|
| BA | Bus Available | Connected to 6510 RDY. VIC-II sets BA low to stop the CPU. BA goes low 3 cycles before the VIC-II needs the bus, as the 6510 ignores RDY during write cycles and can perform up to 3 consecutive writes. |
| AEC | Address Enable Control | Tri-states the address bus. AEC follows ø2 normally (low during ø2 low, high during ø2 high). When the VIC-II steals cycles, AEC remains low during ø2 high, disabling the 6510's address bus drivers. |

## Buses

| Pin(s) | Name | Description |
|---|---|---|
| A0-A13 | Video Address Bus | 14-bit bus to address 16KB of DRAM. A0-A5 and A8-A13 are multiplexed. A6-A11 are on separate lines. |
| D0-D11 | Data Bus | 12-bit wide bus. D0-D7 connect to main RAM and CPU. D8-D11 connect exclusively to the 1Kx4 Color RAM. |

## I/O Signals

| Pin | Name | Description |
|---|---|---|
| IRQ | Interrupt Request | Wired to 6510 IRQ. Triggered by raster compare, sprite collisions, or light pen. |
| LP | Light Pen | Negative edge latches current X/Y position into $D013/$D014. Shared with keyboard matrix. |
| RAS / CAS | DRAM Control | Row and Column Address Strobe signals for the dynamic RAM. |
```