pub mod acia;
pub mod cpu;
pub mod device;
pub mod model;
mod test;

use cpu::{CPU, StatusFlag};
use model::{addr_mode::AddrMode, instruction::Instruction, instruction_and_mode};

impl CPU<'_> {
    fn compare_and_set_flags(&mut self, reg: u8, byte: u8) {
        match reg.cmp(&byte) {
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
    fn check_and_set_z_flag(&mut self, byte: u8) {
        if byte == 0 {
            self.set_flag(StatusFlag::Zero);
        } else {
            self.clear_flag(StatusFlag::Zero);
        }
    }
    fn check_and_set_n_flag(&mut self, byte: u8) {
        if byte & 0x80 == 0 {
            self.clear_flag(StatusFlag::Negative);
        } else {
            self.set_flag(StatusFlag::Negative);
        }
    }
    fn check_and_set_nz_flags(&mut self, byte: u8) {
        self.check_and_set_z_flag(byte);
        self.check_and_set_n_flag(byte);
    }
    fn set_or_clear_flag(&mut self, flag: StatusFlag, val: bool) {
        if val {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }
    fn check_and_set_or_clear_flag(&mut self, flag: StatusFlag, val: u8) {
        self.set_or_clear_flag(flag, val != 0);
    }

    pub fn step(&mut self) {
        if self.reset {
            match self.cycle {
                7 => {
                    self.reset = false;
                    self.cycle = 0;
                }
                _ => {
                    self.cycle += 1;
                    return;
                }
            }
        }
        if self.cycle == 0 && self.irq && self.is_clear(StatusFlag::IRQDisable) {
            self.stack_push_pc(2);
            self.stack_push_flags();
            self.change_flags(&[StatusFlag::IRQDisable], &[StatusFlag::Decimal]);
            self.load_memory_addr(0xfffe);
            self.cycles = 7;
            self.pc = self.tmp_addr;
            self.irq = false;
            return;
        }
        let opcode = self.mem[usize::from(self.pc)];
        if self.cycle == 0 {
            let pc: usize = self.pc.into();
            self.status_line = format!(
                "0b{:08b} a:{:02X} x:{:02X} y:{:02X} 0x{:04X} {}",
                self.st.0,
                self.a,
                self.x,
                self.y,
                self.pc,
                model::disasm(self.pc, &self.mem[pc..pc + 3])
            );
            if let Some(logger) = &mut self.log_instructions {
                logger(&self.status_line);
            }
            let (inst, mode) = instruction_and_mode(opcode);
            self.cycles = 0;
            self.run_load(inst, mode);
            self.run_instruction(inst);
            self.run_store(inst, mode);
            self.cycle += 1;
        } else if self.cycle == self.cycles - 1 {
            self.cycle = 0;
        } else {
            self.cycle += 1;
        }
    }

    fn run_load(&mut self, inst: Instruction, mode: AddrMode) {
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
            (_, AddrMode::Accumulator) => self.tmp[0] = self.a,
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
        self.cycles += mode.get_cycles();
    }

    fn run_instruction(&mut self, inst: Instruction) {
        match inst {
            Instruction::ADC => {
                // add the accumulator and the argument
                let a = self.a;
                let b = self.tmp[0];
                let c = a.wrapping_add(b);

                // An overflow occurs if and only if two numbers with the same sign are added,
                // but the result has the opposite sign:
                let ofl1 = (a ^ c) & (b ^ c) & 0x80 != 0;

                // add the carry
                let d: u8 = if self.is_set(StatusFlag::Carry) { 1 } else { 0 };
                self.a = c.wrapping_add(d);
                let ofl2 = (c ^ self.a) & (d ^ self.a) & 0x80 != 0;

                // we set the overflow flag iff one occurred during either addition
                self.set_or_clear_flag(StatusFlag::Overflow, ofl1 || ofl2);

                // we set the carry flag iff the result is smaller than the first operand
                self.set_or_clear_flag(StatusFlag::Carry, self.a < a);
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::AND => {
                self.a &= self.tmp[0];
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::ASL => {
                self.set_or_clear_flag(StatusFlag::Carry, self.tmp[0] & 0x80 != 0);
                self.tmp[0] <<= 1;
                self.check_and_set_nz_flags(self.tmp[0]);
            }
            Instruction::BBR0 => self.tmp[0] = !(self.tmp[0] & 0x01),
            Instruction::BBR1 => self.tmp[0] = !(self.tmp[0] & 0x02),
            Instruction::BBR2 => self.tmp[0] = !(self.tmp[0] & 0x04),
            Instruction::BBR3 => self.tmp[0] = !(self.tmp[0] & 0x08),
            Instruction::BBR4 => self.tmp[0] = !(self.tmp[0] & 0x10),
            Instruction::BBR5 => self.tmp[0] = !(self.tmp[0] & 0x20),
            Instruction::BBR6 => self.tmp[0] = !(self.tmp[0] & 0x40),
            Instruction::BBR7 => self.tmp[0] = !(self.tmp[0] & 0x80),
            Instruction::BBS0 => self.tmp[0] &= 0x01,
            Instruction::BBS1 => self.tmp[0] &= 0x02,
            Instruction::BBS2 => self.tmp[0] &= 0x04,
            Instruction::BBS3 => self.tmp[0] &= 0x08,
            Instruction::BBS4 => self.tmp[0] &= 0x10,
            Instruction::BBS5 => self.tmp[0] &= 0x20,
            Instruction::BBS6 => self.tmp[0] &= 0x40,
            Instruction::BBS7 => self.tmp[0] &= 0x80,
            Instruction::BCC => {
                self.tmp[0] = if self.is_clear(StatusFlag::Carry) {
                    1
                } else {
                    0
                }
            }
            Instruction::BCS => self.tmp[0] = if self.is_set(StatusFlag::Carry) { 1 } else { 0 },
            Instruction::BEQ => self.tmp[0] = if self.is_set(StatusFlag::Zero) { 1 } else { 0 },
            Instruction::BIT => {
                let test = self.a & self.tmp[0];
                self.check_and_set_nz_flags(test);
                self.check_and_set_or_clear_flag(StatusFlag::Overflow, test & 0x40);
            }
            Instruction::BMI => {
                self.tmp[0] = if self.is_set(StatusFlag::Negative) {
                    1
                } else {
                    0
                }
            }
            Instruction::BNE => {
                self.tmp[0] = if self.is_clear(StatusFlag::Zero) {
                    1
                } else {
                    0
                }
            }
            Instruction::BPL => {
                self.tmp[0] = if self.is_clear(StatusFlag::Negative) {
                    1
                } else {
                    0
                }
            }
            Instruction::BRA => self.tmp[0] = 1,
            Instruction::BRK => {
                if self.is_set(StatusFlag::BRK) {
                    self.inc_pc(2);
                } else {
                    self.stack_push_pc(2);
                    self.stack_push_flags();
                    self.change_flags(
                        &[StatusFlag::BRK, StatusFlag::IRQDisable],
                        &[StatusFlag::Decimal],
                    );
                    self.load_memory_addr(0xfffe);
                    self.cycles = 7;
                    self.pc = self.tmp_addr;
                }
            }
            Instruction::BVC => {
                self.tmp[0] = if self.is_clear(StatusFlag::Overflow) {
                    1
                } else {
                    0
                }
            }
            Instruction::BVS => {
                self.tmp[0] = if self.is_set(StatusFlag::Overflow) {
                    1
                } else {
                    0
                }
            }
            Instruction::CLC => self.clear_flag(StatusFlag::Carry),
            Instruction::CLD => self.clear_flag(StatusFlag::Decimal),
            Instruction::CLI => self.clear_flag(StatusFlag::IRQDisable),
            Instruction::CLV => self.clear_flag(StatusFlag::Overflow),
            Instruction::CMP => self.compare_and_set_flags(self.a, self.tmp[0]),
            Instruction::CPX => self.compare_and_set_flags(self.x, self.tmp[0]),
            Instruction::CPY => self.compare_and_set_flags(self.y, self.tmp[0]),
            Instruction::DEC => {
                self.tmp[0] = self.tmp[0].wrapping_sub(1);
                self.check_and_set_nz_flags(self.tmp[0]);
            }
            Instruction::DEX => {
                self.x = self.x.wrapping_sub(1);
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::DEY => {
                self.y = self.y.wrapping_sub(1);
                self.check_and_set_nz_flags(self.y);
            }
            Instruction::EOR => {
                self.a ^= self.tmp[0];
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::INC => {
                self.tmp[0] = self.tmp[0].wrapping_add(1);
                self.check_and_set_nz_flags(self.tmp[0]);
            }
            Instruction::INX => {
                self.x = self.x.wrapping_add(1);
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::INY => {
                self.y = self.y.wrapping_add(1);
                self.check_and_set_nz_flags(self.y);
            }
            Instruction::JMP => {
                self.set_pc();
            }
            Instruction::JSR => {
                self.stack_push_pc(3);
                self.load_addr_arg();
                self.cycles = 6;
                self.set_pc();
            }
            Instruction::LDA => {
                self.a = self.tmp[0];
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::LDX => {
                self.x = self.tmp[0];
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::LDY => {
                self.y = self.tmp[0];
                self.check_and_set_nz_flags(self.y);
            }
            Instruction::LSR => {
                self.check_and_set_or_clear_flag(StatusFlag::Carry, self.tmp[0] & 0x1);
                self.tmp[0] >>= 1;
            }
            Instruction::NOP => {}
            Instruction::ORA => {
                self.a |= self.tmp[0];
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::PHA => {
                self.tmp[0] = self.a;
                self.stack_push_byte();
            }
            Instruction::PHP => self.stack_push_flags(),
            Instruction::PHX => {
                self.tmp[0] = self.x;
                self.stack_push_byte();
            }
            Instruction::PHY => {
                self.tmp[0] = self.y;
                self.stack_push_byte();
            }
            Instruction::PLA => {
                self.stack_pull_byte();
                self.a = self.tmp[0];
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::PLP => self.stack_pull_flags(),
            Instruction::PLX => {
                self.stack_pull_byte();
                self.x = self.tmp[0];
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::PLY => {
                self.stack_pull_byte();
                self.y = self.tmp[0];
                self.check_and_set_nz_flags(self.y);
            }
            Instruction::RMB0 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x01),
            Instruction::RMB1 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x02),
            Instruction::RMB2 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x04),
            Instruction::RMB3 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x08),
            Instruction::RMB4 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x10),
            Instruction::RMB5 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x20),
            Instruction::RMB6 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x40),
            Instruction::RMB7 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x80),
            Instruction::ROL => {
                let old_carry = self.is_set(StatusFlag::Carry);
                self.check_and_set_or_clear_flag(StatusFlag::Carry, self.tmp[0] & 0x80);
                self.tmp[0] <<= 1;
                self.tmp[0] |= if old_carry { 1 } else { 0 };
            }
            Instruction::ROR => {
                let old_carry = self.is_set(StatusFlag::Carry);
                let b0 = self.tmp[0] & 1;
                self.check_and_set_or_clear_flag(StatusFlag::Carry, b0);
                self.tmp[0] >>= 1;
                if old_carry {
                    self.tmp[0] |= 0x80;
                }
                self.check_and_set_nz_flags(self.tmp[0]);
            }
            Instruction::RTI => {
                self.stack_pull_flags();
                self.stack_pull_addr();
                self.set_pc();
            }
            Instruction::RTS => {
                self.stack_pull_addr();
                self.set_pc();
            }
            Instruction::SBC => {
                // add the accumulator and the complement of the argument
                let a = self.a;
                let b = self.tmp[0];
                let c = a.wrapping_sub(b);
                let mut borrow = c > a;

                // Add the carry
                let d: u8 = if self.is_set(StatusFlag::Carry) { 0 } else { 1 };
                self.a = c.wrapping_sub(d);
                borrow |= self.a > c;

                // An overflow occurs if and only if two numbers with different sign are subtracted,
                // and the result has the opposite sign of the first number:
                let ofl = (a ^ b) & (a ^ self.a) & 0x80 != 0;
                self.set_or_clear_flag(StatusFlag::Overflow, ofl);

                // We clear the carry flag when a borrow occurs.
                //self.set_or_clear_flag(StatusFlag::Carry, self.a <= a);
                self.set_or_clear_flag(StatusFlag::Carry, !borrow);
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::SEC => self.set_flag(StatusFlag::Carry),
            Instruction::SED => self.set_flag(StatusFlag::Decimal),
            Instruction::SEI => self.set_flag(StatusFlag::IRQDisable),
            Instruction::SMB0 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x01),
            Instruction::SMB1 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x02),
            Instruction::SMB2 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x04),
            Instruction::SMB3 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x08),
            Instruction::SMB4 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x10),
            Instruction::SMB5 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x20),
            Instruction::SMB6 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x40),
            Instruction::SMB7 => self.store_memory_byte(self.tmp_addr, self.tmp[0] & !0x80),
            Instruction::STA => self.store_memory_byte(self.tmp_addr, self.a),
            Instruction::STP => {}
            Instruction::STX => self.store_memory_byte(self.tmp_addr, self.x),
            Instruction::STY => self.store_memory_byte(self.tmp_addr, self.y),
            Instruction::STZ => self.store_memory_byte(self.tmp_addr, 0),
            Instruction::TAX => {
                self.x = self.a;
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::TAY => {
                self.y = self.a;
                self.check_and_set_nz_flags(self.y);
            }
            Instruction::TRB => {
                self.check_and_set_z_flag(self.a & self.tmp[0]);
                self.store_memory_byte(self.tmp_addr, !self.a & self.tmp[0])
            }
            Instruction::TSB => {
                self.check_and_set_z_flag(self.a & self.tmp[0]);
                self.store_memory_byte(self.tmp_addr, self.a | self.tmp[0]);
            }
            Instruction::TSX => {
                self.x = self.sp;
                self.check_and_set_nz_flags(self.x);
            }
            Instruction::TXA => {
                self.a = self.x;
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::TXS => self.sp = self.x,
            Instruction::TYA => {
                self.a = self.y;
                self.check_and_set_nz_flags(self.a);
            }
            Instruction::WAI => {}
            Instruction::ILL => {}
        }
    }

    fn run_store(&mut self, inst: Instruction, mode: AddrMode) {
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
                self.store_memory_byte(self.tmp_addr, self.tmp[0]);
                self.cycles += 2;
                self.inc_pc(3);
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
                self.store_memory_byte(self.tmp_addr, self.tmp[0]);
                self.cycles += 2;
                self.inc_pc(2);
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
                self.a = self.tmp[0];
                self.inc_pc(1);
            }
            (Instruction::JSR, _) => {}
            (Instruction::JMP, _) => {}
            (_, AddrMode::Absolute) => self.inc_pc(3),
            (_, AddrMode::AbsoluteIndexedIndirect) => self.inc_pc(3),
            (Instruction::STA, AddrMode::AbsoluteIndexedWithX) => {
                self.cycles += 1;
                self.inc_pc(3);
            }
            (_, AddrMode::AbsoluteIndexedWithX) => self.inc_pc(3),
            (_, AddrMode::AbsoluteIndexedWithY) => self.inc_pc(3),
            (_, AddrMode::AbsoluteIndirect) => self.inc_pc(3),
            (_, AddrMode::Accumulator) => self.inc_pc(1),
            (_, AddrMode::Immediate) => self.inc_pc(2),
            (Instruction::BRK, AddrMode::Implied) => {}
            (Instruction::RTI, AddrMode::Implied) => {}
            (Instruction::RTS, AddrMode::Implied) => {}
            (_, AddrMode::Implied) => self.inc_pc(1),
            (_, AddrMode::Relative) => {
                let take_branch = self.tmp[0] != 0;
                self.load_byte_arg();
                let old_pc = self.pc;
                self.inc_pc(2);
                if take_branch {
                    self.inc_pc(self.tmp[0]);
                    self.cycles += 1;
                }
                if old_pc & 0xff00 != self.pc & 0xff00 {
                    self.cycles += 1;
                }
            }
            (_, AddrMode::ZeroPage) => self.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedIndirect) => self.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedWithX) => self.inc_pc(2),
            (_, AddrMode::ZeroPageIndexedWithY) => self.inc_pc(2),
            (_, AddrMode::ZeroPageIndirect) => self.inc_pc(2),
            (_, AddrMode::ZeroPageIndirectIndexedWithY) => self.inc_pc(2),
            (_, AddrMode::ZeroPageRelative) => self.inc_pc(3),
        }
    }
}
