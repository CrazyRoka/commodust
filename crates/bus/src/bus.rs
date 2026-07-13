const COLOR_RAM_SIZE: usize = 0x0400; // 1024 bytes (1K × 4-bit)

const BASIC_ROM_OFFSET: usize = 0x0000; // 8K — maps to CPU $A000–$BFFF
const BASIC_ROM_SIZE: usize = 0x2000;
const KERNAL_ROM_OFFSET: usize = 0x2000; // 8K — maps to CPU $E000–$FFFF
const KERNAL_ROM_SIZE: usize = 0x2000;
const CHAR_ROM_OFFSET: usize = 0x4000; // 4K — maps to CPU $D000–$DFFF (when CHAREN=0)
const CHAR_ROM_SIZE: usize = 0x1000;

const RAM_SIZE: usize = 0x10000;
const ROM_SIZE: usize = BASIC_ROM_SIZE + KERNAL_ROM_SIZE + CHAR_ROM_SIZE;

const LORAM_DIR: u8 = 0x01;
const HIRAM_DIR: u8 = 0x02;
const CHAREN_DIR: u8 = 0x04;
const CASSETTE_DATA_OUT_DIR: u8 = 0x08;
const CASSETTE_SWITCH_DIR: u8 = 0x10;
const CASSETTE_MOTOR_DIR: u8 = 0x20;

const LORAM: u8 = 0x01;
const HIRAM: u8 = 0x02;
const CHAREN: u8 = 0x04;
const CASSETTE_DATA_OUT: u8 = 0x08;
const CASSETTE_SWITCH: u8 = 0x10;
const CASSETTE_MOTOR: u8 = 0x20;

pub struct Bus {
    ram: [u8; RAM_SIZE],
    rom: [u8; ROM_SIZE],
    data_direction_register: u8,
    data_port: u8,
}

impl Bus {
    pub fn new(rom: &[u8]) -> Self {
        assert_eq!(
            rom.len(),
            ROM_SIZE,
            "ROM size should be equal to {}",
            ROM_SIZE
        );

        let mut new_rom = [0; ROM_SIZE];
        new_rom.copy_from_slice(rom);
        Self {
            ram: [0; RAM_SIZE],
            rom: new_rom,
            data_direction_register: 0b11101111,
            data_port: 0b00110111,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 => self.data_direction_register = value,
            0x0001 => self.data_port = value,
            _ => self.ram[addr as usize] = value,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 => self.data_direction_register,
            0x0001 => self.data_port,
            0xA000..=0xBFFF if self.data_port & LORAM != 0 => {
                self.rom[addr as usize - 0xA000 + BASIC_ROM_OFFSET]
            }
            0xD000..=0xDFFF
                if (self.data_port & CHAREN != 0) && (self.data_port & (LORAM | HIRAM) != 0) =>
            {
                self.read_io(addr)
            }
            0xD000..=0xDFFF
                if (self.data_port & CHAREN != 0) && (self.data_port & (LORAM | HIRAM) != 0) =>
            {
                self.read_io(addr)
            }
            0xD000..=0xDFFF if self.data_port & (LORAM | HIRAM) != 0 => {
                self.rom[addr as usize - 0xD000 + CHAR_ROM_OFFSET]
            }
            0xE000..=0xFFFF if self.data_port & HIRAM != 0 => {
                self.rom[addr as usize - 0xE000 + KERNAL_ROM_OFFSET]
            }
            _ => self.ram[addr as usize],
        }
    }

    fn read_io(&self, addr: u16) -> u8 {
        0xFF
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bus, bus::*};

    /// Create a bus with distinctive ROM patterns so we can verify
    /// which ROM (or RAM) is being read at any address.
    /// - BASIC ROM bytes:   (offset & 0xFF) ^ 0xA0
    /// - Kernal ROM bytes:  (offset & 0xFF) ^ 0xE0
    /// - Character ROM bytes: (offset & 0xFF) ^ 0xD0
    fn make_test_bus() -> Bus {
        let mut rom = [0u8; ROM_SIZE];

        for i in 0..BASIC_ROM_SIZE {
            rom[BASIC_ROM_OFFSET + i] = ((i & 0xFF) as u8) ^ 0xA0;
        }
        for i in 0..KERNAL_ROM_SIZE {
            rom[KERNAL_ROM_OFFSET + i] = ((i & 0xFF) as u8) ^ 0xE0;
        }
        for i in 0..CHAR_ROM_SIZE {
            rom[CHAR_ROM_OFFSET + i] = ((i & 0xFF) as u8) ^ 0xD0;
        }

        Bus::new(&rom)
    }

    /// Set LORAM / HIRAM / CHAREN by writing to the 6510 I/O port at $01.
    /// Other bits keep default values (cassette signals, etc.).
    fn set_banking(bus: &mut Bus, loram: bool, hiram: bool, charen: bool) {
        let mut val: u8 = 0b00110111; // default $37: cassette switch=1, motor=0
        val &= !0x07; // clear LORAM/HIRAM/CHAREN
        if loram {
            val |= 0x01;
        }
        if hiram {
            val |= 0x02;
        }
        if charen {
            val |= 0x04;
        }
        bus.write(0x0001, val);
    }

    #[test]
    fn test_bus_new_creation_successful() {
        let mut rom = [0; ROM_SIZE];
        for i in 0..ROM_SIZE {
            rom[i] = (i & 0xFF) as u8;
        }
        let bus = Bus::new(&rom);

        assert_eq!(bus.ram, [0; RAM_SIZE]);
        assert_eq!(bus.rom, rom);
        assert_eq!(bus.data_direction_register, 0b11101111);
        assert_eq!(bus.data_port, 0b00110111);
    }

    #[test]
    #[should_panic(expected = "ROM size should be equal to 20480")]
    fn test_bus_fails_when_rom_size_is_wrong() {
        Bus::new(&[0; 123]);
    }

    #[test]
    fn test_ram_read_write_basic() {
        let mut bus = make_test_bus();

        bus.write(0x0800, 0x42);
        assert_eq!(bus.read(0x0800), 0x42);

        bus.write(0x0002, 0xFF);
        assert_eq!(bus.read(0x0002), 0xFF);

        bus.write(0xC000, 0x99);
        assert_eq!(bus.read(0xC000), 0x99);
    }

    #[test]
    fn test_ram_write_then_overwrite() {
        let mut bus = make_test_bus();

        bus.write(0x4000, 0x11);
        bus.write(0x4000, 0x22);
        assert_eq!(bus.read(0x4000), 0x22);
    }

    #[test]
    fn test_ram_full_range_accessible() {
        let mut bus = make_test_bus();

        // Test boundaries of always-RAM regions
        for &addr in &[
            0x0002u16, 0x00FF, 0x0100, 0x01FF, 0x0200, 0x7FFF, 0xC000, 0xCFFF,
        ] {
            bus.write(addr, 0x77);
            assert_eq!(bus.read(addr), 0x77, "RAM failed at ${:04X}", addr);
        }
    }

    #[test]
    fn test_default_banking_basic_rom_visible() {
        // Mode 31: LORAM=1, HIRAM=1, CHAREN=1
        let bus = make_test_bus();
        // Default is already mode 31

        // $A000 should read BASIC ROM
        assert_eq!(bus.read(0xA000), 0xA0); // rom[0x0000] = 0x00 ^ 0xA0
        assert_eq!(bus.read(0xA001), 0xA1);
        assert_eq!(bus.read(0xBFFF), 0xFF ^ 0xA0); // rom[0x1FFF]

        // $E000 should read Kernal ROM
        assert_eq!(bus.read(0xE000), 0xE0); // rom[0x2000] = 0x00 ^ 0xE0
        assert_eq!(bus.read(0xE001), 0xE1);

        // $D000 should be I/O (no devices connected → $FF)
        assert_eq!(bus.read(0xD000), 0xFF);

        // $C000 should be RAM
        assert_eq!(bus.read(0xC000), 0x00);
    }

    #[test]
    fn test_banking_mode_31_all_rom() {
        // LORAM=1, HIRAM=1, CHAREN=1 → BASIC, I/O, Kernal
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, true, true);

        assert_eq!(bus.read(0xA000), 0xA0); // BASIC ROM
        assert_eq!(bus.read(0xD000), 0xFF); // I/O
        assert_eq!(bus.read(0xE000), 0xE0); // Kernal ROM
    }

    #[test]
    fn test_banking_mode_27_char_rom() {
        // LORAM=1, HIRAM=1, CHAREN=0 → BASIC, Character ROM, Kernal
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, true, false);

        assert_eq!(bus.read(0xA000), 0xA0); // BASIC ROM
        assert_eq!(bus.read(0xD000), 0xD0); // Character ROM: rom[0x4000] = 0x00 ^ 0xD0
        assert_eq!(bus.read(0xD001), 0xD1);
        assert_eq!(bus.read(0xDFFF), 0xFF ^ 0xD0); // rom[0x4FFF]
        assert_eq!(bus.read(0xE000), 0xE0); // Kernal ROM
    }

    #[test]
    fn test_banking_mode_29_io_no_kernal() {
        // LORAM=1, HIRAM=0, CHAREN=1 → BASIC, I/O, RAM at $E000
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, false, true);

        assert_eq!(bus.read(0xA000), 0xA0); // BASIC ROM
        assert_eq!(bus.read(0xD000), 0xFF); // I/O
        assert_eq!(bus.read(0xE000), 0x00); // RAM (was cleared)
    }

    #[test]
    fn test_banking_mode_25_char_rom_no_kernal() {
        // LORAM=1, HIRAM=0, CHAREN=0 → BASIC, Character ROM, RAM at $E000
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, false, false);

        assert_eq!(bus.read(0xA000), 0xA0); // BASIC ROM
        assert_eq!(bus.read(0xD000), 0xD0); // Character ROM
        assert_eq!(bus.read(0xE000), 0x00); // RAM
    }

    #[test]
    fn test_banking_mode_30_ram_a000() {
        // LORAM=0, HIRAM=1, CHAREN=1 → RAM at $A000, I/O, Kernal
        let mut bus = make_test_bus();
        // Pre-write to RAM under BASIC ROM
        bus.write(0xA000, 0x55);
        set_banking(&mut bus, false, true, true);

        assert_eq!(bus.read(0xA000), 0x55); // RAM (write-through)
        assert_eq!(bus.read(0xD000), 0xFF); // I/O
        assert_eq!(bus.read(0xE000), 0xE0); // Kernal ROM
    }

    #[test]
    fn test_banking_mode_26_ram_char_rom() {
        // LORAM=0, HIRAM=1, CHAREN=0 → RAM at $A000, Character ROM, Kernal
        let mut bus = make_test_bus();
        set_banking(&mut bus, false, true, false);

        assert_eq!(bus.read(0xA000), 0x00); // RAM
        assert_eq!(bus.read(0xD000), 0xD0); // Character ROM
        assert_eq!(bus.read(0xE000), 0xE0); // Kernal ROM
    }

    #[test]
    fn test_banking_mode_28_all_ram() {
        // LORAM=0, HIRAM=0 → RAM everywhere (CHAREN irrelevant)
        let mut bus = make_test_bus();

        // With CHAREN=1
        set_banking(&mut bus, false, false, true);
        assert_eq!(bus.read(0xA000), 0x00); // RAM
        assert_eq!(bus.read(0xD000), 0x00); // RAM (not I/O!)
        assert_eq!(bus.read(0xE000), 0x00); // RAM

        // With CHAREN=0 — should be same (CHAREN irrelevant when LORAM=0 & HIRAM=0)
        set_banking(&mut bus, false, false, false);
        assert_eq!(bus.read(0xA000), 0x00); // RAM
        assert_eq!(bus.read(0xD000), 0x00); // RAM (not Character ROM!)
        assert_eq!(bus.read(0xE000), 0x00); // RAM
    }

    #[test]
    fn test_all_eight_banking_modes_no_cartridge() {
        // Exhaustive test of all 8 LORAM/HIRAM/CHAREN combinations
        let mut bus = make_test_bus();

        let modes = [
            // (loram, hiram, charen, a000_is_rom, d000_is_char_rom, d000_is_io, e000_is_rom)
            (true, true, true, true, false, true, true), // 31
            (true, true, false, true, true, false, true), // 27
            (true, false, true, true, false, true, false), // 29
            (true, false, false, true, true, false, false), // 25
            (false, true, true, false, false, true, true), // 30
            (false, true, false, false, true, false, true), // 26
            (false, false, true, false, false, false, false), // 28
            (false, false, false, false, false, false, false), // 24
        ];

        for (i, &(loram, hiram, charen, a_rom, d_char, d_io, e_rom)) in modes.iter().enumerate() {
            set_banking(&mut bus, loram, hiram, charen);

            // $A000: BASIC ROM or RAM
            if a_rom {
                assert_eq!(
                    bus.read(0xA000),
                    0xA0,
                    "mode {}: expected BASIC ROM at $A000",
                    i
                );
            } else {
                assert_eq!(bus.read(0xA000), 0x00, "mode {}: expected RAM at $A000", i);
            }

            // $D000: Character ROM, I/O, or RAM
            if d_char {
                assert_eq!(
                    bus.read(0xD000),
                    0xD0,
                    "mode {}: expected Character ROM at $D000",
                    i
                );
            } else if d_io {
                assert_eq!(
                    bus.read(0xD000),
                    0xFF,
                    "mode {}: expected I/O ($FF) at $D000",
                    i
                );
            } else {
                assert_eq!(bus.read(0xD000), 0x00, "mode {}: expected RAM at $D000", i);
            }

            // $E000: Kernal ROM or RAM
            if e_rom {
                assert_eq!(
                    bus.read(0xE000),
                    0xE0,
                    "mode {}: expected Kernal ROM at $E000",
                    i
                );
            } else {
                assert_eq!(bus.read(0xE000), 0x00, "mode {}: expected RAM at $E000", i);
            }
        }
    }

    #[test]
    fn test_write_through_to_ram_under_basic_rom() {
        let mut bus = make_test_bus();
        // Default: BASIC ROM visible at $A000
        assert_eq!(bus.read(0xA000), 0xA0); // ROM

        // Write goes to RAM underneath
        bus.write(0xA000, 0x42);

        // Read still returns ROM
        assert_eq!(bus.read(0xA000), 0xA0);

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);

        // Now we see the written value
        assert_eq!(bus.read(0xA000), 0x42);
    }

    #[test]
    fn test_write_through_to_ram_under_kernal_rom() {
        let mut bus = make_test_bus();
        assert_eq!(bus.read(0xE000), 0xE0); // ROM

        bus.write(0xE000, 0x77);

        // Still ROM
        assert_eq!(bus.read(0xE000), 0xE0);

        // Switch Kernal off
        set_banking(&mut bus, true, false, true);

        assert_eq!(bus.read(0xE000), 0x77);
    }

    #[test]
    fn test_write_through_to_ram_under_char_rom() {
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, true, false); // Character ROM at $D000

        assert_eq!(bus.read(0xD000), 0xD0); // ROM

        bus.write(0xD000, 0x33);

        // Still ROM
        assert_eq!(bus.read(0xD000), 0xD0);

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);

        assert_eq!(bus.read(0xD000), 0x33);
    }

    #[test]
    fn test_write_through_to_ram_under_io() {
        let mut bus = make_test_bus();
        // Default: I/O at $D000

        // Write to a non-Color-RAM I/O address (e.g., VIC-II register area)
        bus.write(0xD000, 0x55);

        // Should NOT be readable as I/O $FF (no device connected)
        // but should be in RAM underneath
        assert_eq!(bus.read(0xD000), 0xFF); // I/O still returns $FF

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);

        // Note: the write went to RAM, but $D000 is not a VIC-II register location
        // in RAM mode, so we should see what was written
        assert_eq!(bus.read(0xD000), 0x55);
    }

    #[test]
    fn test_rom_to_ram_copy_pattern() {
        // Simulate: FOR I=$A000 TO $BFFF: POKE I, PEEK(I): NEXT
        let mut bus = make_test_bus();

        // Read from ROM, write to same address (goes to RAM)
        for addr in 0xA000u16..=0xA00F {
            let val = bus.read(addr);
            bus.write(addr, val);
        }

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);

        // Verify RAM matches what was in ROM
        for i in 0..=0x0F {
            let addr = 0xA000 + i;
            let expected = ((i as usize) & 0xFF) as u8 ^ 0xA0;
            assert_eq!(
                bus.read(addr),
                expected,
                "ROM-to-RAM copy failed at ${:04X}",
                addr
            );
        }
    }

    #[test]
    fn test_io_port_read_ddr() {
        let bus = make_test_bus();
        // $00 reads the DDR register, not RAM
        assert_eq!(bus.read(0x0000), 0xEF); // default DDR = $EF
    }

    #[test]
    fn test_io_port_write_ddr_updates_register() {
        let mut bus = make_test_bus();

        for ddr in 0x00..=0xFF {
            bus.write(0x0000, ddr);
            assert_eq!(bus.read(0x0000), ddr);
            assert_eq!(bus.data_direction_register, ddr);
        }
    }

    // #[test]
    // fn test_io_port_write_ddr_also_writes_ram() {
    //     let mut bus = make_test_bus();

    //     bus.write(0x0000, 0xAB);

    //     // Switch to all-RAM (doesn't affect $00/$01 — those are always I/O port)
    //     // But VIC-II can read RAM at $00:
    //     bus.set_vic_bank(0);
    //     assert_eq!(bus.vic_read(0x0000), 0xAB); // VIC-II sees RAM
    // }

    #[test]
    fn test_io_port_read_data_port_default() {
        let bus = make_test_bus();
        // $01 reads data port. Default = $37.
        // Bit 4 (cassette switch) = 1 (no button pressed).
        assert_eq!(bus.read(0x0001), 0x37);
    }

    #[test]
    fn test_io_port_write_data_port_updates_banking() {
        let mut bus = make_test_bus();

        // Initially BASIC ROM visible
        assert_eq!(bus.read(0xA000), 0xA0);

        // Clear LORAM → RAM at $A000
        bus.write(0x0001, 0b00110110); // LORAM=0, HIRAM=1, CHAREN=1

        assert_eq!(bus.read(0xA000), 0x00); // Now RAM
        assert_eq!(bus.data_port & 0x07, 0b110); // LORAM=0, HIRAM=1, CHAREN=1
    }

    // #[test]
    // fn test_io_port_write_data_port_writes_to_ram() {
    //     let mut bus = make_test_bus();

    //     bus.write(0x0001, 0b00110100); // LORAM=0, HIRAM=1, CHAREN=0

    //     // VIC-II reads RAM at $01
    //     bus.set_vic_bank(0);
    //     assert_eq!(bus.vic_read(0x0001), 0b00110100);
    // }

    // #[test]
    // fn test_io_port_cassette_switch_input_bit() {
    //     let mut bus = make_test_bus();

    //     // Write $00 to $01 (all output bits = 0)
    //     bus.write(0x0001, 0x00);

    //     // Reading $01: bit 4 is input (cassette switch = 1, no button)
    //     // Bits 6-7 are not connected → read as 0
    //     // Output bits 0-3, 5 = 0
    //     let val = bus.read(0x0001);
    //     assert_eq!(
    //         val & 0x10,
    //         0x10,
    //         "cassette switch bit should be 1 (no button)"
    //     );
    //     assert_eq!(val & 0x07, 0x00, "banking bits should be 0");
    //     assert_eq!(val & 0x20, 0x00, "motor bit should be 0");
    // }

    // // ─── VIC-II Read Path Tests ────────────────────────────────

    // #[test]
    // fn test_vic_read_ram_bank_0() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(0);

    //     // Write to RAM and read via VIC-II
    //     bus.write(0x0400, 0x42); // screen memory
    //     assert_eq!(bus.vic_read(0x0400), 0x42);

    //     bus.write(0x0002, 0x99);
    //     assert_eq!(bus.vic_read(0x0002), 0x99);
    // }

    // #[test]
    // fn test_vic_read_ram_bank_1() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(1);

    //     bus.write(0x4000, 0x33);
    //     assert_eq!(bus.vic_read(0x0000), 0x33); // VIC offset 0 = CPU $4000
    // }

    // #[test]
    // fn test_vic_read_ram_bank_2() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(2);

    //     bus.write(0x8000, 0x44);
    //     assert_eq!(bus.vic_read(0x0000), 0x44);
    // }

    // #[test]
    // fn test_vic_read_ram_bank_3() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(3);

    //     bus.write(0xC000, 0x55);
    //     assert_eq!(bus.vic_read(0x0000), 0x55);
    // }

    // #[test]
    // fn test_vic_read_char_rom_overlay_bank_0() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(0);

    //     // VIC-II sees Character ROM at offset $1000–$1FFF in bank 0
    //     assert_eq!(bus.vic_read(0x1000), 0xD0); // rom[0x4000] = 0x00 ^ 0xD0
    //     assert_eq!(bus.vic_read(0x1001), 0xD1);
    //     assert_eq!(bus.vic_read(0x1FFF), 0xFF ^ 0xD0); // rom[0x4FFF]
    // }

    // #[test]
    // fn test_vic_read_char_rom_overlay_bank_2() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(2);

    //     // VIC-II sees Character ROM at offset $1000–$1FFF in bank 2
    //     assert_eq!(bus.vic_read(0x1000), 0xD0);
    //     assert_eq!(bus.vic_read(0x1FFF), 0xFF ^ 0xD0);
    // }

    // #[test]
    // fn test_vic_read_no_char_rom_overlay_bank_1() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(1);

    //     // Bank 1 does NOT have Character ROM overlay
    //     // VIC-II should see RAM at $1000–$1FFF (which is CPU $5000–$5FFF)
    //     bus.write(0x5000, 0x77);
    //     assert_eq!(bus.vic_read(0x1000), 0x77); // RAM, not Character ROM
    // }

    // #[test]
    // fn test_vic_read_no_char_rom_overlay_bank_3() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(3);

    //     // Bank 3 does NOT have Character ROM overlay
    //     bus.write(0xD000, 0x88); // CPU $D000 = VIC bank 3 offset $1000
    //     assert_eq!(bus.vic_read(0x1000), 0x88); // RAM, not Character ROM
    // }

    // #[test]
    // fn test_vic_read_sees_ram_under_basic_rom() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(2); // VIC bank 2 = $8000–$BFFF

    //     // Write to RAM under BASIC ROM (BASIC ROM is at $A000–$BFFF)
    //     bus.write(0xA000, 0x66);

    //     // CPU sees BASIC ROM
    //     assert_eq!(bus.read(0xA000), 0xA0);

    //     // VIC-II sees RAM
    //     // VIC bank 2 base = $8000, so VIC offset for $A000 = $2000
    //     assert_eq!(bus.vic_read(0x2000), 0x66);
    // }

    // #[test]
    // fn test_vic_read_sees_ram_under_kernal_rom() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(3); // VIC bank 3 = $C000–$FFFF

    //     // Write to RAM under Kernal ROM
    //     bus.write(0xE000, 0x55);

    //     // CPU sees Kernal ROM
    //     assert_eq!(bus.read(0xE000), 0xE0);

    //     // VIC-II sees RAM
    //     // VIC bank 3 base = $C000, so VIC offset for $E000 = $2000
    //     assert_eq!(bus.vic_read(0x2000), 0x55);
    // }

    // #[test]
    // fn test_vic_read_below_char_rom_overlay() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(0);

    //     // Below $1000, VIC-II sees RAM even in bank 0
    //     bus.write(0x0800, 0x44);
    //     assert_eq!(bus.vic_read(0x0800), 0x44);

    //     bus.write(0x0FFF, 0x55);
    //     assert_eq!(bus.vic_read(0x0FFF), 0x55);
    // }

    // #[test]
    // fn test_vic_read_above_char_rom_overlay() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(0);

    //     // Above $1FFF, VIC-II sees RAM
    //     bus.write(0x2000, 0x66);
    //     assert_eq!(bus.vic_read(0x2000), 0x66);
    // }

    // #[test]
    // fn test_vic_bank_default_is_zero() {
    //     let bus = make_test_bus();
    //     assert_eq!(bus.vic_bank(), 0);
    // }

    // // ─── Color RAM Tests ───────────────────────────────────────

    // #[test]
    // fn test_color_ram_write_stores_lower_nybble() {
    //     let mut bus = make_test_bus();
    //     set_banking(&mut bus, true, true, true); // I/O visible

    //     bus.write(0xD800, 0xFF);
    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x0F);

    //     bus.write(0xD800, 0x30);
    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x00);

    //     bus.write(0xD800, 0x4A);
    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x0A);
    // }

    // #[test]
    // fn test_color_ram_full_range() {
    //     let mut bus = make_test_bus();
    //     set_banking(&mut bus, true, true, true);

    //     // Write to first and last byte of Color RAM
    //     bus.write(0xD800, 0x01);
    //     bus.write(0xDBFF, 0x0E);

    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x01);
    //     assert_eq!(bus.read(0xDBFF) & 0x0F, 0x0E);
    // }

    // #[test]
    // fn test_color_ram_not_accessible_when_char_rom_mapped() {
    //     let mut bus = make_test_bus();
    //     set_banking(&mut bus, true, true, true); // I/O mode

    //     bus.write(0xD800, 0x05);
    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x05);

    //     // Switch to Character ROM
    //     set_banking(&mut bus, true, true, false);

    //     // $D800 should now read Character ROM, not Color RAM
    //     // Character ROM at $D800: offset = $D800 - $D000 = $0800
    //     // rom[CHAR_ROM_OFFSET + 0x0800] = (0x800 & 0xFF) ^ 0xD0 = 0x00 ^ 0xD0 = 0xD0
    //     assert_eq!(bus.read(0xD800), 0xD0); // Character ROM data

    //     // Switch back to I/O
    //     set_banking(&mut bus, true, true, true);
    //     assert_eq!(bus.read(0xD800) & 0x0F, 0x05); // Color RAM preserved
    // }

    // #[test]
    // fn test_color_ram_direct_access_for_vic() {
    //     let mut bus = make_test_bus();

    //     // VIC-II can always access Color RAM regardless of CPU banking
    //     bus.color_ram_write(0, 0x0F);
    //     bus.color_ram_write(512, 0x06);
    //     bus.color_ram_write(1023, 0x0C);

    //     assert_eq!(bus.color_ram_read(0), 0x0F);
    //     assert_eq!(bus.color_ram_read(512), 0x06);
    //     assert_eq!(bus.color_ram_read(1023), 0x0C);
    // }

    // #[test]
    // fn test_color_ram_direct_write_masks_upper_nybble() {
    //     let mut bus = make_test_bus();

    //     bus.color_ram_write(0, 0xFF);
    //     assert_eq!(bus.color_ram_read(0), 0x0F);

    //     bus.color_ram_write(0, 0xF0);
    //     assert_eq!(bus.color_ram_read(0), 0x00);
    // }

    // // ─── I/O Block & Mirroring Tests ───────────────────────────

    // #[test]
    // fn test_io_unconnected_reads_return_ff() {
    //     let bus = make_test_bus();
    //     // Default banking: I/O at $D000

    //     // VIC-II registers (no chip connected)
    //     assert_eq!(bus.read(0xD000), 0xFF);
    //     assert_eq!(bus.read(0xD02E), 0xFF);

    //     // SID registers (no chip connected)
    //     assert_eq!(bus.read(0xD400), 0xFF);
    //     assert_eq!(bus.read(0xD41C), 0xFF);

    //     // SID unconnected addresses
    //     assert_eq!(bus.read(0xD41D), 0xFF);
    //     assert_eq!(bus.read(0xD41F), 0xFF);

    //     // CIA #1 (no chip connected)
    //     assert_eq!(bus.read(0xDC00), 0xFF);
    //     assert_eq!(bus.read(0xDC0F), 0xFF);

    //     // CIA #2 (no chip connected)
    //     assert_eq!(bus.read(0xDD00), 0xFF);
    //     assert_eq!(bus.read(0xDD0F), 0xFF);
    // }

    // #[test]
    // fn test_vic2_register_mirror_within_d000_d3ff() {
    //     // VIC-II uses only 6 address bits → mirror every 64 bytes
    //     let bus = make_test_bus();

    //     // All mirrors should return same value ($FF with no device)
    //     assert_eq!(bus.read(0xD000), 0xFF);
    //     assert_eq!(bus.read(0xD040), 0xFF); // mirror 1
    //     assert_eq!(bus.read(0xD080), 0xFF); // mirror 2
    //     assert_eq!(bus.read(0xD3C0), 0xFF); // last mirror
    // }

    // #[test]
    // fn test_sid_register_mirror_within_d400_d7ff() {
    //     // SID uses only 5 address bits → mirror every 32 bytes
    //     let bus = make_test_bus();

    //     assert_eq!(bus.read(0xD400), 0xFF);
    //     assert_eq!(bus.read(0xD420), 0xFF); // mirror 1
    //     assert_eq!(bus.read(0xD440), 0xFF); // mirror 2
    //     assert_eq!(bus.read(0xD7E0), 0xFF); // last mirror
    // }

    // #[test]
    // fn test_cia1_register_mirror_within_dc00_dcff() {
    //     // CIA #1 uses only 4 address bits → mirror every 16 bytes
    //     let bus = make_test_bus();

    //     assert_eq!(bus.read(0xDC00), 0xFF);
    //     assert_eq!(bus.read(0xDC10), 0xFF); // mirror 1
    //     assert_eq!(bus.read(0xDC20), 0xFF); // mirror 2
    //     assert_eq!(bus.read(0xDCF0), 0xFF); // last mirror
    // }

    // #[test]
    // fn test_cia2_register_mirror_within_dd00_ddff() {
    //     // CIA #2 uses only 4 address bits → mirror every 16 bytes
    //     let bus = make_test_bus();

    //     assert_eq!(bus.read(0xDD00), 0xFF);
    //     assert_eq!(bus.read(0xDD10), 0xFF); // mirror 1
    //     assert_eq!(bus.read(0xDD20), 0xFF); // mirror 2
    //     assert_eq!(bus.read(0xDDF0), 0xFF); // last mirror
    // }

    // // ─── Open Bus Tests ────────────────────────────────────────

    // #[test]
    // fn test_io_expansion_de00_dfff_returns_ff() {
    //     let bus = make_test_bus();

    //     // I/O expansion slot 1 ($DE00–$DEFF)
    //     assert_eq!(bus.read(0xDE00), 0xFF);
    //     assert_eq!(bus.read(0xDEFF), 0xFF);

    //     // I/O expansion slot 2 ($DF00–$DFFF)
    //     assert_eq!(bus.read(0xDF00), 0xFF);
    //     assert_eq!(bus.read(0xDFFF), 0xFF);
    // }

    // #[test]
    // fn test_io_expansion_writes_ignored() {
    //     let mut bus = make_test_bus();

    //     bus.write(0xDE00, 0x42);
    //     bus.write(0xDFFF, 0x99);

    //     // Should still read $FF (no device connected)
    //     assert_eq!(bus.read(0xDE00), 0xFF);
    //     assert_eq!(bus.read(0xDFFF), 0xFF);

    //     // But write-through to RAM should still happen
    //     set_banking(&mut bus, false, false, true); // all-RAM
    //     assert_eq!(bus.read(0xDE00), 0x42);
    //     assert_eq!(bus.read(0xDFFF), 0x99);
    // }

    #[test]
    fn test_hardware_vectors_visible_with_kernal_rom() {
        let bus = make_test_bus();
        // Default: Kernal ROM at $E000–$FFFF

        // Vectors are at $FFFA–$FFFF (last 6 bytes of Kernal ROM)
        // Kernal ROM offset for $FFFA = KERNAL_ROM_OFFSET + ($FFFA - $E000)
        // = 0x2000 + 0x1FFA = 0x3FFA
        // Value at rom[0x3FFA] = (0x1FFA & 0xFF) ^ 0xE0 = 0xFA ^ 0xE0 = 0x1A
        assert_eq!(bus.read(0xFFFA), 0xFA ^ 0xE0); // NMI vector low
        assert_eq!(bus.read(0xFFFB), 0xFB ^ 0xE0); // NMI vector high
        assert_eq!(bus.read(0xFFFC), 0xFC ^ 0xE0); // RESET vector low
        assert_eq!(bus.read(0xFFFD), 0xFD ^ 0xE0); // RESET vector high
        assert_eq!(bus.read(0xFFFE), 0xFE ^ 0xE0); // IRQ vector low
        assert_eq!(bus.read(0xFFFF), 0xFF ^ 0xE0); // IRQ vector high
    }

    #[test]
    fn test_hardware_vectors_from_ram_when_kernal_off() {
        let mut bus = make_test_bus();

        // Write custom vectors to RAM under Kernal ROM
        bus.write(0xFFFC, 0x12); // RESET vector low
        bus.write(0xFFFD, 0x34); // RESET vector high

        // Switch Kernal off
        set_banking(&mut bus, false, false, true);

        // Should read from RAM
        assert_eq!(bus.read(0xFFFC), 0x12);
        assert_eq!(bus.read(0xFFFD), 0x34);
    }

    // // ─── Cartridge Mode Tests ──────────────────────────────────

    // #[test]
    // fn test_ultimax_mode_a000_unmapped() {
    //     let mut bus = make_test_bus();
    //     // Ultimax: GAME=low, EXROM=high
    //     bus.set_game(false);
    //     bus.set_exrom(true);

    //     // In Ultimax mode, $A000–$BFFF is unmapped ("open")
    //     // Reads should return $FF (open bus)
    //     assert_eq!(bus.read(0xA000), 0xFF);
    //     assert_eq!(bus.read(0xBFFF), 0xFF);
    // }

    // #[test]
    // fn test_ultimax_mode_e000_is_cart_rom_hi() {
    //     let mut bus = make_test_bus();
    //     bus.set_game(false);
    //     bus.set_exrom(true);

    //     // In Ultimax, $E000–$FFFF is cartridge ROMH (no cart → open bus)
    //     assert_eq!(bus.read(0xE000), 0xFF);
    //     assert_eq!(bus.read(0xFFFF), 0xFF);
    // }

    // #[test]
    // fn test_ultimax_mode_8000_is_cart_rom_lo() {
    //     let mut bus = make_test_bus();
    //     bus.set_game(false);
    //     bus.set_exrom(true);

    //     // In Ultimax, $8000–$9FFF is cartridge ROML (no cart → open bus)
    //     assert_eq!(bus.read(0x8000), 0xFF);
    //     assert_eq!(bus.read(0x9FFF), 0xFF);
    // }

    // #[test]
    // fn test_ultimax_mode_1000_7fff_unmapped() {
    //     let mut bus = make_test_bus();
    //     bus.set_game(false);
    //     bus.set_exrom(true);

    //     // In Ultimax, $1000–$7FFF is unmapped
    //     assert_eq!(bus.read(0x1000), 0xFF);
    //     assert_eq!(bus.read(0x7FFF), 0xFF);
    // }

    // #[test]
    // fn test_ultimax_mode_c000_cfff_unmapped() {
    //     let mut bus = make_test_bus();
    //     bus.set_game(false);
    //     bus.set_exrom(true);

    //     assert_eq!(bus.read(0xC000), 0xFF);
    //     assert_eq!(bus.read(0xCFFF), 0xFF);
    // }

    // #[test]
    // fn test_8k_cart_at_8000() {
    //     let mut bus = make_test_bus();
    //     // 8K cart at $8000: EXROM=low, GAME=high
    //     bus.set_exrom(false);
    //     bus.set_game(true);

    //     // $8000–$9FFF is cartridge ROML (no cart → open bus)
    //     assert_eq!(bus.read(0x8000), 0xFF);

    //     // $A000–$BFFF should still be BASIC ROM (LORAM=1)
    //     set_banking(&mut bus, true, true, true);
    //     assert_eq!(bus.read(0xA000), 0xA0); // BASIC ROM

    //     // $E000 should be Kernal ROM (HIRAM=1)
    //     assert_eq!(bus.read(0xE000), 0xE0);
    // }

    // #[test]
    // fn test_16k_cart_at_8000() {
    //     let mut bus = make_test_bus();
    //     // 16K cart: EXROM=low, GAME=low
    //     bus.set_exrom(false);
    //     bus.set_game(false);

    //     // $8000–$9FFF is cartridge ROML
    //     assert_eq!(bus.read(0x8000), 0xFF); // no cart → open
    //     // $A000–$BFFF is cartridge ROMH
    //     assert_eq!(bus.read(0xA000), 0xFF); // no cart → open
    //     // $E000 is Kernal ROM (HIRAM=1)
    //     set_banking(&mut bus, true, true, true);
    //     assert_eq!(bus.read(0xE000), 0xE0);
    // }

    // #[test]
    // fn test_no_cartridge_default_game_exrom_high() {
    //     let bus = make_test_bus();

    //     assert!(bus.game);
    //     assert!(bus.exrom);
    // }

    #[test]
    fn test_banking_change_takes_effect_immediately() {
        let mut bus = make_test_bus();

        // Start with BASIC ROM visible
        assert_eq!(bus.read(0xA000), 0xA0);

        // Write to $01 to switch LORAM off
        bus.write(0x01, 0b00110110);

        // Immediately, $A000 should be RAM
        assert_eq!(bus.read(0xA000), 0x00);
    }

    #[test]
    fn test_charen_switch_d000_immediately() {
        let mut bus = make_test_bus();

        // Default: I/O at $D000
        assert_eq!(bus.read(0xD000), 0xFF);

        // Clear CHAREN → Character ROM at $D000
        bus.write(0x01, 0b00110011); // LORAM=1, HIRAM=1, CHAREN=0

        assert_eq!(bus.read(0xD000), 0xD0); // Character ROM

        // Set CHAREN again → I/O at $D000
        bus.write(0x01, 0b00110111);

        assert_eq!(bus.read(0xD000), 0xFF); // I/O
    }

    #[test]
    fn test_d000_boundary_char_rom_vs_io() {
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, true, false); // Character ROM

        // Last byte of Character ROM ($DFFF)
        // rom[CHAR_ROM_OFFSET + 0xFFF] = (0xFFF & 0xFF) ^ 0xD0 = 0xFF ^ 0xD0 = 0x2F
        assert_eq!(bus.read(0xDFFF), 0xFF ^ 0xD0);

        // First byte after I/O/Char ROM region ($E000) = Kernal ROM
        assert_eq!(bus.read(0xE000), 0xE0);
    }

    #[test]
    fn test_a000_boundary_ram_vs_basic() {
        let mut bus = make_test_bus();

        // $9FFF is always RAM
        bus.write(0x9FFF, 0x42);
        assert_eq!(bus.read(0x9FFF), 0x42);

        // $A000 is BASIC ROM (default banking)
        assert_eq!(bus.read(0xA000), 0xA0);

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);
        assert_eq!(bus.read(0x9FFF), 0x42); // still RAM
        assert_eq!(bus.read(0xA000), 0x00); // now RAM
    }

    #[test]
    fn test_cfff_d000_boundary() {
        let mut bus = make_test_bus();

        // $CFFF is always RAM
        bus.write(0xCFFF, 0x33);
        assert_eq!(bus.read(0xCFFF), 0x33);

        // $D000 is I/O (default)
        assert_eq!(bus.read(0xD000), 0xFF);

        // Switch to all-RAM
        set_banking(&mut bus, false, false, true);

        // $CFFF still RAM
        assert_eq!(bus.read(0xCFFF), 0x33);
        // $D000 now RAM (write-through should have stored nothing since it was I/O)
        assert_eq!(bus.read(0xD000), 0x00);
    }

    #[test]
    fn test_color_ram_boundary_dbff_dc00() {
        let mut bus = make_test_bus();
        set_banking(&mut bus, true, true, true); // I/O mode

        // $DBFF is last byte of Color RAM
        bus.write(0xDBFF, 0x0F);
        assert_eq!(bus.read(0xDBFF) & 0x0F, 0x0F);

        // $DC00 is CIA #1 (no device → $FF, NOT Color RAM)
        assert_eq!(bus.read(0xDC00), 0xFF);
    }

    // // ─── VIC-II Bank Selection Inversion Test ──────────────────

    // #[test]
    // fn test_vic_bank_selection_inverted_logic() {
    //     // CIA #2 Port A bits 0-1 use inverted logic:
    //     // 11 = bank 0, 10 = bank 1, 01 = bank 2, 00 = bank 3

    //     let mut bus = make_test_bus();

    //     bus.set_vic_bank(0);
    //     bus.write(0x0400, 0x11); // in bank 0 ($0000-$3FFF)
    //     assert_eq!(bus.vic_read(0x0400), 0x11);

    //     bus.set_vic_bank(1);
    //     bus.write(0x4000, 0x22); // in bank 1 ($4000-$7FFF)
    //     assert_eq!(bus.vic_read(0x0000), 0x22);

    //     bus.set_vic_bank(2);
    //     bus.write(0x8000, 0x33); // in bank 2 ($8000-$BFFF)
    //     assert_eq!(bus.vic_read(0x0000), 0x33);

    //     bus.set_vic_bank(3);
    //     bus.write(0xC000, 0x44); // in bank 3 ($C000-$FFFF)
    //     assert_eq!(bus.vic_read(0x0000), 0x44);
    // }

    // // ─── Combined CPU + VIC-II Consistency Tests ───────────────

    // #[test]
    // fn test_cpu_write_visible_to_vic_same_bank() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(0);

    //     // CPU writes to $0400 (screen memory, in VIC bank 0)
    //     bus.write(0x0400, 0x42);

    //     // VIC-II reads the same data
    //     assert_eq!(bus.vic_read(0x0400), 0x42);
    // }

    // #[test]
    // fn test_cpu_write_under_rom_visible_to_vic() {
    //     let mut bus = make_test_bus();
    //     bus.set_vic_bank(2); // VIC bank 2 = $8000-$BFFF

    //     // CPU writes to $A000 (under BASIC ROM)
    //     bus.write(0xA000, 0x99);

    //     // CPU reads BASIC ROM, not the written value
    //     assert_eq!(bus.read(0xA000), 0xA0);

    //     // VIC-II reads the written RAM value
    //     // VIC bank 2 base = $8000, offset for $A000 = $2000
    //     assert_eq!(bus.vic_read(0x2000), 0x99);
    // }

    // #[test]
    // fn test_char_rom_consistency_cpu_vs_vic() {
    //     // The Character ROM data the CPU sees at $D000 (CHAREN=0)
    //     // should be the same data the VIC-II sees at offset $1000 in bank 0
    //     let mut bus = make_test_bus();

    //     // CPU access: CHAREN=0
    //     set_banking(&mut bus, true, true, false);
    //     let cpu_val = bus.read(0xD000);

    //     // VIC-II access: bank 0, offset $1000
    //     bus.set_vic_bank(0);
    //     let vic_val = bus.vic_read(0x1000);

    //     assert_eq!(
    //         cpu_val, vic_val,
    //         "Character ROM data should be identical for CPU and VIC-II"
    //     );

    //     // Check a few more addresses
    //     for offset in [0x0001, 0x0800, 0x0FFF] {
    //         let cpu_val = bus.read(0xD000 + offset);
    //         let vic_val = bus.vic_read(0x1000 + offset);
    //         assert_eq!(
    //             cpu_val, vic_val,
    //             "Char ROM mismatch at offset ${:04X}",
    //             offset
    //         );
    //     }
    // }
}
