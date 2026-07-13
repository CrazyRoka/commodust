const RAM_SIZE: usize = 0x10000;
const ROM_SIZE: usize = 0x5000;

pub struct Bus {
    ram: [u8; RAM_SIZE],
    rom: [u8; ROM_SIZE],
}

impl Bus {
    fn new(rom: &[u8]) -> Self {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Bus,
        bus::{RAM_SIZE, ROM_SIZE},
    };

    #[test]
    fn test_bus_new_creation_successful() {
        let mut rom = [0; ROM_SIZE];
        for i in 0..ROM_SIZE {
            rom[i] = (i & 0xFF) as u8;
        }
        let bus = Bus::new(&rom);

        assert_eq!(bus.ram, [0; RAM_SIZE]);
        assert_eq!(bus.rom, rom);
    }

    #[test]
    #[should_panic(expected = "ROM size should be equal to 20480")]
    fn test_bus_fails_when_rom_size_is_wrong() {
        Bus::new(&[0; 123]);
    }
}
