pub struct BorderUnit {
    /// Controls the actual video output.
    /// If true: border color ($D020) is displayed.
    /// If false: screen graphics / character pixels are displayed.
    main_border: bool,

    /// Tracks whether the beam is in the upper or lower border region.
    vertical_border: bool,
}

impl BorderUnit {
    pub fn new() -> Self {
        Self {
            main_border: true,
            vertical_border: true,
        }
    }

    pub fn tick(&mut self, cycle: u8, line: u16, rsel: bool, csel: bool, den: bool) {
        let (top, bottom) = if rsel { (51, 251) } else { (55, 247) };
        let (left, right) = if csel { (16, 56) } else { (17, 55) };
        if cycle == 62 || cycle == left {
            if line == bottom {
                self.vertical_border = true;
            }
            if line == top && den {
                self.vertical_border = false;
            }
        }

        if cycle == left && !self.vertical_border {
            self.main_border = false;
        }
        if cycle == right {
            self.main_border = true;
        }
    }

    pub fn is_border_active(&self) -> bool {
        self.main_border
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL / RESET STATE
    //    * main_border = true, vertical_border = true.
    //    * is_border_active() = true.
    //
    // 2. STANDARD 40-COLUMN / 25-ROW WINDOW (RSEL=1, CSEL=1, DEN=1)
    //    * Lines 0..=50: Entire lines are in border (cycles 0..=62).
    //    * Lines 51..=250 (200 display lines):
    //      - Cycles 0..=15: Border active.
    //      - Cycles 16..=55: Graphics active (is_border_active == false).
    //      - Cycles 56..=62: Border active.
    //    * Lines 251..=311: Entire lines are in border (cycles 0..=62).
    //
    // 3. 38-COLUMN WINDOW (CSEL=0)
    //    * Shrinks display window by 1 column on each side (38 cycles).
    //    * Cycles 16 and 55 are now BORDER.
    //    * Graphics active only on cycles 17..=54.
    //
    // 4. 24-ROW WINDOW (RSEL=0)
    //    * Shrinks display window by 4 lines on top and 4 lines on bottom (192 lines).
    //    * Lines 51..=54 and 247..=250 are now BORDER.
    //    * Graphics active only on lines 55..=246.
    //
    // 5. DISPLAY ENABLE (DEN=0) BLANKING
    //    * If DEN=0 when reaching the top edge (line 51), vertical border NEVER opens.
    //    * The entire screen remains 100% border for all lines and cycles.
    //
    // 6. DEMOSCENE TRICKS (HYPERSCREEN / OPENING BORDERS)
    //    * Horizontal: Switching CSEL from 1 to 0 at cycle 56 skips the right-edge
    //      comparison, leaving the right border open.
    //    * Vertical: Switching RSEL from 1 to 0 before line 251 skips the bottom-edge
    //      comparison, leaving the bottom border open.
    // =====================================================================

    fn run_scanline(b: &mut BorderUnit, line: u16, rsel: bool, csel: bool, den: bool) {
        for cycle in 0..63 {
            b.tick(cycle, line, rsel, csel, den);
        }
    }

    /// Final cycle of a raster line. The doc's rules 2/3 say Y is compared
    /// in "cycle 63" — the 1-based VIC convention. In this code's 0-based
    /// numbering (cycles 0..=62) that is cycle 62.
    const LAST_CYCLE: u8 = 62;

    fn run_frame(b: &mut BorderUnit, rsel: bool, csel: bool, den: bool) {
        for line in 0..312 {
            run_scanline(b, line, rsel, csel, den);
        }
    }

    /// Steady-state border map for constant registers (doc rules 1-6).
    fn steady_border(line: u16, cycle: u8, rsel: bool, csel: bool, den: bool) -> bool {
        if !den {
            return true;
        }
        let (top, bottom) = if rsel { (51, 251) } else { (55, 247) };
        let (left, right) = if csel { (16, 56) } else { (17, 55) };
        line < top || line >= bottom || cycle < left || cycle >= right
    }

    // ---------------------------------------------------------------------
    // 1. Initial State
    // ---------------------------------------------------------------------

    #[test]
    fn new_border_unit_starts_with_border_active() {
        let b = BorderUnit::new();
        assert!(b.is_border_active());
    }

    // ---------------------------------------------------------------------
    // 2. Standard 40x25 Window (Defaults: RSEL=1, CSEL=1, DEN=1)
    // ---------------------------------------------------------------------

    #[test]
    fn top_border_lines_are_solid_border() {
        let mut b = BorderUnit::new();

        // Lines 0 through 50 must be 100% border on all cycles
        for line in 0..=50 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                assert!(
                    b.is_border_active(),
                    "Top border must be active on line {}, cycle {}",
                    line,
                    cycle
                );
            }
        }
    }

    #[test]
    fn line_51_first_active_line_opens_at_cycle_16_and_closes_at_cycle_56() {
        let mut b = BorderUnit::new();

        // Run through top border up to line 50
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }

        // Line 51:
        // Cycles 0..=15: Left border
        for cycle in 0..=15 {
            b.tick(cycle, 51, true, true, true);
            assert!(
                b.is_border_active(),
                "Left border active at cycle {}",
                cycle
            );
        }

        // Cycle 16: Border opens!
        b.tick(16, 51, true, true, true);
        assert!(!b.is_border_active(), "Border must open at cycle 16");

        // Cycles 17..=55: Active graphics window
        for cycle in 17..=55 {
            b.tick(cycle, 51, true, true, true);
            assert!(!b.is_border_active(), "Graphics active at cycle {}", cycle);
        }

        // Cycle 56: Right border closes!
        b.tick(56, 51, true, true, true);
        assert!(b.is_border_active(), "Right border must close at cycle 56");

        // Cycles 57..=62: Right border remains active
        for cycle in 57..63 {
            b.tick(cycle, 51, true, true, true);
            assert!(b.is_border_active());
        }
    }

    #[test]
    fn all_200_display_lines_have_exact_same_horizontal_window() {
        let mut b = BorderUnit::new();

        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }

        // Lines 51 through 250 (200 lines total)
        for line in 51..=250 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                let expected_border = cycle < 16 || cycle >= 56;
                assert_eq!(
                    b.is_border_active(),
                    expected_border,
                    "Line {} cycle {} border state mismatch",
                    line,
                    cycle
                );
            }
        }
    }

    #[test]
    fn bottom_border_closes_at_line_251_and_stays_solid() {
        let mut b = BorderUnit::new();

        for line in 0..=250 {
            run_scanline(&mut b, line, true, true, true);
        }

        // Line 251 through 311 (bottom border)
        for line in 251..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                assert!(
                    b.is_border_active(),
                    "Bottom border must be solid on line {}, cycle {}",
                    line,
                    cycle
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // 3. 38-Column Mode (CSEL = 0)
    // ---------------------------------------------------------------------

    #[test]
    fn csel_0_shrinks_screen_to_38_columns() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, false, true); // CSEL = false
        }

        let line = 51;
        // In 38-col mode, cycle 16 is STILL border:
        for cycle in 0..=16 {
            b.tick(cycle, line, true, false, true);
            assert!(
                b.is_border_active(),
                "Cycle {} must be border in 38-col mode",
                cycle
            );
        }

        // Cycle 17: Border opens (1 cycle later than 40-col)!
        b.tick(17, line, true, false, true);
        assert!(!b.is_border_active(), "38-col window must open at cycle 17");

        // Active graphics through cycle 54 (38 columns total: 17..=54)
        for cycle in 18..=54 {
            b.tick(cycle, line, true, false, true);
            assert!(!b.is_border_active());
        }

        // Cycle 55: Border closes (1 cycle earlier than 40-col)!
        b.tick(55, line, true, false, true);
        assert!(b.is_border_active(), "38-col window must close at cycle 55");
    }

    // ---------------------------------------------------------------------
    // 4. 24-Row Mode (RSEL = 0)
    // ---------------------------------------------------------------------

    #[test]
    fn rsel_0_shrinks_screen_to_24_rows() {
        let mut b = BorderUnit::new();

        // In 24-row mode, lines 51..=54 remain solid border:
        for line in 0..=54 {
            for cycle in 0..63 {
                b.tick(cycle, line, false, true, true); // RSEL = false
                assert!(
                    b.is_border_active(),
                    "Lines 0..54 must be border in 24-row mode (line {}, cycle {})",
                    line,
                    cycle
                );
            }
        }

        // Line 55: First active display line!
        for cycle in 0..16 {
            b.tick(cycle, 55, false, true, true);
            assert!(b.is_border_active());
        }
        b.tick(16, 55, false, true, true);
        assert!(
            !b.is_border_active(),
            "Line 55 must open at cycle 16 in 24-row mode"
        );

        // Run through active rows up to line 246
        for line in 56..=246 {
            run_scanline(&mut b, line, false, true, true);
        }

        // Line 247: Bottom border closes (4 lines earlier than 25-row)!
        for cycle in 0..63 {
            b.tick(cycle, 247, false, true, true);
            assert!(
                b.is_border_active(),
                "Line 247 must be solid border in 24-row mode"
            );
        }
    }

    // ---------------------------------------------------------------------
    // 5. DEN Blanking (Screen Disabled)
    // ---------------------------------------------------------------------

    #[test]
    fn den_false_blanks_entire_frame() {
        let mut b = BorderUnit::new();

        // Run a full frame with DEN = false
        for line in 0..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, false); // DEN = false!
                assert!(
                    b.is_border_active(),
                    "Screen must be 100% border when DEN=false (line {}, cycle {})",
                    line,
                    cycle
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // 6. Frame Rollover
    // ---------------------------------------------------------------------

    #[test]
    fn frame_rollover_resets_for_next_frame() {
        let mut b = BorderUnit::new();

        // Run entire Frame 1
        for line in 0..312 {
            run_scanline(&mut b, line, true, true, true);
        }

        // Frame 2, Line 0..50 must still be solid border
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
            assert!(b.is_border_active());
        }

        // Frame 2, Line 51 must cleanly open again at cycle 16
        for cycle in 0..=15 {
            b.tick(cycle, 51, true, true, true);
            assert!(b.is_border_active());
        }
        b.tick(16, 51, true, true, true);
        assert!(
            !b.is_border_active(),
            "Frame 2 must cleanly open graphics window"
        );
    }

    // ---------------------------------------------------------------------
    // 7. Demoscene Exploits (Hyperscreen / Border Removal)
    // ---------------------------------------------------------------------

    #[test]
    fn horizontal_hyperscreen_csel_flip_at_cycle_56_keeps_border_open() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }

        let line = 51;
        for cycle in 0..16 {
            b.tick(cycle, line, true, true, true);
        }
        b.tick(16, line, true, true, true); // Opens border

        // Run up to cycle 55 with CSEL = 1 (40 cols)
        for cycle in 17..=55 {
            b.tick(cycle, line, true, true, true);
            assert!(!b.is_border_active());
        }

        // HYPERSCREEN TRICK: Exactly on cycle 56, software writes CSEL = 0 (38 cols)!
        // Because CSEL is now 0, the right-edge comparator looks for cycle 55 (which passed).
        // It misses the match, so main_border is NEVER set back to true!
        b.tick(56, line, true, false, true); // CSEL switched to false!
        assert!(
            !b.is_border_active(),
            "Right border must remain OPEN due to CSEL switch on cycle 56"
        );

        for cycle in 57..63 {
            b.tick(cycle, line, true, false, true);
            assert!(
                !b.is_border_active(),
                "Right border must stay open through the end of the line"
            );
        }
    }

    #[test]
    fn vertical_hyperscreen_rsel_flip_keeps_bottom_border_open() {
        let mut b = BorderUnit::new();

        // Run lines 0..=247 in 25-row mode (line 247 passes safely)
        for line in 0..=247 {
            run_scanline(&mut b, line, true, true, true);
        }

        // Between lines 248..=250, software switches to 24-row mode (RSEL = 0)!
        // In 24-row mode, the bottom edge is line 247 (which has now already passed).
        // In 25-row mode, the bottom edge was line 251.
        // Because RSEL=0 now, line 251 comparator NEVER matches!
        for line in 248..=250 {
            run_scanline(&mut b, line, false, true, true); // RSEL switched to false
        }

        // Line 251: Normally the bottom border closes here.
        // But with the trick, vertical_border was NEVER set! Graphics open on line 251!
        for cycle in 0..16 {
            b.tick(cycle, 251, false, true, true);
        }
        b.tick(16, 251, false, true, true);
        assert!(
            !b.is_border_active(),
            "Bottom border must remain open on line 251 due to RSEL flip"
        );
    }

    /// Doc-literal reference model used for differential testing: identical
    /// to BorderUnit except that the Y comparisons only run at the check
    /// cycles (final cycle of the line + left comparison value).
    struct DocBorderUnit {
        main: bool,
        vertical: bool,
    }

    impl DocBorderUnit {
        fn new() -> Self {
            Self {
                main: true,
                vertical: true,
            }
        }

        fn tick(&mut self, cycle: u8, line: u16, rsel: bool, csel: bool, den: bool) {
            let (top, bottom) = if rsel { (51, 251) } else { (55, 247) };
            let (left, right) = if csel { (16, 56) } else { (17, 55) };

            if cycle == LAST_CYCLE || cycle == left {
                if line == bottom {
                    self.vertical = true;
                }
                if line == top && den {
                    self.vertical = false;
                }
            }
            if cycle == left && !self.vertical {
                self.main = false;
            }
            if cycle == right {
                self.main = true;
            }
        }

        fn is_border_active(&self) -> bool {
            self.main
        }
    }

    /// Tiny deterministic PRNG (no external crates).
    struct Lcg(u64);
    impl Lcg {
        fn coin(&mut self, percent: u32) -> bool {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 33) as u32) % 100 < percent
        }
    }

    // ---------------------------------------------------------------------
    // 8. Full-frame invariants
    // ---------------------------------------------------------------------

    #[test]
    fn full_frame_matches_steady_state_model_for_all_register_combinations() {
        for rsel in [true, false] {
            for csel in [true, false] {
                for den in [true, false] {
                    let mut b = BorderUnit::new();
                    // Two consecutive frames: frame 2 proves no leaked state.
                    for frame in 0..2 {
                        for line in 0..312 {
                            for cycle in 0..63 {
                                b.tick(cycle, line, rsel, csel, den);
                                assert_eq!(
                                    b.is_border_active(),
                                    steady_border(line, cycle, rsel, csel, den),
                                    "frame={} rsel={} csel={} den={} line={} cycle={}",
                                    frame,
                                    rsel,
                                    csel,
                                    den,
                                    line,
                                    cycle
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn rsel0_and_csel0_combine_to_a_38x24_window() {
        let mut b = BorderUnit::new();
        for line in 0..=54 {
            run_scanline(&mut b, line, false, false, true);
        }

        // First display line is 55, opening one cycle late (17)...
        for cycle in 0..17 {
            b.tick(cycle, 55, false, false, true);
            assert!(b.is_border_active(), "cycle {}", cycle);
        }
        b.tick(17, 55, false, false, true);
        assert!(!b.is_border_active());
        for cycle in 18..=54 {
            b.tick(cycle, 55, false, false, true);
            assert!(!b.is_border_active(), "cycle {}", cycle);
        }
        // ...and closing one cycle early (55).
        b.tick(55, 55, false, false, true);
        assert!(b.is_border_active());

        for line in 56..=246 {
            run_scanline(&mut b, line, false, false, true);
        }
        // Last display line is 246; 247 is solid border.
        for cycle in 0..63 {
            b.tick(cycle, 247, false, false, true);
            assert!(b.is_border_active(), "cycle {}", cycle);
        }
    }

    #[test]
    fn starting_mid_frame_is_border_until_the_next_top_edge() {
        let mut b = BorderUnit::new();
        // "Power on" in the middle of the display area: the vertical flip
        // is still set, so everything is border until the next line 51.
        for line in 200..=250 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                assert!(b.is_border_active(), "line {}", line);
            }
        }
        for line in 251..312 {
            run_scanline(&mut b, line, true, true, true);
        }
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..16 {
            b.tick(cycle, 51, true, true, true);
        }
        b.tick(16, 51, true, true, true);
        assert!(!b.is_border_active());
    }

    // ---------------------------------------------------------------------
    // 9. DEN sampling
    // ---------------------------------------------------------------------

    #[test]
    fn den_must_be_set_exactly_on_the_top_comparison_line() {
        for (rsel, top) in [(true, 51u16), (false, 55u16)] {
            for scenario in 0..3 {
                let mut b = BorderUnit::new();
                for line in 0..312 {
                    for cycle in 0..63 {
                        // (a) DEN=0 only on the top line -> blank frame.
                        // (b) DEN=1 only on the line before it -> blank.
                        // (c) DEN=1 only from the line after it -> blank (too late).
                        let den = match scenario {
                            0 => line != top,
                            1 => line + 1 == top,
                            _ => line > top,
                        };
                        b.tick(cycle, line, rsel, true, den);
                        assert!(
                            b.is_border_active(),
                            "scenario={} rsel={} line={} cycle={}",
                            scenario,
                            rsel,
                            line,
                            cycle
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn den_cleared_after_the_screen_opened_does_not_close_it() {
        let mut b = BorderUnit::new();
        run_frame(&mut b, true, true, true);
        // Frame 2: DEN=1 only on line 51 — enough for the whole frame.
        for line in 0..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, line == 51);
                let expected = line < 51 || line > 250 || cycle < 16 || cycle >= 56;
                assert_eq!(
                    b.is_border_active(),
                    expected,
                    "line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
        // Frame 3: DEN never set -> the top edge can't open -> blank.
        for line in 0..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, false);
                assert!(b.is_border_active(), "line {} cycle {}", line, cycle);
            }
        }
    }

    #[test]
    fn den_set_in_the_final_cycle_of_the_top_line_opens_the_next_line() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, false);
        }
        // Line 51 with DEN=1 only during the very last cycle (the doc's
        // "cycle 63" check — cycle 62 here).
        for cycle in 0..62 {
            b.tick(cycle, 51, true, true, false);
        }
        b.tick(LAST_CYCLE, 51, true, true, true);
        // Line 51 itself stays border: cycle 16 passed with the vertical
        // flip still set, so the main flip was never reset...
        assert!(b.is_border_active());
        // ...but line 52 opens normally at cycle 16.
        for cycle in 0..16 {
            b.tick(cycle, 52, true, true, true);
        }
        b.tick(16, 52, true, true, true);
        assert!(!b.is_border_active());
    }

    // ---------------------------------------------------------------------
    // 10. Dynamic register changes
    // ---------------------------------------------------------------------

    #[test]
    fn csel_changes_take_effect_at_the_next_comparison() {
        // (a) CSEL 1 -> 0 during a display line: right comparison becomes
        //     55, so the border closes one cycle early.
        let mut b = BorderUnit::new();
        for line in 0..=99 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..63 {
            b.tick(cycle, 100, true, cycle <= 40, true);
            assert_eq!(
                b.is_border_active(),
                cycle < 16 || cycle >= 55,
                "(a) cycle {}",
                cycle
            );
        }

        // (b) CSEL 0 -> 1 during a display line: closes one cycle late (56).
        let mut b = BorderUnit::new();
        for line in 0..=99 {
            run_scanline(&mut b, line, true, false, true);
        }
        for cycle in 0..63 {
            b.tick(cycle, 100, true, cycle > 40, true);
            assert_eq!(
                b.is_border_active(),
                cycle < 17 || cycle >= 56,
                "(b) cycle {}",
                cycle
            );
        }

        // (c) CSEL 1 -> 0 before the left comparison: opens one cycle
        //     late (17) and closes one early (55).
        let mut b = BorderUnit::new();
        for line in 0..=99 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..63 {
            b.tick(cycle, 100, true, cycle <= 10, true);
            assert_eq!(
                b.is_border_active(),
                cycle < 17 || cycle >= 55,
                "(c) cycle {}",
                cycle
            );
        }
    }

    #[test]
    fn csel_toggle_between_the_left_comparisons_skips_rule_6() {
        // The CSEL=1 left comparison is at cycle 16, the CSEL=0 one at 17.
        // CSEL=0 during cycle 16 and CSEL=1 during cycle 17 lets X match
        // neither left comparison value, so rule 6 never fires and the
        // main flip stays set for the whole line.
        //
        // NOTE: this pins the cycle-quantized *model*. On real silicon the
        // two left comparisons (X=24 / X=31) are 7 pixels apart — inside a
        // single cycle — so this trick is probably not reproducible on a
        // real C64. The right-edge variant IS real (9 px gap).
        let mut b = BorderUnit::new();
        for line in 0..=51 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..63 {
            b.tick(cycle, 52, true, cycle != 16, true);
            assert!(b.is_border_active(), "cycle {}", cycle);
        }
        // Next line is normal again — the skip affects one line only.
        for cycle in 0..63 {
            b.tick(cycle, 53, true, true, true);
            assert_eq!(
                b.is_border_active(),
                cycle < 16 || cycle >= 56,
                "cycle {}",
                cycle
            );
        }
    }

    #[test]
    fn open_right_border_carries_into_the_next_line() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }
        // Line 51: the classic right-edge trick (CSEL 1 -> 0 at cycle 56).
        for cycle in 0..56 {
            b.tick(cycle, 51, true, true, true);
        }
        for cycle in 56..63 {
            b.tick(cycle, 51, true, false, true);
        }
        assert!(!b.is_border_active());
        // Line 52 with CSEL restored: nothing re-sets the main flip until
        // the right comparison (cycle 56), so even the *left* border of
        // line 52 is "not border" (background colour on real hardware).
        // This is why demos must repeat the trick on every line.
        for cycle in 0..63 {
            b.tick(cycle, 52, true, true, true);
            assert_eq!(b.is_border_active(), cycle >= 56, "cycle {}", cycle);
        }
    }

    #[test]
    fn rsel_1_to_0_before_the_top_edge_delays_the_opening_to_line_55() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }
        for line in 51..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, false, true, true);
                let expected = line < 55 || line > 246 || cycle < 16 || cycle >= 56;
                assert_eq!(
                    b.is_border_active(),
                    expected,
                    "line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
    }

    #[test]
    fn rsel_0_to_1_early_in_line_51_opens_that_line() {
        // A $D011 write landing before the left comparison (cycle 16) of
        // line 51 is still in time for the top-edge check.
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, false, true, true);
        }
        b.tick(0, 51, false, true, true);
        for cycle in 1..16 {
            b.tick(cycle, 51, true, true, true);
        }
        b.tick(16, 51, true, true, true);
        assert!(!b.is_border_active(), "the RSEL write must open line 51");
    }

    #[test]
    fn rsel_0_to_1_between_the_two_top_lines_blanks_the_frame() {
        // 24-row mode through line 54: line 51's check sees RSEL=0 (top=55,
        // no match). 25-row mode from line 55: line 55's check sees top=51
        // (already passed). Neither top comparison ever fires -> the
        // vertical flip never resets -> the whole frame is border.
        let mut b = BorderUnit::new();
        for line in 0..=54 {
            run_scanline(&mut b, line, false, true, true);
        }
        for line in 55..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                assert!(b.is_border_active(), "line {} cycle {}", line, cycle);
            }
        }
        // The next frame recovers: line 51 is reached with RSEL=1.
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..16 {
            b.tick(cycle, 51, true, true, true);
        }
        b.tick(16, 51, true, true, true);
        assert!(!b.is_border_active());
    }

    #[test]
    fn rsel_1_to_0_mid_frame_shrinks_the_bottom_to_line_247() {
        let mut b = BorderUnit::new();
        for line in 0..=99 {
            run_scanline(&mut b, line, true, true, true);
        }
        for line in 100..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, false, true, true);
                let expected = line < 55 || line > 246 || cycle < 16 || cycle >= 56;
                assert_eq!(
                    b.is_border_active(),
                    expected,
                    "line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
    }

    #[test]
    fn rsel_0_to_1_mid_frame_extends_the_display_to_line_250() {
        let mut b = BorderUnit::new();
        for line in 0..=99 {
            run_scanline(&mut b, line, false, true, true);
        }
        for line in 100..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                let expected = line < 51 || line > 250 || cycle < 16 || cycle >= 56;
                assert_eq!(
                    b.is_border_active(),
                    expected,
                    "line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // 11. Extended vertical tricks
    // ---------------------------------------------------------------------

    #[test]
    fn open_bottom_border_removes_the_next_frames_top_border() {
        // Classic full-screen trick: open the bottom border and keep
        // RSEL=0. The vertical flip stays open through the vertical blank,
        // so in the next frame rule 6 fires on every line 0..=50 and even
        // the top border shows "graphics".
        let mut b = BorderUnit::new();
        for line in 0..=247 {
            run_scanline(&mut b, line, true, true, true);
        } // pass 247 with RSEL=1
        for line in 248..312 {
            run_scanline(&mut b, line, false, true, true);
        } // dodge 251 with RSEL=0

        for line in 0..=246 {
            for cycle in 0..63 {
                b.tick(cycle, line, false, true, true);
                assert_eq!(
                    b.is_border_active(),
                    cycle < 16 || cycle >= 56,
                    "frame 2 line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
        // The RSEL=0 bottom comparison (247) finally closes it.
        for cycle in 0..63 {
            b.tick(cycle, 247, false, true, true);
            assert!(b.is_border_active(), "cycle {}", cycle);
        }
    }

    #[test]
    fn alternating_rsel_keeps_the_entire_frame_open() {
        // Dodge *both* bottom comparisons every frame (247 for RSEL=0,
        // 251 for RSEL=1). This is how demos hold the whole screen open
        // indefinitely.
        let mut b = BorderUnit::new();
        for line in 0..=247 {
            run_scanline(&mut b, line, true, true, true);
        }
        for line in 248..312 {
            run_scanline(&mut b, line, false, true, true);
        }

        for frame in 0..2 {
            for line in 0..312 {
                let rsel = line >= 244 && line <= 250; // RSEL=1 while passing 247
                for cycle in 0..63 {
                    b.tick(cycle, line, rsel, true, true);
                    assert_eq!(
                        b.is_border_active(),
                        cycle < 16 || cycle >= 56,
                        "frame {} line {} cycle {}",
                        frame,
                        line,
                        cycle
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // 12. Differential testing against the doc model
    // ---------------------------------------------------------------------

    #[test]
    fn differential_against_doc_model_with_per_line_constant_registers() {
        // Registers are held constant within each line. (With mid-line
        // changes the current implementation diverges from the doc — that
        // divergence is pinned by the #[ignore]d tests in section 13.)
        let mut rng = Lcg(0xC0FF_64);
        let mut b = BorderUnit::new();
        let mut doc = DocBorderUnit::new();
        for frame in 0..3 {
            for line in 0..312 {
                let rsel = rng.coin(20);
                let csel = rng.coin(20);
                let den = rng.coin(70);
                for cycle in 0..63 {
                    b.tick(cycle, line, rsel, csel, den);
                    doc.tick(cycle, line, rsel, csel, den);
                    assert_eq!(
                        b.is_border_active(),
                        doc.is_border_active(),
                        "frame {} line {} cycle {} (rsel={} csel={} den={})",
                        frame,
                        line,
                        cycle,
                        rsel,
                        csel,
                        den
                    );
                }
            }
        }
    }

    #[test]
    fn den_pulse_away_from_the_check_cycles_must_be_ignored() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, false);
        }
        // One-cycle DEN pulse at cycle 5 of line 51: both check cycles
        // (16 and 62) see DEN=0, so the frame must stay blank.
        for cycle in 0..63 {
            b.tick(cycle, 51, true, true, cycle == 5);
        }
        for line in 52..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, false);
                assert!(b.is_border_active(), "line {} cycle {}", line, cycle);
            }
        }
    }

    #[test]
    fn den_pulse_at_cycle_16_is_ignored_when_csel_is_0() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, false, false);
        }
        for cycle in 0..63 {
            b.tick(cycle, 51, true, false, cycle == 16);
        }
        for line in 52..312 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, false, false);
                assert!(b.is_border_active(), "line {} cycle {}", line, cycle);
            }
        }
    }

    #[test]
    fn brief_rsel0_pulse_on_line_247_must_be_ignored() {
        let mut b = BorderUnit::new();
        for line in 0..=246 {
            run_scanline(&mut b, line, true, true, true);
        }
        for cycle in 0..63 {
            b.tick(cycle, 247, !(cycle >= 20 && cycle <= 30), true, true);
        }
        // Lines 248..=250 must still display normally...
        for line in 248..=250 {
            for cycle in 0..63 {
                b.tick(cycle, line, true, true, true);
                assert_eq!(
                    b.is_border_active(),
                    cycle < 16 || cycle >= 56,
                    "line {} cycle {}",
                    line,
                    cycle
                );
            }
        }
        // ...and the bottom border still closes at line 251 as usual.
        for cycle in 0..63 {
            b.tick(cycle, 251, true, true, true);
            assert!(b.is_border_active());
        }
    }

    #[test]
    fn rsel_flip_early_in_line_251_opens_the_bottom_border() {
        let mut b = BorderUnit::new();
        for line in 0..=250 {
            run_scanline(&mut b, line, true, true, true);
        }
        // RSEL=1 during cycle 0, then the write clears it — well before
        // the left comparison (cycle 16).
        b.tick(0, 251, true, true, true);
        for cycle in 1..16 {
            b.tick(cycle, 251, false, true, true);
            assert!(b.is_border_active(), "cycle {}", cycle);
        }
        b.tick(16, 251, false, true, true);
        assert!(!b.is_border_active(), "bottom border must open at cycle 16");
        // The border stays open into the lines below.
        for cycle in 0..16 {
            b.tick(cycle, 252, false, true, true);
        }
        b.tick(16, 252, false, true, true);
        assert!(!b.is_border_active());
    }

    #[test]
    fn den_high_only_at_the_left_check_cycle_is_sufficient() {
        let mut b = BorderUnit::new();
        for line in 0..=50 {
            run_scanline(&mut b, line, true, true, false);
        }
        for cycle in 0..63 {
            b.tick(cycle, 51, true, true, cycle == 16);
        }
        // Vertical flip was reset during the cycle-16 check; line 52 must
        // open at cycle 16 even though DEN=0 everywhere afterwards.
        for cycle in 0..16 {
            b.tick(cycle, 52, true, true, false);
        }
        b.tick(16, 52, true, true, false);
        assert!(!b.is_border_active());
    }
}
