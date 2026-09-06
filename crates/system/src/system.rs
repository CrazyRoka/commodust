use bus::Bus;
use cpu::{BusOperation, Mos6510};
use vic::VideoStandard;

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
}
