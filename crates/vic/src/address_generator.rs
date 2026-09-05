const CHARACTER_ROM_START: u16 = 0x1000;
const CHARACTER_ROM_END: u16 = 0x1FFF;
const HARDWIRED_CHARACTER_ROM_BANKS: [u8; 2] = [0, 2];

#[derive(Debug, PartialEq, Eq)]
pub enum Address {
    CharacterRom(u16),
    Ram(u16),
}

pub struct AddressGenerator {
    bank: u8,
}

impl AddressGenerator {
    pub fn new() -> Self {
        Self { bank: 0 }
    }

    pub fn set_bank(&mut self, bank: u8) {
        self.bank = bank & 3;
    }

    pub fn c_access_address(&self, video_matrix_base: u16, vc: u16) -> Address {
        self.convert_address(video_matrix_base + (vc & 0x3FF))
    }

    pub fn g_access_address(&self, char_base: u16, screen_code: u8, rc: u8) -> Address {
        self.convert_address(char_base + (screen_code as u16) * 8 + (rc & 7) as u16)
    }

    pub fn convert_address(&self, addr: u16) -> Address {
        let addr = addr & 0x3FFF;
        if HARDWIRED_CHARACTER_ROM_BANKS.contains(&self.bank)
            && (CHARACTER_ROM_START..=CHARACTER_ROM_END).contains(&addr)
        {
            Address::CharacterRom(addr - CHARACTER_ROM_START)
        } else {
            Address::Ram(addr + self.bank as u16 * 0x4000)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL STATE
    //    * Default bank := 0.
    //
    // 2. 14-BIT TO 16-BIT PHYSICAL RAM TRANSLATION (OUTSIDE $1000..=$1FFF)
    //    * Bank 0: vic_addr $0000..=$3FFF -> RAM $0000..=$3FFF
    //    * Bank 1: vic_addr $0000..=$3FFF -> RAM $4000..=$7FFF
    //    * Bank 2: vic_addr $0000..=$3FFF -> RAM $8000..=$BFFF
    //    * Bank 3: vic_addr $0000..=$3FFF -> RAM $C000..=$FFFF
    //
    // 3. HARDWIRED CHARACTER ROM OVERLAY ($1000..=$1FFF)
    //    * Bank 0: vic_addr in $1000..=$1FFF MUST map to CharacterRom(offset),
    //      where offset is 0..=4095 (vic_addr - 0x1000).
    //    * Bank 2: vic_addr in $1000..=$1FFF MUST map to CharacterRom(offset).
    //    * Bank 1: vic_addr in $1000..=$1FFF MUST map to RAM ($5000..=$5FFF) - NO OVERLAY!
    //    * Bank 3: vic_addr in $1000..=$1FFF MUST map to RAM ($D000..=$DFFF) - NO OVERLAY!
    //
    // 4. C-ACCESS ADDRESS GENERATION (Video Matrix)
    //    * Formula: (video_matrix_base | (vc & 0x03FF)) & 0x3FFF.
    //    * Respects active bank and maps to RAM.
    //
    // 5. G-ACCESS ADDRESS GENERATION (Character Dot-Data)
    //    * Formula: (char_base | (screen_code << 3) | (rc & 7)) & 0x3FFF.
    //    * In Bank 0/2 with char_base = $1000: maps to CharacterRom.
    //    * In Bank 0 with char_base = $2000 (RAM char set): maps to RAM.
    //    * In Bank 1 with char_base = $1000 (RAM char set): maps to RAM $5000.
    //
    // 6. 14-BIT ADDRESS MASKING
    //    * Addresses >= $4000 wrap modulo 16KB (& 0x3FFF).
    // 7. BANK REGISTER MASKING
    //    * set_bank(b) behaves exactly like set_bank(b & 3); a wrapped bank
    //      inherits its overlay behaviour (set_bank(4) == bank 0).
    // 8. EXHAUSTIVE INVARIANTS
    //    * Full 14-bit translation table holds for every bank; every 16-bit
    //      input aliases its &0x3FFF image; banks 0 and 2 give identical ROM
    //      results.
    // 9. C-ACCESS COVERAGE
    //    * All 16 legal matrix bases ($0000..$3C00 step $0400). Matrix inside
    //      $1000..=$1FFF -> ROM in banks 0/2, RAM in 1/3. vc $3F8..$3FF are
    //      the sprite pointers; 1000..=1015 is unused.
    // 10. G-ACCESS COVERAGE
    //    * All 8 legal char bases ($0000..$3800 step $0800) -- every value of
    //      $D018 bits 1-3 is legal. Highest fetch: $3800+$FF*8+7 == $3FFF.
    // 11. SPRITE / IDLE FETCHES
    //    * Sprite data = pointer*64; blocks 64..=127 hit the ROM overlay in
    //      banks 0/2. Idle fetches address $3FFF.
    // 12. INPUT MASKING: vc masked to 10 bits, rc to 3 bits (see tests 13).
    // =====================================================================

    // ---------------------------------------------------------------------
    // 1. Initial State & Bank Selection
    // ---------------------------------------------------------------------

    #[test]
    fn initial_state_is_bank_0() {
        let generator = AddressGenerator::new();
        // In bank 0, address $0400 maps to physical RAM $0400
        assert_eq!(generator.convert_address(0x0400), Address::Ram(0x0400));
    }

    #[test]
    fn bank_switching_updates_physical_ram_base() {
        let mut generator = AddressGenerator::new();

        generator.set_bank(0);
        assert_eq!(generator.convert_address(0x0200), Address::Ram(0x0200));

        generator.set_bank(1);
        assert_eq!(generator.convert_address(0x0200), Address::Ram(0x4200));

        generator.set_bank(2);
        assert_eq!(generator.convert_address(0x0200), Address::Ram(0x8200));

        generator.set_bank(3);
        assert_eq!(generator.convert_address(0x0200), Address::Ram(0xC200));
    }

    // ---------------------------------------------------------------------
    // 2. Character ROM Overlay in Banks 0 and 2 ($1000..=$1FFF)
    // ---------------------------------------------------------------------

    #[test]
    fn bank_0_overlays_character_rom_at_1000_to_1fff() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // Outside overlay: RAM
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0x0FFF));
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0x2000));

        // Inside overlay: Character ROM with relative offset 0..=4095
        assert_eq!(
            generator.convert_address(0x1000),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(0x1008),
            Address::CharacterRom(0x0008)
        );
        assert_eq!(
            generator.convert_address(0x1800),
            Address::CharacterRom(0x0800)
        );
        assert_eq!(
            generator.convert_address(0x1FFF),
            Address::CharacterRom(0x0FFF)
        );
    }

    #[test]
    fn bank_2_overlays_character_rom_at_1000_to_1fff() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(2);

        // Outside overlay: RAM ($8000 base)
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0x8FFF));
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0xA000));

        // Inside overlay: Character ROM (identical to Bank 0!)
        assert_eq!(
            generator.convert_address(0x1000),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(0x1800),
            Address::CharacterRom(0x0800)
        );
        assert_eq!(
            generator.convert_address(0x1FFF),
            Address::CharacterRom(0x0FFF)
        );
    }

    #[test]
    fn bank_1_does_not_overlay_character_rom() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(1);

        // Bank 1 has NO Character ROM overlay: $1000..=$1FFF must read RAM!
        // Physical RAM address = $4000 + $1000 = $5000
        assert_eq!(generator.convert_address(0x1000), Address::Ram(0x5000));
        assert_eq!(generator.convert_address(0x1800), Address::Ram(0x5800));
        assert_eq!(generator.convert_address(0x1FFF), Address::Ram(0x5FFF));
    }

    #[test]
    fn bank_3_does_not_overlay_character_rom() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(3);

        // Bank 3 has NO Character ROM overlay: $1000..=$1FFF must read RAM!
        // Physical RAM address = $C000 + $1000 = $D000
        assert_eq!(generator.convert_address(0x1000), Address::Ram(0xD000));
        assert_eq!(generator.convert_address(0x1800), Address::Ram(0xD800));
        assert_eq!(generator.convert_address(0x1FFF), Address::Ram(0xDFFF));
    }

    // ---------------------------------------------------------------------
    // 3. C-Access Address Generation (Video Matrix)
    // ---------------------------------------------------------------------

    #[test]
    fn c_access_default_screen_at_0400_in_bank_0() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // Top-left character (column 0, VC=0)
        assert_eq!(generator.c_access_address(0x0400, 0), Address::Ram(0x0400));

        // Column 39 (VC=39)
        assert_eq!(generator.c_access_address(0x0400, 39), Address::Ram(0x0427));

        // Bottom-right character (VC=999)
        assert_eq!(
            generator.c_access_address(0x0400, 999),
            Address::Ram(0x0400 + 999)
        );
    }

    #[test]
    fn c_access_relocated_screen_in_bank_2() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(2); // Bank 2 base = $8000

        // Screen relocated to offset $2000 (CPU RAM $A000, under BASIC ROM)
        assert_eq!(generator.c_access_address(0x2000, 0), Address::Ram(0xA000));
        assert_eq!(generator.c_access_address(0x2000, 40), Address::Ram(0xA028));
    }

    // ---------------------------------------------------------------------
    // 4. G-Access Address Generation (Character Dot-Data)
    // ---------------------------------------------------------------------

    #[test]
    fn g_access_fetches_from_character_rom_in_bank_0() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // char_base = $1000 (Character ROM Set 1)
        // Screen code 0 (@), pixel row 0 (RC = 0)
        assert_eq!(
            generator.g_access_address(0x1000, 0, 0),
            Address::CharacterRom(0x0000)
        );

        // Screen code 1 ('A'), row 0 (RC = 0) -> character 1 starts at byte 8
        assert_eq!(
            generator.g_access_address(0x1000, 1, 0),
            Address::CharacterRom(0x0008)
        );

        // Screen code 1 ('A'), row 5 (RC = 5) -> byte 8 + 5 = 13
        assert_eq!(
            generator.g_access_address(0x1000, 1, 5),
            Address::CharacterRom(0x000D)
        );

        // Lowercase set (char_base = $1800):
        // Screen code 1 ('a'), row 0 -> offset $0800 + 8 = $0808
        assert_eq!(
            generator.g_access_address(0x1800, 1, 0),
            Address::CharacterRom(0x0808)
        );
    }

    #[test]
    fn g_access_fetches_from_ram_when_custom_charset_used() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // Custom character set placed in RAM at offset $2000 (outside Char ROM overlay)
        assert_eq!(
            generator.g_access_address(0x2000, 0, 0),
            Address::Ram(0x2000)
        );
        assert_eq!(
            generator.g_access_address(0x2000, 2, 4), // Screen code 2, row 4 -> 2 * 8 + 4 = 20
            Address::Ram(0x2014)
        );
    }

    #[test]
    fn g_access_in_bank_1_at_1000_fetches_from_ram() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(1); // Bank 1 base = $4000

        // In Bank 1, $1000 is RAM, NOT Character ROM!
        // Physical RAM = $4000 + $1000 = $5000
        assert_eq!(
            generator.g_access_address(0x1000, 0, 0),
            Address::Ram(0x5000)
        );
        assert_eq!(
            generator.g_access_address(0x1000, 1, 0),
            Address::Ram(0x5008)
        );
    }

    // ---------------------------------------------------------------------
    // 5. 14-Bit Address Wrapping
    // ---------------------------------------------------------------------

    #[test]
    fn addresses_wrap_modulo_16kb() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // $4000 wraps to $0000
        assert_eq!(generator.convert_address(0x4000), Address::Ram(0x0000));
        // $5000 wraps to $1000 (which is Character ROM in Bank 0!)
        assert_eq!(
            generator.convert_address(0x5000),
            Address::CharacterRom(0x0000)
        );
    }

    // ---------------------------------------------------------------------
    // 6. Bank Register Masking & Persistence
    // ---------------------------------------------------------------------

    #[test]
    fn set_bank_masks_bank_number_to_two_bits() {
        // Only two wires from $DD00 reach the VIC: any u8 must decode as
        // bank & 3, including the inherited character ROM overlay.
        let mut generator = AddressGenerator::new();
        for raw in 0u8..=255 {
            generator.set_bank(raw);
            let bank = raw & 3;

            assert_eq!(
                generator.convert_address(0x0000),
                Address::Ram(bank as u16 * 0x4000),
                "set_bank({raw:#04x}) must behave as bank {bank}"
            );

            let overlay_active =
                matches!(generator.convert_address(0x1000), Address::CharacterRom(_));
            assert_eq!(
                overlay_active,
                bank == 0 || bank == 2,
                "set_bank({raw:#04x}) must inherit the overlay of bank {bank}"
            );
        }
    }

    #[test]
    fn bank_selection_persists_across_accesses_and_can_be_reselected() {
        let mut generator = AddressGenerator::new();

        generator.set_bank(3);
        assert_eq!(generator.convert_address(0x0000), Address::Ram(0xC000));
        assert_eq!(generator.c_access_address(0x0400, 40), Address::Ram(0xC428));
        assert_eq!(
            generator.g_access_address(0x1000, 0, 0),
            Address::Ram(0xD000)
        );

        for _ in 0..3 {
            generator.set_bank(2);
            assert_eq!(generator.convert_address(0x0000), Address::Ram(0x8000));
            generator.set_bank(0);
            assert_eq!(
                generator.convert_address(0x1000),
                Address::CharacterRom(0x0000)
            );
        }
    }

    // ---------------------------------------------------------------------
    // 7. Exhaustive Translation Tables
    // ---------------------------------------------------------------------

    #[test]
    fn complete_translation_table_for_every_bank() {
        let mut generator = AddressGenerator::new();
        for bank in 0u8..=3 {
            generator.set_bank(bank);
            let overlay = bank == 0 || bank == 2;

            for addr in 0u16..=0x3FFF {
                let expected = if overlay && (0x1000u16..=0x1FFF).contains(&addr) {
                    Address::CharacterRom(addr - 0x1000)
                } else {
                    Address::Ram(bank as u16 * 0x4000 + addr)
                };
                assert_eq!(
                    generator.convert_address(addr),
                    expected,
                    "bank {bank}, vic offset {addr:#06x}"
                );
            }
        }
    }

    #[test]
    fn every_16_bit_input_aliases_its_14_bit_counterpart() {
        // The VIC drives only 14 address lines.
        let mut generator = AddressGenerator::new();
        for bank in 0u8..=3 {
            generator.set_bank(bank);
            for addr in 0u16..=0xFFFF {
                assert_eq!(
                    generator.convert_address(addr),
                    generator.convert_address(addr & 0x3FFF),
                    "bank {bank}, raw address {addr:#06x}"
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // 8. Overlay Boundaries
    // ---------------------------------------------------------------------

    #[test]
    fn overlay_window_edges_for_every_bank() {
        let mut generator = AddressGenerator::new();
        for bank in 0u8..=3 {
            generator.set_bank(bank);
            let rom_bank = bank == 0 || bank == 2;

            for addr in [0x0FFFu16, 0x1000, 0x1800, 0x1FFF, 0x2000] {
                let in_window = addr >= 0x1000 && addr <= 0x1FFF;
                let is_rom = matches!(generator.convert_address(addr), Address::CharacterRom(_));
                assert_eq!(
                    is_rom,
                    rom_bank && in_window,
                    "bank {bank}, addr {addr:#06x}"
                );
            }
        }
    }

    #[test]
    fn overlay_boundary_values_resolve_exactly() {
        let mut generator = AddressGenerator::new();

        generator.set_bank(0);
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0x0FFF));
        assert_eq!(
            generator.convert_address(0x1000),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(0x1001),
            Address::CharacterRom(0x0001)
        );
        assert_eq!(
            generator.convert_address(0x1FFE),
            Address::CharacterRom(0x0FFE)
        );
        assert_eq!(
            generator.convert_address(0x1FFF),
            Address::CharacterRom(0x0FFF)
        );
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0x2000));

        generator.set_bank(1);
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0x4FFF));
        assert_eq!(generator.convert_address(0x1000), Address::Ram(0x5000));
        assert_eq!(generator.convert_address(0x1FFF), Address::Ram(0x5FFF));
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0x6000));

        generator.set_bank(2);
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0x8FFF));
        assert_eq!(
            generator.convert_address(0x1000),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(0x1FFF),
            Address::CharacterRom(0x0FFF)
        );
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0xA000));

        generator.set_bank(3);
        assert_eq!(generator.convert_address(0x0FFF), Address::Ram(0xCFFF));
        assert_eq!(generator.convert_address(0x1000), Address::Ram(0xD000));
        assert_eq!(generator.convert_address(0x1FFF), Address::Ram(0xDFFF));
        assert_eq!(generator.convert_address(0x2000), Address::Ram(0xE000));
    }

    #[test]
    fn wrapped_addresses_reenter_the_rom_window() {
        let mut generator = AddressGenerator::new();

        generator.set_bank(0); // $5000..$5FFF wraps onto $1000..$1FFF -> ROM
        assert_eq!(
            generator.convert_address(0x5000),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(0x5FFF),
            Address::CharacterRom(0x0FFF)
        );
        assert_eq!(generator.convert_address(0x6000), Address::Ram(0x2000));
        assert_eq!(generator.convert_address(0xFFFF), Address::Ram(0x3FFF));

        generator.set_bank(2); // $9000..$9FFF wraps onto the same ROM window
        assert_eq!(
            generator.convert_address(0x9000),
            Address::CharacterRom(0x0000)
        );

        generator.set_bank(1);
        assert_eq!(generator.convert_address(0x5000), Address::Ram(0x5000));
        assert_eq!(generator.convert_address(0xFFFF), Address::Ram(0x7FFF));
        generator.set_bank(3);
        assert_eq!(generator.convert_address(0x5000), Address::Ram(0xD000));
        assert_eq!(generator.convert_address(0xFFFF), Address::Ram(0xFFFF));
    }

    #[test]
    fn character_rom_results_are_identical_in_banks_0_and_2() {
        let mut bank0 = AddressGenerator::new();
        let mut bank2 = AddressGenerator::new();
        bank0.set_bank(0);
        bank2.set_bank(2);

        for addr in 0x1000u16..=0x1FFF {
            assert_eq!(
                bank0.convert_address(addr),
                bank2.convert_address(addr),
                "VIC offset {addr:#06x}"
            );
        }
        // Outside the window the banks are exactly $8000 apart.
        for addr in [0x0000u16, 0x0FFF, 0x2000, 0x3FFF] {
            assert_eq!(bank0.convert_address(addr), Address::Ram(addr));
            assert_eq!(bank2.convert_address(addr), Address::Ram(addr + 0x8000));
        }
    }

    // ---------------------------------------------------------------------
    // 9. C-Access Coverage
    // ---------------------------------------------------------------------

    #[test]
    fn c_access_covers_all_sixteen_legal_video_matrix_bases() {
        // $D018 bits 4-7 select the 1K matrix from $0000 to $3C00 (NOT $F000!).
        let mut generator = AddressGenerator::new();
        generator.set_bank(1); // no overlay: pure arithmetic

        for d018_bits in 0u16..16 {
            let base = d018_bits * 0x0400;
            for vc in [0u16, 1, 39, 40, 399, 400, 999, 0x3FF] {
                assert_eq!(
                    generator.c_access_address(base, vc),
                    Address::Ram(0x4000 + base + vc),
                    "video matrix base {base:#06x}, vc {vc}"
                );
            }
        }
    }

    #[test]
    fn c_access_with_video_matrix_inside_the_rom_window() {
        // Matrix base in $1000..=$1FFF: banks 0/2 fetch "screen codes" from
        // character ROM (the classic garbage screen), banks 1/3 read RAM.
        let mut generator = AddressGenerator::new();

        generator.set_bank(0);
        assert_eq!(
            generator.c_access_address(0x1000, 0x000),
            Address::CharacterRom(0x000)
        );
        assert_eq!(
            generator.c_access_address(0x1000, 0x3FF),
            Address::CharacterRom(0x3FF)
        );
        assert_eq!(
            generator.c_access_address(0x1400, 999),
            Address::CharacterRom(0x7E7)
        );
        assert_eq!(
            generator.c_access_address(0x1C00, 0x3FF),
            Address::CharacterRom(0xFFF)
        );

        generator.set_bank(2);
        assert_eq!(
            generator.c_access_address(0x1800, 0x000),
            Address::CharacterRom(0x800)
        );

        generator.set_bank(1);
        assert_eq!(
            generator.c_access_address(0x1000, 0x000),
            Address::Ram(0x5000)
        );
        assert_eq!(
            generator.c_access_address(0x1C00, 0x3FF),
            Address::Ram(0x5FFF)
        );
    }

    #[test]
    fn c_access_maps_the_full_visible_screen_contiguously() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(3); // no overlay in this bank

        for vc in 0u16..1000 {
            assert_eq!(
                generator.c_access_address(0x0400, vc),
                Address::Ram(0xC400 + vc),
                "vc {vc}"
            );
        }
    }

    #[test]
    fn c_access_covers_the_sprite_pointer_block() {
        // 1K matrix = 1000 screen codes, 16 unused bytes (vc 1000..=1015),
        // then 8 sprite pointers at vc $3F8..=$3FF.
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        assert_eq!(
            generator.c_access_address(0x0400, 1000),
            Address::Ram(0x07E8)
        );
        assert_eq!(
            generator.c_access_address(0x0400, 1015),
            Address::Ram(0x07F7)
        );
        for sprite in 0u16..8 {
            assert_eq!(
                generator.c_access_address(0x0400, 0x3F8 + sprite),
                Address::Ram(0x07F8 + sprite),
                "sprite {sprite} pointer fetch"
            );
        }

        // Same block in bank 2: pointers at $87F8..=$87FF.
        generator.set_bank(2);
        assert_eq!(
            generator.c_access_address(0x0400, 0x3F8),
            Address::Ram(0x87F8)
        );
        assert_eq!(
            generator.c_access_address(0x0400, 0x3FF),
            Address::Ram(0x87FF)
        );
    }

    #[test]
    fn c_access_reaches_the_top_of_the_address_space() {
        // Highest legal matrix base ($3C00) plus highest VC ($3FF) == $3FFF.
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);
        assert_eq!(
            generator.c_access_address(0x3C00, 0x3FF),
            Address::Ram(0x3FFF)
        );
        generator.set_bank(2);
        assert_eq!(
            generator.c_access_address(0x3C00, 0x000),
            Address::Ram(0xBC00)
        );
    }

    // ---------------------------------------------------------------------
    // 10. G-Access Coverage
    // ---------------------------------------------------------------------

    #[test]
    fn g_access_char_base_1000_reaches_every_byte_of_rom_set_1() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        for screen_code in 0u16..=255 {
            for rc in 0u16..=7 {
                assert_eq!(
                    generator.g_access_address(0x1000, screen_code as u8, rc as u8),
                    Address::CharacterRom(screen_code * 8 + rc),
                    "screen code {screen_code}, row {rc}"
                );
            }
        }
    }

    #[test]
    fn g_access_char_base_1800_reaches_rom_set_2() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        assert_eq!(
            generator.g_access_address(0x1800, 0, 0),
            Address::CharacterRom(0x0800)
        );
        assert_eq!(
            generator.g_access_address(0x1800, 128, 4),
            Address::CharacterRom(0x0C04)
        );
        assert_eq!(
            generator.g_access_address(0x1800, 255, 7),
            Address::CharacterRom(0x0FFF)
        );
    }

    #[test]
    fn g_access_full_charset_is_plain_ram_in_banks_1_and_3() {
        let mut generator = AddressGenerator::new();
        for (bank, phys_base) in [(1u8, 0x5000u16), (3u8, 0xD000u16)] {
            generator.set_bank(bank);

            for screen_code in 0u16..=255 {
                for rc in 0u16..=7 {
                    assert_eq!(
                        generator.g_access_address(0x1000, screen_code as u8, rc as u8),
                        Address::Ram(phys_base + screen_code * 8 + rc),
                        "bank {bank}, screen code {screen_code}, row {rc}"
                    );
                }
            }
        }
    }

    #[test]
    fn g_access_covers_all_eight_legal_char_bases() {
        // $D018 bits 1-3: ALL eight values legal, step $0800.
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        for cb in 0u16..8 {
            let base = cb * 0x0800;
            for screen_code in [0u8, 1, 63, 64, 127, 128, 191, 255] {
                for rc in 0u8..=7 {
                    let offset = base + screen_code as u16 * 8 + rc as u16;
                    let expected = if (0x1000u16..=0x1FFF).contains(&offset) {
                        Address::CharacterRom(offset - 0x1000)
                    } else {
                        Address::Ram(offset)
                    };
                    assert_eq!(
                        generator.g_access_address(base, screen_code, rc),
                        expected,
                        "char base {base:#06x}, screen code {screen_code}, row {rc}"
                    );
                }
            }
        }
    }

    #[test]
    fn g_access_charset_just_below_and_above_the_overlay() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // The 2K set at $0800 ends exactly at $0FFF, still RAM...
        assert_eq!(
            generator.g_access_address(0x0800, 255, 7),
            Address::Ram(0x0FFF)
        );
        // ...the set at $1000 is ROM from its first byte...
        assert_eq!(
            generator.g_access_address(0x1000, 0, 0),
            Address::CharacterRom(0x0000)
        );
        // ...and the set at $2000 is RAM again.
        assert_eq!(
            generator.g_access_address(0x2000, 0, 0),
            Address::Ram(0x2000)
        );
        assert_eq!(
            generator.g_access_address(0x2000, 255, 7),
            Address::Ram(0x27FF)
        );
    }

    #[test]
    fn g_access_reaches_the_very_top_of_the_address_space() {
        // Highest possible fetch: $3800 + $FF*8 + 7 == $3FFF.
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);
        assert_eq!(
            generator.g_access_address(0x3800, 255, 7),
            Address::Ram(0x3FFF)
        );
        generator.set_bank(3);
        assert_eq!(
            generator.g_access_address(0x3800, 255, 7),
            Address::Ram(0xFFFF)
        );
    }

    // ---------------------------------------------------------------------
    // 11. Sprite & Idle Fetch Addresses
    // ---------------------------------------------------------------------

    #[test]
    fn sprite_data_blocks_resolve_through_the_overlay() {
        // Sprite data = bank base + pointer*64. Blocks 64..=127 land in
        // $1000..$1FFF, so in banks 0/2 the VIC fetches character ROM instead
        // of the shapes stored in RAM -- genuine hardware behaviour.
        let mut generator = AddressGenerator::new();

        generator.set_bank(0);
        assert_eq!(generator.convert_address(0 * 64), Address::Ram(0x0000));
        assert_eq!(generator.convert_address(63 * 64), Address::Ram(0x0FC0));
        assert_eq!(
            generator.convert_address(64 * 64),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(
            generator.convert_address(127 * 64),
            Address::CharacterRom(0x0FC0)
        );
        assert_eq!(generator.convert_address(128 * 64), Address::Ram(0x2000));
        assert_eq!(generator.convert_address(255 * 64), Address::Ram(0x3FC0));

        generator.set_bank(2);
        assert_eq!(
            generator.convert_address(64 * 64),
            Address::CharacterRom(0x0000)
        );
        assert_eq!(generator.convert_address(0 * 64), Address::Ram(0x8000));

        generator.set_bank(1);
        assert_eq!(generator.convert_address(64 * 64), Address::Ram(0x5000));
        assert_eq!(generator.convert_address(255 * 64), Address::Ram(0x7FC0));

        generator.set_bank(3);
        assert_eq!(generator.convert_address(255 * 64), Address::Ram(0xFFC0));
    }

    #[test]
    fn idle_fetches_use_address_3fff() {
        // Outside display time the VIC runs a (discarded) fetch from
        // bank base + $3FFF on every idle cycle.
        let mut generator = AddressGenerator::new();
        for (bank, expected) in [
            (0u8, Address::Ram(0x3FFF)),
            (1, Address::Ram(0x7FFF)),
            (2, Address::Ram(0xBFFF)),
            (3, Address::Ram(0xFFFF)),
        ] {
            generator.set_bank(bank);
            assert_eq!(generator.convert_address(0x3FFF), expected, "bank {bank}");
        }
    }

    // ---------------------------------------------------------------------
    // 12. Power-On Register Defaults
    // ---------------------------------------------------------------------

    #[test]
    fn d018_and_dd00_power_on_defaults_in_every_bank() {
        // KERNAL default: $DD00 = $3F, $D018 = $15 (matrix $0400, charset $1000).
        // Keep $D018 fixed and walk the VIC bank around it.
        let mut generator = AddressGenerator::new();
        for bank in 0u8..=3 {
            generator.set_bank(bank);

            let screen = generator.c_access_address(0x0400, 0);
            let glyph = generator.g_access_address(0x1000, 1, 0); // code 1, row 0

            assert_eq!(screen, Address::Ram(bank as u16 * 0x4000 + 0x0400));
            if bank == 0 || bank == 2 {
                assert_eq!(glyph, Address::CharacterRom(0x0008));
            } else {
                assert_eq!(glyph, Address::Ram(bank as u16 * 0x4000 + 0x1008));
            }
        }
    }

    // ---------------------------------------------------------------------
    // 13. Input Masking Per The Documented Contract
    //
    // *** THESE TWO TESTS FAIL AGAINST THE CURRENT IMPLEMENTATION ***
    // The contract pinned above specifies
    //     c: (video_matrix_base | (vc & 0x03FF)) & 0x3FFF
    //     g: (char_base | (screen_code << 3) | (rc & 7)) & 0x3FFF
    // but the code uses plain `+`. On the real chip VC is a 10-bit counter
    // and RC is a 3-bit counter, so out-of-range values wrap instead of
    // carrying into the base bits (see review, bugs 1-3, for the fix).
    // If your raster core guarantees in-range inputs by construction, you
    // may instead delete these tests and soften the contract wording.
    // ---------------------------------------------------------------------

    #[test]
    fn c_access_wraps_vc_modulo_1024() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        // With `+` this returns Ram(0x0827); hardware wraps the 10-bit VC.
        assert_eq!(
            generator.c_access_address(0x0400, 0x0427),
            Address::Ram(0x0427)
        );
    }

    #[test]
    fn g_access_wraps_rc_modulo_8() {
        let mut generator = AddressGenerator::new();
        generator.set_bank(0);

        assert_eq!(
            generator.g_access_address(0x1000, 1, 8),
            Address::CharacterRom(0x0008)
        ); // rc&7 == 0
        assert_eq!(
            generator.g_access_address(0x1000, 1, 9),
            Address::CharacterRom(0x0009)
        ); // rc&7 == 1
        assert_eq!(
            generator.g_access_address(0x1000, 1, 0xFF),
            Address::CharacterRom(0x000F)
        ); // rc&7 == 7
    }
}
