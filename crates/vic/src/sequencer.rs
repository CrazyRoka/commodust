pub struct Sequencer {
    /// Video Counter. Generates addresses for the video matrix.
    vc: u16,
    /// Video Counter Base. Holds the start address for the current text line.
    vcbase: u16,
    /// Row Counter. Tracks the 8 pixel lines of a text row (0-7).
    rc: u8,
    /// Video Matrix Line Index. Points to the current position in the internal 40x12-bit video matrix/color line buffer.
    vmli: u8,
    /// Stores the 40 screen codes (8-bit) and 40 color nybbles (4-bit) fetched during a Bad Line.
    line_buffer: [u16; 40],
    /// A latch tracking if DEN (Display Enable, $D011 bit 4) was active on line $30.
    den_on_line_30: bool,
    /// Display state.
    is_display_state: bool,
    /// A latch for the current line. Remembers that this line is a Bad Line.
    is_bad_line: bool,
    is_c_access: bool,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            vc: 0,
            vcbase: 0,
            rc: 0,
            vmli: 0,
            line_buffer: [0; 40],
            den_on_line_30: false,
            is_display_state: false,
            is_bad_line: false,
            is_c_access: false,
        }
    }

    pub fn tick(&mut self, cycle: u8, line: u16, yscroll: u8, den: bool) {
        // VBlank reset
        if line == 0 && cycle == 0 {
            self.vcbase = 0;
            self.den_on_line_30 = false;
        }

        // Line $30 latch
        if line == 0x30 && den {
            self.den_on_line_30 = true;
        }

        // Bad line check
        if cycle == 12 {
            self.is_bad_line = line >= 0x30
                && line <= 0xF7
                && ((line & 7) as u8 == yscroll & 7)
                && self.den_on_line_30;
        }

        // Row start
        if cycle == 14 {
            self.vc = self.vcbase;
            self.vmli = 0;

            if self.is_bad_line && ((line & 7) as u8 == yscroll & 7) && self.den_on_line_30 {
                self.rc = 0;
                self.is_display_state = true;
            }
        }

        // Column fetches
        if cycle >= 15 && cycle <= 54 {
            self.is_c_access = self.is_bad_line;
            if self.is_display_state {
                self.vc = (self.vc + 1) & 0x3FF;
                self.vmli += 1;
            }
        } else {
            self.is_c_access = false;
        }

        // BA release
        if cycle == 55 {
            self.is_bad_line = false;
        }

        // End of line check
        if cycle == 58 && self.is_display_state {
            if self.rc == 7 {
                self.is_display_state = false;
                self.vcbase = self.vc;
            } else {
                self.rc += 1;
            }
        }
    }

    pub fn ba(&self) -> bool {
        !self.is_bad_line
    }

    pub fn is_c_access_cycle(&self) -> bool {
        self.is_c_access
    }

    pub fn vc(&self) -> u16 {
        self.vc
    }

    pub fn rc(&self) -> u8 {
        self.rc
    }

    pub fn vmli(&self) -> u8 {
        self.vmli
    }

    pub fn store_c_access(&mut self, char_code: u8, color: u8) {
        self.line_buffer[self.vmli as usize] = (char_code as u16) | (((color & 0x0F) as u16) << 8);
    }

    /// Returns the screen code and color nybble for an explicit display
    /// column (0..=39) of the current video matrix line.
    pub fn char_and_color_at(&self, column: u8) -> (u8, u8) {
        let value = self.line_buffer[(column % 40) as usize];
        ((value & 0xFF) as u8, (value >> 8) as u8)
    }

    pub fn current_char_and_color(&self) -> (u8, u8) {
        self.char_and_color_at(self.vmli)
    }

    pub fn is_bad_line(&self) -> bool {
        self.is_bad_line
    }

    pub fn is_display_state(&self) -> bool {
        self.is_display_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL / RESET STATE
    //    * vc = 0, vcbase = 0, rc = 0, vmli = 0.
    //    * ba = true (bus available to CPU).
    //    * state = Idle, bad_line = false, den_on_line_30 = false.
    //
    // 2. BAD LINE CONDITION (Evaluated on lines $30..=$F7)
    //    * Condition: (line in 0x30..=0xF7) && ((line & 7) == yscroll) && den_on_line_30.
    //    * den_on_line_30 latches true if DEN=1 at any time during raster line 0x30.
    //    * If DEN was 0 during line 0x30, NO bad lines occur for the entire frame!
    //    * Line 0x2F is NEVER a bad line.
    //    * Line 0xF8 is NEVER a bad line.
    //
    // 3. BUS ARBITRATION (BA Line)
    //    * On a Bad Line:
    //      - At cycle 12: ba goes false.
    //      - Stays false through cycles 12..=54.
    //      - At cycle 55: ba returns true.
    //    * On a Non-Bad Line:
    //      - ba remains true throughout all cycles.
    //
    // 4. ROW INITIALIZATION (Cycle 14)
    //    * vc is reloaded from vcbase (vc = vcbase).
    //    * vmli is reset to 0.
    //    * If Bad Line: rc is reset to 0, state becomes Display.
    //
    // 5. C-ACCESS & G-ACCESS PHASES (Cycles 15..=54)
    //    * is_c_access_cycle() is true on cycles 15..=54 IF AND ONLY IF on a Bad Line.
    //    * In Display state: vc increments by 1 and vmli increments by 1 on each cycle 15..=54
    //      (advancing 40 characters per line).
    //    * In Idle state: vc and vmli DO NOT increment during cycles 15..=54.
    //
    // 6. END OF SCANLINE / ROW CHECK (Cycle 58)
    //    * If in Display state and rc < 7:
    //      - rc increments by 1.
    //      - vcbase remains UNCHANGED (points to start of current text row).
    //    * If in Display state and rc == 7:
    //      - Character row finished!
    //      - vcbase is updated with vc (vcbase = vc, advancing by 40).
    //      - State switches to Idle.
    //
    // 7. FRAME RESET (Line 0)
    //    * On line 0: vcbase is reset to 0.
    //
    // 8. LINE BUFFER STORAGE
    //    * store_c_access writes to line_buffer at current vmli.
    //    * current_char_and_color reads from line_buffer at current vmli.
    //    * Line buffer contents persist across subsequent non-bad lines of the row.
    // =====================================================================

    // Helper: Step a full scanline cycle by cycle (cycles 0..=62 for PAL)
    fn run_scanline(seq: &mut Sequencer, line: u16, yscroll: u8, den: bool) {
        for cycle in 0..63 {
            seq.tick(cycle, line, yscroll, den);
        }
    }

    // Helper: Prime the sequencer past line 0x30 with DEN active
    fn prime_sequencer_to_display_area(seq: &mut Sequencer, yscroll: u8) {
        for line in 0..=0x30 {
            run_scanline(seq, line, yscroll, true);
        }
    }

    /// One full PAL frame (312 lines × 63 cycles), YSCROLL=0, DEN as given.
    fn run_pal_frame(seq: &mut Sequencer, den: bool) {
        for line in 0..312u16 {
            run_scanline(seq, line, 0, den);
        }
    }

    /// Prime the sequencer the way a real frame leaves it after the FIRST
    /// COMPLETE character row (lines $00..=$37): display state OFF (RC hit 7
    /// at cycle 58 of line $37), RC=7, VCBASE=40, VC=40, VMLI=40, DEN latched.
    /// Unlike `prime_sequencer_to_display_area` (which stops mid-row in
    /// display state), this is the state every real row start enters from:
    /// an IDLE -> bad-line transition.
    fn prime_full_row(seq: &mut Sequencer) {
        for line in 0..=0x37u16 {
            run_scanline(seq, line, 0, true);
        }
    }

    // ---------------------------------------------------------------------
    // 1. Initial State
    // ---------------------------------------------------------------------

    #[test]
    fn new_sequencer_starts_in_idle_with_ba_true() {
        let seq = Sequencer::new();
        assert_eq!(seq.vc(), 0);
        assert_eq!(seq.rc(), 0);
        assert_eq!(seq.vmli(), 0);
        assert!(seq.ba());
        assert!(!seq.is_bad_line());
        assert!(!seq.is_display_state());
        assert!(!seq.is_c_access_cycle());
    }

    // ---------------------------------------------------------------------
    // 2. Bad Line Condition Evaluation
    // ---------------------------------------------------------------------

    #[test]
    fn line_0x2f_is_never_a_bad_line() {
        let mut seq = Sequencer::new();
        // Even with DEN=1 and line & 7 == yscroll (0x2F & 7 = 7)
        prime_sequencer_to_display_area(&mut seq, 7);

        seq.tick(12, 0x2F, 7, true);
        assert!(!seq.is_bad_line(), "Line 0x2F must never be a bad line");
        assert!(seq.ba(), "BA must remain high on line 0x2F");
    }

    #[test]
    fn line_0x30_is_a_bad_line_when_yscroll_matches() {
        let mut seq = Sequencer::new();
        // Line 0x30 & 7 == 0. So yscroll = 0 will match.
        // Run lines up to 0x2F
        for line in 0..0x30 {
            run_scanline(&mut seq, line, 0, true);
        }

        // On line 0x30, cycle 12: Bad line must be detected
        seq.tick(12, 0x30, 0, true);
        assert!(
            seq.is_bad_line(),
            "Line 0x30 with yscroll=0 must be a Bad Line"
        );
        assert!(!seq.ba(), "BA must be pulled low at cycle 12 of a Bad Line");
    }

    #[test]
    fn bad_line_requires_den_active_on_line_0x30() {
        let mut seq = Sequencer::new();
        // Run up to line 0x30 with DEN = FALSE
        for line in 0..=0x30 {
            run_scanline(&mut seq, line, 0, false); // DEN=false!
        }

        // Now even on line 0x38 (which matches yscroll=0), if DEN was 0 on line 0x30, NO Bad Lines occur!
        seq.tick(12, 0x38, 0, true);
        assert!(
            !seq.is_bad_line(),
            "Bad line must not trigger if DEN was inactive on line 0x30"
        );
        assert!(seq.ba());
    }

    #[test]
    fn line_0xf7_is_last_possible_bad_line() {
        let mut seq = Sequencer::new();
        // 0xF7 & 7 == 7. YSCROLL = 7.
        prime_sequencer_to_display_area(&mut seq, 7);

        // Run up to line 0xF6
        for line in 0x31..0xF7 {
            run_scanline(&mut seq, line, 7, true);
        }

        seq.tick(12, 0xF7, 7, true);
        assert!(seq.is_bad_line(), "Line 0xF7 must be a Bad Line");

        // Line 0xF8: (0xF8 & 7 == 0). Even with yscroll=0, 0xF8 > 0xF7 so it is NOT a bad line!
        seq.tick(12, 0xF8, 0, true);
        assert!(!seq.is_bad_line(), "Line 0xF8 must never be a bad line");
    }

    // ---------------------------------------------------------------------
    // 3. BA (Bus Available) Timing
    // ---------------------------------------------------------------------

    #[test]
    fn ba_drops_at_cycle_12_and_rises_at_cycle_55_on_bad_line() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Scanline 0x38 (0x38 & 7 == 0 -> Bad Line)
        let line = 0x38;

        for cycle in 0..12 {
            seq.tick(cycle, line, 0, true);
            assert!(
                seq.ba(),
                "BA must be high before cycle 12 (cycle {})",
                cycle
            );
        }

        // Cycle 12: BA goes low!
        seq.tick(12, line, 0, true);
        assert!(!seq.ba(), "BA must be pulled low on cycle 12");

        // Cycles 13..=54: BA stays low
        for cycle in 13..=54 {
            seq.tick(cycle, line, 0, true);
            assert!(
                !seq.ba(),
                "BA must remain low during stolen cycles (cycle {})",
                cycle
            );
        }

        // Cycle 55: BA returns high
        seq.tick(55, line, 0, true);
        assert!(seq.ba(), "BA must return high on cycle 55");
    }

    // ---------------------------------------------------------------------
    // 4. Cycle 14 Updates & C-Access Timing
    // ---------------------------------------------------------------------

    #[test]
    fn cycle_14_resets_rc_and_enters_display_state_on_bad_line() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        let line = 0x38; // Bad Line
        for cycle in 0..14 {
            seq.tick(cycle, line, 0, true);
        }

        // At cycle 14:
        seq.tick(14, line, 0, true);
        assert_eq!(seq.rc(), 0, "RC must reset to 0 on a Bad Line at cycle 14");
        assert_eq!(seq.vmli(), 0, "VMLI must reset to 0 at cycle 14");
        assert!(
            seq.is_display_state(),
            "Must enter Display state at cycle 14"
        );
    }

    #[test]
    fn c_access_active_only_on_bad_lines_during_cycles_15_to_54() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Line 0x38 is a Bad Line
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
            assert!(
                !seq.is_c_access_cycle(),
                "C-access inactive before cycle 15"
            );
        }

        for cycle in 15..=54 {
            seq.tick(cycle, 0x38, 0, true);
            assert!(
                seq.is_c_access_cycle(),
                "C-access must be active on cycle {} of Bad Line",
                cycle
            );
        }

        seq.tick(55, 0x38, 0, true);
        assert!(!seq.is_c_access_cycle(), "C-access inactive after cycle 54");

        // Next line (0x39) is NOT a Bad Line: c-access must NEVER be active
        for cycle in 0..63 {
            seq.tick(cycle, 0x39, 0, true);
            assert!(
                !seq.is_c_access_cycle(),
                "Non-bad line must never have c-accesses"
            );
        }
    }

    // ---------------------------------------------------------------------
    // 5. VC and VMLI Increments
    // ---------------------------------------------------------------------

    #[test]
    fn vc_and_vmli_increment_40_times_during_cycles_15_to_54() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // On Bad Line 0x38:
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert_eq!(seq.vmli(), 0);
        assert_eq!(seq.vc(), 0);

        // Cycles 15..=54 (40 cycles): VC and VMLI must advance each cycle
        for cycle in 15..=54 {
            seq.tick(cycle, 0x38, 0, true);
            let expected_count = (cycle - 14) as u8;
            assert_eq!(seq.vmli(), expected_count);
            assert_eq!(seq.vc(), expected_count as u16);
        }

        assert_eq!(seq.vmli(), 40);
        assert_eq!(seq.vc(), 40);
    }

    // ---------------------------------------------------------------------
    // 6. 8-Scanline Character Row Lifecycle (RC Counter & VCBASE Update)
    // ---------------------------------------------------------------------

    #[test]
    fn character_row_progresses_rc_0_to_7_then_advances_vcbase_by_40() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Row begins on Bad Line 0x38:
        // We will step through 8 consecutive scan lines: 0x38..=0x3F
        for row_line in 0..8 {
            let line = 0x38 + row_line;

            // Step through cycles 0..57
            for cycle in 0..58 {
                seq.tick(cycle, line, 0, true);
            }

            assert_eq!(
                seq.rc(),
                row_line as u8,
                "RC must match row line index {} before cycle 58",
                row_line
            );

            // Cycle 58: check RC increment
            seq.tick(58, line, 0, true);

            if row_line < 7 {
                assert_eq!(
                    seq.rc(),
                    (row_line + 1) as u8,
                    "RC must increment at cycle 58 on line {}",
                    line
                );
                assert_eq!(
                    seq.vcbase, 0,
                    "VCBASE must NOT change until the 8th scanline finishes"
                );
            } else {
                // On the 8th line (row_line == 7, RC == 7):
                // Character row is complete! VCBASE must advance by 40!
                assert_eq!(
                    seq.vcbase, 40,
                    "VCBASE must advance by 40 after the 8th scanline of the row"
                );
                assert!(
                    !seq.is_display_state(),
                    "Sequencer must return to Idle after row completes"
                );
            }

            // Finish the line
            for cycle in 59..63 {
                seq.tick(cycle, line, 0, true);
            }
        }
    }

    // ---------------------------------------------------------------------
    // 7. Line 0 Frame Reset
    // ---------------------------------------------------------------------

    #[test]
    fn line_0_resets_vcbase_to_zero() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Complete 1 character row so VCBASE becomes 40
        for line in 0x38..=0x3F {
            run_scanline(&mut seq, line, 0, true);
        }
        assert_eq!(seq.vcbase, 40);

        // Now simulate reaching line 0 of the next frame
        seq.tick(0, 0, 0, true);
        assert_eq!(seq.vcbase, 0, "VCBASE must reset to 0 on line 0");
    }

    // ---------------------------------------------------------------------
    // 8. Line Buffer Read & Write Persistence
    // ---------------------------------------------------------------------

    #[test]
    fn line_buffer_stores_and_retains_character_data_across_entire_row() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        let bad_line = 0x38;

        // Step up to cycle 15
        for cycle in 0..15 {
            seq.tick(cycle, bad_line, 0, true);
        }

        // On Bad Line: populate line buffer across cycles 15..=54
        for col in 0..40 {
            seq.store_c_access(b'A' + col, col % 16);
            seq.tick(15 + col, bad_line, 0, true);
        }

        // Finish bad line
        for cycle in 55..63 {
            seq.tick(cycle, bad_line, 0, true);
        }

        // Now on the NEXT line (0x39, non-bad line), verify that the line buffer
        // still retains the exact characters we wrote, column by column!
        for cycle in 0..14 {
            seq.tick(cycle, 0x39, 0, true);
        }

        // At cycle 14: VMLI resets to 0
        seq.tick(14, 0x39, 0, true);
        assert_eq!(seq.vmli(), 0);

        for col in 0..40 {
            assert_eq!(
                seq.current_char_and_color(),
                (b'A' + col, col % 16),
                "Line buffer must retain stored data on subsequent line column {}",
                col
            );
            seq.tick(15 + col, 0x39, 0, true);
        }
    }

    // ---------------------------------------------------------------------
    // 9. Idle State During VBlank & Pre-Display (Lines 0..=0x2F)
    // ---------------------------------------------------------------------

    #[test]
    fn remains_idle_with_no_increments_before_line_0x30() {
        let mut seq = Sequencer::new();

        // Run through lines 0..0x2F with DEN=1
        for line in 0..0x30 {
            for cycle in 0..63 {
                seq.tick(cycle, line, 0, true);
                assert!(
                    !seq.is_display_state(),
                    "Must not enter display state before line 0x30 (line {}, cycle {})",
                    line,
                    cycle
                );
                assert_eq!(seq.vc(), 0, "VC must stay 0 before active display");
                assert_eq!(seq.rc(), 0, "RC must stay 0 before active display");
                assert_eq!(seq.vmli(), 0, "VMLI must stay 0 before active display");
            }
        }
    }

    // ---------------------------------------------------------------------
    // 10. Cycle 14 Reload on Non-Bad Lines (RC = 1..=7)
    // ---------------------------------------------------------------------

    #[test]
    fn non_bad_lines_reload_vc_from_vcbase_and_clear_vmli_at_cycle_14() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Run Bad Line 0x38 to completion (VC ends at 40, VMLI ends at 40)
        run_scanline(&mut seq, 0x38, 0, true);

        // Line 0x39 (RC=1, NON-bad line):
        // Up to cycle 13, VC has old value
        for cycle in 0..14 {
            seq.tick(cycle, 0x39, 0, true);
        }

        // At Cycle 14: VC MUST reload from VCBASE (which is 0), VMLI must be 0!
        seq.tick(14, 0x39, 0, true);
        assert_eq!(
            seq.vc(),
            0,
            "Non-bad line must reload VC from VCBASE (0) at cycle 14"
        );
        assert_eq!(
            seq.vmli(),
            0,
            "Non-bad line must reset VMLI to 0 at cycle 14"
        );
        assert!(
            seq.is_display_state(),
            "Must remain in Display state on non-bad line of character row"
        );
        assert_eq!(seq.rc(), 1, "RC must remain 1 on this line");
    }

    // ---------------------------------------------------------------------
    // 11. Counters Frozen Outside Cycles 15..=54
    // ---------------------------------------------------------------------

    #[test]
    fn vmli_and_vc_do_not_increment_outside_cycles_15_to_54() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Run up to cycle 54 on Bad Line 0x38
        for cycle in 0..=54 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert_eq!(seq.vmli(), 40);
        assert_eq!(seq.vc(), 40);

        // Cycles 55..=62: VMLI and VC must FREEZE (must not grow past 40!)
        for cycle in 55..63 {
            seq.tick(cycle, 0x38, 0, true);
            assert_eq!(
                seq.vmli(),
                40,
                "VMLI must not increment past cycle 54 (cycle {})",
                cycle
            );
            assert_eq!(
                seq.vc(),
                40,
                "VC must not increment past cycle 54 (cycle {})",
                cycle
            );
        }
    }

    // ---------------------------------------------------------------------
    // 12. Multiple Character Rows Accumulate VCBASE (0, 40, 80, 120...)
    // ---------------------------------------------------------------------

    #[test]
    fn multiple_rows_advance_vcbase_by_40_per_row() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Run 4 full character rows (4 rows * 8 lines/row = 32 scanlines)
        // Row 0: lines 0x38..=0x3F -> VCBASE becomes 40
        // Row 1: lines 0x40..=0x47 -> VCBASE becomes 80
        // Row 2: lines 0x48..=0x4F -> VCBASE becomes 120
        // Row 3: lines 0x50..=0x57 -> VCBASE becomes 160
        for row in 0..4 {
            let start_line = 0x38 + (row * 8);
            for offset in 0..8 {
                run_scanline(&mut seq, start_line + offset, 0, true);
            }
            let expected_base = (row + 1) * 40;
            assert_eq!(
                seq.vcbase, expected_base,
                "VCBASE must be {} after row {}",
                expected_base, row
            );
        }
    }

    // ---------------------------------------------------------------------
    // 13. 10-Bit Counter Masking (0..=1023)
    // ---------------------------------------------------------------------

    #[test]
    fn vc_and_vcbase_wrap_at_1024() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Pre-set vcbase to 1000 (near 10-bit limit of 1024)
        seq.vcbase = 1000;

        // Run a Bad Line: VC loads 1000 at cycle 14, then increments 40 times
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert_eq!(seq.vc(), 1000);

        // Step 40 cycles (1000 + 40 = 1040 -> must wrap to 1040 & 0x3FF = 16!)
        for cycle in 15..=54 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert_eq!(seq.vc(), 16, "VC must wrap modulo 1024 (10-bit counter)");
    }

    // ---------------------------------------------------------------------
    // 14. DEN Frame Latch Clears on Line 0
    // ---------------------------------------------------------------------

    #[test]
    fn den_on_line_30_clears_on_new_frame() {
        let mut seq = Sequencer::new();

        // Frame 1: DEN active on line 0x30
        prime_sequencer_to_display_area(&mut seq, 0);
        assert!(seq.den_on_line_30);

        // Finish Frame 1 (lines up to 311)
        for line in 0x31..312 {
            run_scanline(&mut seq, line, 0, true);
        }

        // Line 0 of Frame 2: den_on_line_30 must clear!
        seq.tick(0, 0, 0, true);
        assert!(
            !seq.den_on_line_30,
            "den_on_line_30 must reset on line 0 of a new frame"
        );
    }

    // ---------------------------------------------------------------------
    // 15. Bad Line Condition Checked Dynamically at Cycle 12
    // ---------------------------------------------------------------------

    #[test]
    fn yscroll_changed_mid_line_before_cycle_12_affects_bad_line() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        let line = 0x38; // (0x38 & 7 == 0)

        // At cycle 0..11, yscroll is 3 (no match)
        for cycle in 0..12 {
            seq.tick(cycle, line, 3, true);
        }
        assert!(!seq.is_bad_line());
        assert!(seq.ba());

        // Software changes yscroll to 0 exactly at cycle 12!
        seq.tick(12, line, 0, true);
        assert!(
            seq.is_bad_line(),
            "Changing yscroll to match at cycle 12 must trigger a Bad Line"
        );
        assert!(!seq.ba());
    }

    // ---------------------------------------------------------------------
    // 16. Turning DEN Off Mid-Row Does Not Abort Current Character Row
    // ---------------------------------------------------------------------

    #[test]
    fn den_per_frame_latch_keeps_bad_lines_after_den_is_cleared() {
        let mut seq = Sequencer::new();
        prime_full_row(&mut seq);
        for line in 0x38..=0x3F {
            run_scanline(&mut seq, line, 0, false); // DEN cleared mid-frame
        }
        for cycle in 0..=14 {
            seq.tick(cycle, 0x40, 0, true);
        }
        assert!(seq.is_bad_line(), "next row still starts (per-frame latch)");
        assert!(seq.is_display_state());
        assert_eq!(seq.rc(), 0);
    }

    // ---------------------------------------------------------------------
    // 17. Cycle 12-14 BA De-assertion Window
    // ---------------------------------------------------------------------

    #[test]
    fn ba_is_low_specifically_for_cycles_12_to_54() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        let line = 0x38;

        // Cycle 11: BA is high
        seq.tick(11, line, 0, true);
        assert!(seq.ba());

        // Cycle 12: BA drops
        seq.tick(12, line, 0, true);
        assert!(!seq.ba());

        // Cycle 14: BA is still low
        seq.tick(14, line, 0, true);
        assert!(!seq.ba());

        // Cycle 54: BA is still low
        for cycle in 15..=54 {
            seq.tick(cycle, line, 0, true);
            assert!(!seq.ba());
        }

        // Cycle 55: BA rises
        seq.tick(55, line, 0, true);
        assert!(seq.ba());
    }

    // ---------------------------------------------------------------------
    // 19. den_on_line_30 Stays Latched Even If DEN Is Toggled Later
    // ---------------------------------------------------------------------

    #[test]
    fn den_latched_on_line_30_persists_across_intermittent_den_clears() {
        let mut seq = Sequencer::new();

        // Line 0x30: DEN is set to 1
        run_scanline(&mut seq, 0x30, 0, true);
        assert!(seq.den_on_line_30);

        // Lines 0x31..=0x36: Software temporarily turns DEN off (den = false)
        for line in 0x31..=0x36 {
            run_scanline(&mut seq, line, 0, false);
            assert!(
                seq.den_on_line_30,
                "den_on_line_30 must remain latched despite DEN toggling"
            );
        }

        // Line 0x38: Software turns DEN back on. Bad Line must STILL occur!
        seq.tick(12, 0x38, 0, true);
        assert!(
            seq.is_bad_line(),
            "Bad line must trigger if den_on_line_30 was preserved"
        );
    }

    // ---------------------------------------------------------------------
    // 20. Line Buffer VMLI Never Panics Past Cycle 54
    // ---------------------------------------------------------------------

    #[test]
    fn current_char_and_color_safe_at_cycle_boundaries() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Run full Bad Line 0x38
        for cycle in 0..63 {
            seq.tick(cycle, 0x38, 0, true);
            // Calling current_char_and_color on any cycle must never panic!
            if seq.vmli() < 40 {
                let _ = seq.current_char_and_color();
            }
        }

        // On cycle 54, vmli was 40.
        // During cycles 55..62, vmli must not cause out-of-bounds on line_buffer!
        assert!(
            seq.vmli() <= 40,
            "vmli must be clamped or capped at 40 (got {})",
            seq.vmli()
        );
    }

    // ---------------------------------------------------------------------
    // 21. Full Frame Text Screen: 25 Rows Advance VCBASE to 1000
    // ---------------------------------------------------------------------

    #[test]
    fn full_screen_25_character_rows_reach_vcbase_1000() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // A standard C64 screen displays 25 text rows.
        // Row 0 begins on line 0x30 (when yscroll = 0).
        // 25 rows * 8 lines/row = 200 scanlines total (lines 0x30..=0xF7).
        for row in 0..25 {
            let start_line = 0x30 + (row * 8);
            for offset in 0..8 {
                run_scanline(&mut seq, start_line + offset, 0, true);
            }
        }

        // After 25 rows of 40 characters:
        // VCBASE must have accumulated exactly 25 * 40 = 1000 screen codes!
        assert_eq!(
            seq.vcbase, 1000,
            "After all 25 text rows, VCBASE must equal 1000"
        );
        assert!(
            !seq.is_display_state(),
            "Must be Idle after the 25th row completes"
        );
    }

    #[test]
    fn every_yscroll_value_yields_25_rows_starting_at_0x30_plus_yscroll() {
        for yscroll in 0u8..=7 {
            let mut seq = Sequencer::new();
            let first_bad = 0x30 + yscroll as u16;
            for line in 0..first_bad {
                run_scanline(&mut seq, line, yscroll, true);
            }
            assert!(!seq.is_display_state(), "yscroll {}", yscroll);
            seq.tick(12, first_bad, yscroll, true);
            assert!(
                seq.is_bad_line(),
                "first bad line for yscroll={} must be ${:X}",
                yscroll,
                first_bad
            );

            for line in first_bad..312 {
                run_scanline(&mut seq, line, yscroll, true);
            }
            assert_eq!(seq.vcbase, 1000, "yscroll {}", yscroll);
            assert!(!seq.is_display_state(), "yscroll {}", yscroll);
            assert_eq!(seq.rc(), 7, "yscroll {}", yscroll);
        }
    }

    #[test]
    fn yscroll_mismatch_on_candidate_line_suppresses_bad_line() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for y in 1u8..=7 {
            seq.tick(12, 0x38, y, true);
            assert!(!seq.is_bad_line(), "yscroll={} must not match line $38", y);
            assert!(seq.ba());
        }
    }

    #[test]
    fn yscroll_break_before_cycle_12_prevents_bad_line() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for cycle in 0..12 {
            seq.tick(cycle, 0x38, 0, true); // matching through cycle 11
        }
        seq.tick(12, 0x38, 3, true); // CPU writes a new YSCROLL before cycle 12
        assert!(!seq.is_bad_line());
        assert!(seq.ba());
        for cycle in 13..63 {
            seq.tick(cycle, 0x38, 3, true);
            assert!(seq.ba());
            assert!(!seq.is_c_access_cycle());
        }
    }

    #[test]
    fn bad_line_latched_at_cycle_12_survives_mid_fetch_yscroll_and_den_changes() {
        // Documents the snapshot semantics: once cycle 12 has decided "bad
        // line", YSCROLL/DEN writes within 12..=54 do not cancel BA or the
        // c-accesses. (The RC reset is separate — see the bug tests.)
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for cycle in 0..=12 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert!(seq.is_bad_line());
        for cycle in 13..=54 {
            seq.tick(cycle, 0x38, 5, false); // match broken AND DEN cleared
            assert!(
                !seq.ba(),
                "BA stays low for the whole window (cycle {})",
                cycle
            );
            if cycle >= 15 {
                assert!(seq.is_c_access_cycle(), "cycle {}", cycle);
            }
        }
        seq.tick(55, 0x38, 5, false);
        assert!(seq.ba());
    }

    #[test]
    fn den_latches_from_any_cycle_of_line_30_including_the_last() {
        let mut seq = Sequencer::new();
        for line in 0..0x30 {
            run_scanline(&mut seq, line, 0, true); // DEN on, but before line $30
        }
        for cycle in 0..62 {
            seq.tick(cycle, 0x30, 0, false); // off for cycles 0..=61...
        }
        seq.tick(62, 0x30, 0, true); // ...set only at the very last cycle
        assert!(seq.den_on_line_30);
        assert!(
            !seq.is_display_state(),
            "line $30 itself was not a bad line"
        );
        seq.tick(12, 0x38, 0, true);
        assert!(seq.is_bad_line());
    }

    #[test]
    fn den_during_line_2f_does_not_latch() {
        let mut seq = Sequencer::new();
        for line in 0..=0x2F {
            run_scanline(&mut seq, line, 0, true);
        }
        run_scanline(&mut seq, 0x30, 0, false);
        assert!(!seq.den_on_line_30);
        seq.tick(12, 0x38, 0, true);
        assert!(!seq.is_bad_line());
    }

    #[test]
    fn den_set_exactly_at_cycle_12_of_line_30_latches_in_time() {
        // Pins the intra-tick ordering: the DEN latch is evaluated before the
        // bad-line condition within the same tick.
        let mut seq = Sequencer::new();
        for line in 0..0x30 {
            run_scanline(&mut seq, line, 0, false);
        }
        for cycle in 0..12 {
            seq.tick(cycle, 0x30, 0, false);
        }
        seq.tick(12, 0x30, 0, true);
        assert!(seq.den_on_line_30);
        assert!(seq.is_bad_line());
    }

    #[test]
    fn den_off_during_line_30_blanks_the_entire_frame() {
        let mut seq = Sequencer::new();
        for line in 0..312u16 {
            let den = line != 0x30; // off exactly during line $30
            run_scanline(&mut seq, line, 0, den);
        }
        assert!(!seq.den_on_line_30);
        assert!(!seq.is_display_state());
        assert_eq!(seq.vc(), 0);
        assert_eq!(seq.vmli(), 0);
        assert_eq!(seq.rc(), 0);
        assert_eq!(seq.vcbase, 0);
    }

    #[test]
    fn den_latch_lifecycle_across_three_frames() {
        let mut seq = Sequencer::new();

        run_pal_frame(&mut seq, true); // frame 1: display on
        assert!(seq.den_on_line_30);
        assert_eq!(seq.vcbase, 1000);

        run_pal_frame(&mut seq, false); // frame 2: DEN never set
        assert!(!seq.den_on_line_30);
        assert!(!seq.is_display_state());
        assert_eq!(seq.vcbase, 0);

        // frame 3: DEN on again — bad lines return with line $30
        for cycle in 0..=14 {
            seq.tick(cycle, 0x30, 0, true);
        }
        assert!(seq.den_on_line_30);
        assert!(seq.is_bad_line());
        assert!(seq.is_display_state());
    }

    #[test]
    fn non_bad_display_line_full_behavior() {
        // Line $31 is the 2nd line of row 0: display state, but no bad line.
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for cycle in 0..63u8 {
            seq.tick(cycle, 0x31, 0, true);
            assert!(seq.ba(), "g-accesses never steal the bus (cycle {})", cycle);
            assert!(!seq.is_c_access_cycle(), "cycle {}", cycle);
            if cycle < 14 {
                assert_eq!(seq.vc(), 40, "carried from previous line (cycle {})", cycle);
                assert_eq!(seq.vmli(), 40, "cycle {}", cycle);
            } else if cycle <= 54 {
                assert_eq!(seq.vc(), u16::from(cycle - 14), "cycle {}", cycle);
                assert_eq!(seq.vmli(), cycle - 14, "cycle {}", cycle);
            } else {
                assert_eq!(seq.vc(), 40, "frozen after cycle 54");
                assert_eq!(seq.vmli(), 40);
            }
        }
        assert!(seq.is_display_state());
        assert_eq!(seq.rc(), 2, "RC advanced 1 -> 2 at cycle 58");
    }

    #[test]
    fn last_row_line_still_fetches_and_displays_until_cycle_58() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for line in 0x31..=0x36 {
            run_scanline(&mut seq, line, 0, true); // RC 1 -> 7
        }
        assert_eq!(seq.rc(), 7);
        assert!(seq.is_display_state(), "RC=7 alone does not mean idle");
        assert_eq!(seq.vcbase, 0);

        for cycle in 0..14 {
            seq.tick(cycle, 0x37, 0, true);
            assert!(seq.is_display_state(), "cycle {}", cycle);
            assert_eq!(seq.rc(), 7);
        }
        seq.tick(14, 0x37, 0, true);
        assert_eq!(
            seq.vc(),
            0,
            "the 8th row line reloads VC like any display line"
        );
        for cycle in 15..=57 {
            seq.tick(cycle, 0x37, 0, true);
            assert!(
                seq.is_display_state(),
                "still displaying at cycle {}",
                cycle
            );
        }
        assert_eq!(seq.vc(), 40);
        seq.tick(58, 0x37, 0, true);
        assert!(
            !seq.is_display_state(),
            "the row ends at cycle 58 of its 8th line"
        );
        assert_eq!(seq.vcbase, 40, "VCBASE advances only now");
        assert_eq!(seq.rc(), 7, "RC freezes at 7 in idle");
    }

    #[test]
    fn rc_and_display_state_freeze_after_the_last_displayed_row() {
        let mut seq = Sequencer::new();
        run_pal_frame(&mut seq, true);
        // Lines $F8..$137 (the rest of the PAL frame): nothing may change.
        for line in 0xF8..0x138u16 {
            for cycle in 0..63u8 {
                seq.tick(cycle, line, 0, true);
                assert_eq!(seq.rc(), 7);
                assert!(!seq.is_display_state());
                assert!(seq.ba());
                assert!(!seq.is_c_access_cycle());
            }
        }
        assert_eq!(seq.vcbase, 1000);
        assert_eq!(seq.vc(), 1000);
    }

    #[test]
    fn line_0_cycle_0_reset_is_narrow() {
        // The line-0 reset must only touch VCBASE and the DEN latch — the
        // stale VC/VMMLI it leaves behind is exactly why the cycle-14 reload
        // has to be unconditional (see the bug tests).
        let mut seq = Sequencer::new();
        run_pal_frame(&mut seq, true);
        assert_eq!(seq.vcbase, 1000);
        seq.tick(0, 0, 0, true);
        assert_eq!(seq.vcbase, 0);
        assert!(!seq.den_on_line_30);
        assert_eq!(seq.vc(), 1000);
        assert_eq!(seq.vmli(), 0); // Correctly 0 because cycle 14 clears vmli
        assert_eq!(seq.rc(), 7);
    }

    #[test]
    fn vcbase_is_also_masked_to_10_bits() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        seq.vcbase = 1000;
        for line in 0x38..=0x3F {
            run_scanline(&mut seq, line, 0, true);
        }
        assert_eq!(seq.vcbase, 16, "1000 + 40 wraps modulo 1024");
    }

    #[test]
    fn yscroll_dodge_delays_row_start_fld_style() {
        // Classic FLD mechanism applied before cycle 12: on each candidate
        // line software writes a YSCROLL that does not match that line, so no
        // bad line fires; the row start moves to the next undodged candidate.
        let mut seq = Sequencer::new();
        prime_full_row(&mut seq); // row 0 complete: idle, RC=7, VCBASE=40

        for line in 0x38..=0x3F {
            let dodge = (((line & 7) + 1) & 7) as u8;
            for cycle in 0..63u8 {
                seq.tick(cycle, line, dodge, true);
                assert!(seq.ba(), "line ${:X} cycle {}", line, cycle);
                assert!(!seq.is_c_access_cycle());
                assert!(!seq.is_display_state());
            }
            assert_eq!(seq.rc(), 7, "RC untouched while dodging");
        }
        assert_eq!(seq.vcbase, 40);

        for cycle in 0..=14 {
            seq.tick(cycle, 0x40, 0, true); // YSCROLL restored
        }
        assert!(seq.is_display_state());
        assert_eq!(seq.rc(), 0);
        assert_eq!(
            seq.vc(),
            40,
            "row 1 of screen RAM: VC reloads from VCBASE=40"
        );
        for cycle in 15..=54 {
            seq.tick(cycle, 0x40, 0, true);
        }
        assert_eq!(seq.vc(), 80);
    }

    #[test]
    fn every_line_can_be_made_a_bad_line_via_per_line_yscroll() {
        // Rewriting YSCROLL on every line so it always matches turns every
        // line into a bad line: RC resets each line, the row never finishes,
        // VCBASE never advances. (The "maximum DMA" condition.)
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for line in 0x38..=0x45u16 {
            let yscroll = (line & 7) as u8;
            for cycle in 0..63u8 {
                seq.tick(cycle, line, yscroll, true);
                assert_eq!(
                    seq.ba(),
                    !(12..=54).contains(&cycle),
                    "line ${:X} cycle {}",
                    line,
                    cycle
                );
                assert_eq!(
                    seq.is_c_access_cycle(),
                    (15..=54).contains(&cycle),
                    "line ${:X} cycle {}",
                    line,
                    cycle
                );
            }
            assert_eq!(seq.rc(), 1, "RC resets at 14, increments to 1 at 58");
        }
        assert!(seq.is_display_state());
        assert_eq!(seq.vcbase, 0, "RC never reaches 7 -> VCBASE never updates");
    }

    #[test]
    fn line_buffer_roundtrips_extreme_values_without_side_effects() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
        }
        let (vc, vmli) = (seq.vc(), seq.vmli());

        seq.store_c_access(0x00, 0x0);
        assert_eq!(seq.current_char_and_color(), (0x00, 0x00));
        seq.store_c_access(0xFF, 0xF);
        assert_eq!(seq.current_char_and_color(), (0xFF, 0x0F));
        seq.store_c_access(0x80, 0x8);
        assert_eq!(seq.current_char_and_color(), (0x80, 0x08));
        seq.store_c_access(0x42, 0x2); // overwrites the same slot
        assert_eq!(seq.current_char_and_color(), (0x42, 0x02));

        assert_eq!(seq.vc(), vc, "store must not touch the counters");
        assert_eq!(seq.vmli(), vmli);
    }

    #[test]
    fn line_buffer_persists_across_all_seven_following_row_lines() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);

        // Fetch on the bad line $38 (store-before-tick convention):
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
        }
        for col in 0..40usize {
            seq.store_c_access(0x20 + col as u8, (col % 16) as u8);
            seq.tick(15 + col as u8, 0x38, 0, true);
        }
        for cycle in 55..63 {
            seq.tick(cycle, 0x38, 0, true);
        }

        // The remaining 7 lines of the row must read back identical data:
        for l in 1..=7u16 {
            let line = 0x38 + l;
            for cycle in 0..15 {
                seq.tick(cycle, line, 0, true);
            }
            assert_eq!(seq.vmli(), 0, "line ${:X}", line);
            for col in 0..40usize {
                assert_eq!(
                    seq.current_char_and_color(),
                    (0x20 + col as u8, (col % 16) as u8),
                    "line ${:X} column {}",
                    line,
                    col
                );
                seq.tick(15 + col as u8, line, 0, true);
            }
            for cycle in 55..63 {
                seq.tick(cycle, line, 0, true);
            }
        }
    }

    #[test]
    fn full_pal_frame_cycle_by_cycle_invariants() {
        let mut seq = Sequencer::new();

        for line in 0..312u16 {
            let bad_line = (0x30..=0xF7).contains(&line) && (line & 7) == 0;
            for cycle in 0..63u8 {
                seq.tick(cycle, line, 0, true);

                assert_eq!(
                    seq.ba(),
                    !(bad_line && (12..=54).contains(&cycle)),
                    "BA mismatch on line ${:03X} cycle {}",
                    line,
                    cycle
                );
                assert_eq!(
                    seq.is_c_access_cycle(),
                    bad_line && (15..=54).contains(&cycle),
                    "c-access mismatch on line ${:03X} cycle {}",
                    line,
                    cycle
                );

                let display_expected = if !(0x30..=0xF7).contains(&line) {
                    false
                } else {
                    let k = (line - 0x30) % 8;
                    if k == 0 {
                        cycle >= 14
                    } else if k == 7 {
                        cycle < 58
                    } else {
                        true
                    }
                };
                assert_eq!(
                    seq.is_display_state(),
                    display_expected,
                    "display state mismatch on line ${:03X} cycle {}",
                    line,
                    cycle
                );

                let rc_expected: u8 = if line < 0x30 {
                    0
                } else if line > 0xF7 {
                    7
                } else {
                    let k = (line - 0x30) % 8;
                    if k == 0 {
                        if cycle < 14 {
                            if line == 0x30 { 0 } else { 7 } // line $30 starts at 0, not 7!
                        } else if cycle < 58 {
                            0
                        } else {
                            1
                        }
                    } else if k == 7 {
                        7
                    } else if cycle < 58 {
                        k as u8
                    } else {
                        k as u8 + 1
                    }
                };
                assert_eq!(
                    seq.rc(),
                    rc_expected,
                    "RC mismatch on line ${:03X} cycle {}",
                    line,
                    cycle
                );

                let vc_expected: u16 = if line < 0x30 {
                    0
                } else if line > 0xF7 {
                    1000
                } else {
                    let r = (line - 0x30) / 8;
                    let k = (line - 0x30) % 8;
                    if cycle < 14 {
                        if k == 0 { 40 * r } else { 40 * (r + 1) }
                    } else if cycle <= 54 {
                        40 * r + u16::from(cycle - 14)
                    } else {
                        40 * (r + 1)
                    }
                };
                assert_eq!(
                    seq.vc(),
                    vc_expected,
                    "VC mismatch on line ${:03X} cycle {}",
                    line,
                    cycle
                );
            }
        }

        assert_eq!(seq.vcbase, 1000, "25 rows x 40 after a full frame");
    }

    #[test]
    fn frame_statistics_c_accesses_and_stolen_cycles() {
        let mut seq = Sequencer::new();
        let mut c_accesses = 0usize;
        let mut ba_low_cycles = 0usize;
        for line in 0..312u16 {
            for cycle in 0..63u8 {
                seq.tick(cycle, line, 0, true);
                if seq.is_c_access_cycle() {
                    c_accesses += 1;
                }
                if !seq.ba() {
                    ba_low_cycles += 1;
                }
            }
        }
        assert_eq!(c_accesses, 1000, "25 bad lines x 40 c-accesses");
        assert_eq!(ba_low_cycles, 1075, "25 bad lines x 43 cycles ($0C..=$36)");
    }

    #[test]
    fn ntsc_6567r8_65_cycle_lines_still_model_the_same_rows() {
        // The sequencer's rules are cycle-slot based, so a 65-cycle /
        // 263-line frame must work identically.
        let mut seq = Sequencer::new();
        for line in 0..263u16 {
            for cycle in 0..65u8 {
                seq.tick(cycle, line, 0, true);
                if cycle >= 55 {
                    assert!(seq.ba(), "line ${:X} cycle {}", line, cycle);
                    assert!(!seq.is_c_access_cycle());
                }
            }
        }
        assert_eq!(seq.vcbase, 1000);
        assert!(!seq.is_display_state());
        assert_eq!(seq.rc(), 7);
    }

    #[test]
    fn idle_to_bad_line_clears_vmli() {
        let mut seq = Sequencer::new();
        prime_full_row(&mut seq); // realistic: idle, RC=7, VCBASE=40, VMLI=40

        for cycle in 0..=14 {
            seq.tick(cycle, 0x38, 0, true); // line $38: bad line entered from IDLE
        }
        assert_eq!(seq.rc(), 0);
        assert!(seq.is_display_state());
        assert_eq!(seq.vc(), 40, "VC must reload from VCBASE");
        assert_eq!(
            seq.vmli(),
            0,
            "VMLI must be cleared at cycle 14 (currently stale at 40)"
        );
    }

    #[test]
    fn realistic_frame_fetching() {
        // What a real fetch unit does every frame: store fetched data at VMLI
        // on every c-access cycle. This is already broken at line $38 — the
        // second row of the FIRST frame — because VMLI was never reset.
        let mut seq = Sequencer::new();
        for line in 0..312u16 {
            for cycle in 0..63u8 {
                if (15..=54).contains(&cycle) && seq.is_bad_line() {
                    assert!(
                        seq.vmli() < 40,
                        "line ${:03X} cycle {}: VMLI={} — store_c_access would \
                     index out of bounds (BUG 1)",
                        line,
                        cycle,
                        seq.vmli()
                    );
                    seq.store_c_access(line as u8, seq.vmli());
                }
                seq.tick(cycle, line, 0, true);
            }
        }
        assert_eq!(seq.vcbase, 1000);
    }

    #[test]
    fn frame_2_first_bad_line_uses_correct_vc() {
        let mut seq = Sequencer::new();
        run_pal_frame(&mut seq, true); // frame 1 works (fresh counters)
        assert_eq!(seq.vcbase, 1000);

        // Actually run line 0 so line-0 reset occurs!
        run_scanline(&mut seq, 0, 0, true);

        // Frame 2, line $30 (idle -> bad line). VCBASE was reset to 0 on
        // line 0, so VC must restart at 0 — not at frame 1's leftover 1000.
        for cycle in 0..=14 {
            seq.tick(cycle, 0x30, 0, true);
        }
        assert_eq!(
            seq.vc(),
            0,
            "VC must restart from VCBASE=0, got {}",
            seq.vc()
        );
        assert_eq!(seq.vmli(), 0, "VMLI must be 0");
    }

    #[test]
    fn rc_reset_uses_correct_condition() {
        // FLD: the condition holds at cycle 12 (BA drops), but software
        // breaks the YSCROLL match during cycle 13 — before the cycle-14
        // evaluation — so RC must NOT be reset.
        let mut seq = Sequencer::new();
        prime_full_row(&mut seq); // idle, RC=7
        for cycle in 0..=12 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert!(
            seq.is_bad_line(),
            "BA is already low (condition held at 12)"
        );
        seq.tick(13, 0x38, 7, true); // YSCROLL rewritten mid-takeover
        seq.tick(14, 0x38, 7, true);
        assert_eq!(
            seq.rc(),
            7,
            "RC must not reset — the condition is gone at 14"
        );
        assert!(!seq.is_display_state(), "the video logic must stay idle");
    }

    #[test]
    fn store_c_access_masks_color_nibble() {
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0);
        for cycle in 0..15 {
            seq.tick(cycle, 0x38, 0, true);
        }
        seq.store_c_access(0x41, 0xFF);
        assert_eq!(seq.current_char_and_color(), (0x41, 0x0F));
    }

    // TODO:
    // =====================================================================
    // HARDWARE-ACCURACY TESTS (currently ignored)
    //
    // Your spec (and impl) latch DEN for the whole frame from line $30. Real
    // hardware latches DEN *per raster line*: "the DEN bit was set during the
    // raster line in which the Bad Line Condition is detected". The models
    // only agree while DEN is permanently on. Enable these when you switch
    // to the per-line latch — or delete them if the per-frame model is a
    // deliberate simplification.
    // =====================================================================

    #[ignore = "needs per-line DEN latch (real hardware behaviour)"]
    #[test]
    fn hw_den_enabled_mid_frame_starts_display_at_next_candidate_line() {
        let mut seq = Sequencer::new();
        // DEN off through line $36 — in particular during ALL of line $30 —
        // and enabled from line $37 on.
        for line in 0..=0x36 {
            run_scanline(&mut seq, line, 0, false);
        }
        run_scanline(&mut seq, 0x37, 0, true);
        for cycle in 0..=12 {
            seq.tick(cycle, 0x38, 0, true);
        }
        assert!(
            seq.is_bad_line(),
            "DEN set during line $37/$38 makes $38 a bad line on real hardware"
        );
    }

    #[ignore = "needs per-line DEN latch (real hardware behaviour)"]
    #[test]
    fn hw_den_cleared_mid_frame_blanks_the_display_after_the_current_row() {
        let mut seq = Sequencer::new();
        prime_full_row(&mut seq); // row 0 was displayed with DEN on
        for line in 0x38..=0x47 {
            run_scanline(&mut seq, line, 0, false); // DEN cleared
        }
        assert!(
            !seq.is_display_state(),
            "no further bad lines after DEN is cleared -> display stays off"
        );
        assert_eq!(seq.vcbase, 40, "VCBASE frozen at the last completed row");
    }

    #[test]
    fn yscroll_is_masked_to_its_low_3_bits() {
        // Pins the `& 7` fix: only bits 0-2 of the yscroll argument participate
        // in the Bad Line match, so raw $D011-style values are safe to pass.
        let mut seq = Sequencer::new();
        prime_sequencer_to_display_area(&mut seq, 0); // line $38 & 7 == 0

        seq.tick(12, 0x38, 0x08, true); // bit 3 set, bits 0-2 = 0
        assert!(seq.is_bad_line(), "0x08 & 7 == 0 must match line $38");

        seq.tick(12, 0x38, 0x10, true); // bit 4 (the DEN position!) set, bits 0-2 = 0
        assert!(seq.is_bad_line(), "0x10 & 7 == 0 must match line $38");

        seq.tick(12, 0x38, 0x1B, true); // bits 0-2 = 3
        assert!(!seq.is_bad_line(), "0x1B & 7 == 3 must not match line $38");
    }
}
