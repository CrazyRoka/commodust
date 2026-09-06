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
        self.bus.tick();

        if self.bus.nmi() {
            self.cpu.nmi();
        }
        if self.bus.irq() {
            self.cpu.irq();
        }

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
            self.bus.write(prg.load_address() + idx as u16, *value);
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
