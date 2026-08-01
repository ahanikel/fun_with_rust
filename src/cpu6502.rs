use std::collections::HashSet;

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    st: ProcessorStatus,
    pc: u16,
    sp: u8,
    cycle: u8,
    mem: [u8; 65536],
    irq: bool,      // true if the IRQB pin is set to low
    irq_prev: bool, // previous state of the IRQB pin to detect negative transition
    nmi: bool,      // true if the NMIB pin is set to low
    nmi_prev: bool, // previous state of the NMIB pin to detect negative transition
    reset: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StatusFlags {
    Carry,
    Zero,
    IRQDisable,
    Decimal,
    BRK,
    Overflow,
    Negative,
}

impl From<u8> for StatusFlags {
    fn from(value: u8) -> Self {
        let value: usize = value.into();
        [
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

impl Into<u8> for StatusFlags {
    fn into(self) -> u8 {
        match self {
            StatusFlags::Carry => 0,
            StatusFlags::Zero => 2,
            StatusFlags::IRQDisable => 4,
            StatusFlags::Decimal => 8,
            StatusFlags::BRK => 16,
            StatusFlags::Overflow => 64,
            StatusFlags::Negative => 128,
        }
    }
}

impl StatusFlags {
    fn from_u8(value: u8) -> HashSet<Self> {
        let mut ret = HashSet::new();
        if (value & 1) != 0 {
            ret.insert(StatusFlags::Carry);
        }
        if (value & 2) != 0 {
            ret.insert(StatusFlags::Zero);
        }
        if (value & 4) != 0 {
            ret.insert(StatusFlags::IRQDisable);
        }
        if (value & 8) != 0 {
            ret.insert(StatusFlags::Decimal);
        }
        if (value & 16) != 0 {
            ret.insert(StatusFlags::BRK);
        }
        if (value & 64) != 0 {
            ret.insert(StatusFlags::Overflow);
        }
        if (value & 128) != 0 {
            ret.insert(StatusFlags::Negative);
        }
        ret
    }
    fn to_u8(flags: HashSet<Self>) -> u8 {
        let mut ret: u8 = 0;
        for flag in flags {
            let flag: u8 = flag.into();
            ret = ret | flag;
        }
        ret
    }
}

struct ProcessorStatus {
    flags: u8,
}

impl ProcessorStatus {
    fn from_flags(flags: HashSet<StatusFlags>) -> ProcessorStatus {
        let mut ret: u8 = 0;
        for flag in flags {
            let flag: u8 = flag.into();
            ret = ret | flag
        }
        ProcessorStatus { flags: ret }
    }
    fn is_set(&self, flag: StatusFlags) -> bool {
        let flag: u8 = flag.into();
        self.flags & flag != 0
    }
    fn is_clear(&self, flag: StatusFlags) -> bool {
        let flag: u8 = flag.into();
        self.flags & flag == 0
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
        let st = ProcessorStatus { flags: 0 };
        CPU {
            a: 0,
            x: 0,
            y: 0,
            st,
            pc: 0,
            sp: 0xff,
            cycle: 0,
            mem,
            irq: false,
            irq_prev: false,
            nmi: false,
            nmi_prev: false,
            reset: false,
        }
    }
    pub fn load_memory_byte(&self, addr: u16) -> u8 {
        // TODO: insert code for peripherals
        let addr: usize = addr.into();
        self.mem[addr]
    }
    pub fn store_memory_byte(&mut self, addr: u16, byte: u8) {
        let addr: usize = addr.into();
        self.mem[addr] = byte;
    }
    pub fn store_memory_word(&mut self, addr: u16, word: u16) {
        dbg!(addr, word);
        let bytes = u16::to_le_bytes(word);
        self.store_memory_byte(addr, bytes[0]);
        self.store_memory_byte(addr + 1, bytes[1]);
    }
    pub fn load_memory_word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_memory_byte(addr), self.load_memory_byte(addr + 1)])
    }
    pub fn reset(&mut self) {
        self.pc = self.load_memory_word(0xfffc);
        self.st.flags = 0;
        self.irq = false;
        self.nmi = false;
        self.cycle = 6;
        self.reset = true;
    }
    fn stack_push_byte(&mut self, byte: u8) {
        let stack_base: u16 = 0x100;
        let addr: u16 = self.sp.into();
        let addr = stack_base + addr;
        self.store_memory_byte(addr, byte);
        if self.sp == 0 {
            self.sp = 0xff;
        } else {
            self.sp = self.sp - 1;
        }
    }
    fn stack_push_word(&mut self, word: u16) {
        let bytes = u16::to_le_bytes(word);
        self.stack_push_byte(bytes[1]);
        self.stack_push_byte(bytes[0]);
    }
    fn stack_pull_byte(&mut self) -> u8 {
        if self.sp == 0xff {
            self.sp = 0x00;
        } else {
            self.sp = self.sp + 1;
        }
        let stack_base: u16 = 0x100;
        let addr: u16 = self.sp.into();
        let addr = stack_base + addr;
        self.load_memory_byte(addr)
    }
    fn stack_pull_word(&mut self) -> u16 {
        let bytes = [self.stack_pull_byte(), self.stack_pull_byte()];
        u16::from_le_bytes(bytes)
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
    pub fn cycle(&self) -> u8 {
        self.cycle
    }
    pub fn flags(&self) -> u8 {
        self.st.flags
    }
    pub fn pc(&self) -> u16 {
        self.pc
    }
    pub fn step(&mut self) {
        let opcode = self.load_memory_byte(self.pc);
        println!("{}: 0b{:08b} 0x{:04x} 0x{:02x}", self.cycle, self.st.flags, self.pc, opcode);
        if self.reset {
            match self.cycle {
                0 => self.reset = false,
                _ => {
                    self.cycle = self.cycle - 1;
                    return;
                }
            }
        }
        match instruction_and_mode(opcode) {
            (Instruction::BRK, AddrMode::Stack) => {
                if self.st.is_set(StatusFlags::BRK) {
                    match self.cycle {
                        0 => {
                            self.cycle = 2;
                        }
                        1 => {
                            self.pc = self.pc + 2;
                            self.cycle = 0;
                        }
                        _ => self.cycle = self.cycle - 1,
                    }
                } else {
                    match self.cycle {
                        0 => {
                            self.stack_push_word(Self::addr_add(self.pc, 2));
                            self.cycle = 6;
                        }
                        4 => {
                            let push_flags = self.st.flags;
                            self.stack_push_byte(push_flags);
                            self.cycle = 3;
                        }
                        1 => {
                            let brk_flag: u8 = StatusFlags::BRK.into();
                            let i_flag: u8 = StatusFlags::IRQDisable.into();
                            let d_flag: u8 = StatusFlags::Decimal.into();
                            self.st.flags = self.st.flags | brk_flag | i_flag;
                            self.st.flags = self.st.flags & !d_flag;
                            let jump_to = self.load_memory_word(0xfffe);
                            self.pc = jump_to;
                            self.cycle = 0;
                        }
                        _ => self.cycle = self.cycle - 1,
                    }
                }
            }
            (Instruction::ORA, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::TSB, AddrMode::ZeroPage) => {}
            (Instruction::ORA, AddrMode::ZeroPage) => {}
            (Instruction::ASL, AddrMode::ZeroPage) => {}
            (Instruction::RMB0, AddrMode::ZeroPage) => {}
            (Instruction::PHP, AddrMode::Stack) => {}
            (Instruction::ORA, AddrMode::Immediate) => {}
            (Instruction::ASL, AddrMode::Accumulator) => {}
            (Instruction::TSB, AddrMode::Absolute) => {}
            (Instruction::ORA, AddrMode::Absolute) => {}
            (Instruction::ASL, AddrMode::Absolute) => {}
            (Instruction::BBR0, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BPL, AddrMode::ProgramCounterRelative) => {}
            (Instruction::ORA, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::ORA, AddrMode::ZeroPageIndirect) => {}
            (Instruction::TRB, AddrMode::ZeroPage) => {}
            (Instruction::ORA, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::ASL, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::RMB1, AddrMode::ZeroPage) => {}
            (Instruction::CLC, AddrMode::Implied) => {}
            (Instruction::ORA, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::INC, AddrMode::Accumulator) => {}
            (Instruction::TRB, AddrMode::Absolute) => {}
            (Instruction::ORA, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::ASL, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBR1, AddrMode::ProgramCounterRelative) => {}
            (Instruction::JSR, AddrMode::Absolute) => {}
            (Instruction::AND, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::BIT, AddrMode::ZeroPage) => {}
            (Instruction::AND, AddrMode::ZeroPage) => {}
            (Instruction::ROL, AddrMode::ZeroPage) => {}
            (Instruction::RMB2, AddrMode::ZeroPage) => {}
            (Instruction::PLP, AddrMode::Stack) => {}
            (Instruction::AND, AddrMode::Immediate) => {}
            (Instruction::ROL, AddrMode::Accumulator) => {}
            (Instruction::BIT, AddrMode::Absolute) => {}
            (Instruction::AND, AddrMode::Absolute) => {}
            (Instruction::ROL, AddrMode::Absolute) => {}
            (Instruction::BBR2, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BMI, AddrMode::ProgramCounterRelative) => {}
            (Instruction::AND, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::AND, AddrMode::ZeroPageIndirect) => {}
            (Instruction::BIT, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::AND, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::ROL, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::RMB3, AddrMode::ZeroPage) => {}
            (Instruction::SEC, AddrMode::Implied) => {}
            (Instruction::AND, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::DEC, AddrMode::Accumulator) => {}
            (Instruction::BIT, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::AND, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::ROL, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBR3, AddrMode::ProgramCounterRelative) => {}
            (Instruction::RTI, AddrMode::Stack) => {
                let flags = self.stack_pull_byte();
                let pc = self.stack_pull_word();
                self.st.flags = flags;
                self.pc = pc;
                self.cycle = 0;
            }
            (Instruction::EOR, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::EOR, AddrMode::ZeroPage) => {}
            (Instruction::LSR, AddrMode::ZeroPage) => {}
            (Instruction::RMB4, AddrMode::ZeroPage) => {}
            (Instruction::PHA, AddrMode::Stack) => {}
            (Instruction::EOR, AddrMode::Immediate) => {}
            (Instruction::LSR, AddrMode::Accumulator) => {}
            (Instruction::JMP, AddrMode::Absolute) => {}
            (Instruction::EOR, AddrMode::Absolute) => {}
            (Instruction::LSR, AddrMode::Absolute) => {}
            (Instruction::BBR4, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BVC, AddrMode::ProgramCounterRelative) => {}
            (Instruction::EOR, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::EOR, AddrMode::ZeroPageIndirect) => {}
            (Instruction::EOR, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::LSR, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::RMB5, AddrMode::ZeroPage) => {}
            (Instruction::CLI, AddrMode::Implied) => {}
            (Instruction::EOR, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::PHY, AddrMode::Stack) => {}
            (Instruction::EOR, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::LSR, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBR5, AddrMode::ProgramCounterRelative) => {}
            (Instruction::RTS, AddrMode::Stack) => {}
            (Instruction::ADC, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::STZ, AddrMode::ZeroPage) => {}
            (Instruction::ADC, AddrMode::ZeroPage) => {}
            (Instruction::ROR, AddrMode::ZeroPage) => {}
            (Instruction::RMB6, AddrMode::ZeroPage) => {}
            (Instruction::PLA, AddrMode::Stack) => {}
            (Instruction::ADC, AddrMode::Immediate) => {}
            (Instruction::ROR, AddrMode::Accumulator) => {}
            (Instruction::JMP, AddrMode::AbsoluteIndirect) => {}
            (Instruction::ADC, AddrMode::Absolute) => {}
            (Instruction::ROR, AddrMode::Absolute) => {}
            (Instruction::BBR6, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BVS, AddrMode::ProgramCounterRelative) => {}
            (Instruction::ADC, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::ADC, AddrMode::ZeroPageIndirect) => {}
            (Instruction::STZ, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::ADC, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::ROR, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::RMB7, AddrMode::ZeroPage) => {}
            (Instruction::SEI, AddrMode::Implied) => {}
            (Instruction::ADC, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::PLY, AddrMode::Stack) => {}
            (Instruction::JMP, AddrMode::AbsoluteIndexedIndirect) => {}
            (Instruction::ADC, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::ROR, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBR7, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BRA, AddrMode::ProgramCounterRelative) => {}
            (Instruction::STA, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::STY, AddrMode::ZeroPage) => {}
            (Instruction::STA, AddrMode::ZeroPage) => {}
            (Instruction::STX, AddrMode::ZeroPage) => {}
            (Instruction::SMB0, AddrMode::ZeroPage) => {}
            (Instruction::DEY, AddrMode::Implied) => {}
            (Instruction::BIT, AddrMode::Immediate) => {}
            (Instruction::TXA, AddrMode::Implied) => {}
            (Instruction::STY, AddrMode::Absolute) => {}
            (Instruction::STA, AddrMode::Absolute) => {}
            (Instruction::STX, AddrMode::Absolute) => {}
            (Instruction::BBS0, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BCC, AddrMode::ProgramCounterRelative) => {}
            (Instruction::STA, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::STA, AddrMode::ZeroPageIndirect) => {}
            (Instruction::STY, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::STA, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::STX, AddrMode::ZeroPageIndexedWithY) => {}
            (Instruction::SMB1, AddrMode::ZeroPage) => {}
            (Instruction::TYA, AddrMode::Implied) => {}
            (Instruction::STA, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::TXS, AddrMode::Implied) => {}
            (Instruction::STZ, AddrMode::Absolute) => {}
            (Instruction::STA, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::STZ, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBS1, AddrMode::ProgramCounterRelative) => {}
            (Instruction::LDY, AddrMode::Immediate) => {}
            (Instruction::LDA, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::LDX, AddrMode::Immediate) => {}
            (Instruction::LDY, AddrMode::ZeroPage) => {}
            (Instruction::LDA, AddrMode::ZeroPage) => {}
            (Instruction::LDX, AddrMode::ZeroPage) => {}
            (Instruction::SMB2, AddrMode::ZeroPage) => {}
            (Instruction::TAY, AddrMode::Implied) => {}
            (Instruction::LDA, AddrMode::Immediate) => {}
            (Instruction::TAX, AddrMode::Implied) => {}
            (Instruction::LDY, AddrMode::Accumulator) => {}
            (Instruction::LDA, AddrMode::Absolute) => {}
            (Instruction::LDX, AddrMode::Absolute) => {}
            (Instruction::BBS2, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BCS, AddrMode::ProgramCounterRelative) => {}
            (Instruction::LDA, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::LDA, AddrMode::ZeroPageIndirect) => {}
            (Instruction::LDY, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::LDA, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::LDX, AddrMode::ZeroPageIndexedWithY) => {}
            (Instruction::SMB3, AddrMode::ZeroPage) => {}
            (Instruction::CLV, AddrMode::Implied) => {}
            (Instruction::LDA, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::TSX, AddrMode::Implied) => {}
            (Instruction::LDY, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::LDA, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::LDX, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::BBS3, AddrMode::ProgramCounterRelative) => {}
            (Instruction::CPY, AddrMode::Immediate) => {}
            (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::CPY, AddrMode::ZeroPage) => {}
            (Instruction::CMP, AddrMode::ZeroPage) => {}
            (Instruction::DEC, AddrMode::ZeroPage) => {}
            (Instruction::SMB4, AddrMode::ZeroPage) => {}
            (Instruction::INY, AddrMode::Implied) => {}
            (Instruction::CMP, AddrMode::Immediate) => {}
            (Instruction::DEX, AddrMode::Implied) => {}
            (Instruction::WAI, AddrMode::Implied) => {}
            (Instruction::CPY, AddrMode::Absolute) => {}
            (Instruction::CMP, AddrMode::Absolute) => {}
            (Instruction::DEC, AddrMode::Absolute) => {}
            (Instruction::BBS4, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BNE, AddrMode::ProgramCounterRelative) => {}
            (Instruction::CMP, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::CMP, AddrMode::ZeroPageIndirect) => {}
            (Instruction::CMP, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::DEC, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::SMB5, AddrMode::ZeroPage) => {}
            (Instruction::CLD, AddrMode::Implied) => {}
            (Instruction::CMP, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::PHX, AddrMode::Stack) => {}
            (Instruction::STP, AddrMode::Implied) => {}
            (Instruction::CMP, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::DEC, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBS5, AddrMode::ProgramCounterRelative) => {}
            (Instruction::CPX, AddrMode::Immediate) => {}
            (Instruction::SBC, AddrMode::ZeroPageIndexedIndirect) => {}
            (Instruction::CPX, AddrMode::ZeroPage) => {}
            (Instruction::SBC, AddrMode::ZeroPage) => {}
            (Instruction::INC, AddrMode::ZeroPage) => {}
            (Instruction::SMB6, AddrMode::ZeroPage) => {}
            (Instruction::INX, AddrMode::Implied) => {}
            (Instruction::SBC, AddrMode::Immediate) => {}
            (Instruction::NOP, AddrMode::Implied) => {}
            (Instruction::CPX, AddrMode::Absolute) => {}
            (Instruction::SBC, AddrMode::Absolute) => {}
            (Instruction::INC, AddrMode::Absolute) => {}
            (Instruction::BBS6, AddrMode::ProgramCounterRelative) => {}
            (Instruction::BEQ, AddrMode::ProgramCounterRelative) => {}
            (Instruction::SBC, AddrMode::ZeroPageIndirectIndexedWithY) => {}
            (Instruction::SBC, AddrMode::ZeroPageIndirect) => {}
            (Instruction::SBC, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::INC, AddrMode::ZeroPageIndexedWithX) => {}
            (Instruction::SMB7, AddrMode::ZeroPage) => {}
            (Instruction::SED, AddrMode::Implied) => {}
            (Instruction::SBC, AddrMode::AbsoluteIndexedWithY) => {}
            (Instruction::PLX, AddrMode::Stack) => {}
            (Instruction::SBC, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::INC, AddrMode::AbsoluteIndexedWithX) => {}
            (Instruction::BBS7, AddrMode::ProgramCounterRelative) => {}
            (Instruction::ILL, _) => panic!("Illegal opcode {opcode}"),
            (instruction, addr_mode) => panic!(
                "Shouldn't happen: {:?} with addressing mode {:?}",
                instruction, addr_mode
            ),
        }
    }
}

mod test {
    use crate::cpu6502::CPU;
    use std::io::Read;

    #[test]
    fn test_brk() {}
}
