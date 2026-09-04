use std::cell::Cell;

pub const IRQ_RASTER: u8 = 1 << 0;
pub const IRQ_SPRITE_BG: u8 = 1 << 1;
pub const IRQ_SPRITE_SPRITE: u8 = 1 << 2;
pub const IRQ_LIGHTPEN: u8 = 1 << 3;

const SCROLY: usize = 0x11;
const RASTER: usize = 0x12;
const LPENX: usize = 0x13;
const LPENY: usize = 0x14;
const SCROLX: usize = 0x16;
const VMCSB: usize = 0x18;
const VICIRCL: usize = 0x19;
const IRQMSK: usize = 0x1A;
const SPSPCL: usize = 0x1E;
const SPBGCL: usize = 0x1F;
const EXTCOL: usize = 0x20;
const BGCOL0: usize = 0x21;
const BGCOL1: usize = 0x22;
const BGCOL2: usize = 0x23;
const BGCOL3: usize = 0x24;
const SPMC0: usize = 0x25;
const SPMC1: usize = 0x26;
const SP0COL: usize = 0x27;
const SP1COL: usize = 0x28;
const SP2COL: usize = 0x29;
const SP3COL: usize = 0x2A;
const SP4COL: usize = 0x2B;
const SP5COL: usize = 0x2C;
const SP6COL: usize = 0x2D;
const SP7COL: usize = 0x2E;

pub struct Registers {
    values: [u8; 47],
    irq_flags: u8,
    sprite_sprite_collision: Cell<u8>,
    sprite_bg_collision: Cell<u8>,
    lightpen_latched: bool,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            values: [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x9B, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x15, 0x0F, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x0E, 0x06, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03,
                0x04, 0x05, 0x06, 0x07, 0x0C,
            ],
            irq_flags: 0,
            sprite_sprite_collision: Cell::new(0),
            sprite_bg_collision: Cell::new(0),
            lightpen_latched: false,
        }
    }

    /// Read a register offset ($00..=$3F). Needs current raster line from the beam.
    pub fn read(&self, offset: u8, current_line: u16) -> u8 {
        let addr = (offset & 0b111111) as usize;
        match addr {
            SCROLY => (self.values[addr as usize] & 0x7F) | ((current_line & 0x100) >> 1) as u8,
            RASTER => (current_line & 0xFF) as u8,
            SCROLX => self.values[addr as usize] | 0xC0,
            VMCSB => self.values[addr as usize] | 0x01,
            VICIRCL => self.irq_flags | 0x70 | if self.is_irq_asserted() { 0x80 } else { 0x00 },
            IRQMSK => self.values[addr as usize] | 0xF0,
            SPSPCL => self.sprite_sprite_collision.replace(0),
            SPBGCL => self.sprite_bg_collision.replace(0),
            EXTCOL | BGCOL0..=BGCOL3 | SPMC0..=SPMC1 | SP0COL..=SP7COL => 0xF0 | self.values[addr],
            _ if addr < 47 => self.values[addr],
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, offset: u8, value: u8) {
        let addr = (offset & 0b111111) as usize;
        match addr {
            LPENX | LPENY | SPSPCL | SPBGCL => {}
            VICIRCL => self.irq_flags &= !value,
            EXTCOL | BGCOL0..=BGCOL3 | SPMC0..=SPMC1 | SP0COL..=SP7COL => {
                self.values[addr] = value & 0x0F;
            }
            _ if addr < 47 => self.values[addr] = value,
            _ => {}
        }
    }

    /// Checks if current line matches raster compare and sets IRQ flag if true.
    pub fn check_raster_match(&mut self, current_line: u16) {
        if current_line == self.raster_compare() {
            self.irq_flags |= IRQ_RASTER;
        }
    }

    /// Returns true if any enabled interrupt is active.
    pub fn is_irq_asserted(&self) -> bool {
        self.values[IRQMSK] & self.irq_flags != 0
    }

    pub fn raster_compare(&self) -> u16 {
        (self.values[RASTER] as u16) | (((self.values[SCROLY] >> 7) as u16) << 8)
    }

    pub fn border_color(&self) -> u8 {
        self.values[EXTCOL]
    }

    pub fn background_colors(&self, idx: usize) -> u8 {
        self.values[BGCOL0 + idx]
    }

    pub fn sprite_colors(&self, idx: usize) -> u8 {
        self.values[SP0COL + idx]
    }

    pub fn set_sprite_sprite_collision(&mut self, value: u8) {
        if value & !self.sprite_sprite_collision.get() != 0 {
            self.irq_flags |= IRQ_SPRITE_SPRITE;
        }
        *self.sprite_sprite_collision.get_mut() |= value;
    }

    pub fn set_sprite_bg_collision(&mut self, value: u8) {
        if value & !self.sprite_bg_collision.get() != 0 {
            self.irq_flags |= IRQ_SPRITE_BG;
        }
        *self.sprite_bg_collision.get_mut() |= value;
    }

    pub fn yscroll(&self) -> u8 {
        self.values[SCROLY] & 0x07
    }

    pub fn rsel(&self) -> bool {
        self.values[SCROLY] & 0x08 != 0
    }

    pub fn den(&self) -> bool {
        self.values[SCROLY] & 0x10 != 0
    }

    pub fn bmm(&self) -> bool {
        self.values[SCROLY] & 0x20 != 0
    }

    pub fn ecm(&self) -> bool {
        self.values[SCROLY] & 0x40 != 0
    }

    pub fn xscroll(&self) -> u8 {
        self.values[SCROLX] & 0x07
    }

    pub fn csel(&self) -> bool {
        self.values[SCROLX] & 0x08 != 0
    }

    pub fn mcm(&self) -> bool {
        self.values[SCROLX] & 0x10 != 0
    }

    pub fn char_base(&self) -> u16 {
        let bits = (self.values[VMCSB] & 0x0E) >> 1;
        bits as u16 * 0x800
    }

    pub fn video_matrix_base(&self) -> u16 {
        let bits = self.values[VMCSB] >> 4;
        bits as u16 * 0x400
    }

    pub fn latch_lightpen(&mut self, x: u8, line: u16) {
        if !self.lightpen_latched {
            self.lightpen_latched = true;
            self.values[LPENX] = x;
            self.values[LPENY] = (line & 0xFF) as u8;
            self.irq_flags |= IRQ_LIGHTPEN;
        }
    }

    pub fn new_frame(&mut self) {
        self.lightpen_latched = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. RESET / POWER-ON DEFAULTS
    //    * Registers initialize to power-on standard values:
    //      - $D011: $9B (%10011011) -> 25-row, DEN=1, YSCROLL=3, Compare bit 8 = 1
    //      - $D016: $08 (%00001000) -> 40-col, XSCROLL=0
    //      - $D018: $14 (%00010100) -> VM=$0400, CB=$1000
    //      - $D020: $0E (Light Blue)
    //      - $D021: $06 (Blue)
    //      - IRQ flags and mask: $00
    //
    // 2. RASTER BEAM vs. RASTER COMPARE ($D011 / $D012)
    //    * WRITING $D012 sets lower 8 bits of raster compare.
    //    * WRITING $D011 bit 7 sets bit 8 of raster compare.
    //    * READING $D012 returns lower 8 bits of CURRENT BEAM LINE (not compare).
    //    * READING $D011 bit 7 returns bit 8 of CURRENT BEAM LINE (not compare).
    //    * check_raster_match(line): latches IRQ_RASTER in $D019 when line == compare.
    //
    // 3. INTERRUPTS ($D019 / $D01A)
    //    * Writing 1 to $D019 bits 0-3 CLEARS that flag.
    //    * Writing 0 to $D019 has NO effect.
    //    * Bits 4-6 of $D019 always read as 1 (%01110000 = $70).
    //    * Bit 7 of $D019 is 1 iff (irq_flags & irq_mask & 0x0F) != 0.
    //    * Bits 4-7 of $D01A always read as 1 ($F0).
    //    * is_irq_asserted() is true iff (irq_flags & irq_mask & 0x0F) != 0.
    //
    // 4. COLOR REGISTERS ($D020–$D02E)
    //    * Only lower 4 bits are connected.
    //    * Upper 4 bits are ignored on write and ALWAYS read back as 1 ($F0).
    //
    // 5. UNCONNECTED BITS AND REGISTERS
    //    * $D016 bits 6-7 always read as 1 ($C0).
    //    * $D018 bit 0 always reads as 1 ($01).
    //    * $D02F..=$D03F are unconnected and always read as $FF; writes ignored.
    //
    // 6. CLEAR-ON-READ REGISTERS ($D01E, $D01F)
    //    * Reading $D01E (sprite-sprite collision) clears it to $00.
    //    * Reading $D01F (sprite-bg collision) clears it to $00.
    //
    // 7. HELPER ACCESSORS
    //    * yscroll(), xscroll(), den(), rsel(), csel(), bmm(), ecm(), mcm().
    //    * video_matrix_base(): (VM bits >> 4) * 1024.
    //    * char_base(): bits 1-3 of $D018 are CB13..CB11 ->
    //      ((v & $0E) >> 1) * $0800, 2K-aligned, 8 bases $0000..$3800.
    //    * latch_lightpen(x, line): LPENX <- x, LPENY <- line & $FF,
    //      latches IRQ_LIGHTPEN. First trigger per frame only;
    //      new_frame() re-arms the latch (and does nothing else).
    //    * set_sprite_*_collision(v): OR into the latch AND latch the
    //      matching $D019 flag on any 0->1 bit transition. Flag and
    //      latch clear independently (read vs. $D019 acknowledge).
    // =====================================================================

    // ---------------------------------------------------------------------
    // 1. Reset / Power-on Defaults
    // ---------------------------------------------------------------------

    #[test]
    fn power_on_defaults_match_hardware() {
        let r = Registers::new();

        assert_eq!(r.read(0x11, 0), 0x1B, "$D011 default (with line 0 MSB=0)");
        assert_eq!(r.read(0x16, 0), 0xC8, "$D016 default ($08 | $C0 unused)");
        assert_eq!(r.read(0x18, 0), 0x15, "$D018 default ($14 | $01 unused)");
        assert_eq!(
            r.read(0x19, 0),
            0x70,
            "$D019 default (flags 0 | $70 unused)"
        );
        assert_eq!(r.read(0x1A, 0), 0xF0, "$D01A default (mask 0 | $F0 unused)");

        assert_eq!(r.read(0x20, 0), 0xFE, "Border color ($0E | $F0)");
        assert_eq!(r.read(0x21, 0), 0xF6, "Background 0 ($06 | $F0)");
        assert!(!r.is_irq_asserted());
    }

    // ---------------------------------------------------------------------
    // 2. Raster Compare & Current Line Split ($D011 / $D012)
    // ---------------------------------------------------------------------

    #[test]
    fn raster_compare_write_and_read_isolation() {
        let mut r = Registers::new();

        // Write compare value 311 ($137) -> $D012 = $37, $D011 bit 7 = 1
        r.write(0x12, 0x37);
        r.write(0x11, 0x80);

        assert_eq!(r.raster_compare(), 311);

        // Reading $D012 and $D011 must reflect CURRENT LINE (e.g. line 50 = $032), NOT 311!
        let current_line = 50u16;
        assert_eq!(r.read(0x12, current_line), 0x32);
        assert_eq!(
            r.read(0x11, current_line) & 0x80,
            0x00,
            "Bit 7 must be 0 for line 50"
        );

        // Current line 300 ($12C) -> MSB must be 1
        let current_line_high = 300u16;
        assert_eq!(r.read(0x12, current_line_high), 0x2C);
        assert_eq!(
            r.read(0x11, current_line_high) & 0x80,
            0x80,
            "Bit 7 must be 1 for line 300"
        );
    }

    #[test]
    fn check_raster_match_triggers_flag() {
        let mut r = Registers::new();

        // Set compare to line 150
        r.write(0x12, 150);
        r.write(0x11, !0x80);

        r.check_raster_match(149);
        assert_eq!(r.irq_flags & IRQ_RASTER, 0);

        r.check_raster_match(150);
        assert_eq!(r.irq_flags & IRQ_RASTER, IRQ_RASTER);
    }

    // ---------------------------------------------------------------------
    // 3. Interrupt Flags ($D019) and Mask ($D01A)
    // ---------------------------------------------------------------------

    #[test]
    fn irq_flag_write_1_to_clear_logic() {
        let mut r = Registers::new();
        r.irq_flags = IRQ_RASTER | IRQ_LIGHTPEN; // bits 0 and 3 set

        // Writing 0 has no effect
        r.write(0x19, 0x00);
        assert_eq!(r.irq_flags, IRQ_RASTER | IRQ_LIGHTPEN);

        // Writing 1 to bit 0 clears bit 0, leaves bit 3 intact
        r.write(0x19, 0x01);
        assert_eq!(r.irq_flags, IRQ_LIGHTPEN);

        // Writing 1 to bit 3 clears bit 3
        r.write(0x19, 0x08);
        assert_eq!(r.irq_flags, 0x00);
    }

    #[test]
    fn irq_flag_bit_7_reflects_active_interrupt() {
        let mut r = Registers::new();

        // Flag set, but mask disabled -> Bit 7 must be 0
        r.irq_flags = IRQ_RASTER;
        r.write(0x1A, 0x00); // Mask disabled
        assert_eq!(r.read(0x19, 0) & 0x80, 0x00);
        assert!(!r.is_irq_asserted());

        // Enable mask for raster IRQ -> Bit 7 must become 1
        r.write(0x1A, IRQ_RASTER);
        assert_eq!(r.read(0x19, 0) & 0x80, 0x80);
        assert!(r.is_irq_asserted());

        // Clear flag -> Bit 7 drops to 0
        r.write(0x19, IRQ_RASTER);
        assert_eq!(r.read(0x19, 0) & 0x80, 0x00);
        assert!(!r.is_irq_asserted());
    }

    #[test]
    fn irq_registers_floating_bits() {
        let r = Registers::new();

        // $D019 bits 4-6 always read as 1
        assert_eq!(r.read(0x19, 0) & 0x70, 0x70);

        // $D01A bits 4-7 always read as 1
        assert_eq!(r.read(0x1A, 0) & 0xF0, 0xF0);
    }

    // ---------------------------------------------------------------------
    // 4. Color Registers ($D020–$D02E)
    // ---------------------------------------------------------------------

    #[test]
    fn color_registers_mask_upper_4_bits() {
        let mut r = Registers::new();

        // Border color ($D020)
        r.write(0x20, 0xAB); // Try writing upper nibble 0xA
        assert_eq!(r.border_color(), 0x0B);
        assert_eq!(r.read(0x20, 0), 0xFB, "Read must return $F0 | color");

        // Background colors 0..=3 ($D021..=$D024)
        for i in 0..4 {
            let reg = 0x21 + i;
            r.write(reg, 0x70 | i);
            assert_eq!(r.background_colors(i as usize), i);
            assert_eq!(r.read(reg, 0), 0xF0 | i);
        }

        // Sprite colors ($D027..=$D02E)
        for i in 0..8 {
            let reg = 0x27 + i;
            r.write(reg, 0xE0 | (i + 1));
            assert_eq!(r.sprite_colors(i as usize), i + 1);
            assert_eq!(r.read(reg, 0), 0xF0 | (i + 1));
        }
    }

    // ---------------------------------------------------------------------
    // 5. Unconnected Registers and Bits
    // ---------------------------------------------------------------------

    #[test]
    fn unconnected_d016_and_d018_bits() {
        let mut r = Registers::new();

        r.write(0x16, 0x00);
        assert_eq!(r.read(0x16, 0) & 0xC0, 0xC0, "Bits 6-7 of $D016 must be 1");

        r.write(0x18, 0x00);
        assert_eq!(r.read(0x18, 0) & 0x01, 0x01, "Bit 0 of $D018 must be 1");
    }

    #[test]
    fn unconnected_registers_d02f_to_d03f_return_ff() {
        let mut r = Registers::new();

        for reg in 0x2F..=0x3F {
            r.write(reg, 0x00); // Writes must be ignored
            assert_eq!(
                r.read(reg, 0),
                0xFF,
                "Unconnected register 0x{:02X} must read $FF",
                reg
            );
        }
    }

    // ---------------------------------------------------------------------
    // 6. Clear-On-Read Collision Registers
    // ---------------------------------------------------------------------

    #[test]
    fn collision_registers_clear_on_read() {
        let mut r = Registers::new();

        r.set_sprite_sprite_collision(0x55);
        r.set_sprite_bg_collision(0xAA);

        // First read returns value
        assert_eq!(r.read(0x1E, 0), 0x55);
        assert_eq!(r.read(0x1F, 0), 0xAA);

        // Second read must be cleared to 0
        assert_eq!(r.read(0x1E, 0), 0x00);
        assert_eq!(r.read(0x1F, 0), 0x00);
    }

    // ---------------------------------------------------------------------
    // 7. Helper Accessors
    // ---------------------------------------------------------------------

    #[test]
    fn helper_getters_decode_d011_correctly() {
        let mut r = Registers::new();

        // YSCROLL = 5 (%101), RSEL = 0, DEN = 1, BMM = 1, ECM = 0
        r.write(0x11, 0b00110101);

        assert_eq!(r.yscroll(), 5);
        assert!(!r.rsel());
        assert!(r.den());
        assert!(r.bmm());
        assert!(!r.ecm());
    }

    #[test]
    fn helper_getters_decode_d016_correctly() {
        let mut r = Registers::new();

        // XSCROLL = 7 (%111), CSEL = 1, MCM = 1
        r.write(0x16, 0b00011111);

        assert_eq!(r.xscroll(), 7);
        assert!(r.csel());
        assert!(r.mcm());
    }

    #[test]
    fn helper_getters_decode_d018_memory_bases() {
        let mut r = Registers::new();

        // VM = %0001 -> $0400; CB bits 1-3 = %100 -> $2000
        r.write(0x18, 0x18);
        assert_eq!(r.video_matrix_base(), 0x0400);
        assert_eq!(r.char_base(), 0x2000);

        // VM = %1000 -> $2000; CB bits 1-3 = %010 -> $1000
        r.write(0x18, 0x84);
        assert_eq!(r.video_matrix_base(), 0x2000);
        assert_eq!(r.char_base(), 0x1000);

        // VM = %1111 -> $3C00; CB = %111 -> $3800
        r.write(0x18, 0xFE);
        assert_eq!(r.video_matrix_base(), 0x3C00);
        assert_eq!(r.char_base(), 0x3800);

        // Everything at $0000
        r.write(0x18, 0x01);
        assert_eq!(r.video_matrix_base(), 0x0000);
        assert_eq!(r.char_base(), 0x0000);
    }

    // ---------------------------------------------------------------------
    // 8. $D019 Polling Without Mask (Crucial for games)
    // ---------------------------------------------------------------------

    #[test]
    fn irq_flags_visible_on_read_even_when_mask_is_disabled() {
        let mut r = Registers::new();

        // Disable all masks in $D01A
        r.write(0x1A, 0x00);
        assert_eq!(r.read(0x1A, 0) & 0x0F, 0x00);

        // Hardware triggers raster match
        r.write(0x12, 100);
        r.write(0x11, 0x00);
        r.check_raster_match(100);

        // Bit 0 MUST be visible to the CPU for polling, even with mask = 0!
        // Bit 7 must remain 0 (no IRQ asserted).
        let d019 = r.read(0x19, 100);
        assert_eq!(
            d019 & IRQ_RASTER,
            IRQ_RASTER,
            "Flags must be readable even when mask is disabled (polling mode)"
        );
        assert_eq!(d019 & 0x80, 0x00, "Bit 7 must be 0 when unmasked");
        assert!(!r.is_irq_asserted());
    }

    // ---------------------------------------------------------------------
    // 9. Sprite Multicolor Registers 4-Bit Masking ($D025 / $D026)
    // ---------------------------------------------------------------------

    #[test]
    fn sprite_multicolor_registers_mask_upper_4_bits() {
        let mut r = Registers::new();

        for reg in [0x25, 0x26] {
            r.write(reg, 0xFE); // Write 0xE into color, 0xF into upper nibble
            assert_eq!(
                r.read(reg, 0),
                0xFE,
                "Register 0x{:02X} must read back with upper nibble as 0xF",
                reg
            );
            // Verify writing upper nibble noise does not corrupt the lower 4 bits
            r.write(reg, 0x73);
            assert_eq!(r.read(reg, 0), 0xF3);
        }
    }

    // ---------------------------------------------------------------------
    // 10. Read-Only Registers Ignore CPU Writes
    // ---------------------------------------------------------------------

    #[test]
    fn light_pen_registers_ignore_cpu_writes() {
        let mut r = Registers::new();

        // Writes to $D013 and $D014 must not alter the latched light pen values
        r.write(0x13, 0xAA);
        r.write(0x14, 0x55);

        assert_eq!(r.read(0x13, 0), 0x00, "LPX must ignore CPU writes");
        assert_eq!(r.read(0x14, 0), 0x00, "LPY must ignore CPU writes");
    }

    #[test]
    fn collision_registers_ignore_cpu_writes() {
        let mut r = Registers::new();

        r.set_sprite_sprite_collision(0x33);
        // CPU attempts to write to $D01E
        r.write(0x1E, 0xFF);

        // Reading $D01E must return the collision value 0x33 (and then clear)
        assert_eq!(
            r.read(0x1E, 0),
            0x33,
            "Collision register must ignore CPU write"
        );
        assert_eq!(r.read(0x1E, 0), 0x00);
    }

    // ---------------------------------------------------------------------
    // 11. 64-Byte Mirroring Across $D000–$D3FF
    // ---------------------------------------------------------------------

    #[test]
    fn registers_mirror_every_64_bytes() {
        let mut r = Registers::new();

        // Write border color via mirror 1 ($D060 = $D020 + 0x40)
        r.write(0x60, 0x02);
        assert_eq!(r.border_color(), 0x02);
        assert_eq!(
            r.read(0x20, 0),
            0xF2,
            "Base address must reflect mirror write"
        );
        assert_eq!(r.read(0x60, 0), 0xF2, "Mirror 1 must reflect write");
        assert_eq!(r.read(0xA0, 0), 0xF2, "Mirror 2 ($D0A0) must reflect write");
        assert_eq!(r.read(0xE0, 0), 0xF2, "Mirror 3 ($D0E0) must reflect write");

        // Unconnected space ($D02F..=$D03F) mirrored at $D06F
        assert_eq!(r.read(0x6F, 0), 0xFF);
    }

    // ---------------------------------------------------------------------
    // 12. Sprite Coordinates Round-Trip ($D000–$D010)
    // ---------------------------------------------------------------------

    #[test]
    fn sprite_coordinates_round_trip() {
        let mut r = Registers::new();

        // Set positions for all 8 sprites
        for i in 0..8 {
            r.write(i * 2, 100 + i); // X coord
            r.write(i * 2 + 1, 50 + i); // Y coord
        }
        r.write(0x10, 0b10101010); // MSB for sprites 1, 3, 5, 7

        for i in 0..8 {
            assert_eq!(r.read(i * 2, 0), 100 + i);
            assert_eq!(r.read(i * 2 + 1, 0), 50 + i);
        }
        assert_eq!(r.read(0x10, 0), 0b10101010);
    }

    // -----------------------------------------------------------------
    // 13. Full power-on dump (pins every default; catches index drift)
    // -----------------------------------------------------------------

    #[test]
    fn power_on_defaults_full_register_dump() {
        let r = Registers::new();
        let expected: &[u8] = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // $D000-$D007
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // $D008-$D00F
            0x00, 0x1B, 0x00, 0x00, 0x00, 0x00, 0xC8, 0x00, // $D010-$D017
            0x15, 0x70, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, // $D018-$D01F
            0xFE, 0xF6, 0xF1, 0xF2, 0xF3, 0xF4, 0xF0, 0xF1, // $D020-$D027
            0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xFC, // $D028-$D02E
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(r.read(i as u8, 0), exp, "power-on default of ${:02X}", i);
        }
        for i in 0x2F..=0x3F {
            assert_eq!(r.read(i as u8, 0), 0xFF, "unconnected ${:02X}", i);
        }
    }

    #[test]
    fn power_on_default_decoded_state() {
        let r = Registers::new();
        assert_eq!(r.raster_compare(), 256, "$9B bit 7 + $D012=$00");
        assert!(r.den());
        assert!(r.rsel());
        assert_eq!(r.yscroll(), 3);
        assert!(!r.bmm());
        assert!(!r.ecm());
        assert_eq!(r.xscroll(), 0);
        assert!(r.csel());
        assert!(!r.mcm());
        assert_eq!(r.video_matrix_base(), 0x0400);
        assert_eq!(r.char_base(), 0x1000);
        assert_eq!(r.border_color(), 0x0E);
        assert_eq!(r.background_colors(0), 0x06);
        assert_eq!(r.background_colors(1), 0x01);
        assert_eq!(r.background_colors(2), 0x02);
        assert_eq!(r.background_colors(3), 0x03);
        assert_eq!(r.sprite_colors(0), 0x01);
        assert_eq!(r.sprite_colors(1), 0x02);
        assert_eq!(r.sprite_colors(2), 0x03);
        assert_eq!(r.sprite_colors(3), 0x04);
        assert_eq!(r.sprite_colors(4), 0x05);
        assert_eq!(r.sprite_colors(5), 0x06);
        assert_eq!(r.sprite_colors(6), 0x07);
        assert_eq!(r.sprite_colors(7), 0x0C);
    }

    #[test]
    fn sprite7_color_default_is_the_famous_quirk() {
        // Kernal default-table off-by-one: $D02E gets 'L' of "LOAD" ($4C);
        // only the low nibble is connected -> $0C.
        let r = Registers::new();
        assert_eq!(r.read(0x2E, 0), 0xFC);
    }

    // -----------------------------------------------------------------
    // 14. Raster compare / beam, exhaustive
    // -----------------------------------------------------------------

    #[test]
    fn raster_compare_full_9_bit_range() {
        let mut r = Registers::new();
        for cmp in 0u16..=511 {
            r.write(0x11, 0x00);
            r.write(0x12, (cmp & 0xFF) as u8);
            r.write(0x11, if cmp & 0x100 != 0 { 0x80 } else { 0x00 });
            assert_eq!(r.raster_compare(), cmp, "compare = {}", cmp);
        }
    }

    #[test]
    fn writing_d012_alone_keeps_bit8_from_d011() {
        let mut r = Registers::new();
        r.write(0x12, 0xE7); // default $D011 = $9B -> bit 8 already set
        assert_eq!(r.raster_compare(), 0x1E7);
        r.write(0x11, 0x1B); // clear only bit 8
        assert_eq!(r.raster_compare(), 0x0E7);
    }

    #[test]
    fn writing_d011_bit8_keeps_d012() {
        let mut r = Registers::new();
        r.write(0x12, 0x55);
        r.write(0x11, 0x93);
        assert_eq!(r.raster_compare(), 0x155);
        assert_eq!(r.yscroll(), 3);
        assert!(r.den());
        r.write(0x11, 0x13);
        assert_eq!(r.raster_compare(), 0x055);
    }

    #[test]
    fn raster_reads_track_beam_not_compare_exhaustive() {
        let mut r = Registers::new();
        r.write(0x11, 0x00);
        r.write(0x12, 0x00);
        let lines = [0u16, 1, 127, 128, 254, 255, 256, 257, 300, 311, 511];
        for &line in &lines {
            assert_eq!(r.read(0x12, line), (line & 0xFF) as u8, "line {}", line);
            assert_eq!(r.read(0x11, line) & 0x80, ((line & 0x100) >> 1) as u8);
        }
        // Even with compare = 511 latched in, reads still report the beam:
        r.write(0x11, 0x80);
        r.write(0x12, 0xFF);
        for &line in &lines {
            assert_eq!(r.read(0x12, line), (line & 0xFF) as u8, "line {}", line);
            assert_eq!(r.read(0x11, line) & 0x80, ((line & 0x100) >> 1) as u8);
        }
    }

    #[test]
    fn reading_raster_registers_does_not_disturb_compare() {
        let mut r = Registers::new();
        r.write(0x11, 0x80);
        r.write(0x12, 0xF7);
        assert_eq!(r.raster_compare(), 0x1F7);
        for line in 0u16..64 {
            let _ = r.read(0x11, line);
            let _ = r.read(0x12, line);
        }
        assert_eq!(r.raster_compare(), 0x1F7);
    }

    #[test]
    fn beam_position_reconstruction_9_bit() {
        let r = Registers::new();
        for line in [0u16, 100, 255, 256, 257, 310, 311] {
            let lo = r.read(0x12, line) as u16;
            let hi = (r.read(0x11, line) >> 7) as u16;
            assert_eq!(lo | (hi << 8), line, "9-bit beam reconstruction");
        }
    }

    #[test]
    fn raster_match_exhaustive_full_compare_range() {
        let mut r = Registers::new();
        for cmp in 0u16..=511 {
            r.write(0x11, if cmp & 0x100 != 0 { 0x80 } else { 0x00 });
            r.write(0x12, (cmp & 0xFF) as u8);
            r.irq_flags = 0;

            r.check_raster_match((cmp + 511) % 512); // one below (wrapping)
            assert_eq!(r.irq_flags & IRQ_RASTER, 0, "below compare {}", cmp);

            r.check_raster_match(cmp);
            assert_eq!(r.irq_flags & IRQ_RASTER, IRQ_RASTER, "at compare {}", cmp);

            r.irq_flags = 0;
            r.check_raster_match((cmp + 1) % 512);
            assert_eq!(r.irq_flags & IRQ_RASTER, 0, "above compare {}", cmp);
        }
    }

    #[test]
    fn raster_match_latches_only_the_raster_flag() {
        let mut r = Registers::new();
        r.write(0x11, 0x00);
        r.write(0x12, 42);
        r.check_raster_match(42);
        r.check_raster_match(42);
        r.check_raster_match(42);
        assert_eq!(r.irq_flags, IRQ_RASTER);
    }

    #[test]
    fn default_compare_is_line_256() {
        let mut r = Registers::new();
        r.check_raster_match(255);
        assert_eq!(r.irq_flags & IRQ_RASTER, 0);
        r.check_raster_match(256);
        assert_eq!(r.irq_flags & IRQ_RASTER, IRQ_RASTER);
    }

    #[test]
    fn classic_raster_irq_setup_and_acknowledge() {
        let mut r = Registers::new();
        r.write(0x1A, IRQ_RASTER);
        r.write(0x11, 0x1B);
        r.write(0x12, 0xF8);
        for line in 0u16..0xF8 {
            r.check_raster_match(line);
        }
        assert!(!r.is_irq_asserted());
        r.check_raster_match(0xF8);
        assert_eq!(r.read(0x19, 0xF8), 0xF1, "bit7 | $70 | IRQ_RASTER");
        // ISR acknowledges and moves the compare one line down
        r.write(0x19, IRQ_RASTER);
        r.write(0x12, 0xF9);
        assert_eq!(r.read(0x19, 0xF9), 0x70);
        r.check_raster_match(0xF9);
        assert!(r.is_irq_asserted());

        // The classic off-by-256 trap: same $D012 value, bit 8 set
        r.write(0x19, IRQ_RASTER);
        r.write(0x11, 0x9B);
        assert_eq!(r.raster_compare(), 0x1F9);
        r.check_raster_match(0xF9);
        assert!(!r.is_irq_asserted(), "must NOT fire on line $F9 anymore");
        r.check_raster_match(0x1F9);
        assert!(r.is_irq_asserted());
    }

    #[test]
    fn raster_polling_without_irq_latches_exactly_once() {
        let mut r = Registers::new();
        r.write(0x11, 0x00);
        r.write(0x12, 0x40);
        r.write(0x1A, 0x00); // pure polling, no IRQ
        let mut hits = 0;
        for line in 0u16..=0x40 {
            r.check_raster_match(line);
            if r.read(0x19, line) & IRQ_RASTER != 0 {
                hits += 1;
                r.write(0x19, IRQ_RASTER);
            }
        }
        assert_eq!(hits, 1);
        assert!(!r.is_irq_asserted());
    }

    // -----------------------------------------------------------------
    // 15. Interrupt flags and mask, exhaustive
    // -----------------------------------------------------------------

    #[test]
    fn irq_clear_exhaustive_all_256_write_values() {
        let mut r = Registers::new();
        for v in 0u8..=255 {
            r.irq_flags = 0x0F;
            r.write(0x19, v);
            assert_eq!(r.irq_flags, !v & 0x0F, "write ${:02X} to $D019", v);
        }
    }

    #[test]
    fn irq_flags_cannot_be_set_by_cpu_write() {
        let mut r = Registers::new();
        r.write(0x19, 0x0F);
        r.write(0x19, 0xFF);
        r.write(0x19, 0x81);
        assert_eq!(r.irq_flags, 0);
        assert_eq!(r.read(0x19, 0) & 0x0F, 0);
    }

    #[test]
    fn reading_d019_is_non_destructive() {
        let mut r = Registers::new();
        r.irq_flags = IRQ_LIGHTPEN | IRQ_SPRITE_SPRITE;
        for _ in 0..5 {
            assert_eq!(r.read(0x19, 0) & 0x0F, IRQ_LIGHTPEN | IRQ_SPRITE_SPRITE);
        }
        assert_eq!(r.irq_flags, IRQ_LIGHTPEN | IRQ_SPRITE_SPRITE);
    }

    #[test]
    fn d019_read_exhaustive_flags_and_mask_combinations() {
        let mut r = Registers::new();
        for flags in 0u8..=0x0F {
            for mask in 0u8..=0x0F {
                r.irq_flags = flags;
                r.write(0x1A, mask);
                let d = r.read(0x19, 0);
                assert_eq!(d & 0x0F, flags, "flags poll verbatim");
                assert_eq!(d & 0x70, 0x70, "bits 4-6 float high");
                assert_eq!(d & 0x80, if flags & mask != 0 { 0x80 } else { 0x00 });
                assert_eq!(r.is_irq_asserted(), flags & mask != 0);
            }
        }
    }

    #[test]
    fn irq_output_is_gated_combinationally_by_the_mask() {
        let mut r = Registers::new();
        r.irq_flags = IRQ_SPRITE_BG;
        r.write(0x1A, 0x00);
        assert!(!r.is_irq_asserted());
        // Raising the mask bit while the flag is already latched must assert
        // immediately — the gate is combinational, no edge required.
        r.write(0x1A, IRQ_SPRITE_BG);
        assert!(r.is_irq_asserted());
        assert_eq!(r.read(0x19, 0) & 0x80, 0x80);
        // Non-matching mask bits alone must not assert:
        r.write(0x1A, IRQ_RASTER | IRQ_LIGHTPEN);
        assert!(!r.is_irq_asserted());
    }

    #[test]
    fn each_mask_bit_enables_exactly_its_own_flag() {
        for bit in [IRQ_RASTER, IRQ_SPRITE_BG, IRQ_SPRITE_SPRITE, IRQ_LIGHTPEN] {
            let mut r = Registers::new();
            r.irq_flags = 0x0F;
            r.write(0x1A, bit);
            assert!(r.is_irq_asserted(), "mask bit {:#04X}", bit);
            r.write(0x19, bit);
            assert!(!r.is_irq_asserted(), "mask bit {:#04X}", bit);
            assert_eq!(r.irq_flags, 0x0F & !bit, "other flags stay latched");
        }
    }

    #[test]
    fn irq_mask_round_trips_and_upper_nibble_is_inert() {
        let mut r = Registers::new();
        for m in 0u8..=0x0F {
            r.write(0x1A, m);
            assert_eq!(r.read(0x1A, 0), m | 0xF0);
        }
        r.irq_flags = 0x0F;
        r.write(0x1A, 0xF0); // upper nibble set, lower nibble zero
        assert!(!r.is_irq_asserted());
        assert_eq!(r.read(0x1A, 0), 0xF0);
    }

    #[test]
    fn irq_acknowledge_works_via_mirror_address() {
        let mut r = Registers::new();
        r.irq_flags = IRQ_RASTER;
        r.write(0x59, IRQ_RASTER); // $D059 mirrors $D019
        assert_eq!(r.irq_flags, 0);
    }

    // -----------------------------------------------------------------
    // 16. Color registers, exhaustive
    // -----------------------------------------------------------------

    #[test]
    fn color_registers_all_16_colors_round_trip() {
        let mut r = Registers::new();
        for reg in 0x20u8..=0x2E {
            for color in 0u8..=0x0F {
                r.write(reg, color);
                assert_eq!(r.read(reg, 0), 0xF0 | color, "${:02X} <- {}", reg, color);
            }
            r.write(reg, 0xA7);
            assert_eq!(r.read(reg, 0), 0xF7, "upper nibble ignored on ${:02X}", reg);
        }
    }

    #[test]
    fn color_writes_update_accessors_only_for_their_register() {
        let mut r = Registers::new();
        r.write(0x20, 0x09);
        assert_eq!(r.border_color(), 0x09);
        r.write(0x24, 0x0C);
        assert_eq!(r.background_colors(3), 0x0C);
        assert_eq!(r.background_colors(0), 0x06);
        r.write(0x2E, 0x0A);
        assert_eq!(r.sprite_colors(7), 0x0A);
        assert_eq!(r.sprite_colors(0), 0x01);
    }

    #[test]
    fn color_register_reads_ignore_the_beam_position() {
        let r = Registers::new();
        for &line in &[0u16, 255, 256, 311] {
            assert_eq!(r.read(0x20, line), 0xFE);
            assert_eq!(r.read(0x21, line), 0xF6);
            assert_eq!(r.read(0x2E, line), 0xFC);
        }
    }

    // -----------------------------------------------------------------
    // 17. $D011 / $D016 field decode, exhaustive
    // -----------------------------------------------------------------

    #[test]
    fn scroly_bit_fields_exhaustive_over_all_256_values() {
        let mut r = Registers::new();
        for v in 0u8..=255 {
            r.write(0x11, v);
            assert_eq!(r.yscroll(), v & 0x07);
            assert_eq!(r.rsel(), v & 0x08 != 0);
            assert_eq!(r.den(), v & 0x10 != 0);
            assert_eq!(r.bmm(), v & 0x20 != 0);
            assert_eq!(r.ecm(), v & 0x40 != 0);
            // Stored bit 7 is the compare MSB; the read path replaces it
            // with the beam MSB.
            assert_eq!(r.read(0x11, 0), v & 0x7F, "beam < 256");
            assert_eq!(r.read(0x11, 300), (v & 0x7F) | 0x80, "beam >= 256");
            assert_eq!(r.raster_compare() >> 8, (v >> 7) as u16);
        }
    }

    #[test]
    fn scrolx_bit_fields_exhaustive_over_all_256_values() {
        let mut r = Registers::new();
        for v in 0u8..=255 {
            r.write(0x16, v);
            assert_eq!(r.xscroll(), v & 0x07);
            assert_eq!(r.csel(), v & 0x08 != 0);
            assert_eq!(r.mcm(), v & 0x10 != 0);
            // Bits 0-5 (incl. the inert RES bit 5) round-trip; 6-7 float high.
            assert_eq!(r.read(0x16, 0), v | 0xC0);
        }
    }

    #[test]
    fn d016_bit5_res_round_trips_but_has_no_side_effect() {
        let mut r = Registers::new();
        r.write(0x16, 0x20);
        assert_eq!(r.read(0x16, 0), 0xE0);
        assert_eq!(r.xscroll(), 0);
        assert!(!r.csel());
        assert!(!r.mcm());
        r.write(0x16, 0x00);
        assert_eq!(r.read(0x16, 0), 0xC0);
    }

    // -----------------------------------------------------------------
    // 18. $D018 decoding, exhaustive + known configurations
    // -----------------------------------------------------------------

    #[test]
    fn vmcsb_decoding_exhaustive() {
        let mut r = Registers::new();
        for v in 0u8..=255 {
            r.write(0x18, v);
            assert_eq!(r.video_matrix_base(), (v >> 4) as u16 * 0x0400);
            assert_eq!(r.char_base(), (v & 0x0E) as u16 * 0x0400);
            assert_eq!(r.read(0x18, 0), v | 0x01);
        }
    }

    #[test]
    fn d018_known_hardware_configurations() {
        let mut r = Registers::new();
        r.write(0x18, 0x15); // power-on: screen $0400, uppercase ROM
        assert_eq!((r.video_matrix_base(), r.char_base()), (0x0400, 0x1000));
        r.write(0x18, 0x17); // POKE 53272,23: lowercase ROM
        assert_eq!(r.char_base(), 0x1800);
        r.write(0x18, 0x18); // custom charset at $2000
        assert_eq!(r.char_base(), 0x2000);
        r.write(0x18, 0x1A); // charset at $2800
        assert_eq!(r.char_base(), 0x2800);
        r.write(0x18, 0x1E); // charset at $3800
        assert_eq!(r.char_base(), 0x3800);
        r.write(0x18, 0xFC); // screen $3C00, charset $3000
        assert_eq!((r.video_matrix_base(), r.char_base()), (0x3C00, 0x3000));
        r.write(0x18, 0x01);
        assert_eq!((r.video_matrix_base(), r.char_base()), (0x0000, 0x0000));
    }

    // -----------------------------------------------------------------
    // 19. Sprites: coordinates, MSB, control registers
    // -----------------------------------------------------------------

    #[test]
    fn sprite_coordinate_registers_round_trip_all_256_values() {
        let mut r = Registers::new();
        for i in 0u8..8 {
            for v in 0u8..=255 {
                r.write(i * 2, v);
                r.write(i * 2 + 1, v);
                assert_eq!(r.read(i * 2, 0), v, "SP{}X", i);
                assert_eq!(r.read(i * 2 + 1, 0), v, "SP{}Y", i);
            }
        }
    }

    #[test]
    fn msigx_bits_are_independent() {
        let mut r = Registers::new();
        for bit in 0u8..8 {
            r.write(0x10, 1 << bit);
            assert_eq!(r.read(0x10, 0), 1 << bit);
            assert_eq!(r.read(0x00, 0), 0x00, "MSIGX write must not touch SP0X");
        }
        r.write(0x10, 0xFF);
        assert_eq!(r.read(0x10, 0), 0xFF);
        r.write(0x10, 0x00);
        assert_eq!(r.read(0x10, 0), 0x00);
    }

    #[test]
    fn sprite_x_9_bit_position_reconstruction() {
        let mut r = Registers::new();
        r.write(0x04, 0xC4);
        r.write(0x10, 0b0000_0100); // sprite 2 MSB
        let x = ((((r.read(0x10, 0) >> 2) & 1) as u16) << 8) | r.read(0x04, 0) as u16;
        assert_eq!(x, 452);
        r.write(0x0E, 0x40);
        r.write(0x10, 0b1000_0000); // sprite 7 MSB
        let x = ((((r.read(0x10, 0) >> 7) & 1) as u16) << 8) | r.read(0x0E, 0) as u16;
        assert_eq!(x, 320);
    }

    #[test]
    fn sprite_control_registers_round_trip_all_256_values() {
        let mut r = Registers::new();
        // SPENA, YXPAND, SPBGPR, SPMC, XXPAND — fully 8-bit connected
        for reg in [0x15u8, 0x17, 0x1B, 0x1C, 0x1D] {
            for v in 0u8..=255 {
                r.write(reg, v);
                assert_eq!(r.read(reg, 0), v, "${:02X} <- ${:02X}", reg, v);
            }
        }
    }

    #[test]
    fn sprite_control_writes_are_isolated() {
        let mut r = Registers::new();
        r.write(0x15, 0xFF);
        assert_eq!(
            (
                r.read(0x17, 0),
                r.read(0x1B, 0),
                r.read(0x1C, 0),
                r.read(0x1D, 0)
            ),
            (0, 0, 0, 0)
        );
        r.write(0x1D, 0xAA);
        assert_eq!(r.read(0x15, 0), 0xFF);
        assert_eq!(r.read(0x1C, 0), 0x00);
    }

    // -----------------------------------------------------------------
    // 20. Collision latches, deeper
    // -----------------------------------------------------------------

    #[test]
    fn collision_latches_accumulate_by_or() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0b0000_0001);
        r.set_sprite_sprite_collision(0b0000_0010);
        r.set_sprite_sprite_collision(0b1000_0000);
        assert_eq!(r.read(0x1E, 0), 0b1000_0011);
        assert_eq!(r.read(0x1E, 0), 0x00);
        // After a clear-on-read, accumulation starts fresh
        r.set_sprite_sprite_collision(0b0001_0000);
        assert_eq!(r.read(0x1E, 0), 0b0001_0000);
    }

    #[test]
    fn collision_latches_round_trip_all_256_values() {
        let mut r = Registers::new();
        for v in 0u8..=255 {
            r.set_sprite_sprite_collision(v);
            assert_eq!(r.read(0x1E, 0), v);
            assert_eq!(r.read(0x1E, 0), 0);
            r.set_sprite_bg_collision(v);
            assert_eq!(r.read(0x1F, 0), v);
            assert_eq!(r.read(0x1F, 0), 0);
        }
    }

    #[test]
    fn collision_latches_are_independent() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x12);
        r.set_sprite_bg_collision(0x34);
        assert_eq!(r.read(0x1E, 0), 0x12);
        assert_eq!(r.read(0x1F, 0), 0x34, "reading $D01E must not clear $D01F");
        assert_eq!(r.read(0x1E, 0), 0x00);
        assert_eq!(r.read(0x1F, 0), 0x00);
    }

    #[test]
    fn zero_collision_reports_change_nothing() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x00);
        r.set_sprite_bg_collision(0x00);
        assert_eq!((r.read(0x1E, 0), r.read(0x1F, 0)), (0, 0));
    }

    #[test]
    fn sprite_bg_collision_ignores_cpu_writes() {
        let mut r = Registers::new();
        r.set_sprite_bg_collision(0x21);
        r.write(0x1F, 0xFF);
        assert_eq!(r.read(0x1F, 0), 0x21);
        assert_eq!(r.read(0x1F, 0), 0x00);
    }

    #[test]
    fn collision_clear_on_read_applies_through_mirrors() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x77);
        assert_eq!(r.read(0x5E, 0), 0x77, "$D05E mirrors $D01E");
        assert_eq!(r.read(0x1E, 0), 0x00, "the mirror read cleared the latch");
        assert_eq!(r.read(0xDE, 0), 0x00);
    }

    // -----------------------------------------------------------------
    // 21. Light pen
    // -----------------------------------------------------------------

    #[test]
    fn lightpen_registers_ignore_writes_and_read_zero() {
        let mut r = Registers::new();
        for &v in &[0x00u8, 0x5A, 0xFF] {
            r.write(0x13, v);
            r.write(0x14, v);
            assert_eq!(r.read(0x13, 0), 0x00);
            assert_eq!(r.read(0x14, 0), 0x00);
            assert_eq!(r.read(0x13, 300), 0x00);
            assert_eq!(r.read(0x14, 300), 0x00);
        }
    }

    // -----------------------------------------------------------------
    // 22. Mirroring, exhaustive
    // -----------------------------------------------------------------

    #[test]
    fn every_mirror_block_reads_identically() {
        let mut r = Registers::new();
        for reg in 0x00u8..=0x2E {
            r.write(reg, 0x5A);
        }
        // Collision latches stay 0 here — clear-on-read, tested separately.
        for off in 0x00u8..=0x3F {
            for k in 1u8..4 {
                assert_eq!(
                    r.read(off, 0x123),
                    r.read(off + k * 0x40, 0x123),
                    "${:02X} vs mirror ${:02X}",
                    off,
                    off + k * 0x40
                );
            }
        }
    }

    #[test]
    fn writes_through_mirrors_hit_the_base_register() {
        let mut r = Registers::new();
        r.write(0x40, 0xAB); // $D040 -> $D000: sprite 0 X
        assert_eq!(r.read(0x00, 0), 0xAB);
        r.write(0x60, 0x0F); // $D060 -> $D020: border
        assert_eq!(r.read(0x20, 0), 0xFF);
        assert_eq!(r.border_color(), 0x0F);
        r.write(0x5A, 0x0E); // $D05A -> $D01A
        assert_eq!(r.read(0x1A, 0), 0xFE);
    }

    #[test]
    fn offset_masking_wraps_high_offsets() {
        let mut r = Registers::new();
        r.write(0xE0, 0x0D); // 0xE0 & 0x3F = 0x20 -> border
        assert_eq!(r.read(0x20, 0), 0xFD);
        r.set_sprite_bg_collision(0x11);
        assert_eq!(r.read(0x9F, 0), 0x11); // 0x9F & 0x3F = 0x1F
        assert_eq!(r.read(0x1F, 0), 0x00);
    }

    // -----------------------------------------------------------------
    // 23. Write/read mask matrix for every register
    // -----------------------------------------------------------------

    #[test]
    fn write_read_mask_matrix_every_register() {
        let mut r = Registers::new();
        // (register, read after writing $00, read after writing $FF), beam on line 5
        let cases: &[(u8, u8, u8)] = &[
            (0x00, 0x00, 0xFF),
            (0x01, 0x00, 0xFF),
            (0x02, 0x00, 0xFF),
            (0x03, 0x00, 0xFF),
            (0x04, 0x00, 0xFF),
            (0x05, 0x00, 0xFF),
            (0x06, 0x00, 0xFF),
            (0x07, 0x00, 0xFF),
            (0x08, 0x00, 0xFF),
            (0x09, 0x00, 0xFF),
            (0x0A, 0x00, 0xFF),
            (0x0B, 0x00, 0xFF),
            (0x0C, 0x00, 0xFF),
            (0x0D, 0x00, 0xFF),
            (0x0E, 0x00, 0xFF),
            (0x0F, 0x00, 0xFF),
            (0x10, 0x00, 0xFF), // MSIGX
            (0x11, 0x00, 0x7F),
            (0x12, 0x05, 0x05), // SCROLY / RASTER
            (0x13, 0x00, 0x00),
            (0x14, 0x00, 0x00), // LPENX / LPENY
            (0x15, 0x00, 0xFF),
            (0x16, 0xC0, 0xFF), // SPENA / SCROLX
            (0x17, 0x00, 0xFF),
            (0x18, 0x01, 0xFF), // YXPAND / VMCSB
            (0x19, 0x70, 0x70),
            (0x1A, 0xF0, 0xFF), // VICIRCL / IRQMSK
            (0x1B, 0x00, 0xFF),
            (0x1C, 0x00, 0xFF),
            (0x1D, 0x00, 0xFF),
            (0x1E, 0x00, 0x00),
            (0x1F, 0x00, 0x00), // collisions
            (0x20, 0xF0, 0xFF),
            (0x21, 0xF0, 0xFF),
            (0x22, 0xF0, 0xFF),
            (0x23, 0xF0, 0xFF),
            (0x24, 0xF0, 0xFF),
            (0x25, 0xF0, 0xFF),
            (0x26, 0xF0, 0xFF),
            (0x27, 0xF0, 0xFF),
            (0x28, 0xF0, 0xFF),
            (0x29, 0xF0, 0xFF),
            (0x2A, 0xF0, 0xFF),
            (0x2B, 0xF0, 0xFF),
            (0x2C, 0xF0, 0xFF),
            (0x2D, 0xF0, 0xFF),
            (0x2E, 0xF0, 0xFF),
            (0x2F, 0xFF, 0xFF),
            (0x30, 0xFF, 0xFF),
            (0x31, 0xFF, 0xFF),
            (0x32, 0xFF, 0xFF),
            (0x33, 0xFF, 0xFF),
            (0x34, 0xFF, 0xFF),
            (0x35, 0xFF, 0xFF),
            (0x36, 0xFF, 0xFF),
            (0x37, 0xFF, 0xFF),
            (0x38, 0xFF, 0xFF),
            (0x39, 0xFF, 0xFF),
            (0x3A, 0xFF, 0xFF),
            (0x3B, 0xFF, 0xFF),
            (0x3C, 0xFF, 0xFF),
            (0x3D, 0xFF, 0xFF),
            (0x3E, 0xFF, 0xFF),
            (0x3F, 0xFF, 0xFF),
        ];
        for &(reg, expect_zero, expect_ff) in cases {
            r.write(reg, 0x00);
            assert_eq!(r.read(reg, 5), expect_zero, "after $00 -> ${:02X}", reg);
            r.write(reg, 0xFF);
            assert_eq!(r.read(reg, 5), expect_ff, "after $FF -> ${:02X}", reg);
        }
    }

    // -----------------------------------------------------------------
    // 24. Global invariants
    // -----------------------------------------------------------------

    #[test]
    fn reads_are_side_effect_free_except_collision_clear() {
        let mut r = Registers::new();
        r.write(0x11, 0x9B);
        r.write(0x12, 0x77);
        r.write(0x15, 0xF0);
        r.write(0x16, 0x1C);
        r.write(0x18, 0x15);
        r.write(0x1A, 0x05);
        r.irq_flags = IRQ_RASTER;
        for off in 0x00u8..=0x2E {
            for _ in 0..3 {
                let _ = r.read(off, 5);
            }
        }
        assert_eq!(r.read(0x11, 5), 0x1B);
        assert_eq!(r.read(0x12, 5), 5, "beam, not compare");
        assert_eq!(r.read(0x15, 5), 0xF0);
        assert_eq!(r.read(0x16, 5), 0xDC);
        assert_eq!(r.read(0x18, 5), 0x15);
        assert_eq!(r.read(0x1A, 5), 0xF5);
        assert_eq!(r.irq_flags, IRQ_RASTER, "reading $D019 must not clear");
        assert_eq!(r.raster_compare(), 0x177);
        assert!(r.is_irq_asserted());
    }

    #[test]
    fn irq_source_bit_constants_match_hardware() {
        assert_eq!(IRQ_RASTER, 0x01);
        assert_eq!(IRQ_SPRITE_BG, 0x02);
        assert_eq!(IRQ_SPRITE_SPRITE, 0x04);
        assert_eq!(IRQ_LIGHTPEN, 0x08);
    }

    fn lcg(seed: &mut u32) -> u8 {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (*seed >> 16) as u8
    }

    #[test]
    fn fuzz_sprite_coordinate_state_round_trip() {
        let mut r = Registers::new();
        let mut seed: u32 = 0xC0FFEE;
        let mut model = [0u8; 16];
        for reg in 0x00u8..0x10 {
            let v = lcg(&mut seed);
            model[reg as usize] = v;
            r.write(reg, v);
        }
        for reg in 0x00u8..0x10 {
            assert_eq!(r.read(reg, 0), model[reg as usize], "${:02X}", reg);
        }
        // Overwrite one register; its neighbours keep their values
        r.write(0x07, 0x11);
        for reg in 0x00u8..0x10 {
            let expected = if reg == 0x07 {
                0x11
            } else {
                model[reg as usize]
            };
            assert_eq!(r.read(reg, 0), expected, "${:02X}", reg);
        }
    }

    // -----------------------------------------------------------------
    // 25. Collision events latch their IRQ flags ($D019 bits 1 and 2)
    //     set_sprite_*_collision() must also set the matching flag, on any
    //     0 -> 1 transition of the latch. Flag and latch are independent
    //     stores, cleared independently.
    // -----------------------------------------------------------------

    #[test]
    fn sprite_sprite_collision_latches_irq_flag() {
        let mut r = Registers::new();
        r.write(0x1A, IRQ_SPRITE_SPRITE);

        // Sprites 3 and 5 overlap: both bits latch in $D01E
        r.set_sprite_sprite_collision(0b0010_1000);
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, IRQ_SPRITE_SPRITE);
        assert_eq!(r.read(0x19, 0), 0xF4, "bit 2 | $70 | bit 7 (masked)");
        assert!(r.is_irq_asserted());
    }

    #[test]
    fn sprite_bg_collision_latches_irq_flag() {
        let mut r = Registers::new();
        r.write(0x1A, IRQ_SPRITE_BG);

        r.set_sprite_bg_collision(0b0000_0100); // sprite 2 over foreground
        assert_eq!(r.irq_flags & IRQ_SPRITE_BG, IRQ_SPRITE_BG);
        assert_eq!(r.read(0x19, 0), 0xF2);
        assert!(r.is_irq_asserted());
    }

    #[test]
    fn collision_flags_latch_even_when_mask_disables_irq() {
        // Flags latch regardless of $D01A — polling programs rely on this.
        let mut r = Registers::new();
        r.write(0x1A, 0x00);

        r.set_sprite_sprite_collision(0x80);
        r.set_sprite_bg_collision(0x01);

        let d019 = r.read(0x19, 0);
        assert_eq!(d019 & 0x0F, IRQ_SPRITE_BG | IRQ_SPRITE_SPRITE);
        assert_eq!(d019 & 0x80, 0x00, "no IRQ line without a mask bit");
        assert!(!r.is_irq_asserted());
    }

    #[test]
    fn zero_collision_event_does_not_latch_irq() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x00);
        r.set_sprite_bg_collision(0x00);
        assert_eq!(r.irq_flags, 0);
    }

    #[test]
    fn cpu_write_to_collision_register_never_latches_irq() {
        let mut r = Registers::new();
        r.write(0x1E, 0xFF);
        r.write(0x1F, 0xFF);
        assert_eq!(r.irq_flags, 0, "$D01E/$D01F are write-ignored");
    }

    #[test]
    fn collision_irq_flag_is_edge_triggered_on_the_latch() {
        // Hardware only issues the collision IRQ when a bit in $D01E/$D01F
        // changes 0 -> 1.
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x01); // sprite 0 collides
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, IRQ_SPRITE_SPRITE);

        r.write(0x19, IRQ_SPRITE_SPRITE); // ISR acks the flag only
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, 0);

        // Sprite 0 is hit again, but its bit is still latched in $D01E:
        r.set_sprite_sprite_collision(0x01);
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, 0, "no 0->1 edge, no IRQ");

        // A different sprite newly colliding does produce an edge:
        r.set_sprite_sprite_collision(0x02);
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, IRQ_SPRITE_SPRITE);
    }

    #[test]
    fn reading_collision_register_re_arms_the_irq_flag() {
        // Clear-on-read returns the latch to $00, so the same sprite
        // colliding again is a fresh 0->1 edge. (This is the mechanism
        // behind "reading $D01F while still overlapping re-fires the IRQ".)
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x01);
        r.write(0x19, IRQ_SPRITE_SPRITE); // ack flag, latch still $01
        assert_eq!(r.read(0x1E, 0), 0x01); // ...and now the latch is cleared

        r.set_sprite_sprite_collision(0x01);
        assert_eq!(r.irq_flags & IRQ_SPRITE_SPRITE, IRQ_SPRITE_SPRITE);
    }

    #[test]
    fn irq_flag_and_collision_latch_clear_independently() {
        let mut r = Registers::new();
        r.set_sprite_sprite_collision(0x04);

        // Reading $D01E clears the latch but NOT the $D019 flag:
        assert_eq!(r.read(0x1E, 0), 0x04);
        assert_eq!(r.read(0x19, 0) & IRQ_SPRITE_SPRITE, IRQ_SPRITE_SPRITE);

        // Acknowledging $D019 clears the flag but NOT the latch:
        r.set_sprite_sprite_collision(0x04); // re-latch after clear-on-read
        r.write(0x19, IRQ_SPRITE_SPRITE);
        assert_eq!(r.read(0x19, 0) & IRQ_SPRITE_SPRITE, 0);
        assert_eq!(r.read(0x1E, 0), 0x04, "latch survives the acknowledge");
    }

    #[test]
    fn full_collision_irq_workflow_acknowledge_and_retrigger() {
        let mut r = Registers::new();
        r.write(0x1A, IRQ_SPRITE_SPRITE);

        r.set_sprite_sprite_collision(0b0000_0011); // sprites 0 and 1
        assert!(r.is_irq_asserted());

        // ISR: identify source, harvest the mask, acknowledge
        assert_eq!(r.read(0x19, 0) & 0x0F, IRQ_SPRITE_SPRITE);
        assert_eq!(r.read(0x1E, 0), 0b0000_0011);
        r.write(0x19, IRQ_SPRITE_SPRITE);
        assert!(!r.is_irq_asserted());

        // Sprite 0 is hit again (latch was cleared by the read -> fresh edge)
        r.set_sprite_sprite_collision(0b0000_0001);
        assert!(r.is_irq_asserted());
        assert_eq!(r.read(0x1E, 0), 0b0000_0001);
    }

    // -----------------------------------------------------------------
    // 26. Light pen latching ($D013/$D014 + IRQ_LIGHTPEN)
    //     latch_lightpen(x, line): LPENX <- x, LPENY <- line & $FF,
    //     IRQ_LIGHTPEN latched. First trigger per frame only;
    //     new_frame() re-arms.
    // -----------------------------------------------------------------

    #[test]
    fn lightpen_trigger_latches_x_and_y() {
        let mut r = Registers::new();
        r.latch_lightpen(0x54, 100);

        assert_eq!(r.read(0x13, 0), 0x54, "LPENX");
        assert_eq!(r.read(0x14, 0), 100, "LPENY");
        // The beam position argument is irrelevant to the latches:
        assert_eq!(r.read(0x13, 400), 0x54);
        assert_eq!(r.read(0x14, 400), 100);
        // Visible through the mirrors too, and non-destructive:
        assert_eq!(r.read(0x53, 0), 0x54, "$D053 mirrors $D013");
        for _ in 0..3 {
            assert_eq!((r.read(0x13, 0), r.read(0x14, 0)), (0x54, 100));
        }
    }

    #[test]
    fn lightpen_y_latches_only_the_low_8_bits_of_the_line() {
        let mut r = Registers::new();
        r.latch_lightpen(0x20, 300); // line 300 = $12C
        assert_eq!(r.read(0x14, 0), 44, "300 AND $FF — no bit 9 in $D014");
    }

    #[test]
    fn lightpen_trigger_latches_irq_flag() {
        let mut r = Registers::new();
        r.write(0x1A, IRQ_LIGHTPEN);
        r.latch_lightpen(0x33, 0x60);

        assert_eq!(r.irq_flags & IRQ_LIGHTPEN, IRQ_LIGHTPEN);
        assert_eq!(r.read(0x19, 0), 0xF8, "bit 3 | $70 | bit 7 (masked)");
        assert!(r.is_irq_asserted());
    }

    #[test]
    fn lightpen_flag_latches_even_when_unmasked() {
        let mut r = Registers::new();
        r.write(0x1A, 0x00);
        r.latch_lightpen(0x00, 0x00);

        let d019 = r.read(0x19, 0);
        assert_eq!(d019 & 0x0F, IRQ_LIGHTPEN, "pollable without a mask");
        assert_eq!(d019 & 0x80, 0x00);
        assert!(!r.is_irq_asserted());
    }

    #[test]
    fn lightpen_latches_once_per_frame() {
        let mut r = Registers::new();

        r.latch_lightpen(0x10, 50);
        r.write(0x19, IRQ_LIGHTPEN); // ISR acknowledges

        // A second trigger in the same frame is a complete no-op:
        r.latch_lightpen(0x99, 200);
        assert_eq!(r.read(0x13, 0), 0x10, "LPENX not re-latched");
        assert_eq!(r.read(0x14, 0), 50, "LPENY not re-latched");
        assert_eq!(r.irq_flags & IRQ_LIGHTPEN, 0, "no second IRQ");

        // The next frame re-arms the latch:
        r.new_frame();
        r.latch_lightpen(0x99, 200);
        assert_eq!(r.read(0x13, 0), 0x99);
        assert_eq!(r.read(0x14, 0), 200);
        assert_eq!(r.irq_flags & IRQ_LIGHTPEN, IRQ_LIGHTPEN);
    }

    #[test]
    fn new_frame_re_arms_without_clearing_and_has_no_other_side_effects() {
        let mut r = Registers::new();
        r.latch_lightpen(0x77, 0x88);
        r.write(0x19, IRQ_LIGHTPEN);

        r.new_frame();
        r.new_frame(); // idempotent

        // Old coordinates persist until the next trigger; flag stays cleared
        assert_eq!(r.read(0x13, 0), 0x77);
        assert_eq!(r.read(0x14, 0), 0x88);
        assert_eq!(r.irq_flags & IRQ_LIGHTPEN, 0);

        // No side effects on the rest of the register state:
        r.set_sprite_bg_collision(0x80);
        r.new_frame();
        assert_eq!(r.irq_flags & IRQ_SPRITE_BG, IRQ_SPRITE_BG, "flags survive");
        assert_eq!(r.read(0x1F, 0), 0x80, "collision latch survives");

        // And the pen is armed for the new frame:
        r.latch_lightpen(0x12, 0x34);
        assert_eq!((r.read(0x13, 0), r.read(0x14, 0)), (0x12, 0x34));
    }

    #[test]
    fn lightpen_latch_leaves_other_registers_untouched() {
        let mut r = Registers::new();
        r.write(0x12, 0xE2);
        r.write(0x11, 0x1B);
        r.write(0x20, 0x01);

        r.latch_lightpen(0xAA, 0x11);

        assert_eq!(r.raster_compare(), 0x0E2);
        assert_eq!(r.read(0x12, 5), 5, "beam, not compare");
        assert_eq!(r.read(0x20, 0), 0xF1);
    }

    #[test]
    fn all_four_irq_sources_can_be_latched_simultaneously() {
        let mut r = Registers::new();
        r.write(0x11, 0x00);
        r.write(0x12, 0x30);
        r.write(0x1A, 0x0F);

        r.check_raster_match(0x30); // bit 0
        r.set_sprite_bg_collision(0x01); // bit 1
        r.set_sprite_sprite_collision(0x02); // bit 2
        r.latch_lightpen(0x40, 0x30); // bit 3

        assert_eq!(r.irq_flags, 0x0F);
        assert_eq!(r.read(0x19, 0), 0xFF, "all flags | $70 | bit 7");
        assert!(r.is_irq_asserted());

        // Acknowledge everything at once with $0F:
        r.write(0x19, 0x0F);
        assert_eq!(r.read(0x19, 0), 0x70);
        assert!(!r.is_irq_asserted());
    }

    #[test]
    fn cpu_write_cannot_corrupt_latched_lightpen_values() {
        let mut r = Registers::new();
        r.latch_lightpen(0x54, 100);
        r.write(0x13, 0xFF);
        r.write(0x14, 0xFF);
        for _ in 0..3 {
            let _ = r.read(0x13, 0); // reads must not re-arm the latch either
            let _ = r.read(0x14, 0);
        }
        assert_eq!((r.read(0x13, 0), r.read(0x14, 0)), (0x54, 100));
        r.latch_lightpen(0x11, 200); // still suppressed within the frame
        assert_eq!((r.read(0x13, 0), r.read(0x14, 0)), (0x54, 100));
    }
}
