/// Bit 7 of the TOD hours register ($DC0B): 1 = PM, 0 = AM.
const AM_PM_FLAG: u8 = 1 << 7;

/// One bank of four BCD time registers. Used for the live clock, the
/// alarm, and the read-latch snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Registers {
    tenths: u8,
    seconds: u8,
    minutes: u8,
    hours: u8,
}

/// Which register bank read/write address — CRB bit 7 of the owning CIA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockMode {
    Alarm,
    Clock,
}

/// Frequency on the TOD input pin (mains-derived: 50 Hz PAL, 60 Hz NTSC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frequency {
    F50,
    F60,
}

/// Time-of-Day clock of a MOS 6526 CIA ($DC08-$DC0B / $DD08-$DD0B).
#[derive(Debug)]
struct Clock {
    alarm: Registers,
    registers: Registers,
    latch: Option<Registers>,
    mode: ClockMode,
    frequency: Frequency,
    stopped: bool,
    alarm_pending: bool,
    divider: u8,
}

impl Clock {
    /// $DC08 — tenths of seconds, BCD 0–9 (bits 0–3 only).
    const TENTHS: u8 = 0;
    /// $DC09 — seconds, BCD 00–59 (bits 0–6 only).
    const SECONDS: u8 = 1;
    /// $DC0A — minutes, BCD 00–59 (bits 0–6 only).
    const MINUTES: u8 = 2;
    /// $DC0B — hours, BCD 01–12 in bits 0–5, AM/PM flag in bit 7.
    const HOURS: u8 = 3;

    /// Post-/RES state: the clock is STOPPED at 1:00:00.0 AM —
    /// "following a hardware reset, the clock will start at 1:00:00.0"
    /// once nudged by a tenths write — with the rest of the clock and
    /// the whole alarm zero, mode Clock, F50, divider 0, no pending
    /// alarm.
    fn new() -> Self {
        Self {
            alarm: Registers::default(),
            registers: Registers {
                hours: 0x01,
                ..Default::default()
            },
            latch: None,
            mode: ClockMode::Clock,
            frequency: Frequency::F50,
            stopped: true,
            alarm_pending: false,
            divider: 0,
        }
    }

    /// Read one TOD register. Has side effects: see the CONTRACT in the
    /// tests (latch set/release). Reads ALWAYS address the live clock,
    /// via the latch — the ALARM is write-only, so any read of a TOD
    /// address reads time regardless of CRB bit 7.
    fn read(&mut self, offset: u8) -> u8 {
        if (offset & 0x03) == Self::HOURS && self.latch.is_none() {
            self.latch = Some(self.registers);
        }
        let register = self.latch.unwrap_or(self.registers);

        let value = match offset & 0x03 {
            Self::TENTHS => register.tenths,
            Self::SECONDS => register.seconds,
            Self::MINUTES => register.minutes,
            Self::HOURS => register.hours,
            _ => unreachable!("Unexpected Clock register read at offset {offset:02X}"),
        };

        if (offset & 0x03) == Self::TENTHS {
            self.latch = None;
        }
        value
    }

    /// Write one TOD register. Has side effects: see the CONTRACT in the
    /// tests (hours write stops the clock, tenths write restarts it).
    fn write(&mut self, offset: u8, value: u8) {
        let register = match self.mode {
            ClockMode::Alarm => &mut self.alarm,
            ClockMode::Clock => &mut self.registers,
        };

        match offset & 0x03 {
            Self::TENTHS => {
                if self.mode == ClockMode::Clock {
                    self.stopped = false;
                    self.divider = 0;
                }
                register.tenths = value & 0b1111;
            }
            Self::SECONDS => register.seconds = value & 0b1111111,
            Self::MINUTES => register.minutes = value & 0b1111111,
            Self::HOURS => {
                if self.mode == ClockMode::Clock {
                    self.stopped = true;
                }
                register.hours = value & 0b10011111;
            }
            _ => unreachable!("Unexpected Clock register read at offset {offset:02X}"),
        };
    }

    /// CRB bit 7. Pure bank select — never stops/starts/loads anything.
    fn set_mode(&mut self, mode: ClockMode) {
        self.mode = mode;
    }

    fn mode(&self) -> ClockMode {
        self.mode
    }

    fn set_frequency(&mut self, frequency: Frequency) {
        self.frequency = frequency;
    }

    fn frequency(&self) -> Frequency {
        self.frequency
    }

    fn running(&self) -> bool {
        !self.stopped
    }

    /// One pulse on the TOD input pin, i.e. one power-line cycle
    /// (1/50 s at F50, 1/60 s at F60). See the CONTRACT in the tests.
    fn tick(&mut self) {
        if !self.running() {
            return;
        }

        let limit = match self.frequency {
            Frequency::F50 => 5,
            Frequency::F60 => 6,
        };
        self.divider += 1;
        if self.divider >= limit {
            self.divider -= limit;
            self.registers.tenths += 1;

            if self.registers.tenths == 10 {
                self.registers.tenths = 0;
                self.registers.seconds = Self::bcd_increment(self.registers.seconds);

                if self.registers.seconds == 0 {
                    self.registers.minutes = Self::bcd_increment(self.registers.minutes);

                    if self.registers.minutes == 0 {
                        self.registers.hours = Self::bcd_increment(self.registers.hours);
                        if (self.registers.hours & 0b11111) == 0b10010 {
                            self.registers.hours ^= AM_PM_FLAG;
                        } else if (self.registers.hours & 0b11111) == 0b10011 {
                            self.registers.hours &= AM_PM_FLAG | 0b1;
                        }
                    }
                }
            }

            if self.registers == self.alarm {
                self.alarm_pending = true;
            }
        }
    }

    fn bcd_increment(value: u8) -> u8 {
        let digits = (value & 0b1111) % 10;
        let tens = ((value & 0b1110000) >> 4) % 6;
        let am_pm = value & AM_PM_FLAG;

        let (new_digits, carry) = ((digits + 1) % 10, (digits + 1) / 10);
        let new_tens = (tens + carry) % 6;
        am_pm | (new_tens << 4) | new_digits
    }

    /// True if the clock has hit the alarm time since the last
    /// acknowledge_alarm(). (Later: ICR bit 2.)
    fn alarm_pending(&self) -> bool {
        self.alarm_pending
    }

    /// Clear the pending alarm event.
    fn acknowledge_alarm(&mut self) {
        self.alarm_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. REGISTER DECODE
    //    * Offsets are 0..=3 within the TOD block: 0 = tenths ($DC08),
    //      1 = seconds ($DC09), 2 = minutes ($DC0A), 3 = hours ($DC0B).
    //      Offsets outside 0..=3 are not covered; the CIA owns that decode.
    //    * Unused bits are not physically implemented — writes ignore
    //      them, reads return 0 there:
    //        tenths  : bits 0–3 (BCD 0–9)               mask $0F
    //        seconds : bits 0–6 (BCD 00–59)             mask $7F
    //        minutes : bits 0–6 (BCD 00–59)             mask $7F
    //        hours   : bit 7 = AM/PM (1 = PM), bits 0–4 = BCD hours 01–12
    //                  (bit 6 unimplemented)            mask $9F
    //      Only the hours register carries AM_PM_FLAG.
    //    * Tick behaviour from values outside the valid BCD ranges is NOT
    //      pinned (real hardware is chaotic there). The post-/RES state
    //      (hours $01, stopped) IS pinned — see 5.
    //
    // 2. MODE (CRB bit 7)
    //    * Clock : reads and writes address the live clock (via the
    //      latch, 3).
    //    * Alarm : WRITES address the alarm registers. Reads are
    //      mode-independent — the alarm is write-only, so any read of
    //      a TOD address reads time regardless of CRB7.
    //    * set_mode is bookkeeping only: it never starts or stops the
    //      clock and never touches registers, latch, divider or a pending
    //      alarm. The clock keeps ticking in either mode.
    //
    // 3. READS — THE LATCH (mode-independent)
    //    * Unlatched: reads of tenths/seconds/minutes return live values;
    //      a read of HOURS latches (snapshots) all four registers.
    //    * Latched: every read returns the snapshot; no read refreshes it
    //      (not even another HOURS read); a read of TENTHS releases the
    //      latch. Reads of seconds/minutes/hours keep it.
    //    * The clock keeps running while latched; after release, reads are
    //      live again and may jump ahead.
    //    * Writes and mode switches never set or clear the latch; a latch
    //      survives a trip through Alarm mode as long as nothing is read
    //      there (reads operate the latch in either mode).
    //    * Reads in Alarm mode read the clock and latch/release exactly
    //      like Clock-mode reads.
    //
    // 4. WRITES IN CLOCK MODE — STOP / RESTART
    //    * A write to HOURS stores the masked value and STOPS the clock
    //      (even if the value is unchanged). Only one thing restarts it:
    //    * A write to TENTHS stores the masked value, (re)starts the
    //      clock — even if it was already running — and resets the
    //      divider phase to 0.
    //    * Writes to seconds/minutes are plain stores.
    //    * Writes in Alarm mode store into the alarm registers only and
    //      never affect the clock, its run state or the divider.
    //
    // 5. CLOCKING (tick = one pulse on the TOD input pin)
    //      if !running  -> nothing happens at all
    //      else accumulate one power-line cycle into the divider; when it
    //      reaches the ratio (5 at F50, 6 at F60) the divider resets and
    //      the time advances by one tenth of a second, in BCD:
    //        tenths 0–8 -> +1;   9 -> 0, carry
    //        seconds/minutes: plain BCD increment; $59 -> $00, carry
    //        hours $01–$11 -> +1;  $11 -> $12 AND AM/PM flips;
    //              $12 -> $01 (no flip);  $00 -> $01 (no flip)
    //    * Both 50 and 60 Hz therefore keep real time (5 resp. 6 input
    //      cycles per tenth).
    //    * set_frequency never stops/starts the clock and never changes a
    //      register. The divider phase across a frequency change is
    //      deliberately NOT pinned; tests around a change use whole
    //      multiples of the new ratio.
    //    * new(): clock stopped at hours $01 (rest zero), alarm all $00,
    //      mode Clock, F50, divider 0, no pending alarm. /RES halts the
    //      TOD at 1:00:00.0 AM; a Clock-mode TENTHS write restarts it,
    //      exactly as after any other stop.
    //
    // 6. ALARM
    //    * alarm_pending is set when a tick advances the clock ONTO
    //      exactly the alarm time — all four full bytes compared,
    //      including the AM/PM bit of hours. Merely writing the clock
    //      equal to the alarm does not fire; a stopped clock cannot fire.
    //    * Matching is independent of mode (fires while the registers are
    //      addressed as Alarm, too).
    //    * Sticky: nothing but acknowledge_alarm() clears it;
    //      acknowledging with nothing pending is a no-op.
    //
    // MODELING CHOICES where hardware documentation is ambiguous — each
    // is pinned by exactly one or two tests, so flip the CONTRACT line
    // and those tests if your reference says otherwise:
    //    (a) 5/6 cycles per tenth at F50/F60          (real time on both)
    //    (b) new(): F50, stopped, hours $01            (reset state)
    //    (c) a tenths write resynchronises the divider
    //    (d) Alarm-mode writes never touch the run state
    //    (e) the AM/PM bit takes part in the alarm comparison
    //    (f) the alarm fires on tick-arrival, not on write or level
    // =====================================================================

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    /// Canonical set-time idiom: write hours (stops the clock), then
    /// minutes, seconds, and finally tenths (restarts it). Leaves the
    /// clock RUNNING with the divider resynchronised.
    fn set_time(c: &mut Clock, hours: u8, minutes: u8, seconds: u8, tenths: u8) {
        c.set_mode(ClockMode::Clock);
        c.write(Clock::HOURS, hours);
        c.write(Clock::MINUTES, minutes);
        c.write(Clock::SECONDS, seconds);
        c.write(Clock::TENTHS, tenths);
    }

    /// Set the clock and leave it STOPPED: tenths first, hours last —
    /// no trailing tenths write, so nothing restarts the clock.
    fn set_time_stopped(c: &mut Clock, hours: u8, minutes: u8, seconds: u8, tenths: u8) {
        c.set_mode(ClockMode::Clock);
        c.write(Clock::TENTHS, tenths);
        c.write(Clock::HOURS, hours); // stops, for good this time
        c.write(Clock::MINUTES, minutes);
        c.write(Clock::SECONDS, seconds);
    }

    /// Canonical alarm-set idiom (CRB7 toggled around the writes).
    fn set_alarm(c: &mut Clock, hours: u8, minutes: u8, seconds: u8, tenths: u8) {
        c.set_mode(ClockMode::Alarm);
        c.write(Clock::HOURS, hours);
        c.write(Clock::MINUTES, minutes);
        c.write(Clock::SECONDS, seconds);
        c.write(Clock::TENTHS, tenths);
        c.set_mode(ClockMode::Clock);
    }

    /// Atomic time read: hours FIRST (latches), tenths LAST (releases).
    /// Returns (hours, minutes, seconds, tenths).
    fn read_time(c: &mut Clock) -> (u8, u8, u8, u8) {
        c.set_mode(ClockMode::Clock);
        let hours = c.read(Clock::HOURS);
        let minutes = c.read(Clock::MINUTES);
        let seconds = c.read(Clock::SECONDS);
        let tenths = c.read(Clock::TENTHS);
        (hours, minutes, seconds, tenths)
    }

    /// Advance the clock by `tenths` tenths of a second at the CURRENT
    /// frequency (tenths × ratio power-line cycles).
    fn tick_tenths(c: &mut Clock, tenths: u32) {
        let cycles = match c.frequency() {
            Frequency::F50 => 5u32,
            Frequency::F60 => 6,
        };
        for _ in 0..tenths * cycles {
            c.tick();
        }
    }

    // ---------------------------------------------------------------------
    // Reset state
    // ---------------------------------------------------------------------

    #[test]
    fn new_clock_is_stopped_at_one_oclock_in_clock_mode_at_50_hz() {
        // /RES halts the TOD at 1:00:00.0 AM with everything else zeroed;
        // only a Clock-mode tenths write can restart it.
        let mut c = Clock::new();
        assert!(!c.running());
        assert_eq!(c.mode(), ClockMode::Clock);
        assert_eq!(c.frequency(), Frequency::F50);
        assert!(!c.alarm_pending());
        assert_eq!(read_time(&mut c), (0x01, 0x00, 0x00, 0x00));

        // and it stays frozen until nudged:
        tick_tenths(&mut c, 50);
        assert_eq!(read_time(&mut c), (0x01, 0x00, 0x00, 0x00));
    }

    // ---------------------------------------------------------------------
    // Register decode and masks
    // ---------------------------------------------------------------------

    #[test]
    fn write_then_read_round_trips_all_four_registers() {
        let cases = [
            (Clock::TENTHS, 0x09u8),
            (Clock::SECONDS, 0x59),
            (Clock::MINUTES, 0x34),
            (Clock::HOURS, 0x92), // 12 PM
        ];
        for (off, val) in cases {
            let mut c = Clock::new();
            c.write(off, val);
            assert_eq!(c.read(off), val, "round trip at offset {off}");
        }
    }

    #[test]
    fn tenths_ignores_bits_4_to_7() {
        let mut c = Clock::new();
        c.write(Clock::TENTHS, 0xFF);
        assert_eq!(c.read(Clock::TENTHS), 0x0F);
        c.write(Clock::TENTHS, 0x80);
        assert_eq!(c.read(Clock::TENTHS), 0x00);
        c.write(Clock::TENTHS, 0xF9);
        assert_eq!(c.read(Clock::TENTHS), 0x09);
    }

    #[test]
    fn seconds_and_minutes_ignore_bit_7() {
        for &off in &[Clock::SECONDS, Clock::MINUTES] {
            let mut c = Clock::new();
            c.write(off, 0xFF);
            assert_eq!(
                c.read(off),
                0x7F,
                "bit 7 is not implemented at offset {off}"
            );
            c.write(off, 0x80);
            assert_eq!(c.read(off), 0x00);
        }
    }

    #[test]
    fn hours_ignores_bit_6() {
        // (written, expected read-back). A fresh clock per case keeps the latch
        // unarmed, so every read is a transparent read of the live register.
        for &(written, expected) in &[(0xD2u8, 0x92u8), (0x40, 0x00), (0xFF, 0x9F)] {
            let mut c = Clock::new();
            c.write(Clock::HOURS, written);
            assert_eq!(
                c.read(Clock::HOURS),
                expected,
                "mask check for {written:#04x}"
            );
        }
    }

    #[test]
    fn the_alarm_registers_use_the_same_masks() {
        // The alarm is write-only now, so pin its masks functionally:
        // junk in the unimplemented bits must neither break nor shift
        // the match.
        let mut c = Clock::new();
        c.set_mode(ClockMode::Alarm);
        c.write(Clock::TENTHS, 0xF7); // bits 4–7 ignored -> $07
        c.write(Clock::SECONDS, 0xD6); // bit 7 ignored    -> $56
        c.write(Clock::MINUTES, 0xB4); // bit 7 ignored    -> $34
        c.write(Clock::HOURS, 0x45); // bit 6 ignored    -> $05
        c.set_mode(ClockMode::Clock);
        set_time(&mut c, 5, 0x34, 0x56, 6); // running, one tenth short
        assert!(!c.alarm_pending());
        tick_tenths(&mut c, 1); // -> 5:34:56.7 = the masked alarm
        assert!(c.alarm_pending());
    }

    // ---------------------------------------------------------------------
    // Mode (CRB bit 7)
    // ---------------------------------------------------------------------

    #[test]
    fn set_mode_round_trips() {
        let mut c = Clock::new();
        assert_eq!(c.mode(), ClockMode::Clock);
        c.set_mode(ClockMode::Alarm);
        assert_eq!(c.mode(), ClockMode::Alarm);
        c.set_mode(ClockMode::Clock);
        assert_eq!(c.mode(), ClockMode::Clock);
    }

    #[test]
    fn mode_switches_are_bookkeeping_only() {
        let mut c = Clock::new();
        set_time(&mut c, 6, 7, 8, 9);
        c.set_mode(ClockMode::Alarm);
        c.set_mode(ClockMode::Clock);
        assert_eq!(read_time(&mut c), (0x06, 0x07, 0x08, 0x09));

        // the clock keeps ticking while the registers are addressed as alarm:
        c.set_mode(ClockMode::Alarm);
        tick_tenths(&mut c, 2);
        c.set_mode(ClockMode::Clock);
        assert_eq!(read_time(&mut c), (0x06, 0x07, 0x09, 0x01));

        // and mode switches never change the run state:
        assert!(c.running());
        c.write(Clock::HOURS, 0x06); // stop
        c.set_mode(ClockMode::Alarm);
        c.set_mode(ClockMode::Clock);
        assert!(!c.running());
    }

    #[test]
    fn reads_return_the_clock_in_both_modes() {
        // "The ALARM is write-only; any read of a TOD address will read
        // time regardless of the state of the ALARM access bit."
        let mut c = Clock::new();
        set_time(&mut c, 0x0A, 0x11, 0x22, 0x03); // clock: 10:11:22.3 AM
        set_alarm(&mut c, 0x81, 0x44, 0x55, 0x06); // alarm:  1:44:55.6 PM
        c.set_mode(ClockMode::Alarm);
        assert_eq!(c.read(Clock::HOURS), 0x0A, "time, not the alarm");
        assert_eq!(c.read(Clock::MINUTES), 0x11);
        assert_eq!(c.read(Clock::SECONDS), 0x22);
        assert_eq!(c.read(Clock::TENTHS), 0x03); // (releases the latch too)
        c.set_mode(ClockMode::Clock);
        assert_eq!(read_time(&mut c), (0x0A, 0x11, 0x22, 0x03));

        // clock writes never leak into the alarm — pinned functionally
        // now that the alarm can't be read back: after overwriting the
        // whole clock, the alarm still fires at its original time.
        set_time(&mut c, 0x11, 0x59, 0x59, 9); // 11:59:59.9 AM
        tick_tenths(&mut c, 62_957); // -> exactly 1:44:55.6 PM
        assert_eq!(read_time(&mut c), (0x81, 0x44, 0x55, 0x06));
        assert!(c.alarm_pending());
    }

    // ---------------------------------------------------------------------
    // The read latch
    // ---------------------------------------------------------------------

    #[test]
    fn reading_hours_latches_the_time_until_tenths_is_read() {
        let mut c = Clock::new();
        set_time(&mut c, 9, 0x58, 0x59, 7); // 9:58:59.7, running
        let h = c.read(Clock::HOURS); // latches (9, 58, 59, 7)
        assert_eq!(h, 0x09);

        tick_tenths(&mut c, 5); // clock is now 9:59:00.2 — still running
        assert_eq!(c.read(Clock::MINUTES), 0x58, "latched, not live");
        assert_eq!(c.read(Clock::SECONDS), 0x59);
        assert_eq!(c.read(Clock::TENTHS), 0x07); // the snapshot — and it releases
        assert_eq!(c.read(Clock::TENTHS), 0x02, "live again after the release");
        assert_eq!(c.read(Clock::SECONDS), 0x00);
        assert_eq!(c.read(Clock::MINUTES), 0x59);
    }

    #[test]
    fn only_a_tenths_read_releases_the_latch() {
        let mut c = Clock::new();
        set_time(&mut c, 3, 0, 0, 0); // 3:00:00.0, running
        c.read(Clock::HOURS); // latches (3, 0, 0, 0)
        tick_tenths(&mut c, 10); // one second -> 3:00:01.0

        assert_eq!(c.read(Clock::SECONDS), 0x00); // snapshot
        assert_eq!(c.read(Clock::MINUTES), 0x00); // snapshot
        assert_eq!(c.read(Clock::HOURS), 0x03); // snapshot hours, keeps the latch
        assert_eq!(
            c.read(Clock::SECONDS),
            0x00,
            "a second HOURS read must not refresh the snapshot"
        );
        assert_eq!(c.read(Clock::TENTHS), 0x00); // snapshot — releases NOW
        assert_eq!(c.read(Clock::SECONDS), 0x01); // live again
    }

    #[test]
    fn unlatched_reads_are_live() {
        // Without a prior HOURS read there is no latch: consecutive reads
        // track the running clock.
        let mut c = Clock::new();
        set_time(&mut c, 4, 0, 0, 0);
        assert_eq!(c.read(Clock::TENTHS), 0x00);
        tick_tenths(&mut c, 3);
        assert_eq!(c.read(Clock::TENTHS), 0x03);
        tick_tenths(&mut c, 10); // one second
        assert_eq!(c.read(Clock::SECONDS), 0x01);
    }

    #[test]
    fn writes_never_touch_the_latch() {
        // The classic gotcha: after a HOURS read, fresh writes are invisible
        // until the tenths read releases the latch.
        let mut c = Clock::new();
        set_time(&mut c, 2, 0, 0, 0); // 2:00:00.0, running
        c.read(Clock::HOURS); // latch = (2, 0, 0, 0)
        c.write(Clock::SECONDS, 0x30); // live regs -> 2:00:30.0
        c.write(Clock::MINUTES, 0x45); // live regs -> 2:45:30.0
        c.write(Clock::TENTHS, 0x05); // even a tenths WRITE must not release
        assert_eq!(
            c.read(Clock::MINUTES),
            0x00,
            "latched reads hide fresh writes"
        );
        assert_eq!(c.read(Clock::SECONDS), 0x00);
        assert_eq!(c.read(Clock::TENTHS), 0x00); // snapshot — releases NOW
        assert_eq!(c.read(Clock::SECONDS), 0x30);
        assert_eq!(c.read(Clock::MINUTES), 0x45);
        assert_eq!(c.read(Clock::TENTHS), 0x05);
    }

    #[test]
    fn the_latch_survives_a_mode_switch() {
        let mut c = Clock::new();
        set_time(&mut c, 6, 0x30, 0x15, 4); // 6:30:15.4, running
        c.read(Clock::HOURS); // latch = 6:30:15.4

        c.set_mode(ClockMode::Alarm);
        c.set_mode(ClockMode::Clock);
        // (no reads while in Alarm mode: the latch must survive untouched)
        tick_tenths(&mut c, 2); // 6:30:15.6
        assert_eq!(
            c.read(Clock::SECONDS),
            0x15,
            "the latch survived the mode trip"
        );
        assert_eq!(c.read(Clock::TENTHS), 0x04); // snapshot — releases
        assert_eq!(c.read(Clock::TENTHS), 0x06, "live again");
    }

    #[test]
    fn alarm_mode_reads_read_the_clock_and_operate_the_latch() {
        // Reads are mode-independent: they return time (never the
        // write-only alarm) and latch/release exactly as in Clock mode.
        let mut c = Clock::new();
        set_time(&mut c, 2, 0, 0, 0); // running
        set_alarm(&mut c, 9, 0, 0, 0);
        c.set_mode(ClockMode::Alarm);
        assert_eq!(
            c.read(Clock::HOURS),
            0x02,
            "the clock's 2, not the alarm's 9"
        );
        tick_tenths(&mut c, 2);
        assert_eq!(c.read(Clock::SECONDS), 0x00, "latched at 2:00:00.0");
        assert_eq!(c.read(Clock::TENTHS), 0x00); // snapshot — releases
        assert_eq!(c.read(Clock::TENTHS), 0x02, "live again");
        c.set_mode(ClockMode::Clock);
    }

    #[test]
    fn atomic_read_is_consistent_across_ticks() {
        // The whole point of the latch: read hours first, then everything
        // else is a coherent snapshot even while the clock runs on.
        let mut c = Clock::new();
        set_time(&mut c, 0x11, 0x59, 0x59, 9); // 11:59:59.9 AM, running
        let h = c.read(Clock::HOURS); // latches the whole time
        assert_eq!(h, 0x11);
        tick_tenths(&mut c, 3); // clock is now 12:00:00.2 PM
        assert_eq!(c.read(Clock::MINUTES), 0x59);
        assert_eq!(c.read(Clock::SECONDS), 0x59);
        assert_eq!(
            c.read(Clock::HOURS),
            0x11,
            "no refresh: snapshot hours, not the live PM hours"
        );
        assert_eq!(c.read(Clock::TENTHS), 0x09); // releases
        assert_eq!(
            c.read(Clock::TENTHS),
            0x02,
            "the clock ran on while latched"
        );
    }

    #[test]
    fn unlatched_reads_can_tear_across_a_carry() {
        // Reading WITHOUT the hours-first latch can mix two instants —
        // which is exactly why the latch exists.
        let mut c = Clock::new();
        set_time(&mut c, 5, 0x59, 0x59, 9); // 5:59:59.9, running
        let seconds = c.read(Clock::SECONDS); // live, no latch
        assert_eq!(seconds, 0x59);
        tick_tenths(&mut c, 1); // -> 6:00:00.0
        let minutes = c.read(Clock::MINUTES); // live
        // (minutes = 00, seconds = 59) matches NEITHER 5:59:59.9 (min 59)
        // nor 6:00:00.0 (sec 00): the two reads straddle the carry.
        assert_eq!((minutes, seconds), (0x00, 0x59));
    }

    // ---------------------------------------------------------------------
    // Stop / restart on write
    // ---------------------------------------------------------------------

    #[test]
    fn writing_hours_stops_the_clock_even_if_the_value_is_unchanged() {
        let mut c = Clock::new();
        set_time(&mut c, 1, 2, 3, 4);
        assert!(c.running());
        c.write(Clock::HOURS, 0x01); // same value — still a stop
        assert!(!c.running());
        tick_tenths(&mut c, 50);
        assert_eq!(read_time(&mut c), (0x01, 0x02, 0x03, 0x04), "frozen");
    }

    #[test]
    fn writing_seconds_or_minutes_neither_stops_nor_restarts() {
        let mut c = Clock::new();
        set_time(&mut c, 1, 0, 0, 0); // running
        c.write(Clock::SECONDS, 0x30); // plain store
        assert!(c.running());
        tick_tenths(&mut c, 10); // one second
        assert_eq!(c.read(Clock::SECONDS), 0x31);

        c.write(Clock::HOURS, 0x01); // stop
        c.write(Clock::MINUTES, 0x45); // plain store, stays stopped
        assert!(!c.running());
        tick_tenths(&mut c, 10);
        assert_eq!(read_time(&mut c), (0x01, 0x45, 0x31, 0x00));
    }

    #[test]
    fn writing_tenths_restarts_a_stopped_clock() {
        let mut c = Clock::new();
        set_time_stopped(&mut c, 0x05, 0x30, 0x15, 0x09);
        assert!(!c.running());
        c.write(Clock::TENTHS, 0x09); // same value — still a restart
        assert!(c.running());
        tick_tenths(&mut c, 1); // 5:30:15.9 -> 5:30:16.0
        assert_eq!(read_time(&mut c), (0x05, 0x30, 0x16, 0x00));
    }

    #[test]
    fn writing_tenths_keeps_a_running_clock_running() {
        let mut c = Clock::new();
        set_time(&mut c, 0, 0, 0, 0x03); // running, tenths 3
        c.write(Clock::TENTHS, 0x07); // restart while already running
        assert!(c.running());
        assert_eq!(c.read(Clock::TENTHS), 0x07);
        tick_tenths(&mut c, 1);
        assert_eq!(c.read(Clock::TENTHS), 0x08);
    }

    #[test]
    fn writing_hours_last_leaves_the_clock_stopped() {
        // The classic footgun: if hours is the LAST register you write,
        // nothing restarts the clock.
        let mut c = Clock::new();
        c.write(Clock::TENTHS, 0x01);
        c.write(Clock::MINUTES, 0x02);
        c.write(Clock::SECONDS, 0x03);
        c.write(Clock::HOURS, 0x04);
        assert!(!c.running());
        tick_tenths(&mut c, 5); // no-ops
        assert_eq!(read_time(&mut c), (0x04, 0x02, 0x03, 0x01));
    }

    #[test]
    fn alarm_mode_writes_never_stop_or_start_the_clock() {
        let mut c = Clock::new();
        set_time(&mut c, 3, 0, 0, 0); // running
        c.set_mode(ClockMode::Alarm);
        c.write(Clock::HOURS, 0x08); // the ALARM's hours
        assert!(c.running(), "alarm writes must not stop the clock");
        c.write(Clock::TENTHS, 0x05); // the ALARM's tenths
        assert!(c.running());
        c.set_mode(ClockMode::Clock);
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x03, 0x00, 0x00, 0x01), "kept running");

        // now stop it, and prove alarm writes can't restart it either:
        c.write(Clock::HOURS, 0x03); // clock-mode write: stops
        assert!(!c.running());
        c.set_mode(ClockMode::Alarm);
        c.write(Clock::TENTHS, 0x02);
        assert!(!c.running(), "only a CLOCK-mode tenths write restarts");
        c.set_mode(ClockMode::Clock);
        c.write(Clock::TENTHS, 0x01);
        assert!(c.running());
    }

    #[test]
    fn the_canonical_write_sequence_stops_then_restarts_the_clock() {
        // LDA h / STA $DC0B ; LDA m / STA $DC0A ; LDA s / STA $DC09 ;
        // LDA t / STA $DC08
        let mut c = Clock::new();
        c.write(Clock::HOURS, 0x0C); // 12 AM — clock stops
        assert!(!c.running());
        c.write(Clock::MINUTES, 0x34);
        c.write(Clock::SECONDS, 0x56);
        assert!(!c.running(), "only the tenths write restarts");
        c.write(Clock::TENTHS, 0x07);
        assert!(c.running());
        assert_eq!(read_time(&mut c), (0x0C, 0x34, 0x56, 0x07));
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x0C, 0x34, 0x56, 0x08));
    }

    #[test]
    fn reads_never_change_the_run_state() {
        let mut c = Clock::new();
        set_time(&mut c, 1, 0, 0, 0);
        let _ = read_time(&mut c); // latches and releases
        assert!(c.running());
        c.write(Clock::HOURS, 0x01); // stop
        let _ = read_time(&mut c);
        assert!(!c.running());
    }

    // ---------------------------------------------------------------------
    // Clocking: BCD counters
    // ---------------------------------------------------------------------

    #[test]
    fn at_50_hz_five_input_cycles_advance_one_tenth() {
        let mut c = Clock::new(); // stopped after /RES
        c.write(Clock::TENTHS, 0x00); // nudge: divider 0, running
        for _ in 0..4 {
            c.tick();
        }
        assert_eq!(c.read(Clock::TENTHS), 0x00);
        c.tick(); // 5th power-line cycle
        assert_eq!(c.read(Clock::TENTHS), 0x01);
    }

    #[test]
    fn at_60_hz_six_input_cycles_advance_one_tenth() {
        let mut c = Clock::new();
        c.write(Clock::TENTHS, 0x00); // nudge the /RES-stopped clock
        c.set_frequency(Frequency::F60);
        for _ in 0..5 {
            c.tick();
        }
        assert_eq!(c.read(Clock::TENTHS), 0x00);
        c.tick(); // 6th power-line cycle
        assert_eq!(c.read(Clock::TENTHS), 0x01);
    }

    #[test]
    fn tenths_carry_from_9_into_the_seconds() {
        let mut c = Clock::new();
        set_time(&mut c, 0, 0, 0, 9);
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x00, 0x00, 0x01, 0x00));
    }

    #[test]
    fn seconds_count_in_bcd_not_hex() {
        let mut c = Clock::new();
        set_time(&mut c, 0, 0, 0x09, 9); // 0:00:09.9
        tick_tenths(&mut c, 1);
        assert_eq!(
            read_time(&mut c),
            (0x00, 0x00, 0x10, 0x00),
            "BCD: $09 -> $10, never $0A"
        );
        set_time(&mut c, 0, 0, 0x19, 9);
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x00, 0x00, 0x20, 0x00));
    }

    #[test]
    fn seconds_carry_from_59_into_the_minutes() {
        let mut c = Clock::new();
        set_time(&mut c, 0, 0, 0x59, 9);
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x00, 0x01, 0x00, 0x00));
    }

    #[test]
    fn minutes_count_in_bcd_not_hex() {
        let mut c = Clock::new();
        set_time(&mut c, 0, 0x09, 0x59, 9); // 0:09:59.9
        tick_tenths(&mut c, 1);
        assert_eq!(
            read_time(&mut c),
            (0x00, 0x10, 0x00, 0x00),
            "minutes count BCD too"
        );
    }

    #[test]
    fn minutes_carry_from_59_into_the_hours() {
        let mut c = Clock::new();
        set_time(&mut c, 0x00, 0x59, 0x59, 9); // hour $00 is writable
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x01, 0x00, 0x00, 0x00));
    }

    #[test]
    fn the_am_pm_flag_flips_only_between_11_and_12() {
        // The 11 -> 12 carry flips AM/PM (11:59:59.9 AM is followed by
        // noon); the 12 -> 1 carry does not.
        let cases = [
            (0x11u8, 0x92u8), // 11:59:59.9 AM -> 12:00:00.0 PM : flip
            (0x91, 0x12),     // 11:59:59.9 PM -> 12:00:00.0 AM : flip
            (0x12, 0x01),     // 12:59:59.9 AM ->  1:00:00.0 AM : no flip
            (0x92, 0x81),     // 12:59:59.9 PM ->  1:00:00.0 PM : no flip
        ];
        for &(before, after) in &cases {
            let mut c = Clock::new();
            set_time(&mut c, before, 0x59, 0x59, 9);
            tick_tenths(&mut c, 1);
            assert_eq!(
                read_time(&mut c),
                (after, 0x00, 0x00, 0x00),
                "{before:#04x} -> {after:#04x}"
            );
        }
    }

    #[test]
    fn a_bare_tenths_write_nudges_the_reset_clock_into_action() {
        // "It is permissible to write to only the 10ths register to
        // 'nudge' the clock into action, in which case, following a
        // hardware reset, the clock will start at 1:00:00.0."
        let mut c = Clock::new();
        c.write(Clock::TENTHS, 0x00);
        assert!(c.running());
        assert_eq!(read_time(&mut c), (0x01, 0x00, 0x00, 0x00));
        tick_tenths(&mut c, 3_600 * 10); // one hour
        assert_eq!(read_time(&mut c), (0x02, 0x00, 0x00, 0x00));
    }

    #[test]
    fn a_12_hour_period_flips_am_pm_a_24_hour_period_does_not() {
        let mut c = Clock::new();
        set_time(&mut c, 0x87, 0x32, 0x54, 0x06); // 7:32:54.6 PM
        tick_tenths(&mut c, 43_200 * 10); // 12 hours
        assert_eq!(
            read_time(&mut c),
            (0x07, 0x32, 0x54, 0x06),
            "same digits, AM/PM flipped"
        );
        tick_tenths(&mut c, 43_200 * 10); // a full day in total
        assert_eq!(read_time(&mut c), (0x87, 0x32, 0x54, 0x06));
    }

    // ---------------------------------------------------------------------
    // Frequency
    // ---------------------------------------------------------------------

    #[test]
    fn frequency_round_trips() {
        let mut c = Clock::new();
        assert_eq!(c.frequency(), Frequency::F50); // pinned default (PAL)
        c.set_frequency(Frequency::F60);
        assert_eq!(c.frequency(), Frequency::F60);
        c.set_frequency(Frequency::F50);
        assert_eq!(c.frequency(), Frequency::F50);
    }

    #[test]
    fn both_mains_frequencies_keep_a_real_second() {
        let mut c = Clock::new(); // F50, 1:00:00.0, stopped
        c.write(Clock::TENTHS, 0x00); // nudge
        for _ in 0..50 {
            c.tick(); // 50 power cycles = 1 s at 50 Hz
        }
        assert_eq!((c.read(Clock::SECONDS), c.read(Clock::TENTHS)), (1, 0));

        set_time(&mut c, 0, 0, 0, 0);
        c.set_frequency(Frequency::F60);
        for _ in 0..60 {
            c.tick(); // 60 power cycles = 1 s at 60 Hz
        }
        assert_eq!((c.read(Clock::SECONDS), c.read(Clock::TENTHS)), (1, 0));
    }

    #[test]
    fn changing_frequency_mid_run_preserves_time_and_run_state() {
        let mut c = Clock::new();
        set_time(&mut c, 8, 0, 0, 0);
        c.set_frequency(Frequency::F60);
        assert!(c.running());
        assert_eq!(
            read_time(&mut c),
            (0x08, 0x00, 0x00, 0x00),
            "unchanged by the switch"
        );
        // 60 cycles at F60 is one second regardless of the divider phase
        // the switch left behind (any phase is < 6):
        for _ in 0..60 {
            c.tick();
        }
        assert_eq!(read_time(&mut c), (0x08, 0x00, 0x01, 0x00));
        // and a frequency change never starts a stopped clock:
        c.write(Clock::HOURS, 0x08); // stop
        c.set_frequency(Frequency::F50);
        assert!(!c.running());
    }

    // ---------------------------------------------------------------------
    // Divider phase
    // ---------------------------------------------------------------------

    #[test]
    fn a_clock_mode_tenths_write_resynchronises_the_divider() {
        let mut c = Clock::new(); // F50, divider 0, stopped
        c.write(Clock::TENTHS, 0x00); // start it
        c.tick();
        c.tick(); // divider = 2, tenths still 0
        c.write(Clock::TENTHS, 0x00); // resynchronises: divider := 0
        for _ in 0..3 {
            c.tick();
        }
        // Without the resync the divider would sit at 5 by now and the
        // tenth would already have ticked over:
        assert_eq!(
            c.read(Clock::TENTHS),
            0x00,
            "the write resynchronised the divider"
        );
        c.tick();
        c.tick(); // 5 cycles since the write
        assert_eq!(c.read(Clock::TENTHS), 0x01);
    }

    // ---------------------------------------------------------------------
    // The alarm event (future ICR bit 2)
    // ---------------------------------------------------------------------

    #[test]
    fn the_alarm_fires_when_the_clock_ticks_onto_the_alarm_time() {
        let mut c = Clock::new();
        set_alarm(&mut c, 6, 0x15, 0x32, 4); // 6:15:32.4 — tenth resolution
        set_time(&mut c, 6, 0x15, 0x32, 3);
        tick_tenths(&mut c, 1);
        assert_eq!(read_time(&mut c), (0x06, 0x15, 0x32, 0x04));
        assert!(c.alarm_pending());
    }

    #[test]
    fn the_alarm_fires_exactly_on_the_matching_tenth_not_before() {
        let mut c = Clock::new();
        set_alarm(&mut c, 6, 0x15, 0x32, 4);
        set_time(&mut c, 6, 0x15, 0x32, 2);
        tick_tenths(&mut c, 1); // -> 6:15:32.3: one tenth short
        assert!(!c.alarm_pending());
        tick_tenths(&mut c, 1); // -> 6:15:32.4: the match
        assert!(c.alarm_pending());
    }

    #[test]
    fn merely_writing_the_clock_to_the_alarm_time_does_not_fire_it() {
        let mut c = Clock::new();
        set_alarm(&mut c, 4, 0, 0, 0);
        set_time(&mut c, 4, 0, 0, 0); // running, exactly ON the alarm
        assert!(
            !c.alarm_pending(),
            "a write onto the alarm time must not fire it"
        );
        tick_tenths(&mut c, 1); // 4:00:00.1 — moved past
        assert!(!c.alarm_pending(), "no tick ever arrived at 4:00:00.0");
    }

    #[test]
    fn a_stopped_clock_cannot_fire_the_alarm() {
        let mut c = Clock::new();
        set_alarm(&mut c, 5, 0, 0, 0);
        set_time_stopped(&mut c, 5, 0, 0, 0); // stopped exactly on the alarm
        tick_tenths(&mut c, 10);
        assert!(!c.alarm_pending());
    }

    #[test]
    fn the_alarm_is_sticky_until_acknowledged() {
        let mut c = Clock::new();
        set_alarm(&mut c, 3, 0, 0, 0);
        set_time(&mut c, 2, 0x59, 0x59, 9);
        tick_tenths(&mut c, 1); // -> 3:00:00.0
        assert!(c.alarm_pending());
        tick_tenths(&mut c, 50); // five seconds past
        assert!(c.alarm_pending(), "sticky across ticks");
        c.acknowledge_alarm();
        assert!(!c.alarm_pending());
        tick_tenths(&mut c, 10);
        assert!(
            !c.alarm_pending(),
            "the moment is gone; no refire until tomorrow"
        );
    }

    #[test]
    fn nothing_but_acknowledge_clears_the_alarm() {
        let mut c = Clock::new();
        set_alarm(&mut c, 2, 0, 0, 0);
        set_time(&mut c, 1, 0x59, 0x59, 9);
        tick_tenths(&mut c, 1); // -> 2:00:00.0: alarm!
        assert!(c.alarm_pending());

        c.set_mode(ClockMode::Alarm);
        c.set_mode(ClockMode::Clock);
        assert!(c.alarm_pending());
        c.set_frequency(Frequency::F60);
        assert!(c.alarm_pending());
        set_alarm(&mut c, 9, 0, 0, 0); // rewriting the alarm
        assert!(c.alarm_pending());
        c.write(Clock::HOURS, 0x02); // even stopping the clock
        assert!(c.alarm_pending());
        let _ = read_time(&mut c); // latching reads
        assert!(c.alarm_pending());
        tick_tenths(&mut c, 3); // stopped: no-ops
        assert!(c.alarm_pending());

        c.acknowledge_alarm();
        assert!(!c.alarm_pending());
    }

    #[test]
    fn acknowledging_without_a_pending_alarm_is_harmless() {
        let mut c = Clock::new();
        c.acknowledge_alarm();
        assert!(!c.alarm_pending());
        // and it stays armed afterwards:
        set_alarm(&mut c, 2, 0, 0, 0);
        set_time(&mut c, 1, 0x59, 0x59, 9);
        tick_tenths(&mut c, 1);
        assert!(c.alarm_pending());
    }

    #[test]
    fn the_alarm_refires_after_a_full_24_hour_cycle() {
        let mut c = Clock::new();
        set_alarm(&mut c, 3, 0, 0, 0); // 3:00:00.0 AM
        set_time(&mut c, 2, 0x59, 0x59, 9);
        tick_tenths(&mut c, 1); // -> 3:00:00.0 AM
        assert!(c.alarm_pending());
        c.acknowledge_alarm();
        tick_tenths(&mut c, 86_400 * 10); // a full day (passes 3 PM on the way)
        assert_eq!(read_time(&mut c), (0x03, 0x00, 0x00, 0x00));
        assert!(c.alarm_pending(), "the alarm must refire the next day");
    }

    #[test]
    fn the_alarm_comparison_includes_the_am_pm_flag() {
        let mut c = Clock::new();
        set_alarm(&mut c, 0x85, 0, 0, 0); // alarm 5:00:00.0 PM only
        set_time(&mut c, 0x04, 0x59, 0x59, 9); // 4:59:59.9 AM
        tick_tenths(&mut c, 1); // -> 5:00:00.0 AM
        assert!(!c.alarm_pending(), "the AM pass must not match a PM alarm");
        set_time(&mut c, 0x84, 0x59, 0x59, 9); // 4:59:59.9 PM
        tick_tenths(&mut c, 1); // -> 5:00:00.0 PM
        assert!(c.alarm_pending(), "the PM pass must match");
    }

    #[test]
    fn the_alarm_fires_while_the_registers_are_addressed_as_alarm() {
        let mut c = Clock::new();
        set_alarm(&mut c, 8, 0, 0, 0);
        set_time(&mut c, 7, 0x59, 0x59, 9);
        c.set_mode(ClockMode::Alarm);
        tick_tenths(&mut c, 1); // the clock ticks regardless of the mode
        c.set_mode(ClockMode::Clock);
        assert_eq!(read_time(&mut c), (0x08, 0x00, 0x00, 0x00));
        assert!(c.alarm_pending());
    }

    #[test]
    fn the_default_zero_alarm_never_fires() {
        // Hour $00 occurs only while the clock is /RES-frozen, before
        // the first nudge; once running, hours count 1 -> ... -> 12 -> 1
        // ... so no tick ever ARRIVES at 0:00:00.0.
        let mut c = Clock::new(); // clock 1:00:00.0 stopped, alarm 0:00:00.0
        c.write(Clock::TENTHS, 0x00); // nudge: 1:00:00.0 AM, running
        tick_tenths(&mut c, 43_200 * 10); // a full 12 hours
        assert_eq!(
            read_time(&mut c),
            (0x81, 0x00, 0x00, 0x00),
            "1:00 AM + 12h = 1:00 PM"
        );
        assert!(!c.alarm_pending());
    }

    // ---------------------------------------------------------------------
    // End-to-end idiom
    // ---------------------------------------------------------------------

    #[test]
    fn alarm_at_noon_end_to_end() {
        let mut c = Clock::new();
        set_alarm(&mut c, AM_PM_FLAG | 0x12, 0, 0, 0); // alarm at noon
        set_time(&mut c, 0x11, 0x59, 0x58, 9); // 11:59:58.9 AM
        tick_tenths(&mut c, 11); // 11 tenths -> exactly 12:00:00.0 PM
        assert_eq!(read_time(&mut c), (AM_PM_FLAG | 0x12, 0x00, 0x00, 0x00));
        assert!(c.alarm_pending());
    }
}
