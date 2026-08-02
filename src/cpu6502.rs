mod test;

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    st: StatusFlags,
    pc: u16,
    sp: u8,
    cycle: Cycle,
    mem: [u8; 65536],
    irq: bool,      // true if the IRQB pin is set to low
    irq_prev: bool, // previous state of the IRQB pin to detect negative transition
    nmi: bool,      // true if the NMIB pin is set to low
    nmi_prev: bool, // previous state of the NMIB pin to detect negative transition
    reset: bool,
    tmp: [u8; 2],
    tmp_addr: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StatusFlag {
    None,
    Carry,
    Zero,
    IRQDisable,
    Decimal,
    BRK,
    Overflow,
    Negative,
}

impl From<u8> for StatusFlag {
    fn from(value: u8) -> Self {
        let value: usize = value.into();
        [
            Self::None,
            Self::Carry,
            Self::Zero,
            Self::IRQDisable,
            Self::Decimal,
            Self::BRK,
            Self::Overflow,
            Self::Negative,
        ][value]
    }
}

impl Into<u8> for StatusFlag {
    fn into(self) -> u8 {
        match self {
            StatusFlag::None => 0,
            StatusFlag::Carry => 1,
            StatusFlag::Zero => 2,
            StatusFlag::IRQDisable => 4,
            StatusFlag::Decimal => 8,
            StatusFlag::BRK => 16,
            StatusFlag::Overflow => 64,
            StatusFlag::Negative => 128,
        }
    }
}

struct StatusFlags(u8);

impl StatusFlags {
    fn is_set(&self, flag: StatusFlag) -> bool {
        let flag: u8 = flag.into();
        self.0 & flag != 0
    }
    fn is_clear(&self, flag: StatusFlag) -> bool {
        let flag: u8 = flag.into();
        self.0 & flag == 0
    }
}

struct Cycle(u8);

impl Cycle {
    fn plus(&self, n: u8) -> Cycle {
        Cycle(self.0 + n)
    }
}

trait IsOriginal {
    fn is_original(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
enum AddrMode {
    Absolute,
    AbsoluteIndexedIndirect,
    AbsoluteIndexedWithX,
    AbsoluteIndexedWithY,
    AbsoluteIndirect,
    Accumulator,
    Immediate,
    Implied,
    ProgramCounterRelative,
    Stack,
    ZeroPage,
    ZeroPageIndexedIndirect,
    ZeroPageIndexedWithX,
    ZeroPageIndexedWithY,
    ZeroPageIndirect,
    ZeroPageIndirectIndexedWithY,
}

impl IsOriginal for AddrMode {
    fn is_original(&self) -> bool {
        match self {
            AddrMode::AbsoluteIndexedIndirect => false,
            AddrMode::ZeroPageIndirect => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Instruction {
    ADC,
    AND,
    ASL,
    BBR0,
    BBR1,
    BBR2,
    BBR3,
    BBR4,
    BBR5,
    BBR6,
    BBR7,
    BBS0,
    BBS1,
    BBS2,
    BBS3,
    BBS4,
    BBS5,
    BBS6,
    BBS7,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRA,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PHX,
    PHY,
    PLA,
    PLP,
    PLX,
    PLY,
    RMB0,
    RMB1,
    RMB2,
    RMB3,
    RMB4,
    RMB5,
    RMB6,
    RMB7,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    SMB0,
    SMB1,
    SMB2,
    SMB3,
    SMB4,
    SMB5,
    SMB6,
    SMB7,
    STA,
    STP,
    STX,
    STY,
    STZ,
    TAX,
    TAY,
    TRB,
    TSB,
    TSX,
    TXA,
    TXS,
    TYA,
    WAI,
    ILL,
}

trait HasDescription {
    fn desc(&self) -> &str;
}

impl HasDescription for Instruction {
    fn desc(&self) -> &str {
        match self {
            Instruction::ADC => "ADd memory to accumulator with Carry",
            Instruction::AND => "\"AND\" memory with accumulator",
            Instruction::ASL => "Arithmetic Shift one bit Left, memory or accumulator",
            Instruction::BBR0 => "Branch on Bit 0 Reset",
            Instruction::BBR1 => "Branch on Bit 1 Reset",
            Instruction::BBR2 => "Branch on Bit 2 Reset",
            Instruction::BBR3 => "Branch on Bit 3 Reset",
            Instruction::BBR4 => "Branch on Bit 4 Reset",
            Instruction::BBR5 => "Branch on Bit 5 Reset",
            Instruction::BBR6 => "Branch on Bit 6 Reset",
            Instruction::BBR7 => "Branch on Bit 7 Reset",
            Instruction::BBS0 => "Branch of Bit 0 Set",
            Instruction::BBS1 => "Branch of Bit 1 Set",
            Instruction::BBS2 => "Branch of Bit 2 Set",
            Instruction::BBS3 => "Branch of Bit 3 Set",
            Instruction::BBS4 => "Branch of Bit 4 Set",
            Instruction::BBS5 => "Branch of Bit 5 Set",
            Instruction::BBS6 => "Branch of Bit 6 Set",
            Instruction::BBS7 => "Branch of Bit 7 Set",
            Instruction::BCC => "Branch on Carry Clear (Pc=0)",
            Instruction::BCS => "Branch on Carry Set (Pc=1)",
            Instruction::BEQ => "Branch if EQual (Pz=1)",
            Instruction::BIT => "BIt Test",
            Instruction::BMI => "Branch if result MInus (Pn=1)",
            Instruction::BNE => "Branch if Not Equal (Pz=0)",
            Instruction::BPL => "Branch if result PLus (Pn=0)",
            Instruction::BRA => "BRanch Always",
            Instruction::BRK => "BReaK instruction",
            Instruction::BVC => "Branch on oVerflow Clear (Pv=0)",
            Instruction::BVS => "Branch on oVerflow Set (Pv=1)",
            Instruction::CLC => "CLear Cary flag",
            Instruction::CLD => "CLear Decimal mode",
            Instruction::CLI => "CLear Interrupt disable bit",
            Instruction::CLV => "CLear oVerflow flag",
            Instruction::CMP => "CoMPare memory and accumulator",
            Instruction::CPX => "ComPare memory and X register",
            Instruction::CPY => "ComPare memory and Y register",
            Instruction::DEC => "DECrement memory or accumulate by one",
            Instruction::DEX => "DEcrement X by one",
            Instruction::DEY => "DEcrement Y by one",
            Instruction::EOR => "\"Exclusive OR\" memory with accumulate",
            Instruction::INC => "INCrement memory or accumulate by one",
            Instruction::INX => "INcrement X register by one",
            Instruction::INY => "INcrement Y register by one",
            Instruction::JMP => "JuMP to new location",
            Instruction::JSR => "Jump to new location Saving Return (Jump to SubRoutine)",
            Instruction::LDA => "LoaD Accumulator with memory",
            Instruction::LDX => "LoaD the X register with memory",
            Instruction::LDY => "LoaD the Y register with memory",
            Instruction::LSR => "Logical Shift one bit Right memory or",
            Instruction::NOP => "No OPeration",
            Instruction::ORA => "\"OR\" memory with Accumulator",
            Instruction::PHA => "PusH Accumulator on stack",
            Instruction::PHP => "PusH Processor status on stack",
            Instruction::PHX => "PusH X register on stack",
            Instruction::PHY => "PusH Y register on stack",
            Instruction::PLA => "PuLl Accumulator from stack",
            Instruction::PLP => "PuLl Processor status from stack",
            Instruction::PLX => "PuLl X register from stack",
            Instruction::PLY => "PuLl Y register from stack",
            Instruction::RMB0 => "Reset Memory Bit 0",
            Instruction::RMB1 => "Reset Memory Bit 1",
            Instruction::RMB2 => "Reset Memory Bit 2",
            Instruction::RMB3 => "Reset Memory Bit 3",
            Instruction::RMB4 => "Reset Memory Bit 4",
            Instruction::RMB5 => "Reset Memory Bit 5",
            Instruction::RMB6 => "Reset Memory Bit 6",
            Instruction::RMB7 => "Reset Memory Bit 7",
            Instruction::ROL => "ROtate one bit Left memory or accumulator",
            Instruction::ROR => "ROtate one bit Right memory or accumulator",
            Instruction::RTI => "ReTurn from Interrupt",
            Instruction::RTS => "ReTurn from Subroutine",
            Instruction::SBC => "SuBtract memory from accumulator with borrow (Carry bit)",
            Instruction::SEC => "SEt Carry",
            Instruction::SED => "SEt Decimal mode",
            Instruction::SEI => "SEt Interrupt disable status",
            Instruction::SMB0 => "Set Memory Bit 0",
            Instruction::SMB1 => "Set Memory Bit 1",
            Instruction::SMB2 => "Set Memory Bit 2",
            Instruction::SMB3 => "Set Memory Bit 3",
            Instruction::SMB4 => "Set Memory Bit 4",
            Instruction::SMB5 => "Set Memory Bit 5",
            Instruction::SMB6 => "Set Memory Bit 6",
            Instruction::SMB7 => "Set Memory Bit 7",
            Instruction::STA => "STore Accumulator in memory",
            Instruction::STP => "SToP mode",
            Instruction::STX => "STore the X register in memory",
            Instruction::STY => "STore the Y register in memory",
            Instruction::STZ => "STore Zero in memory",
            Instruction::TAX => "Transfer the Accumulator to the X register",
            Instruction::TAY => "Transfer the Accumulator to the Y register",
            Instruction::TRB => "Test and Reset memory Bit",
            Instruction::TSB => "Test and Set memory Bit",
            Instruction::TSX => "Transfer the Stack pointer to the X register",
            Instruction::TXA => "Transfer the X register to the Accumulator",
            Instruction::TXS => "Transfer the X register to the Stack pointer register",
            Instruction::TYA => "Transfer Y register to the Accumulator",
            Instruction::WAI => "WAit for Interrupt",
            Instruction::ILL => "ILLegal instruction",
        }
    }
}

fn instruction_and_mode(opcode: u8) -> (Instruction, AddrMode) {
    let opcode: usize = opcode.into();
    [
        (Instruction::BRK, AddrMode::Stack),
        (Instruction::ORA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::TSB, AddrMode::ZeroPage),
        (Instruction::ORA, AddrMode::ZeroPage),
        (Instruction::ASL, AddrMode::ZeroPage),
        (Instruction::RMB0, AddrMode::ZeroPage),
        (Instruction::PHP, AddrMode::Stack),
        (Instruction::ORA, AddrMode::Immediate),
        (Instruction::ASL, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::TSB, AddrMode::Absolute),
        (Instruction::ORA, AddrMode::Absolute),
        (Instruction::ASL, AddrMode::Absolute),
        (Instruction::BBR0, AddrMode::ProgramCounterRelative),
        (Instruction::BPL, AddrMode::ProgramCounterRelative),
        (Instruction::ORA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::ORA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::TRB, AddrMode::ZeroPage),
        (Instruction::ORA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ASL, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB1, AddrMode::ZeroPage),
        (Instruction::CLC, AddrMode::Implied),
        (Instruction::ORA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::INC, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::TRB, AddrMode::Absolute),
        (Instruction::ORA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ASL, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR1, AddrMode::ProgramCounterRelative),
        (Instruction::JSR, AddrMode::Absolute),
        (Instruction::AND, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::BIT, AddrMode::ZeroPage),
        (Instruction::AND, AddrMode::ZeroPage),
        (Instruction::ROL, AddrMode::ZeroPage),
        (Instruction::RMB2, AddrMode::ZeroPage),
        (Instruction::PLP, AddrMode::Stack),
        (Instruction::AND, AddrMode::Immediate),
        (Instruction::ROL, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::BIT, AddrMode::Absolute),
        (Instruction::AND, AddrMode::Absolute),
        (Instruction::ROL, AddrMode::Absolute),
        (Instruction::BBR2, AddrMode::ProgramCounterRelative),
        (Instruction::BMI, AddrMode::ProgramCounterRelative),
        (Instruction::AND, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::AND, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::BIT, AddrMode::ZeroPageIndexedWithX),
        (Instruction::AND, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ROL, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB3, AddrMode::ZeroPage),
        (Instruction::SEC, AddrMode::Implied),
        (Instruction::AND, AddrMode::AbsoluteIndexedWithY),
        (Instruction::DEC, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::BIT, AddrMode::AbsoluteIndexedWithX),
        (Instruction::AND, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ROL, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR3, AddrMode::ProgramCounterRelative),
        (Instruction::RTI, AddrMode::Stack),
        (Instruction::EOR, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::EOR, AddrMode::ZeroPage),
        (Instruction::LSR, AddrMode::ZeroPage),
        (Instruction::RMB4, AddrMode::ZeroPage),
        (Instruction::PHA, AddrMode::Stack),
        (Instruction::EOR, AddrMode::Immediate),
        (Instruction::LSR, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::JMP, AddrMode::Absolute),
        (Instruction::EOR, AddrMode::Absolute),
        (Instruction::LSR, AddrMode::Absolute),
        (Instruction::BBR4, AddrMode::ProgramCounterRelative),
        (Instruction::BVC, AddrMode::ProgramCounterRelative),
        (Instruction::EOR, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::EOR, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::EOR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LSR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB5, AddrMode::ZeroPage),
        (Instruction::CLI, AddrMode::Implied),
        (Instruction::EOR, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PHY, AddrMode::Stack),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::EOR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LSR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR5, AddrMode::ProgramCounterRelative),
        (Instruction::RTS, AddrMode::Stack),
        (Instruction::ADC, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STZ, AddrMode::ZeroPage),
        (Instruction::ADC, AddrMode::ZeroPage),
        (Instruction::ROR, AddrMode::ZeroPage),
        (Instruction::RMB6, AddrMode::ZeroPage),
        (Instruction::PLA, AddrMode::Stack),
        (Instruction::ADC, AddrMode::Immediate),
        (Instruction::ROR, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::JMP, AddrMode::AbsoluteIndirect),
        (Instruction::ADC, AddrMode::Absolute),
        (Instruction::ROR, AddrMode::Absolute),
        (Instruction::BBR6, AddrMode::ProgramCounterRelative),
        (Instruction::BVS, AddrMode::ProgramCounterRelative),
        (Instruction::ADC, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::ADC, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STZ, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ADC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ROR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB7, AddrMode::ZeroPage),
        (Instruction::SEI, AddrMode::Implied),
        (Instruction::ADC, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PLY, AddrMode::Stack),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::JMP, AddrMode::AbsoluteIndexedIndirect),
        (Instruction::ADC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ROR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR7, AddrMode::ProgramCounterRelative),
        (Instruction::BRA, AddrMode::ProgramCounterRelative),
        (Instruction::STA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STY, AddrMode::ZeroPage),
        (Instruction::STA, AddrMode::ZeroPage),
        (Instruction::STX, AddrMode::ZeroPage),
        (Instruction::SMB0, AddrMode::ZeroPage),
        (Instruction::DEY, AddrMode::Implied),
        (Instruction::BIT, AddrMode::Immediate),
        (Instruction::TXA, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STY, AddrMode::Absolute),
        (Instruction::STA, AddrMode::Absolute),
        (Instruction::STX, AddrMode::Absolute),
        (Instruction::BBS0, AddrMode::ProgramCounterRelative),
        (Instruction::BCC, AddrMode::ProgramCounterRelative),
        (Instruction::STA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::STA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STY, AddrMode::ZeroPageIndexedWithX),
        (Instruction::STA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::STX, AddrMode::ZeroPageIndexedWithY),
        (Instruction::SMB1, AddrMode::ZeroPage),
        (Instruction::TYA, AddrMode::Implied),
        (Instruction::STA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::TXS, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::STZ, AddrMode::Absolute),
        (Instruction::STA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::STZ, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS1, AddrMode::ProgramCounterRelative),
        (Instruction::LDY, AddrMode::Immediate),
        (Instruction::LDA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::LDX, AddrMode::Immediate),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::LDY, AddrMode::ZeroPage),
        (Instruction::LDA, AddrMode::ZeroPage),
        (Instruction::LDX, AddrMode::ZeroPage),
        (Instruction::SMB2, AddrMode::ZeroPage),
        (Instruction::TAY, AddrMode::Implied),
        (Instruction::LDA, AddrMode::Immediate),
        (Instruction::TAX, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::LDY, AddrMode::Accumulator),
        (Instruction::LDA, AddrMode::Absolute),
        (Instruction::LDX, AddrMode::Absolute),
        (Instruction::BBS2, AddrMode::ProgramCounterRelative),
        (Instruction::BCS, AddrMode::ProgramCounterRelative),
        (Instruction::LDA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::LDA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::LDY, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LDA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LDX, AddrMode::ZeroPageIndexedWithY),
        (Instruction::SMB3, AddrMode::ZeroPage),
        (Instruction::CLV, AddrMode::Implied),
        (Instruction::LDA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::TSX, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::LDY, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LDA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LDX, AddrMode::AbsoluteIndexedWithY),
        (Instruction::BBS3, AddrMode::ProgramCounterRelative),
        (Instruction::CPY, AddrMode::Immediate),
        (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::CPY, AddrMode::ZeroPage),
        (Instruction::CMP, AddrMode::ZeroPage),
        (Instruction::DEC, AddrMode::ZeroPage),
        (Instruction::SMB4, AddrMode::ZeroPage),
        (Instruction::INY, AddrMode::Implied),
        (Instruction::CMP, AddrMode::Immediate),
        (Instruction::DEX, AddrMode::Implied),
        (Instruction::WAI, AddrMode::Implied),
        (Instruction::CPY, AddrMode::Absolute),
        (Instruction::CMP, AddrMode::Absolute),
        (Instruction::DEC, AddrMode::Absolute),
        (Instruction::BBS4, AddrMode::ProgramCounterRelative),
        (Instruction::BNE, AddrMode::ProgramCounterRelative),
        (Instruction::CMP, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::CMP, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::CMP, AddrMode::ZeroPageIndexedWithX),
        (Instruction::DEC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::SMB5, AddrMode::ZeroPage),
        (Instruction::CLD, AddrMode::Implied),
        (Instruction::CMP, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PHX, AddrMode::Stack),
        (Instruction::STP, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::CMP, AddrMode::AbsoluteIndexedWithX),
        (Instruction::DEC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS5, AddrMode::ProgramCounterRelative),
        (Instruction::CPX, AddrMode::Immediate),
        (Instruction::SBC, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::CPX, AddrMode::ZeroPage),
        (Instruction::SBC, AddrMode::ZeroPage),
        (Instruction::INC, AddrMode::ZeroPage),
        (Instruction::SMB6, AddrMode::ZeroPage),
        (Instruction::INX, AddrMode::Implied),
        (Instruction::SBC, AddrMode::Immediate),
        (Instruction::NOP, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::CPX, AddrMode::Absolute),
        (Instruction::SBC, AddrMode::Absolute),
        (Instruction::INC, AddrMode::Absolute),
        (Instruction::BBS6, AddrMode::ProgramCounterRelative),
        (Instruction::BEQ, AddrMode::ProgramCounterRelative),
        (Instruction::SBC, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::SBC, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::SBC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::INC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::SMB7, AddrMode::ZeroPage),
        (Instruction::SED, AddrMode::Implied),
        (Instruction::SBC, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PLX, AddrMode::Stack),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied),
        (Instruction::SBC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::INC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS7, AddrMode::ProgramCounterRelative),
    ][opcode]
}

impl CPU {
    pub fn new() -> Self {
        let mem: [u8; 65536] = [0; 65536];
        CPU {
            a: 0,
            x: 0,
            y: 0,
            st: StatusFlags(0),
            pc: 0,
            sp: 0xff,
            cycle: Cycle(0),
            mem,
            irq: false,
            irq_prev: false,
            nmi: false,
            nmi_prev: false,
            reset: false,
            tmp: [0, 0],
            tmp_addr: 0,
        }
    }
    fn _load_memory_byte_lo(&mut self, addr: u16) {
        // TODO: insert code for peripherals
        let addr: usize = addr.into();
        self.tmp[0] = self.mem[addr];
    }
    fn load_memory_byte_lo(&mut self, addr: u16) -> Cycle {
        self._load_memory_byte_lo(addr);
        self.cycle.plus(1)
    }
    fn _load_memory_byte_hi(&mut self, addr: u16) {
        // TODO: insert code for peripherals
        let addr: usize = addr.into();
        self.tmp[1] = self.mem[addr];
    }
    fn load_memory_byte_hi(&mut self, addr: u16) -> Cycle {
        self._load_memory_byte_hi(addr);
        self.cycle.plus(1)
    }
    fn _store_memory_byte(&mut self, addr: u16, byte: u8) {
        let addr: usize = addr.into();
        self.mem[addr] = byte;
    }
    fn store_memory_byte(&mut self, addr: u16, byte: u8) -> Cycle {
        self._store_memory_byte(addr, byte);
        self.cycle.plus(1)
    }
    pub fn reset(&mut self) {
        self._load_memory_byte_lo(0xfffc);
        self._load_memory_byte_hi(0xfffd);
        self.pc = u16::from_le_bytes(self.tmp);
        self.st = StatusFlags(0);
        self.irq = false;
        self.nmi = false;
        self.cycle = Cycle(0);
        self.reset = true;
    }
    fn _stack_push_byte(&mut self, byte: u8) {
        let stack_base: u16 = 0x100;
        let addr: u16 = self.sp.into();
        let addr = stack_base + addr;
        self._store_memory_byte(addr, byte);
        if self.sp == 0 {
            self.sp = 0xff;
        } else {
            self.sp = self.sp - 1;
        }
    }
    fn stack_push_byte(&mut self, byte: u8) -> Cycle {
        self._stack_push_byte(byte);
        self.cycle.plus(1)
    }
    fn _stack_pull_byte(&mut self, hi: bool) {
        if self.sp == 0xff {
            self.sp = 0x00;
        } else {
            self.sp = self.sp + 1;
        }
        let stack_base: u16 = 0x100;
        let addr: u16 = self.sp.into();
        let addr = stack_base + addr;
        if hi {
            self._load_memory_byte_hi(addr);
        } else {
            self._load_memory_byte_lo(addr);
        }
    }
    fn stack_pull_byte_lo(&mut self) -> Cycle {
        self._stack_pull_byte(false);
        self.cycle.plus(1)
    }
    fn stack_pull_byte_hi(&mut self) -> Cycle {
        self._stack_pull_byte(true);
        self.cycle.plus(1)
    }
    fn addr_add(addr: u16, val: u8) -> u16 {
        let ret: u32 = addr.into();
        let val: u32 = val.into();
        let mut ret = ret + val;
        if ret > 0xffff {
            ret = ret - 0x10000;
        }
        let ret: u16 = u16::try_from(ret).unwrap();
        ret
    }
    fn set_flag(&mut self, flag: StatusFlag) {
        let flag: u8 = flag.into();
        self.st.0 = self.st.0 | flag;
    }
    fn set_flags(&mut self, flags: &[StatusFlag]) {
        for flag in flags {
            self.set_flag(*flag);
        }
    }
    fn clear_flag(&mut self, flag: StatusFlag) {
        let flag: u8 = flag.into();
        self.st.0 = self.st.0 & !flag;
    }
    fn clear_flags(&mut self, flags: &[StatusFlag]) {
        for flag in flags {
            self.clear_flag(*flag);
        }
    }
    fn set_pc(&mut self) -> Cycle {
        let pc: u16 = u16::from_le_bytes(self.tmp);
        self.pc = pc;
        Cycle(0)
    }
    fn inc_pc(&mut self, arg: u8) -> Cycle {
        let arg_signed: i8 = arg.cast_signed();
        if arg_signed < 0 {
            let arg: i16 = arg_signed.into();
            let arg: i16 = arg.abs();
            let arg: u16 = arg.cast_unsigned();
            self.pc = self.pc - arg;
        } else {
            let arg: u16 = arg.into();
            self.pc = self.pc + arg;
        }
        Cycle(0)
    }
    fn stack_push_pc_lo(&mut self, increment: u8) -> Cycle {
        let pc = Self::addr_add(self.pc, increment);
        let lo: u8 = (pc & 0xff).try_into().unwrap();
        self.stack_push_byte(lo)
    }
    fn stack_push_pc_hi(&mut self, increment: u8) -> Cycle {
        let pc = Self::addr_add(self.pc, increment);
        let hi: u8 = (pc >> 8).try_into().unwrap();
        self.stack_push_byte(hi)
    }
    fn stack_push_flags(&mut self) -> Cycle {
        self.stack_push_byte(self.st.0)
    }
    fn stack_pull_flags(&mut self) -> Cycle {
        self.stack_pull_byte_lo();
        self.st.0 = self.tmp[0];
        self.cycle.plus(1)
    }
    fn change_flags(&mut self, enable: &[StatusFlag], disable: &[StatusFlag]) -> Cycle {
        self.set_flags(enable);
        self.clear_flags(disable);
        self.cycle.plus(1)
    }
    fn load_byte_arg_lo(&mut self) -> Cycle {
        let addr: u16 = Self::addr_add(self.pc, 1);
        self.tmp[1] = 0;
        self.load_memory_byte_lo(addr)
    }
    fn load_byte_arg_hi(&mut self) -> Cycle {
        let addr: u16 = Self::addr_add(self.pc, 2);
        self.load_memory_byte_hi(addr)
    }
    fn load_indexed_x_lo(&mut self) -> Cycle {
        let addr: u16 = u16::from_le_bytes(self.tmp);
        self.tmp_addr = Self::addr_add(addr, self.x) & 0xff;
        self.load_memory_byte_lo(self.tmp_addr)
    }
    fn load_indexed_x_hi(&mut self) -> Cycle {
        let ret = self.load_memory_byte_hi(self.tmp_addr);
        self.tmp_addr = u16::from_le_bytes(self.tmp);
        ret
    }
    fn compare_and_set_flags(&mut self, byte: u8) {
        match self.a.cmp(&byte) {
            std::cmp::Ordering::Less => self.change_flags(
                &[StatusFlag::Negative],
                &[StatusFlag::Carry, StatusFlag::Zero],
            ),
            std::cmp::Ordering::Equal => self.change_flags(
                &[StatusFlag::Zero, StatusFlag::Carry],
                &[StatusFlag::Negative],
            ),
            std::cmp::Ordering::Greater => self.change_flags(
                &[StatusFlag::Carry],
                &[StatusFlag::Zero, StatusFlag::Negative],
            ),
        };
    }
    pub fn step(&mut self) {
        if self.reset {
            match self.cycle.0 {
                7 => {
                    self.reset = false;
                    self.cycle.0 = 0;
                }
                _ => {
                    self.cycle = self.cycle.plus(1);
                    println!("(Reset)");
                    return;
                }
            }
        }
        let opcode = self.mem[usize::from(self.pc)];
        println!(
            "{}: 0b{:08b} 0x{:04x} 0x{:02x}",
            self.cycle.0, self.st.0, self.pc, opcode
        );
        let (inst, mode) = instruction_and_mode(opcode);
        self.cycle = match (inst, mode, self.cycle.0) {
            // all instructions require at least two cycles
            (_, _, 0) => Cycle(1),
            (_, _, 1) => Cycle(2),

            (Instruction::BRK, AddrMode::Stack, 2) if self.st.is_set(StatusFlag::BRK) => Cycle(3),
            (Instruction::BRK, AddrMode::Stack, 3) if self.st.is_set(StatusFlag::BRK) => {
                self.inc_pc(2)
            }
            (Instruction::BRK, AddrMode::Stack, 2) => self.stack_push_pc_lo(2),
            (Instruction::BRK, AddrMode::Stack, 3) => self.stack_push_pc_hi(2),
            (Instruction::BRK, AddrMode::Stack, 4) => {
                self.stack_push_flags();
                self.change_flags(
                    &[StatusFlag::BRK, StatusFlag::IRQDisable],
                    &[StatusFlag::Decimal],
                )
            }
            (Instruction::BRK, AddrMode::Stack, 5) => self.load_memory_byte_lo(0xfffe),
            (Instruction::BRK, AddrMode::Stack, 6) => {
                self.load_memory_byte_hi(0xffff);
                self.set_pc()
            }

            (Instruction::ORA, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::TSB, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ORA, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ASL, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::RMB0, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::PHP, AddrMode::Stack, _) => Cycle(0),

            (Instruction::ORA, AddrMode::Immediate, 2) => {
                self.load_byte_arg_lo();
                self.a = self.a | self.tmp[0];
                if self.a == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.a >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(2)
            }

            (Instruction::ASL, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::TSB, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::ORA, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::ASL, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBR0, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BPL, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::ORA, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::ORA, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::TRB, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ORA, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::ASL, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::RMB1, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::CLC, AddrMode::Implied, 2) => {
                self.clear_flag(StatusFlag::Carry);
                self.inc_pc(1)
            }

            (Instruction::ORA, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::INC, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::TRB, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::ORA, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::ASL, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR1, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::JSR, AddrMode::Absolute, 2) => self.stack_push_pc_hi(3),
            (Instruction::JSR, AddrMode::Absolute, 3) => self.stack_push_pc_lo(3),
            (Instruction::JSR, AddrMode::Absolute, 4) => {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::JSR, AddrMode::Absolute, 5) => {
                self.load_memory_byte_hi(Self::addr_add(self.pc, 2))
            }
            (Instruction::JSR, AddrMode::Absolute, 6) => self.set_pc(),

            (Instruction::AND, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::BIT, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::AND, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ROL, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::RMB2, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::PLP, AddrMode::Stack, _) => Cycle(0),

            (Instruction::AND, AddrMode::Immediate, 2) => {
                self.load_byte_arg_lo();
                self.a = self.a & self.tmp[0];
                if self.a == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.a >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(2)
            }

            (Instruction::ROL, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::BIT, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::AND, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::ROL, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBR2, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BMI, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::AND, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::AND, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::BIT, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::AND, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::ROL, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::RMB3, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::SEC, AddrMode::Implied, 2) => {
                self.set_flag(StatusFlag::Carry);
                self.inc_pc(1)
            }

            (Instruction::AND, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::DEC, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::BIT, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::AND, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::ROL, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR3, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::RTI, AddrMode::Stack, 2) => self.stack_pull_flags(),
            (Instruction::RTI, AddrMode::Stack, 3) => self.stack_pull_byte_lo(),
            (Instruction::RTI, AddrMode::Stack, 4) => self.stack_pull_byte_lo(),
            (Instruction::RTI, AddrMode::Stack, 5) => self.stack_pull_byte_hi(),
            (Instruction::RTI, AddrMode::Stack, 6) => self.set_pc(),

            (Instruction::EOR, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::EOR, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::LSR, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::RMB4, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::PHA, AddrMode::Stack, 2) => self.stack_push_byte(self.a),
            (Instruction::PHA, AddrMode::Stack, 3) => self.inc_pc(1),

            (Instruction::EOR, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::LSR, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::JMP, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::EOR, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::LSR, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBR4, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BVC, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::EOR, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::EOR, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::EOR, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::LSR, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::RMB5, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::CLI, AddrMode::Implied, 2) => {
                self.clear_flag(StatusFlag::IRQDisable);
                self.inc_pc(1)
            }

            (Instruction::EOR, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::PHY, AddrMode::Stack, _) => Cycle(0),
            (Instruction::EOR, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::LSR, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR5, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::RTS, AddrMode::Stack, 2) => self.stack_pull_byte_lo(),
            (Instruction::RTS, AddrMode::Stack, 3) => self.stack_pull_byte_hi(),
            (Instruction::RTS, AddrMode::Stack, 4) => self.set_pc(),

            (Instruction::ADC, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::STZ, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ADC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::ROR, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::RMB6, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::PLA, AddrMode::Stack, 2) => self.stack_pull_byte_lo(),
            (Instruction::PLA, AddrMode::Stack, 3) => {
                self.a = self.tmp[0];
                if self.a == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.a >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(2)
            }

            (Instruction::ADC, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::ROR, AddrMode::Accumulator, _) => Cycle(0),
            (Instruction::JMP, AddrMode::AbsoluteIndirect, _) => Cycle(0),
            (Instruction::ADC, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::ROR, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBR6, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BVS, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::ADC, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::ADC, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::STZ, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::ADC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::ROR, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::RMB7, AddrMode::ZeroPage, _) => Cycle(0),

            (Instruction::SEI, AddrMode::Implied, 2) => {
                self.set_flag(StatusFlag::IRQDisable);
                self.inc_pc(1)
            }

            (Instruction::ADC, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::PLY, AddrMode::Stack, 2) => self.stack_pull_byte_lo(),
            (Instruction::PLY, AddrMode::Stack, 3) => {
                self.y = self.tmp[0];
                if self.y == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.y >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(2)
            }

            (Instruction::JMP, AddrMode::AbsoluteIndexedIndirect, _) => Cycle(0),
            (Instruction::ADC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::ROR, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBR7, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BRA, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::STY, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::STX, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB0, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::DEY, AddrMode::Implied, _) => Cycle(0),
            (Instruction::BIT, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::TXA, AddrMode::Implied, _) => Cycle(0),

            (Instruction::STY, AddrMode::Absolute, 2) => self.load_byte_arg_lo(),
            (Instruction::STY, AddrMode::Absolute, 3) => self.load_byte_arg_hi(),
            (Instruction::STY, AddrMode::Absolute, 4) => {
                let addr = u16::from_le_bytes(self.tmp);
                self.store_memory_byte(addr, self.y)
            }

            (Instruction::STA, AddrMode::Absolute, 2) => self.load_byte_arg_lo(),
            (Instruction::STA, AddrMode::Absolute, 3) => self.load_byte_arg_hi(),
            (Instruction::STA, AddrMode::Absolute, 4) => {
                let addr = u16::from_le_bytes(self.tmp);
                self.store_memory_byte(addr, self.a)
            }

            (Instruction::STX, AddrMode::Absolute, 2) => self.load_byte_arg_lo(),
            (Instruction::STX, AddrMode::Absolute, 3) => self.load_byte_arg_hi(),
            (Instruction::STX, AddrMode::Absolute, 4) => {
                let addr = u16::from_le_bytes(self.tmp);
                self.store_memory_byte(addr, self.x)
            }

            (Instruction::BBS0, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BCC, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::STY, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::STA, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::STX, AddrMode::ZeroPageIndexedWithY, _) => Cycle(0),
            (Instruction::SMB1, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::TYA, AddrMode::Implied, _) => Cycle(0),
            (Instruction::STA, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::TXS, AddrMode::Implied, _) => Cycle(0),
            (Instruction::STZ, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::STA, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::STZ, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBS1, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::LDY, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::LDA, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::LDX, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::LDY, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::LDA, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::LDX, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB2, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::TAY, AddrMode::Implied, _) => Cycle(0),
            (Instruction::LDA, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::TAX, AddrMode::Implied, _) => Cycle(0),
            (Instruction::LDY, AddrMode::Accumulator, _) => Cycle(0),

            (Instruction::LDA, AddrMode::Absolute, 2) => self.load_memory_byte_lo(Self::addr_add(self.pc, 1)),
            (Instruction::LDA, AddrMode::Absolute, 3) => self.load_memory_byte_hi(Self::addr_add(self.pc, 2)),
            (Instruction::LDA, AddrMode::Absolute, 4) => {
                self.tmp_addr = u16::from_le_bytes(self.tmp);
                self.load_memory_byte_lo(self.tmp_addr);
                self.a = self.tmp[0];
                if self.a == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.a >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(3)
            }

            (Instruction::LDX, AddrMode::Absolute, 2) => self.load_memory_byte_lo(Self::addr_add(self.pc, 1)),
            (Instruction::LDX, AddrMode::Absolute, 3) => self.load_memory_byte_hi(Self::addr_add(self.pc, 2)),
            (Instruction::LDX, AddrMode::Absolute, 4) => {
                self.tmp_addr = u16::from_le_bytes(self.tmp);
                self.load_memory_byte_lo(self.tmp_addr);
                self.x = self.tmp[0];
                if self.x == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.x >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(3)
            }

            (Instruction::BBS2, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BCS, AddrMode::ProgramCounterRelative, 2)
                if self.st.is_set(StatusFlag::Carry) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BCS, AddrMode::ProgramCounterRelative, 3)
                if self.st.is_set(StatusFlag::Carry) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BCS, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

            (Instruction::LDA, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::LDA, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::LDY, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::LDA, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::LDX, AddrMode::ZeroPageIndexedWithY, _) => Cycle(0),
            (Instruction::SMB3, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::CLV, AddrMode::Implied, _) => Cycle(0),
            (Instruction::LDA, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::TSX, AddrMode::Implied, _) => Cycle(0),
            (Instruction::LDY, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::LDA, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::LDX, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),
            (Instruction::BBS3, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::CPY, AddrMode::Immediate, _) => Cycle(0),

            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 2) => self.load_byte_arg_lo(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 3) => self.load_indexed_x_lo(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 4) => self.load_indexed_x_hi(),
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 5) => {
                self.load_memory_byte_lo(self.tmp_addr)
            }
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect, 6) => {
                self.compare_and_set_flags(self.tmp[0]);
                self.inc_pc(2)
            }

            (Instruction::CPY, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::DEC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB4, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::INY, AddrMode::Implied, _) => Cycle(0),

            (Instruction::CMP, AddrMode::Immediate, 2) => self.load_byte_arg_lo(),
            (Instruction::CMP, AddrMode::Immediate, 3) => {
                self.compare_and_set_flags(self.tmp[0]);
                self.inc_pc(2)
            }

            (Instruction::DEX, AddrMode::Implied, _) => Cycle(0),
            (Instruction::WAI, AddrMode::Implied, _) => Cycle(0),
            (Instruction::CPY, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::CMP, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::DEC, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBS4, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::BNE, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::CMP, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::DEC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::SMB5, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::CLD, AddrMode::Implied, _) => Cycle(0),
            (Instruction::CMP, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::PHX, AddrMode::Stack, 2) => self.stack_push_byte(self.x),
            (Instruction::PHX, AddrMode::Stack, 3) => self.inc_pc(1),

            (Instruction::STP, AddrMode::Implied, _) => Cycle(0),
            (Instruction::CMP, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::DEC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBS5, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::CPX, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndexedIndirect, _) => Cycle(0),
            (Instruction::CPX, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::INC, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SMB6, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::INX, AddrMode::Implied, _) => Cycle(0),
            (Instruction::SBC, AddrMode::Immediate, _) => Cycle(0),
            (Instruction::NOP, AddrMode::Implied, _) => Cycle(0),
            (Instruction::CPX, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::SBC, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::INC, AddrMode::Absolute, _) => Cycle(0),
            (Instruction::BBS6, AddrMode::ProgramCounterRelative, _) => Cycle(0),

            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 2)
                if self.st.is_set(StatusFlag::Zero) =>
            {
                self.load_memory_byte_lo(Self::addr_add(self.pc, 1))
            }
            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 3)
                if self.st.is_set(StatusFlag::Zero) =>
            {
                self.inc_pc(self.tmp[0])
            }
            (Instruction::BEQ, AddrMode::ProgramCounterRelative, 2) => self.inc_pc(2),

            (Instruction::SBC, AddrMode::ZeroPageIndirectIndexedWithY, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndirect, _) => Cycle(0),
            (Instruction::SBC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::INC, AddrMode::ZeroPageIndexedWithX, _) => Cycle(0),
            (Instruction::SMB7, AddrMode::ZeroPage, _) => Cycle(0),
            (Instruction::SED, AddrMode::Implied, _) => Cycle(0),
            (Instruction::SBC, AddrMode::AbsoluteIndexedWithY, _) => Cycle(0),

            (Instruction::PLX, AddrMode::Stack, 2) => self.stack_pull_byte_lo(),
            (Instruction::PLX, AddrMode::Stack, 3) => {
                self.x = self.tmp[0];
                if self.x == 0 {
                    self.set_flag(StatusFlag::Zero);
                } else if self.x >= 0x80 {
                    self.set_flag(StatusFlag::Negative);
                }
                self.inc_pc(2)
            }

            (Instruction::SBC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::INC, AddrMode::AbsoluteIndexedWithX, _) => Cycle(0),
            (Instruction::BBS7, AddrMode::ProgramCounterRelative, _) => Cycle(0),
            (Instruction::ILL, _, _) => panic!("Illegal opcode {opcode}"),
            (instruction, addr_mode, _) => panic!(
                "Shouldn't happen: {:?} with addressing mode {:?}",
                instruction, addr_mode
            ),
        }
    }
}
