#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port {
    /// Peripheral data (output) register — what the CPU last wrote.
    latch: u8,
    /// Data direction: 1 = output stage connected.
    ddr: u8,
    /// Pin levels (1 = high, 0 = pulled low externally).
    pins: u8,
    /// Output bits owned by an internal peripheral (Timer A -> PB6, Timer B -> PB7).
    override_mask: u8,
    /// Level the peripheral drives there.
    override_value: u8,
}

impl Port {
    pub fn new() -> Self {
        Self {
            latch: 0,
            ddr: 0,
            pins: 0xFF,
            override_mask: 0,
            override_value: 0,
        }
    }

    pub fn write_latch(&mut self, value: u8) {
        self.latch = value;
    }

    pub fn read_latch(&self) -> u8 {
        self.latch
    }

    pub fn write_ddr(&mut self, value: u8) {
        self.ddr = value;
    }

    pub fn read_ddr(&self) -> u8 {
        self.ddr
    }

    /// External hardware (keyboard, joystick, user port) drives pin levels in.
    pub fn set_pins(&mut self, value: u8) {
        self.pins = value;
    }

    /// What the CPU reads when accessing PRA ($00) or PRB ($01).
    pub fn read_pins(&self) -> u8 {
        let bus_pins = (self.latch | !self.ddr) & self.pins;
        (!self.override_mask & bus_pins) | (self.override_mask & self.override_value)
    }

    /// What the CIA is actively driving out to the motherboard (keyboard columns, VIC bank).
    pub fn driven_pins(&self) -> u8 {
        let driven = self.latch & self.ddr | !self.ddr;
        (!self.override_mask & driven) | (self.override_mask & self.override_value)
    }

    /// Engages a hardware peripheral override (e.g. Timer A on PB6, Timer B on PB7).
    pub fn set_output_override(&mut self, mask: u8, value: u8) {
        self.override_mask = mask;
        self.override_value = value & mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. RESET STATE
    //    * latch = $00, ddr = $00 (all bits are inputs).
    //    * Unconnected pins default to $FF (internal pull-ups).
    //    * No peripheral overrides active (mask = 0, value = 0).
    //    * read_ddr() returns $00, read_latch() returns $00.
    //    * read_pins() and driven_pins() return $FF.
    //
    // 2. DDR READ / WRITE ($02 / $03)
    //    * write_ddr stores the exact byte.
    //    * read_ddr returns the exact byte written.
    //    * Writing DDR never alters the data latch.
    //
    // 3. LATCH READ / WRITE ($00 / $01 WRITES)
    //    * write_latch stores the exact byte regardless of DDR.
    //    * read_latch returns what was written.
    //    * Writing latch never alters the DDR.
    //
    // 4. INPUT MODE (DDR bit = 0)
    //    * Pins set via set_pins() are directly readable via read_pins().
    //    * The latch value for that bit is completely ignored.
    //    * driven_pins() floats high ($FF) for input bits.
    //
    // 5. OUTPUT MODE (DDR bit = 1)
    //    * The latch value drives the pin: driven_pins() reflects the latch.
    //    * read_pins() reads back the driven value.
    //
    // 6. WIRED-AND / PULL-DOWN BEHAVIOR
    //    * If an external device pulls a pin low (set_pins bit = 0),
    //      read_pins() returns 0 even if the latch had 1 or ddr was 0.
    //
    // 7. PERIPHERAL OVERRIDES (PB6 / PB7 TIMERS)
    //    * set_output_override() forces the pin state regardless of DDR.
    //    * Overridden bits reflect override_value in both read_pins() and driven_pins().
    //    * Non-overridden bits continue normal DDR/latch/pin operation.
    //    * Writes to the latch while overridden update the latch silently.
    //    * Releasing the override immediately restores normal latch/DDR behavior.
    // =====================================================================

    // ---------------------------------------------------------------------
    // 1. Reset State
    // ---------------------------------------------------------------------

    #[test]
    fn new_port_is_all_inputs_with_pullups_floating_high() {
        let port = Port::new();

        assert_eq!(port.read_ddr(), 0x00, "DDR must default to all inputs");
        assert_eq!(port.read_latch(), 0x00, "Latch must default to 0x00");
        assert_eq!(
            port.read_pins(),
            0xFF,
            "Disconnected pins read 1 via pull-ups"
        );
        assert_eq!(
            port.driven_pins(),
            0xFF,
            "All-input port drives lines high (pull-ups)"
        );
    }

    // ---------------------------------------------------------------------
    // 2. DDR Access ($02 / $03)
    // ---------------------------------------------------------------------

    #[test]
    fn ddr_round_trips_all_patterns() {
        let mut port = Port::new();

        for ddr in [0x00, 0x01, 0x55, 0xAA, 0x3F, 0xFF] {
            port.write_ddr(ddr);
            assert_eq!(port.read_ddr(), ddr);
            assert_eq!(port.read_latch(), 0x00, "DDR write must not alter latch");
        }
    }

    // ---------------------------------------------------------------------
    // 3. Latch Access ($00 / $01 writes)
    // ---------------------------------------------------------------------

    #[test]
    fn latch_stores_value_regardless_of_ddr() {
        let mut port = Port::new();

        // Write latch while all inputs
        port.write_ddr(0x00);
        port.write_latch(0xA5);
        assert_eq!(port.read_latch(), 0xA5);

        // Change DDR, latch must remain intact
        port.write_ddr(0xFF);
        assert_eq!(port.read_latch(), 0xA5);
    }

    // ---------------------------------------------------------------------
    // 4. Pure Input Mode (DDR = $00)
    // ---------------------------------------------------------------------

    #[test]
    fn input_pins_track_external_signals() {
        let mut port = Port::new();
        port.write_ddr(0x00); // All inputs

        port.set_pins(0x5A);
        assert_eq!(port.read_pins(), 0x5A);

        port.set_pins(0x00);
        assert_eq!(port.read_pins(), 0x00);

        port.set_pins(0xFF);
        assert_eq!(port.read_pins(), 0xFF);
    }

    #[test]
    fn input_pins_ignore_latch_changes() {
        let mut port = Port::new();
        port.write_ddr(0x00); // All inputs
        port.set_pins(0xF0);

        port.write_latch(0x0F);
        assert_eq!(
            port.read_pins(),
            0xF0,
            "Input reads must reflect external pins, not latch"
        );
    }

    // ---------------------------------------------------------------------
    // 5. Pure Output Mode (DDR = $FF)
    // ---------------------------------------------------------------------

    #[test]
    fn output_pins_drive_latch_to_pins() {
        let mut port = Port::new();
        port.write_ddr(0xFF); // All outputs

        port.write_latch(0x3C);
        assert_eq!(port.driven_pins(), 0x3C);
        assert_eq!(port.read_pins(), 0x3C);

        port.write_latch(0xC3);
        assert_eq!(port.driven_pins(), 0xC3);
        assert_eq!(port.read_pins(), 0xC3);
    }

    #[test]
    fn output_pins_can_be_pulled_low_externally_wired_and() {
        let mut port = Port::new();
        port.write_ddr(0xFF); // All outputs (like CIA1 Port A)
        port.write_latch(0xFF); // Latch set high

        // External device (e.g. Joystick 2 fire button on bit 4) grounds pin
        port.set_pins(!0x10);

        assert_eq!(
            port.read_pins(),
            !0x10,
            "Reading output pins must reflect external ground pulls"
        );
    }

    #[test]
    fn changing_ddr_from_input_to_output_reveals_preloaded_latch() {
        let mut port = Port::new();

        // 1. While configured as inputs, write to latch
        port.write_ddr(0x00);
        port.write_latch(0x42);
        port.set_pins(0xFF);
        assert_eq!(port.read_pins(), 0xFF);

        // 2. Switch to output: preloaded latch immediately appears
        port.write_ddr(0xFF);
        assert_eq!(port.driven_pins(), 0x42);
        assert_eq!(port.read_pins(), 0x42);
    }

    // ---------------------------------------------------------------------
    // 6. Mixed DDR Modes
    // ---------------------------------------------------------------------

    #[test]
    fn mixed_mode_isolates_inputs_and_outputs() {
        let mut port = Port::new();

        // Lower 4 bits output (1), upper 4 bits input (0)
        port.write_ddr(0x0F);

        // Latch has $55 (%0101_0101), External pins have $AA (%1010_1010)
        port.write_latch(0x55);
        port.set_pins(0xAA);

        // Driven pins: upper 4 float high (1), lower 4 drive latch ($5 = %0101)
        // Expected driven: %1111_0101 = 0xF5
        assert_eq!(port.driven_pins(), 0xF5);

        // Read pins:
        // Upper 4: from external pins ($A = %1010)
        // Lower 4: pulled down by pins
        // Expected: %1010_0000 = 0xA0
        assert_eq!(port.read_pins(), 0xA0);
    }

    // ---------------------------------------------------------------------
    // 7. Peripheral Overrides (Timer PB6 / PB7)
    // ---------------------------------------------------------------------

    #[test]
    fn override_takes_precedence_over_input_mode() {
        let mut port = Port::new();
        port.write_ddr(0x00); // Input mode

        // Timer A takes over PB6 (bit 6) and drives it low (0)
        port.set_output_override(1 << 6, 0x00);

        // Bit 6 must be 0, all others 1 (pull-ups)
        assert_eq!(port.driven_pins(), !(1 << 6));
        assert_eq!(port.read_pins(), !(1 << 6));

        // Timer A toggles PB6 to high (1)
        port.set_output_override(1 << 6, 1 << 6);
        assert_eq!(port.driven_pins(), 0xFF);
        assert_eq!(port.read_pins(), 0xFF);
    }

    #[test]
    fn override_takes_precedence_over_output_latch() {
        let mut port = Port::new();
        port.write_ddr(0xFF); // All outputs
        port.write_latch(0xFF); // Latch is high

        // Timer B overrides PB7 (bit 7) and forces it low
        port.set_output_override(1 << 7, 0x00);

        assert_eq!(port.read_pins(), 0x7F);
        assert_eq!(port.driven_pins(), 0x7F);

        // Internal latch remains untouched
        assert_eq!(port.read_latch(), 0xFF);
    }

    #[test]
    fn releasing_override_immediately_restores_latch() {
        let mut port = Port::new();
        port.write_ddr(0xFF);
        port.write_latch(0xC0); // Bits 6 and 7 are 1

        // Override PB6 to 0
        port.set_output_override(1 << 6, 0x00);
        assert_eq!(port.read_pins(), 0x80);

        // Write a new latch while overridden
        port.write_latch(0x40); // Bit 6 is 1, bit 7 is 0
        assert_eq!(port.read_pins(), 0x00, "PB6 is still overridden to 0");

        // Release the override (mask = 0)
        port.set_output_override(0x00, 0x00);

        // PB6 immediately reverts to the latch value (1)
        assert_eq!(port.read_pins(), 0x40);
        assert_eq!(port.driven_pins(), 0x40);
    }

    #[test]
    fn independent_timer_a_and_b_overrides_on_pb6_and_pb7() {
        let mut port = Port::new();
        port.write_ddr(0x00); // Inputs

        // Override PB6 (0) and PB7 (1) simultaneously
        port.set_output_override((1 << 6) | (1 << 7), 1 << 7);

        // Bits 0-5 float high (1), Bit 6 is 0, Bit 7 is 1
        // Expected: %1011_1111 = 0xBF
        assert_eq!(port.read_pins(), 0xBF);
        assert_eq!(port.driven_pins(), 0xBF);
    }
}
