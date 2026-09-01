# Time of Day Clock

Each CIA chip includes a Time of Day (TOD) clock with an alarm. The TOD
clock keeps time in Binary Coded Decimal (BCD) format and can generate an
interrupt when the clock reaches a programmed alarm time.

## TOD Register Map (per CIA)

| Offset | Register | Bits | Description |
|---|---|---|---|
| $08 | TODTEN | 0–3 | Tenths of seconds (BCD: 0–9) |
| $09 | TODSEC | 0–6 | Seconds (BCD: 00–59) |
| $0A | TODMIN | 0–6 | Minutes (BCD: 00–59) |
| $0B | TODHRS | 0–4, 7 | Hours (BCD: 01–12) + AM/PM flag (bit 7: 1 = PM, 0 = AM) |

### BCD Format

Each register stores two decimal digits in a single byte, one digit per nybble.
Unused bits are forced to `0` when read. For example, 42 seconds is stored
as `%01000010` ($42), not `$2A`.

| Register | Low Nybble | High Nybble |
|---|---|---|
| TODTEN | Tenths (0–9) | Unused |
| TODSEC | Seconds ones (0–9) | Seconds tens (0–5) |
| TODMIN | Minutes ones (0–9) | Minutes tens (0–5) |
| TODHRS | Hours ones (0–9) | Hours tens (0–1) |

### Detailed Bit Layout

| Register | D7 | D6 | D5 | D4 | D3 | D2 | D1 | D0 |
|---|---|---|---|---|---|---|---|---|
| **TODTEN ($08)** | 0 | 0 | 0 | 0 | T8 | T4 | T2 | T1 |
| **TODSEC ($09)** | 0 | SH4 | SH2 | SH1 | SL8 | SL4 | SL2 | SL1 |
| **TODMIN ($0A)** | 0 | MH4 | MH2 | MH1 | ML8 | ML4 | ML2 | ML1 |
| **TODHRS ($0B)** | PM | 0 | 0 | HH | HL8 | HL4 | HL2 | HL1 |

Hours range from 01 to 12 in 12-hour format. There is no 24-hour mode.

## Latching Behavior

### Reading (Latch on Hours)

Reading the **hours** register ($0B) **freezes** all four TOD registers.
While latched, the internal clock continues to count in the background.

The output registers remain frozen until the **tenths** register ($08)
is read. This prevents time-skew when reading multi-register time values.

To safely read the full time:
1. Read hours ($0B) — this latches all registers
2. Read minutes ($0A)
3. Read seconds ($09)
4. Read tenths ($08) — this unlatches the registers

If only one register needs to be read, it can be read "on the fly" without
latching, provided that if Hours is ever read, Tenths is also read to clear the latch.

### Writing (Stop on Hours)

Writing the **hours** register ($0B) **stops** the TOD clock. The clock
remains stopped until the **tenths** register ($08) is written. This ensures
that the time can be set atomically without the clock ticking mid-write.

To set the time:
1. Write hours ($0B) — this stops the clock
2. Write minutes ($0A)
3. Write seconds ($09)
4. Write tenths ($08) — this starts the clock

## Alarm Function

Bit 7 of Control Register B ($0F) selects whether writes to TOD registers
set the **clock time** or the **alarm time**:

| CRB Bit 7 | Mode | Write Target |
|---|---|---|
| 0 | TOD | Clock time |
| 1 | ALARM | Alarm time |

**Read note:** The ALARM registers are write-only. Reading any TOD address
always returns the current clock time regardless of the state of CRB bit 7.

When the clock time reaches the alarm time, ICR bit 2 is set. If the TOD
alarm interrupt is enabled (via ICR bit 2 mask), an interrupt is generated.

The alarm comparison matches on hours, minutes, seconds, and tenths. The
AM/PM flag is included in the hours comparison.

## Clock Source

The TOD clock is driven by a 50 Hz or 60 Hz signal on the CIA's TOD pin.
The frequency is selected by bit 7 of Control Register A ($0E):

| CRA Bit 7 | TOD Frequency |
|---|---|
| 0 | 60 Hz |
| 1 | 50 Hz |

The C64 uses a 60 Hz signal on the TOD pin (derived from the power supply
or video timing). Setting bit 7 to 1 selects 50 Hz mode, which would cause
the clock to run slow on a 60 Hz system.

## System Usage

### CIA #1 TOD — Largely Unused

The C64's Kernal **never starts** the CIA #1 TOD clock. It never writes to
the tenths register, so the clock sits at zero (stopped state). This is a
known Kernal bug.

**Effect on RND(0):** The BASIC RND(0) function reads CIA #1 TOD seconds
and tenths registers as part of its seed value. Since the clock never runs,
these registers always read 0, making RND(0) ineffective for generating
truly random seeds. The traditional workaround is `RND(-TI)` or
`RND(-RND(0))`.

### CIA #2 TOD — Not Used

The Kernal does not use CIA #2's TOD clock at all. It is available for
user programs.

## Emulation Notes

### TOD Tick Source

The emulator must tick the TOD clock at the correct frequency (50 or 60 Hz,
selected by CRA bit 7). The TOD counter should increment in BCD format,
with proper carry between digits:

- Tenths: 0→9, then carry to seconds
- Seconds ones: 0→9, then carry to seconds tens
- Seconds tens: 0→5, then carry to minutes
- Minutes ones: 0→9, then carry to minutes tens
- Minutes tens: 0→5, then carry to hours
- Hours: 01→12, with AM/PM toggle on wraparound

### Alarm Comparison

The emulator should compare the current TOD time against the alarm time
every TOD tick. When all four registers match, set ICR bit 2.

### Latch/Stop Modeling

The latch-on-read-hours and stop-on-write-hours behaviors must be modeled
exactly. A common emulator bug is to allow the TOD to continue updating
during a read sequence, causing inconsistent time values.

### BCD Arithmetic

All TOD register values are in BCD, not binary. The emulator must perform
BCD increment and comparison operations. For example, incrementing $09 gives
$10 (not $0A). This is separate from the 6510's decimal mode flag, which
only affects ADC/SBC instructions.

### Conceptual Considerations for Emulation

In a Rust emulator, the TOD clock can be modeled as a struct containing:
- Four BCD register values (tenths, seconds, minutes, hours)
- Four alarm BCD values
- A latched flag (set when hours is read, cleared when tenths is read)
- A stopped flag (set when hours is written, cleared when tenths is written)
- A current frequency (50 or 60 Hz, from CRA bit 7)
- An accumulator for sub-TOD-tick timing

The `tick()` method is called at the system clock rate but only advances the
TOD at the configured TOD frequency (once every ~17 ms for 60 Hz).