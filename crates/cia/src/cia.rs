use crate::{
    clock::{Clock, ClockMode, Frequency},
    icr::{InterruptControlRegister, TIMER_A_UNDERFLOW, TIMER_B_UNDERFLOW},
    port::Port,
    timer::{Timer, TimerMode},
};

// ---------------------------------------------------------------------
// Register offsets ($DC00-$DC0F, mirrored every 16 bytes)
// ---------------------------------------------------------------------
const PRA: u8 = 0x00;
const PRB: u8 = 0x01;
const DDRA: u8 = 0x02;
const DDRB: u8 = 0x03;
const TA_LO: u8 = 0x04;
const TA_HI: u8 = 0x05;
const TB_LO: u8 = 0x06;
const TB_HI: u8 = 0x07;
const TOD_10THS: u8 = 0x08;
const TOD_SECS: u8 = 0x09;
const TOD_MINS: u8 = 0x0A;
const TOD_HOURS: u8 = 0x0B;
const SDR: u8 = 0x0C;
const ICR: u8 = 0x0D;
const CRA: u8 = 0x0E;
const CRB: u8 = 0x0F;

// ---------------------------------------------------------------------
// Bits
// ---------------------------------------------------------------------
const ICR_TA: u8 = 1 << 0;
const ICR_TB: u8 = 1 << 1;
const ICR_TOD: u8 = 1 << 2;
const ICR_FLAG: u8 = 1 << 4;
const ICR_IR: u8 = 1 << 7;
const ICR_MASK_SET: u8 = 1 << 7;

const CR_START: u8 = 1 << 0;
const CR_OUTPUT: u8 = 1 << 1;
const CR_TOGGLE: u8 = 1 << 2;
const CR_ONE_SHOT: u8 = 1 << 3;
const CR_FORCE_LOAD: u8 = 1 << 4;

/// CRA bit 5: Timer A counts positive CNT-pin edges instead of φ2.
const CRA_COUNTS_CNT: u8 = 1 << 5;
const CRA_CLOCK_FREQUENCY: u8 = 1 << 7;

/// CRB: Timer B counts underflows of Timer A instead of φ2.
const CRB_COUNTS_TA_UNDERFLOWS: u8 = 1 << 6;
const CRB_ALARM_MODE: u8 = 1 << 7;

pub struct ComplexInterfaceAdapter {
    port_a: Port,
    port_b: Port,
    port_b_override_value: u8,
    timer_a: Timer,
    timer_b: Timer,
    clock: Clock,
    serial_data_register: u8,
    interrupt_control_register: InterruptControlRegister,
    control_register_a: u8,
    control_register_b: u8,
    cnt_pin: bool,
    flag_pin: bool,
}

impl ComplexInterfaceAdapter {
    pub fn new() -> Self {
        Self {
            port_a: Port::new(),
            port_b: Port::new(),
            port_b_override_value: 0,
            timer_a: Timer::new(),
            timer_b: Timer::new(),
            clock: Clock::new(),
            serial_data_register: 0,
            interrupt_control_register: InterruptControlRegister::new(),
            control_register_a: 0,
            control_register_b: 0,
            cnt_pin: false,
            flag_pin: false,
        }
    }

    pub fn read(&mut self, offset: u8) -> u8 {
        match offset & 0x0F {
            PRA => self.port_a.read_pins(),
            PRB => self.port_b.read_pins(),
            DDRA => self.port_a.read_ddr(),
            DDRB => self.port_b.read_ddr(),
            TA_LO => self.timer_a.read_low(),
            TA_HI => self.timer_a.read_high(),
            TB_LO => self.timer_b.read_low(),
            TB_HI => self.timer_b.read_high(),
            TOD_10THS | TOD_SECS | TOD_MINS | TOD_HOURS => self.clock.read(offset),
            SDR => self.serial_data_register,
            ICR => self.interrupt_control_register.read(),
            CRA => self.control_register_a & if self.timer_a.is_running() { !0 } else { !1 },
            CRB => self.control_register_b & if self.timer_b.is_running() { !0 } else { !1 },
            _ => unreachable!("Unexpected read at offset {offset:02X}"),
        }
    }

    pub fn write(&mut self, offset: u8, value: u8) {
        match offset & 0x0F {
            PRA => self.port_a.write_latch(value),
            PRB => self.port_b.write_latch(value),
            DDRA => self.port_a.write_ddr(value),
            DDRB => self.port_b.write_ddr(value),
            TA_LO => self.timer_a.write_low(value),
            TA_HI => self.timer_a.write_high(value),
            TB_LO => self.timer_b.write_low(value),
            TB_HI => self.timer_b.write_high(value),
            TOD_10THS | TOD_SECS | TOD_MINS | TOD_HOURS => self.clock.write(offset, value),
            SDR => self.serial_data_register = value,
            ICR => self.interrupt_control_register.write(value),
            CRA => {
                if value & CR_START != 0 {
                    if !self.timer_a.is_running() && (value & CR_TOGGLE != 0) {
                        self.port_b_override_value |= 1 << 6;
                    }
                    self.timer_a.start();
                } else {
                    self.timer_a.stop();
                }

                if value & CR_ONE_SHOT != 0 {
                    self.timer_a.set_mode(TimerMode::OneShot);
                } else {
                    self.timer_a.set_mode(TimerMode::Continuous);
                }

                if value & CR_FORCE_LOAD != 0 {
                    self.timer_a.reload();
                }

                let clock_frequency = if value & CRA_CLOCK_FREQUENCY != 0 {
                    Frequency::F50
                } else {
                    Frequency::F60
                };
                self.clock.set_frequency(clock_frequency);

                self.control_register_a = value & !CR_FORCE_LOAD;

                self.port_b
                    .set_output_override(self.get_override_mask(), self.port_b_override_value);
            }
            CRB => {
                if value & CR_START != 0 {
                    if !self.timer_b.is_running() && (value & CR_TOGGLE != 0) {
                        self.port_b_override_value |= 1 << 7;
                    }
                    self.timer_b.start();
                } else {
                    self.timer_b.stop();
                }

                if value & CR_ONE_SHOT != 0 {
                    self.timer_b.set_mode(TimerMode::OneShot);
                } else {
                    self.timer_b.set_mode(TimerMode::Continuous);
                }

                if value & CR_FORCE_LOAD != 0 {
                    self.timer_b.reload();
                }

                let clock_mode = if value & CRB_ALARM_MODE != 0 {
                    ClockMode::Alarm
                } else {
                    ClockMode::Clock
                };
                self.clock.set_mode(clock_mode);

                self.control_register_b = value & !CR_FORCE_LOAD;

                self.port_b
                    .set_output_override(self.get_override_mask(), self.port_b_override_value);
            }
            _ => unreachable!("Unexpected read at offset {offset:02X}"),
        };
    }

    /// One φ2 cycle: timers (per their clock source), serial shifter,
    /// PB6/PB7 output flip-flops, ICR event latching — in that order.
    pub fn tick(&mut self) {
        if self.control_register_a & (CR_OUTPUT | CR_TOGGLE) == CR_OUTPUT {
            self.port_b_override_value &= !(1 << 6);
        }

        if self.control_register_b & (CR_OUTPUT | CR_TOGGLE) == CR_OUTPUT {
            self.port_b_override_value &= !(1 << 7);
        }

        let mut timer_a_underflow = false;
        if self.control_register_a & CRA_COUNTS_CNT == 0 {
            timer_a_underflow = self.tick_a();
        }

        let will_timer_b_tick = match (self.control_register_b >> 5) & 0b11 {
            0 => true,
            1 => false,
            2 => timer_a_underflow,
            3 => timer_a_underflow & self.cnt_pin,
            _ => unreachable!("Impossible to reach"),
        };
        if will_timer_b_tick {
            self.tick_b();
        }

        let override_mask = self.get_override_mask();
        self.port_b
            .set_output_override(override_mask, self.port_b_override_value);
    }

    fn tick_a(&mut self) -> bool {
        self.timer_a.tick();
        if self.timer_a.underflow() {
            self.timer_a.acknowledge_underflow();
            self.interrupt_control_register
                .set_interrupt(TIMER_A_UNDERFLOW);

            if self.control_register_a & CR_OUTPUT != 0 {
                if self.control_register_a & CR_TOGGLE != 0 {
                    self.port_b_override_value ^= 1 << 6;
                } else {
                    self.port_b_override_value |= 1 << 6;
                }
            }
            true
        } else {
            false
        }
    }

    fn tick_b(&mut self) {
        self.timer_b.tick();
        if self.timer_b.underflow() {
            self.timer_b.acknowledge_underflow();
            self.interrupt_control_register
                .set_interrupt(TIMER_B_UNDERFLOW);

            if self.control_register_b & CR_OUTPUT != 0 {
                if self.control_register_b & CR_TOGGLE != 0 {
                    self.port_b_override_value ^= 1 << 7;
                } else {
                    self.port_b_override_value |= 1 << 7;
                }
            }
        }
    }

    fn get_override_mask(&self) -> u8 {
        (if self.control_register_a & CR_OUTPUT != 0 {
            1 << 6
        } else {
            0
        }) | (if self.control_register_b & CR_OUTPUT != 0 {
            1 << 7
        } else {
            0
        })
    }

    /// Interrupt line level (wired to IRQ for CIA #1, NMI for CIA #2).
    pub fn irq(&self) -> bool {
        self.interrupt_control_register.is_interrupt_active()
    }

    /// /RES: every component back to its power-on state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// FLAG pin input — a high→low transition latches ICR bit 4.
    fn set_flag(&mut self, level: bool) {
        if self.flag_pin && !level {
            self.interrupt_control_register.set_interrupt(ICR_FLAG);
        }
        self.flag_pin = level;
    }

    /// CNT pin input — positive edges clock timer A/B (CNT modes) and
    /// the serial input shifter.
    fn set_cnt(&mut self, level: bool) {
        let mut timer_a_underflow = false;
        if self.control_register_a & CRA_COUNTS_CNT != 0 && !self.cnt_pin && level {
            timer_a_underflow = self.tick_a();
        }

        let will_timer_b_tick = match (self.control_register_b >> 5) & 0b11 {
            0 => false,
            1 => !self.cnt_pin && level,
            2 => timer_a_underflow,
            3 => timer_a_underflow & level,
            _ => unreachable!("Impossible to reach"),
        };
        if will_timer_b_tick {
            self.tick_b();
        }

        let override_mask = self.get_override_mask();
        self.port_b
            .set_output_override(override_mask, self.port_b_override_value);

        self.cnt_pin = level;
    }

    /// One pulse on the TOD input pin (one mains cycle at the CRA-bit-2 rate).
    pub fn tod_pulse(&mut self) {
        self.clock.tick();
        if self.clock.alarm_pending() {
            self.clock.acknowledge_alarm();
            self.interrupt_control_register.set_interrupt(ICR_TOD);
        }
    }

    /// External pin wiring (keyboard matrix, joysticks, serial bus, VIC bank).
    pub fn set_port_a_pins(&mut self, value: u8) {
        self.port_a.set_pins(value);
    }

    pub fn set_port_b_pins(&mut self, value: u8) {
        self.port_b.set_pins(value);
    }

    pub fn port_a_driven(&self) -> u8 {
        self.port_a.driven_pins()
    }

    pub fn port_b_driven(&self) -> u8 {
        self.port_b.driven_pins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned by these tests.
    //
    // 0. REGISTER DECODE
    //    * read()/write() take a register offset; only A0-A3 are decoded,
    //      so offsets mirror every 16 bytes ($DC10 behaves like $DC00).
    //    * CRA/CRB bit 4 (force load) is a write strobe: it always READS
    //      back as 0.
    //    * CRA/CRB bit 0 reads back the LIVE run state: a one-shot that
    //      has expired reads as stopped.
    //    * CRB bit 7 routes TOD register writes to the alarm bank.
    //    * CRA bit 2 selects the TOD input frequency (50/60 Hz).
    //
    // 1. RESET (new() and reset())
    //    * ports: latch $00, DDR $00; unconnected pins read $FF
    //    * timers: $FFFF, stopped, continuous
    //    * ICR: flags AND mask clear, line released
    //    * TOD: stopped at 0:00:00.0; SDR $00; CRA/CRB $00
    //
    // 2. TIMERS (through $DC04-$DC07 / $DC0E / $DC0F)
    //    * LO writes touch the latch only; HI writes load the counter iff
    //      the timer is stopped; start never reloads; the bit-4 strobe
    //      loads always.
    //    * One tick() = one φ2. The underflow is the tick that decrements
    //      FROM $0000; the period is latch+1 ticks. Continuous reloads and
    //      keeps running; one-shot stops (and self-clears its start bit).
    //    * Timer A may count CNT-pin positive edges instead of φ2 (CRA
    //      bit 5); Timer B may count Timer A underflows (CRB clock select
    //      — see CRB_COUNTS_TA_UNDERFLOWS below).
    //
    // 3. INTERRUPTS
    //    * Timer A/B underflow -> ICR bit 0/1 on the underflow tick itself;
    //      TOD alarm -> bit 2; FLAG falling edge -> bit 4.
    //    * ICR read: returns the latched flags plus bit 7 iff
    //      (flags & mask) != 0, then clears the flags AND acknowledges the
    //      event sources. irq() is exactly (flags & mask) != 0.
    //    * A tick without a new event never resurrects a cleared flag.
    //
    // 4. TOD (through $DC08-$DC0B)
    //    * Writing HOURS stops the clock; writing 10THS starts it (clock
    //      bank only). One tod_pulse() = one power-line cycle; a tenth
    //      advances every 5 pulses (6 at 60 Hz).
    //    * Reading HOURS latches the time; reading 10THS releases it.
    //    * The alarm fires on the tick that makes clock == alarm.
    //
    // 5. PORTS
    //    * PRA/PRB reads see the pins; output bits are wired-ANDed with
    //      the latch; PB6/PB7 carry the timer output when CRA/CRB bit 6
    //      (toggle mode) is engaged, visible in PRB reads.
    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------
    fn cia() -> ComplexInterfaceAdapter {
        ComplexInterfaceAdapter::new()
    }

    fn rd(c: &mut ComplexInterfaceAdapter, reg: u8) -> u8 {
        c.read(reg)
    }

    fn wr(c: &mut ComplexInterfaceAdapter, reg: u8, value: u8) {
        c.write(reg, value)
    }

    /// `n` φ2 clock pulses.
    fn ticks(c: &mut ComplexInterfaceAdapter, n: u64) {
        for _ in 0..n {
            c.tick();
        }
    }

    /// The KERNAL idiom: write the low then the high latch byte while the
    /// timer is stopped (the high write loads the counter).
    fn load_timer_a(c: &mut ComplexInterfaceAdapter, value: u16) {
        wr(c, TA_LO, (value & 0x00FF) as u8);
        wr(c, TA_HI, (value >> 8) as u8);
    }

    fn load_timer_b(c: &mut ComplexInterfaceAdapter, value: u16) {
        wr(c, TB_LO, (value & 0x00FF) as u8);
        wr(c, TB_HI, (value >> 8) as u8);
    }

    fn read_timer_a(c: &mut ComplexInterfaceAdapter) -> u16 {
        u16::from_le_bytes([rd(c, TA_LO), rd(c, TA_HI)])
    }

    fn read_timer_b(c: &mut ComplexInterfaceAdapter) -> u16 {
        u16::from_le_bytes([rd(c, TB_LO), rd(c, TB_HI)])
    }

    // ---------------------------------------------------------------------
    // 1. Reset state
    // ---------------------------------------------------------------------

    #[test]
    fn reset_state_of_every_register() {
        let mut c = cia();
        // ports: all inputs; unconnected pins float high (Port::new default)
        assert_eq!(rd(&mut c, PRA), 0xFF);
        assert_eq!(rd(&mut c, PRB), 0xFF);
        assert_eq!(rd(&mut c, DDRA), 0x00);
        assert_eq!(rd(&mut c, DDRB), 0x00);
        // timers: $FFFF, stopped, continuous
        assert_eq!(rd(&mut c, TA_LO), 0xFF);
        assert_eq!(rd(&mut c, TA_HI), 0xFF);
        assert_eq!(rd(&mut c, TB_LO), 0xFF);
        assert_eq!(rd(&mut c, TB_HI), 0xFF);
        // TOD: 0:00:00.0, stopped
        assert_eq!(rd(&mut c, TOD_10THS), 0x00);
        assert_eq!(rd(&mut c, TOD_SECS), 0x00);
        assert_eq!(rd(&mut c, TOD_MINS), 0x00);
        assert_eq!(rd(&mut c, TOD_HOURS), 0x00);
        // SDR: last-written model, nothing written yet
        assert_eq!(rd(&mut c, SDR), 0x00);
        // ICR: no flags, line released
        assert_eq!(rd(&mut c, ICR), 0x00);
        assert!(!c.irq());
        assert_eq!(rd(&mut c, CRA), 0x00);
        assert_eq!(rd(&mut c, CRB), 0x00);
    }

    // ---------------------------------------------------------------------
    // 2. Ports through the CIA registers
    // ---------------------------------------------------------------------

    #[test]
    fn ddr_registers_round_trip() {
        let mut c = cia();
        wr(&mut c, PRA, 0xA5); // parked in the latch while all-input
        for &v in &[0x00, 0x0F, 0x55, 0xAA, 0xFF] {
            wr(&mut c, DDRA, v);
            assert_eq!(rd(&mut c, DDRA), v);
            wr(&mut c, DDRB, v);
            assert_eq!(rd(&mut c, DDRB), v);
        }
        // DDR writes never disturbed the port latch
        wr(&mut c, DDRA, 0xFF);
        assert_eq!(c.port_a_driven(), 0xA5);
    }

    #[test]
    fn input_bits_read_the_pins_and_ignore_the_latch() {
        let mut c = cia();
        c.set_port_a_pins(0x5A);
        assert_eq!(rd(&mut c, PRA), 0x5A);
        wr(&mut c, PRA, 0x0F); // invisible while the bits are inputs
        assert_eq!(rd(&mut c, PRA), 0x5A);
        c.set_port_a_pins(0x00);
        assert_eq!(rd(&mut c, PRA), 0x00);
        c.set_port_a_pins(0xFF);
        assert_eq!(rd(&mut c, PRA), 0xFF);
    }

    #[test]
    fn output_bits_drive_the_latch_and_external_pulls_win() {
        let mut c = cia();
        wr(&mut c, DDRA, 0xFF);
        wr(&mut c, PRA, 0x3C);
        assert_eq!(c.port_a_driven(), 0x3C);
        assert_eq!(rd(&mut c, PRA), 0x3C);
        // a grounded pin reads 0 even where we drive 1 (wired-AND)
        c.set_port_a_pins(!0x10);
        assert_eq!(rd(&mut c, PRA), 0x2C);
    }

    #[test]
    fn switching_a_bit_to_output_reveals_the_preloaded_latch() {
        let mut c = cia();
        wr(&mut c, PRA, 0x42); // parked while all-input
        c.set_port_a_pins(0xFF);
        assert_eq!(rd(&mut c, PRA), 0xFF);
        wr(&mut c, DDRA, 0xFF);
        assert_eq!(c.port_a_driven(), 0x42);
        assert_eq!(rd(&mut c, PRA), 0x42);
    }

    #[test]
    fn keyboard_matrix_idiom_columns_on_pa_rows_on_pb() {
        // The real matrix junctions belong to the keyboard device; this
        // pins the CIA wiring the idiom relies on.
        let mut c = cia();
        wr(&mut c, DDRA, 0xFF); // PA drives the matrix columns
        wr(&mut c, DDRB, 0x00); // PB reads the rows
        wr(&mut c, PRA, 0xFF);
        c.set_port_b_pins(0xFF); // no keys down
        assert_eq!(rd(&mut c, PRB), 0xFF);
        // select column 0; a key ties PB3 to the driven column
        wr(&mut c, PRA, 0xFE);
        c.set_port_b_pins(0xF7);
        assert_eq!(rd(&mut c, PRB) & 0x08, 0x00, "the key pulls its row low");
        // deselect: the row floats high again
        wr(&mut c, PRA, 0xFF);
        c.set_port_b_pins(0xFF);
        assert_eq!(rd(&mut c, PRB), 0xFF);
    }

    // ---------------------------------------------------------------------
    // 3. Timer programming (no ticking yet)
    // ---------------------------------------------------------------------

    #[test]
    fn writing_timer_registers_while_stopped_loads_the_counter() {
        let mut c = cia();
        wr(&mut c, TA_LO, 0x34);
        wr(&mut c, TA_HI, 0x12);
        assert_eq!(read_timer_a(&mut c), 0x1234);
        wr(&mut c, TB_LO, 0x78);
        wr(&mut c, TB_HI, 0x00);
        assert_eq!(read_timer_b(&mut c), 0x0078);
    }

    #[test]
    fn writing_only_the_low_byte_leaves_the_counter_alone() {
        let mut c = cia();
        wr(&mut c, TA_LO, 0x00); // latch $FF00, counter still $FFFF
        assert_eq!(read_timer_a(&mut c), 0xFFFF);
        wr(&mut c, TA_HI, 0xFF); // stopped: the high write loads
        assert_eq!(read_timer_a(&mut c), 0xFF00);
    }

    #[test]
    fn force_load_strobe_loads_without_starting_and_reads_as_zero() {
        let mut c = cia();
        load_timer_a(&mut c, 0x1010);
        wr(&mut c, TA_LO, 0x55); // latch $1055, counter $1010
        wr(&mut c, CRA, CR_FORCE_LOAD);
        assert_eq!(read_timer_a(&mut c), 0x1055);
        assert_eq!(rd(&mut c, CRA) & CR_START, 0, "the strobe must not start");
        assert_eq!(rd(&mut c, CRA) & CR_FORCE_LOAD, 0, "bit 4 never reads back");

        load_timer_b(&mut c, 0x00FF);
        wr(&mut c, TB_LO, 0x38);
        wr(&mut c, CRB, CR_FORCE_LOAD);
        assert_eq!(read_timer_b(&mut c), 0x0038);
        assert_eq!(rd(&mut c, CRB), 0x00);
    }

    #[test]
    fn force_load_strobe_also_works_while_running() {
        let mut c = cia();
        load_timer_a(&mut c, 0x0010);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 3); // $000D
        wr(&mut c, CRA, CR_START | CR_FORCE_LOAD);
        assert_eq!(read_timer_a(&mut c), 0x0010);
        assert_eq!(rd(&mut c, CRA) & CR_START, CR_START, "still running");
        ticks(&mut c, 1);
        assert_eq!(read_timer_a(&mut c), 0x000F);
    }

    #[test]
    fn control_register_bits_round_trip_except_the_strobe() {
        let mut c = cia();
        wr(&mut c, CRA, 0xE9); // start, one-shot, CNT, toggle, SP out
        assert_eq!(rd(&mut c, CRA), 0xE9);
        wr(&mut c, CRA, 0x00); // stop
        assert_eq!(rd(&mut c, CRA), 0x00);
        wr(&mut c, CRB, 0x80); // TOD alarm select — must not start timer B
        assert_eq!(rd(&mut c, CRB), 0x80);
        wr(&mut c, CRB, 0x48); // one-shot + toggle on B (stopped)
        assert_eq!(rd(&mut c, CRB), 0x48);
        // bit 4 is never stored
        wr(&mut c, CRA, 0x10);
        assert_eq!(rd(&mut c, CRA), 0x00);
        wr(&mut c, CRB, 0x11); // start + load
        assert_eq!(rd(&mut c, CRB), 0x01);
    }

    // ---------------------------------------------------------------------
    // 4. Clocking
    // ---------------------------------------------------------------------

    #[test]
    fn timer_a_counts_one_per_tick_across_byte_boundaries() {
        let mut c = cia();
        load_timer_a(&mut c, 0x0101);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 1);
        assert_eq!((rd(&mut c, TA_HI), rd(&mut c, TA_LO)), (0x01, 0x00));
        ticks(&mut c, 1);
        assert_eq!(
            (rd(&mut c, TA_HI), rd(&mut c, TA_LO)),
            (0x00, 0xFF),
            "borrow out of the high byte"
        );
        ticks(&mut c, 1);
        assert_eq!((rd(&mut c, TA_HI), rd(&mut c, TA_LO)), (0x00, 0xFE));
        assert_eq!(read_timer_a(&mut c), 0x00FE);
    }

    #[test]
    fn the_underflow_is_the_tick_from_zero_and_latches_icr_bit_0() {
        let mut c = cia();
        load_timer_a(&mut c, 3);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 3);
        assert_eq!(read_timer_a(&mut c), 0);
        assert_eq!(rd(&mut c, ICR), 0x00, "reaching zero is not the underflow");
        ticks(&mut c, 1); // THE underflow
        assert_eq!(rd(&mut c, ICR), ICR_TA, "the tick from zero latches bit 0");
        assert_eq!(rd(&mut c, ICR), 0x00, "the read cleared the flags");
        assert_eq!(read_timer_a(&mut c), 3, "continuous reloads the latch");
        assert_eq!(
            rd(&mut c, CRA) & CR_START,
            CR_START,
            "continuous keeps running"
        );
    }

    #[test]
    fn the_period_is_latch_plus_one_ticks() {
        for &latch in &[0u16, 1, 5, 0x0100] {
            let mut c = cia();
            load_timer_a(&mut c, latch);
            wr(&mut c, CRA, CR_START);
            ticks(&mut c, latch as u64);
            assert_eq!(read_timer_a(&mut c), 0, "latch {latch}: at zero");
            assert_eq!(rd(&mut c, ICR), 0x00, "latch {latch}: no event yet");
            ticks(&mut c, 1);
            assert_eq!(
                rd(&mut c, ICR),
                ICR_TA,
                "latch {latch}: event on tick {latch}+1"
            );
            assert_eq!(read_timer_a(&mut c), latch, "latch {latch}: reloaded");
        }
    }

    #[test]
    fn continuous_underflows_repeat_and_events_do_not_queue() {
        let mut c = cia();
        load_timer_a(&mut c, 2); // period 3
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 3);
        assert_eq!(rd(&mut c, ICR), ICR_TA); // underflow 1
        ticks(&mut c, 3);
        assert_eq!(rd(&mut c, ICR), ICR_TA); // underflow 2
        ticks(&mut c, 6); // underflows 3 and 4 between reads
        assert_eq!(
            rd(&mut c, ICR),
            ICR_TA,
            "events collapse, they do not count"
        );
        assert_eq!(rd(&mut c, ICR), 0x00);
        assert!(!c.irq(), "nothing masked");
    }

    #[test]
    fn one_shot_stops_itself_clears_its_start_bit_and_fires_once() {
        let mut c = cia();
        load_timer_a(&mut c, 4); // period 5
        wr(&mut c, CRA, CR_START | CR_ONE_SHOT);
        ticks(&mut c, 5);
        assert_eq!(rd(&mut c, ICR), ICR_TA);
        // real 6526: the expired one-shot self-clears the start bit,
        // so CRA bit 0 reads the live run state
        assert_eq!(
            rd(&mut c, CRA) & CR_START,
            0,
            "expired one-shot reads stopped"
        );
        assert_eq!(read_timer_a(&mut c), 4, "counter reloaded from the latch");
        ticks(&mut c, 100);
        assert_eq!(rd(&mut c, ICR), 0x00, "a one-shot fires exactly once");
        // restart: a full new period (no reload on start)
        wr(&mut c, CRA, CR_START | CR_ONE_SHOT);
        assert_eq!(rd(&mut c, CRA) & CR_START, CR_START);
        ticks(&mut c, 4);
        assert_eq!(rd(&mut c, ICR), 0x00);
        ticks(&mut c, 1);
        assert_eq!(rd(&mut c, ICR), ICR_TA);
    }

    #[test]
    fn timer_b_underflow_latches_icr_bit_1() {
        let mut c = cia();
        load_timer_b(&mut c, 2);
        wr(&mut c, CRB, CR_START);
        ticks(&mut c, 3);
        assert_eq!(rd(&mut c, ICR), ICR_TB);
        assert_eq!(read_timer_b(&mut c), 2, "continuous reload");
        assert_eq!(rd(&mut c, CRB) & CR_START, CR_START);
    }

    #[test]
    fn both_timers_latch_independently_and_read_together() {
        let mut c = cia();
        load_timer_a(&mut c, 2); // fires at tick 3
        load_timer_b(&mut c, 4); // fires at tick 5
        wr(&mut c, CRA, CR_START);
        wr(&mut c, CRB, CR_START);
        ticks(&mut c, 5);
        assert_eq!(rd(&mut c, ICR), ICR_TA | ICR_TB);
    }

    #[test]
    fn latch_writes_while_running_take_effect_at_the_next_underflow() {
        let mut c = cia();
        load_timer_a(&mut c, 5);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 4); // counter 1
        wr(&mut c, TA_LO, 8); // latch := 8
        wr(&mut c, TA_HI, 0); // running: latch only — must NOT load
        assert_eq!(read_timer_a(&mut c), 1);
        ticks(&mut c, 1); // counter 0
        ticks(&mut c, 1); // underflow -> reload with the NEW latch
        assert_eq!(rd(&mut c, ICR), ICR_TA);
        assert_eq!(read_timer_a(&mut c), 8);
    }

    #[test]
    fn restarting_a_stopped_timer_does_not_reload_it() {
        let mut c = cia();
        load_timer_a(&mut c, 0x0010);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 4); // $000C
        wr(&mut c, CRA, 0x00); // stop
        wr(&mut c, CRA, CR_START); // restart — no reload
        assert_eq!(read_timer_a(&mut c), 0x000C, "start must not reload");
        ticks(&mut c, 1);
        assert_eq!(read_timer_a(&mut c), 0x000B);
    }

    #[test]
    fn timer_b_can_count_timer_a_underflows() {
        let mut c = cia();
        load_timer_a(&mut c, 1); // TA period 2
        load_timer_b(&mut c, 2);
        wr(&mut c, CRA, CR_START);
        wr(&mut c, CRB, CR_START | CRB_COUNTS_TA_UNDERFLOWS);
        ticks(&mut c, 2); // TA underflow 1: TB 2 -> 1
        assert_eq!(
            read_timer_b(&mut c),
            1,
            "TB decrements on TA underflows, not on φ2"
        );
        ticks(&mut c, 2); // TA underflow 2: TB 1 -> 0
        assert_eq!(read_timer_b(&mut c), 0);
        ticks(&mut c, 2); // TA underflow 3: TB underflows
        assert_eq!(rd(&mut c, ICR), ICR_TA | ICR_TB);
        assert_eq!(read_timer_b(&mut c), 2, "TB is continuous by default");
    }

    #[test]
    fn timer_a_can_count_cnt_pin_edges_instead_of_phi2() {
        // Written so it passes whether you process CNT edges inside
        // set_cnt() or sample them at the next tick(): asserts only after
        // ticks.
        let mut c = cia();
        c.set_cnt(false);
        load_timer_a(&mut c, 3);
        wr(&mut c, CRA, CR_START | CRA_COUNTS_CNT);
        ticks(&mut c, 10);
        assert_eq!(read_timer_a(&mut c), 3, "no CNT edges: the counter holds");
        c.set_cnt(true); // rising edge
        ticks(&mut c, 1);
        assert_eq!(read_timer_a(&mut c), 2);
        ticks(&mut c, 10); // holding high does not keep counting
        assert_eq!(read_timer_a(&mut c), 2);
        c.set_cnt(false); // falling edge: nothing
        ticks(&mut c, 1);
        assert_eq!(read_timer_a(&mut c), 2);
        c.set_cnt(true); // rising edge again
        ticks(&mut c, 1);
        assert_eq!(read_timer_a(&mut c), 1);
    }

    // ---------------------------------------------------------------------
    // 5. Interrupt control and the line
    // ---------------------------------------------------------------------

    #[test]
    fn masked_underflow_asserts_the_line_and_reading_releases_it() {
        let mut c = cia();
        load_timer_a(&mut c, 1); // period 2
        wr(&mut c, ICR, ICR_MASK_SET | ICR_TA);
        wr(&mut c, CRA, CR_START);
        assert!(!c.irq());
        ticks(&mut c, 2);
        assert!(c.irq(), "a masked underflow asserts the line");
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_TA, "bit 7 set while active");
        assert!(!c.irq(), "reading the ICR releases the line");
        ticks(&mut c, 2); // next period
        assert!(c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_TA);
    }

    #[test]
    fn unmasked_events_latch_but_never_assert_the_line() {
        let mut c = cia();
        load_timer_a(&mut c, 1);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 2);
        assert!(!c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_TA, "no bit 7 without a mask");
        assert!(!c.irq());
    }

    #[test]
    fn enabling_the_mask_after_an_event_asserts_immediately() {
        let mut c = cia();
        load_timer_a(&mut c, 1);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 2); // event, unmasked
        assert!(!c.irq());
        wr(&mut c, ICR, ICR_MASK_SET | ICR_TA);
        assert!(c.irq(), "the pending event asserts the line at once");
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_TA);
    }

    #[test]
    fn disabling_the_mask_deasserts_but_keeps_the_flag() {
        let mut c = cia();
        load_timer_a(&mut c, 1);
        wr(&mut c, ICR, ICR_MASK_SET | ICR_TA);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 2);
        assert!(c.irq());
        wr(&mut c, ICR, ICR_TA); // bit 7 clear: disable
        assert!(!c.irq(), "disabling the mask deasserts the line");
        assert_eq!(rd(&mut c, ICR), ICR_TA, "the flag itself stays latched");
    }

    #[test]
    fn reading_the_icr_acknowledges_the_event_sources() {
        let mut c = cia();
        load_timer_a(&mut c, 9); // period 10
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 10); // underflow
        assert_eq!(rd(&mut c, ICR), ICR_TA);
        // a tick WITHOUT a new underflow must not resurrect the flag
        ticks(&mut c, 5);
        assert_eq!(rd(&mut c, ICR), 0x00, "clear until the next underflow");
        ticks(&mut c, 5); // tick 20: the next underflow
        assert_eq!(rd(&mut c, ICR), ICR_TA);
    }

    #[test]
    fn flag_falling_edge_latches_icr_bit_4_and_only_edges_count() {
        let mut c = cia();
        wr(&mut c, ICR, ICR_MASK_SET | ICR_FLAG);
        c.set_flag(true); // establish the idle level
        assert!(!c.irq());
        c.set_flag(false); // falling edge
        ticks(&mut c, 1);
        assert!(c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_FLAG);
        // holding low and raising do nothing
        c.set_flag(false);
        ticks(&mut c, 1);
        c.set_flag(true);
        ticks(&mut c, 1);
        assert_eq!(rd(&mut c, ICR), 0x00);
        // a fresh falling edge latches again
        c.set_flag(false);
        ticks(&mut c, 1);
        assert!(c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_FLAG);
    }

    #[test]
    fn timer_a_output_toggles_pb6_in_toggle_mode() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 1); // period 2

        // Starting in toggle mode should set PB6 HIGH immediately
        wr(&mut c, CRA, CR_START | CR_OUTPUT | CR_TOGGLE);
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "PB6 is set high when timer starts"
        );

        ticks(&mut c, 2); // first underflow: flips to LOW
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x00,
            "PB6 flips low on first underflow"
        );

        ticks(&mut c, 2); // second underflow: flips back to HIGH
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "PB6 flips high on second underflow"
        );
    }

    // ---------------------------------------------------------------------
    // 6. Time of day (through $DC08-$DC0B)
    // ---------------------------------------------------------------------

    #[test]
    fn tod_hours_write_stops_and_tenths_write_starts_the_clock() {
        let mut c = cia();
        wr(&mut c, TOD_HOURS, 0x01);
        wr(&mut c, TOD_MINS, 0x02);
        wr(&mut c, TOD_SECS, 0x03);
        for _ in 0..50 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x00, "the clock is stopped");
        assert_eq!(rd(&mut c, TOD_SECS), 0x03);
        wr(&mut c, TOD_10THS, 0x00); // the tenths write STARTS it
        for _ in 0..4 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x00);
        c.tod_pulse(); // 5th pulse at 50 Hz
        assert_eq!(rd(&mut c, TOD_10THS), 0x01);
    }

    #[test]
    fn tod_carries_in_bcd() {
        let mut c = cia();
        wr(&mut c, TOD_HOURS, 0x00);
        wr(&mut c, TOD_MINS, 0x59);
        wr(&mut c, TOD_SECS, 0x59);
        wr(&mut c, TOD_10THS, 0x09); // 0:59:59.9, running
        for _ in 0..5 {
            c.tod_pulse();
        } // -> 1:00:00.0
        assert_eq!(rd(&mut c, TOD_10THS), 0x00);
        assert_eq!(rd(&mut c, TOD_SECS), 0x00);
        assert_eq!(rd(&mut c, TOD_MINS), 0x00);
        assert_eq!(rd(&mut c, TOD_HOURS), 0x01);
    }

    #[test]
    fn tod_rolls_over_12_hours_with_the_am_pm_bit() {
        let mut c = cia();
        wr(&mut c, TOD_HOURS, 0x11); // 11 AM
        wr(&mut c, TOD_MINS, 0x59);
        wr(&mut c, TOD_SECS, 0x59);
        wr(&mut c, TOD_10THS, 0x09); // 11:59:59.9 AM, running
        for _ in 0..5 {
            c.tod_pulse();
        } // -> 1:00:00.0
        assert_eq!(rd(&mut c, TOD_10THS), 0x00);
        assert_eq!(rd(&mut c, TOD_HOURS), 0x92, "12 PM = BCD 12 + AM/PM bit");
        // re-program to 12:59:59.9 PM and roll once more
        wr(&mut c, TOD_HOURS, 0x92); // stops the clock
        wr(&mut c, TOD_MINS, 0x59);
        wr(&mut c, TOD_SECS, 0x59);
        wr(&mut c, TOD_10THS, 0x09); // restarts
        for _ in 0..5 {
            c.tod_pulse();
        } // -> 1:00:00.0
        assert_eq!(rd(&mut c, TOD_10THS), 0x00); // also releases the read latch
        assert_eq!(rd(&mut c, TOD_HOURS), 0x81, "1 PM = BCD 1 + AM/PM bit");
    }

    #[test]
    fn reading_hours_latches_the_time_until_tenths_is_read() {
        let mut c = cia();
        wr(&mut c, TOD_HOURS, 0x00);
        wr(&mut c, TOD_MINS, 0x00);
        wr(&mut c, TOD_SECS, 0x00);
        wr(&mut c, TOD_10THS, 0x00); // 0:00:00.0, running
        for _ in 0..30 {
            c.tod_pulse(); // 0:00:00.6
        }
        assert_eq!(rd(&mut c, TOD_HOURS), 0x00); // this read LATCHES 0:00:00.6
        for _ in 0..45 {
            c.tod_pulse(); // the clock runs on to 0:00:01.5
        }
        assert_eq!(rd(&mut c, TOD_SECS), 0x00, "frozen by the hours read");
        assert_eq!(rd(&mut c, TOD_MINS), 0x00);
        assert_eq!(rd(&mut c, TOD_10THS), 0x06, "frozen; this read RELEASES");
        assert_eq!(rd(&mut c, TOD_HOURS), 0x00); // re-latches the live time
        assert_eq!(rd(&mut c, TOD_SECS), 0x01, "live again after release");
        assert_eq!(rd(&mut c, TOD_10THS), 0x05);
    }

    #[test]
    fn reading_tenths_first_gives_live_reads() {
        let mut c = cia();
        wr(&mut c, TOD_10THS, 0x00); // start at 0:00:00.0
        for _ in 0..5 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x01, "no hours read: live");
        for _ in 0..5 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x02);
        assert_eq!(rd(&mut c, TOD_SECS), 0x00);
    }

    #[test]
    fn tod_alarm_fires_icr_bit_2_when_the_clock_passes_it() {
        let mut c = cia();
        wr(&mut c, CRB, 0x80); // TOD writes now target the ALARM
        wr(&mut c, TOD_HOURS, 0x00);
        wr(&mut c, TOD_MINS, 0x00);
        wr(&mut c, TOD_SECS, 0x00);
        wr(&mut c, TOD_10THS, 0x03); // alarm: 0:00:00.3
        wr(&mut c, CRB, 0x00); // back to the clock
        wr(&mut c, ICR, ICR_MASK_SET | ICR_TOD);
        wr(&mut c, TOD_10THS, 0x00); // start the clock at 0:00:00.0
        assert!(!c.irq());
        for _ in 0..14 {
            c.tod_pulse(); // 0:00:00.2
        }
        assert!(!c.irq(), "not yet at the alarm time");
        c.tod_pulse(); // 0:00:00.3 == alarm
        assert!(c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_IR | ICR_TOD);
        assert!(!c.irq());
    }

    #[test]
    fn alarm_writes_neither_stop_nor_start_the_clock() {
        let mut c = cia();
        wr(&mut c, TOD_10THS, 0x00); // start at 0:00:00.0
        for _ in 0..5 {
            c.tod_pulse(); // 0.1 s
        }
        wr(&mut c, CRB, CRB_ALARM_MODE); // alarm mode
        wr(&mut c, TOD_HOURS, 0x11); // alarm write — must NOT stop the clock
        for _ in 0..5 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x02, "the clock kept running");
    }

    #[test]
    fn cra_bit_7_selects_60_hz() {
        // CRA bit 7: 1 = 50 Hz, 0 = 60 Hz
        let mut c = cia();
        wr(&mut c, CRA, 0 & !CRA_CLOCK_FREQUENCY);
        wr(&mut c, TOD_10THS, 0x00); // start
        for _ in 0..5 {
            c.tod_pulse();
        }
        assert_eq!(
            rd(&mut c, TOD_10THS),
            0x00,
            "5 pulses is not a tenth at 60 Hz"
        );
        c.tod_pulse();
        assert_eq!(rd(&mut c, TOD_10THS), 0x01, "6 pulses is");
    }

    // ---------------------------------------------------------------------
    // 7. SDR, ICR decode, reset, mirrors
    // ---------------------------------------------------------------------

    #[test]
    fn sdr_reads_back_the_last_written_value() {
        let mut c = cia();
        assert_eq!(rd(&mut c, SDR), 0x00);
        wr(&mut c, SDR, 0xA5);
        assert_eq!(rd(&mut c, SDR), 0xA5);
        wr(&mut c, SDR, 0x5A);
        assert_eq!(rd(&mut c, SDR), 0x5A);
    }

    #[test]
    fn icr_writes_touch_only_the_mask() {
        let mut c = cia();
        wr(&mut c, ICR, ICR_MASK_SET | 0x1F); // enable everything
        assert_eq!(rd(&mut c, ICR), 0x00, "mask writes create no events");
        assert!(!c.irq());
        wr(&mut c, ICR, 0x1F); // disable everything
        assert!(!c.irq());
    }

    #[test]
    fn reset_restores_the_power_on_state() {
        let mut c = cia();
        // dirty everything
        wr(&mut c, DDRA, 0xFF);
        wr(&mut c, PRA, 0x3C);
        load_timer_a(&mut c, 0x1234);
        wr(&mut c, CRA, CR_START | CR_TOGGLE);
        load_timer_b(&mut c, 0x0007);
        wr(&mut c, CRB, CR_START | CR_ONE_SHOT);
        wr(&mut c, ICR, ICR_MASK_SET | 0x1F);
        wr(&mut c, SDR, 0xA5);
        ticks(&mut c, 10); // timer B one-shot underflows: line asserted
        assert!(c.irq());
        c.reset();
        assert_eq!(rd(&mut c, DDRA), 0x00);
        assert_eq!(rd(&mut c, DDRB), 0x00);
        assert_eq!(rd(&mut c, TA_LO), 0xFF);
        assert_eq!(rd(&mut c, TA_HI), 0xFF);
        assert_eq!(rd(&mut c, TB_LO), 0xFF);
        assert_eq!(rd(&mut c, TB_HI), 0xFF);
        assert_eq!(rd(&mut c, CRA), 0x00);
        assert_eq!(rd(&mut c, CRB), 0x00);
        assert_eq!(rd(&mut c, SDR), 0x00);
        assert!(!c.irq());
        assert_eq!(rd(&mut c, ICR), 0x00, "flags AND mask are cleared");
        // the mask is gone too: an event after reset stays silent
        load_timer_a(&mut c, 1);
        wr(&mut c, CRA, CR_START);
        ticks(&mut c, 2);
        assert!(!c.irq());
        assert_eq!(rd(&mut c, ICR), ICR_TA);
        // the TOD is stopped again
        for _ in 0..50 {
            c.tod_pulse();
        }
        assert_eq!(rd(&mut c, TOD_10THS), 0x00);
    }

    #[test]
    fn timer_b_counts_timer_a_underflows_only_while_cnt_is_high() {
        let mut c = cia();
        load_timer_a(&mut c, 1); // TA period 2
        load_timer_b(&mut c, 5); // TB start at 5

        // CRB bits 6:5 = %11 -> count TA underflows only while CNT is high
        wr(&mut c, CRA, CR_START);
        wr(&mut c, CRB, CR_START | (0b11 << 5));

        // CNT is low (default): Timer B must NOT decrement despite Timer A underflows
        c.set_cnt(false);
        ticks(&mut c, 4); // 2 Timer A underflows
        assert_eq!(read_timer_b(&mut c), 5, "Timer B frozen while CNT is low");

        // Pull CNT high: Timer B should now count Timer A underflows
        c.set_cnt(true);
        ticks(&mut c, 2); // 1 Timer A underflow
        assert_eq!(
            read_timer_b(&mut c),
            4,
            "Timer B decrements when CNT is high"
        );
        ticks(&mut c, 2); // another underflow
        assert_eq!(read_timer_b(&mut c), 3);

        // Pull CNT low again: Timer B freezes again
        c.set_cnt(false);
        ticks(&mut c, 2);
        assert_eq!(
            read_timer_b(&mut c),
            3,
            "Timer B freezes when CNT returns low"
        );
    }

    #[test]
    fn registers_mirror_every_16_bytes() {
        // The 6526 decodes only A0-A3; the C64 maps the whole $DC00-$DCFF
        // block to one CIA. Drop this test if your bus pre-masks offsets.
        let mut c = cia();
        wr(&mut c, DDRA, 0x5A);
        assert_eq!(rd(&mut c, 0x12), 0x5A, "$DC12 mirrors $DC02");
        load_timer_a(&mut c, 0x0123);
        assert_eq!(rd(&mut c, 0x15), 0x01, "$DC15 mirrors $DC05");
    }

    #[test]
    fn timer_a_output_pulses_pb6_for_one_cycle() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 2); // period 3

        // Output mode = pulse (CR_TOGGLE bit 2 is 0)
        wr(&mut c, CRA, CR_START | CR_OUTPUT);

        ticks(&mut c, 2); // cycle before underflow
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x00,
            "PB6 idle low before underflow"
        );

        ticks(&mut c, 1); // the underflow tick
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "PB6 pulses high on underflow cycle"
        );

        ticks(&mut c, 1); // cycle after underflow
        assert_eq!(rd(&mut c, PRB) & 0x40, 0x00, "PB6 returns low immediately");
    }

    #[test]
    fn timer_b_output_toggles_pb7_in_toggle_mode() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_b(&mut c, 1); // period 2

        wr(&mut c, CRB, CR_START | CR_OUTPUT | CR_TOGGLE);
        assert_eq!(
            rd(&mut c, PRB) & 0x80,
            0x80,
            "PB7 initialized high on start"
        );

        ticks(&mut c, 2); // first underflow
        assert_eq!(rd(&mut c, PRB) & 0x80, 0x00, "PB7 toggles to low");

        ticks(&mut c, 2); // second underflow
        assert_eq!(rd(&mut c, PRB) & 0x80, 0x80, "PB7 toggles back to high");
    }

    #[test]
    fn timer_b_counts_cnt_pin_edges() {
        let mut c = cia();
        c.set_cnt(false);
        load_timer_b(&mut c, 4);

        // CRB bits 6:5 = %01 -> count CNT positive edges
        wr(&mut c, CRB, CR_START | (0b01 << 5));
        ticks(&mut c, 10);
        assert_eq!(read_timer_b(&mut c), 4, "no CNT edges: counter holds");

        c.set_cnt(true); // rising edge
        ticks(&mut c, 1);
        assert_eq!(read_timer_b(&mut c), 3, "decremented on rising edge");

        c.set_cnt(false); // falling edge
        ticks(&mut c, 1);
        assert_eq!(read_timer_b(&mut c), 3, "falling edge does not decrement");

        c.set_cnt(true); // second rising edge
        ticks(&mut c, 1);
        assert_eq!(read_timer_b(&mut c), 2);
    }

    #[test]
    fn expired_one_shot_clears_start_bit_permanently_until_restarted() {
        let mut c = cia();
        load_timer_a(&mut c, 2); // period 3
        wr(&mut c, CRA, CR_START | CR_ONE_SHOT);

        ticks(&mut c, 3); // underflow occurs, timer stops
        assert_eq!(rd(&mut c, CRA) & CR_START, 0, "CRA bit 0 is cleared");

        // Write CRA with start bit 0 (modifying only one-shot mode or output)
        wr(&mut c, CRA, CR_ONE_SHOT);
        ticks(&mut c, 10);
        assert_eq!(read_timer_a(&mut c), 2, "timer remains stopped");
    }

    #[test]
    fn timer_a_cnt_underflow_cascades_into_timer_b() {
        let mut c = cia();
        load_timer_a(&mut c, 1); // TA period 2
        load_timer_b(&mut c, 2); // TB start at 2

        // TA counts CNT edges; TB counts TA underflows
        wr(&mut c, CRA, CR_START | CRA_COUNTS_CNT);
        wr(&mut c, CRB, CR_START | CRB_COUNTS_TA_UNDERFLOWS);

        // First rising edge on CNT: TA 1 -> 0
        c.set_cnt(false);
        c.set_cnt(true);
        assert_eq!(read_timer_a(&mut c), 0);
        assert_eq!(read_timer_b(&mut c), 2, "TB has not decremented yet");

        // Second rising edge on CNT: TA underflows (0 -> 1 reload)
        c.set_cnt(false);
        c.set_cnt(true);
        assert_eq!(read_timer_a(&mut c), 1);
        assert_eq!(
            read_timer_b(&mut c),
            1,
            "TB decremented on TA underflow via CNT"
        );
    }

    #[test]
    fn restarting_timer_rearms_toggle_output_high() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 1);

        // Start timer in toggle mode -> sets PB6 high
        wr(&mut c, CRA, CR_START | CR_OUTPUT | CR_TOGGLE);
        ticks(&mut c, 2); // first underflow -> PB6 toggles LOW
        assert_eq!(rd(&mut c, PRB) & 0x40, 0x00);

        // Stop timer
        wr(&mut c, CRA, CR_OUTPUT | CR_TOGGLE);

        // Restart timer -> must re-arm PB6 to HIGH
        wr(&mut c, CRA, CR_START | CR_OUTPUT | CR_TOGGLE);
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "Restarting timer re-arms toggle line high"
        );
    }

    #[test]
    fn enabling_pulse_output_emits_no_phantom_pulse() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 5); // period 6: nothing fires soon
        wr(&mut c, CRA, CR_START | CR_OUTPUT); // pulse mode
        let at_enable = rd(&mut c, PRB) & 0x40;
        ticks(&mut c, 2);
        assert_eq!(
            at_enable,
            rd(&mut c, PRB) & 0x40,
            "PB6 must not move without an underflow"
        );
    }

    #[test]
    fn an_unrelated_control_write_does_not_truncate_a_pulse() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 2); // period 3
        wr(&mut c, CRA, CR_START | CR_OUTPUT);
        ticks(&mut c, 3); // underflow: pulse is up
        assert_eq!(rd(&mut c, PRB) & 0x40, 0x40);
        wr(&mut c, CRB, 0x00); // unrelated: configure timer B
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "pulse must last the full cycle"
        );
    }

    #[test]
    fn a_cnt_edge_does_not_truncate_a_pulse() {
        let mut c = cia();
        c.set_port_b_pins(0xFF);
        wr(&mut c, DDRB, 0x00);
        load_timer_a(&mut c, 2); // period 3, φ2-clocked
        wr(&mut c, CRA, CR_START | CR_OUTPUT); // pulse mode
        ticks(&mut c, 3); // underflow: pulse up
        assert_eq!(rd(&mut c, PRB) & 0x40, 0x40);
        c.set_cnt(true); // CNT edge; TA is on φ2, so unrelated
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "the edge must not end the pulse"
        );
        wr(&mut c, CRB, 0x00); // unrelated CR write
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x40,
            "the pulse owns the full cycle"
        );
        ticks(&mut c, 1);
        assert_eq!(
            rd(&mut c, PRB) & 0x40,
            0x00,
            "one cycle after the underflow"
        );
    }

    // ---------------------------------------------------------------------
    // Deferred until peripheral integration:
    // * SDR shift register / fast serial burst transfer (C128/1571 only)
    // * Exact 1-cycle ICR read-vs-interrupt race condition (Lorenz icr01)
    // ---------------------------------------------------------------------
}
