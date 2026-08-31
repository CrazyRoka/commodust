/// Run mode of a CIA interval timer (bit 3 of CRA/CRB).
///
/// * `Continuous` — bit 3 = 0: counter is reloaded from the latch on every
///   underflow and the timer keeps running.
/// * `OneShot`    — bit 3 = 1: the timer stops after the first underflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerMode {
    OneShot,
    Continuous,
}

/// One 16-bit interval timer of a MOS 6526 CIA (Timer A or Timer B).
#[derive(Debug)]
struct Timer {
    /// 16-bit down counter. This is what the CPU reads at $DC04/$DC05 (A)
    /// or $DC06/$DC07 (B).
    counter: u16,
    /// 16-bit latch (start value). Write-only from the CPU's point of view.
    latch: u16,
    /// CRA/CRB bit 0 (start/stop).
    running: bool,
    /// CRA/CRB bit 3 (run mode).
    mode: TimerMode,
    /// Sticky underflow event. On a real CIA this feeds the ICR
    /// (bit 0 for timer A, bit 1 for timer B) and the PB6/PB7 outputs.
    underflow_pending: bool,
}

impl Timer {
    /// State after /RES: CRA/CRB cleared (stopped, continuous),
    /// timer registers read $0000.
    fn new() -> Self {
        Self {
            counter: 0,
            latch: 0,
            running: false,
            mode: TimerMode::Continuous,
            underflow_pending: false,
        }
    }

    /// Low byte of the *counter* (never the latch), e.g. $DC04.
    fn read_low(&self) -> u8 {
        (self.counter & 0xFF) as u8
    }

    /// High byte of the *counter*, e.g. $DC05.
    fn read_high(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    /// Latch low byte ($DC04). Never touches the counter.
    fn write_low(&mut self, value: u8) {
        self.latch &= 0xFF00;
        self.latch |= value as u16;
    }

    /// Latch high byte ($DC05). If — and only if — the timer is stopped,
    /// this write also copies the updated latch into the counter ("force
    /// load"). While running, only the latch changes; the new value is
    /// used at the next underflow.
    fn write_high(&mut self, value: u8) {
        self.latch &= 0x00FF;
        self.latch |= (value as u16) << 8;

        if !self.running {
            self.counter = self.latch
        }
    }

    /// CRA/CRB bit 0 := 1. Starts counting. Does **not** reload the counter.
    fn start(&mut self) {
        self.running = true;
    }

    /// CRA/CRB bit 0 := 0. Freezes the counter at its current value.
    fn stop(&mut self) {
        self.running = false;
    }

    /// CRA/CRB bit 3. Pure bookkeeping: never starts/stops/loads anything.
    fn set_mode(&mut self, mode: TimerMode) {
        self.mode = mode;
    }

    /// CRA/CRB bit 4 strobe ("load latch into the timer once"):
    /// counter := latch, instantly, regardless of running state.
    /// Neither starts nor stops the timer, and is not an underflow.
    fn reload(&mut self) {
        self.counter = self.latch;
    }

    /// Advance by one φ2 clock. See the CONTRACT comment in the test module.
    fn tick(&mut self) {
        if self.running {
            if self.counter == 0 {
                self.counter = self.latch;
                self.underflow_pending = true;
                if self.mode == TimerMode::OneShot {
                    self.running = false;
                    self.counter = 0xFFFF;
                }
            } else {
                self.counter -= 1;
            }
        }
    }

    /// True if an underflow occurred that has not been acknowledged yet.
    fn underflow(&self) -> bool {
        self.underflow_pending
    }

    /// Acknowledge (clear) the pending underflow event. (Later: call this
    /// from your ICR read handler.)
    fn acknowledge_underflow(&mut self) {
        self.underflow_pending = false;
    }

    /// CRA/CRB bit 0 state.
    fn is_running(&self) -> bool {
        self.running
    }

    /// Current run mode.
    fn mode(&self) -> TimerMode {
        self.mode
    }

    /// Latch value. Write-only on real hardware; exposed for tests.
    fn latch(&self) -> u16 {
        self.latch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. REGISTERS
    //    * read_low / read_high always return the live *counter*, split into
    //      bytes. The latch is never CPU-visible.
    //    * write_low(v) : latch := (latch & $FF00) | v.   Never the counter.
    //    * write_high(v): latch := (latch & $00FF) | v<<8.
    //      Additionally, if the timer is STOPPED, the full updated latch is
    //      copied into the counter (the 6526 "force load on TA HI write").
    //
    // 2. START / STOP (CRA/CRB bit 0)
    //    * start(): running := true. It does NOT reload the counter. Real
    //      6526 (Lorenz, "A Software Model of the CIA6526", which VICE
    //      follows): the counter is loaded only by a write to the high byte
    //      while stopped, by the bit-4 load strobe, or by an underflow in
    //      continuous mode. The datasheet's "the latch is copied into the
    //      counter whenever the timer is started" describes the usual
    //      write-low / write-high / start idiom, where the load has already
    //      happened during the write-high.
    //    * stop(): freezes the counter where it is.
    //
    // 3. FORCE LOAD (CRA/CRB bit 4)
    //    * reload(): counter := latch, instantly, whether running or stopped.
    //      Not a start, not a stop, not an underflow.
    //
    // 4. CLOCKING (tick = one φ2 pulse)
    //      if !running                  -> nothing happens at all
    //      else if counter != 0          -> counter -= 1
    //                                        (reaching $0000 is NOT the underflow)
    //      else                          -> UNDERFLOW (the clock that decrements
    //                                        FROM $0000):
    //                                        - set the pending-event flag
    //                                        - Continuous : counter := latch
    //                                        - OneShot    : counter := $FFFF,
    //                                                       running := false
    //
    // 5. EVENT
    //    * Set only by an underflow; cleared only by acknowledge_underflow().
    //      Like the real ICR, it does not count events — just "something
    //      happened since you last looked".
    // =====================================================================

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    /// Build a stopped timer whose latch AND counter equal `value`
    /// (the standard KERNAL idiom: write low, then high, while stopped).
    fn timer_with(value: u16) -> Timer {
        let mut t = Timer::new();
        t.write_low((value & 0x00FF) as u8);
        t.write_high((value >> 8) as u8);
        t
    }

    /// Read the full 16-bit counter through the register interface.
    fn read_counter(t: &Timer) -> u16 {
        u16::from_le_bytes([t.read_low(), t.read_high()])
    }

    /// A one-shot timer that has counted down and expired.
    fn expired_one_shot(latch: u16) -> Timer {
        let mut t = timer_with(latch);
        t.set_mode(TimerMode::OneShot);
        t.start();
        for _ in 0..(latch as u32 + 2) {
            if !t.is_running() {
                break;
            }
            t.tick();
        }
        assert!(!t.is_running(), "fixture: one-shot must expire");
        t
    }

    // ---------------------------------------------------------------------
    // Reset state
    // ---------------------------------------------------------------------

    #[test]
    fn new_timer_is_stopped_continuous_and_zeroed() {
        // 6526 after /RES: CRA/CRB = 0 -> start bit clear (stopped),
        // run-mode bit clear (continuous); timer registers read $0000.
        let t = Timer::new();
        assert!(!t.is_running());
        assert_eq!(t.mode(), TimerMode::Continuous);
        assert_eq!(t.read_low(), 0x00);
        assert_eq!(t.read_high(), 0x00);
        assert_eq!(t.latch(), 0x0000);
        assert!(!t.underflow());
    }

    // ---------------------------------------------------------------------
    // Reading the counter ($DC04/$DC05)
    // ---------------------------------------------------------------------

    #[test]
    fn read_low_and_read_high_split_the_counter_into_bytes() {
        for value in [
            0x0000u16, 0x0001, 0x00FF, 0x0100, 0x0205, 0x1234, 0xFF00, 0xFFFE, 0xFFFF,
        ] {
            let t = timer_with(value);
            assert_eq!(
                t.read_low(),
                (value & 0x00FF) as u8,
                "low byte of {value:#06x}"
            );
            assert_eq!(
                t.read_high(),
                (value >> 8) as u8,
                "high byte of {value:#06x}"
            );
            assert_eq!(read_counter(&t), value);
        }
    }

    #[test]
    fn reads_track_the_live_counter_across_byte_boundaries() {
        let mut t = timer_with(0x0101);
        t.start();

        t.tick(); // $0100
        assert_eq!((t.read_high(), t.read_low()), (0x01, 0x00));
        t.tick(); // $00FF <- borrow out of the high byte
        assert_eq!((t.read_high(), t.read_low()), (0x00, 0xFF));
        t.tick(); // $00FE
        assert_eq!((t.read_high(), t.read_low()), (0x00, 0xFE));
        assert_eq!(read_counter(&t), 0x00FE);
    }

    #[test]
    fn reading_the_timer_never_disturbs_it() {
        let mut t = timer_with(0x0010);
        t.start();
        for _ in 0..5 {
            assert_eq!(t.read_low(), 0x10);
            assert_eq!(t.read_high(), 0x00);
        }
        assert_eq!(read_counter(&t), 0x0010); // reads never decrement
        assert!(!t.underflow());
        t.tick();
        assert_eq!(read_counter(&t), 0x000F);
    }

    // ---------------------------------------------------------------------
    // Writing the latch
    // ---------------------------------------------------------------------

    #[test]
    fn write_low_touches_only_the_latch_low_byte() {
        let mut t = timer_with(0x1234); // counter = latch = $1234, stopped
        t.write_low(0xAB);
        assert_eq!(t.latch(), 0x12AB);
        assert_eq!(
            read_counter(&t),
            0x1234,
            "TA LO writes must never reach the counter"
        );
    }

    #[test]
    fn write_low_while_running_does_not_disturb_the_counter() {
        let mut t = timer_with(0x0100);
        t.start();
        t.tick(); // $00FF
        t.write_low(0x38);
        assert_eq!(t.latch(), 0x0138);
        assert_eq!(read_counter(&t), 0x00FF);
        t.tick();
        assert_eq!(read_counter(&t), 0x00FE);
    }

    #[test]
    fn write_high_while_stopped_loads_the_whole_latch_into_the_counter() {
        // c64-wiki: "if the timer is stopped, the high-byte will automatically
        // be re-set as well" — we interpret this as the full 16-bit latch
        // being copied, which is what makes "write low, write high, read
        // back" return the value you wrote.
        let mut t = timer_with(0x1000);
        assert!(!t.is_running());
        t.write_low(0x34);
        t.write_high(0x12); // stopped -> force load
        assert_eq!(t.latch(), 0x1234);
        assert_eq!(
            read_counter(&t),
            0x1234,
            "TA HI write while stopped must load the counter"
        );
    }

    #[test]
    fn rewriting_the_high_byte_picks_up_a_new_latch_low() {
        let mut t = timer_with(0x00FF);
        t.write_low(0x77); // latch $0077, counter still $00FF
        assert_eq!(read_counter(&t), 0x00FF);
        t.write_high(0x00); // same high byte, but stopped -> loads again
        assert_eq!(t.latch(), 0x0077);
        assert_eq!(
            read_counter(&t),
            0x0077,
            "rewriting TA HI must reload the full latch, including a newly written low byte"
        );
    }

    #[test]
    fn write_high_while_running_touches_only_the_latch() {
        let mut t = timer_with(0x0010);
        t.start();
        t.tick(); // $000F
        t.write_high(0x02);
        assert_eq!(t.latch(), 0x0210);
        assert_eq!(
            read_counter(&t),
            0x000F,
            "a TA HI write while running must not load"
        );
        t.tick();
        assert_eq!(
            read_counter(&t),
            0x000E,
            "the counter keeps counting from where it was"
        );
    }

    #[test]
    fn writing_timer_registers_never_starts_or_stops_the_timer() {
        let mut t = timer_with(0x00FF);
        t.write_low(0x00);
        t.write_high(0x10);
        assert!(!t.is_running());
        t.start();
        t.write_low(0x01);
        t.write_high(0x10);
        assert!(t.is_running());
        t.stop();
        t.write_high(0x10);
        assert!(!t.is_running());
    }

    #[test]
    fn latch_changes_while_running_take_effect_at_the_next_underflow() {
        let mut t = timer_with(5);
        t.start();
        for _ in 0..4 {
            t.tick();
        }
        assert_eq!(read_counter(&t), 1);

        t.write_low(8);
        t.write_high(0); // running: only the latch changes
        assert_eq!(read_counter(&t), 1);

        t.tick();
        assert_eq!(read_counter(&t), 0);
        t.tick(); // underflow -> reload with the NEW latch value
        assert!(t.underflow());
        assert_eq!(read_counter(&t), 8);

        for _ in 0..8 {
            t.tick(); // 7..0
        }
        assert_eq!(read_counter(&t), 0);
        t.tick(); // second underflow
        assert_eq!(read_counter(&t), 8);
    }

    // ---------------------------------------------------------------------
    // Start / stop (CRA/CRB bit 0)
    // ---------------------------------------------------------------------

    #[test]
    fn start_begins_counting_down_from_the_current_counter() {
        let mut t = timer_with(3);
        t.start();
        assert_eq!(read_counter(&t), 3, "starting must not alter the counter");
        t.tick();
        assert_eq!(read_counter(&t), 2);
        t.tick();
        assert_eq!(read_counter(&t), 1);
    }

    #[test]
    fn stop_freezes_the_counter() {
        let mut t = timer_with(0x000A);
        t.start();
        for _ in 0..3 {
            t.tick(); // 7
        }
        t.stop();
        assert!(!t.is_running());
        for _ in 0..10 {
            t.tick();
        }
        assert_eq!(
            read_counter(&t),
            0x0007,
            "a stopped timer must hold its count"
        );
        assert!(!t.underflow(), "a stopped timer cannot underflow");
    }

    #[test]
    fn tick_while_stopped_is_a_noop_even_at_zero() {
        let mut t = timer_with(1);
        t.set_mode(TimerMode::OneShot);
        t.start();
        t.tick(); // counter 0 — NOT the underflow yet
        assert_eq!(read_counter(&t), 0);
        assert!(!t.underflow());
        t.stop(); // freeze exactly at zero
        for _ in 0..5 {
            t.tick();
        }
        assert_eq!(read_counter(&t), 0);
        assert!(!t.underflow());
        assert!(!t.is_running());
        t.start(); // resumed: the very next clock underflows
        t.tick();
        assert!(t.underflow());
        assert_eq!(read_counter(&t), 0xFFFF);
    }

    #[test]
    fn start_does_not_reload_from_the_latch() {
        // The single most-mis-modelled 6526 quirk. Only a TA HI write while
        // stopped, the bit-4 load strobe, or an underflow reload the counter.
        // (Evidence: CR bit 4 would be pointless if start reloaded; KERNAL
        // code that needs a fresh value writes $11 = start+load.)
        let mut t = timer_with(0x0010);
        t.start();
        for _ in 0..4 {
            t.tick(); // $0C
        }
        t.stop();
        t.start();
        assert_eq!(read_counter(&t), 0x0C, "start() must not reload");
        t.tick();
        assert_eq!(read_counter(&t), 0x0B);
    }

    #[test]
    fn stop_and_restart_resume_from_the_frozen_count() {
        let mut t = timer_with(100);
        t.start();
        for _ in 0..10 {
            t.tick(); // 90
        }
        t.stop();
        for _ in 0..7 {
            t.tick(); // still 90
        }
        assert_eq!(read_counter(&t), 90);
        t.start();
        for _ in 0..89 {
            t.tick(); // 1
        }
        assert_eq!(read_counter(&t), 1);
        t.tick(); // 0
        assert!(!t.underflow());
        t.tick(); // underflow -> continuous reload
        assert!(t.underflow());
        assert!(t.is_running());
        assert_eq!(read_counter(&t), 100);
    }

    #[test]
    fn starting_an_already_running_timer_is_harmless() {
        let mut t = timer_with(8);
        t.start();
        t.tick(); // 7
        t.start(); // CR write with the start bit already set
        assert_eq!(read_counter(&t), 7);
        t.tick();
        assert_eq!(read_counter(&t), 6);
        assert!(t.is_running());
    }

    #[test]
    fn restarting_an_expired_one_shot_counts_down_from_ffff() {
        // Hence a restarted expired one-shot waits a full 65536 clocks.
        let mut t = expired_one_shot(2);
        assert!(!t.is_running());
        assert_eq!(read_counter(&t), 0xFFFF);
        t.acknowledge_underflow();
        t.start();
        assert!(t.is_running());
        assert_eq!(read_counter(&t), 0xFFFF, "restart must not reload");
        t.tick();
        assert_eq!(read_counter(&t), 0xFFFE);
        t.tick();
        assert_eq!(read_counter(&t), 0xFFFD);
    }

    #[test]
    fn the_underflow_is_the_clock_that_decrements_from_zero() {
        let mut t = timer_with(2);
        t.start();
        t.tick();
        assert_eq!(read_counter(&t), 1);
        assert!(!t.underflow());
        t.tick();
        assert_eq!(read_counter(&t), 0);
        assert!(!t.underflow(), "reaching $0000 is not yet the underflow");
        t.tick();
        assert!(
            t.underflow(),
            "the underflow is the clock that decrements FROM $0000"
        );
        assert_eq!(read_counter(&t), 2); // continuous reload
    }

    // ---------------------------------------------------------------------
    // Force load (CRA/CRB bit 4)
    // ---------------------------------------------------------------------

    #[test]
    fn reload_loads_the_counter_when_stopped_without_starting_it() {
        let mut t = timer_with(0x00FF);
        t.write_low(0x38); // latch $0038, counter $00FF
        t.reload();
        assert_eq!(read_counter(&t), 0x0038);
        assert!(!t.is_running(), "the load strobe must not start the timer");
        t.tick();
        assert_eq!(read_counter(&t), 0x0038, "stopped timers do not count");
    }

    #[test]
    fn reload_loads_the_counter_while_running() {
        let mut t = timer_with(10);
        t.start();
        for _ in 0..3 {
            t.tick(); // 7
        }
        t.reload();
        assert_eq!(read_counter(&t), 10);
        assert!(t.is_running());
        t.tick();
        assert_eq!(read_counter(&t), 9);
    }

    #[test]
    fn reload_neither_fires_nor_clears_the_underflow_event() {
        let mut t = timer_with(5);
        t.reload();
        assert!(!t.underflow(), "a force-load is not an underflow");

        let mut t = expired_one_shot(3);
        assert!(t.underflow());
        t.reload();
        assert!(t.underflow(), "reload must not clear a pending event");
    }

    // ---------------------------------------------------------------------
    // Clocking: one-shot behaviour
    // ---------------------------------------------------------------------

    #[test]
    fn tick_decrements_exactly_one_per_clock() {
        let mut t = timer_with(10);
        t.start();
        let mut expected = 10u16;
        for _ in 0..10 {
            t.tick();
            expected -= 1;
            assert_eq!(read_counter(&t), expected);
        }
        assert_eq!(read_counter(&t), 0);
    }

    #[test]
    fn one_shot_full_trace() {
        let mut t = timer_with(3);
        t.set_mode(TimerMode::OneShot);
        t.start();
        for &expected in &[2u16, 1, 0, 0xFFFF] {
            t.tick();
            assert_eq!(read_counter(&t), expected);
        }
        assert!(!t.is_running());
        assert!(t.underflow());
    }

    #[test]
    fn one_shot_expires_with_the_counter_at_ffff() {
        let t = expired_one_shot(3);
        assert!(!t.is_running());
        assert_eq!(
            read_counter(&t),
            0xFFFF,
            "an expired one-shot leaves $FFFF in the counter"
        );
        assert_eq!(t.latch(), 3, "expiry must not clobber the latch");
    }

    #[test]
    fn one_shot_fires_exactly_once() {
        let mut t = expired_one_shot(3);
        t.acknowledge_underflow();
        for _ in 0..100 {
            t.tick();
        }
        assert!(
            !t.underflow(),
            "an expired one-shot must not underflow again"
        );
        assert!(!t.is_running());
        assert_eq!(read_counter(&t), 0xFFFF);
    }

    #[test]
    fn one_shot_with_latch_zero_expires_on_the_first_clock() {
        let mut t = timer_with(0);
        t.set_mode(TimerMode::OneShot);
        t.start();
        assert_eq!(read_counter(&t), 0);
        assert!(!t.underflow());
        t.tick();
        assert!(t.underflow());
        assert!(!t.is_running());
        assert_eq!(read_counter(&t), 0xFFFF);
    }

    #[test]
    fn one_shot_with_latch_ffff_takes_65536_clocks() {
        // The maximum one-shot interval (used for hour-long Timer A+B cascades).
        let mut t = timer_with(0xFFFF);
        t.set_mode(TimerMode::OneShot);
        t.start();
        for _ in 0..0xFFFF {
            t.tick(); // 65535 clocks
        }
        assert_eq!(read_counter(&t), 0x0000);
        assert!(!t.underflow());
        t.tick(); // clock 65536
        assert!(t.underflow());
        assert!(!t.is_running());
    }

    // ---------------------------------------------------------------------
    // Clocking: continuous behaviour
    // ---------------------------------------------------------------------

    #[test]
    fn continuous_full_trace_with_underflows() {
        let mut t = timer_with(3);
        t.start();
        let trace: [u16; 12] = [2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3];
        for (i, &expected) in trace.iter().enumerate() {
            t.tick();
            assert_eq!(read_counter(&t), expected, "counter after clock {}", i + 1);
            if (i + 1) % 4 == 0 {
                assert!(t.underflow(), "underflow after clock {}", i + 1);
                t.acknowledge_underflow();
            } else {
                assert!(!t.underflow(), "no underflow after clock {}", i + 1);
            }
        }
        assert!(t.is_running(), "a continuous timer never stops itself");
    }

    #[test]
    fn continuous_reload_happens_on_the_underflow_clock_itself() {
        let mut t = timer_with(0x1234);
        t.start();
        for _ in 0..0x1234 {
            t.tick();
        }
        assert_eq!(read_counter(&t), 0);
        t.tick(); // the underflow clock itself
        assert!(t.underflow());
        assert_eq!(
            read_counter(&t),
            0x1234,
            "continuous mode reloads from the latch on the underflow clock"
        );
    }

    #[test]
    fn continuous_with_latch_zero_underflows_on_every_clock() {
        // Consistent with the datasheet's "intervals of 1 to 65536 clocks":
        // latch N => period N+1, so latch 0 => period 1.
        let mut t = timer_with(0);
        t.start();
        for i in 0..3 {
            t.tick();
            assert!(t.underflow(), "underflow on clock {}", i + 1);
            assert!(t.is_running());
            assert_eq!(read_counter(&t), 0);
            t.acknowledge_underflow();
            assert!(!t.underflow());
        }
    }

    #[test]
    fn rearming_an_expired_one_shot_by_rewriting_the_high_byte() {
        let mut t = expired_one_shot(5);
        assert_eq!(read_counter(&t), 0xFFFF);
        t.write_low(9);
        t.write_high(0); // stopped -> force load
        assert_eq!(read_counter(&t), 9);
        assert!(
            t.underflow(),
            "the old event stays pending until acknowledged"
        );
        t.start();
        t.tick();
        assert_eq!(read_counter(&t), 8);
    }

    // ---------------------------------------------------------------------
    // Run mode (CRA/CRB bit 3)
    // ---------------------------------------------------------------------

    #[test]
    fn set_mode_is_bookkeeping_only() {
        let mut t = timer_with(9);
        t.set_mode(TimerMode::OneShot);
        assert_eq!(t.mode(), TimerMode::OneShot);
        assert!(!t.is_running());
        t.set_mode(TimerMode::Continuous);
        assert_eq!(t.mode(), TimerMode::Continuous);
        assert!(!t.is_running());
        t.tick();
        assert_eq!(read_counter(&t), 9, "mode changes never start the timer");

        t.start();
        t.set_mode(TimerMode::OneShot);
        assert!(t.is_running(), "mode changes never stop the timer");
        t.tick();
        assert_eq!(read_counter(&t), 8);
    }

    #[test]
    fn switching_to_one_shot_mid_count_stops_at_the_underflow() {
        let mut t = timer_with(4);
        t.start();
        t.tick();
        t.tick(); // 2
        t.set_mode(TimerMode::OneShot);
        t.tick();
        t.tick(); // 0
        t.tick(); // underflow — now one-shot, so it stops
        assert!(!t.is_running());
        assert!(t.underflow());
        assert_eq!(read_counter(&t), 0xFFFF);
    }

    #[test]
    fn switching_to_continuous_mid_count_avoids_stopping() {
        let mut t = timer_with(3);
        t.set_mode(TimerMode::OneShot);
        t.start();
        t.tick();
        t.tick(); // 1
        t.set_mode(TimerMode::Continuous);
        t.tick(); // 0
        t.tick(); // underflow -> reload, keeps running
        assert!(t.is_running());
        assert!(t.underflow());
        assert_eq!(read_counter(&t), 3);
    }

    #[test]
    fn expired_one_shot_revived_as_continuous_counts_from_ffff() {
        let mut t = expired_one_shot(5);
        t.set_mode(TimerMode::Continuous);
        assert!(!t.is_running(), "a mode change must not start the timer");
        t.start();
        t.tick();
        assert_eq!(
            read_counter(&t),
            0xFFFE,
            "restarts from the expired $FFFF value"
        );
        t.tick();
        assert_eq!(read_counter(&t), 0xFFFD);
    }

    // ---------------------------------------------------------------------
    // The underflow event (future ICR wiring)
    // ---------------------------------------------------------------------

    #[test]
    fn underflow_event_is_sticky_until_acknowledged() {
        let mut t = timer_with(2);
        t.start();
        t.tick();
        t.tick();
        t.tick(); // 1, 0, underflow (-> 2)
        assert!(t.underflow());
        t.tick();
        t.tick(); // 1, 0
        assert!(t.underflow(), "the event stays pending across clocks");
        t.acknowledge_underflow();
        assert!(!t.underflow());
        t.tick(); // underflow again
        assert!(t.underflow());
    }

    #[test]
    fn acknowledging_without_a_pending_event_is_harmless() {
        let mut t = Timer::new();
        t.acknowledge_underflow();
        assert!(!t.underflow());
        let mut t = timer_with(5);
        t.acknowledge_underflow();
        t.start();
        t.tick();
        assert!(!t.underflow());
    }

    #[test]
    fn nothing_but_acknowledge_clears_the_event() {
        let mut t = expired_one_shot(3);
        assert!(t.underflow());
        t.stop();
        assert!(t.underflow());
        t.start();
        assert!(t.underflow());
        t.reload();
        assert!(t.underflow());
        t.set_mode(TimerMode::Continuous);
        assert!(t.underflow());
        t.write_low(0);
        assert!(t.underflow());
        t.write_high(0);
        assert!(t.underflow());
        let _ = (t.read_low(), t.read_high());
        assert!(t.underflow());
        t.acknowledge_underflow();
        assert!(!t.underflow());
    }

    #[test]
    fn several_underflows_collapse_into_one_pending_event() {
        // The real ICR does not count events — it just says "one happened".
        let mut t = timer_with(1);
        t.start();
        for _ in 0..3 {
            t.tick(); // 1 -> 0
            t.tick(); // 0 -> underflow -> 1
        }
        assert!(t.underflow());
        t.acknowledge_underflow();
        assert!(!t.underflow(), "past underflows are not remembered");
    }

    #[test]
    fn reading_the_timer_does_not_clear_the_event() {
        // Only an ICR read clears flags on real hardware ($DC04/$DC05 do not).
        let mut t = expired_one_shot(4);
        let _ = t.read_low();
        let _ = t.read_high();
        assert!(t.underflow());
    }

    // ---------------------------------------------------------------------
    // End-to-end idiom
    // ---------------------------------------------------------------------

    #[test]
    fn kernal_style_setup_gives_an_exact_interval() {
        // Mirrors: LDA #lo / STA $DC04 ; LDA #hi / STA $DC05 ; start bit.
        let mut t = Timer::new();
        t.write_low(0x2A);
        t.write_high(0x01); // $012A = 298
        t.start();
        assert_eq!(read_counter(&t), 298);
        for _ in 0..298 {
            t.tick();
        }
        assert_eq!(read_counter(&t), 0);
        assert!(!t.underflow());
        t.tick(); // clock 299: the underflow
        assert!(t.underflow());
        assert!(t.is_running(), "the default mode is continuous");
        assert_eq!(read_counter(&t), 298);
    }
}
