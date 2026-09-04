use crate::VideoStandard;

pub struct RasterBeam {
    cycle: u8,
    line: u16,
    standard: VideoStandard,
}

impl RasterBeam {
    pub fn new(standard: VideoStandard) -> Self {
        Self {
            cycle: 0,
            line: 0,
            standard: standard,
        }
    }

    /// Advances the beam by 1 bus cycle.
    ///
    /// Returns `true` on the very first cycle of a new scan line (cycle == 0).
    pub fn tick(&mut self) -> bool {
        self.cycle += 1;

        if self.cycle >= self.standard.cycles_per_line() {
            self.cycle = 0;
            self.line += 1;

            if self.line >= self.standard.total_lines() {
                self.line = 0;
            }

            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn cycle(&self) -> u8 {
        self.cycle
    }

    #[inline(always)]
    pub fn line(&self) -> u16 {
        self.line
    }

    #[inline(always)]
    pub fn standard(&self) -> VideoStandard {
        self.standard.clone()
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.line = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL STATE
    //    * cycle := 0
    //    * line  := 0
    //
    // 2. CYCLE COUNTING (HORIZONTAL TIMING)
    //    * cycle increments by 1 on every tick().
    //    * PAL:  cycle ranges 0..=62 (63 cycles total).
    //            tick 63 wraps cycle -> 0, increments line -> 1, returns true.
    //    * NTSC: cycle ranges 0..=64 (65 cycles total).
    //            tick 65 wraps cycle -> 0, increments line -> 1, returns true.
    //
    // 3. LINE COUNTING (VERTICAL TIMING)
    //    * line increments by 1 whenever cycle wraps to 0.
    //    * PAL:  line ranges 0..=311 (312 lines total).
    //            wrapping from line 311 cycle 62 wraps line -> 0.
    //    * NTSC: line ranges 0..=262 (263 lines total).
    //            wrapping from line 262 cycle 64 wraps line -> 0.
    //
    // 4. KERNAL PAL BOOT DETECTION
    //    * In PAL mode, line 311 MUST be reached (used by Kernal CINT at $FF5B).
    //    * In NTSC mode, line 311 is NEVER reached (max line is 262).
    //
    // 5. FRAME DURATION
    //    * Exactly 19,656 ticks per PAL frame (63 * 312).
    //    * Exactly 17,095 ticks per NTSC frame (65 * 263).
    // =====================================================================

    // ---------------------------------------------------------------------
    // 1. Reset / Initial State
    // ---------------------------------------------------------------------

    #[test]
    fn new_beam_starts_at_cycle_zero_and_line_zero() {
        let pal = RasterBeam::new(VideoStandard::Pal);
        assert_eq!(pal.cycle(), 0);
        assert_eq!(pal.line(), 0);
        assert_eq!(pal.standard(), VideoStandard::Pal);

        let ntsc = RasterBeam::new(VideoStandard::Ntsc);
        assert_eq!(ntsc.cycle(), 0);
        assert_eq!(ntsc.line(), 0);
        assert_eq!(ntsc.standard(), VideoStandard::Ntsc);
    }

    #[test]
    fn reset_returns_beam_to_origin() {
        let mut beam = RasterBeam::new(VideoStandard::Pal);
        for _ in 0..100 {
            beam.tick();
        }
        assert_ne!(beam.cycle(), 0);
        assert_ne!(beam.line(), 0);

        beam.reset();
        assert_eq!(beam.cycle(), 0);
        assert_eq!(beam.line(), 0);
    }

    // ---------------------------------------------------------------------
    // 2. Horizontal Cycle Counting
    // ---------------------------------------------------------------------

    #[test]
    fn cycle_increments_by_one_per_tick() {
        let mut beam = RasterBeam::new(VideoStandard::Pal);
        for expected_cycle in 1..=62 {
            let is_new_line = beam.tick();
            assert_eq!(beam.cycle(), expected_cycle);
            assert_eq!(beam.line(), 0);
            assert!(!is_new_line, "Must not signal new line mid-scanline");
        }
    }

    #[test]
    fn pal_wraps_cycle_at_63_ticks() {
        let mut beam = RasterBeam::new(VideoStandard::Pal);

        // Advance through cycles 0..=61
        for _ in 0..62 {
            beam.tick();
        }
        assert_eq!(beam.cycle(), 62);
        assert_eq!(beam.line(), 0);

        // 63rd tick wraps to next line
        let new_line = beam.tick();
        assert!(new_line, "Must return true on line advance");
        assert_eq!(beam.cycle(), 0);
        assert_eq!(beam.line(), 1);
    }

    #[test]
    fn ntsc_wraps_cycle_at_65_ticks() {
        let mut beam = RasterBeam::new(VideoStandard::Ntsc);

        // Advance through cycles 0..=63
        for _ in 0..64 {
            beam.tick();
        }
        assert_eq!(beam.cycle(), 64);
        assert_eq!(beam.line(), 0);

        // 65th tick wraps to next line
        let new_line = beam.tick();
        assert!(new_line, "Must return true on line advance");
        assert_eq!(beam.cycle(), 0);
        assert_eq!(beam.line(), 1);
    }

    // ---------------------------------------------------------------------
    // 3. Line Counting & Frame Rollover
    // ---------------------------------------------------------------------

    #[test]
    fn pal_counts_up_to_line_311_then_rolls_over_to_zero() {
        let mut beam = RasterBeam::new(VideoStandard::Pal);

        // Step across 311 full lines (311 * 63 ticks)
        for expected_line in 0..311 {
            assert_eq!(beam.line(), expected_line);
            for _ in 0..63 {
                beam.tick();
            }
        }

        // We are now at line 311, cycle 0
        assert_eq!(beam.line(), 311);
        assert_eq!(beam.cycle(), 0);

        // Tick through line 311 (cycles 1..=62)
        for _ in 0..62 {
            beam.tick();
        }
        assert_eq!(beam.line(), 311);
        assert_eq!(beam.cycle(), 62);

        // Final tick of the frame: must wrap line back to 0!
        let new_line = beam.tick();
        assert!(new_line);
        assert_eq!(beam.line(), 0, "Line must wrap from 311 to 0 on PAL");
        assert_eq!(beam.cycle(), 0);
    }

    #[test]
    fn ntsc_counts_up_to_line_262_then_rolls_over_to_zero() {
        let mut beam = RasterBeam::new(VideoStandard::Ntsc);

        // Step across 262 full lines (262 * 65 ticks)
        for expected_line in 0..262 {
            assert_eq!(beam.line(), expected_line);
            for _ in 0..65 {
                beam.tick();
            }
        }

        assert_eq!(beam.line(), 262);
        assert_eq!(beam.cycle(), 0);

        // Tick through line 262 (cycles 1..=64)
        for _ in 0..64 {
            beam.tick();
        }
        assert_eq!(beam.line(), 262);
        assert_eq!(beam.cycle(), 64);

        // Final tick of the frame: wrap line back to 0
        let new_line = beam.tick();
        assert!(new_line);
        assert_eq!(beam.line(), 0, "Line must wrap from 262 to 0 on NTSC");
        assert_eq!(beam.cycle(), 0);
    }

    // ---------------------------------------------------------------------
    // 4. Kernal PAL/NTSC Detection Invariance
    // ---------------------------------------------------------------------

    #[test]
    fn pal_hits_line_311_for_kernal_boot_detection() {
        // CINT ($FF5B) sets a raster compare for line 311 ($137).
        // PAL systems MUST reach line 311 to pass boot detection.
        let mut beam = RasterBeam::new(VideoStandard::Pal);
        let mut line_311_reached = false;

        for _ in 0..VideoStandard::Pal.total_cycles_per_frame() {
            if beam.line() == 311 {
                line_311_reached = true;
                break;
            }
            beam.tick();
        }

        assert!(line_311_reached, "PAL standard must visit line 311");
    }

    #[test]
    fn ntsc_never_hits_line_311() {
        let mut beam = RasterBeam::new(VideoStandard::Ntsc);

        // Run a full NTSC frame + extra
        for _ in 0..(VideoStandard::Ntsc.total_cycles_per_frame() * 2) {
            assert!(
                beam.line() <= 262,
                "NTSC line must never exceed 262 (hit line {})",
                beam.line()
            );
            beam.tick();
        }
    }

    // ---------------------------------------------------------------------
    // 5. Frame Total Cycle Counts
    // ---------------------------------------------------------------------

    #[test]
    fn exact_cycles_per_full_frame() {
        let mut pal = RasterBeam::new(VideoStandard::Pal);
        for _ in 0..VideoStandard::Pal.total_cycles_per_frame() {
            pal.tick();
        }
        assert_eq!(pal.cycle(), 0);
        assert_eq!(pal.line(), 0);

        let mut ntsc = RasterBeam::new(VideoStandard::Ntsc);
        for _ in 0..VideoStandard::Ntsc.total_cycles_per_frame() {
            ntsc.tick();
        }
        assert_eq!(ntsc.cycle(), 0);
        assert_eq!(ntsc.line(), 0);
    }
}
