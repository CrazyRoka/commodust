use bus::Bus;
use cpu::{BusOperation, Mos6510};
use keyboard::C64Key;
use vic::VideoStandard;

use crate::prg::Prg;

const ROM_BYTES: &[u8] = include_bytes!("../../../roms/c64.bin");

pub struct System {
    cpu: Mos6510,
    pub bus: Bus,
    cpu_data: u8,
}

impl System {
    pub fn new(standard: VideoStandard) -> Self {
        let mut cpu = Mos6510::new();
        let bus = Bus::new(ROM_BYTES, standard);

        let pc_low = bus.read(0xFFFC) as u16;
        let pc_high = bus.read(0xFFFD) as u16;
        let start_pc = (pc_high << 8) | pc_low;

        cpu.set_pc(start_pc);

        Self {
            cpu,
            bus,
            cpu_data: 0xFF,
        }
    }

    pub fn tick(&mut self) {
        let nmi_prev = self.bus.nmi();

        self.bus.tick();

        if !nmi_prev && self.bus.nmi() {
            self.cpu.nmi();
        }
        self.cpu.set_irq(self.bus.irq());

        if !self.bus.ba() {
            return;
        }

        let operation = self.cpu.step(self.cpu_data);
        // println!("CPU: <{:?}>. Operation: <{:?}>", self.cpu, operation);

        match operation {
            BusOperation::Read(addr) => self.cpu_data = self.bus.read(addr),
            BusOperation::Write(addr, value) => {
                self.cpu_data = 0xFF;
                self.bus.write(addr, value)
            }
        }
    }

    pub fn press_key(&mut self, key: &C64Key) {
        self.bus.keyboard_mut().press_key(key);
    }

    pub fn release_key(&mut self, key: &C64Key) {
        self.bus.keyboard_mut().release_key(key);
    }

    pub fn load_prg(&mut self, prg: &Prg, autorun: bool) {
        for (idx, value) in prg.data().iter().enumerate() {
            if let Some(addr) = prg.load_address().checked_add(idx as u16) {
                self.bus.ram_mut()[addr as usize] = *value
            } else {
                break;
            }
        }

        if prg.load_address() == 0x0801 {
            let end_address = prg.load_address() + prg.data().len() as u16;

            // Low byte to $2D, High byte to $2E
            self.bus.write(0x002D, (end_address & 0xFF) as u8);
            self.bus.write(0x002E, (end_address >> 8) as u8);

            // Low byte to $2F, High byte to $30
            self.bus.write(0x002F, (end_address & 0xFF) as u8);
            self.bus.write(0x0030, (end_address >> 8) as u8);

            // Low byte to $31, High byte to $32
            self.bus.write(0x0031, (end_address & 0xFF) as u8);
            self.bus.write(0x0032, ((end_address >> 8) & 0xFF) as u8);
        }

        if autorun {
            self.bus.write(0x0277, 0x52); // 'R'
            self.bus.write(0x0278, 0x55); // 'U'
            self.bus.write(0x0279, 0x4E); // 'N'
            self.bus.write(0x027A, 0x0D); // 'Enter'
            self.bus.write(0x00C6, 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn irq_fires_only_once_when_acknowledged() {
        let mut sys = System::new(VideoStandard::Pal);

        // Map RAM at $E000-$FFFF while keeping I/O at $D000-$DFFF.
        sys.bus.write(0x0001, 0x35);

        // Counter at $0400 = 0.
        sys.bus.write(0x0400, 0x00);

        // Main loop at $1000: CLI; JMP $1001
        sys.bus.write(0x1000, 0x58); // CLI
        sys.bus.write(0x1001, 0x4C); // JMP $1001
        sys.bus.write(0x1002, 0x01);
        sys.bus.write(0x1003, 0x10);

        // IRQ handler at $2000: INC $0400; LDA $DC0D; RTI
        sys.bus.write(0x2000, 0xEE); // INC $0400
        sys.bus.write(0x2001, 0x00);
        sys.bus.write(0x2002, 0x04);
        sys.bus.write(0x2003, 0xAD); // LDA $DC0D (acknowledges CIA #1 IRQ)
        sys.bus.write(0x2004, 0x0D);
        sys.bus.write(0x2005, 0xDC);
        sys.bus.write(0x2006, 0x40); // RTI

        // Hardware IRQ vector $FFFE/$FFFF -> $2000.
        sys.bus.write(0xFFFE, 0x00);
        sys.bus.write(0xFFFF, 0x20);

        sys.cpu.set_pc(0x1000);

        // Trigger a single CIA #1 Timer A one-shot IRQ after 16 cycles.
        sys.bus.write(0xDC04, 0x10);
        sys.bus.write(0xDC05, 0x00);
        sys.bus.write(0xDC0D, 0x81); // Enable Timer A IRQ
        sys.bus.write(0xDC0E, 0x09); // Start Timer A, one-shot

        for _ in 0..200 {
            sys.tick();
        }

        assert_eq!(
            sys.bus.ram()[0x0400],
            1,
            "A single acknowledged IRQ must only run the ISR once"
        );
    }

    #[test]
    fn nmi_fires_only_once_per_falling_edge() {
        let mut sys = System::new(VideoStandard::Pal);

        // Map RAM at $E000-$FFFF while keeping I/O at $D000-$DFFF.
        sys.bus.write(0x0001, 0x35);
        sys.bus.write(0x0400, 0x00);

        // Main loop at $1000: JMP $1000
        sys.bus.write(0x1000, 0x4C);
        sys.bus.write(0x1001, 0x00);
        sys.bus.write(0x1002, 0x10);

        // NMI handler at $2000: INC $0400; LDA $DD0D; RTI
        sys.bus.write(0x2000, 0xEE); // INC $0400
        sys.bus.write(0x2001, 0x00);
        sys.bus.write(0x2002, 0x04);
        sys.bus.write(0x2003, 0xAD); // LDA $DD0D (acknowledges CIA #2 NMI)
        sys.bus.write(0x2004, 0x0D);
        sys.bus.write(0x2005, 0xDD);
        sys.bus.write(0x2006, 0x40); // RTI

        // Hardware NMI vector $FFFA/$FFFB -> $2000.
        sys.bus.write(0xFFFA, 0x00);
        sys.bus.write(0xFFFB, 0x20);

        sys.cpu.set_pc(0x1000);

        // Trigger a single CIA #2 Timer A one-shot NMI after 16 cycles.
        sys.bus.write(0xDD04, 0x10);
        sys.bus.write(0xDD05, 0x00);
        sys.bus.write(0xDD0D, 0x81); // Enable Timer A NMI
        sys.bus.write(0xDD0E, 0x09); // Start Timer A, one-shot

        for _ in 0..200 {
            sys.tick();
        }

        assert_eq!(
            sys.bus.ram()[0x0400],
            1,
            "A single NMI edge must only run the NMI handler once"
        );
    }

    #[test]
    fn load_prg_writes_to_ram_without_corrupting_io() {
        let mut sys = System::new(VideoStandard::Pal);

        // Set VIC-II border color ($D020) to 6 (blue).
        sys.bus.write(0xD020, 0x06);

        // PRG loading 1 byte (0x00) at $D020.
        let prg = Prg::from_bytes(&[0x20, 0xD0, 0x00]).unwrap();
        sys.load_prg(&prg, false);

        assert_eq!(sys.bus.ram()[0xD020], 0x00);
        assert_eq!(
            sys.bus.read(0xD020) & 0x0F,
            0x06,
            "load_prg must not overwrite hardware I/O registers"
        );
    }

    #[test]
    fn load_prg_does_not_overflow_at_end_of_address_space() {
        let mut sys = System::new(VideoStandard::Pal);
        sys.bus.ram_mut()[0x0000] = 0xAA;

        // PRG starting at $FFFE with 3 bytes (3rd byte must not wrap into $0000).
        let prg = Prg::from_bytes(&[0xFE, 0xFF, 0x11, 0x22, 0x33]).unwrap();
        sys.load_prg(&prg, false);

        assert_eq!(sys.bus.ram()[0xFFFE], 0x11);
        assert_eq!(sys.bus.ram()[0xFFFF], 0x22);
        assert_eq!(sys.bus.ram()[0x0000], 0xAA);
    }
}
