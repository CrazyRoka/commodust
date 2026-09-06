use crate::{
    VideoStandard,
    address_generator::{Address, AddressGenerator},
    border::BorderUnit,
    raster::RasterBeam,
    registers::Registers,
    sequencer::Sequencer,
};

/// First cycle of the video-matrix (c-access) fetch phase on bad lines.
const C_ACCESS_FIRST_CYCLE: u8 = 15;
/// Last cycle of the c-access fetch phase (column 39 is fetched here).
const C_ACCESS_LAST_CYCLE: u8 = 54;
/// First display cycle of the 40-column window (CSEL=1): shows column 0.
const DISPLAY_FIRST_CYCLE: u8 = 16;
/// Last display cycle of the 40-column window: shows column 39.
const DISPLAY_LAST_CYCLE: u8 = 55;

pub struct VicII {
    raster: RasterBeam,
    registers: Registers,
    sequencer: Sequencer,
    border_unit: BorderUnit,
    address_generator: AddressGenerator,
    framebuffer: Box<[u8]>,
    frame_ready: bool,
}

impl VicII {
    pub fn new(standard: VideoStandard) -> Self {
        let height = standard.total_lines() as usize;
        let width = standard.cycles_per_line() as usize * 8;
        Self {
            raster: RasterBeam::new(standard),
            registers: Registers::new(),
            sequencer: Sequencer::new(),
            border_unit: BorderUnit::new(),
            address_generator: AddressGenerator::new(),
            framebuffer: vec![0; height * width].into_boxed_slice(),
            frame_ready: false,
        }
    }

    pub fn tick(&mut self, vic_bank: u8, ram: &[u8], char_rom: &[u8], color_ram: &[u8]) {
        let cycle = self.raster.cycle();
        let line = self.raster.line();

        self.border_unit.tick(
            cycle,
            line,
            self.registers.rsel(),
            self.registers.csel(),
            self.registers.den(),
        );

        self.address_generator.set_bank(vic_bank);

        // TODO: can we use sequencer is c access method?
        if (C_ACCESS_FIRST_CYCLE..=C_ACCESS_LAST_CYCLE).contains(&cycle)
            && self.sequencer.is_bad_line()
        {
            let vc = self.sequencer.vc();
            let char_code = match self
                .address_generator
                .c_access_address(self.registers.video_matrix_base(), vc)
            {
                Address::CharacterRom(addr) => char_rom[addr as usize],
                Address::Ram(addr) => ram[addr as usize],
            };
            let color = color_ram[vc as usize & 0x3FF] & 0x0F;
            self.sequencer.store_c_access(char_code, color);
        }

        let offset = line as usize * self.raster.standard().cycles_per_line() as usize * 8
            + cycle as usize * 8;
        if self.border_unit.is_border_active() {
            self.framebuffer[offset..offset + 8].fill(self.registers.border_color());
        } else {
            if self.sequencer.is_display_state() // TODO: should we remove next line?
                && (DISPLAY_FIRST_CYCLE..=DISPLAY_LAST_CYCLE).contains(&cycle)
            {
                let col = cycle - DISPLAY_FIRST_CYCLE;
                let (char_code, color) = self.sequencer.char_and_color_at(col);
                let cell_vc = (self.sequencer.vcbase() + col as u16) & 0x3FF;

                match (
                    self.registers.bmm(),
                    self.registers.mcm(),
                    self.registers.ecm(),
                ) {
                    (false, false, false) => {
                        let glyph = match self.address_generator.g_access_address(
                            self.registers.char_base(),
                            char_code,
                            self.sequencer.rc(),
                        ) {
                            Address::CharacterRom(addr) => char_rom[addr as usize],
                            Address::Ram(addr) => ram[addr as usize],
                        };
                        let background = self.registers.background_colors(0);
                        for idx in 0..8 {
                            self.framebuffer[offset + idx as usize] = if (0x80 >> idx) & glyph != 0
                            {
                                color & 0x0F
                            } else {
                                background
                            };
                        }
                    }
                    (false, true, false) => {
                        let glyph = match self.address_generator.g_access_address(
                            self.registers.char_base(),
                            char_code,
                            self.sequencer.rc(),
                        ) {
                            Address::CharacterRom(addr) => char_rom[addr as usize],
                            Address::Ram(addr) => ram[addr as usize],
                        };

                        if color & 0x08 != 0 {
                            for pair in 0..4 {
                                let result = match (glyph >> (6 - pair * 2)) & 0b11 {
                                    0b00 => self.registers.background_colors(0),
                                    0b01 => self.registers.background_colors(1),
                                    0b10 => self.registers.background_colors(2),
                                    0b11 => color & 0x07,
                                    _ => unreachable!("It's impossible to reach this branch"),
                                };
                                for idx in (pair * 2)..=(pair * 2 + 1) {
                                    self.framebuffer[offset + idx as usize] = result;
                                }
                            }
                        } else {
                            let background = self.registers.background_colors(0);
                            for idx in 0..8 {
                                self.framebuffer[offset + idx as usize] =
                                    if (0x80 >> idx) & glyph != 0 {
                                        color & 0x07
                                    } else {
                                        background
                                    };
                            }
                        }
                    }
                    (false, false, true) => {
                        let glyph = match self.address_generator.g_access_address(
                            self.registers.char_base(),
                            char_code & 0x3F,
                            self.sequencer.rc(),
                        ) {
                            Address::CharacterRom(addr) => char_rom[addr as usize],
                            Address::Ram(addr) => ram[addr as usize],
                        };
                        let background = self
                            .registers
                            .background_colors((char_code >> 6) as usize & 0x03);
                        for idx in 0..8 {
                            self.framebuffer[offset + idx as usize] = if (0x80 >> idx) & glyph != 0
                            {
                                color & 0x0F
                            } else {
                                background
                            };
                        }
                    }
                    (true, false, false) => {
                        let glyph = match self.address_generator.bitmap_address(
                            self.registers.bitmap_base(),
                            cell_vc,
                            self.sequencer.rc(),
                        ) {
                            Address::CharacterRom(addr) => char_rom[addr as usize],
                            Address::Ram(addr) => ram[addr as usize],
                        };
                        for idx in 0..8 {
                            self.framebuffer[offset + idx as usize] = if (0x80 >> idx) & glyph != 0
                            {
                                (char_code >> 4) & 0x0F
                            } else {
                                char_code & 0x0F
                            };
                        }
                    }
                    (true, true, false) => {
                        let glyph = match self.address_generator.bitmap_address(
                            self.registers.bitmap_base(),
                            cell_vc,
                            self.sequencer.rc(),
                        ) {
                            Address::CharacterRom(addr) => char_rom[addr as usize],
                            Address::Ram(addr) => ram[addr as usize],
                        };

                        for pair in 0..4 {
                            let result = match (glyph >> (6 - pair * 2)) & 0b11 {
                                0b00 => self.registers.background_colors(0),
                                0b01 => (char_code >> 4) & 0x0F,
                                0b10 => char_code & 0x0F,
                                0b11 => color & 0x0F,
                                _ => unreachable!("It's impossible to reach this branch"),
                            };
                            for idx in (pair * 2)..=(pair * 2 + 1) {
                                self.framebuffer[offset + idx as usize] = result;
                            }
                        }
                    }
                    (_, _, true) => {
                        self.framebuffer[offset..(offset + 8)].fill(0x00);
                    }
                }
            } else {
                self.framebuffer[offset..offset + 8].fill(self.registers.background_colors(0));
            }
        }

        self.sequencer
            .tick(cycle, line, self.registers.yscroll(), self.registers.den());

        if self.raster.tick() {
            self.registers.check_raster_match(self.raster.line());
            if self.raster.line() == 0 {
                self.frame_ready = true;
            }
        }
    }

    pub fn read(&self, offset: u8) -> u8 {
        self.registers.read(offset, self.raster.line())
    }

    pub fn write(&mut self, offset: u8, value: u8) {
        self.registers.write(offset, value);
    }

    pub fn ba(&self) -> bool {
        self.sequencer.ba()
    }

    pub fn irq(&self) -> bool {
        self.registers.is_irq_asserted()
    }

    pub fn framebuffer(&self) -> &[u8] {
        self.framebuffer.as_ref()
    }

    pub fn frame_ready(&self) -> bool {
        self.frame_ready
    }

    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL / RESET STATE
    //    * ba() = true (bus available to CPU).
    //    * irq() = false (no interrupts asserted).
    //    * frame_ready() = false.
    //    * framebuffer is allocated and initialized.
    //
    // 2. REGISTER DELEGATION
    //    * read(offset) delegates to Registers with the current beam line.
    //    * write(offset, val) delegates to Registers.
    //    * Writing $D020 updates border color; writing $D021 updates background.
    //    * Reading $D012 returns lower 8 bits of current raster line.
    //
    // 3. BUS ARBITRATION (BA Line)
    //    * On Bad Lines, ba() drops to false at cycle 12.
    //    * ba() returns to true at cycle 55.
    //    * On Non-Bad Lines, ba() remains true throughout.
    //
    // 4. INTERRUPTS & KERNAL PAL BOOT
    //    * When the raster line matches raster compare, and the mask in $D01A
    //      is enabled, irq() asserts true.
    //    * Kernal CINT boot test: raster compare at line 311 asserts irq()
    //      when line 311 is reached.
    //
    // 5. FRAME COMPLETION LIFECYCLE
    //    * frame_ready() becomes true when the beam wraps around to line 0, cycle 0.
    //    * clear_frame_ready() resets it back to false.
    //
    // 6. END-TO-END PIXEL RENDERING (Standard Text Mode)
    //    * Top/Bottom borders (lines 0..=50, 251..=311) fill all 504 pixels
    //      with border_color ($D020).
    //    * Display lines (lines 51..=250):
    //      - Cycles 0..=15 (pixels 0..=127): border_color.
    //      - Cycle 16 (first character column): renders 8 pixels based on
    //        the glyph byte from Character ROM:
    //        * bit 1 -> foreground color (from Color RAM via c-access).
    //        * bit 0 -> background color ($D021).
    //      - Cycles 56..=62 (pixels 448..=503): border_color.
    //
    // 7. SCREEN BLANKING (DEN = 0)
    //    * When DEN is 0, the entire screen renders as border_color.
    // =====================================================================

    struct TestHarness {
        vic: VicII,
        ram: [u8; 65536],
        char_rom: [u8; 4096],
        color_ram: [u8; 1024],
    }

    impl TestHarness {
        fn new() -> Self {
            Self {
                vic: VicII::new(VideoStandard::Pal),
                ram: [0; 65536],
                char_rom: [0; 4096],
                color_ram: [0; 1024],
            }
        }

        fn tick(&mut self, vic_bank: u8) {
            self.vic
                .tick(vic_bank, &self.ram, &self.char_rom, &self.color_ram);
        }

        fn tick_scanline(&mut self, vic_bank: u8) {
            for _ in 0..63 {
                self.tick(vic_bank);
            }
        }

        fn tick_full_pal_frame(&mut self, vic_bank: u8) {
            for _ in 0..(312 * 63) {
                self.tick(vic_bank);
            }
        }
    }

    // ---------------------------------------------------------------------
    // 1. Initial State
    // ---------------------------------------------------------------------

    #[test]
    fn new_vic_starts_with_clean_signals_and_unready_frame() {
        let h = TestHarness::new();
        assert!(h.vic.ba(), "BA must be high initially");
        assert!(!h.vic.irq(), "IRQ must be inactive initially");
        assert!(!h.vic.frame_ready(), "Frame must not be ready initially");
        assert_eq!(h.vic.framebuffer().len(), 504 * 312);
    }

    // ---------------------------------------------------------------------
    // 2. Register Delegation & Forwarding
    // ---------------------------------------------------------------------

    #[test]
    fn read_and_write_delegate_to_registers() {
        let mut h = TestHarness::new();

        // Write border color to Light Red ($0A)
        h.vic.write(0x20, 0x0A);
        assert_eq!(h.vic.read(0x20) & 0x0F, 0x0A);

        // Write raster compare low byte
        h.vic.write(0x12, 0x55);

        // Reading $D012 returns the live beam line (currently line 0), NOT compare!
        assert_eq!(h.vic.read(0x12), 0x00);
    }

    // ---------------------------------------------------------------------
    // 3. Raster IRQ & Kernal PAL Boot Detection
    // ---------------------------------------------------------------------

    #[test]
    fn raster_interrupt_asserts_irq_pin() {
        let mut h = TestHarness::new();

        // Set raster compare to line 50 ($32)
        h.vic.write(0x12, 50);
        h.vic.write(0x11, h.vic.read(0x11) & !0x80); // compare bit 8 = 0
        // Enable raster compare interrupt mask in $D01A (bit 0)
        h.vic.write(0x1A, 0x01);

        assert!(!h.vic.irq());

        // Step up to line 49
        for _ in 0..50 {
            h.tick_scanline(0);
        }

        h.tick(0);
        // On line 50, IRQ must assert!
        assert!(
            h.vic.irq(),
            "IRQ pin must assert when beam reaches compare line"
        );

        // Acknowledge interrupt by writing 1 to $D019 bit 0
        h.vic.write(0x19, 0x01);
        assert!(!h.vic.irq(), "Writing 1 to $D019 must de-assert IRQ pin");
    }

    #[test]
    fn kernal_cint_line_311_pal_boot_detection() {
        let mut h = TestHarness::new();

        // Kernal CINT ($FF5B) sequence:
        // Sets raster compare line 311 ($137 -> $D012 = $37, $D011 bit 7 = 1)
        h.vic.write(0x12, 0x37);
        h.vic.write(0x11, h.vic.read(0x11) | 0x80);
        // Enable raster compare interrupt mask
        h.vic.write(0x1A, 0x01);
        // Clear flags
        h.vic.write(0x19, 0x0F);

        assert!(!h.vic.irq());

        // Run 310 full lines (lines 0 through 309).
        // Must NOT trigger IRQ on any of these lines!
        for _ in 0..310 {
            h.tick_scanline(0);
            assert!(!h.vic.irq(), "Must not trigger before line 311");
        }

        // Run line 310: at the end of line 310, the beam enters line 311
        h.tick_scanline(0);
        assert!(
            h.vic.irq(),
            "PAL line 311 match must assert IRQ to pass Kernal CINT detection"
        );
    }

    // ---------------------------------------------------------------------
    // 4. BA Bus Stealing on Bad Lines
    // ---------------------------------------------------------------------

    #[test]
    fn ba_drops_on_bad_lines_and_rises_at_cycle_55() {
        let mut h = TestHarness::new();

        // Explicitly set YSCROLL to 0 (default in $D011 is 3!)
        let d011 = h.vic.read(0x11) & !0x07;
        h.vic.write(0x11, d011);

        // Advance to line 0x30 (which is now a Bad Line because line & 7 == 0 == yscroll)
        for _ in 0..0x30 {
            h.tick_scanline(0);
        }

        // Cycles 0..=11: BA is high
        for _ in 0..12 {
            h.tick(0);
            assert!(h.vic.ba());
        }

        // Cycle 12: BA drops low
        h.tick(0);
        assert!(!h.vic.ba(), "BA must drop at cycle 12 of Bad Line");

        // Cycles 13..=54: BA stays low
        for _ in 13..=54 {
            h.tick(0);
            assert!(!h.vic.ba());
        }

        // Cycle 55: BA returns high
        h.tick(0);
        assert!(h.vic.ba(), "BA must return high at cycle 55");
    }

    // ---------------------------------------------------------------------
    // 5. Frame Completion Lifecycle
    // ---------------------------------------------------------------------

    #[test]
    fn frame_ready_signals_at_start_of_new_frame() {
        let mut h = TestHarness::new();

        // Run 311 full lines (lines 0..311)
        for _ in 0..311 {
            h.tick_scanline(0);
            assert!(!h.vic.frame_ready());
        }

        // Run line 311 up to the last cycle
        for _ in 0..62 {
            h.tick(0);
            assert!(!h.vic.frame_ready());
        }

        // Final tick wraps to line 0, cycle 0: frame is complete!
        h.tick(0);
        assert!(
            h.vic.frame_ready(),
            "frame_ready must be true on frame wrap"
        );

        h.vic.clear_frame_ready();
        assert!(
            !h.vic.frame_ready(),
            "clear_frame_ready must reset the flag"
        );
    }

    // ---------------------------------------------------------------------
    // 6. End-to-End Pixel Rendering: Borders
    // ---------------------------------------------------------------------

    #[test]
    fn top_border_lines_are_100_percent_border_color() {
        let mut h = TestHarness::new();
        let border_color = 0x0E; // Light Blue default
        h.vic.write(0x20, border_color);

        // Render lines 0 through 50 (top border)
        for _ in 0..=50 {
            h.tick_scanline(0);
        }

        let fb = h.vic.framebuffer();
        // Check every pixel on lines 0..=50:
        for line in 0..=50 {
            for pixel in 0..504 {
                let color = fb[line * 504 + pixel];
                assert_eq!(
                    color, border_color,
                    "Pixel on line {} col {} must be border color",
                    line, pixel
                );
            }
        }
    }

    #[test]
    fn display_line_renders_border_margins_around_graphics() {
        let mut h = TestHarness::new();
        let border_color = 0x02; // Red border
        h.vic.write(0x20, border_color);

        // Step past top border up to line 51
        for _ in 0..=50 {
            h.tick_scanline(0);
        }

        // Render line 51
        h.tick_scanline(0);

        let fb = h.vic.framebuffer();
        let line_offset = 51 * 504;

        // Left border: Cycles 0..=15 = Pixels 0..=127
        for x in 0..128 {
            assert_eq!(
                fb[line_offset + x],
                border_color,
                "Left border pixel {} must be border color",
                x
            );
        }

        // Right border: Cycles 56..=62 = Pixels 448..=503
        for x in 448..504 {
            assert_eq!(
                fb[line_offset + x],
                border_color,
                "Right border pixel {} must be border color",
                x
            );
        }
    }

    // ---------------------------------------------------------------------
    // 7. End-to-End Text Mode Character Rendering
    // ---------------------------------------------------------------------

    #[test]
    fn standard_text_mode_renders_character_glyph_from_char_rom() {
        let mut h = TestHarness::new();

        // 1. Setup Colors:
        // Border: 14 (Light Blue)
        // Background 0: 6 (Dark Blue)
        // Foreground: 1 (White)
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.color_ram[0] = 1; // First character position color = White

        // 2. Setup Screen Memory:
        // Default video matrix is at $0400. Put Screen Code 1 ('A') at column 0 ($0400):
        h.ram[0x0400] = 1;

        // 3. Setup Character ROM:
        // Screen code 1 ('A'), row 0 is at offset: 1 * 8 + 0 = 8.
        // Let's define the top row of 'A' as: 0b00011100 (dots in the middle)
        let glyph_row_byte = 0b00011100;
        h.char_rom[8] = glyph_row_byte;

        // 4. Run lines 0..=50 (top border)
        for _ in 0..=50 {
            h.tick_scanline(0);
        }

        // 5. Line 51 is the first display line (Row Counter RC = 0):
        // Run cycles 0..15 (left border)
        for _ in 0..16 {
            h.tick(0);
        }

        // Run cycle 16 (where column 0 is displayed as pixels 128..=135)
        h.tick(0);

        let fb = h.vic.framebuffer();
        let pixel_start = (51 * 504) + (16 * 8); // 128
        let rendered_pixels = &fb[pixel_start..pixel_start + 8];

        // 0b00011100:
        // bit 7: 0 -> Background (6)
        // bit 6: 0 -> Background (6)
        // bit 5: 0 -> Background (6)
        // bit 4: 1 -> Foreground (1)
        // bit 3: 1 -> Foreground (1)
        // bit 2: 1 -> Foreground (1)
        // bit 1: 0 -> Background (6)
        // bit 0: 0 -> Background (6)
        let expected_pixels = [6, 6, 6, 1, 1, 1, 6, 6];
        assert_eq!(
            rendered_pixels, expected_pixels,
            "Rendered 8 pixels must match the Character ROM glyph row and color attributes"
        );
    }

    // ---------------------------------------------------------------------
    // 8. Screen Blanking (DEN = 0)
    // ---------------------------------------------------------------------

    #[test]
    fn screen_blanking_den_zero_renders_entire_screen_as_border() {
        let mut h = TestHarness::new();
        let border_color = 0x05; // Green
        h.vic.write(0x20, border_color);

        // Turn display OFF by clearing DEN bit (bit 4 of $D011)
        let d011 = h.vic.read(0x11) & !0x10;
        h.vic.write(0x11, d011);

        // Run full frame
        h.tick_full_pal_frame(0);

        let fb = h.vic.framebuffer();
        for &pixel in fb.iter() {
            assert_eq!(
                pixel, border_color,
                "Every single pixel must be border color when DEN=0"
            );
        }
    }

    // ---------------------------------------------------------------------
    // 7b. Fetch/display pipeline: column <-> cycle mapping
    // ---------------------------------------------------------------------

    #[test]
    fn second_scanline_of_character_row_renders_the_next_glyph_row() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.color_ram[0] = 1;
        h.ram[0x0400] = 1; // screen code 1 ('A') in column 0
        h.char_rom[8] = 0b00011100; // RC=0 -> rendered on line 51
        h.char_rom[9] = 0b00110011; // RC=1 -> rendered on line 52

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        h.tick_scanline(0); // line 51: bad line, fetches the row, RC = 0
        for _ in 0..16 {
            h.tick(0);
        } // line 52 cycles 0..=15 (left border)
        h.tick(0); // cycle 16: column 0 again, glyph row RC=1
        // (line 52 is NOT a bad line: the line buffer persists)

        let fb = h.vic.framebuffer();
        let rendered = &fb[52 * 504 + 128..52 * 504 + 136];
        assert_eq!(rendered, [6u8, 6, 1, 1, 6, 6, 1, 1]);
    }

    #[test]
    fn last_display_column_is_fetched_at_cycle_54_and_shown_at_cycle_55() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400 + 39] = 1; // screen code 1 in column 39
        h.color_ram[39] = 1;
        h.char_rom[8] = 0b10000001; // glyph row 0 of screen code 1

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..55 {
            h.tick(0);
        } // cycles 0..=54 (column 39 fetched at 54)
        h.tick(0); // cycle 55 displays column 39 (pixels 440..=447)

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 440..51 * 504 + 448];
        assert_eq!(rendered, [1u8, 6, 6, 6, 6, 6, 6, 1]);
    }

    #[test]
    fn second_character_row_fetches_screen_codes_from_vcbase_40() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400 + 40] = 1; // row 1, column 0 (VCBASE = 40)
        h.color_ram[40] = 1;
        h.char_rom[8] = 0b00011100;

        for _ in 0..=58 {
            h.tick_scanline(0);
        } // row 0 (VCBASE 0 -> 40 at line 58)
        for _ in 0..16 {
            h.tick(0);
        } // line 59 cycles 0..=15
        h.tick(0); // cycle 16: column 0 of row 1 (VC = 40)

        let fb = h.vic.framebuffer();
        let rendered = &fb[59 * 504 + 128..59 * 504 + 136];
        assert_eq!(rendered, [6u8, 6, 6, 1, 1, 1, 6, 6]);
    }

    #[test]
    fn csel0_38_column_window_cuts_columns_0_and_39() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.vic.write(0x16, 0x00); // CSEL=0: 38 columns
        h.color_ram[0] = 1;
        h.color_ram[1] = 1;
        h.ram[0x0400] = 1; // column 0
        h.ram[0x0401] = 2; // column 1
        h.char_rom[8] = 0b11111111; // glyph row 0 of screen code 1
        h.char_rom[16] = 0b00011000; // glyph row 0 of screen code 2

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..17 {
            h.tick(0);
        } // cycle 16 is still border in 38-col mode
        h.tick(0); // cycle 17 shows character column 1

        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        for x in 128..136 {
            assert_eq!(fb[line + x], 14, "cycle 16 must remain border (pixel {x})");
        }
        assert_eq!(&fb[line + 136..line + 144], &[6u8, 6, 6, 1, 1, 6, 6, 6]);
    }

    #[test]
    fn opened_side_border_displays_background_color() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..56 {
            h.tick(0);
        } // cycles 0..=55 of line 51

        // Side-border removal: 38 columns just before the cycle-56 comparison.
        h.vic.write(0x16, 0x00); // CSEL = 0
        h.tick(0); // cycle 56: border open, no character column -> background

        let fb = h.vic.framebuffer();
        for x in 448..456 {
            assert_eq!(fb[51 * 504 + x], 6, "opened border must show background");
        }
    }

    #[test]
    fn multiple_full_pal_frames_render_without_panicking() {
        // Regression for the OOB line buffer access at cycle 55 of every
        // display line.
        let mut h = TestHarness::new();
        for frame in 0..3 {
            h.tick_full_pal_frame(0);
            assert!(h.vic.frame_ready(), "frame {frame} completed");
            h.vic.clear_frame_ready();
            assert!(!h.vic.frame_ready());
        }
        // Display window produced graphics (glyph $00 -> background, not border):
        assert_eq!(h.vic.framebuffer()[51 * 504 + 128], 6);
        assert_eq!(h.vic.framebuffer()[51 * 504], 14);
        assert_eq!(h.vic.framebuffer()[51 * 504 + 448], 14);
    }

    // ---------------------------------------------------------------------
    // 9. NTSC Frame Lifecycle & Dimensions
    // ---------------------------------------------------------------------

    #[test]
    fn ntsc_full_frame_renders_with_520_pixel_width_without_shearing() {
        let mut h = TestHarness {
            vic: VicII::new(VideoStandard::Ntsc),
            ram: [0; 65536],
            char_rom: [0; 4096],
            color_ram: [0; 1024],
        };

        let width = 520;
        let height = 263;
        assert_eq!(h.vic.framebuffer().len(), width * height);

        // Run 1 complete NTSC frame (263 lines * 65 cycles)
        for _ in 0..(263 * 65) {
            h.tick(0);
        }

        assert!(
            h.vic.frame_ready(),
            "NTSC must signal frame_ready on line wrap"
        );
        h.vic.clear_frame_ready();
        assert!(!h.vic.frame_ready());
    }

    #[test]
    fn ntsc_fails_line_311_test_allowing_kernal_to_detect_ntsc() {
        // The inverse of the PAL boot test: On NTSC, line 311 is NEVER reached,
        // so the IRQ must NEVER fire. This is how the Kernal auto-detects NTSC!
        let mut h = TestHarness {
            vic: VicII::new(VideoStandard::Ntsc),
            ram: [0; 65536],
            char_rom: [0; 4096],
            color_ram: [0; 1024],
        };

        // Set compare line 311
        h.vic.write(0x12, 0x37);
        h.vic.write(0x11, h.vic.read(0x11) | 0x80);
        h.vic.write(0x1A, 0x01); // Enable mask
        h.vic.write(0x19, 0x0F); // Clear flags

        // Run through TWO entire NTSC frames (2 * 263 lines)
        for _ in 0..(263 * 65 * 2) {
            h.tick(0);
            assert!(!h.vic.irq(), "IRQ must NEVER fire on NTSC for line 311");
        }
    }

    // ---------------------------------------------------------------------
    // 10. Per-Column Color Attributes from Color RAM
    // ---------------------------------------------------------------------

    #[test]
    fn adjacent_character_columns_display_independent_colors() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14); // Border = 14
        h.vic.write(0x21, 0); // Background = 0 (Black)

        // Column 0: Character 'A' (1), Color = 2 (Red)
        h.ram[0x0400] = 1;
        h.color_ram[0] = 2;

        // Column 1: Character 'A' (1), Color = 7 (Yellow)
        h.ram[0x0401] = 1;
        h.color_ram[1] = 7;

        // Solid glyph row: all 1s (0b11111111)
        h.char_rom[8] = 0b11111111;

        // Step up to display line 51
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        } // cycles 0..=15 (border)

        h.tick(0); // cycle 16 displays column 0 (pixels 128..=135)
        h.tick(0); // cycle 17 displays column 1 (pixels 136..=143)

        let fb = h.vic.framebuffer();
        let line = 51 * 504;

        // Column 0 must be all Red (2)
        assert_eq!(&fb[line + 128..line + 136], &[2u8; 8]);
        // Column 1 must be all Yellow (7)
        assert_eq!(&fb[line + 136..line + 144], &[7u8; 8]);
    }

    // ---------------------------------------------------------------------
    // 11. Relocated Video Matrix & Character ROM Base via $D018
    // ---------------------------------------------------------------------

    #[test]
    fn switching_d018_to_lowercase_fetches_from_rom_set_2() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.color_ram[0] = 1;

        // Switch to lowercase character set (POKE 53272, 23 -> $D018 = $17)
        // VM = $0400, CB = $1800 (Set 2)
        h.vic.write(0x18, 0x17);

        h.ram[0x0400] = 1; // Screen code 1
        // In Set 2 ($1800), screen code 1 row 0 is at offset: $0800 + 8 = $0808
        h.char_rom[0x0808] = 0b10101010;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // cycle 16

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        assert_eq!(rendered, &[1u8, 6, 1, 6, 1, 6, 1, 6]);
    }

    // ---------------------------------------------------------------------
    // 12. Bad-Line Window & Latching (full line range, DEN, latch)
    // ---------------------------------------------------------------------

    #[test]
    fn ba_stays_high_on_every_cycle_of_a_non_bad_display_line() {
        // Pins contract §3 "On Non-Bad Lines, ba() remains true throughout"
        // (currently only asserted for cycles 0..=11 of a bad line).
        let mut h = TestHarness::new();
        for _ in 0..=51 {
            h.tick_scanline(0);
        }
        for cycle in 0..63 {
            h.tick(0);
            assert!(
                h.vic.ba(),
                "BA must stay high on non-bad display line 52 (cycle {cycle})"
            );
        }
    }

    #[test]
    fn lines_below_48_matching_yscroll_are_never_bad_lines() {
        let mut h = TestHarness::new();
        // YSCROLL=0: lines 0,8,16,24,32,40 match (line & 7) == yscroll,
        // but bad lines require line >= $30.
        h.vic.write(0x11, h.vic.read(0x11) & !0x07);
        for line in 0..=47 {
            for _ in 0..63 {
                h.tick(0);
                assert!(h.vic.ba(), "BA must stay high below line $30 (line {line})");
            }
        }
    }

    #[test]
    fn lower_border_lines_matching_yscroll_never_steal_the_bus() {
        // ⚠ Pins real-hardware behaviour: bad lines are bounded by the display
        // line window. Lines 251, 259, ... still match (line & 7) == 3 with the
        // default YSCROLL, but BA must stay high. If this fails, the sequencer
        // is missing the upper bound (see finding #2).
        let mut h = TestHarness::new();
        for _ in 0..=250 {
            h.tick_scanline(0);
        }
        for line in 251..=311 {
            for _ in 0..63 {
                h.tick(0);
                assert!(
                    h.vic.ba(),
                    "BA must stay high in the lower border (line {line})"
                );
            }
        }
    }

    #[test]
    fn den_zero_suppresses_bad_lines_completely() {
        // Complements the DEN=0 pixel test: no bus stealing either.
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) & !0x17); // YSCROLL=0 and DEN=0
        for _ in 0..=47 {
            h.tick_scanline(0);
        }
        for _ in 0..63 {
            h.tick(0);
            assert!(h.vic.ba(), "no bad line may occur while DEN=0");
        }
    }

    #[test]
    fn bad_line_condition_is_latched_by_cycle_12() {
        // ⚠ Reference behaviour: the condition is sampled early in the line;
        // clearing DEN after cycle 12 must not cancel the in-flight DMA.
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) & !0x07); // YSCROLL=0 -> line 48 is bad
        for _ in 0..48 {
            h.tick_scanline(0);
        }
        for _ in 0..13 {
            h.tick(0); // cycles 0..=12, bad line latched during cycle 12
        }
        assert!(!h.vic.ba());
        h.vic.write(0x11, h.vic.read(0x11) & !0x10); // clear DEN at cycle 13
        for cycle in 13..=55u8 {
            h.tick(0);
            assert_eq!(
                h.vic.ba(),
                cycle >= 55,
                "cycle {cycle}: latched bad line must survive a late DEN clear"
            );
        }
    }

    // ---------------------------------------------------------------------
    // 13. Row Grid: all 25 rows across the full 200-line window
    // ---------------------------------------------------------------------

    #[test]
    fn every_display_line_shows_its_own_row_across_the_full_window() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 0);
        for row in 0u16..25 {
            h.ram[0x0400 + (row * 40) as usize] = 1; // solid glyph
            h.color_ram[(row * 40) as usize] = (row % 15) as u8 + 1; // unique per row
        }
        for rc in 0..8 {
            h.char_rom[8 + rc] = 0xFF;
        }
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for line in 51..=250u16 {
            h.tick_scanline(0);
            let row = (line - 51) / 8;
            assert_eq!(
                h.vic.framebuffer()[line as usize * 504 + 128],
                (row % 15) as u8 + 1,
                "line {line} (row {row}) must render that row's color RAM entry"
            );
        }
        h.tick_scanline(0); // line 251
        assert_eq!(
            h.vic.framebuffer()[251 * 504 + 128],
            14,
            "window is exactly 200 lines"
        );
    }

    #[test]
    fn bottom_border_lines_are_100_percent_border_color() {
        // Mirror of the top-border test (only the top half is pinned today).
        let mut h = TestHarness::new();
        h.vic.write(0x20, 0x0A);
        h.tick_full_pal_frame(0);
        let fb = h.vic.framebuffer();
        for line in 251..=311 {
            for pixel in 0..504 {
                assert_eq!(fb[line * 504 + pixel], 0x0A, "line {line} pixel {pixel}");
            }
        }
    }

    // ---------------------------------------------------------------------
    // 14. Border Geometry: RSEL, CSEL right edge, YSCROLL independence
    // ---------------------------------------------------------------------

    #[test]
    fn rsel0_shrinks_the_window_to_24_rows() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);

        // Populate column 0 for ALL 25 rows (including row 0 at line 55 and row 24 at line 246!)
        for row in 0..25 {
            h.ram[0x0400 + row * 40] = 1;
            h.color_ram[row * 40] = 1;
        }

        for rc in 0..8 {
            h.char_rom[8 + rc] = 0xFF;
        }
        h.vic.write(0x11, h.vic.read(0x11) & !0x08); // RSEL=0

        h.tick_full_pal_frame(0);
        let fb = h.vic.framebuffer();
        for line in 51..=54 {
            for x in 0..504 {
                assert_eq!(
                    fb[line * 504 + x],
                    14,
                    "RSEL=0: line {line} must be top border"
                );
            }
        }
        assert_eq!(
            &fb[55 * 504 + 128..55 * 504 + 136],
            &[1u8; 8],
            "line 55 is the first display line"
        );
        assert_eq!(fb[246 * 504 + 128], 1, "line 246 is the last display line");
        for line in 247..=311 {
            for x in 0..504 {
                assert_eq!(
                    fb[line * 504 + x],
                    14,
                    "RSEL=0: line {line} must be bottom border"
                );
            }
        }
    }

    #[test]
    fn csel0_closes_the_right_display_edge_at_cycle_55() {
        // Complements the existing CSEL=0 test, which only pins the left edge.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.vic.write(0x16, 0x00); // CSEL=0: 38 columns
        h.ram[0x0400 + 38] = 1; // column 38 is the last visible one
        h.color_ram[38] = 1;
        h.char_rom[8] = 0xFF;
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..54 {
            h.tick(0); // cycles 0..=53
        }
        h.tick(0); // cycle 54: last visible column (pixels 432..=439)
        h.tick(0); // cycle 55: right border
        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        assert_eq!(
            &fb[line + 432..line + 440],
            &[1u8; 8],
            "cycle 54 must show column 38"
        );
        for x in 440..448 {
            assert_eq!(fb[line + x], 14, "cycle 55 must be border (pixel {x})");
        }
    }

    #[test]
    fn yscroll_zero_shifts_rows_up_but_leaves_the_border_where_it_is() {
        // ⚠ Pins that YSCROLL moves the character row grid, NOT the border
        // (this is what makes FLD work). With YSCROLL=0 the first bad line is
        // line 48 (inside the top border) and line 51 shows RC=3 of row 0.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400] = 1;
        h.color_ram[0] = 1;
        h.char_rom[8 + 0] = 0b1000_0000; // RC=0 -> rendered on line 48 (hidden)
        h.char_rom[8 + 3] = 0b0100_0000; // RC=3 -> rendered on line 51 (visible)
        h.vic.write(0x11, h.vic.read(0x11) & !0x07);

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        let fb = h.vic.framebuffer();
        for line in 48..=50 {
            for x in 0..504 {
                assert_eq!(fb[line * 504 + x], 14, "line {line} must stay top border");
            }
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // cycle 16 of line 51
        assert_eq!(
            &h.vic.framebuffer()[51 * 504 + 128..51 * 504 + 136],
            &[6u8, 1, 6, 6, 6, 6, 6, 6],
            "line 51 must show glyph row RC=3"
        );
    }

    #[test]
    fn rsel_switch_after_line_247_opens_the_bottom_border() {
        // ⚠ Classic open-border trick: with RSEL=0 the border-set comparison for
        // line 247 has already passed; switching 1->0 during lines 248..=250
        // means no set condition ever matches. Lines 248..250 still show row 24
        // through the open border; after the RC wrap it's background.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400 + 960] = 1; // row 24, column 0
        h.color_ram[960] = 1;
        for rc in 0..8 {
            h.char_rom[8 + rc] = 0xFF;
        }
        for _ in 0..=247 {
            h.tick_scanline(0);
        }
        h.vic.write(0x11, h.vic.read(0x11) & !0x08); // RSEL=0 during line 248
        for line in 248..=250u16 {
            h.tick_scanline(0);
            assert_eq!(
                h.vic.framebuffer()[line as usize * 504 + 128],
                1,
                "line {line} must still show row 24 through the open border"
            );
        }
        for _ in 0..10 {
            h.tick_scanline(0); // lines 251..=260: display state off
        }
        let fb = h.vic.framebuffer();
        let line = 260 * 504;
        for x in 0..128 {
            assert_eq!(fb[line + x], 14, "left border stays (pixel {x})");
        }
        for x in 128..448 {
            assert_eq!(
                fb[line + x],
                6,
                "open vertical border shows background (pixel {x})"
            );
        }
        for x in 448..504 {
            assert_eq!(fb[line + x], 14, "right border stays (pixel {x})");
        }
    }

    // ---------------------------------------------------------------------
    // 15. Mid-Frame DEN (display-state persistence)
    // ---------------------------------------------------------------------

    #[test]
    fn den_cleared_mid_frame_blanks_only_after_the_row_counter_wraps() {
        // ⚠ Reference behaviour (finding #4): the current row finishes; from the
        // next would-be bad line on, the display area shows background — NOT
        // border — and the lower border still starts at line 251.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400] = 1;
        h.color_ram[0] = 1;
        for rc in 0..8 {
            h.char_rom[8 + rc] = 0xFF;
        }
        for _ in 0..=52 {
            h.tick_scanline(0); // lines 0..=52 (RC 0 and 1 of row 0)
        }
        h.vic.write(0x11, h.vic.read(0x11) & !0x10); // clear DEN during line 53
        for line in 53..=59u16 {
            h.tick_scanline(0);
            let expected = if line <= 58 { 1u8 } else { 6 };
            assert_eq!(
                h.vic.framebuffer()[line as usize * 504 + 128],
                expected,
                "line {line}: row must finish before blanking"
            );
        }
        for _ in 0..192 {
            h.tick_scanline(0); // up to and including line 251
        }
        assert_eq!(
            h.vic.framebuffer()[251 * 504 + 128],
            14,
            "lower border must set regardless of DEN"
        );
    }

    // ---------------------------------------------------------------------
    // 16. Data Sources: RAM charset, VIC banks, color RAM masking
    // ---------------------------------------------------------------------

    #[test]
    fn custom_charset_in_ram_at_2000_is_used_for_glyphs() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400] = 1;
        h.color_ram[0] = 1;
        h.vic.write(0x18, 0x18); // VM=$0400, CB=$2000 (RAM charset)
        h.ram[0x2008] = 0b0001_1100; // screen code 1, row 0
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);
        assert_eq!(
            &h.vic.framebuffer()[51 * 504 + 128..51 * 504 + 136],
            &[6u8, 6, 6, 1, 1, 1, 6, 6]
        );
    }

    #[test]
    fn color_ram_entries_are_masked_to_the_low_nibble() {
        // ⚠ EXPECTED TO FAIL against the current code (finding #1): the color
        // byte is used unmasked as the pixel value. If your intended contract is
        // "color_ram contains pre-masked nibbles", keep this test but move the
        // guarantee to a debug_assert in tick().
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.ram[0x0400] = 1;
        h.color_ram[0] = 0x21; // low nibble says color 1, high bits are garbage
        h.char_rom[8] = 0xFF;
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);
        assert_eq!(
            &h.vic.framebuffer()[51 * 504 + 128..51 * 504 + 136],
            &[1u8; 8]
        );
    }

    #[test]
    fn bank_1_fetches_screen_and_glyphs_from_ram() {
        // First vic-level test with bank != 0. Screen at CPU $4400, char base
        // $1000 -> CPU $5000: everything is RAM, the ROM overlay must NOT apply.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.color_ram[0] = 1;
        h.ram[0x4400] = 1;
        h.ram[0x5008] = 0b0001_1000;
        for _ in 0..=50 {
            h.tick_scanline(1);
        }
        for _ in 0..16 {
            h.tick(1);
        }
        h.tick(1);
        assert_eq!(
            &h.vic.framebuffer()[51 * 504 + 128..51 * 504 + 136],
            &[6u8, 6, 6, 1, 1, 6, 6, 6]
        );
    }

    #[test]
    fn bank_2_uses_character_rom_for_glyphs_and_ram_for_the_screen() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 6);
        h.color_ram[0] = 1;
        h.ram[0x8400] = 1; // screen at CPU $8400
        h.char_rom[8] = 0b1000_0001; // glyphs must come from the ROM overlay
        h.ram[0x9008] = 0xFF; // decoy: the RAM behind the overlay
        for _ in 0..=50 {
            h.tick_scanline(2);
        }
        for _ in 0..16 {
            h.tick(2);
        }
        h.tick(2);
        assert_eq!(
            &h.vic.framebuffer()[51 * 504 + 128..51 * 504 + 136],
            &[1u8, 6, 6, 6, 6, 6, 6, 1]
        );
    }

    #[test]
    fn d020_and_d021_writes_are_masked_to_the_low_nibble_when_rendered() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 0xF2); // border: $02 with garbage high bits
        h.vic.write(0x21, 0xD6); // background: $06 with garbage high bits
        h.ram[0x0400] = 1;
        h.color_ram[0] = 1;
        h.char_rom[8] = 0x00; // empty glyph -> background pixels
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);
        let fb = h.vic.framebuffer();
        assert_eq!(fb[51 * 504 + 0], 0x02, "border color must be masked to $02");
        assert_eq!(
            fb[51 * 504 + 128],
            0x06,
            "background color must be masked to $06"
        );
    }

    // ---------------------------------------------------------------------
    // 17. Register Readback & IRQ Latch Semantics
    // ---------------------------------------------------------------------

    #[test]
    fn d012_and_d011_bit7_report_the_beam_above_line_255() {
        let mut h = TestHarness::new();
        for _ in 0..100 {
            h.tick_scanline(0);
        }
        assert_eq!(h.vic.read(0x12), 100);
        assert_eq!(h.vic.read(0x11) & 0x80, 0x00);
        for _ in 0..200 {
            h.tick_scanline(0); // now on line 300
        }
        assert_eq!(h.vic.read(0x12), (300 - 256) as u8);
        assert_eq!(h.vic.read(0x11) & 0x80, 0x80);
    }

    #[test]
    fn raster_flag_latches_with_mask_off_and_enabling_the_mask_asserts_irq() {
        // ⚠ Reference behaviour: the pending flag in $D019 is set on a match even
        // when the mask is disabled, and enabling the mask afterwards asserts the
        // IRQ pin immediately.
        let mut h = TestHarness::new();
        h.vic.write(0x12, 100);
        h.vic.write(0x11, h.vic.read(0x11) & !0x80);
        h.vic.write(0x1A, 0x00); // mask off
        for _ in 0..101 {
            h.tick_scanline(0);
        }
        assert!(!h.vic.irq(), "IRQ output must be gated by the mask");
        assert_eq!(
            h.vic.read(0x19) & 0x01,
            0x01,
            "pending flag must be readable via $D019"
        );
        h.vic.write(0x1A, 0x01); // enable the mask late
        assert!(
            h.vic.irq(),
            "enabling the mask with a pending flag must assert IRQ"
        );
        h.vic.write(0x19, 0x01);
        assert!(!h.vic.irq());
    }

    #[test]
    fn raster_irq_fires_again_on_every_frame_after_ack() {
        let mut h = TestHarness::new();
        h.vic.write(0x12, 100);
        h.vic.write(0x11, h.vic.read(0x11) & !0x80);
        h.vic.write(0x1A, 0x01);
        for _ in 0..101 {
            h.tick_scanline(0);
        }
        assert!(h.vic.irq());
        h.vic.write(0x19, 0x01);
        assert!(!h.vic.irq());
        for _ in 0..312 {
            h.tick_scanline(0); // one full frame, back past line 100
        }
        assert!(h.vic.irq(), "the raster compare must fire every frame");
    }

    // ---------------------------------------------------------------------
    // 18. Mid-Scanline Register Writes
    // ---------------------------------------------------------------------

    #[test]
    fn mid_line_border_color_write_takes_effect_from_the_next_cycle() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 0x02);
        for _ in 0..=10 {
            h.tick_scanline(0); // now on line 11 (top border)
        }
        for _ in 0..30 {
            h.tick(0); // cycles 0..=29 rendered with $02
        }
        h.vic.write(0x20, 0x0A); // border color change mid-line
        h.tick(0); // cycle 30 -> pixels 240..=247
        let fb = h.vic.framebuffer();
        for x in 0..240 {
            assert_eq!(fb[11 * 504 + x], 0x02, "pixel {x} precedes the write");
        }
        for x in 240..248 {
            assert_eq!(fb[11 * 504 + x], 0x0A, "pixel {x} follows the write");
        }
    }

    // ---------------------------------------------------------------------
    // 19. Frame-to-Frame Consistency
    // ---------------------------------------------------------------------

    #[test]
    fn video_matrix_counters_reset_between_frames() {
        // ⚠ Distinct content per row (finding #3): if VC/VCBASE don't reset at
        // the frame boundary, the second frame renders shifted/wrapped rows.
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14);
        h.vic.write(0x21, 0);
        for row in 0u16..25 {
            h.ram[0x0400 + (row * 40) as usize] = 1;
            h.color_ram[(row * 40) as usize] = (row % 15) as u8 + 1;
        }
        for rc in 0..8 {
            h.char_rom[8 + rc] = 0xFF;
        }
        let row_sample = |h: &TestHarness| -> Vec<u8> {
            (0..25usize)
                .map(|row| h.vic.framebuffer()[(51 + row * 8) * 504 + 128])
                .collect()
        };
        h.tick_full_pal_frame(0);
        let first = row_sample(&h);
        for row in 0..25usize {
            assert_eq!(first[row], (row % 15) as u8 + 1, "frame 1, row {row}");
        }
        h.tick_full_pal_frame(0);
        assert_eq!(row_sample(&h), first, "rows must not drift between frames");
    }

    #[test]
    fn identical_sequences_produce_identical_framebuffers() {
        let run = || {
            let mut h = TestHarness::new();
            h.vic.write(0x20, 14);
            h.vic.write(0x21, 6);
            h.ram[0x0400] = 1;
            h.color_ram[0] = 1;
            h.char_rom[8] = 0b1010_1010;
            h.tick_full_pal_frame(0);
            h.vic.framebuffer().to_vec()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn standard_bitmap_renders_pixels_from_video_matrix_nibbles() {
        let mut h = TestHarness::new();

        // 1. Enable Standard Bitmap Mode: BMM=1, MCM=0, ECM=0
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x16, h.vic.read(0x16) & !0x10); // MCM = 0

        // 2. Setup memory pointers ($D018):
        // Video matrix at $0400 (bits 4-7 = 0x01)
        // Bitmap at $2000 (bit 3 = 1) -> $D018 = 0x18
        h.vic.write(0x18, 0x18);

        // 3. Setup Colors for Cell (col 0, row 0):
        // Video Matrix byte at $0400: High nibble = 2 (Red/FG), Low nibble = 7 (Yellow/BG)
        h.ram[0x0400] = 0x27;

        // Set decoy values in Color RAM and $D021 to ensure they are NOT used
        h.color_ram[0] = 0x01; // White
        h.vic.write(0x21, 0x00); // Black
        h.vic.write(0x20, 0x0E); // Border Light Blue

        // 4. Setup Bitmap Data:
        // Top row of cell (col 0, RC 0) is at $2000:
        // 0b10110001 -> FG, BG, FG, FG, BG, BG, BG, FG
        h.ram[0x2000] = 0b10110001;

        // 5. Advance beam to line 51 (first display line)
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        // Advance to cycle 16 (display window start)
        for _ in 0..16 {
            h.tick(0);
        }
        // Render cycle 16 (column 0)
        h.tick(0);

        let fb = h.vic.framebuffer();
        let pixel_start = 51 * 504 + 128;
        let rendered = &fb[pixel_start..pixel_start + 8];

        let expected = [2, 7, 2, 2, 7, 7, 7, 2];
        assert_eq!(
            rendered, expected,
            "Standard bitmap must map bit 1 to VM high nibble and bit 0 to VM low nibble"
        );
    }

    #[test]
    fn standard_bitmap_progresses_through_rc_and_vc() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x18, 0x18); // VM $0400, Bitmap $2000

        h.ram[0x0400] = 0x16; // Column 0: FG=1 (White), BG=6 (Blue)
        h.ram[0x0401] = 0x35; // Column 1: FG=3 (Cyan), BG=5 (Green)

        // Column 0, scanline 0 (RC 0) at $2000
        h.ram[0x2000] = 0xFF; // All FG (White)
        // Column 0, scanline 1 (RC 1) at $2001
        h.ram[0x2001] = 0x00; // All BG (Blue)
        // Column 1, scanline 0 (RC 0) at $2008
        h.ram[0x2008] = 0xAA; // 0b10101010 -> FG, BG alternating

        // Render line 51 (RC = 0)
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // col 0
        h.tick(0); // col 1

        let l51 = 51 * 504;
        assert_eq!(
            &h.vic.framebuffer()[l51 + 128..l51 + 136],
            &[1u8; 8],
            "Line 51 Col 0 must be White"
        );
        assert_eq!(
            &h.vic.framebuffer()[l51 + 136..l51 + 144],
            &[3, 5, 3, 5, 3, 5, 3, 5],
            "Line 51 Col 1 must be alternating Cyan/Green"
        );

        // Finish line 51 and run line 52 up to col 0 (RC = 1)
        for _ in 18..63 {
            h.tick(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // col 0 on line 52

        let fb = h.vic.framebuffer();
        let l52 = 52 * 504;
        assert_eq!(
            &fb[l52 + 128..l52 + 136],
            &[6u8; 8],
            "Line 52 Col 0 must be Blue from address $2001 (RC 1)"
        );
    }

    #[test]
    fn multicolor_bitmap_renders_four_color_sources_with_double_width_pixels() {
        let mut h = TestHarness::new();

        // 1. Enable Multicolor Bitmap Mode: BMM=1, MCM=1
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x16, h.vic.read(0x16) | 0x10); // MCM = 1
        h.vic.write(0x18, 0x18); // VM $0400, Bitmap $2000

        // 2. Setup the four color sources:
        // 00 -> $D021
        h.vic.write(0x21, 0x00); // 0 (Black)
        // 01 -> VM high nibble; 10 -> VM low nibble
        h.ram[0x0400] = 0x25; // 01 = 2 (Red), 10 = 5 (Green)
        // 11 -> Color RAM
        h.color_ram[0] = 0x07; // 11 = 7 (Yellow)

        // 3. Setup Bitmap byte at $2000:
        // Bit pairs: [00, 01, 10, 11] -> 0b00_01_10_11 = 0x1B
        h.ram[0x2000] = 0b00_01_10_11;

        // 4. Advance beam to line 51, cycle 16
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // render column 0

        let fb = h.vic.framebuffer();
        let pixel_start = 51 * 504 + 128;
        let rendered = &fb[pixel_start..pixel_start + 8];

        // Each bit-pair produces 2 adjacent pixels:
        // pair 0 (00) -> [0, 0]
        // pair 1 (01) -> [2, 2]
        // pair 2 (10) -> [5, 5]
        // pair 3 (11) -> [7, 7]
        let expected = [0, 0, 2, 2, 5, 5, 7, 7];
        assert_eq!(
            rendered, expected,
            "Multicolor bitmap must expand bit-pairs into double-width pixels using all 4 color sources"
        );
    }

    #[test]
    fn multicolor_bitmap_masks_color_ram_to_low_nibble() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x16, h.vic.read(0x16) | 0x10); // MCM = 1
        h.vic.write(0x18, 0x18);

        h.ram[0x0400] = 0x00;
        // High bits in Color RAM must be ignored:
        h.color_ram[0] = 0xF3; // Color 3 (Cyan) with noisy upper nibble
        h.ram[0x2000] = 0b11_11_11_11; // All pair '11'

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        assert_eq!(rendered, &[3u8; 8], "Color RAM high nibble must be masked");
    }

    #[test]
    fn bank_0_bitmap_at_0000_reads_char_rom_for_row_groups_in_overlay() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x16, h.vic.read(0x16) & !0x10); // MCM = 0
        h.vic.write(0x18, 0x10); // VM $0400, Bitmap $0000

        // Row group 12 (VC = 512, row line 0) is at offset: 512 * 8 = $1000.
        // In Bank 0, $1000 is inside Character ROM overlay!
        h.char_rom[0x0000] = 0b10101010; // ROM overlay content at offset 0
        h.ram[0x1000] = 0xFF; // Decoy in RAM behind ROM overlay

        // Video matrix byte for VC 512:
        h.ram[0x0400 + 512] = 0x27; // FG=2, BG=7

        // Row 12 starts at line: 51 + (12 * 8) = line 147
        for _ in 0..147 {
            h.tick_scanline(0);
        }
        // Advance to column 32 (VC = 40 * 12 + 32 = 512): cycle 16 + 32 = cycle 48
        for _ in 0..48 {
            h.tick(0);
        }
        h.tick(0); // display column 32

        let fb = h.vic.framebuffer();
        let pixel_start = 147 * 504 + 48 * 8;
        let rendered = &fb[pixel_start..pixel_start + 8];

        assert_eq!(
            rendered,
            &[2, 7, 2, 7, 2, 7, 2, 7],
            "Bitmap at $0000 in Bank 0 must read from Character ROM at $1000..$1FFF"
        );
    }

    #[test]
    fn bank_1_bitmap_at_0000_reads_ram_at_5000_without_overlay() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x18, 0x10); // VM $0400 (Bank 1: $4400), Bitmap $0000 (Bank 1: $4000)

        // In Bank 1, VIC offset $1000 is physical RAM $5000 (no ROM overlay!)
        h.ram[0x5000] = 0b11110000;
        h.char_rom[0x0000] = 0xAA; // Decoy in Character ROM
        h.ram[0x4400 + 512] = 0x36; // FG=3, BG=6

        // Line 147, column 32 (VC = 512)
        for _ in 0..147 {
            h.tick_scanline(1);
        }
        for _ in 0..48 {
            h.tick(1);
        }
        h.tick(1);

        let fb = h.vic.framebuffer();
        let pixel_start = 147 * 504 + 48 * 8;
        let rendered = &fb[pixel_start..pixel_start + 8];

        assert_eq!(
            rendered,
            &[3, 3, 3, 3, 6, 6, 6, 6],
            "Bank 1 must read physical RAM at $5000 without Character ROM interference"
        );
    }

    #[test]
    fn switching_between_text_and_bitmap_mid_frame() {
        let mut h = TestHarness::new();
        h.vic.write(0x18, 0x18); // VM $0400, Charset $2000 / Bitmap $2000
        h.vic.write(0x21, 6); // BG Blue

        // Setup both Text and Bitmap data at cell (0, 0):
        h.ram[0x0400] = 0x01; // Text mode: Screen code 1. Bitmap mode: FG=0, BG=1.
        h.color_ram[0] = 2; // Text mode: Red foreground

        // In Text mode, Screen code 1 at char_base $2000 is at $2008
        h.ram[0x2008] = 0b11110000;
        // In Bitmap mode, cell (0, 0) row 0 is at $2000
        h.ram[0x2000] = 0b00001111;

        // Line 51: Text Mode (BMM = 0)
        h.vic.write(0x11, h.vic.read(0x11) & !0x20);
        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // col 0 text mode

        let fb = h.vic.framebuffer();
        let l51 = 51 * 504 + 128;
        assert_eq!(
            &fb[l51..l51 + 8],
            &[2, 2, 2, 2, 6, 6, 6, 6],
            "Line 51 must render text mode glyph"
        );

        // Switch to Standard Bitmap Mode during line 51:
        h.vic.write(0x11, h.vic.read(0x11) | 0x20);

        // Advance to line 52 (RC = 1). Bitmap address is $2001.
        h.ram[0x2001] = 0b11001100;
        for _ in 17..63 {
            h.tick(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // col 0 bitmap mode

        let fb = h.vic.framebuffer();
        let l52 = 52 * 504 + 128;
        // VM byte $0400 is 0x01: FG=0, BG=1
        assert_eq!(
            &fb[l52..l52 + 8],
            &[0, 0, 1, 1, 0, 0, 1, 1],
            "Line 52 must render bitmap mode pixels following dynamic mode switch"
        );
    }

    #[test]
    fn csel0_38_column_window_clips_bitmap_mode() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 14); // Border Light Blue
        h.vic.write(0x11, h.vic.read(0x11) | 0x20); // BMM = 1
        h.vic.write(0x16, 0x00); // CSEL=0: 38 columns
        h.vic.write(0x18, 0x18);

        h.ram[0x0400] = 0x12; // col 0: FG=1, BG=2
        h.ram[0x0401] = 0x34; // col 1: FG=3, BG=4
        h.ram[0x2000] = 0xFF;
        h.ram[0x2008] = 0xFF;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // Cycle 16: must be border in 38-column mode!
        h.tick(0); // Cycle 17: Column 1 displayed

        let fb = h.vic.framebuffer();
        let l51 = 51 * 504;
        for x in 128..136 {
            assert_eq!(
                fb[l51 + x],
                14,
                "Cycle 16 (Col 0) must be clipped to border color in 38-column bitmap mode"
            );
        }
        assert_eq!(
            &fb[l51 + 136..l51 + 144],
            &[3u8; 8],
            "Cycle 17 (Col 1) must display bitmap graphics"
        );
    }

    #[test]
    fn bad_line_ba_bus_stealing_occurs_in_bitmap_mode() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, (h.vic.read(0x11) | 0x20) & !0x07); // BMM = 1, YSCROLL = 0

        // Line 0x30 is a Bad Line
        for _ in 0..0x30 {
            h.tick_scanline(0);
        }
        for _ in 0..12 {
            h.tick(0);
            assert!(h.vic.ba());
        }
        h.tick(0);
        assert!(!h.vic.ba(), "BA must drop at cycle 12 in bitmap mode");

        for _ in 13..=54 {
            h.tick(0);
            assert!(!h.vic.ba());
        }
        h.tick(0);
        assert!(h.vic.ba(), "BA must return high at cycle 55 in bitmap mode");
    }

    // =========================================================================
    // MULTICOLOR TEXT MODE (ECM=0, BMM=0, MCM=1)
    // =========================================================================

    #[test]
    fn multicolor_text_renders_standard_glyph_when_color_ram_bit3_is_zero() {
        let mut h = TestHarness::new();
        // Enable Multicolor Text Mode: BMM=0, MCM=1, ECM=0
        h.vic.write(0x11, h.vic.read(0x11) & !0x60);
        h.vic.write(0x16, h.vic.read(0x16) | 0x10);

        h.vic.write(0x21, 6); // Background 0 = Dark Blue (6)
        h.color_ram[0] = 0x02; // Bit 3 is 0 -> Standard character! Color = Red (2)
        h.ram[0x0400] = 1; // Screen code 1
        h.char_rom[8] = 0b10101010;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // display column 0

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        assert_eq!(
            rendered,
            &[2, 6, 2, 6, 2, 6, 2, 6],
            "Multicolor text with Color RAM bit 3 = 0 must render as standard 8-pixel character"
        );
    }

    #[test]
    fn multicolor_text_renders_multicolor_glyph_when_color_ram_bit3_is_one() {
        let mut h = TestHarness::new();
        // Enable Multicolor Text Mode: BMM=0, MCM=1, ECM=0
        h.vic.write(0x11, h.vic.read(0x11) & !0x60);
        h.vic.write(0x16, h.vic.read(0x16) | 0x10);

        // Setup the 4 color sources:
        h.vic.write(0x21, 0); // 00 -> Background 0 = Black (0)
        h.vic.write(0x22, 2); // 01 -> Background 1 = Red (2)
        h.vic.write(0x23, 5); // 10 -> Background 2 = Green (5)
        // Bit 3 is 1 -> Multicolor! Bits 0..2 = 3 (Cyan) -> 11
        h.color_ram[0] = 0x0B;

        h.ram[0x0400] = 1;
        // 4 bit pairs: 00, 01, 10, 11
        h.char_rom[8] = 0b00_01_10_11;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        let expected = [0, 0, 2, 2, 5, 5, 3, 3];
        assert_eq!(
            rendered, expected,
            "Multicolor text must expand bit pairs into double-width pixels using BG0..BG2 and Color RAM bits 0..2"
        );
    }

    #[test]
    fn multicolor_text_foreground_color_masks_bit_3() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) & !0x60);
        h.vic.write(0x16, h.vic.read(0x16) | 0x10);

        // Color RAM has 0x0E (bit 3 = 1 -> multicolor; bits 0..2 = 6, Dark Blue)
        // If bit 3 was not masked, color would be 14 (Light Blue) instead of 6
        h.color_ram[0] = 0x0E;
        h.ram[0x0400] = 1;
        h.char_rom[8] = 0b11_11_11_11; // All pair '11'

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        assert_eq!(
            rendered, &[6u8; 8],
            "Multicolor text pair 11 must use Color RAM bits 0..2 only"
        );
    }

    #[test]
    fn multicolor_text_adjacent_standard_and_multicolor_characters() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, h.vic.read(0x11) & !0x60);
        h.vic.write(0x16, h.vic.read(0x16) | 0x10);

        h.vic.write(0x21, 0); // BG 0 = Black
        h.vic.write(0x22, 2); // BG 1 = Red
        h.vic.write(0x23, 5); // BG 2 = Green

        // Column 0: Standard character (bit 3 = 0, color = Yellow 7)
        h.color_ram[0] = 0x07;
        h.ram[0x0400] = 1;
        h.char_rom[8] = 0b11110000;

        // Column 1: Multicolor character (bit 3 = 1, FG = Cyan 3)
        h.color_ram[1] = 0x0B;
        h.ram[0x0401] = 2;
        h.char_rom[16] = 0b01_10_11_00;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // col 0
        h.tick(0); // col 1

        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        assert_eq!(
            &fb[line + 128..line + 136],
            &[7, 7, 7, 7, 0, 0, 0, 0],
            "Col 0 must render as standard character"
        );
        assert_eq!(
            &fb[line + 136..line + 144],
            &[2, 2, 5, 5, 3, 3, 0, 0],
            "Col 1 must render as multicolor character"
        );
    }

    // =========================================================================
    // EXTENDED COLOR MODE TEXT (ECM=1, BMM=0, MCM=0)
    // =========================================================================

    #[test]
    fn ecm_text_selects_background_color_via_screen_code_bits_6_and_7() {
        let mut h = TestHarness::new();
        // Enable ECM Text Mode: ECM=1, BMM=0, MCM=0
        h.vic.write(0x11, (h.vic.read(0x11) | 0x40) & !0x20);
        h.vic.write(0x16, h.vic.read(0x16) & !0x10);

        // Background color registers 0..3:
        h.vic.write(0x21, 0); // 00 -> Black
        h.vic.write(0x22, 2); // 01 -> Red
        h.vic.write(0x23, 5); // 10 -> Green
        h.vic.write(0x24, 7); // 11 -> Yellow

        // All 4 columns use White foreground
        for col in 0..4 {
            h.color_ram[col] = 1;
        }

        // Screen codes:
        // Col 0: %00_000000 -> BG 0 (0)
        // Col 1: %01_000000 -> BG 1 (2)
        // Col 2: %10_000000 -> BG 2 (5)
        // Col 3: %11_000000 -> BG 3 (7)
        h.ram[0x0400] = 0;
        h.ram[0x0401] = 64;
        h.ram[0x0402] = 128;
        h.ram[0x0403] = 192;

        // All 4 cells display character 0: left 4 pixels FG, right 4 pixels BG
        h.char_rom[0] = 0b11110000;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        for _ in 0..4 {
            h.tick(0); // render cols 0..3
        }

        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        assert_eq!(
            &fb[line + 128..line + 136],
            &[1, 1, 1, 1, 0, 0, 0, 0],
            "Col 0 must use BG0 (Black)"
        );
        assert_eq!(
            &fb[line + 136..line + 144],
            &[1, 1, 1, 1, 2, 2, 2, 2],
            "Col 1 must use BG1 (Red)"
        );
        assert_eq!(
            &fb[line + 144..line + 152],
            &[1, 1, 1, 1, 5, 5, 5, 5],
            "Col 2 must use BG2 (Green)"
        );
        assert_eq!(
            &fb[line + 152..line + 160],
            &[1, 1, 1, 1, 7, 7, 7, 7],
            "Col 3 must use BG3 (Yellow)"
        );
    }

    #[test]
    fn ecm_text_restricts_character_set_to_64_characters() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, (h.vic.read(0x11) | 0x40) & !0x20);
        h.vic.write(0x16, h.vic.read(0x16) & !0x10);

        h.vic.write(0x21, 0); // BG 0 = Black
        h.vic.write(0x22, 2); // BG 1 = Red
        h.color_ram[0] = 1; // FG = White

        // Screen code 65 (%01_000001):
        // Bits 6-7 select BG 1 (Red).
        // Bits 0-5 select character 1 ('A')!
        h.ram[0x0400] = 65;

        // Character 1 glyph:
        h.char_rom[8] = 0b10101010;
        // Character 65 in ROM (at 65 * 8 = 520) contains decoy data:
        h.char_rom[520] = 0xFF;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        let rendered = &fb[51 * 504 + 128..51 * 504 + 136];
        assert_eq!(
            rendered,
            &[1, 2, 1, 2, 1, 2, 1, 2],
            "ECM screen code 65 must wrap to character 1 and use BG 1"
        );
    }

    #[test]
    fn ecm_text_uses_full_4_bit_color_ram_for_foreground() {
        let mut h = TestHarness::new();
        h.vic.write(0x11, (h.vic.read(0x11) | 0x40) & !0x20);
        h.vic.write(0x16, h.vic.read(0x16) & !0x10);

        h.vic.write(0x21, 0);
        // Foreground color = Light Blue (14). Bit 3 is 1; in ECM, all 16 colors are legal!
        h.color_ram[0] = 14;
        h.ram[0x0400] = 0;
        h.char_rom[0] = 0xFF;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        assert_eq!(
            &fb[51 * 504 + 128..51 * 504 + 136],
            &[14u8; 8],
            "ECM text allows all 16 colors for foreground"
        );
    }

    // =========================================================================
    // INVALID DISPLAY MODES (ECM + BMM or ECM + MCM)
    // =========================================================================

    #[test]
    fn invalid_bitmap_mode_ecm_with_bmm_renders_black_pixels() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 2); // Border = Red
        // Enable ECM + BMM (Invalid Bitmap mode)
        h.vic.write(0x11, h.vic.read(0x11) | 0x60); // BMM=1, ECM=1
        h.vic.write(0x16, h.vic.read(0x16) & !0x10); // MCM=0

        h.ram[0x0400] = 0xFF; // Video matrix
        h.color_ram[0] = 0x01;
        h.ram[0x2000] = 0xFF;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0); // display column 0

        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        assert_eq!(
            &fb[line + 128..line + 136],
            &[0u8; 8],
            "Invalid mode ECM+BMM must display black pixels (color 0)"
        );
        // Border must remain unaffected
        assert_eq!(fb[line + 0], 2, "Border must remain normal color");
    }

    #[test]
    fn invalid_text_mode_ecm_with_mcm_renders_black_pixels() {
        let mut h = TestHarness::new();
        h.vic.write(0x20, 5); // Border = Green
        // Enable ECM + MCM with BMM=0 (Invalid Text mode)
        h.vic.write(0x11, (h.vic.read(0x11) | 0x40) & !0x20); // ECM=1, BMM=0
        h.vic.write(0x16, h.vic.read(0x16) | 0x10); // MCM=1

        h.ram[0x0400] = 1;
        h.color_ram[0] = 1;
        h.char_rom[8] = 0xFF;

        for _ in 0..=50 {
            h.tick_scanline(0);
        }
        for _ in 0..16 {
            h.tick(0);
        }
        h.tick(0);

        let fb = h.vic.framebuffer();
        let line = 51 * 504;
        assert_eq!(
            &fb[line + 128..line + 136],
            &[0u8; 8],
            "Invalid mode ECM+MCM must display black pixels (color 0)"
        );
        assert_eq!(fb[line + 0], 5, "Border must remain normal color");
    }
}
