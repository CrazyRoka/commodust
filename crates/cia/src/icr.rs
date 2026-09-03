pub const TIMER_A_UNDERFLOW: u8 = 1 << 0;
pub const TIMER_B_UNDERFLOW: u8 = 1 << 1;
pub const TOD_ALARM: u8 = 1 << 2;
pub const SERIAL_SHIFT_REGISTER: u8 = 1 << 3;
pub const FLAG_LINE: u8 = 1 << 4;
const IR_FLAG: u8 = 1 << 7;
const SET_CLEAR_FLAG: u8 = 1 << 7;
const SOURCE_MASK: u8 = 0x1F;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InterruptControlRegister {
    interrupts: u8,
    mask: u8,
}

impl InterruptControlRegister {
    pub fn new() -> Self {
        Self {
            interrupts: 0,
            mask: 0,
        }
    }

    /// Reads ICR ($0D). Returns pending flags + IR bit (bit 7),
    /// and resets all pending interrupt flags (clear-on-read).
    pub fn read(&mut self) -> u8 {
        let value = self.interrupts
            | if self.is_interrupt_active() {
                IR_FLAG
            } else {
                0
            };
        self.interrupts = 0;
        value
    }

    /// Writes ICR ($0D). Updates the interrupt mask according to bit 7.
    pub fn write(&mut self, value: u8) {
        if value & SET_CLEAR_FLAG != 0 {
            self.mask |= value & SOURCE_MASK;
        } else {
            self.mask &= !(value & SOURCE_MASK);
        }
    }

    /// Latches an interrupt event into the internal register.
    pub fn set_interrupt(&mut self, source: u8) {
        self.interrupts |= source & SOURCE_MASK;
    }

    /// Returns true if any latched interrupt is currently enabled by the mask.
    /// This directly drives the physical IRQ/NMI pin.
    pub fn is_interrupt_active(&self) -> bool {
        self.interrupts & self.mask != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. RESET STATE
    //    * All interrupt flags (bits 0-4) cleared.
    //    * Mask bits (bits 0-4) cleared (all interrupts disabled).
    //    * Interrupt output line is false.
    //    * Reads return $00.
    //
    // 2. WRITE BEHAVIOUR (MASK REGISTER)
    //    * Bit 7 = 1: Any 1 in bits 0-4 SETS that mask bit (enables it).
    //    * Bit 7 = 0: Any 1 in bits 0-4 CLEARS that mask bit (disables it).
    //    * Bits written as 0 never alter their respective mask bits.
    //    * Bits 5-6 are unimplemented and ignored.
    //    * Bit 7 is control-only; it is never stored in the mask itself.
    //
    // 3. SET_INTERRUPT (EVENT LATCHING)
    //    * Events set their corresponding bits (0-4) in the pending register.
    //    * Events latch regardless of whether their mask bit is set or clear.
    //    * Events are sticky until cleared by read().
    //    * Bits outside 0-4 are ignored.
    //
    // 4. READ BEHAVIOUR (CLEAR-ON-READ)
    //    * Returns latched bits 0-4.
    //    * Bits 5-6 always return 0.
    //    * Bit 7 (IR) is 1 if and only if (interrupts & mask) != 0.
    //    * Reading clears ALL pending interrupt flags (bits 0-4 := 0).
    //    * Reading de-asserts the interrupt line.
    //    * Reading does NOT alter the mask.
    //
    // 5. INTERRUPT LINE / CORNER INTERACTIONS
    //    * is_interrupt_active() is true whenever (interrupts & mask) != 0.
    //    * Enabling a mask bit for an ALREADY pending interrupt asserts IRQ immediately.
    //    * Disabling a mask bit for the only pending interrupt de-asserts IRQ immediately.
    // =====================================================================

    // ---------------------------------------------------------------------
    // 1. Reset State
    // ---------------------------------------------------------------------

    #[test]
    fn new_icr_is_completely_cleared() {
        let mut icr = InterruptControlRegister::new();
        assert!(!icr.is_interrupt_active());
        assert_eq!(icr.read(), 0x00);
    }

    // ---------------------------------------------------------------------
    // 2. Write Mask Behavior
    // ---------------------------------------------------------------------

    #[test]
    fn write_with_bit7_set_enables_mask_bits() {
        let mut icr = InterruptControlRegister::new();

        // Enable Timer A
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());

        // Read clears events, mask should still be enabled
        icr.read();
        assert!(!icr.is_interrupt_active());

        // Trigger again to verify mask was preserved across read
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());
    }

    #[test]
    fn write_with_bit7_set_accumulates_without_clearing_others() {
        let mut icr = InterruptControlRegister::new();

        // Enable Timer A
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);
        // Enable TOD Alarm (Timer A must remain enabled)
        icr.write(SET_CLEAR_FLAG | TOD_ALARM);

        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());
        icr.read();

        icr.set_interrupt(TOD_ALARM);
        assert!(icr.is_interrupt_active());
    }

    #[test]
    fn write_with_bit7_clear_disables_mask_bits() {
        let mut icr = InterruptControlRegister::new();

        // Enable Timer A and Timer B
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW | TIMER_B_UNDERFLOW);

        // Clear Timer A (bit 7 is 0)
        icr.write(TIMER_A_UNDERFLOW);

        // Timer A underflow should no longer trigger active interrupt
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(!icr.is_interrupt_active());

        // But Timer B should still trigger
        icr.set_interrupt(TIMER_B_UNDERFLOW);
        assert!(icr.is_interrupt_active());
    }

    #[test]
    fn write_zeros_do_not_modify_unspecified_mask_bits() {
        let mut icr = InterruptControlRegister::new();

        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);

        // Writing bit 7=1 with no other bits should not enable anything else
        icr.write(SET_CLEAR_FLAG);
        icr.set_interrupt(TIMER_B_UNDERFLOW);
        assert!(!icr.is_interrupt_active());

        // Writing 0x00 with bit 7=0 should not clear Timer A
        icr.write(0x00);
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());
    }

    #[test]
    fn write_ignores_bits_5_and_6() {
        let mut icr = InterruptControlRegister::new();

        // Write with bits 5 and 6 set
        icr.write(SET_CLEAR_FLAG | 0x60 | TIMER_A_UNDERFLOW);

        // Verify only valid sources operate
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());
    }

    // ---------------------------------------------------------------------
    // 3. Interrupt Event Latching (set_interrupt)
    // ---------------------------------------------------------------------

    #[test]
    fn disabled_interrupt_latches_flag_without_asserting_line() {
        let mut icr = InterruptControlRegister::new();

        // Timer A is NOT masked (disabled)
        icr.set_interrupt(TIMER_A_UNDERFLOW);

        // Interrupt line must NOT be active
        assert!(!icr.is_interrupt_active());

        // But on read, the latched flag must appear with Bit 7 (IR) = 0
        let val = icr.read();
        assert_eq!(val, TIMER_A_UNDERFLOW);
        assert_eq!(val & IR_FLAG, 0);
    }

    #[test]
    fn enabled_interrupt_latches_flag_and_asserts_line() {
        let mut icr = InterruptControlRegister::new();

        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_A_UNDERFLOW);

        assert!(icr.is_interrupt_active());

        // On read, the source flag AND bit 7 (IR) must both be set
        let val = icr.read();
        assert_eq!(val, IR_FLAG | TIMER_A_UNDERFLOW);
    }

    #[test]
    fn set_interrupt_ignores_bits_outside_sources() {
        let mut icr = InterruptControlRegister::new();

        // Attempt to trigger unused bits 5, 6, 7
        icr.set_interrupt(0xE0);

        assert_eq!(icr.read(), 0x00);
        assert!(!icr.is_interrupt_active());
    }

    #[test]
    fn multiple_interrupts_accumulate() {
        let mut icr = InterruptControlRegister::new();

        icr.set_interrupt(TIMER_A_UNDERFLOW);
        icr.set_interrupt(TOD_ALARM);
        icr.set_interrupt(FLAG_LINE);

        assert_eq!(icr.read(), TIMER_A_UNDERFLOW | TOD_ALARM | FLAG_LINE);
    }

    // ---------------------------------------------------------------------
    // 4. Read Side Effects (Clear-on-Read)
    // ---------------------------------------------------------------------

    #[test]
    fn read_clears_all_pending_interrupts() {
        let mut icr = InterruptControlRegister::new();
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_A_UNDERFLOW);

        let first_read = icr.read();
        assert_eq!(first_read, IR_FLAG | TIMER_A_UNDERFLOW);

        // Second read must immediately return zero
        let second_read = icr.read();
        assert_eq!(second_read, 0x00);
        assert!(!icr.is_interrupt_active());
    }

    #[test]
    fn read_clears_disabled_interrupts_too() {
        let mut icr = InterruptControlRegister::new();
        // Triggered while disabled
        icr.set_interrupt(TIMER_B_UNDERFLOW);

        assert_eq!(icr.read(), TIMER_B_UNDERFLOW);
        assert_eq!(icr.read(), 0x00);
    }

    #[test]
    fn read_never_returns_bits_5_and_6() {
        let mut icr = InterruptControlRegister::new();
        icr.set_interrupt(0xFF); // Try to set all bits

        let val = icr.read();
        assert_eq!(val & (1 << 5), 0);
        assert_eq!(val & (1 << 6), 0);
    }

    // ---------------------------------------------------------------------
    // 5. Mask & Flag Dynamic Interactions
    // ---------------------------------------------------------------------

    #[test]
    fn enabling_mask_for_already_pending_event_asserts_interrupt_immediately() {
        let mut icr = InterruptControlRegister::new();

        // Event occurs while disabled
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(!icr.is_interrupt_active());

        // Software enables the mask later
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);

        // Hardware line asserts immediately
        assert!(icr.is_interrupt_active());

        // Read returns IR bit + source bit
        assert_eq!(icr.read(), IR_FLAG | TIMER_A_UNDERFLOW);
    }

    #[test]
    fn disabling_mask_for_pending_event_deasserts_interrupt_immediately() {
        let mut icr = InterruptControlRegister::new();

        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        assert!(icr.is_interrupt_active());

        // Software disables the mask before reading
        icr.write(TIMER_A_UNDERFLOW); // Bit 7 is 0 -> clear

        // Hardware line de-asserts immediately
        assert!(!icr.is_interrupt_active());

        // Event is still pending, but IR bit (bit 7) is 0
        assert_eq!(icr.read(), TIMER_A_UNDERFLOW);
    }

    #[test]
    fn mixed_enabled_and_disabled_events_readout() {
        let mut icr = InterruptControlRegister::new();

        // Enable only Timer A
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW);

        // Fire Timer A (enabled) and Timer B (disabled)
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_B_UNDERFLOW);

        assert!(icr.is_interrupt_active());

        // Both source bits are present, and Bit 7 is set because Timer A is enabled
        let val = icr.read();
        assert_eq!(val, IR_FLAG | TIMER_A_UNDERFLOW | TIMER_B_UNDERFLOW);

        // Reading clears both
        assert_eq!(icr.read(), 0x00);
    }

    #[test]
    fn all_five_sources_can_fire_and_mask_independently() {
        let sources = [
            TIMER_A_UNDERFLOW,
            TIMER_B_UNDERFLOW,
            TOD_ALARM,
            SERIAL_SHIFT_REGISTER,
            FLAG_LINE,
        ];

        for &src in &sources {
            let mut icr = InterruptControlRegister::new();

            // 1. Fire while masked out
            icr.set_interrupt(src);
            assert!(!icr.is_interrupt_active());
            assert_eq!(icr.read(), src);

            // 2. Enable mask, then fire
            icr.write(SET_CLEAR_FLAG | src);
            icr.set_interrupt(src);
            assert!(icr.is_interrupt_active());
            assert_eq!(icr.read(), IR_FLAG | src);

            // 3. Disable mask again
            icr.write(src);
            icr.set_interrupt(src);
            assert!(!icr.is_interrupt_active());
            assert_eq!(icr.read(), src);
        }
    }

    #[test]
    fn disabling_one_of_two_live_events_keeps_the_line_asserted() {
        let mut icr = InterruptControlRegister::new();
        icr.write(SET_CLEAR_FLAG | TIMER_A_UNDERFLOW | TIMER_B_UNDERFLOW);
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        icr.set_interrupt(TIMER_B_UNDERFLOW);
        assert!(icr.is_interrupt_active());
        icr.write(TIMER_A_UNDERFLOW); // clear only Timer A's mask
        assert!(icr.is_interrupt_active()); // Timer B still holds the line
        assert_eq!(icr.read(), IR_FLAG | TIMER_A_UNDERFLOW | TIMER_B_UNDERFLOW);
    }

    #[test]
    fn all_five_sources_pending_and_masked_read_as_9f() {
        let mut icr = InterruptControlRegister::new();
        icr.write(SET_CLEAR_FLAG | SOURCE_MASK);
        for src in [
            TIMER_A_UNDERFLOW,
            TIMER_B_UNDERFLOW,
            TOD_ALARM,
            SERIAL_SHIFT_REGISTER,
            FLAG_LINE,
        ] {
            icr.set_interrupt(src);
        }
        assert!(icr.is_interrupt_active());
        assert_eq!(icr.read(), 0x9F);
        assert_eq!(icr.read(), 0x00);
    }

    #[test]
    fn cpu_writes_never_set_or_clear_pending_flags() {
        let mut icr = InterruptControlRegister::new();
        icr.write(SET_CLEAR_FLAG | SOURCE_MASK);
        assert_eq!(icr.read(), 0x00); // mask writes create no events
        icr.set_interrupt(TIMER_A_UNDERFLOW);
        icr.write(TIMER_A_UNDERFLOW); // clear-write must not clear the flag
        icr.write(SET_CLEAR_FLAG | TOD_ALARM); // set-write must not set a flag
        assert_eq!(icr.read(), TIMER_A_UNDERFLOW); // IR=0: mask got emptied above
    }
}
