use std::{error::Error, fmt::Display, num::ParseIntError, str::FromStr};

use crate::cpu6502::{cpu::{CPU, StatusFlag}, memory::Memory, model::instruction::Instruction};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddrMode {
    Absolute,
    AbsoluteIndexedIndirect,
    AbsoluteIndexedWithX,
    AbsoluteIndexedWithY,
    AbsoluteIndirect,
    Accumulator,
    Immediate,
    Implied,
    Relative,
    ZeroPage,
    ZeroPageIndexedIndirect,
    ZeroPageIndexedWithX,
    ZeroPageIndexedWithY,
    ZeroPageIndirect,
    ZeroPageIndirectIndexedWithY,
    ZeroPageRelative,
}

impl super::IsOriginal for AddrMode {
    fn is_original(&self) -> bool {
        !matches!(
            self,
            AddrMode::AbsoluteIndexedIndirect
                | AddrMode::ZeroPageIndirect
                | AddrMode::ZeroPageRelative
        )
    }
}

impl AddrMode {
    pub fn get_cycles(&self) -> u8 {
        match self {
            AddrMode::Absolute => 4,
            AddrMode::AbsoluteIndexedIndirect => 6,
            AddrMode::AbsoluteIndexedWithX => 4,
            AddrMode::AbsoluteIndexedWithY => 4,
            AddrMode::AbsoluteIndirect => 6,
            AddrMode::Accumulator => 2,
            AddrMode::Immediate => 2,
            AddrMode::Implied => 2,
            AddrMode::Relative => 2,
            AddrMode::ZeroPage => 3,
            AddrMode::ZeroPageIndexedIndirect => 6,
            AddrMode::ZeroPageIndexedWithX => 4,
            AddrMode::ZeroPageIndexedWithY => 4,
            AddrMode::ZeroPageIndirect => 5,
            AddrMode::ZeroPageIndirectIndexedWithY => 5,
            AddrMode::ZeroPageRelative => 3,
        }
    }
}

pub struct CpuBusTransfer<'a,'b,'c,'d> {
    cpu: &'a mut CPU<'c>,
    mem: &'b mut Memory<'d>,
}

impl <'a,'b,'c,'d> CpuBusTransfer<'a,'b,'c,'d> {
    pub fn new(cpu: &'a mut CPU<'c>, mem: &'b mut Memory<'d>) -> Self {
        CpuBusTransfer { cpu, mem }
    }
    pub fn run_load(&mut self, inst: Instruction, mode: AddrMode) {
        match (inst, mode) {
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::Absolute,
            ) => self.load_addr_arg(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::AbsoluteIndexedIndirect,
            ) => self.load_absolute_indexed_indirect_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::AbsoluteIndexedWithX,
            ) => self.load_absolute_indexed_with_x_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::AbsoluteIndexedWithY,
            ) => self.load_absolute_indexed_with_y_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::AbsoluteIndirect,
            ) => self.load_absolute_indirect_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPage,
            ) => self.load_zp_arg(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPageIndexedIndirect,
            ) => self.load_zp_indexed_indirect_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPageIndexedWithX,
            ) => self.load_zp_indexed_with_x_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPageIndexedWithY,
            ) => self.load_zp_indexed_with_y_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPageIndirect,
            ) => self.load_zp_indirect_addr(),
            (
                Instruction::STA | Instruction::STX | Instruction::STY | Instruction::STZ,
                AddrMode::ZeroPageIndirectIndexedWithY,
            ) => self.load_zp_indirect_indexed_with_y_addr(),
            (_, AddrMode::Absolute) => self.load_absolute_byte(),
            (_, AddrMode::AbsoluteIndexedIndirect) => self.load_absolute_indexed_indirect_byte(),
            (_, AddrMode::AbsoluteIndexedWithX) => self.load_absolute_indexed_with_x_byte(),
            (_, AddrMode::AbsoluteIndexedWithY) => self.load_absolute_indexed_with_y_byte(),
            (_, AddrMode::AbsoluteIndirect) => self.load_absolute_indirect_byte(),
            (_, AddrMode::Accumulator) => self.cpu.tmp[0] = self.cpu.a,
            (_, AddrMode::Immediate) => self.load_byte_arg(),
            (_, AddrMode::Implied) => {}
            (_, AddrMode::Relative) => {}
            (_, AddrMode::ZeroPage) => self.load_zp_byte(),
            (_, AddrMode::ZeroPageIndexedIndirect) => self.load_zp_indexed_indirect_byte(),
            (_, AddrMode::ZeroPageIndexedWithX) => self.load_zp_indexed_with_x_byte(),
            (_, AddrMode::ZeroPageIndexedWithY) => self.load_zp_indexed_with_y_byte(),
            (_, AddrMode::ZeroPageIndirect) => self.load_zp_indirect_byte(),
            (_, AddrMode::ZeroPageIndirectIndexedWithY) => {
                self.load_zp_indirect_indexed_with_y_byte()
            }
            (_, AddrMode::ZeroPageRelative) => self.load_zp_byte(),
        }
        self.cpu.cycles += mode.get_cycles();
    }
    pub fn run_instruction(&mut self, inst: Instruction) {
        match inst {
            Instruction::ADC => {
                // add the accumulator and the argument
                let a = self.cpu.a;
                let b = self.cpu.tmp[0];
                let c = a.wrapping_add(b);

                // An overflow occurs if and only if two numbers with the same sign are added,
                // but the result has the opposite sign:
                let ofl1 = (a ^ c) & (b ^ c) & 0x80 != 0;

                // add the carry
                let d: u8 = if self.cpu.is_set(StatusFlag::Carry) { 1 } else { 0 };
                self.cpu.a = c.wrapping_add(d);
                let ofl2 = (c ^ self.cpu.a) & (d ^ self.cpu.a) & 0x80 != 0;

                // we set the overflow flag iff one occurred during either addition
                self.cpu.set_or_clear_flag(StatusFlag::Overflow, ofl1 || ofl2);

                // we set the carry flag iff the result is smaller than the first operand
                self.cpu.set_or_clear_flag(StatusFlag::Carry, self.cpu.a < a);
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::AND => {
                self.cpu.a &= self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::ASL => {
                self.cpu.set_or_clear_flag(StatusFlag::Carry, self.cpu.tmp[0] & 0x80 != 0);
                self.cpu.tmp[0] <<= 1;
                self.cpu.check_and_set_nz_flags(self.cpu.tmp[0]);
            }
            Instruction::BBR0 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x01),
            Instruction::BBR1 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x02),
            Instruction::BBR2 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x04),
            Instruction::BBR3 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x08),
            Instruction::BBR4 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x10),
            Instruction::BBR5 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x20),
            Instruction::BBR6 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x40),
            Instruction::BBR7 => self.cpu.tmp[0] = !(self.cpu.tmp[0] & 0x80),
            Instruction::BBS0 => self.cpu.tmp[0] &= 0x01,
            Instruction::BBS1 => self.cpu.tmp[0] &= 0x02,
            Instruction::BBS2 => self.cpu.tmp[0] &= 0x04,
            Instruction::BBS3 => self.cpu.tmp[0] &= 0x08,
            Instruction::BBS4 => self.cpu.tmp[0] &= 0x10,
            Instruction::BBS5 => self.cpu.tmp[0] &= 0x20,
            Instruction::BBS6 => self.cpu.tmp[0] &= 0x40,
            Instruction::BBS7 => self.cpu.tmp[0] &= 0x80,
            Instruction::BCC => {
                self.cpu.tmp[0] = if self.cpu.is_clear(StatusFlag::Carry) {
                    1
                } else {
                    0
                }
            }
            Instruction::BCS => self.cpu.tmp[0] = if self.cpu.is_set(StatusFlag::Carry) { 1 } else { 0 },
            Instruction::BEQ => self.cpu.tmp[0] = if self.cpu.is_set(StatusFlag::Zero) { 1 } else { 0 },
            Instruction::BIT => {
                self.cpu.check_and_set_z_flag(self.cpu.a & self.cpu.tmp[0]);
                self.cpu.check_and_set_n_flag(self.cpu.tmp[0]);
                self.cpu.check_and_set_or_clear_flag(StatusFlag::Overflow, self.cpu.tmp[0] & 0x40);
            }
            Instruction::BMI => {
                self.cpu.tmp[0] = if self.cpu.is_set(StatusFlag::Negative) {
                    1
                } else {
                    0
                }
            }
            Instruction::BNE => {
                self.cpu.tmp[0] = if self.cpu.is_clear(StatusFlag::Zero) {
                    1
                } else {
                    0
                }
            }
            Instruction::BPL => {
                self.cpu.tmp[0] = if self.cpu.is_clear(StatusFlag::Negative) {
                    1
                } else {
                    0
                }
            }
            Instruction::BRA => self.cpu.tmp[0] = 1,
            Instruction::BRK => {
                if self.cpu.is_set(StatusFlag::BRK) {
                    self.cpu.inc_pc(2);
                } else {
                    self.stack_push_pc(2);
                    self.stack_push_flags();
                    self.cpu.change_flags(
                        &[StatusFlag::BRK, StatusFlag::IRQDisable],
                        &[StatusFlag::Decimal],
                    );
                    self.cpu.pc = self.mem.load_memory_word(0xfffe);
                    self.cpu.cycles = 7;
                }
            }
            Instruction::BVC => {
                self.cpu.tmp[0] = if self.cpu.is_clear(StatusFlag::Overflow) {
                    1
                } else {
                    0
                }
            }
            Instruction::BVS => {
                self.cpu.tmp[0] = if self.cpu.is_set(StatusFlag::Overflow) {
                    1
                } else {
                    0
                }
            }
            Instruction::CLC => self.cpu.clear_flag(StatusFlag::Carry),
            Instruction::CLD => self.cpu.clear_flag(StatusFlag::Decimal),
            Instruction::CLI => self.cpu.clear_flag(StatusFlag::IRQDisable),
            Instruction::CLV => self.cpu.clear_flag(StatusFlag::Overflow),
            Instruction::CMP => self.cpu.compare_and_set_flags(self.cpu.a, self.cpu.tmp[0]),
            Instruction::CPX => self.cpu.compare_and_set_flags(self.cpu.x, self.cpu.tmp[0]),
            Instruction::CPY => self.cpu.compare_and_set_flags(self.cpu.y, self.cpu.tmp[0]),
            Instruction::DEC => {
                self.cpu.tmp[0] = self.cpu.tmp[0].wrapping_sub(1);
                self.cpu.check_and_set_nz_flags(self.cpu.tmp[0]);
            }
            Instruction::DEX => {
                self.cpu.x = self.cpu.x.wrapping_sub(1);
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::DEY => {
                self.cpu.y = self.cpu.y.wrapping_sub(1);
                self.cpu.check_and_set_nz_flags(self.cpu.y);
            }
            Instruction::EOR => {
                self.cpu.a ^= self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::INC => {
                self.cpu.tmp[0] = self.cpu.tmp[0].wrapping_add(1);
                self.cpu.check_and_set_nz_flags(self.cpu.tmp[0]);
            }
            Instruction::INX => {
                self.cpu.x = self.cpu.x.wrapping_add(1);
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::INY => {
                self.cpu.y = self.cpu.y.wrapping_add(1);
                self.cpu.check_and_set_nz_flags(self.cpu.y);
            }
            Instruction::JMP => {
                self.cpu.set_pc();
            }
            Instruction::JSR => {
                self.stack_push_pc(3);
                self.load_addr_arg();
                self.cpu.cycles = 6;
                self.cpu.set_pc();
            }
            Instruction::LDA => {
                self.cpu.a = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::LDX => {
                self.cpu.x = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::LDY => {
                self.cpu.y = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.y);
            }
            Instruction::LSR => {
                self.cpu.check_and_set_or_clear_flag(StatusFlag::Carry, self.cpu.tmp[0] & 0x1);
                self.cpu.tmp[0] >>= 1;
            }
            Instruction::NOP => {}
            Instruction::ORA => {
                self.cpu.a |= self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::PHA => {
                self.cpu.tmp[0] = self.cpu.a;
                self.stack_push_byte();
            }
            Instruction::PHP => self.stack_push_flags(),
            Instruction::PHX => {
                self.cpu.tmp[0] = self.cpu.x;
                self.stack_push_byte();
            }
            Instruction::PHY => {
                self.cpu.tmp[0] = self.cpu.y;
                self.stack_push_byte();
            }
            Instruction::PLA => {
                self.stack_pull_byte();
                self.cpu.a = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::PLP => self.stack_pull_flags(),
            Instruction::PLX => {
                self.stack_pull_byte();
                self.cpu.x = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::PLY => {
                self.stack_pull_byte();
                self.cpu.y = self.cpu.tmp[0];
                self.cpu.check_and_set_nz_flags(self.cpu.y);
            }
            Instruction::RMB0 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x01),
            Instruction::RMB1 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x02),
            Instruction::RMB2 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x04),
            Instruction::RMB3 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x08),
            Instruction::RMB4 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x10),
            Instruction::RMB5 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x20),
            Instruction::RMB6 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x40),
            Instruction::RMB7 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x80),
            Instruction::ROL => {
                let old_carry = self.cpu.is_set(StatusFlag::Carry);
                self.cpu.check_and_set_or_clear_flag(StatusFlag::Carry, self.cpu.tmp[0] & 0x80);
                self.cpu.tmp[0] <<= 1;
                self.cpu.tmp[0] |= if old_carry { 1 } else { 0 };
            }
            Instruction::ROR => {
                let old_carry = self.cpu.is_set(StatusFlag::Carry);
                let b0 = self.cpu.tmp[0] & 1;
                self.cpu.check_and_set_or_clear_flag(StatusFlag::Carry, b0);
                self.cpu.tmp[0] >>= 1;
                if old_carry {
                    self.cpu.tmp[0] |= 0x80;
                }
                self.cpu.check_and_set_nz_flags(self.cpu.tmp[0]);
            }
            Instruction::RTI => {
                self.stack_pull_flags();
                self.stack_pull_addr();
                self.cpu.set_pc();
            }
            Instruction::RTS => {
                self.stack_pull_addr();
                self.cpu.set_pc();
            }
            Instruction::SBC => {
                // add the accumulator and the complement of the argument
                let a = self.cpu.a;
                let b = self.cpu.tmp[0];
                let c = a.wrapping_sub(b);
                let mut borrow = c > a;

                // Add the carry
                let d: u8 = if self.cpu.is_set(StatusFlag::Carry) { 0 } else { 1 };
                self.cpu.a = c.wrapping_sub(d);
                borrow |= self.cpu.a > c;

                // An overflow occurs if and only if two numbers with different sign are subtracted,
                // and the result has the opposite sign of the first number:
                let ofl = (a ^ b) & (a ^ self.cpu.a) & 0x80 != 0;
                self.cpu.set_or_clear_flag(StatusFlag::Overflow, ofl);

                // We clear the carry flag when a borrow occurs.
                //self.cpu.set_or_clear_flag(StatusFlag::Carry, self.cpu.a <= a);
                self.cpu.set_or_clear_flag(StatusFlag::Carry, !borrow);
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::SEC => self.cpu.set_flag(StatusFlag::Carry),
            Instruction::SED => self.cpu.set_flag(StatusFlag::Decimal),
            Instruction::SEI => self.cpu.set_flag(StatusFlag::IRQDisable),
            Instruction::SMB0 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x01),
            Instruction::SMB1 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x02),
            Instruction::SMB2 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x04),
            Instruction::SMB3 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x08),
            Instruction::SMB4 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x10),
            Instruction::SMB5 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x20),
            Instruction::SMB6 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x40),
            Instruction::SMB7 => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0] & !0x80),
            Instruction::STA => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.a),
            Instruction::STP => {}
            Instruction::STX => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.x),
            Instruction::STY => self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.y),
            Instruction::STZ => self.mem.store_memory_byte(self.cpu.tmp_addr, 0),
            Instruction::TAX => {
                self.cpu.x = self.cpu.a;
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::TAY => {
                self.cpu.y = self.cpu.a;
                self.cpu.check_and_set_nz_flags(self.cpu.y);
            }
            Instruction::TRB => {
                self.cpu.check_and_set_z_flag(self.cpu.a & self.cpu.tmp[0]);
                self.mem.store_memory_byte(self.cpu.tmp_addr, !self.cpu.a & self.cpu.tmp[0])
            }
            Instruction::TSB => {
                self.cpu.check_and_set_z_flag(self.cpu.a & self.cpu.tmp[0]);
                self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.a | self.cpu.tmp[0]);
            }
            Instruction::TSX => {
                self.cpu.x = self.cpu.sp;
                self.cpu.check_and_set_nz_flags(self.cpu.x);
            }
            Instruction::TXA => {
                self.cpu.a = self.cpu.x;
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::TXS => self.cpu.sp = self.cpu.x,
            Instruction::TYA => {
                self.cpu.a = self.cpu.y;
                self.cpu.check_and_set_nz_flags(self.cpu.a);
            }
            Instruction::WAI => {}
            Instruction::ILL => {}
        }
    }
    pub fn run_store(&mut self, inst: Instruction, mode: AddrMode) {
        match (inst, mode) {
            (
                Instruction::ASL
                | Instruction::DEC
                | Instruction::INC
                | Instruction::LSR
                | Instruction::ROL
                | Instruction::ROR,
                AddrMode::Absolute | AddrMode::AbsoluteIndexedWithX,
            ) => {
                self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0]);
                self.cpu.cycles += 2;
                self.cpu.inc_pc(3);
            }
            (
                Instruction::ASL
                | Instruction::DEC
                | Instruction::INC
                | Instruction::LSR
                | Instruction::ROL
                | Instruction::ROR,
                AddrMode::ZeroPage | AddrMode::ZeroPageIndexedWithX,
            ) => {
                self.mem.store_memory_byte(self.cpu.tmp_addr, self.cpu.tmp[0]);
                self.cpu.cycles += 2;
                self.cpu.inc_pc(2);
            }
            (
                Instruction::ASL
                | Instruction::DEC
                | Instruction::INC
                | Instruction::LSR
                | Instruction::ROL
                | Instruction::ROR,
                AddrMode::Accumulator,
            ) => {
                self.cpu.a = self.cpu.tmp[0];
                self.cpu.inc_pc(1);
            }
            (Instruction::JSR, _) => {}
            (Instruction::JMP, _) => {}
            (_, AddrMode::Absolute) => self.cpu.inc_pc(3),
            (_, AddrMode::AbsoluteIndexedIndirect) => self.cpu.inc_pc(3),
            (Instruction::STA, AddrMode::AbsoluteIndexedWithX) => {
                self.cpu.cycles += 1;
                self.cpu.inc_pc(3);
            }
            (_, AddrMode::AbsoluteIndexedWithX) => self.cpu.inc_pc(3),
            (_, AddrMode::AbsoluteIndexedWithY) => self.cpu.inc_pc(3),
            (_, AddrMode::AbsoluteIndirect) => self.cpu.inc_pc(3),
            (_, AddrMode::Accumulator) => self.cpu.inc_pc(1),
            (_, AddrMode::Immediate) => self.cpu.inc_pc(2),
            (Instruction::BRK, AddrMode::Implied) => {}
            (Instruction::RTI, AddrMode::Implied) => {}
            (Instruction::RTS, AddrMode::Implied) => {}
            (_, AddrMode::Implied) => self.cpu.inc_pc(1),
            (_, AddrMode::Relative) => {
                let take_branch = self.cpu.tmp[0] != 0;
                self.load_byte_arg();
                let old_pc = self.cpu.pc;
                self.cpu.inc_pc(2);
                if take_branch {
                    self.cpu.inc_pc(self.cpu.tmp[0]);
                    self.cpu.cycles += 1;
                }
                if old_pc & 0xff00 != self.cpu.pc & 0xff00 {
                    self.cpu.cycles += 1;
                }
            }
            (_, AddrMode::ZeroPage) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedIndirect) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedWithX) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedWithY) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageIndirect) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageIndirectIndexedWithY) => self.cpu.inc_pc(2),
            (_, AddrMode::ZeroPageRelative) => self.cpu.inc_pc(3),
        }
    }
    pub fn stack_push_byte(&mut self) {
        self._stack_push_byte(false);
    }
    pub fn stack_pull_byte(&mut self) {
        self._stack_pull_byte(false);
    }
    pub fn stack_pull_addr(&mut self) {
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        let stack_base: u16 = 0x100;
        let addr: u16 = self.cpu.sp.into();
        let addr = stack_base.wrapping_add(addr);
        self.cpu.tmp_addr = self.mem.load_memory_word(addr);
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
    }
    pub fn stack_push_addr(&mut self) {
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        let stack_base: u16 = 0x100;
        let addr: u16 = self.cpu.sp.into();
        let addr = stack_base.wrapping_add(addr);
        self.mem.store_memory_word(addr, self.cpu.tmp_addr);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
    }
    fn _stack_push_byte(&mut self, hi: bool) {
        let byte = self.cpu.tmp[if hi { 1 } else { 0 }];
        let stack_base: u16 = 0x100;
        let addr: u16 = self.cpu.sp.into();
        let addr = stack_base.wrapping_add(addr);
        self.mem.store_memory_byte(addr, byte);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1)
    }
    fn _stack_pull_byte(&mut self, hi: bool) {
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        let stack_base: u16 = 0x100;
        let addr: u16 = self.cpu.sp.into();
        let addr = stack_base.wrapping_add(addr);
        if hi {
            self.cpu.tmp[1] = self.mem.load_memory_byte(addr.wrapping_add(1));
        } else {
            self.cpu.tmp[0] = self.mem.load_memory_byte(addr);
        }
    }
    pub fn stack_push_pc(&mut self, increment: u8) {
        self.cpu.tmp_addr = self.cpu.pc.wrapping_add(increment.into());
        self.stack_push_addr();
    }
    pub fn stack_push_flags(&mut self) {
        self.cpu.tmp[0] = self.cpu.st.0;
        self.stack_push_byte();
    }
    pub fn stack_pull_flags(&mut self) {
        self.stack_pull_byte();
        self.cpu.st.0 = self.cpu.tmp[0];
    }
    fn _load_byte_arg_lo(&mut self) {
        let addr: u16 = self.cpu.pc.wrapping_add(1);
        self.cpu.tmp[0] = self.mem.load_memory_byte(addr);
    }
    fn _load_byte_arg_hi(&mut self) {
        let addr: u16 = self.cpu.pc.wrapping_add(2);
        self.cpu.tmp[1] = self.mem.load_memory_byte(addr);
    }
    /**
     * Load the byte-sized argument into tmp[0]
     * tmp[1] is set to 0
     */
    pub fn load_byte_arg(&mut self) {
        self.cpu.tmp[1] = 0;
        self._load_byte_arg_lo();
    }
    /**
     * Load the word-sized argument into tmp
     */
    pub fn load_word_arg(&mut self) {
        self._load_byte_arg_lo();
        self._load_byte_arg_hi();
    }
    /**
     * Load the address argument into tmp_addr
     */
    pub fn load_addr_arg(&mut self) {
        self.load_word_arg();
        self.cpu.tmp_addr = u16::from_le_bytes(self.cpu.tmp);
    }
    /**
     * Load the zeropage address argument into tmp_addr
     */
    pub fn load_zp_arg(&mut self) {
        self.load_byte_arg();
        self.cpu.tmp_addr = u16::from_le_bytes(self.cpu.tmp);
    }
    fn _load_absolute_lo(&mut self) {
        self.load_addr_arg();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    // depends on _load_absolute_lo having been called before
    fn _load_absolute_hi(&mut self) {
        self.cpu.tmp[1] = self.mem.load_memory_byte(self.cpu.tmp_addr.wrapping_add(1));
    }
    /**
     * Loads a byte from the address pointed at by the address argument
     */
    pub fn load_absolute_byte(&mut self) {
        self._load_absolute_lo()
    }
    /**
     * Loads an address from the address pointed at by the argument
     */
    pub fn load_absolute_addr(&mut self) {
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
    }
    pub fn load_absolute_indirect_addr(&mut self) {
        self.load_absolute_addr();
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
    }
    /**
     * Loads a byte from (abs)
     */
    pub fn load_absolute_indirect_byte(&mut self) {
        self.load_absolute_indirect_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_absolute_indexed_indirect_addr(&mut self) {
        self.load_addr_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.x.into());
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
    }
    /**
     * Loads a byte from (abs,x)
     */
    pub fn load_absolute_indexed_indirect_byte(&mut self) {
        self.load_absolute_indexed_indirect_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_absolute_indexed_with_x_addr(&mut self) {
        self.load_addr_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.x.into());
    }
    /**
     * Loads a byte from abs,x
     */
    pub fn load_absolute_indexed_with_x_byte(&mut self) {
        self.load_addr_arg();
        let old_addr = self.cpu.tmp_addr;
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.x.into());
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
        self.inc_cycles_if_page_boundary_crossed(old_addr);
    }
    pub fn load_absolute_indexed_with_y_addr(&mut self) {
        self.load_addr_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.y.into());
    }
    /**
     * Loads a byte from a,y
     */
    pub fn load_absolute_indexed_with_y_byte(&mut self) {
        self.load_addr_arg();
        let old_addr = self.cpu.tmp_addr;
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.y.into());
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
        self.inc_cycles_if_page_boundary_crossed(old_addr);
    }
    pub fn load_zp_indexed_indirect_addr(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.x.into());
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
    }
    /**
     * Loads a byte from the (zp,x) address in the argument
     */
    pub fn load_zp_indexed_indirect_byte(&mut self) {
        self.load_zp_indexed_indirect_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_zp_indirect_addr(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
    }
    /**
     * Load a byte from the (zp) address in the argument
     */
    pub fn load_zp_indirect_byte(&mut self) {
        self.load_zp_indirect_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_zp_indirect_indexed_with_y_addr(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.y.into());
    }
    /**
     * Load a byte from the (zp),y address in the argument
     */
    pub fn load_zp_indirect_indexed_with_y_byte(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.mem.load_memory_word(self.cpu.tmp_addr);
        let old_addr = self.cpu.tmp_addr;
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.y.into());
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
        self.inc_cycles_if_page_boundary_crossed(old_addr);
    }
    /**
     * Load a byte from the zp address in the argument
     */
    pub fn load_zp_byte(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_zp_indexed_with_x_addr(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.x.into());
    }
    /**
     * Load a byte from the zp,x address in the argument
     */
    pub fn load_zp_indexed_with_x_byte(&mut self) {
        self.load_zp_indexed_with_x_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    pub fn load_zp_indexed_with_y_addr(&mut self) {
        self.load_zp_arg();
        self.cpu.tmp_addr = self.cpu.tmp_addr.wrapping_add(self.cpu.y.into());
    }
    /**
     * Load a byte from the zp,y address in the argument
     */
    pub fn load_zp_indexed_with_y_byte(&mut self) {
        self.load_zp_indexed_with_y_addr();
        self.cpu.tmp[0] = self.mem.load_memory_byte(self.cpu.tmp_addr);
    }
    fn inc_cycles_if_page_boundary_crossed(&mut self, old_addr: u16) {
        if self.cpu.tmp_addr & 0xff00 != old_addr & 0xff00 {
            self.cpu.cycles += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseAddrModeError{ s: String }

impl Display for ParseAddrModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("ParseAddrModeError: Unable to parse {}", &self.s).as_str())
    }
}

impl Error for ParseAddrModeError {}

impl FromStr for AddrMode {
    type Err = ParseAddrModeError;

    /**
     * Convert a string of the form #$08 or $(ABCD),Y etc to the corresponding AddrMode
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rparen = s.find(')');
        let comma = s.find(',');
        let selector = (s.bytes().nth(0), s.bytes().nth(1), rparen, comma);
        let ret = match selector {
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 5 =>
              Ok(AddrMode::Absolute),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 7 =>
              Ok(AddrMode::AbsoluteIndexedIndirect),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 9 =>
              Ok(AddrMode::AbsoluteIndexedWithX),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 9 =>
              Ok(AddrMode::AbsoluteIndexedWithY),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 7 =>
              Ok(AddrMode::AbsoluteIndirect),
            (Some(b'#'), Some(b'$'), None,None) =>
              Ok(AddrMode::Immediate),
            _ if s.len() == 3 =>
              Ok(AddrMode::Implied),
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 3 =>
              Ok(AddrMode::ZeroPage),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndexedIndirect),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 7 =>
              Ok(AddrMode::ZeroPageIndexedWithX),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 7 =>
              Ok(AddrMode::ZeroPageIndexedWithY),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndirect),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co > rp && s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndirectIndexedWithY),
            _ => Err(ParseAddrModeError { s: s.to_owned() }),
        };
        ret
    }
}

impl From<ParseIntError> for ParseAddrModeError {
    fn from(value: ParseIntError) -> Self {
        ParseAddrModeError { s: format!("ParseIntError occurred: {}", value).to_string() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddrModeWithAddr {
    pub mode: AddrMode,
    pub arg: u16,
    pub arg_size: u8,
}

impl FromStr for AddrModeWithAddr {

    type Err = ParseAddrModeError;

    /**
     * Convert a string of the form #$08 or $(ABCD),Y etc to the corresponding (AddrMode, u16)
     * (the argument)
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rparen = s.find(')');
        let comma = s.find(',');
        let selector = (s.bytes().nth(0), s.bytes().nth(1), rparen, comma);
        let ret = match selector {
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 5 =>
              (AddrMode::Absolute, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 9 =>
              (AddrMode::AbsoluteIndexedIndirect, u16::from_str_radix(&s.to_owned()[2..6], 16)?, 2),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 7 =>
              (AddrMode::AbsoluteIndexedWithX, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 7 =>
              (AddrMode::AbsoluteIndexedWithY, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 7 =>
              (AddrMode::AbsoluteIndirect, u16::from_str_radix(&s.to_owned()[2..6], 16)?, 2),
            (Some(b'#'), Some(b'$'), None,None) =>
              (AddrMode::Immediate, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 3 =>
              (AddrMode::ZeroPage, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 7 =>
              (AddrMode::ZeroPageIndexedIndirect, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 5 =>
              (AddrMode::ZeroPageIndexedWithX, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 5 =>
              (AddrMode::ZeroPageIndexedWithY, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 5 =>
              (AddrMode::ZeroPageIndirect, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co > rp && s.len() == 7 =>
              (AddrMode::ZeroPageIndirectIndexedWithY, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            _ => Err(ParseAddrModeError { s: s.to_owned() })?,
        };
        Ok(AddrModeWithAddr{ mode: ret.0, arg: ret.1, arg_size: ret.2 })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_addr_mode_with_addr_from_str() {
        let res = AddrModeWithAddr::from_str("$AAAA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Absolute, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("($AAAA,X)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedIndirect, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("$AAAA,X");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedWithX, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("$AAAA,Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedWithY, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("($AAAA)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndirect, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("#$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Immediate, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("#$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Immediate, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPage, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA,X)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedIndirect, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA,X");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedWithX, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA,Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedWithY, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndirect, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA),Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndirectIndexedWithY, arg: 0xaa, arg_size: 1 }), res);
    }
}