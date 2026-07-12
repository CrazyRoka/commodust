use crate::mos6510::AddressingMode::AbsoluteX;

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
    And,
    Eor,
    Nop,
    Sta,
    Lda,
    Cmp,
    Jam,
    Asl,
    Rol,
    Lsr,
    Ror,
    Stx,
    Txa,
    Txs,
    Shx,
    Ldx,
    Tax,
    Tsx,
    Dex,
    Dec,
    Inc,
    Anc,
    Slo,
    Rla,
    Alr,
    Sre,
}

impl Instruction {
    fn is_writing(&self) -> bool {
        match self {
            Self::Ora => false,
            Self::And => false,
            Self::Eor => false,
            Self::Nop => false,
            Self::Sta => true,
            Self::Lda => false,
            Self::Cmp => false,
            Self::Jam => false,
            Self::Asl => false,
            Self::Rol => false,
            Self::Lsr => false,
            Self::Ror => false,
            Self::Stx => true,
            Self::Txa => false,
            Self::Txs => false,
            Self::Shx => true,
            Self::Ldx => false,
            Self::Tax => false,
            Self::Tsx => false,
            Self::Dex => false,
            Self::Dec => false,
            Self::Inc => false,
            Self::Anc => false,
            Self::Slo => false,
            Self::Rla => false,
            Self::Alr => false,
            Self::Sre => false,
        }
    }

    fn is_reading_and_writing(&self) -> bool {
        match self {
            Self::Ora => false,
            Self::And => false,
            Self::Eor => false,
            Self::Nop => false,
            Self::Sta => false,
            Self::Lda => false,
            Self::Cmp => false,
            Self::Jam => false,
            Self::Asl => true,
            Self::Rol => true,
            Self::Lsr => true,
            Self::Ror => true,
            Self::Stx => false,
            Self::Txa => false,
            Self::Txs => false,
            Self::Shx => false,
            Self::Ldx => false,
            Self::Tax => false,
            Self::Tsx => false,
            Self::Dex => true,
            Self::Dec => true,
            Self::Inc => true,
            Self::Anc => false,
            Self::Slo => true,
            Self::Rla => true,
            Self::Alr => false,
            Self::Sre => true,
        }
    }
}

const STATUS_REGISTER_CARRY: u8 = 0x01;
const STATUS_REGISTER_ZERO: u8 = 0x02;
const STATUS_REGISTER_INTERRUPT_DISABLE: u8 = 0x04;
const STATUS_REGISTER_DECIMAL_MODE: u8 = 0x08;
const STATUS_REGISTER_BREAK: u8 = 0x10;
const STATUS_REGISTER_UNUSED: u8 = 0x20;
const STATUS_REGISTER_OVERFLOW: u8 = 0x40;
const STATUS_REGISTER_NEGATIVE: u8 = 0x80;

const INSTRUCTION_EXECUTION_PHASE: u8 = 200;

#[derive(PartialEq, Eq, Debug)]
pub struct Mos6510 {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    pc: u16,
    cycle: u8,
    instruction: Instruction,
    addressing_mode: AddressingMode,
    value_latch: u8,
    addr_latch: u16,
    jammed: u8,
}

impl Mos6510 {
    pub fn init() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc: 0,
            cycle: 0,
            instruction: Instruction::Ora,
            addressing_mode: AddressingMode::Immediate,
            value_latch: 0,
            addr_latch: 0,
            jammed: 0,
        }
    }

    pub fn step(&mut self, data_in: u8) -> BusOperation {
        if self.jammed > 0 {
            if self.jammed == 1 {
                self.jammed += 1;
                return BusOperation::Read(0xFFFF);
            } else if self.jammed < 4 {
                self.jammed += 1;
                return BusOperation::Read(0xFFFE);
            } else {
                return BusOperation::Read(0xFFFF);
            }
        }

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
        if self.cycle >= INSTRUCTION_EXECUTION_PHASE {
            return AddressModeProcessingResult::Finished;
        }

        match self.addressing_mode {
            AddressingMode::Implied => AddressModeProcessingResult::Finished,
            AddressingMode::Accumulator => match self.cycle {
                1 => {
                    self.cycle = 2;
                    AddressModeProcessingResult::Operation(BusOperation::Read(self.pc))
                }
                2 => {
                    self.value_latch = self.a;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for Accumulator addressing mode {}",
                    self.cycle
                ),
            },
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
                _ => unreachable!(
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
                    if self.instruction.is_writing() {
                        AddressModeProcessingResult::Finished
                    } else {
                        AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                    }
                }
                3 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for ZeroPage addressing mode {}",
                    self.cycle
                ),
            },
            AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => match self.cycle {
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
                    self.cycle = 4;
                    let add = if self.addressing_mode == AddressingMode::ZeroPageX {
                        self.x
                    } else {
                        self.y
                    };
                    self.addr_latch = (self.addr_latch as u8).wrapping_add(add) as u16;
                    if self.instruction.is_writing() {
                        AddressModeProcessingResult::Finished
                    } else {
                        AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                    }
                }
                4 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
                ),
            },
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
                    if self.instruction.is_writing() {
                        AddressModeProcessingResult::Finished
                    } else {
                        AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                    }
                }
                4 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for Absolute addressing mode {}",
                    self.cycle
                ),
            },
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                let add = if self.addressing_mode == AbsoluteX {
                    self.x
                } else {
                    self.y
                };
                match self.cycle {
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

                        AddressModeProcessingResult::Operation(BusOperation::Read(
                            (self.addr_latch & 0xFF00)
                                | ((self.addr_latch & 0x00FF) as u8).wrapping_add(add) as u16,
                        ))
                    }
                    4 => {
                        let page_cross = self.addr_latch.wrapping_add(add as u16) & 0xFF00
                            == self.addr_latch & 0xFF00;
                        self.addr_latch = self.addr_latch.wrapping_add(add as u16);
                        if !self.instruction.is_reading_and_writing() && page_cross
                            || self.instruction.is_writing()
                        {
                            self.value_latch = data_in;
                            AddressModeProcessingResult::Finished
                        } else {
                            self.cycle = 5;
                            AddressModeProcessingResult::Operation(BusOperation::Read(
                                self.addr_latch,
                            ))
                        }
                    }
                    5 => {
                        self.value_latch = data_in;
                        AddressModeProcessingResult::Finished
                    }
                    _ => unreachable!(
                        "Unexpected cycle for {:?} addressing mode {}",
                        self.addressing_mode, self.cycle
                    ),
                }
            }
            AddressingMode::Indirect => todo!(),
            AddressingMode::IndirectX => match self.cycle {
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
                    self.cycle = 4;
                    self.addr_latch = self.x.wrapping_add(self.addr_latch as u8) as u16;
                    AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                }
                4 => {
                    let addr = (self.addr_latch as u8).wrapping_add(1) as u16;
                    self.cycle = 5;
                    self.addr_latch = data_in as u16;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                5 => {
                    self.cycle = 6;
                    self.addr_latch |= (data_in as u16) << 8;
                    if self.instruction.is_writing() {
                        AddressModeProcessingResult::Finished
                    } else {
                        AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                    }
                }
                6 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
                ),
            },
            AddressingMode::IndirectY => match self.cycle {
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
                    let addr = (self.addr_latch as u8).wrapping_add(1) as u16;
                    self.cycle = 4;
                    self.addr_latch = data_in as u16;
                    AddressModeProcessingResult::Operation(BusOperation::Read(addr))
                }
                4 => {
                    self.cycle = 5;
                    self.addr_latch |= (data_in as u16) << 8;
                    AddressModeProcessingResult::Operation(BusOperation::Read(
                        (self.addr_latch & 0xFF00)
                            | ((self.addr_latch & 0x00FF) as u8).wrapping_add(self.y) as u16,
                    ))
                }
                5 => {
                    let page_cross = self.addr_latch.wrapping_add(self.y as u16) & 0xFF00
                        == self.addr_latch & 0xFF00;
                    self.addr_latch = self.addr_latch.wrapping_add(self.y as u16);
                    if !self.instruction.is_reading_and_writing() && page_cross
                        || self.instruction.is_writing()
                    {
                        self.value_latch = data_in;
                        AddressModeProcessingResult::Finished
                    } else {
                        self.cycle = 6;
                        AddressModeProcessingResult::Operation(BusOperation::Read(self.addr_latch))
                    }
                }
                6 => {
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
                ),
            },
            AddressingMode::Relative => todo!(),
        }
    }

    fn execute_instruction(&mut self) -> BusOperation {
        if self.cycle < INSTRUCTION_EXECUTION_PHASE {
            self.cycle = INSTRUCTION_EXECUTION_PHASE;
        }

        match self.instruction {
            Instruction::Ora => {
                self.a |= self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::And => {
                self.a &= self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::Eor => {
                self.a ^= self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::Nop => {
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                return BusOperation::Read(self.pc);
            }
            Instruction::Sta => {
                let addr = self.addr_latch;
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                return BusOperation::Write(addr, self.a);
            }
            Instruction::Lda => {
                self.a = self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::Cmp => {
                let (res, overflow) = self.a.overflowing_sub(self.value_latch);
                self.set_nz_flags(res);
                self.set_carry(!overflow);
            }
            Instruction::Jam => {
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                self.jammed = 1;
                return BusOperation::Read(self.pc);
            }
            Instruction::Asl => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE
                        if self.addressing_mode == AddressingMode::Accumulator =>
                    {
                        let carry = self.value_latch & 0x80 != 0;
                        self.a = self.value_latch << 1;
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                    }
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let carry = self.value_latch & 0x80 != 0;
                        let res = self.value_latch << 1;
                        self.set_nz_flags(res);
                        self.set_carry(carry);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected ASL execution cycle {}", self.cycle),
                };
            }
            Instruction::Rol => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE
                        if self.addressing_mode == AddressingMode::Accumulator =>
                    {
                        let carry = self.value_latch & 0x80 != 0;
                        self.a = (self.value_latch << 1)
                            | if self.p & STATUS_REGISTER_CARRY != 0 {
                                1
                            } else {
                                0
                            };
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                    }
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let carry = self.value_latch & 0x80 != 0;
                        let res = (self.value_latch << 1)
                            | if self.p & STATUS_REGISTER_CARRY != 0 {
                                1
                            } else {
                                0
                            };
                        self.set_nz_flags(res);
                        self.set_carry(carry);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected ROL execution cycle {}", self.cycle),
                };
            }
            Instruction::Lsr => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE
                        if self.addressing_mode == AddressingMode::Accumulator =>
                    {
                        let carry = self.value_latch & 0x01 != 0;
                        self.a = self.value_latch >> 1;
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                    }
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let carry = self.value_latch & 0x01 != 0;
                        let res = self.value_latch >> 1;
                        self.set_nz_flags(res);
                        self.set_carry(carry);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected LSR execution cycle {}", self.cycle),
                };
            }
            Instruction::Ror => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE
                        if self.addressing_mode == AddressingMode::Accumulator =>
                    {
                        let carry = self.value_latch & 0x01 != 0;
                        self.a = (self.value_latch >> 1)
                            | if self.p & STATUS_REGISTER_CARRY != 0 {
                                0x80
                            } else {
                                0
                            };
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                    }
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let carry = self.value_latch & 0x01 != 0;
                        let res = self.value_latch >> 1
                            | if self.p & STATUS_REGISTER_CARRY != 0 {
                                0x80
                            } else {
                                0
                            };
                        self.set_nz_flags(res);
                        self.set_carry(carry);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected ROR execution cycle {}", self.cycle),
                };
            }
            Instruction::Stx => {
                let addr = self.addr_latch;
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                return BusOperation::Write(addr, self.x);
            }
            Instruction::Txa => {
                self.a = self.x;
                self.set_nz_flags(self.a);
            }
            Instruction::Txs => {
                self.s = self.x;
            }
            Instruction::Shx => {
                let h = (self.addr_latch.wrapping_sub(self.y as u16) >> 8) as u8;
                let val = self.x & h.wrapping_add(1);
                let page_crossed = (self.addr_latch >> 8) as u8 != h;
                let addr = if page_crossed {
                    ((val as u16) << 8) | (self.addr_latch & 0x00FF)
                } else {
                    self.addr_latch
                };
                self.cycle = 0;
                self.value_latch = 0;
                self.addr_latch = 0;
                return BusOperation::Write(addr, val);
            }
            Instruction::Ldx => {
                self.x = self.value_latch;
                self.set_nz_flags(self.x);
            }
            Instruction::Tax => {
                self.x = self.a;
                self.set_nz_flags(self.x);
            }
            Instruction::Tsx => {
                self.x = self.s;
                self.set_nz_flags(self.x);
            }
            Instruction::Dex => {
                self.x = self.x.wrapping_sub(1);
                self.set_nz_flags(self.x);
            }
            Instruction::Dec => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let res = self.value_latch.wrapping_sub(1);
                        self.set_nz_flags(res);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected INC execution cycle {}", self.cycle),
                };
            }
            Instruction::Inc => {
                match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        return BusOperation::Write(self.addr_latch, self.value_latch);
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let res = self.value_latch.wrapping_add(1);
                        self.set_nz_flags(res);
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!("Unexpected INC execution cycle {}", self.cycle),
                };
            }
            Instruction::Anc => {
                self.a &= self.value_latch;
                self.set_nz_flags(self.a);
                self.set_carry(self.p & STATUS_REGISTER_NEGATIVE != 0);
            }
            Instruction::Slo => {
                return match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        BusOperation::Write(self.addr_latch, self.value_latch)
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let overflow = self.value_latch & 0x80 != 0;
                        let res = self.value_latch << 1;
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        self.a |= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(overflow);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!("Unexpected SLO execution cycle {}", self.cycle),
                };
            }
            Instruction::Rla => {
                return match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        BusOperation::Write(self.addr_latch, self.value_latch)
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let old_carry = if self.p & STATUS_REGISTER_CARRY != 0 {
                            1
                        } else {
                            0
                        };
                        let overflow = self.value_latch & 0x80 != 0;
                        let res = (self.value_latch << 1) | old_carry;
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        self.a &= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(overflow);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!("Unexpected RLA execution cycle {}", self.cycle),
                };
            }
            Instruction::Alr => {
                let carry = self.a & self.value_latch & 1 == 1;
                self.a = (self.a & self.value_latch) >> 1;
                self.set_nz_flags(self.a);
                self.set_carry(carry);
            }
            Instruction::Sre => {
                return match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        BusOperation::Write(self.addr_latch, self.value_latch)
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let overflow = self.value_latch & 0x01 != 0;
                        let res = self.value_latch >> 1;
                        let addr = self.addr_latch;
                        self.cycle = 0;
                        self.value_latch = 0;
                        self.addr_latch = 0;
                        self.a ^= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(overflow);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!("Unexpected SRE execution cycle {}", self.cycle),
                };
            }
        }

        self.cycle = 0;
        self.value_latch = 0;
        self.addr_latch = 0;
        self.step(0xFF)
    }

    fn decode_instruction(&mut self, opcode: u8) {
        (self.instruction, self.addressing_mode) = match opcode {
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
            // AND
            0x21 => (Instruction::And, AddressingMode::IndirectX),
            0x25 => (Instruction::And, AddressingMode::ZeroPage),
            0x29 => (Instruction::And, AddressingMode::Immediate),
            0x2D => (Instruction::And, AddressingMode::Absolute),
            0x31 => (Instruction::And, AddressingMode::IndirectY),
            0x35 => (Instruction::And, AddressingMode::ZeroPageX),
            0x39 => (Instruction::And, AddressingMode::AbsoluteY),
            0x3D => (Instruction::And, AddressingMode::AbsoluteX),
            // EOR
            0x41 => (Instruction::Eor, AddressingMode::IndirectX),
            0x45 => (Instruction::Eor, AddressingMode::ZeroPage),
            0x49 => (Instruction::Eor, AddressingMode::Immediate),
            0x4D => (Instruction::Eor, AddressingMode::Absolute),
            0x51 => (Instruction::Eor, AddressingMode::IndirectY),
            0x55 => (Instruction::Eor, AddressingMode::ZeroPageX),
            0x59 => (Instruction::Eor, AddressingMode::AbsoluteY),
            0x5D => (Instruction::Eor, AddressingMode::AbsoluteX),
            // STA
            0x81 => (Instruction::Sta, AddressingMode::IndirectX),
            0x85 => (Instruction::Sta, AddressingMode::ZeroPage),
            0x89 => (Instruction::Nop, AddressingMode::Immediate),
            0x8D => (Instruction::Sta, AddressingMode::Absolute),
            0x91 => (Instruction::Sta, AddressingMode::IndirectY),
            0x95 => (Instruction::Sta, AddressingMode::ZeroPageX),
            0x99 => (Instruction::Sta, AddressingMode::AbsoluteY),
            0x9D => (Instruction::Sta, AddressingMode::AbsoluteX),
            // LDA
            0xA1 => (Instruction::Lda, AddressingMode::IndirectX),
            0xA5 => (Instruction::Lda, AddressingMode::ZeroPage),
            0xA9 => (Instruction::Lda, AddressingMode::Immediate),
            0xAD => (Instruction::Lda, AddressingMode::Absolute),
            0xB1 => (Instruction::Lda, AddressingMode::IndirectY),
            0xB5 => (Instruction::Lda, AddressingMode::ZeroPageX),
            0xB9 => (Instruction::Lda, AddressingMode::AbsoluteY),
            0xBD => (Instruction::Lda, AddressingMode::AbsoluteX),
            // CMP
            0xC1 => (Instruction::Cmp, AddressingMode::IndirectX),
            0xC5 => (Instruction::Cmp, AddressingMode::ZeroPage),
            0xC9 => (Instruction::Cmp, AddressingMode::Immediate),
            0xCD => (Instruction::Cmp, AddressingMode::Absolute),
            0xD1 => (Instruction::Cmp, AddressingMode::IndirectY),
            0xD5 => (Instruction::Cmp, AddressingMode::ZeroPageX),
            0xD9 => (Instruction::Cmp, AddressingMode::AbsoluteY),
            0xDD => (Instruction::Cmp, AddressingMode::AbsoluteX),
            // RMW and Data Movement block
            // ASL
            0x02 => (Instruction::Jam, AddressingMode::Implied),
            0x06 => (Instruction::Asl, AddressingMode::ZeroPage),
            0x0A => (Instruction::Asl, AddressingMode::Accumulator),
            0x0E => (Instruction::Asl, AddressingMode::Absolute),
            0x12 => (Instruction::Jam, AddressingMode::Implied),
            0x16 => (Instruction::Asl, AddressingMode::ZeroPageX),
            0x1A => (Instruction::Nop, AddressingMode::Implied),
            0x1E => (Instruction::Asl, AddressingMode::AbsoluteX),
            // ROL
            0x22 => (Instruction::Jam, AddressingMode::Implied),
            0x26 => (Instruction::Rol, AddressingMode::ZeroPage),
            0x2A => (Instruction::Rol, AddressingMode::Accumulator),
            0x2E => (Instruction::Rol, AddressingMode::Absolute),
            0x32 => (Instruction::Jam, AddressingMode::Implied),
            0x36 => (Instruction::Rol, AddressingMode::ZeroPageX),
            0x3A => (Instruction::Nop, AddressingMode::Implied),
            0x3E => (Instruction::Rol, AddressingMode::AbsoluteX),
            // LSR
            0x42 => (Instruction::Jam, AddressingMode::Implied),
            0x46 => (Instruction::Lsr, AddressingMode::ZeroPage),
            0x4A => (Instruction::Lsr, AddressingMode::Accumulator),
            0x4E => (Instruction::Lsr, AddressingMode::Absolute),
            0x52 => (Instruction::Jam, AddressingMode::Implied),
            0x56 => (Instruction::Lsr, AddressingMode::ZeroPageX),
            0x5A => (Instruction::Nop, AddressingMode::Implied),
            0x5E => (Instruction::Lsr, AddressingMode::AbsoluteX),
            // ROR
            0x62 => (Instruction::Jam, AddressingMode::Implied),
            0x66 => (Instruction::Ror, AddressingMode::ZeroPage),
            0x6A => (Instruction::Ror, AddressingMode::Accumulator),
            0x6E => (Instruction::Ror, AddressingMode::Absolute),
            0x72 => (Instruction::Jam, AddressingMode::Implied),
            0x76 => (Instruction::Ror, AddressingMode::ZeroPageX),
            0x7A => (Instruction::Nop, AddressingMode::Implied),
            0x7E => (Instruction::Ror, AddressingMode::AbsoluteX),
            // STX
            0x82 => (Instruction::Nop, AddressingMode::Immediate),
            0x86 => (Instruction::Stx, AddressingMode::ZeroPage),
            0x8A => (Instruction::Txa, AddressingMode::Accumulator),
            0x8E => (Instruction::Stx, AddressingMode::Absolute),
            0x92 => (Instruction::Jam, AddressingMode::Implied),
            0x96 => (Instruction::Stx, AddressingMode::ZeroPageY),
            0x9A => (Instruction::Txs, AddressingMode::Accumulator),
            0x9E => (Instruction::Shx, AddressingMode::AbsoluteY),
            // LDX
            0xA2 => (Instruction::Ldx, AddressingMode::Immediate),
            0xA6 => (Instruction::Ldx, AddressingMode::ZeroPage),
            0xAA => (Instruction::Tax, AddressingMode::Accumulator),
            0xAE => (Instruction::Ldx, AddressingMode::Absolute),
            0xB2 => (Instruction::Jam, AddressingMode::Implied),
            0xB6 => (Instruction::Ldx, AddressingMode::ZeroPageY),
            0xBA => (Instruction::Tsx, AddressingMode::Accumulator),
            0xBE => (Instruction::Ldx, AddressingMode::AbsoluteY),
            // DEC
            0xC2 => (Instruction::Nop, AddressingMode::Immediate),
            0xC6 => (Instruction::Dec, AddressingMode::ZeroPage),
            0xCA => (Instruction::Dex, AddressingMode::Accumulator),
            0xCE => (Instruction::Dec, AddressingMode::Absolute),
            0xD2 => (Instruction::Jam, AddressingMode::Implied),
            0xD6 => (Instruction::Dec, AddressingMode::ZeroPageX),
            0xDA => (Instruction::Nop, AddressingMode::Implied),
            0xDE => (Instruction::Dec, AddressingMode::AbsoluteX),
            // INC
            0xE2 => (Instruction::Nop, AddressingMode::Immediate),
            0xE6 => (Instruction::Inc, AddressingMode::ZeroPage),
            0xEA => (Instruction::Nop, AddressingMode::Implied),
            0xEE => (Instruction::Inc, AddressingMode::Absolute),
            0xF2 => (Instruction::Jam, AddressingMode::Implied),
            0xF6 => (Instruction::Inc, AddressingMode::ZeroPageX),
            0xFA => (Instruction::Nop, AddressingMode::Implied),
            0xFE => (Instruction::Inc, AddressingMode::AbsoluteX),
            // Unofficial instructions block
            // SLO
            0x03 => (Instruction::Slo, AddressingMode::IndirectX),
            0x07 => (Instruction::Slo, AddressingMode::ZeroPage),
            0x0B => (Instruction::Anc, AddressingMode::Immediate),
            0x0F => (Instruction::Slo, AddressingMode::Absolute),
            0x13 => (Instruction::Slo, AddressingMode::IndirectY),
            0x17 => (Instruction::Slo, AddressingMode::ZeroPageX),
            0x1B => (Instruction::Slo, AddressingMode::AbsoluteY),
            0x1F => (Instruction::Slo, AddressingMode::AbsoluteX),
            // RLA
            0x23 => (Instruction::Rla, AddressingMode::IndirectX),
            0x27 => (Instruction::Rla, AddressingMode::ZeroPage),
            0x2B => (Instruction::Anc, AddressingMode::Immediate),
            0x2F => (Instruction::Rla, AddressingMode::Absolute),
            0x33 => (Instruction::Rla, AddressingMode::IndirectY),
            0x37 => (Instruction::Rla, AddressingMode::ZeroPageX),
            0x3B => (Instruction::Rla, AddressingMode::AbsoluteY),
            0x3F => (Instruction::Rla, AddressingMode::AbsoluteX),
            // SRE
            0x43 => (Instruction::Sre, AddressingMode::IndirectX),
            0x47 => (Instruction::Sre, AddressingMode::ZeroPage),
            0x4B => (Instruction::Alr, AddressingMode::Immediate),
            0x4F => (Instruction::Sre, AddressingMode::Absolute),
            0x53 => (Instruction::Sre, AddressingMode::IndirectY),
            0x57 => (Instruction::Sre, AddressingMode::ZeroPageX),
            0x5B => (Instruction::Sre, AddressingMode::AbsoluteY),
            0x5F => (Instruction::Sre, AddressingMode::AbsoluteX),
            _ => todo!("Unsupported opcode {:#02X}", opcode),
        };
    }

    fn set_nz_flags(&mut self, value: u8) {
        if value == 0 {
            self.p |= STATUS_REGISTER_ZERO;
        } else {
            self.p &= !STATUS_REGISTER_ZERO;
        }
        if value & 0x80 != 0 {
            self.p |= STATUS_REGISTER_NEGATIVE;
        } else {
            self.p &= !STATUS_REGISTER_NEGATIVE;
        }
    }

    fn set_carry(&mut self, overflow: bool) {
        if overflow {
            self.p |= STATUS_REGISTER_CARRY;
        } else {
            self.p &= !STATUS_REGISTER_CARRY;
        }
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
        s: u8,
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
            cpu.s = self.s;
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
                        assert_eq!(Some(&value), memory.get(cpu_read_addr));
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

                        memory.insert(*cpu_write_addr, *cpu_write_value);
                        request = cpu.step(0xFF);
                    }
                    _ => panic!(
                        "Test case '{}': Unexpected cpu operation {:?}, wanted {:?}",
                        case.name, request, cycle_type
                    ),
                }
            }

            let mut expected_cpu = case.final_state.create_cpu();
            expected_cpu.instruction = cpu.instruction;
            expected_cpu.addressing_mode = cpu.addressing_mode;
            expected_cpu.jammed = cpu.jammed;
            if cpu.cycle == 1 {
                // Requesting opcode to match with actual cpu state
                expected_cpu.step(0xFF);
            }

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
        test_ora_ind_x    => "01.json",
        test_ora_zero     => "05.json",
        test_ora_imm      => "09.json",
        test_ora_abs      => "0d.json",
        test_ora_ind_y    => "11.json",
        test_ora_zero_x   => "15.json",
        test_ora_abs_y    => "19.json",
        test_ora_abs_x    => "1d.json",
        test_and_ind_x    => "21.json",
        test_and_zero     => "25.json",
        test_and_imm      => "29.json",
        test_and_abs      => "2d.json",
        test_and_ind_y    => "31.json",
        test_and_zero_x   => "35.json",
        test_and_abs_y    => "39.json",
        test_and_abs_x    => "3d.json",
        test_eor_ind_x    => "41.json",
        test_eor_zero     => "45.json",
        test_eor_imm      => "49.json",
        test_eor_abs      => "4d.json",
        test_eor_ind_y    => "51.json",
        test_eor_zero_x   => "55.json",
        test_eor_abs_y    => "59.json",
        test_eor_abs_x    => "5d.json",
        test_sta_ind_x    => "81.json",
        test_sta_zero     => "85.json",
        test_nop_89       => "89.json",
        test_sta_abs      => "8d.json",
        test_sta_ind_y    => "91.json",
        test_sta_zero_x   => "95.json",
        test_sta_abs_y    => "99.json",
        test_sta_abs_x    => "9d.json",
        test_lda_ind_x    => "a1.json",
        test_lda_zero     => "a5.json",
        test_lda_imm      => "a9.json",
        test_lda_abs      => "ad.json",
        test_lda_ind_y    => "b1.json",
        test_lda_zero_x   => "b5.json",
        test_lda_abs_y    => "b9.json",
        test_lda_abs_x    => "bd.json",
        test_cmp_ind_x    => "c1.json",
        test_cmp_zero     => "c5.json",
        test_cmp_imm      => "c9.json",
        test_cmp_abs      => "cd.json",
        test_cmp_ind_y    => "d1.json",
        test_cmp_zero_x   => "d5.json",
        test_cmp_abs_y    => "d9.json",
        test_cmp_abs_x    => "dd.json",
        test_jam_02       => "02.json",
        test_asl_zero     => "06.json",
        test_asl_acc      => "0a.json",
        test_asl_abs      => "0e.json",
        test_jam_12       => "12.json",
        test_asl_zero_x   => "16.json",
        test_nop_1a       => "1a.json",
        test_asl_abs_x    => "1e.json",
        test_jam_22       => "22.json",
        test_rol_zero     => "26.json",
        test_rol_acc      => "2a.json",
        test_rol_abs      => "2e.json",
        test_jam_32       => "32.json",
        test_rol_zero_x   => "36.json",
        test_nop_3a       => "3a.json",
        test_rol_abs_x    => "3e.json",
        test_jam_42       => "42.json",
        test_lsr_zero     => "46.json",
        test_lsr_acc      => "4a.json",
        test_lsr_abs      => "4e.json",
        test_jam_52       => "52.json",
        test_lsr_zero_x   => "56.json",
        test_nop_5a       => "5a.json",
        test_lsr_abs_x    => "5e.json",
        test_jam_62       => "62.json",
        test_ror_zero     => "66.json",
        test_ror_acc      => "6a.json",
        test_ror_abs      => "6e.json",
        test_jam_72       => "72.json",
        test_ror_zero_x   => "76.json",
        test_nop_7a       => "7a.json",
        test_ror_abs_x    => "7e.json",
        test_nop_82       => "82.json",
        test_stx_zero     => "86.json",
        test_txa          => "8a.json",
        test_stx_abs      => "8e.json",
        test_jam_92       => "92.json",
        test_stx_zero_y   => "96.json",
        test_txs          => "9a.json",
        test_shx          => "9e.json",
        test_ldx_imm      => "a2.json",
        test_ldx_zero     => "a6.json",
        test_tax          => "aa.json",
        test_ldx_abs      => "ae.json",
        test_jam_b2       => "b2.json",
        test_ldx_zero_y   => "b6.json",
        test_tsx          => "ba.json",
        test_ldx_abs_y    => "be.json",
        test_nop_c2       => "c2.json",
        test_dec_zero     => "c6.json",
        test_dex          => "ca.json",
        test_dec_abs      => "ce.json",
        test_jam_d2       => "d2.json",
        test_dec_zero_x   => "d6.json",
        test_nop_da       => "da.json",
        test_dec_abs_x    => "de.json",
        test_nop_e2       => "e2.json",
        test_inc_zero     => "e6.json",
        test_nop_ea       => "ea.json",
        test_inc_abs      => "ee.json",
        test_jam_f2       => "f2.json",
        test_inc_zero_x   => "f6.json",
        test_nop_fa       => "fa.json",
        test_inc_abs_x    => "fe.json",
        test_slo_ind_x    => "03.json",
        test_slo_zero     => "07.json",
        test_anc_imm_0b   => "0b.json",
        test_slo_abs      => "0f.json",
        test_slo_ind_y    => "13.json",
        test_slo_zero_x   => "17.json",
        test_slo_abs_y    => "1b.json",
        test_slo_abs_x    => "1f.json",
        test_rla_ind_x    => "23.json",
        test_rla_zero     => "27.json",
        test_anc_imm_2b   => "2b.json",
        test_rla_abs      => "2f.json",
        test_rla_ind_y    => "33.json",
        test_rla_zero_x   => "37.json",
        test_rla_abs_y    => "3b.json",
        test_rla_abs_x    => "3f.json",
        test_sre_ind_x    => "43.json",
        test_sre_zero     => "47.json",
        test_alr_imm_2b   => "4b.json",
        test_sre_abs      => "4f.json",
        test_sre_ind_y    => "53.json",
        test_sre_zero_x   => "57.json",
        test_sre_abs_y    => "5b.json",
        test_sre_abs_x    => "5f.json",
    }
}
