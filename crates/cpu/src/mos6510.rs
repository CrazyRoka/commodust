#[derive(Debug)]
pub enum BusOperation {
    Read(u16),
    Write(u16, u8),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

enum AddressModeProcessingResult {
    Operation(BusOperation),
    Finished,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Instruction {
    Ora,
}

const STATUS_REGISTER_CARRY: u8 = 0x01;
const STATUS_REGISTER_ZERO: u8 = 0x02;
const STATUS_REGISTER_INTERRUPT_DISABLE: u8 = 0x04;
const STATUS_REGISTER_DECIMAL_MODE: u8 = 0x08;
const STATUS_REGISTER_BREAK: u8 = 0x10;
const STATUS_REGISTER_UNUSED: u8 = 0x20;
const STATUS_REGISTER_OVERFLOW: u8 = 0x40;
const STATUS_REGISTER_NEGATIVE: u8 = 0x80;

#[derive(PartialEq, Eq, Debug)]
pub struct Mos6510 {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u16,
    pc: u16,
    cycle: u8,
    instruction: Instruction,
    addressing_mode: AddressingMode,
    value_latch: u8,
    addr_latch: u16,
}

impl Mos6510 {
    pub fn init() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            sp: 0,
            pc: 0,
            cycle: 0,
            instruction: Instruction::Ora,
            addressing_mode: AddressingMode::Immediate,
            value_latch: 0,
            addr_latch: 0,
        }
    }

    pub fn step(&mut self, data_in: u8) -> BusOperation {
        if self.cycle == 0 {
            let addr = self.pc;
            self.pc = self.pc.wrapping_add(1);
            self.cycle = 1;
            return BusOperation::Read(addr);
        }

        if self.cycle == 1 {
            self.decode_instruction(data_in);
        }

        if let AddressModeProcessingResult::Operation(operation) = self.step_addressing(data_in) {
            return operation;
        }

        self.execute_instruction()
    }

    fn step_addressing(&mut self, data_in: u8) -> AddressModeProcessingResult {
        match self.addressing_mode {
            AddressingMode::Implied => todo!(),
            AddressingMode::Accumulator => todo!(),
            AddressingMode::Immediate => match self.cycle {
                1 => {
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    self.cycle = 2;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                2 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => panic!(
                    "Unexpected cycle for Immediate addressing mode {}",
                    self.cycle
                ),
            },
            AddressingMode::ZeroPage => match self.cycle {
                1 => {
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    self.cycle = 2;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                2 => {
                    self.cycle = 3;
                    self.addr_latch = data_in as u16;
                    AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                }
                3 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => panic!(
                    "Unexpected cycle for Immediate addressing mode {}",
                    self.cycle
                ),
            },
            AddressingMode::ZeroPageX => todo!(),
            AddressingMode::ZeroPageY => todo!(),
            AddressingMode::Absolute => match self.cycle {
                1 => {
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    self.cycle = 2;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                2 => {
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    self.cycle = 3;
                    self.addr_latch = data_in as u16;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                3 => {
                    self.cycle = 4;
                    self.addr_latch |= (data_in as u16) << 8;
                    AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                }
                4 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => panic!(
                    "Unexpected cycle for Immediate addressing mode {}",
                    self.cycle
                ),
            },
            AddressingMode::AbsoluteX => todo!(),
            AddressingMode::AbsoluteY => todo!(),
            AddressingMode::Indirect => todo!(),
            AddressingMode::IndirectX => todo!(),
            AddressingMode::IndirectY => todo!(),
            AddressingMode::Relative => todo!(),
        }
    }

    fn execute_instruction(&mut self) -> BusOperation {
        match self.instruction {
            Instruction::Ora => {
                self.a |= self.value_latch;
                if self.a == 0 {
                    self.p |= STATUS_REGISTER_ZERO;
                } else {
                    self.p &= !STATUS_REGISTER_ZERO;
                }
                if self.a & 0x80 != 0 {
                    self.p |= STATUS_REGISTER_NEGATIVE;
                } else {
                    self.p &= !STATUS_REGISTER_NEGATIVE;
                }
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                self.step(0xFF)
            }
        }
    }

    fn decode_instruction(&mut self, opcode: u8) {
        (self.instruction, self.addressing_mode) = match opcode & 0x1F {
            // ALU block
            // ORA
            0x01 => (Instruction::Ora, AddressingMode::IndirectX),
            0x05 => (Instruction::Ora, AddressingMode::ZeroPage),
            0x09 => (Instruction::Ora, AddressingMode::Immediate),
            0x0D => (Instruction::Ora, AddressingMode::Absolute),
            0x11 => (Instruction::Ora, AddressingMode::IndirectY),
            0x15 => (Instruction::Ora, AddressingMode::ZeroPageX),
            0x19 => (Instruction::Ora, AddressingMode::AbsoluteY),
            0x1D => (Instruction::Ora, AddressingMode::AbsoluteX),
            _ => todo!("Unsupported opcode {:#02X}", opcode & 0x1f),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::Path};

    use serde::Deserialize;

    use crate::{Mos6510, mos6510::BusOperation};

    #[derive(Deserialize)]
    struct TestCaseState {
        pc: u16,
        s: u16,
        a: u8,
        x: u8,
        y: u8,
        p: u8,
        ram: Vec<(u16, u8)>,
    }

    impl TestCaseState {
        fn create_cpu(&self) -> Mos6510 {
            let mut cpu = Mos6510::init();

            cpu.pc = self.pc;
            cpu.sp = self.s;
            cpu.a = self.a;
            cpu.x = self.x;
            cpu.y = self.y;
            cpu.p = self.p;

            cpu
        }

        fn create_memory(&self) -> HashMap<u16, u8> {
            let mut memory = HashMap::default();

            for (addr, value) in self.ram.iter() {
                memory.insert(*addr, *value);
            }

            memory
        }
    }

    #[derive(Deserialize, Debug)]
    enum TestCaseCycleType {
        #[serde(rename = "read")]
        Read,
        #[serde(rename = "write")]
        Write,
    }

    #[derive(Deserialize)]
    struct TestCase {
        name: String,
        #[serde(rename = "initial")]
        initial_state: TestCaseState,
        #[serde(rename = "final")]
        final_state: TestCaseState,
        cycles: Vec<(u16, u8, TestCaseCycleType)>,
    }

    macro_rules! mos6502_tests {
        ($($name:ident => $file:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    execute_test_cases($file);
                }
            )*
        };
    }

    fn execute_test_cases(test_file_name: &str) {
        let test_path = "tests/mos6502/6502/v1";
        let full_test_path = Path::new(test_path).join(test_file_name);
        let test_cases_string = fs::read_to_string(&full_test_path)
            .expect(&format!("Failed to read test file: {full_test_path:?}"));
        let test_cases: Vec<TestCase> =
            serde_json::from_str(&test_cases_string).expect("Test case file failed to deserialize");

        for case in test_cases {
            let mut memory = case.initial_state.create_memory();
            let mut cpu = case.initial_state.create_cpu();
            // Request opcode read
            let mut request = cpu.step(0xFF);

            for (addr, value, cycle_type) in case.cycles {
                match (&cycle_type, &request) {
                    (TestCaseCycleType::Read, BusOperation::Read(cpu_read_addr)) => {
                        assert_eq!(
                            addr, *cpu_read_addr,
                            "Test case '{}': Read address doesn't match",
                            case.name
                        );
                        request = cpu.step(value);
                    }
                    (
                        TestCaseCycleType::Write,
                        BusOperation::Write(cpu_write_addr, cpu_write_value),
                    ) => {
                        assert_eq!(
                            addr, *cpu_write_addr,
                            "Test case '{}': Write address doesn't match",
                            case.name
                        );
                        assert_eq!(
                            value, *cpu_write_value,
                            "Test case '{}': Write value doesn't match",
                            case.name
                        );
                        request = cpu.step(0xFF);
                    }
                    _ => panic!(
                        "Test failed. Unexpected cpu operation {:?}, wanted {:?}",
                        request, cycle_type
                    ),
                }
            }

            let mut expected_cpu = case.final_state.create_cpu();
            // Requesting opcode to match with actual cpu state
            expected_cpu.step(0xFF);
            expected_cpu.instruction = cpu.instruction;
            expected_cpu.addressing_mode = cpu.addressing_mode;

            let expected_memory = case.final_state.create_memory();
            assert_eq!(
                expected_memory, memory,
                "Test Case '{}': RAM state doesn't match the expected final state",
                case.name
            );
            assert_eq!(
                expected_cpu, cpu,
                "Test Case '{}': CPU state doesn't match the expected final state",
                case.name
            );
        }
    }

    mos6502_tests! {
        test_ora_zero     => "05.json",
        test_ora_imm      => "09.json",
        test_ora_abs      => "0d.json",
    }
}
