pub mod addr_mode;
pub mod instruction;

use addr_mode::AddrMode;
use instruction::Instruction;

pub trait IsOriginal {
    fn is_original(&self) -> bool;
}

pub fn instruction_and_mode(opcode: u8) -> (Instruction, AddrMode) {
    let opcode: usize = opcode.into();
    [
        (Instruction::BRK, AddrMode::Implied),
        (Instruction::ORA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 2 0x02
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 3 0x03
        (Instruction::TSB, AddrMode::ZeroPage),
        (Instruction::ORA, AddrMode::ZeroPage),
        (Instruction::ASL, AddrMode::ZeroPage),
        (Instruction::RMB0, AddrMode::ZeroPage),
        (Instruction::PHP, AddrMode::Implied),
        (Instruction::ORA, AddrMode::Immediate),
        (Instruction::ASL, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 11 0x0b
        (Instruction::TSB, AddrMode::Absolute),
        (Instruction::ORA, AddrMode::Absolute),
        (Instruction::ASL, AddrMode::Absolute),
        (Instruction::BBR0, AddrMode::ZeroPageRelative),
        (Instruction::BPL, AddrMode::Relative),
        (Instruction::ORA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::ORA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 19 0x13
        (Instruction::TRB, AddrMode::ZeroPage),
        (Instruction::ORA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ASL, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB1, AddrMode::ZeroPage),
        (Instruction::CLC, AddrMode::Implied),
        (Instruction::ORA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::INC, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 27 0x1b
        (Instruction::TRB, AddrMode::Absolute),
        (Instruction::ORA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ASL, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR1, AddrMode::ZeroPageRelative),
        (Instruction::JSR, AddrMode::Absolute),
        (Instruction::AND, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 34 0x22
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 35 0x23
        (Instruction::BIT, AddrMode::ZeroPage),
        (Instruction::AND, AddrMode::ZeroPage),
        (Instruction::ROL, AddrMode::ZeroPage),
        (Instruction::RMB2, AddrMode::ZeroPage),
        (Instruction::PLP, AddrMode::Implied),
        (Instruction::AND, AddrMode::Immediate),
        (Instruction::ROL, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 43 0x2b
        (Instruction::BIT, AddrMode::Absolute),
        (Instruction::AND, AddrMode::Absolute),
        (Instruction::ROL, AddrMode::Absolute),
        (Instruction::BBR2, AddrMode::ZeroPageRelative),
        (Instruction::BMI, AddrMode::Relative),
        (Instruction::AND, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::AND, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 51 0x33
        (Instruction::BIT, AddrMode::ZeroPageIndexedWithX),
        (Instruction::AND, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ROL, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB3, AddrMode::ZeroPage),
        (Instruction::SEC, AddrMode::Implied),
        (Instruction::AND, AddrMode::AbsoluteIndexedWithY),
        (Instruction::DEC, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 59 0x3b
        (Instruction::BIT, AddrMode::AbsoluteIndexedWithX),
        (Instruction::AND, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ROL, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR3, AddrMode::ZeroPageRelative),
        (Instruction::RTI, AddrMode::Implied),
        (Instruction::EOR, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 66 0x42
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 67 0x43
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 68 0x44
        (Instruction::EOR, AddrMode::ZeroPage),
        (Instruction::LSR, AddrMode::ZeroPage),
        (Instruction::RMB4, AddrMode::ZeroPage),
        (Instruction::PHA, AddrMode::Implied),
        (Instruction::EOR, AddrMode::Immediate),
        (Instruction::LSR, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 75 0x4b
        (Instruction::JMP, AddrMode::Absolute),
        (Instruction::EOR, AddrMode::Absolute),
        (Instruction::LSR, AddrMode::Absolute),
        (Instruction::BBR4, AddrMode::ZeroPageRelative),
        (Instruction::BVC, AddrMode::Relative),
        (Instruction::EOR, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::EOR, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 83 0x53
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 84 0x54
        (Instruction::EOR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LSR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB5, AddrMode::ZeroPage),
        (Instruction::CLI, AddrMode::Implied),
        (Instruction::EOR, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PHY, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 91 0x5b
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 92 0x5c
        (Instruction::EOR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LSR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR5, AddrMode::ZeroPageRelative),
        (Instruction::RTS, AddrMode::Implied),
        (Instruction::ADC, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 98 0x62
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 99 0x63
        (Instruction::STZ, AddrMode::ZeroPage),
        (Instruction::ADC, AddrMode::ZeroPage),
        (Instruction::ROR, AddrMode::ZeroPage),
        (Instruction::RMB6, AddrMode::ZeroPage),
        (Instruction::PLA, AddrMode::Implied),
        (Instruction::ADC, AddrMode::Immediate),
        (Instruction::ROR, AddrMode::Accumulator),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 107 0x6b
        (Instruction::JMP, AddrMode::AbsoluteIndirect),
        (Instruction::ADC, AddrMode::Absolute),
        (Instruction::ROR, AddrMode::Absolute),
        (Instruction::BBR6, AddrMode::ZeroPageRelative),
        (Instruction::BVS, AddrMode::Relative),
        (Instruction::ADC, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::ADC, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 115 0x73
        (Instruction::STZ, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ADC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::ROR, AddrMode::ZeroPageIndexedWithX),
        (Instruction::RMB7, AddrMode::ZeroPage),
        (Instruction::SEI, AddrMode::Implied),
        (Instruction::ADC, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PLY, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 123 0x7b
        (Instruction::JMP, AddrMode::AbsoluteIndexedIndirect),
        (Instruction::ADC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::ROR, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBR7, AddrMode::ZeroPageRelative),
        (Instruction::BRA, AddrMode::Relative),
        (Instruction::STA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 130 0x82
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 131 0x83
        (Instruction::STY, AddrMode::ZeroPage),
        (Instruction::STA, AddrMode::ZeroPage),
        (Instruction::STX, AddrMode::ZeroPage),
        (Instruction::SMB0, AddrMode::ZeroPage),
        (Instruction::DEY, AddrMode::Implied),
        (Instruction::BIT, AddrMode::Immediate),
        (Instruction::TXA, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 139 0x8b
        (Instruction::STY, AddrMode::Absolute),
        (Instruction::STA, AddrMode::Absolute),
        (Instruction::STX, AddrMode::Absolute),
        (Instruction::BBS0, AddrMode::ZeroPageRelative),
        (Instruction::BCC, AddrMode::Relative),
        (Instruction::STA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::STA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 147 0x93
        (Instruction::STY, AddrMode::ZeroPageIndexedWithX),
        (Instruction::STA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::STX, AddrMode::ZeroPageIndexedWithY),
        (Instruction::SMB1, AddrMode::ZeroPage),
        (Instruction::TYA, AddrMode::Implied),
        (Instruction::STA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::TXS, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 155 0x9b
        (Instruction::STZ, AddrMode::Absolute),
        (Instruction::STA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::STZ, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS1, AddrMode::ZeroPageRelative),
        (Instruction::LDY, AddrMode::Immediate),
        (Instruction::LDA, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::LDX, AddrMode::Immediate),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 163 0xa3
        (Instruction::LDY, AddrMode::ZeroPage),
        (Instruction::LDA, AddrMode::ZeroPage),
        (Instruction::LDX, AddrMode::ZeroPage),
        (Instruction::SMB2, AddrMode::ZeroPage),
        (Instruction::TAY, AddrMode::Implied),
        (Instruction::LDA, AddrMode::Immediate),
        (Instruction::TAX, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 171 0xab
        (Instruction::LDY, AddrMode::Absolute),
        (Instruction::LDA, AddrMode::Absolute),
        (Instruction::LDX, AddrMode::Absolute),
        (Instruction::BBS2, AddrMode::ZeroPageRelative),
        (Instruction::BCS, AddrMode::Relative),
        (Instruction::LDA, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::LDA, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 179 0xb3
        (Instruction::LDY, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LDA, AddrMode::ZeroPageIndexedWithX),
        (Instruction::LDX, AddrMode::ZeroPageIndexedWithY),
        (Instruction::SMB3, AddrMode::ZeroPage),
        (Instruction::CLV, AddrMode::Implied),
        (Instruction::LDA, AddrMode::AbsoluteIndexedWithY),
        (Instruction::TSX, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 187 0xbb
        (Instruction::LDY, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LDA, AddrMode::AbsoluteIndexedWithX),
        (Instruction::LDX, AddrMode::AbsoluteIndexedWithY),
        (Instruction::BBS3, AddrMode::ZeroPageRelative),
        (Instruction::CPY, AddrMode::Immediate),
        (Instruction::CMP, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 194 0xc2
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 195 0xc3
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
        (Instruction::BBS4, AddrMode::ZeroPageRelative),
        (Instruction::BNE, AddrMode::Relative),
        (Instruction::CMP, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::CMP, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 211 0xd3
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 212 0xd4
        (Instruction::CMP, AddrMode::ZeroPageIndexedWithX),
        (Instruction::DEC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::SMB5, AddrMode::ZeroPage),
        (Instruction::CLD, AddrMode::Implied),
        (Instruction::CMP, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PHX, AddrMode::Implied),
        (Instruction::STP, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 220 0xdc
        (Instruction::CMP, AddrMode::AbsoluteIndexedWithX),
        (Instruction::DEC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS5, AddrMode::ZeroPageRelative),
        (Instruction::CPX, AddrMode::Immediate),
        (Instruction::SBC, AddrMode::ZeroPageIndexedIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 226 0xe2
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 227 0xe3
        (Instruction::CPX, AddrMode::ZeroPage),
        (Instruction::SBC, AddrMode::ZeroPage),
        (Instruction::INC, AddrMode::ZeroPage),
        (Instruction::SMB6, AddrMode::ZeroPage),
        (Instruction::INX, AddrMode::Implied),
        (Instruction::SBC, AddrMode::Immediate),
        (Instruction::NOP, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 235 0xeb
        (Instruction::CPX, AddrMode::Absolute),
        (Instruction::SBC, AddrMode::Absolute),
        (Instruction::INC, AddrMode::Absolute),
        (Instruction::BBS6, AddrMode::ZeroPageRelative),
        (Instruction::BEQ, AddrMode::Relative),
        (Instruction::SBC, AddrMode::ZeroPageIndirectIndexedWithY),
        (Instruction::SBC, AddrMode::ZeroPageIndirect),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 243 0xf3
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 244 0xf4
        (Instruction::SBC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::INC, AddrMode::ZeroPageIndexedWithX),
        (Instruction::SMB7, AddrMode::ZeroPage),
        (Instruction::SED, AddrMode::Implied),
        (Instruction::SBC, AddrMode::AbsoluteIndexedWithY),
        (Instruction::PLX, AddrMode::Implied),
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 251 0xfb
        (Instruction::ILL, AddrMode::Implied), // Illegal Opcode 252 0xfc
        (Instruction::SBC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::INC, AddrMode::AbsoluteIndexedWithX),
        (Instruction::BBS7, AddrMode::ZeroPageRelative),
    ][opcode]
}

pub fn disasm(pc: u16, mem: &[u8]) -> String {
    match instruction_and_mode(mem[0]) {
        (inst, AddrMode::Absolute) => {
            format!("{} ${:04X}", inst, u16::from_le_bytes([mem[1], mem[2]]))
        }
        (inst, AddrMode::AbsoluteIndexedIndirect) => {
            format!("{} (${:04X},X)", inst, u16::from_le_bytes([mem[1], mem[2]]))
        }
        (inst, AddrMode::AbsoluteIndexedWithX) => {
            format!("{} ${:04X},X", inst, u16::from_le_bytes([mem[1], mem[2]]))
        }
        (inst, AddrMode::AbsoluteIndexedWithY) => {
            format!("{} ${:04X},Y", inst, u16::from_le_bytes([mem[1], mem[2]]))
        }
        (inst, AddrMode::AbsoluteIndirect) => {
            format!("{} (${:04X})", inst, u16::from_le_bytes([mem[1], mem[2]]))
        }
        (inst, AddrMode::Accumulator) => format!("{} #${:02X}", inst, mem[1]),
        (inst, AddrMode::Immediate) => format!("{} #${:02X}", inst, mem[1]),
        (inst, AddrMode::Implied) => format!("{}", inst),
        (inst, AddrMode::Relative) => format!(
            "{} ${:04X}",
            inst,
            pc.wrapping_add(2)
                .wrapping_add_signed(mem[1].cast_signed().into())
        ),
        (inst, AddrMode::ZeroPage) => format!("{} ${:02X}", inst, mem[1]),
        (inst, AddrMode::ZeroPageIndexedIndirect) => format!("{} (${:02X},X)", inst, mem[1]),
        (inst, AddrMode::ZeroPageIndexedWithX) => format!("{} ${:02X},X", inst, mem[1]),
        (inst, AddrMode::ZeroPageIndexedWithY) => format!("{} ${:02X},Y", inst, mem[1]),
        (inst, AddrMode::ZeroPageIndirect) => format!("{} (${:02X})", inst, mem[1]),
        (inst, AddrMode::ZeroPageIndirectIndexedWithY) => format!("{} (${}),Y", inst, mem[1]),
        (inst, AddrMode::ZeroPageRelative) => format!("{} #${:02X} ${:02X}", inst, mem[1], mem[2]),
    }
}
