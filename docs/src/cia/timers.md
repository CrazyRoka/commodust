# CIA Timers

Each CIA chip contains two independent 16-bit programmable timers (Timer A
and Timer B). Both timers count down from a 16-bit latch value at the system
clock rate and can generate interrupts on underflow.

## Timer Register Map (per CIA)

| Offset | Name | Description |
|---|---|---|
| $04 | TIMALO | Timer A low byte |
| $05 | TIMAHI | Timer A high byte |
| $06 | TIMBLO | Timer B low byte |
| $07 | TIMBHI | Timer B high byte |

## Read/Write Behavior

### Writing

Writing to a timer register pair loads the 16-bit **latch** value. The latch
holds the reload value; it does not immediately affect the running timer
counter. The latch is copied to the counter when:
- The timer is started (bit 0 of control register set to 1) and the counter
  has not yet been loaded, or
- The force load strobe (bit 4 of control register) is set to 1, or
- The counter reaches zero and reloads (in continuous mode).

### Reading

Reading a timer register returns the **current counter value** (which is
counting down). The counter decrements by 1 every system clock cycle when
running.

**Latch value formula:** `LATCH = TIMALO + (TIMAHI × 256)`

**Time to underflow:** `TIME = LATCH / CLOCK_SPEED`

| System | Clock | Cycle Time |
|---|---|---|
| NTSC | 1,022,730 Hz | ~0.978 µs |
| PAL | 985,250 Hz | ~1.015 µs |

## Timer Operation

When a timer is running, it decrements the 16-bit counter by 1 each system
clock cycle. When the counter reaches zero:

1. The corresponding ICR bit (bit 0 for Timer A, bit 1 for Timer B) is set.
2. If the interrupt is enabled in the ICR mask, an interrupt (IRQ for CIA #1,
   NMI for CIA #2) is generated.
3. The latch value is reloaded into the counter.
4. In **one-shot mode** (bit 3 of control register = 1): the timer stops
   (bit 0 cleared). In **continuous mode** (bit 3 = 0): counting resumes.

## Control Register A ($0E) — Timer A Control

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Start Timer A (1 = start, 0 = stop) |
| 1 | $02 | Timer A output on Port B bit 6 (1 = enable) |
| 2 | $04 | Port B output mode (1 = toggle bit 6, 0 = pulse bit 6 for 1 cycle) |
| 3 | $08 | Run mode (1 = one-shot, 0 = continuous) |
| 4 | $10 | Force load (1 = strobe latch into counter) |
| 5 | $20 | Input source (1 = system clock, 0 = CNT pin / User Port pin 4) |
| 6 | $40 | Serial port direction (1 = output, 0 = input) |
| 7 | $80 | TOD clock frequency (1 = 50 Hz, 0 = 60 Hz) |

### Bit 4 — Force Load

Writing a 1 to bit 4 forces the latch value into the timer counter
immediately. This bit is write-only; it always reads as 0. It is used to
initialize the counter before starting the timer, or to reset it mid-count.

### Bit 5 — Input Source

When bit 5 = 1 (default), Timer A counts system clock cycles. When bit 5 = 0,
Timer A counts pulses on the CNT line (pin 4 of the User Port for CIA #1;
also connected to pin C of the User Port). This allows the CIA to function
as an external event counter or frequency counter.

## Control Register B ($0F) — Timer B Control

| Bit | Mask | Function |
|---|---|---|
| 0 | $01 | Start Timer B (1 = start, 0 = stop) |
| 1 | $02 | Timer B output on Port B bit 7 (1 = enable) |
| 2 | $04 | Port B output mode (1 = toggle bit 7, 0 = pulse bit 7 for 1 cycle) |
| 3 | $08 | Run mode (1 = one-shot, 0 = continuous) |
| 4 | $10 | Force load (1 = strobe latch into counter) |
| 5 | $20 | Timer B input mode bit 0 |
| 6 | $40 | Timer B input mode bit 1 |
| 7 | $80 | TOD write mode (1 = set alarm, 0 = set clock) |

### Timer B Input Mode (Bits 5–6)

| Bit 6 | Bit 5 | Input Source |
|---|---|---|
| 0 | 0 | System clock (default) |
| 0 | 1 | CNT pin (User Port pin 4) |
| 1 | 0 | Timer A underflow pulses |
| 1 | 1 | Timer A underflow when CNT is also active |

When Timer B counts Timer A underflows, the two timers are effectively
chained into a 32-bit counter. With Timer A latched to count a full 65536
cycles and Timer B counting Timer A underflows, the combined timer can count
up to 4,294,967,296 cycles (approximately 70 minutes at NTSC clock rate)
with accuracy to within 1/15 second.

## Port B Timer Output

When bit 1 of a control register is set, the corresponding timer's output
appears on Port B:
- Timer A → Port B bit 6
- Timer B → Port B bit 7

This output **overrides** the normal data port value and DDR setting for
that bit. The output mode (bit 2) selects:
- **Pulse mode (bit 2 = 0):** The output bit goes high for one machine cycle
  on timer underflow, then returns low.
- **Toggle mode (bit 2 = 1):** The output bit flips state on each timer
  underflow (0→1→0→1...).

## System Usage of CIA #1 Timers

### Timer A — System Jiffy Clock

CIA #1 Timer A is initialized by the Kernal IOINIT routine to generate an
IRQ approximately every 1/60 second (NTSC) or 1/50 second (PAL).

| System | Latch Value | Decimal | Frequency |
|---|---|---|---|
| NTSC | $4118 | 16664 | ~61.4 Hz |
| PAL | $4295 | 17045 | ~57.8 Hz |

The values are tuned for approximately 60 Hz / 50 Hz but are not exact due
to the integer division of the clock frequency.

This timer drives the system IRQ handler that:
- Scans the keyboard
- Updates the software jiffy clock ($A0–$A2)
- Blinks the cursor
- Checks the STOP key
- Maintains the cassette motor interlock

### Timer B — Cassette I/O

CIA #1 Timer B is used for cassette read/write timing. During tape I/O, the
tape routines take over the IRQ vector and use Timer B to time bit
transmission/reception.

## System Usage of CIA #2 Timers

CIA #2 Timers A and B are used for RS-232 timing. When device 2 (RS-232) is
opened:
- Timer A is set to generate NMI interrupts at the baud rate for transmission
- Timer B is set to generate NMI interrupts at the baud rate for reception

The baud rate prescaler value is: `PRESCALER = ((CLOCK / BAUD_RATE) / 2) - 100`

## Emulation Notes

### Cycle-Accurate Timer Counting

The timers must decrement every system clock cycle, not every instruction.
This means the timer counter must be updated between CPU instructions,
interleaved with the CPU's cycle-by-cycle execution. A timer can underflow
mid-instruction, and the interrupt is serviced after the current instruction
completes.

### Force Load Behavior

The force load strobe (bit 4) copies the latch to the counter on the next
cycle. It does not affect the timer's running state. If the timer is running
and force load is strobed, the counter resets to the latch value and
continues counting down.

### Chained Timer Precision

When Timer B counts Timer A underflows, the emulator must ensure that Timer
B increments exactly once for each Timer A underflow. If Timer A is in
one-shot mode, Timer B only counts when Timer A actually underflows (not
when it is manually stopped or force-loaded).

## Timer Output Modes (Port B)

When bit 1 of a control register is set, the timer output appears
on the corresponding Port B bit, overriding the normal data port
value and DDR setting:

| Timer | Port B Bit | CRA/CRB Bit 1 |
|---|---|---|
| Timer A | Bit 6 | CRA bit 1 |
| Timer B | Bit 7 | CRB bit 1 |

### Output Mode (Bit 2)

| Bit 2 | Mode | Behavior |
|---|---|---|
| 0 | Pulse | Output goes high for 1 machine cycle on underflow, then returns low |
| 1 | Toggle | Output flips state (0→1 or 1→0) on each underflow |

The pulse mode produces a very brief (1-cycle) high pulse that is
useful for triggering external devices. The toggle mode produces
a square wave at half the timer underflow frequency.

### Port B Output Interaction

The timer output overrides the Data Port B register value for the
affected bit, but does not change the Data Port B register itself.
When the timer output is disabled (bit 1 = 0), the normal Data
Port B value reappears on the pin. This allows software to pre-load
a value in Data Port B that will appear when the timer output is
disabled.

### Conceptual Considerations for Emulation

In a Rust emulator, each CIA should own two timer state structs, each
containing: a 16-bit counter, a 16-bit latch, a running flag, a one-shot
flag, and the input source selection. The `tick()` method should be called
every system cycle, decrementing running timers and checking for underflow.
The underflow event sets the ICR bit and potentially triggers the interrupt.

The timer's read path must return the live counter value, which requires
the counter to be continuously updated. This is naturally handled by a
cycle-driven design where `tick()` is called before any register read.