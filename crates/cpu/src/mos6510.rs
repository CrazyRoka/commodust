use crate::mos6510::{AddressingMode::AbsoluteX, Interrupt::None};

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
    Brk,
    Php,
    Bpl,
    Clc,
    Jsr,
    Bit,
    Plp,
    Bmi,
    Sec,
    Rti,
    Pha,
    Bvc,
    Cli,
    Rts,
    Pla,
    Jmp,
    Bvs,
    Sei,
    Sty,
    Dey,
    Bcc,
    Tya,
    Shy,
    Ldy,
    Tay,
    Bcs,
    Clv,
    Cpy,
    Iny,
    Bne,
    Cld,
    Cpx,
    Inx,
    Beq,
    Sed,
    Ora,
    And,
    Eor,
    Adc,
    Nop,
    Sta,
    Lda,
    Cmp,
    Sbc,
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
    Arr,
    Rra,
    Sax,
    Ane,
    Sha,
    Tas,
    Lax,
    Las,
    Dcp,
    Sbx,
    Isc,
}

impl Instruction {
    fn is_writing(&self) -> bool {
        match self {
            Self::Brk => false,
            Self::Php => false,
            Self::Bpl => false,
            Self::Clc => false,
            Self::Jsr => false,
            Self::Bit => false,
            Self::Plp => false,
            Self::Bmi => false,
            Self::Sec => false,
            Self::Rti => false,
            Self::Pha => false,
            Self::Bvc => false,
            Self::Cli => false,
            Self::Rts => false,
            Self::Pla => false,
            Self::Jmp => true,
            Self::Bvs => false,
            Self::Sei => false,
            Self::Sty => true,
            Self::Dey => false,
            Self::Bcc => false,
            Self::Tya => false,
            Self::Shy => true,
            Self::Ldy => false,
            Self::Tay => false,
            Self::Bcs => false,
            Self::Clv => false,
            Self::Cpy => false,
            Self::Iny => false,
            Self::Bne => false,
            Self::Cld => false,
            Self::Cpx => false,
            Self::Inx => false,
            Self::Beq => false,
            Self::Sed => false,
            Self::Ora => false,
            Self::And => false,
            Self::Eor => false,
            Self::Adc => false,
            Self::Nop => false,
            Self::Sta => true,
            Self::Lda => false,
            Self::Cmp => false,
            Self::Sbc => false,
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
            Self::Arr => false,
            Self::Rra => false,
            Self::Sax => true,
            Self::Ane => false,
            Self::Sha => true,
            Self::Tas => true,
            Self::Lax => false,
            Self::Las => false,
            Self::Dcp => false,
            Self::Sbx => false,
            Self::Isc => false,
        }
    }

    fn is_reading_and_writing(&self) -> bool {
        match self {
            Self::Brk => false,
            Self::Php => false,
            Self::Bpl => false,
            Self::Clc => false,
            Self::Jsr => false,
            Self::Bit => false,
            Self::Plp => false,
            Self::Bmi => false,
            Self::Sec => false,
            Self::Rti => false,
            Self::Pha => false,
            Self::Bvc => false,
            Self::Cli => false,
            Self::Rts => false,
            Self::Pla => false,
            Self::Jmp => false,
            Self::Bvs => false,
            Self::Sei => false,
            Self::Sty => false,
            Self::Dey => false,
            Self::Bcc => false,
            Self::Tya => false,
            Self::Shy => false,
            Self::Ldy => false,
            Self::Tay => false,
            Self::Bcs => false,
            Self::Clv => false,
            Self::Cpy => false,
            Self::Iny => false,
            Self::Bne => false,
            Self::Cld => false,
            Self::Cpx => false,
            Self::Inx => false,
            Self::Beq => false,
            Self::Sed => false,
            Self::Ora => false,
            Self::And => false,
            Self::Eor => false,
            Self::Adc => false,
            Self::Nop => false,
            Self::Sta => false,
            Self::Lda => false,
            Self::Cmp => false,
            Self::Sbc => false,
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
            Self::Arr => true,
            Self::Rra => true,
            Self::Sax => false,
            Self::Ane => false,
            Self::Sha => false,
            Self::Tas => false,
            Self::Lax => false,
            Self::Las => false,
            Self::Dcp => true,
            Self::Sbx => false,
            Self::Isc => true,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Interrupt {
    Irq,
    Nmi,
    None,
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
    pending_irq: bool,
    pending_nmi: bool,
    interrupt: Interrupt,
}

impl Mos6510 {
    pub fn new() -> Self {
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
            pending_irq: false,
            pending_nmi: false,
            interrupt: Interrupt::None,
        }
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
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
            if self.pending_nmi {
                self.pending_nmi = false;
                self.interrupt = Interrupt::Nmi
            } else if self.pending_irq && self.p & STATUS_REGISTER_INTERRUPT_DISABLE == 0 {
                self.pending_irq = false;
                self.interrupt = Interrupt::Irq
            } else {
                self.interrupt = None;
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                self.cycle = 1;
                return BusOperation::Read(addr);
            }
        }

        if self.interrupt != Interrupt::None {
            return self.handle_interrupt(data_in);
        }

        if self.cycle == 1 {
            self.decode_instruction(data_in);
        }

        if let AddressModeProcessingResult::Operation(operation) = self.step_addressing(data_in) {
            return operation;
        }

        self.execute_instruction(data_in)
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
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
                ),
            },
            AddressingMode::Immediate | AddressingMode::Relative => match self.cycle {
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
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
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
                        let no_page_cross = self.addr_latch.wrapping_add(add as u16) & 0xFF00
                            == self.addr_latch & 0xFF00;
                        self.addr_latch = self.addr_latch.wrapping_add(add as u16);
                        if !self.instruction.is_reading_and_writing() && no_page_cross
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
            AddressingMode::Indirect => match self.cycle {
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
                    self.cycle = 5;
                    self.value_latch = data_in;
                    AddressModeProcessingResult::Operation(BusOperation::Read(
                        (self.addr_latch & 0xFF00)
                            | ((self.addr_latch as u8).wrapping_add(1) as u16),
                    ))
                }
                5 => {
                    self.addr_latch = ((data_in as u16) << 8) | self.value_latch as u16;
                    AddressModeProcessingResult::Finished
                }
                _ => unreachable!(
                    "Unexpected cycle for {:?} addressing mode {}",
                    self.addressing_mode, self.cycle
                ),
            },
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
        }
    }

    fn execute_instruction(&mut self, data_in: u8) -> BusOperation {
        if self.cycle < INSTRUCTION_EXECUTION_PHASE {
            self.cycle = INSTRUCTION_EXECUTION_PHASE;
        }

        match self.instruction {
            Instruction::Brk => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    return BusOperation::Read(addr);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    let addr = 0x0100 | self.s as u16;
                    self.s = self.s.wrapping_sub(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Write(addr, (self.pc >> 8) as u8);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    let addr = 0x0100 | self.s as u16;
                    self.s = self.s.wrapping_sub(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    return BusOperation::Write(addr, self.pc as u8);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    let addr = 0x0100 | self.s as u16;
                    self.s = self.s.wrapping_sub(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 4;
                    return BusOperation::Write(
                        addr,
                        self.p | STATUS_REGISTER_BREAK | STATUS_REGISTER_UNUSED,
                    );
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 4 => {
                    self.p |= STATUS_REGISTER_INTERRUPT_DISABLE;
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 5;
                    return BusOperation::Read(0xFFFE);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 5 => {
                    self.addr_latch = data_in as u16;
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 6;
                    return BusOperation::Read(0xFFFF);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 6 => {
                    self.pc = self.addr_latch | ((data_in as u16) << 8);
                    self.prepare_for_next_instruction();
                    return self.step(0xFF);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Php => {
                let addr = 0x100 | self.s as u16;
                self.s = self.s.wrapping_sub(1);
                self.prepare_for_next_instruction();
                return BusOperation::Write(
                    addr,
                    self.p | STATUS_REGISTER_BREAK | STATUS_REGISTER_UNUSED,
                );
            }
            Instruction::Bpl => return self.branch(self.p & STATUS_REGISTER_NEGATIVE == 0),
            Instruction::Clc => self.p &= !STATUS_REGISTER_CARRY,
            Instruction::Jsr => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    let addr = self.pc;
                    self.pc = self.pc.wrapping_add(1);
                    return BusOperation::Read(addr);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    self.addr_latch = data_in as u16;
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read(0x100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    let addr = 0x0100 | self.s as u16;
                    self.s = self.s.wrapping_sub(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    return BusOperation::Write(addr, (self.pc >> 8) as u8);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    let addr = 0x0100 | self.s as u16;
                    self.s = self.s.wrapping_sub(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 4;
                    return BusOperation::Write(addr, self.pc as u8);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 4 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 5;
                    return BusOperation::Read(self.pc);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 5 => {
                    self.addr_latch |= (data_in as u16) << 8;
                    self.pc = self.addr_latch;
                    self.prepare_for_next_instruction();
                    return self.step(0xFF);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Bmi => return self.branch(self.p & STATUS_REGISTER_NEGATIVE != 0),
            Instruction::Sec => {
                self.p |= STATUS_REGISTER_CARRY;
            }
            Instruction::Plp => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Read(self.pc);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    self.p = (data_in | STATUS_REGISTER_UNUSED) & !STATUS_REGISTER_BREAK;
                    self.prepare_for_next_instruction();
                    return BusOperation::Read(self.pc);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Bit => {
                let res = self.a & self.value_latch;
                if res == 0 {
                    self.p |= STATUS_REGISTER_ZERO;
                } else {
                    self.p &= !STATUS_REGISTER_ZERO;
                }
                if self.value_latch & 0x80 != 0 {
                    self.p |= STATUS_REGISTER_NEGATIVE;
                } else {
                    self.p &= !STATUS_REGISTER_NEGATIVE;
                }
                if self.value_latch & 0x40 != 0 {
                    self.p |= STATUS_REGISTER_OVERFLOW;
                } else {
                    self.p &= !STATUS_REGISTER_OVERFLOW;
                }
            }
            Instruction::Rti => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Read(self.pc);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    self.s = self.s.wrapping_add(1);
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    self.p = (data_in | STATUS_REGISTER_UNUSED) & !STATUS_REGISTER_BREAK;
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 4;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 4 => {
                    self.addr_latch = data_in as u16;
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 5;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 5 => {
                    self.addr_latch |= (data_in as u16) << 8;
                    self.pc = self.addr_latch;
                    self.prepare_for_next_instruction();
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Pha => {
                let addr = 0x0100 | self.s as u16;
                self.s = self.s.wrapping_sub(1);
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, self.a);
            }
            Instruction::Bvc => return self.branch(self.p & STATUS_REGISTER_OVERFLOW == 0),
            Instruction::Cli => {
                self.p &= !STATUS_REGISTER_INTERRUPT_DISABLE;
            }
            Instruction::Rts => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Read(self.pc);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    self.addr_latch = data_in as u16;
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 4;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 4 => {
                    self.addr_latch |= (data_in as u16) << 8;
                    self.pc = self.addr_latch.wrapping_add(1);
                    self.prepare_for_next_instruction();
                    return BusOperation::Read(self.pc.wrapping_sub(1));
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Pla => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Read(self.pc);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                    self.s = self.s.wrapping_add(1);
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 3;
                    return BusOperation::Read(0x0100 | self.s as u16);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 3 => {
                    self.a = data_in;
                    self.set_nz_flags(self.a);
                    self.prepare_for_next_instruction();
                    return BusOperation::Read(self.pc);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Jmp => {
                self.pc = self.addr_latch;
            }
            Instruction::Bvs => return self.branch(self.p & STATUS_REGISTER_OVERFLOW != 0),
            Instruction::Sei => {
                self.p |= STATUS_REGISTER_INTERRUPT_DISABLE;
            }
            Instruction::Sty => {
                let addr = self.addr_latch;
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, self.y);
            }
            Instruction::Dey => {
                self.y = self.y.wrapping_sub(1);
                self.set_nz_flags(self.y);
            }
            Instruction::Bcc => return self.branch(self.p & STATUS_REGISTER_CARRY == 0),
            Instruction::Tya => {
                self.a = self.y;
                self.set_nz_flags(self.a);
            }
            Instruction::Shy => {
                let h = (self.addr_latch.wrapping_sub(self.x as u16) >> 8) as u8;
                let val = self.y & h.wrapping_add(1);
                let page_crossed = (self.addr_latch >> 8) as u8 != h;
                let addr = if page_crossed {
                    ((val as u16) << 8) | (self.addr_latch & 0x00FF)
                } else {
                    self.addr_latch
                };
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, val);
            }
            Instruction::Ldy => {
                self.y = self.value_latch;
                self.set_nz_flags(self.y);
            }
            Instruction::Tay => {
                self.y = self.a;
                self.set_nz_flags(self.y);
            }
            Instruction::Bcs => return self.branch(self.p & STATUS_REGISTER_CARRY != 0),
            Instruction::Clv => self.p &= !STATUS_REGISTER_OVERFLOW,
            Instruction::Cpy => {
                let (res, carry) = self.y.overflowing_sub(self.value_latch);
                self.set_nz_flags(res);
                self.set_carry(!carry);
            }
            Instruction::Iny => {
                self.y = self.y.wrapping_add(1);
                self.set_nz_flags(self.y);
            }
            Instruction::Bne => return self.branch(self.p & STATUS_REGISTER_ZERO == 0),
            Instruction::Cld => self.p &= !STATUS_REGISTER_DECIMAL_MODE,
            Instruction::Cpx => {
                let (res, carry) = self.x.overflowing_sub(self.value_latch);
                self.set_nz_flags(res);
                self.set_carry(!carry);
            }
            Instruction::Inx => {
                self.x = self.x.wrapping_add(1);
                self.set_nz_flags(self.x);
            }
            Instruction::Beq => return self.branch(self.p & STATUS_REGISTER_ZERO != 0),
            Instruction::Sed => self.p |= STATUS_REGISTER_DECIMAL_MODE,
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
            Instruction::Adc => {
                self.run_adc(self.value_latch);
            }
            Instruction::Nop => {
                self.prepare_for_next_instruction();
                return BusOperation::Read(self.pc);
            }
            Instruction::Sta => {
                let addr = self.addr_latch;
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, self.a);
            }
            Instruction::Lda => {
                self.a = self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::Cmp => {
                let (res, carry) = self.a.overflowing_sub(self.value_latch);
                self.set_nz_flags(res);
                self.set_carry(!carry);
            }
            Instruction::Sbc => self.run_sbc(self.value_latch),
            Instruction::Jam => {
                self.prepare_for_next_instruction();
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
                };
            }
            Instruction::Stx => {
                let addr = self.addr_latch;
                self.prepare_for_next_instruction();
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
                self.prepare_for_next_instruction();
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        self.prepare_for_next_instruction();
                        return BusOperation::Write(addr, res);
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        let carry = self.value_latch & 0x80 != 0;
                        let res = self.value_latch << 1;
                        let addr = self.addr_latch;
                        self.prepare_for_next_instruction();
                        self.a |= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        let carry = self.value_latch & 0x80 != 0;
                        let res = (self.value_latch << 1) | old_carry;
                        let addr = self.addr_latch;
                        self.prepare_for_next_instruction();
                        self.a &= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
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
                        let carry = self.value_latch & 0x01 != 0;
                        let res = self.value_latch >> 1;
                        let addr = self.addr_latch;
                        self.prepare_for_next_instruction();
                        self.a ^= res;
                        self.set_nz_flags(self.a);
                        self.set_carry(carry);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
                };
            }
            Instruction::Rra => {
                return match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        BusOperation::Write(self.addr_latch, self.value_latch)
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let old_carry = self.p & STATUS_REGISTER_CARRY != 0;
                        let carry = self.value_latch & 0x01 != 0;
                        let res = (self.value_latch >> 1) | if old_carry { 0x80 } else { 0x00 };
                        let addr = self.addr_latch;
                        self.prepare_for_next_instruction();
                        self.set_carry(carry);
                        self.run_adc(res);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
                };
            }
            Instruction::Arr => {
                let and = self.a & self.value_latch;
                let old_carry = (self.p & STATUS_REGISTER_CARRY) != 0;
                let shifted = (and >> 1) | if old_carry { 0x80 } else { 0 };

                if self.p & STATUS_REGISTER_DECIMAL_MODE != 0 {
                    self.set_nz_flags(shifted);

                    if ((and ^ shifted) & 0x40) != 0 {
                        self.p |= STATUS_REGISTER_OVERFLOW;
                    } else {
                        self.p &= !STATUS_REGISTER_OVERFLOW;
                    }

                    let low_adjusted = if (and & 0x0F) + (and & 0x01) > 5 {
                        (shifted & 0xF0) | (shifted.wrapping_add(6) & 0x0F)
                    } else {
                        shifted
                    };

                    if (and & 0xF0) as u16 + (and & 0x10) as u16 > 0x50 {
                        self.a = (low_adjusted & 0x0F) | (low_adjusted.wrapping_add(0x60) & 0xF0);
                        self.set_carry(true);
                    } else {
                        self.a = low_adjusted;
                        self.set_carry(false);
                    }
                } else {
                    self.a = shifted;
                    self.set_nz_flags(self.a);

                    let c = (shifted & 0x40) != 0;
                    self.set_carry(c);

                    if ((shifted >> 6) ^ (shifted >> 5)) & 1 != 0 {
                        self.p |= STATUS_REGISTER_OVERFLOW;
                    } else {
                        self.p &= !STATUS_REGISTER_OVERFLOW;
                    }
                }
            }
            Instruction::Sax => {
                let addr = self.addr_latch;
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, self.a & self.x);
            }
            Instruction::Ane => {
                self.a = (self.a | 0xEE) & self.x & self.value_latch;
                self.set_nz_flags(self.a);
            }
            Instruction::Tas => {
                let h = (self.addr_latch.wrapping_sub(self.y as u16) >> 8) as u8;
                let page_crossed = h != (self.addr_latch >> 8) as u8;
                let res = self.a & self.x & h.wrapping_add(1);
                let addr = if page_crossed {
                    (self.addr_latch & 0x00FF) | (res as u16) << 8
                } else {
                    self.addr_latch
                };
                self.s = self.a & self.x;
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, res);
            }
            Instruction::Sha => {
                let h = (self.addr_latch.wrapping_sub(self.y as u16) >> 8) as u8;
                let page_crossed = h != (self.addr_latch >> 8) as u8;
                let res = self.a & self.x & h.wrapping_add(1);
                let addr = if page_crossed {
                    (self.addr_latch & 0x00FF) | (res as u16) << 8
                } else {
                    self.addr_latch
                };
                self.prepare_for_next_instruction();
                return BusOperation::Write(addr, res);
            }
            Instruction::Lax => {
                let value = if self.addressing_mode == AddressingMode::Immediate {
                    (self.a | 0xEE) & self.value_latch
                } else {
                    self.value_latch
                };
                self.a = value;
                self.x = value;
                self.set_nz_flags(value);
            }
            Instruction::Las => {
                let res = self.s & self.value_latch;
                self.a = res;
                self.x = res;
                self.s = res;
                self.set_nz_flags(res);
            }
            Instruction::Dcp => match self.cycle {
                INSTRUCTION_EXECUTION_PHASE => {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Write(self.addr_latch, self.value_latch);
                }
                val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                    let intermediate = self.value_latch.wrapping_sub(1);
                    let (res, carry) = self.a.overflowing_sub(intermediate);
                    self.set_nz_flags(res);
                    self.set_carry(!carry);
                    let addr = self.addr_latch;
                    self.prepare_for_next_instruction();
                    return BusOperation::Write(addr, intermediate);
                }
                _ => unreachable!(
                    "Unexpected {:?} execution cycle {}",
                    self.instruction, self.cycle
                ),
            },
            Instruction::Sbx => {
                let (res, carry) = (self.a & self.x).overflowing_sub(self.value_latch);
                self.x = res;
                self.set_nz_flags(res);
                self.set_carry(!carry);
            }
            Instruction::Isc => {
                return match self.cycle {
                    INSTRUCTION_EXECUTION_PHASE => {
                        self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                        BusOperation::Write(self.addr_latch, self.value_latch)
                    }
                    val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                        let res = self.value_latch.wrapping_add(1);
                        let addr = self.addr_latch;
                        self.prepare_for_next_instruction();
                        self.run_sbc(res);
                        BusOperation::Write(addr, res)
                    }
                    _ => unreachable!(
                        "Unexpected {:?} execution cycle {}",
                        self.instruction, self.cycle
                    ),
                };
            }
        }

        self.prepare_for_next_instruction();
        self.step(0xFF)
    }

    fn decode_instruction(&mut self, opcode: u8) {
        (self.instruction, self.addressing_mode) = match opcode {
            // Control Instructions block
            // JSR, BIT, PLP, BMI, SEC
            0x00 => (Instruction::Brk, AddressingMode::Implied),
            0x04 => (Instruction::Nop, AddressingMode::ZeroPage),
            0x08 => (Instruction::Php, AddressingMode::Accumulator),
            0x0C => (Instruction::Nop, AddressingMode::Absolute),
            0x10 => (Instruction::Bpl, AddressingMode::Relative),
            0x14 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0x18 => (Instruction::Clc, AddressingMode::Accumulator),
            0x1C => (Instruction::Nop, AddressingMode::AbsoluteX),
            // JSR, BIT, PLP, BMI, SEC
            0x20 => (Instruction::Jsr, AddressingMode::Implied),
            0x24 => (Instruction::Bit, AddressingMode::ZeroPage),
            0x28 => (Instruction::Plp, AddressingMode::Implied),
            0x2C => (Instruction::Bit, AddressingMode::Absolute),
            0x30 => (Instruction::Bmi, AddressingMode::Relative),
            0x34 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0x38 => (Instruction::Sec, AddressingMode::Accumulator),
            0x3C => (Instruction::Nop, AddressingMode::AbsoluteX),
            // RTI, NOP, PHA, JMP, BVC, NOP, CLI, NOP
            0x40 => (Instruction::Rti, AddressingMode::Implied),
            0x44 => (Instruction::Nop, AddressingMode::ZeroPage),
            0x48 => (Instruction::Pha, AddressingMode::Accumulator),
            0x4C => (Instruction::Jmp, AddressingMode::Absolute),
            0x50 => (Instruction::Bvc, AddressingMode::Relative),
            0x54 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0x58 => (Instruction::Cli, AddressingMode::Accumulator),
            0x5C => (Instruction::Nop, AddressingMode::AbsoluteX),
            // RTS, PLA, JMP, BVS, SEI
            0x60 => (Instruction::Rts, AddressingMode::Implied),
            0x64 => (Instruction::Nop, AddressingMode::ZeroPage),
            0x68 => (Instruction::Pla, AddressingMode::Implied),
            0x6C => (Instruction::Jmp, AddressingMode::Indirect),
            0x70 => (Instruction::Bvs, AddressingMode::Relative),
            0x74 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0x78 => (Instruction::Sei, AddressingMode::Accumulator),
            0x7C => (Instruction::Nop, AddressingMode::AbsoluteX),
            // STY
            0x80 => (Instruction::Nop, AddressingMode::Immediate),
            0x84 => (Instruction::Sty, AddressingMode::ZeroPage),
            0x88 => (Instruction::Dey, AddressingMode::Accumulator),
            0x8C => (Instruction::Sty, AddressingMode::Absolute),
            0x90 => (Instruction::Bcc, AddressingMode::Relative),
            0x94 => (Instruction::Sty, AddressingMode::ZeroPageX),
            0x98 => (Instruction::Tya, AddressingMode::Accumulator),
            0x9C => (Instruction::Shy, AddressingMode::AbsoluteX),
            // LDY
            0xA0 => (Instruction::Ldy, AddressingMode::Immediate),
            0xA4 => (Instruction::Ldy, AddressingMode::ZeroPage),
            0xA8 => (Instruction::Tay, AddressingMode::Accumulator),
            0xAC => (Instruction::Ldy, AddressingMode::Absolute),
            0xB0 => (Instruction::Bcs, AddressingMode::Relative),
            0xB4 => (Instruction::Ldy, AddressingMode::ZeroPageX),
            0xB8 => (Instruction::Clv, AddressingMode::Accumulator),
            0xBC => (Instruction::Ldy, AddressingMode::AbsoluteX),
            // CPY
            0xC0 => (Instruction::Cpy, AddressingMode::Immediate),
            0xC4 => (Instruction::Cpy, AddressingMode::ZeroPage),
            0xC8 => (Instruction::Iny, AddressingMode::Accumulator),
            0xCC => (Instruction::Cpy, AddressingMode::Absolute),
            0xD0 => (Instruction::Bne, AddressingMode::Relative),
            0xD4 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0xD8 => (Instruction::Cld, AddressingMode::Accumulator),
            0xDC => (Instruction::Nop, AddressingMode::AbsoluteX),
            // CPX
            0xE0 => (Instruction::Cpx, AddressingMode::Immediate),
            0xE4 => (Instruction::Cpx, AddressingMode::ZeroPage),
            0xE8 => (Instruction::Inx, AddressingMode::Accumulator),
            0xEC => (Instruction::Cpx, AddressingMode::Absolute),
            0xF0 => (Instruction::Beq, AddressingMode::Relative),
            0xF4 => (Instruction::Nop, AddressingMode::ZeroPageX),
            0xF8 => (Instruction::Sed, AddressingMode::Accumulator),
            0xFC => (Instruction::Nop, AddressingMode::AbsoluteX),
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
            // ADC
            0x61 => (Instruction::Adc, AddressingMode::IndirectX),
            0x65 => (Instruction::Adc, AddressingMode::ZeroPage),
            0x69 => (Instruction::Adc, AddressingMode::Immediate),
            0x6D => (Instruction::Adc, AddressingMode::Absolute),
            0x71 => (Instruction::Adc, AddressingMode::IndirectY),
            0x75 => (Instruction::Adc, AddressingMode::ZeroPageX),
            0x79 => (Instruction::Adc, AddressingMode::AbsoluteY),
            0x7D => (Instruction::Adc, AddressingMode::AbsoluteX),
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
            // SBC
            0xE1 => (Instruction::Sbc, AddressingMode::IndirectX),
            0xE5 => (Instruction::Sbc, AddressingMode::ZeroPage),
            0xE9 => (Instruction::Sbc, AddressingMode::Immediate),
            0xED => (Instruction::Sbc, AddressingMode::Absolute),
            0xF1 => (Instruction::Sbc, AddressingMode::IndirectY),
            0xF5 => (Instruction::Sbc, AddressingMode::ZeroPageX),
            0xF9 => (Instruction::Sbc, AddressingMode::AbsoluteY),
            0xFD => (Instruction::Sbc, AddressingMode::AbsoluteX),
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
            // Rra
            0x63 => (Instruction::Rra, AddressingMode::IndirectX),
            0x67 => (Instruction::Rra, AddressingMode::ZeroPage),
            0x6B => (Instruction::Arr, AddressingMode::Immediate),
            0x6F => (Instruction::Rra, AddressingMode::Absolute),
            0x73 => (Instruction::Rra, AddressingMode::IndirectY),
            0x77 => (Instruction::Rra, AddressingMode::ZeroPageX),
            0x7B => (Instruction::Rra, AddressingMode::AbsoluteY),
            0x7F => (Instruction::Rra, AddressingMode::AbsoluteX),
            // SAX
            0x83 => (Instruction::Sax, AddressingMode::IndirectX),
            0x87 => (Instruction::Sax, AddressingMode::ZeroPage),
            0x8B => (Instruction::Ane, AddressingMode::Immediate),
            0x8F => (Instruction::Sax, AddressingMode::Absolute),
            0x93 => (Instruction::Sha, AddressingMode::IndirectY),
            0x97 => (Instruction::Sax, AddressingMode::ZeroPageY),
            0x9B => (Instruction::Tas, AddressingMode::AbsoluteY),
            0x9F => (Instruction::Sha, AddressingMode::AbsoluteY),
            // LAX
            0xA3 => (Instruction::Lax, AddressingMode::IndirectX),
            0xA7 => (Instruction::Lax, AddressingMode::ZeroPage),
            0xAB => (Instruction::Lax, AddressingMode::Immediate),
            0xAF => (Instruction::Lax, AddressingMode::Absolute),
            0xB3 => (Instruction::Lax, AddressingMode::IndirectY),
            0xB7 => (Instruction::Lax, AddressingMode::ZeroPageY),
            0xBB => (Instruction::Las, AddressingMode::AbsoluteY),
            0xBF => (Instruction::Lax, AddressingMode::AbsoluteY),
            // DCP
            0xC3 => (Instruction::Dcp, AddressingMode::IndirectX),
            0xC7 => (Instruction::Dcp, AddressingMode::ZeroPage),
            0xCB => (Instruction::Sbx, AddressingMode::Immediate),
            0xCF => (Instruction::Dcp, AddressingMode::Absolute),
            0xD3 => (Instruction::Dcp, AddressingMode::IndirectY),
            0xD7 => (Instruction::Dcp, AddressingMode::ZeroPageX),
            0xDB => (Instruction::Dcp, AddressingMode::AbsoluteY),
            0xDF => (Instruction::Dcp, AddressingMode::AbsoluteX),
            // ISC
            0xE3 => (Instruction::Isc, AddressingMode::IndirectX),
            0xE7 => (Instruction::Isc, AddressingMode::ZeroPage),
            0xEB => (Instruction::Sbc, AddressingMode::Immediate),
            0xEF => (Instruction::Isc, AddressingMode::Absolute),
            0xF3 => (Instruction::Isc, AddressingMode::IndirectY),
            0xF7 => (Instruction::Isc, AddressingMode::ZeroPageX),
            0xFB => (Instruction::Isc, AddressingMode::AbsoluteY),
            0xFF => (Instruction::Isc, AddressingMode::AbsoluteX),
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

    fn set_carry(&mut self, carry: bool) {
        if carry {
            self.p |= STATUS_REGISTER_CARRY;
        } else {
            self.p &= !STATUS_REGISTER_CARRY;
        }
    }

    fn set_overflow(&mut self, a: u8, b: u8, res: u8) {
        let overflow = ((a ^ res) & (b ^ res) & 0x80) != 0;
        if overflow {
            self.p |= STATUS_REGISTER_OVERFLOW;
        } else {
            self.p &= !STATUS_REGISTER_OVERFLOW;
        }
    }

    fn prepare_for_next_instruction(&mut self) {
        self.cycle = 0;
        self.value_latch = 0;
        self.addr_latch = 0;
    }

    fn run_adc(&mut self, value: u8) {
        let carry = if self.p & STATUS_REGISTER_CARRY != 0 {
            1
        } else {
            0
        };
        let binary_res = (self.a as u16) + (value as u16) + carry as u16;
        self.set_nz_flags((binary_res & 0xFF) as u8);

        if self.p & STATUS_REGISTER_DECIMAL_MODE == 0 {
            self.set_overflow(self.a, value, (binary_res & 0xFF) as u8);
            self.a = (binary_res & 0xFF) as u8;
            self.set_carry(binary_res > 0xFF);
        } else {
            let mut low = (self.a & 0x0F) + (value & 0x0F) + carry;
            let mut high =
                (self.a & 0xF0) as u16 + (value & 0xF0) as u16 + (if low > 9 { 0x10 } else { 0 });

            self.set_overflow(self.a, value, high as u8);

            if high & 0x80 != 0 {
                self.p |= STATUS_REGISTER_NEGATIVE;
            } else {
                self.p &= !STATUS_REGISTER_NEGATIVE;
            }

            if low > 9 {
                low = (low + 6) & 0x0F;
            } else {
                low = low & 0x0F;
            }

            let decimal_carry = high > 0x90;
            if decimal_carry {
                high = (high + 0x60) & 0xF0;
            } else {
                high = high & 0xF0;
            }

            self.a = (high as u8) | low;
            self.set_carry(decimal_carry);
        }
    }

    fn run_sbc(&mut self, value: u8) {
        let carry = if self.p & STATUS_REGISTER_CARRY != 0 {
            1
        } else {
            0
        };
        let bin_res = (self.a as u16) + (!value as u16) + carry;

        let res_u8 = (bin_res & 0xFF) as u8;
        self.set_nz_flags(res_u8);

        if self.p & STATUS_REGISTER_DECIMAL_MODE == 0 {
            self.set_overflow(self.a, !value, res_u8);
            self.a = res_u8;
            self.set_carry(bin_res > 0xFF);
        } else {
            self.set_overflow(self.a, !value, res_u8);
            self.set_carry(bin_res > 0xFF);

            let mut low = (self.a & 0x0F) as i16 - (value & 0x0F) as i16 - (1 - carry as i16);

            let mut high = (self.a >> 4) as i16 - (value >> 4) as i16 - if low < 0 { 1 } else { 0 };

            if low < 0 {
                low -= 6;
            }
            if high < 0 {
                high -= 6;
            }

            self.a = (((high as u8) & 0x0F) << 4) | ((low as u8) & 0x0F);
        }
    }

    fn branch(&mut self, condition: bool) -> BusOperation {
        match self.cycle {
            INSTRUCTION_EXECUTION_PHASE => {
                if condition {
                    self.addr_latch = if self.value_latch >= 128 {
                        self.pc
                            .wrapping_sub(1)
                            .wrapping_sub(0xFF - self.value_latch as u16)
                    } else {
                        self.pc.wrapping_add(self.value_latch as u16)
                    };
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 1;
                    return BusOperation::Read(self.pc);
                }
            }
            val if val == INSTRUCTION_EXECUTION_PHASE + 1 => {
                if self.pc & 0xFF00 != self.addr_latch & 0xFF00 {
                    self.cycle = INSTRUCTION_EXECUTION_PHASE + 2;
                    return BusOperation::Read((self.pc & 0xFF00) | (self.addr_latch & 0x00FF));
                }
                self.pc = self.addr_latch;
            }
            val if val == INSTRUCTION_EXECUTION_PHASE + 2 => {
                self.pc = self.addr_latch;
            }
            _ => unreachable!(
                "Unexpected {:?} execution cycle {}",
                self.instruction, self.cycle
            ),
        }

        self.prepare_for_next_instruction();
        self.step(0xFF)
    }

    pub fn set_irq(&mut self, value: bool) {
        self.pending_irq = value;
    }

    pub fn nmi(&mut self) {
        self.pending_nmi = true;
    }

    fn handle_interrupt(&mut self, data_in: u8) -> BusOperation {
        let vector = match self.interrupt {
            Interrupt::Irq => 0xFFFE,
            Interrupt::Nmi => 0xFFFA,
            None => unreachable!("NONE interrupt is impossible to reach"),
        };
        return match self.cycle {
            0 => {
                self.cycle = 1;
                BusOperation::Read(self.pc)
            }
            1 => {
                self.cycle = 2;
                BusOperation::Read(self.pc.wrapping_add(1))
            }
            2 => {
                self.cycle = 3;
                self.s = self.s.wrapping_sub(1);
                BusOperation::Write(
                    0x100 | (self.s.wrapping_add(1) as u16),
                    (self.pc >> 8) as u8,
                )
            }
            3 => {
                self.cycle = 4;
                self.s = self.s.wrapping_sub(1);
                BusOperation::Write(
                    0x100 | (self.s.wrapping_add(1) as u16),
                    (self.pc & 0xFF) as u8,
                )
            }
            4 => {
                self.cycle = 5;
                self.s = self.s.wrapping_sub(1);
                BusOperation::Write(
                    0x100 | (self.s.wrapping_add(1) as u16),
                    self.p | STATUS_REGISTER_UNUSED,
                )
            }
            5 => {
                self.cycle = 6;
                BusOperation::Read(vector)
            }
            6 => {
                self.cycle = 7;
                self.addr_latch = data_in as u16;
                BusOperation::Read(vector + 1)
            }
            7 => {
                self.addr_latch |= (data_in as u16) << 8;
                self.pc = self.addr_latch;
                self.p |= STATUS_REGISTER_INTERRUPT_DISABLE;
                self.prepare_for_next_instruction();
                self.step(0xFF)
            }
            _ => unreachable!(
                "Unexpected cycle {} during {:?} interrupt handling",
                self.cycle, self.interrupt
            ),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Mos6510,
        mos6510::{BusOperation, STATUS_REGISTER_DECIMAL_MODE, STATUS_REGISTER_INTERRUPT_DISABLE},
    };
    use serde::Deserialize;
    use std::{collections::HashMap, fs, path::Path};

    fn assert_read_op(op: BusOperation, expected_addr: u16) {
        match op {
            BusOperation::Read(addr) => assert_eq!(addr, expected_addr, "Read address mismatch"),
            other => panic!("Expected Read(0x{:04X}), got {:?}", expected_addr, other),
        }
    }

    fn assert_write_op(op: BusOperation, expected_addr: u16, expected_val: u8) {
        match op {
            BusOperation::Write(addr, val) => {
                assert_eq!(addr, expected_addr, "Write address mismatch");
                assert_eq!(val, expected_val, "Write value mismatch");
            }
            other => panic!(
                "Expected Write(0x{:04X}, 0x{:02X}), got {:?}",
                expected_addr, expected_val, other
            ),
        }
    }

    #[test]
    fn test_irq_basic_sequence() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = 0x00; // I flag clear so IRQ is taken

        cpu.set_irq(true);

        // Cycle 1: Dummy read from PC (PC not incremented)
        assert_read_op(cpu.step(0xFF), 0x1000);
        // Cycle 2: Dummy read from PC + 1
        assert_read_op(cpu.step(0xEA), 0x1001);
        // Cycle 3: Push PC high (S=0xFF -> 0x01FF)
        assert_write_op(cpu.step(0xFF), 0x01FF, 0x10);
        // Cycle 4: Push PC low (S=0xFE -> 0x01FE)
        assert_write_op(cpu.step(0xFF), 0x01FE, 0x00);
        // Cycle 5: Push status — B=0, unused=1 (S=0xFD -> 0x01FD)
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x20);
        // Cycle 6: Read IRQ vector low from 0xFFFE
        assert_read_op(cpu.step(0xFF), 0xFFFE);
        // Cycle 7: Read IRQ vector high from 0xFFFF
        assert_read_op(cpu.step(0x34), 0xFFFF);
        // After interrupt: fetch opcode from vector address 0x1234
        assert_read_op(cpu.step(0x12), 0x1234);

        assert_eq!(cpu.pc, 0x1235); // PC incremented after opcode fetch
        assert_eq!(cpu.s, 0xFC); // Stack pointer decremented by 3
        assert_eq!(
            cpu.p & STATUS_REGISTER_INTERRUPT_DISABLE,
            STATUS_REGISTER_INTERRUPT_DISABLE
        );
    }

    #[test]
    fn test_irq_not_taken_when_interrupts_disabled() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = STATUS_REGISTER_INTERRUPT_DISABLE; // I flag set

        cpu.set_irq(true);

        // CPU fetches opcode normally — IRQ not serviced
        assert_read_op(cpu.step(0xFF), 0x1000);
        // Stack unchanged — no interrupt push happened
        assert_eq!(cpu.s, 0xFF);
    }

    #[test]
    fn test_irq_pushes_b_flag_clear() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        cpu.set_irq(true);

        cpu.step(0xFF); // Read PC
        cpu.step(0xFF); // Read PC+1
        cpu.step(0xFF); // Push PC high
        cpu.step(0xFF); // Push PC low

        // Pushed status must have B=0, unused=1 — unlike BRK which pushes B=1
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x20);
    }

    #[test]
    fn test_irq_preserves_decimal_flag_in_pushed_status() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = STATUS_REGISTER_DECIMAL_MODE; // D flag set, I flag clear

        cpu.set_irq(true);

        cpu.step(0xFF); // Read PC
        cpu.step(0xFF); // Read PC+1
        cpu.step(0xFF); // Push PC high
        cpu.step(0xFF); // Push PC low

        // D=1, B=0, unused=1 => 0x28 — CPU does NOT clear D flag
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x28);
    }

    #[test]
    fn test_nmi_basic_sequence() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x2000;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        cpu.nmi();

        // Cycle 1: Dummy read from PC
        assert_read_op(cpu.step(0xFF), 0x2000);
        // Cycle 2: Dummy read from PC + 1
        assert_read_op(cpu.step(0xFF), 0x2001);
        // Cycle 3: Push PC high
        assert_write_op(cpu.step(0xFF), 0x01FF, 0x20);
        // Cycle 4: Push PC low
        assert_write_op(cpu.step(0xFF), 0x01FE, 0x00);
        // Cycle 5: Push status — B=0, unused=1
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x20);
        // Cycle 6: Read NMI vector low from 0xFFFA
        assert_read_op(cpu.step(0xFF), 0xFFFA);
        // Cycle 7: Read NMI vector high from 0xFFFB
        assert_read_op(cpu.step(0x78), 0xFFFB);
        // After interrupt: fetch opcode from vector address 0x5678
        assert_read_op(cpu.step(0x56), 0x5678);

        assert_eq!(cpu.pc, 0x5679);
        assert_eq!(cpu.s, 0xFC);
        assert_eq!(
            cpu.p & STATUS_REGISTER_INTERRUPT_DISABLE,
            STATUS_REGISTER_INTERRUPT_DISABLE
        );
    }

    #[test]
    fn test_nmi_taken_when_interrupts_disabled() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x2000;
        cpu.s = 0xFF;
        cpu.p = STATUS_REGISTER_INTERRUPT_DISABLE; // I flag set

        cpu.nmi();

        // NMI is taken regardless of I flag
        assert_read_op(cpu.step(0xFF), 0x2000);
        assert_read_op(cpu.step(0xFF), 0x2001);
        assert_write_op(cpu.step(0xFF), 0x01FF, 0x20);
        assert_write_op(cpu.step(0xFF), 0x01FE, 0x00);
        // Pushed status: B=0, unused=1, I=1 => 0x24
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x24);
        assert_read_op(cpu.step(0xFF), 0xFFFA);
        assert_read_op(cpu.step(0x78), 0xFFFB);
        assert_read_op(cpu.step(0x56), 0x5678);
    }

    #[test]
    fn test_nmi_pushes_b_flag_clear() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x2000;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        cpu.nmi();

        cpu.step(0xFF); // Read PC
        cpu.step(0xFF); // Read PC+1
        cpu.step(0xFF); // Push PC high
        cpu.step(0xFF); // Push PC low

        // Pushed status must have B=0, unused=1
        assert_write_op(cpu.step(0xFF), 0x01FD, 0x20);
    }

    #[test]
    fn test_irq_pushes_correct_return_address() {
        // The PC pushed to the stack is the address of the next instruction
        // (where the opcode would have been fetched), NOT PC+1 like BRK.
        let mut cpu = Mos6510::new();
        cpu.pc = 0xABCD;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        cpu.set_irq(true);

        cpu.step(0xFF); // Read PC (0xABCD)
        cpu.step(0xFF); // Read PC+1 (0xABCE)

        // Push PC high — should be 0xAB (from 0xABCD, not 0xABCE)
        assert_write_op(cpu.step(0xFF), 0x01FF, 0xAB);
        // Push PC low — should be 0xCD
        assert_write_op(cpu.step(0xFF), 0x01FE, 0xCD);
    }

    #[test]
    fn test_nmi_pushes_correct_return_address() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0xBEEF;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        cpu.nmi();

        cpu.step(0xFF); // Read PC (0xBEEF)
        cpu.step(0xFF); // Read PC+1 (0xBEF0)

        // Push PC high — should be 0xBE (from 0xBEEF)
        assert_write_op(cpu.step(0xFF), 0x01FF, 0xBE);
        // Push PC low — should be 0xEF
        assert_write_op(cpu.step(0xFF), 0x01FE, 0xEF);
    }

    #[test]
    fn test_nmi_mid_instruction_deferred() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = 0x00;

        // Cycle 0: Read opcode
        assert_read_op(cpu.step(0xFF), 0x1000);

        // Trigger NMI mid-instruction (Cycle 1, before the final cycle of NOP)
        cpu.nmi();

        // Cycle 1 of NOP (0xEA): Dummy read
        assert_read_op(cpu.step(0xEA), 0x1001);

        // Since the instruction has completed, the next step begins the NMI sequence.
        // Cycle 1 of NMI: Dummy read from PC (0x1001)
        assert_read_op(cpu.step(0xFF), 0x1001);

        // Cycle 2 of NMI: Dummy read from PC + 1 (0x1002)
        assert_read_op(cpu.step(0xFF), 0x1002);

        // Cycle 3 of NMI: Push PC high (0x10) to 0x01FF
        assert_write_op(cpu.step(0xFF), 0x01FF, 0x10);

        // Cycle 4 of NMI: Push PC low (0x01) to 0x01FE
        assert_write_op(cpu.step(0xFF), 0x01FE, 0x01);
    }

    #[test]
    fn test_irq_mid_instruction_deferred() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = 0x00; // Interrupts enabled

        // Cycle 0: Read opcode
        assert_read_op(cpu.step(0xFF), 0x1000);

        // Trigger NMI mid-instruction (Cycle 1, before the final cycle of NOP)
        cpu.set_irq(true);

        // Cycle 1 of NOP (0xEA): Dummy read
        assert_read_op(cpu.step(0xEA), 0x1001);

        // Now that the instruction is complete, the IRQ sequence starts.
        // Cycle 1 of IRQ: Dummy read from PC (0x1001)
        assert_read_op(cpu.step(0xFF), 0x1001);

        // Cycle 2 of IRQ: Dummy read from PC + 1 (0x1002)
        assert_read_op(cpu.step(0xFF), 0x1002);

        // Cycle 3 of IRQ: Push PC high (0x10) to 0x01FF
        assert_write_op(cpu.step(0xFF), 0x01FF, 0x10);

        // Cycle 4 of IRQ: Push PC low (0x01) to 0x01FE
        assert_write_op(cpu.step(0xFF), 0x01FE, 0x01);
    }

    #[test]
    fn test_irq_mid_instruction_flip() {
        let mut cpu = Mos6510::new();
        cpu.pc = 0x1000;
        cpu.s = 0xFF;
        cpu.p = 0x00; // Interrupts enabled

        // Cycle 0: Read opcode
        assert_read_op(cpu.step(0xFF), 0x1000);

        // Trigger NMI mid-instruction (Cycle 1, before the final cycle of NOP)
        cpu.set_irq(true);

        // Cycle 1 of NOP (0xEA): Dummy read
        assert_read_op(cpu.step(0xEA), 0x1001);

        cpu.set_irq(false);
        assert_eq!(cpu.pending_irq, false);

        // Now that the instruction is complete, the IRQ is not handled anymore.
        // Cycle 0: Read opcode
        assert_read_op(cpu.step(0xFF), 0x1001);
        assert_eq!(cpu.pending_irq, false);

        // Cycle 1 of NOP (0xEA): Dummy read
        assert_read_op(cpu.step(0xEA), 0x1002);
        assert_eq!(cpu.pending_irq, false);
    }

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
            let mut cpu = Mos6510::new();

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
        test_brk          => "00.json",
        test_nop_04       => "04.json",
        test_php          => "08.json",
        test_nop_0c       => "0c.json",
        test_bpl          => "10.json",
        test_nop_14       => "14.json",
        test_clc          => "18.json",
        test_nop_1c       => "1c.json",
        test_jsr          => "20.json",
        test_bit_zero     => "24.json",
        test_plp          => "28.json",
        test_bit_abs      => "2c.json",
        test_bmi          => "30.json",
        test_nop_34       => "34.json",
        test_sec          => "38.json",
        test_nop_3c       => "3c.json",
        test_rti          => "40.json",
        test_nop_44       => "44.json",
        test_pha          => "48.json",
        test_jmp_abs      => "4c.json",
        test_bvc          => "50.json",
        test_nop_54       => "54.json",
        test_cli          => "58.json",
        test_nop_5c       => "5c.json",
        test_rts          => "60.json",
        test_nop_64       => "64.json",
        test_pla          => "68.json",
        test_jmp_ind      => "6c.json",
        test_bvs          => "70.json",
        test_nop_74       => "74.json",
        test_sei          => "78.json",
        test_nop_7c       => "7c.json",
        test_nop_80       => "80.json",
        test_sty_zero     => "84.json",
        test_dey          => "88.json",
        test_sty_abs      => "8c.json",
        test_bcc          => "90.json",
        test_sty_zero_x   => "94.json",
        test_tya          => "98.json",
        test_shy          => "9c.json",
        test_ldy_ind_x    => "a0.json",
        test_ldy_zero     => "a4.json",
        test_tay          => "a8.json",
        test_ldy_abs      => "ac.json",
        test_bcs          => "b0.json",
        test_ldy_zero_x   => "b4.json",
        test_clv          => "b8.json",
        test_ldy_abs_x    => "bc.json",
        test_cpy_ind_x    => "c0.json",
        test_cpy_zero     => "c4.json",
        test_iny          => "c8.json",
        test_cpy_abs      => "cc.json",
        test_bne          => "d0.json",
        test_nop_d4       => "d4.json",
        test_cld          => "d8.json",
        test_nop_dc       => "dc.json",
        test_cpx_ind_x    => "e0.json",
        test_cpx_zero     => "e4.json",
        test_inx          => "e8.json",
        test_cpx_abs      => "ec.json",
        test_beq          => "f0.json",
        test_nop_f4       => "f4.json",
        test_sed          => "f8.json",
        test_nop_fc       => "fc.json",
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
        test_adc_ind_x    => "61.json",
        test_adc_zero     => "65.json",
        test_adc_imm      => "69.json",
        test_adc_abs      => "6d.json",
        test_adc_ind_y    => "71.json",
        test_adc_zero_x   => "75.json",
        test_adc_abs_y    => "79.json",
        test_adc_abs_x    => "7d.json",
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
        test_sbc_ind_x    => "e1.json",
        test_sbc_zero     => "e5.json",
        test_sbc_imm      => "e9.json",
        test_sbc_abs      => "ed.json",
        test_sbc_ind_y    => "f1.json",
        test_sbc_zero_x   => "f5.json",
        test_sbc_abs_y    => "f9.json",
        test_sbc_abs_x    => "fd.json",
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
        test_rra_ind_x    => "63.json",
        test_rra_zero     => "67.json",
        test_arr_imm_2b   => "6b.json",
        test_rra_abs      => "6f.json",
        test_rra_ind_y    => "73.json",
        test_rra_zero_x   => "77.json",
        test_rra_abs_y    => "7b.json",
        test_rra_abs_x    => "7f.json",
        test_sax_ind_x    => "83.json",
        test_sax_zero     => "87.json",
        test_ane          => "8b.json",
        test_sax_abs      => "8f.json",
        test_sha_ind_y    => "93.json",
        test_sax_zero_y   => "97.json",
        test_tas          => "9b.json",
        test_sha_abs_y    => "9f.json",
        test_lax_ind_x    => "a3.json",
        test_lax_zero     => "a7.json",
        test_lax_imm      => "ab.json",
        test_lax_abs      => "af.json",
        test_lax_ind_y    => "b3.json",
        test_lax_zero_y   => "b7.json",
        test_las          => "bb.json",
        test_lax_abs_y    => "bf.json",
        test_dcp_ind_x    => "c3.json",
        test_dcp_zero     => "c7.json",
        test_sbx_imm_cb   => "cb.json",
        test_dcp_abs      => "cf.json",
        test_dcp_ind_y    => "d3.json",
        test_dcp_zero_x   => "d7.json",
        test_dcp_abs_y    => "db.json",
        test_dcp_abs_x    => "df.json",
        test_isc_ind_x    => "e3.json",
        test_isc_zero     => "e7.json",
        test_sbc_imm_eb   => "eb.json",
        test_isc_abs      => "ef.json",
        test_isc_ind_y    => "f3.json",
        test_isc_zero_x   => "f7.json",
        test_isc_abs_y    => "fb.json",
        test_isc_abs_x    => "ff.json",
    }
}
